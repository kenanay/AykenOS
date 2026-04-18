#!/usr/bin/env bash
#
# AykenOS Ring3 Spin Probe Harness
#
# Purpose: Verify Ring3 fetch/execute pipeline works
#
# Expected outcome:
#   PASS: [R3_FETCH_OK] marker appears in log
#   FAIL: No marker (RIP/CR3/mapping/NX issue)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$REPO_ROOT/out/ring3-spin-probe"

echo "========================================"
echo "Ring3 Spin Probe Harness"
echo "========================================"
echo ""

# Create output directory
mkdir -p "$OUT_DIR"

# Step 1: Build kernel with ring3-spin-probe mode
echo "Step 1: Building kernel with ring3-spin-probe userspace..."
cd "$REPO_ROOT"

USER_MINIMAL_MODE=ring3-spin-probe \
KERNEL_PROFILE=validation \
AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0 \
AYKEN_RING3_FETCH_PROBE=0 \
make efi-img

# Step 2: Run QEMU with debugcon capture (30 second timeout for full boot)
echo ""
echo "Step 2: Running QEMU with debugcon capture..."

OVMF_CODE="${OVMF_CODE:-$REPO_ROOT/firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS="$OUT_DIR/OVMF_VARS.fd"
cp -f "$REPO_ROOT/firmware/ovmf/OVMF_VARS.fd" "$OVMF_VARS"

set +e
timeout 30s qemu-system-x86_64 \
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

# Step 3: Analyze log
echo ""
echo "Step 3: Analyzing debugcon log..."

if [ ! -f "$OUT_DIR/debugcon.log" ]; then
    echo "ERROR: debugcon.log not found"
    exit 2
fi

# Check for Ring3 execution marker
if grep -q "\[R3_FETCH_OK\]" "$OUT_DIR/debugcon.log"; then
    echo ""
    echo "========================================"
    echo "✅ PASS: Ring3 Execution Verified"
    echo "========================================"
    echo ""
    grep "\[R3_FETCH_OK\]" "$OUT_DIR/debugcon.log"
    echo ""
    echo "Ring3 fetch/execute pipeline works correctly."
    echo "Userspace code is executing."
    echo ""
    echo "Next: Proceed to int3 probe (exception path test)"
    exit 0
else
    echo ""
    echo "========================================"
    echo "❌ FAIL: Ring3 transition observed, but post-entry timer-driven observability absent."
    echo "========================================"

    echo ""
    echo "No [R3_FETCH_OK] marker found in log."
    echo ""
    echo "Possible causes:"
    echo "  1. RIP incorrect (ELF entry point or mapping issue)"
    echo "  2. CR3 mapping doesn't include user text"
    echo "  3. NX bit set on user text page"
    echo "  4. User page not executable"
    echo ""
    echo "Debug artifacts:"
    echo "  - Debugcon log: $OUT_DIR/debugcon.log"
    echo "  - Serial log:   $OUT_DIR/serial.log"
    echo ""
    echo "Check for P10_RING3_ENTER marker to confirm Ring3 transition."
    echo "If present, issue is in userspace execution, not Ring3 entry."
    exit 1
fi
