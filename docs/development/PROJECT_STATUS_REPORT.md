# AykenOS Project Status Report (Code-Snapshot)

**Date:** 2026-02-21  
**Status:** Phase 4.5 In Progress (Stabilization)  
**Snapshot:** `464cd009f4d0`

## Executive Summary
Bu rapor, markdown iddialarindan bagimsiz olarak repo kodu uzerinden hazirlandi.

- Core OS tarafinda **Phase 4.4 seviyesi** (boot + ring3 + int80 hatti) mevcut.
- Proje **Phase 4.5 stabilizasyon** asamasinda.
- Syscall v2 ABI araligi **1000-1010 (11 syscall)** olarak kilitli.
- CI freeze zinciri kodda **9 gate** olarak tanimli.
- Scheduler arbitration/mailbox hedefi dokumanlarda geciyor; runtime kodda tam aktif degil.

## Koddan Dogrudan Bulgular

### 1) Syscall ABI ve Dispatcher
- ABI tanimi: `kernel/sys/syscall_v2.h`
  - `SYS_V2_BASE=1000`
  - `SYS_V2_MAX_INDEX=10`
  - `SYS_V2_LAST=1010`
  - `SYS_V2_NR=11`
- Ana dispatcher: `kernel/sys/syscall.c`
  - Sadece `1000..1010` kabul eder.

### 2) Syscall Uygulama Seviyesi
`kernel/sys/syscall_v2.c`:
- Calisan/etkin kisimlar: `debug_putchar`, capability bind/revoke yolu.
- Placeholder/TODO kalan alanlar:
  - map/unmap
  - submit/wait
  - interrupt_return
  - time_query (dummy timestamp)
  - exit (sonsuz `sched_yield` dongusu)

### 3) Scheduler
`kernel/sched/sched.c`:
- Ring3 policy C-call yolu yorum satirina alinmis.
- Etkin secim yolu kernel ready queue mekanik akisi.
- `AYKEN_SCHED_FALLBACK` default 0; strict guard mevcut (`Makefile`, constitutional gate).

### 4) VFS/DevFS
- `kernel/fs/vfs.c` ve `kernel/fs/devfs.c` policy yerine placeholder/compat katmani davranisinda.
- Ring3 tarafindaki policy niyeti (`userspace/libayken/*`) korunuyor.

## CI / Freeze Gercekligi

### `make ci-freeze` (kodda tanimli zincir)
1. abi
2. boundary (symbol-scan)
3. ring0-exports
4. hygiene
5. tooling-isolation
6. constitutional
7. workspace
8. syscall-v2-runtime
9. performance

### Modlar
- `ci-freeze` workflow freeze job: `PERF_BASELINE_MODE=constitutional`.
- Baseline init job: `PERF_BASELINE_MODE=provisional`.
- Provisional modda tooling-isolation gate `SKIP` olabilir.
- Summarizer `PASS/SKIP/WARN` kombinasyonlarini kabul eder.

## Faz Degerlendirmesi

### Guncel Faz
- **Current:** Phase 4.5 (stabilization/integration)

### Neden 4.5 tamam degil?
- Scheduler hedef mimarisi (Ring3 policy bridge) runtime kodda tam devrede degil.
- Birden fazla syscall hala TODO/placeholder davranista.
- Performance governance tarafinda constitutional/provisional yol farklari operasyonel olarak dikkat gerektiriyor.

## Oncelikli Sonraki Adimlar
1. Scheduler bridge'i runtime'da gercekten etkinlestir (mailbox/hint->arbiter modeli veya final karar).
2. TODO syscall'lari gercek mekanizma semantigiyle tamamla.
3. 9-gate freeze run'larini tek run-id altinda istikrarli PASS seviyesine getir.
4. Sonra Phase 3 AI entegrasyonunu ana mile-stone'a al.

## Referans
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `kernel/sys/syscall_v2.h`
- `kernel/sys/syscall_v2.c`
- `kernel/sys/syscall.c`
- `kernel/sched/sched.c`
- `kernel/fs/vfs.c`
- `kernel/fs/devfs.c`
- `scripts/ci/gate_performance.sh`
- `scripts/ci/gate_syscall_v2_runtime.sh`
