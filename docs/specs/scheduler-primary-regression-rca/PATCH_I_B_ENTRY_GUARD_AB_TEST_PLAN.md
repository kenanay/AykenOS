# Patch I-B: ENTRY_GUARD A/B Test Plan

**Date**: 2026-04-19  
**Authority**: Kenan AY - Architectural Steward  
**Context**: Patch I-A ruled out TEXT_PROOF (0.97% impact), testing next suspect

## Objective

Isolate the causal impact of AYKEN_RING3_ENTRY_GUARD on Ring3 entry window performance via controlled A/B test.

## Hypothesis

ENTRY_GUARD may be the actual bottleneck in the Ring3 entry path. If dominant, disabling it should show >10% improvement in entry_latency_ticks.

## Change

**Single Variable**:
- `AYKEN_RING3_ENTRY_GUARD=0` (was 1)

**Control**:
- All other flags identical to Patch G baseline
- Profiling remains OFF (AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE=0)
- TEXT_PROOF remains OFF (AYKEN_RING3_POST_CR3_TEXT_PROBE=0)

## What ENTRY_GUARD Does

ENTRY_GUARD is a diagnostic/validation mechanism in the Ring3 entry path. When enabled (=1), it performs additional checks or validations during the Ring0→Ring3 transition.

**Expected cost**: Unknown (to be measured)

## Validation Criteria

**Test Integrity**:
- preempt_iret_count == 61 (test completed normally)
- No profiling overhead
- Clean measurement

**Performance Metrics** (vs Patch G baseline):
- entry_latency_ticks: 22,646,771 (baseline)
- syscall_latency_ms_proxy: 204.07ms (baseline)
- boot_time_ms: 12,708ms (baseline, ignore artifacts)

## Decision Rules

**If entry_latency_ticks improvement**:
- **≥30%**: ENTRY_GUARD is primary bottleneck → optimize ENTRY_GUARD
- **10-30%**: ENTRY_GUARD is partial contributor → continue testing other segments
- **<10%**: ENTRY_GUARD is NOT primary bottleneck → rule out, test next segment

## Expected Outcome

If ENTRY_GUARD is the dominant bottleneck, we should see:
- entry_latency_ticks: 22.6M → <15.8M (>30% reduction)
- syscall_latency_ms_proxy: 204ms → <143ms (>30% reduction)

If ENTRY_GUARD is not dominant:
- entry_latency_ticks: minimal change (<10%)
- Continue to Patch I-C (CR3 pivot test)

## Next Steps

1. Modify `Makefile`: Set `AYKEN_RING3_ENTRY_GUARD ?= 0`
2. Commit with evidence-based message
3. Push and analyze CI results
4. Compare entry_latency_ticks to Patch G baseline (22.6M)
5. Document findings in `PATCH_I_B_RESULTS.md`

## References

- Patch G baseline: CI run 24637379135
- Patch H profiling: Inconclusive due to overhead
- Patch I-A: TEXT_PROOF ruled out (0.97% impact)

---

**Status**: Ready for implementation  
**Next**: Modify Makefile and commit

