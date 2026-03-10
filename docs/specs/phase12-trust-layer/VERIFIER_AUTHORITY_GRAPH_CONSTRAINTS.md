# Verifier Authority Graph Constraints

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines the graph constraints for verifier authority delegation.

Its job is to prevent distributed verifier authority from becoming cyclic, self-referential, or scope-inflating.

This specification is normative for any verifier delegation model introduced after explicit default-deny authority semantics.

Critical rule:

`verifier delegation graph MUST be a DAG`

Deterministic authority-chain resolution is defined separately in:

`VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`

---

## 2. Problem Statement

Verifier authority can remain cryptographically valid while becoming logically unsound.

The dangerous class is authority graph corruption:
- delegation cycles
- self-delegation
- unbounded delegation depth
- delegated scope expansion

These attacks do not require:
- breaking signatures
- mutating proof bundles
- mutating receipts

They instead corrupt who is allowed to speak for distributed trust.

---

## 3. Graph Model

### 3.1 Nodes

Each verifier authority node is identified by at least:
- `verifier_id`
- `verifier_pubkey_id`
- `verifier_registry_snapshot_hash`
- `authority_scope`

### 3.2 Directed Edges

A directed edge `A -> B` means:
- verifier `A` explicitly delegates a bounded subset of authority to verifier `B`

### 3.3 Root Nodes

Root verifier authorities are those whose authority is granted directly by the verifier trust registry root set.

Nodes with no incoming delegated authority edges are NOT implicitly roots unless explicitly declared by the verifier trust registry.

### 3.4 Delegate Nodes

Delegates are verifier authorities whose distributed trust authority depends on at least one explicit parent authority edge.

### 3.5 Parent Uniqueness Rule

After normalization, lineage filtering, and historical filtering, a current delegated verifier MUST have at most one surviving incoming authority edge.

Conceptual rule:

`in_degree_current(delegate) <= 1`

If more than one surviving incoming edge remains, current authority resolution MUST fail closed as ambiguity.

---

## 4. Acyclicity Rules

### 4.1 DAG Rule

The verifier delegation graph MUST be acyclic.

Any cycle invalidates current distributed authority for every node participating in the cycle.

### 4.2 Self-Delegation Rule

The system MUST reject:

`verifier_id == delegate_verifier_id`

for any delegation edge in the same authority graph.

### 4.3 Indirect Cycle Rule

The system MUST reject indirect cycles, including forms such as:
- `A -> B -> A`
- `A -> B -> C -> A`
- `A -> B -> C -> B`

Cycle detection MUST be semantic, not string-heuristic.

### 4.4 Canonical Node Identity Rule

Cycle detection and edge comparison MUST operate over canonical verifier node identities, not aliases, display labels, or transport-local ordering.

---

## 5. Delegation Depth Rules

### 5.1 Bounded Depth Rule

Delegation depth MUST be bounded.

Recommended initial invariant:

`max_delegation_depth = 8`

### 5.2 Overflow Rule

If a delegation chain exceeds the configured maximum depth, delegated authority resolution MUST fail closed.

### 5.3 Depth Interpretation Rule

Depth counts explicit authority hops, not merely registry lineage hops.

---

## 6. Scope Monotonicity Rules

### 6.1 Narrowing Rule

Delegated authority scope MUST be a subset of parent authority scope.

Conceptual rule:

`delegated_scope ⊆ parent_scope`

### 6.2 No Widening Rule

A delegate MUST NOT gain a broader authority class than its parent explicitly holds.

### 6.3 No Scope Resurrection Rule

A child delegate MUST NOT restore authority scope that was removed or forbidden higher in the chain.

### 6.4 Current vs Historical Rule

`historical-audit-only` authority MUST NOT delegate into current distributed authority.

---

## 7. Resolution Rules

### 7.1 Explicit Edge Rule

Delegation edges MUST be explicit in verifier authority data.

Similarity of names, shared namespace, or shared signer key MUST NOT imply delegation.

### 7.2 Deterministic Resolution Rule

Given the same authority graph inputs, delegation resolution MUST yield the same accepted authority graph.

### 7.3 Ambiguity Rule

If multiple possible parent chains can authorize a delegate and the selection is not uniquely determined, authority resolution MUST fail closed.

---

## 8. Failure Semantics

Recommended failure classes:
- `AUTHORITY_GRAPH_CYCLE`
- `AUTHORITY_GRAPH_SELF_DELEGATION`
- `AUTHORITY_GRAPH_DEPTH_EXCEEDED`
- `AUTHORITY_SCOPE_WIDENING`
- `AUTHORITY_GRAPH_AMBIGUOUS`

These failures:
- MUST NOT degrade to warnings for distributed trust
- MUST invalidate current distributed authority claims for the affected chain
- MAY preserve historical audit interpretation if separately allowed by higher-level rules

---

## 9. Parity Implications

Cross-node parity MUST treat authority graph mismatch as verifier-trust mismatch.

Examples:
- one node resolves a delegation cycle, another rejects it
- one node allows wider delegated scope, another narrows it correctly
- one node exceeds max depth, another does not

Recommended parity effect:
- authority graph mismatch => `PARITY_VERIFIER_MISMATCH`
- authority graph evidence missing => `PARITY_INSUFFICIENT_EVIDENCE`
- superseded but audit-valid authority chain => `PARITY_HISTORICAL_ONLY`

---

## 10. Threat Model Notes

This specification primarily mitigates:
- verifier authority loop attacks
- delegation abuse
- authority amplification through graph cycles
- scope resurrection through multi-hop delegation

It does not itself solve:
- reputation weighting
- quorum trust weighting
- consensus over current authority head

Those remain later-phase concerns.

---

## 11. Acceptance Criteria

11.1. THE System SHALL define verifier authority delegation as a directed acyclic graph, not a general graph
11.2. THE System SHALL reject self-delegation
11.3. THE System SHALL reject indirect delegation cycles
11.4. THE System SHALL define a maximum delegation depth and SHALL fail closed when it is exceeded
11.5. Delegated authority scope SHALL only narrow, never widen
11.6. `historical-audit-only` authority SHALL NOT delegate into current distributed authority
11.7. Ambiguous delegation chain resolution SHALL fail closed
11.8. Cross-node parity SHALL treat authority graph mismatch as verifier-trust mismatch
11.9. Current delegated authority SHALL have at most one surviving incoming parent edge after filtering, or resolution SHALL fail closed
11.10. Cycle detection SHALL operate on canonical verifier node identity, not alias strings or insertion order

---

## 12. Summary

Verifier delegation is safe only when:
- the graph is acyclic
- depth is bounded
- scope only narrows
- ambiguity fails closed

Without these constraints, valid-looking verifier authority can become self-sustaining and unsafe.
