# Authority Sinkhole Absorption Gate

**Version:** 0.1  
**Status:** Draft (Phase-13 reserved collapse-horizon harness)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Gate contract note  
**Target:** `ci-gate-authority-sinkhole-absorption`  
**Related Spec:** `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `PHASE13_COLLAPSE_SCENARIOS.md`, `VERIFICATION_INVARIANTS.md`, `GATE_REGISTRY.md`

---

## 1. Purpose

This future gate detects Verification Basin Collapse.

The collapse appears when verification reuse, replay review, or trust reuse keep falling into one practical authority basin even without explicit authority election.

The shortest correct reading is:

`operational reuse must not collapse into one practical authority basin`

---

## 2. Protected Risk

Primary risk class:

- `authority-sinkhole-drift`

Protected failure meaning:

- verification and replay-boundary flows are being absorbed into one practical authority basin through repeated reuse and operational convenience

This is a system-dynamics harness, not a schema gate.

---

## 3. Verification Basin Collapse

The common path is:

`slight reuse advantage -> repeated reuse -> practical basin preference -> one basin absorbs future traffic`

The dangerous property is gradual irreversibility.

The system may retain nominal topology width while practical verification traffic increasingly converges into one basin.

This is not explicit consensus.

It is operational absorption.

---

## 4. Why Earlier Gates May Still Pass

Earlier gates may still pass because:

- diversity floor may still appear acceptable
- cartel correlation may remain below threshold
- observability routing rules may not be directly violated
- no explicit authority election field may exist

So the earlier gates can still say:

- distribution still exists
- independence still exists

while the temporal flow shape is already collapsing.

The shortest rule is:

`distribution health and independence health do not by themselves prove basin health`

---

## 5. Required Inputs

The expected inputs are:

- Verification Diversity Ledger windows
- authority-chain distribution
- replay-boundary flow evidence where available
- trust-reuse flow evidence where available
- optional authority-topology companion artifacts

Recommended evidence set:

- `vdl_window.json`
- `dominance_analysis.json`
- `authority_chain_flow_report.json`
- `basin_absorption_report.json`
- `basin_window_series.json`
- `violations.txt`

These remain observability artifacts only.

They MUST NOT become authority or routing outputs.

---

## 6. Core Metrics

The initial basin-collapse metric family should include:

- `authority_basin_share`
- `authority_basin_reuse_ratio`
- `authority_basin_repeat_capture_rate`
- `alternate_path_decay_ratio`
- `basin_dominance_slope`

The operational goal is:

`detect slow authority-basin absorption before explicit authority collapse appears`

---

## 7. Example Detection Shapes

Representative suspicious cases:

1. the same `authority_chain_id` repeatedly becomes the terminal verification basin across windows
2. alternate replay-review or trust-reuse paths remain present but stop receiving meaningful flow
3. a practical basin keeps winning future verification reuse despite nominal topology width
4. basin share rises steadily even while verifier diversity remains above floor

Typical pattern:

- `authority_basin_share_window_1 = 0.22`
- `authority_basin_share_window_2 = 0.31`
- `authority_basin_share_window_3 = 0.44`
- `authority_basin_share_window_4 = 0.58`

This suggests basin collapse long before explicit authority election appears.

---

## 8. Expected Outputs

The future gate should export:

- `report.json`
- `authority_chain_flow_report.json`
- `basin_absorption_report.json`
- `basin_window_series.json`
- `violations.txt`

`report.json` remains the CI verdict surface.

The other artifacts are temporal forensic evidence.

---

## 9. Non-Goals

This gate does not:

- elect authority
- recommend preferred basins
- create routing hints
- override replay-boundary policy
- replace diversity or cartel gates

It only answers:

`is verification reuse collapsing into one authority basin over time?`

---

## 10. Short System Model

The larger collapse sequence is:

`diversity floor -> cartel correlation -> basin collapse`

These correspond to:

- distribution health
- independence health
- temporal absorption health

---

## 11. Short Rule

The shortest correct reading is:

`a distributed verifier network can remain nominally wide while operational flow collapses into one authority basin`
