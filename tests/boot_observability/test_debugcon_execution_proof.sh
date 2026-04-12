#!/usr/bin/env bash
# Debugcon Execution Proof Test
# Uses port 0xE9 writes as execution proof
# efi_main() writes 'B' to port 0xE9 immediately after InitializeLib

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Debugcon Execution Proof Test ==="
echo ""

# Check prerequisites
if [[ ! -f "$PROJECT_ROOT/out/build/EFI.img" ]]; then
    echo "FAIL: EFI.img not found. Run 'make bootloader && make efi-img' first."
    exit 1
fi

# Create clean capture file
DEBUGCON_LOG=$(mktemp /tmp/debugcon.XXXXXX.log)
echo "Debugcon capture: $DEBUGCON_LOG"

# Run QEMU with debugcon capture
# efi_main() writes 'B' to port 0xE9 after InitializeLib
# This should appear in debugcon log
timeout 6s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -no-reboot \
    2>/dev/null || true

echo ""
echo "--- Execution Proof Analysis ---"

# Check debugcon capture size
DEBUGCON_SIZE=$(stat -f%z "$DEBUGCON_LOG" 2>/dev/null || stat -c%s "$DEBUGCON_LOG" 2>/dev/null)
echo "Debugcon capture size: $DEBUGCON_SIZE bytes"

if [[ $DEBUGCON_SIZE -eq 0 ]]; then
    echo "RESULT: NO EVIDENCE - Debugcon capture is empty"
    echo ""
    echo "This means:"
    echo "  - efi_main() did not execute, OR"
    echo "  - Port 0xE9 writes not captured by QEMU debugcon"
    echo ""
    echo "Next step: Verify QEMU debugcon support"
    rm -f "$DEBUGCON_LOG"
    exit 1
fi

# Look for 'B' sentinel (bootloader execution marker)
if grep -q "B" "$DEBUGCON_LOG" 2>/dev/null || xxd "$DEBUGCON_LOG" | grep -q "42" 2>/dev/null; then
    echo "✓ EXECUTION PROOF ESTABLISHED"
    echo ""
    echo "Evidence: 'B' character found in debugcon output"
    echo "This proves efi_main() executed past InitializeLib()"
    echo ""
    echo "Debugcon output (hex dump, first 200 bytes):"
    head -c 200 "$DEBUGCON_LOG" | xxd
    echo ""
    echo "Debugcon output (text, first 500 bytes):"
    head -c 500 "$DEBUGCON_LOG" | cat -v
    rm -f "$DEBUGCON_LOG"
    exit 0
else
    echo "RESULT: INCONCLUSIVE - Debugcon has data but no 'B' sentinel"
    echo ""
    echo "Debugcon output (hex dump, first 200 bytes):"
    head -c 200 "$DEBUGCON_LOG" | xxd
    echo ""
    echo "Debugcon output (text, first 500 bytes):"
    head -c 500 "$DEBUGCON_LOG" | cat -v
    rm -f "$DEBUGCON_LOG"
    exit 1
fi
