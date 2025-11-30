// kernel/sched/sched.c
// Scheduler stub implementation

#include <stddef.h>
#include "sched.h"
#include "../arch/x86_64/cpu.h"

static proc_t *ready_head = NULL;
static proc_t *ready_tail = NULL;

proc_t *current_proc = NULL;

static void enqueue_ready(proc_t *p)
{
    p->next = NULL;
    if (!ready_head) {
        ready_head = ready_tail = p;
    } else {
        ready_tail->next = p;
        ready_tail = p;
    }
}

static proc_t *dequeue_ready(void)
{
    if (!ready_head)
        return NULL;

    proc_t *p = ready_head;
    ready_head = ready_head->next;
    if (!ready_head)
        ready_tail = NULL;
    p->next = NULL;
    return p;
}

void sched_init(void)
{
    ready_head = ready_tail = NULL;
    current_proc = NULL;
}

void sched_start(void)
{
    disable_interrupts();
    proc_t *first = dequeue_ready();
    if (!first) {
        enable_interrupts();
        return;
    }

    current_proc = first;
    current_proc->state = PROC_RUNNING;

    __asm__ volatile("mov %0, %%cr3" :: "r"(current_proc->context.cr3));
    enable_interrupts();
    switch_to_first(&current_proc->context);
}

void sched_yield(void)
{
    disable_interrupts();

    proc_t *prev = current_proc;
    proc_t *next = dequeue_ready();

    if (!next) {
        enable_interrupts();
        return;
    }

    if (prev && prev->state == PROC_RUNNING) {
        prev->state = PROC_READY;
        enqueue_ready(prev);
    }

    current_proc = next;
    current_proc->state = PROC_RUNNING;

    __asm__ volatile("mov %0, %%cr3" :: "r"(current_proc->context.cr3));

    if (prev) {
        context_switch(&prev->context, &current_proc->context);
    } else {
        switch_to_first(&current_proc->context);
    }

    enable_interrupts();
}

void sched_add(proc_t *proc)
{
    if (!proc)
        return;
    proc->state = PROC_READY;
    enqueue_ready(proc);
}

void sched_add_task(void *task)
{
    (void)task;
}
