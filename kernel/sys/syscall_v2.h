// kernel/sys/syscall_v2.h
// AykenOS Phase 2.1 - Execution-Centric Syscall Interface
// 
// This header defines the new execution-centric syscall interface that replaces
// the traditional POSIX-like syscalls with a capability-based, execution-focused
// approach aligned with AykenOS's data-centric, AI-native philosophy.
//
// Requirements: FR-2.1.1 - Exactly 10 execution-centric syscalls
// Architecture: Ring0 provides mechanism only, Ring3 provides policy

#ifndef AYKEN_SYSCALL_V2_H
#define AYKEN_SYSCALL_V2_H

#include <stdint.h>
#include "../include/capability.h"

// ============================================================================
// EXECUTION-CENTRIC SYSCALL DEFINITIONS (Phase 2 Documentation Compliance)
// ============================================================================
//
// Syscall Numbering Plan (Final - Phase 2.5):
// - User space calls: 1000-1010 (adds 1000 offset)
// - Kernel internal: 0-10 (direct mapping)
// - All other ranges: Invalid (return -ENOSYS)
//
// Constitutional lock contract:
// - SYS_V2_BASE: fixed ABI base for user visible numbering
// - SYS_V2_MAX_INDEX: max internal index (inclusive)
// - SYS_V2_NR: number of syscalls (MAX_INDEX + 1)
// - SYS_V2_LAST: last user visible syscall number

#define SYS_V2_BASE        1000
#define SYS_V2_MAX_INDEX   10
#define SYS_V2_NR          (SYS_V2_MAX_INDEX + 1)
#define SYS_V2_LAST        (SYS_V2_BASE + SYS_V2_MAX_INDEX)

#define SYS_V2_MAP_MEMORY        0  // Memory mapping mechanism
#define SYS_V2_UNMAP_MEMORY      1  // Memory unmapping mechanism  
#define SYS_V2_SWITCH_CONTEXT    2  // Context switching mechanism
#define SYS_V2_SUBMIT_EXECUTION  3  // BCIB execution submission
#define SYS_V2_WAIT_RESULT       4  // Execution result waiting
#define SYS_V2_INTERRUPT_RETURN  5  // Interrupt handling return
#define SYS_V2_TIME_QUERY        6  // Time query mechanism
#define SYS_V2_CAPABILITY_BIND   7  // Capability token binding
#define SYS_V2_CAPABILITY_REVOKE 8  // Capability token revocation
#define SYS_V2_EXIT              9  // Process termination
#define SYS_V2_DEBUG_PUTCHAR    10  // Debug character output (Ring3 heartbeat)

// Total syscalls: exactly 11 (debug syscall added for Ring3 heartbeat)
#define SYS_V2_MAX_SYSCALL      10

// ============================================================================
// CAPABILITY SYSTEM TYPES
// ============================================================================
//
// Capability tokens provide secure, fine-grained access control to system
// resources, replacing traditional permission models with a token-based
// approach that supports the execution-centric paradigm.
//
// Note: capability_token_t is defined in kernel/include/capability.h

// Capability permission flags
#define CAP_PERM_READ       0x01    // Read access permission
#define CAP_PERM_WRITE      0x02    // Write access permission
#define CAP_PERM_EXECUTE    0x04    // Execute access permission
#define CAP_PERM_ADMIN      0x08    // Administrative access permission

// Capability resource types
#define CAP_RESOURCE_MEMORY     1   // Memory region access
#define CAP_RESOURCE_DEVICE     2   // Device access
#define CAP_RESOURCE_EXECUTION  3   // Execution context access
#define CAP_RESOURCE_TIME       4   // Time service access

// ============================================================================
// EXECUTION CONTEXT TYPES
// ============================================================================
//
// Execution contexts represent the state and environment for BCIB graph
// execution, providing isolation and resource management for data-centric
// operations.

typedef struct execution_context {
    uint64_t context_id;            // Unique execution context identifier
    uint64_t process_id;            // Associated process identifier
    void *memory_base;              // Base address of execution memory
    uint64_t memory_size;           // Size of execution memory region
    capability_token_t *capabilities; // Array of bound capabilities
    uint32_t capability_count;      // Number of bound capabilities
    uint64_t creation_time;         // Context creation timestamp
    uint32_t status;                // Execution status flags
} execution_context_t;

// Execution context status flags
#define EXEC_STATUS_CREATED     0x01    // Context created
#define EXEC_STATUS_RUNNING     0x02    // Execution in progress
#define EXEC_STATUS_WAITING     0x04    // Waiting for result
#define EXEC_STATUS_COMPLETED   0x08    // Execution completed
#define EXEC_STATUS_ERROR       0x10    // Execution error occurred

// ============================================================================
// SYSCALL FUNCTION PROTOTYPES
// ============================================================================
//
// These prototypes define the kernel-side implementation of the execution-
// centric syscalls. Each syscall provides a specific mechanism while
// delegating policy decisions to Ring3 components.

// Memory Management Syscalls
uint64_t sys_v2_map_memory(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags);
uint64_t sys_v2_unmap_memory(uint64_t virt_addr, uint64_t size);

// Context Management Syscalls  
uint64_t sys_v2_switch_context(uint64_t old_ctx_id, uint64_t new_ctx_id);

// Execution Management Syscalls
uint64_t sys_v2_submit_execution(void *bcib_graph, uint64_t graph_size, uint64_t context_id);
uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms);

// Interrupt Management Syscalls
uint64_t sys_v2_interrupt_return(uint64_t interrupt_id, uint64_t result_code);

// Time Management Syscalls
uint64_t sys_v2_time_query(uint64_t query_type, uint64_t *result_buffer);

// Capability Management Syscalls
uint64_t sys_v2_capability_bind(uint64_t execution_ctx_id, capability_token_t *token);
uint64_t sys_v2_capability_revoke(uint64_t token_id);

// Process Management Syscalls
uint64_t sys_v2_exit(uint64_t exit_code);

// Debug Syscalls (Ring3 heartbeat)
uint64_t sys_v2_debug_putchar(uint64_t character);

// ============================================================================
// SYSCALL DISPATCHER
// ============================================================================
//
// The v2 syscall dispatcher routes execution-centric syscalls to their
// appropriate handlers, providing mechanism-only implementations that
// delegate policy decisions to Ring3 components.

uint64_t syscall_v2_handler(uint64_t syscall_num, uint64_t arg1, 
                            uint64_t arg2, uint64_t arg3, uint64_t arg4);

// ============================================================================
// ERROR CODES
// ============================================================================
//
// Standardized error codes for execution-centric syscalls, providing
// consistent error reporting across the new interface.

#define ESYS_V2_SUCCESS         0   // Operation successful
#define ESYS_V2_INVALID_SYSCALL -1  // Invalid syscall number
#define ESYS_V2_INVALID_PARAM   -2  // Invalid parameter
#define ESYS_V2_NO_PERMISSION   -3  // Insufficient permissions
#define ESYS_V2_NO_MEMORY       -4  // Out of memory
#define ESYS_V2_NO_CAPABILITY   -5  // Missing required capability
#define ESYS_V2_TIMEOUT         -6  // Operation timed out
#define ESYS_V2_CONTEXT_ERROR   -7  // Execution context error
#define ESYS_V2_RESOURCE_BUSY   -8  // Resource currently busy
#define ESYS_V2_NOT_IMPLEMENTED -9  // Feature not yet implemented

#endif // AYKEN_SYSCALL_V2_H
