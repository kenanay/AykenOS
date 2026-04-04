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

enum sched_ring3_entry_guard_action {
    SCHED_RING3_ENTRY_GUARD_ARM = 0,
    SCHED_RING3_ENTRY_GUARD_SHOULD_DEFER_IRQ = 1,
};

enum sched_perf_phase_id {
    SCHED_PERF_PHASE_BOOT_START = 0,
    SCHED_PERF_PHASE_CORE_READY,
    SCHED_PERF_PHASE_FIRST_SCHED_ACTIVITY,
    SCHED_PERF_PHASE_FIRST_USER_ENTRY,
    SCHED_PERF_PHASE_FIRST_SYSCALL_GATE_ENTRY,
    SCHED_PERF_PHASE_FIRST_SYSCALL_GATE_RETURN,
    SCHED_PERF_PHASE_FIRST_SYSCALL_ENTRY,
    SCHED_PERF_PHASE_FIRST_SYSCALL_EXIT,
    SCHED_PERF_PHASE_COUNT,
};

enum sched_perf_mb_phase_id {
    SCHED_PERF_MB_PHASE_SNAPSHOT_ENTER = 0,
    SCHED_PERF_MB_PHASE_SNAPSHOT_EXIT,
    SCHED_PERF_MB_PHASE_EXTRACT_ENTER,
    SCHED_PERF_MB_PHASE_EXTRACT_EXIT,
    SCHED_PERF_MB_PHASE_VALIDATE_ENTER,
    SCHED_PERF_MB_PHASE_VALIDATE_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_OWNER_LOOKUP_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_OWNER_LOOKUP_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_LOOKUP_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_LOOKUP_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_SWITCH,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_KEEP_RUNNING,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_REJECT,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_FALLBACK,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_ACCEPT_KEEP_RUNNING,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_ACCEPT_SWITCH,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_REJECT,
    SCHED_PERF_MB_PHASE_ARBITER_KEEP_RUNNING_FALLBACK,
    SCHED_PERF_MB_PHASE_ARBITER_RETURN_NULL,
    SCHED_PERF_MB_PHASE_ARBITER_READY_HEAD_FALLBACK,
    SCHED_PERF_MB_PHASE_HANDOFF_ENTER,
    SCHED_PERF_MB_PHASE_HANDOFF_EXIT,
    SCHED_PERF_MB_PHASE_COUNT,
};

int sched_ring3_entry_guard_control(proc_t *proc, uint32_t action);
void sched_perf_note_phase(enum sched_perf_phase_id id);
void sched_perf_note_mailbox_phase(enum sched_perf_mb_phase_id id);
void sched_perf_note_mailbox_consume(const char *site,
                                     uint64_t old_last_epoch,
                                     uint64_t new_last_epoch,
                                     uint64_t candidate_epoch,
                                     const char *reason);

static inline void sched_arm_ring3_entry_guard_if_ring3(proc_t *proc)
{
    (void)sched_ring3_entry_guard_control(proc, SCHED_RING3_ENTRY_GUARD_ARM);
}

static inline int sched_should_defer_irq_resched_on_ring3_entry(proc_t *proc)
{
    return sched_ring3_entry_guard_control(proc, SCHED_RING3_ENTRY_GUARD_SHOULD_DEFER_IRQ);
}

static inline void sched_perf_note_boot_start(void)
{
    sched_perf_note_phase(SCHED_PERF_PHASE_BOOT_START);
}

static inline void sched_perf_note_core_ready(void)
{
    sched_perf_note_phase(SCHED_PERF_PHASE_CORE_READY);
}

static inline void sched_perf_note_first_syscall_entry(void)
{
    sched_perf_note_phase(SCHED_PERF_PHASE_FIRST_SYSCALL_ENTRY);
}

static inline void sched_perf_note_first_syscall_exit(void)
{
    sched_perf_note_phase(SCHED_PERF_PHASE_FIRST_SYSCALL_EXIT);
}

static inline void sched_perf_note_mailbox_validate_enter(void)
{
    sched_perf_note_mailbox_phase(SCHED_PERF_MB_PHASE_VALIDATE_ENTER);
}

static inline void sched_perf_note_mailbox_validate_exit(void)
{
    sched_perf_note_mailbox_phase(SCHED_PERF_MB_PHASE_VALIDATE_EXIT);
}

#endif // AYKEN_SCHED_H
