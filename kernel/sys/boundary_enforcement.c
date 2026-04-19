#include "boundary_enforcement.h"
#include "syscall_enforcement_matrix.h"
#include "../include/ayken.h"
#include "../include/serial.h"
#include "../include/execution_slot.h"
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

static void boundary_zero_memory(void *ptr, uint64_t size)
{
    uint8_t *bytes = (uint8_t *)ptr;
    for (uint64_t i = 0; i < size; i++) {
        bytes[i] = 0;
    }
}

static void boundary_write_i64(int64_t value)
{
    char buf[32];
    int idx = 0;

    if (value == 0) {
        serial_write_char('0');
        return;
    }

    if (value < 0) {
        serial_write_char('-');
        value = -value;
    }

    while (value > 0 && idx < (int)sizeof(buf)) {
        buf[idx++] = (char)('0' + (value % 10));
        value /= 10;
    }

    while (idx > 0) {
        serial_write_char(buf[--idx]);
    }
}

static void boundary_write_u64(uint64_t value)
{
    char buf[32];
    int idx = 0;

    if (value == 0) {
        serial_write_char('0');
        return;
    }

    while (value > 0 && idx < (int)sizeof(buf)) {
        buf[idx++] = (char)('0' + (value % 10));
        value /= 10;
    }

    while (idx > 0) {
        serial_write_char(buf[--idx]);
    }
}

/**
 * Initialize boundary enforcement subsystem
 * Must be called during kernel initialization
 */
int boundary_enforce_init(void) {
    /* Task 3: Emit diagnostic marker for boot-time init */
    debugcon_write("[DIAG_BOUNDARY_INIT_BOOT_ENTER]\n");
    
    if (boundary_initialized) {
        debugcon_write("[DIAG_BOUNDARY_INIT_BOOT_ALREADY_DONE]\n");
        return 0; /* Already initialized */
    }
    
    /* Clear all boundary states */
    boundary_zero_memory(boundary_states, (uint64_t)sizeof(boundary_states));
    boundary_zero_memory(violation_audit_log, (uint64_t)sizeof(violation_audit_log));
    violation_log_index = 0;
    
    boundary_initialized = 1;
    
    debugcon_write("[DIAG_BOUNDARY_INIT_BOOT_DONE]\n");
    debug_printf("[BOUNDARY] Kernel boundary enforcement initialized\n");
    return 0;
}

/**
 * Set context type for a given context_id
 * Called by syscall dispatcher to register context type before enforcement checks
 * 
 * CRITICAL: This must be called BEFORE boundary_detect_bridge_bypass() to ensure
 * boundary_states[] array is populated with correct context_type
 */
void boundary_set_context_type(uint64_t context_id, execution_context_type_t context_type, uint64_t process_id) {
    if (!boundary_initialized) {
        return; /* Silently fail if not initialized - init will happen soon */
    }
    
    boundary_state_t *state = &boundary_states[context_id % MAX_EXECUTION_CONTEXTS];
    state->context_type = context_type;
    state->context_id = context_id;
    state->process_id = process_id;
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
    if (graph_size < 16) {  /* Minimum: sizeof(bcib_graph_t) = 16 bytes */
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "BCIB graph too small");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    if (graph_size > MAX_BCIB_GRAPH_SIZE) {
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "BCIB graph too large");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Validate BCIB graph magic number */
    uint32_t *magic_ptr = (uint32_t *)bcib_graph;
    if (*magic_ptr != 0x42434942) {  /* "BCIB" in little-endian */
        serial_write("[BOUNDARY] Invalid BCIB graph magic\n");
        boundary_fail_closed_termination(BOUNDARY_ERR_ISOLATION_VIOLATION, context_id,
                                        "Invalid BCIB graph magic number");
        return BOUNDARY_ERR_ISOLATION_VIOLATION;
    }
    
    /* Emit validation success marker */
    serial_write("[BCIB_GRAPH_VALID] magic=0x42434942 size=");
    boundary_write_u64(graph_size);
    serial_write("\n");
    
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
 * 
 * **CRITICAL: This function NEVER returns after a violation**
 * After termination, execution MUST NOT continue.
 */
void boundary_fail_closed_termination(int violation_code, uint64_t context_id, const char *reason) {
    extern proc_t *current_proc;
    
    /* PHASE-16 TASK 10: IMMEDIATE TERMINATION PATH */
    /* CRITICAL: Emit markers FIRST, then immediate state change, then context switch */
    
    /* Log violation before termination */
    debug_printf("[BOUNDARY] FAIL-CLOSED TERMINATION: Code=%d, Context=%lu, Reason=%s\n",
                violation_code, context_id, reason ? reason : "Unknown");
    
    /* QEMU PROOF MARKER - Critical for fail-closed evidence */
    /* Use debugcon for immediate visibility in QEMU trace */
    debugcon_write("[[AYKEN_BOUNDARY_KILL]] process_id=");
    if (current_proc) {
        char buf[32];
        int i = 0, n = (int)current_proc->pid;
        if (n == 0) buf[i++] = '0';
        else {
            char tmp[32];
            int j = 0;
            while (n > 0) { tmp[j++] = '0' + (n % 10); n /= 10; }
            while (j > 0) buf[i++] = tmp[--j];
        }
        buf[i] = '\0';
        debugcon_write(buf);
    } else {
        debugcon_write("0");
    }
    debugcon_write("\n");
    
    /* Emit deterministic error code for validator */
    debugcon_write("[[AYKEN_BOUNDARY_ERR_CODE]] code=");
    {
        char buf[32];
        int i = 0, n = violation_code;
        if (n == 0) buf[i++] = '0';
        else {
            char tmp[32];
            int j = 0;
            while (n > 0) { tmp[j++] = '0' + (n % 10); n /= 10; }
            while (j > 0) buf[i++] = tmp[--j];
        }
        buf[i] = '\0';
        debugcon_write(buf);
    }
    debugcon_write(" reason=");
    debugcon_write(reason ? reason : "Unknown");
    debugcon_write("\n");
    
    /* Also write to serial for logging */
    serial_write("[[AYKEN_BOUNDARY_KILL]] pid=");
    if (current_proc) {
        boundary_write_i64((int64_t)current_proc->pid);
    } else {
        serial_write("0");
    }
    serial_write("\n");
    serial_write("[[AYKEN_BOUNDARY_CODE_");
    boundary_write_i64((int64_t)violation_code);
    serial_write("]]\n");
    
    serial_write("[BOUNDARY_DETAIL] code=");
    boundary_write_i64((int64_t)violation_code);
    serial_write(" context=");
    boundary_write_u64(context_id);
    serial_write(" reason=");
    serial_write(reason ? reason : "Unknown");
    if (current_proc) {
        serial_write(" current_role=");
        boundary_write_i64((int64_t)current_proc->execution_role);
        serial_write(" current_state=");
        boundary_write_i64((int64_t)current_proc->state);
        serial_write(" current_type=");
        boundary_write_i64((int64_t)current_proc->type);
    }
    serial_write("\n");
    
    /* Constitutional compliance: KERNEL.SAFETY.CRITICAL and SECURITY.BOUNDARY.VIOLATION */
    debug_printf("[CONSTITUTIONAL] VIOLATION: KERNEL.SAFETY.CRITICAL + SECURITY.BOUNDARY.VIOLATION\n");
    
    /* Audit the violation */
    boundary_audit_violation(violation_code, context_id, reason);
    
    /* HARD FAIL-CLOSED TERMINATION - NO RETURN */
    if (current_proc && current_proc->type == PROC_TYPE_USER) {
        /* PHASE-16 TASK 10: IMMEDIATE TERMINATION */
        /* Step 1: Mark process as TERMINAL immediately - scheduler will NEVER reschedule */
        current_proc->state = PROC_TERMINAL;
        current_proc->wait_obj = NULL;
        
        debug_printf("[BOUNDARY] Process PID=%d marked TERMINAL - will never reschedule\n", current_proc->pid);
        
        /* Step 2: Remove from runqueue IMMEDIATELY - no more scheduling */
        extern void sched_remove_process_everywhere(proc_t *p);
        sched_remove_process_everywhere(current_proc);
        
        debug_printf("[BOUNDARY] Process removed from scheduler runqueue\n");
        
        /* Step 3: Disable interrupts and force immediate context switch */
        __asm__ volatile("cli");
        
        debug_printf("[BOUNDARY] IMMEDIATE TERMINATION: Forcing context switch - NEVER RETURN\n");
        
        /* Force scheduler to run - this will switch away from TERMINAL process */
        extern void sched_yield(void);
        sched_yield();
        
        /* UNREACHABLE: If we reach here, system is broken */
        debug_printf("[BOUNDARY] CRITICAL: Execution continued after TERMINAL - HALTING\n");
        while (1) {
            __asm__ volatile("hlt");
        }
        
    } else if (current_proc && current_proc->type == PROC_TYPE_KERNEL) {
        /* Kernel process boundary violation - this is critical */
        debug_printf("[BOUNDARY] CRITICAL: Kernel process boundary violation - system halt\n");
        
        /* For kernel processes, we cannot safely terminate, so halt the system */
        debug_printf("[BOUNDARY] CRITICAL: System halt due to kernel boundary violation\n");
        
        /* Disable interrupts and halt */
        __asm__ volatile("cli");
        while (1) {
            __asm__ volatile("hlt");
        }
        
    } else {
        /* No current process or unknown state - halt system */
        debug_printf("[BOUNDARY] CRITICAL: No current process during violation - HALTING\n");
        
        /* Disable interrupts and halt */
        __asm__ volatile("cli");
        while (1) {
            __asm__ volatile("hlt");
        }
    }
    
    /* This point should NEVER be reached */
    __builtin_unreachable();
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
