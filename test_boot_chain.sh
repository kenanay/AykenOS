#!/usr/bin/env bash
# Minimal boot chain test - Phase 16 boot observability debug
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "=== Boot Chain Debug Test ==="
echo "Goal: Verify startup.nsh → BOOTX64.EFI → kernel_main chain"
echo ""

# Clean logs
rm -f /tmp/boot_debugcon.log /tmp/boot_serial.log

# Build with validation profile
echo "[1/4] Building with KERNEL_PROFILE=validation..."
make clean >/dev/null 2>&1 || true
KERNEL_PROFILE=validation make -j8 || {
    echo "ERROR: Build failed"
    exit 1
}

echo "[2/4] Checking EFI.img layout..."
if [[ ! -f out/build/EFI.img ]]; then
    echo "ERROR: EFI.img not found"
    exit 1
fi

# Mount and check (macOS)
MOUNT_POINT=$(mktemp -d /tmp/efi_mount.XXXXXX)
hdiutil attach -mountpoint "$MOUNT_POINT" out/build/EFI.img >/dev/null 2>&1 || {
    echo "ERROR: Cannot mount EFI.img"
    exit 1
}

echo "  Checking startup.nsh..."
if [[ -f "$MOUNT_POINT/startup.nsh" ]]; then
    echo "  ✓ startup.nsh found"
    cat "$MOUNT_POINT/startup.nsh"
else
    echo "  ✗ startup.nsh NOT FOUND"
fi

echo "  Checking BOOTX64.EFI..."
if [[ -f "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI" ]]; then
    echo "  ✓ BOOTX64.EFI found"
else
    echo "  ✗ BOOTX64.EFI NOT FOUND"
fi

hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1
rmdir "$MOUNT_POINT"

echo ""
echo "[3/4] Running QEMU (10s timeout)..."

# Use homebrew OVMF path (macOS)
OVMF_CODE="/opt/homebrew/share/qemu/edk2-x86_64-code.fd"
if [[ ! -f "$OVMF_CODE" ]]; then
    echo "ERROR: OVMF not found at $OVMF_CODE"
    exit 1
fi

timeout 10s qemu-system-x86_64 \
    -machine q35 \
    -m 256M \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive format=raw,file=out/build/EFI.img \
    -debugcon file:/tmp/boot_debugcon.log \
    -global isa-debugcon.iobase=0xE9 \
    -serial file:/tmp/boot_serial.log \
    -nographic \
    -no-reboot || true

echo ""
echo "[4/4] Analyzing logs..."
echo ""

# Decision tree
echo "=== DECISION TREE ==="
echo ""

if grep -q "STARTUP_OK" /tmp/boot_serial.log 2>/dev/null || \
   grep -q "STARTUP_OK" /tmp/boot_debugcon.log 2>/dev/null; then
    echo "✓ Scenario: startup.nsh executed"
    
    if grep -q "UEFI_BOOT_START" /tmp/boot_debugcon.log 2>/dev/null || \
       grep -q "UEFI_BOOT_START" /tmp/boot_serial.log 2>/dev/null; then
        echo "✓ Scenario: BOOTX64.EFI executed"
        
        if grep -q "K0" /tmp/boot_debugcon.log 2>/dev/null || \
           grep -q "K0" /tmp/boot_serial.log 2>/dev/null; then
            echo "✓ Scenario: kernel_main reached"
            
            if grep -q "\[K\]\[LATE\]" /tmp/boot_debugcon.log 2>/dev/null; then
                echo "✓ Scenario: Late init completed"
                echo ""
                echo "SUCCESS: Boot chain verified. Ready for BCIB debug."
            else
                echo "✗ Blocker: Late init not reached"
                echo "  → kernel init path problem"
            fi
        else
            echo "✗ Blocker: kernel_main not reached"
            echo "  → bootloader → kernel handoff broken"
        fi
    else
        echo "✗ Blocker: BOOTX64.EFI not executed"
        echo "  → startup.nsh → BOOTX64.EFI call broken"
    fi
else
    echo "✗ Blocker: startup.nsh not executed"
    echo "  → UEFI shell autostart broken"
fi

echo ""
echo "=== LOG SAMPLES ==="
echo ""
echo "--- debugcon (first 50 lines) ---"
head -50 /tmp/boot_debugcon.log 2>/dev/null || echo "(empty)"
echo ""
echo "--- serial (first 50 lines) ---"
head -50 /tmp/boot_serial.log 2>/dev/null || echo "(empty)"
echo ""
echo "Full logs: /tmp/boot_debugcon.log, /tmp/boot_serial.log"
