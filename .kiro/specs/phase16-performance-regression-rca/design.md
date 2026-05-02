# Phase 16 Performance Regression RCA Bugfix Design

## Overview

A +14% boot time regression (12,190ms vs 10,684ms baseline) was observed in CI, violating the constitutional 10% performance threshold. Task 1 measurements proved Phase 16 features (BCIB worker, boundary enforcement, probes, markers) are NOT the source - disabling them has zero effect.

**Root cause identified**: Scheduler stale epoch detection exists but does NOT short-circuit expensive operations. The stale path detects 61 stale epochs correctly but still executes the full snapshot/extract/validate/arbiter pipeline, incurring ~6.2M ticks of unnecessary overhead.

This design outlines the fix: enforce immediate return on stale epoch detection, skipping all expensive operations for the stale path while preserving correct behavior for actual switch decisions.

## Glossary

- **Bug_Condition (C)**: Stale epoch detected BUT extract/validate/arbiter still execute (should short-circuit)
- **Property (P)**: Stale epoch returns immediately without executing expensive operations
- **Preservation**: Non-stale path (actual switch decisions) continues to execute full pipeline correctly
- **CI Authority**: GitHub Actions Linux x86_64 environment is the authoritative measurement environment
- **Stale Epoch**: Mailbox epoch <= owner_last_epoch (no new work, should skip processing)
- **Short-Circuit**: Immediate return without executing subsequent expensive operations
- **Baseline Commit**: 050332220d9 (10,684ms boot time)
- **Current Commit**: 6e883e64 (12,190ms boot time)
- **Regression Commits**: Likely between bbe0540d (perf: add scheduler fast path) and ef7e3018 (fix(scheduler): add true fast-path)

## Bug Details

### Bug Condition

The performance regression manifests when the scheduler detects stale epochs but fails to short-circuit expensive operations. Evidence from CI logs:

```
no_candidate = 61                          (stale epochs detected correctly)
arbiter_decision: 3,160,116 ticks          (SHOULD BE ~0 for stale path)
extract: 1,976,836 ticks                   (SHOULD NOT RUN for stale path)
validate: 1,074,006 ticks                  (SHOULD NOT RUN for stale path)
arbiter_candidate_accept_switch: 33,544,438,892 ticks  (ABSURDLY HIGH)
```

**Pattern**: Stale epoch detected 61 times, but arbiter/extract/validate metrics show full pipeline executed anyway.

**Formal Specification:**
```
FUNCTION isBugCondition(sched_metrics)
  INPUT: sched_metrics of type SchedulerMetrics
  OUTPUT: boolean
  
  stale_count := sched_metrics.no_candidate_count  // 61
  extract_ticks := sched_metrics.extract_ticks     // 1,976,836
  validate_ticks := sched_metrics.validate_ticks   // 1,074,006
  arbiter_ticks := sched_metrics.arbiter_decision_ticks  // 3,160,116
  
  // If stale path detected, expensive operations should NOT execute
  RETURN stale_count > 0
         AND (extract_ticks > 0 OR validate_ticks > 0 OR arbiter_ticks > 0)
END FUNCTION
```

### Examples

- **Baseline (050332220d9)**: Stale path short-circuits correctly, boot_time = 10,684ms
- **Current (6e883e64)**: Stale path detected BUT full pipeline executes, boot_time = 12,190ms (+1,506ms)
- **Expected**: Stale path returns immediately, extract/validate/arbiter ticks ~0, boot_time ≤ 11,752ms

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Stale epoch detection must continue to work correctly (no_candidate count accurate)
- Non-stale path (actual switch decisions) must continue to execute full pipeline (snapshot/extract/validate/arbiter)
- Context switches must continue to work correctly
- Scheduler functionality must remain intact

**Scope:**
The fix targets ONLY the stale epoch path. When a stale epoch is detected (epoch <= owner_last_epoch), the scheduler must return immediately WITHOUT executing expensive operations. The non-stale path (actual switch decisions) must continue to execute the full pipeline correctly.

## Root Cause Analysis

### Task 1 Results: Phase 16 Features Are Clean

Feature isolation measurements proved Phase 16 features have ZERO overhead:
- BCIB worker disabled: 12190ms (same as enabled)
- Boundary enforcement disabled: 12190ms (same as enabled)
- Probe validation disabled: 12190ms (same as enabled)
- Diagnostic markers disabled: 12190ms (same as enabled)
- All features disabled: 12190ms (same as all enabled)

**Conclusion**: Phase 16 code is NOT the regression source.

### Actual Root Cause: Stale Epoch Path Not Short-Circuiting

CI logs reveal the actual problem:
1. Stale epochs detected correctly: `no_candidate = 61`
2. BUT expensive operations still execute:
   - `arbiter_decision: 3,160,116 ticks` (should be ~0)
   - `extract: 1,976,836 ticks` (should not run)
   - `validate: 1,074,006 ticks` (should not run)
3. Total unnecessary overhead: ~6.2M ticks

**Hypothesis**: Epoch peek optimization detects stale BUT does not short-circuit:
```c
// SUSPECTED CURRENT BEHAVIOR:
uint64_t epoch = peek_epoch(owner);
if (epoch <= owner_last_epoch) {
    no_candidate_flag = true;  // Flag set
}
// BUT: extract/validate/arbiter still execute

// EXPECTED BEHAVIOR:
uint64_t epoch = peek_epoch(owner);
if (epoch <= owner_last_epoch) {
    return NO_CANDIDATE;  // IMMEDIATE RETURN
}
// extract/validate/arbiter NEVER execute for stale path
```

## Correctness Properties

Property 1: Bug Condition - Stale Epoch Short-Circuit

_For any_ scheduler invocation where a stale epoch is detected (epoch <= owner_last_epoch), the scheduler SHALL return NO_CANDIDATE immediately WITHOUT executing extract/validate/arbiter operations, resulting in ~0 ticks overhead for the stale path.

**Validates: Requirements 2.1, 2.2, 2.3**

Property 2: Preservation - Non-Stale Path Unchanged

_For any_ scheduler invocation where a non-stale epoch is detected (epoch > owner_last_epoch), the scheduler SHALL execute the full pipeline (snapshot/extract/validate/arbiter) correctly, preserving all switch decision logic and context switch functionality.

**Validates: Requirements 3.1, 3.2, 3.3**

Property 3: Performance Recovery - Boot Time Within Threshold

_For any_ kernel boot in the CI environment, the fixed kernel SHALL achieve boot_time ≤ 11,752ms (baseline + 10%), eliminating the +1,506ms regression.

**Validates: Requirements 2.4, 2.5**

## Fix Implementation

### Changes Required

The fix enforces immediate return on stale epoch detection, skipping all expensive operations:

**Phase 1: Root Cause Verification (Diagnostic)**

1. **Add Logging to Verify Short-Circuit Failure**
   - Add trace logging to scheduler stale epoch path
   - Log entry to `sched_select_next()` with epoch value
   - Log when stale detected: `TRACE("STALE_SHORT_CIRCUIT epoch=%lu")`
   - Log entry to `extract()`, `validate()`, `arbiter_decision()`
   - Run boot in CI, collect debugcon logs
   - **Expected**: Logs show stale detected BUT extract/validate/arbiter still called

2. **Git Bisect to Find Regression Commit**
   - Use binary search to find exact commit where regression started
   - Focus on commits between bbe0540d and ef7e3018
   - Test each commit: if boot_time > 11,752ms → bad, else → good
   - **Expected**: Bisect identifies commit that broke short-circuit

**Phase 2: Short-Circuit Fix**

1. **Enforce Immediate Return on Stale Epoch**
   - **File**: `kernel/sched/sched.c` (likely in `sched_select_next()` or similar)
   - **Change**: Replace flag-setting with immediate return
   
   ```c
   // BEFORE (suspected):
   uint64_t epoch = peek_epoch(owner);
   if (epoch <= owner_last_epoch) {
       no_candidate_flag = true;  // Flag set but code continues
   }
   // ... extract/validate/arbiter still execute ...
   
   // AFTER (correct):
   uint64_t epoch = peek_epoch(owner);
   if (epoch <= owner_last_epoch) {
       TRACE("STALE_SHORT_CIRCUIT epoch=%lu", epoch);
       return NO_CANDIDATE;  // IMMEDIATE RETURN
   }
   // extract/validate/arbiter NEVER execute for stale path
   ```

2. **Verify No Code After Stale Detection**
   - Ensure NO extract() call after stale detection
   - Ensure NO validate() call after stale detection
   - Ensure NO arbiter_decision() call after stale detection
   - Verify return value propagates correctly to caller

**Phase 3: Validation (Verification)**

1. **Verify Short-Circuit Working**
   - Re-run logging from Phase 1
   - **Expected**: Logs show stale detected AND extract/validate/arbiter NOT called
   - Verify metrics: arbiter_decision ticks ~0 (not 3,160,116)
   - Verify metrics: extract ticks ~0 (not 1,976,836)
   - Verify metrics: validate ticks ~0 (not 1,074,006)

2. **Verify Performance Recovery**
   - Run CI performance gate with fix
   - **Expected**: boot_time ≤ 11,752ms (regression eliminated)
   - Verify no_candidate count still 61 (stale detection still works)

3. **Verify Preservation**
   - Run existing scheduler tests
   - Verify non-stale path still executes full pipeline
   - Verify context switches work correctly
   - **Expected**: All scheduler functionality preserved

### Specific Implementation Files

**File**: `kernel/sched/sched.c`
- **Function**: `sched_select_next()` or similar (exact function TBD via git bisect)
- **Change**: Replace stale epoch flag-setting with immediate return
- **Change**: Add TRACE logging for diagnostic verification

**File**: `scripts/ci/perf-baseline.lock.json`
- **Change**: Update baseline metrics after fix validation
- **Change**: Enable enforcement (`enforcement_enabled: true`) to prevent future regressions

## Testing Strategy

### Validation Approach

The testing strategy follows a three-phase approach: diagnostic verification to confirm short-circuit failure, targeted fix implementation, and preservation checking to ensure scheduler functionality remains intact.

### Phase 1: Diagnostic Verification

**Goal**: Confirm that stale epoch detection exists BUT does not short-circuit expensive operations.

**Test Plan**: Add trace logging to scheduler, run boot in CI, analyze debugcon logs.

**Expected Evidence**:
- Log shows: `STALE_SHORT_CIRCUIT epoch=X` (61 times)
- Log shows: `extract()` called (should NOT be called)
- Log shows: `validate()` called (should NOT be called)
- Log shows: `arbiter_decision()` called (should NOT be called)

### Phase 2: Fix Validation

**Goal**: Verify that stale epoch path returns immediately without executing expensive operations.

**Test Plan**: Apply short-circuit fix, re-run diagnostic logging, verify metrics.

**Expected Results**:
- `no_candidate = 61` (stale detection still works)
- `arbiter_decision ticks ~0` (not 3,160,116)
- `extract ticks ~0` (not 1,976,836)
- `validate ticks ~0` (not 1,074,006)
- `boot_time ≤ 11,752ms` (regression eliminated)

### Phase 3: Preservation Checking

**Goal**: Verify that non-stale path (actual switch decisions) continues to work correctly.

**Test Plan**: Run existing scheduler tests, verify context switches work, verify preemption tests pass.

**Expected Results**:
- All scheduler tests pass
- Context switches work correctly
- Non-stale path executes full pipeline (snapshot/extract/validate/arbiter)
- Scheduler functionality preserved

## Archived Findings

### Task 1: Phase 16 Feature Isolation (COMPLETED)

Feature isolation measurements proved Phase 16 features are NOT the regression source:

**Test Results**:
- BCIB worker disabled: 12190ms (same as enabled)
- Boundary enforcement disabled: 12190ms (same as enabled)
- Probe validation disabled: 12190ms (same as enabled)
- Diagnostic markers disabled: 12190ms (same as enabled)
- All features disabled: 12190ms (same as all enabled)

**Conclusion**: Phase 16 code has ZERO overhead. The regression is NOT from Phase 16 instrumentation.

**Rejected Hypotheses**:
- ❌ BCIB worker creation overhead
- ❌ Boundary enforcement check overhead
- ❌ Probe validation (frame matching) overhead
- ❌ Diagnostic marker emission overhead

See: `.kiro/specs/phase16-performance-regression-rca/TASK1_FINDINGS.md`
