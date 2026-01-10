# AykenOS - Proje Dizin Yapısı

**Son Güncelleme:** 14 Aralık 2025

```text
AykenOS/
├── ayken-core/                    # Rust tabanlı AykenCore AI sistemi
│   ├── Cargo.toml                 # Workspace konfigürasyonu
│   ├── Cargo.lock                # Bağımlılık kilidi
│   ├── crates/                    # Rust crate'leri
│   │   ├── abdf/                  # ABDF (AykenCore Binary Data Format) kütüphanesi
│   │   │   ├── src/               # ABDF kaynak kodları
│   │   │   └── Cargo.toml         # ABDF crate konfigürasyonu
│   │   ├── abdf-builder/          # ABDF builder araçları
│   │   │   ├── src/               # Builder kaynak kodları
│   │   │   └── Cargo.toml         # Builder crate konfigürasyonu
│   │   └── bcib/                  # BCIB (Binary Compressed Instruction Bundle) kütüphanesi
│   │       ├── src/               # BCIB kaynak kodları
│   │       └── Cargo.toml         # BCIB crate konfigürasyonu
│   ├── docs/                      # Dokümantasyon
│   │   ├── abdf/                  # ABDF spesifikasyonları
│   │   │   ├── abdf-spec.md       # ABDF format spesifikasyonu
│   │   │   └── metadata.md        # Metadata yapısı
│   │   ├── api/                   # API dokümantasyonu
│   │   ├── bcib/                  # BCIB dokümantasyonu
│   │   ├── roadmap/               # Geliştirme yol haritası
│   │   └── runtime/               # Runtime dokümantasyonu
│   └── target/                    # Rust build çıktıları
│
├── bootloader/                    # Çoklu mimari bootloader kaynakları
│   ├── efi/                       # x86_64 UEFI bootloader
│   │   ├── efi_main.c             # UEFI giriş noktası ve sistem başlatma
│   │   ├── ayken_boot.c/.h        # Boot API ve kontrol akışı
│   │   ├── boot.S                 # EFI entry stub (assembly)
│   │   ├── elf_loader.c/.h        # ELF kernel yükleyicisi
│   │   └── paging.c               # Boot-time paging hazırlığı
│   ├── arm64/                     # ARM64 bootloader implementasyonu
│   │   ├── arm_boot.c             # ARM64 boot kontrol akışı
│   │   ├── arm_entry.S            # ARM64 assembly entry point
│   │   └── arm_loader.c           # ARM64 kernel yükleyicisi
│   ├── riscv/                     # RISC-V bootloader implementasyonu
│   │   ├── riscv_entry.S          # RISC-V assembly entry point
│   │   └── riscv_loader.c         # RISC-V kernel yükleyicisi
│   ├── rpi/                       # Raspberry Pi özel bootloader
│   │   ├── rpi_boot.S             # RPi assembly boot kodu
│   │   └── rpi_loader.c           # RPi kernel yükleyicisi
│   └── mcu/                       # Mikrodenetleyici bootloader
│       ├── mcu_loader.c           # MCU kernel yükleyicisi
│       └── mcu_startup.S          # MCU başlangıç assembly kodu
│
├── kernel/                        # Çekirdek kaynakları
│   ├── kernel.c                   # kmain, early/late init ve boot akışı
│   ├── include/                   # Ortak kernel header'ları
│   │   ├── ayken.h                # Ana sistem tanımları
│   │   ├── boot_info.h            # Boot bilgi yapıları
│   │   ├── fs.h                   # Dosya sistemi tanımları
│   │   ├── mm.h                   # Bellek yönetimi tanımları
│   │   ├── proc.h                 # Süreç yapıları
│   │   └── syscall.h              # Sistem çağrısı tanımları
│   ├── arch/                      # Mimariye özel kod
│   │   └── x86_64/                # x86_64 mimarisi
│   │       ├── boot.S             # Kernel boot assembly
│   │       ├── context_switch.asm # Context switching assembly
│   │       ├── cpu.c/.h           # CPU kontrol ve özellikler
│   │       ├── gdt_idt.c/.h       # GDT/IDT yönetimi
│   │       ├── interrupts.c/.h    # Interrupt handler'ları
│   │       ├── pic.c/.h           # PIC (Programmable Interrupt Controller)
│   │       ├── port_io.h          # Port I/O makroları
│   │       └── timer.c/.h         # PIT timer yönetimi
│   ├── mm/                        # Bellek yönetimi
│   │   ├── phys_mem.c             # Fiziksel bellek yöneticisi (bitmap)
│   │   ├── paging.c               # 4-seviyeli paging yönetimi
│   │   └── kheap.c                # Kernel heap allocator
│   ├── drivers/                   # Sürücüler
│   │   ├── console/               # Konsol sürücüsü
│   │   │   ├── fb_console.c/.h    # Framebuffer konsolu (UTF-8/Türkçe)
│   │   │   ├── font8x16.c/.h      # 8x16 bitmap font verisi
│   │   │   └── FB_CONSOLE_USAGE.md # Konsol kullanım kılavuzu
│   │   └── ui/                    # Kullanıcı arayüzü bileşenleri
│   │       ├── logo_animator.c/.h # Logo animasyon motoru
│   │       ├── ayken_logo_128.c/.h # 128x128 logo verisi
│   │       └── ayken_logo_256.c/.h # 256x256 logo verisi
│   ├── fs/                        # Dosya sistemi
│   │   ├── vfs.c                  # Virtual File System (RAM tabanlı tarfs)
│   │   └── devfs.c                # Device File System iskeleti
│   ├── sched/                     # Scheduler
│   │   ├── sched.c/.h             # Kooperatif scheduler (ready/blocked kuyrukları)
│   ├── proc/                      # Süreç yönetimi
│   │   └── proc.c                 # Süreç yapıları ve yönetimi
│   ├── sys/                       # Sistem çağrıları
│   │   └── syscall.c              # Syscall giriş noktası ve yönlendirme
│   └── ai/                        # AI entegrasyonu
│       ├── ayken_core_lm.c/.h     # AykenCoreLM çekirdek implementasyonu
│       ├── ayken_core_lm_format.h # LM format tanımları
│       ├── lm_runtime.c/.h        # Language Model runtime
│       ├── lm_tokenizer.c/.h      # Tokenizer implementasyonu
│       ├── ai_boot_analyzer.c     # Boot süreci AI analizi
│       └── ai_system_tuner.c      # Sistem optimizasyon AI'ı
│
├── linker.ld                      # x86_64 kernel linker script
├── Makefile                       # Ana build sistemi (kernel + bootloader)
├── make_efi_img.sh / .ps1         # EFI.img oluşturma scriptleri
├── make_usb_boot.sh / .ps1        # USB boot disk hazırlama scriptleri
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

- **Bootloader Sistemi:**
  - **UEFI/x86_64:** BOOTX64.EFI, kernel.elf yükleyicisi, paging ve framebuffer desteği
  - **Çoklu Mimari:** ARM64, RISC-V, Raspberry Pi ve MCU bootloader implementasyonları
- **Bellek Yönetimi:** Bitmap tabanlı fiziksel bellek yöneticisi, 4-seviyeli paging ve kernel heap
- **Konsol & UI:** Framebuffer konsolu (UTF-8/Türkçe), splash ekran, logo animasyonu ve progres çubuğu
- **AI Entegrasyonu:**
  - **Kernel AI:** AykenCoreLM çekirdeği, tokenizer, runtime ve sistem optimizasyon AI'ı
  - **Rust AI Core:** ABDF/BCIB formatları, AI model builder araçları
- **Dosya Sistemi:** RAM tabanlı tarfs VFS ve DevFS iskeleti

### Rust Ekosistemi (ayken-core/)

- **ABDF (AykenCore Binary Data Format):** AI model verilerinin binary formatı
- **BCIB (Binary Compressed Instruction Bundle):** Sıkıştırılmış instruction bundle formatı
- **ABDF-Builder:** Model build araçları ve format dönüştürücüleri
- **Workspace Yapısı:** Modüler Rust crate organizasyonu

### Kısmi/Erken İmplementasyon

- **Scheduler & Process:** Kooperatif ready/blocked kuyrukları, context switch assembly kodu
- **Syscalls:** Giriş noktası tanımlı, handler tablosu geliştirilmeli
- **DevFS:** İskelet yapısı mevcut, device node implementasyonu gerekli
- **Çoklu Mimari:** Bootloader kodları hazır, kernel portları geliştirilmeli

### Eksik/Planlanan

- **Kullanıcı Alanı:** Init süreci ve kullanıcı uygulamaları
- **Kernel Portları:** ARM64/RISC-V kernel implementasyonları
- **Sürücü Sistemi:** Disk, ağ ve diğer donanım sürücüleri
- **Kalıcı Depolama:** Gerçek dosya sistemi ve disk I/O
- **AI Model Pipeline:** Rust-C entegrasyonu ve model deployment sistemi

---

## Build Sistemi

### Makefile Hedefleri

- `make all` - Hem kernel.elf hem de BOOTX64.EFI üretir
- `make kernel` - Sadece kernel.elf build eder
- `make bootloader` - Sadece UEFI bootloader build eder
- `make efi-img` - EFI.img disk imajı oluşturur
- `make run` - QEMU ile EFI.img'yi başlatır
- `make clean` - Build çıktılarını temizler

### Toolchain Gereksinimleri

- **Kernel:** x86_64-elf-gcc, x86_64-elf-ld, nasm
- **UEFI Bootloader:** clang (veya x86_64-w64-mingw32-gcc)
- **Rust Components:** cargo, rustc (ayken-core workspace için)

### Çoklu Platform Desteği

- **Linux/Mac:** make_efi_img.sh, make_usb_boot.sh
- **Windows:** make_efi_img.ps1, make_usb_boot.ps1

---

## Teknik Detaylar

### Boot Süreci

1. **UEFI Phase:** BOOTX64.EFI yüklenir, sistem bilgileri toplanır
2. **Kernel Loading:** ELF loader kernel.elf'i yükler, paging hazırlanır
3. **Kernel Init:** kmain çağrılır, early/late init süreçleri
4. **AI Bootstrap:** AykenCoreLM çekirdeği başlatılır, sistem optimizasyonu

### Bellek Yönetimi

- **Physical Memory:** Bitmap tabanlı allocator, UEFI memory map entegrasyonu
- **Virtual Memory:** 4-seviyeli paging (PML4), higher-half kernel mapping
- **Kernel Heap:** Dynamic allocation, fragmentation yönetimi

### AI Entegrasyonu

- **Kernel AI:** C tabanlı LM runtime, tokenizer ve sistem tuner
- **Rust AI Core:** ABDF/BCIB formatları, model builder pipeline
- **Model Storage:** VFS üzerinde binary model dosyaları

### Dosya Sistemi Mimarisi

- **VFS Layer:** Virtual File System abstraction
- **TarFS:** RAM tabanlı tar arşiv dosya sistemi
- **DevFS:** Device file system (/dev node'ları)

---

## Geliştirme Yol Haritası

### Kısa Vadeli (1-2 ay)

1. **Syscall Sistemi:** Tam syscall tablosu ve handler implementasyonu
2. **Process Management:** Init süreci, process creation ve scheduling
3. **DevFS Completion:** Device node'ları ve driver interface
4. **Rust-C Bridge:** AykenCore Rust bileşenlerinin kernel entegrasyonu

### Orta Vadeli (3-6 ay)

1. **Çoklu Mimari:** ARM64 ve RISC-V kernel portları
2. **Sürücü Framework:** Disk, ağ ve input device sürücüleri
3. **Kalıcı Depolama:** Gerçek dosya sistemi (ext2/fat32) desteği
4. **AI Model Pipeline:** Model training ve deployment sistemi

### Uzun Vadeli (6+ ay)

1. **Kullanıcı Alanı:** Shell, utilities ve application framework
2. **Network Stack:** TCP/IP implementasyonu
3. **Graphics System:** GPU sürücüleri ve grafik API'ları
4. **AI OS Features:** Intelligent resource management ve predictive optimization
