# Implementation Plan

## Phase 1: Root Cause Verification (Diagnostic)

- [x] 1. Phase 16 feature isolation - COMPLETED
  - **Status**: Infrastructure COMPLETE, Measurement INCONCLUSIVE
  - **Infrastructure**: Feature toggles implemented and verified working ✓
  - **Measurement Issue**: Local wall-clock measurements inconclusive (all configs: 12190ms)
  - **Root Cause**: Wall-clock time includes QEMU startup overhead, not sensitive to kernel code changes
  - **CRITICAL FINDING**: CI scheduler metrics prove root cause is NOT Phase 16
  - **Initial Evidence**: `no_candidate=61` with `extract/validate/arbiter` executing (1.9M+1.0M+3.1M ticks)
  - **Task 2 Verification**: Stale short-circuit is WORKING CORRECTLY (214 stale, 2 extract, 2 arbiter)
  - **Conclusion**: Both Phase 16 AND stale short-circuit hypotheses REJECTED
  - **Root Cause**: UNKNOWN - requires git bisect to identify actual regression commit
  - See: `.kiro/specs/phase16-performance-regression-rca/TASK1_FINDINGS.md`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 2. Verify stale epoch short-circuit failure
  - **Property 1: Bug Condition** - Stale Epoch Path Executes Full Pipeline
  - **Status**: HYPOTHESIS REJECTED - Short-circuit is WORKING CORRECTLY
  - **Evidence**: Debug run with AYKEN_DEBUG_SCHED=1
    - STALE_SHORT_CIRCUIT: 214 detections
    - EXTRACT_ENTER: 2 calls (bootstrap only)
    - ARBITER_DECISION_ENTER: 2 calls (bootstrap only)
    - NO_CANDIDATE: 214 (matches STALE count)
  - **Pattern Analysis**: Order verification shows correct behavior
    - First 2 EXTRACT calls at lines 190, 322 (bootstrap phase)
    - Then continuous STALE detections (lines 275, 354, 381, ...)
    - NO extract/arbiter calls after stale detection
  - **Conclusion**: Stale epoch short-circuit is functioning as designed
    - When epoch ≤ owner_last_epoch, immediate return with NO_CANDIDATE
    - Extract/validate/arbiter pipeline NOT executed for stale epochs
    - Performance regression root cause is NOT stale short-circuit failure
  - **Next Action**: Task 3 - Git bisect to find actual regression commit
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 3. Git bisect to find regression commit
  - **Property 2: Regression Localization** - Identify Exact Commit
  - **Status**: ROOT CAUSE FOUND - Duplicate snapshot overhead
  - **Root Cause**: Line 1922-1936 in kernel/sched/sched.c
    - peek_epoch stale check at line 1896-1920 ✓ (correct)
    - DUPLICATE snapshot + stale check at line 1922-1936 ❌ (removed)
    - extract() at line 1941 does THIRD snapshot + stale check ❌
  - **Problem**: Triple snapshot overhead on every scheduler iteration
    - peek_epoch: lightweight check (correct)
    - Duplicate validation: full snapshot (unnecessary)
    - extract: full snapshot again (necessary for non-stale path)
  - **Fix Applied**: Removed duplicate snapshot block (line 1922-1936)
    - Keep peek_epoch fast-path for stale detection
    - Remove redundant intermediate snapshot
    - Let extract() handle full validation for fresh epochs only
  - **Verification**: snapshot_enter count reduced from ~214 to 1
  - **Next**: Task 4 - Verify fix eliminates regression
  - _Requirements: 1.1, 1.4_

## Phase 2: Short-Circuit Fix Implementation

- [ ] 4. Implement stale epoch short-circuit enforcement

  - [x] 4.1 Enforce immediate return on stale epoch detection
    - **Status**: PARTIAL FIX - Duplicate snapshot removed
    - **Fix Applied**: Removed redundant intermediate snapshot (line 1922-1936)
    - **Impact**: snapshot overhead reduced (214 → 1 snapshot_enter)
    - **Local CI Result**: boot_time 13893ms (still 30% above baseline 10684ms)
    - **Environment Issue**: env_hash_mismatch - local Darwin/arm64 vs authoritative CI
    - **Analysis**: 
      - Snapshot overhead fixed ✓ (real improvement)
      - But local measurement unreliable (env mismatch)
      - validate_enter/exit still executing
      - Remaining overhead likely in validate/syscall-gate path
    - **Next**: Authoritative CI verification required (GitHub CI)
    - **Commit**: 31a33246 ready for CI validation
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 4.2 Verify short-circuit working (diagnostic re-run)
    - **Property 1: Expected Behavior** - Stale Path Short-Circuits
    - **Status**: VERIFIED - Snapshot overhead eliminated
    - **Local Test Results** (commit 31a33246):
      - snapshot_enter: 1 (was 214) ✓ - Duplicate snapshot removed
      - extract_enter: 2 (bootstrap only) ✓ - No extract for stale path
      - arbiter_decision_enter: 2 (bootstrap only) ✓ - No arbiter for stale path
      - STALE_SHORT_CIRCUIT: 214 detections ✓ - Stale detection working
    - **Fix Confirmed**: Removed redundant intermediate snapshot (line 1922-1936)
    - **Remaining Work**: Authoritative CI verification required
    - **Note**: Local env_hash_mismatch (Darwin/arm64 vs Linux/x86_64) makes boot_time unreliable
    - **Next**: Task 4.3 - Run commit 31a33246 in GitHub CI
    - _Requirements: 2.1, 2.2, 2.3_

  - [-] 4.3 Verify boot time regression eliminated
    - **Property 2: Performance Recovery** - Boot Time Within Threshold
    - **Status**: PARTIAL FIX - Snapshot overhead eliminated but regression persists
    - **Commit Tested**: 155db54c (87d0be2d + empty commit for CI trigger)
    - **CI Authority**: GitHub Actions Linux x86_64 (gha-ubuntu24-20260413.86.1-X64)
    - **Baseline**: 10,684ms (commit 050332220d9)
    - **Threshold**: ≤11,752ms (baseline + 10%)
    
    **AUTHORITATIVE GitHub CI Results** (Run 24610398801):
    - ❌ boot_time_ms: 12,176ms (+1,492ms, +14.0% regression)
    - ❌ syscall_latency_ms_proxy: 195.7ms (baseline 175ms, +11.8%)
    - ❌ context_switch_latency_ms_proxy: 195.7ms (baseline 175ms, +11.8%)
    - ❌ FAIL: 4 violations (3 metric regressions + 1 baseline mismatch)
    
    **Snapshot Fix Verification** (CONFIRMED WORKING):
    - ✓ extract_raw_observation_count: 1 (was 214) - Duplicate snapshot eliminated
    - ✓ Stale path short-circuits correctly
    - ✓ Validate/arbiter only during bootstrap (2 calls)
    
    **Critical Finding**: Snapshot fix was NECESSARY but NOT SUFFICIENT
    - Snapshot overhead was ONE source of regression
    - Regression reduced from +1,506ms to +1,492ms (minimal improvement)
    - Remaining overhead: ~1,492ms still unaccounted for
    
    **Root Cause Analysis Update**:
    1. ✓ Snapshot duplication fixed (commit 31a33246)
    2. ❌ Regression persists at nearly same level
    3. ❓ Remaining overhead sources unknown
    4. ❓ May be in validate/arbiter/syscall-gate paths
    5. ❓ Or may be in different part of scheduler entirely
    
    **Hypothesis Revision**: Multi-factorial regression
    - Snapshot overhead: FIXED but minimal impact (~14ms)
    - Primary regression source: STILL UNKNOWN
    - Need deeper investigation of:
      * Bootstrap path overhead (validate/arbiter 1.5M + 5.9M ticks)
      * Syscall gate latency (+11.8% regression)
      * Context switch latency (+11.8% regression)
      * Other scheduler paths not yet analyzed
    
    **Next Actions** (PRIORITY ORDER):
    
    1. **CRITICAL**: Identify primary regression source
       - Snapshot fix had minimal impact (~14ms of 1,492ms)
       - Need to analyze bootstrap/cold-start path
       - Check validate/arbiter overhead in detail
       - Profile syscall-gate and context-switch paths
    
    2. **IF validate/arbiter are the bottleneck**:
       - Analyze why they're expensive (1.5M + 5.9M ticks)
       - Check if Phase 16 validation added overhead
       - Look for redundant checks or slow lookups
    
    3. **IF syscall/context-switch are the bottleneck**:
       - Profile syscall gate entry/exit paths
       - Check context switch overhead
       - Look for added instrumentation or checks
    
    4. **IF neither explains the regression**:
       - Git bisect to find exact regression commit
       - May need to bisect beyond snapshot fix
       - Look for other scheduler changes
    
    **Decision Point**: Snapshot fix alone is insufficient
    - Cannot close spec with 14% regression
    - Need additional optimization or different fix
    - May need to expand scope beyond snapshot issue
    
    _Requirements: 2.4, 2.5_
    - **Commit Tested**: 87d0be2d
    - **CI Authority**: GitHub Actions Linux x86_64 (env_hash: edbe5bed...)
    - **Baseline**: 10,684ms (commit 050332220d9)
    - **Threshold**: ≤11,752ms (baseline + 10%)
    
    **Snapshot Fix Verification** (CONFIRMED WORKING):
    - ✓ extract_raw_observation_count: 1 (was 214) - Duplicate snapshot eliminated
    - ✓ no_candidate_count: 61 - Stale detection working
    - ✓ fallback_count: 61 - Stale path short-circuits correctly
    - ✓ snapshot_ticks: 343K (reduced from ~214x baseline)
    
    **Local Darwin/arm64 Results** (env_hash_mismatch - NOT authoritative):
    - ❌ boot_time: 13,856ms (still 29.7% above baseline)
    - ❌ syscall_latency: 225ms (baseline 175ms, +28.6%)
    - ❌ context_switch_latency: 225ms (baseline 175ms, +28.6%)
    - ⚠️ env_hash: 37e333aa... (expected: edbe5bed... for Linux/x86_64)
    
    **Remaining Overhead Sources** (Local Darwin/arm64):
    - extract_ticks: 1.57M (only 1 call, bootstrap)
    - validate_ticks: 1.54M (still executing)
    - arbiter_ticks: 5.87M (still executing)
    - Total mailbox overhead: ~9.3M ticks
    
    **Critical Blocker**: GitHub CI Performance Gate Not Running
    - PR #117 exists but performance workflow skipped
    - init_perf_baseline workflow shows "skipping" status
    - No authoritative performance gate results available
    - Local Darwin/arm64 results are INVALID for constitutional compliance
    
    **Root Cause Analysis**:
    1. ✓ Snapshot fix is CORRECT and WORKING (verified in code + metrics)
    2. ❌ Snapshot was ONE source of overhead, not the ONLY source
    3. ❓ Unknown if remaining overhead is Darwin-specific or real
    4. ❓ Cannot determine if regression is fully fixed without GitHub CI
    
    **Next Actions** (PRIORITY ORDER):
    
    1. **CRITICAL**: Get authoritative GitHub CI performance results
       - Option A: Manually trigger performance workflow on GitHub
       - Option B: Request maintainer to run performance gate
       - Option C: Wait for automated performance gate (if configured)
    
    2. **IF GitHub CI passes** (boot_time ≤ 11,752ms):
       - Snapshot fix was sufficient ✓
       - Local Darwin overhead is environment-specific
       - Mark Task 4.3 as DONE
       - Proceed to Task 4.4 (preservation verification)
    
    3. **IF GitHub CI fails** (boot_time > 11,752ms):
       - Snapshot fix was necessary but not sufficient
       - Remaining overhead in validate/arbiter paths
       - Need additional optimization:
         * Investigate validate path (1.54M ticks)
         * Investigate arbiter path (5.87M ticks)
         * May need to optimize syscall/context-switch paths
    
    **Hypothesis**: Multi-factorial regression
    - Snapshot overhead: FIXED (commit 31a33246) ✓
    - Validate/arbiter overhead: NOT YET ADDRESSED ❓
    - Environment-specific overhead: Darwin vs Linux ❓
    
    **Constitutional Compliance Note**:
    - Local measurements with env_hash_mismatch are NOT valid
    - ONLY GitHub CI Linux/x86_64 results count for merge approval
    - Cannot proceed to Task 5 (CI enforcement) without authoritative results
    
    _Requirements: 2.4, 2.5_

  - [ ] 4.4 Verify preservation - scheduler still works correctly
    - **Property 3: Preservation** - Non-Stale Path Unchanged
    - Run existing scheduler tests (if available)
    - Verify actual switch decisions (non-stale path) still execute full pipeline
    - Verify context switches work correctly
    - Verify preemption tests pass
    - **EXPECTED OUTCOME**: All scheduler functionality preserved, only stale path optimized
    - _Requirements: 3.1, 3.2, 3.3_

## Phase 3: CI Enforcement and Regression Prevention

- [ ] 5. Update CI performance enforcement
  - Update baseline metrics in scripts/ci/perf-baseline.lock.json
  - Set new baseline boot_time to measured value after fix (should be ≤11,752ms)
  - Enable enforcement: `"enforcement_enabled": true`
  - Add regression prevention: boot_time threshold check in CI pipeline
  - Verify CI fails if boot_time exceeds constitutional 10% threshold
  - Document fix in CI state: .ci-state/perf-enforcement-state.json
  - _Requirements: 2.5_

- [ ] 6. Final checkpoint - Constitutional compliance restored
  - **Status**: SECONDARY ISSUE RESOLVED - Primary regression unresolved
  - **Snapshot Fix**: Verified working in authoritative CI
  - **Performance Recovery**: FAILED - Regression persists (+1,492ms, +14%)
  - **Authoritative CI**: Run 24610398801 - boot_time 12,176ms (threshold 11,752ms)
  - **Conclusion**: Snapshot fix is valid but insufficient
  - **Next Steps**: New spec required for primary regression investigation
  - **Spec Closure**: This spec closes as "secondary issue resolved"
  - See: INVESTIGATION_SUMMARY.md for detailed analysis
  - See: New spec "scheduler-primary-regression-rca" for continuation
