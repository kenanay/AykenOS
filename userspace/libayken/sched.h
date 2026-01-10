// userspace/libayken/sched.h - Ring3 scheduler proxy
#ifndef __USERSPACE_LIBAYKEN_SCHED_H
#define __USERSPACE_LIBAYKEN_SCHED_H

// Forward declaration of proc_t to avoid pulling in full kernel headers.
// This will be an opaque handle for the userspace library.
typedef struct proc proc_t;

typedef struct scheduler_proxy {
    proc_t* (*scheduler_select_next)(void);
    void (*scheduler_add_proc)(proc_t* proc);
    void (*scheduler_remove_proc)(proc_t* proc);
} scheduler_proxy_t;

#endif // __USERSPACE_LIBAYKEN_SCHED_H
