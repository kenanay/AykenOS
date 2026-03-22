#!/usr/bin/env bash
# CI Gate: Alias-Aware Address Space Leak Proof (Phase 11 v1)
# Authority: Phase 11 Memory Model Verification
# Purpose: Validate alias proof witness in validation boot and produce audit evidence

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_alias_proof.sh \
    --evidence-dir evidence/run-<id>/gates/alias-proof \
    [--qemu-timeout <sec>]

Exit codes:
  0: pass (alias proof validated)
  1: tooling/infra failure
  2: proof contract failure (leak detected or witness missing)
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${QEMU_TIMEOUT:-35}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
AYKEN_VALIDATION="${AYKEN_VALIDATION:-1}"
AYKEN_ALIAS_PROOF_SELFTEST="${AYKEN_ALIAS_PROOF_SELFTEST:-1}"
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

# Validate environment
if [[ "${OBSERVED_USER_MINIMAL_MODE}" != "${EXPECTED_USER_MINIMAL_MODE}" ]]; then
  echo "ERROR: alias-proof requires USER_MINIMAL_MODE=${EXPECTED_USER_MINIMAL_MODE} (current=${OBSERVED_USER_MINIMAL_MODE:-unset})" >&2
  exit 3
fi

if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: alias-proof requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
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

# Check required tools
for tool in bash make qemu-system-x86_64; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

AUDIT_SCRIPT="${ROOT}/tools/validation/alias_proof_audit.sh"
if [[ ! -x "${AUDIT_SCRIPT}" ]]; then
  echo "ERROR: audit script not found or not executable: ${AUDIT_SCRIPT}" >&2
  exit 1
fi

# Create evidence directories
mkdir -p "${EVIDENCE_DIR}"
mkdir -p "${EVIDENCE_DIR}/logs"

BOOT_LOG="${EVIDENCE_DIR}/boot.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"

echo "== CI GATE ALIAS PROOF =="
echo "evidence_dir: ${EVIDENCE_DIR}"
echo "qemu_timeout: ${QEMU_TIMEOUT}"
echo "kernel_profile: ${KERNEL_PROFILE}"
echo "ayken_validation: ${AYKEN_VALIDATION}"
echo "ayken_alias_proof_selftest: ${AYKEN_ALIAS_PROOF_SELFTEST}"
echo "user_minimal_mode: ${OBSERVED_USER_MINIMAL_MODE}"
echo ""

# Build kernel with validation profile
echo "Building kernel with KERNEL_PROFILE=validation AYKEN_VALIDATION=1 AYKEN_ALIAS_PROOF_SELFTEST=1..."
cd "${ROOT}"
make clean-kernel > "${EVIDENCE_DIR}/logs/clean.log" 2>&1 || true
make KERNEL_PROFILE=validation \
     AYKEN_VALIDATION=1 \
     AYKEN_ALIAS_PROOF_SELFTEST=1 \
     kernel > "${EVIDENCE_DIR}/logs/build.log" 2>&1

if [[ $? -ne 0 ]]; then
  echo "ERROR: kernel build failed" >&2
  cat "${EVIDENCE_DIR}/logs/build.log" >&2
  exit 1
fi

echo "Kernel build successful"

# Build EFI image
echo "Building EFI image..."
make efi-img > "${EVIDENCE_DIR}/logs/efi-img.log" 2>&1
if [[ $? -ne 0 ]]; then
  echo "ERROR: EFI image build failed" >&2
  cat "${EVIDENCE_DIR}/logs/efi-img.log" >&2
  exit 1
fi

echo "EFI image build successful"

# Run QEMU boot with validation profile
echo "Running QEMU boot (timeout: ${QEMU_TIMEOUT}s)..."

# Detect timeout command
if command -v timeout >/dev/null 2>&1; then
  TIMEOUT_CMD="timeout"
elif command -v gtimeout >/dev/null 2>&1; then
  TIMEOUT_CMD="gtimeout"
else
  echo "ERROR: no timeout command found (timeout or gtimeout)" >&2
  exit 1
fi

# Run QEMU with debugcon output to boot.log
EFI_IMAGE="${ROOT}/EFI.img"
DEBUGCON_LOG="${EVIDENCE_DIR}/debugcon.log"
SERIAL_LOG="${EVIDENCE_DIR}/serial.log"

"${TIMEOUT_CMD}" "${QEMU_TIMEOUT}" qemu-system-x86_64 \
  -drive "format=raw,file=${EFI_IMAGE}" \
  -display none \
  -no-reboot \
  -no-shutdown \
  -serial "file:${SERIAL_LOG}" \
  -debugcon "file:${DEBUGCON_LOG}" \
  -global isa-debugcon.iobase=0xe9 \
  > "${EVIDENCE_DIR}/logs/qemu.log" 2>&1 || true

# Copy debugcon output to boot.log for audit
cp "${DEBUGCON_LOG}" "${BOOT_LOG}"

echo "QEMU boot completed"
echo ""

# Run audit script
echo "Running alias proof audit..."
"${AUDIT_SCRIPT}" "${BOOT_LOG}" "${REPORT_JSON}"
AUDIT_EXIT=$?

if [[ ${AUDIT_EXIT} -eq 0 ]]; then
  echo ""
  echo "== ALIAS PROOF GATE: PASS =="
  echo "All checks passed - alias proof validated"
  echo "Evidence: ${EVIDENCE_DIR}"
  exit 0
else
  echo ""
  echo "== ALIAS PROOF GATE: FAIL =="
  echo "Audit failed with exit code: ${AUDIT_EXIT}"
  if [[ -f "${VIOLATIONS_TXT}" ]]; then
    echo ""
    echo "Violations:"
    cat "${VIOLATIONS_TXT}"
  fi
  echo ""
  echo "Evidence: ${EVIDENCE_DIR}"
  exit 2
fi
