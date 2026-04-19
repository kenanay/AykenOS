# Patch G: Diagnostic Markers A/B Test Plan

**Date**: 2026-04-19  
**Commit Base**: a0a41125 (Patch F)  
**Branch**: test/diagnostic-markers-ab-test  
**Objective**: Measure diagnostic marker overhead in entry window

## Context from Patch F

Patch F (boundary enforcement disabled) revealed:
- Pure syscall improved 14.6% (6.2M → 5.3M ticks)
- Entry window improved 7.0% (24.3M → 22.6M ticks)
- But still failing constitutional threshold (+11.2% over baseline)
- **Remaining bottleneck**: Entry window still 22.6M ticks (81% of total)

**Hypothesis**: Diagnostic markers (`AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1`) are inflating entry window.

## Experimental Design

### Run A: Patch F Results (Baseline for Patch G)
```
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1
```
- CI Run: 24637158182
- entry_latency_ticks: 22,646,453
- syscall_latency_ticks_pure: 5,316,500
- boot_time_ms: 12708
- syscall_latency_ms_proxy: 204.49ms

### Run B: Patch G (Treatment - Markers Disabled)
```
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0
```
- CI Run: TBD
- Results: Pending

### Control Variables (MUST NOT CHANGE)
- `AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0` (same as Patch F)
- `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0` (already disabled)
- `AYKEN_PHASE16_PROBE_VALIDATION_ENABLE=1` (unchanged)
- All Ring3 transition flags (unchanged)
- Kernel profile: release (unchanged)

## Metrics to Compare

| Metric | Run A (Markers ON) | Expected Run B | Delta Interpretation |
|--------|-------------------|----------------|----------------------|
| entry_latency_ticks | 22,646,453 | ? | If ↓↓ → markers inflate entry |
| syscall_latency_ticks_pure | 5,316,500 | ? | Should be stable |
| boot_time_ms | 12708 | ? | If ↓ → boot markers expensive |
| syscall_latency_ms_proxy | 204.49 | ? | Proxy metric |
| preempt_iret_count | 61 | 61 | Must be stable |

## Interpretation Matrix

### Scenario 1: Entry Window Drops Significantly (≥10%)
```
entry_latency_ticks: 22.6M → ~20M (-11%)
syscall_latency_ms_proxy: 204.49 → ~190ms (-7%)
```
**Conclusion**: Diagnostic markers are a major cost center  
**Next Action**: Keep markers disabled in performance builds, separate proof builds  
**Expected**: May get close to constitutional threshold

### Scenario 2: Entry Window Drops Moderately (5-10%)
```
entry_latency_ticks: 22.6M → ~21M (-7%)
syscall_latency_ms_proxy: 204.49 → ~198ms (-3%)
```
**Conclusion**: Markers contribute but aren't dominant  
**Next Action**: Profile Ring3 transition mechanics (CR3, page tables, frame validation)

### Scenario 3: Entry Window Unchanged (<5%)
```
entry_latency_ticks: 22.6M → 22.4M (-1%)
syscall_latency_ms_proxy: 204.49 → 204.0ms (-0.2%)
```
**Conclusion**: Markers are NOT the bottleneck  
**Next Action**: Deep dive into Ring3 transition mechanics  
**Focus**: CR3 pivot, page table operations, frame/text validation

## What Diagnostic Markers Do

When `AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1`:
- Boot audit markers (one-time, low cost)
- Phase 16 proof markers (per-operation, potentially expensive)
- Boundary enforcement audit trails
- BCIB execution traces

**Hypothesis**: These markers emit debugcon I/O on every Ring3 transition, inflating entry window.

## Risk Assessment

- ✅ Low risk: Markers are diagnostic only
- ✅ Reversible: Can re-enable immediately
- ✅ Architecturally safe: Proof builds can keep markers enabled
- ⚠️ Trade-off: Lose some observability in performance builds

## Success Criteria

This is a DIAGNOSTIC test. Success means:
- ✅ IRET count remains 61 (test is clean)
- ✅ Clear delta in entry_latency_ticks
- ✅ Causal relationship established (or refuted)

## Timeline

1. **Now**: Document test plan (this file)
2. **Next**: Modify Makefile, commit, push
3. **Wait**: CI completes (~5 minutes)
4. **Then**: Download artifacts, analyze results
5. **Finally**: Document findings, determine next action

## Expected Outcome

**Most likely**: Scenario 2 (moderate improvement)
- Markers contribute 5-10% to entry window
- Combined with Patch F, total improvement ~15%
- Still need to optimize Ring3 transition mechanics

**Best case**: Scenario 1 (significant improvement)
- Markers are major cost center
- Combined with Patch F, approach constitutional threshold
- Can close investigation with marker disable + enforcement optimization

**Worst case**: Scenario 3 (no improvement)
- Markers are not the problem
- Must profile Ring3 transition mechanics deeply
- Longer investigation ahead

## Notes

- This test builds on Patch F (enforcement already disabled)
- We're isolating marker overhead from enforcement overhead
- Clean experimental design: single variable changed
- Authoritative environment: GitHub CI Linux x86_64 only

---

**Status**: READY TO EXECUTE  
**Next Action**: Commit and push to trigger CI
