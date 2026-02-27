#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_ring3_execution_phase10a2.sh --evidence-dir evidence/run-<id>/gates/ring3-execution-phase10a2 [--qemu-timeout <sec>]

Exit codes:
  0: pass
  1: tooling/infra failure
  2: marker contract failure
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${QEMU_TIMEOUT:-25}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
AYKEN_CR3_PCID="${AYKEN_CR3_PCID:-0}"

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
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 2
fi
if [[ "${AYKEN_CR3_PCID}" != "0" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_CR3_PCID=0 (current=${AYKEN_CR3_PCID})" >&2
  exit 2
fi

for tool in python3 make; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
EXTRACTOR="${ROOT}/tools/ci/extract_phase10_markers.py"
VALIDATOR="${ROOT}/tools/ci/validate_marker_order_phase10a2.py"

if [[ ! -x "${BOOT_AUDIT}" ]]; then
  echo "ERROR: missing boot audit script: ${BOOT_AUDIT}" >&2
  exit 1
fi
if [[ ! -f "${EXTRACTOR}" ]]; then
  echo "ERROR: missing extractor script: ${EXTRACTOR}" >&2
  exit 1
fi
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator script: ${VALIDATOR}" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
BOOT_AUDIT_LOG="${EVIDENCE_DIR}/boot_audit.log"
BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
MARKER_LOG="${EVIDENCE_DIR}/marker.log"
EVENTS_JSONL="${EVIDENCE_DIR}/events.jsonl"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${COMBINED_LOG}"
: > "${MARKER_LOG}"
: > "${EVENTS_JSONL}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

set +e
make -C "${ROOT}" KERNEL_PROFILE=validation AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" guard-context-offsets efi-img > "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e
if [[ "${BUILD_RC}" -ne 0 ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["build_failed"]
}
EOF
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: INFRA FAIL (build_failed)"
  exit 1
fi

set +e
"${BOOT_AUDIT}" \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "[[AYKEN_BOOT_OK]]" \
  --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_AUDIT_LOG}" 2>&1
BOOT_AUDIT_RC=$?
set -e

cat "${BOOT_AUDIT_DIR}/qemu_serial.log" "${BOOT_AUDIT_DIR}/qemu_debugcon.log" 2>/dev/null > "${COMBINED_LOG}" || true
if [[ -s "${BOOT_AUDIT_DIR}/qemu_debugcon.log" ]]; then
  cp -f "${BOOT_AUDIT_DIR}/qemu_debugcon.log" "${MARKER_LOG}"
else
  cp -f "${COMBINED_LOG}" "${MARKER_LOG}"
fi

if [[ "${BOOT_AUDIT_RC}" -ne 0 ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["boot_audit_failed:rc=${BOOT_AUDIT_RC}"],
  "boot_audit_exit_code": ${BOOT_AUDIT_RC},
  "qemu_timeout_seconds": ${QEMU_TIMEOUT}
}
EOF
  echo "boot_audit_failed:rc=${BOOT_AUDIT_RC}" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: INFRA FAIL (boot_audit_failed rc=${BOOT_AUDIT_RC})"
  exit 1
fi

if [[ ! -s "${MARKER_LOG}" ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["marker_log_empty"]
}
EOF
  echo "marker_log_empty" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: FAIL (marker_log_empty)"
  exit 2
fi

if ! python3 "${EXTRACTOR}" --log "${MARKER_LOG}" --out "${EVENTS_JSONL}"; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["extract_markers_failed"]
}
EOF
  echo "extract_markers_failed" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: INFRA FAIL (extract_markers_failed)"
  exit 1
fi

set +e
python3 "${VALIDATOR}" --events "${EVENTS_JSONL}" --log "${MARKER_LOG}" --out "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

python3 - "${REPORT_JSON}" "${BOOT_AUDIT_RC}" "${QEMU_TIMEOUT}" <<'PY'
import json
import sys
path = sys.argv[1]
boot_audit_rc = int(sys.argv[2])
qemu_timeout = int(sys.argv[3])
with open(path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["boot_audit_exit_code"] = boot_audit_rc
row["qemu_timeout_seconds"] = qemu_timeout
with open(path, "w", encoding="utf-8") as fh:
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
violations = report.get("violations", [])
with open(violations_path, "w", encoding="utf-8") as fh:
    for item in violations:
        fh.write(f"{item}\n")
PY

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "ayken_cr3_pcid=${AYKEN_CR3_PCID}"
  echo "build_rc=${BUILD_RC}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "ring3-execution-phase10a2: FAIL (${COUNT} violations)"
  exit 2
fi

echo "ring3-execution-phase10a2: PASS"
exit 0
