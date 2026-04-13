#include "syscall_v2.h"
#include "boundary_enforcement.h"
#include "syscall_enforcement_matrix.h"
#include "../include/ayken.h"
#include "../include/serial.h"
#include "../include/proc.h"

/* Debugcon helper for marker emission */
static void debugcon_write(const char *s) {
    if (!s) return;
    while (*s) {
        __asm__ volatile("outb %0, %1" : : "a"((uint8_t)*s), "Nd"((uint16_t)0xE9));
        s++;
    }
}

/* Debug printf implementation using serial output */
static void debug_printf(const char *fmt, ...) {
    /* Simple implementation - just write the format string for now */
    serial_write("[HARDENED] ");
    serial_write(fmt);
    serial_write("\n");
}

/*
 * Phase-16 Hardened Syscall Handler
 * 
 * Integrates boundary enforcement with existing syscall_v2 infrastructure.
 * Enforces strict kernel boundary controls and fail-closed semantics.
 */

/* Forward declarations from syscall_v2.c */
extern uint64_t sys_v2_map_memory(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags);
extern uint64_t sys_v2_unmap_memory(uint64_t virt_addr, uint64_t size);
extern uint64_t sys_v2_switch_context(uint64_t old_ctx_id, uint64_t new_ctx_id);
extern uint64_t sys_v2_submit_execution(void *bcib_graph, uint64_t graph_size, uint64_t context_id);
extern uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms);
extern uint64_t sys_v2_interrupt_return(uint64_t interrupt_id, uint64_t result_code);
extern uint64_t sys_v2_time_query(uint64_t query_type, uint64_t *result_buffer);
extern uint64_t sys_v2_capability_bind(uint64_t execution_ctx_id, capability_token_t *token);
extern uint64_t sys_v2_capability_revoke(uint64_t token_id);
extern uint64_t sys_v2_exit(uint64_t exit_code);
extern uint64_t sys_v2_debug_putchar(uint64_t character);
extern uint64_t sys_v2_debug_write_str(const char *str, uint64_t length);
extern uint64_t sys_v2_complete_execution(uint64_t execution_id, uint64_t completion_code);
extern uint64_t sys_v2_device_operation(uint64_t device_id, uint64_t operation, uint64_t *buffer, uint64_t buffer_size);
extern uint64_t sys_v2_external_call(uint64_t call_id, uint64_t *args, uint64_t arg_count);
extern uint64_t sys_v2_abdf_operation(uint64_t operation_type, uint64_t handle_id, uint64_t *data, uint64_t data_size);

/* Context detection - simplified for Phase-16 implementation */
static execution_context_type_t detect_execution_context(uint64_t context_id) {
    /* In a full implementation, this would query the execution context manager */
    /* For Phase-16, we use heuristics based on context_id ranges */
    
    if (context_id >= 0x1000 && context_id < 0x2000) {
        return EXEC_CONTEXT_BCIB;
    } else if (context_id >= 0x2000 && context_id < 0x3000) {
        return EXEC_CONTEXT_RUNTIME_BRIDGE;
    } else if (context_id >= 0x3000) {
        return EXEC_CONTEXT_USERSPACE;
    }
    
    return EXEC_CONTEXT_UNKNOWN;
}

/**
 * Hardened syscall handler with boundary enforcement
 * Replaces syscall_v2_handler with boundary checks
 */
uint64_t syscall_v2_hardened_handler(uint64_t syscall_num, uint64_t arg1,
                                     uint64_t arg2, uint64_t arg3, uint64_t arg4) {
    execution_context_type_t context_type;
    int boundary_result;
    
    /* Get current execution context - EXPLICIT ROLE MODEL */
    extern proc_t *current_proc;
    uint64_t context_id = 0;
    uint64_t process_id = 0;
    
    if (current_proc) {
        context_id = (uint64_t)current_proc->pid;
        process_id = (uint64_t)current_proc->pid;
        
        /* CRITICAL FIX: Use explicit execution role instead of heuristics */
        switch (current_proc->execution_role) {
            case PROC_EXECUTION_ROLE_BCIB:
                context_type = EXEC_CONTEXT_BCIB;
                /* Debug: Confirm we're in BCIB context */
                serial_write("[HARDENED] BCIB context detected, syscall_num=");
                {
                    char buf[32];
                    int i = 0, n = (int)syscall_num;
                    if (n == 0) buf[i++] = '0';
                    else {
                        char tmp[32];
                        int j = 0;
                        while (n > 0) { tmp[j++] = '0' + (n % 10); n /= 10; }
                        while (j > 0) buf[i++] = tmp[--j];
                    }
                    buf[i] = '\0';
                    serial_write(buf);
                }
                serial_write("\n");
                break;
            case PROC_EXECUTION_ROLE_RUNTIME_BRIDGE:
                context_type = EXEC_CONTEXT_RUNTIME_BRIDGE;
                break;
            case PROC_EXECUTION_ROLE_USER:
                context_type = EXEC_CONTEXT_USERSPACE;
                break;
            case PROC_EXECUTION_ROLE_KERNEL:
                context_type = EXEC_CONTEXT_KERNEL;
                break;
            default:
                /* Unknown role - fail closed */
                boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                                "Unknown execution role - fail closed");
                return BOUNDARY_ERR_ISOLATION_VIOLATION;
        }
    } else {
        /* No current process - kernel context */
        context_type = EXEC_CONTEXT_KERNEL;
    }
    
    /* Initialize boundary enforcement if not already done */
    static int boundary_init_done = 0;
    if (!boundary_init_done) {
        boundary_enforce_init();
        
        /* CRITICAL: Validate enforcement matrix integrity */
        if (syscall_enforcement_validate_matrix() != 0) {
            boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                            "Enforcement matrix validation failed - system compromised");
            return BOUNDARY_ERR_ISOLATION_VIOLATION;
        }
        
        boundary_init_done = 1;
    }
    
    /* Phase-16 Boundary Enforcement: Validate syscall against context */
    boundary_result = boundary_validate_syscall(syscall_num, context_type, context_id);
    if (boundary_result != 0) {
        /* Boundary violation detected - fail-closed termination already triggered */
        return (uint64_t)boundary_result;
    }
    
    /* Additional boundary checks for specific syscalls */
    boundary_result = boundary_detect_bridge_bypass(syscall_num, context_id);
    if (boundary_result != 0) {
        return (uint64_t)boundary_result;
    }
    
    /* Special handling for SYS_V2_SUBMIT_EXECUTION - BCIB submission path hardening */
    if (syscall_num == SYS_V2_SUBMIT_EXECUTION) {
        void *bcib_graph = (void *)arg1;
        uint64_t graph_size = arg2;
        uint64_t exec_context_id = arg3;
        
        boundary_result = boundary_check_bcib_submission_path(bcib_graph, graph_size, exec_context_id);
        if (boundary_result != 0) {
            return (uint64_t)boundary_result;
        }
        
        /* CRITICAL: Only BCIB contexts can submit execution - no exceptions */
        if (context_type != EXEC_CONTEXT_BCIB) {
            boundary_fail_closed_termination(BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, context_id,
                                            "Non-BCIB context attempting execution submission");
            return BOUNDARY_ERR_UNAUTHORIZED_SYSCALL;
        }
    }
    
    /* Validate syscall number range */
    if (syscall_num >= SYS_V2_NR) {
        boundary_fail_closed_termination(BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, context_id,
                                        "Syscall number exceeds maximum allowed");
        return ESYS_V2_INVALID_SYSCALL;
    }
    
    /* Dispatch to original syscall handlers after boundary validation */
    switch (syscall_num) {
        case SYS_V2_MAP_MEMORY:
            return sys_v2_map_memory(arg1, arg2, arg3);
            
        case SYS_V2_UNMAP_MEMORY:
            return sys_v2_unmap_memory(arg1, arg2);
            
        case SYS_V2_SWITCH_CONTEXT:
            return sys_v2_switch_context(arg1, arg2);
            
        case SYS_V2_SUBMIT_EXECUTION:
            return sys_v2_submit_execution((void *)arg1, arg2, arg3);
            
        case SYS_V2_WAIT_RESULT:
            return sys_v2_wait_result(arg1, arg2);
            
        case SYS_V2_INTERRUPT_RETURN:
            return sys_v2_interrupt_return(arg1, arg2);
            
        case SYS_V2_TIME_QUERY:
            return sys_v2_time_query(arg1, (uint64_t *)arg2);
            
        case SYS_V2_CAPABILITY_BIND:
            return sys_v2_capability_bind(arg1, (capability_token_t *)arg2);
            
        case SYS_V2_CAPABILITY_REVOKE:
            return sys_v2_capability_revoke(arg1);
            
        case SYS_V2_EXIT:
            return sys_v2_exit(arg1);
            
        case SYS_V2_DEBUG_PUTCHAR:
            return sys_v2_debug_putchar(arg1);
            
        case SYS_V2_DEBUG_WRITE_STR:
            return sys_v2_debug_write_str((const char *)arg1, arg2);
            
        case SYS_V2_COMPLETE_EXECUTION:
            return sys_v2_complete_execution(arg1, arg2);
            
        case SYS_V2_DEVICE_OPERATION:
            return sys_v2_device_operation(arg1, arg2, (uint64_t *)arg3, arg4);
            
        case SYS_V2_EXTERNAL_CALL:
            return sys_v2_external_call(arg1, (uint64_t *)arg2, arg3);
            
        case SYS_V2_ABDF_OPERATION:
            return sys_v2_abdf_operation(arg1, arg2, (uint64_t *)arg3, arg4);
            
        default:
            boundary_fail_closed_termination(BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, context_id,
                                            "Unknown syscall number");
            return ESYS_V2_NOT_IMPLEMENTED;
    }
}

/**
 * BCIB-specific syscall validation
 * Ensures BCIB contexts can only use SYS_V2_SUBMIT_EXECUTION
 */
int validate_bcib_syscall_restriction(uint64_t syscall_num, uint64_t context_id) {
    if (syscall_num != SYS_V2_SUBMIT_EXECUTION) {
        boundary_audit_violation(BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, context_id,
                                "BCIB attempted non-submission syscall");
        boundary_fail_closed_termination(BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, context_id,
                                        "BCIB syscall restriction violation");
        return BOUNDARY_ERR_UNAUTHORIZED_SYSCALL;
    }
    return 0;
}

/**
 * Runtime_Bridge syscall validation
 * Ensures Runtime_Bridge cannot bypass or replace syscall surface
 */
int validate_bridge_syscall_restriction(uint64_t syscall_num, uint64_t context_id) {
    /* Runtime_Bridge has limited syscall access - cannot use execution submission */
    if (syscall_num == SYS_V2_SUBMIT_EXECUTION) {
        boundary_audit_violation(BOUNDARY_ERR_BRIDGE_BYPASS, context_id,
                                "Runtime_Bridge attempted execution submission");
        boundary_fail_closed_termination(BOUNDARY_ERR_BRIDGE_BYPASS, context_id,
                                        "Runtime_Bridge cannot submit execution");
        return BOUNDARY_ERR_BRIDGE_BYPASS;
    }
    
    /* Check against allowed syscalls mask */
    uint32_t syscall_mask = 1 << syscall_num;
    if (!(syscall_mask & BRIDGE_ALLOWED_SYSCALLS_MASK)) {
        boundary_audit_violation(BOUNDARY_ERR_BRIDGE_BYPASS, context_id,
                                "Runtime_Bridge unauthorized syscall");
        boundary_fail_closed_termination(BOUNDARY_ERR_BRIDGE_BYPASS, context_id,
                                        "Runtime_Bridge syscall not in allowlist");
        return BOUNDARY_ERR_BRIDGE_BYPASS;
    }
    
    return 0;
}

/**
 * Kernel API exposure prevention
 * Ensures no direct kernel API exposure beyond approved submission interface
 */
int prevent_kernel_api_exposure(uint64_t syscall_num, execution_context_type_t context_type) {
    /* Prevent extension of syscall surface (ABI freeze constraint) */
    if (syscall_num > SYS_V2_MAX_SYSCALL) {
        debug_printf("[BOUNDARY] Attempt to extend syscall surface detected\n");
        return BOUNDARY_ERR_KERNEL_API_EXPOSURE;
    }
    
    /* Ensure execution submission is only available to BCIB contexts */
    if (syscall_num == SYS_V2_SUBMIT_EXECUTION && context_type != EXEC_CONTEXT_BCIB) {
        debug_printf("[BOUNDARY] Non-BCIB context attempting execution submission\n");
        return BOUNDARY_ERR_KERNEL_API_EXPOSURE;
    }
    
    return 0;
}