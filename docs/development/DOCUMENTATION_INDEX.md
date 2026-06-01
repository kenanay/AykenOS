# AykenOS Documentation Index
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-06-01
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution Boundary:** Human-readable documentation metadata only; not runtime authority or execution evidence.
**Current Authority Basis:** `phase17-official-closure` + `CURRENT_PHASE=17` + `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`

## Current Status
- **Runtime:** `Phase-10` officially closed — `ci-freeze` run `22797401328`
- **Verification Substrate:** `Phase-11` officially closed — `ci-freeze` run `22797401328`
- **Trust Layer:** `Phase-12` officially closed — `ci-freeze` run `23099070483` (PR #62)
- **Distributed Observability:** `Phase-13` officially closed — `ci-freeze` run `23706742211` (PR #81)
- **Observability Hardening:** `Phase-14` officially closed — all 5 workstreams merged
- **BCIB Execution Engine v3:** `Phase-15` officially closed — `ci-freeze` run `24213727039` (PR #104)
- **Verification Layer MVP:** `Phase-16` officially closed — `ci-freeze` run `25214669681`, tag `phase16-official-closure`
- **Execution Pipeline:** `Phase-17` officially closed — tag `phase17-official-closure` at `416a5392`
- **Formal Governance Pointer:** `CURRENT_PHASE=17`
- **Active Phase:** Phase-17 OFFICIALLY CLOSED / Phase-18 TRANSITION NOT ACTIVATED
- **Active Execution Priority:** Draft the Phase-18 Platform Constitution RFC set, currently `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md` and `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md`
- **Scope Boundary:** Phase-18 is not active until an explicit `CURRENT_PHASE` pointer transition; kernel expansion, new syscalls, Ring0 policy and AI Runtime authority remain forbidden for Phase-18

## Primary Truth Sources
Current repo truth icin once su dosyalari referans alin:

1. `ARCHITECTURE_FREEZE.md`
2. `docs/roadmap/CURRENT_PHASE` — `CURRENT_PHASE=17`
3. `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md` — **active execution roadmap**
4. `AYKENOS_GUNCEL_DURUM_RAPORU_2026_05_23.md` — **current status report**
5. `README.md`
6. `docs/roadmap/freeze-enforcement-workflow.md`
7. `shared/abi/ayken_abi.h` and `shared/abi/syscall_v2.h` — **canonical frozen ABI inputs**
8. `reports/phase17_official_closure_candidate/closure_decision_record.json`
9. `reports/phase17_official_closure_candidate/closure_manifest.json`
10. `reports/phase17_official_closure_candidate/closure_index.json`
11. `PHASE18_TRANSITION_DECISION.md` — **Phase-18 Platform Constitution transition package; not active pointer**
12. `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md` — **first pre-activation Platform Constitution RFC draft**
13. `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md` — **capability request/decision/receipt/revocation RFC draft**
14. `reports/phase15_official_closure/PHASE15_CLOSURE_REPORT.md`
15. `reports/phase15_official_closure/closure_index.json`
16. `reports/phase13_official_closure_candidate/closure_index.json`
17. `reports/phase12_official_closure_candidate/closure_manifest.json`
18. `reports/phase10_phase11_official_closure_index.json`
19. `userspace/minimal/minimal_bcib_first_retire_probe.S` — **historical Ring3 breakthrough evidence**
20. `Makefile`
21. `.github/workflows/ci-freeze.yml`

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
2. `docs/roadmap/CURRENT_PHASE` — `CURRENT_PHASE=17`
3. `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`
4. `docs/roadmap/freeze-enforcement-workflow.md`
5. `AYKENOS_GUNCEL_DURUM_RAPORU_2026_05_23.md`
6. `PHASE18_TRANSITION_DECISION.md` — transition package only
7. `docs/specs/phase18-platform-constitution/README.md` — Phase-18 spec set index
8. `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md`
9. `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md`
10. `PHASE18_ROADMAP.md` — historical pre-closure runtime-validation roadmap
11. `docs/roadmap/overview.md` — historical 2026-04-24 snapshot only
12. `docs/specs/phase16-ayken-orchestration/README.md`
13. `docs/specs/authority-lineage-v1/README.md`
14. `docs/specs/phase14-distributed-observability/README.md`
15. `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
16. `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`

## Phase-18 Reference Set
1. `PHASE18_TRANSITION_DECISION.md`
2. `docs/specs/phase18-platform-constitution/README.md`
3. `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md`
4. `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md`

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
6. `AYKENOS_SON_DURUM_RAPORU_2026_04_24.md`
7. `docs/roadmap/overview.md`
8. `docs/roadmap/ROADMAP_2026_02_23.md`

## Note
Eski raporlarda gecen blocker veya progress ifadeleri tarihsel baglam icindir.
Current status yorumlari icin Phase-17 official closure otoritesi,
`docs/roadmap/CURRENT_PHASE` ve aktif stabilization roadmap birlikte
kullanilmalidir. Yeni ozellik veya Phase-18 aktivasyonu,
`PHASE18_TRANSITION_DECISION.md` review edilip explicit `CURRENT_PHASE`
transition yapilmadan current execution plani olarak sunulamaz.

**Son Guncelleme:** 2026-05-31
