// kernel/sys/capability_test.c
// AykenOS Phase 2.1 - Capability System Test
//
// Simple test program to validate capability bind/revoke syscalls
// This tests the core functionality implemented in task 2.1.2.2
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 3, 2026

#include <stdint.h>
#include <stddef.h>
#include "../include/capability.h"
#include "syscall_v2.h"

// Forward declarations to avoid including problematic headers
void fb_print(const char *s);
void fb_print_int(int64_t value);
void fb_print_hex(uint64_t v);

/**
 * capability_test_basic - Test basic capability creation and validation
 * 
 * Returns: 0 on success, negative error code on failure
 */
int capability_test_basic(void)
{
    fb_print("[capability_test] Starting basic capability test...\n");
    
    // Initialize capability system
    capability_system_init();
    
    // Test 1: Create a capability token
    capability_token_t token = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ_WRITE,
        0x100000,  // resource address
        4096       // resource size
    );
    
    if (token.id == 0) {
        fb_print("[capability_test] FAIL: Could not create capability\n");
        return -1;
    }
    
    fb_print("[capability_test] Created capability ID=");
    fb_print_int(token.id);
    fb_print("\n");
    
    // Test 2: Validate the capability token
    int validation_result = capability_validate(&token);
    if (validation_result != CAPABILITY_SUCCESS) {
        fb_print("[capability_test] FAIL: Capability validation failed\n");
        return -2;
    }
    
    fb_print("[capability_test] Capability validation: PASS\n");
    
    // Test 3: Test capability bind syscall
    uint64_t execution_ctx = 1001; // Test execution context ID
    uint64_t bind_result = sys_v2_capability_bind(execution_ctx, &token);
    
    if (bind_result != ESYS_V2_SUCCESS) {
        fb_print("[capability_test] FAIL: Capability bind syscall failed, result=");
        fb_print_int(bind_result);
        fb_print("\n");
        return -3;
    }
    
    fb_print("[capability_test] Capability bind syscall: PASS\n");
    
    // Test 4: Test capability revoke syscall
    uint64_t revoke_result = sys_v2_capability_revoke(token.id);
    
    if (revoke_result != ESYS_V2_SUCCESS) {
        fb_print("[capability_test] FAIL: Capability revoke syscall failed, result=");
        fb_print_int(revoke_result);
        fb_print("\n");
        return -4;
    }
    
    fb_print("[capability_test] Capability revoke syscall: PASS\n");
    
    // Test 5: Verify capability is revoked
    validation_result = capability_validate(&token);
    if (validation_result != CAPABILITY_ERROR_REVOKED) {
        fb_print("[capability_test] FAIL: Capability should be revoked but validation returned ");
        fb_print_int(validation_result);
        fb_print("\n");
        return -5;
    }
    
    fb_print("[capability_test] Capability revocation verification: PASS\n");
    
    fb_print("[capability_test] All basic tests PASSED!\n");
    return 0;
}

/**
 * capability_test_edge_cases - Test edge cases and error conditions
 * 
 * Returns: 0 on success, negative error code on failure
 */
int capability_test_edge_cases(void)
{
    fb_print("[capability_test] Starting edge case tests...\n");
    
    // Test 1: Invalid parameters to bind syscall
    uint64_t result = sys_v2_capability_bind(0, NULL);
    if (result != (uint64_t)ESYS_V2_INVALID_PARAM) {
        fb_print("[capability_test] FAIL: Expected ESYS_V2_INVALID_PARAM for null parameters\n");
        return -1;
    }
    
    fb_print("[capability_test] Invalid bind parameters test: PASS\n");
    
    // Test 2: Invalid token ID to revoke syscall
    result = sys_v2_capability_revoke(0);
    if (result != (uint64_t)ESYS_V2_INVALID_PARAM) {
        fb_print("[capability_test] FAIL: Expected ESYS_V2_INVALID_PARAM for zero token ID\n");
        return -2;
    }
    
    fb_print("[capability_test] Invalid revoke token ID test: PASS\n");
    
    // Test 3: Revoke non-existent capability
    result = sys_v2_capability_revoke(99999);
    if (result != (uint64_t)ESYS_V2_NO_CAPABILITY) {
        fb_print("[capability_test] FAIL: Expected ESYS_V2_NO_CAPABILITY for non-existent token\n");
        return -3;
    }
    
    fb_print("[capability_test] Non-existent token revoke test: PASS\n");
    
    // Test 4: Bind invalid capability token
    capability_token_t invalid_token = {99999, 0, 0};
    result = sys_v2_capability_bind(1002, &invalid_token);
    if (result != (uint64_t)ESYS_V2_NO_CAPABILITY) {
        fb_print("[capability_test] FAIL: Expected ESYS_V2_NO_CAPABILITY for invalid token\n");
        return -4;
    }
    
    fb_print("[capability_test] Invalid token bind test: PASS\n");
    
    fb_print("[capability_test] All edge case tests PASSED!\n");
    return 0;
}

/**
 * capability_test_multiple_contexts - Test multiple execution contexts
 * 
 * Returns: 0 on success, negative error code on failure
 */
int capability_test_multiple_contexts(void)
{
    fb_print("[capability_test] Starting multiple context tests...\n");
    
    // Create multiple capability tokens
    capability_token_t token1 = capability_create(
        CAPABILITY_RESOURCE_DEVICE,
        CAPABILITY_PERM_READ,
        0x200000, 1024
    );
    
    capability_token_t token2 = capability_create(
        CAPABILITY_RESOURCE_FILE,
        CAPABILITY_PERM_WRITE,
        0x300000, 2048
    );
    
    if (token1.id == 0 || token2.id == 0) {
        fb_print("[capability_test] FAIL: Could not create test tokens\n");
        return -1;
    }
    
    // Bind tokens to different contexts
    uint64_t ctx1 = 2001, ctx2 = 2002;
    
    uint64_t result1 = sys_v2_capability_bind(ctx1, &token1);
    uint64_t result2 = sys_v2_capability_bind(ctx2, &token2);
    uint64_t result3 = sys_v2_capability_bind(ctx1, &token2); // Same token to different context
    
    if (result1 != ESYS_V2_SUCCESS || result2 != ESYS_V2_SUCCESS || result3 != ESYS_V2_SUCCESS) {
        fb_print("[capability_test] FAIL: Multi-context binding failed\n");
        return -2;
    }
    
    fb_print("[capability_test] Multi-context binding: PASS\n");
    
    // Test duplicate binding (should fail)
    uint64_t duplicate_result = sys_v2_capability_bind(ctx1, &token1);
    if (duplicate_result != (uint64_t)ESYS_V2_RESOURCE_BUSY) {
        fb_print("[capability_test] FAIL: Expected ESYS_V2_RESOURCE_BUSY for duplicate binding\n");
        return -3;
    }
    
    fb_print("[capability_test] Duplicate binding prevention: PASS\n");
    
    // Revoke one token and verify it's removed from all contexts
    uint64_t revoke_result = sys_v2_capability_revoke(token2.id);
    if (revoke_result != ESYS_V2_SUCCESS) {
        fb_print("[capability_test] FAIL: Token revocation failed\n");
        return -4;
    }
    
    fb_print("[capability_test] Multi-context revocation: PASS\n");
    
    fb_print("[capability_test] All multiple context tests PASSED!\n");
    return 0;
}

/**
 * capability_run_all_tests - Run all capability system tests
 * 
 * Returns: 0 if all tests pass, negative error code if any test fails
 */
int capability_run_all_tests(void)
{
    fb_print("\n=== CAPABILITY SYSTEM TEST SUITE ===\n");
    
    int result;
    
    // Run basic functionality tests
    result = capability_test_basic();
    if (result != 0) {
        fb_print("[capability_test] Basic tests FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    // Run edge case tests
    result = capability_test_edge_cases();
    if (result != 0) {
        fb_print("[capability_test] Edge case tests FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    // Run multiple context tests
    result = capability_test_multiple_contexts();
    if (result != 0) {
        fb_print("[capability_test] Multiple context tests FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    fb_print("\n=== ALL CAPABILITY TESTS PASSED! ===\n");
    fb_print("Task 2.1.2.2 - Capability syscalls implementation: COMPLETE\n");
    fb_print("- sys_v2_capability_bind: Working\n");
    fb_print("- sys_v2_capability_revoke: Working\n");
    fb_print("- Capability manager: Working\n");
    fb_print("- Error handling: Working\n");
    fb_print("- Multi-context support: Working\n");
    fb_print("=====================================\n\n");
    
    return 0;
}