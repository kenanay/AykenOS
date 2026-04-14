#!/bin/bash
# Quick BCIB worker boot test

set -e

echo "Building kernel with validation profile..."
make KERNEL_PROFILE=validation clean > /dev/null 2>&1
make KERNEL_PROFILE=validation kernel.elf > /dev/null 2>&1
make KERNEL_PROFILE=validation efi-img > /dev/null 2>&1

echo "Starting QEMU..."
mkdir -p out/logs
cp -f OVMF_VARS.clean.fd out/build/ovmf_vars.fd

timeout 10 qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -debugcon file:out/logs/bcib_test.log \
    -global isa-debugcon.iobase=0xe9 \
    -serial file:out/logs/bcib_serial.log \
    -nographic \
    -no-reboot || true

echo ""
echo "=== BCIB Worker Markers ==="
grep -E "BCIB|LATE\]8" out/logs/bcib_test.log | head -20 || echo "No markers found in debugcon log"

echo ""
echo "=== Scheduler Activity ==="
grep -E "SCHED_MB_ACCEPT|SCHED_MB_REJECT|MB_REASON" out/logs/bcib_test.log | head -10 || echo "No scheduler markers"

echo ""
echo "Done. Full log: out/logs/bcib_test.log"
