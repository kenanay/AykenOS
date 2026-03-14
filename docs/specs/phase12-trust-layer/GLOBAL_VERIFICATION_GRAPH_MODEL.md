# Global Verification Graph Model

**Version:** 1.0
**Status:** Informational global graph model
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative global graph artifact
**Related Spec:** `VERIFICATION_RELATIONSHIP_GRAPH.md`, `VERIFICATION_OBSERVABILITY_MODEL.md`, `DISTRIBUTED_VERIFICATION_TOPOLOGY.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `PHASE13_ARCHITECTURE_MAP.md`, `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`

---

## 1. Purpose

This document defines the global verification graph for AykenOS distributed verification.

Its role is to combine:

- verifier-node topology
- canonical truth surfaces
- relationship graph structure
- authority overlays
- convergence views

into one global diagnostics model.

The central rule is:

`global verification graph = global observability projection`

and:

`global verification graph != global truth engine`

---

## 2. Starting Point

AykenOS already defines:

- canonical verification results
  - `TruthSurface = EvidenceBoundVerificationResult = (Q, V, E)`
- relationship graph
  - `RG = (TS, PR, FA, I, CP, AG)`
- distributed verifier topology
  - many verifier nodes exchanging artifacts without shared mutable truth state

The global verification graph begins only after those objects already exist.

So it is not a replacement for:

- the verification function
- the relationship graph
- the node topology model

It is the global projection that binds them together.

Behavior across multiple runs may additionally be projected through the Verification Diversity Ledger (`VDL`).

That ledger remains a derived observability companion.

It MUST NOT promote graph structure into authority or scheduling output.

---

## 3. Global Graph Objects

Let:

- `VN = {N_i}`
  - verifier nodes
- `TE = {X_ij}`
  - topology or transport edges between nodes
- `TS = {T_k}`
  - canonical truth surfaces
- `B = {B(i,k)}`
  - binding relation between node `N_i` and truth surface `T_k`
- `PR = {P_ab}`
  - parity relations between truth surfaces
- `FA = {F_ab}`
  - failure attributions over parity relations
- `I = {I_l}`
  - incident annotations
- `CP = {C_m}`
  - convergence partitions
- `AG = {A_g}`
  - authority overlays

Define the compact global verification graph:

`GVG = (VN, TE, TS, B, PR, FA, I, CP, AG)`

This means the global graph includes both:

- node-level placement and exchange structure
- truth-level diagnostic relationship structure

---

## 4. Layer Interpretation

The global graph is a layered model.

### 4.1 Topology Layer

`(VN, TE)`

This layer captures:

- which verifier nodes exist
- which artifact-exchange or diagnostics paths exist
- how distributed verification surfaces are connected

It does not define truth.

### 4.2 Truth Layer

`(TS, B)`

This layer captures:

- which canonical truth surfaces exist
- which nodes emitted, observed, or hold those truth surfaces

It does not elect one truth surface over another.

### 4.3 Relationship Layer

`(PR, FA)`

This layer captures:

- which truth surfaces match or diverge
- how mismatch is attributed semantically

### 4.4 Derived Overlay Layer

`(I, CP, AG)`

This layer captures:

- incident severity and findings
- convergence structure
- authority-topology interpretation

These overlays are global diagnostics only.

---

## 5. Binding Rule

The binding relation:

`B(i,k)`

means:

`node N_i is associated with truth surface T_k`

through local verification, artifact possession, or derived observability context.

This relation is important because it prevents the global graph from collapsing nodes and truth surfaces into one object type.

The stable rule is:

`verifier nodes != truth surfaces`

and:

`truth surfaces may be related to nodes without becoming node state`

---

## 6. Global Query Semantics

The global verification graph is designed to answer:

- which nodes produced or hold which truth surfaces
- where mismatches appear across the network
- whether mismatches cluster by subject, context, authority, or evidence
- how authority overlays align with convergence partitions
- where determinism incidents propagate

It is not designed to answer:

- which node should lead
- which verdict becomes globally final
- which authority wins by graph majority
- which cluster commits one state

So the global graph is a fabric-wide explanation surface, not a fabric-wide election surface.

---

## 7. Non-Goals

The global verification graph must not become:

- consensus fabric
- replicated-state graph
- authority-arbitration engine
- global truth-election mechanism

If `GVG` starts selecting winners rather than exposing relationships, it has crossed the AykenOS architecture boundary.

---

## 8. Phase-13 Relevance

This model is the most direct graph-level bridge into Phase-13 because it allows AykenOS to describe a distributed verification fabric without importing consensus assumptions.

Phase-13 can build on `GVG` through:

- read-only graph queries in `proofd`
- multi-node incident views
- authority-overlay exploration
- convergence partition reporting
- transport-aware diagnostics

while keeping the canonical truth rule unchanged:

`same subject + same context + same authority -> same verdict`

---

## 9. Summary

The compact global verification graph is:

`GVG = (VN, TE, TS, B, PR, FA, I, CP, AG)`

It unifies topology, truth surfaces, relationships, incidents, convergence, and authority overlays into one global observability projection without turning diagnostics into consensus, authority arbitration, or truth election.
