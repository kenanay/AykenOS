#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_low_half_kheap_multi_exit_proof.sh \
    --evidence-dir evidence/run-<id>/gates/low-half-kheap-multi-exit-proof \
    [--qemu-timeout <sec>]

Environment:
  AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT=<n>  Positive integer exit count
                                                  for the validation-only N-exit workload

Exit codes:
  0: pass
  1: tooling/infra failure
  2: proof contract failure
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${QEMU_TIMEOUT:-35}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
AYKEN_CR3_PCID="${AYKEN_CR3_PCID:-0}"
AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST:-1}"
AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST:-0}"
AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY:-0}"
AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST="${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST:-0}"
AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST="${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST:-1}"
AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT="${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT:-2}"
EXPECTED_USER_MINIMAL_MODE="phase10a2"
OBSERVED_USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"
BOOT_AUDIT_MARKER="[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_OK]]"
SELFTEST_ARMED_MARKER="[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_ARMED]]"
SELFTEST_LINEAGE_MARKER="[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_LINEAGE]]"
SELFTEST_FAIL_MARKER="[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]]"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --qemu-timeout)
      QEMU_TIMEOUT="$2"
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
if [[ "${OBSERVED_USER_MINIMAL_MODE}" != "${EXPECTED_USER_MINIMAL_MODE}" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires USER_MINIMAL_MODE=${EXPECTED_USER_MINIMAL_MODE} (current=${OBSERVED_USER_MINIMAL_MODE:-unset})" >&2
  exit 3
fi
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 3
fi
if [[ "${AYKEN_CR3_PCID}" != "0" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires AYKEN_CR3_PCID=0 (current=${AYKEN_CR3_PCID})" >&2
  exit 3
fi
if [[ "${AYKEN_MB_SELFTEST}" != "1" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires AYKEN_MB_SELFTEST=1 (current=${AYKEN_MB_SELFTEST})" >&2
  exit 3
fi
if [[ "${AYKEN_GATE4_POLICY_TEST}" != "0" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires AYKEN_GATE4_POLICY_TEST=0 (current=${AYKEN_GATE4_POLICY_TEST})" >&2
  exit 3
fi
if [[ "${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST}" != "0" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST=0 (current=${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST})" >&2
  exit 3
fi
if [[ "${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST}" != "1" ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST=1 (current=${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST})" >&2
  exit 3
fi
if [[ ! "${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT}" =~ ^[1-9][0-9]*$ ]]; then
  echo "ERROR: low-half-kheap-multi-exit-proof requires AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT to be a positive integer (current=${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT})" >&2
  exit 3
fi

for tool in bash make python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
SCAFFOLD_GATE="${ROOT}/scripts/ci/gate_low_half_kheap_scaffold.sh"
if [[ ! -x "${BOOT_AUDIT}" ]]; then
  echo "ERROR: missing boot audit script: ${BOOT_AUDIT}" >&2
  exit 1
fi
if [[ ! -x "${SCAFFOLD_GATE}" ]]; then
  echo "ERROR: missing scaffold gate: ${SCAFFOLD_GATE}" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
BOOT_AUDIT_LOG="${EVIDENCE_DIR}/boot_audit.log"
BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
MARKER_LOG="${EVIDENCE_DIR}/marker.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
CONTRACT_JSON="${EVIDENCE_DIR}/lineage_contract.json"
LINEAGE_PIDS_TXT="${EVIDENCE_DIR}/lineage_pids.txt"
LINEAGE_RESULTS_TXT="${EVIDENCE_DIR}/lineage_results.txt"
LINEAGES_DIR="${EVIDENCE_DIR}/lineages"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${COMBINED_LOG}"
: > "${MARKER_LOG}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"
: > "${LINEAGE_PIDS_TXT}"
: > "${LINEAGE_RESULTS_TXT}"
mkdir -p "${LINEAGES_DIR}"

refresh_marker_log() {
  cat "${BOOT_AUDIT_DIR}/qemu_serial.log" "${BOOT_AUDIT_DIR}/qemu_debugcon.log" 2>/dev/null > "${COMBINED_LOG}" || true
  if [[ -s "${BOOT_AUDIT_DIR}/qemu_debugcon.log" ]]; then
    cp -f "${BOOT_AUDIT_DIR}/qemu_debugcon.log" "${MARKER_LOG}"
  else
    cp -f "${COMBINED_LOG}" "${MARKER_LOG}"
  fi
}

wait_for_marker_log() {
  local attempts=20
  local i
  for ((i = 1; i <= attempts; i++)); do
    refresh_marker_log
    if [[ -s "${MARKER_LOG}" ]]; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

set +e
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" \
  AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST}" \
  AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST}" \
  AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}" \
  AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST=0 \
  AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST=1 \
  AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT="${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT}" \
  clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" \
  AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST}" \
  AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST}" \
  AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}" \
  AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST=0 \
  AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST=1 \
  AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT="${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT}" \
  guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
row = {
    "gate": "low-half-kheap-multi-exit-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["build_failed"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "low-half-kheap-multi-exit-proof: INFRA FAIL (build_failed)"
  exit 1
fi

set +e
"${BOOT_AUDIT}" \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "${BOOT_AUDIT_MARKER}" \
  --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_AUDIT_LOG}" 2>&1
BOOT_AUDIT_RC=$?
set -e

if ! wait_for_marker_log; then
  python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
row = {
    "gate": "low-half-kheap-multi-exit-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["marker_log_empty"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "marker_log_empty" > "${VIOLATIONS_TXT}"
  echo "low-half-kheap-multi-exit-proof: FAIL (marker_log_empty)"
  exit 2
fi

python3 - "${MARKER_LOG}" "${CONTRACT_JSON}" "${LINEAGE_PIDS_TXT}" "${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT}" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path
import re

marker_log = Path(sys.argv[1])
contract_path = Path(sys.argv[2])
pids_path = Path(sys.argv[3])
configured_exit_count = int(sys.argv[4], 0)

ARMED = "[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_ARMED]]"
LINEAGE = "[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_LINEAGE]]"
OK = "[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_OK]]"
FAIL = "[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]]"

def parse_scalar(value: str) -> int:
    match = re.match(r"^(0[xX][0-9A-Fa-f]+|[0-9]+)", value)
    if not match:
        raise ValueError(value)
    return int(match.group(1), 0)


def parse_fields(raw: str, marker: str) -> dict[str, str]:
    idx = raw.find(marker)
    if idx < 0:
        return {}
    body = raw[idx + len(marker):].strip()
    fields: dict[str, str] = {}
    for token in body.split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        fields[key] = value
    return fields

armed = []
lineages = []
ok_rows = []
fail_rows = []

for line_no, raw in enumerate(marker_log.read_text(encoding="utf-8", errors="replace").splitlines(), start=1):
    if ARMED in raw:
        fields = parse_fields(raw, ARMED)
        try:
            armed.append({
                "line": line_no,
                "slot": parse_scalar(fields["slot"]),
                "total": parse_scalar(fields["total"]),
                "owner_pid": parse_scalar(fields["owner_pid"]),
                "exit_pid": parse_scalar(fields["exit_pid"]),
                "return_pid": parse_scalar(fields["return_pid"]),
                "raw_line": raw.strip(),
            })
        except (KeyError, ValueError):
            fail_rows.append({"line": line_no, "raw_line": raw.strip(), "reason": "armed_parse_failed"})
    if LINEAGE in raw:
        fields = parse_fields(raw, LINEAGE)
        try:
            lineages.append({
                "line": line_no,
                "slot": parse_scalar(fields["slot"]),
                "total": parse_scalar(fields["total"]),
                "owner_pid": parse_scalar(fields["owner_pid"]),
                "exit_pid": parse_scalar(fields["exit_pid"]),
                "return_pid": parse_scalar(fields["return_pid"]),
                "switch_from_pid": parse_scalar(fields["switch_from_pid"]),
                "switch_to_pid": parse_scalar(fields["switch_to_pid"]),
                "raw_line": raw.strip(),
            })
        except (KeyError, ValueError):
            fail_rows.append({"line": line_no, "raw_line": raw.strip(), "reason": "lineage_parse_failed"})
    if OK in raw:
        fields = parse_fields(raw, OK)
        try:
            exit_pids = [int(item) for item in re.findall(r"\d+", fields["exit_pids"])]
            ok_rows.append({
                "line": line_no,
                "owner_pid": parse_scalar(fields["owner_pid"]),
                "total": parse_scalar(fields["total"]),
                "return_pid": parse_scalar(fields["return_pid"]),
                "exit_pids": exit_pids,
                "raw_line": raw.strip(),
            })
        except (KeyError, ValueError):
            fail_rows.append({"line": line_no, "raw_line": raw.strip(), "reason": "ok_parse_failed"})
    if FAIL in raw:
        fail_rows.append({"line": line_no, "raw_line": raw.strip(), "reason": "selftest_fail_marker"})

violations: list[str] = []
ok_row = ok_rows[0] if len(ok_rows) == 1 else None

if len(ok_rows) != 1:
    violations.append(f"unexpected_multi_exit_ok_marker_count:{len(ok_rows)}")
if ok_row is None:
    violations.append("missing_multi_exit_ok_marker")

lineages.sort(key=lambda row: (row["slot"], row["line"]))
armed.sort(key=lambda row: (row["slot"], row["line"]))

expected_total = ok_row["total"] if ok_row is not None else None
if expected_total is None and lineages:
    expected_total = lineages[0]["total"]
if expected_total is None and armed:
    expected_total = armed[0]["total"]
if expected_total is None:
    expected_total = 0

if expected_total <= 0:
    violations.append("invalid_expected_multi_exit_total")
if expected_total != configured_exit_count:
    violations.append(
        f"configured_exit_count_mismatch:expected={configured_exit_count}:observed={expected_total}"
    )
if len(armed) != expected_total:
    violations.append(f"unexpected_multi_exit_armed_count:{len(armed)}")
if len(lineages) != expected_total:
    violations.append(f"unexpected_multi_exit_lineage_count:{len(lineages)}")
if len(fail_rows) != 0:
    violations.append(f"multi_exit_fail_markers_present:{len(fail_rows)}")

expected_slots = list(range(1, expected_total + 1))
armed_slots = [row["slot"] for row in armed]
lineage_slots = [row["slot"] for row in lineages]
if armed_slots != expected_slots:
    violations.append("multi_exit_armed_slots_noncanonical")
if lineage_slots != expected_slots:
    violations.append("multi_exit_lineage_slots_noncanonical")

armed_by_slot = {row["slot"]: row for row in armed}
lineage_by_slot = {row["slot"]: row for row in lineages}
owner_values = set()
return_values = set()
lineage_exit_pids = []
for slot in expected_slots:
    armed_row = armed_by_slot.get(slot)
    lineage_row = lineage_by_slot.get(slot)
    if armed_row is None:
        violations.append(f"missing_multi_exit_armed_slot:{slot}")
        continue
    if lineage_row is None:
        violations.append(f"missing_multi_exit_lineage_slot:{slot}")
        continue
    owner_values.add(armed_row["owner_pid"])
    owner_values.add(lineage_row["owner_pid"])
    return_values.add(armed_row["return_pid"])
    return_values.add(lineage_row["return_pid"])
    if armed_row["total"] != expected_total or lineage_row["total"] != expected_total:
        violations.append(f"multi_exit_total_mismatch:slot{slot}")
    if armed_row["exit_pid"] != lineage_row["exit_pid"]:
        violations.append(f"multi_exit_exit_pid_mismatch:slot{slot}")
    if armed_row["return_pid"] != lineage_row["return_pid"]:
        violations.append(f"multi_exit_return_pid_mismatch:slot{slot}")
    if lineage_row["switch_from_pid"] != lineage_row["exit_pid"]:
        violations.append(f"multi_exit_switch_from_mismatch:slot{slot}")
    if lineage_row["switch_to_pid"] != lineage_row["return_pid"]:
        violations.append(f"multi_exit_switch_to_mismatch:slot{slot}")
    lineage_exit_pids.append(lineage_row["exit_pid"])

if len(owner_values) > 1:
    violations.append("multi_exit_owner_pid_inconsistent")
if len(return_values) > 1:
    violations.append("multi_exit_return_pid_inconsistent")
if len(set(lineage_exit_pids)) != len(lineage_exit_pids):
    violations.append("multi_exit_duplicate_exit_pid")
if ok_row is not None:
    if ok_row["exit_pids"] != lineage_exit_pids:
        violations.append("multi_exit_ok_exit_pid_list_mismatch")
    if ok_row["owner_pid"] not in owner_values:
        violations.append("multi_exit_ok_owner_pid_mismatch")
    if ok_row["return_pid"] not in return_values:
        violations.append("multi_exit_ok_return_pid_mismatch")

contract = {
    "verdict": "PASS" if not violations else "FAIL",
    "violations": violations,
    "violations_count": len(violations),
    "configured_exit_count": configured_exit_count,
    "expected_total": expected_total,
    "armed_rows": armed,
    "lineage_rows": lineages,
    "ok_rows": ok_rows,
    "fail_rows": fail_rows,
    "selected_exit_pids": lineage_exit_pids,
    "owner_pid": next(iter(owner_values)) if len(owner_values) == 1 else None,
    "return_pid": next(iter(return_values)) if len(return_values) == 1 else None,
}
contract_path.write_text(json.dumps(contract, indent=2, sort_keys=True) + "\n", encoding="utf-8")

with pids_path.open("w", encoding="utf-8") as fh:
    if not violations:
        for row in contract["lineage_rows"]:
            fh.write(f"{row['slot']} {row['exit_pid']}\n")
PY

while read -r SLOT PID; do
  [[ -z "${SLOT}" || -z "${PID}" ]] && continue
  SLOT_DIR="${LINEAGES_DIR}/slot-${SLOT}-pid-${PID}"
  mkdir -p "${SLOT_DIR}"
  set +e
  bash "${SCAFFOLD_GATE}" \
    --evidence-dir "${SLOT_DIR}/scaffold-proof" \
    --phase10a2-evidence "${EVIDENCE_DIR}" \
    --mode allow \
    --runtime-pid "${PID}" \
    --phase-profile terminal_lineage \
    --require-terminal-slice
  SCAFFOLD_RC=$?
  set -e
  echo "${SLOT} ${PID} ${SCAFFOLD_RC} ${SLOT_DIR}/scaffold-proof/report.json ${SLOT_DIR}/scaffold-proof/runtime_proof.json" >> "${LINEAGE_RESULTS_TXT}"
done < "${LINEAGE_PIDS_TXT}"

python3 - "${REPORT_JSON}" "${CONTRACT_JSON}" "${LINEAGE_RESULTS_TXT}" "${BOOT_AUDIT_RC}" "${QEMU_TIMEOUT}" "${BOOT_AUDIT_MARKER}" "${SELFTEST_ARMED_MARKER}" "${SELFTEST_LINEAGE_MARKER}" "${SELFTEST_FAIL_MARKER}" "${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT}" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
contract_path = Path(sys.argv[2])
lineage_results_path = Path(sys.argv[3])
boot_audit_rc = int(sys.argv[4])
qemu_timeout = int(sys.argv[5])
boot_audit_marker = sys.argv[6]
selftest_armed_marker = sys.argv[7]
selftest_lineage_marker = sys.argv[8]
selftest_fail_marker = sys.argv[9]
configured_exit_count = int(sys.argv[10], 0)

violations: list[str] = []
contract = {}
if contract_path.is_file():
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
else:
    violations.append("missing_lineage_contract")

if boot_audit_rc != 0:
    violations.append(f"boot_audit_failed:rc={boot_audit_rc}")

violations.extend(contract.get("violations", []))
if contract.get("configured_exit_count") != configured_exit_count:
    violations.append("contract_configured_exit_count_mismatch")

results = []
runtime_terminal_pid_union: set[int] = set()
for line in lineage_results_path.read_text(encoding="utf-8", errors="replace").splitlines():
    if not line.strip():
        continue
    slot_s, pid_s, rc_s, report_s, runtime_proof_s = line.split(" ", 4)
    item = {
        "slot": int(slot_s),
        "exit_pid": int(pid_s),
        "scaffold_exit_code": int(rc_s),
        "scaffold_report_path": report_s,
        "runtime_proof_path": runtime_proof_s,
        "scaffold_verdict": "FAIL",
        "terminal_global_lower_half_cleared": False,
        "runtime_terminal_pids": [],
    }
    report_path_nested = Path(report_s)
    if not report_path_nested.is_file():
        violations.append(f"missing_nested_scaffold_report:slot{item['slot']}")
        results.append(item)
        continue
    nested = json.loads(report_path_nested.read_text(encoding="utf-8"))
    item["scaffold_verdict"] = nested.get("verdict", "FAIL")
    if item["scaffold_exit_code"] != 0:
        violations.append(f"nested_scaffold_failed:slot{item['slot']}:rc={item['scaffold_exit_code']}")
    if item["scaffold_verdict"] != "PASS":
        violations.append(f"nested_scaffold_verdict_failed:slot{item['slot']}")
    if nested.get("selected_runtime_pid") != item["exit_pid"]:
        violations.append(f"nested_selected_pid_mismatch:slot{item['slot']}")
    runtime_checks = nested.get("runtime_checks", {})
    runtime_terminal_pids = runtime_checks.get("runtime_terminal_pids", [])
    item["runtime_terminal_pids"] = runtime_terminal_pids
    runtime_terminal_pid_union.update(int(pid) for pid in runtime_terminal_pids)
    temporal = runtime_checks.get("runtime_temporal_invariants", {})
    item["terminal_global_lower_half_cleared"] = bool(
        temporal.get("exit_post_global_lower_half_cleared", False)
    )
    if not item["terminal_global_lower_half_cleared"]:
        violations.append(f"nested_terminal_global_lower_half_not_cleared:slot{item['slot']}")
    results.append(item)

expected_exit_pids = contract.get("selected_exit_pids", [])
observed_slots = [row["slot"] for row in results]
if expected_exit_pids and len(results) != len(expected_exit_pids):
    violations.append("nested_scaffold_coverage_incomplete")
if observed_slots and observed_slots != list(range(1, len(observed_slots) + 1)):
    violations.append("nested_scaffold_slot_order_noncanonical")
if expected_exit_pids and sorted(runtime_terminal_pid_union) != sorted(expected_exit_pids):
    violations.append("runtime_terminal_pid_union_mismatch")

seen = set()
ordered_violations = []
for item in violations:
    if item in seen:
        continue
    seen.add(item)
    ordered_violations.append(item)

row = {
    "gate": "low-half-kheap-multi-exit-proof",
    "verdict": "PASS" if not ordered_violations else "FAIL",
    "violations_count": len(ordered_violations),
    "violations": ordered_violations,
    "boot_audit_exit_code": boot_audit_rc,
    "qemu_timeout_seconds": qemu_timeout,
    "boot_audit_marker": boot_audit_marker,
    "selftest_armed_marker": selftest_armed_marker,
    "selftest_lineage_marker": selftest_lineage_marker,
    "selftest_fail_marker": selftest_fail_marker,
    "owner_pid": contract.get("owner_pid"),
    "return_pid": contract.get("return_pid"),
    "configured_exit_count": configured_exit_count,
    "expected_exit_count": contract.get("expected_total"),
    "selected_exit_pids": expected_exit_pids,
    "lineage_rows": contract.get("lineage_rows", []),
    "armed_rows": contract.get("armed_rows", []),
    "lineage_fail_rows": contract.get("fail_rows", []),
    "coverage_complete": bool(expected_exit_pids) and len(results) == len(expected_exit_pids),
    "runtime_terminal_pid_union": sorted(runtime_terminal_pid_union),
    "lineage_reports": results,
    "debt_removed": False,
}

report_path.write_text(json.dumps(row, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

python3 - "${REPORT_JSON}" "${VIOLATIONS_TXT}" <<'PY'
import json
import sys

report_path = sys.argv[1]
violations_path = sys.argv[2]
with open(report_path, "r", encoding="utf-8") as fh:
    report = json.load(fh)
with open(violations_path, "w", encoding="utf-8") as fh:
    for item in report.get("violations", []):
        fh.write(f"{item}\n")
PY

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "boot_audit_marker=${BOOT_AUDIT_MARKER}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "user_minimal_mode=${OBSERVED_USER_MINIMAL_MODE}"
  echo "ayken_cr3_pcid=${AYKEN_CR3_PCID}"
  echo "ayken_mb_selftest=${AYKEN_MB_SELFTEST}"
  echo "ayken_gate4_policy_test=${AYKEN_GATE4_POLICY_TEST}"
  echo "ayken_sched_bootstrap_policy=${AYKEN_SCHED_BOOTSTRAP_POLICY}"
  echo "ayken_low_half_kheap_exit_proof_selftest=${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST}"
  echo "ayken_low_half_kheap_multi_exit_proof_selftest=${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST}"
  echo "ayken_low_half_kheap_multi_exit_proof_count=${AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT}"
  echo "build_rc=${BUILD_RC}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
} > "${META_TXT}"

if python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
with open(sys.argv[1], "r", encoding="utf-8") as fh:
    report = json.load(fh)
raise SystemExit(0 if report.get("verdict") == "PASS" else 1)
PY
then
  echo "low-half-kheap-multi-exit-proof: PASS"
  exit 0
fi

echo "low-half-kheap-multi-exit-proof: FAIL"
exit 2
