# Patch I-C1: PCID A/B Test Plan

**Date**: 2026-04-19  
**Authority**: Kenan AY - Architectural Steward  
**Context**: TEXT_PROOF ruled out (0.97%), ENTRY_GUARD untestable (contract lock)

## Objective

Measure the causal impact of PCID (Process-Context Identifiers) on CR3 switch cost via controlled A/B test.

## Hypothesis

PCID may reduce TLB flush overhead during CR3 switches. If PCID is beneficial, enabling it should show measurable improvement in entry_latency_ticks.

**Expected**: If PCID reduces TLB flush cost, entry latency should improve by 5-15%.

## Architectural Safety

### What PCID Does

PCID is a CPU feature that tags TLB entries with a process context ID, allowing:
- Selective TLB invalidation (instead of full flush)
- TLB entries from multiple address spaces to coexist
- Reduced TLB miss rate after CR3 switch

### Safety Guarantees

**PCID does NOT change**:
- Ring0/Ring3 isolation model
- Address space separation
- Syscall ABI surface
- Boundary enforcement contract
- Memory safety guarantees

**PCID only affects**:
- TLB flush behavior (hardware optimization)
- CR3 switch micro-cost
- TLB miss rate

**Architectural compliance**:
- ✅ Ring0 = mechanism only (PCID is mechanism)
- ✅ Ring3 = policy only (unchanged)
- ✅ ABI freeze preserved
- ✅ Boundary gate preserved
- ✅ CI contract preserved

## Change

**Single Variable**:
- `AYKEN_CR3_PCID=1` (was 0)

**Control**:
- All other flags identical to Patch G baseline
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
- **≥15%**: PCID is significant optimization → keep enabled, investigate further
- **5-15%**: PCID is partial contributor → keep enabled, continue testing other segments
- **<5%**: PCID has minimal impact → rule out, test next segment

## Expected Outcome

**If PCID reduces TLB flush cost**:
- entry_latency_ticks: 22.6M → 19.2M-21.5M (5-15% reduction)
- syscall_latency_ms_proxy: 204ms → 173-194ms (5-15% reduction)

**If PCID has minimal impact**:
- entry_latency_ticks: ~22.6M (minimal change)
- Continue to Patch I-C2 (canonical stub + skip pivot)

## Measurement Contract Compatibility

**PCID is NOT locked by measurement contract**:
- Contract enforces: ENTRY_GUARD, DETERMINISTIC_EXIT, BOOTSTRAP_POLICY
- Contract does NOT enforce: CR3_PCID
- PCID can be toggled via Makefile without contract override

**Verification**:
- Check `preempt_contract_cr3_pcid` in CI report
- Should reflect Makefile value (=1)
- If overridden, test is invalid (like Patch I-B)

## Risk Assessment

**Architectural Risk**: MINIMAL
- PCID is hardware optimization only
- Does not change isolation model
- Does not change ABI surface
- Does not change boundary enforcement

**Measurement Risk**: LOW
- PCID is not locked by contract
- Can be toggled cleanly
- No measurement model invalidation

**Rollback**: TRIVIAL
- Set AYKEN_CR3_PCID=0
- No code changes required
- No ABI impact

## Implementation Notes

### PCID Requirements

**CPU Support**:
- x86_64 PCID feature (CPUID.01H:ECX.PCID[bit 17])
- CR4.PCIDE must be set
- PCID values in CR3[11:0]

**AykenOS Implementation**:
- PCID support already implemented in kernel
- Flag controls whether PCID is enabled
- Default is disabled (=0)

### PCID Behavior

**When PCID=1**:
- CR3 writes include PCID tag
- TLB entries tagged with PCID
- INVPCID instruction used for selective invalidation
- TLB flush cost reduced

**When PCID=0**:
- CR3 writes flush entire TLB
- No PCID tagging
- Full TLB flush on every CR3 switch

## Next Steps

1. Modify `Makefile`: Set `AYKEN_CR3_PCID=1`
2. Commit with evidence-based message
3. Push and analyze CI results
4. Verify `preempt_contract_cr3_pcid=1` in report (not overridden)
5. Compare entry_latency_ticks to baseline
6. Document findings in `PATCH_I_C1_RESULTS.md`

**If PCID shows improvement (≥5%)**:
- Keep PCID enabled
- Continue to Patch I-C2 (canonical stub tests)

**If PCID shows minimal impact (<5%)**:
- Rule out PCID
- Continue to Patch I-C2 (canonical stub tests)

## References

- Patch G baseline: CI run 24637379135
- Patch I-A: TEXT_PROOF ruled out (0.97%)
- Patch I-B: ENTRY_GUARD untestable (contract lock)
- Intel SDM Vol 3A: Section 4.10.1 (PCID)

## Key Insight

**PCID is the safest first CR3 test** because:
1. Hardware optimization only (no architectural change)
2. Not locked by measurement contract
3. Directly targets TLB flush cost
4. Trivial rollback
5. No isolation impact

**This is architecture-preserving performance forensics**, not architectural change.

---

**Status**: Ready for implementation  
**Risk**: MINIMAL (hardware optimization only)  
**Rollback**: TRIVIAL (single flag)  
**Next**: Modify Makefile and commit

