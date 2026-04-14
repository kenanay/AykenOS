/**
 * @file sched_hint.c
 * @brief Ring3 Scheduler Hint Implementation
 * 
 * Constitutional Design:
 * - Ring3 Policy: Scheduling decisions in userspace
 * - Ring0 Mechanism: Validation only (epoch, atomicity, PID)
 * - Monotonic epoch counter (no even/odd protocol)
 * - Fail-closed: Ring0 rejects invalid hints
 * 
 * Atomicity Model:
 * - Ring0 uses double-read to detect torn writes
 * - No explicit memory barriers (single-core, validation profile)
 * - Write order: candidate_pid first, then epoch
 * - Ring0 reads: epoch, pid, epoch (torn if epochs differ)
 * 
 * Copyright © 2026 Kenan AY
 * License: ASAL v1.0 / ACL v1.0
 */

#include "sched_hint.h"

/**
 * Write scheduling hint to mailbox
 * 
 * Implementation Notes:
 * - Volatile pointer prevents compiler reordering
 * - Write candidate_pid before epoch (Ring0 reads in this order)
 * - Epoch is strictly monotonic (no even/odd semantics)
 * - No syscall needed (mailbox pre-mapped by Ring0)
 * 
 * Ring0 Validation (kernel/sched/sched_mailbox.c):
 * - Double-read: e1 = epoch, pid = candidate_pid, e2 = epoch
 * - If e1 != e2 → REJECT (torn write, reason=1)
 * - If e1 <= last_epoch → REJECT (replay, reason=2)
 * - If pid invalid → REJECT (sanity, reason=3)
 * - Otherwise → ACCEPT
 * 
 * @param candidate_pid PID to hint for next scheduling decision
 */
void ayken_sched_hint(uint32_t candidate_pid) {
    // Volatile prevents compiler optimization/reordering
    volatile sched_mailbox_t *mb = (volatile sched_mailbox_t *)SCHED_MAILBOX_VA;
    
    // Read current epoch
    uint64_t current_epoch = mb->epoch;
    
    // Advance epoch (strictly monotonic)
    uint64_t next_epoch = current_epoch + 1;
    
    // SEQLOCK PROTOCOL: Write payload FIRST, epoch LAST
    // 1. Write candidate_pid first (payload)
    mb->candidate_pid = candidate_pid;
    
    // 2. Write barrier - ensure payload write completes before epoch write
    //    This prevents torn reads where Ring0 sees new epoch but old candidate_pid
    __asm__ volatile("sfence" ::: "memory");  // Write barrier (x86-64)
    
    // 3. Write epoch last (commit indicator)
    mb->epoch = next_epoch;
    
    // Ring0 double-read (seqlock consumer) will detect torn writes if epoch changes
    // between first and second read. The write barrier ensures epoch is written last.
    
    // Validation happens asynchronously on next timer tick
    // Ring0 emits markers: [[AYKEN_SCHED_MB_ACCEPT]] or [[AYKEN_SCHED_MB_REJECT]]
}

/**
 * Read current mailbox state (debugging only)
 * 
 * @param epoch_out Output: current epoch value
 * @param pid_out Output: current candidate PID
 * 
 * @note This is for debugging/testing only
 * @note Production code should not make decisions based on mailbox state
 * @note Ring3 policy should be stateless (write hints, don't read back)
 */
void ayken_sched_hint_read(uint64_t *epoch_out, uint32_t *pid_out) {
    volatile sched_mailbox_t *mb = (volatile sched_mailbox_t *)SCHED_MAILBOX_VA;
    
    if (epoch_out) {
        *epoch_out = mb->epoch;
    }
    
    if (pid_out) {
        *pid_out = mb->candidate_pid;
    }
}
