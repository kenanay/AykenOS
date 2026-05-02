# Bugfix Requirements Document

## Current Status (2026-04-17)

**Phase**: Root Cause Identified  
**Critical Finding**: Stale epoch path NOT short-circuiting - full pipeline executes unnecessarily

### Root Cause Analysis

**Phase 16 features are CLEAN** (Task 1 confirmed: zero overhead from BCIB/boundary/probes/markers)

**ACTUAL BUG**: Scheduler stale epoch detection exists BUT full validation pipeline still executes:

```
Evidence from CI logs:
- no_candidate = 61 (stale epochs detected correctly) ✓
- arbiter_decision: 3,160,116 ticks (SHOULD BE ~0 for stale path) ✗
- extract: 1,976,836 ticks (SHOULD NOT RUN for stale path) ✗
- validate: 1,074,006 ticks (SHOULD NOT RUN for stale path) ✗
- arbiter_candidate_accept_switch: 33,544,438,892 ticks (ABSURDLY HIGH) ✗
```

**Pattern**: `no_candidate = 61` means stale path detected 61 times, but arbiter/extract/validate metrics show FULL pipeline executed anyway.

**Hypothesis**: Epoch peek optimization detects stale BUT does not short-circuit the expensive operations (snapshot/extract/validate/arbiter).

**Expected behavior**: 
```c
if (epoch <= owner_last_epoch) {
    return NO_CANDIDATE;  // SHORT-CIRCUIT: no extract, no validate, no arbiter
}
```

**Actual behavior** (suspected):
```c
peek();
if (stale) {
    flag = NO_CANDIDATE;
}
// BUT: extract/validate/arbiter still execute
```

### Next Actions

1. **Git bisect** - Find exact commit where regression started (likely between bbe0540d and ef7e3018)
2. **Verify short-circuit** - Add logging to confirm stale path skips extract/validate/arbiter
3. **Fix enforcement** - Ensure stale epoch returns IMMEDIATELY without expensive operations
4. **Measure impact** - Verify fix recovers the +1,506ms regression

See detailed findings: `.kiro/specs/phase16-performance-regression-rca/TASK1_FINDINGS.md`

---

## Introduction

A +14% boot time regression (12,190ms vs 10,684ms baseline) was observed in CI. Initial investigation focused on Phase 16 features and scheduler overhead. Task 1 measurements proved Phase 16 features (BCIB worker, boundary enforcement, probes, markers) are NOT the source - disabling them has zero effect.

**Root cause identified**: Scheduler stale epoch detection exists but does NOT short-circuit expensive operations. Evidence:
- `no_candidate = 61` (stale epochs detected)
- BUT: `arbiter_decision: 3,160,116 ticks` (should be ~0 for stale path)
- AND: `extract: 1,976,836 ticks` + `validate: 1,074,006 ticks` (should not execute for stale path)

The bug is NOT "slow code" - it's "unnecessary code execution". The stale epoch fast-path detects stale epochs correctly but fails to skip the expensive snapshot/extract/validate/arbiter pipeline.

This violates the constitutional performance contract (10% threshold) and represents a determinism + baseline + governance violation under constitutional CI.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN the scheduler detects a stale epoch (epoch <= owner_last_epoch) THEN the system sets no_candidate flag BUT continues to execute expensive operations (snapshot/extract/validate/arbiter)

1.2 WHEN the stale epoch path executes 61 times during boot THEN the system incurs 3,160,116 ticks in arbiter_decision + 1,976,836 ticks in extract + 1,074,006 ticks in validate (should be ~0 for stale path)

1.3 WHEN arbiter_candidate_accept_switch executes THEN the system exhibits 33,544,438,892 ticks (absurdly high, indicating loop or repeated execution)

1.4 WHEN the kernel boots THEN the system exhibits boot_time of 12,190ms (baseline: 10,684ms, +1,506ms regression)

1.5 WHEN the regression is measured in CI THEN the system violates the constitutional 10% performance threshold (actual: +14%)

### Expected Behavior (Correct)

2.1 WHEN the scheduler detects a stale epoch (epoch <= owner_last_epoch) THEN the system SHALL return NO_CANDIDATE immediately WITHOUT executing snapshot/extract/validate/arbiter operations

2.2 WHEN the stale epoch path executes 61 times during boot THEN the system SHALL incur ~0 ticks in arbiter_decision/extract/validate (short-circuit before expensive operations)

2.3 WHEN arbiter_candidate_accept_switch executes THEN the system SHALL exhibit reasonable tick counts (not absurdly high values indicating loops)

2.4 WHEN the kernel boots THEN the system SHALL achieve boot_time within 10% of baseline (≤11,752ms)

2.5 WHEN the regression is measured in CI THEN the system SHALL comply with the constitutional 10% performance threshold

### Unchanged Behavior (Regression Prevention)

3.1 WHEN scheduler epoch peek detects stale epochs THEN the system SHALL CONTINUE TO detect them correctly (no_candidate count accurate)

3.2 WHEN scheduler makes actual switch decisions (non-stale path) THEN the system SHALL CONTINUE TO execute full pipeline (snapshot/extract/validate/arbiter) correctly

3.3 WHEN scheduler makes switch decisions (1 actual switch) THEN the system SHALL CONTINUE TO exhibit correct switching behavior

3.4 WHEN Phase 16 features execute THEN the system SHALL CONTINUE TO maintain zero overhead (Task 1 verified: BCIB/boundary/probes/markers are clean)

3.5 WHEN non-scheduler kernel subsystems execute THEN the system SHALL CONTINUE TO maintain baseline performance characteristics

3.6 WHEN CI measures boot_time in the authoritative environment (Linux x86_64 GitHub Actions) THEN the system SHALL CONTINUE TO produce deterministic, reproducible results


---

## Closure Note

**Status**: SECONDARY ISSUE RESOLVED (PRIMARY REGRESSION UNRESOLVED)

This spec identified and fixed a secondary performance issue: duplicate snapshot overhead in the scheduler stale-path flow (lines 1922-1936 in kernel/sched/sched.c).

**What Was Fixed**:
- Duplicate snapshot overhead eliminated
- Extract count reduced from 214 to 1
- Stale path short-circuit verified working
- Fix committed: 31a33246

**What Was Verified**:
- Local verification: snapshot fix working ✓
- Authoritative GitHub CI verification: snapshot fix working ✓
- Performance impact: ~14ms improvement (minimal)

**What Remains Unresolved**:
- Primary regression: +1,492ms (+14%) still present in authoritative CI
- Syscall latency regression: +11.8%
- Context-switch latency regression: +11.8%
- Constitutional performance threshold: STILL FAILING

**Authoritative GitHub CI Evidence** (Run 24610398801):
- boot_time_ms: 12,176 (baseline: 10,684, threshold: 11,752)
- Regression persists despite snapshot fix

**Conclusion**: The snapshot fix is valid and should be preserved, but authoritative GitHub CI proved that it does not resolve the primary constitutional regression. The primary regression investigation is continued in a new spec focused on bootstrap, syscall-gate, context-switch, validate, and arbiter cost breakdown.

**Next Spec**: scheduler-primary-regression-rca

**Date**: 2026-04-18
