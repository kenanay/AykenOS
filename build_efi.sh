#!/bin/bash
# EFI.img oluşturma scripti - UDRW ile RAW

set -e

echo "EFI.img oluşturuluyor..."

# Eski dosyaları temizle
rm -f EFI.img EFI.dmg EFI_raw.dmg

# 64MB FAT32 disk image oluştur
hdiutil create -size 64m -fs MS-DOS -volname "EFI" -o EFI.dmg

# Mount et
hdiutil attach EFI.dmg >/dev/null

# AppleDouble dosyalarını engelle
export COPYFILE_DISABLE=1

# EFI dizin yapısını oluştur ve dosyaları kopyala
mkdir -p /Volumes/EFI/EFI/BOOT
cp -X bootloader/efi/BOOTX64.EFI /Volumes/EFI/EFI/BOOT/
cp -X kernel.elf /Volumes/EFI/EFI/BOOT/
cp -X kernel.elf /Volumes/EFI/  # Root'a da kopyala (bootloader için)

# startup.nsh oluştur (otomatik boot için)
echo "FS0:" > /Volumes/EFI/startup.nsh
echo "cd EFI\BOOT" >> /Volumes/EFI/startup.nsh
echo "BOOTX64.EFI" >> /Volumes/EFI/startup.nsh

# AppleDouble ve .DS_Store dosyalarını temizle
find /Volumes/EFI -name "._*" -delete 2>/dev/null || true
find /Volumes/EFI -name ".DS_Store" -delete 2>/dev/null || true
rm -rf /Volumes/EFI/.fseventsd 2>/dev/null || true
rm -rf /Volumes/EFI/.Spotlight-V100 2>/dev/null || true
rm -rf /Volumes/EFI/.Trashes 2>/dev/null || true

# Unmount
hdiutil detach /Volumes/EFI >/dev/null

# UDRW formatına çevir (RAW read-write)
hdiutil convert EFI.dmg -format UDRW -o EFI_raw
mv EFI_raw.dmg EFI.img
rm -f EFI.dmg

echo "EFI.img hazır!"
echo "Kernel hash kontrolü:"
shasum -a 256 kernel.elf
