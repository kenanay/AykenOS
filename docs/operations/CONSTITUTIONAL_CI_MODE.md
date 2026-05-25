# Constitutional CI Mode

## Status: DEFAULT FREEZE / LOCKED PERFORMANCE AUTHORITY

**Effective date:** 2026-05-25
**Current phase:** Phase-17 active; formal closure pending
**Previous remotely tested S2-D head before accepted-main restack:** `342deab6`
(`ci-gate-phase17-performance-acceptance` run `26391379459` PASS and full
`ci-freeze` run `26391379462` PASS)
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime, evidence,
baseline, merge or closure authority.

## Temel Ilke

Constitutional mode, freeze ve locked-baseline performance kabulunde
fail-closed authority modudur. Gate ihlali, environment authority drift'i
veya baseline regression'i PASS olarak sunulamaz.

## Canonical Workflow Konfigurasyonu

`ci-freeze` freeze job'u ve Phase-17 scoped performance acceptance workflow'u
asagidaki kontrati kullanir:

| Ayar | Constitutional kontrat |
|---|---|
| `PERF_BASELINE_MODE` | `constitutional` |
| `PERF_ENV_MISMATCH_POLICY` | `fail` |
| `PERF_REGRESSION_POLICY` | `fail` |
| `PERF_REQUIRE_CI_FOR_BASELINE_INIT` | `1` |
| Runner | `ubuntu-24.04` |
| Baseline authority | `scripts/ci/perf_authority.env` |
| Baseline lock | `scripts/ci/perf-baseline.lock.json` |

## Strict Freeze Zinciri

`make ci-freeze`, iki fail-fast precondition ve 40 strict gate/cluster
hedefinden olusur. Sirayi bu runbook icinde ikinci bir elle kopya olarak
tutmak yerine canonical kayitlara baglariz:

1. Calistirilan authority: `Makefile` icindeki `ci-freeze` target'i.
2. Review/borc envanteri:
   `docs/governance/CI_GATE_INVENTORY_AND_DEBT_CONTROL_2026_05_25.md`.
3. Gate-order workflow aciklamasi:
   `docs/roadmap/freeze-enforcement-workflow.md`.

Precondition'lar `ci-freeze-guard` ve `preflight-mode-guard`'dir. Bunlardan
sonra gelen strict zincirde performance otoritesi `ci-gate-performance`
tarafindan, evidence/authority ayrimi ise ilgili governance ve isolation
kapilari tarafindan korunur.

## Phase-17 Scoped Performance Acceptance

`.github/workflows/ci-gate-phase17-performance-acceptance.yml`,
constitutional `ci-gate-performance` raporunu Phase-17 kapsamli acceptance
raporuna baglar.

Bu lane:

- timer/preemption hot path locked baseline yuzeyini olcer;
- validation-only worker veya timeout-race payload latency'sini olcmez;
- uyumlu runner digest ve locked baseline PASS gerektirir;
- tek basina Phase-17 closure, merge veya review authority uretmez.

Accepted-main restack oncesindeki S2-D basi `342deab6` icin scoped run
`26391379459` ve full strict run `26391379462` PASS'tir. Restack SHA'si
ayni kontrolleri yeniden gerektirir.

## Baseline Renewal Siniri

Runner digest degisimi performance regression olarak gizlenmez; environment
authority drift'i olarak fail-closed degerlendirilir. Baseline yenileme:

1. pinned digest ile yetkili baseline-init artifact'i uretir;
2. generated lock'u reviewed PR icinde ve gerekli authorization ile tasir;
3. yeni lock uzerinde constitutional scoped acceptance ve full `ci-freeze`
   PASS olmadan authority kurmaz.

Yetkili artifact uretim yolu `.github/workflows/perf-baseline-init.yml` ile
tanimlanir. `ci-freeze.yml` icindeki provisional manual-init varyanti icin
`docs/operations/PROVISIONAL_CI_MODE.md` kullanilir.

## Merge ve Closure Siniri

Issue #145, `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`
ve eslesen canli repository protection konfigurasyonu ile giderilmistir.
`CODEOWNERS`, Kenan AY icin accountability metadata'sidir; bagimsiz
self-review iddiasi uretmez. PR #142 `main`e kabul edilmistir. PR #144 yeni
`main` tabaninda yeni SHA icin required remote CI almadan merge adayi
sayilamaz. Green CI ve merge sonucu da Phase-17 closure manifest/tag yerine
gecmez.

## Referanslar

- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `.github/workflows/ci-gate-phase17-performance-acceptance.yml`
- `.github/workflows/perf-baseline-init.yml`
- `scripts/ci/perf_authority.env`
- `scripts/ci/gate_performance.sh`
- `docs/operations/PROVISIONAL_CI_MODE.md`
- `docs/governance/CI_GATE_INVENTORY_AND_DEBT_CONTROL_2026_05_25.md`
- `docs/roadmap/freeze-enforcement-workflow.md`
- `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`
- GitHub issue #145 resolution record: `https://github.com/kenanay/AykenOS/issues/145`

---

**Dijital imza / attribution:** Kenan AY - Duzenleyen, Gelistiren,
Olusturan ve Mimari Sorumlu
**Yetki notu:** Belgesel metadata; sistem otoritesi, CI verdict'i veya
runtime karari degildir.
