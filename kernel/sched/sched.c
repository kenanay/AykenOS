// kernel/sched/sched.c
// Scheduler stub implementation

#include <stddef.h>
#include "sched.h"
#include "../arch/x86_64/cpu.h"
#include "../include/mm.h"

static proc_t *ready_head = NULL;
static proc_t *ready_tail = NULL;
static proc_t *blocked_head = NULL;

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

static void enqueue_blocked(proc_t *p)
{
    p->next = blocked_head;
    blocked_head = p;
}

static void remove_from_blocked(proc_t *p)
{
    proc_t **iter = &blocked_head;
    while (*iter) {
        if (*iter == p) {
            *iter = p->next;
            p->next = NULL;
            return;
        }
        iter = &((*iter)->next);
    }
}

void sched_init(void)
{
    ready_head = ready_tail = NULL;
    blocked_head = NULL;
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

    paging_load_cr3(current_proc->context.cr3);
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

    paging_load_cr3(current_proc->context.cr3);

    if (prev) {
        context_switch(&prev->context, &current_proc->context);
    } else {
        switch_to_first(&current_proc->context);
    }

    enable_interrupts();
}

void sched_block_current(void)
{
    disable_interrupts();

    proc_t *prev = current_proc;
    if (!prev) {
        enable_interrupts();
        return;
    }

    prev->state = PROC_BLOCKED;
    enqueue_blocked(prev);

    proc_t *next = dequeue_ready();
    if (!next) {
        enable_interrupts();
        return;
    }

    current_proc = next;
    current_proc->state = PROC_RUNNING;
    paging_load_cr3(current_proc->context.cr3);
    context_switch(&prev->context, &current_proc->context);

    enable_interrupts();
}

void sched_wake(proc_t *proc)
{
    if (!proc || proc->state != PROC_BLOCKED)
        return;

    remove_from_blocked(proc);
    proc->state = PROC_READY;
    proc->wait_obj = NULL;
    enqueue_ready(proc);
}

void sched_wake_all(void *wait_obj)
{
    proc_t *iter = blocked_head;
    while (iter) {
        proc_t *next = iter->next;
        if (iter->wait_obj == wait_obj) {
            sched_wake(iter);
        }
        iter = next;
    }
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
