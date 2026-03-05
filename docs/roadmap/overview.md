# AykenOS Roadmap - Kod ve Evidence Temelli Durum (2026-03-05)
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

## Scope
Bu belge roadmap durumunu dogrudan repo kodu, Make hedefleri ve gate evidence ciktisi uzerinden ozetler.

- Snapshot branch/head: `main@7af35acc`
- Kaynaklar: `Makefile`, `kernel/*`, `scripts/ci/*`, `.github/workflows/*`, `evidence/run-*`

## 1) Mimari Omurga (Constitutional)

### 1.1 Ring0/Ring3 Ayrimi
- Ring0: mekanizma (memory, interrupt, context, syscall dispatch)
- Ring3: policy (scheduler policy, AI runtime, userspace davranis)
- Bu ayrim CI gate'lerle fail-closed korunuyor.

### 1.2 Syscall ABI
- V2 ABI araligi: `1000..1010` (11 syscall)
- Dispatcher yalniz bu araligi kabul ediyor.
- ABI tek kaynak disiplini: `kernel/include/ayken_abi.h` + `make generate-abi`

### 1.3 Determinism ve Baseline Governance
- Performance baseline lock dosyasi repoda: `scripts/ci/perf-baseline.lock.json`
- Baseline authority: `github-hosted-ubuntu-24.04-x64`
- Local Darwin/arm64 run'larinda env hash ve digest farki beklenen fail uretebilir.

## 2) Gate Mimarisi (Repo Truth)

### 2.1 Local Discipline
- `make pre-ci`
- Zincir: `ci-gate-abi` -> `ci-gate-boundary` -> `ci-gate-hygiene` -> `ci-gate-constitutional`
- Fail-closed, no-bypass, no-auto-fix

### 2.2 Strict Freeze Zinciri
- `make ci-freeze` su an 21 gate calistirir:
1. `ci-gate-abi`
2. `ci-gate-boundary`
3. `ci-gate-ring0-exports`
4. `ci-gate-hygiene`
5. `ci-gate-tooling-isolation`
6. `ci-gate-constitutional`
7. `ci-gate-governance-policy`
8. `ci-gate-drift-activation`
9. `ci-gate-structural-abi`
10. `ci-gate-runtime-marker-contract`
11. `ci-gate-user-bin-lock`
12. `ci-gate-embedded-elf-hash`
13. `ci-gate-performance`
14. `ci-gate-ring3-execution-phase10a2`
15. `ci-gate-syscall-semantics-phase10b`
16. `$(PHASE10C_FREEZE_GATE)`
17. `ci-gate-workspace`
18. `ci-gate-syscall-v2-runtime`
19. `ci-gate-sched-bridge-runtime`
20. `ci-gate-behavioral-suite`
21. `ci-gate-policy-accept`

### 2.3 Local Freeze Variant
- `make ci-freeze-local`
- 19 gate; `performance` ve `tooling-isolation` hariic tutulur.

## 3) Evidence Tabanli Guncel Durum

### 3.1 Tamamlananlar
- `Phase 4.5` policy-accept milestone tamam.
- Ring0 export gate aktif ve limitte PASS (`165/165`).
- `Phase 10` deterministic baseline lock repoda mevcut.

### 3.2 Aktif Bloklayicilar
- `Phase 10-A2` strict marker zinciri PASS degil.
- Son strict run: `missing_marker:P10_RING3_USER_CODE`.
- Bu eksik marker, "real CPL3 proof complete" iddiasini su an bloke ediyor.

### 3.3 Operasyonel Durum
- Bu worktree'de `make pre-ci` hygiene asamasinda fail veriyor (dirty tracked dosyalar).
- Bu durum aktif gelistirme asamasinda beklenebilir; merge oncesi temizlenmelidir.

## 4) Teknik Bosluklar (Mimari)

### 4.1 Phase 10-A2 Son Bosluk
- `P10_RING3_ENTER` ve syscall marker'lari goruluyor.
- Final user-code marker (`P10_RING3_USER_CODE`) eksik.
- Odak: #BP/IRQ/scheduler etkileşiminde final markerin kaybolma noktasi.

### 4.2 Syscall v2 Semantik Olgunluk
- `syscall_v2.c` icinde birden cok mekanizma TODO/placeholder seviyesinde:
  - `map_memory`, `unmap_memory`
  - `submit_execution`, `wait_result`
  - `interrupt_return`, `time_query`
  - `exit` (sonsuz `sched_yield` dongusu)
- ABI ve dispatch stabil; semantik tamamlanma hala yol haritasi kalemi.

### 4.3 Dokumantasyon Senkronizasyonu
- Birkac eski dokumanda `%40` ve `pending` kalemleri kod gercekligiyle celisiyordu.
- Bu roadmap paketi, son durumla hizalama icin guncellendi.

## 5) Yol Haritasi Karari (As-of 2026-03-05)

### 5.1 Acil (0-48 Saat)
1. `Phase 10-A2` strict gate'i PASS'e cek (`P10_RING3_USER_CODE` eksigini kapat).
2. A2 gate PASS kanitini yeni evidence run-id ile sabitle.
3. Freeze ve status dokumanlarinda blocker bilgisini run-id bazli guncelle.

### 5.2 Kisa Vade (1-2 Hafta)
1. Phase 10-B syscall semantik gap'lerini kapatacak minimum mekanizma implementasyonlari.
2. Phase 10-C scheduler/mailbox akisini strict marker kontratiyla stabilize et.
3. `ci-freeze` zincirinde tutarli PASS hedefi (branch temizligi dahil).

### 5.3 Orta Vade
1. Phase 5.0 AI runtime genislemesi sadece 10-A2/10-B/10-C teknik borcu kapandiktan sonra.
2. Multi-arch ve production hardening asamalarina gecis, freeze cikis kriterleri ile bagli.

## 6) Exit Kriterleri (Bu Faz Icin)
1. `ci-gate-ring3-execution-phase10a2` strict PASS.
2. Marker kontratinda eksik/yanlis sira ihlali yok.
3. Merge oncesi hygiene PASS (clean tracked state).
4. Roadmap + status dokumanlari son run evidence ile senkron.

## Referans
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `.github/workflows/perf-baseline-init.yml`
- `scripts/ci/pre_ci_discipline.sh`
- `scripts/ci/gate_ring3_execution_phase10a2.sh`
- `scripts/ci/gate_performance.sh`
- `kernel/sys/syscall_v2.c`
- `kernel/arch/x86_64/ring3_enter.S`
- `kernel/arch/x86_64/interrupts.c`

---
**Son Guncelleme:** 2026-03-05
**Guncelleme Yontemi:** Kod + Make hedefleri + local evidence run incelemesi
