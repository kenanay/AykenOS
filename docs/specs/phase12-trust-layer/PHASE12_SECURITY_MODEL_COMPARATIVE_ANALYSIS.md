# Phase-12 Security Model Comparative Analysis

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative comparative analysis
**Related Spec:** `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`, `PROOF_BUNDLE_V2_SPEC.md`, `requirements.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document compares the AykenOS Phase-12 security model against several well-known verification ecosystems:

- Sigstore
- TUF
- in-toto
- Reproducible Builds

The goal is not to rank projects simplistically.

The goal is to explain what AykenOS is architecturally solving that is different from, or broader than, a classic artifact-signing stack.

This note is non-normative.

---

## 2. Comparison Lens

The comparison uses the three truth surfaces already defined by Phase-12:

- truth subject
- truth context
- truth authority

In AykenOS terms:

- subject identity
  - `bundle_id`
- context identity
  - `verification_context_id`
- authority identity
  - `authority_chain_id`

This lens matters because many systems verify artifacts successfully while leaving context and authority partially implicit.

AykenOS treats all three as first-class, hash-bound, fail-closed surfaces.

---

## 3. The AykenOS Claim

AykenOS Phase-12 is not only trying to answer:

`Is this artifact authentic?`

It is trying to answer:

- what was verified
- under which rules it was verified
- who is authorized to carry that verification into distributed trust space
- whether another node reached the same distributed truth claim

That makes it closer to a:

`deterministic distributed truth verification architecture`

than to a plain signing or provenance system.

---

## 4. Comparison Summary

| System | Subject Surface | Context Surface | Authority Surface | Distributed Parity Semantics |
| --- | --- | --- | --- | --- |
| Sigstore | Strong | Partial / mostly verifier-local | Partial | Weak |
| TUF | Strong | Strong for update trust | Limited for generic distributed parity | Weak |
| in-toto | Strong | Partial | Weak / implicit | Weak |
| Reproducible Builds | Strong for determinism claims | Minimal | None | None |
| AykenOS Phase-12 | Strong | Strong | Strong | Strong |

This table is intentionally high-level.

The differences become clearer when each system is examined through the same subject/context/authority lens.

---

## 5. Sigstore

### 5.1 Strengths

Sigstore is strong at:

- artifact signing
- signer identity binding
- transparency log support
- ecosystem-scale developer UX

Its typical chain is:

`artifact -> signature -> certificate -> transparency log`

### 5.2 Limits Under the AykenOS Lens

Sigstore is strongest on subject authenticity.

It is weaker on portable verification context.

In practice, many trust decisions still depend on verifier-local policy such as:

- which identities are accepted
- which issuers are trusted
- which policy profile is in effect
- what local acceptance rules are applied

That means:

`same artifact + same signature`

does not by itself imply:

`same distributed truth claim`

across nodes.

### 5.3 AykenOS Difference

AykenOS makes the acceptance context explicit and portable through:

- `verification_context_id`
- verification context object
- context portability package

It also separates:

`signature validity != verifier authority`

which Sigstore-style systems often leave to surrounding operational policy.

---

## 6. TUF

### 6.1 Strengths

TUF is strong at:

- trust-root rotation
- metadata hierarchy
- rollback protection
- update-client security

It explicitly models signed metadata and trust-root evolution.

### 6.2 Limits Under the AykenOS Lens

TUF is optimized for secure software update distribution.

Its context model is strong for that domain, but not designed as a generic distributed truth portability layer.

TUF generally assumes:

- a client already has a trust-root model
- update metadata semantics are domain-specific
- parity between arbitrary verifier nodes is not the main problem

So while TUF has strong metadata trust semantics, it does not natively define:

- a generic portable verification context object
- a generic authority chain for verifier-as-speaker semantics
- parity failure taxonomy across distributed verifiers

### 6.3 AykenOS Difference

AykenOS borrows the idea that metadata lineage matters, but generalizes it beyond update systems.

It defines:

- truth subject
- truth context
- verifier authority

as generic distributed verification surfaces rather than update-only metadata roles.

---

## 7. in-toto

### 7.1 Strengths

in-toto is strong at:

- supply-chain step attestation
- layout-driven provenance
- link metadata
- role-based process integrity

It is much closer than simple signing systems to describing:

`how a result came to exist`

### 7.2 Limits Under the AykenOS Lens

in-toto is strong on provenance semantics, but its trust context is often bound tightly to the layout and its surrounding supply-chain model.

That is powerful, but different from AykenOS’s separation strategy.

Under the AykenOS lens, in-toto often combines:

- what happened
- which process definition is accepted
- who is allowed to attest it

more tightly than Phase-12 wants to.

AykenOS instead insists that:

- proof subject
- trust context
- verifier authority

remain distinct.

### 7.3 AykenOS Difference

AykenOS is less focused on step provenance alone and more focused on:

`portable, reconstructable, parity-comparable truth claims`

across nodes.

That is a different target.

---

## 8. Reproducible Builds

### 8.1 Strengths

Reproducible Builds is strong at:

- deterministic output claims
- same source -> same artifact reasoning
- exposing nondeterminism

It is foundational for trustworthy build verification.

### 8.2 Limits Under the AykenOS Lens

Reproducible Builds provides subject determinism.

It does not, by itself, provide:

- portable verification context
- verifier authority semantics
- distributed truth transport
- parity failure classification

It can tell you that two builds should match.

It does not tell you:

- which policy made the build acceptable
- which registry or rules were in effect
- which verifier is trusted to speak for the claim

### 8.3 AykenOS Difference

AykenOS can incorporate deterministic build evidence, but extends beyond it into:

- context determinism
- authority determinism
- distributed parity determinism

So Reproducible Builds is a component idea inside the broader AykenOS model, not an architectural substitute for it.

---

## 9. Why AykenOS Is More General

The key distinction is this:

Most systems primarily secure artifacts.

AykenOS secures:

- artifact identity
- interpretation identity
- verifier authority identity

as separate but comparable surfaces.

That lets AykenOS represent a portable truth claim as:

`truth subject + truth context + truth authority`

This is more general than:

- artifact signing alone
- provenance recording alone
- trust-root metadata alone
- determinism testing alone

because it can model all of them as sub-cases of distributed truth verification.

---

## 10. What AykenOS Still Does Not Have

This comparison should not overstate current maturity.

AykenOS Phase-12 is architecturally strong, but still operationally incomplete.

Major remaining gaps include:

- larger A/B/C/D parity matrices
- broader negative corpus
- `proofd` service surface
- full context transport exercise outside synthetic local fixtures
- later-phase storage / federation / consensus questions

So the correct claim is:

`AykenOS has a broader security model architecture`

not:

`AykenOS has already finished every distributed systems layer`

---

## 11. Security Interpretation

Under this comparison, AykenOS Phase-12 should be understood as adding a third security category on top of earlier phases:

- execution security
  - Phases 1-10
- proof security
  - Phase 11
- truth security
  - Phase 12

Or more compactly:

`execution -> proof -> truth`

This is why the architecture now feels qualitatively different from a classic build or signing pipeline.

---

## 12. Final Comparison

AykenOS is not better because it signs more things.

It is stronger in architectural scope because it keeps separate:

- what is true
- under which rules it is true
- who may speak that truth into distributed trust space

That separation is what allows:

- fail-closed context transport
- fail-closed authority resolution
- deterministic parity comparison
- portable distributed truth claims

This is the system’s defining advantage over narrower supply-chain verification models.

---

## 13. Summary

Sigstore, TUF, in-toto, and Reproducible Builds each solve important parts of the verification problem.

AykenOS Phase-12 is unusual because it tries to unify their strengths under a stricter architecture:

- subject determinism
- context determinism
- authority determinism
- distributed parity semantics

That is why the Phase-12 security model is best understood not merely as proof verification, but as:

`generic deterministic truth verification`
