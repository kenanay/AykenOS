# AykenOS Roadmap - Kod Snapshot Temelli Durum (2026)
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

## Scope
Bu belge, roadmap durumunu doğrudan repo kodu ve CI tanımları üzerinden özetler.

- Snapshot commit: `464cd009f4d0`
- Kaynaklar: `Makefile`, `kernel/sys/*`, `kernel/sched/*`, `scripts/ci/*`, `.github/workflows/ci-freeze.yml`

## Guncel Durum (Kod Gercekligi)

### Core OS
- **Phase 4.4:** Tamamlanmis kabul edilen temel: UEFI->kernel, Ring3 gecisi, INT 0x80 hatti.
- **Phase 4.5:** Devam ediyor (stabilizasyon ve entegrasyon).
- **Syscall v2 araligi:** `1000..1010` (toplam 11 syscall) (`kernel/sys/syscall_v2.h`).
- **Dispatcher:** Sadece `1000..1010` kabul ediyor (`kernel/sys/syscall.c`).

### Syscall Uygulama Olgunlugu
`kernel/sys/syscall_v2.c` icinde kritik noktalar:
- `capability_bind` / `capability_revoke` ve `debug_putchar` pratikte en net calisan kisimlar.
- `map_memory`, `unmap_memory`, `submit_execution`, `wait_result`, `interrupt_return`, `time_query`, `exit` icin TODO/placeholder seviyesinde mantik mevcut.
- Bu nedenle ABI ve dispatch hatti aktif olsa da tum syscall semantigi "production-complete" degil.

### Scheduler Durumu
- `AYKEN_SCHED_FALLBACK` default `0` (`Makefile`, `kernel/sched/sched.h`).
- Scheduler policy'nin Ring3'e tamamen tasindigi iddiasi kodda tam karsilanmiyor:
  - Ring3 policy C-call yolu `sched.c` icinde yorum satirina alinmis.
  - Etkin secim yolu su an kernel ready-queue mekanik akisi (round-robin benzeri).
- Sonuc: scheduler arbitration/mailbox dokuman hedefi ile runtime kodu arasinda kapanmamis fark var.

### Ring0/Ring3 Dosya Sistemi Siniri
- `kernel/fs/vfs.c` ve `kernel/fs/devfs.c` dosyalari policy degil, placeholder/compat katmani gibi calisiyor.
- Ring3 tarafinda `userspace/libayken/*` policy niyeti korunuyor.

## CI / Freeze Durumu

### `ci-freeze` Zinciri (Kodda)
`Makefile` uzerinden strict zincir:
1. `ci-gate-abi`
2. `ci-gate-boundary` (symbol-scan)
3. `ci-gate-ring0-exports`
4. `ci-gate-hygiene`
5. `ci-gate-tooling-isolation`
6. `ci-gate-constitutional`
7. `ci-gate-workspace`
8. `ci-gate-syscall-v2-runtime`
9. `ci-gate-performance`

### Mode Gercekligi
- `.github/workflows/ci-freeze.yml` freeze job'u default **constitutional** (`PERF_BASELINE_MODE=constitutional`).
- Ayni workflow'de baseline init job'u **provisional** yol kullanir.
- `ci-gate-tooling-isolation`, `PERF_BASELINE_MODE=provisional` iken `SKIP` uretebilir (`Makefile`).
- `ci-summarize` su an `PASS/SKIP/WARN` verdict'lerini kabul eder.

## Faz Yorumu

### Nerede Oldugumuz
- **Net durum:** Phase 4.4 tamam, **Phase 4.5 stabilizasyon ve uyumlandirma asamasinda**.
- "Scheduler arbitration fully implemented" ifadesi mevcut kodla birebir uyumlu degil.
- "11 syscall frozen" dogru, ancak birkac syscall hala TODO/sembolik davranista.

### Phase 4.5 Kapanis Icin Kod-Temelli Kriterler
1. Scheduler Ring3 policy bridge'i (mailbox/stage-next benzeri) runtime'da gercekten aktif olmali.
2. `syscall_v2.c` TODO kalan mekanizmalar gercek semantikle tamamlanmali.
3. `ci-freeze` zincirinin 9 gate'i de ayni run icinde tutarli PASS vermeli.
4. Performance baseline authority / env-hash sureci local-CI farklarinda net governance ile kapanmali.

## Sonraki Surec (Pragmatik)
1. Scheduler path: dokuman hedefini runtime koduyla hizala (yorumdaki Ring3 policy cagrisi modeli yerine gercek bridge).
2. Syscall tamamlanma: placeholder syscall'lari asil mekanizma davranisina cek.
3. CI konsolidasyonu: constitutional/provisional modlarin PR politikasini tek bir operasyon dokumaniyla netlestir.
4. Phase 3 AI entegrasyonu: ancak yukaridaki teknik borc kapanislari sonrasinda ana mile-stone yap.

## Referans Dosyalar
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `kernel/sys/syscall_v2.h`
- `kernel/sys/syscall_v2.c`
- `kernel/sys/syscall.c`
- `kernel/sched/sched.c`
- `kernel/sched/sched.h`
- `kernel/fs/vfs.c`
- `kernel/fs/devfs.c`
- `scripts/ci/gate_performance.sh`
- `scripts/ci/gate_syscall_v2_runtime.sh`

---

**Son Guncelleme:** 2026-02-21  
**Guncelleme Yontemi:** Kod snapshot incelemesi (dokuman iddiasi degil, kaynak gercekligi)
