# AykenOS Documentation Index
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-03-10
**Snapshot Basis:** `local-freeze-p10p11` + `local-phase11-closure` (`evidence_sha=9cb2171b`, `closure_sync_sha=fe9031d7`, `ci_freeze_run=22797401328`)

## Current Status
- **Runtime:** `Phase-10` officially closed via freeze evidence + remote `ci-freeze`
- **Verification Substrate:** `Phase-11` officially closed via proof-chain evidence + remote `ci-freeze`
- **Phase-12 Local Track:** verifier / CLI / receipt / audit / exchange / parity diagnostics gates active in the current worktree
- **Formal Governance Pointer:** `CURRENT_PHASE=10` (phase transition not yet executed)
- **Next Focus:** official closure tag, `P12-14` determinism-severity hardening, `P12-16` `proofd` read-only diagnostics prep

## Primary Truth Sources
Current repo truth icin once su dosyalari referans alin:

1. `README.md`
2. `AYKENOS_SON_DURUM_RAPORU_2026_03_07.md`
3. `RAPOR_OZETI_2026_03_07.md`
4. `reports/phase10_phase11_closure_2026-03-07.md`
5. `docs/development/PROJECT_STATUS_REPORT.md`
6. `docs/roadmap/overview.md`
7. `docs/specs/phase11-verification-substrate/tasks.md`
8. `Makefile`
9. `.github/workflows/ci-freeze.yml`
10. `docs/specs/phase12-trust-layer/tasks.md`
11. `docs/specs/phase12-trust-layer/PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`

## Live Evidence References
1. `evidence/run-local-freeze-p10p11/reports/summary.json`
2. `evidence/run-local-phase11-closure/reports/summary.json`
3. `evidence/run-local-freeze-p10p11/gates/`
4. `evidence/run-local-phase11-closure/gates/`

## CI / Freeze Documentation
1. `docs/operations/CONSTITUTIONAL_CI_MODE.md`
2. `docs/operations/PROVISIONAL_CI_MODE.md`
3. `docs/operations/PERF_BASELINE_POLICY.md`
4. `docs/roadmap/freeze-enforcement-workflow.md`
5. `docs/operations/RUNTIME_INTEGRATION_GUARDRAILS.md`

## Development Notes
1. `docs/development/VENDORED_TOOLCHAIN_SNAPSHOTS.md`

## Roadmap and Status Surfaces
1. `docs/roadmap/README.md`
2. `docs/roadmap/overview.md`
3. `docs/roadmap/CURRENT_PHASE`
4. `docs/development/PROJECT_STATUS_REPORT.md`

## Phase-11 Reference Set
1. `docs/specs/phase11-verification-substrate/design.md`
2. `docs/specs/phase11-verification-substrate/requirements.md`
3. `docs/specs/phase11-verification-substrate/tasks.md`
4. `docs/architecture-board/ABDF_BCIB_PHASE11_CONTRACT_MATRIX.md`
5. `docs/architecture-board/RUNTIME_STATE_MACHINE.md`

## Phase-12 Reference Set
1. `docs/specs/phase12-trust-layer/tasks.md`
2. `docs/specs/phase12-trust-layer/requirements.md`
3. `docs/specs/phase12-trust-layer/PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
4. `docs/specs/phase12-trust-layer/PROOF_VERIFIER_SEMANTIC_CLI_ROADMAP.md`
5. `docs/specs/phase12-trust-layer/PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`
6. `docs/specs/phase12-trust-layer/VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`
7. `docs/specs/phase12-trust-layer/PARITY_LAYER_ARCHITECTURE.md`
8. `docs/specs/phase12-trust-layer/PARITY_LAYER_FORMAL_MODEL.md`
9. `docs/specs/phase12-trust-layer/N_NODE_CONVERGENCE_FORMAL_MODEL.md`
10. `docs/specs/phase12-trust-layer/AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`
11. `docs/specs/phase12-trust-layer/CROSS_NODE_PARITY_HARDENING_CHECKLIST.md`
12. `docs/specs/phase12-trust-layer/PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`

## Historical / Superseded Snapshots
Asagidaki dosyalar tarihsel snapshot niteligindedir; current truth yerine dogrudan kullanilmamalidir:

1. `AYKENOS_SON_DURUM_RAPORU_2026_03_05.md`
2. `PROJE_DURUM_RAPORU_2026_03_02.md`
3. `PHASE_10_FINAL_STATUS.md`
4. `PHASE_10_COMPLETION_SUMMARY.md`
5. `AYKENOS_PROJE_GENEL_YAPI_VE_MIMARI_RAPORU.md`

## Note
Eski raporlarda gecen blocker veya progress ifadeleri tarihsel baglam icindir. Current status yorumlari icin 2026-03-07 official closure truth ve yukaridaki primary truth kaynaklari kullanilmalidir.
