# Cross-Node Parity Hardening Checklist

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative implementation checklist
**Related Spec:** `requirements.md`, `tasks.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`, `VERIFICATION_CONVERGENCE_THEOREM.md`, `AYKENOS_DISTRIBUTED_TRUTH_MODEL_FORMAL_SECURITY_PROPERTIES.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`

---

## 1. Purpose

This document turns the current Phase-12 parity model into an executable hardening checklist for `P12-14`.

The goal is not “more tests”.

The goal is to force the cross-node parity implementation to exercise the actual distributed truth surfaces:

- `S` = subject
- `C` = context
- `A` = authority
- `V` = local verdict

This checklist is the implementation bridge between:

- the formal parity object
  - `P = (S, C, A, V)`
- the convergence theorem
- the current local `ci-gate-cross-node-parity` evidence path

---

## 2. Core Invariant

The main parity invariant remains:

`same normalized (S, C, A) -> same V -> PARITY_MATCH`

The negative form is equally important:

`drift in S or C or A or V -> no PARITY_MATCH`

This checklist exists to ensure that the current gate proves both directions.

---

## 3. Status Set Under Test

The minimum parity status set that the hardening matrix MUST exercise is:

- `PARITY_MATCH`
- `PARITY_SUBJECT_MISMATCH`
- `PARITY_CONTEXT_MISMATCH`
- `PARITY_VERIFIER_MISMATCH`
- `PARITY_VERDICT_MISMATCH`
- `PARITY_HISTORICAL_ONLY`
- `PARITY_INSUFFICIENT_EVIDENCE`

If a later implementation needs a stricter authority ambiguity surface, it MAY add:

- `PARITY_AUTHORITY_AMBIGUOUS`

but the current checklist does not require a new status if the same condition is already fail-closed into:

- `PARITY_VERIFIER_MISMATCH`
- `PARITY_INSUFFICIENT_EVIDENCE`

---

## 4. Scenario Matrix

### Group A - Baseline / Determinism

#### P14-01 Baseline Identical Nodes

- Node A and Node B use the same bundle, same context, same authority, and same verifier contract version
- Expected:
  - `s_equal = true`
  - `c_equal = true`
  - `a_equal = true`
  - `v_equal = true`
  - `actual_status = PARITY_MATCH`

#### P14-02 Repeat-Run Determinism

- The same node verifies the same input twice before parity comparison against another node
- Expected:
  - stable parity tuple across repeated local runs
  - `actual_status = PARITY_MATCH`

#### P14-03 Serialization Noise Only

- JSON field order, whitespace, or formatting differ while canonical content remains equal
- Expected:
  - canonical recomputation preserves equality
  - `actual_status = PARITY_MATCH`

### Group B - Subject Drift

#### P14-04 Bundle Tamper

- Portable payload has checksum or `bundle_id` mismatch
- Expected:
  - `s_equal = false`
  - `actual_status = PARITY_SUBJECT_MISMATCH` or explicit local invalid precondition block

#### P14-05 Overlay Hash Drift With Same Bundle

- `bundle_id` is equal but `trust_overlay_hash` differs
- Expected:
  - subject tuple equality breaks
  - `actual_status != PARITY_MATCH`

#### P14-06 Receipt Subject Mismatch

- Receipt `bundle_id` or another subject tuple field is tampered
- Expected:
  - receipt binding fails
  - parity match forbidden

#### P14-07 Same Portable Payload, Different Verdict Subject Tuple

- Bundle bytes are equal but `policy_hash` or `registry_snapshot_hash` differs
- Expected:
  - portable payload alone is insufficient
  - `actual_status != PARITY_MATCH`

### Group C - Context Drift

#### P14-08 Policy Drift, Same Payload

- Bundle is equal but policy differs
- Expected:
  - `c_equal = false`
  - `actual_status = PARITY_CONTEXT_MISMATCH`

#### P14-09 Registry Drift, Same Payload

- Bundle is equal but producer registry snapshot differs
- Expected:
  - `c_equal = false`
  - `actual_status = PARITY_CONTEXT_MISMATCH`

#### P14-10 `verification_context_id` Drift

- The visible payload appears similar but declared context identity differs
- Expected:
  - direct context mismatch
  - canonical context recomputation path is exercised

#### P14-11 Context Rules Drift

- `context_rules_hash` changes
- Expected:
  - `verification_context_id` changes as well
  - silent parity is forbidden

#### P14-12 Verifier Contract Version Drift

- Verifier contract versions differ under otherwise comparable inputs
- Expected:
  - explicit compatibility required
  - otherwise `c_equal = false`
  - `actual_status != PARITY_MATCH`

### Group D - Authority Drift

#### P14-13 Different Trusted Root Set

- Subject and context are equal but authority root sets differ
- Expected:
  - `a_equal = false`
  - `actual_status = PARITY_VERIFIER_MISMATCH`

#### P14-14 Delegation Chain Drift

- Same verifier node but different delegation path
- Expected:
  - `authority_chain_id_equal = false`
  - parity forbidden

#### P14-15 Authority Scope Drift

- Delegation exists but effective scope differs
- Expected:
  - `effective_authority_scope_equal = false`
  - parity forbidden

#### P14-16 Historical Versus Current Authority

- Node A resolves current authority, Node B resolves historical-only authority
- Expected:
  - `actual_status = PARITY_HISTORICAL_ONLY` or explicit mismatch class
  - definitely not `PARITY_MATCH`

#### P14-17 Ambiguous Authority Graph

- One side detects authority ambiguity
- Expected:
  - fail-closed authority rejection
  - parity forbidden

### Group E - Verdict Drift

#### P14-18 Same `T`, Different `V` Forbidden Test

- Intentionally attempt to force different local verdicts under the same normalized `(S, C, A)`
- Expected:
  - model violation if this ever produces `PARITY_MATCH`
  - this scenario guards the deterministic evaluation property

#### P14-19 Insufficient Evidence Versus Resolved Evidence

- Node A resolves the full context, Node B remains incomplete
- Expected:
  - `actual_status = PARITY_INSUFFICIENT_EVIDENCE`
  - this is not a theorem violation

#### P14-20 Receipt Absent But Parity Artifact Present

- No receipt is present, but local verification outputs are still compared
- Expected:
  - the parity artifact contract is explicit
  - no implicit match based on missing receipt transport

---

## 5. Recommended Rollout

### PR1 - Baseline Hardening Slice

Implement first:

- `P14-01 Baseline Identical Nodes`
- `P14-08 Policy Drift`
- `P14-13 Different Trusted Root Set`

Invariant:

- subject, context, and authority mismatch classes are proven on the current gate path

### PR2 - Historical / Evidence Slice

Implement next:

- `P14-16 Historical Versus Current Authority`
- `P14-19 Insufficient Evidence Versus Resolved Evidence`
- `P14-20 Receipt Absent But Parity Artifact Present`

Invariant:

- the gate can classify historical-only and insufficient-evidence surfaces without collapsing into generic mismatch

### PR3 - Full Matrix Expansion

Implement after that:

- remaining 20-scenario matrix
- scenario-specific JSON evidence
- full matrix aggregation in the parity gate

Invariant:

- the theorem set is exercised as an executable parity matrix instead of isolated hand-written scenarios

---

## 6. Evidence Layout

Recommended evidence layout:

```text
evidence/run-<id>/gates/proof-parity-suite/
  parity_matrix.json
  parity_report.json
  parity_consistency_report.json
  parity_determinism_report.json
  parity_determinism_incidents.json
  parity_convergence_report.json
  parity_drift_attribution_report.json
  scenario_reports/
    p14-01-baseline.json
    p14-02-repeat-run.json
    ...
    p14-20-receipt-absent.json
  violations.txt
  report.json
```

Each parity-matrix row SHOULD contain at least:

```json
{
  "scenario": "p14-08-policy-drift",
  "s_equal": true,
  "c_equal": false,
  "a_equal": true,
  "v_equal": false,
  "expected_status": "PARITY_CONTEXT_MISMATCH",
  "actual_status": "PARITY_CONTEXT_MISMATCH",
  "pass": true
}
```

---

## 7. Implementation Notes

This checklist intentionally separates three concerns:

- verifier engine
- parity comparison logic
- distributed service behavior

Therefore:

- parity hardening SHOULD extend the current harness/gate/evidence path first
- it SHOULD NOT force network or `proofd` behavior into the parity gate early
- it SHOULD preserve the current architecture:
  - verifier core = deterministic evaluation
  - parity gate = executable comparison surface
  - `proofd` = later service layer

Current local implementation note:

- the active gate now emits `parity_drift_attribution_report.json` from node-derived `NodeParityOutcome` partitions so the current matrix explains why nodes disagree instead of only counting mismatches
- the active drift-attribution artifact now also emits `historical_authority_islands` and `insufficient_evidence_islands` summaries so cluster-level epoch lag and evidence lag remain visible even before `proofd`
- the active gate now also emits `parity_determinism_incidents.json`, turning same-surface verdict divergence into explicit incident artifacts instead of only aggregate determinism counts

---

## 8. Current Priority

If the matrix is not implemented all at once, the highest-signal first set is:

1. `P14-01 Baseline Identical Nodes`
2. `P14-08 Policy Drift`
3. `P14-13 Different Trusted Root Set`
4. `P14-16 Historical Versus Current Authority`
5. `P14-19 Insufficient Evidence Versus Resolved Evidence`

This five-scenario slice exercises the main truth surfaces without prematurely expanding into full service semantics.

Local implementation note as of 2026-03-09:

- the active local gate already covers `P14-01`, `P14-05`, `P14-10`, `P14-12`, `P14-13`, `P14-15`, `P14-16`, `P14-18`, `P14-19`, and `P14-20`
- `verification_context_id` parity comparisons now use the same canonical context-object path as exchange validation
- the receipt-absent artifact contract is explicit and currently uses `local_verification_outcome`
- local parity reporting is now split into `parity_consistency_report.json` and `parity_determinism_report.json`
- the active local gate now also emits `parity_determinism_incidents.json` as a first-class node-derived determinism incident surface
- the active local gate now also emits `parity_convergence_report.json` as a node-derived aggregate over stable `NodeParityOutcome` objects, while still preserving the pairwise matrix as the raw classifier surface

---

## 9. Summary

`P12-14` should no longer be interpreted as “add a few parity tests”.

It should be implemented as:

`formal parity semantics -> executable drift matrix`

That is the shortest path from the current theorem set to measurable distributed truth behavior.
