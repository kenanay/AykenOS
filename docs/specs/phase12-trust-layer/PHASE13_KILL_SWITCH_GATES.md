# Phase-13 Kill-Switch Gates

**Version:** 1.0  
**Status:** Draft (Phase-13 boundary hardening)  
**Date:** 2026-03-13  
**Type:** Normative kill-switch profile

---

## 1. Purpose

Phase-13 should not rely on a large number of equal-weight gates.

It should rely on a small number of architectural kill-switch gates.

These gates exist for one reason:

`if category identity starts breaking, the build dies immediately`

They do not test correctness of individual code paths.

They detect category drift of the system as a whole.

So the right model is:

- many technical checks
- few architectural kill switches

---

## 2. The Four Kill Switches

### 2.1 Observability -> Control Plane Kill Switch

- Invariant: `observability != scheduling`
- Risk classes: `topology-feedback-drift`, `control-plane-drift`
- Primary gate: `ci-gate-observability-routing-separation`
- Supporting gates:
  - `ci-gate-proofd-observability-boundary`
  - `ci-gate-diagnostics-consumer-non-authoritative-contract`
  - `ci-gate-diagnostics-callsite-correlation`
- Authoritative failure meaning:
  - observability artifacts or service surfaces have started steering verification behavior

This kill switch prevents:

`diagnostics -> routing bias -> implicit authority`

### 2.2 Authority Election Kill Switch

- Invariant: `truth is computed, not elected`
- Risk class: `truth-election-drift`
- Primary gate: `ci-gate-convergence-non-election-boundary`
- Supporting gates:
  - `ci-gate-graph-non-authoritative-contract`
  - `ci-gate-cross-node-parity`
- Authoritative failure meaning:
  - parity, graph, or convergence surfaces have started turning distributed agreement shape into truth selection

This kill switch prevents:

`majority -> canonical truth`

### 2.3 Verification Artifact Integrity Kill Switch

- Invariant: `artifacts = canonical interface`
- Risk class: `artifact-truth-drift`
- Primary gate: `ci-gate-proof-verdict-binding`
- Supporting gates:
  - `ci-gate-proof-bundle`
  - `ci-gate-proof-receipt`
  - `ci-gate-proofd-service`
- Authoritative failure meaning:
  - verification truth has stopped being artifact-bound and is drifting toward runtime, cache, or transport state

This kill switch prevents:

`state-driven truth`

### 2.4 Verifier Authority Drift Kill Switch

- Invariant: `valid receipt != trusted verifier`
- Risk class: `authority-drift`
- Primary gate: `ci-gate-verifier-authority-resolution`
- Supporting gates:
  - `ci-gate-verifier-reputation-prohibition`
  - `ci-gate-observability-routing-separation`
  - `ci-gate-cross-node-parity`
- Authoritative failure meaning:
  - valid receipt semantics are no longer distinct from trusted verifier authority semantics

This kill switch prevents:

`verifier cluster dominance -> de facto authority`

---

## 3. Why These Four

Together these four kill switches preserve:

1. observability does not steer the system
2. truth is computed rather than elected
3. truth remains artifact-bound
4. verifier authority remains separate from mere verification result validity

If these four boundaries hold, AykenOS remains a:

`deterministic distributed verification system`

If they fail, AykenOS drifts toward:

`distributed consensus behavior`

---

## 4. Primary-Gate Rule

Each kill-switch invariant should have exactly one primary gate.

Other gates may support it, but they should be reported as:

- supporting evidence
- not independent architectural root causes

The intended CI language is:

- `FAIL: observability -> control plane`
  - primary: `ci-gate-observability-routing-separation`
  - support: `ci-gate-proofd-observability-boundary`
  - support: `ci-gate-diagnostics-callsite-correlation`
- `FAIL: authority election`
  - primary: `ci-gate-convergence-non-election-boundary`
  - support: `ci-gate-graph-non-authoritative-contract`

That reporting style is required to prevent Gate Explosion 2.0.

CI summary should therefore reduce gate results into:

- kill-switch category
- trigger path (`PRIMARY_GATE` or `SUPPORTING_GATE`)
- primary gate
- supporting gates

When architectural completeness is required, missing expected kill-switch gates should fail as missing architectural coverage rather than being silently treated as neutral.

---

## 5. Short Rule

The shortest correct Phase-13 reading is:

`few primary kill switches, many supporting checks`

If AykenOS keeps that structure, architectural failures stay readable.
