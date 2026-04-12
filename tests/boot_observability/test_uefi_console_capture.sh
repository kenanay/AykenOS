#!/usr/bin/env bash
# Test 3: UEFI Console Capture
# Properly routes UEFI ConOut to a capturable backend

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== UEFI Console Capture Test ==="

# Build bootloader and EFI image
cd "$PROJECT_ROOT"
make bootloader
make efi-img

# Create capture files
SERIAL_LOG=$(mktemp /tmp/serial.XXXXXX)
MONITOR_LOG=$(mktemp /tmp/monitor.XXXXXX)

echo "Serial log: $SERIAL_LOG"
echo "Monitor log: $MONITOR_LOG"

# Run QEMU with multiple capture points
timeout 8s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -serial file:"$SERIAL_LOG" \
    -monitor file:"$MONITOR_LOG" \
    -debugcon file:/tmp/qemu_debug.log \
    -d guest_errors,cpu_reset \
    -D /tmp/qemu_trace.log \
    -no-reboot \
    2>&1 | tee /tmp/qemu_stdout.log || true

echo ""
echo "--- Capture Results ---"

# Check serial log
SERIAL_SIZE=$(stat -f%z "$SERIAL_LOG" 2>/dev/null || stat -c%s "$SERIAL_LOG" 2>/dev/null)
echo "Serial log size: $SERIAL_SIZE bytes"
if [[ $SERIAL_SIZE -gt 0 ]]; then
    echo "Serial content (first 200 bytes):"
    head -c 200 "$SERIAL_LOG" | xxd
fi

# Check monitor log
MONITOR_SIZE=$(stat -f%z "$MONITOR_LOG" 2>/dev/null || stat -c%s "$MONITOR_LOG" 2>/dev/null)
echo "Monitor log size: $MONITOR_SIZE bytes"

# Check debug console
if [[ -f /tmp/qemu_debug.log ]]; then
    DEBUG_SIZE=$(stat -f%z /tmp/qemu_debug.log 2>/dev/null || stat -c%s /tmp/qemu_debug.log 2>/dev/null)
    echo "Debug console size: $DEBUG_SIZE bytes"
    if [[ $DEBUG_SIZE -gt 0 ]]; then
        echo "Debug console content:"
        head -c 500 /tmp/qemu_debug.log | xxd
    fi
fi

# Check QEMU trace
if [[ -f /tmp/qemu_trace.log ]]; then
    TRACE_SIZE=$(stat -f%z /tmp/qemu_trace.log 2>/dev/null || stat -c%s /tmp/qemu_trace.log 2>/dev/null)
    echo "QEMU trace size: $TRACE_SIZE bytes"
fi

# Check stdout capture
if [[ -f /tmp/qemu_stdout.log ]]; then
    STDOUT_SIZE=$(stat -f%z /tmp/qemu_stdout.log 2>/dev/null || stat -c%s /tmp/qemu_stdout.log 2>/dev/null)
    echo "QEMU stdout size: $STDOUT_SIZE bytes"
    if [[ $STDOUT_SIZE -gt 0 ]]; then
        echo "QEMU stdout content:"
        head -20 /tmp/qemu_stdout.log
    fi
fi

# Cleanup
rm -f "$SERIAL_LOG" "$MONITOR_LOG"

echo ""
echo "=== UEFI Console Capture: DIAGNOSTIC ==="
echo "Expected: UEFI Print() output in at least one channel"
echo "If all channels empty → UEFI ConOut routing issue"
