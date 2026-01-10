# AykenOS

**Oluşturan:** Kenan AY  
**Oluşturma Tarihi:** 01.01.2026

**Faz 1.5 Durumu:** %95 tamamlandı — Toolchain kurulumu, Ring3 round-trip testleri ve QEMU entegrasyonu başarıyla tamamlandı. Kod temizliği ve dokümantasyon güncellemeleri devam ediyor.

**Mimari Not:** Mevcut kernel POSIX-benzeri syscall’lar ve kernel içi VFS/DevFS/AI runtime içeriyor. Faz 2 hedefi: Ring0 minimal 10 syscall yüzeyi (map/unmap/switch/submit_execution/wait_result/interrupt_return/time_query/cap_bind/cap_revoke/exit), politika (scheduler/VFS/AI) kullanıcı moduna taşınacak.

AykenOS, AI destekli, çoklu mimari işletim sistemi projesidir. UEFI tabanlı x86_64 çekirdeği, ARM64/RISC-V bootloader desteği ve Rust tabanlı AI bileşenleri içerir. EFI bootloader, ELF çekirdeği yükler, framebuffer konsolunu başlatır ve AI destekli sistem optimizasyonu sağlar.

## Özellik Özeti
- **Çoklu Mimari Bootloader:** UEFI/x86_64, ARM64, RISC-V, Raspberry Pi ve MCU bootloader implementasyonları
- **Bellek Yönetimi:** Bitmap tabanlı fiziksel bellek yöneticisi, 4-seviyeli paging ve kernel heap
- **Konsol/UI:** Framebuffer konsolu (UTF-8/Türkçe destekli), renkli çıktı, splash ekran, logo animasyonu ve progres çubuğu
- **AI Entegrasyonu:** 
  - **Kernel AI (mevcut, Faz 2’de user-mode’a taşınacak):** AykenCoreLM çekirdeği, tokenizer, runtime ve sistem optimizasyon AI'ı
  - **Rust AI Core:** ABDF/BCIB formatları, AI model builder araçları
- **Dosya Sistemi:** RAM tabanlı tarfs VFS ve DevFS iskeleti
- **Çekirdek İskeleti:** CPU/GDT/IDT/ISR kurulumu, PIC + PIT sürücüleri, süreç yapısı ve kooperatif scheduler

## Derleme ve Çalıştırma

### Gereksinimler
- **C/Assembly:** `x86_64-elf-gcc`, `x86_64-elf-ld`, `nasm`
- **UEFI Bootloader:** `clang` (veya mingw-w64)
- **Rust (Opsiyonel):** `cargo`, `rustc` (ayken-core AI bileşenleri için)

### Temel Akış
```bash
make clean
make all        # kernel.elf ve BOOTX64.EFI
make efi-img    # EFI.img oluşturur
make run        # QEMU ile EFI.img çalıştır
```

### Rust AI Bileşenleri (Opsiyonel)
```bash
cd ayken-core
cargo build     # ABDF, BCIB ve builder araçları
```

## USB'den Boot Etme
- Windows ve Linux/Mac için otomatik scriptler mevcut:
  - `make_usb_boot.ps1`
  - `make_usb_boot.sh`
- Detaylı kurulum ve sorun giderme için `QUICK_START_USB.md` ve `USB_BOOT_GUIDE.md` dosyalarına bakın.

## Dizin Yapısı
- **Detaylı yapı:** `PROJECT_STRUCTURE.md` dosyasını inceleyin
- **Ana bileşenler:**
  - `kernel/` - C tabanlı çekirdek (x86_64)
  - `bootloader/` - Çoklu mimari bootloader'lar
  - `ayken-core/` - Rust tabanlı AI sistemi
  - USB/EFI imaj scriptleri

## Lisans
AykenOS iki lisans modeli ile dağıtılır:

1) AykenOS Source-Available License (ASAL v1.0)
   – Topluluk ve kişisel kullanım için ücretsizdir.
   – Kod görülebilir, incelenebilir, değiştirilebilir.
   – Ancak ticari kullanım, entegrasyon, SaaS, ürün satışı kesinlikle yasaktır.
   – Ticari kullanım için özel lisans alınması gerekir.

2) AykenOS Commercial License (ACL v1.0)
   – Şirketler, üreticiler, OS geliştiricileri, SaaS platformları
     ve tüm ticari kullanım senaryoları için ücretli lisans sağlar.
   – Kodun ticari ürüne entegre edilmesine izin verir.
   – Binaries dağıtımına izin verir.
   – Kod değişiklikleri kapalı tutulabilir.

Hak Sahibi:
Kenan AY — AykenOS Project

Not: Bu README 03.01.2026 tarihinde güncellenmiştir. Faz 1.5 stabilizasyon fazı tamamlandı - toolchain kurulumu, Ring3 round-trip testleri ve QEMU entegrasyonu başarıyla doğrulandı. Kod temizliği ve dokümantasyon güncellemeleri devam ediyor. Sistem artık Faz 2 mimari dönüşümü için hazır. İlgili belgeler: [PROJECT_STATUS_REPORT.md](docs/phase1/PROJECT_STATUS_REPORT.md), [FAZ_1_COMPLETION_REPORT.md](docs/phase1/FAZ_1_COMPLETION_REPORT.md).
