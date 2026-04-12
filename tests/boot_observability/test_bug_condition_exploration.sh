#!/usr/bin/env bash
# Property 1: Bug Condition - Evidence Pipeline Integrity Failure
# This test MUST FAIL on unfixed code

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Bug Condition Exploration Test ==="
cd "$PROJECT_ROOT"

# Build
make bootloader > /dev/null 2>&1
make kernel > /dev/null 2>&1
make efi-img > /dev/null 2>&1

DEBUGCON_LOG="/tmp/qemu_debug_bug_cond.log"
SERIAL_LOG="/tmp/qemu_serial_bug_cond.log"
rm -f "$DEBUGCON_LOG" "$SERIAL_LOG"

timeout 10s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xE9 \
    -serial file:"$SERIAL_LOG" \
    -no-reboot \
    2>&1 | head -5 || true

FAIL=0

DEBUGCON_SIZE=0
if [ -f "$DEBUGCON_LOG" ]; then DEBUGCON_SIZE=$(stat -c%s "$DEBUGCON_LOG" 2>/dev/null || echo "0"); fi

SERIAL_SIZE=0
if [ -f "$SERIAL_LOG" ]; then SERIAL_SIZE=$(stat -c%s "$SERIAL_LOG" 2>/dev/null || echo "0"); fi

if [[ "$DEBUGCON_SIZE" == "0" && "$SERIAL_SIZE" == "0" ]]; then
    echo "❌ BUG DETECTED: Both debugcon and serial logs are empty!"
    FAIL=1
else
    echo "✅ Logs are not completely empty (debugcon: $DEBUGCON_SIZE, serial: $SERIAL_SIZE)"
fi

if grep -q "\[B\]\[UEFI_BOOT_START\]" "$DEBUGCON_LOG" "$SERIAL_LOG" 2>/dev/null; then
    echo "✅ Bootloader marker [B][UEFI_BOOT_START] found"
else
    echo "❌ BUG DETECTED: Missing [B][UEFI_BOOT_START]"
    FAIL=1
fi

if grep -q "\[\[AYKEN_BOOT_OK\]\]" "$DEBUGCON_LOG" "$SERIAL_LOG" 2>/dev/null; then
    echo "✅ Kernel marker [[AYKEN_BOOT_OK]] found"
else
    echo "❌ BUG DETECTED: Missing [[AYKEN_BOOT_OK]]"
    FAIL=1
fi

# Check if sort is used in the evidence pipeline (the original harness)
HARNESS="$PROJECT_ROOT/scripts/qemu-fail-closed-proof-harness.sh"
if grep -qE "(^|\s)sort(\s|$)" "$HARNESS"; then
    echo "❌ BUG DETECTED: Sort operation found in harness"
    FAIL=1
fi


if [[ "$FAIL" == "1" ]]; then
    echo "=== Test FAILS as expected on unfixed code ==="
    exit 1
fi

echo "=== Test PASSES (code is fixed) ==="
exit 0
