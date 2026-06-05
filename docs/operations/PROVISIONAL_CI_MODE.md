# Provisional CI Mode

## Status: DIAGNOSTIC / BASELINE-CANDIDATE ONLY

**Effective date:** 2026-05-25
**Current phase:** Phase-18 active as Platform Constitution only; runtime implementation not authorized
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime, evidence,
baseline, merge or closure authority.

## Authority Boundary

`provisional` mode, deterministik olmayan host kosullarinda olcum almak veya
baseline adayi uretmek icin kullanilan sinirli bir operasyon yoludur.

Bu mod:

- `ci-freeze` PASS yerine gecmez;
- locked-baseline Phase-17 performance acceptance kurmaz;
- baseline lock degisikligini kendiliginden yetkilendirmez;
- merge, review-enforcement veya Phase-17 closure otoritesi uretmez.

Strict freeze ve Phase-17 locked performance kabulunun authority modu
`PERF_BASELINE_MODE=constitutional` olarak kalir.

## Canli Kullanim Yollari

| Yol | Canonical kaynak | Urettigi sonuc | Yetki siniri |
|---|---|---|---|
| Freeze-workflow provisional baseline init dispatch | `.github/workflows/ci-freeze.yml`, `init_perf_baseline=true` | Pinned CI digest'e bagli generated baseline artifact adayi | Artifact ancak reviewed renewal PR ve gerekli authorization ile kabul adayi olabilir |
| Local performance measurement | `make ci-gate-performance-local` | Gitignored local baseline uzerinden median/diagnostic rapor | Remote locked baseline veya closure sayilmaz |
| Local stability/readiness | `make ci-gate-performance-stability`, `make ci-gate-phase17-performance-readiness-local` | Stability FAIL/PASS ve local readiness sinyali | Local PASS bile remote acceptance kurmaz |
| Variance diagnosis | `make ci-gate-phase17-performance-variance-diagnostic`, `make ci-gate-phase17-performance-variance-isolation` | Outlier/fingerprint veya bounded reproduction raporu | Root cause, threshold yenileme veya acceptance sayilmaz |

## Kodla Dogrulanmis Davranis

| Yuzey | `provisional` davranisi | Sonuc siniri |
|---|---|---|
| Runtime defaults | `SYSCALL_V2_RUNTIME_RUNS=3`, timeout `40`, gerekli success rate `%60` olabilir | Strict runtime kabulunde kullanilamaz |
| Performance ihlali | `gate_performance.sh`, ihlal varsa `WARN` uretebilir ve hard-fail etmez | Acceptance verdict'i olamaz |
| Eksik baseline | Provisional performance yolu enforcement'i atlayabilir | Locked baseline kaniti degildir |
| Tooling isolation | `ci-gate-tooling-isolation` provisional modda `SKIP` uretebilir | Freeze zincirine authority tasiyamaz |

## Freeze-Workflow Provisional Baseline Init Siniri

`ci-freeze.yml` icindeki provisional baseline-init dali kontrolsuz degildir:

1. Workflow yalniz manual dispatch ve `init_perf_baseline=true` ile acilir.
2. `PERF_REQUIRE_CI_FOR_BASELINE_INIT=1` olarak kalir.
3. `PERF_CI_IMAGE_DIGEST` pinned ve format-valid olmak zorundadir.
4. Generated lock, checkout `HEAD` SHA ile eslesmelidir.
5. Cikti protected branch'e dogrudan push edilmez; artifact olarak
   incelemeye sunulur.
6. Import edilen lock icin sonraki `constitutional` scoped performance ve
   full `ci-freeze` PASS gereklidir.

Bu nedenle baseline init artifact'i evidence girdisidir; tek basina baseline
kabulu veya faz kapanisi degildir.

Yetkili renewal artifact yolunun ayrica
`.github/workflows/perf-baseline-init.yml` ile uygulanmasi, provisional
sonucun authority oldugu anlamina gelmez. O artifact de reviewed PR import'u
ve sonraki `constitutional` scoped/full PASS ile degerlendirilir.

## Phase-17 Guncel Sinir

PR #144 accepted-main restack oncesindeki son uzaktan dogrulanmis S2-D
basinda (`342deab6`):

- scoped locked-baseline performance run `26391379459` PASS;
- full strict `ci-freeze` run `26391379462` PASS.

Bu PASS sonuclari `constitutional` modda uretilmistir; provisional sonuca
donusturulmez. Issue #145 tek-maintainer authority karari ve eslesen canli
protection ile giderilmis, PR #142 `main`e kabul edilmistir. PR #144
accepted `main` uzerine restack edildiginde yeni SHA icin remote acceptance
yeniden gereklidir; hicbir provisional veya remote PASS tek basina
Phase-17 closure otoritesi kurmaz.

## Referanslar

- `Makefile`
- `.github/workflows/ci-freeze.yml`
- `.github/workflows/perf-baseline-init.yml`
- `.github/workflows/ci-gate-phase17-performance-acceptance.yml`
- `scripts/ci/gate_performance.sh`
- `scripts/ci/gate_performance_local.sh`
- `docs/operations/CONSTITUTIONAL_CI_MODE.md`
- `docs/governance/CI_GATE_INVENTORY_AND_DEBT_CONTROL_2026_05_25.md`
- `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`
- GitHub issue #145 resolution record: `https://github.com/kenanay/AykenOS/issues/145`

---

**Dijital imza / attribution:** Kenan AY - Duzenleyen, Gelistiren,
Olusturan ve Mimari Sorumlu
**Yetki notu:** Belgesel metadata; sistem otoritesi, CI verdict'i veya
runtime karari degildir.
