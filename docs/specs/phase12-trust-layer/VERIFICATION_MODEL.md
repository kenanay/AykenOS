# Verification Model

**Version:** 1.0
**Status:** Informational formal model
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative verification model note
**Related Spec:** `AYKENOS_ARCHITECTURE_ONE_PAGE.md`, `AYKENOS_GLOBAL_ARCHITECTURE_DIAGRAM.md`, `AYKENOS_TECHNICAL_DEFINITION_SET.md`, `DISTRIBUTED_VERIFICATION_SYSTEMS_FORMAL_MODEL.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`

---

## 1. Purpose

This document defines the compact AykenOS verification model.

Its role is to isolate the local architectural core from the broader `Distributed Verification Systems` research family.

The model is intentionally small.

It exists to keep five primitives explicit:

- subject
- context
- authority
- artifact
- verdict

---

## 2. Core Primitives

Let:

- `S`
  - subject surface
- `C`
  - context surface
- `A`
  - authority surface
- `E`
  - artifact surface
- `V`
  - verification verdict

Define the verification input:

`Q = (S, C, A)`

Define deterministic evaluation:

`Eval(Q) -> V`

Define the artifact-bound verification object:

`R = (Q, V, E)`

Define the verification function:

`Verify(S, C, A) -> (V, E)`

So the local AykenOS verification object is not just a verdict.

It is:

`verification input + verdict + artifacts`

---

## 3. AykenOS Surface Bindings

### 3.1 Subject

In AykenOS the subject surface is carried by the verifier result subject:

`S = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`

This keeps subject identity bound to the actual verification inputs rather than only the portable bundle payload.

### 3.2 Context

The context surface is represented by:

`C = verification_context_id`

The current context object may bind:

- policy material
- registry material
- verifier contract version
- context-rules material

### 3.3 Authority

The authority surface is represented as:

`A = (result_class, verifier_registry_snapshot_hash, effective_authority_scope, authority_chain_id)`

This keeps verifier authority semantics explicit and separable from the verification subject itself.

### 3.4 Artifact

The artifact surface is represented by evidence outputs such as:

- signed receipts
- run manifests
- verification reports
- audit-chain artifacts
- parity and diagnostics artifacts

Architectural rule:

`artifacts are derived from verification, not a replacement for verification semantics`

### 3.5 Verdict

At the current architectural level:

`V ∈ {Trusted, Untrusted, Invalid, RejectedByPolicy}`

The exact crate-level naming may vary, but the model assumes a deterministic local verdict class.

---

## 4. Deterministic Verification Rule

The central AykenOS invariant is:

`same subject + same context + same authority -> same verdict`

or equivalently:

`Q_1 = Q_2 => Eval(Q_1) = Eval(Q_2)`

This does not mean all nodes always agree.

It means disagreement must be explainable by one of the following:

- subject drift
- context drift
- authority drift
- insufficient evidence
- explicit determinism violation

### 4.1 Semantic and Artifact-Bound Forms

The semantic form is:

`Eval : (S, C, A) -> V`

The artifact-emitting form is:

`Verify : (S, C, A) -> (V, E)`

This keeps pure verification semantics distinct from artifact emission while still binding them together in the final result object.

---

## 5. Truth Rule

AykenOS does not define truth through consensus or authority election.

Its local truth rule is:

`truth = artifact-bound verification result`

This means:

- truth is computed by deterministic verification
- truth is carried durably by artifacts
- truth is compared across nodes through diagnostics

It does not mean:

- truth is elected by consensus
- truth is chosen by diagnostics
- truth is derived from service availability

---

## 6. Service Relation

`proofd` wraps the verification model but does not replace it.

So the stable relation is:

`proofd = service wrapper over (Q, V, E)`

and not:

`proofd = source of verification semantics`

The service layer may execute verification and expose artifacts, but the model remains artifact-first.

---

## 7. Summary

The AykenOS verification model can be reduced to:

`Q = (S, C, A)`

`Eval(Q) -> V`

`R = (Q, V, E)`

with the governing rule:

`same S + same C + same A -> same V`
