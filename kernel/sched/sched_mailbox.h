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

// Internal functions (not exported from Ring0)
void sched_mailbox_init(void);
void sched_mailbox_selftest(void);
int sched_mailbox_validate_ring3(proc_t *proc); // MVP-1: Ring3 validation
void sched_mailbox_test_ring3_simulation(proc_t *proc); // MVP-2: Ring3 simulation test
