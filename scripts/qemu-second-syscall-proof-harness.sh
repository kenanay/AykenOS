#!/usr/bin/env bash
#
# AykenOS Second Syscall Proof Harness
#
# Purpose: Run second_syscall_proof test to verify boundary_init_done flag behavior
#
# Expected outcome:
#   PASS: 1st syscall takes init path, 2nd syscall takes skip path
#   FAIL: Flag is broken (init path runs on every syscall)
#
# This harness is part of scheduler-primary-regression-rca investigation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$REPO_ROOT/out/second-syscall-evidence"

echo "========================================"
echo "Second Syscall Proof Harness"
echo "========================================"
echo ""

# Create output directory
mkdir -p "$OUT_DIR"

# Step 1: Build kernel + userspace with second-syscall-proof mode
echo "Step 1: Building kernel with second-syscall-proof userspace..."
cd "$REPO_ROOT"

# Build with second-syscall-proof mode
# Use validation profile to ensure diagnostic markers are present
USER_MINIMAL_MODE=second-syscall-proof \
KERNEL_PROFILE=validation \
AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=1 \
AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1 \
AYKEN_RING3_FETCH_PROBE=0 \
make efi-img

# Step 2: Run QEMU with debugcon capture
echo ""
echo "Step 2: Running QEMU with debugcon capture..."

# Prepare OVMF paths
OVMF_CODE="${OVMF_CODE:-$REPO_ROOT/firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS="$OUT_DIR/OVMF_VARS.fd"
cp -f "$REPO_ROOT/firmware/ovmf/OVMF_VARS.fd" "$OVMF_VARS"

# Run QEMU with timeout (test should complete quickly)
# Increased timeout to 60s to allow for full boot + userspace execution
set +e
timeout 60s qemu-system-x86_64 \
    -machine q35 \
    -m 256M \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS" \
    -drive format=raw,file="$REPO_ROOT/out/build/EFI.img" \
    -debugcon file:"$OUT_DIR/debugcon.log" \
    -global isa-debugcon.iobase=0xE9 \
    -serial file:"$OUT_DIR/serial.log" \
    -display none \
    -no-reboot \
    -no-shutdown
QEMU_EXIT=$?
set -e

if [ $QEMU_EXIT -ne 0 ] && [ $QEMU_EXIT -ne 124 ]; then
    echo "WARNING: QEMU exited with code $QEMU_EXIT (expected for test harness)"
fi

# Step 3: Analyze debugcon log
echo ""
echo "Step 3: Analyzing debugcon log for evidence..."

if [ ! -f "$OUT_DIR/debugcon.log" ]; then
    echo "ERROR: debugcon.log not found"
    echo "       QEMU may not have run successfully"
    exit 2
fi

# Run analysis script
python3 "$SCRIPT_DIR/ci/analyze_second_syscall_evidence.py" "$OUT_DIR/debugcon.log"
EXIT_CODE=$?

# Step 4: Report results
echo ""
echo "========================================"
echo "Harness Complete"
echo "========================================"
echo ""
echo "Evidence artifacts:"
echo "  - Debugcon log: $OUT_DIR/debugcon.log"
echo "  - Serial log:   $OUT_DIR/serial.log"
echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ PASS: boundary_init_done flag works correctly"
    echo "   → Init path runs on 1st syscall only"
    echo "   → Skip path runs on subsequent syscalls"
    echo ""
    echo "Next steps:"
    echo "  1. Flag behavior confirmed; first-syscall init is not repeated"
    echo "  2. Proceed to Task 2: Write preservation tests"
    echo "  3. Then Task 3: Optimize init path (move to kernel boot)"
elif [ $EXIT_CODE -eq 1 ]; then
    echo "❌ FAIL: Flag persistence failure observed"
    echo "   → Init path repeats across observed syscalls"
    echo "   → This may explain part of the regression"
    echo ""
    echo "Next steps:"
    echo "  1. Investigate flag scope and lifetime"
    echo "  2. Consider moving init to kernel boot"
    echo "  3. Re-run this harness to verify fix"
else
    echo "⚠ INCONCLUSIVE: Insufficient evidence"
    echo "   → Check that kernel has DIAG_* instrumentation"
    echo "   → Check that test payload ran successfully"
    echo "   → Review debugcon.log manually"
fi

exit $EXIT_CODE
