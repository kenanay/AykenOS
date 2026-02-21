# ci(perf): baseline governance hardening + deterministic regression hook + strict clean

## Amaç

Bu PR, AykenOS performans baseline yönetimini tek kaynak + fail-closed modeline geçirerek CI determinism'ini güçlendirir. Ayrıca regression test canary'si için deterministik, compile-time gated bir hook ekler ve validation-strict hedefini -Werror altında temiz hale getirerek constitutional enforcement zeminini sertleştirir.

Performance gate default olarak fail-closed davranır; regression ve environment mismatch durumlarında soft-pass yoktur.

## Değişiklik Özeti

### 1) Baseline authority – Single Source of Truth

- Authority tek kaynakta merkezileştirildi: `scripts/ci/perf_authority.env`
- Makefile, performance gate ve ilgili workflow'lar aynı authority modelini kullanır hale getirildi.
- Governance dokümanı netleştirildi (terminoloji, drift SLA, intentional regression kuralları).

### 2) Deterministic intentional regression hook

- Yeni flag: `AYKEN_INTENTIONAL_PERF_REGRESSION_MS` (default 0)
- Hem CFLAGS hem ASMFLAGS tarafına tutarlı biçimde geçirildi.
- kernel.c içinde timer/tick tabanlı (busy-loop değil) deterministik gecikme eklendi.
- Compile-time gated: `#if AYKEN_INTENTIONAL_PERF_REGRESSION_MS > 0`
- Default OFF. Sadece canary/regression testi için açılır.

### 3) validation-strict -Werror temizliği

- Uyarılar suppression yerine mümkün olduğunca kod tarafında temizlendi.
- "Future use" kodlar unused-safe hale getirildi.
- Sonuç: `make clean && make validation-strict -j4` → PASS

### 4) Repo Hijyeni

Root'ta bulunan helper script'ler:
- `tools/dev/baseline/` altına taşındı
- `README.md` eklendi
- Executable bit korunarak commitlendi

### 5) Local override davranışı düzeltmesi

- Merkezi authority modeline geçiş sonrası local dev override eziliyordu.
- Makefile güncellendi:
  - Env'den `PERF_BASELINE_AUTHORITY` gelirse koru
  - Gelmezse `perf_authority.env` default'unu kullan

## Kanıt / Doğrulama

### Strict Build

```bash
make clean && make validation-strict -j4 → PASS (-Werror altında)
```

### Deterministic Regression Hook (Local Kanıt)

**Not:** Local ortam Darwin olduğu için baseline/authority drift nedeniyle gate FAIL döner (beklenen). Ama hook'un deterministik gecikme ürettiği metrikte açıkça görülür.

Her koşudan önce `make clean`:

**Delay = 0**
- `RUN_ID=local-canary-pass`
- `AYKEN_INTENTIONAL_PERF_REGRESSION_MS=0`
- `boot_time_ms = 11165`

**Delay = 2000**
- `RUN_ID=local-canary-delay`
- `AYKEN_INTENTIONAL_PERF_REGRESSION_MS=2000`
- `boot_time_ms = 13853`

**Delta: +2688 ms**

Hook deterministik gecikmeyi metrikte üretmiştir.

**Kanıt dosyaları:**
- `evidence/run-local-canary-pass/gates/performance/report.json`
- `evidence/run-local-canary-delay/gates/performance/report.json`

## Commitler

- `965cabe8` docs: add performance baseline governance policy
- `4e556c8c` ci(perf): centralize baseline authority and harden governance policy
- `c0164547` perf(test): add deterministic intentional regression hook
- `96cf41a3` build(validation): make validation-strict clean under -Werror
- `1b01cf08` tools(dev): organize baseline helper scripts
- `f17d7269` ci(perf): preserve local authority override with centralized default

## Risk / Rollback

- **Risk düşük:** Commit sınırları modüler ve izlenebilir.
- **Rollback seçenekleri:**
  - Canary hook geri alınacaksa → `c0164547` revert
  - Local override fix geri alınacaksa → `f17d7269` revert
  - Governance ve strict-clean bağımsızdır

---

## Reviewer Notes

Aşağıdaki 6 noktanın doğrulanması yeterlidir:

1. `scripts/ci/perf_authority.env` tek authority kaynağıdır.
2. `Makefile`, `.github/workflows/ci-freeze.yml`, `.github/workflows/perf-baseline-init.yml`, `scripts/ci/gate_performance.sh` authority konusunda tutarlıdır.
3. `PERF_BASELINE_AUTHORITY` env override çalışır.
4. Hook timer/tick tabanlıdır, default OFF'tur.
5. validation-strict -Werror altında geçer.
6. Canary'de delay=2000 koşusunda boot_time_ms artışı raporda görülür.

---

## Approve Checklist

- [ ] Authority tek kaynaktan yönetiliyor.
- [ ] Env override doğru çalışıyor.
- [ ] Deterministic regression hook default OFF.
- [ ] validation-strict PASS.
- [ ] Helper script'ler `tools/dev/baseline/` altında.
- [ ] Canary'de metrik artışı raporda mevcut.
- [ ] Commit sınırları net ve izlenebilir.

---

## Merge Gate (Short)

- [ ] Authority tek kaynaktan ve CI/perf yollarında tutarlı.
- [ ] Regression hook açıldığında `boot_time_ms` artıyor.
- [ ] validation-strict PASS ve repo hijyeni tamam.

**Merge condition:** strict PASS + deterministic delta observed + authority single source verified.
