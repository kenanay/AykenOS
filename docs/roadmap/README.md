# AykenOS Roadmap Documentation
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Bu dizin, AykenOS roadmap dokumanlarini kod gercekligiyle uyumlu sekilde takip etmek icindir.

## Ana Belgeler
- `overview.md`: Kod snapshot temelli guncel durum ozeti
- `../../ARCHITECTURE_FREEZE.md`: Freeze kontrati ve zorunlu invariants
- `freeze-enforcement-workflow.md`: Gate + evidence kapanis proseduru
- `phase-4-4-status.md`: Phase 4.4 closure baglami
- `phase-4-5-spec.md`: Phase 4.5 hedef spesifikasyonu

## Kod Snapshot Ozeti (2026-02-21)
- Core OS: Phase 4.4 tamam, Phase 4.5 stabilizasyon devam
- Syscall v2 araligi: `1000..1010` (11 syscall)
- Scheduler: Ring3 policy bridge hedefi dokumanda var, runtime kodda tam aktif degil
- Freeze chain: `make ci-freeze` icinde 9 gate tanimli

## Freeze Gate Status (Repo Truth)
### Implemented Gates
- `ci-gate-abi`
- `ci-gate-boundary`
- `ci-gate-ring0-exports`
- `ci-gate-hygiene`
- `ci-gate-tooling-isolation`
- `ci-gate-constitutional`
- `ci-gate-workspace`
- `ci-gate-syscall-v2-runtime`
- `ci-gate-performance`
- `ci-summarize`

### Strict Suite
- `make ci-freeze`

### Local Strict-Without-Perf Shortcut
- `make ci-freeze-local`
  - performance + tooling-isolation gate'lerini local gelistirme icin atlar

## Not
Eski roadmap markdown'larinda kalan bazi "tamamlandi" iddialari kodla birebir ortusmeyebilir. Bu dizinde referans otoritesi artik `overview.md` (kod snapshot temelli) dokumanidir.

---
**Son Guncelleme:** 2026-02-21  
**Guncelleme Temeli:** Repo kodu + build/CI tanimlari
