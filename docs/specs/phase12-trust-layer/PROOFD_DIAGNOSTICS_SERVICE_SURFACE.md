# `proofd` Diagnostics Service Surface

**Version:** 1.0
**Status:** Draft (local closure-ready sync; Phase-13 preparation)
**Date:** 2026-03-11
**Phase:** Kernel Phase 12 / Phase-13 preparation
**Type:** Non-normative architecture/service boundary note
**Related Spec:** `PARITY_LAYER_ARCHITECTURE.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `PROOFD_SERVICE_CLOSURE_PLAN.md`, `PROOFD_SERVICE_FINAL_HARDENING_CHECKLIST.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PHASE13_NEGATIVE_TEST_SPEC.md`, `tasks.md`

---

## 1. Purpose

This document defines the read-only diagnostics service surface for `proofd`.

`proofd` exposes existing verification and parity diagnostics artifacts through a query API.

`proofd` does not introduce new trust semantics.

Current local status:

- a minimal `userspace/proofd/` skeleton may serve diagnostics artifacts read-only
- a local `ci-gate-proofd-service` execution slice may validate root and run-scoped diagnostics passthrough without changing parity semantics
- a local `ci-gate-proofd-observability-boundary` execution slice may validate that `/diagnostics/*` remains read-only, query-safe, and non-authoritative
- a local `POST /verify/bundle` execution family may delegate to verifier-core with explicit `bundle_path`, `policy_path`, `registry_path`, `receipt_mode`, `receipt_signer`, optional `diversity_binding`, optional Stage-2 `replay_boundary_binding`, optional fallback `trust_reuse_binding`, and `run_id` binding while keeping diagnostics endpoints read-only
- when `diversity_binding` is present, `proofd` may now emit `replay_boundary_flow_source.json` from the bundle's own replay runtime surface instead of requiring a request-supplied replay event
- `trust_reuse_flow_source.json` now prefers bundle-native `reports/trust_reuse_runtime_surface.json` evidence, emits `NO_REUSABLE_EVENTS` when native trust-reuse was evaluated but yielded no reusable path, and only falls back to explicit `trust_reuse_binding` when the native runtime surface is absent
- the preferred native trust-reuse runtime surface may now be materialized by the `proof-verifier` `trust-reuse-runtime-evaluator`, which binds signed receipt, verification context, verifier attestation, and verifier trust registry artifacts before `proofd` translates them into Stage-2 companion source evidence
- the native target contract for that future trust-reuse emitter is `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`
- local execution reuse by `run_id` may only occur for an identical canonical request fingerprint; differing requests under the same `run_id` MUST fail closed
- run-level diagnostics discovery, run summary, and run-scoped parity / incidents / drift / convergence / graph / authority endpoints may expose multi-run observability without changing parity semantics
- local `P12-16` closure-ready evidence now proves repeated signed-receipt determinism, request-bound timestamp preservation, run-manifest stability, and diagnostics purity in `run-local-phase12c-closure-2026-03-11`

---

## 2. Architectural Role

`proofd` acts as a verification execution service with a read-only diagnostics surface.

Its diagnostics surface remains read-only even when a local verification execution family exists.

It exposes:

- verification results
- parity artifacts
- determinism incidents
- convergence diagnostics

It does not:

- evaluate cluster truth
- enforce authority
- resolve consensus

Formally:

`proofd = verification execution service + diagnostics service surface`

and:

`proofd != authority surface`

---

## 3. Service Model

`proofd` serves existing artifact surfaces produced by verification and parity analysis.

Examples:

- `parity_report.json`
- `parity_consistency_report.json`
- `parity_determinism_report.json`
- `parity_determinism_incidents.json`
- `parity_authority_suppression_report.json`
- `parity_authority_drift_topology.json`
- `parity_incident_graph.json`
- `parity_convergence_report.json`
- `parity_drift_attribution_report.json`
- `failure_matrix.json`

The service layer MUST NOT reinterpret or transform these artifacts into new trust semantics.

Diagnostics purity rule:

`proofd` MUST serve artifacts as produced.

`proofd` MUST NOT:

- merge incidents across runs
- reinterpret incident identity
- synthesize derived incident classes

---

## 4. Canonical Object Exposure

The service exposes canonical diagnostics objects:

- `NodeParityOutcome`
- `DeterminismIncident`
- `DeterminismOutcomePartition`
- drift-attribution partitions
- convergence partitions

`proofd` MUST NOT redefine these objects.

`proofd` MAY provide:

- filtering
- pagination
- aggregation
- projection

over canonical artifact data.

`proofd` MUST NOT convert those aggregations into verifier reputation, correctness, reliability, or weighted-authority metrics.

---

## 5. Proposed Endpoint Set

The diagnostics surface below remains read-only. A local execution family such as `POST /verify/bundle` belongs to the closure plan and MUST NOT change the semantics of any `GET /diagnostics/*` endpoint.

### 5.1 Incidents

`GET /diagnostics/incidents`

Returns:

- `DeterminismIncidentReport`

Optional filters:

- `severity`
- `surface_key`
- `node_id`

### 5.2 Single Incident

`GET /diagnostics/incidents/{incident_id}`

Returns:

- `DeterminismIncident`

### 5.3 Parity Report

`GET /diagnostics/parity`

Returns:

- `parity_report.json`

### 5.4 Drift Attribution

`GET /diagnostics/drift`

Returns:

- `parity_drift_attribution_report.json`

### 5.5 Convergence Diagnostics

`GET /diagnostics/convergence`

Returns:

- `parity_convergence_report.json`

### 5.6 Raw Failure Matrix

`GET /diagnostics/failure-matrix`

Returns:

- `failure_matrix.json`

### 5.7 Run Discovery

`GET /diagnostics/runs`

Returns:

- run identifiers discoverable under the configured evidence root
- run-level artifact availability for known diagnostics files

### 5.8 Run-Scoped Incidents

`GET /diagnostics/runs/{run_id}/incidents`

Returns:

- run-local `parity_determinism_incidents.json`

### 5.9 Run Summary

`GET /diagnostics/runs/{run_id}`

Returns:

- run identifier
- run-local known artifact list

### 5.10 Run-Scoped Parity

`GET /diagnostics/runs/{run_id}/parity`

Returns:

- run-local `parity_report.json`

### 5.11 Run-Scoped Drift Attribution

`GET /diagnostics/runs/{run_id}/drift`

Returns:

- run-local `parity_drift_attribution_report.json`

### 5.12 Run-Scoped Convergence

`GET /diagnostics/runs/{run_id}/convergence`

Returns:

- run-local `parity_convergence_report.json`

### 5.13 Run-Scoped Failure Matrix

`GET /diagnostics/runs/{run_id}/failure-matrix`

Returns:

- run-local `failure_matrix.json`

### 5.14 Graph Surface

`GET /diagnostics/graph`

Returns:

- `parity_incident_graph.json`

### 5.15 Run-Scoped Graph

`GET /diagnostics/runs/{run_id}/graph`

Returns:

- run-local `parity_incident_graph.json`

### 5.16 Authority Drift Topology

`GET /diagnostics/authority-topology`

Returns:

- `parity_authority_drift_topology.json`

### 5.17 Authority Suppression

`GET /diagnostics/authority-suppression`

Returns:

- `parity_authority_suppression_report.json`

### 5.18 Run-Scoped Authority Drift Topology

`GET /diagnostics/runs/{run_id}/authority-topology`

Returns:

- run-local `parity_authority_drift_topology.json`

### 5.19 Run-Scoped Authority Suppression

`GET /diagnostics/runs/{run_id}/authority-suppression`

Returns:

- run-local `parity_authority_suppression_report.json`

### 5.20 Verification Execute

`POST /verify/bundle`

Returns:

- verifier-core-derived verdict response
- optional signed receipt emission metadata
- run-scoped artifact updates limited to the requested verification run

This endpoint is part of the service contract but not part of the read-only diagnostics family.

---

## 6. Response Contract

All responses must preserve artifact structure.

Example:

```json
{
  "node_count": 13,
  "determinism_incident_count": 1,
  "severity_counts": {
    "pure_determinism_failure": 1
  },
  "incidents": []
}
```

No new fields implying trust semantics may be introduced.

---

## 7. Non-Goals

The `proofd` diagnostics surface MUST NOT:

- select canonical truth
- compute cluster consensus
- resolve majority outcomes
- enforce policy decisions
- rewrite parity artifacts
- redefine canonical verification objects
- compute verifier reputation
- expose historical correctness scores
- rank nodes by trust or reliability
- emit actionable control signals such as `recommended_action`, `routing_hint`, or `execution_override`

If a service performs these functions, it is no longer `proofd`.

---

## 8. Severity Handling

`DeterminismIncidentSeverity` is derived diagnostics metadata.

`proofd` MUST NOT:

- recompute severity
- override severity
- reinterpret severity as policy
- recompute authority suppression decisions
- reinterpret suppression rules as authority arbitration

Severity values are produced by parity analysis.

The service only exposes them.

---

## 9. Graph Surfaces

Current local implementations MAY expose:

- `GET /diagnostics/graph`
- `GET /diagnostics/runs/{run_id}/graph`

Graph objects represent:

- node topology
- parity edges
- incident surfaces
- authority drift clusters

However:

`graph = observability topology`

and:

`graph != consensus topology`

---

## 10. Governance Guardrail

The repository architecture rule remains:

`Parity Layer = Distributed Verification Diagnostics`

`Parity Layer != consensus`

`proofd != authority surface`

`proofd` must preserve this boundary.

The negative-test contract for preserving this boundary is defined in:

- `PHASE13_NEGATIVE_TEST_SPEC.md`
