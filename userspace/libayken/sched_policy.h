// userspace/libayken/sched_policy.h - Ring3 scheduler policy
#ifndef __USERSPACE_LIBAYKEN_SCHED_POLICY_H
#define __USERSPACE_LIBAYKEN_SCHED_POLICY_H

// Forward declaration of proc_t to avoid pulling in full kernel headers.
// This will be an opaque handle for the userspace library.
typedef struct proc proc_t;

typedef struct scheduler_policy {
    proc_t* (*select_next)(proc_t *ready_queue);
    void (*enqueue_ready)(proc_t *proc);
    void (*handle_block)(proc_t *proc, void *wait_obj);
} scheduler_policy_t;

#endif // __USERSPACE_LIBAYKEN_SCHED_POLICY_H
