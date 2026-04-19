# Patch H: Entry Segmentation Profiling Plan

**Date**: 2026-04-19  
**Objective**: Identify which Ring3 entry segment contains the remaining 9.5% regression  
**Status**: READY FOR IMPLEMENTATION

## Context

### What We Know
- **Total regression**: ~18% (syscall_latency: 204ms vs baseline 175ms)
- **Boundary enforcement cost**: ~8.5% (confirmed via Patch F)
- **Diagnostic markers cost**: ~0% (confirmed via Patch G)
- **Remaining bottleneck**: ~9.5% (unidentified)

### Where the Cost Is
- **Entry window**: 22.6M ticks (81% of total latency) ← PRIMARY BOTTLENECK
- **Pure syscall**: 5.3M ticks (19% of total latency)

### Current Build State (Patch G)
```
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0  (enforcement disabled)
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0    (markers disabled)
AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0    (syscall markers disabled)
```

## Hypothesis

The remaining 9.5% regression is in Ring3 transition mechanics:
1. **CR3 pivot** (address space switch, TLB operations)
2. **Entry guard** (`AYKEN_RING3_ENTRY_GUARD=1`)
3. **Post-CR3 text proof** (`AYKEN_RING3_POST_CR3_TEXT_PROBE=1`)
4. **Frame validation** (per-entry validation overhead)
5. **IRET preparation** (stack setup, register restore)

## Experimental Design

### Measurement Strategy
Add surgical tick measurements to Ring3 entry path with bounded sampling to avoid marker spam.

### Entry Segments to Measure

**Note**: This is COARSE-GRAINED profiling (4 segments). Fine-grained profiling (AFTER_GUARD, AFTER_FRAME_VALIDATE) can be added if needed.

```
ENTRY_START
  ↓
CR3_PIVOT (mov %rcx, %cr3)
  ↓
AFTER_CR3
  ↓
POST_CR3_TEXT_PROOF (if enabled)
  ↓
AFTER_TEXT_PROOF
  ↓
[Frame validation + IRET prep combined]
  ↓
BEFORE_IRET
  ↓
ENTRY_END (iretq)
```

### Sampling Strategy
- **Bounded sampling**: Measure only first 3 transitions
- **Reason**: Avoid marker spam (learned from Patch E)
- **Implementation**: Global counter `entry_diag_samples`

### Tick Measurement
Use RDTSC for deterministic cycle counting:
```c
uint64_t t0 = ayken_rdtsc();  // ENTRY_START
// ... segment work ...
uint64_t t1 = ayken_rdtsc();  // AFTER_SEGMENT
uint64_t cost = t1 - t0;
```

### Marker Format
```
DIAG_ENTRY_SEG_START=0x<tsc>
DIAG_ENTRY_SEG_AFTER_CR3=0x<tsc>
DIAG_ENTRY_SEG_AFTER_GUARD=0x<tsc>
DIAG_ENTRY_SEG_AFTER_TEXT_PROOF=0x<tsc>
DIAG_ENTRY_SEG_AFTER_FRAME_VALIDATE=0x<tsc>
DIAG_ENTRY_SEG_BEFORE_IRET=0x<tsc>
DIAG_ENTRY_SEG_END=0x<tsc>
```

## Implementation Plan

### Step 1: Add RDTSC Helper (C)
File: `kernel/include/ayken_rdtsc.h`
```c
static inline uint64_t ayken_rdtsc(void) {
    uint32_t lo, hi;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}
```

### Step 2: Add Sample Counter (ASM)
File: `kernel/arch/x86_64/ring3_enter.S`
```asm
.section .bss
entry_diag_samples:
    .quad 0
```

### Step 3: Add Segmentation Markers (ASM)
File: `kernel/arch/x86_64/ring3_enter.S`

Insert tick measurements at key points in `ring3_enter_post_cr3`:
1. Before CR3 pivot
2. After CR3 pivot
3. After text proof (if applicable)
4. Before IRET

**Register Safety**: Use only caller-saved registers (%rax, %rdx, %r9) with push/pop to avoid clobbering user state.

### Step 4: Add Build Flag
File: `Makefile`
```makefile
AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE ?= 1
```

### Step 5: Conditional Compilation
Wrap profiling code in:
```c
#if defined(AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE) && (AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE == 1)
// profiling code
#endif
```

## Expected Outcomes

### Scenario 1: POST_CR3_TEXT_PROOF Dominant
```
CR3_PIVOT: 500k ticks (2%)
POST_CR3_TEXT_PROOF: 18M ticks (80%) ← DOMINANT
FRAME_VALIDATE: 2M ticks (9%)
IRET_PREP: 2M ticks (9%)
```
**Action**: Optimize or cache text proof

### Scenario 2: ENTRY_GUARD Dominant
```
CR3_PIVOT: 500k ticks (2%)
ENTRY_GUARD: 18M ticks (80%) ← DOMINANT
TEXT_PROOF: 2M ticks (9%)
IRET_PREP: 2M ticks (9%)
```
**Action**: Optimize or cache entry guard validation

### Scenario 3: CR3_PIVOT Dominant
```
CR3_PIVOT: 18M ticks (80%) ← DOMINANT
TEXT_PROOF: 2M ticks (9%)
FRAME_VALIDATE: 2M ticks (9%)
IRET_PREP: 500k ticks (2%)
```
**Action**: Investigate page table/TLB overhead

### Scenario 4: FRAME_VALIDATE Dominant
```
CR3_PIVOT: 500k ticks (2%)
TEXT_PROOF: 2M ticks (9%)
FRAME_VALIDATE: 18M ticks (80%) ← DOMINANT
IRET_PREP: 2M ticks (9%)
```
**Action**: Cache or amortize frame validation

## Success Criteria

Patch H is successful if:
1. ✅ Profiling markers appear in CI artifact
2. ✅ Segment costs sum to ~22.6M ticks (entry window total)
3. ✅ One segment is clearly dominant (>50% of entry window)
4. ✅ IRET count remains stable (61)
5. ✅ Test behavior unchanged (clean measurement)

## Risk Mitigation

### Risk 1: Marker Spam
**Mitigation**: Bounded sampling (3 samples max)

### Risk 2: RDTSC Overhead
**Mitigation**: RDTSC is ~20-30 cycles, negligible vs 22M tick window

### Risk 3: Register Clobbering
**Mitigation**: Use scratch registers only, preserve user state

### Risk 4: Measurement Artifact
**Mitigation**: Compare total measured ticks to entry_latency_ticks

## Next Steps After Patch H

Once dominant segment is identified:

### If POST_CR3_TEXT_PROOF:
- Patch I: One-shot or cached text proof
- Expected gain: ~9.5%

### If ENTRY_GUARD:
- Patch I: Cached validation or fast-path
- Expected gain: ~9.5%

### If CR3_PIVOT:
- Investigate: Page table structure, TLB policy, mapping strategy
- May require architectural change

### If FRAME_VALIDATE:
- Patch I: Bounded or cached validation
- Expected gain: ~9.5%

## Constitutional Compliance

This is a MEASUREMENT patch only:
- ✅ No semantic changes
- ✅ No policy changes
- ✅ No security boundary changes
- ✅ Fail-closed semantics preserved
- ✅ Determinism preserved (RDTSC is deterministic)

## Artifact Locations

**Branch**: `test/entry-segmentation-profile`  
**Commit**: TBD  
**CI Run**: TBD  
**Artifacts**: `gates/performance/boot-audit/qemu_debugcon.log`

---

**Status**: READY FOR IMPLEMENTATION  
**Next**: Implement profiling markers in `ring3_enter.S`  
**Authority**: Kenan AY - Architectural Steward
