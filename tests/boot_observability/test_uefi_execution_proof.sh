#!/usr/bin/env bash
# UEFI Execution Proof Test
# Single purpose: Does efi_main() execute?
# Evidence: UEFI Print("[UEFI_BOOT_OK]") must appear in serial output

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== UEFI Execution Proof Test ==="
echo ""

# Check prerequisites (build artifacts must exist)
if [[ ! -f "$PROJECT_ROOT/out/build/EFI.img" ]]; then
    echo "FAIL: EFI.img not found. Run 'make bootloader && make efi-img' first."
    exit 1
fi

# Create clean capture file
SERIAL_CAPTURE=$(mktemp /tmp/uefi_serial.XXXXXX.log)
echo "Serial capture: $SERIAL_CAPTURE"

# Run QEMU with minimal configuration
# - Serial output to file (UEFI ConOut should route here)
# - 6 second timeout (2s Stall + 4s margin)
# - No other noise
timeout 6s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -serial file:"$SERIAL_CAPTURE" \
    -no-reboot \
    2>/dev/null || true

echo ""
echo "--- Execution Proof Analysis ---"

# Check serial capture size
SERIAL_SIZE=$(stat -f%z "$SERIAL_CAPTURE" 2>/dev/null || stat -c%s "$SERIAL_CAPTURE" 2>/dev/null)
echo "Serial capture size: $SERIAL_SIZE bytes"

if [[ $SERIAL_SIZE -eq 0 ]]; then
    echo "RESULT: NO EVIDENCE - Serial capture is empty"
    echo "Possible causes:"
    echo "  1. efi_main() did not execute"
    echo "  2. UEFI ConOut not routed to serial"
    echo "  3. OVMF serial configuration issue"
    rm -f "$SERIAL_CAPTURE"
    exit 1
fi

# Look for execution sentinel
if grep -q "UEFI_BOOT_OK" "$SERIAL_CAPTURE"; then
    echo "✓ EXECUTION PROOF ESTABLISHED"
    echo ""
    echo "Evidence found in serial capture:"
    grep "UEFI_BOOT_OK" "$SERIAL_CAPTURE"
    echo ""
    echo "Full serial output (first 500 bytes):"
    head -c 500 "$SERIAL_CAPTURE"
    rm -f "$SERIAL_CAPTURE"
    exit 0
else
    echo "RESULT: INCONCLUSIVE - Serial has data but no sentinel"
    echo ""
    echo "Serial output (first 500 bytes):"
    head -c 500 "$SERIAL_CAPTURE"
    echo ""
    echo "This suggests:"
    echo "  - UEFI firmware is producing output"
    echo "  - But efi_main() sentinel not reached"
    echo "  - Or Print() not working as expected"
    rm -f "$SERIAL_CAPTURE"
    exit 1
fi
