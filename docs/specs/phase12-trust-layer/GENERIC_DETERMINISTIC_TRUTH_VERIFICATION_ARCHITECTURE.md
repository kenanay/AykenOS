# Generic Deterministic Truth Verification Architecture

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative architecture note
**Related Spec:** `PROOF_BUNDLE_V2_SPEC.md`, `requirements.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `PARITY_LAYER_ARCHITECTURE.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document explains why AykenOS Phase-12 is no longer only a proof verifier.

It frames the current system as a more general architecture:

`generic deterministic truth verification`

This note is intentionally non-normative.

Normative requirements remain in the Phase-12 contracts and specifications.

Its job is to explain the architectural theorem that now emerges from those normative surfaces.

---

## 2. Why Artifact Verification Is Not Enough

Most verification systems stop at:

`artifact + signature + trust root`

That is sufficient for local authenticity checks, but insufficient for deterministic distributed truth.

Distributed systems need to answer three different questions:
- what was verified
- under which rules it was verified
- who may carry that verification into shared trust space

If these questions collapse into one object, the system drifts into:
- local correctness mistaken for distributed correctness
- receipt transport mistaken for trust transport
- signature validity mistaken for verifier authority

AykenOS Phase-12 explicitly prevents that collapse.

---

## 3. Core Theorem

Two distributed truth claims MAY be treated as the same claim only when all three surfaces match:

`same truth subject AND same verification context AND same verifier authority semantics`

Or, in AykenOS terms:

`same bundle_id AND same verification_context_id AND same authority semantics`

If any one of these differs, the system MUST NOT claim the same distributed truth result.

This is the architectural center of Phase-12.

A useful non-normative abstraction is:

`T = H(subject, context, authority)`

This is not yet a normative wire field.

It is a compact way to express portable truth-claim identity.

Distributed parity remains stricter and still depends on verdict equality as defined elsewhere.

---

## 4. Three Truth Surfaces

### 4.1 Subject Surface

Question:

`What was verified?`

Primary AykenOS identity:

`bundle_id`

This surface covers:
- proof material
- portable bundle identity
- manifest/checksum-bound artifact integrity
- proof-manifest-bound execution evidence

This is the truth subject.

### 4.2 Context Surface

Question:

`Under which rules was it verified?`

Primary AykenOS identity:

`verification_context_id`

This surface covers:
- policy snapshot identity
- registry snapshot identity
- context-rules identity
- verifier contract version
- portability package needed to reconstruct that context across nodes

This is the truth context.

### 4.3 Authority Surface

Question:

`Who may carry this truth claim into distributed trust?`

Primary AykenOS identity:

`authority_chain_id`

This surface covers:
- verifier identity
- verifier trust registry lineage
- authority graph constraints
- delegation path
- current versus historical authority interpretation

This is the truth authority surface.

---

## 5. Deterministic Truth Pipeline

The generic evaluation pipeline in AykenOS now reads:

`subject load`
`-> subject verify`
`-> context resolve`
`-> context verify`
`-> authority resolve`
`-> authority verify`
`-> local verdict`
`-> portable receipt`
`-> cross-node parity comparison`

The order matters.

Subject verification does not depend on verifier authority.

Context verification does not mutate proof identity.

Authority verification does not redefine the proof or the context.

Each stage adds one more layer of truth interpretation without collapsing the earlier one.

---

## 6. Output Classes

The architecture distinguishes four different output classes:

### 6.1 Local Validity

Question:

`Is the subject structurally and cryptographically valid?`

This is not yet a distributed trust claim.

### 6.2 Local Trust Acceptance

Question:

`Does the verifier accept this subject under local policy and registry inputs?`

This still does not imply portability.

### 6.3 Portable Truth Claim

Question:

`Can this acceptance be exported with enough subject, context, and authority material to be reconstructed elsewhere?`

This is where receipts, context portability, and verifier authority semantics meet.

### 6.4 Distributed Parity Status

Question:

`Does another node reach the same distributed truth claim?`

This is not the same as local validity and not the same as local trust acceptance.

So the architecture preserves the distinction:

`valid != trusted != portable != parity-equal`

---

## 7. Failure Taxonomy

The architecture becomes stronger because it classifies failure by layer:

- subject failure
- context failure
- authority failure
- parity failure
- insufficient evidence

This means rejection is no longer a single bucket.

A failure can now answer:
- the proof was wrong
- the context was missing or mismatched
- the verifier authority was invalid or ambiguous
- parity failed even though local verification was correct

This is a major architectural gain over binary pass/fail models.

---

## 8. Portability Versus Authority Versus Parity

AykenOS separates three concepts that many systems merge:

### 8.1 Portability

A truth claim is portable only if another node can reconstruct the same subject and context.

### 8.2 Authority

A portable truth claim is not yet shared trust evidence unless the verifying node is itself trusted to speak in distributed trust space.

### 8.3 Parity

Even portable and authority-valid claims do not imply parity unless another node reaches the same outcome under the same context and authority semantics.

Therefore:

`portable truth claim != trusted verifier authority != parity agreement`

---

## 9. Why This Architecture Is Generic

This architecture is generic because it is not tied only to proof bundles.

The same model can apply to:
- build attestation
- replay verification
- audit claim verification
- distributed compliance evidence
- deterministic workflow certification
- multi-node execution attestation

The generic form is:

`truth subject + truth context + truth authority = portable distributed truth candidate`

That is broader than supply-chain signing alone.

---

## 10. AykenOS Mapping

The current AykenOS Phase-12 mapping is:

- truth subject
  - `bundle_id`
  - `trust_overlay_hash`
  - `verdict_subject`

- truth context
  - `verification_context_id`
  - verification context object
  - context portability package

- truth authority
  - verifier trust registry snapshot
  - deterministic authority resolution
  - `authority_chain_id`

- truth transport
  - signed receipt
  - audit event
  - parity matrix

This means AykenOS already implements the three structural layers required for deterministic distributed truth verification.

---

## 11. Phase-12 to Phase-13 Bridge

Phase-11 delivered:

`portable proof`

Phase-12 delivers:

`trusted verification`

The natural Phase-13 bridge is:

`portable trusted verification across nodes`

In practical terms, the next steps are not new theory but system stress:
- larger cross-node parity matrices
- more negative corpus
- `proofd` service surfaces
- eventually distributed replay and verification network semantics

Phase-13 therefore grows out of Phase-12 by scaling the already separated truth surfaces, not by redefining them.

---

## 12. Non-Goals

This architecture note does not define:
- consensus
- global ordering
- storage backends for content-addressed context or registry material
- receipt DAG federation
- verifier reputation
- quorum trust weighting

Those remain later-phase concerns.

The current goal is not universal distributed consensus.

The current goal is deterministic, reconstructable, fail-closed distributed truth comparison.

---

## 13. Summary

AykenOS Phase-12 is no longer only a proof verification stack.

It now defines a more general architecture:

`Generic Deterministic Truth Verification`

Its critical design decision is the strict separation of:
- truth subject
- truth context
- truth authority

That separation is what allows the system to move from local verification toward deterministic distributed truth without collapsing proof identity, context semantics, and trust authority into one mutable object.
