# AykenOS Roadmap Documentation
This document is subordinate to `ARCHITECTURE_FREEZE.md`. In case of conflict,
the freeze contract prevails.

**Last authority sync:** 2026-06-06 (Phase-19 Pointer Transition Decision)
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime or merge authority.

Bu dizin, aktif gelistirme siralamasini, freeze enforcement kontratini ve
tarihsel roadmap snapshot'larini ayri otorite sinirlarinda tutar.

## Current Authority Chain

1. `../../ARCHITECTURE_FREEZE.md`: frozen mimari invariants ve merge siniri.
2. `CURRENT_PHASE`: formal aktif faz pointer'i (`CURRENT_PHASE=19`).
3. `CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`: aktif execution
   roadmap; kapsam daraltma, technical debt kontrolu ve PR sirasi.
4. `../../AYKENOS_GUNCEL_DURUM_RAPORU_2026_05_23.md`: mevcut changeset ve
   dogrulama raporu.
5. `freeze-enforcement-workflow.md`: strict CI/gate order kontrati.
6. `../../PHASE18_TRANSITION_DECISION.md`: accepted Phase-18 Platform
   Constitution transition package.
7. `../specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md`:
   active Phase-18 Platform Constitution module manifest RFC; authority grant degildir.
8. `../specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md`:
   capability request, decision, receipt ve revocation active RFC; token veya
   trust grant degildir.
9. `../specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md`:
   workspace admission, logical mount, disable, quarantine, revocation ve
   removal active RFC; mount veya capability grant degildir.
10. `../specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md`:
   package identity, version, publisher, hash, signature, dependency ve
   Platform ABI compatibility active RFC; trust/capability/workspace/execution
   grant degildir.
11. `../specs/phase18-platform-constitution/TRUST_CLASSIFICATION_MODEL.md`:
   trust vocabulary, evidence inputs, classification lifecycle ve fail-closed
   policy effects active RFC; capability veya runtime authority grant degildir.
12. `../specs/phase18-platform-constitution/PLUGIN_BOUNDARY_CONTRACT.md`:
   host interface, extension point, compatibility ve binding lifecycle RFC
   active RFC; plugin loading veya capability/trust/workspace authority grant
   degildir.
13. `../specs/phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md`:
   Platform ABI validation order, input bundle, stage result ve validation
   receipt active RFC; validation PASS authority grant degildir.
14. `../specs/phase18-platform-constitution/CROSS_CONSISTENCY_REVIEW.md`:
   Phase-18 RFC set capraz tutarlilik review kaydi; activation veya runtime
   implementation yetkisi degildir.
15. `../../PHASE18_ACTIVATION_DECISION.md`: accepted Phase-18 activation
   decision package; runtime implementation yetkisi degildir.
16. `../specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`:
   active Phase-18 review guard; runtime implementation, loader, issuer veya
   Phase-19 authority grant degildir.
17. `../specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`:
   accepted Phase-18 terminology audit; high-risk vocabulary icin runtime
   authority kurmaz.
18. `../../PHASE19_RUNTIME_DECISION.md`: Phase-19 Platform Runtime MVP
   decision package; implementation authority degildir.
19. `../specs/phase19-platform-runtime/README.md`: Phase-19 Runtime MVP
   active planning/admission/receipt RFC set; runtime implementation authority
   degildir.
20. `../specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`:
   Phase-19 Runtime evidence matrix; evidence PASS veya implementation
   authority degildir.
21. `../specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`:
   Phase-19 Runtime RFC set capraz tutarlilik review kaydi; runtime
   implementation yetkisi degildir.
22. `../../PHASE19_POINTER_TRANSITION_CANDIDATE.md`: Phase-19 pointer
   transition kosullarini tanimlayan accepted candidate kaydi; implementation
   authority degildir.
23. `../../PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md`: Phase-19 activation
   precondition review kaydi; implementation authority degildir.
24. `../../PHASE19_POINTER_TRANSITION_DECISION.md`: `CURRENT_PHASE=19`
   pointer transition decision; yalniz planning/admission/receipt boundary
   kurar, runtime implementation authority degildir.
25. `../../PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md`:
   implementation decision candidate; sonraki exact-SHA decision sinirini
   daraltir, runtime source code veya implementation authority kurmaz.
26. `../../PHASE18_ROADMAP.md`: tarihsel pre-closure runtime-validation
   planlamasi; aktif Phase-18 otoritesi degildir.

## Current Status

| Konu | Durum |
|---|---|
| Son resmi kapanis | Phase-17 OFFICIALLY CLOSED (`phase17-official-closure` at `416a5392`) |
| Aktif faz | Phase-19 ACTIVE / Platform Runtime MVP planning, admission, and receipt boundary only |
| Aktif odak | Phase-19 planning/admission/receipt authority maintenance; runtime implementation remains out of scope |
| Siradaki docs-only odak | `RUNTIME_EVIDENCE_MATRIX.md` sonraki implementation decision icin kanit satirlarini map eder; source code remains unauthorized until a separate exact-SHA implementation decision |
| ABI | Canonical `1000-1011` / 12 syscall, ABI version `0x00010001` |
| Phase-18 | ACCEPTED PLATFORM CONSTITUTION REFERENCE SET; kernel expansion, runtime implementation and new syscalls forbidden unless a separate phase RFC/closure authority exists |
| Phase-19 | ACTIVE AS PLANNING / VALIDATION-INTEGRATION / ADMISSION-RECORD / RECEIPT BOUNDARY; implementation forbidden until a separate decision |

## Active Execution Rule

Phase-17 resmi kapanis tag'i `416a5392` uzerinde uretilip dogrulanmistir.
Phase-18 accepted pointer'i yalniz Platform Constitution kapsamindadir.
`CURRENT_PHASE=19` yalniz Platform Runtime MVP planning,
validation-integration, admission-record ve receipt-boundary kapsamindadir.
Production roadmap'e yeni syscall, kernel ABI genislemesi, Ring0 policy,
runtime loader, package installer, workspace runtime, plugin loading,
capability issuer, trust issuer, authority-genisleten observability veya AI
orchestration isi alinmaz. Bounded runtime/QEMU lanes, strict `ci-freeze`,
standalone locked Performance Gate ve scoped Phase-17 performance acceptance
accepted `main` SHA `416a5392` uzerinde PASS vermistir. Bu closure, genis
BCIB semantic/race/SMP kapsami veya Phase-19 runtime implementation kurmaz.
Phase-18 reference direction'i Platform Constitution'dir:
module/package/workspace, capability, trust classification ve plugin boundary
kontratlari. Phase-19 active direction'i inert input bundle, validation
integration, workspace admission record ve deterministic runtime receipt
siniridir.

## Historical References

Asagidaki belgeler baglamsal veya tarihsel snapshot'tir; current phase veya
current execution priority otoritesi degildir:

- `overview.md` (2026-04-24 durum snapshot'i).
- `ROADMAP_2026_02_23.md` (eski aktif plan snapshot'i).
- `phase-4-4-status.md` ve `phase-4-5-spec.md`.
- `constitutional-system-roadmap.md` (governance system corpus; OS execution
  faz pointer'i degildir).

---

**Next action:** `docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`
sonraki implementation decision icin artifact, positive, negative,
determinism, remote ve production-default kanit satirlarini docs-only olarak
map eder. Gercek Phase-19 runtime implementation karar paketi ancak ayri
exact-SHA PR ve evidence package ile degerlendirilebilir. `CURRENT_PHASE=19`
runtime source code, loader, installer, workspace runtime, plugin host,
capability issuer, trust issuer, Semantic CLI authority veya AI Runtime
authority vermez. High-risk vocabulary `TERMINOLOGY_AUDIT.md` kaydina gore
denetlenir.
