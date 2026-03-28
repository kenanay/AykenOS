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
`make ci-freeze` ile calisan strict zincir (as-of 2026-03-28):
1. `ci-gate-abi`
2. `ci-gate-boundary`
3. `ci-gate-ring0-exports`
4. `ci-gate-hygiene`
5. `ci-gate-tooling-isolation`
6. `ci-gate-constitutional`
7. `ci-gate-governance-policy`
8. `ci-gate-naming-convention`
9. `ci-gate-drift-activation`
10. `ci-gate-structural-abi`
11. `ci-gate-runtime-marker-contract`
12. `ci-gate-user-bin-lock`
13. `ci-gate-embedded-elf-hash`
14. `ci-gate-performance`
15. `ci-gate-ring3-user-leaf-rule`
16. `ci-gate-ring3-execution-phase10a2`
17. `ci-gate-syscall-semantics-phase10b`
18. `ci-gate-low-half-kheap-scaffold`
19. `$(PHASE10C_FREEZE_GATE)`
20. `ci-gate-mailbox-capability-negative`
21. `ci-gate-workspace`
22. `ci-gate-syscall-v2-runtime`
23. `ci-gate-sched-bridge-runtime`
24. `ci-gate-behavioral-suite`
25. `ci-gate-policy-accept`
26. `ci-gate-alias-proof`
27. `ci-kill-switch-phase13`

## Pre-CI ile Iliski
- `make pre-ci` local discipline katmanidir (4 gate)
- CI'nin yerine gecmez
- Dirty tracked state varsa hygiene fail beklenir
- Dedicated local deterministic gate'ler CI topology icinde canli olabilir; bu, broader historical strict/global authority iddiasini tek basina yeniden kurmaz.

## Ring3 Authority Split
- `ci-gate-ring3-user-leaf-rule` = local deterministic executable-leaf rule authority
- `ci-gate-ring3-execution-phase10a2` = broader historical Phase10-A2 strict/global authority surface

## Provisional ile Iliski
- Baseline init veya diagnostik kosullarda provisional yol kullanilabilir
- Merge/freeze karari icin esas otorite constitutional mode'dur

## Referanslar
- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `scripts/ci/pre_ci_discipline.sh`
- `docs/operations/PROVISIONAL_CI_MODE.md`

---
**Son Guncelleme:** 2026-03-28
