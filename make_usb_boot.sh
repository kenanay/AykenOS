#!/bin/bash
# ============================================================
# AykenOS USB Boot Creator (Linux/Mac)
# USB belleğe bootable AykenOS yazdırır
# ============================================================

set -e

# Renkler
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Root kontrolü
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}HATA: Bu script root olarak çalıştırılmalı!${NC}"
    echo -e "${YELLOW}Kullanım: sudo $0 [device]${NC}"
    exit 1
fi

echo -e "${CYAN}============================================================${NC}"
echo -e "${CYAN}  AykenOS USB Boot Creator${NC}"
echo -e "${CYAN}============================================================${NC}"
echo ""

# EFI.img kontrolü
if [ ! -f "EFI.img" ]; then
    echo -e "${RED}HATA: EFI.img bulunamadı!${NC}"
    echo -e "${YELLOW}Önce 'make efi-img' komutunu çalıştırın.${NC}"
    exit 1
fi

IMG_SIZE=$(du -h EFI.img | cut -f1)
echo -e "${GREEN}[OK] EFI.img bulundu ($IMG_SIZE)${NC}"
echo ""

# Platform tespiti
if [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="mac"
    LIST_CMD="diskutil list"
    UNMOUNT_CMD="diskutil unmountDisk"
else
    PLATFORM="linux"
    LIST_CMD="lsblk -o NAME,SIZE,TYPE,MOUNTPOINT"
    UNMOUNT_CMD="umount"
fi

# Disk listesi
echo -e "${YELLOW}Mevcut diskler:${NC}"
if [ "$PLATFORM" == "mac" ]; then
    diskutil list | grep -E "^/dev/disk"
else
    lsblk -o NAME,SIZE,TYPE,MOUNTPOINT,MODEL | grep -E "disk|part"
fi

echo ""
echo -e "${RED}UYARI: Seçilen disk tamamen silinecek!${NC}"
echo ""

# Device seçimi
if [ -z "$1" ]; then
    read -p "USB device'ı girin (örn: /dev/sdb veya /dev/disk2): " DEVICE
else
    DEVICE=$1
fi

# Device kontrolü
if [ ! -b "$DEVICE" ]; then
    echo -e "${RED}HATA: $DEVICE bulunamadı veya block device değil!${NC}"
    exit 1
fi

# Güvenlik kontrolü - sistem diskini koruma
if [[ "$DEVICE" == "/dev/sda" ]] || [[ "$DEVICE" == "/dev/nvme0n1" ]] || [[ "$DEVICE" == "/dev/disk0" ]]; then
    echo -e "${RED}HATA: Sistem diski seçilemez!${NC}"
    echo -e "${YELLOW}Lütfen USB belleği seçin.${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Seçilen device: $DEVICE${NC}"

# Device bilgisi
if [ "$PLATFORM" == "mac" ]; then
    diskutil info $DEVICE | grep -E "Device Node|Disk Size|Protocol"
else
    lsblk -o NAME,SIZE,TYPE,MODEL $DEVICE
fi

echo ""

# Onay
read -p "Bu diski silmek istediğinizden emin misiniz? (EVET yazın): " CONFIRM
if [ "$CONFIRM" != "EVET" ]; then
    echo -e "${YELLOW}İşlem iptal edildi.${NC}"
    exit 0
fi

echo ""
echo -e "${CYAN}USB hazırlanıyor...${NC}"

# Unmount
echo -e "${YELLOW}[1/3] Disk unmount ediliyor...${NC}"
if [ "$PLATFORM" == "mac" ]; then
    diskutil unmountDisk $DEVICE 2>/dev/null || true
else
    umount ${DEVICE}* 2>/dev/null || true
fi
echo -e "${GREEN}[OK] Unmount tamamlandı${NC}"
echo ""

# dd ile yaz
echo -e "${YELLOW}[2/3] EFI.img yazılıyor...${NC}"
echo -e "${CYAN}Bu işlem birkaç dakika sürebilir...${NC}"

if [ "$PLATFORM" == "mac" ]; then
    dd if=EFI.img of=$DEVICE bs=4m status=progress
else
    dd if=EFI.img of=$DEVICE bs=4M status=progress
fi

sync
echo ""
echo -e "${GREEN}[OK] Yazma tamamlandı${NC}"
echo ""

# Doğrulama
echo -e "${YELLOW}[3/3] Doğrulanıyor...${NC}"

# Partition'ı mount et
sleep 2

if [ "$PLATFORM" == "mac" ]; then
    # Mac otomatik mount eder
    MOUNT_POINT="/Volumes/AykenOS"
    if [ ! -d "$MOUNT_POINT" ]; then
        MOUNT_POINT=$(diskutil info ${DEVICE}s1 | grep "Mount Point" | cut -d: -f2 | xargs)
    fi
else
    # Linux için manuel mount
    MOUNT_POINT="/mnt/ayken_usb"
    mkdir -p $MOUNT_POINT
    
    # İlk partition'ı bul
    if [[ "$DEVICE" == *"nvme"* ]] || [[ "$DEVICE" == *"mmcblk"* ]]; then
        PART="${DEVICE}p1"
    else
        PART="${DEVICE}1"
    fi
    
    mount $PART $MOUNT_POINT 2>/dev/null || true
fi

# Dosya kontrolü
if [ -f "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI" ] && [ -f "$MOUNT_POINT/kernel.elf" ]; then
    echo -e "${GREEN}[OK] BOOTX64.EFI bulundu${NC}"
    echo -e "${GREEN}[OK] kernel.elf bulundu${NC}"
    
    # Dosya boyutları
    BOOT_SIZE=$(du -h "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI" | cut -f1)
    KERNEL_SIZE=$(du -h "$MOUNT_POINT/kernel.elf" | cut -f1)
    echo -e "${CYAN}  BOOTX64.EFI: $BOOT_SIZE${NC}"
    echo -e "${CYAN}  kernel.elf: $KERNEL_SIZE${NC}"
else
    echo -e "${RED}UYARI: Bazı dosyalar eksik!${NC}"
    ls -la "$MOUNT_POINT/" 2>/dev/null || true
fi

# Unmount
if [ "$PLATFORM" == "linux" ]; then
    umount $MOUNT_POINT 2>/dev/null || true
    rmdir $MOUNT_POINT 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}============================================================${NC}"
echo -e "${GREEN}  USB HAZIR!${NC}"
echo -e "${GREEN}============================================================${NC}"
echo ""
echo -e "${YELLOW}Sonraki adımlar:${NC}"
echo "  1. USB'yi güvenli çıkar"
echo "  2. Hedef bilgisayara tak"
echo "  3. BIOS'ta Secure Boot'u kapat"
echo "  4. USB'den boot et"
echo ""
echo -e "${CYAN}Detaylı bilgi için: USB_BOOT_GUIDE.md${NC}"
echo ""
