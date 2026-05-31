# AykenOS Roadmap Documentation
This document is subordinate to `ARCHITECTURE_FREEZE.md`. In case of conflict,
the freeze contract prevails.

**Last authority sync:** 2026-05-31 (Phase-17 closure decision exact-SHA evidence refresh)
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime or merge authority.

Bu dizin, aktif gelistirme siralamasini, freeze enforcement kontratini ve
tarihsel roadmap snapshot'larini ayri otorite sinirlarinda tutar.

## Current Authority Chain

1. `../../ARCHITECTURE_FREEZE.md`: frozen mimari invariants ve merge siniri.
2. `CURRENT_PHASE`: formal aktif faz pointer'i (`CURRENT_PHASE=17`).
3. `CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`: aktif execution
   roadmap; kapsam daraltma, technical debt kontrolu ve PR sirasi.
4. `../../AYKENOS_GUNCEL_DURUM_RAPORU_2026_05_23.md`: mevcut changeset ve
   dogrulama raporu.
5. `freeze-enforcement-workflow.md`: strict CI/gate order kontrati.
6. `../../PHASE18_ROADMAP.md`: yalnizca sonraki faz planlamasi; aktif degil.

## Current Status

| Konu | Durum |
|---|---|
| Son resmi kapanis | Phase-16 OFFICIALLY CLOSED (`phase16-official-closure`) |
| Aktif faz | Phase-17 ACTIVE / OFFICIAL CLOSURE DECISION READY, TAG PENDING |
| Aktif odak | Stabilization-first Phase-17 acceptance evidence accepted on `main`; closure decision exact-SHA evidence refreshed on `416a5392`; official closure tag verification still pending |
| ABI | Canonical `1000-1011` / 12 syscall, ABI version `0x00010001` |
| Phase-18 | ROADMAP ONLY; Phase-17 closure olmadan aktivasyon yok |

## Active Execution Rule

Phase-17 resmi kapanis tag'i uretilip dogrulanana kadar production roadmap'e
yeni syscall, yeni platform feature'i, authority-genisleten observability veya
AI orchestration isi alinmaz. Bounded runtime/QEMU lanes, strict `ci-freeze`,
standalone locked Performance Gate ve scoped Phase-17 performance acceptance
accepted `main` SHA `416a5392` uzerinde PASS vermistir. Bu evidence seti
`reports/phase17_official_closure_candidate/` altinda decision package
girdisidir; tag dogrulanana kadar Phase-18 activation veya genis BCIB
semantic/race/SMP kapsami kurmaz.

## Historical References

Asagidaki belgeler baglamsal veya tarihsel snapshot'tir; current phase veya
current execution priority otoritesi degildir:

- `overview.md` (2026-04-24 durum snapshot'i).
- `ROADMAP_2026_02_23.md` (eski aktif plan snapshot'i).
- `phase-4-4-status.md` ve `phase-4-5-spec.md`.
- `constitutional-system-roadmap.md` (governance system corpus; OS execution
  faz pointer'i degildir).

---

**Next action:** `reports/phase17_official_closure_candidate/` exact-SHA
decision package'ini review etmek; `phase17-official-closure` tag'i yalniz
`416a5392afbe217e16d26a59e2e1716fdfa9c8f6` subject SHA uzerinde mint/verify
edilir. Subject SHA degisirse required remote controls yeniden ayni exact-SHA
uzerinde calistirilir.
