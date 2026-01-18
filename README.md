# AykenOS

**Oluşturan:** Kenan AY  
**Oluşturma Tarihi:** 01.01.2026  
**Son Güncelleme:** 15.01.2026

**Proje Durumu:** Faz 3.4 tamamlandı, Faz 3.5 (Semantic CLI Core) aktif geliştirme aşamasında.

---

## 🎯 Proje Vizyonu

AykenOS, yapay zeka destekli, yenilikçi ve çoklu mimari işletim sistemi projesidir. Geleneksel işletim sistemlerinden farklı olarak, **execution-centric** (yürütme merkezli) bir mimari benimser ve **AI-native** (yapay zeka doğal) tasarım prensipleriyle geliştirilmiştir.

### Mimari Dönüşüm

AykenOS, POSIX-benzeri geleneksel işletim sistemi mimarisinden **execution-centric**, **Ring3-empowered** (kullanıcı modu güçlendirilmiş) ve **AI-native** bir mimariye başarıyla dönüştürülmüştür:

- **Ring0 (Kernel Mode):** Sadece 10 temel mekanizma syscall'ı (1000-1009 aralığı)
- **Ring3 (User Mode):** Tüm politika kararları (VFS, DevFS, AI, scheduler) kullanıcı modunda
- **Capability-Based Security:** Yetenek tabanlı güvenlik modeli ile erişim kontrolü
- **BCIB Execution Engine:** Binary Compressed Instruction Bundle formatı ile veri-odaklı yürütme

### Felsefe

AykenOS, klasik işletim sistemi kavramlarını veri-odaklı ve AI-bütünleşik bir yaklaşımla yeniden ele alır. Geleneksel `mkdir`, `cd`, `ls` komutlarıyla sembolize edilen kabuk anlayışı yerine, "AI-native shell + veri işletim sistemi" konseptini hedefler. Dosya sistemi, klasörlerden ziyade anlamlı veri konteynerleri barındıran hibrit bir modelle tasarlanmıştır.

---

## 🚀 Temel Özellikler

### Mimari Yenilikler

- **Execution-Centric Syscall Interface:** Geleneksel POSIX syscall'lar yerine, sadece 10 temel mekanizma syscall'ı
  - `sys_v2_map_memory` (1000): Bellek haritalama
  - `sys_v2_unmap_memory` (1001): Bellek haritalama kaldırma
  - `sys_v2_switch_context` (1002): Bağlam değiştirme
  - `sys_v2_submit_execution` (1003): BCIB yürütme gönderimi
  - `sys_v2_wait_result` (1004): Yürütme sonucu bekleme
  - `sys_v2_interrupt_return` (1005): Kesme dönüşü
  - `sys_v2_time_query` (1006): Zaman sorgulama
  - `sys_v2_capability_bind` (1007): Yetenek bağlama
  - `sys_v2_capability_revoke` (1008): Yetenek iptal etme
  - `sys_v2_exit` (1009): Süreç sonlandırma

- **Ring3 Empowerment:** Tüm politika kararları kullanıcı modunda
  - VFS (Virtual File System) operasyonları Ring3'te
  - DevFS (Device File System) operasyonları Ring3'te
  - AI servisleri Ring3'te
  - Scheduler politika kararları Ring3'te

- **Capability-Based Security:** Yetenek tabanlı güvenlik modeli
  - Token tabanlı erişim kontrolü
  - Granüler izin yönetimi
  - Güvenli kaynak paylaşımı

### Çoklu Mimari Desteği

- **UEFI/x86_64:** Tam özellikli kernel ve bootloader ✅
- **ARM64:** Bootloader implementasyonu (kernel portu devam ediyor) 🔄
- **RISC-V:** Bootloader implementasyonu (kernel portu devam ediyor) 🔄
- **Raspberry Pi:** Özel bootloader desteği ✅
- **MCU:** Mikrodenetleyici bootloader ✅

### Bellek ve Süreç Yönetimi

- **Gelişmiş Bellek Yönetimi:**
  - Bitmap tabanlı fiziksel bellek yöneticisi
  - 4-seviyeli paging (PML4)
  - Kernel heap (kmalloc/kfree)
  - Higher-half kernel mapping

- **Ring3 Kullanıcı Süreçleri:**
  - Tam izolasyon ile kullanıcı modu süreç yürütme
  - Per-process virtual memory
  - Güvenli kernel-user geçişleri

- **Preemptive Multitasking:**
  - Zamanlayıcı tabanlı çoklu görev desteği (100 Hz)
  - Context switching
  - Ready/blocked kuyrukları

### AI ve Veri İşleme

- **Ring3 AI Runtime:** Kullanıcı modunda çalışan AI servisleri
- **ABDF Format:** Ayken Binary Data Format - AI/ML veri desteği
- **BCIB Format:** Binary CLI Instruction Buffer - veri-odaklı komut yapısı
- **Multi-Agent Orchestration:** Çok-ajanlı orkestrasyon sistemi
  - **Planning Engine:** A* ve beam search algoritmaları ile gelişmiş planlama
  - **Coordination Protocols:** Ajan koordinasyonu ve durum senkronizasyonu
  - **Conflict Resolution:** Kaynak çakışma tespiti ve çözümü
  - **Learning & Optimization:** Performans öğrenme ve adaptif optimizasyon

### Kullanıcı Arayüzü

- **Framebuffer Konsolu:**
  - UTF-8/Türkçe desteği
  - Renkli çıktı
  - 8x16 bitmap font

- **Boot UI:**
  - Splash ekran
  - Logo animasyonu (128x128 ve 256x256)
  - Progres çubuğu

- **Semantic CLI:** Doğal dil destekli komut satırı arayüzü (geliştirme aşamasında)

### Dosya Sistemi

- **Ring3 VFS:** Kullanıcı modunda Virtual File System
- **Ring3 DevFS:** Kullanıcı modunda Device File System
- **RAM-based TarFS:** Bellek tabanlı tar arşiv dosya sistemi

---

## � Proje Yapısı

```
AykenOS/
├── kernel/              # C tabanlı çekirdek (x86_64)
│   ├── arch/           # Mimariye özel kod (x86_64)
│   │   └── x86_64/    # x86_64 assembly ve C kodu
│   ├── sys/            # Sistem çağrıları (v2 interface)
│   ├── mm/             # Bellek yönetimi
│   ├── sched/          # Zamanlayıcı
│   ├── proc/           # Süreç yönetimi
│   ├── fs/             # Dosya sistemi (minimal stubs)
│   └── drivers/        # Sürücüler (konsol, UI)
│
├── bootloader/         # Çoklu mimari bootloader'lar
│   ├── efi/           # UEFI x86_64 bootloader
│   ├── arm64/         # ARM64 bootloader
│   ├── riscv/         # RISC-V bootloader
│   ├── rpi/           # Raspberry Pi bootloader
│   └── mcu/           # Mikrodenetleyici bootloader
│
├── userspace/          # Ring3 kullanıcı modu bileşenleri
│   ├── libayken/      # Ring3 VFS/DevFS/Scheduler implementasyonları
│   ├── ai-runtime/    # AI runtime servisleri
│   ├── bcib-runtime/  # BCIB execution engine
│   ├── orchestration/ # Multi-agent orchestration
│   ├── semantic-cli/  # Semantic command-line interface
│   └── dsl-parser/    # Domain-specific language parser
│
├── ayken-core/         # Rust tabanlı AI core sistemi
│   └── crates/        # Rust crate'leri
│       ├── abdf/      # Ayken Binary Data Format
│       ├── abdf-builder/ # ABDF builder araçları
│       └── bcib/      # Binary CLI Instruction Buffer
│
├── docs/               # Dokümantasyon
│   ├── phase1/        # Faz 1 raporları
│   ├── phase2/        # Faz 2 raporları ve spesifikasyonlar
│   ├── development/   # Geliştirme kılavuzları
│   ├── setup/         # Kurulum kılavuzları
│   └── roadmap/       # Yol haritası
│
└── tools/              # Geliştirme araçları

Detaylı yapı için: docs/development/PROJECT_STRUCTURE.md
```

---

## 🛠️ Derleme ve Çalıştırma

### Gereksinimler

**C/Assembly Toolchain:**
- `x86_64-elf-gcc` - Cross-compiler
- `x86_64-elf-ld` - Linker
- `nasm` - Assembler

**UEFI Bootloader:**
- `clang` (veya `x86_64-w64-mingw32-gcc`)

**Rust (Opsiyonel):**
- `cargo` - Rust build tool
- `rustc` - Rust compiler
- Ayken-core AI bileşenleri için gerekli

**Test ve Emülasyon:**
- `qemu-system-x86_64` - QEMU emülatör

### Temel Derleme Akışı

```bash
# Temizlik
make clean

# Kernel ve bootloader derle
make all        # kernel.elf ve BOOTX64.EFI oluşturur

# EFI disk imajı oluştur
make efi-img    # EFI.img oluşturur

# QEMU ile test et
make run        # QEMU ile EFI.img çalıştır
```

### Rust AI Bileşenleri (Opsiyonel)

```bash
cd ayken-core

# Tüm crate'leri derle
cargo build

# Testleri çalıştır
cargo test

# Belirli bir crate'i derle
cargo build -p abdf
cargo build -p bcib
cargo build -p abdf-builder
```

### Userspace Bileşenleri

```bash
cd userspace

# Tüm Rust bileşenlerini derle
cargo build

# Testleri çalıştır
cargo test

# Orchestration testleri
cargo test -p orchestration

# GATE D validation
cargo test -p orchestration gate_d_exit_criteria_validation
```

---

## 💾 USB'den Boot Etme

AykenOS, fiziksel donanımda test edilmek üzere USB'den boot edilebilir.

### Otomatik Scriptler

**Windows:**
```powershell
.\make_usb_boot.ps1
```

**Linux/macOS:**
```bash
./make_usb_boot.sh
```

### Detaylı Kılavuzlar

- **Hızlı Başlangıç:** [docs/setup/QUICK_START_USB.md](docs/setup/QUICK_START_USB.md)
- **Detaylı Kılavuz:** [docs/setup/USB_BOOT_GUIDE.md](docs/setup/USB_BOOT_GUIDE.md)
- **Platform Kılavuzları:**
  - [docs/setup/WINDOWS_WSL_SETUP_GUIDE.md](docs/setup/WINDOWS_WSL_SETUP_GUIDE.md)
  - [docs/setup/LINUX_SETUP_GUIDE.md](docs/setup/LINUX_SETUP_GUIDE.md)
  - [docs/setup/MACOS_SETUP_GUIDE.md](docs/setup/MACOS_SETUP_GUIDE.md)

---

## 📊 Proje Durumu

### Tamamlanan Fazlar

- ✅ **Faz 1:** Çekirdek temelinin tamamlanması (%100)
  - Bootloader ve ELF loader
  - Bellek yönetimi (physical, virtual, heap)
  - CPU/GDT/IDT/ISR kurulumu
  - Temel sürücüler (PIC, PIT, konsol)

- ✅ **Faz 1.5:** Stabilizasyon ve Ring3 doğrulaması (%100)
  - Toolchain kurulumu ve doğrulaması
  - Ring3 round-trip testleri
  - QEMU entegrasyon testleri
  - Kod temizliği ve tutarlılık

- ✅ **Faz 2:** Execution-centric mimari dönüşümü (%100)
  - 10 syscall hedefine ulaşıldı (1000-1009)
  - Ring3 VFS/DevFS implementasyonu
  - BCIB execution engine
  - Capability-based security

- ✅ **Faz 2.5:** Legacy kod temizliği (%100)
  - POSIX syscall'ların kaldırılması
  - Ring0 policy kod temizliği
  - Stub fonksiyonların minimizasyonu

- 🔄 **Faz 3.4:** Multi-Agent Orchestration (tamamlandı ✅)
  - ✅ **GATE A:** Orchestration Core (tamamlandı)
  - ✅ **GATE B:** Agent Pool Management (tamamlandı)
  - ✅ **GATE C:** Hardware Intelligence (tamamlandı)
  - ✅ **GATE D:** Advanced Planning & Coordination (tamamlandı)
  - ✅ **GATE E:** Security & Integration (tamamlandı)

- 🔄 **Faz 3.5:** Semantic CLI Core (aktif geliştirme)
  - ✅ **GATE A:** Parsing Foundation (tamamlandı)
  - 🔄 **GATE B:** BCIB Refactoring & Core Operations (devam ediyor)
    - ✅ **Task 5-8:** Architectural Requirements (AR-1 to AR-4) (tamamlandı)
    - ✅ **Task 9:** Validator Implementation (tamamlandı - güçlü onay)
    - ✅ **Task 10:** Transformer Implementation (tamamlandı - güçlü onay)
    - 🔄 **Task 11:** Context Manager Implementation (devam ediyor)

### Önemli Kilometre Taşları

| Hedef | Durum | Açıklama |
|-------|-------|----------|
| 10 Syscall Hedefi | ✅ | 1000-1009 aralığında execution-centric syscall'lar |
| Ring3 VFS/DevFS | ✅ | Kullanıcı modunda tam implementasyon |
| BCIB Execution Engine | ✅ | Ring3'te operasyonel |
| Capability Security | ✅ | Yetenek tabanlı güvenlik aktif |
| Multi-Agent System | ✅ | Planlama, koordinasyon, çakışma çözümü ve öğrenme |
| Planning Engine | ✅ | A* ve beam search algoritmaları |
| Coordination Protocols | ✅ | Ajan senkronizasyonu ve mesajlaşma |
| Conflict Resolution | ✅ | Kaynak çakışma tespiti ve çözümü |
| Learning & Optimization | ✅ | Performans öğrenme ve adaptif optimizasyon |
| Semantic CLI Core | 🔄 | AST → BCIB transformer, validator, context manager |
| Architectural Refactoring | ✅ | AR-1 to AR-4 requirements (güçlü onay) |
| DSL Parser & Validator | ✅ | Natural language-inspired DSL with BCIB output |

### Performans Metrikleri

**Boot Süresi:**
- UEFI → Kernel entry: ~100ms
- Early init: ~50ms
- Late init: ~50ms
- İlk süreç yürütme: ~10ms
- **Toplam:** ~200ms

**Scheduling:**
- Timer frekansı: 100 Hz (10ms tick)
- Context switch: ~1-2μs
- Syscall latency: ~500ns-1μs

**Multi-Agent Orchestration:**
- Plan oluşturma: < 1s
- Durum senkronizasyonu (10x): < 100ms
- Kaynak istekleri (100x): < 100ms

**Semantic CLI Core:**
- DSL parsing: < 1ms (100x hedef aşımı)
- AST → BCIB transformation: < 1ms (50x hedef aşımı)
- BCIB validation: < 1ms (10x hedef aşımı)
- End-to-end latency: < 5ms

---

## 📚 Dokümantasyon

### Proje Raporları

- **Genel Durum:** [docs/phase1/PROJECT_STATUS_REPORT.md](docs/phase1/PROJECT_STATUS_REPORT.md)
- **Faz 1 Tamamlanma:** [docs/phase1/FAZ_1_COMPLETION_REPORT.md](docs/phase1/FAZ_1_COMPLETION_REPORT.md)
- **Faz 2.5 Tamamlanma:** [PHASE2_5_COMPLETION_REPORT.md](PHASE2_5_COMPLETION_REPORT.md)
- **GATE D Validation:** [GATE_D_VALIDATION_COMPLETION_REPORT.md](GATE_D_VALIDATION_COMPLETION_REPORT.md)
- **Faz 3.5 Semantic CLI:** [.kiro/specs/phase3-5-semantic-interaction/](/.kiro/specs/phase3-5-semantic-interaction/)
  - **Requirements:** [requirements.md](/.kiro/specs/phase3-5-semantic-interaction/requirements.md)
  - **Design:** [design.md](/.kiro/specs/phase3-5-semantic-interaction/design.md)
  - **Tasks:** [tasks.md](/.kiro/specs/phase3-5-semantic-interaction/tasks.md)

### Teknik Dokümantasyon

- **Proje Yapısı:** [docs/development/PROJECT_STRUCTURE.md](docs/development/PROJECT_STRUCTURE.md)
- **Ring3 Implementasyon:** [docs/development/RING3_IMPLEMENTATION.md](docs/development/RING3_IMPLEMENTATION.md)
- **Syscall Geçiş Kılavuzu:** [docs/development/SYSCALL_TRANSITION_GUIDE.md](docs/development/SYSCALL_TRANSITION_GUIDE.md)
- **DevFS Implementasyon:** [docs/development/DEVFS_IMPLEMENTATION.md](docs/development/DEVFS_IMPLEMENTATION.md)

### Yol Haritası

- **Geliştirme Yol Haritası:** [AykenOS Geliştirme Yol Haritası.txt](AykenOS%20Geliştirme%20Yol%20Haritası.txt)
- **Faz 2 Genel Bakış:** [docs/phase2/FAZ_2_OVERVIEW.md](docs/phase2/FAZ_2_OVERVIEW.md)

### Kurulum Kılavuzları

- **Windows WSL:** [docs/setup/WINDOWS_WSL_SETUP_GUIDE.md](docs/setup/WINDOWS_WSL_SETUP_GUIDE.md)
- **Linux:** [docs/setup/LINUX_SETUP_GUIDE.md](docs/setup/LINUX_SETUP_GUIDE.md)
- **macOS:** [docs/setup/MACOS_SETUP_GUIDE.md](docs/setup/MACOS_SETUP_GUIDE.md)
- **Çoklu Platform:** [docs/setup/MULTI_PLATFORM_DEVELOPMENT_GUIDE.md](docs/setup/MULTI_PLATFORM_DEVELOPMENT_GUIDE.md)

---

## 📜 Lisans

AykenOS iki lisans modeli ile dağıtılır:

### 1. AykenOS Source-Available License (ASAL v1.0)

**Topluluk ve kişisel kullanım için ücretsizdir.**

- ✅ Kod görülebilir, incelenebilir, değiştirilebilir
- ✅ Eğitim ve araştırma amaçlı kullanım
- ✅ Kişisel projeler ve deneyler
- ❌ Ticari kullanım, entegrasyon, SaaS, ürün satışı **kesinlikle yasaktır**
- ❌ Ticari kullanım için özel lisans alınması gerekir

### 2. AykenOS Commercial License (ACL v1.0)

**Ticari kullanım için ücretli lisans.**

- ✅ Şirketler, üreticiler, OS geliştiricileri için
- ✅ SaaS platformları ve ticari ürünler için
- ✅ Kodun ticari ürüne entegre edilmesi
- ✅ Binary dağıtımı
- ✅ Kod değişiklikleri kapalı tutulabilir

**Hak Sahibi:** Kenan AY — AykenOS Project

---

## 🎯 Gelecek Hedefler

### Kısa Vadeli (Faz 3.5 - 3.6)

- ✅ **Architectural Refactoring (AR-1 to AR-4):** BCIB instruction graph model (tamamlandı)
- ✅ **AST → BCIB Transformer:** Semantic preservation with performance (tamamlandı)
- ✅ **BCIB Validator:** Contextual capabilities and register tracking (tamamlandı)
- 🔄 **Context Manager:** Read-only context loading and caching (devam ediyor)
- **Query Operations:** Filter evaluation and result formatting
- **System Operations:** Status and agents with contextual capabilities
- **Debug Operations:** Explain, dry-run, and history with sequence references
- **Minimal REPL:** Interactive command interface

### Orta Vadeli (Faz 4)

- **Çoklu Mimari Kernel:** ARM64 ve RISC-V kernel portları
- **Gerçek Donanım Testleri:** Raspberry Pi ve diğer platformlarda test
- **Grafik Arayüzü:** OpenGL tabanlı dashboard ve UI
- **Network Stack:** Temel TCP/IP implementasyonu

### Uzun Vadeli (Faz 5+)

- **Tam AI Entegrasyonu:** TinyLLM modelleri ve AI servisleri
- **Veri-Odaklı Dosya Sistemi:** Tiplenmiş veri konteynerleri
- **Hibrit Shell:** AI-native komut yorumlama
- **Ekosistem Geliştirme:** Üçüncü parti uygulama desteği

---

## 🤝 Katkıda Bulunma

AykenOS açık kaynak bir projedir ve katkılara açıktır. Ancak, ticari kullanım için lisans gereklidir.

**Katkı Yapmak İçin:**
1. Projeyi fork edin
2. Feature branch oluşturun
3. Değişikliklerinizi commit edin
4. Pull request gönderin

**İletişim:**
- Proje Sahibi: Kenan AY
- Proje: AykenOS

---

## 🌟 Öne Çıkan Özellikler

### Neden AykenOS?

1. **Yenilikçi Mimari:** Execution-centric paradigma ile geleneksel OS tasarımından ayrılır
2. **AI-Native Tasarım:** Yapay zeka, sistemin merkezinde, eklenti değil
3. **Ring3 Empowerment:** Politika kararları kullanıcı modunda, güvenlik ve esneklik
4. **Capability-Based Security:** Modern güvenlik modeli
5. **Multi-Agent Orchestration:** Gelişmiş ajan koordinasyonu ve öğrenme
6. **Çoklu Mimari:** x86_64, ARM64, RISC-V desteği
7. **Veri-Odaklı:** Dosya sistemi yerine veri konteynerleri
8. **Açık ve Şeffaf:** Kaynak kodu görülebilir ve incelenebilir

### Teknik Mükemmellik

- **Temiz Mimari:** Ring0/Ring3 ayrımı net ve tutarlı
- **Minimal Kernel:** Sadece mekanizma, politika yok
- **Modüler Tasarım:** Bileşenler bağımsız ve test edilebilir
- **Performans Odaklı:** Düşük latency, yüksek throughput
- **Test Edilmiş:** Kapsamlı test suite ve validation

---

**Son Güncelleme:** 15 Ocak 2026 - Task 10 Transformer Implementation Tamamlandı (Güçlü Onay)  
**Güncelleyen:** Kenan AY

AykenOS, geleneksel işletim sistemi paradigmalarını sorgulayan ve AI-native bir gelecek için temel oluşturan yenilikçi bir projedir. Execution-centric mimari, Ring3 empowerment ve multi-agent orchestration özellikleriyle, modern işletim sistemlerine farklı bir bakış açısı sunmaktadır.

**© 2026 Kenan AY - AykenOS Project**
# AykenOS
