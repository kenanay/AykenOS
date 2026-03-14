# Cross-Surface Basin Alignment Metrics

**Version:** 0.1  
**Status:** Implemented (Phase-13 Stage-2 metric model when companion flow evidence is present)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Normative metric definition note

**Related Spec:** `AUTHORITY_SINKHOLE_ABSORPTION_GATE.md`, `AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`, `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`

---

## 1. Purpose

This document defines the Stage-2 metric family for cross-surface basin alignment.

Its purpose is to turn:

- verification-flow evidence
- replay-boundary companion flow evidence
- trust-reuse companion flow evidence

into explicit, comparable basin-alignment metrics.

The shortest rule is:

`Stage-2 sinkhole analysis is not only about basin share; it is about the same basin absorbing multiple operational surfaces`

---

## 2. Shared Symbols

Let:

- `B` = current reference basin authority chain
- `K` = subject key `(subject_bundle_id, verification_context_id)`
- `V` = terminal verification events from bounded VDL windows
- `P` = terminal replay-boundary events from `replay_boundary_flow_report.json`
- `T` = terminal trust-reuse events from `trust_reuse_flow_report.json`

For each surface:

- `surface_basin(K)` returns the terminal `authority_chain_id` for subject key `K`

Only terminal events are used for alignment metrics.

If a surface lacks a terminal event for a subject key, that key is not comparable on that surface.

---

## 3. Surface-Local Basin Capture Metrics

### 3.1 Replay-Boundary Basin Capture Ratio

`replay_boundary_basin_capture_ratio = |{ K in P : surface_basin_P(K) = B }| / |P|`

This answers:

`how often does replay-boundary flow terminate inside the current reference basin?`

### 3.2 Trust-Reuse Basin Capture Ratio

`trust_reuse_basin_capture_ratio = |{ K in T : surface_basin_T(K) = B }| / |T|`

This answers:

`how often does trust-reuse flow terminate inside the current reference basin?`

### 3.3 Replay-Boundary Repeat Capture Rate

Let `P_r` be replay-boundary events where `reused = true`.

`replay_boundary_repeat_capture_rate = |{ K in P_r : surface_basin_P(K) = B }| / |P_r|`

This isolates operational reuse rather than first-pass observation.

### 3.4 Trust-Reuse Repeat Capture Rate

Let `T_r` be trust-reuse events where `reused = true`.

`trust_reuse_repeat_capture_rate = |{ K in T_r : surface_basin_T(K) = B }| / |T_r|`

This detects repeated return to the same practical basin.

---

## 4. Cross-Surface Alignment Metrics

### 4.1 Verification-to-Replay Alignment Ratio

Let:

- `C_vp = { K : K in V and K in P }`

Then:

`verification_replay_alignment_ratio = |{ K in C_vp : surface_basin_V(K) = surface_basin_P(K) = B }| / |C_vp|`

This asks:

`for subjects observable on both surfaces, how often do verification and replay terminate in the same reference basin?`

### 4.2 Verification-to-Trust Alignment Ratio

Let:

- `C_vt = { K : K in V and K in T }`

Then:

`verification_trust_alignment_ratio = |{ K in C_vt : surface_basin_V(K) = surface_basin_T(K) = B }| / |C_vt|`

### 4.3 Replay-to-Trust Alignment Ratio

Let:

- `C_pt = { K : K in P and K in T }`

Then:

`replay_trust_alignment_ratio = |{ K in C_pt : surface_basin_P(K) = surface_basin_T(K) = B }| / |C_pt|`

### 4.4 Cross-Surface Basin Alignment Ratio

Let:

- `C_all = { K : K in V and K in P and K in T }`

Then:

`cross_surface_basin_alignment_ratio = |{ K in C_all : surface_basin_V(K) = surface_basin_P(K) = surface_basin_T(K) = B }| / |C_all|`

This is the strongest Stage-2 alignment metric.

It asks:

`are verification, replay-boundary, and trust-reuse all collapsing into the same practical basin for the same subjects?`

---

## 5. Alternate-Path Decay Metrics

### 5.1 Cross-Surface Alternate-Path Candidate Set

Let:

- `A_all = { K in C_all : distinct(surface_basin_V(K), surface_basin_P(K), surface_basin_T(K)) > 1 }`

These are keys where alternate paths still exist across surfaces.

### 5.2 Cross-Surface Alternate-Path Decay Ratio

`cross_surface_alternate_path_decay_ratio = |{ K in A_all : at least one surface terminates outside B }| / |A_all|`

Low values indicate that alternate paths are no longer surviving as terminal paths on any meaningful surface.

This is a stronger decay signal than Stage-1.

Stage-1 only sees alternate-path decay inside verification flow.

Stage-2 asks whether alternates still survive anywhere across the operational surfaces.

---

## 6. Temporal Alignment Metrics

### 6.1 Basin Alignment Slope

Let:

- `align_first(B)` = `cross_surface_basin_alignment_ratio` in the first evaluated window
- `align_last(B)` = `cross_surface_basin_alignment_ratio` in the last evaluated window
- `n` = evaluated window count

Then:

`basin_alignment_slope = (align_last(B) - align_first(B)) / (n - 1)`

This detects slow tightening of the same basin across surfaces.

### 6.2 Surface-Capture Slope

The same slope construction may be applied independently to:

- `replay_boundary_basin_capture_ratio`
- `trust_reuse_basin_capture_ratio`

This helps distinguish:

- verification-only basin drift
- replay-boundary-only drift
- trust-reuse-only drift
- genuine cross-surface sinkhole formation

---

## 7. Operational Reading

The Stage-2 reading should be:

- first prove surface-local basin capture
- then prove surface-pair alignment
- then prove three-surface alignment
- only then evaluate temporal slope

The correct order is:

`surface capture -> cross-surface alignment -> cross-surface decay -> temporal slope`

---

## 8. Non-Goals

These metrics do not:

- elect a trusted basin
- suppress alternate paths
- recommend routing
- replace replay-admission boundary policy
- replace Stage-1 sinkhole metrics

They only answer:

`is the same practical basin absorbing multiple operational surfaces for the same subjects over time?`

---

## 9. Short Rule

The shortest correct reading is:

`Stage-1 detects verification-side basin collapse; Stage-2 detects cross-surface basin absorption`
