# Patch I-C2: Canonical Stub + Skip CR3 Pivot A/B Test Plan

**Date**: 2026-04-19  
**Authority**: Kenan AY - Architectural Steward  
**Context**: PCID blocked by freeze guard, ENTRY_GUARD locked by contract

## Objective

Measure the causal impact of skipping CR3 pivot in canonical fetch stub path on Ring3 entry window performance via controlled A/B test.

## Hypothesis

CR3 pivot (address space switch) may be a major cost in the Ring3 entry path. If CR3 pivot is expensive, skipping it via canonical fetch stub should show significant improvement (>10%) in entry_latency_ticks.

**Expected**: If CR3 pivot is the bottleneck, entry latency should improve by 10-30%.

## Architectural Safety

### What This Configuration Does

**AYKEN_RING3_CANONICAL_FETCH_STUB=1**:
- Uses canonical (higher-half) address space for Ring3 code
- Allows Ring3 code to execute in kernel address space
- Enables CR3 pivot optimization

**AYKEN_RING3_SKIP_CR3_PIVOT=1** (requires CANONICAL_FETCH_STUB=1):
- Skips CR3 switch during Ring0→Ring3 transition
- Keeps kernel CR3 active in Ring3
- Eliminates TLB flush from CR3 switch
- Ring3 code executes in canonical window (shared with kernel)

### Safety Guarantees

**This configuration does NOT change**:
- Ring0/Ring3 privilege separation (still enforced by CPU)
- Memory isolation (page table permissions still enforced)
- Syscall ABI surface
- Boundary enforcement contract
- Functional correctness

**This configuration only affects**:
- CR3 switch behavior (skipped vs performed)
- TLB flush frequency (reduced)
- Address space layout (canonical window shared)

**Architectural compliance**:
- ✅ Ring0 = mechanism only (CR3 is mechanism)
- ✅ Ring3 = policy only (unchanged)
- ✅ ABI freeze preserved
- ✅ Boundary gate preserved
- ✅ Memory safety preserved (page permissions still enforced)

### Why This is Safe

**Privilege separation is CPU-enforced**:
- Ring3 cannot access Ring0 pages (page table permissions)
- Ring3 cannot execute privileged instructions (CPU enforces)
- Syscall/interrupt gates still required for Ring0 access

**Memory isolation is page-table-enforced**:
- Ring3 pages marked user-accessible
- Ring0 pages marked supervisor-only
- CPU enforces on every memory access

**CR3 is an optimization, not a security boundary**:
- Security comes from page table permissions, not CR3 value
- Skipping CR3 switch is safe if page permissions are correct

## Change

**Two Variables** (required together):
- `AYKEN_RING3_CANONICAL_FETCH_STUB=1` (was 0)
- `AYKEN_RING3_SKIP_CR3_PIVOT=1` (was 0)

**Control**:
- All other flags identical to Patch G baseline
- AYKEN_CR3_PCID=0 (reverted from Patch I-C1)
- AYKEN_RING3_POST_CR3_TEXT_PROBE=0 (from Patch I-A)
- AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0 (from Patch F)
- AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0 (from Patch G)
- AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE=0 (profiling OFF)

## Validation Criteria

**Test Integrity**:
- preempt_iret_count == 61 (test completed normally)
- No profiling overhead
- Clean measurement
- All constitutional gates PASS

**Performance Metrics** (vs Patch G baseline):
- entry_latency_ticks: 22,646,771 (baseline)
- syscall_latency_ms_proxy: 204.07ms (baseline)
- boot_time_ms: 12,708ms (baseline, ignore artifacts)

## Decision Rules

**If entry_latency_ticks improvement**:
- **≥30%**: CR3 pivot is primary bottleneck → major finding
- **10-30%**: CR3 pivot is significant contributor → partial cause
- **<10%**: CR3 pivot is not primary bottleneck → continue investigation

## Expected Outcome

**If CR3 pivot is expensive**:
- entry_latency_ticks: 22.6M → 15.8M-20.3M (10-30% reduction)
- syscall_latency_ms_proxy: 204ms → 143-184ms (10-30% reduction)

**If CR3 pivot is not the bottleneck**:
- entry_latency_ticks: ~22.6M (minimal change)
- Continue to Patch I-C3 (self-reload CR3) or return to profiling

## Freeze Guard Compatibility

**Verified**: These flags are NOT locked by freeze guard

**Freeze guard only checks**:
- AYKEN_SCHED_FALLBACK=0
- PHASE10C_ENFORCE=1
- AYKEN_CR3_PCID=0
- AYKEN_SCHED_BOOTSTRAP_POLICY=0

**These flags are NOT in freeze guard**:
- ✅ AYKEN_RING3_CANONICAL_FETCH_STUB (can be toggled)
- ✅ AYKEN_RING3_SKIP_CR3_PIVOT (can be toggled)

**Measurement contract**: Need to verify these are not overridden at runtime.

## Risk Assessment

**Architectural Risk**: LOW
- Privilege separation still CPU-enforced
- Memory isolation still page-table-enforced
- No ABI change
- No boundary enforcement change

**Measurement Risk**: LOW
- Not locked by freeze guard
- Not locked by measurement contract (to be verified)
- Clean toggle via Makefile

**Functional Risk**: LOW
- Configuration already exists in codebase
- Has dependency checks (SKIP_CR3_PIVOT requires CANONICAL_FETCH_STUB)
- Used in validation/testing scenarios

**Rollback**: TRIVIAL
- Set both flags to 0
- No code changes required

## Implementation Notes

### Dependency Rules

**Makefile enforces** (lines 338-342):
```makefile
ifeq ($(AYKEN_RING3_SKIP_CR3_PIVOT),1)
ifeq ($(AYKEN_RING3_CANONICAL_FETCH_STUB),0)
$(error AYKEN_RING3_SKIP_CR3_PIVOT=1 requires AYKEN_RING3_CANONICAL_FETCH_STUB=1)
endif
endif
```

**Mutual exclusion** (lines 380-384):
```makefile
ifeq ($(AYKEN_RING3_SKIP_CR3_PIVOT),1)
ifeq ($(AYKEN_RING3_SELF_RELOAD_CR3),1)
$(error AYKEN_RING3_SKIP_CR3_PIVOT=1 and AYKEN_RING3_SELF_RELOAD_CR3=1 are mutually exclusive)
endif
endif
```

### What Happens in Code

**With CANONICAL_FETCH_STUB=1 + SKIP_CR3_PIVOT=1**:
1. Ring3 code placed in canonical (higher-half) address space
2. Ring0→Ring3 transition skips `mov cr3, <user_cr3>`
3. Ring3 executes with kernel CR3 active
4. Page table permissions still enforce Ring3 restrictions
5. No TLB flush from CR3 switch
6. Ring3→Ring0 transition (syscall) also skips CR3 switch

**Performance impact**:
- Eliminates CR3 write instruction (~100s of cycles)
- Eliminates TLB flush (1000s-10000s of cycles depending on TLB state)
- Reduces entry window latency

## Next Steps

1. Modify `Makefile`: Set both flags to 1
2. Commit with evidence-based message
3. Push and analyze CI results
4. Verify flags not overridden by contract
5. Compare entry_latency_ticks to baseline
6. Document findings in `PATCH_I_C2_RESULTS.md`

**If CR3 pivot shows improvement (≥10%)**:
- Document as significant contributor
- Consider if this is acceptable for production

**If CR3 pivot shows minimal impact (<10%)**:
- Rule out CR3 pivot
- Test Patch I-C3 (self-reload CR3) or return to profiling

## References

- Patch G baseline: CI run 24637379135
- Patch I-A: TEXT_PROOF ruled out (0.97%)
- Patch I-B: ENTRY_GUARD untestable (contract lock)
- Patch I-C1: PCID untestable (freeze guard lock)
- Makefile dependency rules: lines 338-342, 380-384

## Key Insight

**CR3 pivot is the most likely remaining bottleneck** because:
1. TEXT_PROOF ruled out (0.97% impact)
2. ENTRY_GUARD untestable (locked)
3. PCID untestable (locked)
4. CR3 switch is expensive (TLB flush + pipeline serialization)
5. Entry window is 22.6M ticks (71% of total latency)
6. CR3 pivot happens in entry window

**This is the strongest remaining hypothesis for the ~9.5% unexplained regression.**

---

**Status**: Ready for implementation  
**Risk**: LOW (privilege/memory isolation preserved)  
**Rollback**: TRIVIAL (two flags)  
**Next**: Modify Makefile and commit

