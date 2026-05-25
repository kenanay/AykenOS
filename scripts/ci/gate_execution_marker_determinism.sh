#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_execution_marker_determinism.sh \
    --evidence-dir evidence/run-<id>/gates/execution-marker-determinism \
    [--qemu-timeout <sec>]

Exit codes:
  0: pass
  1: tooling/build infrastructure failure
  2: evidence contract failure
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${EXECUTION_MARKER_DETERMINISM_QEMU_TIMEOUT:-30}"

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
VALIDATOR="${ROOT}/tools/ci/validate_execution_marker_determinism.py"
POSITIVE_MARKER="[[AYKEN_EXECUTION_MARKER_LIFECYCLE_OK]]"
NEGATIVE_MARKER="[[AYKEN_EXECUTION_MARKER_NEGATIVE_OK]]"

if [[ ! -x "${BOOT_AUDIT}" || ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: determinism evidence dependency missing" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"
BUILD_POSITIVE_LOG="${EVIDENCE_DIR}/build-positive.log"
BUILD_NEGATIVE_LOG="${EVIDENCE_DIR}/build-negative.log"
RUN_A_DIR="${EVIDENCE_DIR}/run-a"
RUN_B_DIR="${EVIDENCE_DIR}/run-b"
NEGATIVE_DIR="${EVIDENCE_DIR}/negative-invalid-order"
RUN_A_LOG="${EVIDENCE_DIR}/run-a.marker.log"
RUN_B_LOG="${EVIDENCE_DIR}/run-b.marker.log"
NEGATIVE_LOG="${EVIDENCE_DIR}/negative-invalid-order.marker.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

POSITIVE_ARGS=(
  KERNEL_PROFILE=validation
  USER_MINIMAL_MODE=phase10a2
  AYKEN_CR3_PCID=0
  AYKEN_MB_SELFTEST=0
  AYKEN_GATE4_POLICY_TEST=0
  AYKEN_SCHED_BOOTSTRAP_POLICY=0
  AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1
  AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST=1
)
NEGATIVE_ARGS=(
  "${POSITIVE_ARGS[@]}"
  AYKEN_PHASE17_MARKER_INJECTION_TEST=1
  AYKEN_MARKER_INJECT_INVALID_ORDER=1
  AYKEN_EXECUTION_MARKER_NEGATIVE_EXPECT_REJECT=1
)

write_build_failure() {
  local build_name="$1"
  printf '{"gate":"execution-marker-determinism","verdict":"FAIL","violations_count":1,"violations":["build_failed:%s"]}\n' \
    "${build_name}" > "${REPORT_JSON}"
  printf 'build_failed:%s\n' "${build_name}" > "${VIOLATIONS_TXT}"
}

if ! make -C "${ROOT}" "${POSITIVE_ARGS[@]}" clean > "${BUILD_POSITIVE_LOG}" 2>&1 ||
   ! make -C "${ROOT}" "${POSITIVE_ARGS[@]}" guard-context-offsets efi-img >> "${BUILD_POSITIVE_LOG}" 2>&1; then
  write_build_failure "positive"
  echo "execution-marker-determinism: INFRA FAIL (positive build)"
  exit 1
fi

set +e
"${BOOT_AUDIT}" --timeout "${QEMU_TIMEOUT}" --marker "${POSITIVE_MARKER}" --out-dir "${RUN_A_DIR}" > "${EVIDENCE_DIR}/run-a.boot-audit.log" 2>&1
RUN_A_RC=$?
"${BOOT_AUDIT}" --timeout "${QEMU_TIMEOUT}" --marker "${POSITIVE_MARKER}" --out-dir "${RUN_B_DIR}" > "${EVIDENCE_DIR}/run-b.boot-audit.log" 2>&1
RUN_B_RC=$?
set -e

if [[ -s "${RUN_A_DIR}/qemu_debugcon.log" ]]; then
  cp -f "${RUN_A_DIR}/qemu_debugcon.log" "${RUN_A_LOG}"
fi
if [[ -s "${RUN_B_DIR}/qemu_debugcon.log" ]]; then
  cp -f "${RUN_B_DIR}/qemu_debugcon.log" "${RUN_B_LOG}"
fi

if ! make -C "${ROOT}" "${NEGATIVE_ARGS[@]}" clean > "${BUILD_NEGATIVE_LOG}" 2>&1 ||
   ! make -C "${ROOT}" "${NEGATIVE_ARGS[@]}" guard-context-offsets efi-img >> "${BUILD_NEGATIVE_LOG}" 2>&1; then
  write_build_failure "negative-invalid-order"
  echo "execution-marker-determinism: INFRA FAIL (negative build)"
  exit 1
fi

set +e
"${BOOT_AUDIT}" --timeout "${QEMU_TIMEOUT}" --marker "${NEGATIVE_MARKER}" --out-dir "${NEGATIVE_DIR}" > "${EVIDENCE_DIR}/negative-invalid-order.boot-audit.log" 2>&1
NEGATIVE_RC=$?
set -e

if [[ -s "${NEGATIVE_DIR}/qemu_debugcon.log" ]]; then
  cp -f "${NEGATIVE_DIR}/qemu_debugcon.log" "${NEGATIVE_LOG}"
fi

set +e
python3 "${VALIDATOR}" \
  --run-a-log "${RUN_A_LOG}" \
  --run-b-log "${RUN_B_LOG}" \
  --negative-log "${NEGATIVE_LOG}" \
  --run-a-exit-code "${RUN_A_RC}" \
  --run-b-exit-code "${RUN_B_RC}" \
  --negative-exit-code "${NEGATIVE_RC}" \
  --qemu-timeout "${QEMU_TIMEOUT}" \
  --out "${REPORT_JSON}" \
  --violations-out "${VIOLATIONS_TXT}"
VALIDATOR_RC=$?
set -e

{
  echo "gate=execution-marker-determinism"
  echo "scope=validation_only_marker_enabled_execution_slot"
  echo "positive_boot_count=2"
  echo "negative_case=invalid_order"
  echo "qemu_timeout_seconds=${QEMU_TIMEOUT}"
  echo "production_default_off=true"
  echo "does_not_prove=ring3_public_syscall_end_to_end,race_isolation,performance_acceptance,phase17_closure"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  echo "execution-marker-determinism: FAIL (see ${REPORT_JSON})"
  exit 2
fi

echo "execution-marker-determinism: PASS"
