# Patch C Verification Plan

> **2026-04-19 correction:** This plan is superseded. Marker checks must read the freeze artifact
> `evidence/run-*/gates/performance/boot-audit/qemu_debugcon.log`, not only `gh run view --log`.
> Production performance runs now keep syscall diagnostics disabled by default.

**Date**: 2026-04-19  
**Status**: AWAITING CI EVIDENCE  
**Commit**: c0830308

## Situation

Patch C (Context Type Cache + Bypass Fast-Path) shows ZERO performance impact in CI Run 24633589543:
- boot_time_ms: 12709 (Patch B: 12714, diff: -5ms = noise)
- syscall_latency: 208.16ms (Patch B: 207.90ms, diff: +0.26ms = noise)

Critical finding: Hot-path markers (`DIAG_HOT_*`) are MISSING from CI logs.

## Hypothesis

**Most Likely**: Enforcement code is executing, but hot-path markers are conditional on `AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=1`.

**Evidence**:
- Makefile default: `AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE ?= 1` (line 100)
- CI performance gate does NOT override this flag
- Therefore, enforcement SHOULD be enabled in CI

**Problem**: Hot-path markers are inside `#if AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE` blocks, so if they're missing, either:
1. Enforcement is disabled (contradicts Makefile default)
2. Markers are not being emitted for another reason
3. Execution path bypasses the marker code

## Verification Strategy

### Added Unconditional Markers (Commit c0830308)

**1. Cache Verification** (`syscall_v2_hardened.c`):
```c
/* PATCH C VERIFICATION: Forced marker to prove cache path is executing */
if (current_proc && current_proc->boundary_cache_valid) {
    debugcon_write_with_timestamp("PATCH_C_CACHE_HIT");
} else {
    debugcon_write_with_timestamp("PATCH_C_CACHE_MISS");
}
```

**2. Bypass Path Verification** (`boundary_enforcement.c`):
```c
/* PATCH C2 VERIFICATION: Forced marker to prove fast-path is executing */
if (ctx_type != EXEC_CONTEXT_BCIB && ctx_type != EXEC_CONTEXT_RUNTIME_BRIDGE) {
    debugcon_write("PATCH_C2_FAST_PATH\n");
    // ... fast path ...
} else {
    debugcon_write("PATCH_C2_SLOW_PATH\n");
    // ... slow path ...
}
```

**Key**: These markers are OUTSIDE `#if` blocks, so they will appear regardless of enforcement flag.

## Expected CI Evidence

### Scenario 1: Markers Appear (Patch C Executing)
**Evidence**:
- `PATCH_C_CACHE_HIT` or `PATCH_C_CACHE_MISS` in CI log
- `PATCH_C2_FAST_PATH` or `PATCH_C2_SLOW_PATH` in CI log

**Interpretation**:
- Patch C code IS executing
- Optimization is insufficient or wrong target
- Need to re-measure hot-path distribution

**Next Steps**:
1. Run hot-path analyzer on CI log
2. Check if segment costs changed
3. Investigate why optimization has no effect

### Scenario 2: Markers Missing (Patch C NOT Executing)
**Evidence**:
- NO `PATCH_C_*` markers in CI log
- NO `PATCH_C2_*` markers in CI log

**Interpretation**:
- Patch C code is NOT executing (dead code)
- Execution path bypasses syscall handler
- OR: CI uses different test harness

**Next Steps**:
1. Verify syscall handler is `syscall_v2_hardened_handler`
2. Check if `current_proc` is NULL (kernel context)
3. Investigate CI test harness execution path

### Scenario 3: Partial Markers (Mixed Execution)
**Evidence**:
- `PATCH_C_CACHE_*` present but `PATCH_C2_*` missing
- OR: vice versa

**Interpretation**:
- Partial execution path
- Some optimizations active, others bypassed

**Next Steps**:
1. Identify which path is executing
2. Investigate why other path is bypassed

## Diagnostic Checklist

### Before CI Push
- ✅ Verification markers added (commit c0830308)
- ✅ Markers are unconditional (outside `#if` blocks)
- ✅ Diagnostic document created
- ⏳ Local test to verify markers appear

### After CI Run
- ⏳ Download CI artifacts
- ⏳ Search for `PATCH_C_*` markers in debugcon log
- ⏳ Search for `PATCH_C2_*` markers in debugcon log
- ⏳ Run hot-path analyzer if markers present
- ⏳ Document findings

### If Markers Present
- ⏳ Extract marker counts (cache hit/miss ratio)
- ⏳ Extract fast/slow path ratio
- ⏳ Re-measure hot-path segment costs
- ⏳ Compare with Patch B baseline

### If Markers Missing
- ⏳ Check syscall handler symbol in kernel.elf
- ⏳ Verify `current_proc` initialization
- ⏳ Trace execution path from Ring3 entry
- ⏳ Check CI test harness code

## Success Criteria

### Minimum Viable Evidence
- Verification markers appear in CI log
- Execution path confirmed (cache hit/miss, fast/slow path)
- Root cause of zero impact identified

### Performance Target (If Optimization Active)
- syscall_latency: <192ms (+10% of baseline 175ms)
- boot_time: <11752ms (+10% of baseline 10684ms)
- context_switch: <192ms (+10% of baseline 175ms)

### Diagnostic Target (If Optimization Inactive)
- Identify why Patch C code not executing
- Document execution path mismatch
- Propose corrective action

## Risk Assessment

### Risk 1: Markers Still Missing (HIGH)
**Scenario**: Next CI run shows no `PATCH_C_*` markers

**Impact**: Patch C code definitively not executing

**Mitigation**: Deep dive into execution path, add more markers

### Risk 2: Markers Present but No Impact (MEDIUM)
**Scenario**: Markers appear but performance unchanged

**Impact**: Optimization insufficient or wrong target

**Mitigation**: Re-profile hot-path, identify remaining bottleneck

### Risk 3: CI Environment Mismatch (LOW)
**Scenario**: Local shows markers, CI doesn't

**Impact**: CI uses different execution path

**Mitigation**: Replicate CI environment locally, debug

## Timeline

### Immediate (Now)
- ✅ Commit verification markers (c0830308)
- ⏳ Push to CI
- ⏳ Monitor CI run

### After CI Run (~30 min)
- ⏳ Download artifacts
- ⏳ Search for markers
- ⏳ Document findings

### Next Action (Depends on Evidence)
- **If markers present**: Re-measure hot-path, investigate insufficient optimization
- **If markers missing**: Deep dive into execution path, add more markers
- **If partial markers**: Identify execution path split, debug

## References

- Zero Impact Diagnosis: `PATCH_C_ZERO_IMPACT_DIAGNOSIS.md`
- Patch C Design: `PATCH_C_DESIGN.md`
- Patch B Verdict: `PATCH_B_CI_VERDICT.md`
- Hot-path Analyzer: `scripts/ci/analyze_enforcement_hotpath.py`

## Commit Message

```
Add Patch C execution verification markers

CRITICAL: Patch C has zero performance impact in CI (Run 24633589543).
Hot-path markers (DIAG_HOT_*) are missing from CI logs.

Root cause hypothesis:
1. Enforcement disabled in CI (most likely)
2. Execution path bypasses Patch C code
3. Wrong measurement target

Verification strategy:
- Added unconditional markers outside #if blocks
- PATCH_C_CACHE_HIT/MISS: Verify cache is being read
- PATCH_C2_FAST_PATH/SLOW_PATH: Verify bypass optimization executes

Next CI run will prove:
- If markers missing: Patch C code not executing (dead code)
- If markers present: Need to re-measure hot-path distribution

Files changed:
- kernel/sys/syscall_v2_hardened.c: Cache verification marker
- kernel/sys/boundary_enforcement.c: Bypass path verification markers
- docs/specs/scheduler-primary-regression-rca/PATCH_C_ZERO_IMPACT_DIAGNOSIS.md: RCA

Ref: User diagnosis 'Cost hesapladığın yerde değil' confirmed by missing markers.
```

## Next CI Run: Awaiting Evidence

**Commit**: c0830308  
**Branch**: fix/scheduler-fast-path  
**Expected Duration**: ~30 minutes  
**Critical Evidence**: Presence/absence of `PATCH_C_*` markers in CI log
