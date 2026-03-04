# AykenOS Roadmap Documentation
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Bu dizin, AykenOS roadmap ve freeze durumunu kod gercekligiyle uyumlu sekilde takip etmek icindir.

## Ana Belgeler
- `overview.md`: Kod + gate evidence temelli guncel mimari durum
- `ROADMAP_2026_02_23.md`: Aktif uygulama roadmap'i (dosya adi tarihsel)
- `../../ARCHITECTURE_FREEZE.md`: Freeze kontrati ve zorunlu invariants
- `freeze-enforcement-workflow.md`: Gate + evidence operasyon kurallari

## Kod Snapshot Ozeti (2026-03-05)
- Snapshot branch/head: `main@7af35acc`
- Core milestone: `Phase 4.5` complete (`v0.4.6-policy-accept`)
- Deterministic baseline: CI authority baseline lock repoda mevcut (`scripts/ci/perf-baseline.lock.json`)
- Active blocker: `Phase 10-A2` strict marker zincirinde `P10_RING3_USER_CODE` eksik

## Freeze / Gate Gercekligi
- `make pre-ci`: 4 gate (abi, boundary, hygiene, constitutional), fail-closed
- `make ci-freeze`: 21 gate strict zincir
- `make ci-freeze-local`: 19 gate (performance + tooling-isolation hariic)

## Su Anki Kritik Teknik Durum
- Ring0 export surface limiti aktif ve sinirda: `165/165`
- Phase 10-A2 gate'i bu snapshot'ta fail ediyor:
  - `missing_marker:P10_RING3_USER_CODE`
- Pre-CI bu worktree'de hygiene nedeniyle fail veriyor (dirty tracked dosyalar mevcut)

## Not
Eski roadmap markdown'larinda kalan bazi "tamamlandi" iddialari kodla birebir ortusmeyebilir.
Bu dizinde referans otoritesi `overview.md` + `ROADMAP_2026_02_23.md` (guncel icerik) dokuman ciftidir.

---
**Son Guncelleme:** 2026-03-05
**Guncelleme Temeli:** Repo kodu + local gate evidence
