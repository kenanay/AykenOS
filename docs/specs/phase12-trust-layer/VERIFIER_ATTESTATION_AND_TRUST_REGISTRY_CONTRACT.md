# Verifier Attestation and Trust Registry Contract

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `PROOF_BUNDLE_V2_SPEC.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines how a verifier proves its own distributed trust identity in Phase-12 and later phases.

It exists to keep three artifact classes separate:
- receipt = what decision was emitted
- verification context object = under which distributed context the decision was emitted
- verifier attestation = why the verifying node itself may be trusted as a distributed trust speaker

This contract is normative for shared receipt reuse, verifier identity trust, and future federated parity claims.

It does not redefine:
- `bundle_id`
- `trust_overlay_hash`
- `verdict_subject`
- `verification_context_id`

Critical rule:

`trusted proof != trusted verifier`

Authority scope and delegation semantics are defined separately in:

`VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`

---

## 2. Problem Statement

Signed receipts alone are insufficient for distributed trust.

A node may prove that:
- it emitted a receipt
- under a particular verdict subject
- under a particular verification context

That still does not prove that other nodes should treat the emitter as a trusted verifier.

Distributed trust therefore requires:
- explicit verifier identity
- explicit verifier key binding
- explicit verifier trust registry semantics
- explicit revocation and historical behavior

Without these surfaces, distributed trust collapses into receipt gossip.

---

## 3. Core Separation

The following surfaces MUST remain separate:

- `verdict_subject`
  identifies what was judged
- `verification_context_id`
  identifies under which distributed context the judgment is shareable
- verifier attestation and verifier trust registry semantics
  identify whether the verifying node is trusted to emit distributed trust evidence

No receipt signature alone may collapse these three surfaces into one.

---

## 4. Verifier Identity Schema

### 4.1 Required Fields

Distributed verifier identity MUST include at least:

```json
{
  "verifier_id": "node-b",
  "verifier_pubkey_id": "receipt-ed25519-key-2026-03-a",
  "verifier_registry_ref": "verifier-registry/main",
  "verifier_key_epoch": 5
}
```

### 4.2 Field Semantics

- `verifier_id`
  - stable verifier identity across key rotations
- `verifier_pubkey_id`
  - one concrete public key identifier used for receipt or attestation signing
- `verifier_registry_ref`
  - verifier trust namespace or registry lineage reference
- `verifier_key_epoch`
  - deterministic key-rotation lineage marker

### 4.3 Identity Rules

- `verifier_id` MUST remain stable across verifier key rotations
- `verifier_pubkey_id` MUST identify exactly one concrete key
- `verifier_key_epoch` MUST be monotonic for a verifier lineage
- verifier identity metadata MUST remain external to portable proof identity

---

## 5. Verifier Attestation Object

### 5.1 Purpose

The verifier attestation object binds verifier identity to a concrete signing key and contract surface.

### 5.2 Minimal Canonical Shape

```json
{
  "attestation_version": 1,
  "verifier_id": "node-b",
  "verifier_pubkey_id": "receipt-ed25519-key-2026-03-a",
  "verifier_registry_ref": "verifier-registry/main",
  "verifier_key_epoch": 5,
  "verifier_contract_version": "phase12-context-v1",
  "attestation_signature_algorithm": "ed25519",
  "attestation_signature": "base64:..."
}
```

### 5.3 Attestation Rules

- the attestation payload MUST be canonicalized before signing
- the attestation signature MUST be detached from the portable proof bundle
- the attestation object MUST bind verifier identity to the declared key and contract version
- the attestation object MUST be independently verifiable from receipt payloads

### 5.4 Detached Rule

Verifier attestation is a distributed trust artifact.

It MUST NOT mutate:
- `bundle_id`
- `trust_overlay_hash`
- `verification_context_id`

---

## 6. Verifier Trust Registry Snapshot

### 6.1 Purpose

The verifier trust registry is distinct from producer trust registry.

It answers:
- which verifier identities are trusted to emit distributed trust evidence
- which verifier keys are active, revoked, or historical

### 6.2 Minimal Registry Shape

```json
{
  "registry_format_version": 1,
  "verifier_registry_snapshot_hash": "<sha256-hex>",
  "root_verifier_ids": ["root-verifier-a"],
  "verifiers": [
    {
      "verifier_id": "node-b",
      "active_verifier_pubkey_ids": ["receipt-ed25519-key-2026-03-a"],
      "revoked_verifier_pubkey_ids": [],
      "historical_verifier_pubkey_ids": []
    }
  ],
  "public_keys": {
    "receipt-ed25519-key-2026-03-a": {
      "algorithm": "ed25519",
      "public_key": "base64:..."
    }
  }
}
```

### 6.3 Registry Rules

- verifier trust registry MUST be canonical and hashable
- verifier trust registry MUST be external to bundle payload
- verifier trust registry MUST be separate from producer trust registry, even if both are distributed together
- verifier key status MUST distinguish at least: `active`, `revoked`, `historical`
- current root verifier authority MUST be declared explicitly by the verifier trust registry
- a verifier with no incoming delegated authority edges MUST NOT be treated as a current root unless it is explicitly listed in `root_verifier_ids`
- root authority is granted by registry declaration, not inferred from missing parent edges alone

---

## 7. Canonical Hash Rule

### 7.1 Normative Formula

`verifier_registry_snapshot_hash = SHA256(JCS(verifier_registry_snapshot_without_hash))`

### 7.2 Verification Rule

Receiving nodes MUST:
1. parse the verifier registry snapshot
2. remove `verifier_registry_snapshot_hash`
3. canonicalize the remaining object
4. recompute the SHA-256 hash
5. compare recomputed hash against the declared value

Mismatch MUST fail closed.

### 7.3 Shared Implementation Rule

Verifier registry hashing SHOULD reuse the same canonicalization implementation surface as:
- receipt payload hashing
- verification context object hashing
- registry snapshot hashing

This reduces cross-node canonicalization drift.

---

## 8. Receipt Acceptance Rule

### 8.1 Non-Negotiable Rule

A signed receipt SHALL NOT be treated as shared distributed trust evidence unless:
- the receipt signature is valid
- the `verdict_subject` is valid
- the `verification_context_id` is valid and equal to the local distributed context
- the receipt signer is trusted under the verifier trust registry

### 8.2 Trust-of-Verifier Rule

Receipt signature validity alone is insufficient.

The verifier that emitted the receipt MUST itself be trusted under explicit verifier trust semantics.

### 8.3 Fail-Closed Rule

If verifier trust registry resolution fails, distributed receipt reuse MUST fail closed.

### 8.4 Historical Rule

Receipts whose signer verifier key was once valid but is no longer currently trusted MAY remain historical artifacts.

They MUST NOT automatically remain current distributed trust evidence.

---

## 9. Revocation and Historical Semantics

### 9.1 Verifier Key Revocation

When a verifier key is revoked:
- future distributed receipt trust under that key MUST fail closed
- previously emitted receipts MAY remain historical artifacts if their original context can still be reconstructed

### 9.2 Historical Classification

Recommended interpretation labels:
- `current`
- `historical_only`
- `revoked`

### 9.3 Epoch Semantics

`verifier_key_epoch` SHOULD be carried through distributed trust surfaces to simplify historical analysis and replay-safe classification.

### 9.4 No Silent Upgrade Rule

An old receipt signed by a historical verifier key MUST NOT be silently reclassified as current distributed trust evidence under a newer verifier key.

---

## 10. Cross-Node Parity Rule

Distributed parity claims require equality of:
- `verdict_subject`
- `verification_context_id`
- trusted verifier semantics

Minimum conceptual rule:

`same verdict_subject + same verification_context_id + same trusted verifier semantics => same distributed acceptance claim`

If any of the three differ, a node MUST NOT claim distributed parity.

Detailed parity mismatch classification is defined in:

`CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`

---

## 11. Threat Model Notes

This contract primarily mitigates:
- receipt amplification
- receipt laundering through untrusted verifier nodes
- verifier split-brain across distributed trust roots
- verifier key revocation confusion
- verifier-context mismatch hidden behind otherwise valid receipt signatures

It does not by itself solve:
- consensus
- global ordering
- receipt DAG federation
- remote verifier attestation transport protocol

Those remain future work.

---

## 12. Acceptance Criteria

12.1. THE System SHALL define a canonical verifier identity schema containing at least: `verifier_id`, `verifier_pubkey_id`, `verifier_registry_ref`, `verifier_key_epoch`
12.2. THE System SHALL define a verifier attestation object that binds verifier identity, signing key, and verifier contract version
12.3. THE System SHALL define a separate verifier trust registry snapshot surface
12.4. THE verifier trust registry SHALL be canonical and hashable
12.5. THE System SHALL compute `verifier_registry_snapshot_hash` as SHA-256 over canonical JSON excluding the declared hash field itself
12.6. THE verifier SHALL reject verifier trust registry snapshots whose declared and recomputed hash differ
12.7. A signed receipt SHALL NOT be treated as shared distributed trust evidence unless its signer verifier is trusted under the verifier trust registry
12.8. THE System SHALL preserve the distinction: `trusted proof != trusted verifier`
12.9. Revoked verifier keys SHALL NOT remain current distributed trust anchors
12.10. Historical receipts MAY remain audit-valid artifacts but SHALL NOT automatically remain current distributed trust evidence after verifier revocation or verifier trust-context change
12.11. Cross-node parity claims SHALL require equal `verdict_subject`, equal `verification_context_id`, and equal trusted verifier semantics
12.12. THE verifier trust registry SHALL declare current root verifier authorities explicitly
12.13. A verifier with no delegated parent SHALL NOT be treated as current root authority unless explicitly listed in the verifier trust registry root set

---

## 13. Summary

Phase-12 distributed trust requires three distinct artifact classes:
- proof artifact
- context artifact
- verifier-trust artifact

This contract defines the third surface.

Without it, signed receipts remain locally meaningful but globally ambiguous.
