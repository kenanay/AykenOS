#ifndef AYKEN_SYSCALL_ENFORCEMENT_MATRIX_H
#define AYKEN_SYSCALL_ENFORCEMENT_MATRIX_H

#include "boundary_enforcement.h"
#include "../include/proc.h"
#include "syscall_v2.h"

/*
 * Phase-16 Syscall Enforcement Matrix
 * 
 * Defines explicit syscall permissions for each execution role.
 * Implements fail-closed enforcement with no exceptions.
 */

/* Syscall enforcement matrix - explicit permissions */
typedef struct {
    proc_execution_role_t role;
    const char *role_name;
    uint32_t allowed_syscalls_mask;
    const char *description;
} syscall_enforcement_entry_t;

/* CRITICAL: Syscall enforcement matrix - embedded in kernel */
static const syscall_enforcement_entry_t SYSCALL_ENFORCEMENT_MATRIX[] = {
    {
        .role = PROC_EXECUTION_ROLE_BCIB,
        .role_name = "BCIB",
        .allowed_syscalls_mask = (1 << SYS_V2_SUBMIT_EXECUTION),
        .description = "BCIB contexts: SUBMIT_EXECUTION only"
    },
    {
        .role = PROC_EXECUTION_ROLE_RUNTIME_BRIDGE,
        .role_name = "RUNTIME_BRIDGE", 
        .allowed_syscalls_mask = (
            (1 << SYS_V2_MAP_MEMORY) |
            (1 << SYS_V2_UNMAP_MEMORY) |
            (1 << SYS_V2_CAPABILITY_BIND) |
            (1 << SYS_V2_CAPABILITY_REVOKE) |
            (1 << SYS_V2_TIME_QUERY) |
            (1 << SYS_V2_DEVICE_OPERATION) |
            (1 << SYS_V2_EXTERNAL_CALL) |
            (1 << SYS_V2_ABDF_OPERATION)
        ),
        .description = "Runtime_Bridge: approved bridge syscalls, NO execution submission"
    },
    {
        .role = PROC_EXECUTION_ROLE_USER,
        .role_name = "USER",
        .allowed_syscalls_mask = 0xFFFFFFFF, /* All syscalls allowed */
        .description = "Regular userspace: full syscall access"
    },
    {
        .role = PROC_EXECUTION_ROLE_KERNEL,
        .role_name = "KERNEL",
        .allowed_syscalls_mask = 0xFFFFFFFF, /* All syscalls allowed */
        .description = "Kernel contexts: unrestricted access"
    },
    {
        .role = PROC_EXECUTION_ROLE_UNKNOWN,
        .role_name = "UNKNOWN",
        .allowed_syscalls_mask = 0, /* No syscalls allowed - fail closed */
        .description = "Unknown role: fail-closed, no syscalls allowed"
    }
};

#define SYSCALL_ENFORCEMENT_MATRIX_SIZE (sizeof(SYSCALL_ENFORCEMENT_MATRIX) / sizeof(SYSCALL_ENFORCEMENT_MATRIX[0]))

/* Function declarations */
int syscall_enforcement_validate(proc_execution_role_t role, uint64_t syscall_num);
const char* syscall_enforcement_get_role_name(proc_execution_role_t role);
uint32_t syscall_enforcement_get_allowed_mask(proc_execution_role_t role);
int syscall_enforcement_validate_matrix(void);

/* Critical enforcement rules - NO EXCEPTIONS */
#define ENFORCEMENT_RULE_BCIB_SUBMIT_ONLY 1
#define ENFORCEMENT_RULE_BRIDGE_NO_SUBMIT 2
#define ENFORCEMENT_RULE_UNKNOWN_FAIL_CLOSED 3

#endif /* AYKEN_SYSCALL_ENFORCEMENT_MATRIX_H */
