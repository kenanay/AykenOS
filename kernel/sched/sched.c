// kernel/sched/sched.c
// Scheduler stub implementation

#include "sched.h"

void sched_init(void)
{
    // TODO: Scheduler initialization
    // - Task queue setup
    // - Timer interrupt handler registration
}

void sched_start(void)
{
    // TODO: Start scheduler
    // For now, just halt
    for (;;) {
        __asm__ __volatile__("hlt");
    }
}

void sched_yield(void)
{
    // TODO: Yield CPU to next task
}

void sched_add_task(void *task)
{
    // TODO: Add task to scheduler queue
    (void)task;
}
