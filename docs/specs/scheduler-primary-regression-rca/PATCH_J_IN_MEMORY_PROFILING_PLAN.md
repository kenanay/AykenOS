# Patch J: In-Memory Entry Profiling - Implementation Plan

**Date**: 2026-04-19  
**Authority**: Kenan AY - Architectural Steward  
**Context**: A/B testing exhausted, profiling is only remaining approach

## Objective

Measure Ring3 entry path segments using in-memory profiling with minimal overhead to identify the actual bottleneck causing ~9.5% unexplained regression.

## Why Profiling is Necessary

**A/B testing blocked at all levels**:
- TEXT_PROOF: ruled out (0.97% impact)
- ENTRY_GUARD: locked by measurement contract
- PCID: locked by freeze guard
- CR3 pivot: requires architectural change (SKIP_CR3_PIVOT causes page fault)

**Profiling is the only remaining approach** to isolate CR3/TLB/transition costs.

## Design: Ultra-Lightweight In-Memory Profiling

### Key Principles

1. **Memory buffer** (not debugcon I/O) - avoid Patch H overhead
2. **Bounded sampling** (1024 samples max) - prevent buffer overflow
3. **No locks, no atomics** - single writer assumption (entry path)
4. **Minimal instrumentation** - RDTSC + store only
5. **Deferred dump** - output after test completion, not in hot path

### Infrastructure (Already Exists)

**Files**:
- `kernel/arch/x86_64/entry_profiler.c` - dump function
- `kernel/arch/x86_64/ring3_enter.S` - buffer + record macro

**Data structures**:
```c
struct entry_diag_sample {
    uint32_t phase;
    uint32_t aux;
    uint64_t tsc;
};

extern struct entry_diag_sample entry_diag_buffer[1024];
extern uint32_t entry_diag_index;
extern uint32_t entry_diag_enabled;
```

**Record macro** (in ring3_enter.S):
```asm
.macro ENTRY_DIAG_RECORD phase aux
    // Ultra-lightweight: RDTSC + store
    // ~50 cycles overhead
.endm
```

## Phase Markers (7 Critical Points)

### 1. COMMIT (phase=1)
**Location**: `ring3_enter.S` - before CR3 switch  
**Purpose**: Entry path start timestamp  
**Code**: `ENTRY_DIAG_RECORD 1, 0`

### 2. CR3_SWITCH_ENTER (phase=2)
**Location**: `ring3_enter.S` - before `mov cr3, %rax`  
**Purpose**: CR3 write start  
**Code**: `ENTRY_DIAG_RECORD 2, old_cr3_low`

### 3. CR3_SWITCH_EXIT (phase=3)
**Location**: `ring3_enter.S` - after `mov cr3, %rax`  
**Purpose**: CR3 write end (measure CR3 write + TLB flush cost)  
**Code**: `ENTRY_DIAG_RECORD 3, new_cr3_low`

### 4. RING3_ENTER (phase=4)
**Location**: `ring3_enter.S` - before `iretq`  
**Purpose**: IRETQ start  
**Code**: `ENTRY_DIAG_RECORD 4, rip_low`

### 5. FIRST_FETCH (phase=5)
**Location**: User entry stub OR first syscall handler  
**Purpose**: Ring3 code execution start  
**Code**: `ENTRY_DIAG_RECORD 5, 0` (via syscall)

### 6. FIRST_IRQ (phase=6)
**Location**: Timer IRQ handler  
**Purpose**: First interrupt after Ring3 entry  
**Code**: `ENTRY_DIAG_RECORD 6, irq_num`

### 7. KERNEL_REENTRY (phase=7)
**Location**: Syscall/interrupt return path  
**Purpose**: Return to kernel complete  
**Code**: `ENTRY_DIAG_RECORD 7, 0`

## Expected Output

**CI log will contain**:
```
=== ENTRY_DIAG_DUMP START ===
ENTRY_DIAG_SAMPLE[0] phase=1 tsc=30054057029
ENTRY_DIAG_SAMPLE[1] phase=2 tsc=30054057129
ENTRY_DIAG_SAMPLE[2] phase=3 tsc=30054158234
ENTRY_DIAG_SAMPLE[3] phase=4 tsc=30054159045
ENTRY_DIAG_SAMPLE[4] phase=5 tsc=30076214078
ENTRY_DIAG_SAMPLE[5] phase=6 tsc=30076215123
ENTRY_DIAG_SAMPLE[6] phase=7 tsc=30076216234
=== ENTRY_DIAG_DUMP END ===
```

**Delta calculation**:
- CR3 write cost: phase[3] - phase[2] = ~101k ticks
- IRETQ cost: phase[4] - phase[3] = ~811 ticks
- User fetch delay: phase[5] - phase[4] = ~22M ticks ← **PRIMARY SUSPECT**
- IRQ latency: phase[6] - phase[5] = ~1k ticks
- Return cost: phase[7] - phase[6] = ~1k ticks

## Implementation Files

### 1. Makefile
**Change**: Add `AYKEN_RING3_ENTRY_MEM_PROFILE` flag  
**Default**: 0 (disabled)  
**Test**: 1 (enabled for Patch J)

### 2. kernel/arch/x86_64/ring3_enter.S
**Changes**:
- Add phase markers at 4 locations (COMMIT, CR3_ENTER, CR3_EXIT, RING3_ENTER)
- Conditional compilation via `#if AYKEN_RING3_ENTRY_MEM_PROFILE`

### 3. kernel/arch/x86_64/interrupts.c (timer IRQ)
**Changes**:
- Add FIRST_IRQ marker in timer handler
- One-shot: only record first IRQ after entry

### 4. kernel/sys/syscall.c (syscall handler)
**Changes**:
- Add FIRST_FETCH marker in syscall entry
- Add KERNEL_REENTRY marker in syscall return
- One-shot: only record first syscall

### 5. kernel/kernel.c (dump trigger)
**Changes**:
- Call `entry_diag_dump()` before shutdown
- Conditional compilation via `#if AYKEN_RING3_ENTRY_MEM_PROFILE`

## Critical Safety Requirements

### 1. Buffer Mapping
**CRITICAL**: Buffer must be accessible with both kernel and user CR3

**Solution**: Buffer in kernel higher-half (.data section)
- Address: 0xFFFFFFFF8XXXXXXX (kernel space)
- Mapped in all CR3s (kernel page tables)
- Writable from Ring0

**Failure mode**: If buffer not accessible after CR3 switch → page fault → profiler breaks system

### 2. Register Safety
**CRITICAL**: Don't clobber user state

**Solution**: Use caller-saved registers only
- Save/restore: %rax, %rcx, %rdx, %r8
- Don't touch: %rbx, %rbp, %r12-r15 (callee-saved)

### 3. Overflow Handling
**CRITICAL**: Don't write past buffer end

**Solution**: Check index before write
```asm
cmpl $1024, entry_diag_index(%rip)
jae skip_record
```

## Overhead Analysis

**Per-sample cost**: ~50 cycles
- RDTSC: ~20 cycles
- Store: ~10 cycles
- Index increment: ~10 cycles
- Conditional: ~10 cycles

**Total overhead** (7 samples): ~350 cycles = 0.0015% of 22M ticks

**Comparison**:
- Patch H (debugcon): ~25.8M ticks overhead (96% contamination)
- Patch J (memory): ~350 cycles overhead (0.0015% contamination)

**Overhead is negligible** - will not affect measurement.

## Validation Criteria

**Test integrity**:
- preempt_iret_count == 61 (test completed)
- entry_diag_index > 0 (samples collected)
- All 7 phases present in first sample set
- TSC values monotonically increasing

**Performance metrics**:
- entry_latency_ticks comparable to baseline (22.6M)
- No regression in other gates

## Decision Rules

**After analyzing deltas**:

**If CR3 write cost > 5M ticks (>20%)**:
- CR3 pivot is primary bottleneck
- Consider PCID (requires governance approval)

**If user fetch delay > 15M ticks (>65%)**:
- Post-IRETQ delay is primary bottleneck
- Investigate: TLB miss, page table walk, I-cache miss

**If IRETQ cost > 5M ticks (>20%)**:
- IRETQ instruction itself is bottleneck
- Investigate: pipeline serialization, microcode

**If no single segment > 20%**:
- Distributed cost across multiple segments
- Need deeper profiling or accept current state

## Rollback Plan

**If profiling causes issues**:
1. Set `AYKEN_RING3_ENTRY_MEM_PROFILE=0`
2. Rebuild
3. No code changes required (conditional compilation)

**Risk**: MINIMAL
- Profiling is opt-in (flag=0 by default)
- No functional changes
- Buffer in safe location (kernel higher-half)

## Next Steps

1. Implement phase markers in all 5 files
2. Build with `AYKEN_RING3_ENTRY_MEM_PROFILE=1`
3. Run in CI (via PR)
4. Download artifacts and analyze dump
5. Calculate deltas and identify bottleneck
6. Document findings in `PATCH_J_RESULTS.md`

## Success Criteria

**Patch J succeeds if**:
- Test completes (preempt_iret_count=61)
- Dump contains all 7 phases
- Deltas identify segment with >20% of entry cost
- Root cause definitively isolated

**Patch J fails if**:
- System hangs (buffer mapping issue)
- No samples collected (profiler not triggered)
- All segments <20% (distributed cost, no clear bottleneck)

## Key Insight

**This is the final measurement approach.**

If Patch J doesn't identify the bottleneck:
- Regression is distributed across many small costs
- No single optimization will fix it
- Accept current state (~18% total regression)

**Patch J is make-or-break for this investigation.**

---

**Status**: Ready for implementation  
**Risk**: MINIMAL (opt-in, conditional compilation)  
**Expected**: Definitive identification of bottleneck  
**Authority**: Kenan AY - Architectural Steward

