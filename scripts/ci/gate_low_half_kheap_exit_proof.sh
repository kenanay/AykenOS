#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_low_half_kheap_exit_proof.sh \
    --evidence-dir evidence/run-<id>/gates/low-half-kheap-exit-proof \
    [--qemu-timeout <sec>]

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
AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST="${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST:-1}"
EXPECTED_USER_MINIMAL_MODE="phase10a2"
OBSERVED_USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"
BOOT_AUDIT_MARKER="[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_OK]]"
SELFTEST_ARMED_MARKER="[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_ARMED]]"
SELFTEST_FAIL_MARKER="[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_FAIL]]"
SELFTEST_OK_PRESENT=0
SELFTEST_OK_PARSE_OK=0
SELFTEST_OK_COUNT=0
SELFTEST_EXIT_PID=""
SELFTEST_RETURN_PID=""

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
  echo "ERROR: low-half-kheap-exit-proof requires USER_MINIMAL_MODE=${EXPECTED_USER_MINIMAL_MODE} (current=${OBSERVED_USER_MINIMAL_MODE:-unset})" >&2
  exit 3
fi
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: low-half-kheap-exit-proof requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 3
fi
if [[ "${AYKEN_CR3_PCID}" != "0" ]]; then
  echo "ERROR: low-half-kheap-exit-proof requires AYKEN_CR3_PCID=0 (current=${AYKEN_CR3_PCID})" >&2
  exit 3
fi
if [[ "${AYKEN_MB_SELFTEST}" != "1" ]]; then
  echo "ERROR: low-half-kheap-exit-proof requires AYKEN_MB_SELFTEST=1 (current=${AYKEN_MB_SELFTEST})" >&2
  exit 3
fi
if [[ "${AYKEN_GATE4_POLICY_TEST}" != "0" ]]; then
  echo "ERROR: low-half-kheap-exit-proof requires AYKEN_GATE4_POLICY_TEST=0 (current=${AYKEN_GATE4_POLICY_TEST})" >&2
  exit 3
fi
if [[ "${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST}" != "1" ]]; then
  echo "ERROR: low-half-kheap-exit-proof requires AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST=1 (current=${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST})" >&2
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
SCAFFOLD_DIR="${EVIDENCE_DIR}/scaffold-proof"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${COMBINED_LOG}"
: > "${MARKER_LOG}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

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
  AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST="${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST}" \
  clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" \
  AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST}" \
  AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST}" \
  AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}" \
  AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST="${AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST}" \
  guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
row = {
    "gate": "low-half-kheap-exit-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["build_failed"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "low-half-kheap-exit-proof: INFRA FAIL (build_failed)"
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
    "gate": "low-half-kheap-exit-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["marker_log_empty"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "marker_log_empty" > "${VIOLATIONS_TXT}"
  echo "low-half-kheap-exit-proof: FAIL (marker_log_empty)"
  exit 2
fi

SELFTEST_ARMED_PRESENT=0
SELFTEST_FAIL_PRESENT=0
if grep -q -F "${SELFTEST_ARMED_MARKER}" "${MARKER_LOG}"; then
  SELFTEST_ARMED_PRESENT=1
fi
if grep -q -F "${SELFTEST_FAIL_MARKER}" "${MARKER_LOG}"; then
  SELFTEST_FAIL_PRESENT=1
fi
SELFTEST_OK_COUNT="$(grep -c -F "${BOOT_AUDIT_MARKER}" "${MARKER_LOG}" || true)"
if [[ "${SELFTEST_OK_COUNT}" -gt 0 ]]; then
  SELFTEST_OK_PRESENT=1
fi
if [[ "${SELFTEST_OK_COUNT}" == "1" ]]; then
  SELFTEST_OK_LINE="$(grep -F "${BOOT_AUDIT_MARKER}" "${MARKER_LOG}" | tail -n 1)"
  if [[ "${SELFTEST_OK_LINE}" =~ exit_pid=([0-9]+)[[:space:]]+return_pid=([0-9]+) ]]; then
    SELFTEST_EXIT_PID="${BASH_REMATCH[1]}"
    SELFTEST_RETURN_PID="${BASH_REMATCH[2]}"
    SELFTEST_OK_PARSE_OK=1
  fi
fi

SCAFFOLD_ARGS=(
  --evidence-dir "${SCAFFOLD_DIR}"
  --phase10a2-evidence "${EVIDENCE_DIR}"
  --mode allow
  --require-terminal-slice
)
if [[ -n "${SELFTEST_EXIT_PID}" ]]; then
  SCAFFOLD_ARGS+=(--runtime-pid "${SELFTEST_EXIT_PID}")
fi

set +e
bash "${SCAFFOLD_GATE}" "${SCAFFOLD_ARGS[@]}"
SCAFFOLD_RC=$?
set -e

python3 - \
  "${REPORT_JSON}" \
  "${SCAFFOLD_DIR}/report.json" \
  "${BOOT_AUDIT_RC}" \
  "${QEMU_TIMEOUT}" \
  "${SELFTEST_ARMED_PRESENT}" \
  "${SELFTEST_FAIL_PRESENT}" \
  "${SELFTEST_OK_PRESENT}" \
  "${SELFTEST_OK_PARSE_OK}" \
  "${SELFTEST_OK_COUNT}" \
  "${SELFTEST_EXIT_PID}" \
  "${SELFTEST_RETURN_PID}" \
  "${SCAFFOLD_RC}" \
  "${BOOT_AUDIT_MARKER}" \
  "${SELFTEST_ARMED_MARKER}" \
  "${SELFTEST_FAIL_MARKER}" <<'PY'
import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
scaffold_report_path = Path(sys.argv[2])
boot_audit_rc = int(sys.argv[3])
qemu_timeout = int(sys.argv[4])
selftest_armed_present = int(sys.argv[5])
selftest_fail_present = int(sys.argv[6])
selftest_ok_present = int(sys.argv[7])
selftest_ok_parse_ok = int(sys.argv[8])
selftest_ok_count = int(sys.argv[9])
selftest_exit_pid = int(sys.argv[10]) if sys.argv[10] else None
selftest_return_pid = int(sys.argv[11]) if sys.argv[11] else None
scaffold_rc = int(sys.argv[12])
boot_audit_marker = sys.argv[13]
selftest_armed_marker = sys.argv[14]
selftest_fail_marker = sys.argv[15]

violations: list[str] = []
scaffold_report = {}
if scaffold_report_path.is_file():
    with scaffold_report_path.open("r", encoding="utf-8") as fh:
        scaffold_report = json.load(fh)
else:
    violations.append("missing_scaffold_report")

if boot_audit_rc != 0:
    violations.append(f"boot_audit_failed:rc={boot_audit_rc}")
if selftest_armed_present == 0:
    violations.append("missing_exit_selftest_armed_marker")
if selftest_fail_present != 0:
    violations.append("exit_selftest_fail_marker_present")
if selftest_ok_present == 0:
    violations.append("missing_exit_selftest_ok_marker")
if selftest_ok_count != 1:
    violations.append(f"unexpected_exit_selftest_ok_marker_count:{selftest_ok_count}")
if selftest_ok_parse_ok == 0:
    violations.append("exit_selftest_ok_marker_parse_failed")
if scaffold_rc != 0:
    violations.append(f"scaffold_gate_failed:rc={scaffold_rc}")
violations.extend(scaffold_report.get("violations", []))

seen = set()
ordered_violations = []
for item in violations:
    if item in seen:
        continue
    seen.add(item)
    ordered_violations.append(item)

selected_runtime_pid = scaffold_report.get("selected_runtime_pid")
runtime_checks = scaffold_report.get("runtime_checks", {})
phase_records = runtime_checks.get("runtime_phase_records", scaffold_report.get("phase_records", {}))
terminal_slice_observed = (
    "exit_teardown_pre" in phase_records and "exit_teardown_post" in phase_records
)
runtime_temporal_invariants = runtime_checks.get("runtime_temporal_invariants", {})
terminal_global_lower_half_cleared = bool(
    runtime_temporal_invariants.get("exit_post_global_lower_half_cleared", False)
)
runtime_pid_source = "selftest_ok_marker" if selftest_exit_pid is not None else "terminal_slice_auto"
debt_state = scaffold_report.get("policy", {}).get("state")
all_runtime_pids = runtime_checks.get("runtime_all_pids", [])
terminal_runtime_pids = runtime_checks.get("runtime_terminal_pids", [])
unexpected_terminal_pids = [
    pid for pid in terminal_runtime_pids if selftest_exit_pid is not None and pid != selftest_exit_pid
]
nonselected_runtime_pids = [
    pid for pid in all_runtime_pids if selected_runtime_pid is not None and pid != selected_runtime_pid
]

if selected_runtime_pid is not None and selftest_exit_pid is not None and selected_runtime_pid != selftest_exit_pid:
    ordered_violations.append(
        f"selected_runtime_pid_mismatch:selftest_exit_pid={selftest_exit_pid}:selected={selected_runtime_pid}"
    )
if unexpected_terminal_pids:
    ordered_violations.append(
        "unexpected_terminal_runtime_pids:" + ",".join(str(pid) for pid in unexpected_terminal_pids)
    )
if terminal_slice_observed and not terminal_global_lower_half_cleared:
    ordered_violations.append("terminal_global_lower_half_not_cleared")

row = {
    "gate": "low-half-kheap-exit-proof",
    "verdict": "PASS" if not ordered_violations else "FAIL",
    "violations_count": len(ordered_violations),
    "violations": ordered_violations,
    "boot_audit_exit_code": boot_audit_rc,
    "qemu_timeout_seconds": qemu_timeout,
    "boot_audit_marker": boot_audit_marker,
    "selftest_armed_marker": selftest_armed_marker,
    "selftest_fail_marker": selftest_fail_marker,
    "selftest_armed_present": bool(selftest_armed_present),
    "selftest_fail_present": bool(selftest_fail_present),
    "selftest_ok_present": bool(selftest_ok_present),
    "selftest_ok_marker_count": selftest_ok_count,
    "selftest_ok_marker_parse_ok": bool(selftest_ok_parse_ok),
    "selftest_exit_pid": selftest_exit_pid,
    "selftest_return_pid": selftest_return_pid,
    "scaffold_gate_exit_code": scaffold_rc,
    "runtime_terminal_slice_required": True,
    "selected_runtime_pid": selected_runtime_pid,
    "selected_runtime_pid_source": runtime_pid_source,
    "terminal_slice_observed": terminal_slice_observed,
    "terminal_global_lower_half_cleared": terminal_global_lower_half_cleared,
    "all_runtime_pids": all_runtime_pids,
    "terminal_runtime_pids": terminal_runtime_pids,
    "nonselected_runtime_pids": nonselected_runtime_pids,
    "unexpected_terminal_pids": unexpected_terminal_pids,
    "marker_uniqueness_contract": {
        "strict_determinism": True,
        "required_ok_marker_count": 1,
        "observed_ok_marker_count": selftest_ok_count,
    },
    "debt_state": debt_state,
    "debt_removed": debt_state == "DEBT_REMOVED",
    "phase_records": phase_records,
    "scaffold_report_path": str(scaffold_report_path),
}

with report_path.open("w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
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
  echo "build_rc=${BUILD_RC}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
  echo "scaffold_rc=${SCAFFOLD_RC}"
  echo "selftest_armed_present=${SELFTEST_ARMED_PRESENT}"
  echo "selftest_fail_present=${SELFTEST_FAIL_PRESENT}"
  echo "selftest_ok_present=${SELFTEST_OK_PRESENT}"
  echo "selftest_ok_count=${SELFTEST_OK_COUNT}"
  echo "selftest_ok_parse_ok=${SELFTEST_OK_PARSE_OK}"
  echo "selftest_exit_pid=${SELFTEST_EXIT_PID}"
  echo "selftest_return_pid=${SELFTEST_RETURN_PID}"
} > "${META_TXT}"

if python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
with open(sys.argv[1], "r", encoding="utf-8") as fh:
    report = json.load(fh)
raise SystemExit(0 if report.get("verdict") == "PASS" else 1)
PY
then
  echo "low-half-kheap-exit-proof: PASS"
  exit 0
fi

echo "low-half-kheap-exit-proof: FAIL"
exit 2
