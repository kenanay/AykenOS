# N-Node Convergence Formal Model

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-09
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative formal model note
**Related Spec:** `PARITY_LAYER_FORMAL_MODEL.md`, `AYKENOS_DISTRIBUTED_TRUTH_MODEL_FORMAL_SECURITY_PROPERTIES.md`, `VERIFICATION_CONVERGENCE_THEOREM.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `CROSS_NODE_PARITY_HARDENING_CHECKLIST.md`, `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `tasks.md`

---

## 1. Purpose

This document extends the current pairwise parity model into an `N`-node convergence model.

The current local gate now exports `parity_convergence_report.json` as a first node-derived aggregate over stable `NodeParityOutcome` objects. That artifact now materializes `D_i` / `K_i` partitions in local evidence, while the underlying raw classifier still remains pairwise.
The local drift artifact also now summarizes `historical_authority_islands` and `insufficient_evidence_islands`, so early cluster-level lag classes are visible before service-backed diagnostics exist.
The local determinism surface now also exports `parity_determinism_incidents.json`, lifting same-`D_i` / different-`K_i` conditions into explicit node-derived incident artifacts.

It is non-normative.

Its role is to describe how a set of node outcomes can be analyzed as:

- a consistency structure
- a determinism structure
- a convergence structure

This note does not redefine the current pairwise parity contract.

It gives the formal bridge from:

`pairwise parity classification`

to:

`cluster-level distributed convergence analysis`

---

## 2. Core Objects

For each node `i`, let:

- `S_i`
  - subject surface
- `C_i`
  - context surface
- `A_i`
  - authority surface
- `V_i`
  - local verification verdict
- `artifact_form_i`
  - parity artifact form
- `evidence_state_i`
  - parity evidence sufficiency

Define:

`O_i = (S_i, C_i, A_i, V_i)`

and:

`E_i = (artifact_form_i, evidence_state_i)`

The node-level parity object is then:

`N_i = (O_i, E_i)`

For an `N`-node set:

`M = {N_1, N_2, ..., N_n}`

---

## 3. AykenOS Mapping

The current AykenOS mapping remains:

- `S_i`
  - `VerdictSubject`
  - `(bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`
- `C_i`
  - `verification_context_id`
- `A_i`
  - `(result_class, verifier_registry_snapshot_hash, effective_authority_scope, authority_chain_id)`
- `V_i`
  - `{Trusted, Untrusted, Invalid, RejectedByPolicy}`
- `artifact_form_i`
  - `{signed_receipt, local_verification_outcome}`
- `evidence_state_i`
  - `{sufficient, insufficient}`

This keeps the `N`-node model consistent with the current executable parity layer.

---

## 4. Convergence Keys

Two distinct keys are required.

### 4.1 Surface Key

Define:

`D_i = H(S_i, C_i, A_i)`

This is the surface-convergence key.

Its job is to group nodes that reached the same normalized truth surfaces, regardless of verdict.

### 4.2 Outcome Key

Define:

`K_i = H(S_i, C_i, A_i, V_i)`

This is the final convergence key.

Its job is to group nodes that reached the same full outcome.

The distinction is critical:

- same `D_i`, same `K_i`
  - full convergence
- same `D_i`, different `K_i`
  - determinism violation
- different `D_i`
  - ordinary consistency split

---

## 5. Partitions

The `N`-node model uses two partitions.

### 5.1 Surface Partition

Partition the sufficient nodes by `D_i`:

`P_surface = partition(M_sufficient by D_i)`

This groups nodes by:

`same S + same C + same A`

### 5.2 Outcome Partition

Partition the sufficient nodes by `K_i`:

`P_outcome = partition(M_sufficient by K_i)`

This groups nodes by:

`same S + same C + same A + same V`

### 5.3 Interpretation

The relationship between these partitions gives the high-level meaning:

- `|P_surface| = 1` and `|P_outcome| = 1`
  - full convergence
- `|P_surface| > 1`
  - consistency split
- `|P_surface| = 1` and `|P_outcome| > 1`
  - determinism violation

---

## 6. Aggregate Measures

### 6.1 Surface Consistency Ratio

Let:

`max_surface_cluster = max_j |cluster_j in P_surface|`

Define:

`surface_consistency_ratio = max_surface_cluster / |M_sufficient|`

This measures how many sufficient nodes agree on the same `(S, C, A)`.

### 6.2 Outcome Convergence Ratio

Let:

`max_outcome_cluster = max_j |cluster_j in P_outcome|`

Define:

`outcome_convergence_ratio = max_outcome_cluster / |M_sufficient|`

This measures how many sufficient nodes agree on the same `(S, C, A, V)`.

### 6.3 Determinism Violation Indicator

Define:

`determinism_violation = exists i,j : D_i = D_j and V_i != V_j`

This is the aggregate alarm condition for the determinism surface.

---

## 7. Pairwise Graph View

The `N`-node model may also be represented as a complete labeled graph:

`G = (Nodes, Edges)`

where each edge is a pairwise parity classification:

`edge(i,j) = Parity(N_i, N_j)`

This view is useful because it can expose:

- cliques of fully matching nodes
- minority divergence islands
- historical-only islands
- determinism-conflict edges

So the pairwise model remains useful, but the `N`-node model adds cluster-level interpretation on top of it.

---

## 8. Aggregate Status Set

The cleanest future aggregate classification set is:

- `N_PARITY_CONVERGED`
- `N_PARITY_CONSISTENCY_SPLIT`
- `N_PARITY_DETERMINISM_VIOLATION`
- `N_PARITY_HISTORICAL_ISLAND`
- `N_PARITY_INSUFFICIENT_EVIDENCE`
- `N_PARITY_MIXED`

These are not replacements for pairwise parity statuses.

They are aggregate interpretations over the graph and partitions.

### 8.1 `N_PARITY_CONVERGED`

Conditions:

- all sufficient nodes belong to one surface partition
- all sufficient nodes belong to one outcome partition
- no historical-only island remains

### 8.2 `N_PARITY_CONSISTENCY_SPLIT`

Conditions:

- `|P_surface| > 1`
- no determinism violation is required to explain the split

Meaning:

nodes are seeing different truth surfaces.

### 8.3 `N_PARITY_DETERMINISM_VIOLATION`

Conditions:

- `|P_surface| = 1`
- `|P_outcome| > 1`

Meaning:

the same normalized truth surfaces produced different verdicts.

### 8.4 `N_PARITY_HISTORICAL_ISLAND`

Conditions:

- at least one cluster is historical-only
- at least one cluster remains current

Meaning:

the system contains a temporal authority interpretation island.

### 8.5 `N_PARITY_INSUFFICIENT_EVIDENCE`

Conditions:

- one or more nodes remain insufficient
- and that insufficiency prevents clean aggregate classification

### 8.6 `N_PARITY_MIXED`

Conditions:

- multiple aggregate conditions coexist
- for example, consistency split plus insufficient evidence plus historical-only island

This is the correct top-level class for composite distributed failure structure.

---

## 9. Axis Counters

The `N`-node model should track which surface is actually splitting.

Minimum aggregate counters:

- `unique_subject_count`
- `unique_context_count`
- `unique_authority_count`
- `unique_outcome_count`
- `historical_only_count`
- `insufficient_evidence_count`

These counters help distinguish:

- subject fork
- context fork
- authority island
- verdict divergence

without collapsing everything into one aggregate label.

---

## 10. Core Theorems

### 10.1 N-Node Consistency Theorem

If all sufficient nodes normalize to the same `(S, C, A)`, then all sufficient nodes MUST belong to a single surface partition.

Formally:

`same D_i for all sufficient i -> |P_surface| = 1`

### 10.2 N-Node Determinism Theorem

If all sufficient nodes normalize to the same `(S, C, A)`, then all sufficient nodes MUST produce the same verdict.

Formally:

`same D_i for all sufficient i -> same V_i for all sufficient i`

Equivalently:

`same D_i for all sufficient i -> |P_outcome| = 1`

### 10.3 N-Node Convergence Theorem

If all sufficient nodes normalize to the same `(S, C, A)`, then all sufficient nodes MUST converge to the same final outcome key.

Formally:

`same D_i for all sufficient i -> same K_i for all sufficient i`

This is the cluster-level extension of the current pairwise convergence theorem.

---

## 11. Residual Risks

This model is stronger than the current implementation surface.

The main gaps are:

- current parity remains pairwise, not yet aggregate `N`-node
- determinism and consistency still share one report surface
- authority scope and chain already exist pairwise, but no cluster-level authority-island analysis exists yet
- insufficient-evidence handling is pairwise, not yet full-partition-aware
- `proofd` does not yet provide the service layer needed for live `N`-node orchestration

So the correct claim is:

`the formal N-node model is ready before the service-scale execution surface is ready`

---

## 12. Summary

The cleanest `N`-node parity model for AykenOS is:

`N_i = (O_i, E_i)`

where:

`O_i = (S_i, C_i, A_i, V_i)`

and:

`E_i = (artifact_form_i, evidence_state_i)`

Then:

- `D_i = H(S_i, C_i, A_i)` defines the surface partition
- `K_i = H(S_i, C_i, A_i, V_i)` defines the final convergence partition

This gives the decisive rule:

- same `D`, different `K`
  - determinism violation
- different `D`
  - consistency split
- insufficient evidence
  - classification boundary

This is the shortest formal path from the current pairwise parity layer to a true distributed convergence engine.
