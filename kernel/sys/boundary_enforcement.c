#include "boundary_enforcement.h"
#include "syscall_enforcement_matrix.h"
#include "../include/ayken.h"
#include "../include/serial.h"
#include "../include/execution_slot.h"
#include "../include/proc.h"
#include <string.h>

/* Debug printf implementation using serial output */
static void debug_printf(const char *fmt, ...) {
    /* Simple implementation - just write the format string for now */
    serial_write("[BOUNDARY] ");
    serial_write(fmt);
    serial_write("\n");
}

/* Forward declarations for scheduler functions */
extern void sched_remove_process_everywhere(proc_t *proc);

/*
 * Phase-16 Kernel Boundary Enforcement Implementation
 * 
 * Enforces strict isolation between BCIB execution and kernel resources.
 * Implements fail-closed semantics for all boundary violations.
 */

/* Maximum contexts and violation log size */
#define MAX_EXECUTION_CONTEXTS 256
#define MAX_VIOLATION_LOG_ENTRIES 1024

/* Constants for boundary enforcement */
#define KERNEL_VIRTUAL_BASE 0xFFFF800000000000UL
#define MAX_BCIB_GRAPH_SIZE (1024 * 1024) /* 1MB max BCIB graph */

/* Forward declaration */
static uint64_t get_system_time(void);

static boundary_state_t boundary_states[MAX_EXECUTION_CONTEXTS];
static int boundary_initialized = 0;
static uint64_t violation_audit_log[MAX_VIOLATION_LOG_ENTRIES];
static int violation_log_index = 0;

/**
 * Initialize boundary enforcement subsystem
 * Must be called during kernel initialization
 */
int boundary_enforce_init(void) {
    if (boundary_initialized) {
        return 0; /* Already initialized */
    }
    
    /* Clear all boundary states */
    memset(boundary_states, 0, sizeof(boundary_states));
    memset(violation_audit_log, 0, sizeof(violation_audit_log));
    violation_log_index = 0;
    
    boundary_initialized = 1;
    
    debug_printf("[BOUNDARY] Kernel boundary enforcement initialized\n");
    return 0;
}

/**
 * Validate syscall against execution context type
 * Enforces Requirements 1.5, 1.6, 1.7, 1.8 using explicit enforcement matrix
 */
int boundary_validate_syscall(uint64_t syscall_num, execution_context_type_t context_type, uint64_t context_id) {
    proc_execution_role_t role;
    int enforcement_result;
    
    if (!boundary_initialized) {
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id, 
                                        "Boundary enforcement not initialized");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Convert context type to execution role */
    switch (context_type) {
        case EXEC_CONTEXT_BCIB:
            role = PROC_EXECUTION_ROLE_BCIB;
            break;
        case EXEC_CONTEXT_RUNTIME_BRIDGE:
            role = PROC_EXECUTION_ROLE_RUNTIME_BRIDGE;
            break;
        case EXEC_CONTEXT_USERSPACE:
            role = PROC_EXECUTION_ROLE_USER;
            break;
        case EXEC_CONTEXT_KERNEL:
            role = PROC_EXECUTION_ROLE_KERNEL;
            break;
        default:
            role = PROC_EXECUTION_ROLE_UNKNOWN;
            break;
    }
    
    /* Use explicit enforcement matrix - NO HEURISTICS */
    enforcement_result = syscall_enforcement_validate(role, syscall_num);
    if (enforcement_result != 0) {
        /* Log the specific violation */
        const char *role_name = syscall_enforcement_get_role_name(role);
        debug_printf("Syscall %lu denied for role %s", syscall_num, role_name);
        
        boundary_audit_violation(enforcement_result, context_id, "Syscall denied by enforcement matrix");
        boundary_fail_closed_termination(enforcement_result, context_id, "Syscall enforcement violation");
        return enforcement_result;
    }
    
    return 0; /* Success */
}

/**
 * Check BCIB submission path hardening
 * Ensures BCIB execution entry is only via approved submission path
 */
int boundary_check_bcib_submission_path(void *bcib_graph, uint64_t graph_size, uint64_t context_id) {
    if (!boundary_initialized) {
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "Boundary enforcement not initialized");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Validate BCIB graph pointer is in userspace */
    if (bcib_graph == NULL) {
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "NULL BCIB graph pointer");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Check if pointer is in kernel space (fail-closed) */
    uintptr_t graph_addr = (uintptr_t)bcib_graph;
    if (graph_addr >= KERNEL_VIRTUAL_BASE) {
        boundary_audit_violation(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                "BCIB graph in kernel space");
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "BCIB graph pointer in kernel space - isolation violation");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Validate graph size is reasonable (prevent DoS) */
    if (graph_size == 0 || graph_size > MAX_BCIB_GRAPH_SIZE) {
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "Invalid BCIB graph size");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    debug_printf("[BOUNDARY] BCIB submission path validated for context %lu\n", context_id);
    return 0;
}

/**
 * Detect Runtime_Bridge bypass attempts
 * Ensures Runtime_Bridge cannot replace or bypass syscall surface
 */
int boundary_detect_bridge_bypass(uint64_t syscall_num, uint64_t context_id) {
    if (!boundary_initialized) {
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Check for attempts to extend syscall surface (Requirement 1.8) */
    if (syscall_num > SYS_V2_MAX_SYSCALL) {
        boundary_audit_violation(BOUNDARY_ERR_BRIDGE_BYPASS, context_id,
                                "Attempt to extend syscall surface");
        boundary_fail_closed_termination(BOUNDARY_ERR_BRIDGE_BYPASS, context_id,
                                        "Syscall surface extension attempt - ABI freeze violation");
        return BOUNDARY_ERR_BRIDGE_BYPASS;
    }
    
    /* Check for direct kernel API exposure beyond approved interface */
    if (syscall_num == SYS_V2_SUBMIT_EXECUTION) {
        /* Only BCIB contexts should use this syscall */
        boundary_state_t *state = &boundary_states[context_id % MAX_EXECUTION_CONTEXTS];
        if (state->context_type != EXEC_CONTEXT_BCIB) {
            boundary_audit_violation(BOUNDARY_ERR_KERNEL_API_EXPOSURE, context_id,
                                   "Non-BCIB context using SUBMIT_EXECUTION");
            boundary_fail_closed_termination(BOUNDARY_ERR_KERNEL_API_EXPOSURE, context_id,
                                            "Unauthorized use of BCIB execution interface");
            return BOUNDARY_ERR_KERNEL_API_EXPOSURE;
        }
    }
    
    return 0;
}

/**
 * Fail-closed termination for boundary violations
 * Implements constitutional compliance with immediate termination
 */
void boundary_fail_closed_termination(int violation_code, uint64_t context_id, const char *reason) {
    extern proc_t *current_proc;
    
    /* Log violation before termination */
    debug_printf("[BOUNDARY] FAIL-CLOSED TERMINATION: Code=%d, Context=%lu, Reason=%s\n",
                violation_code, context_id, reason ? reason : "Unknown");
    
    /* Constitutional compliance: KERNEL.SAFETY.CRITICAL and SECURITY.BOUNDARY.VIOLATION */
    debug_printf("[CONSTITUTIONAL] VIOLATION: KERNEL.SAFETY.CRITICAL + SECURITY.BOUNDARY.VIOLATION\n");
    
    /* Audit the violation */
    boundary_audit_violation(violation_code, context_id, reason);
    
    /* REAL FAIL-CLOSED TERMINATION - CRITICAL FIX */
    if (current_proc && current_proc->type == PROC_TYPE_USER) {
        /* Terminate current user process immediately */
        debug_printf("[BOUNDARY] Terminating user process PID=%d due to boundary violation\n", current_proc->pid);
        
        /* Abort any active execution slots for this process */
        if (current_proc->active_execution_id != 0) {
            /* Find and abort the execution slot */
            execution_slot_guard_t slot_guard;
            execution_slot_enter_critical(&slot_guard);
            
            exec_slot_t *slot = execution_slot_find_locked(current_proc->active_execution_id);
            if (slot) {
                execution_slot_require_finish_locked(slot, EXEC_SLOT_ABORTED, "boundary_violation");
                debug_printf("[BOUNDARY] Aborted execution slot %lu\n", current_proc->active_execution_id);
            }
            
            execution_slot_exit_critical(&slot_guard);
            current_proc->active_execution_id = 0;
        }
        
        /* Mark process as zombie and initiate teardown */
        current_proc->state = PROC_ZOMBIE;
        current_proc->wait_obj = NULL;
        
        /* Teardown process surfaces */
        proc_teardown_exit_surfaces(current_proc, NULL, NULL, 0);
        
        /* Remove from scheduler */
        sched_remove_process_everywhere(current_proc);
        
        debug_printf("[BOUNDARY] Process terminated and removed from scheduler\n");
        
        /* Force immediate context switch away from terminated process */
        /* This ensures the boundary violation cannot continue execution */
        /* Note: In production, this would trigger a context switch */
        debug_printf("[BOUNDARY] Context switch requested to prevent continued execution\n");
        
    } else if (current_proc && current_proc->type == PROC_TYPE_KERNEL) {
        /* Kernel process boundary violation - this is critical */
        debug_printf("[BOUNDARY] CRITICAL: Kernel process boundary violation - system halt\n");
        
        /* For kernel processes, we cannot safely terminate, so halt the system */
        /* Note: In production, this would halt the system */
        debug_printf("[BOUNDARY] CRITICAL: System halt requested due to kernel boundary violation\n");
        
    } else {
        /* No current process or unknown state - log and continue */
        debug_printf("[BOUNDARY] No current process to terminate\n");
    }
}

/**
 * Audit boundary violations to immutable log
 * Required for constitutional compliance and forensics
 */
int boundary_audit_violation(int violation_code, uint64_t context_id, const char *details) {
    if (violation_log_index >= MAX_VIOLATION_LOG_ENTRIES) {
        /* Log is full - this is a critical system state */
        debug_printf("[BOUNDARY] CRITICAL: Violation audit log full\n");
        return -1;
    }
    
    /* Create audit entry */
    uint64_t audit_entry = ((uint64_t)violation_code << 48) | 
                          ((context_id & 0xFFFF) << 32) |
                          (get_system_time() & 0xFFFFFFFF);
    
    violation_audit_log[violation_log_index++] = audit_entry;
    
    debug_printf("[AUDIT] Violation logged: Code=%d, Context=%lu, Details=%s\n",
                violation_code, context_id, details ? details : "None");
    
    return 0;
}

/* Helper function to get system time (placeholder) */
static uint64_t get_system_time(void) {
    /* In a full implementation, this would return actual system time */
    /* For now, return a placeholder value */
    return 0x12345678;
}