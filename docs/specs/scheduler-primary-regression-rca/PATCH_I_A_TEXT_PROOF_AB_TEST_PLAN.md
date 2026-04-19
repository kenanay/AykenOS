# Patch I-A: TEXT_PROOF A/B Test Plan

**Date**: 2026-04-19  
**Objective**: Measure POST_CR3_TEXT_PROBE impact on entry window  
**Status**: READY FOR CI

## Context from Patch H

Patch H revealed (low confidence):
- No single dominant segment in sampled window
- TEXT_PROOF: ~31% of measured window
- But sample size too small (3% of total)
- Need targeted A/B to establish causality

## Experimental Design

### Run A: Patch G Baseline (TEXT_PROOF ON)
```
AYKEN_RING3_POST_CR3_TEXT_PROBE=1
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0
AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE=0
```
- CI Run: 24637379135
- entry_latency_ticks: 22,646,771
- syscall_latency_ms_proxy: 204.07ms
- preempt_iret_count: 61

### Run B: Patch I-A (TEXT_PROOF OFF)
```
AYKEN_RING3_POST_CR3_TEXT_PROBE=0  ← CHANGED
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0
AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE=0
```
- CI Run: TBD
- entry_latency_ticks: ?
- syscall_latency_ms_proxy: ?
- preempt_iret_count: ? (expect 61)

### Control Variables (MUST NOT CHANGE)
- All other Ring3 flags unchanged
- Boundary enforcement OFF (same as Patch G)
- Diagnostic markers OFF (same as Patch G)
- Profiling OFF (no overhead)

## What TEXT_PROOF Does

`EMIT_POST_CR3_TEXT_PROBE` in `ring3_enter.S`:
```asm
#if defined(AYKEN_RING3_POST_CR3_TEXT_PROBE) && (AYKEN_RING3_POST_CR3_TEXT_PROBE == 1)
    EMIT_CSTR p10_post_cr3_text_probe
    mov %cr3, %r10
    EMIT_HEX64 %r10
    EMIT_CSTR p10_post_cr3_text_probe_rip
    EMIT_HEX64 %r11
    EMIT_CSTR p10_post_cr3_text_probe_qword
    mov (%r11), %r10
    EMIT_HEX64 %r10
    EMIT_CSTR p10_newline
#endif
```

**Cost**: Debugcon I/O + memory read + register operations

## Expected Outcomes

### Scenario A: TEXT_PROOF Dominant (>30% impact)
```
entry_latency_ticks: 22.6M → ~15M (-30%+)
syscall_latency_ms_proxy: 204ms → ~180ms
```
**Interpretation**: TEXT_PROOF is major bottleneck  
**Action**: Patch I = Optimize/cache/remove TEXT_PROOF  
**Confidence**: HIGH - single variable causality

### Scenario B: TEXT_PROOF Moderate (10-30% impact)
```
entry_latency_ticks: 22.6M → 18-20M (-10-20%)
syscall_latency_ms_proxy: 204ms → 185-195ms
```
**Interpretation**: TEXT_PROOF contributes but not sole cause  
**Action**: Test ENTRY_GUARD next, then combine optimizations  
**Confidence**: MEDIUM - partial causality

### Scenario C: TEXT_PROOF Minor (<10% impact)
```
entry_latency_ticks: 22.6M → 21-22M (-5-10%)
syscall_latency_ms_proxy: 204ms → 195-200ms
```
**Interpretation**: TEXT_PROOF not primary bottleneck  
**Action**: Test ENTRY_GUARD or other segments  
**Confidence**: LOW - need more tests

### Scenario D: TEXT_PROOF Zero Impact
```
entry_latency_ticks: 22.6M → 22.6M (unchanged)
syscall_latency_ms_proxy: 204ms → 204ms (unchanged)
```
**Interpretation**: TEXT_PROOF not in hot path  
**Action**: Focus on other segments (ENTRY_GUARD, CR3, IRET_PREP)  
**Confidence**: HIGH - ruled out

## Success Criteria

Test is successful if:
1. ✅ preempt_iret_count = 61 (test completes normally)
2. ✅ entry_latency_ticks is numeric and measurable
3. ✅ syscall_latency_ms_proxy is numeric
4. ✅ No test artifacts or anomalies

## Decision Matrix

| Impact | entry_latency Δ | Action |
|--------|----------------|--------|
| **Dominant** | -30%+ | Patch I = TEXT_PROOF optimize |
| **Moderate** | -10-30% | Test ENTRY_GUARD, then combine |
| **Minor** | -5-10% | Test other segments |
| **Zero** | <5% | Rule out TEXT_PROOF |

## Next Steps After Patch I-A

### If TEXT_PROOF Dominant
- Design Patch I to optimize TEXT_PROOF
- Options: Cache, one-shot, remove, or reduce scope
- Expected gain: ~9.5% (sufficient for constitutional threshold)

### If TEXT_PROOF Not Dominant
- Patch I-B: Test ENTRY_GUARD (AYKEN_RING3_ENTRY_GUARD=0)
- Patch I-C: Test other segments
- Combine findings for holistic optimization

## Risk Assessment

**Low Risk**:
- TEXT_PROOF is diagnostic only (not functional)
- Disabling it doesn't affect correctness
- Can be safely disabled for performance measurement
- No semantic changes

**Validation**:
- Constitutional gate must pass
- Determinism must be preserved
- IRET count must be 61

## Build Verification

```bash
make kernel.elf  # ✅ PASS
grep "AYKEN_RING3_POST_CR3_TEXT_PROBE=0" # ✅ Confirmed
```

---

**Status**: READY FOR CI  
**Next**: Commit, push, analyze results  
**Authority**: Kenan AY - Architectural Steward
