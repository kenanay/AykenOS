# Verifier Authority Resolution Algorithm

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines the deterministic algorithm used to resolve verifier authority from:
- verifier trust registry snapshots
- verifier authority semantics
- verifier delegation graph constraints

Its job is to ensure that the same authority inputs always produce the same resolved authority interpretation.

Critical rule:

`multiple valid parent chains => fail closed`

Phase-12 prefers explicit uniqueness over hidden tie-breaking.

---

## 2. Problem Statement

Even with:
- valid receipts
- valid verifier registry lineage
- valid authority graph constraints

distributed trust can still drift if nodes resolve authority differently.

The most dangerous class is authority resolution ambiguity:
- two valid-looking parent chains
- different local parent selection
- implicit tie-breaking hidden in implementation

This document forbids such silent divergence.

---

## 3. Inputs

The authority resolution algorithm consumes:

- canonical verifier trust registry snapshot
- verifier authority semantics
- verifier authority delegation graph
- verifier registry lineage interpretation

Minimum logical input tuple:

`(verifier_registry_snapshot_hash, verifier_registry_epoch, root_verifier_ids, authority_nodes, delegation_edges, requested_authority_scope)`

---

## 4. Output Model

The algorithm MUST emit one of:
- resolved current authority
- resolved historical-only authority
- deterministic invalid / ambiguous failure

When authority resolution succeeds, the result MUST include:
- resolved authority chain
- `authority_chain_id`

Recommended result classes:
- `AUTHORITY_RESOLVED_ROOT`
- `AUTHORITY_RESOLVED_DELEGATED`
- `AUTHORITY_HISTORICAL_ONLY`
- `AUTHORITY_GRAPH_AMBIGUOUS`
- `AUTHORITY_GRAPH_CYCLE`
- `AUTHORITY_GRAPH_DEPTH_EXCEEDED`
- `AUTHORITY_SCOPE_WIDENING`
- `AUTHORITY_NO_VALID_CHAIN`

### 4.1 Authority Chain Identity

Successful authority resolution SHALL expose a canonical chain identity:

`authority_chain_id = "sha256:" + SHA256(JCS(authority_chain_representation))`

Where `authority_chain_representation` is an ordered canonical structure containing at least:
- canonical authority node identities from root to resolved verifier
- effective authority scope
- verifier registry snapshot identity

---

## 5. Normalization Rules

### 5.1 Root Authority Source

Phase-12 uses explicit root declaration.

Current root verifier authorities MUST come from the verifier trust registry root set.

Nodes with no incoming authority edge MUST NOT be treated as current roots unless explicitly listed in `root_verifier_ids`.

### 5.2 Pre-Resolution Normalization

Before resolution begins, the system MUST:
1. verify canonical registry snapshot hash
2. verify registry lineage acceptance rules
3. normalize authority nodes by canonical identity fields
4. normalize explicit root declarations by canonical identity fields
5. normalize delegation edges as explicit directed edges only

No transport-local ordering, insertion order, or hash-map iteration order may affect the result.

---

## 6. Resolution Algorithm

### 6.1 Step 1: Build Candidate Graph

Build a directed graph whose:
- nodes are canonical verifier authority nodes
- edges are explicit delegation edges

### 6.2 Step 2: Structural Validation

Reject immediately on:
- self-delegation
- direct or indirect cycles
- missing referenced node
- referenced root verifier missing from the canonical authority node set

### 6.3 Step 3: Scope Validation

Reject any edge whose delegated scope is not a subset of parent scope.

### 6.4 Step 4: Depth Validation

Reject any chain exceeding configured maximum delegation depth.

Depth is counted as explicit delegation hops from an explicit root authority, not raw node count.

### 6.5 Step 5: Root and Candidate Chain Enumeration

For a requested verifier authority, enumerate all candidate parent chains from the explicit root set that could justify current authority.

Implementations MAY use bounded DFS, reverse BFS, or equivalent graph traversal, provided the externally visible candidate-chain set is independent of traversal order.

### 6.6 Step 6: Historical Filtering

Remove chains that are:
- revoked as current authority
- superseded by lineage rules
- historical-only under current policy

### 6.7 Step 7: Uniqueness Check

If zero chains remain:
- emit `AUTHORITY_NO_VALID_CHAIN`

If exactly one chain remains:
- accept it as the resolved authority chain
- compute canonical `authority_chain_id`

If more than one chain remains:
- emit `AUTHORITY_GRAPH_AMBIGUOUS`

### 6.8 Step 8: Result Classification

Classify the surviving chain as:
- `AUTHORITY_RESOLVED_ROOT`
- `AUTHORITY_RESOLVED_DELEGATED`
- or `AUTHORITY_HISTORICAL_ONLY`

according to explicit lineage and scope semantics.

---

## 7. Parent Chain Selection Rule

### 7.1 Phase-12 Rule

Phase-12 does NOT allow hidden parent selection heuristics.

The delegate MUST resolve to exactly one effective parent chain after validation and filtering.

This is stronger than deterministic tie-breaking:
- one surviving chain => accept
- multiple surviving chains => ambiguity

### 7.2 Forbidden Tie-Breakers

The system MUST NOT use silent implementation-defined tie-breakers such as:
- lowest `verifier_id`
- lexicographically smallest parent
- first insertion order
- first parsed edge

unless a future versioned contract explicitly introduces such behavior.

### 7.3 Rationale

Fail-closed ambiguity is safer than deterministic but implicit authority choice.

---

## 8. Cycle Detection Rule

Cycle detection MUST be semantic over the normalized directed authority graph.

Cycle detection MUST operate on canonical verifier node identities.

The implementation MAY use standard graph algorithms such as:
- DFS back-edge detection
- Kahn topological elimination

But the externally visible behavior MUST be:
- deterministic
- fail-closed
- independent of iteration order

---

## 9. Determinism Invariants

Given the same:
- verifier registry snapshot
- authority node set
- delegation edge set
- requested authority scope

the resolver MUST produce the same result class and the same accepted authority chain.

If resolution succeeds, it MUST also produce the same `authority_chain_id`.

No resolver may claim distributed parity if authority resolution is implementation-dependent.

---

## 10. Parity Implications

Cross-node parity requires equality of:
- resolved authority result class
- resolved authority chain identity
- `authority_chain_id`
- effective authority scope

If two nodes resolve the same verifier through different valid-looking parent chains, parity MUST fail as:
- `PARITY_VERIFIER_MISMATCH`, or
- `PARITY_INSUFFICIENT_EVIDENCE`

depending on whether the mismatch is proven or ambiguous.

---

## 11. Threat Model Notes

This specification primarily mitigates:
- delegation fork attack
- hidden parent-chain selection drift
- authority resolution nondeterminism
- loop masking through implementation order

It does not itself solve:
- trust weighting
- verifier reputation
- quorum authority aggregation

Those remain later-phase concerns.

---

## 12. Acceptance Criteria

12.1. THE System SHALL define a deterministic verifier authority resolution algorithm
12.2. THE resolver SHALL normalize authority nodes and delegation edges before evaluation
12.3. Self-delegation, cycles, scope widening, and depth overflow SHALL fail closed
12.4. THE resolver SHALL enumerate candidate parent chains for delegated authority
12.5. A delegated verifier SHALL resolve to exactly one effective parent chain after filtering, or authority resolution SHALL fail closed
12.6. Silent tie-breakers for parent selection SHALL NOT be used in Phase-12
12.7. Current root verifier authorities SHALL come from an explicit verifier trust registry root set, not from missing parent edges alone
12.8. Successful authority resolution SHALL expose a canonical `authority_chain_id` for parity and audit comparison
12.9. Cross-node parity SHALL require equal resolved authority class and equal effective authority chain interpretation

---

## 13. Summary

Phase-12 trust cannot rely on “some parent chain looked fine”.

It must resolve verifier authority deterministically, uniquely, and fail-closed.

Without this, delegation graphs remain structurally valid but semantically unstable.
