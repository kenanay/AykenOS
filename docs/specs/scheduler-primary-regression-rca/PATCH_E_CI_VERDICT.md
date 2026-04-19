# Patch E CI Verdict - Partial Success, Insufficient Impact

**Date**: 2026-04-19  
**CI Run**: 24636726029  
**Commit**: 12316741  
**Verdict**: ❌ FAIL (but progress made)

## 📊 Performance Metrics

| Metric | Before (24634827102) | After (24636726029) | Delta | Status |
|--------|---------------------|---------------------|-------|--------|
| syscall_latency_ms_proxy | 207.61ms | 204.90ms | -2.7ms (-1.3%) | ❌ FAIL (need ≤183.84ms) |
| boot_time_ms | 12707ms | 12714ms | +7ms (+0.05%) | ❌ FAIL (need ≤11752ms) |
| entry_latency_ticks | 22,045,664 | 24,343,714 | +2.3M (+10.4%) | ⚠️ WORSE |
| syscall_latency_ticks_pure | 8,905,309 | 6,225,818 | -2.7M (-30.1%) | ✅ BETTER |

**Baseline Requirements**:
- syscall_latency_ms_proxy ≤ 183.836ms (baseline 175.08ms +5%)
- boot_time_ms ≤ 11752ms (baseline 10684ms +10%)

**Actual**:
- syscall_latency_ms_proxy: 204.90ms (FAIL by 21.06ms, +17.0% over baseline)
- boot_time_ms: 12714ms (FAIL by 962ms, +19.0% over baseline)

## 🔍 Marker Reduction (SUCCESS)

| Marker | Before | After | Reduction |
|--------|--------|-------|-----------|
| P10_RING3_ATTEMPT | 63 | 0 | -100% ✅ |
| P10_RING3_COMMIT | 63 | 0 | -100% ✅ |
| PIC_MASK | 126 | 0 | -100% ✅ |
| P10_RING3_FRAME_PROOF | 1 | 1 | 0% ✅ |
| P10_TEXT_ROOT_PROOF | 1 | 1 | 0% ✅ |
| P10_ROOT_FRAME_WITNESS | many | 3 | ~95% ✅ |
| P10_TEXT_FRAME_WITNESS | many | 3 | ~95% ✅ |
| MARK:IRET | 61 | 162 | +166% ⚠️ |
| Debugcon size | 186KB | 106KB | -43% ✅ |

**Observation**: One-shot throttling WORKED for verbose markers, but IRET count increased significantly.

## 🎯 What Worked

1. **Debugcon I/O Reduction**: 186KB → 106KB (-43%)
2. **Pure Syscall Improvement**: 8.9M → 6.2M ticks (-30%)
3. **Marker Throttling**: Ring3 verbose markers eliminated
4. **One-shot Implementation**: Frame proof and text proof correctly throttled

## ❌ What Didn't Work

1. **Entry Window Increased**: 22M → 24.3M ticks (+10%)
2. **IRET Count Increased**: 61 → 162 (+166%)
3. **Net Latency**: Only 1.3% improvement (need 17% to pass)

## 🔥 Root Cause Analysis

### Why Entry Window Increased

The entry window INCREASED despite marker reduction. Possible causes:

1. **IRET Count Anomaly**: 61 → 162 transitions
   - More context switches = more entry overhead
   - Proxy metric: `preempt_qemu_run_time_ms / preempt_iret_count`
   - Before: 12664ms / 61 = 207.61ms per IRET
   - After: 12497ms / 162 = 77.14ms per IRET (!)

2. **Measurement Variance**: IRET count should be stable
   - Preempt test is deterministic
   - 61 → 162 suggests test behavior changed
   - Possible: Timer tick rate changed, scheduler behavior changed

3. **Timing Paradox**: Pure syscall improved but total didn't
   - Pure syscall: -30% (good)
   - Entry window: +10% (bad)
   - Net: -1.3% (insufficient)

### Why Pure Syscall Improved

Pure syscall latency dropped 30%:
- Before: 8.9M ticks
- After: 6.2M ticks
- Savings: 2.7M ticks

**Likely cause**: Reduced debugcon I/O in syscall body
- Patch C markers still emit but less frequently
- Boundary enforcement markers reduced
- Less contention on debugcon port

### Why Net Impact Is Small

```
Total latency = entry_window + pure_syscall + return

Before: 22M + 8.9M + 0.5M = 31.4M ticks
After:  24.3M + 6.2M + 0.5M = 31.0M ticks
Delta:  +2.3M - 2.7M + 0    = -0.4M ticks (-1.3%)
```

Entry window increase CANCELLED OUT pure syscall improvement.

## 📊 IRET Count Investigation

### Expected Behavior

Preempt test should have stable IRET count:
- Deterministic timer ticks
- Fixed number of context switches
- Consistent scheduler behavior

### Actual Behavior

IRET count tripled: 61 → 162

**Possible causes**:
1. Timer tick rate changed (unlikely - deterministic test)
2. Scheduler fallback behavior changed
3. Mailbox operations increased
4. Test harness changed

**Evidence needed**:
- Check preempt.analysis.log for scheduler path breakdown
- Check mailbox operation counts
- Check fallback vs switch path ratios

## 🔍 Detailed Breakdown

### Before (Run 24634827102)

```
preempt_iret_count: 61
preempt_qemu_run_time_ms: 12664ms
syscall_latency_ms_proxy: 207.61ms

Breakdown:
- Entry: 22.0M ticks (71%)
- Pure syscall: 8.9M ticks (29%)
- Total: 30.9M ticks
```

### After (Run 24636726029)

```
preempt_iret_count: 162
preempt_qemu_run_time_ms: 12497ms (!)
syscall_latency_ms_proxy: 204.90ms

Breakdown:
- Entry: 24.3M ticks (80%)
- Pure syscall: 6.2M ticks (20%)
- Total: 30.5M ticks
```

**Observation**: Total ticks DECREASED but entry window INCREASED. This suggests measurement model changed.

## 🎓 Key Insights

### 1. Measurement Model Sensitivity

The proxy metric is extremely sensitive to IRET count:
- Before: 12664ms / 61 = 207.61ms
- After: 12497ms / 162 = 77.14ms

**But**: Actual latency didn't improve proportionally. This suggests:
- IRET count is not stable
- Measurement model has variance
- Need to understand why IRET count changed

### 2. Optimization Trade-offs

Patch E reduced debugcon I/O but:
- Entry window increased (unknown cause)
- IRET count increased (unknown cause)
- Net improvement insufficient

### 3. Diminishing Returns

We've optimized:
- Syscall enforcement (Patch C): 500k → 120k ticks
- Debugcon I/O (Patch E): 186KB → 106KB
- Pure syscall: 8.9M → 6.2M ticks (-30%)

But still 17% over baseline. Remaining bottleneck is entry window.

## 🚨 Next Steps

### Immediate: Investigate IRET Count Anomaly

```bash
# Check preempt analysis log
cat preempt.analysis.log | grep -E "fallback|switch|iret"

# Check mailbox operations
cat report.json | jq '.results.mailbox_phase_breakdown_ticks'

# Check scheduler path distribution
cat report.json | jq '.results.mailbox_phase_breakdown_ticks.path_durations'
```

### Option 1: Fix IRET Count Variance

If IRET count should be 61:
- Find why it increased to 162
- Fix scheduler/timer behavior
- Re-run performance test

### Option 2: More Aggressive Throttling

If IRET count is correct:
- Disable ALL diagnostic markers in perf builds
- Add `AYKEN_PERF_BUILD` flag
- Separate proof builds from performance builds

### Option 3: Profile Entry Window

If entry window is the real bottleneck:
- Profile page table operations
- Profile scheduler mailbox operations
- Profile remaining marker overhead

## 📋 Artifact Locations

**CI Artifacts**: `/tmp/patch-e-results/`

**Key Files**:
- `gates/performance/report.json` - Full metrics
- `gates/performance/boot-audit/qemu_debugcon.log` - Execution markers (106KB)
- `gates/performance/preempt.analysis.log` - IRET cadence

## 🏁 Verdict

**Patch E Status**: ✅ TECHNICALLY CORRECT, ❌ INSUFFICIENT IMPACT

**What Worked**:
- Marker throttling: 100% reduction in verbose markers
- Debugcon I/O: 43% reduction
- Pure syscall: 30% improvement

**What Failed**:
- Entry window increased 10%
- IRET count increased 166%
- Net improvement only 1.3% (need 17%)

**Root Cause**: Unknown factor increased entry window and IRET count, cancelling out syscall improvements.

**Next Action**: Investigate IRET count anomaly and entry window increase.

---

**Supersedes**: PATCH_E_CI_READY.md

**Authoritative Source**: CI artifact `/tmp/patch-e-results/`

