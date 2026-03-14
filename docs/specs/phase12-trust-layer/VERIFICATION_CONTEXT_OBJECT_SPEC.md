# Verification Context Object Specification

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `PROOF_BUNDLE_V2_SPEC.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines the canonical object that materializes distributed verification context in Phase-12.

It exists to make `verification_context_id` concrete, portable, and recomputable across nodes.

This specification defines:
- the canonical field schema
- self-hash rules
- content-addressed distribution rules
- optional epoch and historical semantics
- parity implications for distributed receipt reuse

Transport of this object and its referenced trust material is defined separately in:

`VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`

This specification does not define:
- `bundle_id`
- `trust_overlay_hash`
- receipt schema
- audit event schema
- wire-protocol framing

---

## 2. Relationship to Verdict Subject

The following distinction is mandatory:

`verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`

`verification_context_id = distributed interpretation context identity`

The verification context object does not replace `verdict_subject`.

Instead:
- `verdict_subject` identifies what was judged
- the verification context object identifies under which distributed rules that judgment may be shared

Distributed trust claims require both surfaces.

Trusted distributed receipt reuse additionally requires verifier-trust semantics defined separately in:

`VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`

---

## 3. Canonical Object Schema

### 3.1 Required Fields

The canonical verification context object MUST contain at least:

```json
{
  "context_version": 1,
  "verification_context_id": "sha256:<hex>",
  "policy_hash": "<sha256-hex>",
  "registry_snapshot_hash": "<sha256-hex>",
  "verifier_contract_version": "phase12-context-v1",
  "context_rules_hash": "<sha256-hex>"
}
```

### 3.2 Field Rules

- `context_version`
  - schema version of the context object
  - initial value: `1`

- `verification_context_id`
  - content identity of the canonical object
  - MUST use `sha256:<64-lowercase-hex>` form

- `policy_hash`
  - canonical hash of the verifier-local policy snapshot
  - MUST be 64 lowercase hex characters without prefix

- `registry_snapshot_hash`
  - canonical hash of the verifier-local registry snapshot
  - MUST be 64 lowercase hex characters without prefix

- `verifier_contract_version`
  - version of the distributed verification contract semantics
  - MUST be explicit and MUST NOT be inferred implicitly from binary version alone

- `context_rules_hash`
  - canonical hash of the distributed interpretation rules object
  - MUST be 64 lowercase hex characters without prefix

### 3.3 Optional Fields

Optional fields MAY include:

```json
{
  "context_epoch": 5,
  "historical_cutoff_utc": "2026-03-08T12:00:00Z",
  "policy_snapshot_ref": "cas:sha256:<hex>",
  "registry_snapshot_ref": "cas:sha256:<hex>",
  "time_semantics_mode": "historical-aware"
}
```

### 3.4 Optional Field Semantics

- `context_epoch`
  - optional monotonic integer for distributed context lineage

- `historical_cutoff_utc`
  - optional timestamp for historical-only interpretation

- `policy_snapshot_ref`
  - optional content-addressed reference to exact policy bytes

- `registry_snapshot_ref`
  - optional content-addressed reference to exact registry bytes

- `time_semantics_mode`
  - optional explicit mode for receipt aging and historical classification

---

## 4. Hash and Canonicalization Rules

### 4.1 Canonicalization

The context object MUST be canonicalized using RFC 8785 JCS semantics.

The verifier ecosystem SHOULD reuse the same canonical implementation surface as the core verifier canonical module to avoid cross-library drift.

### 4.2 Normative Hash Formula

`verification_context_id = "sha256:" + SHA256(JCS(context_object_without_verification_context_id))`

### 4.3 Exclusion Rule

`verification_context_id` MUST be excluded from its own hash computation.

No other required field may be excluded.

### 4.4 Verification Rule

When a context object is received, the verifier MUST:
1. parse the object
2. remove `verification_context_id`
3. canonicalize the remaining object
4. recompute the SHA-256 hash
5. compare recomputed identity to the declared `verification_context_id`

Mismatch MUST fail closed.

---

## 5. Context Rules Object

`context_rules_hash` MUST be derived from a separate canonical rules object.

Recommended minimal rules object:

```json
{
  "rules_version": 1,
  "policy_import_mode": "external-only",
  "registry_import_mode": "external-only",
  "context_mismatch_mode": "fail-closed",
  "historical_receipt_mode": "historical-only",
  "receipt_acceptance_mode": "context-bound-only"
}
```

This object MUST be canonicalized and hashed deterministically.

---

## 6. Content-Addressed Distribution

### 6.1 Inline or Reference

A distributed surface MAY carry:
- the full context object inline, or
- a content-addressed reference to it

### 6.2 Reference Rule

If a reference is used, it MUST resolve to the exact canonical bytes that produce the declared `verification_context_id`.

### 6.3 Resolution Failure

A receiving node MUST reject distributed trust claims if:
- the reference cannot be resolved
- the object does not parse
- the recomputed `verification_context_id` mismatches the declared value

### 6.4 External Input Rule

The context object may describe policy and registry inputs, but it MUST NOT override the rule that policy and registry remain external.

The bundle itself remains non-authoritative for distributed trust context.

---

## 7. Epoch and Historical Semantics

### 7.1 Epoch Purpose

`context_epoch` is optional for local verification but strongly recommended for distributed deployments.

Its role is to make major trust-context changes legible across nodes.

### 7.2 Historical Rule

A receipt that was valid under an older:
- `policy_hash`
- `registry_snapshot_hash`
- or `context_epoch`

MAY remain a valid historical artifact.

It MUST NOT automatically remain current distributed acceptance evidence.

### 7.3 Recommended Interpretation

Such receipts SHOULD be classified as:
- `historical_only`

rather than current acceptance evidence.

---

## 8. Cross-Node Parity Implications

Cross-node parity requires equality of:
- `verdict_subject`
- `verification_context_id`

If optional context fields are present, they MUST remain semantically consistent with the recomputed canonical object.

Parity surfaces SHOULD therefore carry at least:
- `bundle_id`
- `trust_overlay_hash`
- `policy_hash`
- `registry_snapshot_hash`
- `verification_context_id`
- optionally `context_epoch`

No node may claim distributed parity using `bundle_id` alone.

---

## 9. Acceptance Criteria

9.1. THE System SHALL define a canonical verification context object schema
9.2. THE canonical object SHALL contain at least: `context_version`, `verification_context_id`, `policy_hash`, `registry_snapshot_hash`, `verifier_contract_version`, `context_rules_hash`
9.3. THE System SHALL compute `verification_context_id` as a SHA-256 hash over canonical JSON excluding the `verification_context_id` field itself
9.4. THE verifier SHALL reject context objects whose declared and recomputed `verification_context_id` differ
9.5. THE System SHALL define `context_rules_hash` over an explicit canonical context-rules object
9.6. THE System SHALL allow content-addressed transport of the verification context object
9.7. THE verifier SHALL reject unresolved or mismatched context references
9.8. Optional `context_epoch` support SHOULD be provided for distributed historical interpretation
9.9. Cross-node parity claims SHALL require equal `verification_context_id` in addition to equal `verdict_subject`

---

## 10. Summary

The verification context distribution contract explains why context matters.

This object specification explains what the context is.

Without a canonical context object, `verification_context_id` is only an idea.

With it, distributed trust context becomes a transportable and recomputable artifact.
