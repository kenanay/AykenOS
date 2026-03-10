# Verifier Authority Semantics and Delegation Contract

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines verifier authority semantics for distributed trust.

It exists to answer four questions:
- when a verifier is trusted as a distributed trust speaker
- what authority scope that verifier actually holds
- whether authority may be delegated
- how verifier authority lineage, revocation, and ambiguity are handled

This contract is normative for distributed receipt reuse and future verifier federation.

Critical rule:

`receipt signature validity != verifier authority`

---

## 2. Problem Statement

A signed receipt may be:
- structurally valid
- cryptographically valid
- context-bound

and still come from a verifier that has no authority to speak for distributed trust.

The dangerous failure mode is not broken cryptography.

It is authority confusion:
- untrusted verifier treated as trusted
- historical verifier treated as current authority
- delegated verifier treated as unconstrained authority
- ambiguous verifier identity treated as authoritative

---

## 3. Core Separation

The following surfaces MUST remain distinct:

- verifier identity
- verifier attestation
- verifier authority semantics
- verifier trust registry membership

Critical rules:

- `trusted verifier != any verifier with a valid key`
- `delegated verifier != root verifier`
- `historical verifier != current verifier authority`
- `ambiguous verifier mapping => fail closed`

---

## 4. Verifier Authority Model

### 4.1 Required Authority Fields

Verifier authority semantics MUST include at least:

```json
{
  "verifier_id": "node-b",
  "verifier_pubkey_id": "receipt-ed25519-key-2026-03-a",
  "verifier_registry_ref": "verifier-registry/main",
  "verifier_registry_epoch": 12,
  "verifier_registry_parent_hash": "sha256:<hex>",
  "authority_scope": "distributed-receipt-issuer",
  "delegation_mode": "default-deny"
}
```

### 4.2 Field Semantics

- `verifier_registry_epoch`
  - monotonic authority epoch for the verifier trust registry
- `verifier_registry_parent_hash`
  - previous authority snapshot identity for lineage tracking
- `authority_scope`
  - explicit authority class granted to the verifier
- `delegation_mode`
  - whether downstream delegation is forbidden or explicitly bounded

### 4.3 Default Authority Rule

Verifier authority MUST be explicit.

No verifier gains distributed trust authority merely by appearing in a registry.

---

## 5. Authority Scope Semantics

### 5.1 Minimum Scope Set

Recommended minimum scope values:

- `distributed-receipt-issuer`
- `parity-reporter`
- `context-distributor`
- `historical-audit-only`

### 5.2 Scope Rule

Verifier authority MUST be least-privilege.

If a verifier is trusted only to emit local audit or historical artifacts, it MUST NOT be treated as a current distributed acceptance speaker.

### 5.3 No Scope Inflation Rule

A verifier MUST NOT be interpreted with a broader authority scope than the registry explicitly grants.

Missing scope MUST fail closed.

---

## 6. Delegation Semantics

### 6.1 Default-Deny Rule

Delegation of verifier authority is forbidden unless explicitly declared.

Normative default:

`delegation_mode = default-deny`

Additional graph constraints for delegation are defined in:

`VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`

### 6.2 Explicit Delegation Rule

If delegation is permitted, the registry MUST define explicit bounded semantics for:
- delegator verifier identity
- delegate verifier identity
- delegated scope
- delegation epoch
- delegation expiry or revocation behavior

### 6.3 No Implicit Delegation Rule

The following MUST NOT imply delegation by themselves:
- shared namespace
- similar `verifier_id`
- matching `verifier_contract_version`
- matching `verification_context_id`

### 6.4 Delegation Narrowing Rule

A delegate MUST NOT obtain authority broader than its delegator's explicitly declared delegated scope.

---

## 7. Identity Shadowing and Ambiguity Rules

### 7.1 Identity Shadowing

If two distinct public keys may plausibly resolve to the same verifier authority identity, the system MUST fail closed.

### 7.2 Ambiguous Mapping Rule

The verifier trust registry MUST reject ambiguous mapping between:
- `verifier_id`
- `verifier_pubkey_id`
- authority scope

### 7.3 No Fuzzy Identity Rule

String similarity, alias heuristics, or transport-local identity hints MUST NOT influence verifier authority resolution.

---

## 8. Revocation and Lineage Semantics

### 8.1 Epoch Rule

`verifier_registry_epoch` MUST be monotonic.

### 8.2 Parent Hash Rule

`verifier_registry_parent_hash` SHOULD bind verifier registry lineage to simplify split-brain and rollback detection.

### 8.3 Historical Resurrection Rule

A verifier key or authority entry that has moved to historical or revoked state MUST NOT be silently reclassified as current authority.

### 8.4 Rollback Rule

If a node receives an older verifier registry snapshot that conflicts with a newer known lineage, the node MUST NOT silently downgrade authority interpretation.

Detailed verifier registry lineage and distribution rules are defined in:

`VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`

---

## 9. Distributed Receipt Acceptance Rule

A receipt may be treated as shared distributed trust evidence only when all of the following hold:

- receipt signature is valid
- `verdict_subject` is valid
- `verification_context_id` matches local distributed context
- signer verifier is currently trusted
- signer verifier authority scope permits distributed receipt issuance
- verifier authority lineage is not revoked, shadowed, or ambiguously mapped

Failure of any condition MUST fail closed.

---

## 10. Threat Model Notes

This contract primarily mitigates:
- verifier authority capture
- verifier identity shadowing
- delegation abuse
- historical trust resurrection
- cross-registry authority split-brain

It does not by itself solve:
- consensus
- verifier reputation weighting
- receipt DAG federation

Those remain future work.

---

## 11. Acceptance Criteria

11.1. THE System SHALL define verifier authority semantics distinct from receipt signature validity
11.2. THE System SHALL define `verifier_registry_epoch` and `verifier_registry_parent_hash` semantics for verifier trust lineage
11.3. THE System SHALL define explicit verifier authority scopes
11.4. Missing or ambiguous verifier authority scope SHALL fail closed
11.5. Delegation SHALL default to deny unless explicitly declared
11.6. Delegated verifier authority SHALL be explicitly bounded and SHALL NOT exceed declared delegated scope
11.7. Ambiguous verifier identity or key mapping SHALL fail closed
11.8. Historical or revoked verifier authority SHALL NOT be silently upgraded to current distributed authority
11.9. Shared distributed receipt acceptance SHALL require both trusted verifier identity and trusted verifier authority scope

---

## 12. Summary

Phase-12 distributed trust needs more than:
- valid proof
- valid receipt
- valid context

It also needs correct authority semantics for the verifier that emits the receipt.

Without explicit authority scope, lineage, and default-deny delegation, a system can produce valid-looking lies.
