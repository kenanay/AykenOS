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
#include "sched.h"
#include "sched_mailbox.h"

#ifndef AYKEN_GATE45_PROOF
#define AYKEN_GATE45_PROOF 0
#endif

#ifndef AYKEN_C2_STRICT_MARKERS
#define AYKEN_C2_STRICT_MARKERS 0
#endif

#ifndef AYKEN_PHASE11_MAILBOX_CAPABILITY_ENFORCE
#define AYKEN_PHASE11_MAILBOX_CAPABILITY_ENFORCE 0
#endif

// MVP-0 self-test state (kept separate from per-process runtime mailbox path).
static ayken_sched_mailbox_t g_selftest_mb __attribute__((aligned(64)));
static uint64_t g_selftest_last_epoch = 0;
static volatile uint8_t g_gate4_epoch1_pending = 0;

static void mb_reset(void) {
    g_selftest_mb.magic = AYKEN_SCHED_MB_MAGIC;
    g_selftest_mb.version = AYKEN_SCHED_MB_VERSION;
    g_selftest_mb.kind = AYKEN_SCHED_HINT_NONE;
    g_selftest_mb.epoch = 0;
    g_selftest_mb.proposer_pid = 0;
    g_selftest_mb.candidate_pid = 0;
    g_selftest_mb.flags = 0;
    g_selftest_mb.status = AYKEN_SCHED_STATUS_EMPTY;
    g_selftest_mb.reject_reason = AYKEN_SCHED_REJECT_NONE;
}

void sched_mailbox_init(void) {
    mb_reset();
    g_selftest_last_epoch = 0;
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    g_gate4_epoch1_pending = 1;
#else
    g_gate4_epoch1_pending = 0;
#endif
}

int sched_mailbox_gate4_epoch1_pending(void)
{
    return g_gate4_epoch1_pending ? 1 : 0;
}

static ayken_sched_mailbox_t* sched_mailbox_get_selftest(void) {
    return &g_selftest_mb;
}

static int reject(ayken_sched_mailbox_t* mb, ayken_sched_reject_reason_t why) {
    mb->status = AYKEN_SCHED_STATUS_REJECT;
    mb->reject_reason = (uint32_t)why;
    return -((int)why);
}

static int sched_mailbox_validate_capability_envelope(
    const ayken_sched_mailbox_t* mb,
    uint32_t* reject_reason
) {
    if (!mb || !reject_reason) {
        return -1;
    }

#if AYKEN_PHASE11_MAILBOX_CAPABILITY_ENFORCE
    if ((mb->flags & AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED) == 0u) {
        *reject_reason = REJ_CAP_MISSING;
        return -1;
    }
#else
    /*
     * Backward-compatible default:
     * enforce capability envelope only when explicitly requested by Ring3.
     */
    if ((mb->flags & AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED) == 0u) {
        return 0;
    }
#endif

    if ((mb->flags & AYKEN_SCHED_MB_FLAG_SIG_VALID) == 0u) {
        *reject_reason = REJ_BAD_SIG;
        return -1;
    }

    if ((mb->flags & AYKEN_SCHED_MB_FLAG_CAP_PRESENT) == 0u) {
        *reject_reason = REJ_CAP_MISSING;
        return -1;
    }

    if ((mb->flags & AYKEN_SCHED_MB_FLAG_BUDGET_OK) == 0u ||
        mb->reserved > AYKEN_SCHED_MB_CAP_BUDGET_MAX) {
        *reject_reason = REJ_BUDGET_EXCEEDED;
        return -1;
    }

    return 0;
}

static int sched_mailbox_validate_candidate(ayken_sched_mailbox_t* mb, proc_t** out_proc) {
    if (!mb || !out_proc) return -1;
    *out_proc = NULL;

    if (mb->magic != AYKEN_SCHED_MB_MAGIC) return reject(mb, AYKEN_SCHED_REJECT_BAD_MAGIC);
    if (mb->version != AYKEN_SCHED_MB_VERSION) return reject(mb, AYKEN_SCHED_REJECT_BAD_VERSION);
    if (mb->kind != AYKEN_SCHED_HINT_CANDIDATE) return reject(mb, AYKEN_SCHED_REJECT_BAD_KIND);

    uint32_t cap_reject = AYKEN_SCHED_REJECT_NONE;
    if (sched_mailbox_validate_capability_envelope(mb, &cap_reject) != 0) {
        return reject(mb, (ayken_sched_reject_reason_t)cap_reject);
    }

    // Epoch must advance deterministically
    if (mb->epoch <= g_selftest_last_epoch) return reject(mb, AYKEN_SCHED_REJECT_STALE_EPOCH);

    proc_t* p = proc_find_by_pid((int)mb->candidate_pid);
    if (!p) return reject(mb, REJ_INVALID_PID);

    // Minimal runnable definition for MVP
    if (!(p->state == PROC_READY || p->state == PROC_RUNNING)) {
        return reject(mb, AYKEN_SCHED_REJECT_NOT_RUNNABLE);
    }

    // Accept
    g_selftest_last_epoch = mb->epoch;
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

static void marker_accept(uint32_t owner, uint64_t epoch, uint32_t cand, const char *site) {
#if AYKEN_C2_STRICT_MARKERS
    dbg_print("[[AYKEN_SCHED_MB_ACCEPT]] owner=");
    dbg_print_u32(owner);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    dbg_print(" cand=");
    dbg_print_u32(cand);
    dbg_print(" site=");
    dbg_print(site ? site : "IRQ");
    outb(0xE9, '\n');
#else
    (void)owner;
    (void)site;
    dbg_print("[[AYKEN_SCHED_MB_ACCEPT]] pid=");
    dbg_print_u64((uint64_t)cand);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    outb(0xE9, '\n');
#endif
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

#define MB_VALIDATE_REJECT_TORN_READ 90u
#define MB_VALIDATE_REJECT_OWNER_MISMATCH 91u
#define MB_VALIDATE_REJECT_OWNER_TARGET_MISMATCH 92u

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)

static void marker_validate_enter(proc_t *proc, uint64_t epoch, uint32_t pid)
{
    dbg_print("P10_MB_VALIDATE_ENTER pid=");
    dbg_print_u32(proc ? (uint32_t)proc->pid : 0u);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    dbg_print(" cand=");
    dbg_print_u32(pid);
    outb(0xE9, '\n');
}

static void marker_validate_result(proc_t *proc, uint64_t epoch, uint32_t pid, uint32_t accept, uint32_t reason)
{
    dbg_print("P10_MB_VALIDATE_RESULT pid=");
    dbg_print_u32(proc ? (uint32_t)proc->pid : 0u);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    dbg_print(" cand=");
    dbg_print_u32(pid);
    dbg_print(" accept=");
    dbg_print_u32(accept);
    dbg_print(" reason=");
    dbg_print_u32(reason);
    outb(0xE9, '\n');
}

static void marker_ring3_publish(uint32_t pid, uint64_t epoch)
{
    dbg_print("[[AYKEN_RING3_PUBLISH]] pid=");
    dbg_print_u32(pid);
    dbg_print(" epoch=");
    dbg_print_u64(epoch);
    outb(0xE9, '\n');
}
#endif

void sched_mailbox_selftest(void) {
    ayken_sched_mailbox_t* mb = sched_mailbox_get_selftest();
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
    if (rc == 0 && out) marker_accept((uint32_t)out->pid, mb->epoch, (uint32_t)out->pid, "IRQ");
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
    ayken_sched_mailbox_t original = *mb;

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

    // Do not poison runtime mailbox state used by Phase10 scheduler path.
    *mb = original;
}

// MVP-1: Ring3 mailbox validation (called from timer tick)
// Validates Ring3-written mailbox data with double-read atomicity check
// Emits standardized markers for CI gate validation
int sched_mailbox_validate_ring3(proc_t *proc) {
    int result = -1;

    sched_perf_note_mailbox_validate_enter();
    if (!proc || !proc->mailbox_pa) {
        goto out;
    }
    if (proc->type != PROC_TYPE_USER) {
        goto out;
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
        goto out;
    }

    // Double-read for atomicity (detect torn writes from Ring3)
    uint64_t e1 = mb->epoch;
    uint32_t pid = mb->candidate_pid;
    uint64_t e2 = mb->epoch;
    uint32_t reject_reason = AYKEN_SCHED_REJECT_NONE;

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    marker_validate_enter(proc, e1, pid);
#endif

#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4 isolated proof uses strict ABI checks to ensure ACCEPT cannot be
    // produced by partial/legacy mailbox layouts.
    if (mb->magic != AYKEN_SCHED_MB_MAGIC) {
        reject_reason = AYKEN_SCHED_REJECT_BAD_MAGIC;
        goto reject;
    }
    if (mb->version != AYKEN_SCHED_MB_VERSION) {
        reject_reason = AYKEN_SCHED_REJECT_BAD_VERSION;
        goto reject;
    }
    if (mb->kind != AYKEN_SCHED_HINT_CANDIDATE) {
        reject_reason = AYKEN_SCHED_REJECT_BAD_KIND;
        goto reject;
    }
#endif

    if (sched_mailbox_validate_capability_envelope(mb, &reject_reason) != 0) {
        goto reject;
    }

    // Check 1: Torn read detection
    if (e1 != e2) {
        reject_reason = MB_VALIDATE_REJECT_TORN_READ;
        goto reject;
    }

    // Check 2: Epoch monotonicity (must advance)
    // Epoch 0 means Ring3 has not published a valid hint yet.
    // Keep this silent to avoid polluting monotonic gate evidence.
    if (e1 == 0) {
        reject_reason = AYKEN_SCHED_REJECT_STALE_EPOCH;
        goto reject;
    }

    if (e1 <= proc->mailbox_last_epoch) {
        reject_reason = AYKEN_SCHED_REJECT_STALE_EPOCH;
        goto reject;
    }

    // Check 3: PID validity (basic sanity check)
    if (pid == 0 || pid > 1000) {
        reject_reason = REJ_INVALID_PID;
        goto reject;
    }

    proc_t *cand = proc_find_by_pid((int)pid);
    if (!cand) {
        reject_reason = REJ_INVALID_PID;
        goto reject;
    }
    if (!(cand->state == PROC_READY || cand->state == PROC_RUNNING)) {
        reject_reason = AYKEN_SCHED_REJECT_NOT_RUNNABLE;
        goto reject;
    }

#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4 baseline: self-target proposal only.
    // Gate-4.5 proof: proposer must stay owner, candidate may differ (cross-target).
    if (mb->proposer_pid != (uint32_t)proc->pid) {
        reject_reason = MB_VALIDATE_REJECT_OWNER_MISMATCH;
        goto reject;
    }
#if AYKEN_GATE45_PROOF
    if (cand->type != PROC_TYPE_USER) {
        reject_reason = MB_VALIDATE_REJECT_OWNER_TARGET_MISMATCH;
        goto reject;
    }
#else
    if (pid != (uint32_t)proc->pid) {
        reject_reason = MB_VALIDATE_REJECT_OWNER_MISMATCH;
        goto reject;
    }
    if (cand != proc) {
        reject_reason = MB_VALIDATE_REJECT_OWNER_TARGET_MISMATCH;
        goto reject;
    }
#endif
    if (!(proc->state == PROC_READY || proc->state == PROC_RUNNING)) {
        reject_reason = AYKEN_SCHED_REJECT_NOT_RUNNABLE;
        goto reject;
    }
    if (!proc->gate4_publish_emitted && e1 == 1) {
        proc->gate4_publish_emitted = 1;
        marker_ring3_publish((uint32_t)proc->pid, e1);
    }
#endif

    // ACCEPT: Update last epoch and emit marker
#if AYKEN_GATE45_PROOF
    // Gate-4.5: leave epoch consume to scheduler decision path so decision->switch
    // proof can consume the first accepted epoch deterministically.
#else
    proc->mailbox_last_epoch = e1;
#endif
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    if (e1 == 1) {
        g_gate4_epoch1_pending = 0;
    }
    marker_validate_result(proc, e1, pid, 1u, AYKEN_SCHED_REJECT_NONE);
#endif
#if AYKEN_GATE45_PROOF
    // Gate-4.5 proof expects a single owner ACCEPT(epoch=1) marker even if
    // timer validation sees the same epoch repeatedly before scheduler consume.
    if (e1 == 1) {
        if (!proc->gate4_accept_epoch1_emitted) {
            proc->gate4_accept_epoch1_emitted = 1;
            marker_accept((uint32_t)proc->pid, e1, pid, "IRQ");
        }
    } else {
        marker_accept((uint32_t)proc->pid, e1, pid, "IRQ");
    }
#else
    marker_accept((uint32_t)proc->pid, e1, pid, "IRQ");
#endif
    result = 0;
    goto out;

reject:
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    marker_validate_result(proc, e1, pid, 0u, reject_reason);
#else
    (void)reject_reason;
#endif
out:
    sched_perf_note_mailbox_validate_exit();
    return result;
}
