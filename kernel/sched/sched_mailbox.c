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
#include "../drivers/console/fb_console.h"
#include "../include/proc.h"
#include "../include/sched_mailbox_abi.h"
#include "sched_mailbox.h"

// MVP-0: single shared instance in .bss (later: map to userspace page)
static ayken_sched_mailbox_t g_mb __attribute__((aligned(64)));
static uint64_t g_last_epoch = 0;

// Helper: print uint64_t via fb_print_int
static void fb_print_u64(uint64_t v) {
    fb_print_int((int64_t)v);
}

// Helper: print uint32_t via fb_print_int
static void fb_print_u32(uint32_t v) {
    fb_print_int((int64_t)v);
}

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

ayken_sched_mailbox_t* sched_mailbox_get(void) {
    return &g_mb;
}

static int reject(ayken_sched_mailbox_t* mb, ayken_sched_reject_reason_t why) {
    mb->status = AYKEN_SCHED_STATUS_REJECT;
    mb->reject_reason = (uint32_t)why;
    return -((int)why);
}

int sched_mailbox_validate_candidate(ayken_sched_mailbox_t* mb, proc_t** out_proc) {
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
static void marker_accept(int pid, uint64_t epoch) {
    fb_print("[[AYKEN_SCHED_MB_ACCEPT]] pid=");
    fb_print_int(pid);
    fb_print(" epoch=");
    fb_print_u64(epoch);
    fb_print("\n");
}

static void marker_reject(uint32_t reason, uint64_t epoch, uint32_t pid) {
    fb_print("[[AYKEN_SCHED_MB_REJECT]] reason=");
    fb_print_u32(reason);
    fb_print(" epoch=");
    fb_print_u64(epoch);
    fb_print(" pid=");
    fb_print_u32(pid);
    fb_print("\n");
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
