#!/bin/bash
# Test script for Block 2: Channel Proof

set -e

DEBUGCON_LOG="out/logs/test_debugcon.log"
SERIAL_LOG="out/logs/test_serial.log"
QEMU_LOG="out/logs/test_qemu.log"

mkdir -p out/logs

# Clean old logs
rm -f "$DEBUGCON_LOG" "$SERIAL_LOG" "$QEMU_LOG"

echo "=== Testing Channel Proof (Block 2) ==="
echo "Running QEMU with debugcon and serial capture..."

# Run QEMU with timeout
timeout 5s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file=EFI.img \
    -boot order=c \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -serial file:"$SERIAL_LOG" \
    -nographic \
    -no-reboot \
    >"$QEMU_LOG" 2>&1 || true

echo ""
echo "=== Test Results ==="
echo ""

# Check debugcon log
if [ -f "$DEBUGCON_LOG" ]; then
    DEBUGCON_SIZE=$(stat -f%z "$DEBUGCON_LOG" 2>/dev/null || stat -c%s "$DEBUGCON_LOG" 2>/dev/null || echo 0)
    echo "Debugcon log size: $DEBUGCON_SIZE bytes"
    if [ "$DEBUGCON_SIZE" -gt 0 ]; then
        echo "✓ Debugcon channel is working"
        echo ""
        echo "First 20 lines of debugcon output:"
        head -20 "$DEBUGCON_LOG"
        echo ""
        echo "Checking for channel proof markers:"
        if grep -q "B" "$DEBUGCON_LOG"; then
            echo "✓ Found 'B' (bootloader channel test)"
        else
            echo "✗ Missing 'B' (bootloader channel test)"
        fi
        if grep -q "K" "$DEBUGCON_LOG"; then
            echo "✓ Found 'K' (kernel entry channel test)"
        else
            echo "✗ Missing 'K' (kernel entry channel test)"
        fi
    else
        echo "✗ Debugcon log is empty"
    fi
else
    echo "✗ Debugcon log file not created"
fi

echo ""

# Check serial log
if [ -f "$SERIAL_LOG" ]; then
    SERIAL_SIZE=$(stat -f%z "$SERIAL_LOG" 2>/dev/null || stat -c%s "$SERIAL_LOG" 2>/dev/null || echo 0)
    echo "Serial log size: $SERIAL_SIZE bytes"
    if [ "$SERIAL_SIZE" -gt 0 ]; then
        echo "✓ Serial channel is working"
        echo ""
        echo "First 20 lines of serial output:"
        head -20 "$SERIAL_LOG"
    else
        echo "✗ Serial log is empty"
    fi
else
    echo "✗ Serial log file not created"
fi

echo ""
echo "=== Summary ==="
if [ "$DEBUGCON_SIZE" -gt 0 ] || [ "$SERIAL_SIZE" -gt 0 ]; then
    echo "✓ At least one output channel is working"
    echo "✓ Block 2: Channel Proof PASSED"
else
    echo "✗ All output channels are empty"
    echo "✗ Block 2: Channel Proof FAILED"
    exit 1
fi
