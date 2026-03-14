#ifndef AYKEN_RING3_SCHEDULER_H
#define AYKEN_RING3_SCHEDULER_H

#include <stddef.h>
#include <stdint.h>

typedef struct proc proc_t;

typedef struct scheduler_policy {
    proc_t* (*select_next)(proc_t *ready_queue);
    void (*enqueue_ready)(proc_t *proc);
    void (*handle_block)(proc_t *proc, void *wait_obj);
    int (*init)(void);
    void (*cleanup)(void);
    int (*get_stats)(char *stats_buffer, size_t buffer_size);
    const char *name;
    const char *version;
    const char *description;
} scheduler_policy_t;

typedef enum {
    SCHED_POLICY_ROUND_ROBIN = 0,
    SCHED_POLICY_PRIORITY,
    SCHED_POLICY_CFS,
    SCHED_POLICY_REALTIME,
    SCHED_POLICY_CUSTOM
} scheduler_policy_type_t;

typedef struct scheduler_config {
    scheduler_policy_type_t type;
    uint32_t time_slice_ms;
    uint32_t max_priority;
    uint32_t default_priority;
    uint32_t flags;
} scheduler_config_t;

int scheduler_register_policy(const scheduler_policy_t *policy,
                             const scheduler_config_t *config);
int scheduler_unregister_policy(void);
const scheduler_policy_t* scheduler_get_current_policy(void);
int scheduler_request_schedule(void);
int scheduler_notify_state_change(proc_t *proc, int old_state, int new_state);
int scheduler_validate_policy(const scheduler_policy_t *policy);

extern const scheduler_policy_t scheduler_default_round_robin;

#define SCHED_ERROR_INVALID_POLICY  (-1)
#define SCHED_ERROR_ALREADY_REGISTERED (-2)
#define SCHED_ERROR_NOT_REGISTERED  (-3)
#define SCHED_ERROR_INIT_FAILED     (-4)
#define SCHED_ERROR_SYSCALL_FAILED  (-5)
#define SCHED_ERROR_INVALID_PROC    (-6)

#endif
