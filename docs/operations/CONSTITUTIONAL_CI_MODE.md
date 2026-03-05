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
`make ci-freeze` ile calisan strict zincir (as-of 2026-03-05):
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

## Pre-CI ile Iliski
- `make pre-ci` local discipline katmanidir (4 gate)
- CI'nin yerine gecmez
- Dirty tracked state varsa hygiene fail beklenir

## Provisional ile Iliski
- Baseline init veya diagnostik kosullarda provisional yol kullanilabilir
- Merge/freeze karari icin esas otorite constitutional mode'dur

## Referanslar
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `scripts/ci/pre_ci_discipline.sh`
- `docs/operations/PROVISIONAL_CI_MODE.md`

---
**Son Guncelleme:** 2026-03-05
