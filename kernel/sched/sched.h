// kernel/include/sched.h
#ifndef AYKEN_SCHED_H
#define AYKEN_SCHED_H

#include <stdint.h>
#include "proc.h"

// Scheduler API
void sched_init(void);
void sched_add(proc_t *proc);
void sched_yield(void);
void sched_start(void);
void sched_block_current(void);
void sched_wake(proc_t *proc);
void sched_wake_all(void *wait_obj);

extern proc_t *current_proc;

#endif // AYKEN_SCHED_H
