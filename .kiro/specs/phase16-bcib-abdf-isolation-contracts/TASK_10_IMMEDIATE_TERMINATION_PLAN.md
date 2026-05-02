# Task 10: Immediate Termination Implementation Plan

## Problem Statement

Current enforcement timing is NOT immediate:
- Window: 4636 lines between SYSCALL_ENTER and BOUNDARY_KILL
- Root cause: Teardown happens BEFORE scheduler removal
- Risk: Side-effect leakage, race conditions, exploit window

## Current Flow (WRONG)

```
violation_detect()
  → emit_BOUNDARY_KILL()
  → abort_execution_slots()      // SLOW
  → proc_teardown_exit_surfaces() // SLOW
  → sched_remove_process()        // SLOW
  → sched_yield()
  → (scheduler eventually switches)
```

**Problem:** Process continues running during teardown!

## Required Flow (CORRECT)

```
violation_detect()
  → emit_BOUNDARY_KILL()          // marker FIRST
  → current->state = TERMINAL     // immediate
  → remove_from_runqueue()        // scheduler skip
  → force_context_switch()        // NEVER RETURN
  → (async teardown in reaper)    // background
```

**Guarantee:** Process NEVER runs again after BOUNDARY_KILL

## Implementation Strategy

### Phase 1: Immediate State Change

**File:** `kernel/sys/boundary_enforcement.c`
**Function:** `boundary_fail_closed_termination()`

**Changes:**

1. Move BOUNDARY_KILL marker to TOP (before any teardown)
2. Set `current_proc->state = PROC_TERMINAL` immediately
3. Remove from runqueue BEFORE teardown
4. Force context switch with `cli` + `sched_yield()`

### Phase 2: Async Teardown

**New concept:** TERMINAL state

```c
typedef enum {
    PROC_READY,
    PROC_RUNNING,
    PROC_BLOCKED,
    PROC_ZOMBIE,
    PROC_TERMINAL,  // NEW: Immediate kill, no reschedule
} proc_state_t;
```

**Scheduler change:**

```c
// In scheduler: NEVER schedule TERMINAL processes
if (proc->state == PROC_TERMINAL) {
    continue;  // skip forever
}
```

### Phase 3: Reaper Thread

**New:** Background reaper for TERMINAL processes

```c
void reaper_thread() {
    while (1) {
        for_each_terminal_process(proc) {
            // Async teardown
            abort_execution_slots(proc);
            teardown_surfaces(proc);
            free_resources(proc);
            proc->state = PROC_ZOMBIE;
        }
        sleep(100ms);
    }
}
```

## Critical Path Changes

### 1. boundary_enforcement.c

```c
void boundary_fail_closed_termination(...) {
    // STEP 1: Emit marker FIRST
    emit_boundary_kill_marker();
    emit_error_code_marker();
    
    // STEP 2: Immediate terminal state
    current_proc->state = PROC_TERMINAL;
    
    // STEP 3: Remove from runqueue
    cli();  // disable interrupts
    sched_remove_from_runqueue(current_proc);
    
    // STEP 4: Force context switch - NEVER RETURN
    sched_yield();
    
    // UNREACHABLE
    __builtin_unreachable();
}
```

### 2. scheduler.c

```c
proc_t *sched_next() {
    for_each_process(proc) {
        // CRITICAL: Skip TERMINAL processes
        if (proc->state == PROC_TERMINAL) {
            continue;
        }
        
        if (proc->state == PROC_READY) {
            return proc;
        }
    }
    return idle_proc;
}
```

### 3. New: reaper.c

```c
void reaper_init() {
    // Create reaper kernel thread
    proc_t *reaper = proc_create_kernel("reaper", reaper_thread);
    sched_add_process(reaper);
}

void reaper_thread() {
    while (1) {
        reaper_cleanup_terminal_processes();
        sched_sleep(100);  // 100ms interval
    }
}

void reaper_cleanup_terminal_processes() {
    for_each_process(proc) {
        if (proc->state != PROC_TERMINAL) continue;
        
        // Async teardown
        if (proc->active_execution_id) {
            abort_execution_slot(proc->active_execution_id);
        }
        
        proc_teardown_exit_surfaces(proc);
        
        // Mark as zombie (ready for final cleanup)
        proc->state = PROC_ZOMBIE;
    }
}
```

## Success Criteria

### Timing Goals

- Window: < 100 lines (ideal: < 20)
- Deterministic: Same window every run
- Single kill: Exactly one BOUNDARY_KILL
- No continuation: Zero post-kill markers

### Validation

```bash
# Run forbidden path test
make USER_MINIMAL_MODE=runtime-bridge-forbidden efi-img
bash scripts/qemu-runtime-bridge-proof-harness.sh forbidden

# Validate timing
python3 scripts/validate_fail_closed_markers.py \
    evidence/runtime-bridge-proof/qemu_kernel_trace_forbidden.log

# Expected:
# - Window: < 100 lines
# - PASS without warnings
```

## Risk Analysis

### Race Conditions

**Risk:** Reaper accesses process during context switch
**Mitigation:** Use process locks, atomic state transitions

### Resource Leaks

**Risk:** Async teardown never completes
**Mitigation:** Reaper timeout, force cleanup after 1 second

### Scheduler Starvation

**Risk:** Too many TERMINAL processes block scheduler
**Mitigation:** Reaper runs frequently (100ms), bounded TERMINAL list

## Implementation Order

1. Add PROC_TERMINAL state to proc.h
2. Update scheduler to skip TERMINAL processes
3. Refactor boundary_fail_closed_termination() for immediate kill
4. Implement reaper thread
5. Test with forbidden path payload
6. Validate window < 100 lines
7. Update validator to enforce stricter window

## Files to Modify

- `kernel/include/proc.h` - Add PROC_TERMINAL state
- `kernel/sys/boundary_enforcement.c` - Immediate termination
- `kernel/sched/scheduler.c` - Skip TERMINAL processes
- `kernel/sys/reaper.c` - NEW: Async cleanup thread
- `scripts/validate_fail_closed_markers.py` - Stricter window (100 lines)

## Constitutional Compliance

This implements:
- `KERNEL.SAFETY.CRITICAL` - Immediate termination
- `SECURITY.BOUNDARY.VIOLATION` - No exploit window
- `DETERMINISM.GLOBAL` - Deterministic timing

## Next Steps

After Task 10 completion:
- Task 5 production work (DevFS/ABDF integration)
- Task 6 sandbox enforcement
- Task 11 CI gate integration
