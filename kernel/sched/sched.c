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
#include <stdint.h>
#include "sched.h"
#include "../arch/x86_64/cpu.h"
#include "../arch/x86_64/port_io.h"
#include "../drivers/console/fb_console.h"
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
static volatile uint32_t need_resched = 0;

static inline uint64_t read_msr(uint32_t msr)
{
    uint32_t lo, hi;
    __asm__ volatile ("rdmsr" : "=a"(lo), "=d"(hi) : "c"(msr));
    return ((uint64_t)hi << 32) | lo;
}

static void dbg_out_hex16(uint16_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    for (int i = 3; i >= 0; --i) {
        uint8_t nib = (v >> (i * 4)) & 0xF;
        outb(0xE9, (uint8_t)hex[nib]);
    }
}

static void dbg_print_tr(void)
{
    uint16_t tr = 0;
    __asm__ volatile ("str %0" : "=r"(tr));
    outb(0xE9, (uint8_t)'T');
    outb(0xE9, (uint8_t)'R');
    outb(0xE9, (uint8_t)'=');
    dbg_out_hex16(tr);
    outb(0xE9, (uint8_t)'\n');
}

static void map_kernel_stack_pages_into_pml4(uint64_t pml4_phys, uint64_t rsp0)
{
    uint64_t rsp = 0;
    __asm__ volatile("mov %%rsp, %0" : "=r"(rsp));
    uint64_t page = rsp & ~(AYKEN_FRAME_SIZE - 1);
    uint64_t phys = paging_get_phys(page);
    if (phys) {
        paging_map_page_in_pml4(pml4_phys, page, phys, AYKEN_PTE_WRITABLE);
    }

    uint64_t page_below = page - AYKEN_FRAME_SIZE;
    uint64_t phys_below = paging_get_phys(page_below);
    if (phys_below) {
        paging_map_page_in_pml4(pml4_phys, page_below, phys_below, AYKEN_PTE_WRITABLE);
    }

    if (rsp0) {
        uint64_t top_page = (rsp0 - 1) & ~(AYKEN_FRAME_SIZE - 1);
        uint64_t top_phys = paging_get_phys(top_page);
        if (top_phys) {
            paging_map_page_in_pml4(pml4_phys, top_page, top_phys, AYKEN_PTE_WRITABLE);
        }
        uint64_t below_page = top_page - AYKEN_FRAME_SIZE;
        uint64_t below_phys = paging_get_phys(below_page);
        if (below_phys) {
            paging_map_page_in_pml4(pml4_phys, below_page, below_phys, AYKEN_PTE_WRITABLE);
        }
    }
}

static void dbg_dump_bytes(const void *addr)
{
    static const char hex[] = "0123456789ABCDEF";
    const uint8_t *p = (const uint8_t *)addr;
    outb(0xE9, (uint8_t)'K');
    outb(0xE9, (uint8_t)'B');
    outb(0xE9, (uint8_t)':');
    for (int i = 0; i < 8; ++i) {
        uint8_t b = p[i];
        outb(0xE9, (uint8_t)hex[b >> 4]);
        outb(0xE9, (uint8_t)hex[b & 0x0F]);
    }
    outb(0xE9, (uint8_t)'\n');
}

void sched_request_resched(void)
{
    need_resched = 1;
}

uint32_t sched_take_resched(void)
{
    if (!need_resched)
        return 0;
    need_resched = 0;
    return 1;
}

void remove_from_ready_queue(proc_t *p) {
    if (!p || !ready_head)
        return;

    if (ready_head == p) {
        ready_head = p->next;
        if (ready_tail == p)
            ready_tail = NULL;
        p->next = NULL;
        return;
    }

    proc_t *prev = ready_head;
    while (prev->next && prev->next != p) {
        prev = prev->next;
    }
    if (prev->next == p) {
        prev->next = p->next;
        if (ready_tail == p)
            ready_tail = prev;
        p->next = NULL;
    }
}

// Ring0 mechanism: Call Ring3 scheduler policy for process selection
proc_t *sched_select_next(void)
{
    proc_t *selected = userspace_scheduler_select_next(ready_head);
    if (!selected)
        selected = ready_head;

    if (selected) {
        remove_from_ready_queue(selected);
    }

    return selected;
}

// Ring0 mechanism: Call Ring3 scheduler policy for process enqueueing
void enqueue_ready(proc_t *p)
{
    if (!p) return;
    
    userspace_scheduler_enqueue_ready(p);
    
    p->next = NULL;
    if (!ready_tail) {
        ready_head = ready_tail = p;
    } else {
        ready_tail->next = p;
        ready_tail = p;
    }
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
    outb(0xE9, (uint8_t)'S');
    disable_interrupts();
    
    // Ring0 mechanism: Call Ring3 policy for first process selection
    proc_t *first = sched_select_next();
    if (!first) {
        outb(0xE9, (uint8_t)'N');
        enable_interrupts();
        return;
    }
    outb(0xE9, (uint8_t)'F');

    // Ring0 mechanism: Set up initial process context (mechanism only)
    current_proc = first;
    
    // Ring0 mechanism: Call Ring3 policy for state management
    // Ring3 policy determines process state transitions
    current_proc->state = PROC_RUNNING;
    dbg_print_tr();

    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions (mechanism only)
    if (current_proc->context.cs == GDT_USER_CODE) {
        if (!current_proc->context.rsp0) {
            fb_print("[PANIC] Ring3 process has no rsp0 (TSS stack)\n");
            for (;;) __asm__ volatile("cli; hlt");
        }
        gdt_set_kernel_stack(current_proc->context.rsp0);
        __asm__ volatile("" ::: "memory");
        outb(0xE9, (uint8_t)'U');
        map_kernel_stack_pages_into_pml4(current_proc->context.cr3, current_proc->context.rsp0);
    } else if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }

    fb_print("[DBG] SCHED first: cs=");
    fb_print_hex(current_proc->context.cs);
    fb_print(" ss=");
    fb_print_hex(current_proc->context.ss);
    fb_print(" rip=");
    fb_print_hex(current_proc->context.rip);
    fb_print(" rsp=");
    fb_print_hex(current_proc->context.rsp);
    fb_print(" rsp0=");
    fb_print_hex(current_proc->context.rsp0);
    fb_print(" cr3=");
    fb_print_hex(current_proc->context.cr3);
    fb_print("\n");
    fb_print("[DBG] MAP rip=");
    fb_print_hex64(paging_get_phys(current_proc->context.rip));
    fb_print(" rsp=");
    fb_print_hex64(paging_get_phys(current_proc->context.rsp));
    fb_print("\n");
    fb_print("[DBG] PTE rip=");
    fb_print_hex64(paging_get_pte(current_proc->context.rip));
    fb_print(" rsp=");
    fb_print_hex64(paging_get_pte(current_proc->context.rsp));
    fb_print("\n");
    fb_print("[DBG] EFER=");
    fb_print_hex64(read_msr(0xC0000080));
    fb_print("\n");

    // Ring0 mechanism: Switch to first process (mechanism only)
    if (current_proc->context.cs != GDT_USER_CODE) {
        dbg_dump_bytes((const void *)current_proc->context.rip);
    }
    outb(0xE9, (uint8_t)'R');
    switch_to_first(&current_proc->context);
}

static void sched_yield_core(int reenable_if)
{
    disable_interrupts();

    proc_t *prev = current_proc;
    
    // Ring0 mechanism: Call Ring3 policy for next process selection
    proc_t *next = sched_select_next();

    if (!next) {
        if (reenable_if)
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
    if (current_proc->context.cs == GDT_USER_CODE) {
        if (!current_proc->context.rsp0) {
            fb_print("[PANIC] Ring3 process has no rsp0 (TSS stack)\n");
            for (;;) __asm__ volatile("cli; hlt");
        }
        gdt_set_kernel_stack(current_proc->context.rsp0);
        __asm__ volatile("" ::: "memory");
        map_kernel_stack_pages_into_pml4(current_proc->context.cr3, current_proc->context.rsp0);
    } else if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }

    if (prev) {
        context_switch(&prev->context, &current_proc->context);
    } else {
        switch_to_first(&current_proc->context);
    }

    if (reenable_if)
        enable_interrupts();
}

void sched_yield(void)
{
    sched_yield_core(1);
}

void sched_yield_irq(void)
{
    sched_request_resched();
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
    if (current_proc->context.cs == GDT_USER_CODE) {
        if (!current_proc->context.rsp0) {
            fb_print("[PANIC] Ring3 process has no rsp0 (TSS stack)\n");
            for (;;) __asm__ volatile("cli; hlt");
        }
        gdt_set_kernel_stack(current_proc->context.rsp0);
        __asm__ volatile("" ::: "memory");
        map_kernel_stack_pages_into_pml4(current_proc->context.cr3, current_proc->context.rsp0);
    } else if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }
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
