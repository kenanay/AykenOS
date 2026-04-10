# AykenOS
**The Constitutional AI Operating System**

*Anayasal Yapay Zeka İşletim Sistemi*

This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Copyright (c) 2026 Kenan AY. All rights reserved.**

[![License: Proprietary](https://img.shields.io/badge/License-Proprietary-red.svg)](LICENSE)
[![Author: Kenan AY](https://img.shields.io/badge/Author-Kenan%20AY-blue.svg)](mailto:kenanay@example.com)
[![Status: Protected](https://img.shields.io/badge/Status-Protected-orange.svg)](#-important-legal-notice)

**Oluşturan:** Kenan AY
**Oluşturma Tarihi:** 01.01.2026
**Son Güncelleme:** 10.04.2026
**Closure Evidence:** `local-freeze-p10p11` + `local-phase11-closure` + `run-local-phase12c-closure-2026-03-11` + `run-local-p13-kill-switch-20260315T000051Z` + `phase15-official-closure`
**Evidence Git SHA (Phase-10/11):** `9cb2171b` | **Evidence Git SHA (Phase-12C):** `01d1cb5c` | **Evidence Git SHA (Phase-13):** `40158350` | **Evidence Git SHA (Phase-15):** `48970cd0`
**Closure Sync / Remote CI (Phase-10/11):** `fe9031d7` (`ci-freeze#22797401328 = success`)
**Remote CI (Phase-12):** `ci-freeze#23099070483 = success` (PR #62)
**Remote CI (Phase-13):** `ci-freeze#23706742211 = success` (PR #81)
**Remote CI (Phase-15):** `ci-freeze#24213727039 = success` (PR #104) | tag `phase15-official-closure`
**CURRENT_PHASE:** `15` (`Phase-15 OFFICIALLY CLOSED — CURRENT_PHASE=15 — Phase-16 PENDING`)
**Freeze Zinciri:** `make ci-freeze` = strict freeze suite | `make ci-freeze-local` = local freeze suite with local performance authority
**Acil Blocker:** `yok`
**Yakın Hedef:** Phase-16 (Ayken CLI Faz B + BCIB toolchain surface)
**Ring0 Export Ceiling:** `193 symbols` (current enforced ceiling)
**Performance Baseline:** `gha-ubuntu24-20260406.80.1-X64`

**Proje Durumu:** Core OS Phase 4.5 TAMAMLANDI ✅ | Phase 10 runtime CLOSED (official) ✅ | Phase 11 verification substrate CLOSED (official) ✅ | Phase 12 trust layer OFFICIALLY CLOSED ✅ | Phase 13 distributed observability OFFICIALLY CLOSED ✅ | Phase 14 observability hardening OFFICIALLY CLOSED ✅ | Phase 15 BCIB Execution Engine v3 OFFICIALLY CLOSED ✅ | CURRENT_PHASE=15 ✅ | Architecture Freeze ACTIVE ✅
**Boot/Kernel Bring-up:** UEFI→kernel handoff doğrulandı ✅ | Ring3 process preparation operasyonel ✅ | ELF64 loader çalışıyor ✅ | User address space creation aktif ✅ | Syscall roundtrip doğrulandı ✅ | IRQ-tail preempt doğrulama hattı mevcut ✅
**Phase 10 Status:** Runtime determinism officially closed ✅ | remote `ci-freeze` run `22797401328`
**Phase 11 Status:** Replay + KPL + proof bundle officially closed ✅
**Phase 12 Status:** OFFICIALLY CLOSED ✅ | tag `phase12-official-closure-confirmed` at `1d79d4b1` | remote `ci-freeze` run `23099070483` (PR #62)
**Phase 13 Status:** OFFICIALLY CLOSED ✅ | tag `phase13-official-closure-confirmed` at `8b23fe0d` | remote `ci-freeze` run `23706742211` (PR #81) | Architecture Map §4 workstreams COMPLETE
**Phase 14 Status:** OFFICIALLY CLOSED ✅ | all 5 workstreams merged | `obs-cli` consumer crate complete | Phase-14 observability invariants preserved
**Phase 15 Status:** OFFICIALLY CLOSED ✅ | tag `phase15-official-closure` at `48970cd0` | remote `ci-freeze` run `24213727039` (PR #104) | BCIB Execution Engine v3: three-layer architecture, 293 tests PASS, 12 property tests PASS | `ayken-cli` v0.1 (Faz A wrapper) shipped | `tools/ayken-cli/`
**Architecture Quick Map:** `docs/specs/phase12-trust-layer/AYKENOS_GATE_ARCHITECTURE.md`
**Canonical Technical Definition:** AykenOS is a deterministic verification architecture that separates kernel execution, verification semantics, evidence artifacts, and distributed diagnostics into explicit layers. The kernel provides mechanism, userspace verification services produce artifact-bound verdicts and receipts, and parity/topology surfaces expose cross-node observability without elevating diagnostics into authority or consensus.

⚠️ **CI Mode:** `ci-freeze` workflow varsayılan olarak **CONSTITUTIONAL** modda çalışır (`PERF_BASELINE_MODE=constitutional`). Ayrıntı: [Constitutional CI Mode](docs/operations/CONSTITUTIONAL_CI_MODE.md).

---

## Phase Status

- **Current Phase:** `15`
- **Status:** `OFFICIALLY CLOSED`
- **Closure Index:** `reports/phase15_official_closure/closure_index.json`
- **Next Phase:** `16` (`pending` — Ayken CLI Faz B + BCIB toolchain surface)

## Authority Model

- **Official Closure**
  - Phase-tagged, immutable
  - Verified via `ayken closure verify`
- **Verified Head**
  - Development SHA validated by remote `ci-freeze`
  - Verified via `ayken head verify`
- **Authority Lineage**
  - Advisory ancestry diagnostics only
  - Must not inherit verified authority across SHAs

Current verified-head records live under `reports/verified_heads/<FULL_SHA>.json`.
These records are SHA-scoped CI projections. `ayken head verify` only succeeds when an exact record for the current SHA is available locally.
Authority lineage, when added, is diagnostic context only.

> A verified head is not a closure.

---

## 🔒 IMPORTANT LEGAL NOTICE

This software is **proprietary and confidential**. All rights reserved by Kenan AY.

### ⚖️ Usage Restrictions:
- ✅ **Educational viewing** permitted for learning purposes
- ❌ **Commercial use** prohibited without license
- ❌ **Modification** prohibited without written permission
- ❌ **Distribution** prohibited without authorization
- ❌ **Reverse engineering** prohibited

### � Licensing Contact:
For commercial licensing, partnerships, or permissions:
- **Email**: kenanay@example.com
- **Subject**: "AykenOS Licensing Inquiry"

---

## 🎯 Proje Vizyonu

AykenOS, yapay zeka destekli, yenilikçi ve çoklu mimari işletim sistemi projesidir. Geleneksel işletim sistemlerinden farklı olarak, **execution-centric** (yürütme merkezli) bir mimari benimser ve **AI-native** (yapay zeka doğal) tasarım prensipleriyle geliştirilmiştir.

### Mimari Dönüşüm

- **Ring0 (Kernel Mode):** 12 execution-centric mekanizma syscall'ı (1000-1011 aralığı)
- **Ring3 (User Mode):** Tüm politika kararları (VFS, DevFS, AI, scheduler) kullanıcı modunda
- **Capability-Based Security:** Yetenek tabanlı güvenlik modeli ile erişim kontrolü
- **BCIB Execution Engine:** Binary Compressed Instruction Bundle formatı ile veri-odaklı yürütme

---

## 🚀 Temel Özellikler

### Execution-Centric Syscall Interface (1000-1011)

| ID | Syscall | Açıklama |
|----|---------|----------|
| 1000 | `sys_v2_map_memory` | Bellek haritalama |
| 1001 | `sys_v2_unmap_memory` | Bellek haritalama kaldırma |
| 1002 | `sys_v2_switch_context` | Bağlam değiştirme |
| 1003 | `sys_v2_submit_execution` | BCIB yürütme gönderimi |
| 1004 | `sys_v2_wait_result` | Yürütme sonucu bekleme |
| 1005 | `sys_v2_interrupt_return` | Kesme dönüşü |
| 1006 | `sys_v2_time_query` | Zaman sorgulama |
| 1007 | `sys_v2_capability_bind` | Yetenek bağlama |
| 1008 | `sys_v2_capability_revoke` | Yetenek iptal etme |
| 1009 | `sys_v2_exit` | Süreç sonlandırma |
| 1010 | `sys_v2_debug_putchar` | Ring3 debug heartbeat |
| 1011 | `sys_v2_complete_execution` | Yürütme slot yaşam döngüsü tamamlama |

### Çoklu Mimari Desteği

- **UEFI/x86_64:** Tam özellikli kernel ve bootloader ✅
- **ARM64:** Bootloader implementasyonu 🔄
- **RISC-V:** Bootloader implementasyonu 🔄
- **Raspberry Pi:** Özel bootloader desteği ✅
- **MCU:** Mikrodenetleyici bootloader ✅

---

## 📁 Proje Yapısı

```
AykenOS/
├── kernel/              # C tabanlı çekirdek (Ring0, x86_64)
├── bootloader/          # Çoklu mimari bootloader'lar
├── userspace/           # Ring3 bileşenleri (Rust + C)
│   ├── libayken/       # Ring3 VFS/DevFS/Scheduler (C)
│   ├── bcib-runtime/   # BCIB execution engine
│   ├── semantic-cli/   # Semantic CLI
│   ├── dsl-parser/     # DSL parser
│   └── proofd/         # Proof daemon service
├── ayken-core/          # AI/data systems (Rust)
│   └── crates/
│       ├── abdf/       # Ayken Binary Data Format
│       ├── bcib/       # Binary CLI Instruction Buffer
│       └── proof-verifier/ # Trust layer verification
├── ayken/               # Constitutional governance tool (Rust)
├── docs/                # Dokümantasyon
│   └── specs/phase12-trust-layer/  # Phase 12 spesifikasyonları
├── scripts/ci/          # CI gate scriptleri
├── tests/               # External invariant-based scenarios and validators
├── tools/test_runner/   # External scenario runner + normalizer + validator pipeline
├── tools/ci/            # CI test araçları
├── evidence/            # CI gate evidence (auto-generated)
└── constitution/        # Constitutional framework
```

---

## 🛠️ Derleme ve Çalıştırma

### Gereksinimler

- `clang` + `ld.lld` — Kernel toolchain
- `nasm` — Assembler
- `qemu-system-x86_64` — Test/emülasyon
- `cargo` / `rustc` — Rust bileşenleri (opsiyonel)

### Temel Komutlar

```bash
# Temiz build
make clean && make all

# EFI disk imajı + QEMU
make efi-img
make run

# Profil bazlı build
make release          # Optimized (default)
make validation       # Debug + instrumentation
make validation-strict # Validation + -Werror
```

### CI Gates

```bash
# Pre-CI discipline (local, ~30-60s)
make ci-gate-abi
make ci-gate-boundary
make ci-gate-hygiene
make ci-gate-constitutional
make ci-gate-ring3-user-leaf-rule
make ci-gate-test-naming
make ci-gate-error-codes
make ci-gate-kernel-test-pipeline
make ci-kernel-tests

# Tam CI suite
make ci-freeze        # strict freeze suite (fail-closed)
make ci-freeze-local  # local freeze suite (local perf authority active)
make ci-gate-performance-local  # local perf gate with auto-init gitignored baseline
```

### Rust Bileşenleri

```bash
cd ayken-core && cargo build && cargo test
cd userspace && cargo build && cargo test
cd ayken && cargo build && ./target/debug/ayken check
```

---

## 📊 Proje Durumu

### Tamamlanan Fazlar

| Faz | Durum | Açıklama |
|-----|-------|----------|
| Phase 1 — Core Kernel | ✅ CLOSED | UEFI boot, bellek, GDT/IDT, sürücüler |
| Phase 1.5 — Stabilization | ✅ CLOSED | Ring3 round-trip, toolchain doğrulama |
| Phase 2 — Execution-Centric | ✅ CLOSED | 11 syscall, Ring3 VFS/DevFS, BCIB |
| Phase 2.5 — Legacy Cleanup | ✅ CLOSED | POSIX kaldırma, Ring0 policy temizliği |
| Phase 3.4 — Multi-Agent | ✅ CLOSED | Gate A-E tamamlandı |
| Phase 4.3 — Performance | ✅ CLOSED | HashMap→Indexed (3-5x), 80%+ mem azalma |
| Phase 4.4 — Ring3 Model | ✅ CLOSED | Ring3 execution, syscall roundtrip |
| Phase 4.5 — Policy Accept | ✅ CLOSED | Gate-4 policy-accept proof operasyonel |
| Phase 10 — Runtime | ✅ OFFICIALLY CLOSED | CPL3 entry, deterministic runtime |
| Phase 11 — Verification | ✅ OFFICIALLY CLOSED | Ledger, ETI, replay, proof bundle |
| Phase 12 — Trust Layer | ✅ OFFICIALLY CLOSED | tag `phase12-official-closure-confirmed`, remote CI run `23099070483` (PR #62) |
| Phase 13 — Distributed Observability | ✅ OFFICIALLY CLOSED | tag `phase13-official-closure-confirmed`, remote CI run `23706742211` (PR #81) |
| Phase 14 — Distributed Observability Hardening | ✅ OFFICIALLY CLOSED | Replay determinism, proofd boundary, cross-node graph, observability UX |
| Phase 15 — BCIB Execution Engine v3 | ✅ OFFICIALLY CLOSED | Three-layer BCIB runtime, 293 tests PASS, 12 property tests PASS |

### Phase 12 Detayı

Phase 12 trust layer kapsamında tamamlananlar:

- ✅ `P12-01..P12-18` — Tüm gate'ler GREEN (20/20 PASS)
- ✅ Authority Sinkhole Absorption — `gate_authority_sinkhole_absorption.sh`
- ✅ Authority Sinkhole Companion Flow/Producer
- ✅ Trust Reuse Runtime Evaluator / Surface / Emitter
- ✅ Verification Context Object + Verifier Attestation
- ✅ Verification Diversity Floor / Ledger / Producer
- ✅ Cartel Correlation gate
- ✅ proofd service observability boundary
- ✅ Cross-surface basin alignment metrics
- ✅ Remote `ci-freeze` run `23099070483` confirmed (PR #62)
- ✅ Official closure tag: `phase12-official-closure-confirmed` at `1d79d4b1`

### CI Gate Durumu (18 Mart 2026)

| Gate | Durum |
|------|-------|
| ABI | ✅ PASS |
| Boundary | ✅ PASS |
| Hygiene | ✅ PASS |
| Constitutional | ✅ PASS |
| Ring0 Exports | ✅ PASS |
| Syscall v2 Runtime | ✅ PASS |
| Sched Bridge Runtime | ✅ PASS |
| Policy Accept | ✅ PASS |
| Performance | ✅ PASS |
| proofd-service | ✅ PASS |

### Worktree-Local Ring3 User-Leaf Rule

- `ci-gate-ring3-user-leaf-rule` artik active, local deterministic, fail-closed enforcement olarak baglidir.
- Runtime authority zinciri: `P10_TEXT_FRAME_WITNESS -> P10_POST_CR3_TEXT_PROBE -> P10_RING3_USER_CODE`
- Bu gate executable user-leaf rule'unu korur; broader `ci-gate-ring3-execution-phase10a2` strict/global authority iddiasi yerine gecmez.

---

## 📚 Dokümantasyon

- **Architecture Map:** `docs/specs/phase12-trust-layer/AYKENOS_GATE_ARCHITECTURE.md`
- **Phase 13 Hazırlık:** `docs/specs/phase12-trust-layer/PHASE13_ARCHITECTURE_MAP.md`
- **Verification Observability:** `docs/specs/phase12-trust-layer/VERIFICATION_OBSERVABILITY_MODEL.md`
- **Trust Reuse Runtime:** `docs/specs/phase12-trust-layer/TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`
- **Authority Sinkhole:** `docs/specs/phase12-trust-layer/AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`
- **Constitutional CI Mode:** `docs/operations/CONSTITUTIONAL_CI_MODE.md`
- **Freeze Workflow:** `docs/roadmap/freeze-enforcement-workflow.md`
- **Documentation Index:** `docs/development/DOCUMENTATION_INDEX.md`
- **Ring3 User-Leaf Rule:** `docs/governance/RING3_USER_LEAF_ALLOCATION_RULE.md`
- **Ring3 Runtime Closure Note:** `docs/governance/RING3_RUNTIME_CLOSURE_NOTE.md`
- **Test Naming Convention:** `docs/governance/TEST_NAMING_CONVENTION.md`
- **Test Pipeline Contract:** `docs/governance/TEST_PIPELINE_CONTRACT.md`

---

## � Lisans

AykenOS iki lisans modeli ile dağıtılır:

### ASAL v1.0 — AykenOS Source-Available License
- ✅ Eğitim, araştırma, kişisel kullanım
- ❌ Ticari kullanım yasak

### ACL v1.0 — AykenOS Commercial License
- ✅ Ticari ürünler, SaaS, entegrasyon
- ✅ Binary dağıtımı
- Lisans için: kenanay@example.com

---

## 🎯 Sonraki Hedefler

**Kısa Vadeli (Phase-16):**
- Ayken CLI Faz B: `ayken status`
- Ayken CLI Faz B: `ayken gate all`
- Ayken CLI Faz B: `ayken closure status --json` (advisory)
- Ayken CLI Faz B: `ayken closure verify` (binding, fail-closed)
- Ayken CLI Faz B: `ayken head verify` (binding, CI-backed)
- Ayken CLI Faz C: `ayken bcib verify`
- Ayken CLI Faz C: `ayken bcib hash`
- Ayken CLI Faz C: `ayken bcib inspect`
- Authority modeli: `ci-freeze` authoritative, local tooling advisory

**Orta Vadeli:**
- ARM64 + RISC-V kernel portları
- Gerçek donanım testleri (Raspberry Pi)
- Network stack (temel TCP/IP)

**Uzun Vadeli:**
- Tam AI entegrasyonu (TinyLLM)
- Veri-odaklı dosya sistemi
- AI-native shell
- Ekosistem geliştirme

---

**Son Güncelleme:** 10 Nisan 2026 — Phase-15 OFFICIALLY CLOSED (ci-freeze#24213727039, PR #104); CURRENT_PHASE=15; Phase-16 pending.

**© 2026 Kenan AY — AykenOS Project**
