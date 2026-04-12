#!/bin/bash
# Test with -debugcon stdio to see immediate output

echo "=== Testing with -debugcon stdio (diagnostic mode) ==="
echo "This will show debugcon output in real-time..."
echo ""

timeout 3s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -debugcon stdio \
    -global isa-debugcon.iobase=0xe9 \
    -nographic \
    -no-reboot 2>&1 | head -50 || true

echo ""
echo "=== Test complete ==="
