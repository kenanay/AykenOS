#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_bcib_forbidden_proof.sh --evidence-dir evidence/run-<id>/gates/bcib-forbidden-proof [--qemu-timeout <sec>]

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
AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY:-1}"
EXPECTED_USER_MINIMAL_MODE="bcib-forbidden"
OBSERVED_USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"

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
  echo "FATAL: bcib-forbidden-proof gate invoked with USER_MINIMAL_MODE=${OBSERVED_USER_MINIMAL_MODE:-unset} (expected=${EXPECTED_USER_MINIMAL_MODE})" >&2
  exit 2
fi
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: bcib-forbidden-proof requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 2
fi

for tool in python3 make; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
VALIDATOR="${ROOT}/scripts/validate_fail_closed_markers.py"
# Stop after the forbidden test completes (or kills)
BOOT_AUDIT_MARKER="BCIB_FORBIDDEN"

if [[ ! -x "${BOOT_AUDIT}" ]]; then
  echo "ERROR: missing boot audit script: ${BOOT_AUDIT}" >&2
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
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
DEBUGCON_LOG="${BOOT_AUDIT_DIR}/qemu_debugcon.log"
SERIAL_LOG="${BOOT_AUDIT_DIR}/qemu_serial.log"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${COMBINED_LOG}"
: > "${MARKER_LOG}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

refresh_marker_log() {
  cat "${SERIAL_LOG}" "${DEBUGCON_LOG}" 2>/dev/null > "${COMBINED_LOG}" || true
  if [[ -s "${DEBUGCON_LOG}" ]]; then
    cp -f "${DEBUGCON_LOG}" "${MARKER_LOG}"
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
make -C "${ROOT}" KERNEL_PROFILE=validation USER_MINIMAL_MODE="${EXPECTED_USER_MINIMAL_MODE}" AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" KERNEL_PROFILE=validation USER_MINIMAL_MODE="${EXPECTED_USER_MINIMAL_MODE}" AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "bcib-forbidden-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["build_failed"]
}
EOF
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "bcib-forbidden-proof: INFRA FAIL (build_failed)"
  exit 1
fi

# Verify build manifest (AUTHORITY)
MANIFEST="${ROOT}/out/build/payload_manifest.json"
if [[ ! -f "${MANIFEST}" ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "bcib-forbidden-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["manifest_missing"]
}
EOF
  echo "manifest_missing" > "${VIOLATIONS_TXT}"
  echo "bcib-forbidden-proof: FAIL (manifest_missing)"
  exit 2
fi

MANIFEST_MODE=$(python3 -c "import json; print(json.load(open('${MANIFEST}'))['selected_mode'])" 2>/dev/null || echo "")
if [[ "${MANIFEST_MODE}" != "${EXPECTED_USER_MINIMAL_MODE}" ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "bcib-forbidden-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["manifest_mode_mismatch:expected=${EXPECTED_USER_MINIMAL_MODE}:actual=${MANIFEST_MODE}"]
}
EOF
  echo "manifest_mode_mismatch:expected=${EXPECTED_USER_MINIMAL_MODE}:actual=${MANIFEST_MODE}" > "${VIOLATIONS_TXT}"
  echo "bcib-forbidden-proof: FAIL (manifest_mode_mismatch)"
  exit 2
fi

set +e
"${BOOT_AUDIT}" \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "${BOOT_AUDIT_MARKER}" \
  --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_AUDIT_LOG}" 2>&1
BOOT_AUDIT_RC=$?
set -e

if ! wait_for_marker_log; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "bcib-forbidden-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["authoritative_debugcon_missing"]
}
EOF
  echo "authoritative_debugcon_missing" > "${VIOLATIONS_TXT}"
  echo "bcib-forbidden-proof: FAIL (authoritative_debugcon_missing)"
  exit 2
fi

# Verify boot marker (AUTHORITY)
if ! grep -q '\[K\]\[PAYLOAD_MODE=bcib-forbidden\]' "${MARKER_LOG}"; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "bcib-forbidden-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["boot_marker_missing"]
}
EOF
  echo "boot_marker_missing" > "${VIOLATIONS_TXT}"
  echo "bcib-forbidden-proof: FAIL (boot_marker_missing)"
  exit 2
fi

# Run fail-closed validator
set +e
python3 "${VALIDATOR}" "${MARKER_LOG}"
VALIDATOR_RC=$?
set -e

# Generate report metadata
python3 - "${REPORT_JSON}" "${BOOT_AUDIT_RC}" "${QEMU_TIMEOUT}" "${BUILD_LOG}" "${BOOT_AUDIT_LOG}" "${DEBUGCON_LOG}" "${SERIAL_LOG}" "${MARKER_LOG}" <<'PY'
import json
import hashlib
from pathlib import Path
import sys

def file_meta(path_str: str) -> dict:
    path = Path(path_str)
    if not path.is_file():
        return {
            "path": str(path),
            "exists": False,
            "bytes": 0,
            "sha256": "",
            "mtime_epoch": 0,
        }
    return {
        "path": str(path),
        "exists": True,
        "bytes": path.stat().st_size,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "mtime_epoch": int(path.stat().st_mtime),
    }

path = sys.argv[1]
boot_audit_rc = int(sys.argv[2])
qemu_timeout = int(sys.argv[3])

# Load existing report from validator or create new
if Path(path).exists():
    with open(path, "r", encoding="utf-8") as fh:
        row = json.load(fh)
else:
    row = {
        "gate": "bcib-forbidden-proof",
        "verdict": "UNKNOWN",
        "violations": [],
        "violations_count": 0
    }

row["boot_audit_exit_code"] = boot_audit_rc
row["qemu_timeout_seconds"] = qemu_timeout
row["boot_audit_marker"] = "BCIB_FORBIDDEN"
row["kernel_profile"] = "validation"
row["user_minimal_mode"] = "bcib-forbidden"
row["ayken_ring3_mask_irq0_first_entry"] = 1
row["authoritative_marker_source"] = "qemu_debugcon.log"
row["same_run_evidence"] = {
    "cross_run_stitching_allowed": False,
    "marker_log_source": "debugcon_only",
}
row["evidence_files"] = {
    "build_log": file_meta(sys.argv[4]),
    "boot_audit_log": file_meta(sys.argv[5]),
    "qemu_debugcon_log": file_meta(sys.argv[6]),
    "qemu_serial_log": file_meta(sys.argv[7]),
    "marker_log": file_meta(sys.argv[8]),
}

with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

# Add boot audit failure to violations if needed
python3 - "${REPORT_JSON}" "${BOOT_AUDIT_RC}" <<'PY'
import json
import sys

path = sys.argv[1]
boot_audit_rc = int(sys.argv[2])
with open(path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
if boot_audit_rc != 0:
    violations = list(row.get("violations", []))
    tag = f"boot_audit_failed:rc={boot_audit_rc}"
    if tag not in violations:
        violations.append(tag)
    row["verdict"] = "FAIL"
    row["violations"] = violations
    row["violations_count"] = len(violations)
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

# Extract violations to text file
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
  echo "boot_audit_marker=${BOOT_AUDIT_MARKER}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "user_minimal_mode=${OBSERVED_USER_MINIMAL_MODE}"
  echo "ayken_ring3_mask_irq0_first_entry=${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}"
  echo "authoritative_marker_source=qemu_debugcon.log"
  echo "build_rc=${BUILD_RC}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 || "${BOOT_AUDIT_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "bcib-forbidden-proof: FAIL (${COUNT} violations)"
  exit 2
fi

echo "bcib-forbidden-proof: PASS"
exit 0
