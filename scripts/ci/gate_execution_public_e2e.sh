#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY
# Attribution is script metadata only and has no runtime authority.
#
# Validation-only public Ring3 submit/wait evidence. Completion uses the
# deterministic BCIB stub and is not proof of a real BCIB interpreter.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_execution_public_e2e.sh \
    --evidence-dir evidence/run-<id>/gates/execution-public-e2e \
    [--qemu-timeout <sec>]

Exit codes:
  0: pass
  1: tooling/build infrastructure failure
  2: public Ring3 evidence contract failure
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${EXECUTION_PUBLIC_E2E_QEMU_TIMEOUT:-35}"

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
VALIDATOR="${ROOT}/tools/ci/validate_execution_public_e2e.py"
# The Ring3 payload emits [U][SYSCALL_OK] only after validating the mapped
# result. The existing debug_putchar tracker synthesizes this atomic marker.
BOOT_AUDIT_MARKER="[[AYKEN_SYSCALL_V2_OK]]"

if [[ ! -x "${BOOT_AUDIT}" || ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: public E2E evidence dependency missing" >&2
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

BUILD_ARGS=(
  -C "${ROOT}"
  KERNEL_PROFILE=validation
  USER_MINIMAL_MODE=bcib-public-e2e
  AYKEN_CR3_PCID=0
  AYKEN_RING3_ENTRY_GUARD=1
  AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0
  AYKEN_DEBUG_SCHED=0
  AYKEN_DEBUG_IRQ=0
  AYKEN_MB_SELFTEST=0
  AYKEN_GATE4_POLICY_TEST=0
  AYKEN_SCHED_BOOTSTRAP_POLICY=0
  AYKEN_BCIB_STUB_RESULT_ENABLE=1
  AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1
  AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1
)

set +e
make "${BUILD_ARGS[@]}" clean > "${BUILD_LOG}" 2>&1 || true
make "${BUILD_ARGS[@]}" guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  printf '%s\n' '{"gate":"execution-public-e2e","verdict":"FAIL","violations_count":1,"violations":["build_failed"]}' > "${REPORT_JSON}"
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "execution-public-e2e: INFRA FAIL (build_failed)"
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
  echo "gate=execution-public-e2e"
  echo "scope=validation_only_public_ring3_submit_wait_result_publication"
  echo "qemu_timeout_seconds=${QEMU_TIMEOUT}"
  echo "boot_audit_exit_code=${BOOT_AUDIT_RC}"
  echo "user_minimal_mode=bcib-public-e2e"
  echo "ayken_bcib_stub_result_enable=1"
  echo "ayken_bcib_public_e2e_selftest=1"
  echo "ayken_execution_marker_validation_enable=1"
  echo "ayken_ring3_entry_guard=1"
  echo "ayken_ring3_mask_irq0_first_entry=0"
  echo "ayken_debug_sched=0"
  echo "ayken_debug_irq=0"
  echo "production_default_off=true"
  echo "does_not_prove=real_bcib_worker_completion,phase17_closure,race_isolation,performance_acceptance"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  echo "execution-public-e2e: FAIL (see ${REPORT_JSON})"
  exit 2
fi

echo "execution-public-e2e: PASS"
