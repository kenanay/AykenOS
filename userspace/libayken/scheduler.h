/**
 * @file scheduler.h
 * @brief Ring3 Scheduler Policy Interface for AykenOS
 * 
 * This header defines the Ring3 scheduler policy interface as part of the
 * Phase 2.2 architectural transformation. The scheduler policy operates
 * entirely in Ring3 userspace, while Ring0 provides only the context
 * switch mechanism.
 * 
 * Requirements:
 * - FR-3.2.1: Scheduling policy must execute entirely in Ring3
 * - FR-3.2.2: Ring0 must provide only context switch mechanism
 * - FR-3.2.3: Process selection algorithms must be implemented in Ring3
 * - FR-3.2.4: Scheduler policy must be replaceable without kernel changes
 * 
 * @author Kenan AY
 * @date January 3, 2026
 * @version 1.0
 */

#ifndef AYKEN_RING3_SCHEDULER_H
#define AYKEN_RING3_SCHEDULER_H

#include <stdint.h>
#include <stddef.h>

// Forward declaration - will be defined based on Ring3 process representation
typedef struct proc proc_t;

/**
 * @brief Ring3 Scheduler Policy Interface
 * 
 * This structure defines the function pointers for scheduler policy operations
 * that will be implemented in Ring3 userspace. The policy is separated from
 * the mechanism (context switching) which remains in Ring0.
 * 
 * Design Principles:
 * - Policy in Ring3: All scheduling decisions made in userspace
 * - Mechanism in Ring0: Only context switch operations in kernel
 * - Pluggable: Different policies can be loaded without kernel changes
 * - Secure: Policy cannot directly access Ring0 resources
 */
typedef struct scheduler_policy {
    /**
     * @brief Select the next process to run
     * 
     * This function implements the core scheduling algorithm. It examines
     * the ready queue and selects which process should run next based on
     * the policy's algorithm (round-robin, priority-based, etc.).
     * 
     * @param ready_queue Pointer to the head of the ready process queue
     * @return Pointer to the selected process, or NULL if no process is ready
     * 
     * Requirements:
     * - Must not modify Ring0 state directly
     * - Must implement policy logic only (no mechanism)
     * - Must handle empty queue gracefully
     * - Must be deterministic for same input state
     */
    proc_t* (*select_next)(proc_t *ready_queue);
    
    /**
     * @brief Add a process to the ready queue
     * 
     * This function handles enqueueing a process that becomes ready to run.
     * The policy determines where in the queue the process should be placed
     * based on priority, fairness, or other scheduling criteria.
     * 
     * @param proc Pointer to the process to enqueue
     * 
     * Requirements:
     * - Must maintain queue integrity
     * - Must implement policy-specific ordering
     * - Must handle NULL process gracefully
     * - Must not directly invoke Ring0 operations
     */
    void (*enqueue_ready)(proc_t *proc);
    
    /**
     * @brief Handle process blocking
     * 
     * This function is called when a process blocks on a resource (I/O,
     * synchronization object, etc.). The policy determines how to handle
     * the blocked process and may update scheduling state accordingly.
     * 
     * @param proc Pointer to the process that is blocking
     * @param wait_obj Pointer to the object the process is waiting on
     * 
     * Requirements:
     * - Must remove process from ready queue if present
     * - Must track blocking reason for debugging/monitoring
     * - Must handle wait_obj lifetime correctly
     * - Must coordinate with Ring0 mechanism for actual blocking
     */
    void (*handle_block)(proc_t *proc, void *wait_obj);
    
    /**
     * @brief Policy-specific initialization
     * 
     * Optional function called when the scheduler policy is loaded.
     * Can be NULL if no initialization is required.
     * 
     * @return 0 on success, negative error code on failure
     */
    int (*init)(void);
    
    /**
     * @brief Policy-specific cleanup
     * 
     * Optional function called when the scheduler policy is unloaded.
     * Can be NULL if no cleanup is required.
     */
    void (*cleanup)(void);
    
    /**
     * @brief Get policy statistics
     * 
     * Optional function to retrieve policy-specific statistics for
     * monitoring and debugging. Can be NULL if not supported.
     * 
     * @param stats_buffer Buffer to write statistics to
     * @param buffer_size Size of the statistics buffer
     * @return Number of bytes written, or negative error code
     */
    int (*get_stats)(char *stats_buffer, size_t buffer_size);
    
    /**
     * @brief Policy name and version information
     */
    const char *name;           /**< Human-readable policy name */
    const char *version;        /**< Policy version string */
    const char *description;    /**< Brief policy description */
} scheduler_policy_t;

/**
 * @brief Scheduler Policy Types
 * 
 * Enumeration of common scheduler policy types for identification
 * and configuration purposes.
 */
typedef enum {
    SCHED_POLICY_ROUND_ROBIN = 0,   /**< Round-robin scheduling */
    SCHED_POLICY_PRIORITY,          /**< Priority-based scheduling */
    SCHED_POLICY_CFS,               /**< Completely Fair Scheduler */
    SCHED_POLICY_REALTIME,          /**< Real-time scheduling */
    SCHED_POLICY_CUSTOM             /**< Custom/experimental policy */
} scheduler_policy_type_t;

/**
 * @brief Scheduler Configuration
 * 
 * Configuration structure for scheduler policy parameters.
 */
typedef struct scheduler_config {
    scheduler_policy_type_t type;   /**< Policy type */
    uint32_t time_slice_ms;         /**< Time slice in milliseconds */
    uint32_t max_priority;          /**< Maximum priority level */
    uint32_t default_priority;      /**< Default process priority */
    uint32_t flags;                 /**< Policy-specific flags */
} scheduler_config_t;

/**
 * @brief Ring3 Scheduler Interface Functions
 * 
 * These functions provide the interface between the Ring3 scheduler policy
 * and the Ring0 scheduling mechanism. They will be implemented as part of
 * the Ring3 runtime library.
 */

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
                             const scheduler_config_t *config);

/**
 * @brief Unregister the current scheduler policy
 * 
 * Unregisters the current scheduler policy and reverts to the default
 * policy if available.
 * 
 * @return 0 on success, negative error code on failure
 */
int scheduler_unregister_policy(void);

/**
 * @brief Get the current scheduler policy
 * 
 * Returns a pointer to the currently registered scheduler policy.
 * 
 * @return Pointer to current policy, or NULL if none registered
 */
const scheduler_policy_t* scheduler_get_current_policy(void);

/**
 * @brief Request a scheduling decision
 * 
 * Requests the current policy to make a scheduling decision. This function
 * interfaces with the Ring0 mechanism to perform the actual context switch.
 * 
 * @return 0 on success, negative error code on failure
 */
int scheduler_request_schedule(void);

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
int scheduler_notify_state_change(proc_t *proc, int old_state, int new_state);

/**
 * @brief Default Round-Robin Policy
 * 
 * A default round-robin scheduler policy implementation that can be used
 * as a reference or fallback policy.
 */
extern const scheduler_policy_t scheduler_default_round_robin;

/**
 * @brief Error Codes
 */
#define SCHED_ERROR_INVALID_POLICY  (-1)    /**< Invalid policy structure */
#define SCHED_ERROR_ALREADY_REGISTERED (-2) /**< Policy already registered */
#define SCHED_ERROR_NOT_REGISTERED  (-3)    /**< No policy registered */
#define SCHED_ERROR_INIT_FAILED     (-4)    /**< Policy initialization failed */
#define SCHED_ERROR_SYSCALL_FAILED  (-5)    /**< Ring0 syscall failed */
#define SCHED_ERROR_INVALID_PROC    (-6)    /**< Invalid process pointer */

/**
 * @brief Scheduler Policy Validation
 * 
 * Validates that a scheduler policy structure is properly formed and
 * contains all required function pointers.
 * 
 * @param policy Pointer to the policy to validate
 * @return 0 if valid, negative error code if invalid
 */
int scheduler_validate_policy(const scheduler_policy_t *policy);

#endif /* AYKEN_RING3_SCHEDULER_H */