#!/usr/bin/env bash
set -e

IMG=EFI.img

# DEBUG MODE: Always rebuild to ensure fresh image
# TODO: Re-enable deterministic mode after debugging is complete
# if [[ -f "$IMG" ]]; then
#     echo "[*] EFI.img already exists – reuse (deterministic validation)"
#     exit 0
# fi

# Force clean rebuild
rm -f "$IMG" "$IMG.dmg"


if [[ "${FORCE_MTOOLS:-0}" != "1" ]] && [[ "$(uname)" == "Darwin" ]] && command -v hdiutil >/dev/null 2>&1; then
    MOUNT_VOL="/Volumes/EFI"

    echo "[*] FAT32 EFI image oluşturuluyor (hdiutil GPTSPUD)..."

    TMP_DMG="${IMG}.dmg"
    DEV=""

    if hdiutil create -size 200m -layout GPTSPUD -partitionType EFI -fs "MS-DOS FAT32" -volname EFI "$TMP_DMG" >/dev/null 2>&1; then
        DEV=$(hdiutil attach -nomount "$TMP_DMG" | head -n1 | awk '{print $1}')
        if [[ -n "$DEV" ]] && diskutil mount "${DEV}s1" >/dev/null 2>&1; then
            mkdir -p "$MOUNT_VOL/EFI/BOOT"
            echo "[BOOTX64.EFI kopyalanıyor]"
            cp bootloader/efi/BOOTX64.EFI "$MOUNT_VOL/EFI/BOOT/"
            echo "[kernel.elf kopyalanıyor]"
            cp kernel.elf "$MOUNT_VOL/"
            
            # CRITICAL FIX: Generate startup.nsh with correct content (no stray %)
            echo "[startup.nsh oluşturuluyor]"
            cat > "$MOUNT_VOL/startup.nsh" <<'EOF'
fs0:
\EFI\BOOT\BOOTX64.EFI
EOF

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
dd if=/dev/zero of=$IMG bs=1M count=64  >/dev/null 2>&1

echo "[mformat]"
mformat -i $IMG -F ::

echo "[mkdir EFI/BOOT]"
mmd -i $IMG ::EFI
mmd -i $IMG ::EFI/BOOT

echo "[BOOTX64.EFI kopyalanıyor]"
mcopy -i $IMG bootloader/efi/BOOTX64.EFI ::EFI/BOOT/

echo "[kernel.elf kopyalanıyor]"
mcopy -i $IMG kernel.elf ::

echo "[startup.nsh kopyalanıyor]"
# CRITICAL FIX: Generate startup.nsh with correct content (no stray %)
printf "fs0:\n\\\\EFI\\\\BOOT\\\\BOOTX64.EFI\n" | mcopy -i $IMG - ::startup.nsh

echo "[*] EFI.img hazır!"
