#!/usr/bin/env bash
# Block 3: Deterministic Boot Marker Chain Test
# Validates full boot chain from bootloader through kernel entry
# Channel-local analysis (debugcon only, no merge, no sort)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Test artifacts
DEBUGCON_LOG="/tmp/qemu_debug_block3.log"
RESULT_LOG="$PROJECT_ROOT/test_block3_result.log"

# Clean previous artifacts
rm -f "$DEBUGCON_LOG" "$RESULT_LOG"

echo "=== Block 3: Boot Marker Chain Test ===" | tee "$RESULT_LOG"
echo "Timestamp: $(date)" | tee -a "$RESULT_LOG"
echo "" | tee -a "$RESULT_LOG"

# Build bootloader and EFI image
echo "[1/4] Building bootloader and EFI image..." | tee -a "$RESULT_LOG"
cd "$PROJECT_ROOT"
make bootloader > /dev/null 2>&1
make kernel > /dev/null 2>&1
make efi-img > /dev/null 2>&1

# Run QEMU with debugcon capture
echo "[2/4] Running QEMU with debugcon capture..." | tee -a "$RESULT_LOG"

timeout 10s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xE9 \
    -no-reboot \
    2>&1 | head -5 || true

# Verify debugcon log exists
if [[ ! -f "$DEBUGCON_LOG" ]]; then
    echo "❌ FAIL: debugcon log not created" | tee -a "$RESULT_LOG"
    exit 1
fi

DEBUGCON_SIZE=$(wc -c < "$DEBUGCON_LOG")
echo "Debugcon log size: $DEBUGCON_SIZE bytes" | tee -a "$RESULT_LOG"
echo "" | tee -a "$RESULT_LOG"

# Expected marker sequence (channel-local, append-order preserved)
EXPECTED_MARKERS=(
    "[B][UEFI_BOOT_START]"
    "[B][KERNEL_ELF_LOADED]"
    "[B][JUMP_NOW]"
)

# Optional kernel markers (at least one must be present)
KERNEL_MARKERS=(
    "[[AYKEN_BOOT_OK]]"
    "[K][EARLY_BOOT_OK]"
)

echo "[3/4] Validating marker chain..." | tee -a "$RESULT_LOG"
echo "" | tee -a "$RESULT_LOG"

# Extract markers preserving order (NO sort, NO reorder)
FAIL=0

# Check bootloader markers in order
echo "Checking bootloader marker sequence:" | tee -a "$RESULT_LOG"
PREV_LINE=0
for MARKER in "${EXPECTED_MARKERS[@]}"; do
    LINE=$(grep -n "$MARKER" "$DEBUGCON_LOG" | head -1 | cut -d: -f1 || echo "0")
    if [[ "$LINE" == "0" ]]; then
        echo "  ❌ MISSING: $MARKER" | tee -a "$RESULT_LOG"
        FAIL=1
    elif [[ "$LINE" -le "$PREV_LINE" ]]; then
        echo "  ❌ ORDER VIOLATION: $MARKER at line $LINE (expected after $PREV_LINE)" | tee -a "$RESULT_LOG"
        FAIL=1
    else
        echo "  ✅ FOUND: $MARKER at line $LINE" | tee -a "$RESULT_LOG"
        PREV_LINE=$LINE
    fi
done

echo "" | tee -a "$RESULT_LOG"

# Check kernel markers (at least one must be present after bootloader markers)
echo "Checking kernel entry markers:" | tee -a "$RESULT_LOG"
KERNEL_FOUND=0
for MARKER in "${KERNEL_MARKERS[@]}"; do
    LINE=$(grep -n "$MARKER" "$DEBUGCON_LOG" | head -1 | cut -d: -f1 || echo "0")
    if [[ "$LINE" != "0" ]]; then
        if [[ "$LINE" -le "$PREV_LINE" ]]; then
            echo "  ❌ ORDER VIOLATION: $MARKER at line $LINE (expected after $PREV_LINE)" | tee -a "$RESULT_LOG"
            FAIL=1
        else
            echo "  ✅ FOUND: $MARKER at line $LINE" | tee -a "$RESULT_LOG"
            KERNEL_FOUND=1
        fi
    fi
done

if [[ "$KERNEL_FOUND" == "0" ]]; then
    echo "  ❌ MISSING: No kernel entry marker found" | tee -a "$RESULT_LOG"
    FAIL=1
fi

echo "" | tee -a "$RESULT_LOG"

# Final verdict
if [[ "$FAIL" == "0" ]]; then
    echo "✅ BLOCK 3 PASS: Full boot marker chain verified" | tee -a "$RESULT_LOG"
    echo "   - Bootloader markers present and ordered" | tee -a "$RESULT_LOG"
    echo "   - Kernel entry marker present" | tee -a "$RESULT_LOG"
    echo "   - Channel-local analysis (no merge, no sort)" | tee -a "$RESULT_LOG"
    exit 0
else
    echo "❌ BLOCK 3 FAIL: Marker chain incomplete or out of order" | tee -a "$RESULT_LOG"
    exit 1
fi
