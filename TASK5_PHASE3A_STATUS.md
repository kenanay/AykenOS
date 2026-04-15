# Task 5 Phase 3A: Scheduler Debug - Critical Finding

**Date**: 2026-04-15  
**Status**: BLOCKER IDENTIFIED

## Problem Statement

Kernel reaches `[K][ABOUT_TO_SCHED]` but never calls `sched_start()`. Execution stops immediately after printing the marker.

## Evidence

### Serial Log Output (out/logs/bcib_phase3a_v2_serial.log)
```
[[AYKEN_BOOT_OK]]
[K][PAYLOAD_MODE=bcib-worker-bootstrap]
[K][PAYLOAD_SHA=20c65ce36f7bb1c07025f5cc9e92826873d6e4e3ac9cc70fec94ebd840f753cd]
P10_TSS_OK
[MARKER] [MARKER] [MARKER] [MARKER] 000000001C760000[MARKER] [MARKER] 000000001C760000[MARKER] [MARKER] [MARKER] [K][LATE_INIT_RETURN]
[MARKER] [MARKER] [MARKER] [K][BOOT_OK] Phase 4.4 minimal boot reached
[K][ABOUT_TO_SCHED]
```

### Expected vs Actual

**Expected** (from kernel/kernel.c lines 475-482):
```c
dual_channel_write("[K][ABOUT_TO_SCHED]\n");
outb(0xE9, (uint8_t)'B');  // Should appear 3 times
outb(0xE9, (uint8_t)'B');
outb(0xE9, (uint8_t)'B');
outb(0xE9, (uint8_t)'\n');
sched_start();  // Should print [[SCHED_START]]
```

**Actual**:
- `[K][ABOUT_TO_SCHED]` appears
- NO 'BBB' markers
- NO `[[SCHED_START]]` marker
- NO scheduler activity
- NO IRQ0 ticks after boot

## Critical Findings

### Finding 1: Execution Stops After dual_channel_write
The kernel successfully prints `[K][ABOUT_TO_SCHED]` but the very next instruction (`outb(0xE9, 'B')`) never executes. This suggests:
- Crash/exception immediately after the print
- Hang in an infinite loop
- Jump to invalid address

### Finding 2: Timer Works During Boot But Not After
- `[[AYKEN_TICK]]` appears during `[K][LATE]2 TIMER` init
- NO timer ticks after boot completes
- This suggests timer is initialized but IRQ0 is masked or not firing

### Finding 3: BCIB Worker Created Successfully
- `[[AYKEN_BCIB_WORKER_CREATE_OK]] pid=2 role=BCIB` confirmed
- Worker infrastructure is in place
- But scheduler never starts to run it

## Phase 3A Markers Added

### Layer 1: Scheduler Entry
**Location**: `kernel/sched/sched.c:5758` (sched_start function)
```c
// Immediate entry marker using direct outb
outb(0xE9, (uint8_t)'[');
outb(0xE9, (uint8_t)'[');
outb(0xE9, (uint8_t)'S');
outb(0xE9, (uint8_t)'C');
outb(0xE9, (uint8_t)'H');
outb(0xE9, (uint8_t)'E');
outb(0xE9, (uint8_t)'D');
outb(0xE9, (uint8_t)'_');
outb(0xE9, (uint8_t)'S');
outb(0xE9, (uint8_t)'T');
outb(0xE9, (uint8_t)'A');
outb(0xE9, (uint8_t)'R');
outb(0xE9, (uint8_t)'T');
outb(0xE9, (uint8_t)']');
outb(0xE9, (uint8_t)']');
outb(0xE9, (uint8_t)'\n');
```

**Status**: NEVER REACHED

### Layer 2: IRQ0 Tick Marker
**Location**: `kernel/arch/x86_64/timer.c:145` (timer_isr_c function)
```c
static uint8_t irq0_marker_emitted = 0;
if (!irq0_marker_emitted && tick_count >= 1) {
    irq0_marker_emitted = 1;
    timer_debugcon_write("[[AYKEN_IRQ0_TICK]] count=");
    timer_debugcon_hex64(tick_count);
    timer_debugcon_write("\n");
}
```

**Status**: NEVER REACHED (no IRQ0 after boot)

### Layer 1b: Scheduler Tick Marker
**Location**: `kernel/sched/sched.c:5953` (sched_yield_core function)
```c
static uint8_t sched_tick_marker_emitted = 0;
if (!sched_tick_marker_emitted) {
    sched_tick_marker_emitted = 1;
    sched_emit_marker("[[AYKEN_SCHED_TICK]]\n");
}
```

**Status**: NEVER REACHED (scheduler never starts)

## Root Cause Hypothesis

### Hypothesis 1: Crash After dual_channel_write (MOST LIKELY)
The kernel crashes or triple-faults immediately after `dual_channel_write("[K][ABOUT_TO_SCHED]\n")`. Possible causes:
- Stack corruption
- Invalid memory access
- Exception handler not set up correctly
- Page fault in kernel code

### Hypothesis 2: Infinite Loop in dual_channel_write
The `dual_channel_write` function enters an infinite loop. But this is unlikely because:
- The function is simple (just outb in a loop)
- We see the full `[K][ABOUT_TO_SCHED]` string printed
- No evidence of repeated characters

### Hypothesis 3: QEMU Hang
QEMU itself hangs. But this is unlikely because:
- Timeout kills QEMU successfully
- No QEMU error messages
- Previous markers work fine

## Next Steps

### Immediate Action: Add Exception Markers
Add markers in exception handlers to detect crashes:
1. Add marker in GP fault handler (#GP)
2. Add marker in page fault handler (#PF)
3. Add marker in double fault handler (#DF)
4. Add marker in invalid opcode handler (#UD)

### Alternative Approach: Skip dual_channel_write
Replace `dual_channel_write("[K][ABOUT_TO_SCHED]\n")` with direct `outb` calls to isolate the problem.

### Diagnostic: Check Stack
Add marker to print stack pointer before and after `dual_channel_write` to detect stack corruption.

### Diagnostic: Check Interrupts
Add marker to check if interrupts are enabled/disabled at crash point.

## Files Modified

- `kernel/kernel.c`: Added 'BBB' markers after ABOUT_TO_SCHED
- `kernel/sched/sched.c`: Added [[SCHED_START]] entry marker
- `kernel/sched/sched.c`: Added [[AYKEN_SCHED_TICK]] marker
- `kernel/arch/x86_64/timer.c`: Added [[AYKEN_IRQ0_TICK]] marker

## Conclusion

**Phase 3A Status**: INCOMPLETE - Scheduler never starts

**Critical Blocker**: Kernel crashes or hangs immediately after printing `[K][ABOUT_TO_SCHED]`, before calling `sched_start()`.

**Impact**: 
- No scheduler activity
- No IRQ0 ticks after boot
- BCIB worker created but never scheduled
- No Ring3 execution possible

**Recommendation**: Focus on crash detection (exception markers) before adding more scheduler markers. The scheduler is not the problem - the kernel never reaches it.

