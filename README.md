# AykenOS
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Copyright (c) 2026 Kenan AY. All rights reserved.**

[![License: Proprietary](https://img.shields.io/badge/License-Proprietary-red.svg)](LICENSE)
[![Author: Kenan AY](https://img.shields.io/badge/Author-Kenan%20AY-blue.svg)](mailto:kenanay@example.com)
[![Status: Protected](https://img.shields.io/badge/Status-Protected-orange.svg)](#-important-legal-notice)

**Oluşturan:** Kenan AY  
**Oluşturma Tarihi:** 01.01.2026  
**Son Güncelleme:** 21.02.2026

**Proje Durumu:** Core OS Phase 4.4 TAMAMLANDI ✅ | Core OS Phase 4.5 STABILIZATION IN PROGRESS 🚧 | Constitutional Rule System Phases 1-12 tamamlandı ✅ | Architecture Freeze ACTIVE ✅  
**Boot/Kernel Bring-up:** UEFI→kernel handoff doğrulandı ✅ | Ring3 execution model operasyonel ✅ | Syscall roundtrip doğrulandı ✅ | IRQ-tail preempt doğrulama hattı mevcut ✅
**Scheduler Notu (Kod Gerçekliği):** Ring0->Ring3 doğrudan C çağrısı kapalı; scheduler seçim yolu şu an kernel içi ready-queue mekanik akışla çalışıyor (mailbox/stage-next sözleşmesi dokümanda hedef durumdur).

⚠️ **CI Mode:** `ci-freeze` workflow varsayılan olarak **CONSTITUTIONAL** modda çalışır (`PERF_BASELINE_MODE=constitutional`); baseline-init akışında ve yerel denemelerde **PROVISIONAL** yol kullanılabilir. Ayrıntı: [Constitutional CI Mode](docs/operations/CONSTITUTIONAL_CI_MODE.md), [Provisional CI Mode](docs/operations/PROVISIONAL_CI_MODE.md).

---

## 🔒 IMPORTANT LEGAL NOTICE

This software is **proprietary and confidential**. All rights reserved by Kenan AY.

### ⚖️ Usage Restrictions:
- ✅ **Educational viewing** permitted for learning purposes
- ❌ **Commercial use** prohibited without license
- ❌ **Modification** prohibited without written permission  
- ❌ **Distribution** prohibited without authorization
- ❌ **Reverse engineering** prohibited

### 📧 Licensing Contact:
For commercial licensing, partnerships, or permissions:
- **Email**: kenanay@example.com
- **Subject**: "AykenOS Licensing Inquiry"

---

## 🎯 Proje Vizyonu

AykenOS, yapay zeka destekli, yenilikçi ve çoklu mimari işletim sistemi projesidir. Geleneksel işletim sistemlerinden farklı olarak, **execution-centric** (yürütme merkezli) bir mimari benimser ve **AI-native** (yapay zeka doğal) tasarım prensipleriyle geliştirilmiştir.

### Mimari Dönüşüm

AykenOS, POSIX-benzeri geleneksel işletim sistemi mimarisinden **execution-centric**, **Ring3-empowered** (kullanıcı modu güçlendirilmiş) ve **AI-native** bir mimariye başarıyla dönüştürülmüştür:

- **Ring0 (Kernel Mode):** 11 execution-centric mekanizma syscall'ı (1000-1010 aralığı)
- **Ring3 (User Mode):** Tüm politika kararları (VFS, DevFS, AI, scheduler) kullanıcı modunda
- **Capability-Based Security:** Yetenek tabanlı güvenlik modeli ile erişim kontrolü
- **BCIB Execution Engine:** Binary Compressed Instruction Bundle formatı ile veri-odaklı yürütme

### Felsefe

AykenOS, klasik işletim sistemi kavramlarını veri-odaklı ve AI-bütünleşik bir yaklaşımla yeniden ele alır. Geleneksel `mkdir`, `cd`, `ls` komutlarıyla sembolize edilen kabuk anlayışı yerine, "AI-native shell + veri işletim sistemi" konseptini hedefler. Dosya sistemi, klasörlerden ziyade anlamlı veri konteynerleri barındıran hibrit bir modelle tasarlanmıştır.

---

## 🚀 Temel Özellikler

### Mimari Yenilikler

- **Execution-Centric Syscall Interface:** Geleneksel POSIX syscall'lar yerine, 11 temel mekanizma syscall'ı
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
  - `sys_v2_debug_putchar` (1010): Ring3 debug heartbeat

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
├── ayken/              # Rust tabanlı constitutional rule system (AykenOS geliştirme aracı)
│   ├── ahs/           # Architecture Health Score system
│   ├── ahts/          # Architecture Health Trend Score system
│   ├── allow/         # Allow attribute parsing and validation
│   ├── audit/         # Immutable audit trail system
│   ├── cli/           # Command-line interface (ayken check)
│   ├── constitution/  # Constitutional policy constants & integration limits
│   ├── decision/      # Constitutional decision tree
│   ├── diagnostic/    # Diagnostic output (VS Code, CI, text)
│   ├── escalation/    # Allow escalation detection
│   ├── exception/     # Exception hierarchy (allow/waiver)
│   ├── explain/       # Rule explanation engine
│   ├── lifecycle/     # Waiver lifecycle management
│   ├── mars/          # Module-level Architecture Risk Score system (risk, CI, VS Code, dashboard)
│   ├── phase/         # Phase detection and matrix
│   ├── rules/         # Constitutional rule definitions
│   ├── scanner/       # Rule violation scanning
│   ├── steering/      # Steering files management
│   ├── vscode/        # VS Code integration
│   └── waiver/        # Waiver system with expiry and renewal
│
├── docs/               # Dokümantasyon
│   ├── phase1/        # Faz 1 raporları
│   ├── phase2/        # Faz 2 raporları ve spesifikasyonlar
│   ├── development/   # Geliştirme kılavuzları
│   ├── setup/         # Kurulum kılavuzları
│   ├── roadmap/       # Yol haritası + freeze workflow
│   ├── rfc/           # RFC şablonları
│   ├── waivers/       # Waiver registry + template
│   └── architecture-board/ # Karar kayıtları
│
├── .github/            # PR template ve workflow metadata
│   └── pull_request_template.md
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

### Profil Bazlı Build ve Preempt Doğrulama

```bash
# Release profil (default, temiz binary)
make release

# Validation profil (debug + instrumentation)
make validation

# Validation profil + -Werror
make validation-strict

# Deterministic preempt doğrulama (marker + fallback)
make run-preempt

# Strict marker-mode preempt doğrulama (fallback kapalı)
make run-preempt-strict

# Boundary CI gate (symbol deny/allow + evidence raporu)
make ci-gate-boundary

# Hygiene CI gate (tracked artifact + dirty tracked kontrolü)
make ci-gate-hygiene

# Drift activation gate (Phase-9 requirement: PASS/FAIL/SKIP)
make ci-gate-drift-activation

# Performance regression gate (baseline + drift persistence + allowlist)
make ci-gate-performance

# Summary gate (auto-discovery + PASS zorlaması)
make ci-summarize

# Tam CI akışı (boundary + hygiene + validate-full)
make ci

# Strict freeze suite (planned gate stubları dahil; implement edilene kadar fail expected)
make ci-freeze
```

Notlar:
- `run-preempt-strict` otomatik olarak `STRICT_MARKERS=1` ve `FORCE_EFI_REBUILD=1` ile çalışır.
- `run_preempt_test.sh` log analizinde `debugcon + serial` birleşik kaynağı kullanır.
- Context ABI tek-kaynak modeli aktiftir: `kernel/include/ayken_abi.h` ana kaynaktır, NASM için `kernel/include/generated/ayken_abi.inc` otomatik üretilir.
- `make guard-context-offsets` context_switch offset disiplinini build aşamasında zorunlu kılar.
- `make ci-gate-boundary` çıktıları `evidence/run-<RUN_ID>/` altında saklanır (`reports/summary.json` dahil).
- `make ci-gate-hygiene` tracked artifact ve tracked dirty-tree ihlallerini evidence ile fail eder.
- `make ci-summarize` gate raporlarını auto-discovery ile birleştirir; `summary.json` PASS değilse fail eder.
- Freeze workflow ve done kriterleri: `docs/roadmap/freeze-enforcement-workflow.md`.
- Freeze PR şablonu: `docs/development/PR_FREEZE_TEMPLATE.md` ve `.github/pull_request_template.md`.

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
  - 11 syscall aralığı aktif (1000-1010)
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

- ✅ **Phase 4.3:** Performance Optimization (tamamlandı ✅)
  - ✅ **Evidence-Based Optimization:** HashMap → Indexed structures (3-5x improvement)
  - ✅ **Algorithmic Improvements:** O(n²) → O(n) complexity transformation
  - ✅ **Memory Optimization:** 80%+ reduction in allocations (285KB → <50KB)
  - ✅ **Single-Pass Processing:** 7 passes → 1 pass streaming architecture
  - ✅ **Constitutional Compliance:** All optimizations preserve deterministic behavior
  - ✅ **Performance Validation:** 3-10x improvements achieved, 97.9% test coverage
  - ✅ **Integration Success:** Zero breaking changes, seamless codebase integration

- ✅ **Phase 4.4:** Ring3 Execution Model (TAMAMLANDI ✅)
  - ✅ **Ring3 User Process Execution:** Kullanıcı modu süreç yürütme başarılı
  - ✅ **Syscall Interface Operational:** INT 0x80 syscall interface çalışıyor
  - ✅ **Syscall Roundtrip Validated:** Kernel ↔ Ring3 geçişleri doğrulandı
  - ✅ **11 Execution-Centric Syscalls:** 1000-1010 aralığı operasyonel
  - ✅ **Capability-Based Security:** Yetenek tabanlı güvenlik aktif
  - ✅ **Ring3 VFS/DevFS:** Kullanıcı modunda dosya sistemi operasyonları
  - ✅ **BCIB Execution Engine:** Binary instruction execution çalışıyor

- 🚀 **Constitutional Integration:** Constitutional Stabilization & Lock (başlamaya hazır)
  - **Single Decision Authority:** All decisions flow through Gate C constitutional validation
  - **Evidence-Based Governance:** 100% evidence-backed decisions with complete audit trail
  - **Constitutional Lock Enforcement:** Violations result in system block/exit
  - **Unified Compliance Framework:** All constitutional requirements enforced at single point

### 🛠️ Ayken Constitutional Rule System (Geliştirme Aracı)

AykenOS'un geliştirilmesi için oluşturulan constitutional rule system:

- ✅ **Task 10.1:** MARS Module Detection (tamamlandı ✅)
  - ✅ **ModuleDetector:** Automatic boundary detection with hierarchical structure
  - ✅ **Module Configuration:** TOML-based custom boundary definitions
  - ✅ **File-to-Module Mapping:** Complete coverage with conflict detection
  - ✅ **Constitutional Compliance:** Deterministic assignment, identity preservation
  - ✅ **Test Coverage:** 5/5 MARS tests passing with constitutional validation

### Önemli Kilometre Taşları

| Hedef | Durum | Açıklama |
|-------|-------|----------|
| 11 Syscall Aralığı | ✅ | 1000-1010 aralığında execution-centric syscall'lar |
| Ring3 VFS/DevFS | ✅ | Kullanıcı modunda tam implementasyon |
| BCIB Execution Engine | ✅ | Ring3'te operasyonel |
| Capability Security | ✅ | Yetenek tabanlı güvenlik aktif |
| Multi-Agent System | ✅ | Planlama, koordinasyon, çakışma çözümü ve öğrenme |
| Planning Engine | ✅ | A* ve beam search algoritmaları |
| Coordination Protocols | ✅ | Ajan senkronizasyonu ve mesajlaşma |
| Conflict Resolution | ✅ | Kaynak çakışma tespiti ve çözümü |
| Learning & Optimization | ✅ | Performans öğrenme ve adaptif optimizasyon |
| Gate C Submission Bridge | ✅ | Submit-only interface, mutation intents, pipeline planning |
| Gate C IR Planner | ✅ | Semantic analysis, ordering hints, parallelism analysis |
| Gate C Security Operations | ✅ | Security inspection, capability-based redaction |
| Gate C REPL Visibility | ✅ | Plan visualization, semantic explanation |
| Constitutional Lock (B-MODE) | ✅ | Register invariants, integration, reports modules locked |
| Phase 4.2 Performance Engineering | ✅ | Evidence-based measurement infrastructure operational |
| Performance Constitution | ✅ | Measurable > Optimized principle established |
| Complexity Budget Tests | ✅ | Evidence for Phase 4.3 optimization identified |
| Evidence Schema Lock | ✅ | Immutable performance evidence format |
| Phase 4.3 Performance Optimization | ✅ | Evidence-based algorithmic improvements completed |
| HashMap → Indexed Optimization | ✅ | 3-5x performance improvement achieved |
| Memory Allocation Optimization | ✅ | 80%+ reduction in allocations (285KB → <50KB) |
| Single-Pass Processing | ✅ | O(n²) → O(n) complexity transformation |
| Constitutional Compliance | ✅ | All optimizations preserve deterministic behavior |
| Ring3 Execution Model | ✅ | Kullanıcı modu süreç yürütme operasyonel |
| Syscall Roundtrip | ✅ | INT 0x80 kernel ↔ Ring3 geçişleri doğrulandı |
| Phase 4.4 Ring3 Model | ✅ | Ring3 execution model tamamlandı |
| Evidence-Based Management | ✅ | 100% evidence-backed decisions with audit trail |
| Constitutional Integration Spec | 🚀 | Single decision authority framework ready to begin |
| **Ayken Constitutional Rule System** | | **Development Tool for AykenOS** |
| MARS System Implementation | ✅ | Module-level Architecture Risk Score system operational |
| Task 10.1 Module Detection | ✅ | File-to-module mapping with constitutional compliance |
| Module Boundary Validation | ✅ | Deterministic assignment with identity preservation |

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

**Gate C Submission Bridge:**
- Plan submission: < 1ms (100x hedef aşımı)
- Mutation conflict detection: < 1ms (50x hedef aşımı)
- Pipeline dependency analysis: < 1ms (10x hedef aşımı)
- Security inspection: < 2ms (5x hedef aşımı)
- End-to-end submission: < 5ms

**Phase 4.3 Performance Optimization:**
- Evidence-based algorithmic improvements: HashMap → Indexed (3-5x improvement)
- Memory allocation optimization: 285KB → <50KB (80%+ reduction)
- Single-pass processing: 7 passes → 1 pass (O(n²) → O(n))
- Constitutional compliance: All optimizations preserve deterministic behavior
- Complexity budget compliance: All tests pass with optimized algorithms
- Performance validation: 3-10x improvements achieved in critical paths
- Test coverage: 97.9% maintained (exceeds >95% target)
- Integration success: Zero breaking changes introduced

---

## 📚 Dokümantasyon

### Proje Raporları

- **Genel Durum:** [docs/phase1/PROJECT_STATUS_REPORT.md](docs/phase1/PROJECT_STATUS_REPORT.md)
- **Faz 1 Tamamlanma:** [docs/phase1/FAZ_1_COMPLETION_REPORT.md](docs/phase1/FAZ_1_COMPLETION_REPORT.md)
- **Faz 2.5 Tamamlanma:** [PHASE2_5_COMPLETION_REPORT.md](PHASE2_5_COMPLETION_REPORT.md)
- **GATE D Validation:** [GATE_D_VALIDATION_COMPLETION_REPORT.md](GATE_D_VALIDATION_COMPLETION_REPORT.md)
- **Gate C Submission Bridge:** [_ayken/specs/gate-c-submission-bridge/](_ayken/specs/gate-c-submission-bridge/)
  - **Requirements:** [requirements.md](_ayken/specs/gate-c-submission-bridge/requirements.md)
  - **Design:** [design.md](_ayken/specs/gate-c-submission-bridge/design.md)
  - **Tasks:** [tasks.md](_ayken/specs/gate-c-submission-bridge/tasks.md)
- **Constitutional Lock Analysis:** [CONSTITUTIONAL_LOCK_MANIFEST.md](CONSTITUTIONAL_LOCK_MANIFEST.md)
- **Phase 4.2 Performance Engineering:** [PHASE_4_2_FINAL_RESOLUTION_SUMMARY.md](PHASE_4_2_FINAL_RESOLUTION_SUMMARY.md)
- **Phase 4.3 Performance Optimization:** [_ayken/specs/phase-4-3-performance-optimization/](_ayken/specs/phase-4-3-performance-optimization/)
  - **Requirements:** [requirements.md](_ayken/specs/phase-4-3-performance-optimization/requirements.md)
  - **Design:** [design.md](_ayken/specs/phase-4-3-performance-optimization/design.md)
  - **Tasks:** [tasks.md](_ayken/specs/phase-4-3-performance-optimization/tasks.md)
- **Phase 4.3 Transition:** [PHASE_4_3_TRANSITION_REPORT.md](PHASE_4_3_TRANSITION_REPORT.md)

### Teknik Dokümantasyon

- **Proje Yapısı:** [docs/development/PROJECT_STRUCTURE.md](docs/development/PROJECT_STRUCTURE.md)
- **Dokümantasyon İndeksi:** [docs/development/DOCUMENTATION_INDEX.md](docs/development/DOCUMENTATION_INDEX.md)
- **Ring3 Implementasyon:** [docs/development/RING3_IMPLEMENTATION.md](docs/development/RING3_IMPLEMENTATION.md)
- **Syscall Geçiş Kılavuzu:** [docs/development/SYSCALL_TRANSITION_GUIDE.md](docs/development/SYSCALL_TRANSITION_GUIDE.md)
- **DevFS Implementasyon:** [docs/development/DEVFS_IMPLEMENTATION.md](docs/development/DEVFS_IMPLEMENTATION.md)

### Sistem Spesifikasyonları

- **Scheduler Arbitration Contract:** [docs/development/SCHEDULER_ARBITRATION_CONTRACT.md](docs/development/SCHEDULER_ARBITRATION_CONTRACT.md)
- **Capability System Reference:** [docs/development/CAPABILITY_SYSTEM_REFERENCE.md](docs/development/CAPABILITY_SYSTEM_REFERENCE.md)
- **BCIB Submission Protocol:** [docs/development/BCIB_SUBMISSION_PROTOCOL.md](docs/development/BCIB_SUBMISSION_PROTOCOL.md)

### CI ve Operasyonlar

- **Constitutional CI Mode:** [docs/operations/CONSTITUTIONAL_CI_MODE.md](docs/operations/CONSTITUTIONAL_CI_MODE.md)
- **Provisional CI Mode:** [docs/operations/PROVISIONAL_CI_MODE.md](docs/operations/PROVISIONAL_CI_MODE.md)
- **Performance Baseline Policy:** [docs/operations/PERF_BASELINE_POLICY.md](docs/operations/PERF_BASELINE_POLICY.md)

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

### Kısa Vadeli (Constitutional Integration)

- 🚀 **Constitutional Integration:** AykenOS Constitutional Stabilization & Lock (başlamaya hazır)
  - Single constitutional decision authority for all system decisions
  - Evidence-based governance with 100% evidence backing requirement
  - Constitutional lock enforcement with system block/exit on violations
  - Complete audit trail for all decisions with constitutional compliance tracking
  - Unified compliance framework ensuring system-wide constitutional adherence

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

**Son Güncelleme:** 21 Şubat 2026 - README, kod gerçekliğiyle hizalandı (`HEAD: 464cd009f4d0`). Yeni teknik dokümantasyon eklendi:
- Scheduler Arbitration Contract (Yol A)
- Capability System Reference
- BCIB Submission Protocol

**Güncelleyen:** Kenan AY

AykenOS, geleneksel işletim sistemi paradigmalarını sorgulayan ve AI-native bir gelecek için temel oluşturan yenilikçi bir projedir. Execution-centric mimari, Ring3 empowerment, multi-agent orchestration, constitutional CI guards ve evidence-based performance optimization özellikleriyle, modern işletim sistemlerine farklı bir bakış açısı sunmaktadır.

**Ayken Constitutional Rule System**: AykenOS'un geliştirilmesi için oluşturulan constitutional rule system, Task 10.1 MARS Module Detection ile modül seviyesinde risk atıfı sağlar.

**© 2026 Kenan AY - AykenOS Project**
