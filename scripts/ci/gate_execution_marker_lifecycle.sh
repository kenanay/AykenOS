#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_execution_marker_lifecycle.sh \
    --evidence-dir evidence/run-<id>/gates/execution-marker-lifecycle \
    [--qemu-timeout <sec>]

Exit codes:
  0: pass
  1: tooling/build infrastructure failure
  2: lifecycle evidence contract failure
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${EXECUTION_MARKER_LIFECYCLE_QEMU_TIMEOUT:-30}"

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

for tool in make python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
VALIDATOR="${ROOT}/tools/ci/validate_execution_marker_lifecycle.py"
BOOT_AUDIT_MARKER="[[AYKEN_EXECUTION_MARKER_LIFECYCLE_OK]]"

if [[ ! -x "${BOOT_AUDIT}" || ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: lifecycle evidence dependency missing" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"
BUILD_LOG="${EVIDENCE_DIR}/build.log"
BOOT_AUDIT_LOG="${EVIDENCE_DIR}/boot_audit.log"
BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
MARKER_LOG="${EVIDENCE_DIR}/marker.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${MARKER_LOG}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

set +e
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID=0 \
  AYKEN_MB_SELFTEST=0 \
  AYKEN_GATE4_POLICY_TEST=0 \
  AYKEN_SCHED_BOOTSTRAP_POLICY=0 \
  AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
  AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST=1 \
  clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID=0 \
  AYKEN_MB_SELFTEST=0 \
  AYKEN_GATE4_POLICY_TEST=0 \
  AYKEN_SCHED_BOOTSTRAP_POLICY=0 \
  AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
  AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST=1 \
  guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  printf '%s\n' '{"gate":"execution-marker-lifecycle","verdict":"FAIL","violations_count":1,"violations":["build_failed"]}' > "${REPORT_JSON}"
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "execution-marker-lifecycle: INFRA FAIL (build_failed)"
  exit 1
fi

set +e
"${BOOT_AUDIT}" \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "${BOOT_AUDIT_MARKER}" \
  --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_AUDIT_LOG}" 2>&1
BOOT_AUDIT_RC=$?
set -e

if [[ -s "${BOOT_AUDIT_DIR}/qemu_debugcon.log" ]]; then
  cp -f "${BOOT_AUDIT_DIR}/qemu_debugcon.log" "${MARKER_LOG}"
fi

set +e
python3 "${VALIDATOR}" \
  --log "${MARKER_LOG}" \
  --out "${REPORT_JSON}" \
  --violations-out "${VIOLATIONS_TXT}" \
  --boot-audit-exit-code "${BOOT_AUDIT_RC}" \
  --qemu-timeout "${QEMU_TIMEOUT}"
VALIDATOR_RC=$?
set -e

{
  echo "gate=execution-marker-lifecycle"
  echo "scope=validation_only_marker_enabled_execution_slot"
  echo "qemu_timeout_seconds=${QEMU_TIMEOUT}"
  echo "boot_audit_exit_code=${BOOT_AUDIT_RC}"
  echo "ayken_execution_marker_validation_enable=1"
  echo "ayken_execution_marker_lifecycle_selftest=1"
  echo "production_default_off=true"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  echo "execution-marker-lifecycle: FAIL (see ${REPORT_JSON})"
  exit 2
fi

echo "execution-marker-lifecycle: PASS"
