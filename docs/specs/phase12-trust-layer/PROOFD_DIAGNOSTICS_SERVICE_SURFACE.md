# `proofd` Diagnostics Service Surface

**Version:** 1.0
**Status:** Draft (Phase-13 preparation)
**Date:** 2026-03-10
**Phase:** Phase-13 Observability Layer
**Type:** Non-normative architecture/service boundary note
**Related Spec:** `PARITY_LAYER_ARCHITECTURE.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `tasks.md`

---

## 1. Purpose

This document defines the read-only diagnostics service surface for `proofd`.

`proofd` exposes existing verification and parity diagnostics artifacts through a query API.

`proofd` does not introduce new trust semantics.

Current local status:

- a minimal `userspace/proofd/` skeleton may serve diagnostics artifacts read-only
- run-level diagnostics discovery and run-scoped parity / incidents endpoints may expose multi-run observability without changing parity semantics
- full verification execution, receipt emission, and normative `P12-16` closure behavior remain pending

---

## 2. Architectural Role

`proofd` acts as a verification diagnostics service.

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

`proofd = diagnostics service surface`

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

---

## 5. Proposed Endpoint Set

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

### 5.9 Run-Scoped Parity

`GET /diagnostics/runs/{run_id}/parity`

Returns:

- run-local `parity_report.json`

### 5.10 Graph Surface

`GET /diagnostics/graph`

Returns:

- `parity_incident_graph.json`

### 5.11 Run-Scoped Graph

`GET /diagnostics/runs/{run_id}/graph`

Returns:

- run-local `parity_incident_graph.json`

### 5.12 Authority Drift Topology

`GET /diagnostics/authority-topology`

Returns:

- `parity_authority_drift_topology.json`

### 5.13 Run-Scoped Authority Drift Topology

`GET /diagnostics/runs/{run_id}/authority-topology`

Returns:

- run-local `parity_authority_drift_topology.json`

### 5.14 Authority Drift Suppression

`GET /diagnostics/authority-suppression`

Returns:

- `parity_authority_suppression_report.json`

### 5.15 Run-Scoped Authority Drift Suppression

`GET /diagnostics/runs/{run_id}/authority-suppression`

Returns:

- run-local `parity_authority_suppression_report.json`

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
