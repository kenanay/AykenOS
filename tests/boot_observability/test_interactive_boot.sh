#!/usr/bin/env bash
# Interactive Boot Test
# Runs QEMU with VNC to see what OVMF is actually doing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Interactive Boot Test ==="
echo ""
echo "This will launch QEMU with graphical output."
echo "Watch for:"
echo "  1. UEFI shell appearing"
echo "  2. startup.nsh being executed"
echo "  3. BOOTX64.EFI being loaded"
echo "  4. Any error messages"
echo ""
echo "Press Ctrl+C to exit QEMU"
echo ""
echo "Starting QEMU in 2 seconds..."
sleep 2

# Run QEMU with graphical output and debugcon logging
DEBUGCON_LOG=$(mktemp /tmp/debugcon_interactive.XXXXXX.log)
echo "Debugcon log: $DEBUGCON_LOG"

qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -no-reboot \
    -m 256M

echo ""
echo "QEMU exited."
echo ""
echo "Debugcon log size: $(stat -f%z "$DEBUGCON_LOG" 2>/dev/null || stat -c%s "$DEBUGCON_LOG" 2>/dev/null) bytes"

if [[ -s "$DEBUGCON_LOG" ]]; then
    echo "Debugcon output (first 500 bytes):"
    head -c 500 "$DEBUGCON_LOG" | xxd
fi

rm -f "$DEBUGCON_LOG"
