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
#define AYKEN_SCHED_BOOTSTRAP_POLICY 0
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
void sched_remove_process_everywhere(proc_t *p);
void sched_exit_current(void) __attribute__((noreturn));

// Ring0 mechanism: Add task to scheduler bookkeeping
void sched_add_task(void *task);

// Ring0 mechanism state: Current running process
extern proc_t *current_proc;
extern volatile uint32_t sched_irq_user_ctx_saved;

// Ring0 mechanism: schedule-entry execution delivery hook for current process
int sched_try_pickup_execution_work(void);

uint32_t sched_active_owner_pid(void);
int sched_request_owner_transfer(proc_t *caller_owner, proc_t *successor);

void sched_validation_set_active_owner(proc_t *owner);
int sched_validation_take_owner_transfer_event(int *from_pid, int *to_pid);
int sched_validation_take_mailbox_decision_event(int *from_pid,
                                                 int *to_pid,
                                                 int *src_pid,
                                                 uint64_t *decision_id);
int sched_validation_non_owner_publish_would_fail(proc_t *publisher);
void sched_validation_arm_exit_successor(proc_t *forced_next);
void sched_validation_disarm_exit_successor(void);
int sched_validation_take_exit_switch_event(int *from_pid, int *to_pid);

void sched_perf_note_boot_start(void);
void sched_perf_note_core_ready(void);
void sched_perf_note_first_scheduler_activity(void);
void sched_perf_note_first_user_entry(void);
void sched_perf_note_first_syscall_entry(void);
void sched_perf_note_first_syscall_exit(void);
void sched_perf_note_mailbox_snapshot_enter(void);
void sched_perf_note_mailbox_snapshot_exit(void);
void sched_perf_note_mailbox_extract_enter(void);
void sched_perf_note_mailbox_extract_exit(void);
void sched_perf_note_mailbox_validate_enter(void);
void sched_perf_note_mailbox_validate_exit(void);
void sched_perf_note_mailbox_arbiter_enter(void);
void sched_perf_note_mailbox_arbiter_exit(void);
void sched_perf_note_mailbox_arbiter_owner_lookup_enter(void);
void sched_perf_note_mailbox_arbiter_owner_lookup_exit(void);
void sched_perf_note_mailbox_arbiter_candidate_lookup_enter(void);
void sched_perf_note_mailbox_arbiter_candidate_lookup_exit(void);
void sched_perf_note_mailbox_arbiter_decision_enter(void);
void sched_perf_note_mailbox_arbiter_decision_exit(void);
void sched_perf_note_mailbox_arbiter_decision_path_switch(void);
void sched_perf_note_mailbox_arbiter_decision_path_keep_running(void);
void sched_perf_note_mailbox_arbiter_decision_path_reject(void);
void sched_perf_note_mailbox_arbiter_decision_path_fallback(void);
void sched_perf_note_mailbox_arbiter_path_switch_enter(void);
void sched_perf_note_mailbox_arbiter_path_switch_exit(void);
void sched_perf_note_mailbox_arbiter_path_keep_running_enter(void);
void sched_perf_note_mailbox_arbiter_path_keep_running_exit(void);
void sched_perf_note_mailbox_arbiter_path_reject_enter(void);
void sched_perf_note_mailbox_arbiter_path_reject_exit(void);
void sched_perf_note_mailbox_arbiter_path_fallback_enter(void);
void sched_perf_note_mailbox_arbiter_path_fallback_exit(void);
void sched_perf_note_mailbox_arbiter_candidate_accept_keep_running(void);
void sched_perf_note_mailbox_arbiter_candidate_accept_switch(void);
void sched_perf_note_mailbox_arbiter_candidate_reject(void);
void sched_perf_note_mailbox_arbiter_keep_running_fallback(void);
void sched_perf_note_mailbox_arbiter_return_null(void);
void sched_perf_note_mailbox_arbiter_ready_head_fallback(void);
void sched_perf_note_mailbox_handoff_enter(void);
void sched_perf_note_mailbox_handoff_exit(void);

#endif // AYKEN_SCHED_H
