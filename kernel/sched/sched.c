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
#include "../include/execution_slot.h"
#include "../arch/x86_64/cpu.h"
#include "../arch/x86_64/interrupts.h"
#include "../arch/x86_64/pic.h"
#include "../arch/x86_64/timer.h"
#include "../arch/x86_64/port_io.h"
#include "../drivers/console/fb_console.h"
#include "../include/mm.h"
#include "../include/gdt_idt.h"

#define memset __builtin_memset
#define memcpy __builtin_memcpy

// Set by Ring3 #BP proof path once user instruction marker is emitted.
extern volatile uint32_t phase10_ring3_user_code_seen;

#ifndef AYKEN_DEBUG_SCHED
#define AYKEN_DEBUG_SCHED 0
#endif

#ifndef AYKEN_GATE45_PROOF
#define AYKEN_GATE45_PROOF 0
#endif

#ifndef AYKEN_C2_STRICT_MARKERS
#define AYKEN_C2_STRICT_MARKERS 0
#endif

#ifndef AYKEN_MB_SELFTEST
#define AYKEN_MB_SELFTEST 0
#endif

#ifndef AYKEN_USER_MINIMAL_MODE_STRING
#define AYKEN_USER_MINIMAL_MODE_STRING "unknown"
#endif

#ifndef AYKEN_DETERMINISTIC_EXIT
#define AYKEN_DETERMINISTIC_EXIT 0
#endif

#ifndef AYKEN_RING3_SECOND_CANONICAL_PROBE
#define AYKEN_RING3_SECOND_CANONICAL_PROBE 0
#endif

#ifndef AYKEN_RING3_FRESH_FRAME_PROBE
#define AYKEN_RING3_FRESH_FRAME_PROBE 0
#endif

#ifndef AYKEN_RING3_LOW_FETCH_STUB
#define AYKEN_RING3_LOW_FETCH_STUB 0
#endif

#ifndef AYKEN_RING3_CANONICAL_FETCH_STUB
#define AYKEN_RING3_CANONICAL_FETCH_STUB 0
#endif

#ifndef AYKEN_RING3_SPLIT_IRETQ_PAGE
#define AYKEN_RING3_SPLIT_IRETQ_PAGE 0
#endif

#ifndef AYKEN_RING3_ALT_STAGEB_SOURCE
#define AYKEN_RING3_ALT_STAGEB_SOURCE 0
#endif

#ifndef AYKEN_RING3_FORCE_KERNEL_CR3
#define AYKEN_RING3_FORCE_KERNEL_CR3 0
#endif

#ifndef AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY
#define AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY 0
#endif

#ifndef AYKEN_RING3_STERILE_ALT_ROOT
#define AYKEN_RING3_STERILE_ALT_ROOT 0
#endif

#ifndef AYKEN_RING3_HIGH_PHYS_STERILE_ALT_ROOT
#define AYKEN_RING3_HIGH_PHYS_STERILE_ALT_ROOT 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF
#define AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_FULL_UPPER_HALF
#define AYKEN_RING3_STERILE_GRAFT_FULL_UPPER_HALF 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_LOWER_PML4_SEM
#define AYKEN_RING3_STERILE_GRAFT_LOWER_PML4_SEM 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_LOWER_PDPT_SEM
#define AYKEN_RING3_STERILE_GRAFT_LOWER_PDPT_SEM 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_PD2
#define AYKEN_RING3_STERILE_GRAFT_PD2 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_PD3
#define AYKEN_RING3_STERILE_GRAFT_PD3 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_PD3_SEM_ONLY
#define AYKEN_RING3_STERILE_GRAFT_PD3_SEM_ONLY 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_TEXT
#define AYKEN_RING3_STERILE_GRAFT_TEXT 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_CANARY
#define AYKEN_RING3_STERILE_GRAFT_CANARY 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_STACK
#define AYKEN_RING3_STERILE_GRAFT_STACK 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_STACK_TOP
#define AYKEN_RING3_STERILE_GRAFT_STACK_TOP 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_STACK_LOW
#define AYKEN_RING3_STERILE_GRAFT_STACK_LOW 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_MAILBOX
#define AYKEN_RING3_STERILE_GRAFT_MAILBOX 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_INBOX
#define AYKEN_RING3_STERILE_GRAFT_INBOX 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_PAYLOAD
#define AYKEN_RING3_STERILE_GRAFT_PAYLOAD 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_UPPER_PML4_SEM
#define AYKEN_RING3_STERILE_GRAFT_UPPER_PML4_SEM 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_UPPER_PDPT
#define AYKEN_RING3_STERILE_GRAFT_UPPER_PDPT 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_UPPER_PD
#define AYKEN_RING3_STERILE_GRAFT_UPPER_PD 0
#endif

#ifndef AYKEN_RING3_STERILE_GRAFT_UPPER_PT
#define AYKEN_RING3_STERILE_GRAFT_UPPER_PT 0
#endif

#ifndef AYKEN_SCHED_PT_ENTRIES
#define AYKEN_SCHED_PT_ENTRIES 512u
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

static void dbg_out_hex64(uint64_t v);
static void __attribute__((unused)) sched_ring3_diag_panic(const char *reason);
static void sched_emit_u64_dec(uint64_t v);
void sched_emit_ring3_frame_proof(const uint64_t *frame_rsp);
static void sched_mask_irq0_before_first_ring3_entry(proc_t *proc);
static uint8_t phase10_first_entry_irq0_masked = 0;
static void sched_note_first_user_entry_if_ring3(proc_t *proc);

static inline uint64_t ayken_rdtsc(void)
{
    uint32_t lo = 0;
    uint32_t hi = 0;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | (uint64_t)lo;
}

enum sched_perf_phase_id {
    SCHED_PERF_PHASE_BOOT_START = 0,
    SCHED_PERF_PHASE_CORE_READY,
    SCHED_PERF_PHASE_FIRST_SCHED_ACTIVITY,
    SCHED_PERF_PHASE_FIRST_USER_ENTRY,
    SCHED_PERF_PHASE_FIRST_SYSCALL_GATE_ENTRY,
    SCHED_PERF_PHASE_FIRST_SYSCALL_GATE_RETURN,
    SCHED_PERF_PHASE_FIRST_SYSCALL_ENTRY,
    SCHED_PERF_PHASE_FIRST_SYSCALL_EXIT,
    SCHED_PERF_PHASE_COUNT,
};

static uint8_t sched_perf_phase_emitted[SCHED_PERF_PHASE_COUNT];

enum sched_perf_mb_phase_id {
    SCHED_PERF_MB_PHASE_SNAPSHOT_ENTER = 0,
    SCHED_PERF_MB_PHASE_SNAPSHOT_EXIT,
    SCHED_PERF_MB_PHASE_EXTRACT_ENTER,
    SCHED_PERF_MB_PHASE_EXTRACT_EXIT,
    SCHED_PERF_MB_PHASE_VALIDATE_ENTER,
    SCHED_PERF_MB_PHASE_VALIDATE_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_OWNER_LOOKUP_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_OWNER_LOOKUP_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_LOOKUP_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_LOOKUP_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_ENTER,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_EXIT,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_SWITCH,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_KEEP_RUNNING,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_REJECT,
    SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_FALLBACK,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_ACCEPT_KEEP_RUNNING,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_ACCEPT_SWITCH,
    SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_REJECT,
    SCHED_PERF_MB_PHASE_ARBITER_KEEP_RUNNING_FALLBACK,
    SCHED_PERF_MB_PHASE_ARBITER_RETURN_NULL,
    SCHED_PERF_MB_PHASE_ARBITER_READY_HEAD_FALLBACK,
    SCHED_PERF_MB_PHASE_HANDOFF_ENTER,
    SCHED_PERF_MB_PHASE_HANDOFF_EXIT,
    SCHED_PERF_MB_PHASE_COUNT,
};

static uint8_t sched_perf_mb_phase_emitted[SCHED_PERF_MB_PHASE_COUNT];

static void sched_emit_marker(const char *text)
{
    if (!text) {
        return;
    }
    while (*text) {
        outb(0xE9, (uint8_t)*text++);
    }
}

static void sched_emit_perf_phase_marker(const char *name)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!name || !*name) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_PHASE]] name=");
    sched_emit_marker(name);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_emit_perf_mb_phase_marker(const char *name)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!name || !*name) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_PHASE]] name=");
    sched_emit_marker(name);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_emit_perf_mb_path_marker(const char *name, const char *phase)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!name || !*name || !phase || !*phase) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_PATH]] name=");
    sched_emit_marker(name);
    sched_emit_marker(" phase=");
    sched_emit_marker(phase);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_emit_perf_mb_reason_marker(const char *name)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!name || !*name) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_REASON]] name=");
    sched_emit_marker(name);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_emit_perf_mb_extract_reason_marker(const char *name)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!name || !*name) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_EXTRACT_REASON]] name=");
    sched_emit_marker(name);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_emit_perf_mb_extract_raw_marker(uint64_t epoch,
                                                  uint32_t candidate_pid,
                                                  uint64_t owner_last_epoch)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_EXTRACT_RAW]] epoch=");
    sched_emit_u64_dec(epoch);
    sched_emit_marker(" candidate_pid=");
    sched_emit_u64_dec((uint64_t)candidate_pid);
    sched_emit_marker(" owner_last_epoch=");
    sched_emit_u64_dec(owner_last_epoch);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_emit_perf_mb_candidate_visibility_marker(const char *name, uint32_t pid)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!name || !*name) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_VISIBLE]] name=");
    sched_emit_marker(name);
    sched_emit_marker(" pid=");
    sched_emit_u64_dec((uint64_t)pid);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

void sched_perf_note_mailbox_consume(const char *site,
                                     uint64_t old_last_epoch,
                                     uint64_t new_last_epoch,
                                     uint64_t candidate_epoch,
                                     const char *reason)
{
    uint64_t ticks = 0;
    uint32_t tick_valid = 0;
    uint64_t tsc = 0;

    if (!site || !*site || !reason || !*reason) {
        return;
    }

    tsc = ayken_rdtsc();
    if (tsc != 0) {
        ticks = tsc;
        tick_valid = 2;
    } else if (timer_is_initialized() != 0) {
        ticks = timer_ticks();
        tick_valid = 1;
    }

    sched_emit_marker("[[AYKEN_PERF_MB_CONSUME]] site=");
    sched_emit_marker(site);
    sched_emit_marker(" old_last_epoch=");
    sched_emit_u64_dec(old_last_epoch);
    sched_emit_marker(" new_last_epoch=");
    sched_emit_u64_dec(new_last_epoch);
    sched_emit_marker(" candidate_epoch=");
    sched_emit_u64_dec(candidate_epoch);
    sched_emit_marker(" reason=");
    sched_emit_marker(reason);
    sched_emit_marker(" ticks=");
    sched_emit_u64_dec(ticks);
    sched_emit_marker(" tick_valid=");
    sched_emit_u64_dec((uint64_t)tick_valid);
    sched_emit_marker("\n");
}

static void sched_note_perf_phase_once(enum sched_perf_phase_id id, const char *name)
{
    if ((uint32_t)id >= (uint32_t)SCHED_PERF_PHASE_COUNT) {
        return;
    }
    if (sched_perf_phase_emitted[id]) {
        return;
    }
    sched_perf_phase_emitted[id] = 1;
    sched_emit_perf_phase_marker(name);
}

static void sched_note_perf_mb_phase_once(enum sched_perf_mb_phase_id id, const char *name)
{
    if ((uint32_t)id >= (uint32_t)SCHED_PERF_MB_PHASE_COUNT) {
        return;
    }
    if (sched_perf_mb_phase_emitted[id]) {
        return;
    }
    sched_perf_mb_phase_emitted[id] = 1;
    sched_emit_perf_mb_phase_marker(name);
}

void sched_perf_note_boot_start(void)
{
    sched_note_perf_phase_once(SCHED_PERF_PHASE_BOOT_START, "boot_start");
}

void sched_perf_note_core_ready(void)
{
    sched_note_perf_phase_once(SCHED_PERF_PHASE_CORE_READY, "core_ready");
}

void sched_perf_note_first_scheduler_activity(void)
{
    sched_note_perf_phase_once(
        SCHED_PERF_PHASE_FIRST_SCHED_ACTIVITY,
        "first_sched_activity");
}

void sched_perf_note_first_user_entry(void)
{
    sched_note_perf_phase_once(SCHED_PERF_PHASE_FIRST_USER_ENTRY, "first_user_entry");
}

void sched_perf_note_first_syscall_gate_entry(void)
{
    sched_note_perf_phase_once(
        SCHED_PERF_PHASE_FIRST_SYSCALL_GATE_ENTRY,
        "first_syscall_gate_entry");
}

void sched_perf_note_first_syscall_gate_return(void)
{
    sched_note_perf_phase_once(
        SCHED_PERF_PHASE_FIRST_SYSCALL_GATE_RETURN,
        "first_syscall_gate_return");
}

void sched_perf_note_first_syscall_entry(void)
{
    sched_note_perf_phase_once(SCHED_PERF_PHASE_FIRST_SYSCALL_ENTRY, "first_syscall_entry");
}

void sched_perf_note_first_syscall_exit(void)
{
    sched_note_perf_phase_once(SCHED_PERF_PHASE_FIRST_SYSCALL_EXIT, "first_syscall_exit");
}

void sched_perf_note_mailbox_snapshot_enter(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_SNAPSHOT_ENTER, "snapshot_enter");
}

void sched_perf_note_mailbox_snapshot_exit(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_SNAPSHOT_EXIT, "snapshot_exit");
}

void sched_perf_note_mailbox_extract_enter(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_EXTRACT_ENTER, "extract_enter");
}

void sched_perf_note_mailbox_extract_exit(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_EXTRACT_EXIT, "extract_exit");
}

void sched_perf_note_mailbox_validate_enter(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_VALIDATE_ENTER, "validate_enter");
}

void sched_perf_note_mailbox_validate_exit(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_VALIDATE_EXIT, "validate_exit");
}

void sched_perf_note_mailbox_arbiter_enter(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_ARBITER_ENTER, "arbiter_enter");
}

void sched_perf_note_mailbox_arbiter_exit(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_ARBITER_EXIT, "arbiter_exit");
}

void sched_perf_note_mailbox_arbiter_owner_lookup_enter(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_OWNER_LOOKUP_ENTER,
        "arbiter_owner_lookup_enter");
}

void sched_perf_note_mailbox_arbiter_owner_lookup_exit(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_OWNER_LOOKUP_EXIT,
        "arbiter_owner_lookup_exit");
}

void sched_perf_note_mailbox_arbiter_candidate_lookup_enter(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_LOOKUP_ENTER,
        "arbiter_candidate_lookup_enter");
}

void sched_perf_note_mailbox_arbiter_candidate_lookup_exit(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_LOOKUP_EXIT,
        "arbiter_candidate_lookup_exit");
}

void sched_perf_note_mailbox_arbiter_decision_enter(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_DECISION_ENTER,
        "arbiter_decision_enter");
}

void sched_perf_note_mailbox_arbiter_decision_exit(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_DECISION_EXIT,
        "arbiter_decision_exit");
}

void sched_perf_note_mailbox_arbiter_decision_path_switch(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_SWITCH,
        "arbiter_decision_path_switch");
}

void sched_perf_note_mailbox_arbiter_decision_path_keep_running(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_KEEP_RUNNING,
        "arbiter_decision_path_keep_running");
}

void sched_perf_note_mailbox_arbiter_decision_path_reject(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_REJECT,
        "arbiter_decision_path_reject");
}

void sched_perf_note_mailbox_arbiter_decision_path_fallback(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_DECISION_PATH_FALLBACK,
        "arbiter_decision_path_fallback");
}

void sched_perf_note_mailbox_arbiter_path_switch_enter(void)
{
    sched_emit_perf_mb_path_marker("switch", "enter");
}

void sched_perf_note_mailbox_arbiter_path_switch_exit(void)
{
    sched_emit_perf_mb_path_marker("switch", "exit");
}

void sched_perf_note_mailbox_arbiter_path_keep_running_enter(void)
{
    sched_emit_perf_mb_path_marker("keep_running", "enter");
}

void sched_perf_note_mailbox_arbiter_path_keep_running_exit(void)
{
    sched_emit_perf_mb_path_marker("keep_running", "exit");
}

void sched_perf_note_mailbox_arbiter_path_reject_enter(void)
{
    sched_emit_perf_mb_path_marker("reject", "enter");
}

void sched_perf_note_mailbox_arbiter_path_reject_exit(void)
{
    sched_emit_perf_mb_path_marker("reject", "exit");
}

void sched_perf_note_mailbox_arbiter_path_fallback_enter(void)
{
    sched_emit_perf_mb_path_marker("fallback", "enter");
}

void sched_perf_note_mailbox_arbiter_path_fallback_exit(void)
{
    sched_emit_perf_mb_path_marker("fallback", "exit");
}

void sched_perf_note_mailbox_arbiter_candidate_accept_keep_running(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_ACCEPT_KEEP_RUNNING,
        "arbiter_candidate_accept_keep_running");
}

void sched_perf_note_mailbox_arbiter_candidate_accept_switch(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_ACCEPT_SWITCH,
        "arbiter_candidate_accept_switch");
}

void sched_perf_note_mailbox_arbiter_candidate_reject(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_CANDIDATE_REJECT,
        "arbiter_candidate_reject");
}

void sched_perf_note_mailbox_arbiter_keep_running_fallback(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_KEEP_RUNNING_FALLBACK,
        "arbiter_keep_running_fallback");
}

void sched_perf_note_mailbox_arbiter_return_null(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_RETURN_NULL,
        "arbiter_return_null");
}

void sched_perf_note_mailbox_arbiter_ready_head_fallback(void)
{
    sched_note_perf_mb_phase_once(
        SCHED_PERF_MB_PHASE_ARBITER_READY_HEAD_FALLBACK,
        "arbiter_ready_head_fallback");
}

void sched_perf_note_mailbox_handoff_enter(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_HANDOFF_ENTER, "handoff_enter");
}

void sched_perf_note_mailbox_handoff_exit(void)
{
    sched_note_perf_mb_phase_once(SCHED_PERF_MB_PHASE_HANDOFF_EXIT, "handoff_exit");
}

static void sched_note_first_user_entry_if_ring3(proc_t *proc)
{
    if (!proc) {
        return;
    }
    if ((proc->context.cs & 0x3u) == 0x3u) {
        sched_perf_note_first_user_entry();
    }
}

static void sched_mask_irq0_before_first_ring3_entry(proc_t *proc)
{
#if AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY == 1
    if (!phase10_first_entry_irq0_masked &&
        proc &&
        ((proc->context.cs & 0x3u) == 0x3u)) {
        phase10_first_entry_irq0_masked = 1;
        pic_set_mask(0);
        sched_emit_marker("P10_IRQ0_MASK_FIRST_ENTRY\n");
    }
#else
    (void)proc;
#endif
}

static void sched_force_ring3_entry_cr3_to_kernel_root(proc_t *proc)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_RING3_FORCE_KERNEL_CR3 == 1)
    uint64_t active_cr3 = 0;

    if (!proc || (proc->context.cs & 0x3) != 0x3) {
        return;
    }

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == 0) {
        return;
    }

    sched_emit_marker("P10_FORCE_KERNEL_CR3 O=");
    dbg_out_hex64(proc->context.cr3);
    sched_emit_marker(" A=");
    dbg_out_hex64(active_cr3);
    sched_emit_marker("\n");
    proc->context.cr3 = active_cr3;
#else
    (void)proc;
#endif
}

static uint64_t sched_get_sterile_kernel_clone_root_phys(void)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_RING3_STERILE_ALT_ROOT == 1)
    static uint64_t sterile_root_phys = 0;
    uint64_t kernel_root_phys;
    uint64_t *src;
    uint64_t *dst;

    if (sterile_root_phys != 0) {
        return sterile_root_phys;
    }

    kernel_root_phys = paging_get_kernel_pml4_phys() & AYKEN_PTE_ADDR_MASK;
    if (kernel_root_phys == 0) {
        return 0;
    }

    if (AYKEN_RING3_HIGH_PHYS_STERILE_ALT_ROOT == 1) {
        sterile_root_phys = phys_alloc_frame_high();
    } else {
        sterile_root_phys = paging_alloc_page_table();
    }
    if (sterile_root_phys == 0) {
        return 0;
    }

    src = (uint64_t *)paging_phys_to_virt(kernel_root_phys);
    dst = (uint64_t *)paging_phys_to_virt(sterile_root_phys);
    if (!src || !dst) {
        return 0;
    }

    memcpy(dst, src, AYKEN_FRAME_SIZE);
    sched_emit_marker("P10_STERILE_ALT_ROOT K=");
    dbg_out_hex64(kernel_root_phys);
    sched_emit_marker(" S=");
    dbg_out_hex64(sterile_root_phys);
    sched_emit_marker("\n");
    return sterile_root_phys;
#else
    return 0;
#endif
}

static void sched_force_ring3_entry_cr3_to_sterile_root(proc_t *proc)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_RING3_STERILE_ALT_ROOT == 1)
    uint64_t sterile_root_phys;

    if (!proc || (proc->context.cs & 0x3) != 0x3) {
        return;
    }

    sterile_root_phys = sched_get_sterile_kernel_clone_root_phys();
    if (sterile_root_phys == 0) {
        sched_ring3_diag_panic("sterile_alt_root_missing");
    }

    sched_emit_marker("P10_FORCE_STERILE_CR3 O=");
    dbg_out_hex64(proc->context.cr3);
    sched_emit_marker(" S=");
    dbg_out_hex64(sterile_root_phys);
    sched_emit_marker("\n");
    proc->context.cr3 = sterile_root_phys;
#else
    (void)proc;
#endif
}

static void sched_graft_pt_leaf_entry(uint64_t src_pde,
                                      uint64_t *dst_pde,
                                      uint16_t pt_index)
{
    uint64_t *src_pt;
    uint64_t *dst_pt;
    uint64_t dst_entry;

    if (!dst_pde || (src_pde & AYKEN_PTE_PRESENT) == 0 || (*dst_pde & AYKEN_PTE_PRESENT) == 0) {
        return;
    }
    if ((src_pde & (1ULL << 7)) != 0 || (*dst_pde & (1ULL << 7)) != 0) {
        return;
    }

    dst_entry = *dst_pde;
    *dst_pde = (dst_entry & AYKEN_PTE_ADDR_MASK) | (src_pde & ~AYKEN_PTE_ADDR_MASK);

    src_pt = (uint64_t *)paging_phys_to_virt(src_pde & AYKEN_PTE_ADDR_MASK);
    dst_pt = (uint64_t *)paging_phys_to_virt((*dst_pde) & AYKEN_PTE_ADDR_MASK);
    if (!src_pt || !dst_pt) {
        return;
    }

    dst_pt[pt_index] = src_pt[pt_index];
}

static void sched_graft_pt_leaf_span(uint64_t src_pde,
                                     uint64_t *dst_pde,
                                     uint16_t first_pt_index,
                                     uint16_t count)
{
    uint16_t i;

    for (i = 0; i < count; ++i) {
        sched_graft_pt_leaf_entry(src_pde, dst_pde, (uint16_t)(first_pt_index + i));
    }
}

static void sched_graft_real_user_state_into_sterile_root(proc_t *proc)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_RING3_STERILE_ALT_ROOT == 1) && \
    ((AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_FULL_UPPER_HALF == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_LOWER_PML4_SEM == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_LOWER_PDPT_SEM == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_PD2 == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_PD3 == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_PD3_SEM_ONLY == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_TEXT == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_CANARY == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_STACK == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_STACK_TOP == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_STACK_LOW == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_MAILBOX == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_INBOX == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_PAYLOAD == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_UPPER_PML4_SEM == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_UPPER_PDPT == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_UPPER_PD == 1) || \
     (AYKEN_RING3_STERILE_GRAFT_UPPER_PT == 1))
    uint64_t sterile_root_phys;
    uint64_t source_root_phys;
    uint64_t *src_pml4;
    uint64_t *dst_pml4;
    uint64_t src_pml4e;
    uint64_t src_upper_pml4e;
    uint64_t dst_pml4e;
    uint64_t *src_pdpt;
    uint64_t *dst_pdpt;
    uint64_t src_pdpte;
    uint64_t dst_pdpte;
    uint64_t *src_pd;
    uint64_t *dst_pd;
    uint64_t dst_upper_pml4e;
    uint64_t *src_upper_pdpt;
    uint64_t *dst_upper_pdpt;
    uint64_t src_upper_pdpte;
    uint64_t dst_upper_pdpte;
    uint64_t *src_upper_pd;
    uint64_t *dst_upper_pd;
    uint64_t src_upper_pde;
    uint64_t dst_upper_pde;
    uint64_t *src_upper_pt;
    uint64_t *dst_upper_pt;
    const uint16_t upper_pml4_i =
        (uint16_t)((AYKEN_RING3_CANONICAL_STAGE_A_VA >> 39) & 0x1FFu);
    const uint16_t upper_pdpt_i =
        (uint16_t)((AYKEN_RING3_CANONICAL_STAGE_A_VA >> 30) & 0x1FFu);
    const uint16_t upper_pd_i =
        (uint16_t)((AYKEN_RING3_CANONICAL_STAGE_A_VA >> 21) & 0x1FFu);
    const uint16_t upper_pt_stage_a_i =
        (uint16_t)((AYKEN_RING3_CANONICAL_STAGE_A_VA >> 12) & 0x1FFu);
    const uint16_t upper_pt_stage_b_i =
        (uint16_t)((AYKEN_RING3_CANONICAL_STAGE_B_VA >> 12) & 0x1FFu);
    const uint16_t upper_pt_stage_c_i =
        (uint16_t)((AYKEN_RING3_CANONICAL_STAGE_C_VA >> 12) & 0x1FFu);
    const uint16_t text_pt_i = (uint16_t)((0x0000000000400000ULL >> 12) & 0x1FFu);
    const uint16_t canary_pt_i = (uint16_t)((0x0000000000405000ULL >> 12) & 0x1FFu);
    const uint16_t stack_top_pt_i =
        (uint16_t)(((USER_STACK_TOP - 8ULL) >> 12) & 0x1FFu);
    const uint16_t stack_below_pt_i =
        (uint16_t)(((USER_STACK_TOP - AYKEN_FRAME_SIZE - 8ULL) >> 12) & 0x1FFu);
    const uint16_t mailbox_pt_i = (uint16_t)((SCHED_MAILBOX_VA >> 12) & 0x1FFu);
    const uint16_t inbox_pt_i = (uint16_t)((EXECUTION_INBOX_VA >> 12) & 0x1FFu);
    const uint16_t payload_pt_i = (uint16_t)((EXECUTION_PAYLOAD_VA >> 12) & 0x1FFu);
    const uint8_t do_lower =
        (uint8_t)((AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_LOWER_PML4_SEM == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_LOWER_PDPT_SEM == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_PD2 == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_PD3 == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_PD3_SEM_ONLY == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_TEXT == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_CANARY == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_STACK == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_STACK_TOP == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_STACK_LOW == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_MAILBOX == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_INBOX == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_PAYLOAD == 1));
    const uint8_t copy_lower_pml4_sem =
        (uint8_t)((AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_LOWER_PML4_SEM == 1));
    const uint8_t copy_lower_pdpt_sem =
        (uint8_t)((AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_LOWER_PDPT_SEM == 1));
    const uint8_t do_upper =
        (uint8_t)((AYKEN_RING3_STERILE_GRAFT_FULL_UPPER_HALF == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_UPPER_PML4_SEM == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_UPPER_PDPT == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_UPPER_PD == 1) ||
                  (AYKEN_RING3_STERILE_GRAFT_UPPER_PT == 1));

    if (!proc || (proc->context.cs & 0x3) != 0x3) {
        return;
    }

    source_root_phys = proc->context.cr3 & AYKEN_PTE_ADDR_MASK;
    sterile_root_phys = sched_get_sterile_kernel_clone_root_phys() & AYKEN_PTE_ADDR_MASK;
    if (source_root_phys == 0 || sterile_root_phys == 0 || source_root_phys == sterile_root_phys) {
        return;
    }

    src_pml4 = (uint64_t *)paging_phys_to_virt(source_root_phys);
    dst_pml4 = (uint64_t *)paging_phys_to_virt(sterile_root_phys);
    if (!src_pml4 || !dst_pml4) {
        return;
    }

    src_pml4e = src_pml4[0];
    src_upper_pml4e = src_pml4[upper_pml4_i];
    if (!do_lower && !do_upper) {
        return;
    }

    sched_emit_marker("P10_STERILE_GRAFT R=");
    dbg_out_hex64(source_root_phys);
    sched_emit_marker(" S=");
    dbg_out_hex64(sterile_root_phys);
    sched_emit_marker(" FL=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF);
    sched_emit_marker(" FU=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_FULL_UPPER_HALF);
    sched_emit_marker(" LP4=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_LOWER_PML4_SEM);
    sched_emit_marker(" LP3=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_LOWER_PDPT_SEM);
    sched_emit_marker(" P2=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_PD2);
    sched_emit_marker(" P3=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_PD3);
    sched_emit_marker(" P3S=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_PD3_SEM_ONLY);
    sched_emit_marker(" TX=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_TEXT);
    sched_emit_marker(" CY=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_CANARY);
    sched_emit_marker(" ST=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_STACK);
    sched_emit_marker(" STT=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_STACK_TOP);
    sched_emit_marker(" STL=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_STACK_LOW);
    sched_emit_marker(" MB=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_MAILBOX);
    sched_emit_marker(" IN=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_INBOX);
    sched_emit_marker(" PL=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_PAYLOAD);
    sched_emit_marker(" U4=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_UPPER_PML4_SEM);
    sched_emit_marker(" U3=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_UPPER_PDPT);
    sched_emit_marker(" U2=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_UPPER_PD);
    sched_emit_marker(" U1=");
    sched_emit_u64_dec((uint64_t)AYKEN_RING3_STERILE_GRAFT_UPPER_PT);
    sched_emit_marker("\n");

    if (do_lower && ((src_pml4e & AYKEN_PTE_PRESENT) != 0)) {
#if AYKEN_RING3_STERILE_GRAFT_FULL_LOWER_HALF == 1
        dst_pml4[0] = src_pml4e;
#else
        dst_pml4e = dst_pml4[0];
        if ((dst_pml4e & AYKEN_PTE_PRESENT) != 0) {
            src_pdpt = (uint64_t *)paging_phys_to_virt(src_pml4e & AYKEN_PTE_ADDR_MASK);
            dst_pdpt = (uint64_t *)paging_phys_to_virt(dst_pml4e & AYKEN_PTE_ADDR_MASK);
            if (src_pdpt && dst_pdpt) {
                src_pdpte = src_pdpt[0];
                dst_pdpte = dst_pdpt[0];
                if ((src_pdpte & AYKEN_PTE_PRESENT) != 0 &&
                    (dst_pdpte & AYKEN_PTE_PRESENT) != 0) {
                    src_pd = (uint64_t *)paging_phys_to_virt(src_pdpte & AYKEN_PTE_ADDR_MASK);
                    dst_pd = (uint64_t *)paging_phys_to_virt(dst_pdpte & AYKEN_PTE_ADDR_MASK);
                    if (src_pd && dst_pd) {
                        if (copy_lower_pml4_sem) {
                            dst_pml4[0] =
                                (dst_pml4e & AYKEN_PTE_ADDR_MASK) |
                                (src_pml4e & ~AYKEN_PTE_ADDR_MASK);
                        }
                        if (copy_lower_pdpt_sem) {
                            dst_pdpt[0] =
                                (dst_pdpte & AYKEN_PTE_ADDR_MASK) |
                                (src_pdpte & ~AYKEN_PTE_ADDR_MASK);
                        }
#if AYKEN_RING3_STERILE_GRAFT_PD2 == 1
                        dst_pd[2] = src_pd[2];
#endif
#if AYKEN_RING3_STERILE_GRAFT_PD3 == 1
                        dst_pd[3] = src_pd[3];
#endif
#if AYKEN_RING3_STERILE_GRAFT_PD3_SEM_ONLY == 1
                        dst_pd[3] =
                            (dst_pd[3] & AYKEN_PTE_ADDR_MASK) |
                            (src_pd[3] & ~AYKEN_PTE_ADDR_MASK);
#endif
#if AYKEN_RING3_STERILE_GRAFT_TEXT == 1
                        sched_graft_pt_leaf_entry(src_pd[2], &dst_pd[2], text_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_CANARY == 1
                        sched_graft_pt_leaf_entry(src_pd[2], &dst_pd[2], canary_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_STACK == 1
                        sched_graft_pt_leaf_entry(src_pd[3], &dst_pd[3], stack_top_pt_i);
                        sched_graft_pt_leaf_entry(src_pd[3], &dst_pd[3], stack_below_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_STACK_TOP == 1
                        sched_graft_pt_leaf_entry(src_pd[3], &dst_pd[3], stack_top_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_STACK_LOW == 1
                        sched_graft_pt_leaf_entry(src_pd[3], &dst_pd[3], stack_below_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_MAILBOX == 1
                        sched_graft_pt_leaf_entry(src_pd[3], &dst_pd[3], mailbox_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_INBOX == 1
                        sched_graft_pt_leaf_entry(src_pd[3], &dst_pd[3], inbox_pt_i);
#endif
#if AYKEN_RING3_STERILE_GRAFT_PAYLOAD == 1
                        sched_graft_pt_leaf_span(
                            src_pd[3],
                            &dst_pd[3],
                            payload_pt_i,
                            AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES);
#endif
                    }
                }
            }
        }
#endif
    }

    if (do_upper && ((src_upper_pml4e & AYKEN_PTE_PRESENT) != 0)) {
#if AYKEN_RING3_STERILE_GRAFT_FULL_UPPER_HALF == 1
        dst_pml4[upper_pml4_i] = src_upper_pml4e;
#else
        dst_upper_pml4e = dst_pml4[upper_pml4_i];
        if ((dst_upper_pml4e & AYKEN_PTE_PRESENT) == 0) {
            return;
        }

#if AYKEN_RING3_STERILE_GRAFT_UPPER_PML4_SEM == 1
        dst_pml4[upper_pml4_i] =
            (dst_upper_pml4e & AYKEN_PTE_ADDR_MASK) |
            (src_upper_pml4e & ~AYKEN_PTE_ADDR_MASK);
        dst_upper_pml4e = dst_pml4[upper_pml4_i];
#endif

        src_upper_pdpt =
            (uint64_t *)paging_phys_to_virt(src_upper_pml4e & AYKEN_PTE_ADDR_MASK);
        dst_upper_pdpt =
            (uint64_t *)paging_phys_to_virt(dst_upper_pml4e & AYKEN_PTE_ADDR_MASK);
        if (!src_upper_pdpt || !dst_upper_pdpt) {
            return;
        }

        src_upper_pdpte = src_upper_pdpt[upper_pdpt_i];
        dst_upper_pdpte = dst_upper_pdpt[upper_pdpt_i];
        if ((src_upper_pdpte & AYKEN_PTE_PRESENT) == 0 ||
            (dst_upper_pdpte & AYKEN_PTE_PRESENT) == 0) {
            return;
        }

#if AYKEN_RING3_STERILE_GRAFT_UPPER_PDPT == 1
        dst_upper_pdpt[upper_pdpt_i] = src_upper_pdpte;
        dst_upper_pdpte = dst_upper_pdpt[upper_pdpt_i];
#endif

        src_upper_pd =
            (uint64_t *)paging_phys_to_virt(src_upper_pdpte & AYKEN_PTE_ADDR_MASK);
        dst_upper_pd =
            (uint64_t *)paging_phys_to_virt(dst_upper_pdpte & AYKEN_PTE_ADDR_MASK);
        if (!src_upper_pd || !dst_upper_pd) {
            return;
        }

        src_upper_pde = src_upper_pd[upper_pd_i];
        dst_upper_pde = dst_upper_pd[upper_pd_i];
        if ((src_upper_pde & AYKEN_PTE_PRESENT) == 0 ||
            (dst_upper_pde & AYKEN_PTE_PRESENT) == 0) {
            return;
        }

#if AYKEN_RING3_STERILE_GRAFT_UPPER_PD == 1
        dst_upper_pd[upper_pd_i] = src_upper_pde;
        dst_upper_pde = dst_upper_pd[upper_pd_i];
#endif

#if AYKEN_RING3_STERILE_GRAFT_UPPER_PT == 1
        if ((src_upper_pde & (1ULL << 7)) == 0 && (dst_upper_pde & (1ULL << 7)) == 0) {
            src_upper_pt =
                (uint64_t *)paging_phys_to_virt(src_upper_pde & AYKEN_PTE_ADDR_MASK);
            dst_upper_pt =
                (uint64_t *)paging_phys_to_virt(dst_upper_pde & AYKEN_PTE_ADDR_MASK);
            if (src_upper_pt && dst_upper_pt) {
                dst_upper_pt[upper_pt_stage_a_i] = src_upper_pt[upper_pt_stage_a_i];
                dst_upper_pt[upper_pt_stage_b_i] = src_upper_pt[upper_pt_stage_b_i];
                dst_upper_pt[upper_pt_stage_c_i] = src_upper_pt[upper_pt_stage_c_i];
            }
        }
#endif
#endif
    }
#else
    (void)proc;
#endif
}

// Ring0 mechanism state - only for context switching
static proc_t *ready_head = NULL;
static proc_t *ready_tail = NULL;
static proc_t *blocked_head = NULL;
static proc_t *sched_owner_cached = NULL;
static uint32_t g_sched_active_owner_pid = AYKEN_SCHED_OWNER_PID;
static volatile uint8_t g_sched_owner_transfer_pending = 0;
static volatile int g_sched_owner_transfer_from_pid = 0;
static volatile int g_sched_owner_transfer_to_pid = 0;
static volatile uint8_t g_sched_validation_owner_transfer_seen = 0;
static volatile int g_sched_validation_owner_transfer_from_pid = 0;
static volatile int g_sched_validation_owner_transfer_to_pid = 0;
static volatile uint8_t g_sched_validation_mailbox_decision_seen = 0;
static volatile int g_sched_validation_mailbox_decision_from_pid = 0;
static volatile int g_sched_validation_mailbox_decision_to_pid = 0;
static volatile int g_sched_validation_mailbox_decision_src_pid = 0;
static volatile uint64_t g_sched_validation_mailbox_decision_id = 0;

static void remove_from_blocked(proc_t *p);
static int sched_is_owner(const proc_t *p);

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

typedef enum {
    SCHED_DECISION_SITE_START = 0,
    SCHED_DECISION_SITE_YIELD = 1,
    SCHED_DECISION_SITE_BLOCK = 2,
    SCHED_DECISION_SITE_IRQ = 3,
} sched_decision_site_t;

static const char *sched_site_name(sched_decision_site_t site)
{
    switch (site) {
    case SCHED_DECISION_SITE_START:
        return "START";
    case SCHED_DECISION_SITE_YIELD:
        return "YIELD";
    case SCHED_DECISION_SITE_BLOCK:
        return "BLOCK";
    case SCHED_DECISION_SITE_IRQ:
        return "IRQ";
    default:
        return "YIELD";
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

#if AYKEN_GATE45_PROOF || AYKEN_C2_STRICT_MARKERS
static uint64_t sched_c2_decision_counter = 0;

static uint64_t sched_next_c2_decision_id(void)
{
    sched_c2_decision_counter++;
    return sched_c2_decision_counter;
}

static void sched_emit_gate45_arbiter_decision(
    uint64_t decision_id,
    sched_decision_site_t site,
    uint32_t owner_pid,
    uint32_t from_pid,
    uint32_t to_pid,
    uint64_t epoch)
{
#if AYKEN_C2_STRICT_MARKERS
    sched_emit_marker("[[AYKEN_SCHED_ARBITER_DECISION]] decision_id=");
    sched_emit_u64_dec(decision_id);
    sched_emit_marker(" site=");
    sched_emit_marker(sched_site_name(site));
    sched_emit_marker(" owner=");
    sched_emit_u64_dec((uint64_t)owner_pid);
    sched_emit_marker(" from=");
    sched_emit_u64_dec((uint64_t)from_pid);
    sched_emit_marker(" to=");
    sched_emit_u64_dec((uint64_t)to_pid);
    sched_emit_marker(" epoch=");
    sched_emit_u64_dec(epoch);
    sched_emit_marker("\n");
#else
    (void)decision_id;
    (void)site;
    (void)owner_pid;
    sched_emit_marker("[[AYKEN_SCHED_ARBITER_DECISION]] from=");
    sched_emit_u64_dec((uint64_t)from_pid);
    sched_emit_marker(" to=");
    sched_emit_u64_dec((uint64_t)to_pid);
    sched_emit_marker(" epoch=");
    sched_emit_u64_dec(epoch);
    sched_emit_marker("\n");
#endif
}

static void sched_emit_gate45_ctx_switch(uint64_t decision_id, uint32_t from_pid, uint32_t to_pid)
{
#if AYKEN_C2_STRICT_MARKERS
    sched_emit_marker("[[AYKEN_CTX_SWITCH]] decision_id=");
    sched_emit_u64_dec(decision_id);
    sched_emit_marker(" from=");
    sched_emit_u64_dec((uint64_t)from_pid);
    sched_emit_marker(" to=");
    sched_emit_u64_dec((uint64_t)to_pid);
    sched_emit_marker("\n");
#else
    (void)decision_id;
    sched_emit_marker("[[AYKEN_CTX_SWITCH]] from=");
    sched_emit_u64_dec((uint64_t)from_pid);
    sched_emit_marker(" to=");
    sched_emit_u64_dec((uint64_t)to_pid);
    sched_emit_marker("\n");
#endif
}

static void sched_emit_gate45_cursor_advance(
    uint64_t decision_id,
    uint32_t owner_pid,
    uint32_t next_owner_pid)
{
#if AYKEN_C2_STRICT_MARKERS
    sched_emit_marker("[[AYKEN_SCHED_CURSOR_ADVANCE]] decision_id=");
    sched_emit_u64_dec(decision_id);
    sched_emit_marker(" owner=");
    sched_emit_u64_dec((uint64_t)owner_pid);
    sched_emit_marker(" next_owner=");
    sched_emit_u64_dec((uint64_t)next_owner_pid);
    sched_emit_marker("\n");
#else
    (void)decision_id;
    (void)owner_pid;
    (void)next_owner_pid;
#endif
}

static void sched_emit_gate45_chain_once(
    proc_t *prev,
    proc_t *next,
    uint64_t epoch,
    uint32_t owner_pid,
    int used_mailbox,
    sched_decision_site_t site)
{
    static uint8_t gate45_chain_emitted = 0;
    if (gate45_chain_emitted) {
        return;
    }
    if (!used_mailbox || epoch == 0 || !prev || !next || prev == next) {
        return;
    }
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4.5 ordering contract: emit only after timer-path epoch=1 ACCEPT.
    if (sched_mailbox_gate4_epoch1_pending()) {
        return;
    }
#endif
    gate45_chain_emitted = 1;
    uint64_t decision_id = sched_next_c2_decision_id();
    sched_emit_gate45_arbiter_decision(
        decision_id,
        site,
        owner_pid,
        (uint32_t)prev->pid,
        (uint32_t)next->pid,
        epoch);
    sched_emit_gate45_ctx_switch(decision_id, (uint32_t)prev->pid, (uint32_t)next->pid);
    sched_emit_gate45_cursor_advance(decision_id, owner_pid, owner_pid);
}
#else
static inline void sched_emit_gate45_chain_once(
    proc_t *prev,
    proc_t *next,
    uint64_t epoch,
    uint32_t owner_pid,
    int used_mailbox,
    sched_decision_site_t site)
{
    (void)prev;
    (void)next;
    (void)epoch;
    (void)owner_pid;
    (void)used_mailbox;
    (void)site;
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

static void sched_emit_mailbox_miss_fatal_pre(
    sched_decision_site_t site,
    const proc_t *prev,
    const proc_t *owner)
{
    sched_emit_marker("P10_MAILBOX_MISS_FATAL_PRE site=");
    sched_emit_marker(sched_site_name(site));
    sched_emit_marker(" owner=");
    sched_emit_u64_dec(owner ? (uint64_t)(uint32_t)owner->pid : 0);
    sched_emit_marker(" current=");
    sched_emit_u64_dec(prev ? (uint64_t)(uint32_t)prev->pid : 0);
    // Do not dereference queue nodes here: under user CR3, non-current kernel
    // heap pages may be unmapped and would fault in the fatal-report path.
    sched_emit_marker(" ready_head_ptr=");
    sched_emit_u64_dec((uint64_t)(uintptr_t)ready_head);
    sched_emit_marker(" blocked_head_ptr=");
    sched_emit_u64_dec((uint64_t)(uintptr_t)blocked_head);
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

static int sched_mailbox_read_snapshot(proc_t *owner, ayken_sched_mailbox_t *out_mb)
{
    uint64_t active_cr3 = 0;
    uint64_t kernel_cr3 = paging_get_kernel_pml4_phys();
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;
    const ayken_sched_mailbox_t *src = NULL;

    if (!owner || !owner->mailbox_pa || !out_mb) {
        return 0;
    }

    sched_perf_note_mailbox_snapshot_enter();
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if (kernel_cr3 &&
        ((active_cr3 & AYKEN_PTE_ADDR_MASK) != (kernel_cr3 & AYKEN_PTE_ADDR_MASK))) {
        __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
        __asm__ volatile("cli");
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
        switched_to_kernel_cr3 = 1;
    }
    src = (const ayken_sched_mailbox_t *)paging_phys_to_virt(owner->mailbox_pa);

    if (!src) {
        if (switched_to_kernel_cr3) {
            __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
            if (saved_rflags & (1ULL << 9)) {
                __asm__ volatile("sti");
            }
        }
        sched_perf_note_mailbox_snapshot_exit();
        return 0;
    }

    *out_mb = *src;

    if (switched_to_kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
        if (saved_rflags & (1ULL << 9)) {
            __asm__ volatile("sti");
        }
    }

    sched_perf_note_mailbox_snapshot_exit();
    return 1;
}

#if AYKEN_GATE45_PROOF
static void sched_gate45_arm_cross_target_once(proc_t *owner)
{
    static uint8_t armed = 0;
    if (armed || !sched_is_owner(owner)) {
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
    ayken_sched_mailbox_t mb_snapshot;
    const ayken_sched_mailbox_t *mb = NULL;

    if (!owner || !out_epoch || !out_pid || !owner->mailbox_pa) {
        return 0;
    }
    sched_perf_note_mailbox_extract_enter();
    if (!sched_mailbox_read_snapshot(owner, &mb_snapshot)) {
        sched_emit_perf_mb_extract_reason_marker("snapshot_fail");
        sched_perf_note_mailbox_extract_exit();
        return 0;
    }
    mb = &mb_snapshot;
    sched_emit_perf_mb_extract_raw_marker(
        mb->epoch,
        mb->candidate_pid,
        owner->mailbox_last_epoch);
    if (mb->magic != AYKEN_SCHED_MB_MAGIC) {
        sched_emit_perf_mb_extract_reason_marker("bad_magic");
        sched_perf_note_mailbox_extract_exit();
        return 0;
    }
    if (mb->version != AYKEN_SCHED_MB_VERSION) {
        sched_emit_perf_mb_extract_reason_marker("bad_version");
        sched_perf_note_mailbox_extract_exit();
        return 0;
    }
    if (mb->kind != AYKEN_SCHED_HINT_CANDIDATE) {
        sched_emit_perf_mb_extract_reason_marker("bad_kind");
        sched_perf_note_mailbox_extract_exit();
        return 0;
    }
    if (mb->epoch == 0 || mb->epoch <= owner->mailbox_last_epoch) {
        sched_emit_perf_mb_extract_reason_marker("epoch_stale");
        sched_perf_note_mailbox_extract_exit();
        return 0;
    }
    if (mb->candidate_pid == 0) {
        sched_emit_perf_mb_extract_reason_marker("pid_zero");
        sched_perf_note_mailbox_extract_exit();
        return 0;
    }
    *out_epoch = mb->epoch;
    *out_pid = mb->candidate_pid;
    sched_emit_perf_mb_extract_reason_marker("ok");
    sched_perf_note_mailbox_extract_exit();
    return 1;
}

uint32_t sched_active_owner_pid(void)
{
    return g_sched_active_owner_pid;
}

static int sched_is_owner(const proc_t *p)
{
    return p && (uint32_t)p->pid == g_sched_active_owner_pid;
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
    proc_t *owner = proc_find_by_pid((int)g_sched_active_owner_pid);
    if (owner && sched_is_owner(owner)) {
        sched_owner_cached = owner;
        return owner;
    }

    return NULL;
}

static int sched_mailbox_has_any_candidate(proc_t *p)
{
    ayken_sched_mailbox_t mb_snapshot;
    const ayken_sched_mailbox_t *mb = NULL;

    if (!sched_mailbox_read_snapshot(p, &mb_snapshot)) {
        return 0;
    }
    mb = &mb_snapshot;
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
    int arbiter_decision_open = 0;
    const char *arbiter_reason_name = NULL;

#define SCHED_MB_DECISION_BEGIN() \
    do { \
        if (!arbiter_decision_open) { \
            sched_perf_note_mailbox_arbiter_decision_enter(); \
            arbiter_decision_open = 1; \
        } \
    } while (0)

#define SCHED_MB_DECISION_END() \
    do { \
        if (arbiter_decision_open) { \
            sched_perf_note_mailbox_arbiter_decision_exit(); \
            arbiter_decision_open = 0; \
        } \
    } while (0)

#define SCHED_MB_REASON(name) \
    do { \
        arbiter_reason_name = (name); \
    } while (0)

#define SCHED_MB_ARBITER_RETURN(value) \
    do { \
        if (arbiter_reason_name && *arbiter_reason_name) { \
            sched_emit_perf_mb_reason_marker(arbiter_reason_name); \
        } \
        SCHED_MB_DECISION_END(); \
        sched_perf_note_mailbox_arbiter_exit(); \
        return (value); \
    } while (0)

    sched_perf_note_mailbox_arbiter_enter();
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
        SCHED_MB_DECISION_BEGIN();
        SCHED_MB_REASON("gate45_non_owner");
        sched_perf_note_mailbox_arbiter_path_fallback_enter();
        sched_perf_note_mailbox_arbiter_decision_path_fallback();
        sched_perf_note_mailbox_arbiter_keep_running_fallback();
        sched_perf_note_mailbox_arbiter_path_fallback_exit();
        SCHED_MB_ARBITER_RETURN(prev);
    }
#endif

    sched_perf_note_mailbox_arbiter_owner_lookup_enter();
    proc_t *owner = sched_owner_proc(prev, site);
    sched_perf_note_mailbox_arbiter_owner_lookup_exit();
    if (!owner) {
        SCHED_MB_DECISION_BEGIN();
        SCHED_MB_REASON("owner_missing");
        sched_perf_note_mailbox_arbiter_path_reject_enter();
        sched_perf_note_mailbox_arbiter_decision_path_reject();
        sched_emit_mailbox_miss_fatal_pre(site, prev, NULL);
        sched_emit_marker("P10_MAILBOX_OWNER_MISSING_FATAL\n");
        sched_perf_note_mailbox_arbiter_return_null();
        sched_perf_note_mailbox_arbiter_path_reject_exit();
        SCHED_MB_ARBITER_RETURN(NULL);
    }
    if (!(owner->state == PROC_READY || owner->state == PROC_RUNNING)) {
        SCHED_MB_DECISION_BEGIN();
        SCHED_MB_REASON("owner_not_ready");
        sched_perf_note_mailbox_arbiter_path_reject_enter();
        sched_perf_note_mailbox_arbiter_decision_path_reject();
        sched_emit_mailbox_miss_fatal_pre(site, prev, owner);
        sched_emit_marker("P10_MAILBOX_OWNER_NOT_READY_FATAL\n");
        sched_perf_note_mailbox_arbiter_return_null();
        sched_perf_note_mailbox_arbiter_path_reject_exit();
        SCHED_MB_ARBITER_RETURN(NULL);
    }

    // Non-owner fresh decision attempt is a protocol violation.
    if (prev && prev->type == PROC_TYPE_USER && !sched_is_owner(prev) &&
        sched_mailbox_has_any_candidate(prev)) {
        SCHED_MB_DECISION_BEGIN();
        SCHED_MB_REASON("owner_mismatch");
        sched_perf_note_mailbox_arbiter_path_reject_enter();
        sched_emit_marker("P10_MAILBOX_OWNER_MISMATCH\n");
#if AYKEN_SCHED_BOOTSTRAP_POLICY
        if (site != SCHED_DECISION_SITE_YIELD) {
            sched_perf_note_mailbox_arbiter_decision_path_reject();
            sched_perf_note_mailbox_arbiter_return_null();
            sched_perf_note_mailbox_arbiter_path_reject_exit();
            SCHED_MB_ARBITER_RETURN(NULL);
        }
#else
        sched_perf_note_mailbox_arbiter_decision_path_reject();
        sched_perf_note_mailbox_arbiter_return_null();
        sched_perf_note_mailbox_arbiter_path_reject_exit();
        SCHED_MB_ARBITER_RETURN(NULL);
#endif
    }

    // Single-authority path: only owner mailbox is consumed.
    {
        uint64_t epoch = 0;
        uint32_t pid = 0;
        int extracted = sched_mailbox_extract_candidate(owner, &epoch, &pid);
        if (extracted) {
            int consume_epoch = 1;
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
            // Gate-4 proof requires timer-path ACCEPT for epoch=1 before scheduler
            // consumes that epoch in any decision site.
            if (epoch == 1 && sched_mailbox_gate4_epoch1_pending()) {
                consume_epoch = 0;
                sched_perf_note_mailbox_consume(
                    sched_site_name(site),
                    owner->mailbox_last_epoch,
                    owner->mailbox_last_epoch,
                    epoch,
                    "gate4_epoch1_pending_bypass");
            }
#endif
#if AYKEN_GATE45_PROOF
            // Gate-4.5: do not consume epoch=1 on self-keep-running path.
            // This keeps epoch=1 available until Ring3 flips candidate to cross-target.
            if (epoch == 1 &&
                prev && prev->type == PROC_TYPE_USER &&
                pid == (uint32_t)prev->pid) {
                consume_epoch = 0;
                sched_perf_note_mailbox_consume(
                    sched_site_name(site),
                    owner->mailbox_last_epoch,
                    owner->mailbox_last_epoch,
                    epoch,
                    "gate45_self_keep_running_bypass");
            }
#endif
            SCHED_MB_DECISION_BEGIN();
            if (prev && prev->type == PROC_TYPE_USER &&
                prev->state == PROC_RUNNING &&
                pid == (uint32_t)prev->pid) {
                sched_emit_perf_mb_candidate_visibility_marker("visible", pid);
                sched_perf_note_mailbox_arbiter_path_keep_running_enter();
                if (consume_epoch) {
                    uint64_t old_last_epoch = owner->mailbox_last_epoch;
                    owner->mailbox_last_epoch = epoch;
                    sched_perf_note_mailbox_consume(
                        sched_site_name(site),
                        old_last_epoch,
                        owner->mailbox_last_epoch,
                        epoch,
                        "scheduler_keep_running_consume");
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
                sched_perf_note_mailbox_arbiter_decision_path_keep_running();
                sched_perf_note_mailbox_arbiter_candidate_accept_keep_running();
                sched_perf_note_mailbox_arbiter_path_keep_running_exit();
                SCHED_MB_ARBITER_RETURN(prev);
            }
            sched_perf_note_mailbox_arbiter_candidate_lookup_enter();
            proc_t *cand = proc_find_by_pid((int)pid);
            if (cand && (cand->state == PROC_READY || cand->state == PROC_RUNNING)) {
                sched_perf_note_mailbox_arbiter_candidate_lookup_exit();
                sched_emit_perf_mb_candidate_visibility_marker("visible", pid);
                sched_perf_note_mailbox_arbiter_path_switch_enter();
                if (consume_epoch) {
                    uint64_t old_last_epoch = owner->mailbox_last_epoch;
                    owner->mailbox_last_epoch = epoch;
                    sched_perf_note_mailbox_consume(
                        sched_site_name(site),
                        old_last_epoch,
                        owner->mailbox_last_epoch,
                        epoch,
                        "scheduler_switch_consume");
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
                sched_perf_note_mailbox_arbiter_decision_path_switch();
                sched_perf_note_mailbox_arbiter_candidate_accept_switch();
                remove_from_ready_queue(cand);
                sched_perf_note_mailbox_arbiter_path_switch_exit();
                SCHED_MB_ARBITER_RETURN(cand);
            }
            sched_perf_note_mailbox_arbiter_candidate_lookup_exit();
            if (!cand) {
                SCHED_MB_REASON("candidate_proc_missing");
                sched_emit_perf_mb_candidate_visibility_marker("proc_missing", pid);
            } else {
                SCHED_MB_REASON("candidate_proc_not_schedulable");
                sched_emit_perf_mb_candidate_visibility_marker("proc_not_schedulable", pid);
            }
            sched_perf_note_mailbox_arbiter_candidate_reject();
        }
        if (!extracted) {
            SCHED_MB_REASON("no_candidate");
        }
    }

    // Yield-only safety invariant: without fresh decision, keep current Ring3 context.
    if (allow_keep_running && prev && prev->type == PROC_TYPE_USER) {
        SCHED_MB_DECISION_BEGIN();
        if (prev->state != PROC_RUNNING) {
            SCHED_MB_REASON("invalid_state");
            sched_perf_note_mailbox_arbiter_path_reject_enter();
            sched_emit_marker("P10_MAILBOX_MISS_KEEP_RUNNING_INVALID_STATE\n");
            sched_perf_note_mailbox_arbiter_decision_path_reject();
            sched_perf_note_mailbox_arbiter_return_null();
            sched_perf_note_mailbox_arbiter_path_reject_exit();
            SCHED_MB_ARBITER_RETURN(NULL);
        }
#if AYKEN_DEBUG_SCHED
        if (ready_head == prev || ready_tail == prev) {
            sched_debug_assert_fail('Q');
        }
#endif
#if AYKEN_SCHED_BOOTSTRAP_POLICY
        static uint8_t keep_running_marker_emitted = 0;
        if (!keep_running_marker_emitted) {
            keep_running_marker_emitted = 1;
            sched_emit_marker("P10_MAILBOX_MISS_KEEP_RUNNING\n");
        }
        if (!arbiter_reason_name) {
            SCHED_MB_REASON("bootstrap_keep_running");
        }
        sched_perf_note_mailbox_arbiter_path_fallback_enter();
        sched_perf_note_mailbox_arbiter_decision_path_fallback();
        sched_perf_note_mailbox_arbiter_keep_running_fallback();
        sched_perf_note_mailbox_arbiter_path_fallback_exit();
        SCHED_MB_ARBITER_RETURN(prev);
#else
        // Phase10-A2 bootstrap barrier: until first user-code proof marker is seen,
        // avoid mailbox fatal on yield miss and keep current Ring3 context running.
        if (phase10_ring3_user_code_seen == 0u) {
            static uint8_t pre_user_bypass_marker_emitted = 0;
            if (!pre_user_bypass_marker_emitted) {
                pre_user_bypass_marker_emitted = 1;
                sched_emit_marker("P10_MAILBOX_MISS_PRE_USER_BYPASS\n");
            }
            if (!arbiter_reason_name) {
                SCHED_MB_REASON("pre_user_bypass");
            }
            sched_perf_note_mailbox_arbiter_path_fallback_enter();
            sched_perf_note_mailbox_arbiter_decision_path_fallback();
            sched_perf_note_mailbox_arbiter_keep_running_fallback();
            sched_perf_note_mailbox_arbiter_path_fallback_exit();
            SCHED_MB_ARBITER_RETURN(prev);
        }
        if (!arbiter_reason_name) {
            SCHED_MB_REASON("yield_fatal");
        }
        sched_perf_note_mailbox_arbiter_path_reject_enter();
        sched_emit_mailbox_miss_fatal_pre(site, prev, owner);
        sched_emit_marker("P10_MAILBOX_MISS_YIELD_FATAL\n");
        sched_perf_note_mailbox_arbiter_decision_path_reject();
        sched_perf_note_mailbox_arbiter_return_null();
        sched_perf_note_mailbox_arbiter_path_reject_exit();
        SCHED_MB_ARBITER_RETURN(NULL);
#endif
    }

    // Transitional fallback is compile-time gated; default constitutional mode is fail-closed.
#if AYKEN_SCHED_FALLBACK
#if AYKEN_SCHED_BOOTSTRAP_POLICY
    SCHED_MB_DECISION_BEGIN();
    SCHED_MB_REASON("ready_head_fallback");
    sched_perf_note_mailbox_arbiter_path_fallback_enter();
    sched_emit_marker("P10_SCHED_FALLBACK\n");
    sched_emit_marker("P10_READY_HEAD_FALLBACK\n");
    sched_perf_note_mailbox_arbiter_decision_path_fallback();
    sched_perf_note_mailbox_arbiter_ready_head_fallback();
    sched_perf_note_mailbox_arbiter_path_fallback_exit();
    SCHED_MB_ARBITER_RETURN(sched_select_next_ready_head_fallback());
#else
    SCHED_MB_DECISION_BEGIN();
    SCHED_MB_REASON("fallback_forbidden");
    sched_perf_note_mailbox_arbiter_path_reject_enter();
    sched_emit_marker("P10_SCHED_FALLBACK_FORBIDDEN\n");
    sched_perf_note_mailbox_arbiter_decision_path_reject();
    sched_perf_note_mailbox_arbiter_return_null();
    sched_perf_note_mailbox_arbiter_path_reject_exit();
    SCHED_MB_ARBITER_RETURN(NULL);
#endif
#else
    SCHED_MB_DECISION_BEGIN();
    if (site == SCHED_DECISION_SITE_BLOCK) {
        SCHED_MB_REASON("block_fatal");
    } else if (site == SCHED_DECISION_SITE_START) {
        SCHED_MB_REASON("bootstrap_fatal");
    } else if (!arbiter_reason_name) {
        SCHED_MB_REASON("yield_null");
    }
    sched_perf_note_mailbox_arbiter_path_reject_enter();
    if (site == SCHED_DECISION_SITE_BLOCK) {
        sched_emit_mailbox_miss_fatal_pre(site, prev, owner);
        sched_emit_marker("P10_MAILBOX_MISS_BLOCK_FATAL\n");
    } else if (site == SCHED_DECISION_SITE_START) {
        sched_emit_mailbox_miss_fatal_pre(site, prev, owner);
        sched_emit_marker("P10_MAILBOX_MISS_BOOTSTRAP_FATAL\n");
    } else {
        sched_emit_mailbox_miss_fatal_pre(site, prev, owner);
        sched_emit_marker("P10_MAILBOX_MISS_YIELD_NULL\n");
    }
    sched_perf_note_mailbox_arbiter_decision_path_reject();
    sched_perf_note_mailbox_arbiter_return_null();
    sched_perf_note_mailbox_arbiter_path_reject_exit();
    SCHED_MB_ARBITER_RETURN(NULL);
#endif

#undef SCHED_MB_DECISION_END
#undef SCHED_MB_DECISION_BEGIN
#undef SCHED_MB_REASON
#undef SCHED_MB_ARBITER_RETURN
}

// Canonical preempt observability markers must remain available even when
// verbose scheduler debug tracing is compiled out.
static void sched_dbg_mark_pid(uint32_t pid)
{
    if (pid != 2u && pid != 3u) {
        return;
    }
    sched_emit_marker("MARK:PID=");
    outb(0xE9, (uint8_t)('0' + (uint8_t)pid));
    outb(0xE9, (uint8_t)'\n');
}

static void sched_dbg_mark_sw(char from, char to)
{
    sched_emit_marker("MARK:SW=");
    outb(0xE9, (uint8_t)from);
    outb(0xE9, (uint8_t)'>');
    outb(0xE9, (uint8_t)to);
    outb(0xE9, (uint8_t)'\n');
}

static void sched_dbg_mark_iret(void)
{
    sched_emit_marker("MARK:IRET\n");
}

proc_t *current_proc = NULL;

static proc_t *g_sched_validation_exit_forced_next = NULL;
static volatile uint8_t g_sched_validation_exit_switch_seen = 0;
static volatile int g_sched_validation_exit_from_pid = 0;
static volatile int g_sched_validation_exit_to_pid = 0;

static void sched_clear_owner_transfer_request(void)
{
    g_sched_owner_transfer_pending = 0;
    g_sched_owner_transfer_from_pid = 0;
    g_sched_owner_transfer_to_pid = 0;
}

static void sched_clear_validation_owner_transfer_event(void)
{
    g_sched_validation_owner_transfer_seen = 0;
    g_sched_validation_owner_transfer_from_pid = 0;
    g_sched_validation_owner_transfer_to_pid = 0;
}

static void sched_clear_validation_mailbox_decision_event(void)
{
    g_sched_validation_mailbox_decision_seen = 0;
    g_sched_validation_mailbox_decision_from_pid = 0;
    g_sched_validation_mailbox_decision_to_pid = 0;
    g_sched_validation_mailbox_decision_src_pid = 0;
    g_sched_validation_mailbox_decision_id = 0;
}

static void sched_record_mailbox_decision_event(proc_t *prev,
                                                proc_t *next,
                                                uint64_t decision_id,
                                                uint32_t decision_src_pid,
                                                int used_mailbox)
{
    if (!used_mailbox || !prev || !next || decision_src_pid == 0) {
        return;
    }

    g_sched_validation_mailbox_decision_seen = 1;
    g_sched_validation_mailbox_decision_from_pid = prev->pid;
    g_sched_validation_mailbox_decision_to_pid = next->pid;
    g_sched_validation_mailbox_decision_src_pid = (int)decision_src_pid;
    g_sched_validation_mailbox_decision_id = decision_id;
}

static void sched_commit_owner_transfer_if_pending(proc_t *prev, proc_t *next)
{
    sched_perf_note_mailbox_handoff_enter();
    if (!g_sched_owner_transfer_pending || !prev || !next) {
        sched_perf_note_mailbox_handoff_exit();
        return;
    }
    if (prev->pid != g_sched_owner_transfer_from_pid ||
        next->pid != g_sched_owner_transfer_to_pid) {
        sched_perf_note_mailbox_handoff_exit();
        return;
    }

    g_sched_active_owner_pid = (uint32_t)next->pid;
    sched_owner_cached = next;
    g_sched_validation_owner_transfer_seen = 1;
    g_sched_validation_owner_transfer_from_pid = prev->pid;
    g_sched_validation_owner_transfer_to_pid = next->pid;
    sched_clear_owner_transfer_request();
    sched_perf_note_mailbox_handoff_exit();
}

int sched_request_owner_transfer(proc_t *caller_owner, proc_t *successor)
{
    int result = -1;

    disable_interrupts();

    if (!caller_owner || !successor) {
        goto out;
    }
    if (!sched_is_owner(caller_owner) || caller_owner->state == PROC_ZOMBIE) {
        goto out;
    }
    if (successor == caller_owner || successor->pid <= 0) {
        goto out;
    }
    if (successor->type != PROC_TYPE_USER || successor->state == PROC_ZOMBIE) {
        goto out;
    }
    if (!(successor->state == PROC_READY || successor->state == PROC_RUNNING)) {
        goto out;
    }
    if (successor->mailbox_pa == 0 || sched_mailbox_validate_ring3(successor) != 0) {
        goto out;
    }
    if (g_sched_owner_transfer_pending) {
        goto out;
    }

    g_sched_owner_transfer_pending = 1;
    g_sched_owner_transfer_from_pid = caller_owner->pid;
    g_sched_owner_transfer_to_pid = successor->pid;
    sched_clear_validation_owner_transfer_event();
    sched_clear_validation_mailbox_decision_event();
    result = 0;

out:
    enable_interrupts();
    return result;
}

void sched_validation_set_active_owner(proc_t *owner)
{
    disable_interrupts();
    g_sched_active_owner_pid =
        owner && owner->pid > 0 ? (uint32_t)owner->pid : AYKEN_SCHED_OWNER_PID;
    sched_owner_cached = owner && sched_is_owner(owner) ? owner : NULL;
    sched_clear_owner_transfer_request();
    sched_clear_validation_owner_transfer_event();
    sched_clear_validation_mailbox_decision_event();
    enable_interrupts();
}

int sched_validation_take_owner_transfer_event(int *from_pid, int *to_pid)
{
    if (!g_sched_validation_owner_transfer_seen) {
        return 0;
    }

    if (from_pid) {
        *from_pid = g_sched_validation_owner_transfer_from_pid;
    }
    if (to_pid) {
        *to_pid = g_sched_validation_owner_transfer_to_pid;
    }

    sched_clear_validation_owner_transfer_event();
    return 1;
}

int sched_validation_take_mailbox_decision_event(int *from_pid,
                                                 int *to_pid,
                                                 int *src_pid,
                                                 uint64_t *decision_id)
{
    if (!g_sched_validation_mailbox_decision_seen) {
        return 0;
    }

    if (from_pid) {
        *from_pid = g_sched_validation_mailbox_decision_from_pid;
    }
    if (to_pid) {
        *to_pid = g_sched_validation_mailbox_decision_to_pid;
    }
    if (src_pid) {
        *src_pid = g_sched_validation_mailbox_decision_src_pid;
    }
    if (decision_id) {
        *decision_id = g_sched_validation_mailbox_decision_id;
    }

    sched_clear_validation_mailbox_decision_event();
    return 1;
}

int sched_validation_non_owner_publish_would_fail(proc_t *publisher)
{
    if (!publisher || publisher->type != PROC_TYPE_USER) {
        return 0;
    }

    return !sched_is_owner(publisher) && sched_mailbox_has_any_candidate(publisher);
}

void sched_validation_arm_exit_successor(proc_t *forced_next)
{
    g_sched_validation_exit_forced_next = forced_next;
    g_sched_validation_exit_switch_seen = 0;
    g_sched_validation_exit_from_pid = 0;
    g_sched_validation_exit_to_pid = 0;
}

void sched_validation_disarm_exit_successor(void)
{
    g_sched_validation_exit_forced_next = NULL;
    g_sched_validation_exit_switch_seen = 0;
    g_sched_validation_exit_from_pid = 0;
    g_sched_validation_exit_to_pid = 0;
}

int sched_validation_take_exit_switch_event(int *from_pid, int *to_pid)
{
    if (!g_sched_validation_exit_switch_seen) {
        return 0;
    }

    if (from_pid) {
        *from_pid = g_sched_validation_exit_from_pid;
    }
    if (to_pid) {
        *to_pid = g_sched_validation_exit_to_pid;
    }

    g_sched_validation_exit_switch_seen = 0;
    g_sched_validation_exit_from_pid = 0;
    g_sched_validation_exit_to_pid = 0;
    return 1;
}

static void sched_zero_execution_payload_window(proc_t *worker)
{
    uint32_t i;

    if (!worker) {
        return;
    }

    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (worker->execution_payload_pas[i] == 0) {
            continue;
        }
        {
            void *dst = paging_phys_to_virt(worker->execution_payload_pas[i]);
            if (dst) {
                memset(dst, 0, AYKEN_FRAME_SIZE);
            }
        }
    }
}

static void sched_reset_execution_delivery_surface(proc_t *worker)
{
    ayken_execution_inbox_v1_t *inbox;

    if (!worker || worker->execution_inbox_pa == 0) {
        return;
    }

    sched_zero_execution_payload_window(worker);

    inbox = (ayken_execution_inbox_v1_t *)paging_phys_to_virt(worker->execution_inbox_pa);
    if (!inbox) {
        return;
    }

    inbox->magic = AYKEN_EXECUTION_INBOX_MAGIC;
    inbox->version = AYKEN_EXECUTION_INBOX_VERSION;
    inbox->state = AXIB_STATE_EMPTY;
    inbox->execution_id = 0;
    inbox->target_context_id = 0;
    inbox->bcib_user_va = EXECUTION_PAYLOAD_VA;
    inbox->bcib_size = 0;
    inbox->bcib_window_size = AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE;
    inbox->flags = 0;
    memset(inbox->reserved, 0, sizeof(inbox->reserved));
}

static int sched_publish_execution_delivery(proc_t *worker, const exec_slot_t *slot)
{
    ayken_execution_inbox_v1_t *inbox;
    uint64_t next_delivery_seq;
    uint32_t i;
    uint64_t remaining;

    if (!worker || !slot) {
        return -1;
    }
    if (worker->active_execution_id != slot->execution_id || worker->active_execution_id == 0) {
        return -1;
    }
    if (!execution_slot_can_publish_locked(slot)) {
        return -1;
    }
    if (worker->execution_inbox_pa == 0) {
        return -1;
    }
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (worker->execution_payload_pas[i] == 0) {
            return -1;
        }
    }

    inbox = (ayken_execution_inbox_v1_t *)paging_phys_to_virt(worker->execution_inbox_pa);
    if (!inbox) {
        return -1;
    }

    sched_zero_execution_payload_window(worker);

    remaining = slot->bcib_size;
    for (i = 0; i < slot->bcib_frame_count; ++i) {
        uint64_t copy_size = remaining > AYKEN_FRAME_SIZE ? AYKEN_FRAME_SIZE : remaining;
        const void *src;
        void *dst;

        src = paging_phys_to_virt(slot->bcib_frames[i]);
        dst = paging_phys_to_virt(worker->execution_payload_pas[i]);
        if (!src || !dst) {
            sched_reset_execution_delivery_surface(worker);
            return -1;
        }

        memcpy(dst, src, copy_size);
        remaining -= copy_size;
    }

    inbox->magic = AYKEN_EXECUTION_INBOX_MAGIC;
    inbox->version = AYKEN_EXECUTION_INBOX_VERSION;
    inbox->state = AXIB_STATE_READY;
    inbox->execution_id = slot->execution_id;
    inbox->target_context_id = slot->target_context_id;
    inbox->bcib_user_va = EXECUTION_PAYLOAD_VA;
    inbox->bcib_size = slot->bcib_size;
    inbox->bcib_window_size = AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE;
    inbox->flags = 0;
    memset(inbox->reserved, 0, sizeof(inbox->reserved));

    __sync_synchronize();

    next_delivery_seq = worker->execution_delivery_seq + 1;
    if (next_delivery_seq == 0) {
        next_delivery_seq = 1;
    }
    worker->execution_delivery_seq = next_delivery_seq;
    inbox->delivery_seq = next_delivery_seq;
    return 0;
}

int sched_try_pickup_execution_work(void)
{
    execution_slot_guard_t slot_guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    exec_slot_t *slot = NULL;

    if (!current_proc || current_proc->pid <= 0) {
        return 0;
    }
    if (current_proc->type != PROC_TYPE_USER) {
        return 0;
    }
    if (current_proc->active_execution_id != 0) {
        return 0;
    }

    execution_slot_enter_critical(&slot_guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_PICKUP);
    slot = execution_slot_pickup_locked((uint64_t)current_proc->pid);
    if (slot) {
        current_proc->active_execution_id = slot->execution_id;
        if (execution_slot_prepare_output_locked(slot) != 0 ||
            proc_bind_execution_output_window(current_proc,
                                              slot->output_frames,
                                              slot->output_frame_count,
                                              slot->execution_id) != 0 ||
            sched_publish_execution_delivery(current_proc, slot) != 0) {
            sched_reset_execution_delivery_surface(current_proc);
            proc_unmap_execution_output_window(current_proc);
            current_proc->active_execution_id = 0;
            execution_slot_require_finish_locked(slot,
                                                 EXEC_SLOT_ABORTED,
                                                 "sched_try_pickup_execution_work");
            slot = NULL;
        }
    }
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&slot_guard);

    return slot != NULL ? 1 : 0;
}
static volatile uint32_t need_resched = 0;
// One-shot by design: proves mailbox decision/apply path exists without per-tick log churn.
// NOTE: current path is single-CPU validation; SMP enablement requires atomic/lock.
static uint8_t phase10c_decision_markers_emitted = 0;
// One-shot IRQ decision marker for strict-mode preemption diagnosis.
// NOTE: current path is single-CPU validation; SMP enablement requires atomic/lock.
static uint8_t phase10_irq_decision_marker_emitted = 0;
static uint8_t phase10_retire_witness_marker_emitted = 0;
static uint8_t phase10_text_witness_marker_emitted = 0;
// Set by IRQ path when current user context is explicitly snapshotted.
// context_switch.asm consumes this flag to avoid overwriting user RIP/RSP
// with kernel scheduler frame values.
volatile uint32_t sched_irq_user_ctx_saved = 0;

#define RING3_CANARY_ADDR 0x0000000000405000ULL
#define RING3_CANARY_PRE  0x1111111122222222ULL
#define RING3_CANARY_POST 0x3333333344444444ULL
#define RING3_TEXT_WITNESS_ADDR (USER_TEXT_BASE + 0x100ULL)
#define RING3_TEXT_WITNESS_SIG  0x5458543157544E53ULL

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

static void __attribute__((unused)) sched_dbg_emit_text(const char *text)
{
#if AYKEN_DEBUG_SCHED
    if (!text) {
        return;
    }
    while (*text) {
        SCHED_DBG_OUT((uint8_t)*text++);
    }
#else
    (void)text;
#endif
}

static void sched_dbg_emit_bytes_hex(const uint8_t *bytes, size_t len)
{
#if AYKEN_DEBUG_SCHED
    static const char hex[] = "0123456789ABCDEF";

    if (!bytes) {
        return;
    }

    for (size_t i = 0; i < len; ++i) {
        uint8_t b = bytes[i];
        SCHED_DBG_OUT((uint8_t)hex[(b >> 4) & 0x0F]);
        SCHED_DBG_OUT((uint8_t)hex[b & 0x0F]);
    }
#else
    (void)bytes;
    (void)len;
#endif
}

static uint16_t sched_dbg_read_tr_selector(void)
{
    uint16_t tr = 0;
    __asm__ volatile("str %0" : "=r"(tr));
    return tr;
}

static uint64_t sched_dbg_idt_entry_offset(const struct idt_entry *entry)
{
    if (!entry) {
        return 0;
    }

    return ((uint64_t)entry->offset_low) |
           (((uint64_t)entry->offset_mid) << 16) |
           (((uint64_t)entry->offset_high) << 32);
}

void sched_emit_ring3_frame_proof(const uint64_t *frame_rsp)
{
#if AYKEN_DEBUG_SCHED
    const struct idt_entry *gp = &idt_table[13];
    const struct idt_entry *pf = &idt_table[14];
    const uint64_t ctx_rsp0 = current_proc ? current_proc->context.rsp0 : 0;

    if (!frame_rsp) {
        return;
    }

    sched_emit_marker("P10_RING3_FRAME_PROOF FRSP=");
    dbg_out_hex64((uint64_t)(uintptr_t)frame_rsp);
    sched_emit_marker(" RIP=");
    dbg_out_hex64(frame_rsp[0]);
    sched_emit_marker(" CS=");
    dbg_out_hex64(frame_rsp[1]);
    sched_emit_marker(" RF=");
    dbg_out_hex64(frame_rsp[2]);
    sched_emit_marker(" RSP=");
    dbg_out_hex64(frame_rsp[3]);
    sched_emit_marker(" SS=");
    dbg_out_hex64(frame_rsp[4]);
    sched_emit_marker(" BYTES=");
    sched_dbg_emit_bytes_hex((const uint8_t *)frame_rsp, sizeof(uint64_t) * 5u);
    sched_emit_marker("\n");

    sched_emit_marker("P10_RING3_GATE_PROOF TR=");
    dbg_out_hex16(sched_dbg_read_tr_selector());
    sched_emit_marker(" CTX0=");
    dbg_out_hex64(ctx_rsp0);
    sched_emit_marker(" TSS0=");
    dbg_out_hex64(kernel_tss.rsp0);
    sched_emit_marker(" FCS=");
    dbg_out_hex64(frame_rsp[1]);
    sched_emit_marker(" FSS=");
    dbg_out_hex64(frame_rsp[4]);
    sched_emit_marker(" IDTB=");
    dbg_out_hex64(idt_descriptor.base);
    sched_emit_marker(" IDTL=");
    dbg_out_hex64((uint64_t)idt_descriptor.limit);
    sched_emit_marker(" GPSEL=");
    dbg_out_hex16(gp->selector);
    sched_emit_marker(" GPIST=");
    dbg_out_hex64((uint64_t)gp->ist);
    sched_emit_marker(" GPTA=");
    dbg_out_hex64((uint64_t)gp->type_attr);
    sched_emit_marker(" GPOFF=");
    dbg_out_hex64(sched_dbg_idt_entry_offset(gp));
    sched_emit_marker(" PFSEL=");
    dbg_out_hex16(pf->selector);
    sched_emit_marker(" PFIST=");
    dbg_out_hex64((uint64_t)pf->ist);
    sched_emit_marker(" PFTA=");
    dbg_out_hex64((uint64_t)pf->type_attr);
    sched_emit_marker(" PFOFF=");
    dbg_out_hex64(sched_dbg_idt_entry_offset(pf));
    sched_emit_marker("\n");
#else
    (void)frame_rsp;
#endif
}

static int __attribute__((unused)) sched_dbg_watch_table_phys(uint64_t phys)
{
    uint64_t page = phys & AYKEN_PTE_ADDR_MASK;

    return page >= 0x000000000010A000ULL && page < 0x000000000010E000ULL;
}

static void __attribute__((unused)) sched_dbg_emit_stack_map_request(uint64_t root_phys,
                                                                     uint64_t virt_addr,
                                                                     uint64_t phys_addr,
                                                                     uint64_t flags)
{
#if AYKEN_DEBUG_SCHED
    if (!sched_dbg_watch_table_phys(root_phys) &&
        !sched_dbg_watch_table_phys(phys_addr)) {
        return;
    }

    sched_dbg_emit_text("SKMP R=");
    dbg_out_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(virt_addr);
    sched_dbg_emit_text(" P=");
    dbg_out_hex64(phys_addr & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" F=");
    dbg_out_hex64(flags);
    sched_dbg_emit_text("\n");
#else
    (void)root_phys;
    (void)virt_addr;
    (void)phys_addr;
    (void)flags;
#endif
}

static uint64_t __attribute__((unused)) sched_dbg_read_u64_le(const uint8_t *ptr)
{
    uint64_t value = 0;

    if (!ptr) {
        return 0;
    }

    for (uint32_t i = 0; i < 8; ++i) {
        value |= ((uint64_t)ptr[i]) << (i * 8);
    }
    return value;
}

static const uint8_t *__attribute__((unused)) sched_dbg_identity_page(uint64_t phys_page)
{
#if AYKEN_DEBUG_SCHED
    phys_page &= AYKEN_PTE_ADDR_MASK;
    if (phys_page == 0 || phys_page >= AYKEN_IDENTITY_MAP_SIZE) {
        return NULL;
    }
    /* Validation lanes retain low identity mappings under the active kernel CR3. */
    return (const uint8_t *)(uintptr_t)phys_page;
#else
    (void)phys_page;
    return NULL;
#endif
}

static uint64_t __attribute__((unused)) sched_dbg_hash_page_bytes(const uint8_t *page)
{
    uint64_t hash = 1469598103934665603ULL;

    if (!page) {
        return 0;
    }

    for (uint64_t i = 0; i < AYKEN_FRAME_SIZE; ++i) {
        hash ^= page[i];
        hash *= 1099511628211ULL;
    }

    return hash;
}

static uint64_t __attribute__((unused)) sched_dbg_hash_phys_page_identity(uint64_t phys_page)
{
    return sched_dbg_hash_page_bytes(sched_dbg_identity_page(phys_page));
}

static void sched_emit_phys_frame_witness(const char *tag,
                                          const char *phase,
                                          uint64_t root_phys,
                                          uint64_t pte,
                                          uint64_t phys_page)
{
#if AYKEN_DEBUG_SCHED
    const uint8_t *page;
    uint64_t active_cr3 = 0;
    uint64_t kernel_cr3;
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;
    int used;

    if (!tag || !phase) {
        return;
    }

    phys_page &= AYKEN_PTE_ADDR_MASK;
    kernel_cr3 = paging_get_kernel_pml4_phys() & AYKEN_PTE_ADDR_MASK;
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if (kernel_cr3 != 0 &&
        ((active_cr3 & AYKEN_PTE_ADDR_MASK) != kernel_cr3)) {
        __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
        __asm__ volatile("cli");
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
        switched_to_kernel_cr3 = 1;
    }

    page = phys_page ? (const uint8_t *)paging_phys_to_virt(phys_page) : NULL;
    used = phys_page ? phys_frame_is_used(phys_page) : 0;

    sched_emit_marker(tag);
    sched_emit_marker(" phase=");
    sched_emit_marker(phase);
    sched_emit_marker(" root=");
    dbg_out_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
    sched_emit_marker(" pte=");
    dbg_out_hex64(pte);
    sched_emit_marker(" phys=");
    dbg_out_hex64(phys_page);
    sched_emit_marker(" used=");
    sched_emit_u64_dec((uint64_t)((phys_page != 0 && used == 1) ? 1u : 0u));
    sched_emit_marker(" lo=");
    dbg_out_hex64(sched_dbg_read_u64_le(page));
    sched_emit_marker(" hi=");
    dbg_out_hex64(sched_dbg_read_u64_le(page ? page + 8 : NULL));
    sched_emit_marker(" hash=");
    dbg_out_hex64(sched_dbg_hash_page_bytes(page));
    sched_emit_marker("\n");

    if (switched_to_kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
        if (saved_rflags & (1ULL << 9)) {
            __asm__ volatile("sti");
        }
    }
#else
    (void)tag;
    (void)phase;
    (void)root_phys;
    (void)pte;
    (void)phys_page;
#endif
}

extern void ring3_enter_iretq(void);
extern char ring3_enter_post_cr3[];
extern char ring3_enter_trampoline[];
extern char ring3_enter_iret_trampoline[];
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1) && \
    defined(AYKEN_RING3_ALT_STAGEB_SOURCE) && (AYKEN_RING3_ALT_STAGEB_SOURCE == 1)
extern char ring3_enter_alt_bridge_trampoline[];
#endif
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
extern char ring3_enter_final_iret_trampoline[];
#endif
static void map_kernel_stack_pages_into_pml4(uint64_t pml4_phys, uint64_t rsp0);
static int read_user_u64_via_pml4(uint64_t pml4_phys, uint64_t va, uint64_t *out);

typedef struct {
    uint64_t root_phys;
    uint64_t va;
    uint64_t pml4_table_phys;
    uint64_t pml4e_phys;
    uint64_t pml4e;
    uint64_t pdpt_table_phys;
    uint64_t pdpte_phys;
    uint64_t pdpte;
    uint64_t pd_table_phys;
    uint64_t pde_phys;
    uint64_t pde;
    uint64_t pt_table_phys;
    uint64_t pte_phys;
    uint64_t pte;
    uint64_t final_phys;
    uint8_t valid;
} sched_walk_snapshot_t;

static int __attribute__((unused)) sched_capture_identity_walk_snapshot(
    uint64_t root_phys,
    uint64_t va,
    sched_walk_snapshot_t *out)
{
    const uint64_t *pml4;
    uint16_t pml4_i;
    uint16_t pdpt_i;
    uint16_t pd_i;
    uint16_t pt_i;

    if (!out || !root_phys) {
        return 0;
    }

    memset(out, 0, sizeof(*out));
    out->root_phys = root_phys & AYKEN_PTE_ADDR_MASK;
    out->va = va;
    out->pml4_table_phys = out->root_phys;

    pml4 = (const uint64_t *)sched_dbg_identity_page(out->root_phys);
    if (!pml4) {
        return 0;
    }

    pml4_i = (uint16_t)((va >> 39) & 0x1FF);
    pdpt_i = (uint16_t)((va >> 30) & 0x1FF);
    pd_i = (uint16_t)((va >> 21) & 0x1FF);
    pt_i = (uint16_t)((va >> 12) & 0x1FF);

    out->pml4e_phys = out->pml4_table_phys + ((uint64_t)pml4_i * sizeof(uint64_t));
    out->pml4e = pml4[pml4_i];
    if ((out->pml4e & AYKEN_PTE_PRESENT) == 0) {
        return 0;
    }

    out->pdpt_table_phys = out->pml4e & AYKEN_PTE_ADDR_MASK;
    {
        const uint64_t *pdpt = (const uint64_t *)sched_dbg_identity_page(out->pdpt_table_phys);
        if (!pdpt) {
            return 0;
        }
        out->pdpte_phys = out->pdpt_table_phys + ((uint64_t)pdpt_i * sizeof(uint64_t));
        out->pdpte = pdpt[pdpt_i];
        if ((out->pdpte & AYKEN_PTE_PRESENT) == 0) {
            return 0;
        }
        if (out->pdpte & (1ULL << 7)) {
            out->final_phys = (out->pdpte & AYKEN_PTE_ADDR_MASK) | (va & ((1ULL << 30) - 1));
            out->valid = 1;
            return 1;
        }
    }

    out->pd_table_phys = out->pdpte & AYKEN_PTE_ADDR_MASK;
    {
        const uint64_t *pd = (const uint64_t *)sched_dbg_identity_page(out->pd_table_phys);
        if (!pd) {
            return 0;
        }
        out->pde_phys = out->pd_table_phys + ((uint64_t)pd_i * sizeof(uint64_t));
        out->pde = pd[pd_i];
        if ((out->pde & AYKEN_PTE_PRESENT) == 0) {
            return 0;
        }
        if (out->pde & (1ULL << 7)) {
            out->final_phys = (out->pde & AYKEN_PTE_ADDR_MASK) | (va & ((1ULL << 21) - 1));
            out->valid = 1;
            return 1;
        }
    }

    out->pt_table_phys = out->pde & AYKEN_PTE_ADDR_MASK;
    {
        const uint64_t *pt = (const uint64_t *)sched_dbg_identity_page(out->pt_table_phys);
        if (!pt) {
            return 0;
        }
        out->pte_phys = out->pt_table_phys + ((uint64_t)pt_i * sizeof(uint64_t));
        out->pte = pt[pt_i];
        if ((out->pte & AYKEN_PTE_PRESENT) == 0) {
            return 0;
        }
    }

    out->final_phys = (out->pte & AYKEN_PTE_ADDR_MASK) | (va & (AYKEN_FRAME_SIZE - 1));
    out->valid = 1;
    return 1;
}

static int sched_capture_walk_snapshot(
    uint64_t root_phys,
    uint64_t va,
    sched_walk_snapshot_t *out)
{
    uint64_t active_cr3;
    uint64_t kernel_cr3;
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;
    uint64_t *pml4;
    uint16_t pml4_i;
    uint16_t pdpt_i;
    uint16_t pd_i;
    uint16_t pt_i;

    if (!out || !root_phys) {
        return 0;
    }

    memset(out, 0, sizeof(*out));
    out->root_phys = root_phys & AYKEN_PTE_ADDR_MASK;
    out->va = va;
    out->pml4_table_phys = out->root_phys;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    kernel_cr3 = paging_get_kernel_pml4_phys() & AYKEN_PTE_ADDR_MASK;
    if (kernel_cr3 != 0 &&
        ((active_cr3 & AYKEN_PTE_ADDR_MASK) != kernel_cr3)) {
        __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
        __asm__ volatile("cli");
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
        switched_to_kernel_cr3 = 1;
    }

    pml4 = (uint64_t *)paging_phys_to_virt(out->root_phys);
    if (!pml4) {
        goto out;
    }

    pml4_i = (uint16_t)((va >> 39) & 0x1FF);
    pdpt_i = (uint16_t)((va >> 30) & 0x1FF);
    pd_i = (uint16_t)((va >> 21) & 0x1FF);
    pt_i = (uint16_t)((va >> 12) & 0x1FF);

    out->pml4e_phys = out->pml4_table_phys + ((uint64_t)pml4_i * sizeof(uint64_t));
    out->pml4e = pml4[pml4_i];
    if ((out->pml4e & AYKEN_PTE_PRESENT) == 0) {
        goto out;
    }

    out->pdpt_table_phys = out->pml4e & AYKEN_PTE_ADDR_MASK;
    {
        uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(out->pdpt_table_phys);
        if (!pdpt) {
            goto out;
        }
        out->pdpte_phys = out->pdpt_table_phys + ((uint64_t)pdpt_i * sizeof(uint64_t));
        out->pdpte = pdpt[pdpt_i];
        if ((out->pdpte & AYKEN_PTE_PRESENT) == 0) {
            goto out;
        }
        if (out->pdpte & (1ULL << 7)) {
            out->final_phys = (out->pdpte & AYKEN_PTE_ADDR_MASK) | (va & ((1ULL << 30) - 1));
            out->valid = 1;
            goto out;
        }
    }

    out->pd_table_phys = out->pdpte & AYKEN_PTE_ADDR_MASK;
    {
        uint64_t *pd = (uint64_t *)paging_phys_to_virt(out->pd_table_phys);
        if (!pd) {
            goto out;
        }
        out->pde_phys = out->pd_table_phys + ((uint64_t)pd_i * sizeof(uint64_t));
        out->pde = pd[pd_i];
        if ((out->pde & AYKEN_PTE_PRESENT) == 0) {
            goto out;
        }
        if (out->pde & (1ULL << 7)) {
            out->final_phys = (out->pde & AYKEN_PTE_ADDR_MASK) | (va & ((1ULL << 21) - 1));
            out->valid = 1;
            goto out;
        }
    }

    out->pt_table_phys = out->pde & AYKEN_PTE_ADDR_MASK;
    {
        uint64_t *pt = (uint64_t *)paging_phys_to_virt(out->pt_table_phys);
        if (!pt) {
            goto out;
        }
        out->pte_phys = out->pt_table_phys + ((uint64_t)pt_i * sizeof(uint64_t));
        out->pte = pt[pt_i];
        if ((out->pte & AYKEN_PTE_PRESENT) == 0) {
            goto out;
        }
    }

    out->final_phys = (out->pte & AYKEN_PTE_ADDR_MASK) | (va & (AYKEN_FRAME_SIZE - 1));
    out->valid = 1;
out:
    if (switched_to_kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
        if (saved_rflags & (1ULL << 9)) {
            __asm__ volatile("sti");
        }
    }

    return out->valid ? 1 : 0;
}

static void __attribute__((unused)) sched_emit_walk_snapshot_line(const char *tag,
                                                                  const sched_walk_snapshot_t *snap)
{
#if AYKEN_DEBUG_SCHED
    if (!tag || !snap) {
        return;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" OK=");
    sched_emit_u64_dec((uint64_t)(snap->valid ? 1u : 0u));
    sched_dbg_emit_text(" R=");
    dbg_out_hex64(snap->root_phys);
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(snap->va);
    sched_dbg_emit_text(" 4T=");
    dbg_out_hex64(snap->pml4_table_phys);
    sched_dbg_emit_text(" 4A=");
    dbg_out_hex64(snap->pml4e_phys);
    sched_dbg_emit_text(" 4E=");
    dbg_out_hex64(snap->pml4e);
    sched_dbg_emit_text(" 3T=");
    dbg_out_hex64(snap->pdpt_table_phys);
    sched_dbg_emit_text(" 3A=");
    dbg_out_hex64(snap->pdpte_phys);
    sched_dbg_emit_text(" 3E=");
    dbg_out_hex64(snap->pdpte);
    sched_dbg_emit_text(" 2T=");
    dbg_out_hex64(snap->pd_table_phys);
    sched_dbg_emit_text(" 2A=");
    dbg_out_hex64(snap->pde_phys);
    sched_dbg_emit_text(" 2E=");
    dbg_out_hex64(snap->pde);
    sched_dbg_emit_text(" 1T=");
    dbg_out_hex64(snap->pt_table_phys);
    sched_dbg_emit_text(" 1A=");
    dbg_out_hex64(snap->pte_phys);
    sched_dbg_emit_text(" 1E=");
    dbg_out_hex64(snap->pte);
    sched_dbg_emit_text(" FPA=");
    dbg_out_hex64(snap->final_phys);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)snap;
#endif
}

static uint8_t sched_walk_reserved_suspect(uint64_t entry)
{
    const uint64_t allowed =
        AYKEN_PTE_ADDR_MASK |
        AYKEN_PTE_PRESENT |
        AYKEN_PTE_WRITABLE |
        AYKEN_PTE_USER |
        AYKEN_PTE_GLOBAL |
        AYKEN_PTE_NO_EXEC |
        (1ULL << 3) |
        (1ULL << 4) |
        (1ULL << 5) |
        (1ULL << 6) |
        (1ULL << 7);

    return (uint8_t)((entry & ~allowed) != 0);
}

static uint8_t __attribute__((unused)) sched_walk_exec_ok(uint64_t entry)
{
    if ((entry & AYKEN_PTE_PRESENT) == 0) {
        return 0;
    }
    if ((entry & AYKEN_PTE_NO_EXEC) != 0) {
        return 0;
    }
    return (uint8_t)(sched_walk_reserved_suspect(entry) == 0);
}

static void __attribute__((unused)) sched_emit_walk_level_semantics(char level_tag,
                                                                    uint64_t entry,
                                                                    uint8_t leaf)
{
#if AYKEN_DEBUG_SCHED
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & AYKEN_PTE_PRESENT) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'W');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & AYKEN_PTE_WRITABLE) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'U');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & AYKEN_PTE_USER) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'N');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & AYKEN_PTE_NO_EXEC) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'G');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & AYKEN_PTE_GLOBAL) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'H');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & (1ULL << 7)) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'A');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & (1ULL << 5)) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'D');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((entry & (1ULL << 6)) ? 1u : 0u);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((uint64_t)sched_walk_reserved_suspect(entry));
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'X');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((uint64_t)sched_walk_exec_ok(entry));
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'L');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((uint64_t)leaf);
#else
    (void)level_tag;
    (void)entry;
    (void)leaf;
#endif
}

static void __attribute__((unused)) sched_emit_walk_semantics_line(const char *tag,
                                                                   const sched_walk_snapshot_t *snap)
{
#if AYKEN_DEBUG_SCHED
    uint8_t leaf_3;
    uint8_t leaf_2;
    uint8_t leaf_1;

    if (!tag || !snap) {
        return;
    }

    leaf_3 = (uint8_t)((snap->pdpte & AYKEN_PTE_PRESENT) && (snap->pdpte & (1ULL << 7)));
    leaf_2 = (uint8_t)((snap->pde & AYKEN_PTE_PRESENT) && (snap->pde & (1ULL << 7)));
    leaf_1 = (uint8_t)((snap->pte & AYKEN_PTE_PRESENT) != 0);

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" OK=");
    sched_emit_u64_dec((uint64_t)(snap->valid ? 1u : 0u));
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(snap->va);
    sched_emit_walk_level_semantics('4', snap->pml4e, 0);
    sched_emit_walk_level_semantics('3', snap->pdpte, leaf_3);
    sched_emit_walk_level_semantics('2', snap->pde, leaf_2);
    sched_emit_walk_level_semantics('1', snap->pte, leaf_1);
    sched_dbg_emit_text(" FPA=");
    dbg_out_hex64(snap->final_phys);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)snap;
#endif
}

static void __attribute__((unused)) sched_emit_pte_compare_side(char side_tag,
                                                                uint64_t entry)
{
#if AYKEN_DEBUG_SCHED
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)side_tag);
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(entry);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)side_tag);
    SCHED_DBG_OUT((uint8_t)'F');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(entry & AYKEN_PTE_ADDR_MASK);
    sched_emit_walk_level_semantics(side_tag, entry, (uint8_t)((entry & AYKEN_PTE_PRESENT) != 0));
#else
    (void)side_tag;
    (void)entry;
#endif
}

static void __attribute__((unused)) sched_emit_pte_compare_line(const char *tag,
                                                                uint64_t va,
                                                                uint64_t kernel_entry,
                                                                uint64_t target_entry)
{
#if AYKEN_DEBUG_SCHED
    if (!tag) {
        return;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(va);
    sched_dbg_emit_text(" EQ=");
    sched_emit_u64_dec((uint64_t)(kernel_entry == target_entry));
    sched_emit_pte_compare_side('K', kernel_entry);
    sched_emit_pte_compare_side('T', target_entry);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)va;
    (void)kernel_entry;
    (void)target_entry;
#endif
}

static uint64_t __attribute__((unused)) sched_walk_level_child_frame(
    const sched_walk_snapshot_t *snap,
    char level_tag)
{
    if (!snap) {
        return 0;
    }

    switch (level_tag) {
    case '4':
        return snap->pdpt_table_phys;
    case '3':
        return snap->pd_table_phys;
    case '2':
        return snap->pt_table_phys;
    case '1':
        return snap->final_phys & AYKEN_PTE_ADDR_MASK;
    default:
        return 0;
    }
}

static uint64_t __attribute__((unused)) sched_walk_level_entry(
    const sched_walk_snapshot_t *snap,
    char level_tag)
{
    if (!snap) {
        return 0;
    }

    switch (level_tag) {
    case '4':
        return snap->pml4e;
    case '3':
        return snap->pdpte;
    case '2':
        return snap->pde;
    case '1':
        return snap->pte;
    default:
        return 0;
    }
}

static uint64_t __attribute__((unused)) sched_walk_entry_semantics(uint64_t entry)
{
    return entry & ~AYKEN_PTE_ADDR_MASK;
}

static void __attribute__((unused)) sched_emit_chain_compare_level(char level_tag,
                                                                   uint64_t kernel_entry,
                                                                   uint64_t kernel_frame,
                                                                   uint64_t target_entry,
                                                                   uint64_t target_frame)
{
#if AYKEN_DEBUG_SCHED
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'Q');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((uint64_t)(
        sched_walk_entry_semantics(kernel_entry) ==
        sched_walk_entry_semantics(target_entry)));
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'F');
    SCHED_DBG_OUT((uint8_t)'Q');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((uint64_t)(kernel_frame == target_frame));
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'K');
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(kernel_entry);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'K');
    SCHED_DBG_OUT((uint8_t)'F');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(kernel_frame);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)'E');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(target_entry);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)'F');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(target_frame);
#else
    (void)level_tag;
    (void)kernel_entry;
    (void)kernel_frame;
    (void)target_entry;
    (void)target_frame;
#endif
}

static void __attribute__((unused)) sched_emit_chain_compare_line(
    const char *tag,
    const sched_walk_snapshot_t *kernel_snap,
    const sched_walk_snapshot_t *target_snap)
{
#if AYKEN_DEBUG_SCHED
    if (!tag || !kernel_snap || !target_snap) {
        return;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(target_snap->va);
    sched_dbg_emit_text(" OK=");
    sched_emit_u64_dec((uint64_t)(kernel_snap->valid && target_snap->valid));
    sched_emit_chain_compare_level(
        '4',
        sched_walk_level_entry(kernel_snap, '4'),
        sched_walk_level_child_frame(kernel_snap, '4'),
        sched_walk_level_entry(target_snap, '4'),
        sched_walk_level_child_frame(target_snap, '4'));
    sched_emit_chain_compare_level(
        '3',
        sched_walk_level_entry(kernel_snap, '3'),
        sched_walk_level_child_frame(kernel_snap, '3'),
        sched_walk_level_entry(target_snap, '3'),
        sched_walk_level_child_frame(target_snap, '3'));
    sched_emit_chain_compare_level(
        '2',
        sched_walk_level_entry(kernel_snap, '2'),
        sched_walk_level_child_frame(kernel_snap, '2'),
        sched_walk_level_entry(target_snap, '2'),
        sched_walk_level_child_frame(target_snap, '2'));
    sched_emit_chain_compare_level(
        '1',
        sched_walk_level_entry(kernel_snap, '1'),
        sched_walk_level_child_frame(kernel_snap, '1'),
        sched_walk_level_entry(target_snap, '1'),
        sched_walk_level_child_frame(target_snap, '1'));
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)kernel_snap;
    (void)target_snap;
#endif
}

static void __attribute__((unused)) sched_emit_walk_level_detail_line(const char *tag,
                                                                      char level_tag,
                                                                      uint64_t index,
                                                                      uint64_t root_phys,
                                                                      uint64_t va,
                                                                      uint64_t table_phys,
                                                                      uint64_t entry_phys,
                                                                      uint64_t entry,
                                                                      uint64_t child_phys)
{
#if AYKEN_DEBUG_SCHED
    uint8_t leaf = (uint8_t)((level_tag == '1') ? ((entry & AYKEN_PTE_PRESENT) != 0) :
                              ((entry & AYKEN_PTE_PRESENT) && (entry & (1ULL << 7))));

    if (!tag) {
        return;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" R=");
    dbg_out_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(va);
    sched_dbg_emit_text(" I=");
    dbg_out_hex64(index);
    sched_dbg_emit_text(" TP=");
    dbg_out_hex64(table_phys & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" EP=");
    dbg_out_hex64(entry_phys & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" E=");
    dbg_out_hex64(entry);
    sched_dbg_emit_text(" CF=");
    dbg_out_hex64(child_phys & AYKEN_PTE_ADDR_MASK);
    sched_emit_walk_level_semantics(level_tag, entry, leaf);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)level_tag;
    (void)index;
    (void)root_phys;
    (void)va;
    (void)table_phys;
    (void)entry_phys;
    (void)entry;
    (void)child_phys;
#endif
}

static void sched_emit_pre_dispatch_text_walk_proof(const proc_t *proc)
{
#if AYKEN_DEBUG_SCHED
    sched_walk_snapshot_t target_text_walk;
    uint64_t active_cr3 = 0;
    uint64_t kernel_cr3;
    uint64_t target_cr3;
    uint64_t active_text_pte;
    uint64_t target_text_pte;
    uint64_t text_phys;
    uint64_t pml4_i;
    uint64_t pdpt_i;
    uint64_t pd_i;
    uint64_t pt_i;

    if (!proc || ((proc->context.cs & 0x3u) != 0x3u) || proc->context.cr3 == 0) {
        return;
    }

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));

    kernel_cr3 = paging_get_kernel_pml4_phys() & AYKEN_PTE_ADDR_MASK;
    target_cr3 = proc->context.cr3 & AYKEN_PTE_ADDR_MASK;
    active_text_pte = paging_get_pte(proc->context.rip);
    target_text_pte = paging_get_pte_in_pml4(target_cr3, proc->context.rip);
    text_phys = target_text_pte & AYKEN_PTE_ADDR_MASK;
    sched_capture_walk_snapshot(target_cr3, proc->context.rip, &target_text_walk);

    pml4_i = (proc->context.rip >> 39) & 0x1FFu;
    pdpt_i = (proc->context.rip >> 30) & 0x1FFu;
    pd_i = (proc->context.rip >> 21) & 0x1FFu;
    pt_i = (proc->context.rip >> 12) & 0x1FFu;

    sched_emit_marker("P10_TEXT_ROOT_PROOF AC=");
    dbg_out_hex64(active_cr3);
    sched_emit_marker(" KC=");
    dbg_out_hex64(kernel_cr3);
    sched_emit_marker(" TC=");
    dbg_out_hex64(target_cr3);
    sched_emit_marker(" RC=");
    dbg_out_hex64(target_text_walk.root_phys);
    sched_emit_marker(" RIP=");
    dbg_out_hex64(proc->context.rip);
    sched_emit_marker(" AP=");
    dbg_out_hex64(active_text_pte);
    sched_emit_marker(" TP=");
    dbg_out_hex64(target_text_pte);
    sched_emit_marker("\n");
    sched_emit_phys_frame_witness(
        "P10_ROOT_FRAME_WITNESS",
        "pre_dispatch",
        target_cr3,
        0,
        target_cr3);
    sched_emit_phys_frame_witness(
        "P10_TEXT_FRAME_WITNESS",
        "pre_dispatch",
        target_cr3,
        target_text_pte,
        text_phys);

    sched_emit_walk_level_detail_line("TX4",
                                      '4',
                                      pml4_i,
                                      target_cr3,
                                      proc->context.rip,
                                      target_text_walk.pml4_table_phys,
                                      target_text_walk.pml4e_phys,
                                      target_text_walk.pml4e,
                                      sched_walk_level_child_frame(&target_text_walk, '4'));
    sched_emit_walk_level_detail_line("TX3",
                                      '3',
                                      pdpt_i,
                                      target_cr3,
                                      proc->context.rip,
                                      target_text_walk.pdpt_table_phys,
                                      target_text_walk.pdpte_phys,
                                      target_text_walk.pdpte,
                                      sched_walk_level_child_frame(&target_text_walk, '3'));
    sched_emit_walk_level_detail_line("TX2",
                                      '2',
                                      pd_i,
                                      target_cr3,
                                      proc->context.rip,
                                      target_text_walk.pd_table_phys,
                                      target_text_walk.pde_phys,
                                      target_text_walk.pde,
                                      sched_walk_level_child_frame(&target_text_walk, '2'));
    sched_emit_walk_level_detail_line("TX1",
                                      '1',
                                      pt_i,
                                      target_cr3,
                                      proc->context.rip,
                                      target_text_walk.pt_table_phys,
                                      target_text_walk.pte_phys,
                                      target_text_walk.pte,
                                      sched_walk_level_child_frame(&target_text_walk, '1'));
    sched_emit_pte_compare_line("TXP", proc->context.rip, active_text_pte, target_text_pte);
#else
    (void)proc;
#endif
}

static const uint64_t *__attribute__((unused)) sched_root_diff_table_page(uint64_t phys)
{
    if ((phys & AYKEN_PTE_ADDR_MASK) == 0) {
        return NULL;
    }
    return (const uint64_t *)sched_dbg_identity_page(phys & AYKEN_PTE_ADDR_MASK);
}

static void __attribute__((unused)) sched_count_table_diffs(const uint64_t *lhs,
                                                            const uint64_t *rhs,
                                                            uint32_t start,
                                                            uint32_t end,
                                                            uint32_t *out_diff,
                                                            uint32_t *out_sem_diff,
                                                            uint32_t *out_pfn_diff,
                                                            uint32_t *out_present_diff)
{
    uint32_t diff = 0;
    uint32_t sem_diff = 0;
    uint32_t pfn_diff = 0;
    uint32_t present_diff = 0;

    if (!lhs || !rhs || start >= AYKEN_SCHED_PT_ENTRIES ||
        end >= AYKEN_SCHED_PT_ENTRIES || start > end) {
        if (out_diff) {
            *out_diff = 0;
        }
        if (out_sem_diff) {
            *out_sem_diff = 0;
        }
        if (out_pfn_diff) {
            *out_pfn_diff = 0;
        }
        if (out_present_diff) {
            *out_present_diff = 0;
        }
        return;
    }

    for (uint32_t i = start; i <= end; ++i) {
        uint64_t lhs_entry = lhs[i];
        uint64_t rhs_entry = rhs[i];

        if (lhs_entry != rhs_entry) {
            ++diff;
        }
        if (sched_walk_entry_semantics(lhs_entry) != sched_walk_entry_semantics(rhs_entry)) {
            ++sem_diff;
        }
        if ((lhs_entry & AYKEN_PTE_ADDR_MASK) != (rhs_entry & AYKEN_PTE_ADDR_MASK)) {
            ++pfn_diff;
        }
        if (((lhs_entry ^ rhs_entry) & AYKEN_PTE_PRESENT) != 0) {
            ++present_diff;
        }
    }

    if (out_diff) {
        *out_diff = diff;
    }
    if (out_sem_diff) {
        *out_sem_diff = sem_diff;
    }
    if (out_pfn_diff) {
        *out_pfn_diff = pfn_diff;
    }
    if (out_present_diff) {
        *out_present_diff = present_diff;
    }
}

static void __attribute__((unused)) sched_emit_root_diff_summary_line(const char *tag,
                                                                      uint64_t lhs_root,
                                                                      uint64_t rhs_root)
{
#if AYKEN_DEBUG_SCHED
    const uint64_t *lhs;
    const uint64_t *rhs;
    uint32_t total_diff = 0;
    uint32_t total_sem_diff = 0;
    uint32_t total_pfn_diff = 0;
    uint32_t total_present_diff = 0;
    uint32_t lower_diff = 0;
    uint32_t lower_sem_diff = 0;
    uint32_t lower_pfn_diff = 0;
    uint32_t lower_present_diff = 0;
    uint32_t upper_diff = 0;
    uint32_t upper_sem_diff = 0;
    uint32_t upper_pfn_diff = 0;
    uint32_t upper_present_diff = 0;

    if (!tag) {
        return;
    }

    lhs = sched_root_diff_table_page(lhs_root);
    rhs = sched_root_diff_table_page(rhs_root);
    if (!lhs || !rhs) {
        return;
    }

    sched_count_table_diffs(lhs,
                            rhs,
                            0,
                            AYKEN_SCHED_PT_ENTRIES - 1,
                            &total_diff,
                            &total_sem_diff,
                            &total_pfn_diff,
                            &total_present_diff);
    sched_count_table_diffs(lhs,
                            rhs,
                            0,
                            (AYKEN_SCHED_PT_ENTRIES / 2) - 1,
                            &lower_diff,
                            &lower_sem_diff,
                            &lower_pfn_diff,
                            &lower_present_diff);
    sched_count_table_diffs(lhs,
                            rhs,
                            AYKEN_SCHED_PT_ENTRIES / 2,
                            AYKEN_SCHED_PT_ENTRIES - 1,
                            &upper_diff,
                            &upper_sem_diff,
                            &upper_pfn_diff,
                            &upper_present_diff);

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" LR=");
    dbg_out_hex64(lhs_root & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" RR=");
    dbg_out_hex64(rhs_root & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" D=");
    sched_emit_u64_dec(total_diff);
    sched_dbg_emit_text(" SD=");
    sched_emit_u64_dec(total_sem_diff);
    sched_dbg_emit_text(" FD=");
    sched_emit_u64_dec(total_pfn_diff);
    sched_dbg_emit_text(" PD=");
    sched_emit_u64_dec(total_present_diff);
    sched_dbg_emit_text(" LD=");
    sched_emit_u64_dec(lower_diff);
    sched_dbg_emit_text(" LSD=");
    sched_emit_u64_dec(lower_sem_diff);
    sched_dbg_emit_text(" LFD=");
    sched_emit_u64_dec(lower_pfn_diff);
    sched_dbg_emit_text(" LPD=");
    sched_emit_u64_dec(lower_present_diff);
    sched_dbg_emit_text(" UD=");
    sched_emit_u64_dec(upper_diff);
    sched_dbg_emit_text(" USD=");
    sched_emit_u64_dec(upper_sem_diff);
    sched_dbg_emit_text(" UFD=");
    sched_emit_u64_dec(upper_pfn_diff);
    sched_dbg_emit_text(" UPD=");
    sched_emit_u64_dec(upper_present_diff);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)lhs_root;
    (void)rhs_root;
#endif
}

static void __attribute__((unused)) sched_emit_table_diff_entry_line(const char *tag,
                                                                     uint8_t level,
                                                                     uint16_t p4,
                                                                     uint16_t p3,
                                                                     uint16_t p2,
                                                                     uint16_t index,
                                                                     uint64_t lhs_entry,
                                                                     uint64_t rhs_entry)
{
#if AYKEN_DEBUG_SCHED
    if (!tag) {
        return;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" L=");
    sched_emit_u64_dec(level);
    sched_dbg_emit_text(" P4=");
    sched_emit_u64_dec(p4);
    sched_dbg_emit_text(" P3=");
    sched_emit_u64_dec((p3 == 0xFFFFu) ? 0xFFFFu : p3);
    sched_dbg_emit_text(" P2=");
    sched_emit_u64_dec((p2 == 0xFFFFu) ? 0xFFFFu : p2);
    sched_dbg_emit_text(" I=");
    sched_emit_u64_dec(index);
    sched_dbg_emit_text(" EQ=");
    sched_emit_u64_dec((uint64_t)(lhs_entry == rhs_entry));
    sched_dbg_emit_text(" FQ=");
    sched_emit_u64_dec((uint64_t)(
        (lhs_entry & AYKEN_PTE_ADDR_MASK) == (rhs_entry & AYKEN_PTE_ADDR_MASK)));
    sched_emit_pte_compare_side('K', lhs_entry);
    sched_emit_pte_compare_side('T', rhs_entry);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)level;
    (void)p4;
    (void)p3;
    (void)p2;
    (void)index;
    (void)lhs_entry;
    (void)rhs_entry;
#endif
}

static void __attribute__((unused)) sched_emit_subtree_diff_recursive(const char *summary_tag,
                                                                      const char *entry_tag,
                                                                      uint8_t level,
                                                                      uint16_t p4,
                                                                      uint16_t p3,
                                                                      uint16_t p2,
                                                                      uint64_t lhs_table_phys,
                                                                      uint64_t rhs_table_phys)
{
#if AYKEN_DEBUG_SCHED
    const uint64_t *lhs;
    const uint64_t *rhs;
    uint32_t diff = 0;
    uint32_t sem_diff = 0;
    uint32_t pfn_diff = 0;
    uint32_t present_diff = 0;

    lhs = sched_root_diff_table_page(lhs_table_phys);
    rhs = sched_root_diff_table_page(rhs_table_phys);
    if (!lhs || !rhs) {
        return;
    }

    sched_count_table_diffs(lhs,
                            rhs,
                            0,
                            AYKEN_SCHED_PT_ENTRIES - 1,
                            &diff,
                            &sem_diff,
                            &pfn_diff,
                            &present_diff);

    if (summary_tag) {
        const char *summary_ptr = summary_tag;
        SCHED_DBG_OUT((uint8_t)'W');
        while (*summary_ptr) {
            SCHED_DBG_OUT((uint8_t)*summary_ptr++);
        }
        sched_dbg_emit_text(" L=");
        sched_emit_u64_dec(level);
        sched_dbg_emit_text(" P4=");
        sched_emit_u64_dec(p4);
        sched_dbg_emit_text(" P3=");
        sched_emit_u64_dec((p3 == 0xFFFFu) ? 0xFFFFu : p3);
        sched_dbg_emit_text(" P2=");
        sched_emit_u64_dec((p2 == 0xFFFFu) ? 0xFFFFu : p2);
        sched_dbg_emit_text(" D=");
        sched_emit_u64_dec(diff);
        sched_dbg_emit_text(" SD=");
        sched_emit_u64_dec(sem_diff);
        sched_dbg_emit_text(" FD=");
        sched_emit_u64_dec(pfn_diff);
        sched_dbg_emit_text(" PD=");
        sched_emit_u64_dec(present_diff);
        sched_dbg_emit_text(" KF=");
        dbg_out_hex64(lhs_table_phys & AYKEN_PTE_ADDR_MASK);
        sched_dbg_emit_text(" TF=");
        dbg_out_hex64(rhs_table_phys & AYKEN_PTE_ADDR_MASK);
        sched_dbg_emit_text("\n");
    }

    if (diff == 0) {
        return;
    }

    for (uint16_t i = 0; i < AYKEN_SCHED_PT_ENTRIES; ++i) {
        uint64_t lhs_entry = lhs[i];
        uint64_t rhs_entry = rhs[i];
        uint16_t next_p3 = p3;
        uint16_t next_p2 = p2;

        if (lhs_entry == rhs_entry) {
            continue;
        }

        sched_emit_table_diff_entry_line(entry_tag, level, p4, p3, p2, i, lhs_entry, rhs_entry);

        if (level <= 1) {
            continue;
        }
        if ((lhs_entry & AYKEN_PTE_PRESENT) == 0 || (rhs_entry & AYKEN_PTE_PRESENT) == 0) {
            continue;
        }
        if ((lhs_entry & (1ULL << 7)) != 0 || (rhs_entry & (1ULL << 7)) != 0) {
            continue;
        }

        if (level == 3) {
            next_p3 = i;
        } else if (level == 2) {
            next_p2 = i;
        }

        sched_emit_subtree_diff_recursive(summary_tag,
                                          entry_tag,
                                          (uint8_t)(level - 1),
                                          p4,
                                          next_p3,
                                          next_p2,
                                          lhs_entry & AYKEN_PTE_ADDR_MASK,
                                          rhs_entry & AYKEN_PTE_ADDR_MASK);
    }
#else
    (void)summary_tag;
    (void)entry_tag;
    (void)level;
    (void)p4;
    (void)p3;
    (void)p2;
    (void)lhs_table_phys;
    (void)rhs_table_phys;
#endif
}

static void __attribute__((unused)) sched_emit_lower_half_root_diff(const char *summary_tag,
                                                                    const char *entry_tag,
                                                                    uint64_t lhs_root,
                                                                    uint64_t rhs_root)
{
#if AYKEN_DEBUG_SCHED
    const uint64_t *lhs;
    const uint64_t *rhs;

    lhs = sched_root_diff_table_page(lhs_root);
    rhs = sched_root_diff_table_page(rhs_root);
    if (!lhs || !rhs) {
        return;
    }

    for (uint16_t i = 0; i < AYKEN_SCHED_PT_ENTRIES / 2; ++i) {
        uint64_t lhs_entry = lhs[i];
        uint64_t rhs_entry = rhs[i];

        if (lhs_entry == rhs_entry) {
            continue;
        }

        sched_emit_table_diff_entry_line(entry_tag, 4, i, 0xFFFFu, 0xFFFFu, i, lhs_entry, rhs_entry);
        if ((lhs_entry & AYKEN_PTE_PRESENT) == 0 || (rhs_entry & AYKEN_PTE_PRESENT) == 0) {
            continue;
        }
        if ((lhs_entry & (1ULL << 7)) != 0 || (rhs_entry & (1ULL << 7)) != 0) {
            continue;
        }

        sched_emit_subtree_diff_recursive(summary_tag,
                                          entry_tag,
                                          3,
                                          i,
                                          0xFFFFu,
                                          0xFFFFu,
                                          lhs_entry & AYKEN_PTE_ADDR_MASK,
                                          rhs_entry & AYKEN_PTE_ADDR_MASK);
    }
#else
    (void)summary_tag;
    (void)entry_tag;
    (void)lhs_root;
    (void)rhs_root;
#endif
}

static void __attribute__((unused)) sched_emit_root_surface_compare(const char *tag,
                                                                    uint64_t lhs_root,
                                                                    uint64_t rhs_root,
                                                                    uint64_t va)
{
#if AYKEN_DEBUG_SCHED
    sched_walk_snapshot_t lhs_snap;
    sched_walk_snapshot_t rhs_snap;
    char pte_tag[8] = {'P', 'S', 'U', 'R', 'F', '\0', '\0', '\0'};
    uint32_t i = 0;

    sched_capture_walk_snapshot(lhs_root, va, &lhs_snap);
    sched_capture_walk_snapshot(rhs_root, va, &rhs_snap);
    sched_emit_chain_compare_line(tag, &lhs_snap, &rhs_snap);
    if (tag) {
        for (i = 0; i < 4 && tag[i] != '\0'; ++i) {
            pte_tag[i + 1] = tag[i];
        }
        pte_tag[i + 1] = '\0';
    }
    sched_emit_pte_compare_line(pte_tag, va, lhs_snap.pte, rhs_snap.pte);
#else
    (void)tag;
    (void)lhs_root;
    (void)rhs_root;
    (void)va;
#endif
}

static void __attribute__((unused)) sched_emit_cr3_consistency_line(const char *tag,
                                                                    uint64_t expected_root,
                                                                    uint64_t kernel_root,
                                                                    uint64_t active_root)
{
#if AYKEN_DEBUG_SCHED
    if (!tag) {
        return;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" E=");
    dbg_out_hex64(expected_root & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" K=");
    dbg_out_hex64(kernel_root & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" A=");
    dbg_out_hex64(active_root & AYKEN_PTE_ADDR_MASK);
    sched_dbg_emit_text(" AE=");
    sched_emit_u64_dec((uint64_t)(
        (active_root & AYKEN_PTE_ADDR_MASK) == (expected_root & AYKEN_PTE_ADDR_MASK)));
    sched_dbg_emit_text(" AK=");
    sched_emit_u64_dec((uint64_t)(
        (active_root & AYKEN_PTE_ADDR_MASK) == (kernel_root & AYKEN_PTE_ADDR_MASK)));
    sched_dbg_emit_text(" EK=");
    sched_emit_u64_dec((uint64_t)(
        (expected_root & AYKEN_PTE_ADDR_MASK) == (kernel_root & AYKEN_PTE_ADDR_MASK)));
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)expected_root;
    (void)kernel_root;
    (void)active_root;
#endif
}

static uint8_t __attribute__((unused)) sched_walk_snapshot_matches(
    const sched_walk_snapshot_t *lhs,
    const sched_walk_snapshot_t *rhs)
{
    if (!lhs || !rhs) {
        return 0;
    }

    return (uint8_t)(
        lhs->root_phys == rhs->root_phys &&
        lhs->va == rhs->va &&
        lhs->pml4_table_phys == rhs->pml4_table_phys &&
        lhs->pml4e_phys == rhs->pml4e_phys &&
        lhs->pml4e == rhs->pml4e &&
        lhs->pdpt_table_phys == rhs->pdpt_table_phys &&
        lhs->pdpte_phys == rhs->pdpte_phys &&
        lhs->pdpte == rhs->pdpte &&
        lhs->pd_table_phys == rhs->pd_table_phys &&
        lhs->pde_phys == rhs->pde_phys &&
        lhs->pde == rhs->pde &&
        lhs->pt_table_phys == rhs->pt_table_phys &&
        lhs->pte_phys == rhs->pte_phys &&
        lhs->pte == rhs->pte &&
        lhs->final_phys == rhs->final_phys &&
        lhs->valid == rhs->valid);
}

static void __attribute__((unused)) sched_emit_walk_integrity_field(char level_tag,
                                                                    uint64_t phys_page)
{
#if AYKEN_DEBUG_SCHED
    int used = phys_page ? phys_frame_is_used(phys_page) : 0;
    uint64_t digest = phys_page ? sched_dbg_hash_phys_page_identity(phys_page) : 0;

    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'F');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(phys_page);
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'U');
    SCHED_DBG_OUT((uint8_t)'=');
    sched_emit_u64_dec((uint64_t)((phys_page != 0 && used == 1) ? 1u : 0u));
    sched_dbg_emit_text(" ");
    SCHED_DBG_OUT((uint8_t)level_tag);
    SCHED_DBG_OUT((uint8_t)'D');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(digest);
#else
    (void)level_tag;
    (void)phys_page;
#endif
}

static void __attribute__((unused)) sched_emit_walk_integrity_line(const char *tag,
                                                                   const sched_walk_snapshot_t *snap)
{
#if AYKEN_DEBUG_SCHED
    sched_walk_snapshot_t identity_snap;
    uint8_t identity_walk_ok = 0;
    uint8_t identity_match = 0;
    uint64_t leaf_page_phys = 0;

    if (!tag || !snap) {
        return;
    }

    if (snap->root_phys != 0 &&
        sched_capture_identity_walk_snapshot(snap->root_phys, snap->va, &identity_snap)) {
        identity_walk_ok = 1;
        identity_match = sched_walk_snapshot_matches(snap, &identity_snap);
    }

    if (snap->valid) {
        leaf_page_phys = snap->final_phys & AYKEN_PTE_ADDR_MASK;
    }

    SCHED_DBG_OUT((uint8_t)'W');
    while (*tag) {
        SCHED_DBG_OUT((uint8_t)*tag++);
    }
    sched_dbg_emit_text(" OK=");
    sched_emit_u64_dec((uint64_t)(snap->valid ? 1u : 0u));
    sched_dbg_emit_text(" IW=");
    sched_emit_u64_dec((uint64_t)identity_walk_ok);
    sched_dbg_emit_text(" IM=");
    sched_emit_u64_dec((uint64_t)identity_match);
    sched_dbg_emit_text(" V=");
    dbg_out_hex64(snap->va);
    sched_emit_walk_integrity_field('4', snap->pml4_table_phys);
    sched_emit_walk_integrity_field('3', snap->pdpt_table_phys);
    sched_emit_walk_integrity_field('2', snap->pd_table_phys);
    sched_emit_walk_integrity_field('1', snap->pt_table_phys);
    sched_emit_walk_integrity_field('F', leaf_page_phys);
    sched_dbg_emit_text("\n");
#else
    (void)tag;
    (void)snap;
#endif
}

static void __attribute__((unused)) sched_ring3_diag_panic(const char *reason)
{
    fb_print("[PANIC] Ring3 diagnostic precondition failed: ");
    fb_print(reason ? reason : "unknown");
    fb_print("\n");
    __asm__ volatile("cli; 1: hlt; jmp 1b");
}

#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1) && \
    defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1) && \
    defined(AYKEN_RING3_FRESH_FRAME_PROBE) && (AYKEN_RING3_FRESH_FRAME_PROBE == 1)
static uint64_t g_sched_ring3_probe_frame_phys = 0;
static uint64_t g_sched_ring3_probe_source_frame_phys = 0;

static int sched_ring3_probe_frame_matches_symbol(uint64_t probe_frame_phys)
{
    const uint8_t *source_page =
        (const uint8_t *)((uintptr_t)ring3_enter_post_cr3 & ~(uintptr_t)(AYKEN_FRAME_SIZE - 1));
    const uint8_t *probe_page = (const uint8_t *)paging_phys_to_virt(probe_frame_phys);
    uint64_t i;

    if (!source_page || !probe_page) {
        return 0;
    }

    for (i = 0; i < AYKEN_FRAME_SIZE; ++i) {
        if (source_page[i] != probe_page[i]) {
            return 0;
        }
    }

    return 1;
}

static uint64_t sched_ring3_probe_mapping_phys(void)
{
    uint64_t source_frame_phys =
        paging_get_phys((uint64_t)(uintptr_t)ring3_enter_post_cr3) & AYKEN_PTE_ADDR_MASK;
    uint8_t *dst_page;
    const uint8_t *src_page;

    if (source_frame_phys == 0) {
        return 0;
    }

    if (g_sched_ring3_probe_frame_phys != 0 &&
        g_sched_ring3_probe_source_frame_phys == source_frame_phys &&
        sched_ring3_probe_frame_matches_symbol(g_sched_ring3_probe_frame_phys)) {
        return g_sched_ring3_probe_frame_phys;
    }

    if (g_sched_ring3_probe_frame_phys == 0) {
        g_sched_ring3_probe_frame_phys = phys_alloc_frame();
        if (g_sched_ring3_probe_frame_phys == 0) {
            return 0;
        }
    }

    src_page =
        (const uint8_t *)((uintptr_t)ring3_enter_post_cr3 & ~(uintptr_t)(AYKEN_FRAME_SIZE - 1));
    dst_page = (uint8_t *)paging_phys_to_virt(g_sched_ring3_probe_frame_phys);
    if (!src_page || !dst_page) {
        return 0;
    }

    memcpy(dst_page, src_page, AYKEN_FRAME_SIZE);
    g_sched_ring3_probe_source_frame_phys = source_frame_phys;
    return g_sched_ring3_probe_frame_phys;
}
#endif

#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1) && \
    defined(AYKEN_RING3_ALT_STAGEB_SOURCE) && (AYKEN_RING3_ALT_STAGEB_SOURCE == 1)
static uint64_t g_sched_ring3_alt_stageb_frame_phys = 0;
static uint64_t g_sched_ring3_alt_stageb_source_frame_phys = 0;

static int sched_ring3_alt_stageb_frame_matches_symbol(uint64_t probe_frame_phys)
{
    const uint8_t *source_page =
        (const uint8_t *)((uintptr_t)ring3_enter_alt_bridge_trampoline &
                          ~(uintptr_t)(AYKEN_FRAME_SIZE - 1));
    const uint8_t *probe_page = (const uint8_t *)paging_phys_to_virt(probe_frame_phys);
    uint64_t i;

    if (!source_page || !probe_page) {
        return 0;
    }

    for (i = 0; i < AYKEN_FRAME_SIZE; ++i) {
        if (source_page[i] != probe_page[i]) {
            return 0;
        }
    }

    return 1;
}

static uint64_t sched_ring3_alt_stageb_mapping_phys(void)
{
    uint64_t source_frame_phys =
        paging_get_phys((uint64_t)(uintptr_t)ring3_enter_alt_bridge_trampoline) &
        AYKEN_PTE_ADDR_MASK;
    uint8_t *dst_page;
    const uint8_t *src_page;

    if (source_frame_phys == 0) {
        return 0;
    }

    if (g_sched_ring3_alt_stageb_frame_phys != 0 &&
        g_sched_ring3_alt_stageb_source_frame_phys == source_frame_phys &&
        sched_ring3_alt_stageb_frame_matches_symbol(g_sched_ring3_alt_stageb_frame_phys)) {
        return g_sched_ring3_alt_stageb_frame_phys;
    }

    if (g_sched_ring3_alt_stageb_frame_phys == 0) {
        g_sched_ring3_alt_stageb_frame_phys = phys_alloc_frame_high();
        if (g_sched_ring3_alt_stageb_frame_phys == 0) {
            return 0;
        }
    }

    src_page =
        (const uint8_t *)((uintptr_t)ring3_enter_alt_bridge_trampoline &
                          ~(uintptr_t)(AYKEN_FRAME_SIZE - 1));
    dst_page = (uint8_t *)paging_phys_to_virt(g_sched_ring3_alt_stageb_frame_phys);
    if (!src_page || !dst_page) {
        return 0;
    }

    memcpy(dst_page, src_page, AYKEN_FRAME_SIZE);
    g_sched_ring3_alt_stageb_source_frame_phys = source_frame_phys;
    return g_sched_ring3_alt_stageb_frame_phys;
}
#endif

static void __attribute__((unused)) sched_debug_dump_walk(const char *tag,
                                                          uint64_t root_phys,
                                                          uint64_t va)
{
#if AYKEN_DEBUG_SCHED
    uint64_t *pml4;
    uint64_t pml4e = 0;
    uint64_t pdpte = 0;
    uint64_t pde = 0;
    uint64_t pte = 0;

    if (!root_phys) {
        return;
    }

    pml4 = (uint64_t *)paging_phys_to_virt(root_phys & AYKEN_PTE_ADDR_MASK);
    if (!pml4) {
        return;
    }

    {
        uint16_t pml4_i = (uint16_t)((va >> 39) & 0x1FF);
        uint16_t pdpt_i = (uint16_t)((va >> 30) & 0x1FF);
        uint16_t pd_i = (uint16_t)((va >> 21) & 0x1FF);
        uint16_t pt_i = (uint16_t)((va >> 12) & 0x1FF);

        pml4e = pml4[pml4_i];
        if (pml4e & AYKEN_PTE_PRESENT) {
            uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);
            if (pdpt) {
                pdpte = pdpt[pdpt_i];
                if ((pdpte & AYKEN_PTE_PRESENT) && ((pdpte & (1ULL << 7)) == 0)) {
                    uint64_t *pd = (uint64_t *)paging_phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);
                    if (pd) {
                        pde = pd[pd_i];
                        if ((pde & AYKEN_PTE_PRESENT) && ((pde & (1ULL << 7)) == 0)) {
                            uint64_t *pt = (uint64_t *)paging_phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);
                            if (pt) {
                                pte = pt[pt_i];
                            }
                        }
                    }
                }
            }
        }
    }

    SCHED_DBG_OUT((uint8_t)'W');
    if (tag) {
        while (*tag) {
            SCHED_DBG_OUT((uint8_t)*tag++);
        }
    }
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(root_phys);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'V');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(va);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'4');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(pml4e);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'3');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(pdpte);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'2');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(pde);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'1');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(pte);
    SCHED_DBG_OUT((uint8_t)'\n');
#else
    (void)tag;
    (void)root_phys;
    (void)va;
#endif
}

static uint64_t sched_ring3_stage_b_source_phys(void)
{
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1) && \
    defined(AYKEN_RING3_ALT_STAGEB_SOURCE) && (AYKEN_RING3_ALT_STAGEB_SOURCE == 1)
    return sched_ring3_alt_stageb_mapping_phys();
#else
    return paging_get_phys((uint64_t)(uintptr_t)ring3_enter_iret_trampoline);
#endif
}

static void sched_debug_ring3_entry_window(proc_t *proc)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    uint64_t active_rsp = 0;
    uint64_t active_cr3 = 0;
    uint64_t active_page;
    uint64_t text_page;
    uint64_t text_ip;
    uint64_t text_phys;
    uint64_t kernel_transition_pte;
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    uint64_t high_va;
    uint64_t high_pte;
    uint64_t high_phys;
#endif
#if (defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1) && \
     defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)) || \
    (defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1))
    uint64_t second_fetch_va;
    sched_walk_snapshot_t target_second_walk;
    sched_walk_snapshot_t kernel_second_walk;
#endif
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    uint64_t third_fetch_va;
    sched_walk_snapshot_t target_third_walk;
    sched_walk_snapshot_t kernel_third_walk;
#endif
#if defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    uint64_t low_fetch_va;
    sched_walk_snapshot_t target_low_walk;
    sched_walk_snapshot_t kernel_low_walk;
#endif
    uint64_t user_text_lo = 0;
    uint64_t user_text_hi = 0;
    uint64_t frame_text_lo = 0;
    uint64_t frame_text_hi = 0;
    uint64_t symbol_text_lo = 0;
    uint64_t symbol_text_hi = 0;
    int user_text_lo_ok;
    int user_text_hi_ok;
    uint64_t idt_pf_va;
    uint64_t canonical_fetch_va;
    uint64_t rsp0_page;
    uint64_t cr3;
    uint64_t kernel_cr3;
    uint64_t sterile_root_cr3 = 0;
    uint64_t active_pte;
    uint64_t text_pte;
    uint64_t rsp0_pte;
    sched_walk_snapshot_t target_text_walk;
    sched_walk_snapshot_t kernel_text_walk;
    sched_walk_snapshot_t target_canonical_walk;
    sched_walk_snapshot_t kernel_canonical_walk;
    sched_walk_snapshot_t target_idt_walk;
    sched_walk_snapshot_t kernel_idt_walk;

    if (!proc || proc->context.cr3 == 0 || proc->context.rsp0 == 0) {
        return;
    }

    __asm__ volatile("mov %%rsp, %0" : "=r"(active_rsp));
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    active_page = active_rsp & ~(AYKEN_FRAME_SIZE - 1);
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
#if defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)
    text_page = AYKEN_RING3_SECOND_CANONICAL_PROBE_VA & ~(AYKEN_FRAME_SIZE - 1);
    text_ip = AYKEN_RING3_SECOND_CANONICAL_PROBE_VA;
    second_fetch_va = AYKEN_RING3_SECOND_CANONICAL_PROBE_VA + 3;
#else
    text_page = AYKEN_RING3_TRAMPOLINE_VA & ~(AYKEN_FRAME_SIZE - 1);
    text_ip = AYKEN_RING3_TRAMPOLINE_VA;
#endif
#elif defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1)
    text_page = AYKEN_RING3_CANONICAL_STAGE_A_VA & ~(AYKEN_FRAME_SIZE - 1);
    text_ip = AYKEN_RING3_CANONICAL_STAGE_A_VA;
    second_fetch_va = AYKEN_RING3_CANONICAL_STAGE_B_VA;
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    third_fetch_va = AYKEN_RING3_CANONICAL_STAGE_C_VA;
#endif
#elif defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1)
    text_page = AYKEN_RING3_TRAMPOLINE_VA & ~(AYKEN_FRAME_SIZE - 1);
    text_ip = AYKEN_RING3_TRAMPOLINE_VA;
    low_fetch_va = AYKEN_RING3_TRAMPOLINE_VA + 3;
#else
    text_page = ((uint64_t)(uintptr_t)ring3_enter_post_cr3) & ~(AYKEN_FRAME_SIZE - 1);
    text_ip = (uint64_t)(uintptr_t)ring3_enter_post_cr3;
#endif
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    high_va = (uint64_t)(uintptr_t)ring3_enter_post_cr3;
#endif
    canonical_fetch_va = ((uint64_t)(uintptr_t)ring3_enter_post_cr3) + 3;
    idt_pf_va = (uint64_t)(uintptr_t)&idt_table[14];
    rsp0_page = (proc->context.rsp0 - 1) & ~(AYKEN_FRAME_SIZE - 1);
    cr3 = proc->context.cr3;
    kernel_cr3 = paging_get_kernel_pml4_phys();
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_RING3_STERILE_ALT_ROOT == 1)
    sterile_root_cr3 = sched_get_sterile_kernel_clone_root_phys();
#endif
    active_pte = paging_get_pte_in_pml4(cr3, active_page);
    text_pte = paging_get_pte_in_pml4(cr3, text_page);
    text_phys = text_pte & AYKEN_PTE_ADDR_MASK;
    kernel_transition_pte = paging_get_pte(text_ip);
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    high_pte = paging_get_pte_in_pml4(cr3, high_va);
    high_phys = high_pte & AYKEN_PTE_ADDR_MASK;
#endif
    rsp0_pte = paging_get_pte_in_pml4(cr3, rsp0_page);
    user_text_lo_ok = read_user_u64_via_pml4(cr3, text_ip, &user_text_lo);
    user_text_hi_ok = read_user_u64_via_pml4(cr3, text_ip + 8, &user_text_hi);
    sched_capture_walk_snapshot(cr3, text_ip, &target_text_walk);
    sched_capture_walk_snapshot(kernel_cr3, text_ip, &kernel_text_walk);
    if (text_phys) {
        const uint8_t *frame = (const uint8_t *)paging_phys_to_virt(text_phys);
        frame_text_lo = sched_dbg_read_u64_le(frame);
        frame_text_hi = sched_dbg_read_u64_le(frame ? frame + 8 : NULL);
    }
    {
        const uint8_t *sym =
#if defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
            (const uint8_t *)(uintptr_t)ring3_enter_trampoline;
#elif defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1)
            (const uint8_t *)(uintptr_t)ring3_enter_trampoline;
#else
            (const uint8_t *)(uintptr_t)ring3_enter_post_cr3;
#endif
        symbol_text_lo = sched_dbg_read_u64_le(sym);
        symbol_text_hi = sched_dbg_read_u64_le(sym ? sym + 8 : NULL);
    }

    sched_emit_cr3_consistency_line("CR3C", cr3, kernel_cr3, active_cr3);
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_RING3_STERILE_ALT_ROOT == 1)
    if ((sterile_root_cr3 & AYKEN_PTE_ADDR_MASK) != 0 &&
        (sterile_root_cr3 & AYKEN_PTE_ADDR_MASK) != (cr3 & AYKEN_PTE_ADDR_MASK)) {
        sched_emit_root_diff_summary_line("R4DS", sterile_root_cr3, cr3);
        sched_emit_lower_half_root_diff("RTDS", "RTDE", sterile_root_cr3, cr3);
        sched_emit_root_surface_compare("SCTX", sterile_root_cr3, cr3, USER_TEXT_BASE);
        sched_emit_root_surface_compare("SCS0",
                                        sterile_root_cr3,
                                        cr3,
                                        USER_STACK_TOP - 8);
        sched_emit_root_surface_compare("SCS1",
                                        sterile_root_cr3,
                                        cr3,
                                        USER_STACK_TOP - AYKEN_FRAME_SIZE - 8);
        sched_emit_root_surface_compare("SCCA", sterile_root_cr3, cr3, RING3_CANARY_ADDR);
        sched_emit_root_surface_compare("SCMB", sterile_root_cr3, cr3, SCHED_MAILBOX_VA);
        sched_emit_root_surface_compare("SCIN", sterile_root_cr3, cr3, EXECUTION_INBOX_VA);
        for (uint32_t i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
            char tag[5] = {'S', 'C', 'P', (char)('0' + (int)i), '\0'};
            sched_emit_root_surface_compare(
                tag,
                sterile_root_cr3,
                cr3,
                EXECUTION_PAYLOAD_VA + ((uint64_t)i * AYKEN_FRAME_SIZE));
        }
        sched_emit_root_surface_compare("SCHA",
                                        sterile_root_cr3,
                                        cr3,
                                        AYKEN_RING3_CANONICAL_STAGE_A_VA);
        sched_emit_root_surface_compare("SCHB",
                                        sterile_root_cr3,
                                        cr3,
                                        AYKEN_RING3_CANONICAL_STAGE_B_VA);
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
        sched_emit_root_surface_compare("SCHC",
                                        sterile_root_cr3,
                                        cr3,
                                        AYKEN_RING3_CANONICAL_STAGE_C_VA);
#endif
    }
#endif

    SCHED_DBG_OUT((uint8_t)'V');
    SCHED_DBG_OUT((uint8_t)'C');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(cr3);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'S');
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(active_page);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(active_pte);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'T');
    SCHED_DBG_OUT((uint8_t)'X');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(text_page);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(text_pte);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'R');
    SCHED_DBG_OUT((uint8_t)'0');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(rsp0_page);
    SCHED_DBG_OUT((uint8_t)' ');
    SCHED_DBG_OUT((uint8_t)'P');
    SCHED_DBG_OUT((uint8_t)'=');
    dbg_out_hex64(rsp0_pte);
    SCHED_DBG_OUT((uint8_t)'\n');

    sched_debug_dump_walk("UTX", cr3, text_ip);
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    sched_debug_dump_walk("UHX", cr3, high_va);
#endif
    sched_debug_dump_walk("UID", cr3, idt_pf_va);
    sched_debug_dump_walk("UR0", cr3, proc->context.rsp0 - 8);
    sched_debug_dump_walk("KTX", kernel_cr3, text_ip);
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    sched_debug_dump_walk("KHX", kernel_cr3, high_va);
#endif
    sched_debug_dump_walk("KID", kernel_cr3, idt_pf_va);
    sched_capture_walk_snapshot(cr3, canonical_fetch_va, &target_canonical_walk);
    sched_capture_walk_snapshot(kernel_cr3, canonical_fetch_va, &kernel_canonical_walk);
    sched_capture_walk_snapshot(cr3, idt_pf_va, &target_idt_walk);
    sched_capture_walk_snapshot(kernel_cr3, idt_pf_va, &kernel_idt_walk);
    sched_emit_walk_snapshot_line("TCH", &target_canonical_walk);
    sched_emit_walk_snapshot_line("KCH", &kernel_canonical_walk);
    sched_emit_walk_snapshot_line("TIH", &target_idt_walk);
    sched_emit_walk_snapshot_line("KIH", &kernel_idt_walk);
    sched_emit_walk_semantics_line("TCS", &target_canonical_walk);
    sched_emit_walk_semantics_line("KCS", &kernel_canonical_walk);
    sched_emit_walk_semantics_line("TIS", &target_idt_walk);
    sched_emit_walk_semantics_line("KIS", &kernel_idt_walk);
    sched_emit_walk_integrity_line("TCI", &target_canonical_walk);
    sched_emit_walk_integrity_line("KCI", &kernel_canonical_walk);
    sched_emit_walk_integrity_line("TII", &target_idt_walk);
    sched_emit_walk_integrity_line("KII", &kernel_idt_walk);
    sched_emit_chain_compare_line("FCMP", &kernel_text_walk, &target_text_walk);
    sched_emit_chain_compare_line("FCHI", &kernel_canonical_walk, &target_canonical_walk);
    sched_emit_pte_compare_line("PTX", text_ip, kernel_transition_pte, text_pte);
#if defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    sched_capture_walk_snapshot(cr3, low_fetch_va, &target_low_walk);
    sched_capture_walk_snapshot(kernel_cr3, low_fetch_va, &kernel_low_walk);
    sched_emit_walk_snapshot_line("LFH", &target_low_walk);
    sched_emit_walk_snapshot_line("LKH", &kernel_low_walk);
    sched_emit_walk_semantics_line("LFS", &target_low_walk);
    sched_emit_walk_semantics_line("LKS", &kernel_low_walk);
    sched_emit_walk_integrity_line("LFI", &target_low_walk);
    sched_emit_walk_integrity_line("LKI", &kernel_low_walk);
#endif
#if (defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1) && \
     defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)) || \
    (defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1))
    sched_capture_walk_snapshot(cr3, second_fetch_va, &target_second_walk);
    sched_capture_walk_snapshot(kernel_cr3, second_fetch_va, &kernel_second_walk);
    sched_emit_walk_snapshot_line("2TH", &target_second_walk);
    sched_emit_walk_snapshot_line("2KH", &kernel_second_walk);
    sched_emit_walk_semantics_line("2TS", &target_second_walk);
    sched_emit_walk_semantics_line("2KS", &kernel_second_walk);
    sched_emit_walk_integrity_line("2TI", &target_second_walk);
    sched_emit_walk_integrity_line("2KI", &kernel_second_walk);
    sched_emit_chain_compare_line("FCM2", &kernel_second_walk, &target_second_walk);
    sched_emit_pte_compare_line("P2X", second_fetch_va, kernel_second_walk.pte,
                                target_second_walk.pte);
#endif
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    sched_capture_walk_snapshot(cr3, third_fetch_va, &target_third_walk);
    sched_capture_walk_snapshot(kernel_cr3, third_fetch_va, &kernel_third_walk);
    sched_emit_walk_snapshot_line("3TH", &target_third_walk);
    sched_emit_walk_snapshot_line("3KH", &kernel_third_walk);
    sched_emit_walk_semantics_line("3TS", &target_third_walk);
    sched_emit_walk_semantics_line("3KS", &kernel_third_walk);
    sched_emit_walk_integrity_line("3TI", &target_third_walk);
    sched_emit_walk_integrity_line("3KI", &kernel_third_walk);
    sched_emit_chain_compare_line("FCM3", &kernel_third_walk, &target_third_walk);
    sched_emit_pte_compare_line("P3X", third_fetch_va, kernel_third_walk.pte,
                                target_third_walk.pte);
#endif

    sched_dbg_emit_text("WFX AC=");
    dbg_out_hex64(active_cr3);
    sched_dbg_emit_text(" KC=");
    dbg_out_hex64(kernel_cr3);
    sched_dbg_emit_text(" TC=");
    dbg_out_hex64(cr3);
    sched_dbg_emit_text(" AV=");
    dbg_out_hex64(text_ip);
    sched_dbg_emit_text(" AP=");
    dbg_out_hex64(text_pte);
    sched_dbg_emit_text(" AF=");
    dbg_out_hex64(text_phys);
    sched_dbg_emit_text(" KP=");
    dbg_out_hex64(kernel_transition_pte);
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    sched_dbg_emit_text(" HV=");
    dbg_out_hex64(high_va);
    sched_dbg_emit_text(" HP=");
    dbg_out_hex64(high_pte);
    sched_dbg_emit_text(" HF=");
    dbg_out_hex64(high_phys);
#endif
    sched_dbg_emit_text(" U0=");
    dbg_out_hex64(user_text_lo_ok ? user_text_lo : 0);
    sched_dbg_emit_text(" U1=");
    dbg_out_hex64(user_text_hi_ok ? user_text_hi : 0);
    sched_dbg_emit_text(" P0=");
    dbg_out_hex64(frame_text_lo);
    sched_dbg_emit_text(" P1=");
    dbg_out_hex64(frame_text_hi);
    sched_dbg_emit_text(" S0=");
    dbg_out_hex64(symbol_text_lo);
    sched_dbg_emit_text(" S1=");
    dbg_out_hex64(symbol_text_hi);
    sched_dbg_emit_text("\n");
#else
    (void)proc;
#endif
}

static int sched_is_canonical_addr(uint64_t addr)
{
    const uint64_t upper = addr >> 48;
    const uint64_t sign = (addr >> 47) & 1ULL;
    return sign ? (upper == 0xFFFFULL) : (upper == 0x0000ULL);
}

static void sched_ring3_contract_panic(const char *reason, const proc_t *proc)
{
    sched_emit_marker("P10_RING3_PRECONDITION_FAIL reason=");
    sched_emit_marker(reason ? reason : "unknown");
    if (proc) {
        sched_emit_marker(" pid=");
        sched_emit_u64_dec((uint64_t)(uint32_t)proc->pid);
    }
    sched_emit_marker("\n");

    fb_print("[PANIC] Ring3 transition precondition failed: ");
    fb_print(reason ? reason : "unknown");
    if (proc) {
        fb_print(" pid=");
        fb_print_hex((uint64_t)(uint32_t)proc->pid);
    }
    fb_print("\n");

    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static uint32_t g_sched_ring3_contract_warn_mask = 0;

static void sched_ring3_contract_warn_once(uint32_t warn_bit,
                                           const char *reason,
                                           const proc_t *proc)
{
    if ((g_sched_ring3_contract_warn_mask & warn_bit) != 0) {
        return;
    }
    g_sched_ring3_contract_warn_mask |= warn_bit;

    sched_emit_marker("P10_RING3_PRECONDITION_WARN reason=");
    sched_emit_marker(reason ? reason : "unknown");
    if (proc) {
        sched_emit_marker(" pid=");
        sched_emit_u64_dec((uint64_t)(uint32_t)proc->pid);
    }
    sched_emit_marker("\n");
}

static uint64_t sched_require_supervisor_exec_pte_or_panic(const proc_t *proc,
                                                           uint64_t root_phys,
                                                           uint64_t va,
                                                           const char *noncanonical_reason,
                                                           const char *invalid_reason,
                                                           const char *nx_reason,
                                                           const char *large_page_reason,
                                                           const char *writable_warn_reason,
                                                           uint32_t writable_warn_bit)
{
    sched_walk_snapshot_t snap;
    uint64_t effective_root = root_phys ? (root_phys & AYKEN_PTE_ADDR_MASK)
                                        : paging_get_kernel_pml4_phys();

    if (!sched_is_canonical_addr(va)) {
        sched_ring3_contract_panic(noncanonical_reason, proc);
    }
    if (effective_root == 0 || !sched_capture_walk_snapshot(effective_root, va, &snap)) {
        sched_ring3_contract_panic(invalid_reason, proc);
    }
    if (((snap.pdpte & AYKEN_PTE_PRESENT) != 0 && (snap.pdpte & (1ULL << 7)) != 0) ||
        ((snap.pde & AYKEN_PTE_PRESENT) != 0 && (snap.pde & (1ULL << 7)) != 0)) {
        sched_ring3_contract_panic(large_page_reason, proc);
    }
    if ((snap.pml4e & AYKEN_PTE_NO_EXEC) != 0 ||
        (snap.pdpte & AYKEN_PTE_NO_EXEC) != 0 ||
        (snap.pde & AYKEN_PTE_NO_EXEC) != 0 ||
        (snap.pte & AYKEN_PTE_NO_EXEC) != 0) {
        sched_ring3_contract_panic(nx_reason, proc);
    }
    if ((snap.pte & AYKEN_PTE_PRESENT) == 0 ||
        (snap.pte & AYKEN_PTE_USER) != 0) {
        sched_ring3_contract_panic(invalid_reason, proc);
    }
    if ((snap.pte & AYKEN_PTE_WRITABLE) != 0 && writable_warn_reason) {
        sched_ring3_contract_warn_once(writable_warn_bit, writable_warn_reason, proc);
    }

    return snap.pte;
}

static void sched_prepare_dispatch_context_or_panic(proc_t *proc)
{
    uint64_t rip_pte;
    uint64_t rsp_pte;
    uint64_t rsp0_pte;
    uint64_t active_high_entry_pte;
    uint64_t high_entry_pte;
    uint64_t idt_pte;
#if ((defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1)) || \
     (defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1))) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    uint64_t trampoline_phys;
#endif
#if defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    uint64_t active_low_entry_pte;
    uint64_t low_entry_pte;
#endif
#if defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    uint64_t active_canonical_entry_pte;
    uint64_t canonical_entry_pte;
    uint64_t active_canonical_iret_entry_pte;
    uint64_t canonical_iret_entry_pte;
    uint64_t iret_trampoline_phys;
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    uint64_t active_final_iret_entry_pte;
    uint64_t final_iret_entry_pte;
    uint64_t final_iret_trampoline_phys;
#endif
#endif

    if (!proc) {
        sched_ring3_contract_panic("null_proc", NULL);
    }

    if (proc->context.cs != GDT_USER_CODE) {
        if (proc->context.rsp0) {
            gdt_set_kernel_stack(proc->context.rsp0);
        }
        return;
    }

    if (proc->context.ss != GDT_USER_DATA) {
        sched_ring3_contract_panic("bad_user_ss", proc);
    }
    if (proc->context.cr3 == 0 ||
        (proc->context.cr3 & (AYKEN_FRAME_SIZE - 1)) != 0) {
        sched_ring3_contract_panic("bad_user_cr3", proc);
    }
    if (proc->context.rsp0 == 0) {
        sched_ring3_contract_panic("missing_rsp0", proc);
    }
    if (!sched_is_canonical_addr(proc->context.rip)) {
        sched_ring3_contract_panic("rip_noncanonical", proc);
    }
    if (!sched_is_canonical_addr(proc->context.rsp)) {
        sched_ring3_contract_panic("rsp_noncanonical", proc);
    }
    if (!sched_is_canonical_addr(proc->context.rsp0)) {
        sched_ring3_contract_panic("rsp0_noncanonical", proc);
    }
    if (proc->context.rsp0 < KERNEL_VIRT_BASE) {
        sched_ring3_contract_panic("rsp0_not_kernel_half", proc);
    }
    if (proc->context.rip < USER_TEXT_BASE ||
        proc->context.rip >= USER_STACK_TOP) {
        sched_ring3_contract_panic("rip_out_of_user_range", proc);
    }
    if (proc->context.rsp < USER_TEXT_BASE ||
        proc->context.rsp >= USER_STACK_TOP) {
        sched_ring3_contract_panic("rsp_out_of_user_range", proc);
    }
    if ((proc->context.rsp & 0xFULL) != 0x8ULL) {
        sched_ring3_contract_panic("rsp_alignment", proc);
    }

    rip_pte = paging_get_pte_in_pml4(proc->context.cr3, proc->context.rip);
    if ((rip_pte & (AYKEN_PTE_PRESENT | AYKEN_PTE_USER)) !=
        (AYKEN_PTE_PRESENT | AYKEN_PTE_USER)) {
        sched_ring3_contract_panic("rip_not_user_mapped", proc);
    }
    if ((rip_pte & AYKEN_PTE_NO_EXEC) != 0) {
        sched_ring3_contract_panic("rip_noexec", proc);
    }

    rsp_pte = paging_get_pte_in_pml4(proc->context.cr3, proc->context.rsp);
    if ((rsp_pte & (AYKEN_PTE_PRESENT | AYKEN_PTE_USER | AYKEN_PTE_WRITABLE)) !=
        (AYKEN_PTE_PRESENT | AYKEN_PTE_USER | AYKEN_PTE_WRITABLE)) {
        sched_ring3_contract_panic("rsp_not_user_writable", proc);
    }

    gdt_set_kernel_stack(proc->context.rsp0);
    __asm__ volatile("" ::: "memory");
    map_kernel_stack_pages_into_pml4(proc->context.cr3, proc->context.rsp0);

    rsp0_pte = paging_get_pte_in_pml4(proc->context.cr3, proc->context.rsp0 - 8);
    if ((rsp0_pte & (AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE)) !=
        (AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE) ||
        (rsp0_pte & AYKEN_PTE_USER) != 0) {
        sched_ring3_contract_panic("rsp0_not_supervisor_reachable", proc);
    }

    active_high_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        0,
        (uint64_t)(uintptr_t)ring3_enter_post_cr3,
        "transition_text_active_root_noncanonical",
        "transition_text_active_root_invalid",
        "transition_text_active_root_nx_violation",
        "transition_text_active_root_not_4kb_leaf",
        "transition_text_active_root_writable",
        1u);
    high_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        proc->context.cr3,
        (uint64_t)(uintptr_t)ring3_enter_post_cr3,
        "transition_text_high_half_noncanonical",
        "transition_text_high_half_missing",
        "transition_text_high_half_nx_violation",
        "transition_text_high_half_not_4kb_leaf",
        "transition_text_high_half_writable",
        2u);
    if ((active_high_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (high_entry_pte & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_active_root_frame_mismatch", proc);
    }
#if defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    trampoline_phys = paging_get_phys((uint64_t)(uintptr_t)ring3_enter_trampoline);
    if (trampoline_phys == 0) {
        sched_ring3_contract_panic("transition_text_low_alias_source_missing", proc);
    }
    active_low_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        0,
        AYKEN_RING3_TRAMPOLINE_VA,
        "transition_text_low_alias_active_root_noncanonical",
        "transition_text_low_alias_active_root_missing",
        "transition_text_low_alias_active_root_nx_violation",
        "transition_text_low_alias_active_root_not_4kb_leaf",
        "transition_text_low_alias_active_root_writable",
        3u);
    low_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        proc->context.cr3,
        AYKEN_RING3_TRAMPOLINE_VA,
        "transition_text_low_alias_target_root_noncanonical",
        "transition_text_low_alias_target_root_missing",
        "transition_text_low_alias_target_root_nx_violation",
        "transition_text_low_alias_target_root_not_4kb_leaf",
        "transition_text_low_alias_target_root_writable",
        4u);
    if ((active_low_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_low_alias_active_frame_mismatch", proc);
    }
    if ((low_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_low_alias_target_frame_mismatch", proc);
    }
#endif
#if defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1) && \
    (!defined(AYKEN_RING3_FETCH_PROBE) || (AYKEN_RING3_FETCH_PROBE == 0))
    trampoline_phys = paging_get_phys((uint64_t)(uintptr_t)ring3_enter_trampoline);
    if (trampoline_phys == 0) {
        sched_ring3_contract_panic("transition_text_canonical_alias_source_missing", proc);
    }
    iret_trampoline_phys = sched_ring3_stage_b_source_phys();
    if (iret_trampoline_phys == 0) {
        sched_ring3_contract_panic("transition_text_canonical_iret_source_missing", proc);
    }
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    final_iret_trampoline_phys =
        paging_get_phys((uint64_t)(uintptr_t)ring3_enter_final_iret_trampoline);
    if (final_iret_trampoline_phys == 0) {
        sched_ring3_contract_panic("transition_text_canonical_final_iret_source_missing", proc);
    }
#endif
    active_canonical_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        0,
        AYKEN_RING3_CANONICAL_STAGE_A_VA,
        "transition_text_canonical_alias_active_root_noncanonical",
        "transition_text_canonical_alias_active_root_missing",
        "transition_text_canonical_alias_active_root_nx_violation",
        "transition_text_canonical_alias_active_root_not_4kb_leaf",
        "transition_text_canonical_alias_active_root_writable",
        5u);
    canonical_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        proc->context.cr3,
        AYKEN_RING3_CANONICAL_STAGE_A_VA,
        "transition_text_canonical_alias_target_root_noncanonical",
        "transition_text_canonical_alias_target_root_missing",
        "transition_text_canonical_alias_target_root_nx_violation",
        "transition_text_canonical_alias_target_root_not_4kb_leaf",
        "transition_text_canonical_alias_target_root_writable",
        6u);
    if ((active_canonical_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_canonical_alias_active_frame_mismatch", proc);
    }
    if ((canonical_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_canonical_alias_target_frame_mismatch", proc);
    }
    active_canonical_iret_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        0,
        AYKEN_RING3_CANONICAL_STAGE_B_VA,
        "transition_text_canonical_iret_active_root_noncanonical",
        "transition_text_canonical_iret_active_root_missing",
        "transition_text_canonical_iret_active_root_nx_violation",
        "transition_text_canonical_iret_active_root_not_4kb_leaf",
        "transition_text_canonical_iret_active_root_writable",
        7u);
    canonical_iret_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        proc->context.cr3,
        AYKEN_RING3_CANONICAL_STAGE_B_VA,
        "transition_text_canonical_iret_target_root_noncanonical",
        "transition_text_canonical_iret_target_root_missing",
        "transition_text_canonical_iret_target_root_nx_violation",
        "transition_text_canonical_iret_target_root_not_4kb_leaf",
        "transition_text_canonical_iret_target_root_writable",
        8u);
    if ((active_canonical_iret_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (iret_trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_canonical_iret_active_frame_mismatch", proc);
    }
    if ((canonical_iret_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (iret_trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic("transition_text_canonical_iret_target_frame_mismatch", proc);
    }
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    active_final_iret_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        0,
        AYKEN_RING3_CANONICAL_STAGE_C_VA,
        "transition_text_canonical_final_iret_active_root_noncanonical",
        "transition_text_canonical_final_iret_active_root_missing",
        "transition_text_canonical_final_iret_active_root_nx_violation",
        "transition_text_canonical_final_iret_active_root_not_4kb_leaf",
        "transition_text_canonical_final_iret_active_root_writable",
        9u);
    final_iret_entry_pte = sched_require_supervisor_exec_pte_or_panic(
        proc,
        proc->context.cr3,
        AYKEN_RING3_CANONICAL_STAGE_C_VA,
        "transition_text_canonical_final_iret_target_root_noncanonical",
        "transition_text_canonical_final_iret_target_root_missing",
        "transition_text_canonical_final_iret_target_root_nx_violation",
        "transition_text_canonical_final_iret_target_root_not_4kb_leaf",
        "transition_text_canonical_final_iret_target_root_writable",
        10u);
    if ((active_final_iret_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (final_iret_trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic(
            "transition_text_canonical_final_iret_active_frame_mismatch", proc);
    }
    if ((final_iret_entry_pte & AYKEN_PTE_ADDR_MASK) !=
        (final_iret_trampoline_phys & AYKEN_PTE_ADDR_MASK)) {
        sched_ring3_contract_panic(
            "transition_text_canonical_final_iret_target_frame_mismatch", proc);
    }
#endif
#endif
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    {
        uint64_t entry_pte = paging_get_pte_in_pml4(
#if defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)
            proc->context.cr3, AYKEN_RING3_SECOND_CANONICAL_PROBE_VA);
        if ((entry_pte & AYKEN_PTE_PRESENT) == 0 ||
            (entry_pte & AYKEN_PTE_USER) != 0 ||
            (entry_pte & AYKEN_PTE_NO_EXEC) != 0) {
            sched_ring3_contract_panic("transition_text_second_high_half_missing", proc);
        }
#else
            proc->context.cr3, AYKEN_RING3_TRAMPOLINE_VA);
        if ((entry_pte & AYKEN_PTE_PRESENT) == 0 ||
            (entry_pte & AYKEN_PTE_USER) == 0 ||
            (entry_pte & AYKEN_PTE_WRITABLE) != 0 ||
            (entry_pte & AYKEN_PTE_NO_EXEC) != 0) {
            sched_ring3_contract_panic("transition_text_not_user_rx", proc);
        }
#endif
#if defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1) && \
    defined(AYKEN_RING3_FRESH_FRAME_PROBE) && (AYKEN_RING3_FRESH_FRAME_PROBE == 1)
        if (!sched_ring3_probe_frame_matches_symbol(entry_pte & AYKEN_PTE_ADDR_MASK)) {
            sched_ring3_contract_panic("transition_text_probe_bytes_mismatch", proc);
        }
#else
        if ((entry_pte & AYKEN_PTE_ADDR_MASK) !=
            (high_entry_pte & AYKEN_PTE_ADDR_MASK)) {
            sched_ring3_contract_panic("transition_text_frame_mismatch", proc);
        }
#endif
    }
#endif

    idt_pte = paging_get_pte_in_pml4(proc->context.cr3, (uint64_t)(uintptr_t)&idt_table[14]);
    if ((idt_pte & AYKEN_PTE_PRESENT) == 0 ||
        (idt_pte & AYKEN_PTE_USER) != 0) {
        sched_ring3_contract_panic("idt_not_supervisor_reachable", proc);
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

#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    {
        uint64_t trampoline_phys =
            paging_get_phys((uint64_t)(uintptr_t)ring3_enter_post_cr3);
        if (trampoline_phys) {
#if defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)
            uint64_t probe_phys = trampoline_phys & AYKEN_PTE_ADDR_MASK;
            uint64_t probe_adjacent_phys = 0;
            uint64_t existing_kernel_pte = paging_get_pte(AYKEN_RING3_SECOND_CANONICAL_PROBE_VA);
            uint64_t existing_target_pte =
                paging_get_pte_in_pml4(pml4_phys, AYKEN_RING3_SECOND_CANONICAL_PROBE_VA);

#if defined(AYKEN_RING3_FRESH_FRAME_PROBE) && (AYKEN_RING3_FRESH_FRAME_PROBE == 1)
            probe_phys = sched_ring3_probe_mapping_phys();
            if (probe_phys == 0) {
                sched_ring3_diag_panic("second_canonical_fresh_frame_alloc_failed");
            }
            probe_adjacent_phys =
                paging_get_phys((((uint64_t)(uintptr_t)ring3_enter_post_cr3) &
                                 ~(AYKEN_FRAME_SIZE - 1)) +
                                AYKEN_FRAME_SIZE);
            if (probe_adjacent_phys == 0) {
                sched_ring3_diag_panic("second_canonical_adjacent_page_missing");
            }
#endif

            if ((existing_kernel_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_kernel_pte & AYKEN_PTE_ADDR_MASK) != probe_phys ||
                 (existing_kernel_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_kernel_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("second_canonical_kernel_collision");
            }
            if ((existing_target_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_target_pte & AYKEN_PTE_ADDR_MASK) != probe_phys ||
                 (existing_target_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_target_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("second_canonical_target_collision");
            }
            paging_map_page(
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA,
                probe_phys,
#if defined(AYKEN_RING3_FRESH_FRAME_PROBE) && (AYKEN_RING3_FRESH_FRAME_PROBE == 1)
                AYKEN_PTE_WRITABLE);
            sched_dbg_emit_stack_map_request(
                pml4_phys,
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA,
                probe_phys,
                AYKEN_PTE_WRITABLE);
            paging_map_page_in_pml4(
                pml4_phys,
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA,
                probe_phys,
                AYKEN_PTE_WRITABLE);
            paging_map_page(
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA + AYKEN_FRAME_SIZE,
                probe_adjacent_phys,
                AYKEN_PTE_WRITABLE);
            sched_dbg_emit_stack_map_request(
                pml4_phys,
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA + AYKEN_FRAME_SIZE,
                probe_adjacent_phys,
                AYKEN_PTE_WRITABLE);
            paging_map_page_in_pml4(
                pml4_phys,
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA + AYKEN_FRAME_SIZE,
                probe_adjacent_phys,
                AYKEN_PTE_WRITABLE);
#else
                AYKEN_PTE_READ_ONLY);
            sched_dbg_emit_stack_map_request(
                pml4_phys,
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA,
                probe_phys,
                AYKEN_PTE_READ_ONLY);
            paging_map_page_in_pml4(
                pml4_phys,
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA,
                probe_phys,
                AYKEN_PTE_READ_ONLY);
#endif
#else
            paging_map_page(
                AYKEN_RING3_TRAMPOLINE_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY);
            sched_dbg_emit_stack_map_request(
                pml4_phys,
                AYKEN_RING3_TRAMPOLINE_VA,
                trampoline_phys,
                AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY);
            paging_map_page_in_pml4(
                pml4_phys,
                AYKEN_RING3_TRAMPOLINE_VA,
                trampoline_phys,
                AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY);
#endif
        }
    }
#elif defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1)
    {
        uint64_t trampoline_phys =
            paging_get_phys((uint64_t)(uintptr_t)ring3_enter_trampoline);
        if (trampoline_phys) {
            uint64_t existing_kernel_pte = paging_get_pte(AYKEN_RING3_TRAMPOLINE_VA);
            uint64_t existing_target_pte =
                paging_get_pte_in_pml4(pml4_phys, AYKEN_RING3_TRAMPOLINE_VA);

            if ((existing_kernel_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_kernel_pte & AYKEN_PTE_ADDR_MASK) != trampoline_phys ||
                 (existing_kernel_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_kernel_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_kernel_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("low_fetch_alias_kernel_collision");
            }
            if ((existing_target_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_target_pte & AYKEN_PTE_ADDR_MASK) != trampoline_phys ||
                 (existing_target_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_target_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_target_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("low_fetch_alias_target_collision");
            }

            paging_map_page(
                AYKEN_RING3_TRAMPOLINE_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY);
            sched_dbg_emit_stack_map_request(
                pml4_phys,
                AYKEN_RING3_TRAMPOLINE_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY);
            paging_map_page_in_pml4(
                pml4_phys,
                AYKEN_RING3_TRAMPOLINE_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY);
        }
    }
#elif defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1)
    {
        uint64_t trampoline_phys =
            paging_get_phys((uint64_t)(uintptr_t)ring3_enter_trampoline);
        uint64_t iret_trampoline_phys = sched_ring3_stage_b_source_phys();
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
        uint64_t final_iret_trampoline_phys =
            paging_get_phys((uint64_t)(uintptr_t)ring3_enter_final_iret_trampoline);
#endif
        if (trampoline_phys) {
            uint64_t existing_kernel_pte =
                paging_get_pte(AYKEN_RING3_CANONICAL_STAGE_A_VA);
            uint64_t existing_target_pte =
                paging_get_pte_in_pml4(pml4_phys, AYKEN_RING3_CANONICAL_STAGE_A_VA);
            uint64_t existing_iret_kernel_pte =
                paging_get_pte(AYKEN_RING3_CANONICAL_STAGE_B_VA);
            uint64_t existing_iret_target_pte =
                paging_get_pte_in_pml4(pml4_phys, AYKEN_RING3_CANONICAL_STAGE_B_VA);
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
            uint64_t existing_final_iret_kernel_pte =
                paging_get_pte(AYKEN_RING3_CANONICAL_STAGE_C_VA);
            uint64_t existing_final_iret_target_pte =
                paging_get_pte_in_pml4(pml4_phys, AYKEN_RING3_CANONICAL_STAGE_C_VA);
#endif

            if ((existing_kernel_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_kernel_pte & AYKEN_PTE_ADDR_MASK) != trampoline_phys ||
                 (existing_kernel_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_kernel_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_kernel_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("canonical_fetch_alias_kernel_collision");
            }
            if ((existing_target_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_target_pte & AYKEN_PTE_ADDR_MASK) != trampoline_phys ||
                 (existing_target_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_target_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_target_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("canonical_fetch_alias_target_collision");
            }
            if (iret_trampoline_phys != 0 &&
                (existing_iret_kernel_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_iret_kernel_pte & AYKEN_PTE_ADDR_MASK) != iret_trampoline_phys ||
                 (existing_iret_kernel_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_iret_kernel_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_iret_kernel_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("canonical_iret_alias_kernel_collision");
            }
            if (iret_trampoline_phys != 0 &&
                (existing_iret_target_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_iret_target_pte & AYKEN_PTE_ADDR_MASK) != iret_trampoline_phys ||
                 (existing_iret_target_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_iret_target_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_iret_target_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("canonical_iret_alias_target_collision");
            }
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
            if (final_iret_trampoline_phys != 0 &&
                (existing_final_iret_kernel_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_final_iret_kernel_pte & AYKEN_PTE_ADDR_MASK) !=
                     final_iret_trampoline_phys ||
                 (existing_final_iret_kernel_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_final_iret_kernel_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_final_iret_kernel_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("canonical_final_iret_alias_kernel_collision");
            }
            if (final_iret_trampoline_phys != 0 &&
                (existing_final_iret_target_pte & AYKEN_PTE_PRESENT) != 0 &&
                ((existing_final_iret_target_pte & AYKEN_PTE_ADDR_MASK) !=
                     final_iret_trampoline_phys ||
                 (existing_final_iret_target_pte & AYKEN_PTE_USER) != 0 ||
                 (existing_final_iret_target_pte & AYKEN_PTE_WRITABLE) != 0 ||
                 (existing_final_iret_target_pte & AYKEN_PTE_NO_EXEC) != 0)) {
                sched_ring3_diag_panic("canonical_final_iret_alias_target_collision");
            }
#endif

            paging_map_page(
                AYKEN_RING3_CANONICAL_STAGE_A_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
            sched_dbg_emit_stack_map_request(
                pml4_phys,
                AYKEN_RING3_CANONICAL_STAGE_A_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
            paging_map_page_in_pml4(
                pml4_phys,
                AYKEN_RING3_CANONICAL_STAGE_A_VA,
                trampoline_phys,
                AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
            if (iret_trampoline_phys != 0) {
                paging_map_page(
                    AYKEN_RING3_CANONICAL_STAGE_B_VA,
                    iret_trampoline_phys,
                    AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
                sched_dbg_emit_stack_map_request(
                    pml4_phys,
                    AYKEN_RING3_CANONICAL_STAGE_B_VA,
                    iret_trampoline_phys,
                    AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
                paging_map_page_in_pml4(
                    pml4_phys,
                    AYKEN_RING3_CANONICAL_STAGE_B_VA,
                    iret_trampoline_phys,
                    AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
            }
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
            if (final_iret_trampoline_phys != 0) {
                paging_map_page(
                    AYKEN_RING3_CANONICAL_STAGE_C_VA,
                    final_iret_trampoline_phys,
                    AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
                sched_dbg_emit_stack_map_request(
                    pml4_phys,
                    AYKEN_RING3_CANONICAL_STAGE_C_VA,
                    final_iret_trampoline_phys,
                    AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
                paging_map_page_in_pml4(
                    pml4_phys,
                    AYKEN_RING3_CANONICAL_STAGE_C_VA,
                    final_iret_trampoline_phys,
                    AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL);
            }
#endif
        }
    }
#endif

    uint64_t rsp = 0;
    __asm__ volatile("mov %%rsp, %0" : "=r"(rsp));
    uint64_t page = rsp & ~(AYKEN_FRAME_SIZE - 1);
    uint64_t phys = paging_get_phys(page);
    if (phys) {
        sched_dbg_emit_stack_map_request(pml4_phys, page, phys, AYKEN_PTE_WRITABLE);
        paging_map_page_in_pml4(pml4_phys, page, phys, AYKEN_PTE_WRITABLE);
    }

    uint64_t page_below = page - AYKEN_FRAME_SIZE;
    uint64_t phys_below = paging_get_phys(page_below);
    if (phys_below) {
        sched_dbg_emit_stack_map_request(
            pml4_phys, page_below, phys_below, AYKEN_PTE_WRITABLE);
        paging_map_page_in_pml4(pml4_phys, page_below, phys_below, AYKEN_PTE_WRITABLE);
    }

    if (rsp0) {
        uint64_t top_page = (rsp0 - 1) & ~(AYKEN_FRAME_SIZE - 1);
        uint64_t top_phys = paging_get_phys(top_page);
        if (top_phys) {
            sched_dbg_emit_stack_map_request(
                pml4_phys, top_page, top_phys, AYKEN_PTE_WRITABLE);
            paging_map_page_in_pml4(pml4_phys, top_page, top_phys, AYKEN_PTE_WRITABLE);
        }
        uint64_t below_page = top_page - AYKEN_FRAME_SIZE;
        uint64_t below_phys = paging_get_phys(below_page);
        if (below_phys) {
            sched_dbg_emit_stack_map_request(
                pml4_phys, below_page, below_phys, AYKEN_PTE_WRITABLE);
            paging_map_page_in_pml4(pml4_phys, below_page, below_phys, AYKEN_PTE_WRITABLE);
        }
    }

    {
        struct {
            uint64_t va;
            uint64_t flags;
        } kernel_pages[] = {
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
            {
#if defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)
                AYKEN_RING3_SECOND_CANONICAL_PROBE_VA,
#if defined(AYKEN_RING3_FRESH_FRAME_PROBE) && (AYKEN_RING3_FRESH_FRAME_PROBE == 1)
                AYKEN_PTE_WRITABLE,
#else
                AYKEN_PTE_READ_ONLY,
#endif
            },
#else
                AYKEN_RING3_TRAMPOLINE_VA,
                AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY,
            },
#endif
#endif
            {
                ((uint64_t)(uintptr_t)&idt_table[14]) & ~(AYKEN_FRAME_SIZE - 1),
                AYKEN_PTE_WRITABLE,
            },
            {
                ((uint64_t)(uintptr_t)&kernel_tss) & ~(AYKEN_FRAME_SIZE - 1),
                AYKEN_PTE_WRITABLE,
            },
        };

        for (size_t i = 0; i < (sizeof(kernel_pages) / sizeof(kernel_pages[0])); ++i) {
            uint64_t va = kernel_pages[i].va;
            uint64_t phys = paging_get_phys(va);
            if (phys) {
                sched_dbg_emit_stack_map_request(
                    pml4_phys, va, phys, kernel_pages[i].flags);
                paging_map_page_in_pml4(pml4_phys, va, phys, kernel_pages[i].flags);
            }
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

void sched_remove_process_everywhere(proc_t *p)
{
    if (!p) {
        return;
    }

    remove_from_ready_queue(p);
    remove_from_blocked(p);
    p->next = NULL;
    p->wait_obj = NULL;
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
    memset(sched_perf_phase_emitted, 0, sizeof(sched_perf_phase_emitted));
    memset(sched_perf_mb_phase_emitted, 0, sizeof(sched_perf_mb_phase_emitted));
    
    // Ring0 mechanism: No policy initialization in Ring0
    // Ring3 scheduler policy handles all policy setup
}

void sched_start(void)
{
    proc_drain_deferred_reap();
    sched_perf_note_first_scheduler_activity();

    // Runtime-observed config marker for CI/gates (independent from shell env echo).
    sched_emit_marker("[K][CFG] user_minimal_mode=");
    sched_emit_marker(AYKEN_USER_MINIMAL_MODE_STRING);
    sched_emit_marker(" bootstrap_policy=");
    sched_emit_u64_dec((uint64_t)AYKEN_SCHED_BOOTSTRAP_POLICY);
    sched_emit_marker(" mb_selftest=");
    sched_emit_u64_dec((uint64_t)AYKEN_MB_SELFTEST);
    sched_emit_marker(" deterministic_exit=");
    sched_emit_u64_dec((uint64_t)AYKEN_DETERMINISTIC_EXIT);
    sched_emit_marker("\n");

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
    sched_try_pickup_execution_work();
    
    // MVP-0: Scheduler bridge self-test (emits markers for gate validation)
    // Called here after current_proc is set but before switch_to_first
    // Compile-out in release: self-test is validation-only
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
#if (!defined(AYKEN_MB_SELFTEST) || (AYKEN_MB_SELFTEST == 1)) && \
    (!defined(AYKEN_C2_STRICT_MARKERS) || (AYKEN_C2_STRICT_MARKERS == 0))
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
    sched_prepare_dispatch_context_or_panic(current_proc);

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
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (!defined(AYKEN_C2_STRICT_MARKERS) || (AYKEN_C2_STRICT_MARKERS == 0))
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
#if AYKEN_C2_STRICT_MARKERS && !AYKEN_GATE45_PROOF
    if (used_mailbox && decision_id > 0 && decision_src_pid > 0) {
        proc_t *start_prev = proc_find_by_pid(1);
        if (start_prev && start_prev != first) {
            sched_emit_gate45_chain_once(
                start_prev,
                first,
                decision_id,
                decision_src_pid,
                1,
                SCHED_DECISION_SITE_START);
        }
    }
#endif
    sched_graft_real_user_state_into_sterile_root(current_proc);
    sched_debug_ring3_entry_window(current_proc);
    sched_force_ring3_entry_cr3_to_sterile_root(current_proc);
    sched_force_ring3_entry_cr3_to_kernel_root(current_proc);
    sched_emit_pre_dispatch_text_walk_proof(current_proc);
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
    proc_drain_deferred_reap();

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
        reenable_if ? SCHED_DECISION_SITE_YIELD : SCHED_DECISION_SITE_IRQ);
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
#if !AYKEN_SCHED_BOOTSTRAP_POLICY
        fb_print("[PANIC] owner mailbox decision missing on yield\n");
        for (;;) __asm__ volatile("cli; hlt");
#endif
        SCHED_DBG_OUT((uint8_t)'X');
        SCHED_DBG_OUT((uint8_t)'\n');
        if (reenable_if)
            enable_interrupts();
        return;
    }

    sched_record_mailbox_decision_event(prev, next, decision_id, decision_src_pid, used_mailbox);
    sched_commit_owner_transfer_if_pending(prev, next);

    // If policy returns the currently running Ring3 process, keep running in place.
    if (prev && next == prev) {
        // IRQ no-op reschedule still represents a preempt/return cadence event.
        // Emit canonical markers so strict preempt harness can measure cadence
        // even when policy keeps the owner process running in place.
        if (!reenable_if) {
            char ring = ((current_proc->context.cs & 0x3) == 0x3) ? 'U' : 'K';
            sched_dbg_mark_pid((uint32_t)current_proc->pid);
            sched_dbg_mark_sw(ring, ring);
            sched_dbg_mark_iret();
        }
        if (used_mailbox && !phase10c_decision_markers_emitted) {
            phase10c_decision_markers_emitted = 1;
            sched_emit_phase10c_decision(
                "P10_MAILBOX_DECISION", decision_id, decision_pid, 1, decision_src_pid);
            sched_emit_phase10c_decision(
                "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
        }
        if (!reenable_if && ((current_proc->context.cs & 0x3) == 0x3)) {
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
            static uint8_t irq_canonical_return_marker_emitted = 0;
            if (!irq_canonical_return_marker_emitted) {
                irq_canonical_return_marker_emitted = 1;
                sched_emit_marker("P10_IRQ_CANONICAL_RETURN\n");
            }
#endif
            sched_note_first_user_entry_if_ring3(current_proc);
            sched_mask_irq0_before_first_ring3_entry(current_proc);
            sched_emit_pre_dispatch_text_walk_proof(current_proc);
            context_switch(&prev->context, &current_proc->context);
        }
        if (!reenable_if) {
            // Kernel IRQ no-op path returns via timer_isr_asm iretq.
            // Ring3 IRQ no-op path above canonicalizes through context_switch().
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
    sched_try_pickup_execution_work();

    if (emit_phase10c_markers) {
        sched_emit_phase10c_decision(
            "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
    }

    sched_dbg_mark_pid(current_proc->pid);

    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions (mechanism only)
    sched_prepare_dispatch_context_or_panic(current_proc);

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
    sched_emit_pre_dispatch_text_walk_proof(current_proc);

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
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (!defined(AYKEN_C2_STRICT_MARKERS) || (AYKEN_C2_STRICT_MARKERS == 0))
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
            sched_is_owner(current_proc)) {
            sched_gate45_arm_cross_target_once(current_proc);
        }
#endif
        sched_emit_gate45_chain_once(
            prev,
            current_proc,
            decision_id,
            decision_src_pid,
            used_mailbox,
            reenable_if ? SCHED_DECISION_SITE_YIELD : SCHED_DECISION_SITE_IRQ);
        sched_note_first_user_entry_if_ring3(current_proc);
        sched_graft_real_user_state_into_sterile_root(current_proc);
        sched_force_ring3_entry_cr3_to_sterile_root(current_proc);
        sched_force_ring3_entry_cr3_to_kernel_root(current_proc);
        sched_mask_irq0_before_first_ring3_entry(current_proc);
        sched_emit_pre_dispatch_text_walk_proof(current_proc);
        context_switch(&prev->context, &current_proc->context);
        
        // Ring3 INT80 diagnostic: verify whether user code resumed after syscall.
        if (prev && prev->context.cs == GDT_USER_CODE) {
            uint64_t canary = 0;
            uint64_t text_witness = 0;
            SCHED_DBG_OUT((uint8_t)'[');
            SCHED_DBG_OUT((uint8_t)'C');
            SCHED_DBG_OUT((uint8_t)'A');
            SCHED_DBG_OUT((uint8_t)'N');
            SCHED_DBG_OUT((uint8_t)'=');
            if (read_user_u64_via_pml4(prev->context.cr3, RING3_CANARY_ADDR, &canary)) {
                dbg_out_hex64(canary);
                SCHED_DBG_OUT((uint8_t)' ');
                if (canary == RING3_CANARY_POST) {
                    if (!phase10_retire_witness_marker_emitted) {
                        phase10_retire_witness_marker_emitted = 1;
                        sched_emit_marker("P10_RETIRE_WITNESS_POST rip=");
                        dbg_out_hex64(prev->context.rip);
                        sched_emit_marker(" cr3=");
                        dbg_out_hex64(prev->context.cr3);
                        sched_emit_marker("\n");
                    }
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

            if (read_user_u64_via_pml4(prev->context.cr3, RING3_TEXT_WITNESS_ADDR, &text_witness) &&
                text_witness == RING3_TEXT_WITNESS_SIG) {
                if (!phase10_text_witness_marker_emitted) {
                    phase10_text_witness_marker_emitted = 1;
                    sched_emit_marker("P10_TEXT_RETIRE_WITNESS rip=");
                    dbg_out_hex64(prev->context.rip);
                    sched_emit_marker(" cr3=");
                    dbg_out_hex64(prev->context.cr3);
                    sched_emit_marker("\n");
                }
            }
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
        sched_note_first_user_entry_if_ring3(current_proc);
        sched_mask_irq0_before_first_ring3_entry(current_proc);
        
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
    proc_drain_deferred_reap();

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

    sched_record_mailbox_decision_event(prev, next, decision_id, decision_src_pid, used_mailbox);
    sched_commit_owner_transfer_if_pending(prev, next);

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
    sched_try_pickup_execution_work();
    sched_prepare_dispatch_context_or_panic(current_proc);

    if (emit_phase10c_markers) {
        sched_emit_phase10c_decision(
            "P10_DECISION_APPLIED", decision_id, decision_pid, 0, decision_src_pid);
    }
    sched_emit_gate45_chain_once(
        prev,
        current_proc,
        decision_id,
        decision_src_pid,
        used_mailbox,
        SCHED_DECISION_SITE_BLOCK);
    sched_note_first_user_entry_if_ring3(current_proc);
    sched_mask_irq0_before_first_ring3_entry(current_proc);
    sched_emit_pre_dispatch_text_walk_proof(current_proc);
    context_switch(&prev->context, &current_proc->context);

    enable_interrupts();
}

void sched_exit_current(void)
{
    uint64_t decision_id = 0;
    uint32_t decision_pid = 0;
    uint32_t decision_src_pid = 0;
    int used_mailbox = 0;
    proc_t *prev;
    proc_t *next;

    disable_interrupts();

    prev = current_proc;
    if (!prev) {
        fb_print("[PANIC] exit path without current process\n");
        for (;;) __asm__ volatile("cli; hlt");
    }

    sched_remove_process_everywhere(prev);

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    next = g_sched_validation_exit_forced_next;
    if (next) {
        if (next == prev || next->state == PROC_ZOMBIE) {
            fb_print("[PANIC] invalid validation exit successor\n");
            for (;;) __asm__ volatile("cli; hlt");
        }
        sched_remove_process_everywhere(next);
        g_sched_validation_exit_switch_seen = 1;
        g_sched_validation_exit_from_pid = prev->pid;
        g_sched_validation_exit_to_pid = next->pid;
        g_sched_validation_exit_forced_next = NULL;
    } else
#endif
    next = sched_select_next_mailbox(prev,
                                     &decision_id,
                                     &decision_pid,
                                     &decision_src_pid,
                                     &used_mailbox,
                                     0,
                                     SCHED_DECISION_SITE_BLOCK);
    if (!next) {
        fb_print("[PANIC] exiting task without mailbox successor\n");
        for (;;) __asm__ volatile("cli; hlt");
    }

    sched_record_mailbox_decision_event(prev, next, decision_id, decision_src_pid, used_mailbox);
    sched_commit_owner_transfer_if_pending(prev, next);

    current_proc = next;
    current_proc->state = PROC_RUNNING;
    sched_try_pickup_execution_work();

    sched_prepare_dispatch_context_or_panic(current_proc);

    sched_note_first_user_entry_if_ring3(current_proc);
    sched_mask_irq0_before_first_ring3_entry(current_proc);
    sched_emit_pre_dispatch_text_walk_proof(current_proc);
    context_switch(&prev->context, &current_proc->context);

    fb_print("[PANIC] sched_exit_current returned unexpectedly\n");
    for (;;) __asm__ volatile("cli; hlt");
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
