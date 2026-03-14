# Parity Graph Model

**Version:** 1.0
**Status:** Informational graph model
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative graph model note
**Related Spec:** `DISTRIBUTED_VERIFICATION_TOPOLOGY.md`, `VERIFICATION_MODEL.md`, `VERIFICATION_INVARIANTS.md`, `VERIFICATION_OBSERVABILITY_MODEL.md`, `VERIFICATION_RELATIONSHIP_GRAPH.md`, `PARITY_LAYER_ARCHITECTURE.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`

---

## 1. Purpose

This document defines the compact graph model used for AykenOS parity and diagnostics surfaces.

Its role is to explain how diagnostics graphs relate node-local verification results without turning graph structure into consensus or truth election.

The central rule is:

`graph = observability topology`

and:

`graph != consensus topology`

---

## 2. Graph Objects

Let:

- `N_i`
  - node-local verification object
- `P_ij`
  - parity relation between nodes `i` and `j`
- `I_k`
  - determinism incident object
- `T_m`
  - authority-topology cluster or partition

The compact graph is therefore:

`G = (Nodes, Edges, Incidents, TopologyPartitions)`

where:

- `Nodes`
  - node-local verification outputs
- `Edges`
  - derived comparison relations
- `Incidents`
  - graph-associated determinism findings
- `TopologyPartitions`
  - authority and convergence grouping artifacts

---

## 3. Node Model

Graph nodes are derived from canonical verification outputs, not invented by the graph layer.

Typical node inputs include:

- verdict subject
- verification context
- authority surface
- verdict class
- artifact availability

Architectural rule:

`graph nodes are derived from canonical verification objects`

---

## 4. Edge Model

### 4.1 Parity Edges

Parity edges describe whether two node-local outputs match or diverge.

These edges may encode:

- subject mismatch
- context mismatch
- authority mismatch
- verdict mismatch
- insufficient evidence

### 4.2 Incident Edges

Incident edges connect nodes or node groups through determinism failures or drift-class relationships.

These edges remain diagnostic only.

### 4.3 Authority / Topology Edges

Authority and topology edges may explain:

- shared authority lineage
- authority drift clusters
- dominance partitions
- historical-only islands

These edges do not choose a winning authority.

---

## 5. Graph Semantics

The graph answers:

- which results relate
- where they diverge
- how divergence clusters
- how authority and convergence partitions appear

The graph does not answer:

- which node wins
- which result is final
- which state must be committed

So the graph is a diagnostic structure over verification outputs.

---

## 6. Derived-Only Rule

The parity graph is derived from:

- canonical verification objects
- parity outputs
- determinism incidents
- topology partitions

It must not introduce new truth-bearing objects.

The stable rule is:

`graph is derived and non-canonical`

---

## 7. Summary

The compact parity graph model is:

`G = (Nodes, Edges, Incidents, TopologyPartitions)`

This graph makes cross-node verification relationships visible without turning observability into consensus.
