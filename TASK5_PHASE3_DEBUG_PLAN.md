# Task 5 Phase 3: Aggressive Debug Plan - Scheduler Activation

**Date**: 2026-04-15  
**Goal**: Understand why scheduler stops at `[K][ABOUT_TO_SCHED]`

## Problem Statement

BCIB worker is created (`pid=2 role=BCIB`) but never executes. Kernel reaches `[K][ABOUT_TO_SCHED]` and stops. No Ring3 entry, no syscall, no execution proof.

## Debug Strategy: Layered Visibility

### Layer 1: Scheduler Loop Entry (CRITICAL)
**Question**: Does scheduler loop even start?

**Markers to add**:
```c
// kernel/sched/sched.c - in sched_run() or main scheduler loop
debugcon_write("[[AYKEN_SCHED_LOOP_ENTRY]]\n");

// After first scheduler decision
debugcon_write("[[AYKEN_SCHED_FIRST_DECISION]]\n");

// In scheduler tick handler
debugcon_write("[[AYKEN_SCHED_TICK]]\n");
```

**Expected outcome**:
- If `[[AYKEN_SCHED_LOOP_ENTRY]]` appears → scheduler starts
- If NOT → scheduler never called (IRQ0 problem or init problem)

### Layer 2: IRQ0 Tick Verification
**Question**: Are timer interrupts reaching scheduler?

**Markers to add**:
```c
// kernel/arch/x86_64/interrupts.c - in IRQ0 handler
static uint64_t irq0_count = 0;
irq0_count++;
debugcon_write("[[AYKEN_IRQ0_TICK]] count=");
debugcon_write_uint(irq0_count);
debugcon_write("\n");
```

**Expected outcome**:
- If ticks appear → timer works, scheduler should be called
- If NO ticks → timer not configured or IRQ masked

### Layer 3: Ready Queue State
**Question**: Is BCIB worker in ready queue?

**Markers to add**:
```c
// kernel/sched/sched.c - after worker creation or in scheduler
debugcon_write("[[AYKEN_READY_QUEUE_DUMP]]\n");
for (each process in ready queue) {
    debugcon_write("  pid=");
    debugcon_write_uint(proc->pid);
    debugcon_write(" state=");
    debugcon_write_uint(proc->state);
    debugcon_write(" role=");
    debugcon_write_uint(proc->execution_role);
    debugcon_write("\n");
}
```

**Expected outcome**:
- BCIB worker (pid=2) should be in ready queue with state=READY
- If NOT in queue → creation didn't add to scheduler
- If state != READY → worker blocked or zombie

### Layer 4: Scheduler Decision Path
**Question**: Why doesn't scheduler pick BCIB worker?

**Markers to add**:
```c
// kernel/sched/sched.c - in sched_pick_next() or equivalent
debugcon_write("[[AYKEN_SCHED_PICK_NEXT]]\n");
debugcon_write("  candidates=");
debugcon_write_uint(num_candidates);
debugcon_write("\n");

if (picked_process) {
    debugcon_write("[[AYKEN_SCHED_PICKED]] pid=");
    debugcon_write_uint(picked_process->pid);
    debugcon_write("\n");
} else {
    debugcon_write("[[AYKEN_SCHED_NO_PICK]]\n");
}
```

**Expected outcome**:
- Should pick BCIB worker (pid=2) eventually
- If picks init (pid=1) only → BCIB worker not in queue or not ready
- If NO_PICK → ready queue empty (critical bug)

### Layer 5: Context Switch Path
**Question**: Does context switch to BCIB worker succeed?

**Markers to add**:
```c
// kernel/sched/sched.c - before context switch
debugcon_write("[[AYKEN_CONTEXT_SWITCH_BEGIN]] from_pid=");
debugcon_write_uint(current->pid);
debugcon_write(" to_pid=");
debugcon_write_uint(next->pid);
debugcon_write("\n");

// After context switch (if reached)
debugcon_write("[[AYKEN_CONTEXT_SWITCH_OK]]\n");
```

**Expected outcome**:
- Should see switch from init (pid=1) to BCIB worker (pid=2)
- If switch fails → CR3/stack/entry point problem

### Layer 6: Ring3 Entry Verification
**Question**: Does BCIB worker reach Ring3?

**Markers to add**:
```c
// kernel/arch/x86_64/ring3_enter.S or equivalent - BEFORE iretq
// (assembly marker)
mov dx, 0xE9
mov al, '['
out dx, al
; ... emit [[AYKEN_RING3_IRETQ_ATTEMPT]] pid=X
```

**In userspace** (already exists):
```asm
; userspace/minimal/minimal_bcib_worker.S
_start:
    ; Emit [BCIB_WORKER_START] marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    ; ... (already implemented)
```

**Expected outcome**:
- `[[AYKEN_RING3_IRETQ_ATTEMPT]]` → kernel tries to enter Ring3
- `[BCIB_WORKER_START]` → userspace code executes
- If iretq attempt but no start → Ring3 entry fails (GPF/PF)

## File-by-File Modification Plan

### 1. kernel/sched/sched.c
**Priority**: CRITICAL

**Markers to add**:
- Scheduler loop entry
- Scheduler tick handler
- Ready queue dump
- Pick next process
- Context switch begin/end

**Functions to instrument**:
- `sched_run()` or main loop
- `sched_tick()` or IRQ0 handler call
- `sched_pick_next()` or equivalent
- `sched_switch_to()` or context switch

### 2. kernel/arch/x86_64/interrupts.c
**Priority**: HIGH

**Markers to add**:
- IRQ0 tick counter
- IRQ0 → scheduler call marker

**Functions to instrument**:
- `irq0_handler()` or timer interrupt handler

### 3. kernel/proc/bcib_worker.c
**Priority**: MEDIUM

**Markers to add**:
- Worker state after creation
- Worker added to ready queue confirmation

**Functions to instrument**:
- `bcib_worker_create()` - add state logging after `proc_create_user_process()`

### 4. kernel/arch/x86_64/ring3_enter.S
**Priority**: MEDIUM

**Markers to add**:
- Ring3 entry attempt (before iretq)
- Include pid in marker

**Assembly to add**:
- Marker emission before iretq instruction

### 5. userspace/minimal/minimal_bcib_worker.S
**Priority**: LOW (already has markers)

**Verify**:
- `[BCIB_WORKER_START]` marker at entry
- `[BCIB_SUBMIT_OK]` marker after syscall

## Execution Plan

### Phase 3A: Scheduler Visibility (Day 1)
1. Add Layer 1 markers (scheduler loop entry)
2. Add Layer 2 markers (IRQ0 ticks)
3. Rebuild kernel
4. Run QEMU with 30s timeout
5. Analyze: Does scheduler start? Do ticks arrive?

### Phase 3B: Ready Queue Analysis (Day 1-2)
1. Add Layer 3 markers (ready queue dump)
2. Add Layer 4 markers (scheduler decision)
3. Rebuild kernel
4. Run QEMU
5. Analyze: Is BCIB worker in queue? Why not picked?

### Phase 3C: Context Switch Validation (Day 2)
1. Add Layer 5 markers (context switch)
2. Add Layer 6 markers (Ring3 entry)
3. Rebuild kernel
4. Run QEMU
5. Analyze: Does switch succeed? Does Ring3 entry work?

### Phase 3D: Execution Proof (Day 2-3)
1. Verify `[BCIB_WORKER_START]` appears
2. Verify `[BCIB_SUBMIT_OK]` appears
3. Verify no `[[AYKEN_BOUNDARY_KILL]]`
4. Verify execution slot allocation
5. Document complete execution pipeline

## Success Criteria

### Minimum (Phase 3A Complete)
- `[[AYKEN_SCHED_LOOP_ENTRY]]` appears
- `[[AYKEN_IRQ0_TICK]]` appears
- Understand why scheduler stops

### Target (Phase 3B Complete)
- BCIB worker in ready queue
- Scheduler picks BCIB worker
- Context switch to BCIB worker

### Full (Phase 3D Complete)
- `[BCIB_WORKER_START]` appears
- `[BCIB_SUBMIT_OK]` appears
- No `[[AYKEN_BOUNDARY_KILL]]`
- Execution pipeline proven end-to-end

## Risk Mitigation

### Risk 1: Scheduler Never Starts
**Symptom**: No `[[AYKEN_SCHED_LOOP_ENTRY]]`

**Possible causes**:
- Scheduler not called from kernel init
- IRQ0 not configured
- Scheduler disabled in validation profile

**Mitigation**:
- Check kernel init calls scheduler
- Verify timer initialization
- Check validation profile flags

### Risk 2: IRQ0 Ticks Don't Arrive
**Symptom**: No `[[AYKEN_IRQ0_TICK]]`

**Possible causes**:
- Timer not initialized
- IRQ0 masked
- QEMU timer not configured

**Mitigation**:
- Check PIC/APIC initialization
- Verify IRQ mask register
- Check QEMU timer configuration

### Risk 3: BCIB Worker Not in Ready Queue
**Symptom**: Ready queue dump doesn't show pid=2

**Possible causes**:
- Worker not added to scheduler after creation
- Worker state is BLOCKED or ZOMBIE
- Scheduler queue corruption

**Mitigation**:
- Check `proc_create_user_process()` adds to ready queue
- Verify worker state is READY after creation
- Add queue integrity checks

### Risk 4: Ring3 Entry Fails
**Symptom**: `[[AYKEN_RING3_IRETQ_ATTEMPT]]` but no `[BCIB_WORKER_START]`

**Possible causes**:
- Invalid entry point
- Stack corruption
- CR3 not set correctly
- GPF/PF on iretq

**Mitigation**:
- Verify ELF entry point is correct
- Check stack setup
- Verify CR3 points to valid page table
- Add exception handler markers

## Next Immediate Steps

1. **Add Layer 1 + Layer 2 markers** (scheduler loop + IRQ0)
2. **Rebuild kernel** with markers
3. **Run QEMU** with 30s timeout
4. **Analyze logs** for scheduler/IRQ0 activity
5. **Report findings** and proceed to next layer

**Estimated time**: 2-4 hours for Phase 3A, 1-2 days for full Phase 3
