#ifndef AYKEN_BOUNDARY_ENFORCEMENT_H
#define AYKEN_BOUNDARY_ENFORCEMENT_H

#include <stdint.h>
#include "syscall_v2.h"

/*
 * Phase-16 Kernel Boundary Enforcement
 * 
 * This module implements strict kernel boundary hardening to ensure:
 * 1. BCIB can only use SYS_V2_SUBMIT_EXECUTION for execution submission
 * 2. Runtime_Bridge cannot bypass or replace syscall surface
 * 3. All boundary violations result in fail-closed termination
 * 4. Constitutional compliance with KERNEL.SAFETY.CRITICAL and SECURITY.BOUNDARY.VIOLATION
 */

/* Boundary violation error codes - fail-closed semantics */
#define BOUNDARY_ERR_ISOLATION_VIOLATION        -100
#define BOUNDARY_ERR_BRIDGE_BYPASS             -101
#define BOUNDARY_ERR_UNAUTHORIZED_SYSCALL      -102
#define BOUNDARY_ERR_KERNEL_API_EXPOSURE       -103
#define BOUNDARY_ERR_DIRECT_INVOCATION         -104

/* Execution context types for boundary enforcement */
typedef enum {
    EXEC_CONTEXT_UNKNOWN = 0,
    EXEC_CONTEXT_BCIB = 1,
    EXEC_CONTEXT_RUNTIME_BRIDGE = 2,
    EXEC_CONTEXT_KERNEL = 3,
    EXEC_CONTEXT_USERSPACE = 4
} execution_context_type_t;

/* Boundary enforcement state */
typedef struct {
    execution_context_type_t context_type;
    uint64_t context_id;
    uint64_t process_id;
    uint32_t allowed_syscalls_mask;
    uint32_t violation_count;
    uint64_t last_violation_time;
} boundary_state_t;

/* Function declarations */
int boundary_enforce_init(void);
int boundary_validate_syscall(uint64_t syscall_num, execution_context_type_t context_type, uint64_t context_id);
void boundary_set_context_type(uint64_t context_id, execution_context_type_t context_type, uint64_t process_id);
int boundary_check_bcib_submission_path(void *bcib_graph, uint64_t graph_size, uint64_t context_id);
int boundary_detect_bridge_bypass(execution_context_type_t ctx_type, uint64_t syscall_num, uint64_t context_id);  /* PATCH C2: Added ctx_type parameter */
void boundary_fail_closed_termination(int violation_code, uint64_t context_id, const char *reason) __attribute__((noreturn));
int boundary_audit_violation(int violation_code, uint64_t context_id, const char *details);

/* PATCH C1: Context type cache helper - converts role to context type (cold-path only) */
execution_context_type_t boundary_role_to_context_type(int role);

/* Syscall allowlist for BCIB contexts - only SYS_V2_SUBMIT_EXECUTION allowed */
#define BCIB_ALLOWED_SYSCALLS_MASK (1 << SYS_V2_SUBMIT_EXECUTION)

/* Runtime_Bridge allowlist - limited syscall surface */
#define BRIDGE_ALLOWED_SYSCALLS_MASK ( \
    (1 << SYS_V2_MAP_MEMORY) | \
    (1 << SYS_V2_UNMAP_MEMORY) | \
    (1 << SYS_V2_CAPABILITY_BIND) | \
    (1 << SYS_V2_CAPABILITY_REVOKE) | \
    (1 << SYS_V2_TIME_QUERY) | \
    (1 << SYS_V2_DEVICE_OPERATION) | \
    (1 << SYS_V2_EXTERNAL_CALL) | \
    (1 << SYS_V2_ABDF_OPERATION) \
)

/* Constitutional compliance markers */
#define CONSTITUTIONAL_RULE_KERNEL_SAFETY_CRITICAL 1
#define CONSTITUTIONAL_RULE_SECURITY_BOUNDARY_VIOLATION 2

#endif /* AYKEN_BOUNDARY_ENFORCEMENT_H */
