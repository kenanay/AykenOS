# Parity Layer Formal Model

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-09
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative formal model note
**Related Spec:** `AYKENOS_DISTRIBUTED_TRUTH_MODEL_FORMAL_SECURITY_PROPERTIES.md`, `VERIFICATION_CONVERGENCE_THEOREM.md`, `TRUTH_STABILITY_THEOREM.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `CROSS_NODE_PARITY_HARDENING_CHECKLIST.md`, `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document states the formal model that now emerges from the Phase-12 parity layer.

It is non-normative.

Its job is to describe parity as more than a simple equality check.

The AykenOS parity layer is better understood as:

`a deterministic convergence classifier, not merely an equality checker`

This note exists to make the executable parity surface easier to reason about academically, architecturally, and operationally.

Stability rule:

`NodeParityOutcome` is the crate-owned canonical node object and `authority/parity.rs` is the single hash authority for parity `D_i` / `K_i` generation.

---

## 2. Core Objects

Let:

- `S`
  - subject surface
- `C`
  - context surface
- `A`
  - authority surface
- `V`
  - local verification verdict

The current parity layer operates first on a verifier outcome:

`Outcome = (S, C, A, V)`

But parity comparison needs one more layer beyond the outcome itself:

- how that outcome is transported
- whether the comparison has enough evidence to classify the result

So parity operates on:

`ParityInput = (Outcome, artifact_form, evidence_state)`

This gives the general form:

`Parity(Left, Right) -> Status`

---

## 3. Outcome Model

### 3.1 Subject Surface

In the current AykenOS implementation, the subject surface is carried by:

`S = verdict_subject`

with:

`S = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`

This means parity does not compare only the portable payload hash.

It compares the full verdict-subject identity already emitted by the verifier core.

### 3.2 Context Surface

The current parity layer treats context as:

`C = verification_context_id`

This context identity is expected to be recomputed from the canonical verification context object.

In practice, the context object binds:

- `policy_hash`
- `registry_snapshot_hash`
- `verifier_contract_version`
- `context_rules_hash`

So the current parity model is:

`same context = same canonical verification_context_id`

not:

`same loose local inputs`

### 3.3 Authority Surface

The authority surface is best modeled against the current executable implementation as:

`A = (result_class, verifier_registry_snapshot_hash, effective_authority_scope, authority_chain_id)`

This is more precise than reducing authority to only roots or delegation links.

It matches the current authority-aware parity logic more faithfully.

### 3.4 Verdict Surface

At the abstract level:

`V ∈ VerdictClass`

AykenOS mapping:

`V ∈ {Trusted, Untrusted, Invalid, RejectedByPolicy}`

This keeps the formal model compact while remaining faithful to the implementation.

### 3.5 Canonical Node Object Boundary

The executable model now assumes a single constructor path for parity node objects:

`build_node_parity_outcome(...) -> NodeParityOutcome`

This boundary matters because:

- `surface_key = D_i = H(S_i, C_i, A_i)`
- `outcome_key = K_i = H(S_i, C_i, A_i, V_i)`

must be produced by one canonical implementation.

So the architectural rule is:

- external layers MAY consume `NodeParityOutcome`
- external layers MUST NOT recompute `surface_key` or `outcome_key` independently
- helper hash functions remain internal to the parity layer

---

## 4. Artifact Form And Evidence State

Parity comparison is not determined only by `Outcome`.

It also depends on whether the system has enough material to perform a valid comparison and what artifact form is being compared.

### 4.1 Artifact Form

The current active parity surface supports:

- `artifact_form = signed_receipt`
- `artifact_form = local_verification_outcome`

This is critical because receipt presence is not identical to truth availability.

Receipt transport is evidence transport, not truth creation.

### 4.2 Evidence State

The evidence axis is best modeled separately from artifact form:

- `evidence_state = sufficient`
- `evidence_state = insufficient`

This is necessary because:

- a receipt may be absent while parity still has enough local outcome material
- a receipt may be present while required context or authority material is still insufficient

So:

`artifact_form != evidence_state`

---

## 5. Parity Classification Order

The current parity layer is most accurately modeled as an ordered classifier.

The classification order is:

1. if `evidence_state = insufficient`
   - `Status = PARITY_INSUFFICIENT_EVIDENCE`
2. else if `S` differs
   - `Status = PARITY_SUBJECT_MISMATCH`
3. else if `C` differs
   - `Status = PARITY_CONTEXT_MISMATCH`
4. else if `A` differs
   - `Status = PARITY_VERIFIER_MISMATCH`
5. else if `S = C = A` but `V` differs
   - `Status = PARITY_VERDICT_MISMATCH`
6. else if authority interpretation is equal but historical-only
   - `Status = PARITY_HISTORICAL_ONLY`
7. else
   - `Status = PARITY_MATCH`

This ordered view matters.

It means parity is not:

`one boolean equality test`

It is:

`an ordered fail-closed convergence classifier`

---

## 6. Consistency Versus Determinism Separation

This is the most important conceptual distinction in the model.

### 6.1 Consistency Surface

The following outcomes belong to the consistency surface:

- `PARITY_SUBJECT_MISMATCH`
- `PARITY_CONTEXT_MISMATCH`
- `PARITY_VERIFIER_MISMATCH`
- `PARITY_HISTORICAL_ONLY`
- `PARITY_INSUFFICIENT_EVIDENCE`

These are not model violations.

They are explicit, expected distributed classifications.

They mean:

- the compared nodes did not hold the same truth surfaces
- or the comparison did not have enough evidence to prove convergence

### 6.2 Determinism Surface

`PARITY_VERDICT_MISMATCH` is different.

It belongs to the determinism surface, not the ordinary consistency surface.

It means:

`same S + same C + same A but different V`

That is a different class of event from ordinary distributed mismatch.

It is a determinism alarm surface.

So the right conceptual rule is:

`consistency failure != determinism failure`

This distinction exists even if the current gate still exports both surfaces through one matrix report.

### 6.3 Reporting Implication

The current local gate now exports a split surface:

- `parity_consistency_report.json`
- `parity_determinism_report.json`
- `parity_determinism_incidents.json`
- `parity_convergence_report.json`

The convergence artifact is now built from stable node-level `Outcome` material rather than only re-reading pairwise match edges.
The determinism artifact set now also lifts same-surface verdict divergence into explicit `DeterminismIncident` objects rather than leaving it implicit inside pairwise rows.

This is the cleanest shape because it preserves the distinction between:

- expected distributed drift
- deterministic model alarm

---

## 7. Core Theorems

### 7.1 Subject Preservation Theorem

If the compared nodes do not preserve the same subject surface, parity MUST NOT produce `PARITY_MATCH`.

Formally:

`S_left != S_right -> Status != PARITY_MATCH`

### 7.2 Context Preservation Theorem

If the compared nodes do not preserve the same context surface, parity MUST NOT produce `PARITY_MATCH`.

Formally:

`C_left != C_right -> Status != PARITY_MATCH`

### 7.3 Authority Preservation Theorem

If the compared nodes do not preserve the same authority surface, parity MUST NOT produce `PARITY_MATCH`.

Formally:

`A_left != A_right -> Status != PARITY_MATCH`

### 7.4 Deterministic Verdict Theorem

If the compared nodes preserve the same normalized subject, context, and authority surfaces, then the local verdict must converge.

Formally:

`S_left = S_right AND C_left = C_right AND A_left = A_right -> V_left = V_right`

If not:

`Status = PARITY_VERDICT_MISMATCH`

This is the executable determinism guard now active in the parity layer.

### 7.5 Receipt Non-Primacy Theorem

Receipt transport is not the truth source.

If a receipt is absent but parity still has sufficient outcome material, parity may still classify.

So:

`receipt absent != parity impossible`

This is why the current model permits:

`artifact_form = local_verification_outcome`

---

## 8. AykenOS Mapping

Current executable mapping:

- `S`
  - `VerdictSubject`
  - `bundle_id`, `trust_overlay_hash`, `policy_hash`, `registry_snapshot_hash`
- `C`
  - `verification_context_id`
  - canonical context object identity
- `A`
  - `result_class`
  - `verifier_registry_snapshot_hash`
  - `effective_authority_scope`
  - `authority_chain_id`
- `V`
  - `Trusted`, `Untrusted`, `Invalid`, `RejectedByPolicy`
- `artifact_form`
  - `signed_receipt` or `local_verification_outcome`
- `evidence_state`
  - `sufficient` or `insufficient`

This is what the current parity gate is actually comparing.

---

## 9. Residual Risks

The formal model is stronger than the currently exercised matrix.

The main residual gaps are:

- authority scope drift is not yet separated into its own executable mismatch slice
- the active gate now exports split consistency/determinism/convergence reports, and the convergence artifact now uses stable node-level `D_i` / `K_i` partitions, but the primary executable classifier still remains fundamentally pairwise
- verifier-contract-version drift is currently classified as context drift rather than being separately summarized
- service-backed parity transport through `proofd` is not yet active

So the correct current claim is:

`the parity formal model is strong, but the distributed execution matrix is still expanding`

---

## 10. Summary

The current AykenOS parity layer is best modeled as:

`ParityInput = (Outcome, artifact_form, evidence_state)`

where:

`Outcome = (S, C, A, V)`

and:

`Parity(Left, Right) -> Status`

The most important architectural conclusion is:

`AykenOS parity layer is a deterministic convergence classifier, not merely an equality checker`

This is what allows parity to distinguish:

- ordinary distributed drift
- historical-only interpretation
- insufficient evidence
- deterministic model violation

without collapsing all disagreement into one undifferentiated mismatch class.
