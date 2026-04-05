# Observability UX Contract v1

**Phase:** 14
**Workstream:** 3.5
**Status:** CONTRACT-FIRST
**Authority:** `ARCHITECTURE_FREEZE.md`
**Related Surface:** `GET /diagnostics/summary`, `GET /diagnostics/runs/{run_id}/summary`

---

## 1. Purpose

This document defines the canonical Phase-14 contract target for the
human-readable observability surface.

Its job is to make existing diagnostics readable without turning presentation
into authority, ranking, or decision logic.

The shortest correct sentence is:

`summary explains the system; summary does not decide for the system`

This contract exists before endpoint implementation so the UX layer grows from a
declared truth surface instead of route-first convenience.

---

## 2. Current Status Boundary

Current `main` state:

- `GET /diagnostics/summary` does not exist yet
- `GET /diagnostics/runs/{run_id}/summary` does not exist yet
- the canonical upstream diagnostics surfaces already exist on `main`

That means this document is:

- canonical for 3.5 field vocabulary and semantic limits
- authoritative for 3.5 design direction
- not yet runtime-authoritative until the 3.5 implementation slice lands

This document MUST NOT be read as:

- a scoring system
- a truth-election surface
- a routing surface
- a ranking protocol

---

## 3. Non-Negotiable Invariants

- `summary = derived diagnostics`
- `summary != authority`
- `summary != decision input`
- `summary != ranking`
- `summary != routing hint`
- `summary != truth selection`
- `summary != replay admission signal`
- `summary MUST NOT invent data`

If any 3.5 implementation violates those rules, it is not Phase-14
observability UX.

---

## 4. Covered Surfaces

This contract covers:

- `GET /diagnostics/summary`
- `GET /diagnostics/runs/{run_id}/summary`
- human-readable JSON summary payloads
- optional structured text renderings derived from the same canonical summary
  object

This contract does not cover:

- `GET /diagnostics/graph`
- `GET /diagnostics/graph/overlay`
- `POST /verify/bundle`
- `POST /internal/replay`
- any authority-bearing or execution-bearing surface

---

## 5. Security Boundary

The observability UX surface is security-sensitive because explanation can be
misread as authority.

The UX contract MUST therefore preserve these boundaries:

- summary generation MUST NOT create new truth semantics
- summary generation MUST NOT produce routing or verifier preference
- summary generation MUST NOT collapse partitions into a winner
- summary generation MUST remain read-only and descriptive

The two most important security sentences are:

- `display language MUST NOT imply correctness selection`
- `summary output MUST NOT be consumed by execution-bearing paths`

---

## 6. Canonical Inputs

The observability UX surface is a projection layer over existing canonical
diagnostics surfaces. It does not create new truth-bearing inputs.

### 6.1 Root Summary Inputs

`GET /diagnostics/summary` MAY only be derived from declared root diagnostics
surfaces such as:

- `GET /diagnostics/version`
- `GET /diagnostics/incidents`
- `GET /diagnostics/graph`
- `GET /diagnostics/graph/overlay`

### 6.2 Run-Scoped Summary Inputs

`GET /diagnostics/runs/{run_id}/summary` MAY only be derived from declared
run-scoped diagnostics surfaces such as:

- `GET /diagnostics/runs/{run_id}`
- `GET /diagnostics/runs/{run_id}/incidents`
- `GET /diagnostics/runs/{run_id}/graph`
- `GET /diagnostics/runs/{run_id}/boundary`

### 6.3 Dependency Rule

Summary builders MUST use explicit upstream dependencies.

That means:

- no ambient global state
- no hidden scoring inputs
- no recomputed authority decisions
- no undeclared artifact dependency

The shortest correct builder rule is:

`summary = explicit projection of declared diagnostics surfaces`

---

## 7. Epistemic Boundary

Every v1 summary payload MUST declare its epistemic boundary explicitly.

The required v1 boundary object is:

```json
{
  "summary_origin": "derived",
  "authority_classification": "non_authoritative",
  "display_mode": "human_readable",
  "epistemic_boundary": {
    "produces_truth": false,
    "produces_decision": false,
    "produces_ranking": false
  }
}
```

Rules:

- `summary_origin` MUST equal `derived`
- `authority_classification` MUST equal `non_authoritative`
- `display_mode` MUST equal `human_readable`
- `epistemic_boundary.produces_truth` MUST equal `false`
- `epistemic_boundary.produces_decision` MUST equal `false`
- `epistemic_boundary.produces_ranking` MUST equal `false`

This boundary object is not optional in v1.

---

## 8. Response Envelope

The public response root MUST be a JSON object.

### 8.1 Root Summary Envelope

`GET /diagnostics/summary` serves a human-readable root summary surface.

The v1 target envelope is:

```json
{
  "summary_origin": "derived",
  "authority_classification": "non_authoritative",
  "display_mode": "human_readable",
  "epistemic_boundary": {
    "produces_truth": false,
    "produces_decision": false,
    "produces_ranking": false
  },
  "snapshot": {
    "partition_count": 3,
    "total_nodes": 128,
    "total_incidents": 12
  },
  "overlay": {
    "agreements": 87,
    "conflicts": 9,
    "islands": 32
  },
  "incidents": {
    "pure_determinism_failure": 2,
    "authority_drift": 3,
    "context_drift": 4,
    "subject_drift": 2,
    "mixed": 1
  },
  "explanation": [
    "Most partitions agree on observed verification outcomes.",
    "Conflicts remain localized to a subset of partitions.",
    "No global truth or winner is selected by this surface."
  ]
}
```

Required top-level fields:

- `summary_origin`
- `authority_classification`
- `display_mode`
- `epistemic_boundary`
- `snapshot`
- `overlay`
- `incidents`
- `explanation`

### 8.2 Run-Scoped Summary Envelope

`GET /diagnostics/runs/{run_id}/summary` serves a human-readable run summary
surface.

The v1 target envelope is:

```json
{
  "summary_origin": "derived",
  "authority_classification": "non_authoritative",
  "display_mode": "human_readable",
  "epistemic_boundary": {
    "produces_truth": false,
    "produces_decision": false,
    "produces_ranking": false
  },
  "run_id": "run-20260405-1",
  "snapshot": {
    "node_count": 3,
    "incident_count": 1
  },
  "incidents": {
    "pure_determinism_failure": 1,
    "authority_drift": 0,
    "context_drift": 0,
    "subject_drift": 0,
    "mixed": 0
  },
  "explanation": [
    "This run produced one localized determinism incident.",
    "The run-scoped graph remains descriptive and non-authoritative."
  ]
}
```

Required top-level fields:

- `summary_origin`
- `authority_classification`
- `display_mode`
- `epistemic_boundary`
- `run_id`
- `snapshot`
- `incidents`
- `explanation`

Unknown additive fields are allowed in v1 if they remain descriptive-only and
non-authoritative.

---

## 9. Allowed Content

The UX summary MAY include:

- count summaries
- incident severity distributions
- partition totals
- agreement/conflict/island totals
- short explanatory statements
- operator-oriented descriptive labels

All such content remains descriptive-only.

---

## 10. Forbidden Semantics

The UX summary MUST NOT expose fields or semantics such as:

- `score`
- `winner`
- `winning_partition`
- `preferred_node`
- `preferred_verifier`
- `routing_hint`
- `resolved_truth`
- `recommended_action`
- `trust_ranking`
- `priority`

Those fields are schema-level exclusions in v1.

That means:

- they MUST NOT appear as null placeholders
- they MUST NOT appear as optional fields
- they MUST NOT appear in explanation text as imperative recommendations

The summary MAY describe observations such as:

- `higher agreement density`
- `localized conflicts`
- `partition isolation`

Those descriptions remain explanatory only.

---

## 11. Determinism and Query Rules

The UX surface MUST remain deterministic.

Rules:

- summaries MUST be derived from deterministic upstream diagnostics surfaces
- explanation arrays MUST have deterministic ordering
- root and run-scoped summary endpoints are queryless in v1
- unsupported query parameters MUST return `400 unsupported_query_parameter`

That means:

- `GET /diagnostics/summary` accepts no query parameters
- `GET /diagnostics/runs/{run_id}/summary` accepts no query parameters

---

## 12. Failure Meaning

If this contract is violated, the UX surface has drifted from:

`explain existing diagnostics`

toward:

`rank, select, or decide`

That is a Phase-14 architectural failure.

---

## 13. References

- `docs/specs/phase14-distributed-observability/README.md`
- `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
- `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`
- `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
- `docs/specs/phase14-distributed-observability/PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md`
