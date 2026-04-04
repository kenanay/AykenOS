# Cross-Node Observability Graph Contract v1

**Phase:** 14  
**Workstream:** 3.4  
**Status:** CONTRACT-FIRST  
**Authority:** `ARCHITECTURE_FREEZE.md`  
**Related Surface:** `GET /diagnostics/graph`, `GET /diagnostics/runs/{run_id}/graph`

---

## 1. Purpose

This document defines the canonical Phase-14 contract target for the cross-node
observability graph surface.

Its job is to make distributed verification drift readable without turning graph
shape into authority, routing, or consensus.

The shortest correct sentence is:

`graph explains divergence; graph does not decide truth`

This contract exists before full 3.4 implementation so endpoint behavior can be
built against one declared truth surface instead of growing route-first.

---

## 2. Current Status Boundary

Current `proofd` graph endpoints still expose Phase-13-derived graph artifacts.

That means this document is:

- canonical for 3.4 field vocabulary and shape intent
- authoritative for 3.4 design direction
- not yet runtime-authoritative until the 3.4 implementation slice lands

This document MUST NOT be read as:

- consensus semantics
- truth-election semantics
- routing semantics
- verifier ranking semantics

---

## 3. Non-Negotiable Invariants

- `graph = derived diagnostics`
- `graph != authority`
- `graph != routing hint`
- `graph != consensus`
- `graph != decision input`
- `graph != replay admission signal`

If any 3.4 implementation violates those rules, it is not Phase-14 observability.

---

## 4. Covered Surfaces

This contract covers:

- `GET /diagnostics/graph`
- `GET /diagnostics/runs/{run_id}/graph`
- the graph payload shape carried by `parity_incident_graph.json`
- cluster/drift vocabulary used to explain multi-node divergence

This contract does not cover:

- `POST /verify/bundle`
- `POST /internal/replay`
- authority topology selection
- convergence election
- verifier scheduling

---

## 5. Canonical Inputs

The 3.4 graph is derived from existing diagnostics artifacts. It does not create
new truth-bearing inputs.

The canonical upstream inputs are:

- `parity_report.json`
- `parity_determinism_incidents.json`
- `parity_convergence_report.json`
- `parity_drift_attribution_report.json`
- `parity_authority_drift_topology.json`

The graph surface MAY reference those inputs as provenance. It MUST NOT reuse
them as policy or authority.

---

## 6. Response Envelope

The public response root MUST be a JSON object.

The current v1 target envelope is:

```json
{
  "status": "PASS",
  "graph": {
    "node_count": 3,
    "edge_count": 2,
    "incident_count": 1,
    "nodes": [],
    "edges": [],
    "incidents": [],
    "clusters": []
  }
}
```

### Root Rules

- `graph` is required
- `status` is allowed and remains descriptive-only
- unknown top-level fields are allowed in v1 if they remain non-authoritative

---

## 7. Graph Object Contract

The `graph` object is the canonical graph payload.

### Required Fields

| Field | Type | Meaning |
|---|---|---|
| `node_count` | number | total node count represented by the graph |
| `edge_count` | number | total edge count represented by the graph |
| `incident_count` | number | total incident count represented by the graph |
| `nodes` | array | node set |
| `edges` | array | edge set |
| `incidents` | array | incident set |

### Optional Fields

| Field | Type | Meaning |
|---|---|---|
| `clusters` | array | derived cluster explanations |
| `surface_partition_count` | number | descriptive partition count only |
| `largest_surface_partition_size` | number | descriptive partition size only |
| `largest_outcome_cluster_size` | number | descriptive outcome-cluster size only |
| `surface_consistency_ratio` | number | descriptive ratio only |
| `outcome_convergence_ratio` | number | descriptive ratio only |
| `historical_only_node_count` | number | historical-only node count |
| `insufficient_evidence_node_count` | number | insufficient-evidence node count |

Optional fields remain diagnostics. They MUST NOT imply preferred cluster,
winner selection, or finality.

---

## 8. Node Model

Each `graph.nodes[]` entry MUST be a JSON object.

### Required Node Fields

| Field | Type | Meaning |
|---|---|---|
| `id` | string | stable node identity within the graph |
| `surface_key` | string | surface grouping key |
| `outcome_key` | string | outcome grouping key |
| `verdict` | string | descriptive verification verdict label |

### Allowed Verdict Values

- `TRUSTED`
- `UNTRUSTED`
- `INVALID`
- `REJECTED_BY_POLICY`

### Optional Node Fields

| Field | Type | Meaning |
|---|---|---|
| `run_id` | string | source run identifier |
| `authority_chain_id` | string | descriptive authority identity only |
| `execution_cluster_id` | string | deployment grouping hint only |
| `historical_only` | boolean | historical-only explanatory marker |
| `insufficient_evidence` | boolean | insufficient-evidence explanatory marker |

Node entries MUST describe. They MUST NOT rank.

---

## 9. Edge Model

Each `graph.edges[]` entry MUST be a JSON object.

### Required Edge Fields

| Field | Type | Meaning |
|---|---|---|
| `from` | string | source node id |
| `to` | string | target node id |
| `edge_type` | string | relationship kind |

### Allowed `edge_type` Values

- `same_outcome`
- `incident`

### Optional Edge Fields

| Field | Type | Meaning |
|---|---|---|
| `incident_id` | string | incident linked to the edge |
| `surface_key` | string | descriptive grouping key |

Edges MUST explain parity relationships. They MUST NOT encode routing or
selection instructions.

---

## 10. Incident Model

Each `graph.incidents[]` entry MUST be a JSON object.

### Required Incident Fields

| Field | Type | Meaning |
|---|---|---|
| `incident_id` | string | stable incident id |
| `surface_key` | string | surface grouping key |
| `severity` | string | descriptive incident severity |
| `nodes` | array | affected node ids |
| `node_count` | number | affected node count |

### Allowed `severity` Values

- `pure_determinism_failure`
- `authority_drift`
- `context_drift`
- `subject_drift`
- `mixed`

Incident severity remains descriptive. It MUST NOT be reused as a policy or
priority signal.

---

## 11. Cluster Model

`graph.clusters[]` is optional in v1, but if present it MUST remain descriptive.

### Required Cluster Fields

| Field | Type | Meaning |
|---|---|---|
| `cluster_id` | string | stable cluster id |
| `cluster_kind` | string | explanatory cluster class |
| `node_ids` | array | member node ids |
| `node_count` | number | member count |

### Allowed `cluster_kind` Values

- `surface_partition`
- `outcome_cluster`
- `authority_cluster`
- `historical_only_island`
- `insufficient_evidence_island`
- `incident_component`

Clusters MAY explain grouping. They MUST NOT elect a winner.

---

## 12. Determinism Rules

The graph surface MUST remain deterministic.

### Sorting Rules

- `nodes` sorted by `id`
- `edges` sorted by `from`, then `to`, then `edge_type`, then `incident_id`
- `incidents` sorted by `incident_id`
- `clusters` sorted by `node_count` descending, then `cluster_id`

### Counting Rules

- `node_count == len(nodes)`
- `edge_count == len(edges)`
- `incident_count == len(incidents)`

If a cluster array exists, `node_count` still refers to graph nodes, not summed
cluster sizes.

---

## 13. Forbidden Semantics

The graph surface MUST NOT expose fields or semantics such as:

- `selected_truth`
- `winning_verdict`
- `winning_cluster`
- `selected_authority`
- `preferred_verifier`
- `routing_hint`
- `verification_weight`
- `consensus_strength`
- `recommended_action`

The graph MAY expose descriptive dominance metadata from upstream diagnostics,
such as:

- `dominant_authority_chain_id`
- `dominant_authority_cluster_key`
- `surface_consistency_ratio`
- `outcome_convergence_ratio`

Those remain descriptive-only.

---

## 14. Read-Only Rules

The graph contract inherits the external diagnostics boundary:

- `GET` only
- unsupported query parameters => `400 unsupported_query_parameter`
- write methods => `405 method_not_allowed`
- unknown path => `404 not_found`

No 3.4 graph surface may mutate artifacts or execution state.

---

## 15. Failure Meaning

If this contract is violated, the graph surface has drifted from:

`explain distributed verification shape`

toward:

`select truth or influence execution`

That is a Phase-14 architectural failure.

---

## 16. References

- `docs/specs/phase14-distributed-observability/README.md`
- `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
- `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`
- `docs/specs/phase12-trust-layer/GRAPH_NON_AUTHORITATIVE_CONTRACT_GATE.md`
- `docs/specs/phase12-trust-layer/CONVERGENCE_NON_ELECTION_BOUNDARY_GATE.md`
- `docs/specs/phase12-trust-layer/PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`
