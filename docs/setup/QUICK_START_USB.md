# 🚀 AykenOS - USB'den Hızlı Başlangıç

**Son Güncelleme:** 14 Aralık 2025

## Sistem Gereksinimleri

### Build Araçları
- **C/Assembly:** x86_64-elf-gcc, x86_64-elf-ld, nasm
- **UEFI Bootloader:** clang (veya mingw-w64)
- **Rust (Opsiyonel):** cargo, rustc (ayken-core AI bileşenleri için)

## Windows Kullanıcıları

### Otomatik Yöntem (Önerilen)
```powershell
# 1. Projeyi derle
make clean
make all
make efi-img

# 2. PowerShell'i YÖNETİCİ olarak aç

# 3. USB script'ini çalıştır
.\make_usb_boot.ps1

# Script size disk listesini gösterecek
# USB disk numarasını girin (örn: 1, 2, 3)
# EVET yazarak onaylayın
```

### Manuel Yöntem (Rufus)
```powershell
# 1. Projeyi derle
make clean
make all
make efi-img

# 2. Rufus'u indir: https://rufus.ie

# 3. Rufus'ta:
#    - Device: USB belleğinizi seçin
#    - Boot selection: EFI.img
#    - Partition scheme: GPT
#    - Target system: UEFI (non CSM)
#    - File system: FAT32
#    - START

# 4. BIOS'ta Secure Boot'u kapat

# 5. USB'den boot et
```

---

## Linux/Mac Kullanıcıları

### Otomatik Yöntem (Önerilen)
```bash
# 1. Projeyi derle
make clean
make all
make efi-img

# 2. USB script'ini çalıştır
sudo ./make_usb_boot.sh

# Veya device belirterek:
sudo ./make_usb_boot.sh /dev/sdb

# Script size disk listesini gösterecek
# USB device'ı girin (örn: /dev/sdb)
# EVET yazarak onaylayın
```

### Manuel Yöntem (dd)
```bash
# 1. Projeyi derle
make clean
make all
make efi-img

# 2. USB device'ı bul
lsblk                    # Linux
diskutil list            # Mac

# 3. USB'ye yaz (DİKKAT: Doğru device'ı seçin!)
sudo dd if=EFI.img of=/dev/sdX bs=4M status=progress    # Linux
sudo dd if=EFI.img of=/dev/diskX bs=4m                  # Mac

# 4. Sync
sudo sync

# 5. BIOS'ta Secure Boot'u kapat

# 6. USB'den boot et
```

---

## ⚠️ Önemli Uyarılar

1. **Doğru USB'yi Seçin!**
   - Yanlış disk seçimi veri kaybına neden olur
   - Sistem diskini (C:, /dev/sda, /dev/disk0) SEÇMEYİN!

2. **Secure Boot'u Kapatın**
   - BIOS/UEFI'ye girin (F2, F12, DEL)
   - Secure Boot: Disabled
   - UEFI Mode: Enabled
   - CSM/Legacy: Disabled

3. **USB Boyutu**
   - En az 100 MB boş alan gerekli
   - USB 3.0 önerilir (daha hızlı)

---

## 🎯 Başarı Kriterleri

Boot başarılı olduğunda göreceksiniz:
- ✅ AykenOS splash ekranı
- ✅ Animasyonlu logo (128x128 veya 256x256)
- ✅ Progress bar
- ✅ Sağ altta mini-terminal (UTF-8/Türkçe desteği)
- ✅ Boot mesajları ve sistem bilgileri
- ✅ AI sistem başlatma mesajları (AykenCoreLM)

---

## 🐛 Sorun Giderme

### USB boot etmiyor
- Secure Boot kapalı mı kontrol edin
- UEFI mode'da mı kontrol edin
- USB'yi farklı porta takın
- BIOS boot order'ında USB öncelikli mi kontrol edin

### Siyah ekran
- QEMU'da önce test edin: `make run`
- Framebuffer desteği var mı kontrol edin
- Farklı çözünürlük deneyin
- Serial debug çıktısını kontrol edin

### Build hataları
- Toolchain kurulu mu kontrol edin (`x86_64-elf-gcc --version`)
- `make clean` yapıp tekrar deneyin
- `BUILD_FIXES_COMPLETE.md` dosyasına bakın

### Rust bileşenleri (opsiyonel)
- `cd ayken-core && cargo build` ile Rust bileşenlerini test edin
- Rust toolchain güncel mi kontrol edin

### Detaylı yardım
- `USB_BOOT_GUIDE.md` dosyasına bakın
- `DEPENDENCY_FIX_SUMMARY.md` bağımlılık sorunları için

---

## 📚 Daha Fazla Bilgi

### Dokümantasyon
- **Detaylı Kılavuz:** `USB_BOOT_GUIDE.md`
- **Proje Yapısı:** `PROJECT_STRUCTURE.md`
- **Console Kullanımı:** `kernel/drivers/console/FB_CONSOLE_USAGE.md`

### Sorun Giderme
- **Build Düzeltmeleri:** `BUILD_FIXES_COMPLETE.md`
- **Bağımlılık Sorunları:** `DEPENDENCY_FIX_SUMMARY.md`
- **USB Boot Özeti:** `USB_BOOT_SUMMARY.md`

### AI Bileşenleri
- **Rust Workspace:** `ayken-core/` klasörü
- **ABDF Spesifikasyonu:** `ayken-core/docs/abdf/abdf-spec.md`
- **AI Kernel Entegrasyonu:** `kernel/ai/` modülleri

---

**AykenOS - AI destekli işletim sistemi, gerçek donanımda test etmeye hazır!** 🎉
