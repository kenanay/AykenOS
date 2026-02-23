#!/usr/bin/env bash
# ============================================================================
# Deterministic UEFI Boot Image Builder
# ============================================================================
# Purpose:
#   Create EFI.img with deterministic boot path (no Shell dependency)
#
# Mimari Gereksinimler:
#   - No startup.nsh (Shell'e bağımlılık yok)
#   - Standard boot path: \EFI\BOOT\BOOTX64.EFI
#   - OVMF removable media boot kullanır
#   - %100 deterministik boot
#
# Boot Flow:
#   OVMF → Removable Media → \EFI\BOOT\BOOTX64.EFI → kernel.elf
#
# Copyright © 2026 Kenan AY
# ============================================================================

set -euo pipefail

IMG=EFI.img

echo "=== Deterministic UEFI Boot Image Builder ==="
echo ""

# Force clean rebuild (deterministic)
if [[ -f "$IMG" || -f "$IMG.dmg" ]]; then
    echo "[*] Removing existing EFI image artifacts for clean rebuild..."
    rm -f "$IMG" "$IMG.dmg"
fi

# Check prerequisites
if [[ ! -f "bootloader/efi/BOOTX64.EFI" ]]; then
    echo "ERROR: bootloader/efi/BOOTX64.EFI not found"
    echo "Run: make bootloader"
    exit 1
fi

if [[ ! -f "kernel.elf" ]]; then
    echo "ERROR: kernel.elf not found"
    echo "Run: make kernel"
    exit 1
fi

# macOS: Use hdiutil (preferred)
if [[ "$(uname)" == "Darwin" ]] && command -v hdiutil >/dev/null 2>&1; then
    echo "[*] Using hdiutil (macOS native)..."
    
    MOUNT_VOL="/Volumes/EFI"
    TMP_DMG="${IMG}.dmg"
    
    # Create GPT disk with EFI System Partition
    echo "[*] Creating 200MB GPT disk with ESP..."
    hdiutil create -size 200m -layout GPTSPUD -partitionType EFI \
        -fs "MS-DOS FAT32" -volname EFI "$TMP_DMG" >/dev/null 2>&1
    
    # Attach and mount
    echo "[*] Attaching disk image..."
    DEV=$(hdiutil attach -nomount "$TMP_DMG" | head -n1 | awk '{print $1}')
    
    if [[ -z "$DEV" ]]; then
        echo "ERROR: Failed to attach disk image"
        rm -f "$TMP_DMG"
        exit 1
    fi
    
    echo "[*] Mounting ESP partition..."
    if ! diskutil mount "${DEV}s1" >/dev/null 2>&1; then
        echo "ERROR: Failed to mount ESP"
        hdiutil detach "$DEV" >/dev/null 2>&1 || true
        rm -f "$TMP_DMG"
        exit 1
    fi
    
    # Create standard EFI directory structure
    echo "[*] Creating EFI directory structure..."
    mkdir -p "$MOUNT_VOL/EFI/BOOT"
    
    # Copy bootloader to standard path
    echo "[*] Copying BOOTX64.EFI to \\EFI\\BOOT\\..."
    cp bootloader/efi/BOOTX64.EFI "$MOUNT_VOL/EFI/BOOT/"
    
    # Copy kernel to root (bootloader expects it here)
    echo "[*] Copying kernel.elf to root..."
    cp kernel.elf "$MOUNT_VOL/"
    
    # NO startup.nsh - direct boot only!
    echo "[*] Skipping startup.nsh (deterministic direct boot)"
    
    # Sync and unmount
    echo "[*] Syncing filesystem..."
    sync
    
    echo "[*] Unmounting..."
    diskutil unmount "$MOUNT_VOL" >/dev/null 2>&1 || true
    hdiutil detach "$DEV" >/dev/null 2>&1 || true
    
    # Rename to final image
    mv -f "$TMP_DMG" "$IMG"
    
    echo ""
    echo "✅ EFI.img created successfully (deterministic mode)"
    echo ""
    echo "Boot path: \\EFI\\BOOT\\BOOTX64.EFI (UEFI standard)"
    echo "No Shell dependency: ✅"
    echo "Deterministic boot: ✅"
    echo ""
    exit 0
fi

# Linux/Other: Use mtools
if ! command -v mformat >/dev/null 2>&1; then
    echo "ERROR: mtools not found"
    echo "Install: brew install mtools (macOS) or apt install mtools (Linux)"
    exit 1
fi

echo "[*] Using mtools (Linux/fallback)..."

# Create 64MB FAT32 image
echo "[*] Creating 64MB FAT32 image..."
dd if=/dev/zero of=$IMG bs=1M count=64 >/dev/null 2>&1

# Format as FAT32
echo "[*] Formatting as FAT32..."
mformat -i $IMG -F ::

# Create EFI directory structure
echo "[*] Creating EFI directory structure..."
mmd -i $IMG ::EFI
mmd -i $IMG ::EFI/BOOT

# Copy bootloader
echo "[*] Copying BOOTX64.EFI..."
mcopy -i $IMG bootloader/efi/BOOTX64.EFI ::EFI/BOOT/

# Copy kernel
echo "[*] Copying kernel.elf..."
mcopy -i $IMG kernel.elf ::

# NO startup.nsh!
echo "[*] Skipping startup.nsh (deterministic direct boot)"

echo ""
echo "✅ EFI.img created successfully (deterministic mode)"
echo ""
echo "Boot path: \\EFI\\BOOT\\BOOTX64.EFI (UEFI standard)"
echo "No Shell dependency: ✅"
echo "Deterministic boot: ✅"
echo ""
