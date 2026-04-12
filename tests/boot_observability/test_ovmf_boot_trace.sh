#!/usr/bin/env bash
# OVMF Boot Trace Test
# Captures QEMU trace to see what OVMF is doing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== OVMF Boot Trace Test ==="
echo ""

DEBUGCON_LOG=$(mktemp /tmp/debugcon_trace.XXXXXX.log)
QEMU_LOG=$(mktemp /tmp/qemu_trace.XXXXXX.log)

echo "Debugcon log: $DEBUGCON_LOG"
echo "QEMU trace log: $QEMU_LOG"
echo ""
echo "Running QEMU with full tracing..."

timeout 5s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -d guest_errors,unimp \
    -D "$QEMU_LOG" \
    -nographic \
    -no-reboot \
    2>&1 | head -50 || true

echo ""
echo "--- Analysis ---"
echo ""

DEBUGCON_SIZE=$(stat -f%z "$DEBUGCON_LOG" 2>/dev/null || stat -c%s "$DEBUGCON_LOG" 2>/dev/null)
QEMU_SIZE=$(stat -f%z "$QEMU_LOG" 2>/dev/null || stat -c%s "$QEMU_LOG" 2>/dev/null)

echo "Debugcon log size: $DEBUGCON_SIZE bytes"
echo "QEMU trace log size: $QEMU_SIZE bytes"
echo ""

if [[ $DEBUGCON_SIZE -gt 0 ]]; then
    echo "Debugcon output (first 500 bytes):"
    head -c 500 "$DEBUGCON_LOG" | xxd
    echo ""
fi

if [[ $QEMU_SIZE -gt 0 ]]; then
    echo "QEMU trace (first 50 lines):"
    head -50 "$QEMU_LOG"
    echo ""
fi

# Check if BOOTX64.EFI was accessed
if [[ -s "$QEMU_LOG" ]] && grep -qi "BOOTX64" "$QEMU_LOG"; then
    echo "✓ QEMU accessed BOOTX64.EFI"
else
    echo "✗ No evidence of BOOTX64.EFI access in trace"
fi

rm -f "$DEBUGCON_LOG" "$QEMU_LOG"
