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
AYKEN_C2_STRICT_MARKERS="${AYKEN_C2_STRICT_MARKERS:-0}"
AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST:-1}"
AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST:-0}"
AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY:-0}"
AYKEN_RING3_FETCH_PROBE="${AYKEN_RING3_FETCH_PROBE:-0}"
AYKEN_RING3_SECOND_CANONICAL_PROBE="${AYKEN_RING3_SECOND_CANONICAL_PROBE:-0}"
AYKEN_RING3_FRESH_FRAME_PROBE="${AYKEN_RING3_FRESH_FRAME_PROBE:-0}"
AYKEN_RING3_IRETQ_DIAG_PROBE="${AYKEN_RING3_IRETQ_DIAG_PROBE:-0}"
AYKEN_SHARE_KERNEL_UPPER_HALF="${AYKEN_SHARE_KERNEL_UPPER_HALF:-0}"
AYKEN_RING3_LOW_FETCH_STUB="${AYKEN_RING3_LOW_FETCH_STUB:-0}"
AYKEN_RING3_CANONICAL_FETCH_STUB="${AYKEN_RING3_CANONICAL_FETCH_STUB:-0}"
AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY:-0}"
ENFORCED_AYKEN_CR3_PCID="0"
ENFORCED_AYKEN_RING3_FETCH_PROBE="0"
ENFORCED_AYKEN_RING3_SECOND_CANONICAL_PROBE="0"
ENFORCED_AYKEN_RING3_FRESH_FRAME_PROBE="0"
ENFORCED_AYKEN_RING3_IRETQ_DIAG_PROBE="0"
ENFORCED_AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="1"
EXPECTED_USER_MINIMAL_MODE="phase10a2"
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
  echo "FATAL: ring3-execution-phase10a2 gate invoked with USER_MINIMAL_MODE=${OBSERVED_USER_MINIMAL_MODE:-unset} (expected=${EXPECTED_USER_MINIMAL_MODE})" >&2
  exit 2
fi
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 2
fi
if ! [[ "${AYKEN_CR3_PCID}" =~ ^[01]$ ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_CR3_PCID in {0,1} (current=${AYKEN_CR3_PCID})" >&2
  exit 2
fi
if ! [[ "${AYKEN_C2_STRICT_MARKERS}" =~ ^[01]$ ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_C2_STRICT_MARKERS in {0,1} (current=${AYKEN_C2_STRICT_MARKERS})" >&2
  exit 2
fi
if ! [[ "${AYKEN_MB_SELFTEST}" =~ ^[01]$ ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_MB_SELFTEST in {0,1} (current=${AYKEN_MB_SELFTEST})" >&2
  exit 2
fi
if ! [[ "${AYKEN_GATE4_POLICY_TEST}" =~ ^[01]$ ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_GATE4_POLICY_TEST in {0,1} (current=${AYKEN_GATE4_POLICY_TEST})" >&2
  exit 2
fi
if ! [[ "${AYKEN_SCHED_BOOTSTRAP_POLICY}" =~ ^[01]$ ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_SCHED_BOOTSTRAP_POLICY in {0,1} (current=${AYKEN_SCHED_BOOTSTRAP_POLICY})" >&2
  exit 2
fi
if ! [[ "${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" =~ ^[01]$ ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY in {0,1} (current=${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY})" >&2
  exit 2
fi
for flag_name in AYKEN_RING3_FETCH_PROBE AYKEN_RING3_SECOND_CANONICAL_PROBE AYKEN_RING3_FRESH_FRAME_PROBE AYKEN_RING3_IRETQ_DIAG_PROBE AYKEN_SHARE_KERNEL_UPPER_HALF AYKEN_RING3_LOW_FETCH_STUB AYKEN_RING3_CANONICAL_FETCH_STUB AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY; do
  flag_value="${!flag_name}"
  if ! [[ "${flag_value}" =~ ^[01]$ ]]; then
    echo "ERROR: ring3-execution-phase10a2 requires ${flag_name} in {0,1} (current=${flag_value})" >&2
    exit 2
  fi
done
if [[ "${AYKEN_CR3_PCID}" != "${ENFORCED_AYKEN_CR3_PCID}" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_CR3_PCID=${ENFORCED_AYKEN_CR3_PCID} (current=${AYKEN_CR3_PCID})" >&2
  exit 2
fi
if [[ "${AYKEN_RING3_FETCH_PROBE}" != "${ENFORCED_AYKEN_RING3_FETCH_PROBE}" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_RING3_FETCH_PROBE=${ENFORCED_AYKEN_RING3_FETCH_PROBE} (current=${AYKEN_RING3_FETCH_PROBE})" >&2
  exit 2
fi
if [[ "${AYKEN_RING3_SECOND_CANONICAL_PROBE}" != "${ENFORCED_AYKEN_RING3_SECOND_CANONICAL_PROBE}" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_RING3_SECOND_CANONICAL_PROBE=${ENFORCED_AYKEN_RING3_SECOND_CANONICAL_PROBE} (current=${AYKEN_RING3_SECOND_CANONICAL_PROBE})" >&2
  exit 2
fi
if [[ "${AYKEN_RING3_FRESH_FRAME_PROBE}" != "${ENFORCED_AYKEN_RING3_FRESH_FRAME_PROBE}" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_RING3_FRESH_FRAME_PROBE=${ENFORCED_AYKEN_RING3_FRESH_FRAME_PROBE} (current=${AYKEN_RING3_FRESH_FRAME_PROBE})" >&2
  exit 2
fi
if [[ "${AYKEN_RING3_IRETQ_DIAG_PROBE}" != "${ENFORCED_AYKEN_RING3_IRETQ_DIAG_PROBE}" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_RING3_IRETQ_DIAG_PROBE=${ENFORCED_AYKEN_RING3_IRETQ_DIAG_PROBE} (current=${AYKEN_RING3_IRETQ_DIAG_PROBE})" >&2
  exit 2
fi
if [[ "${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" != "${ENFORCED_AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" ]]; then
  echo "ERROR: ring3-execution-phase10a2 requires AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=${ENFORCED_AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY} (current=${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY})" >&2
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
# Stop only after the final canonical A2 marker; earlier boot markers can cut
# the syscall/capability/user-return evidence and create false instability.
BOOT_AUDIT_MARKER="P10_RING3_USER_CODE"

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
DEBUGCON_LOG="${BOOT_AUDIT_DIR}/qemu_debugcon.log"
SERIAL_LOG="${BOOT_AUDIT_DIR}/qemu_serial.log"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${COMBINED_LOG}"
: > "${MARKER_LOG}"
: > "${EVENTS_JSONL}"
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
make -C "${ROOT}" KERNEL_PROFILE=validation AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" AYKEN_C2_STRICT_MARKERS="${AYKEN_C2_STRICT_MARKERS}" AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST}" AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST}" AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}" AYKEN_RING3_FETCH_PROBE="${AYKEN_RING3_FETCH_PROBE}" AYKEN_RING3_SECOND_CANONICAL_PROBE="${AYKEN_RING3_SECOND_CANONICAL_PROBE}" AYKEN_RING3_FRESH_FRAME_PROBE="${AYKEN_RING3_FRESH_FRAME_PROBE}" AYKEN_RING3_IRETQ_DIAG_PROBE="${AYKEN_RING3_IRETQ_DIAG_PROBE}" AYKEN_SHARE_KERNEL_UPPER_HALF="${AYKEN_SHARE_KERNEL_UPPER_HALF}" AYKEN_RING3_LOW_FETCH_STUB="${AYKEN_RING3_LOW_FETCH_STUB}" AYKEN_RING3_CANONICAL_FETCH_STUB="${AYKEN_RING3_CANONICAL_FETCH_STUB}" AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" KERNEL_PROFILE=validation AYKEN_CR3_PCID="${AYKEN_CR3_PCID}" AYKEN_C2_STRICT_MARKERS="${AYKEN_C2_STRICT_MARKERS}" AYKEN_MB_SELFTEST="${AYKEN_MB_SELFTEST}" AYKEN_GATE4_POLICY_TEST="${AYKEN_GATE4_POLICY_TEST}" AYKEN_SCHED_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}" AYKEN_RING3_FETCH_PROBE="${AYKEN_RING3_FETCH_PROBE}" AYKEN_RING3_SECOND_CANONICAL_PROBE="${AYKEN_RING3_SECOND_CANONICAL_PROBE}" AYKEN_RING3_FRESH_FRAME_PROBE="${AYKEN_RING3_FRESH_FRAME_PROBE}" AYKEN_RING3_IRETQ_DIAG_PROBE="${AYKEN_RING3_IRETQ_DIAG_PROBE}" AYKEN_SHARE_KERNEL_UPPER_HALF="${AYKEN_SHARE_KERNEL_UPPER_HALF}" AYKEN_RING3_LOW_FETCH_STUB="${AYKEN_RING3_LOW_FETCH_STUB}" AYKEN_RING3_CANONICAL_FETCH_STUB="${AYKEN_RING3_CANONICAL_FETCH_STUB}" AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e
if [[ "${BUILD_RC}" -ne 0 ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "enforced_ayken_cr3_pcid": ${ENFORCED_AYKEN_CR3_PCID},
  "observed_ayken_cr3_pcid": ${AYKEN_CR3_PCID},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["build_failed"]
}
EOF
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: INFRA FAIL (build_failed)"
  exit 1
fi

# Verify build manifest (AUTHORITY)
MANIFEST="${ROOT}/out/build/payload_manifest.json"
if [[ ! -f "${MANIFEST}" ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "enforced_ayken_cr3_pcid": ${ENFORCED_AYKEN_CR3_PCID},
  "observed_ayken_cr3_pcid": ${AYKEN_CR3_PCID},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["manifest_missing"]
}
EOF
  echo "manifest_missing" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: FAIL (manifest_missing)"
  exit 2
fi

MANIFEST_MODE=$(python3 -c "import json; print(json.load(open('${MANIFEST}'))['selected_mode'])" 2>/dev/null || echo "")
if [[ "${MANIFEST_MODE}" != "${EXPECTED_USER_MINIMAL_MODE}" ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "enforced_ayken_cr3_pcid": ${ENFORCED_AYKEN_CR3_PCID},
  "observed_ayken_cr3_pcid": ${AYKEN_CR3_PCID},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["manifest_mode_mismatch:expected=${EXPECTED_USER_MINIMAL_MODE}:actual=${MANIFEST_MODE}"]
}
EOF
  echo "manifest_mode_mismatch:expected=${EXPECTED_USER_MINIMAL_MODE}:actual=${MANIFEST_MODE}" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: FAIL (manifest_mode_mismatch)"
  exit 2
fi

# Verify build log (DIAGNOSTIC - WARNING only)
if grep -q "DAYKEN_USER_MINIMAL_MODE_STRING=\"${EXPECTED_USER_MINIMAL_MODE}\"" "${BUILD_LOG}"; then
  echo "Build log mode string verification: PASS (diagnostic)" >> "${BUILD_LOG}"
else
  echo "WARNING: Build log does not show DAYKEN_USER_MINIMAL_MODE_STRING=\"${EXPECTED_USER_MINIMAL_MODE}\" (diagnostic only)" >> "${BUILD_LOG}"
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
  "gate": "ring3-execution-phase10a2",
  "enforced_ayken_cr3_pcid": ${ENFORCED_AYKEN_CR3_PCID},
  "observed_ayken_cr3_pcid": ${AYKEN_CR3_PCID},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["authoritative_debugcon_missing"]
}
EOF
  echo "authoritative_debugcon_missing" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: FAIL (authoritative_debugcon_missing)"
  exit 2
fi

# Verify boot marker (AUTHORITY)
if ! grep -q '\[K\]\[PAYLOAD_MODE=phase10a2\]' "${MARKER_LOG}"; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "enforced_ayken_cr3_pcid": ${ENFORCED_AYKEN_CR3_PCID},
  "observed_ayken_cr3_pcid": ${AYKEN_CR3_PCID},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["boot_marker_missing"]
}
EOF
  echo "boot_marker_missing" > "${VIOLATIONS_TXT}"
  echo "ring3-execution-phase10a2: FAIL (boot_marker_missing)"
  exit 2
fi

if ! python3 "${EXTRACTOR}" --log "${MARKER_LOG}" --out "${EVENTS_JSONL}"; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "ring3-execution-phase10a2",
  "enforced_ayken_cr3_pcid": ${ENFORCED_AYKEN_CR3_PCID},
  "observed_ayken_cr3_pcid": ${AYKEN_CR3_PCID},
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

python3 - "${REPORT_JSON}" "${BOOT_AUDIT_RC}" "${QEMU_TIMEOUT}" "${ENFORCED_AYKEN_CR3_PCID}" "${AYKEN_CR3_PCID}" "${AYKEN_C2_STRICT_MARKERS}" "${AYKEN_MB_SELFTEST}" "${AYKEN_GATE4_POLICY_TEST}" "${AYKEN_SCHED_BOOTSTRAP_POLICY}" "${AYKEN_RING3_FETCH_PROBE}" "${AYKEN_RING3_SECOND_CANONICAL_PROBE}" "${AYKEN_RING3_FRESH_FRAME_PROBE}" "${AYKEN_RING3_IRETQ_DIAG_PROBE}" "${AYKEN_SHARE_KERNEL_UPPER_HALF}" "${AYKEN_RING3_LOW_FETCH_STUB}" "${AYKEN_RING3_CANONICAL_FETCH_STUB}" "${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}" "${BUILD_LOG}" "${BOOT_AUDIT_LOG}" "${DEBUGCON_LOG}" "${SERIAL_LOG}" "${MARKER_LOG}" "${EVENTS_JSONL}" <<'PY'
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
enforced_ayken_cr3_pcid = int(sys.argv[4])
observed_ayken_cr3_pcid = int(sys.argv[5])
with open(path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["boot_audit_exit_code"] = boot_audit_rc
row["qemu_timeout_seconds"] = qemu_timeout
row["boot_audit_marker"] = "P10_RING3_USER_CODE"
row["enforced_ayken_cr3_pcid"] = enforced_ayken_cr3_pcid
row["observed_ayken_cr3_pcid"] = observed_ayken_cr3_pcid
row["ayken_c2_strict_markers"] = int(sys.argv[6])
row["ayken_mb_selftest"] = int(sys.argv[7])
row["ayken_gate4_policy_test"] = int(sys.argv[8])
row["ayken_sched_bootstrap_policy"] = int(sys.argv[9])
row["ayken_ring3_fetch_probe"] = int(sys.argv[10])
row["ayken_ring3_second_canonical_probe"] = int(sys.argv[11])
row["ayken_ring3_fresh_frame_probe"] = int(sys.argv[12])
row["ayken_ring3_iretq_diag_probe"] = int(sys.argv[13])
row["ayken_share_kernel_upper_half"] = int(sys.argv[14])
row["ayken_ring3_low_fetch_stub"] = int(sys.argv[15])
row["ayken_ring3_canonical_fetch_stub"] = int(sys.argv[16])
row["ayken_ring3_mask_irq0_first_entry"] = int(sys.argv[17])
row["canonical_probe_free"] = (
    row["ayken_ring3_fetch_probe"] == 0
    and row["ayken_ring3_second_canonical_probe"] == 0
    and row["ayken_ring3_fresh_frame_probe"] == 0
    and row["ayken_ring3_iretq_diag_probe"] == 0
    and row["ayken_ring3_canonical_fetch_stub"] == 0
)
row["authoritative_marker_source"] = "qemu_debugcon.log"
row["same_run_evidence"] = {
    "cross_run_stitching_allowed": False,
    "marker_log_source": "debugcon_only",
}
row["evidence_files"] = {
    "build_log": file_meta(sys.argv[18]),
    "boot_audit_log": file_meta(sys.argv[19]),
    "qemu_debugcon_log": file_meta(sys.argv[20]),
    "qemu_serial_log": file_meta(sys.argv[21]),
    "marker_log": file_meta(sys.argv[22]),
    "events_jsonl": file_meta(sys.argv[23]),
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

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
  echo "enforced_ayken_cr3_pcid=${ENFORCED_AYKEN_CR3_PCID}"
  echo "observed_ayken_cr3_pcid=${AYKEN_CR3_PCID}"
  echo "ayken_cr3_pcid=${AYKEN_CR3_PCID}"
  echo "ayken_c2_strict_markers=${AYKEN_C2_STRICT_MARKERS}"
  echo "ayken_mb_selftest=${AYKEN_MB_SELFTEST}"
  echo "ayken_gate4_policy_test=${AYKEN_GATE4_POLICY_TEST}"
  echo "ayken_sched_bootstrap_policy=${AYKEN_SCHED_BOOTSTRAP_POLICY}"
  echo "ayken_ring3_fetch_probe=${AYKEN_RING3_FETCH_PROBE}"
  echo "ayken_ring3_second_canonical_probe=${AYKEN_RING3_SECOND_CANONICAL_PROBE}"
  echo "ayken_ring3_fresh_frame_probe=${AYKEN_RING3_FRESH_FRAME_PROBE}"
  echo "ayken_ring3_iretq_diag_probe=${AYKEN_RING3_IRETQ_DIAG_PROBE}"
  echo "ayken_share_kernel_upper_half=${AYKEN_SHARE_KERNEL_UPPER_HALF}"
  echo "ayken_ring3_low_fetch_stub=${AYKEN_RING3_LOW_FETCH_STUB}"
  echo "ayken_ring3_canonical_fetch_stub=${AYKEN_RING3_CANONICAL_FETCH_STUB}"
  echo "ayken_ring3_mask_irq0_first_entry=${AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY}"
  echo "authoritative_marker_source=qemu_debugcon.log"
  echo "user_minimal_mode=${OBSERVED_USER_MINIMAL_MODE}"
  echo "build_rc=${BUILD_RC}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 || "${BOOT_AUDIT_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "ring3-execution-phase10a2: FAIL (${COUNT} violations)"
  exit 2
fi

echo "ring3-execution-phase10a2: PASS"
exit 0
