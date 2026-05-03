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


# CRITICAL FIX: hdiutil GPTSPUD creates DMG files that QEMU cannot boot from
# The DMG format is not compatible with QEMU's raw disk image expectation
# Always use mtools for QEMU-compatible FAT32 images
# 
# Previous hdiutil approach disabled due to boot failure:
# - hdiutil creates Apple DMG format (not raw disk image)
# - QEMU requires raw disk images with proper MBR/GPT + FAT32
# - mtools creates proper raw FAT32 images that QEMU can boot
#
# if [[ "${FORCE_MTOOLS:-0}" != "1" ]] && [[ "$(uname)" == "Darwin" ]] && command -v hdiutil >/dev/null 2>&1; then
#     ... hdiutil code disabled ...
# fi

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
printf "fs0:\n\\\\EFI\\\\BOOT\\\\BOOTX64.EFI\n" | mcopy -i "$IMG" - ::startup.nsh

echo "[*] EFI.img hazır!"
