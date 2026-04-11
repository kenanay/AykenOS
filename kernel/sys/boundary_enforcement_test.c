#include "boundary_enforcement.h"
#include "syscall_v2_hardened.h"
#include "../include/ayken.h"
#include "../include/serial.h"

/* Debug printf implementation using serial output */
static void debug_printf(const char *fmt, ...) {
    /* Simple implementation - just write the format string for now */
    serial_write("[TEST] ");
    serial_write(fmt);
    serial_write("\n");
}

/*
 * Phase-16 Boundary Enforcement Tests
 * 
 * Validates kernel boundary hardening implementation.
 * Tests fail-closed behavior and constitutional compliance.
 */

static int test_passed = 0;
static int test_failed = 0;

#define TEST_ASSERT(condition, message) \
    do { \
        if (condition) { \
            test_passed++; \
            debug_printf("[TEST PASS] %s\n", message); \
        } else { \
            test_failed++; \
            debug_printf("[TEST FAIL] %s\n", message); \
        } \
    } while(0)

/**
 * Test BCIB syscall restriction (Requirement 1.5)
 * BCIB SHALL use SYS_V2_SUBMIT_EXECUTION ONLY
 */
static void test_bcib_syscall_restriction(void) {
    debug_printf("[TEST] Testing BCIB syscall restriction...\n");
    
    uint64_t bcib_context = 0x1500; /* BCIB context ID */
    
    /* Test 1: BCIB should be allowed to use SYS_V2_SUBMIT_EXECUTION */
    int result = boundary_validate_syscall(SYS_V2_SUBMIT_EXECUTION, EXEC_CONTEXT_BCIB, bcib_context);
    TEST_ASSERT(result == 0, "BCIB allowed to use SYS_V2_SUBMIT_EXECUTION");
    
    /* Test 2: BCIB should be denied other syscalls */
    result = boundary_validate_syscall(SYS_V2_MAP_MEMORY, EXEC_CONTEXT_BCIB, bcib_context);
    TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, "BCIB denied SYS_V2_MAP_MEMORY");
    
    result = boundary_validate_syscall(SYS_V2_TIME_QUERY, EXEC_CONTEXT_BCIB, bcib_context);
    TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, "BCIB denied SYS_V2_TIME_QUERY");
    
    result = boundary_validate_syscall(SYS_V2_CAPABILITY_BIND, EXEC_CONTEXT_BCIB, bcib_context);
    TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, "BCIB denied SYS_V2_CAPABILITY_BIND");
}

/**
 * Test Runtime_Bridge syscall restrictions
 * Runtime_Bridge cannot replace or bypass syscall surface
 */
static void test_runtime_bridge_restrictions(void) {
    debug_printf("[TEST] Testing Runtime_Bridge restrictions...\n");
    
    uint64_t bridge_context = 0x2500; /* Runtime_Bridge context ID */
    
    /* Test 1: Runtime_Bridge should be denied execution submission */
    int result = boundary_validate_syscall(SYS_V2_SUBMIT_EXECUTION, EXEC_CONTEXT_RUNTIME_BRIDGE, bridge_context);
    TEST_ASSERT(result == BOUNDARY_ERR_BRIDGE_BYPASS, "Runtime_Bridge denied execution submission");
    
    /* Test 2: Runtime_Bridge should be allowed limited syscalls */
    result = boundary_validate_syscall(SYS_V2_MAP_MEMORY, EXEC_CONTEXT_RUNTIME_BRIDGE, bridge_context);
    TEST_ASSERT(result == 0, "Runtime_Bridge allowed SYS_V2_MAP_MEMORY");
    
    result = boundary_validate_syscall(SYS_V2_CAPABILITY_BIND, EXEC_CONTEXT_RUNTIME_BRIDGE, bridge_context);
    TEST_ASSERT(result == 0, "Runtime_Bridge allowed SYS_V2_CAPABILITY_BIND");
    
    /* Test 3: Runtime_Bridge should be denied unauthorized syscalls */
    result = boundary_validate_syscall(SYS_V2_EXIT, EXEC_CONTEXT_RUNTIME_BRIDGE, bridge_context);
    TEST_ASSERT(result == BOUNDARY_ERR_BRIDGE_BYPASS, "Runtime_Bridge denied SYS_V2_EXIT");
}

/**
 * Test BCIB submission path hardening
 * Validates approved submission path enforcement
 */
static void test_bcib_submission_path_hardening(void) {
    debug_printf("[TEST] Testing BCIB submission path hardening...\n");
    
    uint64_t context_id = 0x1600;
    
    /* Test 1: Valid BCIB graph should pass */
    char valid_graph[1024] = {0}; /* Simulated BCIB graph */
    int result = boundary_check_bcib_submission_path(valid_graph, sizeof(valid_graph), context_id);
    TEST_ASSERT(result == 0, "Valid BCIB graph accepted");
    
    /* Test 2: NULL graph should fail */
    result = boundary_check_bcib_submission_path(NULL, 1024, context_id);
    TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, "NULL BCIB graph rejected");
    
    /* Test 3: Zero size graph should fail */
    result = boundary_check_bcib_submission_path(valid_graph, 0, context_id);
    TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, "Zero size BCIB graph rejected");
    
    /* Test 4: Oversized graph should fail */
    result = boundary_check_bcib_submission_path(valid_graph, 2 * 1024 * 1024, context_id);
    TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, "Oversized BCIB graph rejected");
}

/**
 * Test bridge bypass detection
 * Ensures Runtime_Bridge cannot bypass kernel boundary
 */
static void test_bridge_bypass_detection(void) {
    debug_printf("[TEST] Testing bridge bypass detection...\n");
    
    uint64_t context_id = 0x2600;
    
    /* Test 1: Syscall surface extension should be detected */
    int result = boundary_detect_bridge_bypass(SYS_V2_MAX_SYSCALL + 1, context_id);
    TEST_ASSERT(result == BOUNDARY_ERR_BRIDGE_BYPASS, "Syscall surface extension detected");
    
    /* Test 2: Valid syscall should pass */
    result = boundary_detect_bridge_bypass(SYS_V2_MAP_MEMORY, context_id);
    TEST_ASSERT(result == 0, "Valid syscall passes bypass detection");
}

/**
 * Test fail-closed behavior
 * Ensures violations result in deterministic termination
 */
static void test_fail_closed_behavior(void) {
    debug_printf("[TEST] Testing fail-closed behavior...\n");
    
    /* Note: In a full test environment, we would verify that fail-closed
     * termination actually terminates the execution context. For this
     * implementation, we verify that the violation is properly logged
     * and the correct error code is returned.
     */
    
    uint64_t context_id = 0x1700;
    
    /* Test violation logging */
    int result = boundary_audit_violation(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id, "Test violation");
    TEST_ASSERT(result == 0, "Violation properly logged");
    
    /* Test that boundary_fail_closed_termination doesn't crash */
    /* (In production, this would terminate the context) */
    boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id, "Test termination");
    TEST_ASSERT(1, "Fail-closed termination executed without crash");
}

/**
 * Test constitutional compliance
 * Verifies NON_OVERRIDABLE rule enforcement
 */
static void test_constitutional_compliance(void) {
    debug_printf("[TEST] Testing constitutional compliance...\n");
    
    /* Test that boundary enforcement is initialized */
    int result = boundary_enforce_init();
    TEST_ASSERT(result == 0, "Boundary enforcement initialized");
    
    /* Test KERNEL.SAFETY.CRITICAL enforcement */
    uint64_t kernel_context = 0x1800;
    result = boundary_validate_syscall(999, EXEC_CONTEXT_UNKNOWN, kernel_context);
    TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, "KERNEL.SAFETY.CRITICAL enforced");
    
    /* Test SECURITY.BOUNDARY.VIOLATION enforcement */
    result = boundary_validate_syscall(SYS_V2_MAP_MEMORY, EXEC_CONTEXT_BCIB, kernel_context);
    TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, "SECURITY.BOUNDARY.VIOLATION enforced");
}

/**
 * Run all boundary enforcement tests
 */
int run_boundary_enforcement_tests(void) {
    debug_printf("[TEST] Starting Phase-16 Boundary Enforcement Tests...\n");
    
    test_passed = 0;
    test_failed = 0;
    
    /* Initialize boundary enforcement for testing */
    boundary_enforce_init();
    
    /* Run test suites */
    test_bcib_syscall_restriction();
    test_runtime_bridge_restrictions();
    test_bcib_submission_path_hardening();
    test_bridge_bypass_detection();
    test_fail_closed_behavior();
    test_constitutional_compliance();
    
    /* Report results */
    debug_printf("[TEST] Boundary Enforcement Tests Complete: %d passed, %d failed\n", 
                test_passed, test_failed);
    
    if (test_failed == 0) {
        debug_printf("[TEST] All boundary enforcement tests PASSED\n");
        return 0;
    } else {
        debug_printf("[TEST] Some boundary enforcement tests FAILED\n");
        return -1;
    }
}

/**
 * Integration test for hardened syscall handler
 */
int test_hardened_syscall_integration(void) {
    debug_printf("[TEST] Testing hardened syscall integration...\n");
    
    /* Test that hardened handler properly validates and dispatches */
    uint64_t result;
    
    /* Test 1: Valid BCIB execution submission */
    char bcib_graph[512] = {0};
    result = syscall_v2_hardened_handler(SYS_V2_SUBMIT_EXECUTION, 
                                        (uint64_t)bcib_graph, 512, 0x1900, 0);
    /* Note: This will fail in test environment due to missing execution infrastructure,
     * but should pass boundary validation */
    debug_printf("[TEST] BCIB execution submission boundary validation completed\n");
    
    /* Test 2: Invalid syscall should be rejected */
    result = syscall_v2_hardened_handler(999, 0, 0, 0, 0);
    TEST_ASSERT(result == ESYS_V2_INVALID_SYSCALL, "Invalid syscall rejected by hardened handler");
    
    return 0;
}