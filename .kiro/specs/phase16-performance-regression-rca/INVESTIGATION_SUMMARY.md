# Phase 16 Performance Regression Investigation Summary

## Executive Summary

**Status**: PARTIAL FIX - Snapshot overhead eliminated but primary regression source remains unidentified

**Authoritative Result** (GitHub CI Linux x86_64):
- boot_time: 12,176ms (baseline: 10,684ms, threshold: 11,752ms)
- Regression: +1,492ms (+14.0%) - STILL FAILING
- Snapshot fix verified working but insufficient

## Investigation Timeline

### Task 1: Phase 16 Feature Isolation
**Result**: Phase 16 features are CLEAN (zero overhead)
- BCIB worker, boundary enforcement, probes, markers all disabled → no change
- Conclusion: Phase 16 code is NOT the regression source

### Task 2: Stale Epoch Short-Circuit Verification
**Result**: Stale path is WORKING CORRECTLY
- 214 stale detections, only 2 extract calls (bootstrap)
- Short-circuit functioning as designed
- Conclusion: Stale path is NOT the regression source

### Task 3: Git Bisect - Duplicate Snapshot Discovery
**Result**: Found ONE regression source (lines 1922-1936 in sched.c)
- Triple snapshot overhead: peek_epoch + duplicate + extract
- Fix: Removed redundant intermediate snapshot block
- Commit: 31a33246

### Task 4: Fix Verification
**Result**: Snapshot fix WORKS but regression PERSISTS

**Local Darwin/arm64** (informational only):
- extract_raw_observation_count: 214 → 1 ✓
- boot_time: 13,856ms (env_hash_mismatch - invalid)

**Authoritative GitHub CI** (Linux x86_64):
- boot_time: 12,176ms (+1,492ms, +14%)
- syscall_latency: 195.7ms (+11.8%)
- context_switch_latency: 195.7ms (+11.8%)
- **FAIL**: Regression still above constitutional threshold

## Key Findings

### What We Fixed
✓ Duplicate snapshot overhead (lines 1922-1936)
✓ Stale path now short-circuits correctly
✓ Extract count reduced from 214 to 1

### What We Learned
❌ Snapshot was ONE source, not THE source
❌ Primary regression (~1,478ms of 1,492ms) is elsewhere
❌ Syscall and context-switch latency also regressed (+11.8%)

### Remaining Overhead Sources (Hypotheses)

1. **Bootstrap/Cold-Start Path** (most likely)
   - validate: 1.54M ticks (only 2 calls but expensive)
   - arbiter: 5.87M ticks (only 2 calls but expensive)
   - Total: ~7.4M ticks in bootstrap path
   - May be disproportionately expensive

2. **Syscall Gate Path** (correlated evidence)
   - syscall_latency regressed +11.8%
   - May indicate added overhead in syscall entry/exit
   - Could be instrumentation or validation

3. **Context Switch Path** (correlated evidence)
   - context_switch_latency regressed +11.8%
   - May indicate added overhead in switch mechanics
   - Could be related to arbiter or handoff

4. **Other Scheduler Changes** (unknown)
   - May be commits before or after snapshot issue
   - Need broader git bisect range
   - Could be unrelated to mailbox path

## Metrics Analysis

### Baseline vs Current (GitHub CI)

| Metric | Baseline | Current | Delta | % Change |
|--------|----------|---------|-------|----------|
| boot_time_ms | 10,684 | 12,176 | +1,492 | +14.0% |
| syscall_latency_ms | 175.1 | 195.7 | +20.6 | +11.8% |
| context_switch_latency_ms | 175.1 | 195.7 | +20.6 | +11.8% |

### Snapshot Fix Impact

| Metric | Before Fix | After Fix | Improvement |
|--------|------------|-----------|-------------|
| extract_raw_observation_count | 214 | 1 | -213 (-99.5%) |
| boot_time_ms (estimated) | ~12,190 | 12,176 | ~14ms (~0.1%) |

**Conclusion**: Snapshot fix had minimal impact on overall boot time.

## Technical Debt

### Commits Applied
- 31a33246: perf(scheduler): remove duplicate snapshot overhead
- 6a81fc45: tools: add performance bisect script
- 87d0be2d: (rebase)
- 155db54c: ci: trigger performance gate

### Code Changes
- Removed lines 1922-1936 in kernel/sched/sched.c
- Kept peek_epoch fast-path (lines 1896-1920)
- Preserved extract() for fresh epochs only

### Verification Status
- ✓ Local verification: snapshot fix working
- ✓ GitHub CI verification: snapshot fix working
- ❌ Performance recovery: FAILED (regression persists)

## Recommendations

### Immediate Actions

1. **Expand Git Bisect Range**
   - Current bisect found snapshot issue
   - Need to bisect broader range for primary regression
   - Focus on commits affecting syscall/context-switch paths

2. **Profile Bootstrap Path**
   - Analyze why validate costs 1.54M ticks (2 calls)
   - Analyze why arbiter costs 5.87M ticks (2 calls)
   - Compare with baseline to find added overhead

3. **Profile Syscall/Context-Switch Paths**
   - Syscall latency regressed +11.8%
   - Context-switch latency regressed +11.8%
   - May be correlated with bootstrap overhead

### Scope Decision

**Option A**: Expand current spec
- Add new tasks for deeper investigation
- Continue in same bugfix workflow
- Risk: Scope creep, unclear completion criteria

**Option B**: Close current spec, open new spec
- Current spec: "Snapshot overhead fix" (DONE)
- New spec: "Primary regression source identification"
- Benefit: Clear separation of concerns

**Recommendation**: Option B - Close current spec as "partial fix", open new investigation spec for primary regression.

## Conclusion

The snapshot fix (commit 31a33246) is technically correct and working as designed. However, it addresses only a minor component (~14ms) of the total regression (+1,492ms). The primary regression source remains unidentified and requires deeper investigation.

**Next Steps**:
1. Commit snapshot fix to main (it's a valid optimization)
2. Open new spec for primary regression investigation
3. Focus on bootstrap/syscall/context-switch paths
4. Use broader git bisect range to find actual regression commit

---

**Investigation Date**: 2026-04-18
**Authoritative CI Run**: 24610398801
**Spec ID**: phase16-performance-regression-rca
