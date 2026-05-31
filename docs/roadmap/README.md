# AykenOS Roadmap Documentation
This document is subordinate to `ARCHITECTURE_FREEZE.md`. In case of conflict,
the freeze contract prevails.

**Last authority sync:** 2026-05-31 (Phase-17 official closure tag verification)
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
6. `../../PHASE18_TRANSITION_DECISION.md`: Phase-18 Platform Constitution
   transition package; aktif faz pointer'i degildir.
7. `../../PHASE18_ROADMAP.md`: tarihsel pre-closure runtime-validation
   planlamasi; aktif Phase-18 otoritesi degildir.

## Current Status

| Konu | Durum |
|---|---|
| Son resmi kapanis | Phase-17 OFFICIALLY CLOSED (`phase17-official-closure` at `416a5392`) |
| Aktif faz | Phase-17 OFFICIALLY CLOSED / Phase-18 transition not activated |
| Aktif odak | Phase-18 Platform Constitution transition package review; no activation without explicit pointer transition |
| ABI | Canonical `1000-1011` / 12 syscall, ABI version `0x00010001` |
| Phase-18 | TRANSITION DECISION PACKAGE ONLY; kernel expansion and new syscalls forbidden unless a separate phase RFC/closure authority exists |

## Active Execution Rule

Phase-17 resmi kapanis tag'i `416a5392` uzerinde uretilip dogrulanmistir.
Phase-18 icin ayri transition karari aktif pointer'a baglanana kadar production
roadmap'e yeni syscall, kernel ABI genislemesi, Ring0 policy, authority-genisleten
observability veya AI orchestration isi alinmaz. Bounded runtime/QEMU lanes,
strict `ci-freeze`, standalone locked Performance Gate ve scoped Phase-17
performance acceptance accepted `main` SHA `416a5392` uzerinde PASS vermistir.
Bu closure, Phase-18 activation veya genis BCIB semantic/race/SMP kapsami kurmaz.
Phase-18'in proposed direction'i Platform Constitution'dir: module/package,
workspace, capability, trust classification ve plugin boundary kontratlari.

## Historical References

Asagidaki belgeler baglamsal veya tarihsel snapshot'tir; current phase veya
current execution priority otoritesi degildir:

- `overview.md` (2026-04-24 durum snapshot'i).
- `ROADMAP_2026_02_23.md` (eski aktif plan snapshot'i).
- `phase-4-4-status.md` ve `phase-4-5-spec.md`.
- `constitutional-system-roadmap.md` (governance system corpus; OS execution
  faz pointer'i degildir).

---

**Next action:** `../../PHASE18_TRANSITION_DECISION.md` review edilip
fail-closed Phase-18 transition karari ayrica verilir; `CURRENT_PHASE` explicit
pointer transition olmadan `18` yapilmaz.
