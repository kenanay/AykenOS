# AykenOS Roadmap Documentation
This document is subordinate to `ARCHITECTURE_FREEZE.md`. In case of conflict,
the freeze contract prevails.

**Last authority sync:** 2026-05-24 (PR-4B bounded local variance isolation update)
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
| Aktif faz | Phase-17 ACTIVE / FORMAL CLOSURE PENDING |
| Aktif odak | Stabilization-first: PR-1 lifecycle, PR-2 determinism/negative, PR-2A public S1.E2E, PR-2B stub-off bounded fixture completion ve PR-3 IRQ timeout-race kernel/QEMU local PASS; PR-4 readiness FAIL; PR-4A ortak `sample-6` diagnostic local PASS; PR-4B bounded kampanyada sapma yeniden uretilmedi / kok neden ve remote locked authority pending |
| ABI | Canonical `1000-1011` / 12 syscall, ABI version `0x00010001` |
| Phase-18 | ROADMAP ONLY; Phase-17 closure olmadan aktivasyon yok |

## Active Execution Rule

Phase-17 resmi kapanis kaniti uretilene kadar production roadmap'e yeni
syscall, yeni platform feature'i, authority-genisleten observability veya AI
orchestration isi alinmaz. Marker-enabled gercek kernel/QEMU lifecycle ile
determinism/invalid-order local kaniti 2026-05-23 tarihinde, public Ring3
submit/wait S1.E2E validation-only kaniti 2026-05-24 tarihinde PASS uretti;
stub kapali bounded fixture worker public completion kaniti da 2026-05-24
tarihinde PASS uretti. Validation-only bounded deadline ile gercek timer IRQ
timeout'u kazanirken gecikmis public `1011` reddi de PR-3 olarak 2026-05-24
tarihinde local PASS uretti. Mevcut timer/preemption hot-path uzerindeki
PR-4 local median alt-kapisi PASS uretti, ancak local readiness validator'u
repeat stability range guard ihlallerini fail-closed `FAIL` olarak kaydetti.
PR-4A diagnostics-only hedefi bu FAIL raporunu referans PASS raporuyla
karsilastirip uc proxy'de ortak `sample-6` outlier'i siniflandirdi; PASS
sonucu yalniz analiz butunlugudur ve feature payload latency'sini veya
closure'i kanitlamaz. PR-4B, ayni runtime kontratiyla image-reuse ve
rebuild-per-run bounded kosularinda bu sapmayi yeniden uretmedi; bu da
onceki readiness FAIL'i veya kok neden belirsizligini kaldirmaz. Clean-tree
remote runtime kabulü ve locked-baseline performance authority bekler; genel
BCIB semantic ya da broader race/SMP kapsamı gerekirse ayrik paketlenir.

## Historical References

Asagidaki belgeler baglamsal veya tarihsel snapshot'tir; current phase veya
current execution priority otoritesi degildir:

- `overview.md` (2026-04-24 durum snapshot'i).
- `ROADMAP_2026_02_23.md` (eski aktif plan snapshot'i).
- `phase-4-4-status.md` ve `phase-4-5-spec.md`.
- `constitutional-system-roadmap.md` (governance system corpus; OS execution
  faz pointer'i degildir).

---

**Next action:** `CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`
altindaki PR-0, locally stacked PR-1, PR-2, PR-2A, PR-2B, PR-3 ve PR-4
readiness wiring'ini clean-tree PR CI ile kabul ettirmeden once PR-4A'nin
ortak `sample-6` varyansi icin PR-4B bounded non-reproduction kaydini
korumak; simdi remote locked performance acceptance sonucunu almak ve sapma
remote ortamda yinelenirse ayni stage-localization ayrimini orada
calistirmak. Feature-specific latency, genel BCIB semantic ya da broader
race/SMP kapsami gerekiyorsa ayrik sinirli paketlerle ilerletilir.
