#include "syscall_enforcement_matrix.h"
#include "../include/serial.h"

/*
 * Phase-16 Syscall Enforcement Matrix Implementation
 * 
 * Provides explicit syscall validation based on execution role.
 * Implements fail-closed enforcement with constitutional compliance.
 */

/* Debug output */
static void enforcement_debug(const char *msg) {
    serial_write("[ENFORCEMENT] ");
    serial_write(msg);
    serial_write("\n");
}

/**
 * Validate syscall against execution role using enforcement matrix
 * Returns 0 if allowed, error code if denied
 */
int syscall_enforcement_validate(proc_execution_role_t role, uint64_t syscall_num) {
    const syscall_enforcement_entry_t *entry = NULL;
    uint32_t syscall_mask;
    
    /* Find enforcement entry for role */
    for (int i = 0; i < SYSCALL_ENFORCEMENT_MATRIX_SIZE; i++) {
        if (SYSCALL_ENFORCEMENT_MATRIX[i].role == role) {
            entry = &SYSCALL_ENFORCEMENT_MATRIX[i];
            break;
        }
    }
    
    if (!entry) {
        /* Role not found in matrix - fail closed */
        enforcement_debug("Role not found in enforcement matrix - fail closed");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Validate syscall number range */
    if (syscall_num >= SYS_V2_NR) {
        enforcement_debug("Syscall number exceeds maximum - fail closed");
        return BOUNDARY_ERR_UNAUTHORIZED_SYSCALL;
    }
    
    /* Check syscall against allowed mask */
    syscall_mask = 1 << syscall_num;
    if (!(entry->allowed_syscalls_mask & syscall_mask)) {
        /* Syscall not allowed for this role */
        enforcement_debug("Syscall denied by enforcement matrix");
        return BOUNDARY_ERR_UNAUTHORIZED_SYSCALL;
    }
    
    /* CRITICAL ENFORCEMENT RULES - NO EXCEPTIONS */
    
    /* Rule 1: BCIB can ONLY use SUBMIT_EXECUTION */
    if (role == PROC_EXECUTION_ROLE_BCIB && syscall_num != SYS_V2_SUBMIT_EXECUTION) {
        enforcement_debug("CRITICAL: BCIB attempted non-submission syscall");
        return BOUNDARY_ERR_UNAUTHORIZED_SYSCALL;
    }
    
    /* Rule 2: Runtime_Bridge CANNOT use SUBMIT_EXECUTION */
    if (role == PROC_EXECUTION_ROLE_RUNTIME_BRIDGE && syscall_num == SYS_V2_SUBMIT_EXECUTION) {
        enforcement_debug("CRITICAL: Runtime_Bridge attempted execution submission");
        return BOUNDARY_ERR_BRIDGE_BYPASS;
    }
    
    /* Rule 3: Unknown role gets nothing - fail closed */
    if (role == PROC_EXECUTION_ROLE_UNKNOWN) {
        enforcement_debug("CRITICAL: Unknown role attempted syscall - fail closed");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Syscall allowed */
    return 0;
}

/**
 * Get human-readable role name for logging
 */
const char* syscall_enforcement_get_role_name(proc_execution_role_t role) {
    for (int i = 0; i < SYSCALL_ENFORCEMENT_MATRIX_SIZE; i++) {
        if (SYSCALL_ENFORCEMENT_MATRIX[i].role == role) {
            return SYSCALL_ENFORCEMENT_MATRIX[i].role_name;
        }
    }
    return "INVALID_ROLE";
}

/**
 * Get allowed syscall mask for role
 */
uint32_t syscall_enforcement_get_allowed_mask(proc_execution_role_t role) {
    for (int i = 0; i < SYSCALL_ENFORCEMENT_MATRIX_SIZE; i++) {
        if (SYSCALL_ENFORCEMENT_MATRIX[i].role == role) {
            return SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask;
        }
    }
    return 0; /* Fail closed - no syscalls allowed */
}

/**
 * Validate enforcement matrix integrity at runtime
 */
int syscall_enforcement_validate_matrix(void) {
    /* Verify all required roles are present */
    int bcib_found = 0, bridge_found = 0, user_found = 0, kernel_found = 0, unknown_found = 0;
    
    for (int i = 0; i < SYSCALL_ENFORCEMENT_MATRIX_SIZE; i++) {
        switch (SYSCALL_ENFORCEMENT_MATRIX[i].role) {
            case PROC_EXECUTION_ROLE_BCIB:
                bcib_found = 1;
                /* Verify BCIB has only SUBMIT_EXECUTION */
                if (SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask != (1 << SYS_V2_SUBMIT_EXECUTION)) {
                    enforcement_debug("CRITICAL: BCIB enforcement matrix corrupted");
                    return -1;
                }
                break;
            case PROC_EXECUTION_ROLE_RUNTIME_BRIDGE:
                bridge_found = 1;
                /* Verify Runtime_Bridge does NOT have SUBMIT_EXECUTION */
                if (SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask & (1 << SYS_V2_SUBMIT_EXECUTION)) {
                    enforcement_debug("CRITICAL: Runtime_Bridge has execution submission - security violation");
                    return -1;
                }
                if (!(SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask & (1 << SYS_V2_DEVICE_OPERATION)) ||
                    !(SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask & (1 << SYS_V2_EXTERNAL_CALL)) ||
                    !(SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask & (1 << SYS_V2_ABDF_OPERATION))) {
                    enforcement_debug("CRITICAL: Runtime_Bridge missing required bridge syscalls");
                    return -1;
                }
                break;
            case PROC_EXECUTION_ROLE_USER:
                user_found = 1;
                break;
            case PROC_EXECUTION_ROLE_KERNEL:
                kernel_found = 1;
                break;
            case PROC_EXECUTION_ROLE_UNKNOWN:
                unknown_found = 1;
                /* Verify unknown role has no syscalls */
                if (SYSCALL_ENFORCEMENT_MATRIX[i].allowed_syscalls_mask != 0) {
                    enforcement_debug("CRITICAL: Unknown role has syscall permissions - fail-closed violation");
                    return -1;
                }
                break;
        }
    }
    
    /* Verify all roles are present */
    if (!bcib_found || !bridge_found || !user_found || !kernel_found || !unknown_found) {
        enforcement_debug("CRITICAL: Enforcement matrix missing required roles");
        return -1;
    }
    
    enforcement_debug("Enforcement matrix validation passed");
    return 0;
}
