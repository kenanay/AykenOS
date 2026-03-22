#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/check_phase10b_execution_hardening.sh \
    --evidence-dir evidence/run-<id>/gates/phase10b-execution-hardening

Exit codes:
  0: pass
  2: source-level hardening guard failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
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

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
ROOT="${ROOT}" REPORT_JSON="${REPORT_JSON}" VIOLATIONS_TXT="${VIOLATIONS_TXT}" python3 - <<'PY'
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

root = Path(os.environ["ROOT"])
report_path = Path(os.environ["REPORT_JSON"])
violations_path = Path(os.environ["VIOLATIONS_TXT"])

serialization_targets = [
    {
        "file": "kernel/sys/syscall_v2.c",
        "function": "sys_v2_submit_execution",
        "required_tokens": [
            "execution_slot_enter_critical",
            "execution_slot_require_transition_locked",
        ],
    },
    {
        "file": "kernel/sys/syscall_v2.c",
        "function": "sys_v2_wait_result",
        "required_tokens": ["execution_slot_enter_critical"],
    },
    {
        "file": "kernel/sys/syscall_v2.c",
        "function": "sys_v2_complete_execution",
        "required_tokens": [
            "execution_slot_enter_critical",
            "execution_slot_require_finish_locked",
        ],
    },
    {
        "file": "kernel/sys/syscall_v2.c",
        "function": "sys_v2_exit",
        "required_tokens": [
            "execution_slot_enter_critical",
            "execution_slot_prepare_process_exit_locked",
        ],
    },
    {
        "file": "kernel/sched/sched.c",
        "function": "sched_try_pickup_execution_work",
        "required_tokens": [
            "execution_slot_enter_critical",
            "execution_slot_pickup_locked",
            "execution_slot_require_finish_locked",
        ],
    },
    {
        "file": "kernel/arch/x86_64/timer.c",
        "function": "timer_isr_c",
        "required_tokens": [
            "execution_slot_enter_critical",
            "execution_slot_process_timeouts_locked",
        ],
    },
]

execution_surface_files = [
    "kernel/sys/syscall_v2.c",
    "kernel/sys/execution_slot.c",
    "kernel/include/execution_slot.h",
    "kernel/include/execution_inbox_abi.h",
    "kernel/include/execution_output_abi.h",
    "kernel/include/execution_output_structured_abi.h",
    "kernel/include/execution_result_hash_abi.h",
    "shared/abi/execution_inbox_abi.h",
    "shared/abi/execution_output_abi.h",
    "shared/abi/execution_output_structured_abi.h",
    "shared/abi/execution_result_hash_abi.h",
    "userspace/bcib-runtime/src/executor.rs",
    "userspace/bcib-runtime/src/bin/dispatcher.rs",
]

scheduler_mailbox_files = [
    "kernel/sched/sched_mailbox.c",
    "kernel/sched/sched_mailbox.h",
    "kernel/include/sched_mailbox_abi.h",
    "kernel/sched/sched_mailbox_abi_sanity.c",
    "shared/abi/sched_mailbox_abi.h",
]

scheduler_mailbox_pattern = re.compile(
    r"\b(?:sched_mailbox_|SCHED_MAILBOX_VA|AYKEN_SCHED_MB_|ayken_sched_mailbox_t)\b"
)
execution_surface_pattern = re.compile(
    r"\b(?:EXECUTION_INBOX_VA|EXECUTION_PAYLOAD_VA|EXECUTION_OUTPUT_VA|"
    r"AYKEN_EXECUTION_INBOX_|AYKEN_EXECUTION_OUTPUT_|AYKEN_EXECUTION_RESULT_HASH_|"
    r"ayken_execution_inbox_v1_t|ayken_execution_output_v1_t|"
    r"ayken_execution_output_v2_t|ayken_execution_result_hash_v1_t)\b"
)
raw_transition_helper_pattern = re.compile(
    r"\bexecution_slot_(?:transition|finish)_locked\s*\("
)

runtime_fail_closed_files = [
    "kernel/sys/syscall_v2.c",
    "kernel/sched/sched.c",
]


def extract_function(text: str, function_name: str) -> str | None:
    match = re.search(
        rf"(?m)^[^\S\n]*[A-Za-z_][A-Za-z0-9_\s\*]*\b{re.escape(function_name)}\s*\([^;]*\)\s*\{{",
        text,
    )
    if not match:
        return None

    brace_start = text.find("{", match.start())

    depth = 0
    for idx in range(brace_start, len(text)):
        ch = text[idx]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return text[brace_start : idx + 1]
    return None


def scan_forbidden(path: Path, pattern: re.Pattern[str]) -> list[dict[str, object]]:
    hits: list[dict[str, object]] = []
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if pattern.search(raw):
            hits.append({"line": line_no, "text": raw.strip()})
    return hits


report: dict[str, object] = {
    "gate": "phase10b-execution-hardening",
    "verdict": "FAIL",
    "violations": [],
    "violations_count": 0,
    "serialization_checks": [],
    "fail_closed_checks": [],
    "boundary_checks": [],
}

violations: list[str] = []

for target in serialization_targets:
    path = root / str(target["file"])
    body = extract_function(path.read_text(encoding="utf-8"), str(target["function"]))
    missing: list[str] = []
    if body is None:
        missing.extend(target["required_tokens"])
        violations.append(f"missing_function:{target['file']}:{target['function']}")
    else:
        for token in target["required_tokens"]:
            if token not in body:
                missing.append(token)
                violations.append(
                    f"missing_serialization_token:{target['file']}:{target['function']}:{token}"
                )

    report["serialization_checks"].append(
        {
            "file": target["file"],
            "function": target["function"],
            "required_tokens": target["required_tokens"],
            "missing_tokens": missing,
            "ok": not missing,
        }
    )

for scope, files, pattern in (
    ("execution_surface", execution_surface_files, scheduler_mailbox_pattern),
    ("scheduler_mailbox_surface", scheduler_mailbox_files, execution_surface_pattern),
):
    scope_hits: dict[str, list[dict[str, object]]] = {}
    for rel_path in files:
        path = root / rel_path
        hits = scan_forbidden(path, pattern)
        if hits:
            scope_hits[rel_path] = hits
            violations.append(f"boundary_violation:{scope}:{rel_path}")

    report["boundary_checks"].append(
        {
            "scope": scope,
            "files": files,
            "forbidden_pattern": pattern.pattern,
            "hits": scope_hits,
            "ok": not scope_hits,
        }
    )

for rel_path in runtime_fail_closed_files:
    path = root / rel_path
    hits = scan_forbidden(path, raw_transition_helper_pattern)
    if hits:
        violations.append(f"fail_closed_violation:{rel_path}")
    report["fail_closed_checks"].append(
        {
            "file": rel_path,
            "forbidden_pattern": raw_transition_helper_pattern.pattern,
            "hits": hits,
            "ok": not hits,
        }
    )

report["violations"] = violations
report["violations_count"] = len(violations)
report["verdict"] = "PASS" if not violations else "FAIL"

report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
violations_path.write_text(
    "".join(f"{item}\n" for item in violations),
    encoding="utf-8",
)

sys.exit(0 if not violations else 2)
PY
RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "root=${ROOT}"
  echo "validator_rc=${RC}"
} > "${META_TXT}"

if [[ "${RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "phase10b-execution-hardening: FAIL (${COUNT} violations)"
  exit 2
fi

echo "phase10b-execution-hardening: PASS"
exit 0
