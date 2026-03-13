# Verifier Reputation Prohibition Gate

**Version:** 1.0
**Status:** Draft (Phase-13 preparation)
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Gate contract note
**Target:** `ci-gate-verifier-reputation-prohibition`
**Related Spec:** `PHASE13_NEGATIVE_TEST_SPEC.md`, `VERIFICATION_INVARIANTS.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `PHASE13_ARCHITECTURE_MAP.md`

---

## 1. Purpose

This gate blocks hidden verifier reputation semantics from entering Phase-13 observability artifacts.

The gate enforces:

`verification history != verifier reputation`

It exists because graph analytics can drift into implicit authority scoring without changing the core verification function.

---

## 2. Required Inputs

The gate validates the following diagnostics artifacts:

- `parity_report.json`
- `parity_determinism_incidents.json`
- `parity_drift_attribution_report.json`
- `parity_convergence_report.json`
- `parity_authority_drift_topology.json`
- `parity_authority_suppression_report.json`
- `parity_incident_graph.json`

By default the gate bootstraps these artifacts via the local cross-node parity harness.

For tests or local contract checks, an explicit `--artifact-root` may be provided.

---

## 3. Forbidden Payload Fields

Examples of exact forbidden fields:

- `verifier_score`
- `trust_score`
- `reliability_index`
- `weighted_authority`
- `correctness_rate`
- `agreement_ratio`
- `node_success_ratio`
- `verifier_reputation`
- `historical_correctness_index`
- `authority_alignment_score`
- `dominant_verifier_frequency`
- `convergence_leadership_score`

Pattern-based forbidden fields also fail closed when they imply:

- verifier reputation
- node reliability
- historical correctness
- weighted authority
- leaderboard or ranking semantics

---

## 4. Violation Matrix

The gate currently enforces these Phase-13 negative cases:

- `P13-NEG-15`
  Payload exposes verifier reputation or scoring outputs
- `P13-NEG-16`
  Verification history is transformed into implicit authority ranking

Any hit against these cases produces a gate failure.

---

## 5. Outputs

The gate exports:

- `report.json`
- `reputation_prohibition_report.json`
- `violations.txt`
- `meta.txt`

`report.json` is the CI summary surface.

`reputation_prohibition_report.json` is the detailed contract report.

---

## 6. Make Target

```bash
make ci-gate-verifier-reputation-prohibition
```

This target is intentionally standalone for now.

It is Phase-13 boundary enforcement, not yet part of the strict freeze chain.
