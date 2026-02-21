# Provisional CI Mode

## Status: AVAILABLE (Not Default for Freeze)

Bu belge provisional mode'un ne zaman ve nasil kullanilacagini aciklar.

## Gercek Durum
- `ci-freeze` workflow freeze job'u varsayilan olarak **constitutional** modda calisir.
- Provisional mode su an esas olarak su yollar icin kullanilir:
  1. Baseline init akislari
  2. Hosted/local diagnostik run'lar
  3. Deterministik olmayan ortamlarda gecici olcum

## Provisional Mode Nedir?
Hosted/non-deterministic ortamlarda performans ve runtime gate esiklerini yumusatmak icin kullanilan operasyon modudur.

### Tipik etkiler
- Runtime gate daha gevsek timeout/run/success-rate ile calisabilir.
- Performance gate ihlallerinde `WARN` uretebilir (hard fail yerine).
- Tooling isolation gate provisional modda `SKIP` uretebilir.

## Ne Zaman Kullanilmali?
- Baseline uretimi veya tanisal (diagnostic) calismalar
- Deterministik authority disi hostlarda gecici olcum

## Ne Zaman Kullanilmamali?
- Mainline freeze kararini tek basina vermek icin
- "Production-ready" claim'i yapmak icin

## Referanslar
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `scripts/ci/gate_performance.sh`
- `scripts/ci/gate_syscall_v2_runtime.sh`
- `docs/operations/CONSTITUTIONAL_CI_MODE.md`

---
**Son Guncelleme:** 2026-02-21
