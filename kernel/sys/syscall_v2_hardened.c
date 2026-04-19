#include "syscall_v2.h"
#include "boundary_enforcement.h"
#include "syscall_enforcement_matrix.h"
#include "syscall_enforcement_matrix_fast.h"  /* PATCH B: Fast-path bitmask optimization */
#include "../include/ayken.h"
#include "../include/serial.h"
#include "../include/proc.h"

/* Debugcon helper for marker emission with timestamp */
static inline uint64_t read_tsc(void) {
    uint32_t lo, hi;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

static void debugcon_write_with_timestamp(const char *marker) {
    uint64_t rflags, ts;
    if (!marker) return;
    
    ts = read_tsc();
    
    __asm__ volatile("pushfq; pop %0; cli" : "=r"(rflags) : : "memory");
    
    // Emit marker
    while (*marker) {
        __asm__ volatile("outb %0, %1" : : "a"((uint8_t)*marker), "Nd"((uint16_t)0xE9));
        marker++;
    }
    
    // Emit space
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)' '), "Nd"((uint16_t)0xE9));
    
    // Emit timestamp in hex
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'0'), "Nd"((uint16_t)0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'x'), "Nd"((uint16_t)0xE9));
    for (int i = 15; i >= 0; i--) {
        uint8_t nibble = (ts >> (i * 4)) & 0xF;
        uint8_t ch = nibble < 10 ? '0' + nibble : 'a' + (nibble - 10);
        __asm__ volatile("outb %0, %1" : : "a"(ch), "Nd"((uint16_t)0xE9));
    }
    
    // Emit newline
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'\n'), "Nd"((uint16_t)0xE9));
    
    __asm__ volatile("push %0; popfq" : : "r"(rflags) : "memory", "cc");
}

/* Original debugcon helper for non-timestamped markers */
static void debugcon_write(const char *s) {
    uint64_t rflags;
    if (!s) return;
    __asm__ volatile("pushfq; pop %0; cli" : "=r"(rflags) : : "memory");
    while (*s) {
        __asm__ volatile("outb %0, %1" : : "a"((uint8_t)*s), "Nd"((uint16_t)0xE9));
        s++;
    }
    __asm__ volatile("push %0; popfq" : : "r"(rflags) : "memory", "cc");
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
    /* FORCED EXECUTION PROOF - UNCONDITIONAL
     * If this handler is called, this marker MUST appear in debugcon log.
     * No conditions, no macros, no optimizations can remove this.
     * Missing marker = handler not executing = wrong execution path.
     */
    debugcon_write("HARDENED_ENTRY\n");
    
    execution_context_type_t context_type;
    int boundary_result;
    
    /* DIAGNOSTIC: Anchored sequence tracking for second syscall proof
     * When SYS_V2_DEBUG_PUTCHAR('S') is seen, set anchor and start counting
     * subsequent syscalls as ANCHORED_SEQ_1, ANCHORED_SEQ_2, etc.
     */
    static int test_anchor_seen = 0;
    static uint64_t anchored_seq = 0;
    
    /* Check if this is the anchor syscall (debug_putchar 'S') */
    if (syscall_num == 10 && arg1 == 0x53) {  /* SYS_V2_DEBUG_PUTCHAR, 'S' */
        test_anchor_seen = 1;
        anchored_seq = 0;
        debugcon_write_with_timestamp("DIAG_TEST_ANCHOR_SET");
    }
    
    /* If anchor is set, emit anchored sequence markers for subsequent syscalls */
    if (test_anchor_seen && syscall_num == 10) {
        anchored_seq++;
        if (anchored_seq == 1) {
            debugcon_write_with_timestamp("DIAG_ANCHORED_SEQ_1");
        } else if (anchored_seq == 2) {
            debugcon_write_with_timestamp("DIAG_ANCHORED_SEQ_2");
        } else if (anchored_seq == 3) {
            debugcon_write_with_timestamp("DIAG_ANCHORED_SEQ_3");
        }
    }
    
    /* DIAGNOSTIC: Kernel handler entry */
    debugcon_write_with_timestamp("DIAG_KERNEL_HANDLER_ENTRY");
    
    /* Get current execution context - EXPLICIT ROLE MODEL */
    extern proc_t *current_proc;
    uint64_t context_id = 0;
    uint64_t process_id = 0;
    
    if (current_proc) {
        context_id = (uint64_t)current_proc->pid;
        process_id = (uint64_t)current_proc->pid;
        
        /* PATCH C1: Read cached context type (hot-path optimization)
         * Cache is updated only on role transitions (cold-path)
         * This eliminates per-syscall role-to-context conversion
         */
        if (current_proc->boundary_cache_valid) {
            context_type = (execution_context_type_t)current_proc->boundary_context_type_cached;
        } else {
            /* Cache invalid - fall back to role-based detection (should never happen) */
            switch (current_proc->execution_role) {
                case PROC_EXECUTION_ROLE_BCIB:
                    context_type = EXEC_CONTEXT_BCIB;
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
        }
        
        /* Debug: Confirm context type for BCIB */
        if (context_type == EXEC_CONTEXT_BCIB) {
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
        }
    } else {
        /* No current process - kernel context */
        context_type = EXEC_CONTEXT_KERNEL;
    }
    
    /* DIAGNOSTIC: Context detection complete */
    debugcon_write_with_timestamp("DIAG_CONTEXT_DETECTION_DONE");
    
    /* Initialize boundary enforcement if not already done */
    static int boundary_init_done = 0;
    static uint64_t boundary_init_call_count = 0;
    
    /* DIAGNOSTIC: Track how many times we enter this block */
    boundary_init_call_count++;
    
    if (!boundary_init_done) {
        /* DIAGNOSTIC: Flag is 0 - entering init path */
        debugcon_write_with_timestamp("DIAG_BOUNDARY_INIT_ENTER");
        
        boundary_enforce_init();
        
        /* DIAGNOSTIC: boundary_enforce_init() complete */
        debugcon_write_with_timestamp("DIAG_BOUNDARY_ENFORCE_INIT_DONE");
        
        /* CRITICAL: Validate enforcement matrix integrity */
        /* PATCH B: Use fast table validation instead of legacy matrix scan */
        if (syscall_enforcement_fast_validate_table() != 0) {
            boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                            "Enforcement matrix validation failed - system compromised");
            return BOUNDARY_ERR_ISOLATION_VIOLATION;
        }
        
        /* DIAGNOSTIC: syscall_enforcement_validate_matrix() complete */
        debugcon_write_with_timestamp("DIAG_MATRIX_VALIDATE_DONE");
        
        boundary_init_done = 1;
        
        /* DIAGNOSTIC: Flag set to 1 - init complete */
        debugcon_write_with_timestamp("DIAG_BOUNDARY_INIT_FLAG_SET");
    } else {
        /* DIAGNOSTIC: Flag is 1 - skipping init (fast path) */
        debugcon_write_with_timestamp("DIAG_BOUNDARY_INIT_SKIPPED");
    }
    
    /* DIAGNOSTIC: Boundary init complete */
    debugcon_write_with_timestamp("DIAG_BOUNDARY_INIT_DONE");
    
    /* PATCH C1: Context type is now cached in proc_t, no need to set per-syscall
     * The boundary_set_context_type() call is removed from hot-path
     * Context type is read directly from current_proc->boundary_context_type_cached above
     */
    
    /* PATCH C VERIFICATION: Forced marker to prove cache path is executing */
    if (current_proc && current_proc->boundary_cache_valid) {
        debugcon_write_with_timestamp("PATCH_C_CACHE_HIT");
    } else {
        debugcon_write_with_timestamp("PATCH_C_CACHE_MISS");
    }
    
#if defined(AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE) && (AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE == 1)
    /* Phase-16 Boundary Enforcement: Validate syscall against context */
    
    /* DIAGNOSTIC: HOT-PATH MICRO-PROFILE - Syscall validation */
    debugcon_write_with_timestamp("DIAG_HOT_VALIDATE_SYSCALL_ENTER");
    boundary_result = boundary_validate_syscall(syscall_num, context_type, context_id);
    debugcon_write_with_timestamp("DIAG_HOT_VALIDATE_SYSCALL_DONE");
    
    if (boundary_result != 0) {
        /* Boundary violation detected - fail-closed termination already triggered */
        return (uint64_t)boundary_result;
    }
    
    /* DIAGNOSTIC: Boundary validation complete */
    debugcon_write_with_timestamp("DIAG_BOUNDARY_VALIDATE_DONE");
    
    /* Additional boundary checks for specific syscalls */
    
    /* DIAGNOSTIC: HOT-PATH MICRO-PROFILE - Bridge bypass detection */
    debugcon_write_with_timestamp("DIAG_HOT_BYPASS_CHECK_ENTER");
    boundary_result = boundary_detect_bridge_bypass(context_type, syscall_num, context_id);  /* PATCH C2: Pass cached context_type */
    debugcon_write_with_timestamp("DIAG_HOT_BYPASS_CHECK_DONE");
    
    if (boundary_result != 0) {
        return (uint64_t)boundary_result;
    }
    
    /* DIAGNOSTIC: Bridge bypass detection complete */
    debugcon_write_with_timestamp("DIAG_BRIDGE_BYPASS_CHECK_DONE");
#else
    /* Phase 16 boundary enforcement disabled for performance measurement */
    (void)boundary_result; /* Suppress unused variable warning */
#endif
    
    /* Special handling for SYS_V2_SUBMIT_EXECUTION - BCIB submission path hardening */
    if (syscall_num == SYS_V2_SUBMIT_EXECUTION) {
        void *bcib_graph = (void *)arg1;
        uint64_t graph_size = arg2;
        uint64_t exec_context_id = arg3;
        
        /* DIAGNOSTIC: HOT-PATH MICRO-PROFILE - BCIB submission check */
        debugcon_write_with_timestamp("DIAG_HOT_BCIB_SUBMIT_ENTER");
        boundary_result = boundary_check_bcib_submission_path(bcib_graph, graph_size, exec_context_id);
        debugcon_write_with_timestamp("DIAG_HOT_BCIB_SUBMIT_DONE");
        
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
    
    /* DIAGNOSTIC: BCIB submission check complete */
    debugcon_write_with_timestamp("DIAG_BCIB_SUBMISSION_CHECK_DONE");
    
    /* Validate syscall number range */
    if (syscall_num >= SYS_V2_NR) {
        boundary_fail_closed_termination(BOUNDARY_ERR_UNAUTHORIZED_SYSCALL, context_id,
                                        "Syscall number exceeds maximum allowed");
        return ESYS_V2_INVALID_SYSCALL;
    }
    
    /* DIAGNOSTIC: Syscall range check complete */
    debugcon_write_with_timestamp("DIAG_SYSCALL_RANGE_CHECK_DONE");
    
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