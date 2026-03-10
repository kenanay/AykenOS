# AykenOS Distributed Truth Model Formal Security Properties

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative formal security note
**Related Spec:** `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`, `PHASE12_SECURITY_MODEL_COMPARATIVE_ANALYSIS.md`, `VERIFICATION_CONVERGENCE_THEOREM.md`, `TRUTH_STABILITY_THEOREM.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `PROOF_BUNDLE_V2_SPEC.md`, `requirements.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document states the formal security properties implied by the AykenOS Phase-12 trust model.

It is not a new normative contract.

Its purpose is to express the security logic already emerging from the normative documents in a more theorem-like form:

- security invariants
- trust theorem
- attack classes
- failure guarantees

This note exists to make the model easier to reason about academically and architecturally.

---

## 2. Core Objects

Let:

- `S`
  - truth subject
  - concretely represented by `verdict_subject`
- `C`
  - truth context
  - concretely represented by `verification_context_id`
- `A`
  - truth authority semantics
  - concretely represented by verifier authority interpretation, including `authority_chain_id` when delegated authority is current
- `V`
  - local verification verdict

The Phase-12 model can then be described by two related abstractions:

### 2.1 Distributed Truth Claim Identity

Conceptually:

`T = H(S, C, A)`

This is a compact way to say:

`same subject + same context + same authority semantics => same portable truth-claim identity`

This is a conceptual identity, not currently a normative on-wire field.

### 2.2 Parity Comparison Object

Distributed parity remains stricter than truth-claim identity alone.

Parity comparison requires:

`P = (S, C, A, V)`

So:

`T_A == T_B`

is necessary for parity, but not sufficient by itself unless the local verdict also matches.

This preserves the current Phase-12 rule that local verifier verdict and distributed parity status are different surfaces.

---

## 3. Core Security Invariants

### 3.1 Subject Integrity Invariant

If the proof subject is mutated, the verifier MUST NOT preserve the same truth subject.

In practice:

- tampered proof material
- tampered manifest binding
- tampered checksums

must cause subject failure or subject identity drift.

### 3.2 Context Integrity Invariant

If policy, registry, or distributed interpretation rules differ, the verifier MUST NOT preserve the same truth context.

In practice:

- policy drift
- registry drift
- context-rules drift

must cause `verification_context_id` mismatch or context rejection.

### 3.3 Authority Integrity Invariant

If verifier trust lineage or delegated authority semantics differ, the verifier MUST NOT preserve the same authority interpretation.

In practice:

- root drift
- delegation ambiguity
- historical/revoked change
- verifier trust registry drift

must cause authority mismatch or fail-closed authority rejection.

### 3.4 Surface Non-Collapse Invariant

The system MUST preserve:

`subject != context != authority`

This means:

- proof identity MUST NOT absorb context identity
- context identity MUST NOT absorb verifier authority semantics
- signature validity MUST NOT imply verifier authority

This is the fundamental anti-collapse property of the AykenOS model.

---

## 4. Trust Theorem

### 4.1 Portable Truth Claim Theorem

A portable distributed truth claim exists only when:

- subject verification succeeds
- context verification succeeds
- authority verification succeeds

That is:

`portable truth claim = valid subject + valid context + valid authority`

### 4.2 Distributed Parity Theorem

Two nodes MAY claim distributed parity only if:

- `S_A == S_B`
- `C_A == C_B`
- `A_A == A_B`
- `V_A == V_B`

Equivalently:

`P_A == P_B`

where:

`P = (S, C, A, V)`

Any inequality in these four surfaces MUST prevent `PARITY_MATCH`.

### 4.3 Deterministic Evaluation Property

For compliant verifiers, identical subject, context, and authority inputs MUST produce identical local verdicts.

That is:

`(S, C, A) -> deterministic V`

This is one of the model's strongest claims.

It means the verifier is not merely checking authenticity; it is executing a deterministic truth evaluation over the same truth surfaces.

This property is what makes `P = (S, C, A, V)` meaningful as a parity comparison object rather than just an audit tuple.

### 4.4 Trust Speaker Theorem

A valid receipt does not imply a valid distributed trust speaker.

So:

`valid receipt != trusted verifier authority`

Distributed trust reuse therefore requires both:

- receipt validity
- authority validity

---

## 5. Attack Classes

The formal model implies four primary attack classes:

### 5.1 Subject Attacks

Examples:

- portable proof tampering
- proof-manifest drift
- checksum or hash binding drift

Goal:

break `S`

### 5.2 Context Attacks

Examples:

- policy substitution
- registry substitution
- context-rules substitution
- receipt forwarding without reconstructable context

Goal:

break `C`

### 5.3 Authority Attacks

Examples:

- verifier authority capture
- delegation fork
- root drift
- authority loop
- identity shadowing

Goal:

break `A`

### 5.4 Parity Attacks

Examples:

- parity misclassification
- hiding mismatch behind weak status labels
- presenting local correctness as distributed agreement

Goal:

break comparison over `P = (S, C, A, V)`

---

## 6. Failure Guarantees

The AykenOS model provides the following guarantees:

### 6.1 Fail-Closed Subject Guarantee

If subject evidence cannot be recomputed and matched, the verifier rejects the claim.

### 6.2 Fail-Closed Context Guarantee

If distributed context cannot be resolved or reconstructed exactly, the verifier rejects distributed trust reuse.

### 6.3 Fail-Closed Authority Guarantee

If authority cannot be resolved uniquely and validly, the verifier rejects distributed trust speaker semantics.

### 6.4 Non-Ambiguous Parity Guarantee

If parity evidence is incomplete or mismatched, the system emits a failure classification rather than silently collapsing into generic success or generic untrusted behavior.

### 6.5 Historical Safety Guarantee

Historical artifacts may remain interpretable, but they MUST NOT silently re-enter the current trust surface.

---

## 7. Failure Taxonomy

The model implies a layered taxonomy:

- subject failure
- context failure
- authority failure
- parity failure
- insufficient evidence

This taxonomy is one of the model’s key strengths.

It means failure reason is itself transportable and comparable, not just the final accept/reject bit.

---

## 8. AykenOS Mapping

Within AykenOS Phase-12:

- `S`
  - `verdict_subject`
  - rooted in `bundle_id`, `trust_overlay_hash`, `policy_hash`, `registry_snapshot_hash`

- `C`
  - `verification_context_id`
  - rooted in the verification context object and its portability package

- `A`
  - verifier-trust registry semantics
  - deterministic authority resolution
  - `authority_chain_id` when current delegated authority applies

- `V`
  - local verification verdict

This means the current implementation already approximates a formal distributed truth model, even if the notation itself is not yet part of the wire contract.

---

## 9. Residual Risks

The formal model is stronger than the current operational exercise surface.

Key residual risks remain:

- parity matrix scale is still small
- context transport outside synthetic fixtures remains immature
- `proofd` service surfaces are not yet active
- broader negative corpus is still needed
- later storage and federation semantics remain open

So the right statement is:

`the formal model is strong, but the distributed operating surface is still being expanded`

---

## 10. Non-Goals

This note does not define:

- consensus
- distributed storage
- reputation or trust weighting
- quorum math
- receipt DAG federation

Those belong to later phases.

The purpose here is to show that the current AykenOS model already has formal security structure before those later layers arrive.

---

## 11. Summary

The AykenOS Phase-12 trust layer can be expressed formally as a distributed truth model over:

- subject
- context
- authority
- verdict

with:

`T = H(S, C, A)`

as a useful abstraction for portable truth-claim identity, and:

`P = (S, C, A, V)`

as the stricter object required for distributed parity.

This is what makes the model stronger than ordinary artifact verification:

it secures not only what is true, but also under which rules it is true and who may speak that truth into distributed trust space.
