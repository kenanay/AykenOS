// kernel/sched/sched.c
// Ring0 Scheduler Mechanism - mailbox-first scheduling path
//
// Ring0 owns execution mechanics (context switch, CR3/TSS updates).
// Ring3 owns scheduling decisions and publishes mailbox proposals.
//
// Phase10-C fail-closed rules in this file:
// - Block path never "keeps running" the blocked process.
// - Cold-start path never uses legacy ready-head selection when fallback is disabled.
// - Legacy ready-queue fallback is compile-time gated by AYKEN_SCHED_FALLBACK.

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

#ifndef AYKEN_GATE45_PROOF
#define AYKEN_GATE45_PROOF 0
#endif

#if AYKEN_GATE45_PROOF
#ifndef AYKEN_GATE45_TARGET_PID
#define AYKEN_GATE45_TARGET_PID 3u
#endif
#endif

#if AYKEN_DEBUG_SCHED
#define SCHED_DBG_OUT(ch) outb(0xE9, (uint8_t)(ch))
#else
#define SCHED_DBG_OUT(ch) do { (void)(ch); } while (0)
#endif

static void sched_emit_marker(const char *text)
{
    if (!text) {
        return;
    }
    while (*text) {
        outb(0xE9, (uint8_t)*text++);
    }
}

// Ring0 mechanism state - only for context switching
static proc_t *ready_head = NULL;
static proc_t *ready_tail = NULL;
static proc_t *blocked_head = NULL;
static proc_t *sched_owner_cached = NULL;

static int sched_list_contains(proc_t *head, const proc_t *target)
{
    if (!target) {
        return 0;
    }
    for (proc_t *iter = head; iter; iter = iter->next) {
        if (iter == target) {
            return 1;
        }
    }
    return 0;
}

static int is_in_blocked_queue(const proc_t *p)
{
    return sched_list_contains(blocked_head, p);
}

static void sched_emit_u64_dec(uint64_t v)
{
    char buf[32];
    int i = 0;
    if (v == 0) {
        outb(0xE9, (uint8_t)'0');
        return;
    }
    while (v > 0 && i < (int)sizeof(buf)) {
        buf[i++] = (char)('0' + (v % 10));
        v /= 10;
    }
    while (i > 0) {
        outb(0xE9, (uint8_t)buf[--i]);
    }
}

// "valid" mirrors mailbox slot validity: 1 when decision is observed, 0 after consume/apply.
static void sched_emit_phase10c_decision(
    const char *token,
    uint64_t id,
    uint32_t pid,
    uint32_t valid,
    uint32_t src_pid)
{
    sched_emit_marker(token);
    sched_emit_marker(" id=");
    sched_emit_u64_dec(id);
    sched_emit_marker(" pid=");
    sched_emit_u64_dec((uint64_t)pid);
    sched_emit_marker(" valid=");
    sched_emit_u64_dec((uint64_t)valid);
    sched_emit_marker(" src=");
    sched_emit_u64_dec((uint64_t)src_pid);
    sched_emit_marker("\n");
}

#if AYKEN_GATE45_PROOF
static void sched_emit_gate45_arbiter_decision(uint32_t from_pid, uint32_t to_pid, uint64_t epoch)
{
    sched_emit_marker("[[AYKEN_SCHED_ARBITER_DECISION]] from=");
    sched_emit_u64_dec((uint64_t)from_pid);
    sched_emit_marker(" to=");
    sched_emit_u64_dec((uint64_t)to_pid);
    sched_emit_marker(" epoch=");
    sched_emit_u64_dec(epoch);
    sched_emit_marker("\n");
}

static void sched_emit_gate45_ctx_switch(uint32_t from_pid, uint32_t to_pid)
{
    sched_emit_marker("[[AYKEN_CTX_SWITCH]] from=");
    sched_emit_u64_dec((uint64_t)from_pid);
    sched_emit_marker(" to=");
    sched_emit_u64_dec((uint64_t)to_pid);
    sched_emit_marker("\n");
}

static void sched_emit_gate45_chain_once(
    proc_t *prev,
    proc_t *next,
    uint64_t decision_id,
    int used_mailbox)
{
    static uint8_t gate45_chain_emitted = 0;
    if (gate45_chain_emitted) {
        return;
    }
    if (!used_mailbox || decision_id == 0 || !prev || !next || prev == next) {
        return;
    }
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4.5 ordering contract: emit only after timer-path epoch=1 ACCEPT.
    if (sched_mailbox_gate4_epoch1_pending()) {
        return;
    }
#endif
    gate45_chain_emitted = 1;
    sched_emit_gate45_arbiter_decision(
        (uint32_t)prev->pid, (uint32_t)next->pid, decision_id);
    sched_emit_gate45_ctx_switch((uint32_t)prev->pid, (uint32_t)next->pid);
}
#else
static inline void sched_emit_gate45_chain_once(
    proc_t *prev,
    proc_t *next,
    uint64_t decision_id,
    int used_mailbox)
{
    (void)prev;
    (void)next;
    (void)decision_id;
    (void)used_mailbox;
}
#endif

static void sched_emit_irq_decision(proc_t *prev, proc_t *next, int used_mailbox)
{
    sched_emit_marker("P10_IRQ_SCHED_DECISION prev=");
    sched_emit_u64_dec(prev ? (uint64_t)(uint32_t)prev->pid : 0);
    sched_emit_marker(" next=");
    sched_emit_u64_dec(next ? (uint64_t)(uint32_t)next->pid : 0);
    sched_emit_marker(" used_mailbox=");
    sched_emit_u64_dec((uint64_t)(used_mailbox ? 1 : 0));
    sched_emit_marker(" keep_running=");
    sched_emit_u64_dec((uint64_t)((prev && next == prev) ? 1 : 0));
    sched_emit_marker("\n");
}

static ayken_sched_mailbox_t *sched_mailbox_view_for_owner(proc_t *owner)
{
    if (!owner || !owner->mailbox_pa) {
        return NULL;
    }
    uint64_t active_cr3 = 0;
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (owner->context.cr3 & AYKEN_PTE_ADDR_MASK)) {
        return (ayken_sched_mailbox_t *)(uintptr_t)SCHED_MAILBOX_VA;
    }
    return (ayken_sched_mailbox_t *)paging_phys_to_virt(owner->mailbox_pa);
}

#if AYKEN_GATE45_PROOF
static void sched_gate45_arm_cross_target_once(proc_t *owner)
{
    static uint8_t armed = 0;
    if (armed || !owner || owner->pid != (int)AYKEN_SCHED_OWNER_PID) {
        return;
    }
    ayken_sched_mailbox_t *mb = sched_mailbox_view_for_owner(owner);
    if (!mb) {
        return;
    }
    mb->candidate_pid = AYKEN_GATE45_TARGET_PID;
    armed = 1;
}
#endif

static int sched_mailbox_extract_candidate(proc_t *owner, uint64_t *out_epoch, uint32_t *out_pid)
{
    if (!owner || !out_epoch || !out_pid || !owner->mailbox_pa) {
        return 0;
    }
    ayken_sched_mailbox_t *mb = sched_mailbox_view_for_owner(owner);
    if (!mb) {
        return 0;
    }
    if (mb->magic != AYKEN_SCHED_MB_MAGIC ||
        mb->version != AYKEN_SCHED_MB_VERSION ||
        mb->kind != AYKEN_SCHED_HINT_CANDIDATE) {
        return 0;
    }
    if (mb->epoch == 0 || mb->epoch <= owner->mailbox_last_epoch) {
        return 0;
    }
    if (mb->candidate_pid == 0) {
        return 0;
    }
    *out_epoch = mb->epoch;
    *out_pid = mb->candidate_pid;
    return 1;
}

typedef enum {
    SCHED_DECISION_SITE_START = 0,
    SCHED_DECISION_SITE_YIELD = 1,
    SCHED_DECISION_SITE_BLOCK = 2,
} sched_decision_site_t;

static int sched_is_owner(const proc_t *p)
{
    return p && p->pid == (int)AYKEN_SCHED_OWNER_PID;
}

static proc_t *sched_owner_proc(proc_t *prev, sched_decision_site_t site)
{
    if (prev && sched_is_owner(prev)) {
        sched_owner_cached = prev;
        return prev;
    }

    if (sched_owner_cached && sched_is_owner(sched_owner_cached)) {
        return sched_owner_cached;
    }

    (void)site;
    proc_t *owner = proc_find_by_pid((int)AYKEN_SCHED_OWNER_PID);
    if (owner && sched_is_owner(owner)) {
        sched_owner_cached = owner;
        return owner;
    }

    return NULL;
}

static int sched_mailbox_has_any_candidate(proc_t *p)
{
    ayken_sched_mailbox_t *mb = sched_mailbox_view_for_owner(p);
    if (!mb) {
        return 0;
    }
    if (mb->magic != AYKEN_SCHED_MB_MAGIC || mb->version != AYKEN_SCHED_MB_VERSION) {
        return 0;
    }
    if (mb->kind != AYKEN_SCHED_HINT_CANDIDATE || mb->candidate_pid == 0) {
        return 0;
    }
    // Ignore kernel-seeded initial payload (epoch=1, candidate=self).
    if (mb->epoch == 1 && mb->candidate_pid == (uint32_t)p->pid) {
        return 0;
    }
    return 1;
}

#if AYKEN_DEBUG_SCHED
static __attribute__((noreturn)) void sched_debug_assert_fail(char code);
#endif

#if AYKEN_SCHED_BOOTSTRAP_POLICY || AYKEN_SCHED_FALLBACK
static proc_t *sched_select_next_ready_head_fallback(void);
#endif

static proc_t *sched_select_next_mailbox(
    proc_t *prev,
    uint64_t *decision_id,
    uint32_t *decision_pid,
    uint32_t *decision_src_pid,
    int *used_mailbox,
    int allow_keep_running,
    sched_decision_site_t site)
{
    if (decision_id) {
        *decision_id = 0;
    }
    if (decision_pid) {
        *decision_pid = 0;
    }
    if (decision_src_pid) {
        *decision_src_pid = 0;
    }
    if (used_mailbox) {
        *used_mailbox = 0;
    }

#if AYKEN_GATE45_PROOF
    // Gate-4.5 effect proof is single handoff (owner -> target). After handoff,
    // keep non-owner running and do not dereference owner mailbox under foreign CR3.
    if (site == SCHED_DECISION_SITE_YIELD && allow_keep_running &&
        prev && prev->type == PROC_TYPE_USER && !sched_is_owner(prev)) {
        return prev;
    }
#endif

    proc_t *owner = sched_owner_proc(prev, site);
    if (!owner) {
        sched_emit_marker("P10_MAILBOX_OWNER_MISSING_FATAL\n");
        return NULL;
    }
    if (!(owner->state == PROC_READY || owner->state == PROC_RUNNING)) {
        sched_emit_marker("P10_MAILBOX_OWNER_NOT_READY_FATAL\n");
        return NULL;
    }

    // Non-owner fresh decision attempt is a protocol violation.
    if (prev && prev->type == PROC_TYPE_USER && !sched_is_owner(prev) &&
        sched_mailbox_has_any_candidate(prev)) {
        sched_emit_marker("P10_MAILBOX_OWNER_MISMATCH\n");
        if (site != SCHED_DECISION_SITE_YIELD) {
            return NULL;
        }
    }

    // Single-authority path: only owner mailbox is consumed.
    {
        uint64_t epoch = 0;
        uint32_t pid = 0;
        if (sched_mailbox_extract_candidate(owner, &epoch, &pid)) {
            int consume_epoch = 1;
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
            // Gate-4 proof requires timer-path ACCEPT for epoch=1 before scheduler
            // consumes that epoch in any decision site.
            if (epoch == 1 && sched_mailbox_gate4_epoch1_pending()) {
                consume_epoch = 0;
            }
#endif
#if AYKEN_GATE45_PROOF
            // Gate-4.5: do not consume epoch=1 on self-keep-running path.
            // This keeps epoch=1 available until Ring3 flips candidate to cross-target.
            if (epoch == 1 &&
                prev && prev->type == PROC_TYPE_USER &&
                pid == (uint32_t)prev->pid) {
                consume_epoch = 0;
            }
#endif
            if (prev && prev->type == PROC_TYPE_USER &&
                prev->state == PROC_RUNNING &&
                pid == (uint32_t)prev->pid) {
                if (consume_epoch) {
                    owner->mailbox_last_epoch = epoch;
                }
                if (decision_id) {
                    *decision_id = epoch;
                }
                if (decision_pid) {
                    *decision_pid = pid;
                }
                if (decision_src_pid) {
                    *decision_src_pid = (uint32_t)owner->pid;
                }
                if (used_mailbox) {
                    *used_mailbox = 1;
                }
                return prev;
            }
            proc_t *cand = proc_find_by_pid((int)pid);
            if (cand && (cand->state == PROC_READY || cand->state == PROC_RUNNING)) {
                if (consume_epoch) {
                    owner->mailbox_last_epoch = epoch;
                }
                if (decision_id) {
                    *decision_id = epoch;
                }
                if (decision_pid) {
                    *decision_pid = pid;
                }
                if (decision_src_pid) {
                    *decision_src_pid = (uint32_t)owner->pid;
                }
                if (used_mailbox) {
                    *used_mailbox = 1;
                }
                remove_from_ready_queue(cand);
                return cand;
            }
        }
    }

    // Yield-only safety invariant: without fresh decision, keep current Ring3 context.
    if (allow_keep_running && prev && prev->type == PROC_TYPE_USER) {
        if (prev->state != PROC_RUNNING) {
            sched_emit_marker("P10_MAILBOX_MISS_KEEP_RUNNING_INVALID_STATE\n");
            return NULL;
        }
#if AYKEN_DEBUG_SCHED
        if (ready_head == prev || ready_tail == prev) {
            sched_debug_assert_fail('Q');
        }
#endif
        sched_emit_marker("P10_MAILBOX_MISS_KEEP_RUNNING\n");
        return prev;
    }

    // Transitional fallback is compile-time gated; default constitutional mode is fail-closed.
#if AYKEN_SCHED_FALLBACK
    sched_emit_marker("P10_SCHED_FALLBACK\n");
    sched_emit_marker("P10_READY_HEAD_FALLBACK\n");
    return sched_select_next_ready_head_fallback();
#else
    if (site == SCHED_DECISION_SITE_BLOCK) {
        sched_emit_marker("P10_MAILBOX_MISS_BLOCK_FATAL\n");
    } else if (site == SCHED_DECISION_SITE_START) {
        sched_emit_marker("P10_MAILBOX_MISS_BOOTSTRAP_FATAL\n");
    } else {
        sched_emit_marker("P10_MAILBOX_MISS_YIELD_NULL\n");
    }
    return NULL;
#endif
}

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

proc_t *current_proc = NULL;
static volatile uint32_t need_resched = 0;
// One-shot by design: proves mailbox decision/apply path exists without per-tick log churn.
// NOTE: current path is single-CPU validation; SMP enablement requires atomic/lock.
static uint8_t phase10c_decision_markers_emitted = 0;
// One-shot IRQ decision marker for strict-mode preemption diagnosis.
// NOTE: current path is single-CPU validation; SMP enablement requires atomic/lock.
static uint8_t phase10_irq_decision_marker_emitted = 0;
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

#if AYKEN_DEBUG_SCHED
static int sched_irqs_enabled(void)
{
    uint64_t rflags = 0;
    __asm__ volatile("pushfq; popq %0" : "=r"(rflags));
    return (rflags & (1ULL << 9)) != 0;
}
#endif

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
#if AYKEN_DEBUG_SCHED
    // REQUIRES: interrupts disabled by caller.
    if (sched_irqs_enabled()) {
        sched_debug_assert_fail('I');
    }
#endif

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

#if AYKEN_SCHED_BOOTSTRAP_POLICY || AYKEN_SCHED_FALLBACK
// Transitional/internal helper: deterministic ready-head fallback selection.
static proc_t *sched_select_next_ready_head_fallback(void)
{
    // DEBUG: Scheduler selection entry marker
    SCHED_DBG_OUT((uint8_t)'[');
    SCHED_DBG_OUT((uint8_t)'S');
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'L');
    SCHED_DBG_OUT((uint8_t)']');
    
    // Internal fallback selector for transitional modes only.
    proc_t *selected = ready_head;

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
#endif

// Ring0 mechanism: Call Ring3 scheduler policy for process enqueueing
void enqueue_ready(proc_t *p)
{
    if (!p) return;
    
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
    if (!p) {
        return;
    }
    if (is_in_blocked_queue(p)) {
        return;
    }
    p->next = blocked_head;
    blocked_head = p;
}

static void remove_from_blocked(proc_t *p)
{
    if (!p || !blocked_head) {
        return;
    }
    if (blocked_head == p) {
        blocked_head = p->next;
        p->next = NULL;
        return;
    }
    proc_t *prev = blocked_head;
    while (prev->next && prev->next != p) {
        prev = prev->next;
    }
    if (prev->next == p) {
        prev->next = p->next;
        p->next = NULL;
    }
}

void sched_init(void)
{
    // Ring0 mechanism: Initialize only mechanism state
    // All policy initialization handled by Ring3
    ready_head = ready_tail = NULL;
    blocked_head = NULL;
    sched_owner_cached = NULL;
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

    uint64_t decision_id = 0;
    uint32_t decision_pid = 0;
    uint32_t decision_src_pid = 0;
    int used_mailbox = 0;

#if AYKEN_SCHED_BOOTSTRAP_POLICY
    // Transitional bootstrap mode: explicit, auditable policy bridge until
    // first mailbox-owner protocol is fully externalized.
    sched_emit_marker("P10_BOOTSTRAP_POLICY_ACTIVE\n");
    proc_t *first = sched_select_next_ready_head_fallback();
#else
    // Strict mode: cold-start must also be mailbox-driven.
    proc_t *first = sched_select_next_mailbox(
        NULL,
        &decision_id,
        &decision_pid,
        &decision_src_pid,
        &used_mailbox,
        0,
        SCHED_DECISION_SITE_START);
#endif
    if (!first) {
        fb_print("[PANIC] scheduler bootstrap has no runnable decision\n");
        for (;;) __asm__ volatile("cli; hlt");
    }
    SCHED_DBG_OUT((uint8_t)'F');

    // Ring0 mechanism: Set up initial process context (mechanism only)
    current_proc = first;
    current_proc->state = PROC_RUNNING;
    if (sched_is_owner(current_proc)) {
        sched_owner_cached = current_proc;
    }
    
    // MVP-0: Scheduler bridge self-test (emits markers for gate validation)
    // Called here after current_proc is set but before switch_to_first
    // Compile-out in release: self-test is validation-only
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
#if !defined(AYKEN_MB_SELFTEST) || (AYKEN_MB_SELFTEST == 1)
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
    
    // Gate-2: Context switch validation marker (validation-only)
    // Emitted before first context switch (switch_to_first)
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    {
        static int g_ctx_switch_marker_emitted_first = 0;
        if (!g_ctx_switch_marker_emitted_first) {
            g_ctx_switch_marker_emitted_first = 1;
            const char *marker = "[[AYKEN_CTX_SWITCH]]\n";
            while (*marker) {
                __asm__ volatile("outb %0, %1" : : "a"((uint8_t)*marker), "Nd"((uint16_t)0xE9));
                marker++;
            }
        }
    }
#endif
    
    // CRITICAL: Call switch_to_first with interrupts disabled
    // Interrupts will be enabled by the first process's RFLAGS (IF=1)
    // This prevents timer interrupts from firing before we have a proper context
    sched_emit_marker("P10_SCHED_DISPATCH\n");
    if (used_mailbox && !phase10c_decision_markers_emitted) {
        phase10c_decision_markers_emitted = 1;
        sched_emit_phase10c_decision(
            "P10_MAILBOX_DECISION", decision_id, decision_pid, 1, decision_src_pid);
        sched_emit_phase10c_decision(
            "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
    }
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
#if AYKEN_DEBUG_SCHED
    if (prev && prev->state == PROC_RUNNING &&
        (ready_head == prev || ready_tail == prev)) {
        sched_debug_assert_fail('q');
    }
#endif
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
    
    // Phase10-C path: consume mailbox decision; fall back path is explicitly marked.
    uint64_t decision_id = 0;
    uint32_t decision_pid = 0;
    uint32_t decision_src_pid = 0;
    int used_mailbox = 0;
    proc_t *next = sched_select_next_mailbox(
        prev,
        &decision_id,
        &decision_pid,
        &decision_src_pid,
        &used_mailbox,
        1,
        SCHED_DECISION_SITE_YIELD);
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

    if (!reenable_if && !phase10_irq_decision_marker_emitted) {
        phase10_irq_decision_marker_emitted = 1;
        sched_emit_irq_decision(prev, next, used_mailbox);
    }

    if (!next) {
        if (!reenable_if) {
            // No switch from IRQ path: do not leave stale snapshot state armed.
            sched_irq_user_ctx_saved = 0;
        }
        SCHED_DBG_OUT((uint8_t)'X');
        SCHED_DBG_OUT((uint8_t)'\n');
        if (reenable_if)
            enable_interrupts();
        return;
    }

    // If policy returns the currently running Ring3 process, keep running in place.
    if (prev && next == prev) {
        if (used_mailbox && !phase10c_decision_markers_emitted) {
            phase10c_decision_markers_emitted = 1;
            sched_emit_phase10c_decision(
                "P10_MAILBOX_DECISION", decision_id, decision_pid, 1, decision_src_pid);
            sched_emit_phase10c_decision(
                "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
        }
        if (!reenable_if) {
            // No context_switch() call will consume this in IRQ no-op path.
            sched_irq_user_ctx_saved = 0;
        }
        if (reenable_if)
            enable_interrupts();
        return;
    }

    int emit_phase10c_markers = 0;
    if (used_mailbox && !phase10c_decision_markers_emitted) {
        phase10c_decision_markers_emitted = 1;
        emit_phase10c_markers = 1;
        sched_emit_phase10c_decision(
            "P10_MAILBOX_DECISION", decision_id, decision_pid, 1, decision_src_pid);
    }

#if AYKEN_DEBUG_SCHED
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

    if (emit_phase10c_markers) {
        sched_emit_phase10c_decision(
            "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
    }

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
        
        // Gate-2: Context switch validation marker (validation-only)
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
        {
            static int g_ctx_switch_marker_emitted = 0;
            if (!g_ctx_switch_marker_emitted) {
                g_ctx_switch_marker_emitted = 1;
                // Local debugcon writer (no export needed)
                const char *marker = "[[AYKEN_CTX_SWITCH]]\n";
                while (*marker) {
                    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)*marker), "Nd"((uint16_t)0xE9));
                    marker++;
                }
            }
        }
#endif
        
        #if AYKEN_GATE45_PROOF
        if (used_mailbox && decision_id == 1 &&
            current_proc && current_proc->pid == (int)AYKEN_SCHED_OWNER_PID) {
            sched_gate45_arm_cross_target_once(current_proc);
        }
        #endif
        sched_emit_gate45_chain_once(prev, current_proc, decision_id, used_mailbox);
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

    // Ring0 mechanism: state transition bookkeeping.
    remove_from_ready_queue(prev);
    prev->state = PROC_BLOCKED;
    
    // Ring0 mechanism: blocked queue bookkeeping.
    enqueue_blocked(prev);

    // Phase10-C path: consume mailbox decision; fall back path is explicitly marked.
    uint64_t decision_id = 0;
    uint32_t decision_pid = 0;
    uint32_t decision_src_pid = 0;
    int used_mailbox = 0;
    proc_t *next = sched_select_next_mailbox(
        prev,
        &decision_id,
        &decision_pid,
        &decision_src_pid,
        &used_mailbox,
        0,
        SCHED_DECISION_SITE_BLOCK);
    if (!next) {
        fb_print("[PANIC] blocked task without mailbox successor\n");
        for (;;) __asm__ volatile("cli; hlt");
    }

    int emit_phase10c_markers = 0;
    if (used_mailbox && !phase10c_decision_markers_emitted) {
        phase10c_decision_markers_emitted = 1;
        emit_phase10c_markers = 1;
        sched_emit_phase10c_decision(
            "P10_MAILBOX_DECISION", decision_id, decision_pid, 1, decision_src_pid);
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

    if (emit_phase10c_markers) {
        sched_emit_phase10c_decision(
            "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
    }
    sched_emit_gate45_chain_once(prev, current_proc, decision_id, used_mailbox);
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
    proc_t *prev = NULL;

    while (iter) {
        proc_t *next = iter->next;
        if (!wait_obj || iter->wait_obj == wait_obj) {
            if (prev) {
                prev->next = next;
            } else {
                blocked_head = next;
            }
            iter->next = NULL;
            iter->state = PROC_READY;
            iter->wait_obj = NULL;
            enqueue_ready(iter);
        } else {
            prev = iter;
        }
        iter = next;
    }
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
