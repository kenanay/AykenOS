#!/usr/bin/env bash
# Block 4: Regression Lock Test
# This is the CANONICAL test for the Boot Chain Observability evidence pipeline.
# It enforces strict CI gates and treats debugcon as the primary evidence channel.
# 
# CI GATES:
# 1. Zero-byte debugcon log -> HARD FAIL
# 2. Bootloader marker absent -> FAIL
# 3. Kernel marker absent -> FAIL
# 4. Marker order broken -> FAIL
# 5. NO forbidden operations (sort, uniq, cross-channel merge) in analysis

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Primary authoritative evidence channel
DEBUGCON_LOG="/tmp/qemu_debugcon_canonical.log"
# Secondary/diagnostic channel (not used for CI gates)
SERIAL_LOG="/tmp/qemu_serial_diagnostic.log"

echo "=== Block 4: Regression Lock (Canonical Test) ==="
echo "Timestamp: $(date)"

# Clean previous artifacts
rm -f "$DEBUGCON_LOG" "$SERIAL_LOG"

echo "[1/4] Building bootloader and EFI image..."
cd "$PROJECT_ROOT"
make bootloader > /dev/null 2>&1
make kernel > /dev/null 2>&1
make efi-img > /dev/null 2>&1

echo "[2/4] Running QEMU to capture evidence pipeline..."
# Launch QEMU with fail-closed timeout
# Using -debugcon as authoritative channel
timeout 10s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xE9 \
    -serial file:"$SERIAL_LOG" \
    -no-reboot \
    > /dev/null 2>&1 || true

# Wait a moment for file sync
sleep 1

echo "[3/4] Evaluating CI Gates..."

# GATE 1: Zero-byte debugcon log -> HARD FAIL
if [[ ! -f "$DEBUGCON_LOG" ]]; then
    echo "❌ HARD FAIL: Debugcon log file not created!"
    exit 1
fi

DEBUGCON_SIZE=$(stat -f%z "$DEBUGCON_LOG" 2>/dev/null || stat -c%s "$DEBUGCON_LOG" 2>/dev/null)
if [[ -z "$DEBUGCON_SIZE" || "$DEBUGCON_SIZE" -eq 0 ]]; then
    echo "❌ HARD FAIL: Debugcon log is zero bytes! Evidence pipeline is completely broken."
    exit 1
fi

echo "✅ GATE 1 PASS: Debugcon log captured ($DEBUGCON_SIZE bytes)"

# Expected markers in strict chronological order
EXPECTED_MARKERS=(
    "[B][UEFI_BOOT_START]"
    "[B][KERNEL_ELF_LOADED]"
    "[B][JUMP_NOW]"
    "[[AYKEN_BOOT_OK]]"
    "[K][EARLY_BOOT_OK]"
)

FAIL=0
PREV_LINE=0

echo "Checking canonical marker chain in debugcon:"
for MARKER in "${EXPECTED_MARKERS[@]}"; do
    # Find the FIRST occurrence of the marker to verify order
    # Using head -1 to avoid finding later duplicate markers or loops causing order issues
    LINE=$(grep -n -F "$MARKER" "$DEBUGCON_LOG" | head -1 | cut -d: -f1 || echo "0")
    
    # GATE 2 & 3: Marker absent -> FAIL
    if [[ -z "$LINE" || "$LINE" == "0" ]]; then
        echo "  ❌ FAIL: Missing mandatory marker: $MARKER"
        FAIL=1
    # GATE 4: Marker order broken -> FAIL
    elif [[ "$LINE" -le "$PREV_LINE" ]]; then
        echo "  ❌ FAIL: ORDER VIOLATION: $MARKER found at line $LINE, which is before/at previous marker line $PREV_LINE"
        FAIL=1
    else
        echo "  ✅ FOUND: $MARKER at line $LINE"
        PREV_LINE=$LINE
    fi
done

if [[ "$FAIL" -ne 0 ]]; then
    echo ""
    echo "❌ BLOCK 4 REGRESSION LOCK: FAILED"
    echo "Evidence pipeline integrity violation detected."
    exit 1
fi

echo ""
echo "✅ BLOCK 4 REGRESSION LOCK: PASSED"
echo "All markers present and correctly ordered in authoritative channel."
exit 0
