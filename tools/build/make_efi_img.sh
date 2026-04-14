#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/tools/lib/ayken_path_contract.sh"
cd "${ROOT}"
ayken_prepare_out_dirs

IMG="${EFI_IMG:-${AYKEN_EFI_IMG}}"
BOOT_EFI_PATH="${BOOT_EFI:-${AYKEN_BOOT_EFI}}"
KERNEL_ELF_PATH="${KERNEL_ELF:-${AYKEN_KERNEL_ELF}}"
IMG_DMG="${IMG}.dmg"

# DEBUG MODE: Always rebuild to ensure fresh image
# TODO: Re-enable deterministic mode after debugging is complete
# if [[ -f "$IMG" ]]; then
#     echo "[*] EFI.img already exists – reuse (deterministic validation)"
#     exit 0
# fi

# Force clean rebuild
rm -f "$IMG" "$IMG_DMG"
mkdir -p "$(dirname "$IMG")"


if [[ "${FORCE_MTOOLS:-0}" != "1" ]] && [[ "$(uname)" == "Darwin" ]] && command -v hdiutil >/dev/null 2>&1; then
    MOUNT_VOL="/Volumes/EFI"

    echo "[*] FAT32 EFI image oluşturuluyor (hdiutil GPTSPUD)..."

    TMP_DMG="${IMG_DMG}"
    DEV=""

    if hdiutil create -size 200m -layout GPTSPUD -partitionType EFI -fs "MS-DOS FAT32" -volname EFI "$TMP_DMG" >/dev/null 2>&1; then
        DEV=$(hdiutil attach -nomount "$TMP_DMG" | head -n1 | awk '{print $1}')
        if [[ -n "$DEV" ]] && diskutil mount "${DEV}s1" >/dev/null 2>&1; then
            mkdir -p "$MOUNT_VOL/EFI/BOOT"
            echo "[BOOTX64.EFI kopyalanıyor]"
            cp "$BOOT_EFI_PATH" "$MOUNT_VOL/EFI/BOOT/"
            echo "[kernel.elf kopyalanıyor]"
            cp "$KERNEL_ELF_PATH" "$MOUNT_VOL/"
            
            # CRITICAL FIX: Generate startup.nsh with correct content (no stray %)
            # Place in root for UEFI shell auto-execution
            echo "[startup.nsh oluşturuluyor (root)]"
            printf '%s\r\n%s\r\n' 'fs0:' '\EFI\BOOT\BOOTX64.EFI' > "$MOUNT_VOL/startup.nsh"
            
            # Also place in EFI/BOOT for fallback
            echo "[startup.nsh oluşturuluyor (EFI/BOOT)]"
            printf '%s\r\n%s\r\n' 'fs0:' '\EFI\BOOT\BOOTX64.EFI' > "$MOUNT_VOL/EFI/BOOT/startup.nsh"

            sync
            diskutil unmount "$MOUNT_VOL" >/dev/null 2>&1 || true
            hdiutil detach "$DEV" >/dev/null 2>&1 || true
            mv -f "$TMP_DMG" "$IMG"
            echo "[*] EFI.img hazır!"
            exit 0
        fi
    fi

    echo "[WARN] hdiutil GPTSPUD yolu başarısız, mtools fallback kullanılacak."
    if [[ -n "${DEV:-}" ]]; then
        hdiutil detach "$DEV" >/dev/null 2>&1 || true
    fi
    rm -f "$TMP_DMG"
fi

echo "[*] FAT32 EFI image oluşturuluyor..."
dd if=/dev/zero of="$IMG" bs=1M count=64 >/dev/null 2>&1

echo "[mformat]"
mformat -i "$IMG" -F ::

echo "[mkdir EFI/BOOT]"
mmd -i "$IMG" ::EFI
mmd -i "$IMG" ::EFI/BOOT

echo "[BOOTX64.EFI kopyalanıyor]"
mcopy -i "$IMG" "$BOOT_EFI_PATH" ::EFI/BOOT/

echo "[kernel.elf kopyalanıyor]"
mcopy -i "$IMG" "$KERNEL_ELF_PATH" ::

echo "[startup.nsh kopyalanıyor]"
# CRITICAL FIX: Generate startup.nsh with correct content (no stray %)
printf '%s\r\n%s\r\n' 'fs0:' '\EFI\BOOT\BOOTX64.EFI' | mcopy -i "$IMG" - ::startup.nsh

echo "[*] EFI.img hazır!"
