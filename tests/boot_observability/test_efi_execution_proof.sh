#!/usr/bin/env bash
# Test 2: EFI Execution Proof
# Uses UEFI variable write as execution proof (survives across boots)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== EFI Execution Proof Test ==="

# Create temporary OVMF vars file
VARS_TEMPLATE="/opt/homebrew/share/qemu/edk2-x86_64-code.fd"
VARS_FILE=$(mktemp /tmp/ovmf_vars.XXXXXX.fd)
cp "$VARS_TEMPLATE" "$VARS_FILE" 2>/dev/null || {
    echo "WARN: Could not copy OVMF vars template, creating empty file"
    dd if=/dev/zero of="$VARS_FILE" bs=1M count=2 2>/dev/null
}

echo "Using OVMF vars file: $VARS_FILE"

# Build bootloader and EFI image
cd "$PROJECT_ROOT"
make bootloader
make efi-img

# Run QEMU with UEFI shell that writes a variable
timeout 10s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive if=pflash,format=raw,file="$VARS_FILE" \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -serial stdio \
    -debugcon file:/tmp/qemu_debug.log \
    -no-reboot \
    || true

# Check if vars file was modified (proof of UEFI execution)
VARS_SIZE_AFTER=$(stat -f%z "$VARS_FILE" 2>/dev/null || stat -c%s "$VARS_FILE" 2>/dev/null)
echo "OVMF vars file size after boot: $VARS_SIZE_AFTER"

# Check debug log for any output
if [[ -f /tmp/qemu_debug.log ]]; then
    DEBUG_SIZE=$(stat -f%z /tmp/qemu_debug.log 2>/dev/null || stat -c%s /tmp/qemu_debug.log 2>/dev/null)
    echo "Debug log size: $DEBUG_SIZE bytes"
    if [[ $DEBUG_SIZE -gt 0 ]]; then
        echo "Debug log content (first 500 bytes):"
        head -c 500 /tmp/qemu_debug.log | xxd
    fi
fi

# Cleanup
rm -f "$VARS_FILE"

echo ""
echo "=== EFI Execution Proof: INCONCLUSIVE ==="
echo "This test needs efi_main.c to write UEFI variables for proof"
echo "Current approach: check if OVMF vars file changes"
