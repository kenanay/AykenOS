# 🔥 AykenOS - USB'den Gerçek Donanımda Çalıştırma Kılavuzu
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Tarih:** 14 Aralık 2025  
**Platform:** x86_64 UEFI  
**Hedef:** Gerçek bilgisayarda USB'den boot

---

## 📋 Gereksinimler

### Donanım

- ✅ UEFI destekli bilgisayar (2012 sonrası çoğu PC)
- ✅ En az 4 GB USB bellek (FAT32 formatlanacak)
- ✅ x86_64 işlemci

### Yazılım (Windows)

- ✅ Rufus (USB yazdırma aracı) - https://rufus.ie
- ✅ Veya DiskPart (Windows built-in)
- ✅ Derleme araçları (make, gcc, clang)

### Yazılım (Linux/Mac)

- ✅ dd komutu (built-in)
- ✅ Veya Etcher - https://etcher.balena.io
- ✅ Derleme araçları

---

## 🔨 Adım 1: Projeyi Derle

### Windows (PowerShell/CMD)

```powershell
cd C:\AykenOS

# Temiz derleme
make clean
make all

# EFI image oluştur
make efi-img

# Kontrol et
dir EFI.img
```

### Linux/Mac

```bash
cd ~/AykenOS

# Temiz derleme
make clean
make all

# EFI image oluştur
make efi-img

# Kontrol et
ls -lh EFI.img
```

**Beklenen Çıktı:**

```
EFI.img - yaklaşık 10-50 MB boyutunda
```

---

## 💾 Adım 2: USB Belleği Hazırla

### Yöntem A: Rufus ile (Windows - ÖNERİLEN)

1. **Rufus'u İndir ve Çalıştır**

   - https://rufus.ie adresinden indir
   - Yönetici olarak çalıştır

2. **Ayarları Yap**

   ```
   Device: [USB belleğinizi seçin]
   Boot selection: [Disk or ISO image]
   Image: [EFI.img dosyasını seç]
   Partition scheme: GPT
   Target system: UEFI (non CSM)
   File system: FAT32
   Cluster size: 4096 bytes (default)
   ```

3. **START'a Bas**

   - Uyarıyı onayla (USB içeriği silinecek!)
   - İşlem tamamlanana kadar bekle

4. **Doğrula**
   - USB'yi aç
   - `EFI/BOOT/BOOTX64.EFI` dosyası olmalı
   - `kernel.elf` dosyası olmalı

---

### Yöntem B: DiskPart ile (Windows - Manuel)

1. **PowerShell'i Yönetici Olarak Aç**

2. **DiskPart Başlat**

```powershell
diskpart
```

3. **USB Belleği Bul**

```
list disk
```

**ÖNEMLİ:** USB belleğinizin disk numarasını not edin (örn: Disk 2)

4. **USB'yi Temizle ve Formatla**

```
select disk 2          # USB disk numaranızı yazın!
clean
convert gpt
create partition primary
format fs=fat32 quick
assign letter=U        # Boş bir harf seçin
exit
```

5. **EFI.img İçeriğini Kopyala**

```powershell
# 7-Zip veya WinRAR ile EFI.img'yi aç
# Veya PowerShell ile:

# EFI.img'yi mount et (Windows 10+)
Mount-DiskImage -ImagePath "C:\AykenOS\EFI.img"

# Mount edilen sürücüyü bul (örn: E:)
# İçeriği USB'ye kopyala
Copy-Item E:\* U:\ -Recurse -Force

# Unmount
Dismount-DiskImage -ImagePath "C:\AykenOS\EFI.img"
```

---

### Yöntem C: dd ile (Linux/Mac - ÖNERİLEN)

1. **USB Belleği Bul**

```bash
# Linux
lsblk
sudo fdisk -l

# Mac
diskutil list
```

**ÖNEMLİ:** USB belleğinizin device adını not edin (örn: /dev/sdb veya /dev/disk2)

2. **USB'ye Yaz**

```bash
# Linux
sudo dd if=EFI.img of=/dev/sdb bs=4M status=progress
sudo sync

# Mac
sudo dd if=EFI.img of=/dev/disk2 bs=4m
sudo sync
```

**UYARI:** `of=` parametresini yanlış yazmayın! Yanlış disk seçimi veri kaybına neden olur!

3. **Doğrula**

```bash
# Linux
sudo mount /dev/sdb1 /mnt
ls -la /mnt/EFI/BOOT/
sudo umount /mnt

# Mac
# Otomatik mount olur, Finder'dan kontrol et
```

---

### Yöntem D: Etcher ile (Linux/Mac/Windows)

1. **Etcher'ı İndir**

   - https://etcher.balena.io

2. **Kullan**

   - "Flash from file" → EFI.img seç
   - "Select target" → USB belleği seç
   - "Flash!" → Başlat

3. **Doğrula**
   - Otomatik doğrulama yapılır

---

## 🚀 Adım 3: BIOS/UEFI Ayarları

### Boot Öncesi Ayarlar

1. **BIOS/UEFI'ye Gir**

   - Bilgisayarı başlatırken:
     - Dell: F2 veya F12
     - HP: F10 veya ESC
     - Lenovo: F1 veya F2
     - ASUS: F2 veya DEL
     - MSI: DEL
     - Acer: F2

2. **Gerekli Ayarlar**

   ```
   ✅ UEFI Mode: Enabled
   ✅ Secure Boot: Disabled (ÖNEMLİ!)
   ✅ CSM/Legacy: Disabled
   ✅ Fast Boot: Disabled (önerilen)
   ```

3. **Boot Sırasını Ayarla**

   - USB belleği ilk sıraya al
   - Veya boot menüsünden (F12, F8, vb.) USB'yi seç

4. **Kaydet ve Çık**
   - F10 (Save & Exit)

---

## 🎮 Adım 4: Boot Et!

### İlk Boot

1. **USB Takılı Olarak Başlat**

   - Bilgisayarı yeniden başlat
   - Boot menüsünden USB'yi seç

2. **Beklenen Görüntü**

   ```
   [UEFI Firmware]
     ↓
   [BOOTX64.EFI yükleniyor]
     ↓
   [AykenOS Splash Screen]
     ↓
   [Boot mesajları]
     ↓
   [Mini-terminal sağ altta]
   ```

3. **Başarılı Boot Göstergeleri**
   - ✅ Splash ekran görünür
   - ✅ Logo animasyonu çalışır
   - ✅ Progress bar ilerler
   - ✅ Sağ altta mini-terminal açılır
   - ✅ Boot mesajları görünür
   - ✅ Sistem HLT loop'a girer (bekler)

---

## 🐛 Sorun Giderme

### Sorun 1: USB Boot Etmiyor

**Belirtiler:**

- USB boot menüsünde görünmüyor
- "No bootable device" hatası

**Çözümler:**

1. ✅ Secure Boot'u kapat
2. ✅ UEFI mode'da olduğundan emin ol
3. ✅ USB'yi farklı bir porta tak
4. ✅ USB'yi yeniden formatla (FAT32, GPT)
5. ✅ BIOS'u güncelle

---

### Sorun 2: Siyah Ekran

**Belirtiler:**

- Boot ediyor ama ekran siyah
- Hiçbir şey görünmüyor

**Çözümler:**

1. ✅ Framebuffer init kontrol et
2. ✅ Serial port debug ekle
3. ✅ QEMU'da test et önce
4. ✅ Farklı çözünürlük dene

**Debug için serial output ekle:**

```c
// kernel/kernel.c içinde
#include "arch/x86_64/serial.h"

void kmain(ayken_boot_info_t *boot) {
    serial_init();  // COM1 başlat
    serial_print("AykenOS booting...\n");

    fb_console_init(boot);
    // ...
}
```

---

### Sorun 3: Kernel Panic / Crash

**Belirtiler:**

- Boot başlıyor ama çöküyor
- Triple fault
- Reboot loop

**Çözümler:**

1. ✅ Memory map kontrol et
2. ✅ Paging setup kontrol et
3. ✅ Stack overflow kontrol et
4. ✅ GDB ile debug et

**GDB Debug (QEMU üzerinden):**

```bash
# Terminal 1
qemu-system-x86_64 -drive format=raw,file=EFI.img -s -S

# Terminal 2
gdb kernel.elf
(gdb) target remote localhost:1234
(gdb) break kmain
(gdb) continue
```

---

### Sorun 4: "Invalid Signature" Hatası

**Belirtiler:**

- UEFI "Invalid signature detected" diyor

**Çözüm:**

- ✅ Secure Boot'u KAPAT (zorunlu)
- AykenOS henüz imzalı değil

---

### Sorun 5: Framebuffer Çalışmıyor

**Belirtiler:**

- Boot ediyor ama grafik yok
- Text mode'da kalıyor

**Çözümler:**

1. ✅ GOP (Graphics Output Protocol) kontrol et
2. ✅ Farklı video mode dene
3. ✅ UEFI firmware güncellemesi

**Alternatif: Text mode console ekle**

```c
// VGA text mode fallback
if (boot->fb_phys_addr == 0) {
    vga_text_init();  // 80x25 text mode
}
```

---

## 📊 Test Checklist

### Başarılı Boot Kriterleri

- [ ] USB boot menüsünde görünüyor
- [ ] UEFI BOOTX64.EFI yükleniyor
- [ ] Splash ekran gösteriliyor
- [ ] Logo animasyonu çalışıyor
- [ ] Progress bar ilerliyor
- [ ] Mini-terminal açılıyor
- [ ] Boot mesajları görünüyor
- [ ] Sistem stabil (crash yok)
- [ ] Keyboard input alınıyor (gelecekte)

---

## 🔧 Gelişmiş: Multi-Boot Setup

### GRUB ile Birlikte Kullanma

1. **USB'de GRUB Kur**

```bash
sudo grub-install --target=x86_64-efi --efi-directory=/mnt/usb --boot-directory=/mnt/usb/boot --removable
```

2. **GRUB Config Ekle**

```bash
# /mnt/usb/boot/grub/grub.cfg
menuentry "AykenOS" {
    insmod efi_gop
    insmod efi_uga
    chainloader /EFI/BOOT/BOOTX64.EFI
}
```

---

## 📸 Beklenen Görüntü

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│              [AykenOS Logo - Animated]              │
│                                                     │
│                  AykenOS 0.1-dev                    │
│              64-bit Kernel Booting...               │
│                                                     │
│         ████████████████░░░░░░░░░░░░░░░░            │
│                      75%                            │
│                                                     │
│                                                     │
│                                    ┌──────────────┐ │
│                                    │ Boot Log     │ │
│                                    ├──────────────┤ │
│                                    │[OK] CPU init │ │
│                                    │[OK] Memory   │ │
│                                    │[OK] Paging   │ │
│                                    │[OK] Heap     │ │
│                                    │[OK] Drivers  │ │
│                                    │Ready!        │ │
│                                    └──────────────┘ │
└─────────────────────────────────────────────────────┘
```

---

## 🎯 Hızlı Başlangıç (TL;DR)

### Windows

```powershell
# 1. Derle
make clean && make all && make efi-img

# 2. Rufus ile USB'ye yaz
# - EFI.img seç
# - GPT + UEFI seç
# - START

# 3. BIOS'ta Secure Boot'u kapat

# 4. USB'den boot et
```

### Linux

```bash
# 1. Derle
make clean && make all && make efi-img

# 2. USB'ye yaz
sudo dd if=EFI.img of=/dev/sdX bs=4M status=progress
sudo sync

# 3. BIOS'ta Secure Boot'u kapat

# 4. USB'den boot et
```

---

## 📚 Ek Kaynaklar

### Dokümantasyon

- UEFI Spec: https://uefi.org/specifications
- OSDev Wiki: https://wiki.osdev.org/UEFI
- AykenOS Docs: `docs/` klasörü

### Araçlar

- Rufus: https://rufus.ie
- Etcher: https://etcher.balena.io
- QEMU: https://www.qemu.org

### Debug

- Serial Console: COM1 (115200 baud)
- QEMU Monitor: Ctrl+Alt+2
- GDB Remote: port 1234

---

## ⚠️ Önemli Notlar

1. **Veri Kaybı Riski**

   - USB yazdırma işlemi USB içeriğini siler
   - Doğru USB'yi seçtiğinizden emin olun!

2. **Secure Boot**

   - Mutlaka kapatılmalı
   - AykenOS imzalı değil

3. **Uyumluluk**

   - UEFI firmware gerekli (2012+)
   - Legacy BIOS desteklenmiyor

4. **Performans**

   - USB 3.0 önerilir
   - USB 2.0 da çalışır ama yavaş

5. **Güvenlik**
   - Test amaçlı kullanın
   - Production kullanımı için imzalama gerekli

---

**AykenOS USB Boot Guide v1.0**  
_Gerçek donanımda test etmeye hazır!_ 🚀
