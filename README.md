# AykenOS

AykenOS, UEFI tabanlı x86_64 mimarisi için tasarlanmış deneysel bir işletim sistemi çekirdeğidir. EFI bootloader, ELF biçimindeki çekirdeği yükler, framebuffer konsolunu başlatır ve ilk init aşamalarını çalıştırır. Çekirdek; bellek yöneticisi, temel süreç/scheduler iskeleti, basit bir RAM tabanlı VFS ve AI alt sistemi için yerleşik altyapı içerir.

## Özellik Özeti
- **UEFI Bootloader:** BOOTX64.EFI ile kernel.elf yüklenir, paging devralınır ve `kmain` çağrılır.
- **Bellek Yönetimi:** Fiziksel bellek bitmap yöneticisi, 4-seviyeli sayfalama ve kernel heap bulunur.
- **Konsol/UI:** Framebuffer konsolu (UTF-8/Türkçe destekli), renkli çıktı, splash ekran, logo animasyonu ve progres çubuğu.
- **Çekirdek İskeleti:** CPU/GDT/IDT/ISR kurulumu, PIC + PIT sürücüleri, süreç yapısı ve kooperatif scheduler iskeleti.
- **Dosya Sistemi:** Boot aşamasında RAM içine alınan basit tarfs tabanlı VFS; dummy AykenCoreLM modeli dosyası üretir.
- **AI Alt Sistemi:** AykenCoreLM çekirdeği, tokenizer ve runtime kodu çekirdeğe gömülü durumdadır.

## Derleme ve Çalıştırma
- Araçlar: `x86_64-elf-gcc`, `x86_64-elf-ld`, `nasm`, EFI tarafı için `clang` (veya uygun mingw/gnu-efi toolchain).
- Temel akış:
  ```bash
  make clean
  make all        # kernel.elf ve BOOTX64.EFI
  make efi-img    # EFI.img oluşturur
  make run        # QEMU ile EFI.img çalıştır
  ```

## USB'den Boot Etme
- Windows ve Linux/Mac için otomatik scriptler mevcut:
  - `make_usb_boot.ps1`
  - `make_usb_boot.sh`
- Detaylı kurulum ve sorun giderme için `QUICK_START_USB.md` ve `USB_BOOT_GUIDE.md` dosyalarına bakın.

## Dizin Yapısı
- Genel dizin özeti için `PROJECT_STRUCTURE.md` dosyasını inceleyin.
- Başlıca kaynaklar `kernel/`, `bootloader/` ve USB/EFI imaj scriptleridir.

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
