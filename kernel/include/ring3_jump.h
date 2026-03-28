// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Ring3 Jump Interface (Phase 10-A2)
// Authority: Phase 10 specification
// Constitutional: Ring0 mechanism only (no policy)

#ifndef RING3_JUMP_H
#define RING3_JUMP_H

#include <stdint.h>
#include "ring3_contract.h"

// Ring3 transition mechanism (assembly)
// Transitions from Ring0 to Ring3 via IRETQ
// Parameters:
//   rip: User entry point (canonical address)
//   rsp: User stack pointer (canonical, aligned)
//   user_cr3: User page table physical address
// Preconditions:
//   - TSS.RSP0 set to valid kernel stack
//   - User RIP/RSP are canonical, in-range, and mapped in the target CR3
//   - Target CR3 can still reach the canonical post-CR3 transition text, IDT, and rsp0
//   - Fetch-probe builds additionally require the diagnostic trampoline alias
//   - Selector values come from the frozen ring3 contract header
extern void ring3_enter(uint64_t rip, uint64_t rsp, uint64_t user_cr3);

// Ring3 initialization (C wrapper)
// Loads embedded user ELF, creates a runnable user process and returns.
// Actual Ring3 entry happens via scheduler/context-switch path.
void jump_to_ring3(void);

#endif // RING3_JUMP_H
