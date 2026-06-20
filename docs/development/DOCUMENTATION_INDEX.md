# AykenOS Documentation Index
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-06-20
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution Boundary:** Human-readable documentation metadata only; not runtime authority or execution evidence.
**Current Authority Basis:** `phase17-official-closure` + `CURRENT_PHASE=19` + `PHASE19_POINTER_TRANSITION_DECISION.md` + `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`

## Current Status
- **Runtime:** `Phase-10` officially closed — `ci-freeze` run `22797401328`
- **Verification Substrate:** `Phase-11` officially closed — `ci-freeze` run `22797401328`
- **Trust Layer:** `Phase-12` officially closed — `ci-freeze` run `23099070483` (PR #62)
- **Distributed Observability:** `Phase-13` officially closed — `ci-freeze` run `23706742211` (PR #81)
- **Observability Hardening:** `Phase-14` officially closed — all 5 workstreams merged
- **BCIB Execution Engine v3:** `Phase-15` officially closed — `ci-freeze` run `24213727039` (PR #104)
- **Verification Layer MVP:** `Phase-16` officially closed — `ci-freeze` run `25214669681`, tag `phase16-official-closure`
- **Execution Pipeline:** `Phase-17` officially closed — tag `phase17-official-closure` at `416a5392`
- **Formal Governance Pointer:** `CURRENT_PHASE=19`
- **Active Phase:** Phase-19 ACTIVE / Platform Runtime MVP planning, admission, and receipt boundary only
- **Active Execution Priority:** Maintain Phase-19 planning/admission/receipt authority after PR #181 bounded merge and main exact-SHA post-merge PASS, without runtime activation or authority expansion
- **Accepted Decision Package Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md`; bounded implementation acceptance is exact-SHA scoped and merge remains separate
- **Current Evidence Package Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md`; records historical draft PR #181 subject `22d5e86a` evidence
- **Current Acceptance Review Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md`; acceptance is not granted and PR #181 remains draft
- **Current Additional Transcript Evidence Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_ADDITIONAL_TRANSCRIPT_EVIDENCE.md`; binds missing denial transcript evidence without granting acceptance
- **Current Acceptance Review Update Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md`; accepts additional transcript evidence as input but requires a new implementation subject before acceptance
- **Current Reason-Class Update Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_REASON_CLASS_UPDATE.md`; records updated implementation subject `64fa4762` without granting acceptance
- **Current Evidence Re-Bind Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md`; re-binds evidence for updated subject `64fa4762` without granting acceptance by itself
- **Current Final Acceptance Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md`; grants bounded implementation acceptance for subject `64fa4762` without merge authority or runtime activation
- **Current Merge Review Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_REVIEW.md`; reviews PR #181 merge-decision readiness without granting merge authority
- **Current Merge Decision Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION.md`; conditionally authorizes bounded PR #181 merge only after decision-record remote PASS and recorded maintainer action, without runtime activation or Phase-19 closure
- **Current Review Findings Update Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_REVIEW_FINDINGS_UPDATE.md`; records updated implementation subject `0a067dba` and invalidates inherited merge authority
- **Current Review Findings Evidence Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_REVIEW_FINDINGS_EVIDENCE_REBIND.md`; re-binds exact-SHA evidence without granting acceptance by itself
- **Current Post-Review Acceptance Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_POST_REVIEW.md`; grants bounded acceptance for `0a067dba` without merge authority
- **Current Merge Decision Update Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION_UPDATE.md`; supersedes the prior merge decision for the updated subject and remains conditional on remote PASS, resolved threads, and maintainer action
- **Current Main Exact-SHA Evidence Boundary:** `PHASE19_RUNTIME_IMPLEMENTATION_MAIN_EXACT_SHA_EVIDENCE_SYNC.md`; records PR #181 merged at main SHA `ed7e2798` with post-merge strict freeze and full Dev Loop PASS, without runtime activation or Phase-19 closure
- **Scope Boundary:** `CURRENT_PHASE=19` does not authorize general runtime behavior; kernel expansion, new syscalls, Ring0 policy and AI Runtime authority remain forbidden

## Primary Truth Sources
Current repo truth icin once su dosyalari referans alin:

1. `ARCHITECTURE_FREEZE.md`
2. `docs/roadmap/CURRENT_PHASE` — `CURRENT_PHASE=19`
3. `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md` — **active execution roadmap**
4. `AYKENOS_GUNCEL_DURUM_RAPORU_2026_05_23.md` — **current status report**
5. `README.md`
6. `docs/roadmap/freeze-enforcement-workflow.md`
7. `shared/abi/ayken_abi.h` and `shared/abi/syscall_v2.h` — **canonical frozen ABI inputs**
8. `reports/phase17_official_closure_candidate/closure_decision_record.json`
9. `reports/phase17_official_closure_candidate/closure_manifest.json`
10. `reports/phase17_official_closure_candidate/closure_index.json`
11. `PHASE18_TRANSITION_DECISION.md` — **accepted Phase-18 Platform Constitution transition package**
12. `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md` — **active Platform Constitution module manifest RFC**
13. `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md` — **active capability request/decision/receipt/revocation RFC**
14. `docs/specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md` — **active workspace admission/logical-mount lifecycle RFC**
15. `docs/specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md` — **active package metadata evidence-only RFC**
16. `docs/specs/phase18-platform-constitution/TRUST_CLASSIFICATION_MODEL.md` — **active trust classification policy-input RFC**
17. `docs/specs/phase18-platform-constitution/PLUGIN_BOUNDARY_CONTRACT.md` — **active plugin boundary compatibility RFC**
18. `docs/specs/phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md` — **active Platform ABI validation order/receipt RFC**
19. `docs/specs/phase18-platform-constitution/CROSS_CONSISTENCY_REVIEW.md` — **Phase-18 RFC set cross-consistency review; not activation**
20. `PHASE18_ACTIVATION_DECISION.md` — **accepted Phase-18 activation decision package; runtime not authorized**
21. `docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md` — **active Phase-18 authority drift review guard; runtime not authorized**
22. `docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md` — **accepted Phase-18 terminology audit; runtime not authorized**
23. `PHASE19_RUNTIME_DECISION.md` — **Phase-19 Runtime MVP decision package; implementation not authorized**
24. `docs/specs/phase19-platform-runtime/README.md` — **Phase-19 Runtime MVP active planning/admission/receipt RFC set; runtime not authorized**
25. `docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md` — **Phase-19 Runtime evidence matrix; evidence PASS and runtime not authorized**
26. `docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md` — **Phase-19 Runtime RFC set cross-consistency review; runtime not authorized**
27. `PHASE19_POINTER_TRANSITION_CANDIDATE.md` — **Phase-19 pointer transition candidate; implementation not authorized**
28. `PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` — **Phase-19 activation preconditions review; implementation not authorized**
29. `PHASE19_POINTER_TRANSITION_DECISION.md` — **Phase-19 pointer transition decision; `CURRENT_PHASE=19`, runtime implementation not authorized**
30. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md` — **Phase-19 implementation decision candidate; runtime source code not authorized**
31. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md` — **Phase-19 implementation decision package candidate; runtime source code not authorized**
32. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` — **Phase-19 implementation decision package draft; runtime source code not authorized**
33. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md` — **Phase-19 implementation decision package boundary; runtime source code not authorized**
34. `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md` — **Phase-19 implementation evidence package; not acceptance**
35. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md` — **Phase-19 implementation acceptance review; acceptance not granted**
36. `PHASE19_RUNTIME_IMPLEMENTATION_ADDITIONAL_TRANSCRIPT_EVIDENCE.md` — **Phase-19 additional transcript evidence; acceptance not granted**
37. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md` — **Phase-19 acceptance review update; new implementation subject required**
38. `PHASE19_RUNTIME_IMPLEMENTATION_REASON_CLASS_UPDATE.md` — **Phase-19 reason-class implementation update; final acceptance review separate**
39. `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md` — **Phase-19 implementation evidence re-bind; not acceptance by itself**
40. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md` — **Phase-19 final bounded acceptance review; merge not authorized**
41. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_REVIEW.md` — **Phase-19 merge review; merge decision not granted**
42. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION.md` — **Phase-19 conditional bounded merge decision; PR not merged and runtime not activated**
43. `PHASE19_RUNTIME_IMPLEMENTATION_REVIEW_FINDINGS_UPDATE.md` — **Phase-19 review findings implementation update; new subject requires re-bind**
44. `PHASE19_RUNTIME_IMPLEMENTATION_REVIEW_FINDINGS_EVIDENCE_REBIND.md` — **Phase-19 review findings evidence re-bind; not acceptance by itself**
45. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_POST_REVIEW.md` — **Phase-19 post-review bounded acceptance; merge separate**
46. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION_UPDATE.md` — **Phase-19 updated conditional merge decision; PR not merged**
47. `PHASE19_RUNTIME_IMPLEMENTATION_MAIN_EXACT_SHA_EVIDENCE_SYNC.md` — **Phase-19 PR #181 merged main exact-SHA evidence sync; runtime not activated**
48. `reports/phase15_official_closure/PHASE15_CLOSURE_REPORT.md`
49. `reports/phase15_official_closure/closure_index.json`
50. `reports/phase13_official_closure_candidate/closure_index.json`
51. `reports/phase12_official_closure_candidate/closure_manifest.json`
52. `reports/phase10_phase11_official_closure_index.json`
53. `userspace/minimal/minimal_bcib_first_retire_probe.S` — **historical Ring3 breakthrough evidence**
54. `Makefile`
55. `.github/workflows/ci-freeze.yml`

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
2. `docs/roadmap/CURRENT_PHASE` — `CURRENT_PHASE=19`
3. `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`
4. `docs/roadmap/freeze-enforcement-workflow.md`
5. `AYKENOS_GUNCEL_DURUM_RAPORU_2026_05_23.md`
6. `PHASE18_TRANSITION_DECISION.md` — transition package only
7. `docs/specs/phase18-platform-constitution/README.md` — Phase-18 spec set index
8. `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md`
9. `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md`
10. `docs/specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md`
11. `docs/specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md`
12. `docs/specs/phase18-platform-constitution/TRUST_CLASSIFICATION_MODEL.md`
13. `docs/specs/phase18-platform-constitution/PLUGIN_BOUNDARY_CONTRACT.md`
14. `docs/specs/phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md`
15. `docs/specs/phase18-platform-constitution/CROSS_CONSISTENCY_REVIEW.md`
16. `PHASE18_ACTIVATION_DECISION.md` — accepted activation decision package
17. `docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md` — active authority drift review guard
18. `docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md` — accepted terminology audit
19. `PHASE19_RUNTIME_DECISION.md` — Phase-19 Runtime MVP decision package; not implementation authority
20. `docs/specs/phase19-platform-runtime/README.md` — Phase-19 Runtime MVP active planning/admission/receipt RFC set; not implementation authority
21. `docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md` — Phase-19 Runtime evidence matrix; not evidence PASS or implementation authority
22. `docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md` — Phase-19 Runtime RFC set cross-consistency review; not active implementation authority
23. `PHASE19_POINTER_TRANSITION_CANDIDATE.md` — Phase-19 pointer transition candidate; not implementation authority
24. `PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` — Phase-19 activation preconditions review; not implementation authority
25. `PHASE19_POINTER_TRANSITION_DECISION.md` — Phase-19 pointer transition decision; not implementation authority
26. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md` — Phase-19 implementation decision candidate; not implementation authority
27. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md` — Phase-19 implementation decision package candidate; not implementation authority
28. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` — Phase-19 implementation decision package draft; not implementation authority
29. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md` — Phase-19 implementation decision package boundary; not implementation authority
30. `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md` — Phase-19 implementation evidence package; not acceptance review or merge authority
31. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md` — Phase-19 implementation acceptance review; acceptance not granted and PR #181 remains draft
32. `PHASE19_RUNTIME_IMPLEMENTATION_ADDITIONAL_TRANSCRIPT_EVIDENCE.md` — Phase-19 implementation additional transcript evidence; not acceptance review update or merge authority
33. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md` — Phase-19 acceptance review update; acceptance not granted and new implementation subject required
34. `PHASE19_RUNTIME_IMPLEMENTATION_REASON_CLASS_UPDATE.md` — Phase-19 reason-class implementation update; not evidence package, acceptance review, or merge authority
35. `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md` — Phase-19 implementation evidence re-bind; not acceptance review or merge authority
36. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md` — Phase-19 final bounded acceptance review; not merge authority or runtime activation
37. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_REVIEW.md` — Phase-19 merge review; not merge decision or merge authority
38. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION.md` — Phase-19 conditional bounded merge decision; not merge completion, runtime activation, or Phase-19 closure
39. `PHASE18_ROADMAP.md` — historical pre-closure runtime-validation roadmap
40. `docs/roadmap/overview.md` — historical 2026-04-24 snapshot only
41. `docs/specs/phase16-ayken-orchestration/README.md`
42. `docs/specs/authority-lineage-v1/README.md`
43. `docs/specs/phase14-distributed-observability/README.md`
44. `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
44. `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`

## Phase-18 Reference Set
1. `PHASE18_TRANSITION_DECISION.md`
2. `docs/specs/phase18-platform-constitution/README.md`
3. `docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md`
4. `docs/specs/phase18-platform-constitution/CAPABILITY_CONTRACT_SPECIFICATION.md`
5. `docs/specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md`
6. `docs/specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md`
7. `docs/specs/phase18-platform-constitution/TRUST_CLASSIFICATION_MODEL.md`
8. `docs/specs/phase18-platform-constitution/PLUGIN_BOUNDARY_CONTRACT.md`
9. `docs/specs/phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md`
10. `docs/specs/phase18-platform-constitution/CROSS_CONSISTENCY_REVIEW.md`
11. `PHASE18_ACTIVATION_DECISION.md`
12. `docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`
13. `docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`

## Phase-19 Decision And RFC Set
1. `PHASE19_RUNTIME_DECISION.md` — Platform Runtime MVP decision boundary;
   does not authorize implementation.
2. `docs/specs/phase19-platform-runtime/README.md` — Phase-19 Runtime MVP
   active planning/admission/receipt RFC set index; does not authorize
   implementation.
3. `docs/specs/phase19-platform-runtime/RUNTIME_LIFECYCLE_SPECIFICATION.md`
4. `docs/specs/phase19-platform-runtime/RUNTIME_INPUT_BUNDLE_SPECIFICATION.md`
5. `docs/specs/phase19-platform-runtime/PLATFORM_VALIDATION_INTEGRATION_SPECIFICATION.md`
6. `docs/specs/phase19-platform-runtime/WORKSPACE_ADMISSION_RUNTIME_SPECIFICATION.md`
7. `docs/specs/phase19-platform-runtime/RUNTIME_RECEIPT_SPECIFICATION.md`
8. `docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_PLAN.md`
9. `docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`
10. `docs/specs/phase19-platform-runtime/RUNTIME_NON_GOALS_AND_DENIALS.md`
11. `docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md` —
    accepted cross-consistency review; does not authorize implementation.
12. `PHASE19_POINTER_TRANSITION_CANDIDATE.md` — pointer transition
    precondition candidate; does not authorize implementation.
13. `PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` — activation
    preconditions review; does not authorize implementation.
14. `PHASE19_POINTER_TRANSITION_DECISION.md` — pointer transition decision;
    activates `CURRENT_PHASE=19` only as planning/admission/receipt boundary
    and does not authorize implementation.
15. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md` —
    implementation decision candidate; narrows a later exact-SHA decision and
    does not authorize runtime source code.
16. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md` —
    implementation decision package candidate; narrows the package contents
    for a later exact-SHA decision and does not authorize runtime source code.
17. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` —
    implementation decision package draft; narrows the package draft contents
    for a later exact-SHA decision and does not authorize runtime source code.
18. `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md` —
    implementation decision package boundary; accepts the exact-SHA package
    boundary and does not authorize implementation PR, evidence package,
    acceptance review, or runtime source code.
19. `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md` —
    implementation evidence package; records draft PR #181 subject evidence
    and does not authorize acceptance review, merge, or runtime authority.
20. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md` —
    implementation acceptance review; does not grant acceptance and keeps PR
    #181 draft.
21. `PHASE19_RUNTIME_IMPLEMENTATION_ADDITIONAL_TRANSCRIPT_EVIDENCE.md` —
    additional transcript evidence; binds missing denial transcript surfaces
    and does not grant acceptance, acceptance review update, merge, or runtime
    authority.
22. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md` —
    acceptance review update; accepts additional transcript evidence as input,
    rejects current reason-class granularity for final acceptance, and
    requires a new implementation subject without granting merge or runtime
    authority.
23. `PHASE19_RUNTIME_IMPLEMENTATION_REASON_CLASS_UPDATE.md` —
    reason-class implementation update; records subject `64fa4762` and
    does not authorize acceptance, merge, or runtime authority.
24. `PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md` —
    evidence package re-bind; records updated subject `64fa4762` evidence
    inputs and does not authorize acceptance, merge, or runtime authority.
25. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md` —
    final bounded acceptance review; grants exact-SHA scoped acceptance for
    subject `64fa4762` and does not authorize merge or runtime activation.
26. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_REVIEW.md` —
    merge review; reviews PR #181 merge-decision readiness and does not
    authorize merge, runtime activation, or Phase-19 closure.
27. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION.md` —
    conditional bounded merge decision; requires decision-record remote PASS
    and recorded maintainer action and does not activate runtime or close
    Phase-19.
28. `PHASE19_RUNTIME_IMPLEMENTATION_REVIEW_FINDINGS_UPDATE.md` —
    records bounded subject `0a067dba` and makes the prior merge decision
    inapplicable to the changed source.
29. `PHASE19_RUNTIME_IMPLEMENTATION_REVIEW_FINDINGS_EVIDENCE_REBIND.md` —
    re-binds local and remote evidence for `0a067dba` without acceptance.
30. `PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_POST_REVIEW.md` —
    grants bounded acceptance for `0a067dba` without merge authority.
31. `PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION_UPDATE.md` —
    records the updated conditional merge decision without merge completion,
    runtime activation, or Phase-19 closure.
32. `PHASE19_RUNTIME_IMPLEMENTATION_MAIN_EXACT_SHA_EVIDENCE_SYNC.md` —
    records PR #181 merge commit `ed7e2798` and post-merge exact-SHA remote
    PASS without runtime activation, general runtime authority, or Phase-19
    closure.

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
`docs/roadmap/CURRENT_PHASE`, `PHASE19_POINTER_TRANSITION_DECISION.md` ve
aktif stabilization roadmap birlikte kullanilmalidir. Yeni runtime
implementation, loader, installer, workspace runtime, plugin host, capability
issuer, trust issuer, Semantic CLI authority veya AI Runtime authority ayri
implementation decision ve evidence package olmadan current execution plani
olarak sunulamaz.

**Son Guncelleme:** 2026-06-06
