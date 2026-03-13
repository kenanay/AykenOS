# Verification Relationship Graph

**Version:** 1.0
**Status:** Informational graph note
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative relationship-graph artifact
**Related Spec:** `VERIFICATION_OBSERVABILITY_MODEL.md`, `GLOBAL_VERIFICATION_GRAPH_MODEL.md`, `PARITY_GRAPH_MODEL.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `DISTRIBUTED_VERIFICATION_TOPOLOGY.md`, `VERIFICATION_FAILURE_MODEL.md`, `PHASE13_ARCHITECTURE_MAP.md`

---

## 1. Purpose

This document defines the unified relationship graph for AykenOS distributed verification.

Its role is to combine:

- parity graph structure
- authority graph structure
- convergence partitions
- determinism-incident annotations

into one derived graph projection over canonical verification results.

The central rule is:

`relationship graph = observability projection over verification outputs`

and:

`relationship graph != truth engine`

---

## 2. Starting Point

AykenOS already defines:

- verification input
  - `Q = (S, C, A)`
- local verification semantics
  - `Eval(Q) -> V`
- canonical truth surface
  - `TruthSurface = EvidenceBoundVerificationResult = (Q, V, E)`

The relationship graph begins only after canonical `TruthSurface` objects already exist.

So the graph is not an extension of the verification function.

It is a projection over verification outputs.

It does not modify, extend, or influence the verification function.

---

## 3. Graph Projection Objects

Let:

- `TS = {T_i}`
  - truth-surface nodes
- `PR = {P_ij}`
  - parity relations
- `FA = {F_ij}`
  - failure attributions
- `I = {I_k}`
  - determinism-incident annotations
- `CP = {C_m}`
  - convergence partitions
- `AG = {A_g}`
  - authority-graph overlays

Define the relationship graph as:

`RG = (TS, PR, FA, I, CP, AG)`

This graph is therefore a compact global view of how verification results relate across nodes and runs.

---

## 4. Node Semantics

Each node in `TS` is a canonical verification result:

`T_i = (Q_i, V_i, E_i)`

So graph nodes represent:

- verification inputs
- resulting verdicts
- durable evidence artifacts

They do not represent:

- leader state
- consensus membership
- control-plane ownership

Architectural rule:

`graph nodes are canonical verification results; graph structure is derived`

---

## 5. Relation Families

### 5.1 Parity Relations

`PR` captures whether two truth surfaces:

- match
- diverge by subject
- diverge by context
- diverge by authority
- diverge by verdict
- lack sufficient evidence for comparison

These are operational relation labels.

### 5.2 Failure Attributions

`FA` maps relation mismatch into semantic classes:

- subject drift
- context drift
- authority drift
- artifact loss
- determinism violation

This keeps relation semantics explicit.

### 5.3 Incident Annotations

`I` marks high-severity semantic findings such as:

- determinism incidents
- drift clusters requiring investigation
- repeated insufficient-evidence islands

Incidents annotate the graph.

They do not replace node-local verdicts.

### 5.4 Convergence Partitions

`CP` groups truth surfaces into partitions such as:

- fully converged clusters
- ordinary consistency splits
- determinism-conflict partitions
- historical-only islands
- insufficient-evidence islands

These partitions describe graph shape.

They do not force agreement.

### 5.5 Authority Overlays

`AG` adds authority-topology interpretation such as:

- shared authority lineage
- authority drift clusters
- suppression overlays
- historical shadow relations

Authority overlays remain derived diagnostics.

They do not arbitrate authority.

---

## 6. Relationship Flow

The intended graph-building flow is:

`TruthSurface -> ParityRelations -> FailureAttributions -> Incident / Convergence / Authority overlays`

In compact form:

`TS -> PR -> FA -> {I, CP, AG}`

This preserves the required architecture order:

- verification semantics first
- artifact truth surfaces second
- relationship graph third

It prevents graph structure from becoming a hidden semantic governor.

---

## 7. Query Semantics

The relationship graph is designed to answer:

- which truth surfaces match
- where divergence begins
- whether divergence is attributable
- how mismatch clusters
- how authority and convergence overlays interact

It is not designed to answer:

- which node wins
- which verdict becomes final
- which cluster should be trusted by election

So the dominant query class is:

`relationship explanation`

not:

`truth selection`

---

## 8. Non-Goals

The relationship graph must not become:

- consensus graph
- leader-election graph
- authority-arbitration graph
- replicated-state machine view

If the graph starts selecting winners rather than explaining relations, it has crossed the AykenOS architectural boundary.

---

## 9. Phase-13 Relevance

This artifact is the cleanest bridge between current local parity outputs and future service-backed distributed observability.

Phase-13 can build on this graph through:

- read-only graph queries
- incident severity filtering
- convergence partition views
- authority-overlay analysis
- cross-node diagnostics transport

without changing the canonical truth rule:

`same subject + same context + same authority -> same verdict`

---

## 10. Summary

The compact AykenOS relationship graph is:

`RG = (TS, PR, FA, I, CP, AG)`

It unifies parity, authority, convergence, and incident views into one observability projection without turning diagnostics into consensus, arbitration, or truth election.
