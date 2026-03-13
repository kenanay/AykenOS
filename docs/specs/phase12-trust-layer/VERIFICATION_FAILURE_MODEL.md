# Verification Failure Model

**Version:** 1.0
**Status:** Informational failure model
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative failure taxonomy artifact
**Related Spec:** `VERIFICATION_MODEL.md`, `VERIFICATION_INVARIANTS.md`, `VERIFICATION_OBSERVABILITY_MODEL.md`, `ARTIFACT_SCHEMA.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PARITY_GRAPH_MODEL.md`

---

## 1. Purpose

This document defines the compact failure taxonomy for AykenOS verification.

Its role is to classify failure as a structured model rather than an undifferentiated error surface.

The central rule is:

`verification failure must be attributable`

---

## 2. Failure Classes

AykenOS treats the following as primary failure classes:

- subject drift
- context drift
- authority drift
- artifact loss
- determinism violation

These classes are sufficient to explain most verification and cross-node comparison failures at the current architecture boundary.

---

## 3. Subject Drift

Subject drift means the compared or evaluated verification subject is not the same object.

Typical causes:

- `bundle_id` mismatch
- `trust_overlay_hash` mismatch
- `policy_hash` mismatch
- `registry_snapshot_hash` mismatch

Architectural rule:

`different subject => no deterministic identity claim`

---

## 4. Context Drift

Context drift means verification is being interpreted under different rules.

Typical causes:

- `verification_context_id` mismatch
- different policy material
- different registry material
- different verifier contract version
- different context-rules material

Architectural rule:

`same artifact + different context != same verification meaning`

---

## 5. Authority Drift

Authority drift means the trust-bearing verifier interpretation is not the same across evaluations.

Typical causes:

- registry lineage mismatch
- authority scope mismatch
- different `authority_chain_id`
- trusted on one side, historical or invalid on the other

Architectural rule:

`valid receipt != equal authority interpretation`

---

## 6. Artifact Loss

Artifact loss means verification or distributed comparison lacks required evidence artifacts.

Typical causes:

- missing receipt
- missing manifest
- missing verification context object
- missing verifier registry snapshot
- missing diagnostics artifact required for comparison

Architectural rule:

`missing evidence => fail closed or insufficient evidence`

Artifact loss is not the same as semantic disagreement.

It is an evidence-availability failure.

---

## 7. Determinism Violation

Determinism violation means the same effective input surface does not yield the same verdict.

This is the most severe semantic failure.

Formal condition:

`Q_1 = Q_2 and Eval(Q_1) != Eval(Q_2)`

Typical causes:

- hidden input drift
- nondeterministic implementation behavior
- unstable authority resolution
- unstable context interpretation

Architectural rule:

`determinism violation = semantic integrity failure`

---

## 8. Taxonomy Interpretation

These classes separate three kinds of problems:

- input mismatch
- evidence insufficiency
- semantic failure

Mapping:

- subject drift, context drift, authority drift
  - input mismatch
- artifact loss
  - evidence insufficiency
- determinism violation
  - semantic failure

This separation matters because not every failure should be handled as a trust or implementation bug.

---

## 9. Relation To Parity

Cross-node parity builds on this taxonomy.

Parity mismatch classes such as:

- subject mismatch
- context mismatch
- verifier mismatch
- insufficient evidence
- verdict mismatch

are operational surfaces over the same underlying failure model.

So the failure taxonomy is broader than parity labeling.

Parity is one consumer of it.

### 9.1 Parity Label Mapping

| Parity Label | Underlying Failure Class |
|---|---|
| `PARITY_SUBJECT_MISMATCH` | subject drift |
| `PARITY_CONTEXT_MISMATCH` | context drift |
| `PARITY_VERIFIER_MISMATCH` | authority drift |
| `PARITY_INSUFFICIENT_EVIDENCE` | artifact loss |
| `PARITY_VERDICT_MISMATCH` | determinism violation, assuming the same effective `(S, C, A)` surface |

Additional interpretation notes:

- `PARITY_MATCH`
  - no failure class is active
- `PARITY_HISTORICAL_ONLY`
  - historical interpretation boundary, not one of the primary failure classes above

---

## 10. Summary

The compact AykenOS verification failure model is:

- subject drift
- context drift
- authority drift
- artifact loss
- determinism violation

The model exists to keep failure diagnosis explicit, fail-closed, and semantically interpretable.
