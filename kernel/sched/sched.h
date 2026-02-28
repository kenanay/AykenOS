// kernel/sched/sched.h
// Ring0 Scheduler Mechanism Interface (mailbox-first)
//
// Ring0 provides execution mechanics (context switch, CR3/TSS updates).
// Ring3 publishes scheduling decisions through scheduler mailbox ABI.
//
// Requirements:
// - Mailbox decision path is authoritative
// - Legacy fallback is compile-time gated (AYKEN_SCHED_FALLBACK)

#ifndef AYKEN_SCHED_H
#define AYKEN_SCHED_H

#include <stdint.h>
#include "../include/proc.h"

#ifndef AYKEN_SCHED_FALLBACK
#define AYKEN_SCHED_FALLBACK 0
#endif
#ifndef AYKEN_SCHED_BOOTSTRAP_POLICY
#define AYKEN_SCHED_BOOTSTRAP_POLICY 1
#endif
#ifndef AYKEN_SCHED_OWNER_PID
#define AYKEN_SCHED_OWNER_PID 2u
#endif

// ============================================================================
// RING0 SCHEDULER MECHANISM API (NO POLICY)
// ============================================================================

// Ring0 mechanism: Initialize scheduler mechanism state
void sched_init(void);

// Ring0 mechanism: Add process to scheduler bookkeeping
void sched_add(proc_t *proc);

// Ring0 mechanism: Yield CPU via mailbox decision
void sched_yield(void);
// Ring0 mechanism: Yield from IRQ context (no IF re-enable)
void sched_yield_irq(void);

// Ring0 mechanism: Deferred preemption request/ack (IRQ-safe)
void sched_request_resched(void);
void sched_request_resched_irq(void);
uint32_t sched_take_resched(void);

// Ring0 mechanism: Start scheduler with first mailbox-backed process
void sched_start(void);

// Ring0 mechanism: Block current process and switch via mailbox decision
void sched_block_current(void);

// Ring0 mechanism: Wake blocked process
void sched_wake(proc_t *proc);

// Ring0 mechanism: Wake all processes waiting on object
void sched_wake_all(void *wait_obj);

// Ring0 mechanism: Ready queue bookkeeping
void enqueue_ready(proc_t *p);

// Ring0 mechanism: Ready queue bookkeeping
void remove_from_ready_queue(proc_t *p);

// Ring0 mechanism: Add task to scheduler bookkeeping
void sched_add_task(void *task);

// Ring0 mechanism state: Current running process
extern proc_t *current_proc;
extern volatile uint32_t sched_irq_user_ctx_saved;

#endif // AYKEN_SCHED_H
