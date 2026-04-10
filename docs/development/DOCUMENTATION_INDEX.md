# AykenOS Documentation Index
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-04-10
**Snapshot Basis:** `local-freeze-p10p11` + `local-phase11-closure` + `run-run-local-phase12c-closure-2026-03-11` + `run-local-p13-kill-switch-20260315T000051Z` + `phase15-official-closure`

## Current Status
- **Runtime:** `Phase-10` officially closed — `ci-freeze` run `22797401328`
- **Verification Substrate:** `Phase-11` officially closed — `ci-freeze` run `22797401328`
- **Trust Layer:** `Phase-12` officially closed — `ci-freeze` run `23099070483` (PR #62)
- **Distributed Observability:** `Phase-13` officially closed — `ci-freeze` run `23706742211` (PR #81)
- **Observability Hardening:** `Phase-14` officially closed — all 5 workstreams merged
- **BCIB Execution Engine v3:** `Phase-15` officially closed — `ci-freeze` run `24213727039` (PR #104)
- **Formal Governance Pointer:** `CURRENT_PHASE=15` (formal transition at `48970cd0`)
- **Active Phase:** Phase-16 pending — Ayken CLI Faz B + BCIB toolchain surface
- **Performance Baseline:** `gha-ubuntu24-20260406.80.1-X64` (PR #104)

## Primary Truth Sources
Current repo truth icin once su dosyalari referans alin:

1. `README.md`
2. `docs/roadmap/CURRENT_PHASE` — `CURRENT_PHASE=15`
3. `docs/roadmap/overview.md`
4. `docs/development/PROJECT_STATUS_REPORT.md`
5. `reports/phase15_official_closure/PHASE15_CLOSURE_REPORT.md`
6. `reports/phase15_official_closure/closure_index.json`
7. `reports/phase13_official_closure_candidate/closure_index.json`
8. `reports/phase12_official_closure_candidate/closure_manifest.json`
9. `reports/phase10_phase11_official_closure_index.json`
10. `Makefile`
11. `.github/workflows/ci-freeze.yml`

Phase-14 workstream numbering ve aktif durum yorumu icin canonical truth source:

1. `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`

README/spec dili ve architecture map bu tracker ile hizali okunmalidir.

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
3. `docs/roadmap/CURRENT_PHASE` — `CURRENT_PHASE=15`
4. `docs/development/PROJECT_STATUS_REPORT.md`
5. `docs/specs/phase16-ayken-orchestration/README.md`
6. `docs/specs/authority-lineage-v1/README.md`
7. `docs/specs/phase14-distributed-observability/README.md`
8. `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
9. `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`
10. `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
11. `docs/specs/phase14-distributed-observability/OBSERVABILITY_UX_CONTRACT_v1.md`

## Phase-14 Reference Set
### Architecture and Observability Surfaces
1. `docs/specs/phase14-distributed-observability/README.md`
2. `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
3. `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`
4. `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
5. `docs/specs/phase14-distributed-observability/OBSERVABILITY_UX_CONTRACT_v1.md`
6. `docs/specs/phase14-distributed-observability/PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md`

### obs-cli Consumer (Workstream 3.5 CLI Layer)
1. `.kiro/specs/obs-cli-consumer/requirements.md`
2. `.kiro/specs/obs-cli-consumer/design.md`
3. `.kiro/specs/obs-cli-consumer/tasks.md`
4. `userspace/obs-cli/` — implementation crate (all modules complete)

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
Eski raporlarda gecen blocker veya progress ifadeleri tarihsel baglam icindir. Current status yorumlari icin Phase-15 official closure truth (`reports/phase15_official_closure/closure_index.json`) ve yukaridaki primary truth kaynaklari birlikte kullanilmalidir.

**Son Guncelleme:** 2026-04-10
