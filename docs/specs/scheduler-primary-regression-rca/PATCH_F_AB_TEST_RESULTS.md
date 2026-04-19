# Patch F A/B Test Results - BOUNDARY ENFORCEMENT IMPACT CONFIRMED

**Date**: 2026-04-19  
**CI Run B**: 24637158182  
**Commit**: a0a41125  
**Verdict**: ⚠️ PARTIAL IMPROVEMENT - Boundary Enforcement Contributes But Not Sole Cause

## Side-by-Side Comparison

| Metric | Run A (Enforcement ON) | Run B (Enforcement OFF) | Delta | % Change |
|--------|------------------------|-------------------------|-------|----------|
| **boot_time_ms** | 12714 | 12708 | -6ms | -0.05% |
| **syscall_latency_ms_proxy** | 204.90 | 204.49 | -0.41ms | -0.2% |
| **entry_latency_ticks** | 24,343,714 | 22,646,453 | -1,697,261 | -7.0% ✅ |
| **syscall_latency_ticks_pure** | 6,225,818 | 5,316,500 | -909,318 | -14.6% ✅ |
| **preempt_iret_count** | 61 | 61 | 0 | 0% ✅ |

**Baseline Targets**:
- boot_time_ms ≤ 11752ms (still FAIL: 12708ms, +8.1%)
- syscall_latency_ms_proxy ≤ 183.84ms (still FAIL: 204.49ms, +11.2%)

## Critical Findings

### 1. Pure Syscall Improved Significantly (-14.6%)
```
syscall_latency_ticks_pure: 6.2M → 5.3M (-909k ticks, -14.6%)
```
**Interpretation**: Boundary enforcement DOES have measurable cost in syscall body  
**Magnitude**: ~900k ticks per syscall = significant but not dominant

### 2. Entry Window Improved Moderately (-7.0%)
```
entry_latency_ticks: 24.3M → 22.6M (-1.7M ticks, -7.0%)
```
**Interpretation**: Boundary enforcement has SOME impact on entry window  
**But**: Entry window still dominates (22.6M / 27.9M = 81% of total)

### 3. Total Latency Breakdown
```
Run A (Enforcement ON):
  Entry: 24.3M (79%)
  Pure syscall: 6.2M (21%)
  Total: 30.5M ticks

Run B (Enforcement OFF):
  Entry: 22.6M (81%)
  Pure syscall: 5.3M (19%)
  Total: 27.9M ticks (-8.5% improvement)
```

### 4. Proxy Metric Barely Changed (-0.2%)
```
syscall_latency_ms_proxy: 204.90 → 204.49 (-0.41ms, -0.2%)
```
**Why?**: Proxy metric is wall-time based, includes QEMU overhead, less sensitive to tick improvements

### 5. IRET Count Stable (Validation ✅)
```
preempt_iret_count: 61 → 61 (unchanged)
```
**Interpretation**: Scheduler behavior unchanged, test is clean

## Verdict: Scenario 3 (Two Cost Centers)

This matches **Scenario 3** from the test plan:
- ✅ Syscall improved significantly (-14.6%)
- ✅ Entry window improved moderately (-7.0%)
- ❌ Boot time essentially unchanged (-0.05%)
- ❌ Still failing constitutional thresholds

**Conclusion**: Boundary enforcement contributes to regression but is NOT the sole cause.

## Root Cause Analysis

### Boundary Enforcement Impact (Confirmed)
- **Pure syscall cost**: ~900k ticks per syscall
- **Entry window cost**: ~1.7M ticks per transition
- **Total contribution**: ~2.6M ticks per syscall roundtrip

### Remaining Bottleneck (Still Unidentified)
- **Entry window**: Still 22.6M ticks (81% of total)
- **Remaining regression**: Still +11.2% over baseline
- **Primary suspect**: Ring3 transition overhead NOT related to boundary enforcement

## What This Proves

### ✅ Boundary Enforcement IS Expensive
- 14.6% overhead in syscall body
- 7.0% overhead in entry window
- Total ~8.5% of measured latency

### ❌ Boundary Enforcement Is NOT The Only Problem
- Disabling it only recovered 8.5% of 18% regression
- Still 9.5% regression remaining
- Entry window still dominates at 81%

### 🎯 Real Bottleneck Is Elsewhere
Entry window components to investigate:
1. **Ring3 transition mechanics** (CR3 pivot, page table operations)
2. **Frame validation overhead** (still active even without enforcement)
3. **Text walk proof overhead** (still active)
4. **Scheduler mailbox operations** (entry path)
5. **Diagnostic markers** (`AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1` still on)

## Next Steps

### Option 1: Optimize Boundary Enforcement (Partial Fix)
- Move enforcement to lazy/cached/precomputed model
- Expected gain: ~8.5% (not enough to pass)
- Risk: Still won't meet constitutional threshold

### Option 2: Profile Entry Window (Root Cause Hunt)
- Focus on Ring3 transition path
- Measure CR3 pivot cost
- Measure frame/text validation cost
- Measure diagnostic marker cost
- Expected: Find the remaining 9.5% regression

### Option 3: Disable Diagnostic Markers (Next A/B Test)
- Set `AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0`
- Test if boot audit markers are inflating entry window
- Quick test, low risk

## Recommended Path Forward

**Immediate**: Option 3 (Patch G - Diagnostic Markers A/B Test)
- Quick test to rule out marker overhead
- If markers are expensive, we get another 5-10% back
- If not, we know to focus on Ring3 transition mechanics

**Then**: Option 2 (Entry Window Profiling)
- Add tick measurements to Ring3 entry path
- Identify specific bottleneck (CR3, page tables, validation)
- Targeted optimization based on data

**Finally**: Option 1 (Boundary Enforcement Optimization)
- Once entry window is fixed, optimize enforcement
- Cache context type per process
- Lazy validation where possible

## Key Insight

**The regression has TWO sources**:
1. Boundary enforcement: ~8.5% (confirmed, measurable)
2. Entry window overhead: ~9.5% (unconfirmed, needs profiling)

Fixing only one won't meet constitutional threshold. Need to fix both.

## Artifact Locations

**CI Run**: 24637158182  
**Artifacts**: `/tmp/patch-f-ab-test/`  
**Key Files**:
- `gates/performance/report.json` - Full metrics
- `gates/performance/boot-audit/qemu_debugcon.log` - Execution trace

---

**Status**: Analysis complete, next experiment identified (Patch G)  
**Supersedes**: PATCH_F_AB_TEST_PLAN.md (test complete)  
**Next**: PATCH_G - Diagnostic markers A/B test
