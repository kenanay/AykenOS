# AykenOS - Proje Dizin Yapısı

**Son Güncelleme:** 30 Kasım 2025

```text
AykenOS/
├── bootloader/                    # UEFI + diğer mimariler için bootloader kaynakları
│   ├── efi/                       # x86_64 UEFI bootloader
│   │   ├── ayken_boot.c/.h        # Boot API ve kontrol akışı
│   │   ├── boot.S                 # EFI entry stub
│   │   ├── efi_main.c             # UEFI giriş noktası
│   │   ├── elf_loader.c/.h        # ELF kernel yükleyicisi
│   │   └── paging.c               # Boot-time paging hazırlığı
│   ├── arm64/                     # ARM64 bootloader iskeleti (C + assembly)
│   ├── riscv/                     # RISC-V bootloader iskeleti
│   ├── rpi/                       # Raspberry Pi bootloader iskeleti
│   └── mcu/                       # Basit MCU başlangıç kodu
│
├── kernel/                        # Çekirdek kaynakları
│   ├── kernel.c                   # kmain, early/late init ve boot akışı
│   ├── include/                   # Ortak kernel header'ları (boot_info, mm, fs, syscall, proc)
│   ├── arch/x86_64/               # Mimariye özel kod (CPU, GDT/IDT, ISR, PIC, PIT, port I/O)
│   ├── mm/                        # Bellek yöneticisi (phys_mem, paging, kheap)
│   ├── drivers/
│   │   ├── console/               # Framebuffer konsolu + font (UTF-8/Türkçe desteği)
│   │   └── ui/                    # Logo animasyonu ve logo veri setleri
│   ├── fs/                        # RAM tabanlı tarfs VFS + devfs iskeleti
│   ├── sched/                     # Kooperatif scheduler kuyruğu (sched.c/h)
│   ├── proc/                      # süreç yapıları (proc.c)
│   ├── sys/                       # syscall giriş noktası (syscall.c)
│   └── ai/                        # AykenCoreLM çekirdeği, tokenizer, runtime, boot/optimizer modülleri
│
├── linker.ld                      # x86_64 kernel linker script
├── Makefile                       # Kernel + UEFI bootloader build, EFI.img üretimi ve QEMU çalıştırma hedefleri
├── make_efi_img.sh / .ps1         # EFI.img oluşturma scriptleri (Linux/Mac/Windows)
├── make_usb_boot.sh / .ps1        # USB'ye yazma scriptleri
├── QUICK_START_USB.md             # USB için hızlı başlangıç kılavuzu
├── USB_BOOT_GUIDE.md              # Detaylı USB boot kılavuzu
├── USB_BOOT_SUMMARY.md            # USB boot durum özeti
├── BUILD_FIXES_COMPLETE.md        # Build düzeltmeleri raporu
├── DEPENDENCY_FIX_SUMMARY.md      # Bağımlılık düzeltme raporu
├── FB_CONSOLE_COMPLETE.md         # Konsol sürücüsü özellik raporu
├── README.md                      # Genel tanıtım ve kullanım
└── PROJECT_STRUCTURE.md           # Bu dosya
```

---

## Durum Özeti

### Çalışan Bileşenler
- **Bootloader (UEFI/x86_64):** BOOTX64.EFI, kernel.elf'i yükler, paging devralır ve framebuffer bilgisiyle `kmain`'i çağırır.
- **Bellek Yönetimi:** Bitmap tabanlı fiziksel bellek yöneticisi, 4-seviyeli paging ve kernel heap inicializasyonu.
- **Konsol & UI:** Framebuffer konsolu (UTF-8/Türkçe), splash ekran, logo animasyonu ve progres çubuğu hazır.
- **AI Entegrasyonu:** AykenCoreLM çekirdeği + tokenizer/runtime kodu çekirdeğe gömülü; dummy model VFS'e ekleniyor.
- **Dosya Sistemi:** RAM tabanlı tarfs VFS başlangıçta kuruluyor ve dummy model dosyasını sağlıyor.

### Kısmi/Erken İmplementasyon
- **Scheduler & Process:** Kooperatif ready/blocked kuyrukları ve context switch kancaları mevcut; proses oluşturma akışı ve görev ekleme (`sched_add_task`) iskelet düzeyinde.
- **Syscalls:** `syscall.c` giriş noktası tanımlı fakat syscall tablosu/handler içerikleri doldurulmamış.
- **DevFS:** `devfs.c` iskelet halinde, gerçek device node'ları eklenmeli.

### Eksik/Planlanan
- Kullanıcı alanı uygulamaları ve init süreci.
- ARM64/RISC-V çekirdek implementasyonları (bootloader iskeleti mevcut).
- Kapsamlı sürücü seti (disk, ağ vb.) ve gerçek dosya sistemi kalıcılığı.

---

## Teknik Notlar
- `make all` hem kernel.elf'i hem de BOOTX64.EFI'yi üretir; `make efi-img` ile EFI.img hazırlanır, `make run` QEMU ile EFI.img'yi başlatır.
- Boot sırasında UEFI memory map'i `phys_mem_init` ile bitmap'e aktarılır; bootloader'dan gelen PML4, `paging_init` ile devralınır.
- VFS, boot sırasında RAM üzerinde tar arşivi oluşturur ve AykenCoreLM dummy modelini `system/aykencorelm/model.bin` yoluna ekler.
- Konsol sürücüsü `fb_console_init` ile framebuffer bilgilerini alır, splash ekran ve mini terminal çizimi yapar.

---

## Sonraki Adımlar (öneri)
1. Proses oluşturma/temel init işlemlerini tamamlayıp scheduler ile entegre etmek.
2. Syscall tablosunu ve kullanıcı alanı giriş noktasını tanımlamak.
3. DevFS ve kalıcı dosya sistemi (örn. tarfs okuma + gerçek blok aygıtı) eklemek.
4. ARM64/RISC-V kernel portlarını başlatmak.
