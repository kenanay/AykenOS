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

void sched_mailbox_init(void);
ayken_sched_mailbox_t* sched_mailbox_get(void);

/* 0 accept, <0 reject (reason is stored in mb->reject_reason) */
int sched_mailbox_validate_candidate(ayken_sched_mailbox_t* mb, proc_t** out_proc);

/* MVP-0 proof: emits fb_console markers once per boot */
void sched_mailbox_selftest(void);
