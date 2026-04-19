#ifndef AYKEN_SYSCALL_ENFORCEMENT_MATRIX_FAST_H
#define AYKEN_SYSCALL_ENFORCEMENT_MATRIX_FAST_H

#include "boundary_enforcement.h"
#include "../include/proc.h"
#include "syscall_v2.h"

/*
 * Phase-16 Fast Syscall Enforcement - Bitmask Optimization
 * 
 * Replaces linear search + branch-heavy validation with O(1) bitmask lookup.
 * Preserves fail-closed semantics and all enforcement rules.
 * 
 * Performance target: Reduce boundary_validate_syscall() from 195k ticks to <50k ticks
 */

/* Maximum syscall number supported (must be power of 2 for efficient division) */
#define SYSCALL_ENFORCEMENT_MAX_SYSCALL 64

/* Bitmask for syscall permissions per role */
typedef struct {
    uint64_t allowed_syscalls;  /* Bitmask: bit N = syscall N allowed */
} syscall_enforcement_fast_entry_t;

/* Fast enforcement table - indexed by proc_execution_role_t */
static syscall_enforcement_fast_entry_t SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_MAX];

/* Initialization flag */
static int syscall_enforcement_fast_initialized = 0;

/**
 * Initialize fast enforcement table from legacy matrix
 * MUST be called during kernel boot (cold path)
 */
static inline void syscall_enforcement_fast_init(void) {
    if (syscall_enforcement_fast_initialized) {
        return;
    }
    
    /* BCIB: SUBMIT_EXECUTION only */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_BCIB].allowed_syscalls = 
        (1ULL << SYS_V2_SUBMIT_EXECUTION);
    
    /* Runtime_Bridge: approved bridge syscalls, NO execution submission */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_RUNTIME_BRIDGE].allowed_syscalls = (
        (1ULL << SYS_V2_MAP_MEMORY) |
        (1ULL << SYS_V2_UNMAP_MEMORY) |
        (1ULL << SYS_V2_CAPABILITY_BIND) |
        (1ULL << SYS_V2_CAPABILITY_REVOKE) |
        (1ULL << SYS_V2_TIME_QUERY) |
        (1ULL << SYS_V2_DEBUG_PUTCHAR) |
        (1ULL << SYS_V2_DEVICE_OPERATION) |
        (1ULL << SYS_V2_EXTERNAL_CALL) |
        (1ULL << SYS_V2_ABDF_OPERATION)
    );
    
    /* User: all syscalls allowed */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_USER].allowed_syscalls = 0xFFFFFFFFFFFFFFFFULL;
    
    /* Kernel: all syscalls allowed */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_KERNEL].allowed_syscalls = 0xFFFFFFFFFFFFFFFFULL;
    
    /* Unknown: no syscalls allowed (fail-closed) */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_UNKNOWN].allowed_syscalls = 0;
    
    syscall_enforcement_fast_initialized = 1;
}

/**
 * Fast syscall validation - O(1) bitmask check
 * Returns 1 if allowed, 0 if denied
 * 
 * CRITICAL: This is the hot-path. Every cycle counts.
 */
static inline int syscall_enforcement_fast_validate(proc_execution_role_t role, uint32_t syscall_num) {
    /* Fail-closed: invalid role */
    if (__builtin_expect(role >= PROC_EXECUTION_ROLE_MAX, 0)) {
        return 0;
    }
    
    /* Fail-closed: syscall number out of range */
    if (__builtin_expect(syscall_num >= SYSCALL_ENFORCEMENT_MAX_SYSCALL, 0)) {
        return 0;
    }
    
    /* O(1) bitmask check - this is the entire hot-path */
    uint64_t mask = SYSCALL_ENFORCEMENT_FAST_TABLE[role].allowed_syscalls;
    return (mask >> syscall_num) & 1;
}

/**
 * Validate fast enforcement table integrity
 * MUST be called during kernel boot after init
 */
static inline int syscall_enforcement_fast_validate_table(void) {
    if (!syscall_enforcement_fast_initialized) {
        return -1;
    }
    
    /* Verify BCIB has only SUBMIT_EXECUTION */
    uint64_t bcib_mask = SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_BCIB].allowed_syscalls;
    if (bcib_mask != (1ULL << SYS_V2_SUBMIT_EXECUTION)) {
        return -1;
    }
    
    /* Verify Runtime_Bridge does NOT have SUBMIT_EXECUTION */
    uint64_t bridge_mask = SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_RUNTIME_BRIDGE].allowed_syscalls;
    if (bridge_mask & (1ULL << SYS_V2_SUBMIT_EXECUTION)) {
        return -1;
    }
    
    /* Verify Runtime_Bridge has required bridge syscalls */
    if (!(bridge_mask & (1ULL << SYS_V2_DEVICE_OPERATION)) ||
        !(bridge_mask & (1ULL << SYS_V2_EXTERNAL_CALL)) ||
        !(bridge_mask & (1ULL << SYS_V2_ABDF_OPERATION))) {
        return -1;
    }
    
    /* Verify unknown role has no syscalls */
    uint64_t unknown_mask = SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_UNKNOWN].allowed_syscalls;
    if (unknown_mask != 0) {
        return -1;
    }
    
    return 0;
}

#endif /* AYKEN_SYSCALL_ENFORCEMENT_MATRIX_FAST_H */
