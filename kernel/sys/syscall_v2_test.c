// kernel/sys/syscall_v2_test.c
// Test for execution-centric syscall interface (Phase 2.5 Final)
//
// This test verifies that the final syscall dispatcher correctly handles
// only the execution-centric syscalls:
// - SYS_V2_BASE..SYS_V2_LAST range: Execution-centric (v2) syscalls
// - All other ranges: Return -ENOSYS
//
// Requirements: AC-6 - Only execution-centric syscalls remain
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Updated: January 10, 2026 - Phase 2.5 Legacy Cleanup

#include <stdint.h>
#include "../include/syscall.h"
#include "syscall_v2.h"

// Forward declarations to avoid including problematic headers
void fb_print(const char *s);
void fb_print_int(int64_t value);

/**
 * test_execution_centric_syscalls - Test the execution-centric syscall interface
 * 
 * This function tests representative execution-centric syscalls to ensure they
 * are properly routed and handled by the v2 syscall dispatcher.
 */
void test_execution_centric_syscalls(void)
{
    fb_print("[syscall_test] Testing execution-centric syscall interface...\n");
    
    // Test 1: SYS_V2_MAP_MEMORY (1000)
    fb_print("[syscall_test] Test 1: V2 syscall (map_memory)\n");
    uint64_t result1 = syscall_handler(1000, 0x400000, 0x100000, 0x01, 0);
    fb_print("[syscall_test] V2 map_memory result: ");
    fb_print_int(result1);
    fb_print(" (should be 0 = SUCCESS)\n");
    
    // Test 2: SYS_V2_TIME_QUERY (1006)
    fb_print("[syscall_test] Test 2: V2 syscall (time_query)\n");
    uint64_t time_result = 0;
    uint64_t result2 = syscall_handler(1006, 0, (uint64_t)&time_result, 0, 0);
    fb_print("[syscall_test] V2 time_query result: ");
    fb_print_int(result2);
    fb_print(" (should be 0 = SUCCESS)\n");
    
    // Test 3: SYS_V2_CAPABILITY_BIND (1007)
    fb_print("[syscall_test] Test 3: V2 syscall (capability_bind)\n");
    capability_token_t test_token = {0, 0x01, CAP_RESOURCE_MEMORY};
    uint64_t result3 = syscall_handler(1007, 1, (uint64_t)&test_token, 0, 0);
    fb_print("[syscall_test] V2 capability_bind result: ");
    fb_print_int(result3);
    fb_print(" (should be capability ID > 0)\n");
    
    // Test 4: SYS_V2_EXIT (1009)
    fb_print("[syscall_test] Test 4: V2 syscall (exit) - NOTE: This will loop\n");
    // Note: We don't actually call exit as it will loop forever
    fb_print("[syscall_test] Skipping exit test to avoid infinite loop\n");
    
    fb_print("[syscall_test] Execution-centric syscall test completed.\n");
}

/**
 * test_syscall_numbering_plan - Test the final syscall numbering plan
 * 
 * This function tests that only the SYS_V2_BASE..SYS_V2_LAST range is valid and all
 * other syscall numbers return -ENOSYS.
 */
void test_syscall_numbering_plan(void)
{
    fb_print("[syscall_test] Testing final syscall numbering plan...\n");
    
    // Test v2 range boundaries (only valid range)
    fb_print("[syscall_test] V2 range: SYS_V2_BASE..SYS_V2_LAST (ONLY valid range)\n");
    fb_print("[syscall_test] Testing syscall 1000 (map_memory): ");
    uint64_t r1000 = syscall_handler(1000, 0x400000, 0x100000, 0x01, 0);
    fb_print_int(r1000);
    fb_print(" (should be 0 = SUCCESS)\n");
    
    fb_print("[syscall_test] Testing syscall 1009 (exit): ");
    // Note: We don't actually call exit as it will loop forever
    fb_print("SKIPPED (would cause infinite loop)\n");
    
    // Test invalid ranges (all should return -38 = -ENOSYS)
    fb_print("[syscall_test] Invalid ranges (should ALL return -38):\n");
    
    fb_print("[syscall_test] Testing syscall 0 (old v1): ");
    uint64_t r0 = syscall_handler(0, 0, 0, 0, 0);
    fb_print_int(r0);
    fb_print("\n");
    
    fb_print("[syscall_test] Testing syscall 1 (old v1): ");
    uint64_t r1 = syscall_handler(1, 0, 0, 0, 0);
    fb_print_int(r1);
    fb_print("\n");
    
    fb_print("[syscall_test] Testing syscall 99 (old v1): ");
    uint64_t r99 = syscall_handler(99, 0, 0, 0, 0);
    fb_print_int(r99);
    fb_print("\n");
    
    fb_print("[syscall_test] Testing syscall 100: ");
    uint64_t r100 = syscall_handler(100, 0, 0, 0, 0);
    fb_print_int(r100);
    fb_print("\n");
    
    fb_print("[syscall_test] Testing syscall 999: ");
    uint64_t r999 = syscall_handler(999, 0, 0, 0, 0);
    fb_print_int(r999);
    fb_print("\n");
    
    fb_print("[syscall_test] Testing syscall 1015: ");
    uint64_t r1015 = syscall_handler(1015, 0, 0, 0, 0);
    fb_print_int(r1015);
    fb_print("\n");
    
    fb_print("[syscall_test] Testing syscall 2000: ");
    uint64_t r2000 = syscall_handler(2000, 0, 0, 0, 0);
    fb_print_int(r2000);
    fb_print("\n");
    
    fb_print("[syscall_test] Final numbering plan test completed.\n");
    fb_print("[syscall_test] SUMMARY: Only SYS_V2_BASE..SYS_V2_LAST should work, all others return -38\n");
}
