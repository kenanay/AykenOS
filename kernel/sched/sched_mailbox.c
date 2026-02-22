// kernel/sched/sched_mailbox.c
// Scheduler Bridge Mailbox - Ring0 Mechanism Implementation
//
// Constitutional requirement: Ring0 mechanism only, Ring3 policy
// This file implements ONLY the Ring0 scheduler bridge mechanism.
// ALL scheduling policy decisions are made in Ring3 userspace.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Phase: 4.5 - Scheduler Bridge MVP

#include <stddef.h>
#include "../include/proc.h"
#include "../include/sched_mailbox_abi.h"
#include "../include/mm.h"
#include "../arch/x86_64/port_io.h"
#include "sched_mailbox.h"

// MVP-0: single shared instance in .bss (later: map to userspace page)
static ayken_sched_mailbox_t g_mb __attribute__((aligned(64)));
static uint64_t g_last_epoch = 0;

static void mb_reset(void) {
    g_mb.magic = AYKEN_SCHED_MB_MAGIC;
    g_mb.version = AYKEN_SCHED_MB_VERSION;
    g_mb.kind = AYKEN_SCHED_HINT_NONE;
    g_mb.epoch = 0;
    g_mb.proposer_pid = 0;
    g_mb.candidate_pid = 0;
    g_mb.flags = 0;
    g_mb.status = AYKEN_SCHED_STATUS_EMPTY;
    g_mb.reject_reason = AYKEN_SCHED_REJECT_NONE;
}

void sched_mailbox_init(void) {
    mb_reset();
    g_last_epoch = 0;
}

static ayken_sched_mailbox_t* sched_mailbox_get(void) {
    return &g_mb;
}

static int reject(ayken_sched_mailbox_t* mb, ayken_sched_reject_reason_t why) {
    mb->status = AYKEN_SCHED_STATUS_REJECT;
    mb->reject_reason = (uint32_t)why;
    return -((int)why);
}

static int sched_mailbox_validate_candidate(ayken_sched_mailbox_t* mb, proc_t** out_proc) {
    if (!mb || !out_proc) return -1;
    *out_proc = NULL;

    if (mb->magic != AYKEN_SCHED_MB_MAGIC) return reject(mb, AYKEN_SCHED_REJECT_BAD_MAGIC);
    if (mb->version != AYKEN_SCHED_MB_VERSION) return reject(mb, AYKEN_SCHED_REJECT_BAD_VERSION);
    if (mb->kind != AYKEN_SCHED_HINT_CANDIDATE) return reject(mb, AYKEN_SCHED_REJECT_BAD_KIND);

    // Epoch must advance deterministically
    if (mb->epoch <= g_last_epoch) return reject(mb, AYKEN_SCHED_REJECT_STALE_EPOCH);

    proc_t* p = proc_find_by_pid((int)mb->candidate_pid);
    if (!p) return reject(mb, AYKEN_SCHED_REJECT_BAD_PID);

    // Minimal runnable definition for MVP
    if (!(p->state == PROC_READY || p->state == PROC_RUNNING)) {
        return reject(mb, AYKEN_SCHED_REJECT_NOT_RUNNABLE);
    }

    // Accept
    g_last_epoch = mb->epoch;
    mb->status = AYKEN_SCHED_STATUS_ACCEPT;
    mb->reject_reason = AYKEN_SCHED_REJECT_NONE;
    *out_proc = p;
    return 0;
}

// Marker format MUST stay stable for grep-based gate
// Output to debugcon (port 0xE9) for CI validation
static void dbg_print(const char* s) {
    if (!s) return;
    while (*s) {
        outb(0xE9, (uint8_t)*s++);
    }
}

static void dbg_print_u64(uint64_t v) {
    char buf[32];
    int i = 0;
    if (v == 0) {
        outb(0xE9, '0');
        return;
    }
    while (v > 0 && i < 31) {
        buf[i++] = '0' + (v % 10);
        v /= 10;
    }
    while (i > 0) {
        outb(0xE9, (uint8_t)buf[--i]);
    }
}

static void dbg_print_u32(uint32_t v) {
    dbg_print_u64((uint64_t)v);
}

static void marker_accept(int pid, uint64_t epoch) {
    dbg_print("[[AYKEN_SCHED_MB_ACCEPT]] pid=");
    dbg_print_u64((uint64_t)pid);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    outb(0xE9, '\n');
}

static void marker_reject(uint32_t reason, uint64_t epoch, uint32_t pid) {
    dbg_print("[[AYKEN_SCHED_MB_REJECT]] reason=");
    dbg_print_u32(reason);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    dbg_print(" pid=");
    dbg_print_u32(pid);
    outb(0xE9, '\n');
}

void sched_mailbox_selftest(void) {
    ayken_sched_mailbox_t* mb = sched_mailbox_get();
    proc_t* out = NULL;

    // Use current_proc if available for deterministic ACCEPT
    extern proc_t *current_proc;
    uint32_t candidate = 1; // fallback

    if (current_proc && current_proc->pid > 0) {
        candidate = (uint32_t)current_proc->pid;
    }

    // CASE 1: ACCEPT (if candidate exists and runnable)
    mb->magic = AYKEN_SCHED_MB_MAGIC;
    mb->version = AYKEN_SCHED_MB_VERSION;
    mb->kind = AYKEN_SCHED_HINT_CANDIDATE;
    mb->epoch = 1;
    mb->proposer_pid = 0xBEEF; // kernel self-test marker
    mb->candidate_pid = candidate;

    int rc = sched_mailbox_validate_candidate(mb, &out);
    if (rc == 0 && out) marker_accept(out->pid, mb->epoch);
    else marker_reject(mb->reject_reason, mb->epoch, mb->candidate_pid);

    // CASE 2: STALE epoch reject
    mb->epoch = 1; // same epoch
    mb->candidate_pid = candidate;
    (void)sched_mailbox_validate_candidate(mb, &out);
    marker_reject(mb->reject_reason, mb->epoch, mb->candidate_pid);

    // CASE 3: BAD PID reject
    mb->epoch = 2;
    mb->candidate_pid = 0x7FFFFFFF;
    (void)sched_mailbox_validate_candidate(mb, &out);
    marker_reject(mb->reject_reason, mb->epoch, mb->candidate_pid);

    // Clean end state
    mb->kind = AYKEN_SCHED_HINT_NONE;
}

// MVP-2: Ring3 simulation test (validates Ring3 library behavior)
// Simulates Ring3 ayken_sched_hint() writes to mailbox
// Tests Ring0 validation with real Ring3-style writes
void sched_mailbox_test_ring3_simulation(proc_t *proc) {
    if (!proc || !proc->mailbox_pa) {
        dbg_print("[MVP-2] No mailbox for simulation test\n");
        return;
    }

    ayken_sched_mailbox_t *mb = (ayken_sched_mailbox_t *)paging_phys_to_virt(proc->mailbox_pa);
    if (!mb) return;

    dbg_print("[MVP-2] Ring3 Simulation Test Start\n");

    // Simulate Ring3 ayken_sched_hint(42)
    // This mimics userspace/libayken/sched_hint.c behavior
    uint64_t current_epoch = mb->epoch;
    uint64_t next_epoch = current_epoch + 1;
    
    // Write sequence (same as Ring3 library)
    mb->candidate_pid = 42;
    mb->epoch = next_epoch;
    
    dbg_print("[MVP-2] Simulated Ring3 write: pid=42 epoch=");
    dbg_print_u64(next_epoch);
    outb(0xE9, '\n');

    // Trigger validation (same as timer tick would)
    sched_mailbox_validate_ring3(proc);

    // Simulate invalid PID write
    current_epoch = mb->epoch;
    next_epoch = current_epoch + 1;
    mb->candidate_pid = 2147483647; // Invalid PID
    mb->epoch = next_epoch;
    
    dbg_print("[MVP-2] Simulated Ring3 write: pid=2147483647 epoch=");
    dbg_print_u64(next_epoch);
    outb(0xE9, '\n');

    sched_mailbox_validate_ring3(proc);

    dbg_print("[MVP-2] Ring3 Simulation Test Complete\n");
}

// MVP-1: Ring3 mailbox validation (called from timer tick)
// Validates Ring3-written mailbox data with double-read atomicity check
// Emits standardized markers for CI gate validation
int sched_mailbox_validate_ring3(proc_t *proc) {
    if (!proc || !proc->mailbox_pa) {
        marker_reject(4, 0, proc ? (uint32_t)proc->pid : 0); // reason=4 (no_mb)
        return -1;
    }

    /*
     * In timer IRQ context we run on the interrupted process CR3. For user
     * processes the mailbox is mapped at fixed user VA (SCHED_MAILBOX_VA).
     * Using paging_phys_to_virt() can return identity VA for low physical
     * addresses, which is not guaranteed to be mapped in user CR3 and can PF.
     */
    uint64_t active_cr3 = 0;
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    ayken_sched_mailbox_t *mb = NULL;
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (proc->context.cr3 & AYKEN_PTE_ADDR_MASK)) {
        mb = (ayken_sched_mailbox_t *)(uintptr_t)SCHED_MAILBOX_VA;
    } else {
        mb = (ayken_sched_mailbox_t *)paging_phys_to_virt(proc->mailbox_pa);
    }
    if (!mb) {
        marker_reject(4, 0, (uint32_t)proc->pid);
        return -1;
    }

    // Double-read for atomicity (detect torn writes from Ring3)
    uint64_t e1 = mb->epoch;
    uint32_t pid = mb->candidate_pid;
    uint64_t e2 = mb->epoch;

    // Check 1: Torn read detection
    if (e1 != e2) {
        marker_reject(1, e1, pid); // reason=1 (torn)
        return -1;
    }

    // Check 2: Epoch monotonicity (must advance)
    if (e1 <= proc->mailbox_last_epoch) {
        marker_reject(2, e1, pid); // reason=2 (epoch)
        return -1;
    }

    // Check 3: PID validity (basic sanity check)
    if (pid == 0 || pid > 1000) {
        marker_reject(3, e1, pid); // reason=3 (pid)
        return -1;
    }

    // ACCEPT: Update last epoch and emit marker
    proc->mailbox_last_epoch = e1;
    marker_accept((int)pid, e1);
    return 0;
}
