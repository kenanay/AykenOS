/**
 * @file sched_hint.h
 * @brief Ring3 Scheduler Hint Interface
 * 
 * Constitutional Design:
 * - Ring3 Policy: Scheduling decisions in userspace
 * - Ring0 Mechanism: Validation only (epoch, atomicity, PID)
 * - No syscalls required (mailbox pre-mapped at boot)
 * - Fail-closed: Invalid hints rejected by Ring0
 * 
 * ABI Compliance:
 * - No new syscalls (1000-1011 frozen)
 * - No Ring0 exports (165/165 ceiling maintained)
 * - Fixed VA: 0x700000 (SCHED_MAILBOX_VA)
 * 
 * Copyright © 2026 Kenan AY
 * License: ASAL v1.0 / ACL v1.0
 */

#ifndef AYKEN_SCHED_HINT_H
#define AYKEN_SCHED_HINT_H

#include <stdint.h>

/**
 * Fixed virtual address for scheduler mailbox
 * Mapped by Ring0 at process creation (proc_create)
 * One mailbox per process (per-process isolation)
 */
#define SCHED_MAILBOX_VA 0x700000UL

/**
 * Scheduler mailbox structure
 * Ring3 writes, Ring0 validates on timer tick
 * 
 * Layout matches shared/abi/ayken_abi.h
 */
typedef struct {
    uint64_t epoch;           // Monotonic counter (replay prevention)
    uint32_t candidate_pid;   // Scheduling hint (which PID to run next)
    uint32_t reserved;        // Padding (future use)
} sched_mailbox_t;

/**
 * Write scheduling hint to mailbox
 * 
 * Ring3 Policy Decision:
 * - Caller decides which PID should run next
 * - No kernel involvement in decision logic
 * - Pure userspace policy
 * 
 * Ring0 Validation (on timer tick):
 * - Epoch monotonicity (replay prevention)
 * - Double-read atomicity (torn write detection)
 * - PID sanity check (0 < pid <= 1000)
 * 
 * Atomicity:
 * - Ring0 uses double-read to detect torn writes
 * - If epoch changes during read → REJECT (reason=1)
 * - No explicit memory barriers needed (single-core, validation profile)
 * 
 * @param candidate_pid PID to hint for next scheduling decision
 * 
 * @note This function does NOT block or wait for Ring0 validation
 * @note Validation happens asynchronously on next timer tick
 * @note Invalid hints are silently rejected by Ring0 (fail-closed)
 */
void ayken_sched_hint(uint32_t candidate_pid);

/**
 * Read current mailbox state (for debugging)
 * 
 * @param epoch_out Output: current epoch value
 * @param pid_out Output: current candidate PID
 * 
 * @note This is for debugging only, not for production logic
 * @note Ring3 should not make decisions based on mailbox state
 */
void ayken_sched_hint_read(uint64_t *epoch_out, uint32_t *pid_out);

#endif /* AYKEN_SCHED_HINT_H */
