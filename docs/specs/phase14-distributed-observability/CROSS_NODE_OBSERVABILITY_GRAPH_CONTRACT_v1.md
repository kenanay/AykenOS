# Cross-Node Observability Graph Contract v1

**Phase:** 14  
**Workstream:** 3.4  
**Status:** MERGED
**Authority:** `ARCHITECTURE_FREEZE.md`  
**Related Surface:** `GET /diagnostics/graph`, `GET /diagnostics/graph/overlay`, `GET /diagnostics/runs/{run_id}/graph`

---

## 1. Purpose

This document defines the canonical Phase-14 contract target for the cross-node
observability graph surface.

Its job is to make distributed verification drift readable without turning graph
shape into authority, routing, or consensus.

The shortest correct sentence is:

`graph explains divergence; graph does not decide truth`

This contract exists so endpoint behavior can be built and reviewed against one
declared truth surface instead of growing route-first.

---

## 2. Current Status Boundary

Current `main` state:

- `GET /diagnostics/runs/{run_id}/graph` serves a contract-bound Phase-14
  envelope backed by `parity_incident_graph.json`
- `GET /diagnostics/graph` serves a partitioned derived surface grouped by
  `graph_version + authority + env_hash + artifact_set_hash`
- `GET /diagnostics/graph/overlay` serves overlay-only agreement/conflict/island
  diagnostics derived from those partitions

That means this document is:

- canonical for 3.4 field vocabulary and shape intent
- authoritative for 3.4 design direction
- aligned to the merged implementation slice on `main`

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
- `GET /diagnostics/graph/overlay`
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

## 5. Security Boundary

The 3.4 graph surface is security-sensitive because it aggregates multi-node
diagnostics.

The graph contract MUST therefore preserve these boundaries:

- graph aggregation MUST NOT become truth selection
- graph aggregation MUST NOT become verifier routing input
- graph aggregation MUST NOT become replay admission input
- graph aggregation MUST NOT mix incompatible execution classes
- graph output MUST remain read-only and descriptive

The two most important security sentences are:

- `cluster size MUST NOT imply correctness`
- `graph output MUST NOT be consumed by execution-bearing paths`

---

## 6. Canonical Inputs

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

## 7. Response Envelope

The public response root MUST be a JSON object.

### 7.1 Run-Scoped Envelope

`GET /diagnostics/runs/{run_id}/graph` serves the Phase-14 run-scoped graph
envelope:

```json
{
  "graph_version": "v1",
  "authority": "github-hosted-ubuntu-24.04-x64",
  "env_hash": "sha256...",
  "status": "PASS",
  "provenance": {
    "artifact_set_hash": "sha256...",
    "source_runs": ["run-20260405-1"]
  },
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

Run-scoped required top-level fields:

- `graph_version`
- `authority`
- `env_hash`
- `status`
- `provenance`
- `graph`

Run-scoped rules:

- `graph_version` MUST equal `v1`
- `authority` MUST be non-empty
- `env_hash` MUST be non-empty
- `provenance.artifact_set_hash` MUST be non-empty
- `provenance.source_runs` MUST be a sorted, unique, non-empty string array
- unknown top-level fields are allowed in v1 if they remain non-authoritative

### 7.2 Root Partitioned Envelope

`GET /diagnostics/graph` serves a partitioned derived surface:

```json
{
  "graph_origin": "derived",
  "authority_classification": "non_authoritative",
  "aggregation_mode": "overlay_only",
  "partition_count": 1,
  "partitions": [
    {
      "partition_id": "sha256...",
      "partition_key": {
        "graph_version": "v1",
        "authority": "github-hosted-ubuntu-24.04-x64",
        "env_hash": "sha256...",
        "artifact_set_hash": "sha256..."
      },
      "run_count": 2,
      "run_ids": ["run-20260405-a", "run-20260405-b"],
      "source_runs": ["phase12-cross-node-parity"],
      "graph": {
        "node_count": 3,
        "edge_count": 2,
        "incident_count": 1,
        "nodes": [],
        "edges": [],
        "incidents": []
      }
    }
  ]
}
```

Root partitioned required top-level fields:

- `graph_origin`
- `authority_classification`
- `aggregation_mode`
- `partition_count`
- `partitions`

Root partitioned rules:

- `graph_origin` MUST equal `derived`
- `authority_classification` MUST equal `non_authoritative`
- `aggregation_mode` MUST equal `overlay_only`
- `partition_count == len(partitions)`
- each `partition_id` MUST be unique
- each `partition_key.graph_version` MUST equal `v1`
- each `partition_key.authority`, `partition_key.env_hash`, and
  `partition_key.artifact_set_hash` MUST be non-empty
- `run_ids` and `source_runs` MUST each be sorted, unique string arrays
- `run_count == len(run_ids)`
- `graph` MUST satisfy the shared graph object contract from this document

Partitioning rule:

- incompatible execution classes MUST be represented as separate partitions, not
  one merged graph

### 7.3 Root Overlay Envelope

`GET /diagnostics/graph/overlay` serves overlay-only aggregation diagnostics:

```json
{
  "graph_origin": "derived",
  "authority_classification": "non_authoritative",
  "aggregation_mode": "overlay_only",
  "partition_count": 2,
  "agreement_count": 1,
  "conflict_count": 1,
  "island_count": 2,
  "agreements": [],
  "conflicts": [],
  "islands": []
}
```

Root overlay required top-level fields:

- `graph_origin`
- `authority_classification`
- `aggregation_mode`
- `partition_count`
- `agreement_count`
- `conflict_count`
- `island_count`
- `agreements`
- `conflicts`
- `islands`

Root overlay rules:

- `graph_origin` MUST equal `derived`
- `authority_classification` MUST equal `non_authoritative`
- `aggregation_mode` MUST equal `overlay_only`
- `agreement_count == len(agreements)`
- `conflict_count == len(conflicts)`
- `island_count == len(islands)`
- overlay explains agreements, conflicts, and isolation only
- overlay MUST NOT expose selected truth, winning cluster, or resolved verdict

### 7.4 Authority and Provenance Rules

- `authority` identifies the execution class that produced the graph inputs
- `env_hash` binds the graph to a deterministic execution fingerprint
- `provenance.artifact_set_hash` binds the graph to a specific artifact set
- `provenance.source_runs` lists the contributing run ids

Mixed authority or mixed `env_hash` inputs are forbidden inside one partition.

If inputs come from incompatible execution classes, they MUST be represented as
separate partitions, not one merged graph.

---

## 8. Graph Object Contract

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

## 9. Node Model

Each `graph.nodes[]` entry MUST be a JSON object.

### Required Node Fields

| Field | Type | Meaning |
|---|---|---|
| `id` | string | stable node identity within the graph |
| `node_fingerprint` | string | stable fingerprint for the node observation source |
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

`id` is presentation-facing. `node_fingerprint` is the binding surface.

Implementations MUST treat `node_fingerprint` as the stable identity anchor for
graph membership and correlation. A plain display id is not sufficient.

Node entries MUST describe. They MUST NOT rank.

---

## 10. Edge Model

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

## 11. Incident Model

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

Severity is not ordered in v1.

That means:

- severity MUST NOT be compared for ranking
- severity MUST NOT be converted into trust weighting
- severity MUST NOT imply preferred node or preferred cluster

---

## 12. Conflict Classification

The graph MAY expose conflict classes, but they remain descriptive only.

Allowed conflict kinds include:

- `verdict_mismatch`
- `subject_drift`
- `context_drift`
- `authority_drift`
- `mixed`

Conflict labels explain disagreement shape. They MUST NOT be used as execution
switches.

---

## 13. Cluster Model

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

Cluster size is descriptive only.

That means:

- larger cluster size MUST NOT imply correctness
- larger cluster size MUST NOT imply authority
- larger cluster size MUST NOT imply preferred routing
- cluster dominance metadata MUST remain explanatory only

---

## 14. Determinism Rules

The graph surface MUST remain deterministic.

### Sorting Rules

- root `partitions` sorted by `partition_id`
- `nodes` sorted by `id`
- `edges` sorted by `from`, then `to`, then `edge_type`, then `incident_id`
- `incidents` sorted by `incident_id`
- `clusters` sorted by `node_count` descending, then `cluster_id`
- overlay `agreements` sorted by `node_fingerprint`
- overlay `conflicts` sorted by `node_fingerprint`
- overlay `islands` sorted by `partition_id`

### Counting Rules

- `node_count == len(nodes)`
- `edge_count == len(edges)`
- `incident_count == len(incidents)`
- `partition_count == len(partitions)` for root partitioned responses
- `agreement_count == len(agreements)` for root overlay responses
- `conflict_count == len(conflicts)` for root overlay responses
- `island_count == len(islands)` for root overlay responses

If a cluster array exists, `node_count` still refers to graph nodes, not summed
cluster sizes.

---

## 15. Bounded Graph Rules

The graph surface MUST be bounded.

That means:

- graph generation MUST enforce an implementation-defined upper bound on
  `node_count`
- graph generation MUST enforce an implementation-defined upper bound on
  `edge_count`
- graph generation MUST NOT permit unbounded query-driven fan-out

Exact numeric ceilings may be implementation-specific, but they MUST be fixed
before the 3.4 endpoint is promoted as a stable Phase-14 graph surface.

---

## 16. Forbidden Semantics

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

## 17. Read-Only Rules

The graph contract inherits the external diagnostics boundary:

- `GET` only
- unsupported query parameters => `400 unsupported_query_parameter`
- write methods => `405 method_not_allowed`
- unknown path => `404 not_found`

In v1, graph endpoints are queryless by default.

That means:

- `GET /diagnostics/graph` accepts no query parameters
- `GET /diagnostics/graph/overlay` accepts no query parameters
- `GET /diagnostics/runs/{run_id}/graph` accepts no query parameters

No 3.4 graph surface may mutate artifacts or execution state.

---

## 18. Threat Model Summary

The v1 graph contract is designed to reduce these concrete risks:

| Threat | Contract Countermeasure |
|---|---|
| truth-election drift | forbidden semantics + non-negotiable invariants |
| graph-to-routing drift | read-only rules + routing-hint prohibition |
| data poisoning | provenance + `authority` + `env_hash` + `node_fingerprint` |
| cross-authority contamination | mixed authority / mixed `env_hash` forbidden |
| topology inference misuse | descriptive-only cluster and conflict model |
| DoS by oversized graph | bounded graph rules |

---

## 19. Failure Meaning

If this contract is violated, the graph surface has drifted from:

`explain distributed verification shape`

toward:

`select truth or influence execution`

That is a Phase-14 architectural failure.

---

## 20. References

- `docs/specs/phase14-distributed-observability/README.md`
- `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
- `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`
- `docs/specs/phase12-trust-layer/GRAPH_NON_AUTHORITATIVE_CONTRACT_GATE.md`
- `docs/specs/phase12-trust-layer/CONVERGENCE_NON_ELECTION_BOUNDARY_GATE.md`
- `docs/specs/phase12-trust-layer/PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`
