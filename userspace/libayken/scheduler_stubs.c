/**
 * @file scheduler_stubs.c
 * @brief Ring3 Scheduler Runtime + Mailbox Bridge (Phase 2.6)
 *
 * Ring0 no longer calls Ring3 policy functions directly. Ring3 stages the
 * selected next runnable process through the scheduler mailbox syscall bridge.
 *
 * Legacy userspace_scheduler_* stubs are kept behind a compile-time switch
 * for controlled migration only.
 */

#include "scheduler.h"
#include "../../kernel/include/proc.h"
#include "../../kernel/include/sched_mailbox_abi.h"
#include <stddef.h>
#include <stdint.h>

#ifndef AYKEN_LEGACY_RING3_SCHED_STUBS
#define AYKEN_LEGACY_RING3_SCHED_STUBS 0
#endif

// Ring3 scheduler policy state
static scheduler_policy_t *current_policy = NULL;
static scheduler_config_t current_config = {0};

static inline uint64_t ring0_syscall(uint64_t syscall_num,
                                     uint64_t arg1,
                                     uint64_t arg2,
                                     uint64_t arg3,
                                     uint64_t arg4)
{
#if defined(__x86_64__)
    uint64_t ret;
    __asm__ volatile(
        "movq %5, %%r10\n\t"
        "int $0x80"
        : "=a"(ret)
        : "a"(syscall_num), "D"(arg1), "S"(arg2), "d"(arg3), "r"(arg4)
        : "rcx", "r10", "r11", "memory");
    return ret;
#else
    (void)syscall_num;
    (void)arg1;
    (void)arg2;
    (void)arg3;
    (void)arg4;
    return 0;
#endif
}

#if AYKEN_LEGACY_RING3_SCHED_STUBS
/**
 * @brief Legacy Ring3 scheduler policy stub - select next process
 *
 * Deprecated: strict-mode path uses scheduler_stage_next() mailbox bridge.
 */
proc_t* userspace_scheduler_select_next(proc_t *ready_queue)
{
    // Basic round-robin policy implementation in Ring3
    if (!ready_queue) {
        return NULL;
    }
    
    // If we have a registered policy, use it
    if (current_policy && current_policy->select_next) {
        return current_policy->select_next(ready_queue);
    }
    
    // Default policy: simple round-robin (select first in queue)
    return ready_queue;
}

/**
 * @brief Legacy Ring3 scheduler policy stub - enqueue ready process
 *
 * Deprecated: strict-mode path uses scheduler_stage_next() mailbox bridge.
 */
void userspace_scheduler_enqueue_ready(proc_t *proc)
{
    if (!proc) {
        return;
    }
    
    // If we have a registered policy, use it
    if (current_policy && current_policy->enqueue_ready) {
        current_policy->enqueue_ready(proc);
        return;
    }
    
    // Default policy: simple FIFO enqueueing
    // Note: The actual queue management is handled by Ring0 mechanism
    // This is just for policy-specific decisions
}

/**
 * @brief Legacy Ring3 scheduler policy stub - handle process blocking
 *
 * Deprecated: strict-mode path uses scheduler_stage_next() mailbox bridge.
 */
void userspace_scheduler_handle_block(proc_t *proc, void *wait_obj)
{
    if (!proc) {
        return;
    }
    
    // If we have a registered policy, use it
    if (current_policy && current_policy->handle_block) {
        current_policy->handle_block(proc, wait_obj);
        return;
    }
    
    // Default policy: simple blocking (no special handling)
    // Note: The actual blocking mechanism is handled by Ring0
    // This is just for policy-specific decisions
}
#endif /* AYKEN_LEGACY_RING3_SCHED_STUBS */

/**
 * @brief Register a scheduler policy
 * 
 * Registers a scheduler policy with the Ring3 runtime. The policy will
 * be used for all subsequent scheduling decisions until replaced.
 * 
 * @param policy Pointer to the scheduler policy structure
 * @param config Pointer to the scheduler configuration
 * @return 0 on success, negative error code on failure
 */
int scheduler_register_policy(const scheduler_policy_t *policy, 
                             const scheduler_config_t *config)
{
    if (!policy) {
        return SCHED_ERROR_INVALID_POLICY;
    }
    
    // Validate policy structure
    if (scheduler_validate_policy(policy) != 0) {
        return SCHED_ERROR_INVALID_POLICY;
    }
    
    // Initialize policy if needed
    if (policy->init && policy->init() != 0) {
        return SCHED_ERROR_INIT_FAILED;
    }
    
    // Register the policy
    current_policy = (scheduler_policy_t*)policy;
    if (config) {
        current_config = *config;
    }
    
    return 0;
}

/**
 * @brief Unregister the current scheduler policy
 * 
 * Unregisters the current scheduler policy and reverts to the default
 * policy if available.
 * 
 * @return 0 on success, negative error code on failure
 */
int scheduler_unregister_policy(void)
{
    if (!current_policy) {
        return SCHED_ERROR_NOT_REGISTERED;
    }
    
    // Cleanup policy if needed
    if (current_policy->cleanup) {
        current_policy->cleanup();
    }
    
    // Unregister the policy
    current_policy = NULL;
    current_config = (scheduler_config_t){0};
    
    return 0;
}

/**
 * @brief Get the current scheduler policy
 * 
 * Returns a pointer to the currently registered scheduler policy.
 * 
 * @return Pointer to current policy, or NULL if none registered
 */
const scheduler_policy_t* scheduler_get_current_policy(void)
{
    return current_policy;
}

/**
 * @brief Request a scheduling decision
 * 
 * Requests the current policy to make a scheduling decision. This function
 * interfaces with the Ring0 mechanism to perform the actual context switch.
 * 
 * @return 0 on success, negative error code on failure
 */
int scheduler_request_schedule(void)
{
    // This would interface with Ring0 via syscalls in a full implementation
    // For now, this is a stub that would trigger Ring0 scheduling
    return 0;
}

int scheduler_stage_next(proc_t *proc)
{
    if (!proc) {
        return SCHED_ERROR_INVALID_PROC;
    }

    uint64_t rc = ring0_syscall(SYS_V2_SCHED_STAGE_NEXT, (uint64_t)proc, 0, 0, 0);
    if ((int64_t)rc < 0) {
        return SCHED_ERROR_SYSCALL_FAILED;
    }
    return 0;
}

/**
 * @brief Notify policy of process state change
 * 
 * Notifies the scheduler policy of a process state change (ready, blocked, etc.).
 * The policy can update its internal state accordingly.
 * 
 * @param proc Pointer to the process that changed state
 * @param old_state Previous process state
 * @param new_state New process state
 * @return 0 on success, negative error code on failure
 */
int scheduler_notify_state_change(proc_t *proc, int old_state, int new_state)
{
    if (!proc) {
        return SCHED_ERROR_INVALID_PROC;
    }
    
    // Policy can track state changes for statistics or optimization
    // This is a stub implementation
    return 0;
}

/**
 * @brief Scheduler Policy Validation
 * 
 * Validates that a scheduler policy structure is properly formed and
 * contains all required function pointers.
 * 
 * @param policy Pointer to the policy to validate
 * @return 0 if valid, negative error code if invalid
 */
int scheduler_validate_policy(const scheduler_policy_t *policy)
{
    if (!policy) {
        return SCHED_ERROR_INVALID_POLICY;
    }
    
    // Check required function pointers
    if (!policy->select_next) {
        return SCHED_ERROR_INVALID_POLICY;
    }
    
    // enqueue_ready and handle_block are optional but recommended
    
    return 0;
}

/**
 * @brief Default Round-Robin Policy Implementation
 */
static proc_t* default_select_next(proc_t *ready_queue)
{
    // Simple round-robin: select first process in queue
    return ready_queue;
}

static void default_enqueue_ready(proc_t *proc)
{
    // Default: no special enqueueing logic
    (void)proc;
}

static void default_handle_block(proc_t *proc, void *wait_obj)
{
    // Default: no special blocking logic
    (void)proc;
    (void)wait_obj;
}

const scheduler_policy_t scheduler_default_round_robin = {
    .select_next = default_select_next,
    .enqueue_ready = default_enqueue_ready,
    .handle_block = default_handle_block,
    .init = NULL,
    .cleanup = NULL,
    .get_stats = NULL,
    .name = "Default Round-Robin",
    .version = "1.0",
    .description = "Simple round-robin scheduler policy"
};
