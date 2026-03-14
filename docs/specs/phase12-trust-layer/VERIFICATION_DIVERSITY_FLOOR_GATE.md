# Verification Diversity Floor Gate

**Version:** 0.1  
**Status:** Initial implementation (Phase-13 collapse-horizon harness)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Gate contract note  
**Target:** `ci-gate-verification-diversity-floor`
**Related Spec:** `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `PHASE13_COLLAPSE_SCENARIOS.md`, `VERIFICATION_INVARIANTS.md`, `PHASE13_ARCHITECTURE_MAP.md`, `GATE_REGISTRY.md`

---

## 1. Purpose

This gate detects Verification Gravity Collapse before explicit authority election or consensus semantics appear.

The gate enforces the behavioral reading:

`verification scheduling must preserve diversity`

Its role is not to validate local correctness of a single run.

Its role is to detect concentration drift across a multi-run horizon.

---

## 2. Protected Risk

Primary risk class:

- `verification-gravity-drift`

Protected failure meaning:

- verification behavior has concentrated below an acceptable verifier-diversity floor

This is a collapse-horizon harness, not a schema gate.

---

## 3. Required Inputs

The gate is expected to consume:

- `Verification Diversity Ledger` window artifacts
- authority-chain distribution derived from VDL entries
- lineage distribution derived from VDL entries

Recommended input set:

- `vdl_window.json`
- `diversity_metrics.json`
- `lineage_distribution.json`
- `cluster_distribution.json`
- `dominance_analysis.json`
- `entropy_report.json`

These artifacts are derived from the VDL.

They are not independent truth surfaces.

---

## 4. Window Model

The preferred model is dual-window evaluation:

- run window
- time window

Example:

- `window_runs = 200`
- `window_time = 24h`

This is preferred because it exposes both:

- short burst concentration
- longer-horizon gravity collapse

Subject-scoped and context-scoped windows MAY also be used when concentration appears localized.

---

## 5. Required Metrics

The minimum metric set should include:

- `unique_verifier_count`
- `unique_verification_node_count`
- `unique_authority_chain_count`
- `unique_lineage_count`
- `dominance_ratio`
- `lineage_entropy`

Recommended extended metrics:

- `pairwise_verdict_correlation`
- `lineage_dominance_ratio`
- `authority_chain_dominance_ratio`
- `verification_node_dominance_ratio`
- `execution_cluster_dominance_ratio`

These metrics remain descriptive diagnostics only.

They MUST NOT be used as routing, authority, or scheduling input.

---

## 6. Threshold Policy Separation

The gate must consume threshold policy from a separate policy surface.

The VDL itself MUST NOT encode thresholds.

Possible policy sentences:

- `min_unique_verifiers >= 3`
- `min_unique_verification_nodes >= 3`
- `max_dominance_ratio <= 0.40`
- `min_lineage_entropy >= 1.2`

Threshold policy must remain independent so that:

`artifact != policy`

remains true.

---

## 7. Example Evaluation Model

The intended evaluation sequence is:

1. load dual-window VDL slice
2. derive verifier, node, authority-chain, and lineage distributions
3. compute dominance and entropy metrics
4. compare against policy thresholds
5. emit fail-closed evidence if diversity floor is violated

The shortest operational reading is:

`local validity is not enough if the verification population has behaviorally collapsed`

---

## 8. Expected Outputs

The gate should export:

- `report.json`
- `vdl_window.json`
- `diversity_metrics.json`
- `lineage_distribution.json`
- `cluster_distribution.json`
- `dominance_analysis.json`
- `entropy_report.json`
- `violations.txt`

`report.json` is the CI summary surface.

The other artifacts are the behavioral evidence surface.

---

## 9. Non-Goals

This gate does not:

- elect authority
- rank verifiers by trust
- create routing hints
- recommend preferred clusters
- replace current kill-switch gates

It only detects diversity collapse.

It does not prove verifier independence once cartel-style correlation becomes the dominant risk.

That later blind spot is handled by:

- `VERIFIER_CARTEL_CORRELATION_GATE.md`

---

## 10. Short Rule

The shortest correct reading is:

`distributed verification must remain behaviorally diverse, not only nominally distributed`
