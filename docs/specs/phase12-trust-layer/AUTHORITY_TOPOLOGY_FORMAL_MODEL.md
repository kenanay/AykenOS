# Authority Topology Formal Model

**Version:** 1.0
**Status:** Draft (Phase-13 preparation)
**Date:** 2026-03-10
**Phase:** Phase-13 Observability Layer
**Type:** Non-normative formal model note
**Related Spec:** `PARITY_LAYER_FORMAL_MODEL.md`, `PARITY_LAYER_ARCHITECTURE.md`, `VERIFICATION_RELATIONSHIP_GRAPH.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document formalizes the authority-topology and authority-suppression surfaces now emerging from the parity layer.

It is non-normative.

Its role is to explain how AykenOS can:

- visualize authority clustering
- distinguish true authority drift from semantic-equivalent authority variation
- expose those results through diagnostics artifacts and `proofd`

without turning authority topology into authority selection.

The core rule is:

`authority topology = derived observability artifact`

and:

`authority topology != authority resolution`

---

## 2. Authority Surface

The executable parity model already treats authority as:

`A_i = (result_class, verifier_registry_snapshot_hash, effective_authority_scope, authority_chain_id)`

for node `i`.

This note refines that by separating:

- raw authority surface
- normalized semantic authority surface
- derived clustering artifacts

So for each node:

- `A_i`
  - raw authority surface
- `A_norm_i`
  - normalized authority surface used for semantic drift suppression

Authority topology is built over `A_i`, while suppression reasons about `A_norm_i`.

---

## 3. Canonical Input Boundary

Authority topology and suppression MUST derive from canonical parity objects.

Current canonical input:

- `NodeParityOutcome`

Relevant fields are:

- `authority_result_class`
- `verifier_registry_snapshot_hash`
- `effective_authority_scope`
- `authority_chain_id`

So the rule is:

- topology MAY consume canonical parity objects
- topology MUST NOT redefine authority truth objects
- suppression MAY normalize authority surfaces for diagnostics
- suppression MUST NOT arbitrate authority

---

## 4. Raw Authority Topology

### 4.1 Cluster Identity

The current local model groups current authority nodes by:

`cluster_key = (authority_chain_id, normalized_scope)`

Historical-only and unresolved nodes are held outside current clusters.

### 4.2 Cluster Classes

The current executable topology uses:

- `current`
- `current_drift`
- `historical_only`
- `unresolved`

These are observability classes, not authority decisions.

### 4.3 Dominant Cluster

The topology may compute a dominant current cluster:

`dominant_cluster = argmax(current_cluster_size)`

with deterministic tie-breaking.

This is diagnostic only.

So:

`dominant cluster != authoritative cluster`

It is a reporting reference, not an authority choice.

---

## 5. Semantic Authority Normalization

Raw authority drift can produce false positives.

So parity may compute:

`A_norm_i = normalize(A_i)`

The goal of normalization is not to decide authority.

Its goal is only:

`apparent drift -> semantic explanation`

### 5.1 Scope Normalization

The local model may canonicalize scope aliases such as:

- `*`
- `global`
- `root`
- `all`

into one normalized scope token.

### 5.2 Historical Shadow

If a node is `historical_only` but its `authority_chain_id` matches a current cluster, parity may record:

`historical_shadow`

This means:

- the node is not current
- the node is not unresolved
- the node still points to the same semantic authority chain lineage

### 5.3 Registry Skew

If nodes share the same normalized authority chain and scope but disagree only on `verifier_registry_snapshot_hash`, parity may record:

`registry_skew`

This means:

- apparent authority drift exists at the raw snapshot layer
- but the drift may reflect registry lag rather than a genuine authority split

---

## 6. Suppression Model

Authority suppression exists to prevent false drift inflation.

The rule is:

`false authority drift suppression = diagnostic normalization`

not:

`authority arbitration`

### 6.1 Suppression Predicate

Suppression applies when:

- raw `A_i` values differ
- but the difference is explained by an allowed semantic-equivalence or lag class

### 6.2 Current Suppression Classes

The current local model may emit:

- `scope_alias`
- `registry_skew`
- `historical_shadow`

These classes mean:

- `scope_alias`
  - raw scope strings differ but canonical scope is equivalent
- `registry_skew`
  - authority identity matches but registry snapshot differs
- `historical_shadow`
  - historical-only nodes shadow a current authority cluster

### 6.3 Suppression Artifact

Suppression is exported as:

- `parity_authority_suppression_report.json`

This artifact is observability only.

It MUST NOT:

- rewrite authority topology
- resolve which cluster is trusted
- downgrade true authority drift into success

It only records why apparent drift should not be treated as a fresh authority split.

---

## 7. Formal Rules

### 7.1 Derived Artifact Rule

`authority_topology(NodeParityOutcome[]) -> TopologyArtifact`

`authority_suppression(NodeParityOutcome[]) -> SuppressionArtifact`

Both outputs are derived diagnostics.

### 7.2 Non-Arbitration Rule

If topology or suppression produces:

- a dominant cluster
- a suppressed drift
- a historical shadow

none of those outputs imply:

- truth selection
- final authority
- policy acceptance

### 7.3 Suppression Purity Rule

Suppression MAY explain why drift is semantically non-material.

Suppression MUST NOT:

- hide true authority drift
- mutate canonical parity objects
- become policy or consensus input

### 7.4 `proofd` Exposure Rule

`proofd` may expose:

- `parity_authority_drift_topology.json`
- `parity_authority_suppression_report.json`

`proofd` MUST NOT:

- recompute authority topology
- recompute suppression outcomes
- reinterpret suppression as authority arbitration

So:

`proofd = read-only authority diagnostics surface`

and:

`proofd != authority resolver`

---

## 8. Relationship to Incident And Graph Models

Authority topology is parallel to incident topology.

- incident graph explains `same D_i + different K_i`
- authority topology explains clustering inside `A_i`
- authority suppression explains when raw authority drift is semantically non-material

These models complement each other:

- incident graph
  - verdict/topology side
- authority topology
  - authority clustering side
- authority suppression
  - false-drift guard side

Together they increase observability without creating a new truth layer.

---

## 9. Governance Boundary

The architectural boundary remains:

- `Parity Layer = Distributed Verification Diagnostics`
- `Parity Layer != consensus`
- `proofd != authority surface`

Authority topology and suppression MUST preserve that boundary.

If a later component:

- chooses an authoritative cluster
- upgrades dominant cluster into trust
- treats suppression as arbitration

it is no longer parity diagnostics.

---

## 10. Summary

Authority topology formalizes a diagnostic view over `A_i`.

Authority suppression formalizes a diagnostic explanation for semantically non-material drift.

Neither surface:

- resolves authority
- selects truth
- implements consensus

The correct model is:

`authority topology = observability`

and:

`authority suppression = semantic drift explanation`

not:

`authority arbitration`
