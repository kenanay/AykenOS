#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_alias_proof.sh \
    --evidence-dir evidence/run-<id>/gates/alias-proof \
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
AYKEN_VALIDATION="${AYKEN_VALIDATION:-1}"
AYKEN_ALIAS_PROOF_SELFTEST="${AYKEN_ALIAS_PROOF_SELFTEST:-1}"
EXPECTED_USER_MINIMAL_MODE="phase10a2"
OBSERVED_USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"
BOOT_OK_MARKER="[[AYKEN_BOOT_OK]]"
ALIAS_PROOF_MARKER="[[AYKEN_ALIAS_PROOF_OK]]"
ALIAS_PROOF_ARMED_MARKER="[[AYKEN_ALIAS_PROOF_ARMED]]"
ALIAS_PROOF_FAIL_MARKER="[[AYKEN_ALIAS_PROOF_FAIL]]"

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
  echo "ERROR: alias-proof requires USER_MINIMAL_MODE=${EXPECTED_USER_MINIMAL_MODE} (current=${OBSERVED_USER_MINIMAL_MODE:-unset})" >&2
  exit 3
fi
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: alias-proof requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 3
fi
if [[ "${AYKEN_CR3_PCID}" != "0" ]]; then
  echo "ERROR: alias-proof requires AYKEN_CR3_PCID=0 (current=${AYKEN_CR3_PCID})" >&2
  exit 3
fi
if [[ "${AYKEN_MB_SELFTEST}" != "1" ]]; then
  echo "ERROR: alias-proof requires AYKEN_MB_SELFTEST=1 (current=${AYKEN_MB_SELFTEST})" >&2
  exit 3
fi
if [[ "${AYKEN_GATE4_POLICY_TEST}" != "0" ]]; then
  echo "ERROR: alias-proof requires AYKEN_GATE4_POLICY_TEST=0 (current=${AYKEN_GATE4_POLICY_TEST})" >&2
  exit 3
fi
if [[ "${AYKEN_VALIDATION}" != "1" ]]; then
  echo "ERROR: alias-proof requires AYKEN_VALIDATION=1 (current=${AYKEN_VALIDATION})" >&2
  exit 3
fi
if [[ "${AYKEN_ALIAS_PROOF_SELFTEST}" != "1" ]]; then
  echo "ERROR: alias-proof requires AYKEN_ALIAS_PROOF_SELFTEST=1 (current=${AYKEN_ALIAS_PROOF_SELFTEST})" >&2
  exit 3
fi

for tool in bash make python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
ALIAS_AUDIT="${ROOT}/tools/validation/alias_proof_audit.sh"
if [[ ! -x "${BOOT_AUDIT}" ]]; then
  echo "ERROR: missing boot audit script: ${BOOT_AUDIT}" >&2
  exit 1
fi
if [[ ! -x "${ALIAS_AUDIT}" ]]; then
  echo "ERROR: missing alias audit script: ${ALIAS_AUDIT}" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
BOOT_AUDIT_LOG="${EVIDENCE_DIR}/boot_audit.log"
BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
ALIAS_AUDIT_LOG="${EVIDENCE_DIR}/alias_audit.log"
COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
MARKER_LOG="${EVIDENCE_DIR}/marker.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${ALIAS_AUDIT_LOG}"
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
  AYKEN_VALIDATION="${AYKEN_VALIDATION}" \
  AYKEN_ALIAS_PROOF_SELFTEST="${AYKEN_ALIAS_PROOF_SELFTEST}" \
  clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" \
  AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST}" \
  AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST}" \
  AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}" \
  AYKEN_VALIDATION="${AYKEN_VALIDATION}" \
  AYKEN_ALIAS_PROOF_SELFTEST="${AYKEN_ALIAS_PROOF_SELFTEST}" \
  guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
row = {
    "gate": "alias-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["build_failed"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "alias-proof: INFRA FAIL (build_failed)"
  exit 1
fi

# Step 1: Boot witness check
set +e
"${BOOT_AUDIT}" \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "${BOOT_OK_MARKER}" \
  --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_AUDIT_LOG}" 2>&1
BOOT_AUDIT_RC=$?
set -e

if ! wait_for_marker_log; then
  python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
row = {
    "gate": "alias-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["marker_log_empty"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "marker_log_empty" > "${VIOLATIONS_TXT}"
  echo "alias-proof: FAIL (marker_log_empty)"
  exit 2
fi

# Check boot witness
BOOT_OK_PRESENT=0
if grep -q -F "${BOOT_OK_MARKER}" "${MARKER_LOG}"; then
  BOOT_OK_PRESENT=1
fi

if [[ "${BOOT_OK_PRESENT}" -eq 0 ]]; then
  python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
row = {
    "gate": "alias-proof",
    "verdict": "FAIL",
    "violations_count": 1,
    "violations": ["missing_boot_ok_witness"],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "missing_boot_ok_witness" > "${VIOLATIONS_TXT}"
  echo "alias-proof: FAIL (missing_boot_ok_witness)"
  exit 2
fi

# Step 2: Alias proof audit
ALIAS_PROOF_ARMED_PRESENT=0
ALIAS_PROOF_FAIL_PRESENT=0
ALIAS_PROOF_OK_PRESENT=0
ALIAS_PROOF_OK_COUNT=0

if grep -q -F "${ALIAS_PROOF_ARMED_MARKER}" "${MARKER_LOG}"; then
  ALIAS_PROOF_ARMED_PRESENT=1
fi
if grep -q -F "${ALIAS_PROOF_FAIL_MARKER}" "${MARKER_LOG}"; then
  ALIAS_PROOF_FAIL_PRESENT=1
fi
ALIAS_PROOF_OK_COUNT="$(grep -c -F "${ALIAS_PROOF_MARKER}" "${MARKER_LOG}" || true)"
if [[ "${ALIAS_PROOF_OK_COUNT}" -gt 0 ]]; then
  ALIAS_PROOF_OK_PRESENT=1
fi

# Run alias audit script
set +e
"${ALIAS_AUDIT}" "${MARKER_LOG}" "${EVIDENCE_DIR}/alias_audit_report.json" > "${ALIAS_AUDIT_LOG}" 2>&1
ALIAS_AUDIT_RC=$?
set -e

python3 - \
  "${REPORT_JSON}" \
  "${EVIDENCE_DIR}/alias_audit_report.json" \
  "${BOOT_AUDIT_RC}" \
  "${ALIAS_AUDIT_RC}" \
  "${QEMU_TIMEOUT}" \
  "${BOOT_OK_PRESENT}" \
  "${ALIAS_PROOF_ARMED_PRESENT}" \
  "${ALIAS_PROOF_FAIL_PRESENT}" \
  "${ALIAS_PROOF_OK_PRESENT}" \
  "${ALIAS_PROOF_OK_COUNT}" \
  "${BOOT_OK_MARKER}" \
  "${ALIAS_PROOF_MARKER}" \
  "${ALIAS_PROOF_ARMED_MARKER}" \
  "${ALIAS_PROOF_FAIL_MARKER}" <<'PY'
import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
alias_audit_report_path = Path(sys.argv[2])
boot_audit_rc = int(sys.argv[3])
alias_audit_rc = int(sys.argv[4])
qemu_timeout = int(sys.argv[5])
boot_ok_present = int(sys.argv[6])
alias_proof_armed_present = int(sys.argv[7])
alias_proof_fail_present = int(sys.argv[8])
alias_proof_ok_present = int(sys.argv[9])
alias_proof_ok_count = int(sys.argv[10])
boot_ok_marker = sys.argv[11]
alias_proof_marker = sys.argv[12]
alias_proof_armed_marker = sys.argv[13]
alias_proof_fail_marker = sys.argv[14]

violations: list[str] = []
alias_audit_report = {}
if alias_audit_report_path.is_file():
    with alias_audit_report_path.open("r", encoding="utf-8") as fh:
        alias_audit_report = json.load(fh)
else:
    violations.append("missing_alias_audit_report")

if boot_audit_rc != 0:
    violations.append(f"boot_audit_failed:rc={boot_audit_rc}")
if boot_ok_present == 0:
    violations.append("missing_boot_ok_witness")
if alias_proof_armed_present == 0:
    violations.append("missing_alias_proof_armed_marker")
if alias_proof_fail_present != 0:
    violations.append("alias_proof_fail_marker_present")
if alias_proof_ok_present == 0:
    violations.append("missing_alias_proof_ok_marker")
if alias_proof_ok_count != 1:
    violations.append(f"unexpected_alias_proof_ok_marker_count:{alias_proof_ok_count}")
if alias_audit_rc != 0:
    violations.append(f"alias_audit_failed:rc={alias_audit_rc}")
violations.extend(alias_audit_report.get("violations", []))

seen = set()
ordered_violations = []
for item in violations:
    if item in seen:
        continue
    seen.add(item)
    ordered_violations.append(item)

row = {
    "gate": "alias-proof",
    "verdict": "PASS" if not ordered_violations else "FAIL",
    "violations_count": len(ordered_violations),
    "violations": ordered_violations,
    "boot_audit_exit_code": boot_audit_rc,
    "alias_audit_exit_code": alias_audit_rc,
    "qemu_timeout_seconds": qemu_timeout,
    "boot_ok_marker": boot_ok_marker,
    "boot_ok_present": bool(boot_ok_present),
    "alias_proof_marker": alias_proof_marker,
    "alias_proof_armed_marker": alias_proof_armed_marker,
    "alias_proof_fail_marker": alias_proof_fail_marker,
    "alias_proof_armed_present": bool(alias_proof_armed_present),
    "alias_proof_fail_present": bool(alias_proof_fail_present),
    "alias_proof_ok_present": bool(alias_proof_ok_present),
    "alias_proof_ok_marker_count": alias_proof_ok_count,
    "marker_uniqueness_contract": {
        "strict_determinism": True,
        "required_ok_marker_count": 1,
        "observed_ok_marker_count": alias_proof_ok_count,
    },
    "alias_audit_report": alias_audit_report,
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
  echo "boot_ok_marker=${BOOT_OK_MARKER}"
  echo "alias_proof_marker=${ALIAS_PROOF_MARKER}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "user_minimal_mode=${OBSERVED_USER_MINIMAL_MODE}"
  echo "ayken_cr3_pcid=${AYKEN_CR3_PCID}"
  echo "ayken_mb_selftest=${AYKEN_MB_SELFTEST}"
  echo "ayken_gate4_policy_test=${AYKEN_GATE4_POLICY_TEST}"
  echo "ayken_sched_bootstrap_policy=${AYKEN_SCHED_BOOTSTRAP_POLICY}"
  echo "ayken_validation=${AYKEN_VALIDATION}"
  echo "ayken_alias_proof_selftest=${AYKEN_ALIAS_PROOF_SELFTEST}"
  echo "build_rc=${BUILD_RC}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
  echo "alias_audit_rc=${ALIAS_AUDIT_RC}"
  echo "boot_ok_present=${BOOT_OK_PRESENT}"
  echo "alias_proof_armed_present=${ALIAS_PROOF_ARMED_PRESENT}"
  echo "alias_proof_fail_present=${ALIAS_PROOF_FAIL_PRESENT}"
  echo "alias_proof_ok_present=${ALIAS_PROOF_OK_PRESENT}"
  echo "alias_proof_ok_count=${ALIAS_PROOF_OK_COUNT}"
} > "${META_TXT}"

if python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
with open(sys.argv[1], "r", encoding="utf-8") as fh:
    report = json.load(fh)
raise SystemExit(0 if report.get("verdict") == "PASS" else 1)
PY
then
  echo "alias-proof: PASS"
  exit 0
fi

echo "alias-proof: FAIL"
exit 2
