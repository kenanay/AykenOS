# Patch I-C1 Status: PCID A/B Test

**Date**: 2026-04-19  
**Status**: CI IN PROGRESS  
**CI Run**: 24638819404  
**Commit**: d218a5c3

## What Was Done

1. Created `PATCH_I_C1_PCID_AB_TEST_PLAN.md` with detailed test plan
2. Modified `Makefile`: Set `AYKEN_CR3_PCID=1` (was 0)
3. Committed with comprehensive evidence-based message
4. Pushed to branch `test/diagnostic-markers-ab-test`

## Test Configuration

**Changed**:
- AYKEN_CR3_PCID=1 (was 0)

**Unchanged** (control variables):
- AYKEN_RING3_POST_CR3_TEXT_PROBE=0 (from Patch I-A)
- AYKEN_RING3_ENTRY_GUARD=0 (attempted in Patch I-B, but overridden)
- AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0 (from Patch F)
- AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0 (from Patch G)
- AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE=0 (profiling OFF)

## Architectural Safety

**PCID is the safest CR3 test** because:
- Hardware optimization only (no architectural change)
- Does NOT change Ring0/Ring3 isolation
- Does NOT change address space separation
- Does NOT change syscall ABI
- Does NOT change boundary enforcement
- Does NOT change memory safety guarantees

**PCID only affects**:
- TLB flush behavior (selective vs full)
- CR3 switch micro-cost
- TLB miss rate

## Expected Outcomes

### If PCID Reduces TLB Flush Cost (5-15% improvement)
- entry_latency_ticks: 22.6M → 19.2M-21.5M
- syscall_latency_ms_proxy: 204ms → 173-194ms
- **Action**: Keep PCID enabled, continue testing other segments

### If PCID Has Significant Impact (≥15% improvement)
- entry_latency_ticks: 22.6M → <19.2M
- syscall_latency_ms_proxy: 204ms → <173ms
- **Action**: PCID is major optimization, investigate further

### If PCID Has Minimal Impact (<5% improvement)
- entry_latency_ticks: ~22.6M (minimal change)
- syscall_latency_ms_proxy: ~204ms (minimal change)
- **Action**: Rule out PCID, continue to Patch I-C2 (canonical stub tests)

## Baseline Comparison (Patch G)

**Patch G Metrics** (CI run 24637379135):
- entry_latency_ticks: 22,646,771
- syscall_latency_ms_proxy: 204.07ms
- boot_time_ms: 12,708ms (ignore artifacts)
- preempt_iret_count: 61

## Measurement Contract Verification

**Critical Check**: Verify PCID is NOT overridden by contract

In CI report, check:
```json
"preempt_contract_cr3_pcid": "1",
"preempt_contract_cr3_pcid_source": "env",
"preempt_observed_cr3_pcid": "1"
```

**If PCID=0 in report**: Test is invalid (contract override, like Patch I-B)  
**If PCID=1 in report**: Test is valid (no contract override)

## Next Steps

1. Wait for CI run 24638819404 to complete
2. Download artifacts and analyze metrics
3. **VERIFY**: Check `preempt_contract_cr3_pcid` in report
4. Compare entry_latency_ticks to baseline
5. Document findings in `PATCH_I_C1_RESULTS.md`

**Based on results**:
- If ≥15%: PCID is major optimization
- If 5-15%: PCID is partial contributor, continue to I-C2
- If <5%: Rule out PCID, continue to I-C2

## Subsequent Tests (If Needed)

**Patch I-C2**: Canonical stub + skip pivot
- AYKEN_RING3_CANONICAL_FETCH_STUB=1
- AYKEN_RING3_SKIP_CR3_PIVOT=1

**Patch I-C3**: Canonical stub + self reload
- AYKEN_RING3_CANONICAL_FETCH_STUB=1
- AYKEN_RING3_SELF_RELOAD_CR3=1

**Note**: These are mutually exclusive, test sequentially.

## Risk Assessment

**Architectural Risk**: MINIMAL
- Hardware optimization only
- No isolation change
- No ABI change

**Measurement Risk**: LOW
- PCID not locked by contract
- Clean toggle via Makefile

**Rollback**: TRIVIAL
- Set AYKEN_CR3_PCID=0
- No code changes needed

## CI Run Link

https://github.com/kenanay/AykenOS/actions/runs/24638819404

---

**Authority**: Kenan AY - Architectural Steward  
**Status**: Awaiting CI results  
**Framework**: Architecture-preserving performance forensics

