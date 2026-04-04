// kernel/sched/sched_mailbox.h
// Scheduler Bridge Mailbox - Ring0 Mechanism Interface
//
// Constitutional requirement: Ring0 mechanism only, Ring3 policy
// This header defines the Ring0 mechanism interface for scheduler bridge.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Phase: 4.5 - Scheduler Bridge MVP

#pragma once

#include "../include/proc.h"
#include "../include/sched_mailbox_abi.h"

// MVP-1: Fixed VA for per-process mailbox (Ring3 write, Ring0 read)
// Location: 0x700000 (7 MiB) - safe from loader collision
// Mapping: USER | WRITABLE | PRESENT (per-process, isolated)
#define SCHED_MAILBOX_VA 0x700000ULL

enum sched_mailbox_control_op {
    SCHED_MAILBOX_CONTROL_INIT = 0,
    SCHED_MAILBOX_CONTROL_VALIDATE_RING3,
    SCHED_MAILBOX_CONTROL_SELFTEST,
    SCHED_MAILBOX_CONTROL_TEST_RING3_SIMULATION,
    SCHED_MAILBOX_CONTROL_GATE4_EPOCH1_PENDING,
};

int sched_mailbox_control(uint32_t op, proc_t *proc);

static inline void sched_mailbox_init(void)
{
    (void)sched_mailbox_control(SCHED_MAILBOX_CONTROL_INIT, NULL);
}

static inline void sched_mailbox_selftest(void)
{
    (void)sched_mailbox_control(SCHED_MAILBOX_CONTROL_SELFTEST, NULL);
}

static inline int sched_mailbox_validate_ring3(proc_t *proc)
{
    return sched_mailbox_control(SCHED_MAILBOX_CONTROL_VALIDATE_RING3, proc);
}

static inline void sched_mailbox_test_ring3_simulation(proc_t *proc)
{
    (void)sched_mailbox_control(SCHED_MAILBOX_CONTROL_TEST_RING3_SIMULATION, proc);
}

static inline int sched_mailbox_gate4_epoch1_pending(void)
{
    return sched_mailbox_control(SCHED_MAILBOX_CONTROL_GATE4_EPOCH1_PENDING, NULL);
}
