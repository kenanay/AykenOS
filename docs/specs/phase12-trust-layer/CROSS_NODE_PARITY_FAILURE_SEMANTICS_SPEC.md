# Cross-Node Parity Failure Semantics Specification

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `PROOF_BUNDLE_V2_SPEC.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `CROSS_NODE_PARITY_HARDENING_CHECKLIST.md`, `tasks.md`

---

## 1. Purpose

This document defines the normative failure semantics for cross-node parity in Phase-12.

Its job is to make distributed disagreement explicit and deterministic.

It exists to answer:
- when two nodes may claim parity
- when two nodes must reject parity
- how mismatch classes are labeled
- which mismatch classes are current trust failures versus historical-only divergence

This document is normative for future:
- `ci-gate-cross-node-parity`
- distributed receipt comparison
- `proofd` parity responses
- failure matrix reporting

It does not redefine:
- local verification verdict semantics
- portable proof identity
- verification context object schema
- verifier trust registry schema

---

## 2. Problem Statement

Two nodes may each be locally correct and still fail distributed parity.

Examples:
- same `bundle_id`, different `verification_context_id`
- same context, different verifier trust semantics
- same subject and context, one node uses revoked registry state
- same subject and context, one node reports `historical_only`

Without explicit failure semantics, distributed systems drift into:
- warning-only acceptance
- ambiguous operator reporting
- incorrect trust reuse
- false “same proof, same result” claims

Phase-12 therefore requires parity failure to be classified, not hand-waved.

---

## 3. Core Separation

Three surfaces remain distinct:

- local verification verdict
- distributed parity status
- historical interpretation status

Critical rules:

- parity failure MUST NOT be silently collapsed into a local verifier verdict
- `historical_only` is not a verifier verdict
- parity mismatch MUST NOT be re-labeled as `UNTRUSTED`

---

## 4. Inputs to Parity Comparison

Cross-node parity comparison MUST consider at least:

- `verdict_subject`
- `verification_context_id`
- trusted verifier semantics
- local verification verdict

The minimal parity input tuple is:

`(verdict_subject, verification_context_id, verifier_trust_semantics, local_verdict)`

Where:
- `verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`
- `verifier_trust_semantics` means the effective verifier-trust interpretation under the verifier trust registry, attestation contract, and current revocation state

Authority scope, delegation semantics, verifier registry lineage, and authority graph validity are part of `verifier_trust_semantics`.

When delegated verifier authority is in scope, `verifier_trust_semantics` SHALL expose canonical `authority_chain_id`.

No weaker tuple is acceptable for parity claims.

---

## 5. Normative Parity Status Set

Cross-node parity MUST classify outcomes using a status distinct from local verification verdicts.

Minimum status set:

- `PARITY_MATCH`
- `PARITY_SUBJECT_MISMATCH`
- `PARITY_CONTEXT_MISMATCH`
- `PARITY_VERIFIER_MISMATCH`
- `PARITY_VERDICT_MISMATCH`
- `PARITY_HISTORICAL_ONLY`
- `PARITY_INSUFFICIENT_EVIDENCE`

### 5.1 `PARITY_MATCH`

Use only when:
- `verdict_subject` is equal
- `verification_context_id` is equal
- trusted verifier semantics are equal
- local verification verdict is equal
- delegated `authority_chain_id` is equal when present

### 5.2 `PARITY_SUBJECT_MISMATCH`

Use when:
- `bundle_id` differs, or
- `trust_overlay_hash` differs, or
- `policy_hash` differs, or
- `registry_snapshot_hash` differs

### 5.3 `PARITY_CONTEXT_MISMATCH`

Use when:
- `verification_context_id` differs, or
- one node cannot supply the referenced context object, or
- recomputed and declared context identity differ on one side

### 5.4 `PARITY_VERIFIER_MISMATCH`

Use when:
- verifier trust registry semantics differ, or
- verifier attestation validity differs, or
- verifier signer is trusted on one node but not the other, or
- `authority_chain_id` differs under otherwise comparable delegated authority

### 5.5 `PARITY_VERDICT_MISMATCH`

Use when:
- `verdict_subject` matches
- `verification_context_id` matches
- trusted verifier semantics match
- local verification verdict differs

### 5.6 `PARITY_HISTORICAL_ONLY`

Use when:
- compared artifacts are valid historical artifacts
- but current distributed acceptance cannot be claimed

### 5.7 `PARITY_INSUFFICIENT_EVIDENCE`

Use when:
- a required receipt, context object, verifier attestation, or verifier registry snapshot is missing
- parity cannot be determined from available artifacts

---

## 6. Classification Priority

When multiple mismatch conditions are present, classification MUST follow this priority:

1. `PARITY_INSUFFICIENT_EVIDENCE`
2. `PARITY_SUBJECT_MISMATCH`
3. `PARITY_CONTEXT_MISMATCH`
4. `PARITY_VERIFIER_MISMATCH`
5. `PARITY_VERDICT_MISMATCH`
6. `PARITY_HISTORICAL_ONLY`
7. `PARITY_MATCH`

---

## 7. Fail-Closed Rules

A node MUST reject positive parity claims when:
- any required parity input is missing
- `verdict_subject` differs
- `verification_context_id` differs
- verifier trust semantics differ
- local verification verdict differs

Additional rules:
- parity mismatch MUST NOT be downgraded to warning-only acceptance
- parity mismatch MUST NOT be re-labeled as `UNTRUSTED`
- `PARITY_HISTORICAL_ONLY` MUST NOT be treated as current distributed acceptance

---

## 8. Historical and Temporal Semantics

### 8.1 Historical Rule

Historical parity is allowed only as a reporting surface, not as current trust acceptance.

### 8.2 Revocation Rule

If verifier revocation or registry/policy epoch shift moves a receipt into historical-only interpretation, parity status MUST be `PARITY_HISTORICAL_ONLY`, not `PARITY_MATCH`.

### 8.3 No Silent Upgrade Rule

Old receipts or parity records MUST NOT be silently upgraded into current distributed trust claims under a newer:
- verifier trust registry
- verification context object
- policy snapshot
- registry snapshot

---

## 9. Failure Matrix Reporting

Cross-node parity reporting SHOULD emit machine-readable rows containing at least:

```json
{
  "node_a": "node-a",
  "node_b": "node-b",
  "parity_status": "PARITY_CONTEXT_MISMATCH",
  "bundle_id_equal": true,
  "trust_overlay_hash_equal": true,
  "policy_hash_equal": true,
  "registry_snapshot_hash_equal": true,
  "verification_context_id_equal": false,
  "authority_chain_id_equal": null,
  "trusted_verifier_semantics_equal": true,
  "local_verdict_equal": true
}
```

---

## 10. Threat Model Notes

This specification primarily mitigates:
- context fork attacks
- cross-registry split-brain hidden behind valid receipts
- verifier identity shadowing
- false parity claims built on incomplete evidence
- historical receipt reuse misrepresented as current distributed agreement

It does not itself solve:
- receipt DAG federation
- consensus
- global total ordering

Those remain later-phase concerns.

---

## 11. Acceptance Criteria

11.1. THE System SHALL define a parity status set distinct from local verifier verdicts
11.2. THE System SHALL include at least: `PARITY_MATCH`, `PARITY_SUBJECT_MISMATCH`, `PARITY_CONTEXT_MISMATCH`, `PARITY_VERIFIER_MISMATCH`, `PARITY_VERDICT_MISMATCH`, `PARITY_HISTORICAL_ONLY`, `PARITY_INSUFFICIENT_EVIDENCE`
11.3. A positive parity claim SHALL require equality of `verdict_subject`, `verification_context_id`, trusted verifier semantics, and local verification verdict
11.4. Context mismatch SHALL classify as `PARITY_CONTEXT_MISMATCH` and SHALL NOT be downgraded to warning-only behavior
11.5. Trusted verifier mismatch SHALL classify as `PARITY_VERIFIER_MISMATCH` and SHALL NOT be re-labeled as receipt signature failure alone
11.6. Historical-only distributed interpretation SHALL classify as `PARITY_HISTORICAL_ONLY`, not `PARITY_MATCH`
11.7. Missing required parity artifacts SHALL classify as `PARITY_INSUFFICIENT_EVIDENCE`
11.8. THE System SHALL define a deterministic classification priority order for multiple simultaneous mismatch conditions
11.9. Cross-node parity reporting SHALL export machine-readable failure classification sufficient to build `failure_matrix.json`

---

## 12. Summary

Cross-node parity is not a boolean.

It is a deterministic classification problem over:
- trust subject
- distributed context
- trusted verifier semantics
- local verification result

Without explicit failure semantics, distributed verification drifts into ambiguous trust claims.
