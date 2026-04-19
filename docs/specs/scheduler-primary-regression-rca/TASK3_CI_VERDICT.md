# Task 3.1 CI Verdict: REGRESSION DETECTED

**Authority**: GitHub CI (ubuntu-24.04-x64)  
**Run**: 24632031864  
**Date**: 2026-04-19T15:02:36Z  
**Verdict**: ❌ FAIL - Case D (Full Regression)

## Performance Metrics (Authoritative CI)

| Metric | Baseline | Actual | Threshold | Delta | Status |
|--------|----------|--------|-----------|-------|--------|
| `boot_time_ms` | 10684 | 12711 | 11752 | +19.0% | ❌ FAIL |
| `syscall_latency_ms_proxy` | 175.08 | 207.87 | 183.84 | +18.7% | ❌ FAIL |
| `context_switch_latency_ms_proxy` | 175.08 | 207.87 | 183.84 | +18.7% | ❌ FAIL |

## Decision Matrix Classification

**Case D: REGRESSION**
- boot ↑ (+19.0%)
- syscall ↑ (+18.7%)
- context ↑ (+18.7%)

**Interpretation**: Init cost was NOT eliminated. Instead:
1. Init moved to boot → boot time increased
2. Init logic leaked to syscall path → syscall latency increased
3. Init logic leaked to scheduler → context switch latency increased

## Root Cause Analysis

### What Happened
Moving `boundary_enforce_init()` to `kernel_late_init()` did NOT eliminate the init cost. Instead, it:
1. Added boot-time overhead (validation matrix scan)
2. Failed to remove the actual expensive operations from hot paths
3. Possibly introduced new overhead in syscall/context-switch paths

### Evidence
- Boot init markers present: `DIAG_BOUNDARY_INIT_BOOT_ENTER/DONE` ✓
- Syscall skip markers present: `DIAG_BOUNDARY_INIT_SKIPPED` ✓
- But performance degraded across ALL metrics ❌

### Why This Happened
The optimization moved the MARKER logic but not the COST. The expensive operations are:
1. `boundary_set_context_type()` - called on EVERY syscall
2. `boundary_validate_syscall()` - called on EVERY syscall
3. `boundary_detect_bridge_bypass()` - called on EVERY syscall

These are NOT init operations - they are per-syscall enforcement checks.

## Local vs CI Comparison

| Environment | boot_time_ms | syscall_latency | context_switch |
|-------------|--------------|-----------------|----------------|
| Local (Mac ARM64) | 13878 (+29.9%) | ~225 | ~225 |
| CI (Linux x64) | 12711 (+19.0%) | 207.87 (+18.7%) | 207.87 (+18.7%) |

Both environments show regression, confirming this is NOT an environment artifact.

## Conclusion

**Task 3.1 Status**: ❌ FAILED

The implementation is INCORRECT. Moving `boundary_enforce_init()` to boot was the wrong optimization target. The real cost is in per-syscall enforcement checks, not one-time initialization.

## Next Steps

1. **Revert Task 3.1 changes** - this optimization made things worse
2. **Re-analyze first-syscall hotspot** - identify the REAL expensive operation
3. **Target correct optimization** - likely need to optimize per-syscall checks, not init
4. **Verify with profiling** - use detailed profiling to find actual bottleneck

## Lessons Learned

1. ✅ Preservation tests passed - but they only verify BEHAVIOR, not PERFORMANCE
2. ❌ Performance assumption was wrong - init was not the bottleneck
3. ✅ CI authority was essential - local measurements were misleading
4. ❌ "Init at boot" was a premature optimization without profiling data

**User was correct**: "init cost may have been moved, not eliminated"
