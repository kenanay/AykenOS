#!/usr/bin/env bash
set -e

IMG=EFI.img

echo "[*] FAT32 EFI image oluşturuluyor..."
dd if=/dev/zero of=$IMG bs=1M count=64  >/dev/null 2>&1

echo "[mformat]"
mformat -i $IMG ::

echo "[mkdir EFI/BOOT]"
mmd -i $IMG ::EFI
mmd -i $IMG ::EFI/BOOT

echo "[BOOTX64.EFI kopyalanıyor]"
mcopy -i $IMG bootloader/efi/BOOTX64.EFI ::EFI/BOOT/

echo "[kernel.elf kopyalanıyor]"
mcopy -i $IMG kernel.elf ::

echo "[*] EFI.img hazır!"
