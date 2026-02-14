// kernel/sched/sched.h
// Ring0 Scheduler Mechanism Interface - No Policy Code
//
// This header defines the Ring0 scheduler mechanism interface.
// All scheduling policy decisions are made in Ring3 userspace.
// Ring0 provides only the mechanism for context switching and
// basic process state management.
//
// Requirements:
// - Task: No policy code remains in Ring0
// - Ring0 contains only mechanism (context switching, memory management)
// - Ring3 contains all policy (scheduling algorithms, queue management)

#ifndef AYKEN_SCHED_H
#define AYKEN_SCHED_H

#include <stdint.h>
#include "../include/proc.h"

// ============================================================================
// RING0 SCHEDULER MECHANISM API (NO POLICY)
// ============================================================================

// Ring0 mechanism: Initialize scheduler mechanism state
void sched_init(void);

// Ring0 mechanism: Add process to scheduler (calls Ring3 policy)
void sched_add(proc_t *proc);

// Ring0 mechanism: Yield CPU to next process (calls Ring3 policy)
void sched_yield(void);
// Ring0 mechanism: Yield from IRQ context (no IF re-enable)
void sched_yield_irq(void);

// Ring0 mechanism: Deferred preemption request/ack (IRQ-safe)
void sched_request_resched(void);
void sched_request_resched_irq(void);
uint32_t sched_take_resched(void);

// Ring0 mechanism: Start scheduler with first process (calls Ring3 policy)
void sched_start(void);

// Ring0 mechanism: Block current process (calls Ring3 policy)
void sched_block_current(void);

// Ring0 mechanism: Wake blocked process (calls Ring3 policy)
void sched_wake(proc_t *proc);

// Ring0 mechanism: Wake all processes waiting on object (calls Ring3 policy)
void sched_wake_all(void *wait_obj);

// Ring0 mechanism: Select next process to run (calls Ring3 policy)
proc_t* sched_select_next(void);

// Ring0 mechanism: Add process to ready queue (calls Ring3 policy)
void enqueue_ready(proc_t *p);

// Ring0 mechanism: Remove process from ready queue (calls Ring3 policy)
void remove_from_ready_queue(proc_t *p);

// Ring0 mechanism: Add task to scheduler (calls Ring3 policy)
void sched_add_task(void *task);

// Ring0 mechanism state: Current running process
extern proc_t *current_proc;
extern volatile uint32_t sched_irq_user_ctx_saved;

#endif // AYKEN_SCHED_H
