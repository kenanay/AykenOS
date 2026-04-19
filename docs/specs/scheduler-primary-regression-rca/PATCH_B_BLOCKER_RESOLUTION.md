# Patch B Blocker Resolution

**Date**: 2026-04-19  
**Status**: ROOT CAUSE IDENTIFIED - Fix ready  
**CI Run**: 24632513605 (failed with missing metrics)

## Executive Summary

**Problem**: CI performance metrics missing (syscall_latency_ms_proxy, context_switch_latency_ms_proxy) → Cannot verify Patch B optimization.

**Root Cause**: `AYKEN_RING3_FETCH_PROBE=1` in Makefile default → Diagnostic code interferes with preempt test → No markers emitted → Metrics cannot be calculated.

**Solution**: Change Makefile default to `AYKEN_RING3_FETCH_PROBE ?= 0` (production mode).

**Impact**: This is the SAME diagnostic flag that caused Task 1 false blocker (infinite loop in CPL0). It was resolved for local testing but never fixed in Makefile default.

## Technical Analysis

### 1. Static State Bug (RESOLVED)

**Issue**: Header-only static state caused ODR violation.

**Evidence**:
```c
// BEFORE (syscall_enforcement_matrix_fast.h):
static syscall_enforcement_fast_entry_t SYSCALL_ENFORCEMENT_FAST_TABLE[...];
static int syscall_enforcement_fast_initialized = 0;
```

**Fix** (commit 15cc9210):
```c
// AFTER (syscall_enforcement_matrix_fast.c):
syscall_enforcement_fast_entry_t SYSCALL_ENFORCEMENT_FAST_TABLE[...];
int syscall_enforcement_fast_initialized = 0;

// Header only has extern declarations:
extern syscall_enforcement_fast_entry_t SYSCALL_ENFORCEMENT_FAST_TABLE[...];
extern int syscall_enforcement_fast_initialized;
```

**Status**: ✅ FIXED - Single instance in .c file, extern in header.

### 2. AYKEN_RING3_FETCH_PROBE Interference (PRIMARY BLOCKER)

**Issue**: Makefile default enables diagnostic code that breaks preempt test.

**Evidence**:
```makefile
# Makefile line 58:
AYKEN_RING3_FETCH_PROBE ?= 1  # ❌ DIAGNOSTIC MODE (breaks CI)
```

**Impact**:
- Diagnostic code in `kernel/arch/x86_64/ring3_enter.S` activates
- Infinite loop or blocking behavior in Ring3 entry path
- Timer interrupts blocked or delayed
- Preempt test cannot generate context switch markers
- No `[SW|MARK:SW]` or `[IRET markers]` in output
- Metrics calculation fails → MISSING

**Historical Context**:
- Task 1 identified this as false blocker (infinite loop in CPL0)
- Fixed locally by setting `AYKEN_RING3_FETCH_PROBE=0` in test harness
- Never fixed in Makefile default → CI still uses diagnostic mode

**Fix**:
```makefile
# Change Makefile line 58:
AYKEN_RING3_FETCH_PROBE ?= 0  # ✅ PRODUCTION MODE (CI compatible)
```

### 3. Preempt Test Failure Chain

**Failure Sequence**:
1. CI builds with `AYKEN_RING3_FETCH_PROBE=1` (Makefile default)
2. Kernel boots with diagnostic code active
3. `run_preempt_test.sh` executes `syscall-v2-runtime` mode
4. Diagnostic code interferes with Ring3 entry
5. Timer interrupts blocked or delayed
6. No context switches occur
7. No markers emitted: `sw_count=0`, `iret_count=0`
8. Metrics calculation fails: `syscall_latency_ms_proxy=MISSING`

**Evidence from CI log**:
```
preempt_test_failed:run_preempt_test.sh
preempt_marker_missing:sw_count=0
preempt_marker_missing:iret_count=0
context_switch_latency_proxy_invalid:INF
syscall_latency_proxy_invalid:INF
```

## Fix Implementation

### Change 1: Makefile Default

**File**: `Makefile`  
**Line**: 58  
**Change**:
```makefile
# BEFORE:
AYKEN_RING3_FETCH_PROBE ?= 1

# AFTER:
AYKEN_RING3_FETCH_PROBE ?= 0
```

**Rationale**:
- Production builds should NOT have diagnostic code active
- Diagnostic mode is for local debugging only
- CI must run in production mode to get valid metrics
- Explicit override still possible: `make AYKEN_RING3_FETCH_PROBE=1`

### Change 2: Preservation Test Update

**File**: `scripts/ci/verify_diagnostic_flags.sh`  
**Status**: Already enforces `AYKEN_RING3_FETCH_PROBE=0` in production  
**Action**: No change needed (test will pass after Makefile fix)

## Verification Plan

### Step 1: Apply Fix
```bash
# Change Makefile line 58
git add Makefile
git commit -m "fix(build): Disable AYKEN_RING3_FETCH_PROBE by default for CI compatibility"
git push
```

### Step 2: Wait for CI
- CI run should complete in ~5 minutes
- Check performance gate output

### Step 3: Verify Metrics Present
Expected CI output:
```
syscall_latency_ms_proxy: <numeric value>  # NOT MISSING
context_switch_latency_ms_proxy: <numeric value>  # NOT MISSING
sw_count: >0
iret_count: >0
```

### Step 4: Assess Patch B Impact
Once metrics are available:
- Compare syscall_latency before/after Patch B
- Target: 207ms → ~180ms (closer to 175ms baseline)
- Verify boot_time_ms not regressed further

## Success Criteria

**Immediate Success** (metrics available):
- ✅ `syscall_latency_ms_proxy`: numeric value
- ✅ `context_switch_latency_ms_proxy`: numeric value
- ✅ `sw_count > 0` and `iret_count > 0`

**Patch B Success** (performance improvement):
- ✅ `syscall_latency_ms_proxy` < 207ms (current)
- ✅ `boot_time_ms` ≈ baseline (10684ms ±5%)
- ✅ `context_switch_latency_ms_proxy` improved

**If Patch B Insufficient**:
- Proceed to Patch C (context type cache)
- Target: Move `boundary_set_context_type()` out of hot-path

## Lessons Learned

### 1. Diagnostic Flags Must Default to Production Mode
- Development diagnostics should be opt-in, not opt-out
- CI must run in production configuration
- Local debugging can override with explicit flags

### 2. Test Harness Isolation Insufficient
- `run_preempt_test.sh` sets `AYKEN_RING3_FETCH_PROBE=0` explicitly
- But Makefile default overrides this during build
- Build-time flags must align with runtime expectations

### 3. Static State in Headers is ODR Violation
- Multiple translation units → multiple instances
- Init in one TU, read from another → undefined behavior
- Always: single instance in .c, extern in header

## Next Steps

1. **Immediate**: Apply Makefile fix, push, wait for CI
2. **After metrics available**: Assess Patch B performance impact
3. **If insufficient**: Implement Patch C (context type cache)
4. **Final**: Update Task 3 status based on CI verdict

## Confidence Level

**Root Cause Confidence**: HIGH (99%)
- Same flag caused Task 1 blocker
- Makefile default clearly wrong for CI
- Preempt test failure symptoms match diagnostic interference

**Fix Confidence**: HIGH (95%)
- Simple one-line change
- Preservation test already enforces this
- No side effects expected

**Patch B Success Confidence**: MEDIUM (60%)
- Implementation correct (bitmask optimization)
- Static state bug fixed
- But performance gain not yet measured
- May need Patch C for full baseline recovery
