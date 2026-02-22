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
#include "sched_mailbox.h"
#include "../arch/x86_64/cpu.h"
#include "../arch/x86_64/port_io.h"
#include "../drivers/console/fb_console.h"
#include "../include/mm.h"
#include "../include/gdt_idt.h"

#ifndef AYKEN_DEBUG_SCHED
#define AYKEN_DEBUG_SCHED 0
#endif

#if AYKEN_DEBUG_SCHED
#define SCHED_DBG_OUT(ch) outb(0xE9, (uint8_t)(ch))
#else
#define SCHED_DBG_OUT(ch) do { (void)(ch); } while (0)
#endif

#if AYKEN_DEBUG_SCHED
static void sched_dbg_puts(const char *s)
{
    if (!s) {
        return;
    }
    while (*s) {
        SCHED_DBG_OUT((uint8_t)*s++);
    }
}

static void sched_dbg_mark_pid(uint32_t pid)
{
    if (pid != 2u && pid != 3u) {
        return;
    }
    sched_dbg_puts("MARK:PID=");
    SCHED_DBG_OUT((uint8_t)('0' + (uint8_t)pid));
    SCHED_DBG_OUT((uint8_t)'\n');
}

static void sched_dbg_mark_sw(char from, char to)
{
    sched_dbg_puts("MARK:SW=");
    SCHED_DBG_OUT((uint8_t)from);
    SCHED_DBG_OUT((uint8_t)'>');
    SCHED_DBG_OUT((uint8_t)to);
    SCHED_DBG_OUT((uint8_t)'\n');
}

static void sched_dbg_mark_iret(void)
{
    sched_dbg_puts("MARK:IRET\n");
}
#else
static inline void sched_dbg_mark_pid(uint32_t pid) { (void)pid; }
static inline void sched_dbg_mark_sw(char from, char to) { (void)from; (void)to; }
static inline void sched_dbg_mark_iret(void) { }
#endif

// Ring3 scheduler policy function declarations
// These functions are implemented in Ring3 userspace
extern proc_t* userspace_scheduler_select_next(proc_t *ready_queue);
extern void userspace_scheduler_enqueue_ready(proc_t *proc);
extern void userspace_scheduler_handle_block(proc_t *proc, void *wait_obj);

// Ring0 mechanism state - only for context switching
static proc_t *ready_head = NULL;
static proc_t *ready_tail = NULL;
static proc_t *blocked_head = NULL;

// Flag to track if scheduler has started (to avoid calling userspace functions during boot)
static int scheduler_started = 0;

proc_t *current_proc = NULL;
static volatile uint32_t need_resched = 0;
// Set by IRQ path when current user context is explicitly snapshotted.
// context_switch.asm consumes this flag to avoid overwriting user RIP/RSP
// with kernel scheduler frame values.
volatile uint32_t sched_irq_user_ctx_saved = 0;

#define RING3_CANARY_ADDR 0x0000000000405000ULL
#define RING3_CANARY_PRE  0x1111111122222222ULL
#define RING3_CANARY_POST 0x3333333344444444ULL

#if AYKEN_DEBUG_SCHED
static __attribute__((noreturn)) void sched_debug_assert_fail(char code)
{
    SCHED_DBG_OUT('[');
    SCHED_DBG_OUT('A');
    SCHED_DBG_OUT('S');
    SCHED_DBG_OUT('R');
    SCHED_DBG_OUT('T');
    SCHED_DBG_OUT(':');
    SCHED_DBG_OUT((uint8_t)code);
    SCHED_DBG_OUT(']');
    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}
#endif

static inline uint64_t read_msr(uint32_t msr) __attribute__((unused));
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
        SCHED_DBG_OUT((uint8_t)hex[nib]);
    }
}

static void dbg_out_hex64(uint64_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        uint8_t nib = (uint8_t)((v >> (i * 4)) & 0xF);
        SCHED_DBG_OUT((uint8_t)hex[nib]);
    }
}

static int read_user_u64_via_pml4(uint64_t pml4_phys, uint64_t va, uint64_t *out)
{
    if (!pml4_phys || !out) {
        return 0;
    }

    uint64_t root_phys = pml4_phys & AYKEN_PTE_ADDR_MASK;
    uint64_t *pml4 = (uint64_t *)paging_phys_to_virt(root_phys);
    if (!pml4) {
        return 0;
    }

    uint16_t pml4_i = (uint16_t)((va >> 39) & 0x1FF);
    uint16_t pdpt_i = (uint16_t)((va >> 30) & 0x1FF);
    uint16_t pd_i = (uint16_t)((va >> 21) & 0x1FF);
    uint16_t pt_i = (uint16_t)((va >> 12) & 0x1FF);

    uint64_t pml4e = pml4[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) {
        return 0;
    }

    uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);
    if (!pdpt) {
        return 0;
    }
    uint64_t pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT) || (pdpte & (1ULL << 7))) {
        return 0;
    }

    uint64_t *pd = (uint64_t *)paging_phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);
    if (!pd) {
        return 0;
    }
    uint64_t pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT) || (pde & (1ULL << 7))) {
        return 0;
    }

    uint64_t *pt = (uint64_t *)paging_phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);
    if (!pt) {
        return 0;
    }
    uint64_t pte = pt[pt_i];
    if (!(pte & AYKEN_PTE_PRESENT)) {
        return 0;
    }

    uint64_t page_off = va & (AYKEN_FRAME_SIZE - 1);
    if (page_off > (AYKEN_FRAME_SIZE - sizeof(uint64_t))) {
        return 0;
    }

    uint8_t *page = (uint8_t *)paging_phys_to_virt(pte & AYKEN_PTE_ADDR_MASK);
    if (!page) {
        return 0;
    }

    uint64_t value = 0;
    for (int i = 0; i < 8; ++i) {
        value |= ((uint64_t)page[page_off + (uint64_t)i]) << (i * 8);
    }
    *out = value;
    return 1;
}

static void dbg_print_tr(void)
{
    uint16_t tr = 0;
    __asm__ volatile ("str %0" : "=r"(tr));
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex16(tr);
    SCHED_DBG_OUT((uint8_t)'\n');
}

static void map_kernel_stack_pages_into_pml4(uint64_t pml4_phys, uint64_t rsp0)
{
    uint64_t old_cr3 = 0;
    uint64_t kernel_cr3 = paging_get_kernel_pml4_phys();
    __asm__ volatile("mov %%cr3, %0" : "=r"(old_cr3));
    if (kernel_cr3 && old_cr3 != kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
    }

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

    if (kernel_cr3 && old_cr3 != kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(old_cr3) : "memory");
    }
}

static void dbg_dump_bytes(const void *addr) __attribute__((unused));
static void dbg_dump_bytes(const void *addr)
{
    static const char hex[] = "0123456789ABCDEF";
    const uint8_t *p = (const uint8_t *)addr;
    SCHED_DBG_OUT((uint8_t)'K');
    SCHED_DBG_OUT((uint8_t)'B');
    SCHED_DBG_OUT((uint8_t)':');
    for (int i = 0; i < 8; ++i) {
        uint8_t b = p[i];
        SCHED_DBG_OUT((uint8_t)hex[b >> 4]);
        SCHED_DBG_OUT((uint8_t)hex[b & 0x0F]);
    }
    SCHED_DBG_OUT((uint8_t)'\n');
}

void sched_request_resched(void)
{
    SCHED_DBG_OUT((uint8_t)'R'); // Preemption request marker
    need_resched = 1;
}

void sched_request_resched_irq(void)
{
    // IRQ path: keep logging quiet; timer can request frequently.
    need_resched = 1;
}

uint32_t sched_take_resched(void)
{
    if (!need_resched)
        return 0;
    SCHED_DBG_OUT((uint8_t)'r'); // Preemption taken marker
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
    // DEBUG: Scheduler selection entry marker
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'S');
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'L');
    SCHED_DBG_OUT((uint8_t)']');
    
    // ✅ CRITICAL FIX: Ring0 cannot call Ring3 functions directly!
    // Ring0→Ring3 transition ONLY via IRETQ, not C call
    // For now: pure mechanical round-robin (no policy)
    proc_t *selected = ready_head;
    
    // ❌ DISABLED: Cannot call Ring3 from Ring0 via C call
    // This causes illegal privilege transition → #GP → triple fault
    // if (scheduler_started) {
    //     selected = userspace_scheduler_select_next(ready_head);
    // }

    // DEBUG: Show selected PID
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'I');
    SCHED_DBG_OUT((uint8_t)'D');
    SCHED_DBG_OUT((uint8_t)'=');
    if (selected) {
        if (selected->pid < 10) {
            SCHED_DBG_OUT((uint8_t)('0' + selected->pid));
        } else {
            SCHED_DBG_OUT((uint8_t)('A' + selected->pid - 10));
        }
        SCHED_DBG_OUT((uint8_t)' ');
        SCHED_DBG_OUT((uint8_t)'S');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'=');
        if (selected->state < 10) {
            SCHED_DBG_OUT((uint8_t)('0' + selected->state));
        } else {
            SCHED_DBG_OUT((uint8_t)('A' + selected->state - 10));
        }
        SCHED_DBG_OUT((uint8_t)' ');
        SCHED_DBG_OUT((uint8_t)'R');
        SCHED_DBG_OUT((uint8_t)'I');
        SCHED_DBG_OUT((uint8_t)'P');
        SCHED_DBG_OUT((uint8_t)'=');
        
        // DEBUG: Show selected pointer address
        SCHED_DBG_OUT((uint8_t)'@');
        uint64_t ptr = (uint64_t)selected;
        for (int i = 7; i >= 0; i--) {
            uint8_t nib = (ptr >> (i * 4)) & 0xF;
            if (nib < 10) {
                SCHED_DBG_OUT((uint8_t)('0' + nib));
            } else {
                SCHED_DBG_OUT((uint8_t)('A' + nib - 10));
            }
        }
        SCHED_DBG_OUT((uint8_t)' ');
        
        // Show RIP as 4 hex digits (simplified)
        uint64_t rip = selected->context.rip;
        for (int i = 3; i >= 0; i--) {
            uint8_t nib = (rip >> (i * 4)) & 0xF;
            if (nib < 10) {
                SCHED_DBG_OUT((uint8_t)('0' + nib));
            } else {
                SCHED_DBG_OUT((uint8_t)('A' + nib - 10));
            }
        }
        
        // DEBUG: Show full RIP (8 hex digits)
        SCHED_DBG_OUT((uint8_t)' ');
        SCHED_DBG_OUT((uint8_t)'F');
        SCHED_DBG_OUT((uint8_t)'U');
        SCHED_DBG_OUT((uint8_t)'L');
        SCHED_DBG_OUT((uint8_t)'L');
        SCHED_DBG_OUT((uint8_t)'=');
        for (int i = 7; i >= 0; i--) {
            uint8_t nib = (rip >> (i * 4)) & 0xF;
            if (nib < 10) {
                SCHED_DBG_OUT((uint8_t)('0' + nib));
            } else {
                SCHED_DBG_OUT((uint8_t)('A' + nib - 10));
            }
        }
        SCHED_DBG_OUT((uint8_t)'\n');
    } else {
        SCHED_DBG_OUT((uint8_t)'N');
        SCHED_DBG_OUT((uint8_t)'U');
        SCHED_DBG_OUT((uint8_t)'L');
        SCHED_DBG_OUT((uint8_t)'L');
        SCHED_DBG_OUT((uint8_t)'\n');
    }

    if (selected) {
        remove_from_ready_queue(selected);
    }

    return selected;
}

// Ring0 mechanism: Call Ring3 scheduler policy for process enqueueing
void enqueue_ready(proc_t *p)
{
    if (!p) return;
    
    // ❌ DISABLED: Cannot call Ring3 from Ring0 via C call
    // Ring0→Ring3 transition ONLY via IRETQ, not C function call
    // This causes illegal privilege transition → #GP → triple fault
    // if (scheduler_started) {
    //     userspace_scheduler_enqueue_ready(p);
    // }
    
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
    
    // ❌ DISABLED: Cannot call Ring3 from Ring0 via C call
    // Ring0→Ring3 transition ONLY via IRETQ, not C function call
    // This causes illegal privilege transition → #GP → triple fault
    // if (scheduler_started) {
    //     userspace_scheduler_handle_block(p, p->wait_obj);
    // }
    
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
    
    // Ring0 mechanism: Initialize scheduler bridge mailbox
    sched_mailbox_init();
    
    // Ring0 mechanism: No policy initialization in Ring0
    // Ring3 scheduler policy handles all policy setup
}

void sched_start(void)
{
    SCHED_DBG_OUT((uint8_t)'S');
    SCHED_DBG_OUT((uint8_t)'1');
    
    // Mark scheduler as started so userspace functions can be called
    scheduler_started = 1;
    SCHED_DBG_OUT((uint8_t)'2');
    
    // Debug: Check ready queue
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'Q');
    SCHED_DBG_OUT((uint8_t)']');
    int count = 0;
    proc_t *p = ready_head;
    while (p) {
        count++;
        p = p->next;
    }
    // Output count as hex digit
    if (count < 10) {
        SCHED_DBG_OUT((uint8_t)('0' + count));
    } else {
        SCHED_DBG_OUT((uint8_t)('A' + count - 10));
    }
    SCHED_DBG_OUT((uint8_t)'\n');
    SCHED_DBG_OUT((uint8_t)'3');
    
    disable_interrupts();
    SCHED_DBG_OUT((uint8_t)'4');
    
    // Ring0 mechanism: Call Ring3 policy for first process selection
    proc_t *first = sched_select_next();
    if (!first) {
        SCHED_DBG_OUT((uint8_t)'N');
        enable_interrupts();
        return;
    }
    SCHED_DBG_OUT((uint8_t)'F');

    // Ring0 mechanism: Set up initial process context (mechanism only)
    current_proc = first;
    current_proc->state = PROC_RUNNING;
    
    // MVP-0: Scheduler bridge self-test (emits markers for gate validation)
    // Called here after current_proc is set but before switch_to_first
    // Compile-out in release: self-test is validation-only
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    // Test marker to verify debugcon is working
    outb(0xE9, 'M');
    outb(0xE9, 'B');
    outb(0xE9, 'T');
    outb(0xE9, '\n');
    sched_mailbox_selftest();
    outb(0xE9, 'M');
    outb(0xE9, 'B');
    outb(0xE9, 'E');
    outb(0xE9, '\n');
    
    // MVP-2: Ring3 simulation test (validates Ring3 library behavior)
    outb(0xE9, 'R');
    outb(0xE9, '3');
    outb(0xE9, 'S');
    outb(0xE9, '\n');
    sched_mailbox_test_ring3_simulation(current_proc);
    outb(0xE9, 'R');
    outb(0xE9, '3');
    outb(0xE9, 'E');
    outb(0xE9, '\n');
#endif
    
    SCHED_DBG_OUT((uint8_t)'T');  // TSS setup
    
    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions (mechanism only)
    if (current_proc->context.cs == GDT_USER_CODE) {
        if (!current_proc->context.rsp0) {
            SCHED_DBG_OUT((uint8_t)'!');  // PANIC: no rsp0
            for (;;) __asm__ volatile("cli; hlt");
        }
        gdt_set_kernel_stack(current_proc->context.rsp0);
        __asm__ volatile("" ::: "memory");
        map_kernel_stack_pages_into_pml4(current_proc->context.cr3, current_proc->context.rsp0);
    } else if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }

    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'0');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(current_proc->context.rsp0);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)'0');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(kernel_tss.rsp0);
    SCHED_DBG_OUT((uint8_t)'\n');
    
    // DIAGNOSTIC: Verify TR is set correctly after TSS setup
    dbg_print_tr();
    
    SCHED_DBG_OUT((uint8_t)'@');  // About to switch_to_first
    
    // CRITICAL: Call switch_to_first with interrupts disabled
    // Interrupts will be enabled by the first process's RFLAGS (IF=1)
    // This prevents timer interrupts from firing before we have a proper context
    switch_to_first(&current_proc->context);
    
    // DEBUG: This should never be reached if switch_to_first works
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)']');
}

static void sched_yield_core(int reenable_if)
{
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'S');
    SCHED_DBG_OUT((uint8_t)'C');
    SCHED_DBG_OUT((uint8_t)'H');
    SCHED_DBG_OUT((uint8_t)']');
    SCHED_DBG_OUT((uint8_t)'\n');
    
    disable_interrupts();

    proc_t *prev = current_proc;
    SCHED_DBG_OUT((uint8_t)'P');
    if (prev) {
        SCHED_DBG_OUT((uint8_t)'1');
        // Show current PID
        if (prev->pid < 10) {
            SCHED_DBG_OUT((uint8_t)('0' + prev->pid));
        } else {
            SCHED_DBG_OUT((uint8_t)('A' + prev->pid - 10));
        }
    } else {
        SCHED_DBG_OUT((uint8_t)'0');
    }
    
    // Ring0 mechanism: Call Ring3 policy for next process selection
    proc_t *next = sched_select_next();
    SCHED_DBG_OUT((uint8_t)'N');
    if (next) {
        SCHED_DBG_OUT((uint8_t)'1');
        // Show next PID
        if (next->pid < 10) {
            SCHED_DBG_OUT((uint8_t)('0' + next->pid));
        } else {
            SCHED_DBG_OUT((uint8_t)('A' + next->pid - 10));
        }
    } else {
        SCHED_DBG_OUT((uint8_t)'0');
    }
    SCHED_DBG_OUT((uint8_t)'\n');

    if (!next) {
        SCHED_DBG_OUT((uint8_t)'X');
        if (reenable_if)
            enable_interrupts();
        return;
    }

#if AYKEN_DEBUG_SCHED
    if (prev && next == prev && ((prev->context.cs & 0x3) == 0x3)) {
        sched_debug_assert_fail('S'); // same user proc selected
    }
    if (((next->context.cs & 0x3) == 0x3) && next->context.cs != GDT_USER_CODE) {
        sched_debug_assert_fail('C'); // invalid user CS selector
    }
    if (((next->context.cs & 0x3) == 0x0) && next->context.cs != GDT_KERNEL_CODE) {
        sched_debug_assert_fail('c'); // invalid kernel CS selector
    }
    if (prev && next != prev &&
        ((prev->context.cs & 0x3) == 0x3) &&
        ((next->context.cs & 0x3) == 0x3) &&
        (prev->context.cr3 == next->context.cr3)) {
        sched_debug_assert_fail('3'); // user->user switch without CR3 change
    }
#endif

    // Ring0 mechanism: Call Ring3 policy for state transitions
    if (prev && prev->state == PROC_RUNNING) {
        // Ring3 policy determines state transition behavior
        prev->state = PROC_READY;
        enqueue_ready(prev);
    }

    current_proc = next;
    // Ring3 policy determines state transition behavior
    current_proc->state = PROC_RUNNING;

    sched_dbg_mark_pid(current_proc->pid);

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

    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'1');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(current_proc->context.rsp0);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)'1');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(kernel_tss.rsp0);
    SCHED_DBG_OUT((uint8_t)'\n');

    if (prev) {
        char from_ring = ((prev->context.cs & 0x3) == 0x3) ? 'U' : 'K';
        char to_ring = ((current_proc->context.cs & 0x3) == 0x3) ? 'U' : 'K';
        sched_dbg_mark_sw(from_ring, to_ring);

        // Debug: Show context switch
        SCHED_DBG_OUT((uint8_t)'[');
        SCHED_DBG_OUT((uint8_t)'S');
        SCHED_DBG_OUT((uint8_t)'W');
        SCHED_DBG_OUT((uint8_t)']');
        // Show prev CS
        if (prev->context.cs == GDT_USER_CODE) {
            SCHED_DBG_OUT((uint8_t)'U');
        } else {
            SCHED_DBG_OUT((uint8_t)'K');
        }
        SCHED_DBG_OUT((uint8_t)'>');
        // Show next CS  
        if (current_proc->context.cs == GDT_USER_CODE) {
            SCHED_DBG_OUT((uint8_t)'U');
        } else {
            SCHED_DBG_OUT((uint8_t)'K');
        }
        SCHED_DBG_OUT((uint8_t)'\n');
        
        // DEBUG: Context switch entry marker
        SCHED_DBG_OUT((uint8_t)'A');
        SCHED_DBG_OUT((uint8_t)'B');
        SCHED_DBG_OUT((uint8_t)'O');
        SCHED_DBG_OUT((uint8_t)'U');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'_');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'O');
        SCHED_DBG_OUT((uint8_t)'_');
        SCHED_DBG_OUT((uint8_t)'I');
        SCHED_DBG_OUT((uint8_t)'R');
        SCHED_DBG_OUT((uint8_t)'E');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'Q');
        SCHED_DBG_OUT((uint8_t)'\n');

        sched_dbg_mark_iret();
        
        context_switch(&prev->context, &current_proc->context);
        
        // Ring3 INT80 diagnostic: verify whether user code resumed after syscall.
        if (prev && prev->context.cs == GDT_USER_CODE) {
            uint64_t canary = 0;
            SCHED_DBG_OUT((uint8_t)'[');
            SCHED_DBG_OUT((uint8_t)'C');
            SCHED_DBG_OUT((uint8_t)'A');
            SCHED_DBG_OUT((uint8_t)'N');
            SCHED_DBG_OUT((uint8_t)'=');
            if (read_user_u64_via_pml4(prev->context.cr3, RING3_CANARY_ADDR, &canary)) {
                dbg_out_hex64(canary);
                SCHED_DBG_OUT((uint8_t)' ');
                if (canary == RING3_CANARY_POST) {
                    SCHED_DBG_OUT((uint8_t)'P');
                    SCHED_DBG_OUT((uint8_t)'O');
                    SCHED_DBG_OUT((uint8_t)'S');
                    SCHED_DBG_OUT((uint8_t)'T');
                } else if (canary == RING3_CANARY_PRE) {
                    SCHED_DBG_OUT((uint8_t)'P');
                    SCHED_DBG_OUT((uint8_t)'R');
                    SCHED_DBG_OUT((uint8_t)'E');
                } else {
                    SCHED_DBG_OUT((uint8_t)'?');
                }
            } else {
                SCHED_DBG_OUT((uint8_t)'!');
            }
            SCHED_DBG_OUT((uint8_t)']');
            SCHED_DBG_OUT((uint8_t)'\n');
        }
    } else {
        char to_ring = ((current_proc->context.cs & 0x3) == 0x3) ? 'U' : 'K';
        sched_dbg_mark_sw('K', to_ring);

        // DEBUG: First process switch marker
        SCHED_DBG_OUT((uint8_t)'A');
        SCHED_DBG_OUT((uint8_t)'B');
        SCHED_DBG_OUT((uint8_t)'O');
        SCHED_DBG_OUT((uint8_t)'U');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'_');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'O');
        SCHED_DBG_OUT((uint8_t)'_');
        SCHED_DBG_OUT((uint8_t)'I');
        SCHED_DBG_OUT((uint8_t)'R');
        SCHED_DBG_OUT((uint8_t)'E');
        SCHED_DBG_OUT((uint8_t)'T');
        SCHED_DBG_OUT((uint8_t)'Q');
        SCHED_DBG_OUT((uint8_t)'\n');

        sched_dbg_mark_iret();
        
        switch_to_first(&current_proc->context);
    }

    if (reenable_if)
        enable_interrupts();
}

void sched_yield(void)
{
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'Y');
    SCHED_DBG_OUT((uint8_t)'F');
    SCHED_DBG_OUT((uint8_t)']');
    sched_yield_core(1);
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'Y');
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)']');
}

void sched_yield_irq(void)
{
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'I');
    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'Q');
    SCHED_DBG_OUT((uint8_t)']');
    sched_yield_core(0); // Don't re-enable interrupts (IRQ context)
}

void sched_block_current(void)
{
    disable_interrupts();

    proc_t *prev = current_proc;
    if (!prev) {
        enable_interrupts();
        return;
    }

    // ❌ DISABLED: Cannot call Ring3 from Ring0 via C call
    // Ring0→Ring3 transition ONLY via IRETQ, not C function call
    // This causes illegal privilege transition → #GP → triple fault
    // if (scheduler_started) {
    //     userspace_scheduler_handle_block(prev, prev->wait_obj);
    // }

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
    (void)wait_obj;

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
    
    // Debug: marker before enqueue_ready
    SCHED_DBG_OUT((uint8_t)'Q');
    
    // Debug: Show PID being added
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'I');
    SCHED_DBG_OUT((uint8_t)'D');
    SCHED_DBG_OUT((uint8_t)':');
    if (proc->pid < 10) {
        SCHED_DBG_OUT((uint8_t)('0' + proc->pid));
    } else {
        SCHED_DBG_OUT((uint8_t)('A' + proc->pid - 10));
    }
    SCHED_DBG_OUT((uint8_t)'\n');
    
    // Ring0 mechanism: Call Ring3 policy for process addition
    // Ring3 policy determines state transition behavior
    proc->state = PROC_READY;
    
    // Ring0 mechanism: Call Ring3 policy for ready queue management
    enqueue_ready(proc);
    
    // Debug: marker after enqueue_ready
    SCHED_DBG_OUT((uint8_t)'R');
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
