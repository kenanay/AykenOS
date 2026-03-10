# Verification Context Distribution Contract

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `PROOF_BUNDLE_V2_SPEC.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines how verification context is identified, transported, and interpreted across nodes in Phase-12 and beyond.

Its job is to prevent a distributed verifier from confusing:
- the proof object being evaluated
- the trust context under which it was evaluated
- the historical artifact that records that evaluation

This contract is normative for future distributed receipt exchange, cross-node parity, and `proofd`-level trust transport.

Portable carriage of the context material referenced by this contract is defined separately in:

`VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`

It does not redefine:
- `bundle_id`
- `trust_overlay_hash`
- `verdict_subject`
- portable bundle identity semantics

It adds a separate distributed context identity for shared trust interpretation.

---

## 2. Problem Statement

Phase-12 already guarantees deterministic local verification under explicit external inputs:

`same bundle_id + same trust_overlay_hash + same policy_hash + same registry_snapshot_hash => same verdict`

This is necessary but not sufficient for distributed trust.

Across nodes, a receipt can be valid and still be misinterpreted if:
- the receiving node does not know which policy snapshot was used
- the receiving node does not know which registry snapshot was used
- the receiving node silently substitutes a different verifier contract or mismatch rule set
- the receiving node treats a historical receipt as current acceptance evidence

Distributed trust therefore needs an explicit context identity in addition to the proof subject.

---

## 3. Core Separation

### 3.1 Verdict Subject

The verdict subject remains unchanged:

`verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`

This tuple identifies the proof and the immediate trust inputs used to derive the verdict.

### 3.2 Verification Context

Distributed trust adds a second identity:

`verification_context_id`

This identity names the distributed acceptance context under which receipts, audit events, and parity claims may be shared.

### 3.3 Non-Negotiable Rule

`verdict_subject != verification_context_id`

The subject identifies what was judged.
The context identifies under which distributed rules that judgment may be shared or reused.

The verifier MUST NOT collapse these two identities into one field.

---

## 4. Context Identity Model

### 4.1 Conceptual Formula

Conceptually:

`verification_context_id = H(policy_hash || registry_snapshot_hash || verifier_contract_version || context_rules_hash)`

### 4.2 Normative Canonical Formula

The normative computation MUST be:

`verification_context_id = SHA256(JCS(verification_context_object_without_verification_context_id))`

where the canonical object includes at least:
- `policy_hash`
- `registry_snapshot_hash`
- `verifier_contract_version`
- `context_rules_hash`

Rationale:
- the compact formula defines the conceptual dependency set
- the canonical object removes delimiter ambiguity and preserves deterministic hashing

### 4.3 Context Rules Hash

`context_rules_hash` MUST identify the verifier rules that control distributed interpretation.

At minimum it MUST cover:
- policy import mode
- registry import mode
- context mismatch behavior
- historical receipt handling behavior
- receipt acceptance mode

Recommended conceptual rule:

`context_rules_hash = SHA256(JCS(context_rules_object))`

### 4.4 Verifier Contract Version

`verifier_contract_version` identifies the distributed interpretation contract, not the portable proof identity.

It MUST be versioned independently from:
- `bundle_id`
- `policy_hash`
- `registry_snapshot_hash`

Examples:
- `phase12-context-v1`
- `proof-verifier-context/1`

The verifier binary version alone is not sufficient unless the contract version is explicitly bound.

---

## 5. Context Object Schema

The distributed context object MUST be canonical, hashable, and externally supplied.

The canonical field-level schema is defined in:

`VERIFICATION_CONTEXT_OBJECT_SPEC.md`

Recommended minimal object:

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

Optional extensions MAY include:
- `context_epoch`
- `historical_cutoff_utc`
- `policy_snapshot_ref`
- `registry_snapshot_ref`
- `time_semantics_mode`

Design rule:
- optional fields MAY enrich auditability
- required fields MUST remain sufficient for deterministic distributed comparison

---

## 6. Context Binding Rules

### 6.1 Receipt Interpretation Rule

A verification receipt is not standalone distributed trust evidence.

It is meaningful only together with:
- the receipt payload
- the signed receipt binding
- the verification context under which it was issued

### 6.2 Distributed Trust Rule

A receipt SHALL NOT be treated as shared distributed trust evidence unless its verification context is explicitly present, hash-bound, and equal to the verifier-local acceptance context.

Shared distributed trust evidence also requires trusted verifier semantics as defined in:

`VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`

### 6.3 Transport Rule

Policy and registry inputs MUST remain external.

Therefore:
- the bundle MUST NOT silently import policy
- the bundle MUST NOT silently import registry
- a receiving node MUST NOT infer distributed trust context from bundle contents alone

### 6.4 Availability Rule

`verification_context_id` alone is insufficient if the referenced context object is unavailable.

For distributed acceptance claims, a node MUST have either:
- the full context object, or
- a content-addressed reference that resolves to the exact same canonical object

### 6.5 Binding Surface Rule

Future distributed trust surfaces MUST carry context binding explicitly.

This includes:
- exchanged receipts
- exchanged audit events
- cross-node parity reports
- `proofd` transport responses

### 6.6 Local Artifact Rule

Verifier-local receipts and audit ledgers produced before explicit context transport support remain valid local artifacts.

However:
- they MUST NOT be treated as shared distributed trust evidence by default
- they MAY be retained as historical or local audit artifacts

This rule preserves current P12-11 and P12-12 local completion status without overstating distributed readiness.

---

## 7. Context Mismatch Semantics

### 7.1 Fail-Closed Rule

Context mismatch is not a warning.

It MUST fail closed for distributed acceptance.

### 7.2 Missing Context

If a node receives a receipt intended for shared distributed trust, but explicit context is missing, the node MUST reject the distributed trust claim.

Recommended interpretation:
- distributed claim status: `INVALID`

### 7.3 Unequal Context

If:
- `verification_context_id` differs, or
- the recomputed local context differs from the carried context, or
- the referenced context object resolves to different canonical bytes

then the node MUST reject the distributed trust claim.

Recommended interpretation:
- distributed claim status: `INVALID`

### 7.4 Historical-Only Classification

`historical_only` is not a verification verdict.

It is an interpretation state for receipts or audit artifacts that:
- were valid under their original context
- are still useful as historical evidence
- are not valid proof of current acceptance under the receiver's local context

Examples:
- receipt issued before policy tightening
- receipt issued before key revocation
- receipt issued under a superseded registry snapshot

### 7.5 Verdict Preservation Rule

Existing verifier verdicts keep their current meanings:
- `INVALID`
- `UNTRUSTED`
- `REJECTED_BY_POLICY`
- `TRUSTED`

Context mismatch MUST NOT be re-labeled as `UNTRUSTED`.

Reason:
- `UNTRUSTED` is a trust-set result under the same context
- context mismatch is a distributed interpretation failure across contexts

---

## 8. Cross-Node Parity Contract

Cross-node parity claims are valid only when all of the following are equal:
- `bundle_id`
- `trust_overlay_hash`
- `policy_hash`
- `registry_snapshot_hash`
- `verification_context_id`

Therefore:

`same verdict_subject + same verification_context_id => same distributed acceptance claim`

No weaker parity claim is acceptable.

Detailed mismatch classification semantics for cross-node parity are defined in:

`CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`

In particular:
- same `bundle_id` is insufficient
- same `verdict_subject` without same `verification_context_id` is insufficient

---

## 9. Acceptance Criteria

### 9.1 Context Object

9.1.1. THE System SHALL define `verification_context_id` as a distributed context identity distinct from `verdict_subject`
9.1.2. THE System SHALL compute `verification_context_id` from at least: `policy_hash`, `registry_snapshot_hash`, `verifier_contract_version`, and `context_rules_hash`
9.1.3. THE normative hash computation SHALL use deterministic canonical JSON and SHA-256
9.1.4. THE System SHALL define `context_rules_hash` as a hash over explicit distributed interpretation rules

### 9.2 Binding Rules

9.2.1. A receipt SHALL NOT be sufficient for shared distributed trust without explicit context binding
9.2.2. Policy and registry inputs SHALL remain external and SHALL NOT be silently imported from bundle contents
9.2.3. Distributed receipt, audit, and parity surfaces SHALL carry explicit context binding or a content-addressed equivalent
9.2.4. A receiving node SHALL reject shared trust claims if the referenced verification context object is unavailable

### 9.3 Mismatch Semantics

9.3.1. Verification context mismatch SHALL fail closed
9.3.2. Context mismatch SHALL NOT degrade to warning-only behavior
9.3.3. Context mismatch SHALL NOT be re-labeled as `UNTRUSTED`
9.3.4. Historical receipts MAY be retained as audit artifacts but SHALL NOT be treated as current distributed acceptance proof

### 9.4 Distributed Parity

9.4.1. Cross-node parity claims SHALL require equal `verification_context_id` in addition to equal `verdict_subject`
9.4.2. A receipt exchanged across nodes SHALL be interpreted under the verifier-local context only if the carried context matches the local acceptance context exactly

---

## 10. Phase Mapping

### Phase-12B

This contract is informative for local verifier hardening.

Local receipt and audit features MAY exist before full distributed context transport is implemented.

### Phase-12C

This contract becomes normative for:
- bundle exchange
- cross-node parity
- `proofd` trust transport

### Phase-13+

This contract becomes foundational for:
- receipt DAG interpretation
- distributed audit federation
- shared trust graph construction

---

## 11. Summary

Phase-12 already knows how to verify a proof.

The next distributed problem is not signature math.
It is context identity.

The key architectural rule is:

`verdict_subject identifies the decision object`

while:

`verification_context_id identifies the distributed interpretation context`

If these two remain separate, explicit, and hash-bound, AykenOS can move into distributed verification without faking consistency.
