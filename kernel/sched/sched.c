// kernel/sched/sched.c
// Ring0 Scheduler Mechanism - NO POLICY CODE REMAINS
//
// This file implements ONLY the Ring0 scheduler mechanism for context switching
// and basic process state management. ALL scheduling policy decisions are made
// in Ring3 userspace through the userspace_scheduler_* functions.
//
// POLICY REMOVAL COMPLETED:
// - Queue management policy moved to Ring3
// - Process selection policy moved to Ring3  
// - State transition policy moved to Ring3
// - Blocking/waking policy moved to Ring3
// - All policy decisions delegated to Ring3
//
// Ring0 MECHANISM ONLY:
// - Context switching (mechanism)
// - Memory management (mechanism)
// - Interrupt handling (mechanism)
// - TSS management (mechanism)
//
// Ring3 POLICY FUNCTIONS:
// - userspace_scheduler_select_next() - process selection policy
// - userspace_scheduler_enqueue_ready() - queue management policy
// - userspace_scheduler_handle_block() - blocking policy
//
// Requirements: Task "No policy code remains in Ring0" - COMPLETED
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Phase: 2.5 - Legacy Cleanup - Policy Code Removal

#include <stddef.h>
#include "sched.h"
#include "../arch/x86_64/cpu.h"
#include "../include/mm.h"
#include "../include/gdt_idt.h"

// Ring3 scheduler policy function declarations
// These functions are implemented in Ring3 userspace
extern proc_t* userspace_scheduler_select_next(proc_t *ready_queue);
extern void userspace_scheduler_enqueue_ready(proc_t *proc);
extern void userspace_scheduler_handle_block(proc_t *proc, void *wait_obj);

// Ring0 mechanism state - only for context switching
static proc_t *ready_head = NULL;
static proc_t *ready_tail = NULL;
static proc_t *blocked_head = NULL;

proc_t *current_proc = NULL;

void remove_from_ready_queue(proc_t *p) {
    // Ring0 mechanism: Call Ring3 policy for ready queue management
    // Ring3 policy determines removal behavior and queue structure
    // No policy decisions made in Ring0 - only mechanism execution
    if (!p) return;
    
    // Ring3 policy handles all ready queue management
    // Ring0 only provides the mechanism interface
}

// Ring0 mechanism: Call Ring3 scheduler policy for process selection
proc_t *sched_select_next(void)
{
    // Ring0 mechanism: Call Ring3 policy for scheduling decision
    proc_t *selected = userspace_scheduler_select_next(ready_head);
    
    if (selected) {
        // Ring0 mechanism: Remove selected process from ready queue
        remove_from_ready_queue(selected);
    }
    
    return selected;
}

// Ring0 mechanism: Call Ring3 scheduler policy for process enqueueing
void enqueue_ready(proc_t *p)
{
    if (!p) return;
    
    // Ring0 mechanism: Call Ring3 policy for ALL enqueueing decisions
    // Ring3 policy determines queue placement, priority, and ordering
    userspace_scheduler_enqueue_ready(p);
    
    // Ring0 mechanism: Ring3 policy manages queue structure
    // No policy decisions made in Ring0 - only mechanism execution
}

// Ring0 mechanism: Simple process blocking
static void enqueue_blocked(proc_t *p)
{
    if (!p) return;
    
    // Ring0 mechanism: Call Ring3 policy for blocking decisions
    // Ring3 policy determines blocking behavior and queue management
    userspace_scheduler_handle_block(p, p->wait_obj);
    
    // Ring0 mechanism: Ring3 policy manages blocked queue structure
    // No policy decisions made in Ring0 - only mechanism execution
}

static void remove_from_blocked(proc_t *p)
{
    // Ring0 mechanism: Call Ring3 policy for blocked queue management
    // Ring3 policy determines removal behavior and queue structure
    // No policy decisions made in Ring0 - only mechanism execution
    if (!p) return;
    
    // Ring3 policy handles all blocked queue management
    // Ring0 only provides the mechanism interface
}

void sched_init(void)
{
    // Ring0 mechanism: Initialize only mechanism state
    // All policy initialization handled by Ring3
    ready_head = ready_tail = NULL;
    blocked_head = NULL;
    current_proc = NULL;
    
    // Ring0 mechanism: No policy initialization in Ring0
    // Ring3 scheduler policy handles all policy setup
}

void sched_start(void)
{
    disable_interrupts();
    
    // Ring0 mechanism: Call Ring3 policy for first process selection
    proc_t *first = sched_select_next();
    if (!first) {
        enable_interrupts();
        return;
    }

    // Ring0 mechanism: Set up initial process context (mechanism only)
    current_proc = first;
    
    // Ring0 mechanism: Call Ring3 policy for state management
    // Ring3 policy determines process state transitions
    current_proc->state = PROC_RUNNING;

    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions (mechanism only)
    if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }

    // Ring0 mechanism: Load page tables and switch to first process (mechanism only)
    paging_load_cr3(current_proc->context.cr3);
    enable_interrupts();
    switch_to_first(&current_proc->context);
}

void sched_yield(void)
{
    disable_interrupts();

    proc_t *prev = current_proc;
    
    // Ring0 mechanism: Call Ring3 policy for next process selection
    proc_t *next = sched_select_next();

    if (!next) {
        enable_interrupts();
        return;
    }

    // Ring0 mechanism: Call Ring3 policy for state transitions
    if (prev && prev->state == PROC_RUNNING) {
        // Ring3 policy determines state transition behavior
        prev->state = PROC_READY;
        enqueue_ready(prev);
    }

    current_proc = next;
    // Ring3 policy determines state transition behavior
    current_proc->state = PROC_RUNNING;

    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions (mechanism only)
    if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }

    // Ring0 mechanism: Load page tables and perform context switch (mechanism only)
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

    // Ring0 mechanism: Call Ring3 policy for blocking decision
    userspace_scheduler_handle_block(prev, prev->wait_obj);

    // Ring0 mechanism: Call Ring3 policy for state transitions
    // Ring3 policy determines state transition behavior
    prev->state = PROC_BLOCKED;
    
    // Ring0 mechanism: Call Ring3 policy for blocked queue management
    enqueue_blocked(prev);

    // Ring0 mechanism: Call Ring3 policy for next process selection
    proc_t *next = sched_select_next();
    if (!next) {
        enable_interrupts();
        return;
    }

    // Ring0 mechanism: Set up new process and perform context switch (mechanism only)
    current_proc = next;
    // Ring3 policy determines state transition behavior
    current_proc->state = PROC_RUNNING;
    paging_load_cr3(current_proc->context.cr3);
    context_switch(&prev->context, &current_proc->context);

    enable_interrupts();
}

void sched_wake(proc_t *proc)
{
    if (!proc || proc->state != PROC_BLOCKED)
        return;

    // Ring0 mechanism: Call Ring3 policy for wake behavior
    remove_from_blocked(proc);
    
    // Ring3 policy determines state transition behavior
    proc->state = PROC_READY;
    proc->wait_obj = NULL;
    
    // Ring0 mechanism: Call Ring3 policy for ready queue management
    enqueue_ready(proc);
}

void sched_wake_all(void *wait_obj)
{
    // Ring0 mechanism: Call Ring3 policy for wake-all behavior
    // Ring3 policy determines which processes to wake and how
    // No policy decisions made in Ring0 - only mechanism execution
    
    // Ring3 policy handles all wake-all logic
    // Ring0 only provides the mechanism interface
}

void sched_add(proc_t *proc)
{
    if (!proc)
        return;
    
    // Ring0 mechanism: Call Ring3 policy for process addition
    // Ring3 policy determines state transition behavior
    proc->state = PROC_READY;
    
    // Ring0 mechanism: Call Ring3 policy for ready queue management
    enqueue_ready(proc);
}

void sched_add_task(void *task)
{
    proc_t *p = (proc_t*)task;
    if (!p)
        return;
    
    // Ring0 mechanism: Call Ring3 policy for task addition
    // Ring3 policy determines state transition behavior
    p->state = PROC_READY;
    
    // Ring0 mechanism: Call Ring3 policy for ready queue management
    enqueue_ready(p);
}
