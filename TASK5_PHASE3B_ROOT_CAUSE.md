# Task 5 Phase 3B: Root Cause Analysis - IRQ0 Preemption

**Date**: 2026-04-15  
**Status**: ROOT CAUSE IDENTIFIED

## Problem Statement

BCIB worker enters Ring3 successfully but userspace code never executes. Worker stuck in infinite Ring3 entry loop with no progress.

## Root Cause: Immediate IRQ0 Preemption

### Evidence from Debugcon Log

**Pattern observed** (repeats indefinitely):
```
P10_RING3_ENTER
rY[IRQ][SCH]
```

**Analysis**:
1. Worker enters Ring3 (`P10_RING3_ENTER`)
2. Immediately after, IRQ0 arrives (`[IRQ]`)
3. Scheduler runs (`[SCH]`)
4. Worker rescheduled to same RIP (0x400000)
5. Loop repeats

### Critical Finding: RFLAGS IF Bit

Log shows:
```
P10_RFLAGS_IF_ON
P10_RING3_FRAME_PROOF ... RF=0000000000000202
```

RFLAGS = 0x202 means:
- Bit 9 (IF) = 1 → Interrupts ENABLED
- Bit 1 (reserved) = 1 (always set)

**This means**: Worker enters Ring3 with interrupts ENABLED, so IRQ0 can fire immediately after `iretq`, before first instruction executes!

### Why No Progress?

1. Kernel executes `iretq` to enter Ring3
2. CPU loads RIP=0x400000, RFLAGS=0x202 (IF=1)
3. BEFORE first instruction at 0x400000 executes, IRQ0 fires (timer tick pending)
4. CPU saves RIP=0x400000 (unchanged) and enters IRQ0 handler
5. Scheduler runs, picks same worker (pid=2)
6. Context switch back to worker with RIP=0x400000
7. Goto step 1 (infinite loop)

**Result**: Worker never executes even ONE instruction in userspace!

## Why Userspace Marker Never Appears

The first instruction in BCIB worker is:
```asm
_start:
    mov $0x700000, %rbx  # First instruction at 0x400000
```

This instruction NEVER executes because IRQ0 preempts before it runs.

The marker emission code:
```asm
mov $0xE9, %dx
mov 

```

Never reached because first instruction never executes.

## Solution Options

### Option 1: Disable Interrupts on Ring3 Entry (RECOMMENDED)

Enter Ring3 with RFLAGS.IF=0, allow userspace to execute initial setup, then enable interrupts with `sti`.

**Implementation**:
```c
// In sched.c, before switch_to_first/context_switch:
current_proc->context.rflags &= ~(1ULL << 9);  // Clear IF bit
```

**Userspace code**:
```asm
_start:
    # Interrupts disabled on entry
    mov $0x700000, %rbx
    # ... emit markers ...
    # ... initial setup ...
    sti  # Enable interrupts after setup
    # ... main loop ...
```

**Pros**:
- Simple, deterministic
- Guarantees initial code executes
- Standard practice for critical initialization

**Cons**:
- Requires userspace to call `sti`
- Delays interrupt handling slightly

### Option 2: Mask IRQ0 on First Ring3 Entry

Use existing `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY` flag to mask IRQ0 before first Ring3 entry.

**Implementation**:
```bash
# Rebuild with flag enabled:
make kernel AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1 ...
```

**Pros**:
- Already implemented in code
- Surgical fix (only affects first entry)

**Cons**:
- Only masks IRQ0, other interrupts still fire
- Requires rebuild with specific flag

### Option 3: Reduce Timer Frequency

Lower IRQ0 frequency to give userspace more time between ticks.

**Implementation**:
```c
// In timer.c:
timer_init(10);  // 10 Hz instead of 100 Hz
```

**Pros**:
- No code changes to Ring3 entry path
- Simple

**Cons**:
- Doesn't solve root cause
- Still possible (though less likely) to preempt immediately
- Affects all timing-sensitive code

## Recommended Solution

**Use Option 1**: Enter Ring3 with interrupts disabled.

**Rationale**:
1. Most deterministic - guarantees initial code executes
2. Standard practice in OS development
3. Gives userspace control over when to enable interrupts
4. No special build flags needed
5. Works for all processes, not just BCIB worker

## Implementation Plan

### Step 1: Modify Ring3 Entry RFLAGS

**File**: `kernel/sched/sched.c`

**Location**: Before `switch_to_first()` and in context switch path

**Code**:
```c
// Clear IF bit in RFLAGS before Ring3 entry
current_proc->context.rflags &= ~(1ULL << 9);
```

### Step 2: Add `sti` to Userspace Entry

**File**: `userspace/minimal/minimal_bcib_worker.S`

**Location**: After initial marker emission

**Code**:
```asm
_start:
    # Interrupts disabled on entry (IF=0)
    mov $0x700000, %rbx
    
    # Emit start marker (safe, no interrupts)
    mov $0xE9, %dx
    mov 

    # ... (marker emission code) ...
    
    # Enable interrupts after initial setup
    sti
    
work_loop:
    # Now interrupts are enabled, normal operation
    # ...
```

### Step 3: Rebuild and Test

```bash
# Rebuild kernel
make kernel KERNEL_PROFILE=validation AYKEN_PHASE16_BCIB_PROOF_TEST=1 USER_MINIMAL_MODE=bcib-worker-bootstrap

# Rebuild EFI image
make efi-img KERNEL_PROFILE=validation AYKEN_PHASE16_BCIB_PROOF_TEST=1 USER_MINIMAL_MODE=bcib-worker-bootstrap

# Run QEMU with correct debugcon port
timeout 30 qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive format=raw,file=EFI.img \
  -m 512M \
  -serial file:out/logs/serial.log \
  -debugcon file:out/logs/debugcon.log \
  -global isa-debugcon.iobase=0xE9 \
  -nographic \
  -no-reboot
```

### Step 4: Verify Success

**Expected markers in debugcon log**:
```
[[SCHED_START]]
P10_RING3_ENTER
[BCIB_WORKER_START]  ← NEW! This should now appear
[BCIB_SUBMIT_OK]     ← NEW! This should appear after syscall
```

## Alternative: Quick Test with IRQ0 Masking

If we want to test immediately without modifying userspace code:

```bash
# Rebuild with IRQ0 masking flag
make kernel \
  KERNEL_PROFILE=validation \
  AYKEN_PHASE16_BCIB_PROOF_TEST=1 \
  USER_MINIMAL_MODE=bcib-worker-bootstrap \
  AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1
```

This will mask IRQ0 before first Ring3 entry, giving userspace time to execute.

## Conclusion

**Root Cause**: BCIB worker enters Ring3 with interrupts enabled (RFLAGS.IF=1), causing immediate IRQ0 preemption before first instruction executes.

**Impact**: Worker stuck in infinite Ring3 entry loop, no userspace progress, no marker emission.

**Solution**: Enter Ring3 with interrupts disabled (RFLAGS.IF=0), allow initial setup to complete, then enable interrupts with `sti`.

**Next Step**: Implement Option 1 (disable interrupts on entry) and verify `[BCIB_WORKER_START]` marker appears.

