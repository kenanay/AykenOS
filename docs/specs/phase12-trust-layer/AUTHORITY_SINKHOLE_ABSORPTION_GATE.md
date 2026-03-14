# Authority Sinkhole Absorption Gate

**Version:** 0.1  
**Status:** Implemented (Phase-13 Stage-1 harness with optional Stage-2 cross-surface alignment metrics)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Gate contract note  
**Target:** `ci-gate-authority-sinkhole-absorption`  
**Related Spec:** `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`, `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`, `CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`, `PHASE13_COLLAPSE_SCENARIOS.md`, `VERIFICATION_INVARIANTS.md`, `GATE_REGISTRY.md`

---

## 1. Purpose

This gate detects Verification Basin Collapse.

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

The Stage-1 harness consumes:

- Verification Diversity Ledger windows
- authority-chain distribution
- bounded basin-window series derived from the Verification Diversity Ledger

Stage-1 is intentionally VDL-only.

Future versions may additionally consume:

- replay-boundary flow evidence where available
- trust-reuse flow evidence where available
- optional authority-topology companion artifacts

The canonical Stage-2 companion contract is:

- `AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`
- `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`
- `CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`

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

### 6.1 Stage-1 Mathematical Definitions

Stage-1 uses bounded Verification Diversity Ledger windows only.

Let:

- `E` = selected verification events in the bounded window
- `|E|` = selected entry count
- `B` = reference basin authority chain selected as the current dominant authority chain by bounded share
- `S` = subject groups keyed by `(subject_bundle_id, verification_context_id)`
- `R` = repeated subject groups where `event_count >= 2`
- `A` = alternate-path subject groups in `R` where more than one authority chain appears

Then Stage-1 metrics are defined as:

- `authority_basin_share = |{ e in E : authority_chain_id(e) = B }| / |E|`
- `authority_basin_reuse_ratio = |{ s in R : terminal_authority_chain(s) = B }| / |R|`
- `authority_basin_repeat_capture_rate = |{ s in R : terminal_authority_chain(s) = B and count_B(s) >= 2 }| / |R|`
- `alternate_path_decay_ratio = |{ s in A : terminal_authority_chain(s) != B }| / |A|`
- `basin_dominance_slope = (share_last(B) - share_first(B)) / (window_count - 1)`

Where:

- `terminal_authority_chain(s)` is the authority chain attached to the latest event in subject group `s`
- `count_B(s)` is the number of events in subject group `s` whose authority chain is `B`
- `share_first(B)` and `share_last(B)` are the reference-basin shares in the first and last basin-series windows

Stage-1 therefore measures:

- bounded basin dominance
- terminal subject capture
- repeated capture by the same basin
- decay of alternate terminal paths
- simple temporal drift toward the same basin

### 6.2 Reference Basin Selection

Stage-1 selects the reference basin as the highest-share authority chain in the bounded window.

This is intentionally simple.

Later stages may refine basin selection using:

- dominant-by-terminal-capture
- dominant-by-repeat-capture
- cross-surface dominant-basin agreement

So the correct reading is:

`Stage-1 identifies the current practical basin, not a final authority`

### 6.3 Stage-2 Companion Metrics

Stage-2 should extend Stage-1 by consuming replay-boundary and trust-reuse companion evidence where available.

The first Stage-2 metric family should include:

- `replay_boundary_basin_capture_ratio`
- `replay_boundary_repeat_capture_rate`
- `trust_reuse_basin_capture_ratio`
- `trust_reuse_repeat_capture_rate`
- `cross_surface_basin_alignment_ratio`
- `cross_surface_alternate_path_decay_ratio`
- `cross_surface_basin_slope`

The purpose is to answer a stronger question:

`is the same practical basin absorbing verification flow and replay or trust reuse flow at the same time?`

### 6.4 Stage-2 Companion Evidence

The recommended Stage-2 companion evidence set is:

- `replay_boundary_flow_report.json`
- `trust_reuse_flow_report.json`
- `cross_surface_basin_alignment_report.json`

Those companion surfaces are normatively defined in:

- `AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`
- `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`
- `CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`

The minimum expected companion event fields are:

- `timestamp_unix_ns`
- `subject_bundle_id`
- `verification_context_id`
- `authority_chain_id`
- `flow_surface`
- `terminal`
- `reused`

Where `flow_surface` should distinguish:

- `verification`
- `replay_boundary`
- `trust_reuse`

These remain observability artifacts only.

They MUST NOT be consumed as routing or authority outputs.

### 6.5 Metric Evolution Rule

The sinkhole harness must evolve conservatively.

Short rule:

`Stage-1 proves bounded basin absorption from VDL alone; Stage-2 proves cross-surface basin absorption using replay and trust-reuse companion evidence`

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

The gate exports:

- `report.json`
- `authority_sinkhole_absorption_report.json`
- `vdl_window.json`
- `dominance_analysis.json`
- `authority_chain_flow_report.json`
- `basin_absorption_report.json`
- `basin_window_series.json`
- `cross_surface_basin_alignment_report.json`
- `violations.txt`

`report.json` remains the CI verdict surface.

The other artifacts are temporal forensic evidence.

`cross_surface_basin_alignment_report.json` is emitted with:

- `NOT_EVALUATED` when Stage-2 companion flow evidence is absent
- `PASS` when Stage-2 companion evidence is present and below threshold
- `FAIL` when Stage-2 companion evidence shows cross-surface basin absorption beyond policy

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
