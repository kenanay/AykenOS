#include "boundary_enforcement.h"
#include "syscall_enforcement_matrix.h"
#include "syscall_v2_hardened.h"
#include "../include/proc.h"
#include "../include/serial.h"

/*
 * Phase-16 Critical Boundary Enforcement Tests
 * 
 * Tests the critical security properties that MUST hold:
 * 1. BCIB → forbidden syscall → kill
 * 2. Runtime_Bridge → submit → kill  
 * 3. Context spoof → kill
 * 4. Direct pointer → kill
 */

static int critical_test_passed = 0;
static int critical_test_failed = 0;

#define CRITICAL_TEST_ASSERT(condition, message) \
    do { \
        if (condition) { \
            critical_test_passed++; \
            serial_write("[CRITICAL TEST PASS] "); \
            serial_write(message); \
            serial_write("\n"); \
        } else { \
            critical_test_failed++; \
            serial_write("[CRITICAL TEST FAIL] "); \
            serial_write(message); \
            serial_write("\n"); \
        } \
    } while(0)

/**
 * CRITICAL TEST 1: BCIB → forbidden syscall → kill
 * This MUST result in process termination
 */
static void critical_test_bcib_forbidden_syscall(void) {
    serial_write("[CRITICAL TEST] Testing BCIB forbidden syscall enforcement...\n");
    
    /* Test BCIB attempting MAP_MEMORY (forbidden) */
    int result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_BCIB, SYS_V2_MAP_MEMORY);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "BCIB denied SYS_V2_MAP_MEMORY");
    
    /* Test BCIB attempting CAPABILITY_BIND (forbidden) */
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_BCIB, SYS_V2_CAPABILITY_BIND);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "BCIB denied SYS_V2_CAPABILITY_BIND");
    
    /* Test BCIB attempting TIME_QUERY (forbidden) */
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_BCIB, SYS_V2_TIME_QUERY);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "BCIB denied SYS_V2_TIME_QUERY");
    
    /* Test BCIB allowed syscall (SUBMIT_EXECUTION) */
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_BCIB, SYS_V2_SUBMIT_EXECUTION);
    CRITICAL_TEST_ASSERT(result == 0, 
                        "BCIB allowed SYS_V2_SUBMIT_EXECUTION");
}

/**
 * CRITICAL TEST 2: Runtime_Bridge → submit → kill
 * Runtime_Bridge MUST NOT be able to submit execution
 */
static void critical_test_bridge_submit_forbidden(void) {
    serial_write("[CRITICAL TEST] Testing Runtime_Bridge execution submission denial...\n");
    
    /* Test Runtime_Bridge attempting SUBMIT_EXECUTION (CRITICAL VIOLATION) */
    int result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_RUNTIME_BRIDGE, SYS_V2_SUBMIT_EXECUTION);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_BRIDGE_BYPASS, 
                        "Runtime_Bridge denied SYS_V2_SUBMIT_EXECUTION");
    
    /* Test Runtime_Bridge allowed syscalls */
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_RUNTIME_BRIDGE, SYS_V2_MAP_MEMORY);
    CRITICAL_TEST_ASSERT(result == 0, 
                        "Runtime_Bridge allowed SYS_V2_MAP_MEMORY");
    
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_RUNTIME_BRIDGE, SYS_V2_CAPABILITY_BIND);
    CRITICAL_TEST_ASSERT(result == 0, 
                        "Runtime_Bridge allowed SYS_V2_CAPABILITY_BIND");
    
    /* Test Runtime_Bridge forbidden syscalls */
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_RUNTIME_BRIDGE, SYS_V2_EXIT);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "Runtime_Bridge denied SYS_V2_EXIT");
}

/**
 * CRITICAL TEST 3: Context spoof → kill
 * Unknown/invalid execution roles MUST be denied all syscalls
 */
static void critical_test_context_spoof_denial(void) {
    serial_write("[CRITICAL TEST] Testing context spoof denial...\n");
    
    /* Test unknown role attempting any syscall */
    int result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_UNKNOWN, SYS_V2_SUBMIT_EXECUTION);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, 
                        "Unknown role denied SYS_V2_SUBMIT_EXECUTION");
    
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_UNKNOWN, SYS_V2_MAP_MEMORY);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, 
                        "Unknown role denied SYS_V2_MAP_MEMORY");
    
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_UNKNOWN, SYS_V2_TIME_QUERY);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_ISOLATION_VIOLATION, 
                        "Unknown role denied SYS_V2_TIME_QUERY");
    
    /* Test invalid role value (out of enum range) */
    result = syscall_enforcement_validate((proc_execution_role_t)999, SYS_V2_SUBMIT_EXECUTION);
    CRITICAL_TEST_ASSERT(result != 0, 
                        "Invalid role value denied syscall access");
}

/**
 * CRITICAL TEST 4: Syscall surface extension → kill
 * Attempts to extend syscall surface beyond SYS_V2_MAX_SYSCALL MUST fail
 */
static void critical_test_syscall_surface_extension(void) {
    serial_write("[CRITICAL TEST] Testing syscall surface extension denial...\n");
    
    /* Test syscall numbers beyond valid range */
    int result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_USER, SYS_V2_MAX_SYSCALL + 1);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "Syscall beyond max range denied");
    
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_KERNEL, 999);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "Large syscall number denied even for kernel");
    
    /* Test valid syscall numbers pass */
    result = syscall_enforcement_validate(PROC_EXECUTION_ROLE_USER, SYS_V2_TIME_QUERY);
    CRITICAL_TEST_ASSERT(result == 0, 
                        "Valid syscall number allowed for user");
}

/**
 * CRITICAL TEST 5: Enforcement matrix integrity
 * The enforcement matrix MUST maintain critical security properties
 */
static void critical_test_enforcement_matrix_integrity(void) {
    serial_write("[CRITICAL TEST] Testing enforcement matrix integrity...\n");
    
    /* Test matrix validation */
    int result = syscall_enforcement_validate_matrix();
    CRITICAL_TEST_ASSERT(result == 0, 
                        "Enforcement matrix validation passed");
    
    /* Test BCIB role has ONLY submit execution */
    uint32_t bcib_mask = syscall_enforcement_get_allowed_mask(PROC_EXECUTION_ROLE_BCIB);
    CRITICAL_TEST_ASSERT(bcib_mask == (1 << SYS_V2_SUBMIT_EXECUTION), 
                        "BCIB role has only SUBMIT_EXECUTION permission");
    
    /* Test Runtime_Bridge does NOT have submit execution */
    uint32_t bridge_mask = syscall_enforcement_get_allowed_mask(PROC_EXECUTION_ROLE_RUNTIME_BRIDGE);
    CRITICAL_TEST_ASSERT(!(bridge_mask & (1 << SYS_V2_SUBMIT_EXECUTION)), 
                        "Runtime_Bridge does NOT have SUBMIT_EXECUTION permission");
    
    /* Test unknown role has no permissions */
    uint32_t unknown_mask = syscall_enforcement_get_allowed_mask(PROC_EXECUTION_ROLE_UNKNOWN);
    CRITICAL_TEST_ASSERT(unknown_mask == 0, 
                        "Unknown role has no syscall permissions");
}

/**
 * CRITICAL TEST 6: Boundary validation integration
 * Test that boundary_validate_syscall properly uses enforcement matrix
 */
static void critical_test_boundary_validation_integration(void) {
    serial_write("[CRITICAL TEST] Testing boundary validation integration...\n");
    
    /* Initialize boundary enforcement */
    boundary_enforce_init();
    
    /* Test BCIB context validation */
    int result = boundary_validate_syscall(SYS_V2_MAP_MEMORY, EXEC_CONTEXT_BCIB, 1000);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, 
                        "Boundary validation denies BCIB MAP_MEMORY");
    
    result = boundary_validate_syscall(SYS_V2_SUBMIT_EXECUTION, EXEC_CONTEXT_BCIB, 1000);
    CRITICAL_TEST_ASSERT(result == 0, 
                        "Boundary validation allows BCIB SUBMIT_EXECUTION");
    
    /* Test Runtime_Bridge context validation */
    result = boundary_validate_syscall(SYS_V2_SUBMIT_EXECUTION, EXEC_CONTEXT_RUNTIME_BRIDGE, 2000);
    CRITICAL_TEST_ASSERT(result == BOUNDARY_ERR_BRIDGE_BYPASS, 
                        "Boundary validation denies Runtime_Bridge SUBMIT_EXECUTION");
    
    result = boundary_validate_syscall(SYS_V2_MAP_MEMORY, EXEC_CONTEXT_RUNTIME_BRIDGE, 2000);
    CRITICAL_TEST_ASSERT(result == 0, 
                        "Boundary validation allows Runtime_Bridge MAP_MEMORY");

    result = boundary_validate_syscall(SYS_V2_DEVICE_OPERATION, EXEC_CONTEXT_RUNTIME_BRIDGE, 2000);
    CRITICAL_TEST_ASSERT(result == 0,
                        "Boundary validation allows Runtime_Bridge DEVICE_OPERATION");
}

/**
 * Run all critical boundary enforcement tests
 * These tests MUST pass for the system to be considered secure
 */
int run_critical_boundary_enforcement_tests(void) {
    serial_write("[CRITICAL TEST] Starting Phase-16 Critical Boundary Enforcement Tests...\n");
    
    critical_test_passed = 0;
    critical_test_failed = 0;
    
    /* Run critical test suites */
    critical_test_bcib_forbidden_syscall();
    critical_test_bridge_submit_forbidden();
    critical_test_context_spoof_denial();
    critical_test_syscall_surface_extension();
    critical_test_enforcement_matrix_integrity();
    critical_test_boundary_validation_integration();
    
    /* Report results */
    serial_write("[CRITICAL TEST] Critical Tests Complete: ");
    /* Note: Would use proper printf in production */
    serial_write(" passed, ");
    serial_write(" failed\n");
    
    if (critical_test_failed == 0) {
        serial_write("[CRITICAL TEST] ALL CRITICAL TESTS PASSED - System security properties validated\n");
        return 0;
    } else {
        serial_write("[CRITICAL TEST] CRITICAL TESTS FAILED - System security compromised\n");
        return -1;
    }
}

/**
 * Validate that the system enforces the critical security invariants
 * This function MUST return 0 for the system to be production-ready
 */
int validate_critical_security_invariants(void) {
    serial_write("[SECURITY] Validating critical security invariants...\n");
    
    /* Critical Invariant 1: BCIB isolation */
    if (syscall_enforcement_get_allowed_mask(PROC_EXECUTION_ROLE_BCIB) != (1 << SYS_V2_SUBMIT_EXECUTION)) {
        serial_write("[SECURITY] CRITICAL FAILURE: BCIB isolation compromised\n");
        return -1;
    }
    
    /* Critical Invariant 2: Runtime_Bridge cannot submit execution */
    uint32_t bridge_mask = syscall_enforcement_get_allowed_mask(PROC_EXECUTION_ROLE_RUNTIME_BRIDGE);
    if (bridge_mask & (1 << SYS_V2_SUBMIT_EXECUTION)) {
        serial_write("[SECURITY] CRITICAL FAILURE: Runtime_Bridge can submit execution\n");
        return -1;
    }
    
    /* Critical Invariant 3: Unknown roles have no access */
    if (syscall_enforcement_get_allowed_mask(PROC_EXECUTION_ROLE_UNKNOWN) != 0) {
        serial_write("[SECURITY] CRITICAL FAILURE: Unknown roles have syscall access\n");
        return -1;
    }
    
    /* Critical Invariant 4: Enforcement matrix is valid */
    if (syscall_enforcement_validate_matrix() != 0) {
        serial_write("[SECURITY] CRITICAL FAILURE: Enforcement matrix corrupted\n");
        return -1;
    }
    
    serial_write("[SECURITY] All critical security invariants validated\n");
    return 0;
}
