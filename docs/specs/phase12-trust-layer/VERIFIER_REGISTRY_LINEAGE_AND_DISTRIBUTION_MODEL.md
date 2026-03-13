# Verifier Registry Lineage and Distribution Model

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines how verifier trust registry snapshots are versioned, distributed, and compared across nodes.

Its job is to prevent distributed verifier trust from collapsing under:
- registry split-brain
- registry rollback
- lineage ambiguity
- content drift hidden behind valid local receipts

This document is normative for distributed verifier-trust interpretation.

It does not define:
- producer registry transport
- consensus
- remote fetch protocol details

---

## 2. Problem Statement

Even when:
- receipts are signed
- verification context is explicit
- verifier authority semantics are defined

distributed trust can still fail if nodes do not agree on verifier registry lineage.

The critical failure mode is:

`same receipt + different verifier registry lineage => different trust authority interpretation`

Without explicit lineage rules, nodes may each be locally correct and still produce incompatible distributed trust claims.

---

## 3. Canonical Snapshot Model

### 3.1 Required Fields

Verifier trust registry snapshots MUST contain at least:

```json
{
  "registry_format_version": 1,
  "verifier_registry_snapshot_hash": "sha256:<hex>",
  "verifier_registry_parent_hash": "sha256:<hex>",
  "verifier_registry_epoch": 12,
  "registry_scope": "verifier-trust/main"
}
```

### 3.2 Field Semantics

- `verifier_registry_snapshot_hash`
  - canonical identity of the current snapshot
- `verifier_registry_parent_hash`
  - canonical identity of the immediate parent snapshot
- `verifier_registry_epoch`
  - monotonic integer for registry lineage ordering
- `registry_scope`
  - namespace for the verifier trust registry lineage

### 3.3 Genesis Rule

The first snapshot in a lineage MAY use:
- `verifier_registry_parent_hash = "genesis"`

Any other special value MUST be explicitly specified.

---

## 4. Hash and Canonicalization Rules

### 4.1 Normative Formula

`verifier_registry_snapshot_hash = "sha256:" + SHA256(JCS(snapshot_without_verifier_registry_snapshot_hash))`

### 4.2 Exclusion Rule

Only `verifier_registry_snapshot_hash` itself may be excluded from the hash input.

### 4.3 Verification Rule

Receiving nodes MUST:
1. parse the snapshot
2. remove `verifier_registry_snapshot_hash`
3. canonicalize the remaining object
4. recompute the hash
5. compare recomputed identity with declared identity

Mismatch MUST fail closed.

---

## 5. Lineage Rules

### 5.1 Monotonic Epoch Rule

Within one `registry_scope`, `verifier_registry_epoch` MUST be monotonic.

### 5.2 Parent Link Rule

For non-genesis snapshots, `verifier_registry_parent_hash` MUST identify the exact previous snapshot in the accepted lineage.

### 5.3 No Silent Fork Rule

If two snapshots share:
- the same `registry_scope`
- the same `verifier_registry_epoch`

but have different `verifier_registry_snapshot_hash`, the system MUST treat this as forked lineage, not benign duplication.

### 5.4 No Silent Rollback Rule

If a node has already accepted a newer snapshot in the same lineage, it MUST NOT silently downgrade to an older snapshot without explicit historical-mode handling.

---

## 6. Distribution Rules

### 6.1 Content-Addressed Distribution

Verifier trust registry snapshots SHOULD be transported as:
- full inline objects, or
- content-addressed references that resolve to exact canonical snapshot bytes

### 6.2 Resolution Rule

If a reference is used, it MUST resolve to canonical bytes whose recomputed hash equals the declared `verifier_registry_snapshot_hash`.

### 6.3 Availability Rule

Shared distributed verifier-trust claims MUST NOT rely on a verifier registry snapshot that cannot be resolved or reconstructed.

### 6.4 External Input Rule

Verifier registry snapshots remain external trust inputs.

They MUST NOT be silently imported from portable proof bundle contents.

---

## 7. Split-Brain and Rollback Semantics

### 7.1 Split-Brain

The following constitutes split-brain for one `registry_scope`:
- two different accepted snapshot hashes at the same epoch
- incompatible parent linkage
- incompatible authority interpretation for the same verifier identity under supposedly current snapshots

### 7.2 Rollback

The following constitutes rollback:
- current node accepts an older epoch as if it were current
- current node accepts a snapshot whose lineage contradicts a newer already accepted snapshot

### 7.3 Required Interpretation

Split-brain and rollback MUST NOT be downgraded to operator warnings for distributed trust claims.

They MUST invalidate current shared verifier authority claims unless explicitly reclassified as historical-only.

---

## 8. Historical Semantics

### 8.1 Historical Registry Use

Older verifier registry snapshots MAY remain valid for:
- audit reconstruction
- historical-only receipt interpretation

### 8.2 Current vs Historical Rule

An older snapshot MUST NOT automatically remain a current verifier authority source once a newer accepted lineage supersedes it.

### 8.3 Historical Classification

When an older snapshot is still used for audit interpretation, the resulting distributed classification SHOULD be:
- `historical_only`

and MUST NOT be treated as current distributed authority.

---

## 9. Parity Implications

Verifier trust parity requires more than equal receipt signatures.

Distributed parity claims MUST assume verifier-trust equality only when:
- `verifier_registry_snapshot_hash` matches
- lineage interpretation is compatible
- authority scope interpretation is compatible

If verifier registry lineage differs, the parity layer MUST NOT emit `PARITY_MATCH`.

Recommended classifications:
- lineage fork => `PARITY_VERIFIER_MISMATCH`
- rollback ambiguity => `PARITY_INSUFFICIENT_EVIDENCE`
- superseded but audit-valid lineage => `PARITY_HISTORICAL_ONLY`

---

## 10. Threat Model Notes

This model primarily mitigates:
- cross-node verifier registry split-brain
- verifier registry rollback
- stale verifier authority resurrection
- content-address ambiguity in distributed verifier trust

It does not itself solve:
- consensus on current registry head
- network transport authenticity
- global total ordering

Those remain later-phase concerns.

---

## 11. Acceptance Criteria

11.1. THE System SHALL define a canonical verifier registry snapshot lineage model
11.2. THE System SHALL define `verifier_registry_snapshot_hash`, `verifier_registry_parent_hash`, and `verifier_registry_epoch` semantics together
11.3. THE verifier SHALL reject snapshots whose declared and recomputed `verifier_registry_snapshot_hash` differ
11.4. The verifier SHALL treat same-scope same-epoch different-hash snapshots as lineage fork, not benign variation
11.5. THE verifier SHALL NOT silently downgrade current verifier authority interpretation to an older snapshot in the same lineage
11.6. Shared distributed trust claims SHALL require resolvable and current-enough verifier registry lineage
11.7. Older verifier registry snapshots MAY remain historical audit artifacts but SHALL NOT automatically remain current distributed authority sources
11.8. Cross-node parity claims SHALL treat incompatible verifier registry lineage as non-parity

---

## 12. Summary

Verifier trust does not depend only on who signed a receipt.

It also depends on which verifier registry lineage authorized that signer.

Without explicit lineage and distribution rules, valid local trust can fragment into incompatible distributed truth.
