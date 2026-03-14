/**
 * @file sched_hint_test.c
 * @brief Ring3 Scheduler Hint Test Harness
 * 
 * Test Strategy:
 * - Write valid hint → expect ACCEPT marker
 * - Write invalid hint → expect REJECT marker
 * - Deterministic test (no timing dependencies)
 * - CI gate validates markers in kernel log
 * 
 * Constitutional Compliance:
 * - No syscalls (mailbox pre-mapped)
 * - No Ring0 exports
 * - Pure Ring3 policy test
 * - Fail-closed validation
 * 
 * Copyright © 2026 Kenan AY
 * License: ASAL v1.0 / ACL v1.0
 */

#include "../sched_hint.h"
#include <stdio.h>

/**
 * Test 1: Valid hint (should ACCEPT)
 * 
 * Expected Ring0 behavior:
 * - Epoch advances (monotonic check passes)
 * - PID valid (1 <= pid <= 1000)
 * - No torn read (double-read consistent)
 * - Marker: [[AYKEN_SCHED_MB_ACCEPT]] pid=42 epoch=1
 */
void test_valid_hint(void) {
    printf("[TEST] Writing valid hint: pid=42\n");
    
    // Write valid scheduling hint
    ayken_sched_hint(42);
    
    // Read back for verification (debugging only)
    uint64_t epoch;
    uint32_t pid;
    ayken_sched_hint_read(&epoch, &pid);
    
    printf("[TEST] Mailbox state: epoch=%llu pid=%u\n", epoch, pid);
    printf("[TEST] Waiting for timer tick validation...\n");
    
    // Ring0 validates on next timer tick
    // CI gate will check for ACCEPT marker in kernel log
}

/**
 * Test 2: Invalid PID (should REJECT)
 * 
 * Expected Ring0 behavior:
 * - PID out of range (pid > 1000)
 * - Marker: [[AYKEN_SCHED_MB_REJECT]] reason=3 epoch=2 pid=2147483647
 */
void test_invalid_pid(void) {
    printf("[TEST] Writing invalid hint: pid=2147483647 (out of range)\n");
    
    // Write invalid PID (exceeds kernel limit)
    ayken_sched_hint(2147483647);
    
    uint64_t epoch;
    uint32_t pid;
    ayken_sched_hint_read(&epoch, &pid);
    
    printf("[TEST] Mailbox state: epoch=%llu pid=%u\n", epoch, pid);
    printf("[TEST] Expecting REJECT (reason=3, invalid PID)...\n");
}

/**
 * Test 3: Epoch monotonicity
 * 
 * Expected Ring0 behavior:
 * - First write: ACCEPT (epoch advances)
 * - Second write: ACCEPT (epoch advances again)
 * - Epochs are strictly increasing
 */
void test_epoch_monotonicity(void) {
    printf("[TEST] Testing epoch monotonicity...\n");
    
    uint64_t epoch1, epoch2;
    uint32_t pid;
    
    // First hint
    ayken_sched_hint(10);
    ayken_sched_hint_read(&epoch1, &pid);
    printf("[TEST] First hint: epoch=%llu pid=%u\n", epoch1, pid);
    
    // Second hint (epoch should advance)
    ayken_sched_hint(20);
    ayken_sched_hint_read(&epoch2, &pid);
    printf("[TEST] Second hint: epoch=%llu pid=%u\n", epoch2, pid);
    
    if (epoch2 > epoch1) {
        printf("[TEST] ✓ Epoch monotonicity verified\n");
    } else {
        printf("[TEST] ✗ Epoch monotonicity FAILED\n");
    }
}

/**
 * Main test entry point
 * 
 * Test Execution:
 * 1. Valid hint (expect ACCEPT)
 * 2. Invalid PID (expect REJECT)
 * 3. Epoch monotonicity (expect increasing epochs)
 * 
 * CI Gate Validation:
 * - Parse kernel log for markers
 * - Verify ACCEPT/REJECT counts
 * - Check marker format (pid=, epoch= fields)
 */
int main(void) {
    printf("=== Ring3 Scheduler Hint Test ===\n\n");
    
    // Test 1: Valid hint
    test_valid_hint();
    printf("\n");
    
    // Test 2: Invalid PID
    test_invalid_pid();
    printf("\n");
    
    // Test 3: Epoch monotonicity
    test_epoch_monotonicity();
    printf("\n");
    
    printf("=== Test Complete ===\n");
    printf("Check kernel log for markers:\n");
    printf("  - [[AYKEN_SCHED_MB_ACCEPT]] (expected: 3)\n");
    printf("  - [[AYKEN_SCHED_MB_REJECT]] (expected: 1, reason=3)\n");
    
    return 0;
}
