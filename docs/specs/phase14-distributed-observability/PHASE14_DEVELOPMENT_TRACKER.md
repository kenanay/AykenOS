# Phase-14 Development Tracker

> ⚠️ OBSOLETE — Superseded by Phase-15 official closure. This tracker is retained as a historical workstream record; current authoritative closure truth is `reports/phase15_official_closure/closure_index.json`.

**Phase:** 14  
**Status:** CLOSED (official closure confirmed)  
**Tracker State:** IMMUTABLE  
**Last Updated:** 2026-04-08
**Authority:** `ARCHITECTURE_FREEZE.md`  
**Formal Pointer:** `docs/roadmap/CURRENT_PHASE` = `CURRENT_PHASE=15`  
**Primary Spec:** `docs/specs/phase14-distributed-observability/README.md`  
**Architecture Map:** `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
**Canonical Workstream Truth:** This tracker is the authoritative Phase-14 workstream numbering and status surface.

---

## 1. Purpose

This file records the completed development flow of Phase-14.

For Phase-14 workstream IDs and state labels, this tracker is authoritative. The spec README, architecture map, and related status surfaces MUST align to this file.

It exists to answer four historical questions:

1. Which Phase-14 workstream is active?
2. What is already implemented?
3. What is validated versus only locally in progress?
4. What is the next concrete step?

This tracker is operational, not historical. Historical closure truth remains in closure artifacts and roadmap truth surfaces.

---

## 2. Current Snapshot

- `Phase-13`: OFFICIALLY CLOSED
- `Phase-14`: CLOSED (official closure confirmed)
- Current theme: distributed observability hardening
- Entry rule: externalization without violating Phase-13 invariants
- Existing substrate note:
  - `ABDF` remains the existing data substrate
  - `BCIB` remains the existing execution substrate
  - `Phase-11` remains the existing reality / proof substrate
  - Phase-14 does not re-center these substrates as primary workstreams; it hardens the `proofd` layer above them
- Core invariants:
  - `service != authority`
  - `diagnostics != decision`
  - `parity != consensus`
  - `trust does not affect verdict`
  - `observability does not imply scheduling`

---

## 3. Workstream Board

| ID | Workstream | Status | Current State | Next Concrete Step |
|---|---|---|---|---|
| 3.1 | Read-Only External API Stabilization | MERGED | Versioned diagnostics discovery/header surface and canonical external contract doc are on `main` | Keep contract and tracker surfaces aligned as 3.3 hardening expands |
| 3.2 | Replay Determinism Stability Hardening | MERGED | Canonical request fingerprint, determinism contract artifacts, internal replay route, and replay consistency gate are on `main` | Preserve replay determinism boundary while 3.3 hardening continues |
| 3.3 | `proofd` Query/Service Boundary Hardening | MERGED | Canonical diagnostics contract registry, runtime forbidden-field enforcement, response schema contract, explicit schema coverage governance, and unified contract-driven public diagnostics dispatch are on `main` | Preserve the boundary contract while 3.4 graph and UX work expand adjacent read-only surfaces |
| 3.4 | Cross-Node Observability Graph | MERGED | Run-scoped graph artifacts carry the Phase-14 envelope on `main`; root `/diagnostics/graph` returns partitioned derived graphs and `/diagnostics/graph/overlay` returns overlay-only diagnostics on `main` | Preserve the non-authoritative boundary while observing post-merge partition and overlay stability |
| 3.5 | Observability UX (Human-Readable Layer) | MERGED | `GET /diagnostics/summary` (human_readable + machine_structured) and `GET /diagnostics/runs/{run_id}/summary` bound to UX contract on `main`; schema validation, forbidden-field enforcement, epistemic boundary, and `obs-cli` consumer crate all complete | Observe post-merge stability; no further 3.5 implementation required |

### Status Legend

- `TODO`: not started
- `IN PROGRESS`: active implementation or local worktree activity exists
- `VALIDATED_LOCAL`: implementation completed and validated locally
- `MERGED`: merged to main
- `CI_CONFIRMED`: remote CI confirmation obtained
- `BLOCKED`: cannot proceed without prerequisite or governance clarification

---

## 4. Activity Log

### 2026-04-07

**Workstream 3.5 validated as complete**
- `GET /diagnostics/summary` is bound to `OBSERVABILITY_UX_CONTRACT_v1.md` on `main`:
  - `human_readable` display mode: `RootDiagnosticsSummaryBody` with `snapshot`, `overlay`, `incidents`, `explanation`
  - `machine_structured` display mode: `MachineSummaryBody` with `counts`, `flags`, `incident_groups`
  - queryless in v1 (unsupported query parameters return `400 unsupported_query_parameter`)
  - forbidden-field enforcement active (`score`, `winner`, etc. → `500 forbidden_observability_field_exposed`)
  - schema validation active (`Full` coverage, fail-closed on missing required fields)
  - epistemic boundary declared: `produces_truth=false`, `produces_decision=false`, `produces_ranking=false`
- `GET /diagnostics/runs/{run_id}/summary` is bound to the run-scoped UX contract on `main`:
  - `RunScopedDiagnosticsSummaryBody` with `run_id`, `snapshot`, `incidents`, `explanation`
  - same epistemic boundary and forbidden-field enforcement
- `obs-cli` consumer crate (`userspace/obs-cli/`) is complete and ready to consume `machine_structured` projection
- Local validation observed:
  - `cargo test -p proofd` passed (`210` lib tests + `6` main tests)
  - `cargo test -p obs-cli` passed (`63` tests)
- All Phase-14 workstreams (3.1–3.5) are now MERGED

### 2026-04-05

**Workstream 3.3 completed and merged to `main`**
- Canonical external diagnostics contract registry added at `userspace/proofd/src/api_contract.rs`
- `/diagnostics/version`, query allowlists, harness expectations, and forbidden-field scan now derive from a single contract surface
- Runtime forbidden-field enforcement added for public diagnostics responses with fail-closed `500 forbidden_observability_field_exposed`
- Service-owned response schema registry added at `userspace/proofd/src/api_schema.rs`
- Service-owned diagnostics responses now fail closed with `500 diagnostics_schema_contract_violation` on missing required fields or required top-level type mismatch
- Explicit schema coverage declarations now exist for every public diagnostics endpoint (`none`, `root_only`, `full`)
- `ci-gate-proofd-schema-coverage` added as a pure validation gate over `proofd-service` evidence
- Public diagnostics routing now resolves through a unified `path -> endpoint_id -> handler -> schema` flow
- External diagnostics contract document updated to record schema coverage and boundary rules
- Local and remote validation observed:
  - `cargo test -p proofd` passed (`171` lib tests + `6` main tests)
  - `ci-gate-observability-routing-separation` passed
  - `ci-gate-proofd-observability-boundary` passed
  - `ci-gate-proofd-service` passed
- `ci-gate-proofd-schema-coverage` passed
  - PR `#94` merged after `ci-freeze` run `23989067554` passed on the final dispatch-hardening slice

**Workstream 3.4 opened with contract-first scope**
- Canonical graph contract drafted at `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
- 3.4 remains explicitly non-authoritative:
  - `graph = derived diagnostics`
  - `graph != authority`
  - `graph != routing hint`
  - `graph != consensus`
- Security hardening added directly at the contract layer:
  - `authority` / `env_hash` boundary required
  - graph provenance required
  - `node_fingerprint` required for stable node binding
  - graph endpoints remain queryless in v1
  - bounded graph generation is required before endpoint promotion
- Current implementation note:
  - the contract froze the field vocabulary, edge model, cluster model, partition semantics, and determinism rules before endpoint implementation

**Workstream 3.4 first implementation slice merged to `main`**
- `parity_incident_graph.json` now carries the Phase-14 run-scoped graph envelope:
  - `graph_version`
  - `authority`
  - `env_hash`
  - `provenance.artifact_set_hash`
  - `provenance.source_runs`
  - `graph.nodes[*].node_fingerprint`
- `GET /diagnostics/runs/{run_id}/graph` now enforces the Phase-14 graph contract at runtime
- `GET /diagnostics/graph` now returns a partitioned derived root surface:
  - `graph_origin = derived`
  - `authority_classification = non_authoritative`
  - `aggregation_mode = overlay_only`
  - `partitions[]` keyed by `graph_version + authority + env_hash + artifact_set_hash`
- `GET /diagnostics/graph/overlay` now returns overlay-only diagnostics:
  - `agreements[]`
  - `conflicts[]`
  - `islands[]`
- root graph aggregation remains descriptive-only:
  - no majority semantics
  - no resolved verdict
  - no routing hint
  - no authority selection
- Local and remote validation observed:
  - `cargo test -p proof-verifier` passed
  - `cargo test -p proofd` passed
  - `ci-gate-proofd-service` passed
  - `ci-gate-proofd-schema-coverage` passed
  - `ci-gate-proofd-observability-boundary` passed
  - `ci-gate-observability-routing-separation` passed
  - `ci-gate-graph-non-authoritative-contract` passed
  - `ci-gate-diagnostics-consumer-non-authoritative-contract` passed
  - PR `#96` merged after `ci-freeze` run `23999026616` passed

**Workstream 3.5 opened with contract-first scope**
- Canonical UX contract drafted at `docs/specs/phase14-distributed-observability/OBSERVABILITY_UX_CONTRACT_v1.md`
- 3.5 remains explicitly non-authoritative:
  - `summary = derived diagnostics`
  - `summary != authority`
  - `summary != decision input`
  - `summary != ranking`
- Explicit epistemic boundary added to the contract target:
  - `summary_origin = derived`
  - `authority_classification = non_authoritative`
  - `display_mode = human_readable`
  - `produces_truth = false`
  - `produces_decision = false`
  - `produces_ranking = false`
- Current implementation note:
  - no runtime summary endpoint is on `main` yet
  - the contract freezes allowed summary content before endpoint implementation

**obs-cli consumer crate fully implemented**
- `userspace/obs-cli/` crate is complete — all modules implemented: `error`, `models`, `parser`, `formatter`, `printer`, `fetcher`, `threshold`, `cli`, `diff`, `main`
- Spec: `.kiro/specs/obs-cli-consumer/` (requirements, design, tasks)
- The crate is a pure read-only CLI consumer for the `machine_structured` projection from `GET /diagnostics/summary`
- Key design properties preserved:
  - no `f32`/`f64` anywhere — all counts are `usize`, all deltas are `i64`
  - no `unwrap()`/`expect()` in non-test code — all errors propagate via `AppError`
  - `BTreeMap<String, usize>` for `incident_groups` — lexicographic key order is a type-level guarantee
  - exit codes: 0=success, 1=usage, 2=I/O, 3=schema/parse, 4=threshold violation
  - `authority_classification == "non_authoritative"` enforced at parse time
  - all three epistemic flags (`produces_truth`, `produces_decision`, `produces_ranking`) must be `false`
- The `obs-cli` crate is the CLI consumer layer for workstream 3.5; it will consume `GET /diagnostics/summary?display_mode=machine_structured` once the runtime endpoint lands

### 2026-04-03

**Phase-14 opened and architecture anchored**
- Historical note: `CURRENT_PHASE=14` was the formal pointer when Phase-14 opened; current repo truth has since advanced to `CURRENT_PHASE=15`
- Phase-14 spec is active
- Architecture map created and merged through PR `#87`
- Reference: `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`

**Workstream 3.1 started**
- Current implementation slice:
  - `GET /diagnostics/version`
  - `X-Ayken-API-Version` response header
  - contract tests for endpoint and header behavior
  - diagnostics contract corrected to exclude execution surface declarations
  - phase label removed from API discovery payload to avoid contract/runtime coupling
- Local validation observed:
  - `cargo test -p proofd` passed
- Local repo discipline note:
  - `pre_ci_discipline.sh` stopped at hygiene because the worktree contained tracked modifications, not because of a contract/test failure

**Workstream 3.2 started**
- Current implementation slice:
  - canonical request fingerprint ignores `run_id` execution metadata
  - `verification_determinism_contract.json` artifact emitted per verify run
  - internal replay comparison surface added at `POST /internal/replay`
  - `verification_determinism_incident.json` model added for replay/hash mismatch
  - cross-run determinism stability and replay regression tests added
  - determinism logic extracted into dedicated `determinism/` and `internal/` modules to keep routing, pure logic, and internal execution surfaces separated
  - test temp-dir allocation hardened to avoid cross-test filesystem collisions during property and boundary runs
  - `proofd --internal-replay --run-dir <run-dir> --verify-request-path <path>` CLI path added for gate/runtime enforcement against exact request artifacts
  - `ci-gate-determinism-replay-consistency` now validates existing `proofd-service` evidence instead of generating its own run
  - determinism replay gate reduced to pure validation + reporting; bootstrap production remains upstream in `ci-gate-proofd-service`
  - `ci-gate-determinism-replay-consistency` added to the `Makefile`, `pre_ci_discipline.sh`, and `ci-freeze` chain
- Local validation observed:
  - `cargo test -p proofd` passed with determinism tests included
  - `cargo fmt -p proofd` passed after module extraction and test isolation cleanup
  - `bash scripts/ci/test_pre_ci_discipline.sh` passed with determinism gate coverage added
  - `make ci-gate-determinism-replay-consistency RUN_ID=local-determinism-gate-pure` passed via upstream `proofd-service` + determinism contract producer chain
  - `bash scripts/ci/gate_determinism_replay_consistency.sh --evidence-dir out/evidence/run-local-determinism-gate-direct/gates/determinism-replay-consistency --source-gate-dir out/evidence/run-local-determinism-gate-pure/gates/proofd-service` passed as a direct pure-validation replay check
- Scope note:
  - internal replay surface is not part of external diagnostics contract
  - full `pre_ci_discipline.sh` still stops at hygiene on this branch because the worktree remains dirty
  - current slice establishes local determinism evidence and local gate coverage; merge and remote CI confirmation remain open

---

## 5. Validation Log

| Date | Scope | Result | Notes |
|---|---|---|---|
| 2026-04-03 | `cargo test -p proofd` | PASS | Phase-14 API slice tests passed locally |
| 2026-04-03 | `cargo test -p proofd` | PASS | Workstream 3.2 determinism slice passed locally (`160` lib tests + `4` main tests) |
| 2026-04-03 | `bash scripts/ci/test_pre_ci_discipline.sh` | PASS | Determinism gate added to fail-closed pre-CI discipline harness |
| 2026-04-03 | `make ci-gate-determinism-replay-consistency RUN_ID=local-determinism-gate-pure` | PASS | Makefile wiring validated through `proofd-service`, verification-contract, and pure replay-consistency gate |
| 2026-04-03 | `bash scripts/ci/gate_determinism_replay_consistency.sh --evidence-dir out/evidence/run-local-determinism-gate-direct/gates/determinism-replay-consistency --source-gate-dir out/evidence/run-local-determinism-gate-pure/gates/proofd-service` | PASS | Direct gate run validated existing `proofd-service` artifacts without bootstrapping a new run |
| 2026-04-03 | `bash scripts/ci/pre_ci_discipline.sh` | FAIL-CLOSED | Stopped at hygiene because tracked files were modified in worktree |
| 2026-04-07 | `cargo test -p proofd` | PASS | Workstream 3.5 validation: `210` lib tests + `6` main tests passed; summary endpoints, schema validation, forbidden-field enforcement all confirmed |
| 2026-04-07 | `cargo test -p obs-cli` | PASS | `obs-cli` consumer crate: `63` tests passed; `machine_structured` projection consumer ready |
| 2026-04-05 | `cargo test -p proofd` | PASS | Final dispatch-hardening slice passed locally (`171` lib tests + `6` main tests) |
| 2026-04-05 | `bash scripts/ci/gate_proofd_service.sh --evidence-dir /tmp/proofd-service-final-dispatch` | PASS | Unified public resolver preserved service contract expectations |
| 2026-04-05 | `bash scripts/ci/gate_proofd_schema_coverage.sh --evidence-dir /tmp/proofd-schema-coverage-final-dispatch --source-gate-dir /tmp/proofd-service-final-dispatch` | PASS | Explicit coverage declarations remained complete after dispatch unification |
| 2026-04-05 | `bash scripts/ci/gate_proofd_observability_boundary.sh --evidence-dir /tmp/proofd-observability-boundary-final-dispatch` | PASS | Runtime forbidden-field enforcement remained intact after route hardening |
| 2026-04-05 | `bash scripts/ci/gate_observability_routing_separation.sh --evidence-dir /tmp/proofd-routing-separation-final-dispatch` | PASS | Contract-driven routing preserved observability/scheduling separation |
| 2026-04-05 | `bash scripts/ci/gate_diagnostics_consumer_non_authoritative_contract.sh --evidence-dir /tmp/diag-consumer-final-dispatch` | PASS | Canonical diagnostics registry remained boundary-only and did not regress into consumer misuse |
| 2026-04-05 | `cargo test -p proof-verifier` | PASS | Run-scoped graph envelope producer and incident graph helpers passed locally |
| 2026-04-05 | `cargo test -p proofd` | PASS | Root partitioned graph and overlay-only diagnostics slice passed locally (`187` lib tests + `6` main tests) |
| 2026-04-05 | `bash scripts/ci/gate_proofd_service.sh --evidence-dir /tmp/proofd-service-root-graph-v2c` | PASS | Root graph and overlay endpoint contract declarations aligned with service harness expectations |
| 2026-04-05 | `bash scripts/ci/gate_proofd_schema_coverage.sh --evidence-dir /tmp/proofd-schema-root-graph-v2c --source-gate-dir /tmp/proofd-service-root-graph-v2c` | PASS | Root graph moved from passthrough to full computed schema coverage without drift |
| 2026-04-05 | `bash scripts/ci/gate_proofd_observability_boundary.sh --evidence-dir /tmp/proofd-observability-root-graph-v2c` | PASS | Root graph and overlay remained read-only and fail-closed under forbidden-field enforcement |
| 2026-04-05 | `bash scripts/ci/gate_observability_routing_separation.sh --evidence-dir /tmp/proofd-routing-root-graph-v2c` | PASS | Root graph promotion did not regress observability/scheduling boundary separation |
| 2026-04-05 | `bash scripts/ci/gate_graph_non_authoritative_contract.sh --evidence-dir /tmp/graph-non-authoritative-root-graph-v2c` | PASS | Partitioned graph and overlay surfaces remained descriptive-only and non-authoritative |
| 2026-04-05 | `bash scripts/ci/gate_diagnostics_consumer_non_authoritative_contract.sh --evidence-dir /tmp/diag-consumer-root-graph-v2c` | PASS | New graph surfaces did not regress into execution-bearing consumer misuse |
| 2026-04-05 | `ci-freeze` run `23999026616` | PASS | PR `#96` remote confirmation for the first root partitioning and overlay-only graph slice |

---

## 6. Evidence and Reference Links

- Formal phase pointer: `docs/roadmap/CURRENT_PHASE`
- Phase-14 spec: `docs/specs/phase14-distributed-observability/README.md`
- Phase-14 architecture map: `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
- Phase-14 graph contract: `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
- Phase-13 closure confirmation: `reports/phase13_official_closure_candidate/closure_index.json`
- Active implementation surface:
  - `userspace/proofd/src/lib.rs`
  - `userspace/proofd/src/main.rs`

---

## 7. Open Items

1. Should root `/diagnostics/graph` remain strictly queryless in v1, or should any bounded filter surface be considered only in a later non-authoritative v2?
2. Should deeper graph-structure validation expand beyond current referential integrity into explicit node-fingerprint uniqueness and partition integrity rules?
3. Should non-GET diagnostics method rejection eventually move from namespace prefix logic into the same endpoint registry layer, or remain a separate boundary check?

---

## 8. Next Steps

1. All Phase-14 workstreams (3.1–3.5) are MERGED. Observe post-merge stability across all surfaces.
2. Decide whether deeper graph-structure validation should expand into node-fingerprint uniqueness and partition-integrity checks.
3. Decide whether non-GET diagnostics method rejection should be folded into endpoint-registry-driven dispatch or remain an explicit namespace boundary.
4. Evaluate Phase-14 closure readiness: local evidence basis + remote `ci-freeze` confirmation path.

---

## 9. Update Rules

When updating this tracker:

1. Distinguish clearly between `local worktree`, `merged`, and `remote CI confirmed`.
2. Do not mark a workstream `MERGED` unless the change is on `main`.
3. Do not mark a workstream `CI_CONFIRMED` without the exact workflow/run reference.
4. If a failure is caused by hygiene or dirty worktree state, record that explicitly instead of implying product failure.
5. Preserve invariant language exactly; do not soften `service != authority`, `diagnostics != decision`, or `parity != consensus`.

---

## 10. Closure Record

| Field | Value |
|-------|-------|
| closure_state | IMMUTABLE |
| closure_date | 2026-04-08 |
| closure_timestamp_utc | 2026-04-07T22:14:38Z |
| head_sha | 97fd3cde83d28bc3d978e22326761fcaa50ab895 |
| ci_freeze_run_id | 24106883657 |
| ci_freeze_result | PASS |
| closure_tag | phase14-official-closure-confirmed |
| next_phase | 15 |
| closure_package | reports/phase14_official_closure_candidate/ |
| closure_index | reports/phase14_official_closure_candidate/closure_index.json |

Phase-14 (Distributed Observability Hardening) resmi olarak KAPALI.
Tüm iş akışları (3.1–3.5) MERGED. Uzak ci-freeze PASS onayı alındı.
Bu tracker artık değiştirilemez (IMMUTABLE). Yeni değişiklikler Phase-15 gerektirir.
