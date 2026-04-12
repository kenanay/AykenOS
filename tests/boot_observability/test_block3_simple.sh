#!/usr/bin/env bash
# Simplified Block 3 test for debugging

set -euo pipefail

PROJECT_ROOT="/Users/asel/Desktop/AykenOS"
DEBUGCON_LOG="/tmp/test_block3_simple.log"

echo "=== Simple Block 3 Test ==="
echo "Debugcon log: $DEBUGCON_LOG"

# Clean
rm -f "$DEBUGCON_LOG"

# Build
cd "$PROJECT_ROOT"
make bootloader > /dev/null 2>&1
make kernel > /dev/null 2>&1
make efi-img > /dev/null 2>&1

echo "Running QEMU..."

# Run QEMU
set +e
timeout 8s qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
  -nographic \
  -debugcon file:/tmp/test_block3_simple.log \
  -global isa-debugcon.iobase=0xE9 \
  -no-reboot \
  > /dev/null 2>&1
QEMU_EXIT=$?
set -e

echo "QEMU exit: $QEMU_EXIT"
echo "Log size: $(wc -c < /tmp/test_block3_simple.log) bytes"
echo ""
echo "Markers found:"
grep -E "\[B\]\[UEFI_BOOT_START\]|\[B\]\[KERNEL_ELF_LOADED\]|\[B\]\[JUMP_NOW\]|\[\[AYKEN_BOOT_OK\]\]|\[K\]\[EARLY_BOOT_OK\]" /tmp/test_block3_simple.log || echo "No markers found"
