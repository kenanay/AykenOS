#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_low_half_kheap_scaffold.sh \
    --evidence-dir evidence/run-<id>/gates/low-half-kheap-scaffold \
    [--mode allow|forbid] \
    [--runtime-pid <pid>] \
    [--phase-profile full|terminal_lineage] \
    [--require-terminal-slice] \
    [--phase10a2-evidence evidence/run-<id>/gates/ring3-execution-phase10a2]

Modes:
  allow   Phase10 visibility mode. Scaffold may remain, but it must be explicit,
          bounded, documented as removable before Phase11 closure, and proven
          by same-run runtime evidence.
  forbid  Phase11 closure mode. Any active low-half kernel-heap scaffold fails.

Exit codes:
  0: pass
  2: scaffold policy violation
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
MODE="allow"
A2_EVIDENCE_DIR=""
RUNTIME_PID=""
REQUIRE_TERMINAL_SLICE="0"
PHASE_PROFILE="full"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --mode)
      MODE="$2"
      shift 2
      ;;
    --runtime-pid)
      RUNTIME_PID="$2"
      shift 2
      ;;
    --phase-profile)
      PHASE_PROFILE="$2"
      shift 2
      ;;
    --require-terminal-slice)
      REQUIRE_TERMINAL_SLICE="1"
      shift 1
      ;;
    --phase10a2-evidence)
      A2_EVIDENCE_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 3
      ;;
  esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi

if [[ "${MODE}" != "allow" && "${MODE}" != "forbid" ]]; then
  echo "ERROR: --mode must be allow or forbid" >&2
  exit 3
fi

if [[ -n "${RUNTIME_PID}" ]] && ! [[ "${RUNTIME_PID}" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --runtime-pid must be a decimal pid" >&2
  exit 3
fi

if [[ "${PHASE_PROFILE}" != "full" && "${PHASE_PROFILE}" != "terminal_lineage" ]]; then
  echo "ERROR: --phase-profile must be full or terminal_lineage" >&2
  exit 3
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

if [[ -z "${A2_EVIDENCE_DIR}" ]]; then
  CANDIDATE_A2_DIR="$(cd "$(dirname "${EVIDENCE_DIR}")" && pwd)/ring3-execution-phase10a2"
  if [[ -d "${CANDIDATE_A2_DIR}" ]]; then
    A2_EVIDENCE_DIR="${CANDIDATE_A2_DIR}"
  fi
fi

if [[ -z "${A2_EVIDENCE_DIR}" ]]; then
  echo "ERROR: missing Phase10-A2 evidence; pass --phase10a2-evidence or use a same-run sibling gate dir" >&2
  exit 3
fi

REPORT_JSON="${EVIDENCE_DIR}/report.json"
RUNTIME_PROOF_JSON="${EVIDENCE_DIR}/runtime_proof.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
ROOT="${ROOT}" \
MODE="${MODE}" \
A2_EVIDENCE_DIR="${A2_EVIDENCE_DIR}" \
RUNTIME_PID="${RUNTIME_PID}" \
REQUIRE_TERMINAL_SLICE="${REQUIRE_TERMINAL_SLICE}" \
PHASE_PROFILE="${PHASE_PROFILE}" \
REPORT_JSON="${REPORT_JSON}" \
RUNTIME_PROOF_JSON="${RUNTIME_PROOF_JSON}" \
VIOLATIONS_TXT="${VIOLATIONS_TXT}" \
python3 - <<'PY'
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path
from typing import Any

root = Path(os.environ["ROOT"])
mode = os.environ["MODE"]
a2_evidence_dir = Path(os.environ["A2_EVIDENCE_DIR"])
runtime_pid_arg = os.environ.get("RUNTIME_PID", "").strip()
require_terminal_slice = os.environ.get("REQUIRE_TERMINAL_SLICE", "0") == "1"
phase_profile = os.environ.get("PHASE_PROFILE", "full").strip()
report_path = Path(os.environ["REPORT_JSON"])
runtime_proof_path = Path(os.environ["RUNTIME_PROOF_JSON"])
violations_path = Path(os.environ["VIOLATIONS_TXT"])

HIGHER_HALF_MIN = 0xFFFF800000000000
RUNTIME_MARKER = "[[AYKEN_LOW_HALF_KHEAP_RUNTIME]]"
RUNTIME_REQUIRED_FIELDS = [
    "phase",
    "seq",
    "pid",
    "pml4",
    "kheap_start",
    "kernel_virt_base",
    "pte",
    "present",
    "user",
    "writable",
    "nx",
    "kheap_low_half",
    "kernel_higher_half",
    "scaffold",
]
RUNTIME_FULL_PHASES = ["create", "syscall_entry", "timer_irq"]
if phase_profile == "terminal_lineage":
    RUNTIME_REQUIRED_PHASES = ["create"]
else:
    RUNTIME_REQUIRED_PHASES = list(RUNTIME_FULL_PHASES)
RUNTIME_OPTIONAL_PHASES = ["exit_teardown_pre", "exit_teardown_post"]
RUNTIME_KNOWN_PHASES = RUNTIME_FULL_PHASES + RUNTIME_OPTIONAL_PHASES
RUNTIME_OPTIONAL_INT_FIELDS = [
    "lower_half_roots",
    "lower_half_leaves",
    "lower_half_user_leaves",
]

code_files = {
    "mm_h": root / "kernel/include/mm.h",
    "kernel_limits_h": root / "shared/abi/kernel_limits.h",
    "paging_c": root / "kernel/mm/paging.c",
    "user_as_c": root / "kernel/mm/user_as.c",
}

doc_files = {
    "runtime_reality": root / "docs/development/SYSCALL_RUNTIME_REALITY.md",
    "phase10_requirements": root / "docs/specs/phase10b-execution-path-hardening/requirements.md",
    "phase10_design": root / "docs/specs/phase10b-execution-path-hardening/design.md",
    "phase10_tasks": root / "docs/specs/phase10b-execution-path-hardening/tasks.md",
    "phase11_requirements": root / "docs/specs/phase11-verification-substrate/requirements.md",
    "phase11_tasks": root / "docs/specs/phase11-verification-substrate/tasks.md",
}

texts = {
    name: path.read_text(encoding="utf-8")
    for name, path in {**code_files, **doc_files}.items()
}
lower_texts = {name: text.lower() for name, text in texts.items()}

macro_token = "AYKEN_LOW_HALF_KHEAP_SCAFFOLD_ACTIVE"
kheap_start_token = "AYKEN_KHEAP_START"
kernel_virt_base_token = "KERNEL_VIRT_BASE"
helper_token = "paging_seed_user_kernel_heap_window"

phase11_deadline_phrase = "must be removed before phase 11 closure"
phase11_gate_name = "ci-gate-no-low-half-kernel-dependency"
phase10_visibility_gate_name = "ci-gate-low-half-kheap-scaffold"
contradictory_truth_phrases = [
    "no longer mirrors the current low-half kernel heap",
    "no longer mirrors low-half kernel heap",
    "removed low-half kernel heap mirror",
    "removed the low-half kernel heap mirror",
    "scaffold removed",
    "scaffold inactive",
]


def contains_all(name: str, fragments: list[str]) -> bool:
    haystack = lower_texts[name]
    return all(fragment in haystack for fragment in fragments)


def parse_define_int(text: str, token: str) -> int | None:
    match = re.search(
        rf"(?m)^\s*#define\s+{re.escape(token)}\s+\(?([0-9A-Fa-fxX]+)(?:ULL|UL|LL|L|U)?\)?\s*$",
        text,
    )
    if not match:
        return None
    return int(match.group(1), 0)


def find_contradictions(name: str) -> list[str]:
    haystack = lower_texts[name]
    return [phrase for phrase in contradictory_truth_phrases if phrase in haystack]


def parse_scalar(value: str) -> int:
    match = re.match(r"^(0[xX][0-9A-Fa-f]+|[0-9]+)", value)
    if not match:
        raise ValueError(f"invalid scalar:{value}")
    return int(match.group(1), 0)


def parse_runtime_record(line: str) -> dict[str, Any]:
    marker_idx = line.find(RUNTIME_MARKER)
    if marker_idx < 0:
        raise ValueError("missing runtime marker prefix")
    body = line[marker_idx + len(RUNTIME_MARKER) :].strip()
    fields: dict[str, Any] = {"raw_line": line[marker_idx:].strip()}
    for token in body.split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        fields[key] = value
    missing = [key for key in RUNTIME_REQUIRED_FIELDS if key not in fields]
    if missing:
        raise ValueError("missing runtime fields:" + ",".join(missing))
    phase = str(fields["phase"])
    if phase not in RUNTIME_KNOWN_PHASES:
        raise ValueError(f"unknown runtime phase:{phase}")
    parsed = {
        "phase": phase,
        "seq": parse_scalar(str(fields["seq"])),
        "pid": parse_scalar(str(fields["pid"])),
        "pml4": parse_scalar(str(fields["pml4"])),
        "kheap_start": parse_scalar(str(fields["kheap_start"])),
        "kernel_virt_base": parse_scalar(str(fields["kernel_virt_base"])),
        "pte": parse_scalar(str(fields["pte"])),
        "present": parse_scalar(str(fields["present"])),
        "user": parse_scalar(str(fields["user"])),
        "writable": parse_scalar(str(fields["writable"])),
        "nx": parse_scalar(str(fields["nx"])),
        "kheap_low_half": parse_scalar(str(fields["kheap_low_half"])),
        "kernel_higher_half": parse_scalar(str(fields["kernel_higher_half"])),
        "scaffold": parse_scalar(str(fields["scaffold"])),
        "raw_line": fields["raw_line"],
    }
    for key in RUNTIME_OPTIONAL_INT_FIELDS:
        parsed[key] = parse_scalar(str(fields[key])) if key in fields else None
    parsed["user_accessible"] = bool(parsed["user"])
    parsed["present_bool"] = bool(parsed["present"])
    return parsed


def format_runtime_record(record: dict[str, Any]) -> dict[str, Any]:
    formatted = {
        **record,
        "pml4": f"0x{record['pml4']:016x}",
        "kheap_start": f"0x{record['kheap_start']:016x}",
        "kernel_virt_base": f"0x{record['kernel_virt_base']:016x}",
        "pte": f"0x{record['pte']:016x}",
    }
    for key in RUNTIME_OPTIONAL_INT_FIELDS:
        formatted[key] = record.get(key)
    return formatted


def choose_runtime_log_path(a2_dir: Path) -> Path | None:
    candidates = [
        a2_dir / "marker.log",
        a2_dir / "combined.log",
        a2_dir / "boot-audit" / "qemu_debugcon.log",
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    return None


runtime_truth_ok = contains_all(
    "runtime_reality",
    [
        "user cr3 roots now explicitly mirror the current low-half kernel heap",
        "supervisor-only compatibility window",
        "temporary",
        "promoted out of the low half",
    ],
)
design_truth_ok = contains_all(
    "phase10_design",
    [
        "user cr3 roots now also mirror the current low-half kernel heap",
        "supervisor-only compatibility window",
        "temporary",
        "scaffolding",
        "target memory model",
    ],
)
requirements_truth_ok = contains_all(
    "phase10_requirements",
    [
        "mirror the low-half kernel heap",
        "temporary",
        "compatibility scaffold",
        "must remain bounded",
        "must be removed once kmalloc/proc metadata is promoted out of the low",
        "half",
    ],
)
tasks_truth_ok = phase10_visibility_gate_name in lower_texts["phase10_tasks"]
phase11_requirements_ok = (
    phase11_gate_name in lower_texts["phase11_requirements"]
    and phase11_deadline_phrase in lower_texts["phase11_requirements"]
)
phase11_tasks_ok = (
    phase11_gate_name in lower_texts["phase11_tasks"]
    and "low-half heap scaffold is removed" in lower_texts["phase11_tasks"]
)

macro_present = macro_token in texts["mm_h"]
macro_value = parse_define_int(texts["mm_h"], macro_token)
macro_enabled = macro_value not in (None, 0)
helper_declared = f"int {helper_token}(uint64_t pml4_phys);" in texts["mm_h"]
helper_implemented = f"int {helper_token}(uint64_t pml4_phys)" in texts["paging_c"]
helper_used_in_paging_create = (
    f"if ({helper_token}(new_pml4_phys) != 0)" in texts["paging_c"]
)
helper_used_in_user_as = (
    f"if ({helper_token}(new_pml4_phys) != 0)" in texts["user_as_c"]
)

kheap_start = parse_define_int(texts["kernel_limits_h"], kheap_start_token)
if kheap_start is None:
    kheap_start = parse_define_int(texts["mm_h"], kheap_start_token)
kernel_virt_base = parse_define_int(texts["kernel_limits_h"], kernel_virt_base_token)
kheap_is_low_half = kheap_start is not None and kheap_start < HIGHER_HALF_MIN
kheap_is_higher_half = kheap_start is not None and kheap_start >= HIGHER_HALF_MIN
kernel_virt_base_is_higher_half = (
    kernel_virt_base is not None and kernel_virt_base >= HIGHER_HALF_MIN
)

scaffold_present = any(
    [
        macro_enabled,
        helper_declared,
        helper_implemented,
        helper_used_in_paging_create,
        helper_used_in_user_as,
    ]
)

contradictory_truth_hits = {
    name: find_contradictions(name)
    for name in ("runtime_reality", "phase10_requirements", "phase10_design")
}
has_contradictory_truth = any(contradictory_truth_hits.values())

debt_state = "DEBT_PRESENT" if scaffold_present else "DEBT_REMOVED"
allowed = not scaffold_present or mode == "allow"

runtime_log_path = choose_runtime_log_path(a2_evidence_dir)
runtime_records: list[dict[str, Any]] = []
runtime_parse_errors: list[str] = []
if runtime_log_path is not None:
    for line_no, raw in enumerate(
        runtime_log_path.read_text(encoding="utf-8", errors="replace").splitlines(),
        start=1,
    ):
        if RUNTIME_MARKER not in raw:
            continue
        try:
            parsed = parse_runtime_record(raw.strip())
            parsed["line"] = line_no
            runtime_records.append(parsed)
        except ValueError as exc:
            runtime_parse_errors.append(f"runtime_marker_parse_error:line{line_no}:{exc}")

runtime_records.sort(key=lambda record: (record["seq"], record["line"]))
runtime_all_pids = sorted({record["pid"] for record in runtime_records})
runtime_all_terminal_pids = sorted(
    {
        record["pid"]
        for record in runtime_records
        if record["phase"] in {"exit_teardown_pre", "exit_teardown_post"}
    }
)
runtime_selected_pid: int | None = None
runtime_pid_selection = "single_pid_only"
if runtime_pid_arg:
    runtime_selected_pid = int(runtime_pid_arg, 10)
    runtime_pid_selection = "explicit_pid"
elif require_terminal_slice:
    if len(runtime_all_terminal_pids) == 1:
        runtime_selected_pid = runtime_all_terminal_pids[0]
        runtime_pid_selection = "terminal_slice_pid"
else:
    if len(runtime_all_pids) == 1:
        runtime_selected_pid = runtime_all_pids[0]

runtime_violations: list[str] = []
if runtime_log_path is None:
    runtime_violations.append("missing_phase10a2_runtime_log")
if not a2_evidence_dir.is_dir():
    runtime_violations.append("missing_phase10a2_evidence_dir")
if not runtime_records:
    runtime_violations.append("missing_runtime_proof_marker")
runtime_violations.extend(runtime_parse_errors)

if runtime_selected_pid is None and runtime_records:
    if runtime_pid_arg:
        runtime_violations.append(f"runtime_pid_not_found:{runtime_pid_arg}")
    elif require_terminal_slice:
        runtime_violations.append("runtime_terminal_slice_pid_ambiguous")
    else:
        runtime_violations.append("runtime_pid_ambiguous")

if runtime_selected_pid is not None:
    runtime_records = [record for record in runtime_records if record["pid"] == runtime_selected_pid]

runtime_latest = runtime_records[-1] if runtime_records else None
runtime_records_by_phase: dict[str, list[dict[str, Any]]] = {
    phase: [] for phase in RUNTIME_KNOWN_PHASES
}
for record in runtime_records:
    runtime_records_by_phase.setdefault(record["phase"], []).append(record)

for phase in RUNTIME_REQUIRED_PHASES:
    if not runtime_records_by_phase.get(phase):
        runtime_violations.append(f"missing_runtime_phase:{phase}")
for phase in RUNTIME_KNOWN_PHASES:
    if len(runtime_records_by_phase.get(phase, [])) > 1:
        runtime_violations.append(f"duplicate_runtime_phase:{phase}")

runtime_selected_by_phase = {
    phase: records[0]
    for phase, records in runtime_records_by_phase.items()
    if records
}

if require_terminal_slice:
    if "exit_teardown_pre" not in runtime_selected_by_phase:
        runtime_violations.append("missing_runtime_phase:exit_teardown_pre")
    if "exit_teardown_post" not in runtime_selected_by_phase:
        runtime_violations.append("missing_runtime_phase:exit_teardown_post")
elif "exit_teardown_pre" in runtime_selected_by_phase or "exit_teardown_post" in runtime_selected_by_phase:
    if "exit_teardown_pre" not in runtime_selected_by_phase:
        runtime_violations.append("missing_runtime_phase:exit_teardown_pre")
    if "exit_teardown_post" not in runtime_selected_by_phase:
        runtime_violations.append("missing_runtime_phase:exit_teardown_post")

runtime_seq_values = [record["seq"] for record in runtime_records]
if runtime_seq_values and any(
    later <= earlier for earlier, later in zip(runtime_seq_values, runtime_seq_values[1:])
):
    runtime_violations.append("runtime_sequence_not_strictly_increasing")

runtime_pid_values = {record["pid"] for record in runtime_records}
if len(runtime_pid_values) > 1:
    runtime_violations.append("runtime_pid_inconsistent")

runtime_pml4_values = {record["pml4"] for record in runtime_records}
if len(runtime_pml4_values) > 1:
    runtime_violations.append("runtime_pml4_inconsistent")

runtime_static_invariant_keys = [
    "kheap_start",
    "kernel_virt_base",
    "kheap_low_half",
    "kernel_higher_half",
    "scaffold",
]
for key in runtime_static_invariant_keys:
    values = {record[key] for record in runtime_records}
    if len(values) > 1:
        runtime_violations.append(f"runtime_proof_inconsistent:{key}")

if runtime_selected_by_phase:
    create_record = runtime_selected_by_phase.get("create")
    if create_record is not None:
        for later_phase in ("syscall_entry", "timer_irq", "exit_teardown_pre", "exit_teardown_post"):
            later_record = runtime_selected_by_phase.get(later_phase)
            if later_record is not None and later_record["seq"] <= create_record["seq"]:
                runtime_violations.append(f"runtime_phase_order_invalid:create->{later_phase}")
    exit_pre = runtime_selected_by_phase.get("exit_teardown_pre")
    exit_post = runtime_selected_by_phase.get("exit_teardown_post")
    if exit_pre is not None and exit_post is not None and exit_post["seq"] <= exit_pre["seq"]:
        runtime_violations.append("runtime_phase_order_invalid:exit_teardown_pre->exit_teardown_post")

ordered_kheap_half = [record["kheap_low_half"] for record in runtime_records]
if any(later > earlier for earlier, later in zip(ordered_kheap_half, ordered_kheap_half[1:])):
    runtime_violations.append("runtime_kheap_half_reverted_to_low_half")

for record in runtime_records:
    phase = record["phase"]
    if kheap_start is not None and record["kheap_start"] != kheap_start:
        runtime_violations.append(f"runtime_kheap_start_mismatch:{phase}")
    if kernel_virt_base is not None and record["kernel_virt_base"] != kernel_virt_base:
        runtime_violations.append(f"runtime_kernel_virt_base_mismatch:{phase}")
    if record["kernel_higher_half"] != 1:
        runtime_violations.append(f"runtime_kernel_not_higher_half:{phase}")
    if record["user"] != 0:
        runtime_violations.append(f"runtime_kheap_mapping_user_accessible:{phase}")
    if record["scaffold"] != int(scaffold_present):
        runtime_violations.append(f"runtime_scaffold_state_mismatch:{phase}")
    if record["kheap_low_half"] != int(bool(kheap_is_low_half)):
        runtime_violations.append(f"runtime_kheap_half_mismatch:{phase}")

    if phase in {"create", "syscall_entry", "timer_irq", "exit_teardown_pre"}:
        if record["present"] != 1:
            runtime_violations.append(f"runtime_kheap_mapping_not_present:{phase}")
        if record["writable"] != 1:
            runtime_violations.append(f"runtime_kheap_mapping_not_writable:{phase}")
    elif phase == "exit_teardown_post":
        for key in RUNTIME_OPTIONAL_INT_FIELDS:
            if record.get(key) is None:
                runtime_violations.append(f"missing_runtime_global_count:{phase}:{key}")
        if record["kheap_low_half"] == 1:
            if record["present"] != 0:
                runtime_violations.append("runtime_exit_post_low_half_mapping_still_present")
            if record["writable"] != 0:
                runtime_violations.append("runtime_exit_post_low_half_mapping_still_writable")
            if record.get("lower_half_roots") != 0:
                runtime_violations.append("runtime_exit_post_lower_half_root_entries_remaining")
            if record.get("lower_half_leaves") != 0:
                runtime_violations.append("runtime_exit_post_lower_half_leaf_mappings_remaining")
            if record.get("lower_half_user_leaves") != 0:
                runtime_violations.append("runtime_exit_post_lower_half_user_leaf_mappings_remaining")
        else:
            if record["present"] != 1:
                runtime_violations.append("runtime_exit_post_higher_half_mapping_not_present")
            if record["writable"] != 1:
                runtime_violations.append("runtime_exit_post_higher_half_mapping_not_writable")

if scaffold_present and runtime_latest is not None and runtime_latest["kheap_low_half"] != 1:
    runtime_violations.append("runtime_scaffold_active_but_not_low_half")
if (not scaffold_present) and runtime_latest is not None and runtime_latest["kheap_low_half"] != 0:
    runtime_violations.append("runtime_no_scaffold_but_low_half_kheap")

runtime_proof = {
    "status": "FAIL",
    "mode": mode,
    "phase_profile": phase_profile,
    "phase10a2_evidence_dir": str(a2_evidence_dir),
    "selected_pid": runtime_selected_pid,
    "pid_selection_mode": runtime_pid_selection,
    "require_terminal_slice": require_terminal_slice,
    "all_runtime_pids": runtime_all_pids,
    "all_terminal_runtime_pids": runtime_all_terminal_pids,
    "runtime_marker_log": str(runtime_log_path) if runtime_log_path is not None else None,
    "runtime_marker": RUNTIME_MARKER,
    "records_count": len(runtime_records),
    "required_phases": RUNTIME_REQUIRED_PHASES,
    "optional_phases": RUNTIME_OPTIONAL_PHASES,
    "phase_records": {
        phase: format_runtime_record(record)
        for phase, record in runtime_selected_by_phase.items()
    },
    "records": [format_runtime_record(record) for record in runtime_records],
    "latest_record": format_runtime_record(runtime_latest) if runtime_latest is not None else None,
    "temporal_invariants": {
        "required_phase_presence": all(
            phase in runtime_selected_by_phase for phase in RUNTIME_REQUIRED_PHASES
        ),
        "single_pid": len(runtime_pid_values) <= 1,
        "single_pml4": len(runtime_pml4_values) <= 1,
        "strict_seq_order": "runtime_sequence_not_strictly_increasing" not in runtime_violations,
        "kheap_half_monotonic_nonincreasing": (
            "runtime_kheap_half_reverted_to_low_half" not in runtime_violations
        ),
        "user_zero_all_phases": not any(
            violation.startswith("runtime_kheap_mapping_user_accessible:")
            for violation in runtime_violations
        ),
        "exit_post_global_lower_half_cleared": not any(
            violation in runtime_violations
            for violation in (
                "runtime_exit_post_lower_half_root_entries_remaining",
                "runtime_exit_post_lower_half_leaf_mappings_remaining",
                "runtime_exit_post_lower_half_user_leaf_mappings_remaining",
            )
        ),
    },
    "violations": runtime_violations,
    "violations_count": len(runtime_violations),
}

if not runtime_violations:
    runtime_proof["status"] = "PASS"

report: dict[str, Any] = {
    "gate": "low-half-kheap-scaffold",
    "mode": mode,
    "phase_profile": phase_profile,
    "verdict": "FAIL",
    "scaffold_present": scaffold_present,
    "phase10a2_evidence_dir": str(a2_evidence_dir),
    "runtime_proof_json": str(runtime_proof_path),
    "selected_runtime_pid": runtime_selected_pid,
    "policy": {
        "state": debt_state,
        "allowed": allowed,
        "deadline_phase": "Phase11",
        "target_memory_model": "higher_half_only_kernel_heap",
        "mode": mode,
    },
    "code_checks": {
        "macro_present": macro_present,
        "macro_enabled": macro_enabled,
        "macro_value": macro_value,
        "helper_declared": helper_declared,
        "helper_implemented": helper_implemented,
        "helper_used_in_paging_create_user_pml4": helper_used_in_paging_create,
        "helper_used_in_user_as_create": helper_used_in_user_as,
    },
    "address_checks": {
        "kheap_start": f"0x{kheap_start:016x}" if kheap_start is not None else None,
        "kernel_virt_base": (
            f"0x{kernel_virt_base:016x}" if kernel_virt_base is not None else None
        ),
        "higher_half_min": f"0x{HIGHER_HALF_MIN:016x}",
        "kheap_is_low_half": kheap_is_low_half,
        "kheap_is_higher_half": kheap_is_higher_half,
        "kernel_virt_base_is_higher_half": kernel_virt_base_is_higher_half,
    },
    "doc_checks": {
        "runtime_truth_ok": runtime_truth_ok,
        "phase10_design_truth_ok": design_truth_ok,
        "phase10_requirements_truth_ok": requirements_truth_ok,
        "phase10_tasks_gate_registered": tasks_truth_ok,
        "phase11_requirements_deadline_ok": phase11_requirements_ok,
        "phase11_tasks_closure_blocker_ok": phase11_tasks_ok,
        "contradictory_truth_hits": contradictory_truth_hits,
    },
    "runtime_checks": {
        "phase10a2_runtime_log": str(runtime_log_path) if runtime_log_path is not None else None,
        "runtime_all_pids": runtime_all_pids,
        "runtime_terminal_pids": runtime_all_terminal_pids,
        "runtime_selected_pid": runtime_selected_pid,
        "runtime_pid_selection_mode": runtime_pid_selection,
        "runtime_phase_profile": phase_profile,
        "runtime_terminal_slice_required": require_terminal_slice,
        "runtime_proof_present": bool(runtime_records),
        "runtime_records_count": len(runtime_records),
        "runtime_required_phases": RUNTIME_REQUIRED_PHASES,
        "runtime_phase_records": runtime_proof["phase_records"],
        "runtime_latest_record": runtime_proof["latest_record"],
        "runtime_temporal_invariants": runtime_proof["temporal_invariants"],
        "runtime_status": runtime_proof["status"],
    },
    "violations": [],
    "violations_count": 0,
}

violations: list[str] = []

if kernel_virt_base is None:
    violations.append("missing_kernel_virt_base_anchor")
elif not kernel_virt_base_is_higher_half:
    violations.append("kernel_virt_base_not_in_higher_half")

if kheap_start is None:
    violations.append("missing_kheap_start_anchor")

if has_contradictory_truth:
    for doc_name, hits in contradictory_truth_hits.items():
        for phrase in hits:
            violations.append(f"contradictory_truth_statement:{doc_name}:{phrase}")

if scaffold_present:
    if not macro_present:
        violations.append("scaffold_present_without_explicit_macro")
    if macro_value is None:
        violations.append("scaffold_present_without_macro_value")
    elif macro_value == 0:
        violations.append("scaffold_code_present_but_macro_disabled")
    if not helper_declared:
        violations.append("scaffold_present_without_helper_declaration")
    if not helper_implemented:
        violations.append("scaffold_present_without_helper_implementation")
    if not helper_used_in_paging_create:
        violations.append("scaffold_present_without_paging_create_seed")
    if not helper_used_in_user_as:
        violations.append("scaffold_present_without_user_as_seed")
    if not kheap_is_low_half:
        violations.append("scaffold_active_but_kheap_not_in_low_half")
    if not runtime_truth_ok:
        violations.append("missing_runtime_truth_for_active_scaffold")
    if not design_truth_ok:
        violations.append("missing_design_truth_for_active_scaffold")
    if not requirements_truth_ok:
        violations.append("missing_requirements_truth_for_active_scaffold")
    if not tasks_truth_ok:
        violations.append("missing_phase10_visibility_gate_registration")
    if not phase11_requirements_ok:
        violations.append("missing_phase11_deadline_requirement")
    if not phase11_tasks_ok:
        violations.append("missing_phase11_closure_blocker")
    if mode == "forbid":
        violations.append("low_half_kheap_scaffold_present")
else:
    if kheap_start is not None and not kheap_is_higher_half:
        violations.append("kheap_not_in_higher_half_after_scaffold_removal")
    stale_doc_hits = []
    if "mirror the current low-half kernel heap" in lower_texts["runtime_reality"]:
        stale_doc_hits.append("runtime_reality")
    if "mirror the current low-half kernel heap" in lower_texts["phase10_design"]:
        stale_doc_hits.append("phase10_design")
    if "temporary supervisor-only compatibility scaffold" in lower_texts["phase10_requirements"]:
        stale_doc_hits.append("phase10_requirements")
    for hit in stale_doc_hits:
        violations.append(f"stale_scaffold_doc:{hit}")

violations.extend(runtime_violations)
report["violations"] = violations
report["violations_count"] = len(violations)

runtime_proof_path.write_text(
    json.dumps(runtime_proof, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)

if violations:
    report["verdict"] = "FAIL"
    report["status"] = (
        "active_scaffold_blocked_for_phase11_closure"
        if mode == "forbid" and scaffold_present
        else "scaffold_truth_violation"
    )
    violations_path.write_text("\n".join(violations) + "\n", encoding="utf-8")
    report_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    sys.exit(2)

report["verdict"] = "PASS"
report["status"] = (
    "temporary_scaffold_visible_and_bounded"
    if scaffold_present
    else "no_low_half_kernel_dependency_detected"
)
violations_path.write_text("", encoding="utf-8")
report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
status=$?
set -e

cat > "${META_TXT}" <<EOF
mode=${MODE}
phase10a2_evidence_dir=${A2_EVIDENCE_DIR}
report_json=${REPORT_JSON}
runtime_proof_json=${RUNTIME_PROOF_JSON}
violations_txt=${VIOLATIONS_TXT}
EOF

exit "${status}"
