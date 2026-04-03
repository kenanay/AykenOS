# Phase-14 Development Tracker

**Phase:** 14  
**Status:** ACTIVE  
**Tracker State:** LIVE  
**Last Updated:** 2026-04-03  
**Authority:** `ARCHITECTURE_FREEZE.md`  
**Formal Pointer:** `docs/roadmap/CURRENT_PHASE` = `CURRENT_PHASE=14`  
**Primary Spec:** `docs/specs/phase14-distributed-observability/README.md`  
**Architecture Map:** `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
**Canonical Workstream Truth:** This tracker is the authoritative Phase-14 workstream numbering and status surface.

---

## 1. Purpose

This file records the active development flow of Phase-14.

For Phase-14 workstream IDs and state labels, this tracker is authoritative. The spec README, architecture map, and related status surfaces MUST align to this file.

It exists to answer four questions at any given time:

1. Which Phase-14 workstream is active?
2. What is already implemented?
3. What is validated versus only locally in progress?
4. What is the next concrete step?

This tracker is operational, not historical. Historical closure truth remains in closure artifacts and roadmap truth surfaces.

---

## 2. Current Snapshot

- `Phase-13`: OFFICIALLY CLOSED
- `Phase-14`: ACTIVE
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
| 3.1 | Read-Only External API Stabilization | IN PROGRESS | Phase-14 API contract work started; `/diagnostics/version` and `X-Ayken-API-Version` slice is the current implementation target | Commit and merge the first stable versioned API slice, then document the external contract |
| 3.2 | Replay Determinism Stability Hardening | IN PROGRESS | Canonical request fingerprint, determinism contract artifact, internal replay comparison, incident model, dedicated `determinism/` + `internal/` module boundaries, and a producer-driven `ci-gate-determinism-replay-consistency` are implemented and validated locally | Commit and merge the slice, then obtain clean-worktree pre-CI and remote `ci-freeze` confirmation |
| 3.3 | `proofd` Query/Service Boundary Hardening | TODO | Architecture defined, implementation order fixed after 3.1 | Extend query validation and forbidden-field enforcement after API surface stabilization |
| 3.4 | Cross-Node Observability Graph | TODO | Architectural target exists; current graph surface remains Phase-13-derived | Define Phase-14 graph contract and artifact shape |
| 3.5 | Observability UX (Human-Readable Layer) | TODO | No implementation started | Define read-only summary surface without introducing scoring or authority semantics |

### Status Legend

- `TODO`: not started
- `IN PROGRESS`: active implementation or local worktree activity exists
- `VALIDATED_LOCAL`: implementation completed and validated locally
- `MERGED`: merged to main
- `CI_CONFIRMED`: remote CI confirmation obtained
- `BLOCKED`: cannot proceed without prerequisite or governance clarification

---

## 4. Activity Log

### 2026-04-03

**Phase-14 opened and architecture anchored**
- `CURRENT_PHASE=14` formal transition already executed in repo truth surfaces
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

---

## 6. Evidence and Reference Links

- Formal phase pointer: `docs/roadmap/CURRENT_PHASE`
- Phase-14 spec: `docs/specs/phase14-distributed-observability/README.md`
- Phase-14 architecture map: `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
- Phase-13 closure confirmation: `reports/phase13_official_closure_candidate/closure_index.json`
- Active implementation surface:
  - `userspace/proofd/src/lib.rs`
  - `userspace/proofd/src/main.rs`

---

## 7. Open Items

1. Should Phase-14 API versioning remain header-only plus discovery endpoint, or also define an explicit path/version namespace?
2. Which `/diagnostics/*` surfaces are officially external-facing in v1, and which remain internal or experimental?
3. Do we want a dedicated Phase-14 contract document for response schemas, separate from the architecture map?
4. What is the exact evidence bundle required to mark Workstream 3.1 as `CI_CONFIRMED`?

---

## 8. Next Steps

1. Commit and merge the Phase-14 tracker/doc sync plus determinism gate slice.
2. Re-run `pre_ci_discipline.sh` on a clean worktree so the new determinism gate is exercised in the full local discipline chain.
3. Obtain remote `ci-freeze` confirmation for Workstream 3.2 evidence before considering `CI_CONFIRMED`.
4. Move to Workstream 3.3 hardening once 3.1 and 3.2 are merged and locally disciplined.

---

## 9. Update Rules

When updating this tracker:

1. Distinguish clearly between `local worktree`, `merged`, and `remote CI confirmed`.
2. Do not mark a workstream `MERGED` unless the change is on `main`.
3. Do not mark a workstream `CI_CONFIRMED` without the exact workflow/run reference.
4. If a failure is caused by hygiene or dirty worktree state, record that explicitly instead of implying product failure.
5. Preserve invariant language exactly; do not soften `service != authority`, `diagnostics != decision`, or `parity != consensus`.
