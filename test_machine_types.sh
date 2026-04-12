#!/bin/bash
# Test different QEMU machine types to isolate debugcon issue

set -e

mkdir -p out/logs

echo "=== Testing Different Machine Types ==="
echo ""

# Test 1: q35 (current)
echo "Test 1: -machine q35"
timeout 2s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -debugcon file:out/logs/test_q35_debugcon.log \
    -global isa-debugcon.iobase=0xe9 \
    -serial file:out/logs/test_q35_serial.log \
    -nographic \
    -no-reboot 2>&1 | head -20 || true

Q35_DEBUGCON=$(stat -f%z out/logs/test_q35_debugcon.log 2>/dev/null || echo 0)
Q35_SERIAL=$(stat -f%z out/logs/test_q35_serial.log 2>/dev/null || echo 0)
echo "  debugcon: $Q35_DEBUGCON bytes, serial: $Q35_SERIAL bytes"
echo ""

# Test 2: pc (legacy)
echo "Test 2: -machine pc"
timeout 2s qemu-system-x86_64 \
    -machine pc \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -debugcon file:out/logs/test_pc_debugcon.log \
    -global isa-debugcon.iobase=0xe9 \
    -serial file:out/logs/test_pc_serial.log \
    -nographic \
    -no-reboot 2>&1 | head -20 || true

PC_DEBUGCON=$(stat -f%z out/logs/test_pc_debugcon.log 2>/dev/null || echo 0)
PC_SERIAL=$(stat -f%z out/logs/test_pc_serial.log 2>/dev/null || echo 0)
echo "  debugcon: $PC_DEBUGCON bytes, serial: $PC_SERIAL bytes"
echo ""

# Test 3: stdio test (diagnostic)
echo "Test 3: -debugcon stdio (diagnostic)"
timeout 2s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -debugcon stdio \
    -global isa-debugcon.iobase=0xe9 \
    -nographic \
    -no-reboot 2>&1 | tee out/logs/test_stdio_output.log | head -50 || true

STDIO_SIZE=$(stat -f%z out/logs/test_stdio_output.log 2>/dev/null || echo 0)
echo "  stdio output: $STDIO_SIZE bytes"
echo ""

echo "=== Summary ==="
echo "q35:  debugcon=$Q35_DEBUGCON, serial=$Q35_SERIAL"
echo "pc:   debugcon=$PC_DEBUGCON, serial=$PC_SERIAL"
echo "stdio: $STDIO_SIZE bytes"
echo ""

if [ "$Q35_DEBUGCON" -gt 0 ] || [ "$Q35_SERIAL" -gt 0 ]; then
    echo "✓ q35 machine type works"
elif [ "$PC_DEBUGCON" -gt 0 ] || [ "$PC_SERIAL" -gt 0 ]; then
    echo "⚠ pc machine type works, q35 doesn't"
elif [ "$STDIO_SIZE" -gt 0 ]; then
    echo "⚠ stdio works, file capture doesn't"
else
    echo "✗ No output on any configuration"
    echo "  → Port writes may not be executing"
    echo "  → OR OVMF not routing port 0xE9"
fi
