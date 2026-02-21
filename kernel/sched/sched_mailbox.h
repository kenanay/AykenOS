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

// Internal functions (not exported from Ring0)
void sched_mailbox_init(void);
void sched_mailbox_selftest(void);
