#!/bin/bash
# Test UEFI Print (ground truth) to determine if bootloader executes

set -e

QEMU_OUTPUT="out/logs/qemu_console.log"
mkdir -p out/logs

echo "=== Testing UEFI Print (Ground Truth) ==="
echo "Capturing QEMU console output..."
echo ""

# Run QEMU and capture ALL output (stdout + stderr)
timeout 3s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -nographic \
    -no-reboot \
    > "$QEMU_OUTPUT" 2>&1 || true

echo "=== QEMU Console Output (last 100 lines) ==="
tail -100 "$QEMU_OUTPUT"
echo ""
echo "=== Ground Truth Test Results ==="

# Check for UEFI Print marker
if grep -q "\[UEFI_BOOT_OK\]" "$QEMU_OUTPUT"; then
    echo "✓ UEFI Print FOUND: [UEFI_BOOT_OK]"
    echo "✓ Bootloader IS executing"
    echo ""
    echo "This means:"
    echo "  - Bootloader code runs successfully"
    echo "  - Problem is in capture path (debugcon/serial)"
    echo "  - NOT a bootloader execution issue"
else
    echo "✗ UEFI Print NOT FOUND"
    echo "✗ Bootloader may not be executing"
    echo ""
    echo "This means:"
    echo "  - Bootloader handoff may be failing"
    echo "  - OR UEFI Print not reaching console"
fi

echo ""
echo "=== Checking for AykenOS markers ==="
if grep -i "ayken" "$QEMU_OUTPUT"; then
    echo "✓ Found AykenOS-related output"
else
    echo "✗ No AykenOS output found"
fi
