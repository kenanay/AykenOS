# AykenOS Documentation Index
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-03-16
**Snapshot Basis:** `local-freeze-p10p11` + `local-phase11-closure` + `run-run-local-phase12c-closure-2026-03-11` (`evidence_sha=9cb2171b`/`01d1cb5c`, `closure_sync_sha=fe9031d7`, `ci_freeze_run=22797401328`/`23099070483`)

## Current Status
- **Runtime:** `Phase-10` officially closed via freeze evidence + remote `ci-freeze` run `22797401328`
- **Verification Substrate:** `Phase-11` officially closed via proof-chain evidence + remote `ci-freeze` run `22797401328`
- **Trust Layer:** `Phase-12` officially closed via normative gate set + remote `ci-freeze` run `23099070483` (PR #62)
- **Phase-13 Kill-Switch:** 6/6 gates PASS at `0ec4bb5e` (tag: `phase13-kill-switch-gates-pass`); boundary hardening active
- **Formal Governance Pointer:** `CURRENT_PHASE=12` (formal transition executed at `0adb2a84`)
- **Next Focus:** Phase-13 boundary hardening workstreams per Architecture Map §4

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
6. `docs/operations/PHASE12_OFFICIAL_CLOSURE_EXECUTION.md`
7. `docs/operations/PHASE_TRANSITION_RUNBOOK.md`

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
### Architecture and Service Surfaces
1. `docs/specs/phase12-trust-layer/tasks.md`
2. `docs/specs/phase12-trust-layer/requirements.md`
3. `docs/specs/phase12-trust-layer/PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
4. `docs/specs/phase12-trust-layer/PROOF_VERIFIER_SEMANTIC_CLI_ROADMAP.md`
5. `docs/specs/phase12-trust-layer/PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`
6. `docs/specs/phase12-trust-layer/PARITY_LAYER_ARCHITECTURE.md`
7. `docs/specs/phase12-trust-layer/CROSS_NODE_PARITY_HARDENING_CHECKLIST.md`
8. `docs/specs/phase12-trust-layer/PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`
9. `docs/specs/phase12-trust-layer/PROOFD_OBSERVABILITY_BOUNDARY_GATE.md`
10. `docs/specs/phase12-trust-layer/GRAPH_NON_AUTHORITATIVE_CONTRACT_GATE.md`
11. `docs/specs/phase12-trust-layer/CONVERGENCE_NON_ELECTION_BOUNDARY_GATE.md`
12. `docs/specs/phase12-trust-layer/DIAGNOSTICS_CONSUMER_NON_AUTHORITATIVE_CONTRACT_GATE.md`
13. `docs/specs/phase12-trust-layer/DIAGNOSTICS_CALLSITE_CORRELATION_GATE.md`
14. `docs/specs/phase12-trust-layer/OBSERVABILITY_ROUTING_SEPARATION_GATE.md`
15. `docs/specs/phase12-trust-layer/AYKENOS_GATE_ARCHITECTURE.md`
16. `docs/specs/phase12-trust-layer/GATE_REGISTRY.md`
17. `docs/specs/phase12-trust-layer/VERIFICATION_DETERMINISM_CONTRACT_GATE.md`
18. `docs/specs/phase12-trust-layer/PROOFD_SERVICE_CLOSURE_PLAN.md`
19. `docs/specs/phase12-trust-layer/PROOFD_SERVICE_FINAL_HARDENING_CHECKLIST.md`
20. `docs/specs/phase12-trust-layer/PHASE12_CLOSURE_ORDER.md`
21. `docs/operations/PHASE12_OFFICIAL_CLOSURE_EXECUTION.md`
22. `docs/specs/phase12-trust-layer/PHASE13_ARCHITECTURE_MAP.md`
23. `docs/specs/phase12-trust-layer/PHASE13_NEGATIVE_TEST_SPEC.md`
24. `docs/specs/phase12-trust-layer/PHASE13_KILL_SWITCH_GATES.md`
25. `docs/specs/phase12-trust-layer/PHASE13_COLLAPSE_SCENARIOS.md`
26. `docs/specs/phase12-trust-layer/VERIFICATION_DIVERSITY_LEDGER_SPEC.md`
27. `docs/specs/phase12-trust-layer/VERIFICATION_DIVERSITY_LEDGER_PRODUCER_SPEC.md`
28. `docs/specs/phase12-trust-layer/VERIFICATION_DIVERSITY_FLOOR_GATE.md`
29. `docs/specs/phase12-trust-layer/VERIFIER_CARTEL_CORRELATION_GATE.md`
30. `docs/specs/phase12-trust-layer/AUTHORITY_SINKHOLE_ABSORPTION_GATE.md`
31. `docs/specs/phase12-trust-layer/AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`
32. `docs/specs/phase12-trust-layer/TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`
33. `docs/specs/phase12-trust-layer/CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`
34. `docs/specs/phase12-trust-layer/VERIFIER_REPUTATION_PROHIBITION_GATE.md`
35. `docs/specs/phase12-trust-layer/AYKENOS_ARCHITECTURE_ONE_PAGE.md`
36. `docs/specs/phase12-trust-layer/AYKENOS_GLOBAL_ARCHITECTURE_DIAGRAM.md`
37. `docs/specs/phase12-trust-layer/AYKENOS_TECHNICAL_DEFINITION_SET.md`
38. `docs/specs/phase12-trust-layer/AYKENOS_SYSTEM_POSITIONING_TABLE.md`

### Verification Core
23. `docs/specs/phase12-trust-layer/VERIFICATION_MODEL.md`
24. `docs/specs/phase12-trust-layer/VERIFICATION_INVARIANTS.md`
25. `docs/specs/phase12-trust-layer/VERIFICATION_FAILURE_MODEL.md`
26. `docs/specs/phase12-trust-layer/VERIFICATION_OBSERVABILITY_MODEL.md`
27. `docs/specs/phase12-trust-layer/VERIFICATION_RELATIONSHIP_GRAPH.md`
28. `docs/specs/phase12-trust-layer/GLOBAL_VERIFICATION_GRAPH_MODEL.md`
29. `docs/specs/phase12-trust-layer/ARTIFACT_SCHEMA.md`
30. `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_MODEL.md`
31. `docs/specs/phase12-trust-layer/PARITY_GRAPH_MODEL.md`
32. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_TOPOLOGY.md`

### Theory and Formal Set
33. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_THEORY.md`
34. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_SYSTEMS.md`
35. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_SYSTEMS_FORMAL_MODEL.md`
36. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_SYSTEMS_SECURITY_MODEL.md`
37. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_SYSTEMS_VS_CAP_THEOREM.md`
38. `docs/specs/phase12-trust-layer/PARITY_LAYER_FORMAL_MODEL.md`
39. `docs/specs/phase12-trust-layer/N_NODE_CONVERGENCE_FORMAL_MODEL.md`
40. `docs/specs/phase12-trust-layer/AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`

### Research and Comparative Set
41. `docs/specs/phase12-trust-layer/AYKENOS_RESEARCH_POSITIONING.md`
42. `docs/specs/phase12-trust-layer/AYKENOS_SYSTEM_CATEGORY_NOTE.md`
43. `docs/specs/phase12-trust-layer/AYKENOS_UNIQUE_ARCHITECTURAL_DECISIONS.md`
44. `docs/specs/phase12-trust-layer/AYKENOS_VS_BLOCKCHAIN_ARCHITECTURAL_DIFFERENCE.md`
45. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_SYSTEMS_PAPER_OUTLINE.md`
46. `docs/specs/phase12-trust-layer/DISTRIBUTED_VERIFICATION_SYSTEMS_PAPER.md`

## Historical / Superseded Snapshots
Asagidaki dosyalar tarihsel snapshot niteligindedir; current truth yerine dogrudan kullanilmamalidir:

1. `AYKENOS_SON_DURUM_RAPORU_2026_03_05.md`
2. `PROJE_DURUM_RAPORU_2026_03_02.md`
3. `PHASE_10_FINAL_STATUS.md`
4. `PHASE_10_COMPLETION_SUMMARY.md`
5. `AYKENOS_PROJE_GENEL_YAPI_VE_MIMARI_RAPORU.md`

## Note
Eski raporlarda gecen blocker veya progress ifadeleri tarihsel baglam icindir. Current status yorumlari icin 2026-03-07 official closure truth ve yukaridaki primary truth kaynaklari kullanilmalidir.
