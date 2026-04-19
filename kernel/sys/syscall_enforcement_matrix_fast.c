#include "syscall_enforcement_matrix_fast.h"
#include "syscall_v2.h"

/*
 * Phase-16 Fast Syscall Enforcement - Bitmask Optimization
 * Implementation file - single instance of fast enforcement table
 */

/* Fast enforcement table - indexed by proc_execution_role_t */
syscall_enforcement_fast_entry_t SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_MAX];

/* Initialization flag */
int syscall_enforcement_fast_initialized = 0;

#define SYS_V2_ALL_FROZEN_SYSCALLS_MASK ((1ULL << SYS_V2_NR) - 1ULL)

/**
 * Initialize fast enforcement table from enforcement rules
 * MUST be called during kernel boot (cold path)
 */
void syscall_enforcement_fast_init(void) {
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
    
    /* User: all frozen v2 syscalls allowed */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_USER].allowed_syscalls = SYS_V2_ALL_FROZEN_SYSCALLS_MASK;
    
    /* Kernel: all frozen v2 syscalls allowed */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_KERNEL].allowed_syscalls = SYS_V2_ALL_FROZEN_SYSCALLS_MASK;
    
    /* Unknown: no syscalls allowed (fail-closed) */
    SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_UNKNOWN].allowed_syscalls = 0;
    
    syscall_enforcement_fast_initialized = 1;
}

/**
 * Validate fast enforcement table integrity
 * MUST be called during kernel boot after init
 */
int syscall_enforcement_fast_validate_table(void) {
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

    if (SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_USER].allowed_syscalls !=
        SYS_V2_ALL_FROZEN_SYSCALLS_MASK) {
        return -1;
    }

    if (SYSCALL_ENFORCEMENT_FAST_TABLE[PROC_EXECUTION_ROLE_KERNEL].allowed_syscalls !=
        SYS_V2_ALL_FROZEN_SYSCALLS_MASK) {
        return -1;
    }
    
    return 0;
}
