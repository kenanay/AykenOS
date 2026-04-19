#ifndef AYKEN_SYSCALL_ENFORCEMENT_MATRIX_FAST_H
#define AYKEN_SYSCALL_ENFORCEMENT_MATRIX_FAST_H

#include "boundary_enforcement.h"
#include "../include/proc.h"

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

/* Fast enforcement table - single instance in .c file */
extern syscall_enforcement_fast_entry_t SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_MAX];
extern int syscall_enforcement_fast_initialized;

/* Function declarations */
void syscall_enforcement_fast_init(void);
int syscall_enforcement_fast_validate_table(void);

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

#endif /* AYKEN_SYSCALL_ENFORCEMENT_MATRIX_FAST_H */
