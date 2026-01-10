// kernel/include/capability.h
// AykenOS Phase 2.1 - Capability-Based Security System
//
// This header defines the capability token system that provides secure
// resource access control for the execution-centric architecture.
// Capabilities replace traditional permission checks with unforgeable tokens.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 3, 2026

#ifndef AYKEN_CAPABILITY_H
#define AYKEN_CAPABILITY_H

#include <stdint.h>

// ============================================================================
// CAPABILITY TOKEN STRUCTURE
// ============================================================================
//
// Core capability token as specified in task 2.1.2.1
// Requirements: FR-2.2.1 - Capability tokens must provide secure resource access control

typedef struct capability_token {
    uint64_t id;                    // Unique capability identifier
    uint32_t permissions;           // Permission bitmask (see CAPABILITY_PERM_*)
    uint32_t resource_type;         // Resource type (see CAPABILITY_RESOURCE_*)
} capability_token_t;

// ============================================================================
// RESOURCE TYPES
// ============================================================================
//
// Defines the types of resources that can be protected by capabilities
// Requirements: FR-3.3.1, FR-5.1.2 - Device and AI model access via capabilities

#define CAPABILITY_RESOURCE_MEMORY      0x01    // Physical/virtual memory regions
#define CAPABILITY_RESOURCE_DEVICE      0x02    // Hardware devices (DevFS)
#define CAPABILITY_RESOURCE_FILE        0x03    // File system objects (VFS)
#define CAPABILITY_RESOURCE_NETWORK     0x04    // Network interfaces
#define CAPABILITY_RESOURCE_GPU         0x05    // GPU compute resources
#define CAPABILITY_RESOURCE_AI_MODEL    0x06    // AI model weights and inference
#define CAPABILITY_RESOURCE_EXECUTION   0x07    // Execution contexts and BCIB graphs
#define CAPABILITY_RESOURCE_TIME        0x08    // Timer and scheduling resources
#define CAPABILITY_RESOURCE_IPC         0x09    // Inter-process communication
#define CAPABILITY_RESOURCE_SYSTEM      0x0A    // System-level operations

// ============================================================================
// PERMISSION FLAGS
// ============================================================================
//
// Permission bitmasks that can be combined for fine-grained access control
// Requirements: FR-2.2.2 - Capability binding must associate permissions with execution contexts

// Basic access permissions
#define CAPABILITY_PERM_READ            (1 << 0)    // Read access
#define CAPABILITY_PERM_WRITE           (1 << 1)    // Write access
#define CAPABILITY_PERM_EXECUTE         (1 << 2)    // Execute access
#define CAPABILITY_PERM_DELETE          (1 << 3)    // Delete/destroy access

// Advanced permissions
#define CAPABILITY_PERM_CREATE          (1 << 4)    // Create new resources
#define CAPABILITY_PERM_MODIFY_META     (1 << 5)    // Modify metadata/attributes
#define CAPABILITY_PERM_GRANT           (1 << 6)    // Grant capabilities to others
#define CAPABILITY_PERM_REVOKE          (1 << 7)    // Revoke capabilities from others

// System-level permissions
#define CAPABILITY_PERM_ADMIN           (1 << 8)    // Administrative access
#define CAPABILITY_PERM_DEBUG           (1 << 9)    // Debug/introspection access
#define CAPABILITY_PERM_EXCLUSIVE       (1 << 10)   // Exclusive access (no sharing)
#define CAPABILITY_PERM_PERSISTENT      (1 << 11)   // Capability survives context switch

// Convenience permission combinations
#define CAPABILITY_PERM_READ_WRITE      (CAPABILITY_PERM_READ | CAPABILITY_PERM_WRITE)
#define CAPABILITY_PERM_FULL_ACCESS     (CAPABILITY_PERM_READ | CAPABILITY_PERM_WRITE | \
                                        CAPABILITY_PERM_EXECUTE | CAPABILITY_PERM_DELETE)

// ============================================================================
// CAPABILITY STATES
// ============================================================================
//
// Capability lifecycle states for tracking and validation
// Requirements: FR-2.2.3 - Capability revocation must immediately invalidate access rights

typedef enum {
    CAPABILITY_STATE_INVALID = 0,   // Uninitialized or corrupted capability
    CAPABILITY_STATE_ACTIVE = 1,    // Valid and usable capability
    CAPABILITY_STATE_SUSPENDED = 2, // Temporarily disabled capability
    CAPABILITY_STATE_REVOKED = 3,   // Permanently revoked capability
    CAPABILITY_STATE_EXPIRED = 4    // Time-limited capability that has expired
} capability_state_t;

// ============================================================================
// EXTENDED CAPABILITY STRUCTURE
// ============================================================================
//
// Extended capability structure for internal kernel use
// Includes additional metadata for security and lifecycle management

typedef struct capability_extended {
    capability_token_t token;       // Core capability token
    capability_state_t state;       // Current capability state
    uint64_t owner_context;         // Execution context that owns this capability
    uint64_t resource_address;      // Physical address or handle of protected resource
    uint64_t resource_size;         // Size of protected resource (for memory/files)
    uint64_t creation_time;         // When capability was created (for auditing)
    uint64_t expiration_time;       // When capability expires (0 = never)
    uint32_t reference_count;       // Number of active references
    uint32_t flags;                 // Additional capability flags
} capability_extended_t;

// ============================================================================
// CAPABILITY FLAGS
// ============================================================================
//
// Additional flags for capability behavior control

#define CAPABILITY_FLAG_TRANSFERABLE    (1 << 0)    // Can be transferred to other contexts
#define CAPABILITY_FLAG_DELEGATABLE     (1 << 1)    // Can create derived capabilities
#define CAPABILITY_FLAG_AUDITABLE       (1 << 2)    // Access is logged for security audit
#define CAPABILITY_FLAG_TIME_LIMITED    (1 << 3)    // Has expiration time
#define CAPABILITY_FLAG_SINGLE_USE      (1 << 4)    // Automatically revoked after first use
#define CAPABILITY_FLAG_CONTEXT_BOUND   (1 << 5)    // Cannot survive context switches

// ============================================================================
// CAPABILITY MANAGER INTERFACE
// ============================================================================
//
// Function prototypes for capability system operations
// These will be implemented in the capability manager module

// Capability lifecycle operations
capability_token_t capability_create(uint32_t resource_type, uint32_t permissions, 
                                   uint64_t resource_address, uint64_t resource_size);
int capability_validate(const capability_token_t *token);
int capability_revoke(uint64_t capability_id);
int capability_suspend(uint64_t capability_id);
int capability_resume(uint64_t capability_id);

// Capability binding operations (for syscalls)
int capability_bind_to_context(uint64_t execution_ctx, const capability_token_t *token);
int capability_unbind_from_context(uint64_t execution_ctx, uint64_t capability_id);
capability_token_t *capability_get_by_context(uint64_t execution_ctx, uint32_t resource_type);

// Capability verification operations
int capability_check_permission(const capability_token_t *token, uint32_t required_permission);
int capability_check_resource_access(const capability_token_t *token, uint64_t resource_address, 
                                    uint64_t access_size, uint32_t access_type);

// Capability delegation operations
capability_token_t capability_derive(const capability_token_t *parent, uint32_t new_permissions);
int capability_transfer(const capability_token_t *token, uint64_t source_ctx, uint64_t dest_ctx);

// ============================================================================
// CAPABILITY SYSTEM INITIALIZATION
// ============================================================================
//
// System initialization and management functions

void capability_system_init(void);
void capability_system_cleanup(void);
int capability_system_status(void);

// Statistics and debugging
typedef struct {
    uint64_t total_capabilities;
    uint64_t active_capabilities;
    uint64_t revoked_capabilities;
    uint64_t expired_capabilities;
    uint64_t memory_usage;
} capability_stats_t;

int capability_get_stats(capability_stats_t *stats);
void capability_dump_table(void);  // Debug function

// ============================================================================
// INTEGRATION WITH SYSCALLS
// ============================================================================
//
// These macros and functions integrate with the v2 syscall interface
// Requirements: FR-2.1.1 - Integration with sys_v2_capability_bind/revoke

// Convert capability token to syscall parameter format
static inline void *capability_to_syscall_param(const capability_token_t *token) {
    return (void *)token;
}

// Convert syscall parameter to capability token
static inline const capability_token_t *capability_from_syscall_param(const void *param) {
    return (const capability_token_t *)param;
}

// ============================================================================
// ERROR CODES
// ============================================================================
//
// Capability-specific error codes

#define CAPABILITY_SUCCESS              0       // Operation successful
#define CAPABILITY_ERROR_INVALID_TOKEN  -1      // Invalid or corrupted capability token
#define CAPABILITY_ERROR_PERMISSION     -2      // Insufficient permissions
#define CAPABILITY_ERROR_NOT_FOUND      -3      // Capability not found
#define CAPABILITY_ERROR_ALREADY_EXISTS -4      // Capability already exists
#define CAPABILITY_ERROR_REVOKED        -5      // Capability has been revoked
#define CAPABILITY_ERROR_EXPIRED        -6      // Capability has expired
#define CAPABILITY_ERROR_CONTEXT_BOUND  -7      // Capability is bound to different context
#define CAPABILITY_ERROR_NOT_TRANSFERABLE -8    // Capability cannot be transferred
#define CAPABILITY_ERROR_RESOURCE_BUSY  -9      // Resource is exclusively locked
#define CAPABILITY_ERROR_SYSTEM_LIMIT   -10     // System capability limit reached

// ============================================================================
// SECURITY CONSIDERATIONS
// ============================================================================
//
// Requirements: NFR-3.1 - Capability system must prevent privilege escalation
//
// Security features implemented in this design:
// 1. Unforgeable tokens with unique IDs
// 2. Fine-grained permission control
// 3. Resource-specific access validation
// 4. Automatic expiration and revocation
// 5. Audit trail for security monitoring
// 6. Context binding to prevent token theft
// 7. Reference counting to prevent use-after-free
//
// The capability system provides the foundation for secure resource access
// in the Ring3-centric architecture, replacing traditional permission checks
// with cryptographically secure capability tokens.

#endif // AYKEN_CAPABILITY_H