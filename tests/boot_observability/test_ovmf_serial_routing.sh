#!/usr/bin/env bash
# OVMF Serial Routing Diagnostic
# Tests if OVMF routes any output to serial at all

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== OVMF Serial Routing Diagnostic ==="
echo ""

# Test 1: Check if OVMF produces ANY serial output during boot
echo "Test 1: OVMF boot messages to serial"
SERIAL_LOG=$(mktemp /tmp/ovmf_serial.XXXXXX.log)

timeout 3s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -serial file:"$SERIAL_LOG" \
    -nographic \
    -no-reboot \
    2>/dev/null || true

SERIAL_SIZE=$(stat -f%z "$SERIAL_LOG" 2>/dev/null || stat -c%s "$SERIAL_LOG" 2>/dev/null)
echo "Serial log size: $SERIAL_SIZE bytes"

if [[ $SERIAL_SIZE -gt 0 ]]; then
    echo "✓ OVMF produces serial output"
    echo "First 200 bytes:"
    head -c 200 "$SERIAL_LOG" | cat -v
else
    echo "✗ OVMF produces NO serial output"
    echo "This suggests -serial file: may not work with OVMF on this system"
fi

rm -f "$SERIAL_LOG"
echo ""

# Test 2: Try with stdio instead of file
echo "Test 2: OVMF boot messages to stdio"
echo "Running QEMU with -serial stdio for 2 seconds..."
echo "---"

timeout 2s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -serial stdio \
    -nographic \
    -no-reboot \
    2>/dev/null || true

echo ""
echo "---"
echo ""
echo "If you saw UEFI shell or boot messages above, serial routing works."
echo "If you saw nothing, OVMF may not be configured for serial console."
