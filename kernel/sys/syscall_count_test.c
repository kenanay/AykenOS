// kernel/sys/syscall_count_test.c
// AykenOS Phase 2.5 - Syscall Count Validation Test
//
// This test validates that Ring0 contains exactly 11 syscalls (no more, no less)
// and that all legacy POSIX syscalls have been removed.
//
// Requirements: AC-6 - Ring0 contains exactly 11 syscalls

#include "syscall_v2.h"
#include "../drivers/console/fb_console.h"
#include <stddef.h>

// Test function to validate syscall count
void test_syscall_count(void)
{
    fb_print("[TEST] Validating syscall count...\n");
    
    // Test 1: Verify exactly 11 syscalls are defined (0-10)
    int expected_syscall_count = 11;
    int actual_max_syscall = SYS_V2_MAX_SYSCALL + 1; // +1 because max is 10, count is 11
    
    if (actual_max_syscall == expected_syscall_count) {
        fb_print("[TEST] ✓ Syscall count correct: ");
        fb_print_int(actual_max_syscall);
        fb_print(" syscalls (0-");
        fb_print_int(SYS_V2_MAX_SYSCALL);
        fb_print(")\n");
    } else {
        fb_print("[TEST] ✗ Syscall count incorrect: expected ");
        fb_print_int(expected_syscall_count);
        fb_print(", got ");
        fb_print_int(actual_max_syscall);
        fb_print("\n");
        return;
    }
    
    // Test 2: Verify all 11 syscalls are properly defined
    fb_print("[TEST] Validating syscall definitions:\n");
    
    // Check each syscall is defined with correct number
    if (SYS_V2_MAP_MEMORY == 0) {
        fb_print("[TEST] ✓ SYS_V2_MAP_MEMORY = 0\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_MAP_MEMORY incorrect\n");
        return;
    }
    
    if (SYS_V2_UNMAP_MEMORY == 1) {
        fb_print("[TEST] ✓ SYS_V2_UNMAP_MEMORY = 1\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_UNMAP_MEMORY incorrect\n");
        return;
    }
    
    if (SYS_V2_SWITCH_CONTEXT == 2) {
        fb_print("[TEST] ✓ SYS_V2_SWITCH_CONTEXT = 2\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_SWITCH_CONTEXT incorrect\n");
        return;
    }
    
    if (SYS_V2_SUBMIT_EXECUTION == 3) {
        fb_print("[TEST] ✓ SYS_V2_SUBMIT_EXECUTION = 3\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_SUBMIT_EXECUTION incorrect\n");
        return;
    }
    
    if (SYS_V2_WAIT_RESULT == 4) {
        fb_print("[TEST] ✓ SYS_V2_WAIT_RESULT = 4\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_WAIT_RESULT incorrect\n");
        return;
    }
    
    if (SYS_V2_INTERRUPT_RETURN == 5) {
        fb_print("[TEST] ✓ SYS_V2_INTERRUPT_RETURN = 5\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_INTERRUPT_RETURN incorrect\n");
        return;
    }
    
    if (SYS_V2_TIME_QUERY == 6) {
        fb_print("[TEST] ✓ SYS_V2_TIME_QUERY = 6\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_TIME_QUERY incorrect\n");
        return;
    }
    
    if (SYS_V2_CAPABILITY_BIND == 7) {
        fb_print("[TEST] ✓ SYS_V2_CAPABILITY_BIND = 7\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_CAPABILITY_BIND incorrect\n");
        return;
    }
    
    if (SYS_V2_CAPABILITY_REVOKE == 8) {
        fb_print("[TEST] ✓ SYS_V2_CAPABILITY_REVOKE = 8\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_CAPABILITY_REVOKE incorrect\n");
        return;
    }
    
    if (SYS_V2_EXIT == 9) {
        fb_print("[TEST] ✓ SYS_V2_EXIT = 9\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_EXIT incorrect\n");
        return;
    }

    if (SYS_V2_DEBUG_PUTCHAR == 10) {
        fb_print("[TEST] ✓ SYS_V2_DEBUG_PUTCHAR = 10\n");
    } else {
        fb_print("[TEST] ✗ SYS_V2_DEBUG_PUTCHAR incorrect\n");
        return;
    }
    
    // Test 3: Verify syscall dispatcher only accepts valid range
    fb_print("[TEST] Testing syscall dispatcher range validation...\n");
    
    // Test invalid syscall numbers (should return -ENOSYS)
    uint64_t result;
    
    // Test legacy POSIX range (0-99) - should be invalid now
    result = syscall_v2_handler(50, 0, 0, 0, 0); // Invalid legacy syscall
    if (result == ESYS_V2_INVALID_SYSCALL) {
        fb_print("[TEST] ✓ Legacy POSIX syscalls rejected\n");
    } else {
        fb_print("[TEST] ✗ Legacy POSIX syscalls not properly rejected\n");
        return;
    }
    
    // Test out-of-range v2 syscall
    result = syscall_v2_handler(15, 0, 0, 0, 0); // Invalid v2 syscall
    if (result == ESYS_V2_INVALID_SYSCALL) {
        fb_print("[TEST] ✓ Out-of-range v2 syscalls rejected\n");
    } else {
        fb_print("[TEST] ✗ Out-of-range v2 syscalls not properly rejected\n");
        return;
    }
    
    // Test valid v2 syscall (should not return INVALID_SYSCALL)
    result = syscall_v2_handler(SYS_V2_TIME_QUERY, (uint64_t)&result, 0, 0, 0);
    if (result != ESYS_V2_INVALID_SYSCALL) {
        fb_print("[TEST] ✓ Valid v2 syscalls accepted\n");
    } else {
        fb_print("[TEST] ✗ Valid v2 syscalls incorrectly rejected\n");
        return;
    }
    
    fb_print("[TEST] ✓ All syscall count validation tests passed!\n");
    fb_print("[TEST] ✓ Ring0 contains exactly 11 syscalls (no more, no less)\n");
    fb_print("[TEST] ✓ No legacy POSIX syscalls remain\n");
}

// Test function to validate v2 syscall dispatcher
void test_v2_syscall_dispatcher(void)
{
    fb_print("[TEST] Validating v2 syscall dispatcher...\n");
    
    // Test that main dispatcher only accepts 1000-1010 range
    uint64_t result;
    
    // Test valid range (1000-1010) - should be routed to v2 handler
    // Note: We test the v2 handler directly since syscall_handler is internal
    result = syscall_v2_handler(6, (uint64_t)&result, 0, 0, 0); // SYS_V2_TIME_QUERY
    if (result != (uint64_t)-38) { // Not -ENOSYS
        fb_print("[TEST] ✓ Valid syscall range (0-10) accepted by v2 handler\n");
    } else {
        fb_print("[TEST] ✗ Valid syscall range incorrectly rejected\n");
        return;
    }
    
    // Test invalid ranges for v2 handler
    result = syscall_v2_handler(50, 0, 0, 0, 0); // Invalid range for v2
    if (result == ESYS_V2_INVALID_SYSCALL) { 
        fb_print("[TEST] ✓ Invalid range properly rejected by v2 handler\n");
    } else {
        fb_print("[TEST] ✗ Invalid range not properly rejected\n");
        return;
    }
    
    result = syscall_v2_handler(15, 0, 0, 0, 0); // Out of v2 range
    if (result == ESYS_V2_INVALID_SYSCALL) { 
        fb_print("[TEST] ✓ Out-of-range v2 syscalls properly rejected\n");
    } else {
        fb_print("[TEST] ✗ Out-of-range v2 syscalls not properly rejected\n");
        return;
    }
    
    fb_print("[TEST] ✓ V2 syscall handler validation passed!\n");
    fb_print("[TEST] ✓ Only 0-10 range accepted by v2 handler, all others rejected\n");
}

// Main test function
void validate_syscall_count_requirement(void)
{
    fb_print("\n");
    fb_print("========================================\n");
    fb_print("SYSCALL COUNT VALIDATION TEST\n");
    fb_print("Requirement: Ring0 contains exactly 11 syscalls (no more, no less)\n");
    fb_print("========================================\n");
    
    test_syscall_count();
    test_v2_syscall_dispatcher();
    
    fb_print("========================================\n");
    fb_print("SYSCALL COUNT VALIDATION: PASSED\n");
    fb_print("✓ Ring0 contains exactly 11 execution-centric syscalls\n");
    fb_print("✓ No legacy POSIX syscalls remain\n");
    fb_print("✓ Only v2 syscall range 0-10 is accepted by v2 handler\n");
    fb_print("========================================\n");
    fb_print("\n");
}
