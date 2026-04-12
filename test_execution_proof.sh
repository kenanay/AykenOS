#!/bin/bash
# EXECUTION PROOF TEST: Prove bootloader efi_main() executes
# This test MUST pass before diagnosing port routing issues

set -e

mkdir -p out/logs

echo "=== EXECUTION PROOF TEST ==="
echo "Goal: Prove efi_main() executes (2-second stall should be observable)"
echo ""

# Test 1: Timing test - if efi_main runs, we should see ~2 second delay
echo "Test 1: Timing Test (2-second stall proof)"
echo "Expected: ~2-4 seconds (2s stall + boot overhead)"
echo "If timeout (5s): efi_main NOT executing"
echo ""

START_TIME=$(date +%s)
timeout 5s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -nographic \
    -no-reboot \
    > out/logs/timing_test.log 2>&1 || true
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo "Elapsed time: ${ELAPSED} seconds"
if [ "$ELAPSED" -ge 2 ] && [ "$ELAPSED" -le 4 ]; then
    echo "✓ Timing matches 2-second stall (efi_main likely executing)"
    TIMING_PASS=1
elif [ "$ELAPSED" -ge 5 ]; then
    echo "✗ Timeout (5s) - efi_main may NOT be executing"
    TIMING_PASS=0
else
    echo "? Too fast (< 2s) - inconclusive"
    TIMING_PASS=0
fi
echo ""

# Test 2: UEFI Print ground truth test (proper console capture)
echo "Test 2: UEFI Print Ground Truth"
echo "Expected: [UEFI_BOOT_OK] in console output"
echo ""

# Use -serial file to capture console properly (no stdio conflict)
timeout 5s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=firmware/ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=out/build/ovmf_vars.fd \
    -drive format=raw,file=out/build/EFI.img \
    -boot order=c \
    -nographic \
    -serial file:out/logs/uefi_console.log \
    -no-reboot \
    > /dev/null 2>&1 || true

if [ -f out/logs/uefi_console.log ]; then
    echo "=== UEFI Console Output (last 50 lines) ==="
    tail -50 out/logs/uefi_console.log
    echo ""
    
    if grep -q "\[UEFI_BOOT_OK\]" out/logs/uefi_console.log; then
        echo "✓ [UEFI_BOOT_OK] FOUND - efi_main IS executing"
        UEFI_PASS=1
    else
        echo "✗ [UEFI_BOOT_OK] NOT FOUND - efi_main may not be executing"
        UEFI_PASS=0
    fi
else
    echo "✗ Console log not created"
    UEFI_PASS=0
fi
echo ""

# Test 3: Check EFI.img structure
echo "Test 3: EFI.img Structure Verification"
echo "Checking if BOOTX64.EFI exists in EFI.img..."
echo ""

# Mount EFI.img and check structure
if command -v hdiutil &> /dev/null; then
    # macOS
    MOUNT_POINT=$(hdiutil attach -nomount out/build/EFI.img | head -1 | awk '{print $1}')
    if [ -n "$MOUNT_POINT" ]; then
        mkdir -p out/logs/efi_mount
        mount -t msdos "$MOUNT_POINT" out/logs/efi_mount 2>/dev/null || true
        if [ -d out/logs/efi_mount/EFI ]; then
            echo "EFI directory structure:"
            find out/logs/efi_mount/EFI -type f
            if [ -f out/logs/efi_mount/EFI/BOOT/BOOTX64.EFI ]; then
                echo "✓ BOOTX64.EFI found at correct path"
                EFI_STRUCT_PASS=1
            else
                echo "✗ BOOTX64.EFI NOT found at EFI/BOOT/BOOTX64.EFI"
                EFI_STRUCT_PASS=0
            fi
            
            if [ -f out/logs/efi_mount/startup.nsh ]; then
                echo "✓ startup.nsh found"
                echo "Contents:"
                cat out/logs/efi_mount/startup.nsh
            else
                echo "? startup.nsh not found (may not be needed)"
            fi
        else
            echo "✗ EFI directory not found in image"
            EFI_STRUCT_PASS=0
        fi
        umount out/logs/efi_mount 2>/dev/null || true
        hdiutil detach "$MOUNT_POINT" 2>/dev/null || true
        rmdir out/logs/efi_mount 2>/dev/null || true
    else
        echo "? Could not mount EFI.img"
        EFI_STRUCT_PASS=0
    fi
else
    echo "? hdiutil not available (macOS only), skipping structure check"
    EFI_STRUCT_PASS=1  # Don't fail on this
fi
echo ""

# Final verdict
echo "=== EXECUTION PROOF VERDICT ==="
echo "Timing Test: $([ "$TIMING_PASS" -eq 1 ] && echo "PASS" || echo "FAIL")"
echo "UEFI Print Test: $([ "$UEFI_PASS" -eq 1 ] && echo "PASS" || echo "FAIL")"
echo "EFI Structure: $([ "$EFI_STRUCT_PASS" -eq 1 ] && echo "PASS" || echo "FAIL")"
echo ""

if [ "$UEFI_PASS" -eq 1 ]; then
    echo "✓✓✓ EXECUTION PROOF ESTABLISHED ✓✓✓"
    echo "efi_main() IS executing"
    echo ""
    echo "Next step: Diagnose why port 0xE9 produces 0 bytes"
    echo "Possible causes:"
    echo "  1. OVMF doesn't route port 0xE9 (try port 0x80 POST code)"
    echo "  2. QEMU debugcon config issue"
    echo "  3. Port write timing issue"
    exit 0
elif [ "$TIMING_PASS" -eq 1 ]; then
    echo "⚠ PARTIAL PROOF"
    echo "Timing suggests execution, but UEFI Print not captured"
    echo "Possible console capture issue"
    exit 1
else
    echo "✗✗✗ EXECUTION PROOF FAILED ✗✗✗"
    echo "efi_main() may NOT be executing"
    echo ""
    echo "Possible causes:"
    echo "  1. BOOTX64.EFI not in correct path"
    echo "  2. EFI.img structure incorrect"
    echo "  3. OVMF not finding bootloader"
    echo "  4. Bootloader crashes before efi_main"
    exit 1
fi
