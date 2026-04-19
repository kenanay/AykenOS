# CI Measurement Failure Root Cause Analysis

**Date**: 2026-04-19  
**CI Run**: 24632513605  
**Status**: BLOCKED - Metrics missing, cannot verify Patch B

## Problem Statement

Performance gate fails with missing metrics:
- `syscall_latency_ms_proxy`: MISSING (baseline: 175ms)
- `context_switch_latency_ms_proxy`: MISSING (baseline: 175ms)
- `boot_time_ms`: 12717ms (+19% regression, measured)

**Impact**: Cannot verify Patch B performance improvement. Optimization is implemented but unverified.

## Root Cause

### Primary Cause: Preempt Test Failure

```
preempt_test_failed:run_preempt_test.sh
preempt_marker_missing:sw_count=0
preempt_marker_missing:iret_count=0
context_switch_latency_proxy_invalid:INF
syscall_latency_proxy_invalid:INF
```

**Analysis**:
1. Preempt test harness (`run_preempt_test.sh`) fails to execute
2. No context switch markers (`[SW|MARK:SW]`) in output
3. No interrupt return markers (`[IRET markers]`) in output
4. Without markers, latency metrics cannot be calculated
5. Metrics default to INF → reported as MISSING

### Secondary Cause: Test Environment

**Evidence from CI log**:
```
measurement_contract: deterministic_preempt_harness
preempt_user_minimal_mode: syscall-v2-runtime
```

The test uses `syscall-v2-runtime` mode, which requires:
- Deterministic preemption enabled
- Timer interrupts firing correctly
- Context switches happening
- Markers being emitted

**Hypothesis**: One of these requirements is not met in current build.

## Potential Root Causes (Ranked by Likelihood)

### 1. AYKEN_RING3_FETCH_PROBE Interference (HIGH)

**Evidence**: Task 1 blocker was caused by `AYKEN_RING3_FETCH_PROBE=1` creating infinite loop in CPL0.

**Hypothesis**: If this flag is still enabled in CI build:
- Infinite loop blocks timer interrupts
- No preemption occurs
- No markers emitted
- Metrics missing

**Check**: Verify CI build flags for `AYKEN_RING3_FETCH_PROBE`

### 2. Patch B Breaking Preempt Path (MEDIUM)

**Evidence**: Metrics were present before Patch B, missing after.

**Hypothesis**: Fast-path bitmask optimization may have:
- Broken syscall path in subtle way
- Caused early termination
- Prevented marker emission
- Blocked preemption

**Check**: Compare syscall handler execution before/after Patch B

### 3. Static State Bug Side Effects (MEDIUM)

**Evidence**: Header-only static state caused ODR violation.

**Hypothesis**: Before fix (commit 15cc9210):
- Multiple table instances existed
- Init in one TU, read from another
- Undefined behavior in enforcement checks
- Possible crash/hang in syscall path

**Check**: Verify if fix (15cc9210) resolves metrics

### 4. Build Configuration Drift (LOW)

**Evidence**: CI uses different build flags than local.

**Hypothesis**: CI build may have:
- Wrong USER_MINIMAL_MODE
- Missing diagnostic flags
- Incorrect QEMU timeout
- Wrong kernel profile

**Check**: Compare CI build command with working baseline

## Diagnostic Steps

### Step 1: Check CI Build Flags

```bash
gh run view 24632513605 --log | grep -E "AYKEN_.*=" | grep -E "FETCH_PROBE|USER_MINIMAL"
```

Expected:
- `AYKEN_RING3_FETCH_PROBE=0` (not 1)
- `USER_MINIMAL_MODE=syscall-v2-runtime`

### Step 2: Check QEMU Execution

```bash
gh run view 24632513605 --log | grep -E "qemu-system|QEMU.*timeout"
```

Look for:
- QEMU command line
- Timeout value
- Exit code
- Crash/hang indicators

### Step 3: Check Marker Emission

```bash
gh run view 24632513605 --log | grep -E "\[SW\|MARK:SW\]|\[IRET markers\]"
```

Expected: Multiple markers in output  
Actual: Zero markers (confirmed)

### Step 4: Check Syscall Path

```bash
gh run view 24632513605 --log | grep -E "DIAG_HOT_.*_ENTER|DIAG_HOT_.*_DONE"
```

Look for: Hot-path diagnostic markers from Patch B

## Immediate Actions

### Action 1: Verify Static State Fix (15cc9210)

**Status**: Pushed, awaiting CI  
**Expected**: Metrics should appear if this was the cause  
**Timeline**: ~5 minutes for CI run

### Action 2: Disable FETCH_PROBE in CI

**If**: Metrics still missing after 15cc9210  
**Then**: Explicitly set `AYKEN_RING3_FETCH_PROBE=0` in CI build  
**How**: Check Makefile or CI workflow for flag override

### Action 3: Isolate Patch B Impact

**If**: Metrics still missing  
**Then**: Temporarily revert Patch B to verify it's not the cause  
**How**: `git revert 9a3402f9` (Patch B commit)

### Action 4: Check Preempt Test Harness

**If**: All above fail  
**Then**: Debug `run_preempt_test.sh` directly  
**How**: Run locally with same flags as CI

## Success Criteria

Metrics are considered "fixed" when CI shows:
- `syscall_latency_ms_proxy`: Numeric value (not MISSING)
- `context_switch_latency_ms_proxy`: Numeric value (not MISSING)
- `sw_count > 0` and `iret_count > 0` in preempt markers

## Next Steps After Fix

Once metrics are available:

1. **Verify Patch B Impact**:
   - Compare syscall_latency before/after Patch B
   - Target: 207ms → ~180ms (closer to 175ms baseline)
   - Verify validate_syscall cost reduction

2. **Assess Boot Regression**:
   - Current: 12717ms (+19%)
   - Investigate why boot is still slow
   - May need additional optimization

3. **Proceed to Patch C** (if Patch B insufficient):
   - Context type cache
   - Move ctx_type out of syscall path

## Conclusion

**Current State**: Patch B implemented and static-state bug fixed, but performance verification blocked by CI measurement failure.

**Root Cause**: Preempt test harness not producing markers → metrics cannot be calculated.

**Most Likely Cause**: Static state bug (before 15cc9210) or FETCH_PROBE interference.

**Next Milestone**: Wait for CI run 15cc9210 to complete. If metrics still missing, escalate to preempt harness debugging.

**Confidence**: MEDIUM - Multiple potential causes, need CI evidence to narrow down.
