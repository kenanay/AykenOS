#!/bin/bash
# Test script for VCP Diagnostic Evidence Stubs (Task 5)
#
# This script builds the validation kernel, boots it in QEMU, and treats the
# late-init PASS marker as the integration proof for diagnostic-only evidence.

set -euo pipefail

QEMU_TIMEOUT_SECONDS="${QEMU_TIMEOUT_SECONDS:-45}"
LOG_PATH="out/logs/debug_run.log"

find_timeout_bin() {
    if [ -n "${TIMEOUT_BIN:-}" ]; then
        printf '%s\n' "$TIMEOUT_BIN"
        return 0
    fi

    if command -v timeout >/dev/null 2>&1; then
        command -v timeout
        return 0
    fi

    if command -v gtimeout >/dev/null 2>&1; then
        command -v gtimeout
        return 0
    fi

    if [ -x /opt/homebrew/bin/timeout ]; then
        printf '%s\n' /opt/homebrew/bin/timeout
        return 0
    fi

    return 1
}

log_has() {
    pattern="$1"
    if command -v rg >/dev/null 2>&1; then
        rg -q "$pattern" "$LOG_PATH"
    else
        grep -Eq "$pattern" "$LOG_PATH"
    fi
}

show_log_matches() {
    pattern="$1"
    if command -v rg >/dev/null 2>&1; then
        rg -n "$pattern" "$LOG_PATH" || true
    else
        grep -En "$pattern" "$LOG_PATH" || true
    fi
}

echo "=========================================="
echo "VCP Diagnostic Evidence Integration Runner"
echo "=========================================="
echo ""
echo "Building kernel with VCP diagnostic evidence property tests enabled..."
echo ""

./scripts/check_vcp_runtime_contract.sh

timeout_bin="$(find_timeout_bin)" || {
    echo "ERROR: timeout command not found. Install coreutils or set TIMEOUT_BIN." >&2
    exit 1
}

mkdir -p out/logs
: > "$LOG_PATH"

make clean > /dev/null 2>&1 || true
make kernel.elf \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    AYKEN_VCP_EVIDENCE_TEST=1 \
    AYKEN_VCP_FAIL_CLOSED_TEST=0 \
    AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
    AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
    AYKEN_MB_SELFTEST=0 \
    AYKEN_GATE4_POLICY_TEST=0

echo ""
echo "Build complete. Booting QEMU validation profile..."
echo ""

set +e
echo "fs0:" > /tmp/ayken_boot_commands.txt
echo "\\EFI\\BOOT\\BOOTX64.EFI" >> /tmp/ayken_boot_commands.txt
"$timeout_bin" "$QEMU_TIMEOUT_SECONDS" make run \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    AYKEN_VCP_EVIDENCE_TEST=1 \
    AYKEN_VCP_FAIL_CLOSED_TEST=0 \
    AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
    AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
    AYKEN_MB_SELFTEST=0 \
    AYKEN_GATE4_POLICY_TEST=0 < /tmp/ayken_boot_commands.txt
run_status=$?
set -e

echo ""
echo "QEMU exit status: $run_status"
if [ "$run_status" -ne 0 ] && [ "$run_status" -ne 124 ]; then
    echo "ERROR: QEMU run failed before validation result could be trusted." >&2
    show_log_matches "VCP_EVIDENCE_TESTS|CRITICAL FAILURE|\\[FAIL\\]"
    exit 1
fi

if log_has "VCP_EVIDENCE_TESTS FAILED|CRITICAL FAILURE|\\[FAIL\\] VCP diagnostic evidence"; then
    echo "ERROR: VCP diagnostic evidence integration failed." >&2
    show_log_matches "VCP_EVIDENCE_TESTS|CRITICAL FAILURE|\\[FAIL\\]"
    exit 1
fi

if ! log_has "VCP_EVIDENCE_TESTS PASSED"; then
    echo "ERROR: VCP diagnostic evidence PASS marker was not found in $LOG_PATH." >&2
    show_log_matches "VCP_EVIDENCE_TESTS|LATE_INIT|BOOT_VALIDATION"
    exit 1
fi

for marker in \
    "\\[VCP_EVIDENCE\\]\\[VALIDATION_CHECK\\]" \
    "\\[VCP_EVIDENCE\\]\\[CONTRACT_EXECUTION\\]" \
    "\\[VCP_EVIDENCE\\]\\[BOUNDARY_CROSSING\\]" \
    "\\[VCP_FAIL_CLOSED\\]\\[BLOCK\\]" \
    "\\[VCP_EVIDENCE\\]\\[COMPREHENSIVE\\]" \
    "\\[VCP_EVIDENCE\\]\\[FAIL_CLOSED_COMPLETE\\]"
do
    if ! log_has "$marker"; then
        echo "ERROR: expected diagnostic evidence marker not found: $marker" >&2
        show_log_matches "VCP_EVIDENCE|VCP_FAIL_CLOSED|VCP_EVIDENCE_TESTS|CRITICAL FAILURE|\\[FAIL\\]"
        exit 1
    fi
done

echo "Boot evidence:"
show_log_matches "VCP_EVIDENCE_TESTS PASSED|VCP_EVIDENCE|VCP_FAIL_CLOSED"
echo ""
echo "PASS: VCP diagnostic evidence integration test passed."
echo ""
