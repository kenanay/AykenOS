// kernel/sys/capability_security_test.c
// AykenOS Phase 2.1 - Capability Security Enforcement Test
//
// This test validates that the capability system properly enforces security
// and prevents privilege escalation attacks as required by NFR-3.1 and NFR-3.3
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 10, 2026

#include <stdint.h>
#include <stddef.h>
#include "../include/capability.h"
#include "syscall_v2.h"

// Forward declarations to avoid including problematic headers
void fb_print(const char *s);
void fb_print_int(int64_t value);
void fb_print_hex(uint64_t v);

// Function declaration for external use
int capability_security_run_all_tests(void);

/**
 * test_privilege_escalation_prevention - Test that capability system prevents privilege escalation
 * 
 * Requirements: NFR-3.1 - Capability system must prevent privilege escalation
 * 
 * Returns: 0 on success (security enforced), negative error code on failure
 */
int test_privilege_escalation_prevention(void)
{
    fb_print("[security_test] Testing privilege escalation prevention...\n");
    
    // Initialize capability system
    capability_system_init();
    
    // Test 1: Attempt to bind invalid capability (should fail)
    capability_token_t invalid_token = {99999, CAPABILITY_PERM_ADMIN, CAPABILITY_RESOURCE_SYSTEM};
    uint64_t result = sys_v2_capability_bind(1001, &invalid_token);
    
    if (result != (uint64_t)ESYS_V2_NO_CAPABILITY) {
        fb_print("[security_test] FAIL: Invalid capability was accepted (privilege escalation risk)\n");
        return -1;
    }
    
    fb_print("[security_test] ✓ Invalid capability rejected\n");
    
    // Test 2: Attempt to access memory without capability (should fail)
    result = sys_v2_map_memory(0x100000, 0x200000, 0x03);
    
    if (result != (uint64_t)ESYS_V2_NO_CAPABILITY) {
        fb_print("[security_test] FAIL: Memory mapping allowed without capability (privilege escalation)\n");
        return -2;
    }
    
    fb_print("[security_test] ✓ Memory access denied without capability\n");
    
    // Test 3: Attempt to switch context without capability (should fail)
    result = sys_v2_switch_context(1001, 1002);
    
    if (result != (uint64_t)ESYS_V2_NO_CAPABILITY) {
        fb_print("[security_test] FAIL: Context switch allowed without capability (privilege escalation)\n");
        return -3;
    }
    
    fb_print("[security_test] ✓ Context switch denied without capability\n");
    
    // Test 4: Create legitimate capability and verify it works
    capability_token_t memory_cap = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ_WRITE,
        0x200000, 4096
    );
    
    if (memory_cap.id == 0) {
        fb_print("[security_test] FAIL: Could not create legitimate capability\n");
        return -4;
    }
    
    // Bind the legitimate capability
    result = sys_v2_capability_bind(1001, &memory_cap);
    if (result != memory_cap.id) {
        fb_print("[security_test] FAIL: Could not bind legitimate capability\n");
        return -5;
    }
    
    fb_print("[security_test] ✓ Legitimate capability created and bound\n");
    
    fb_print("[security_test] Privilege escalation prevention: PASS\n");
    return 0;
}

/**
 * test_resource_access_mediation - Test that resource access is mediated through capability tokens
 * 
 * Requirements: NFR-3.3 - Resource access must be mediated through capability tokens
 * 
 * Returns: 0 on success (mediation enforced), negative error code on failure
 */
int test_resource_access_mediation(void)
{
    fb_print("[security_test] Testing resource access mediation...\n");
    
    // Test 1: Create capability with limited permissions
    capability_token_t read_only_cap = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ,  // Read-only
        0x300000, 4096
    );
    
    if (read_only_cap.id == 0) {
        fb_print("[security_test] FAIL: Could not create read-only capability\n");
        return -1;
    }
    
    // Test 2: Verify permission checking works
    int permission_result = capability_check_permission(&read_only_cap, CAPABILITY_PERM_WRITE);
    if (permission_result == CAPABILITY_SUCCESS) {
        fb_print("[security_test] FAIL: Write permission granted on read-only capability\n");
        return -2;
    }
    
    fb_print("[security_test] ✓ Write permission correctly denied on read-only capability\n");
    
    // Test 3: Verify read permission works
    permission_result = capability_check_permission(&read_only_cap, CAPABILITY_PERM_READ);
    if (permission_result != CAPABILITY_SUCCESS) {
        fb_print("[security_test] FAIL: Read permission denied on read-only capability\n");
        return -3;
    }
    
    fb_print("[security_test] ✓ Read permission correctly granted on read-only capability\n");
    
    // Test 4: Test bounds checking
    int bounds_result = capability_check_resource_access(&read_only_cap, 0x300000, 4096, CAPABILITY_PERM_READ);
    if (bounds_result != CAPABILITY_SUCCESS) {
        fb_print("[security_test] FAIL: Valid bounds check failed\n");
        return -4;
    }
    
    fb_print("[security_test] ✓ Valid bounds check passed\n");
    
    // Test 5: Test bounds violation detection
    bounds_result = capability_check_resource_access(&read_only_cap, 0x300000, 8192, CAPABILITY_PERM_READ);
    if (bounds_result == CAPABILITY_SUCCESS) {
        fb_print("[security_test] FAIL: Bounds violation not detected (buffer overflow risk)\n");
        return -5;
    }
    
    fb_print("[security_test] ✓ Bounds violation correctly detected\n");
    
    fb_print("[security_test] Resource access mediation: PASS\n");
    return 0;
}

/**
 * test_capability_revocation_security - Test that revoked capabilities cannot be used
 * 
 * Requirements: FR-2.2.3 - Capability revocation must immediately invalidate access rights
 * 
 * Returns: 0 on success (revocation enforced), negative error code on failure
 */
int test_capability_revocation_security(void)
{
    fb_print("[security_test] Testing capability revocation security...\n");
    
    // Test 1: Create and bind capability
    capability_token_t test_cap = capability_create(
        CAPABILITY_RESOURCE_DEVICE,
        CAPABILITY_PERM_READ_WRITE,
        0x400000, 1024
    );
    
    if (test_cap.id == 0) {
        fb_print("[security_test] FAIL: Could not create test capability\n");
        return -1;
    }
    
    uint64_t bind_result = sys_v2_capability_bind(2001, &test_cap);
    if (bind_result != test_cap.id) {
        fb_print("[security_test] FAIL: Could not bind test capability\n");
        return -2;
    }
    
    // Test 2: Verify capability works before revocation
    int validation_result = capability_validate(&test_cap);
    if (validation_result != CAPABILITY_SUCCESS) {
        fb_print("[security_test] FAIL: Capability invalid before revocation\n");
        return -3;
    }
    
    fb_print("[security_test] ✓ Capability valid before revocation\n");
    
    // Test 3: Revoke the capability
    uint64_t revoke_result = sys_v2_capability_revoke(test_cap.id);
    if (revoke_result != ESYS_V2_SUCCESS) {
        fb_print("[security_test] FAIL: Could not revoke capability\n");
        return -4;
    }
    
    fb_print("[security_test] ✓ Capability revoked successfully\n");
    
    // Test 4: Verify capability is invalid after revocation
    validation_result = capability_validate(&test_cap);
    if (validation_result != CAPABILITY_ERROR_REVOKED) {
        fb_print("[security_test] FAIL: Revoked capability still validates (security vulnerability)\n");
        return -5;
    }
    
    fb_print("[security_test] ✓ Revoked capability correctly invalidated\n");
    
    // Test 5: Attempt to use revoked capability (should fail)
    int permission_result = capability_check_permission(&test_cap, CAPABILITY_PERM_READ);
    if (permission_result != CAPABILITY_ERROR_REVOKED) {
        fb_print("[security_test] FAIL: Revoked capability still grants permissions (security vulnerability)\n");
        return -6;
    }
    
    fb_print("[security_test] ✓ Revoked capability correctly denies permissions\n");
    
    fb_print("[security_test] Capability revocation security: PASS\n");
    return 0;
}

/**
 * test_context_isolation - Test that capabilities are properly isolated between contexts
 * 
 * Requirements: FR-2.2.2 - Capability binding must associate permissions with execution contexts
 * 
 * Returns: 0 on success (isolation enforced), negative error code on failure
 */
int test_context_isolation(void)
{
    fb_print("[security_test] Testing context isolation...\n");
    
    // Test 1: Create capabilities for different contexts
    capability_token_t ctx1_cap = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ,
        0x500000, 4096
    );
    
    capability_token_t ctx2_cap = capability_create(
        CAPABILITY_RESOURCE_DEVICE,
        CAPABILITY_PERM_WRITE,
        0x600000, 2048
    );
    
    if (ctx1_cap.id == 0 || ctx2_cap.id == 0) {
        fb_print("[security_test] FAIL: Could not create test capabilities\n");
        return -1;
    }
    
    // Test 2: Bind capabilities to different contexts
    uint64_t bind1_result = sys_v2_capability_bind(3001, &ctx1_cap);
    uint64_t bind2_result = sys_v2_capability_bind(3002, &ctx2_cap);
    
    if (bind1_result != ctx1_cap.id || bind2_result != ctx2_cap.id) {
        fb_print("[security_test] FAIL: Could not bind capabilities to contexts\n");
        return -2;
    }
    
    // Test 3: Verify context 1 can access its capability
    capability_token_t *found_cap = capability_get_by_context(3001, CAPABILITY_RESOURCE_MEMORY);
    if (found_cap == NULL || found_cap->id != ctx1_cap.id) {
        fb_print("[security_test] FAIL: Context 1 cannot access its own capability\n");
        return -3;
    }
    
    fb_print("[security_test] ✓ Context 1 can access its own capability\n");
    
    // Test 4: Verify context 1 cannot access context 2's capability
    found_cap = capability_get_by_context(3001, CAPABILITY_RESOURCE_DEVICE);
    if (found_cap != NULL) {
        fb_print("[security_test] FAIL: Context 1 can access context 2's capability (isolation breach)\n");
        return -4;
    }
    
    fb_print("[security_test] ✓ Context 1 cannot access context 2's capability\n");
    
    // Test 5: Verify context 2 can access its capability
    found_cap = capability_get_by_context(3002, CAPABILITY_RESOURCE_DEVICE);
    if (found_cap == NULL || found_cap->id != ctx2_cap.id) {
        fb_print("[security_test] FAIL: Context 2 cannot access its own capability\n");
        return -5;
    }
    
    fb_print("[security_test] ✓ Context 2 can access its own capability\n");
    
    // Test 6: Verify context 2 cannot access context 1's capability
    found_cap = capability_get_by_context(3002, CAPABILITY_RESOURCE_MEMORY);
    if (found_cap != NULL) {
        fb_print("[security_test] FAIL: Context 2 can access context 1's capability (isolation breach)\n");
        return -6;
    }
    
    fb_print("[security_test] ✓ Context 2 cannot access context 1's capability\n");
    
    fb_print("[security_test] Context isolation: PASS\n");
    return 0;
}

/**
 * capability_security_run_all_tests - Run all capability security tests
 * 
 * Returns: 0 if all tests pass, negative error code if any test fails
 */
int capability_security_run_all_tests(void)
{
    fb_print("\n=== CAPABILITY SECURITY TEST SUITE ===\n");
    fb_print("Testing NFR-3.1: Capability system must prevent privilege escalation\n");
    fb_print("Testing NFR-3.3: Resource access must be mediated through capability tokens\n");
    fb_print("Testing FR-2.2.3: Capability revocation must immediately invalidate access rights\n");
    fb_print("Testing FR-2.2.2: Capability binding must associate permissions with execution contexts\n\n");
    
    int result;
    
    // Test 1: Privilege escalation prevention
    result = test_privilege_escalation_prevention();
    if (result != 0) {
        fb_print("[security_test] Privilege escalation prevention FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    // Test 2: Resource access mediation
    result = test_resource_access_mediation();
    if (result != 0) {
        fb_print("[security_test] Resource access mediation FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    // Test 3: Capability revocation security
    result = test_capability_revocation_security();
    if (result != 0) {
        fb_print("[security_test] Capability revocation security FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    // Test 4: Context isolation
    result = test_context_isolation();
    if (result != 0) {
        fb_print("[security_test] Context isolation FAILED with code ");
        fb_print_int(result);
        fb_print("\n");
        return result;
    }
    
    fb_print("\n=== ALL CAPABILITY SECURITY TESTS PASSED! ===\n");
    fb_print("✓ NFR-3.1: Privilege escalation prevention - ENFORCED\n");
    fb_print("✓ NFR-3.3: Resource access mediation - ENFORCED\n");
    fb_print("✓ FR-2.2.3: Capability revocation security - ENFORCED\n");
    fb_print("✓ FR-2.2.2: Context isolation - ENFORCED\n");
    fb_print("\nCapability system security enforcement: COMPLETE\n");
    fb_print("Task: 'Capability system enforces security' - IMPLEMENTED\n");
    fb_print("=====================================\n\n");
    
    return 0;
}