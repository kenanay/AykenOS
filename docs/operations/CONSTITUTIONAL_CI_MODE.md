# Constitutional CI Mode

## Status: Default Freeze Mode

Bu belge, freeze kararlarinda kullanilan varsayilan CI modunu aciklar.

## Temel Ilke
Constitutional mode'da gate ihlali merge-blocking davranis uretir (fail-closed).

## Freeze Job Konfigurasyon Ozet
Kaynak: `.github/workflows/ci-freeze.yml`

- `PERF_BASELINE_MODE=constitutional`
- `PERF_ENV_MISMATCH_POLICY=fail`
- `PERF_REGRESSION_POLICY=fail`
- `PERF_REQUIRE_CI_FOR_BASELINE_INIT=1`
- `runs-on: ubuntu-24.04`

## Gate Zinciri
`make ci-freeze` ile calisan zincir:
1. abi
2. boundary
3. ring0-exports
4. hygiene
5. tooling-isolation
6. constitutional
7. workspace
8. syscall-v2-runtime
9. performance

## Provisional ile Iliski
- Baseline init veya diagnostik calismalarda provisional yol kullanilabilir.
- Ancak freeze/merge karari icin esas otorite constitutional mode'dur.

## Operasyonel Not
`ci-summarize` tarafinda `PASS/SKIP/WARN` kabul mantigi vardir; bu nedenle run yorumlanirken gate bazli raporlar da birlikte incelenmelidir.

## Referanslar
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `tools/ci/summarize.sh`
- `docs/operations/PROVISIONAL_CI_MODE.md`

---
**Son Guncelleme:** 2026-02-21
