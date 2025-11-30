// kernel/include/sched.h
#ifndef AYKEN_SCHED_H
#define AYKEN_SCHED_H

#include <stdint.h>
#include "proc.h"

// Process durumları
typedef enum {
    PROC_READY = 0,
    PROC_RUNNING,
    PROC_BLOCKED,
    PROC_ZOMBIE
} proc_state_t;

// Scheduler API
void sched_init(void);
void sched_add(proc_t *proc);
void sched_yield(void);
void sched_start(void);

extern proc_t *current_proc;

#endif // AYKEN_SCHED_H
