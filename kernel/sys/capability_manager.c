// kernel/sys/capability_manager.c
// AykenOS Phase 2.1 - Capability System Implementation
//
// This file implements the capability-based security system that provides
// secure resource access control for the execution-centric architecture.
// Capabilities replace traditional permission checks with unforgeable tokens.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 3, 2026

#include <stdint.h>
#include <stddef.h>
#include "../include/capability.h"

// Forward declarations to avoid including problematic headers
void fb_print(const char *s);
void fb_print_int(int64_t value);
void fb_print_hex(uint64_t v);

// ============================================================================
// CAPABILITY SYSTEM STATE
// ============================================================================

// Maximum number of capabilities in the system
#define MAX_CAPABILITIES 1024

// Maximum number of execution contexts
#define MAX_EXECUTION_CONTEXTS 256

// Global capability table
static capability_extended_t capability_table[MAX_CAPABILITIES];
static uint32_t capability_count = 0;
static uint64_t next_capability_id = 1;

// Execution context capability bindings
// Each execution context can have multiple capabilities bound to it
#define MAX_CAPABILITIES_PER_CONTEXT 32

typedef struct {
    uint64_t context_id;
    uint64_t capability_ids[MAX_CAPABILITIES_PER_CONTEXT];
    uint32_t capability_count;
} execution_context_capabilities_t;

static execution_context_capabilities_t context_capabilities[MAX_EXECUTION_CONTEXTS];
static uint32_t context_count = 0;

// System initialization flag
static int capability_system_initialized = 0;

// ============================================================================
// INTERNAL HELPER FUNCTIONS
// ============================================================================

/**
 * find_capability_by_id - Find capability in table by ID
 * @capability_id: ID of capability to find
 * 
 * Returns: Pointer to capability or NULL if not found
 */
static capability_extended_t *find_capability_by_id(uint64_t capability_id)
{
    for (uint32_t i = 0; i < capability_count; i++) {
        if (capability_table[i].token.id == capability_id) {
            return &capability_table[i];
        }
    }
    return NULL;
}

/**
 * find_context_capabilities - Find capability bindings for execution context
 * @execution_ctx: ID of execution context
 * 
 * Returns: Pointer to context capabilities or NULL if not found
 */
static execution_context_capabilities_t *find_context_capabilities(uint64_t execution_ctx)
{
    for (uint32_t i = 0; i < context_count; i++) {
        if (context_capabilities[i].context_id == execution_ctx) {
            return &context_capabilities[i];
        }
    }
    return NULL;
}

/**
 * create_context_capabilities - Create new context capability binding
 * @execution_ctx: ID of execution context
 * 
 * Returns: Pointer to new context capabilities or NULL if table full
 */
static execution_context_capabilities_t *create_context_capabilities(uint64_t execution_ctx)
{
    if (context_count >= MAX_EXECUTION_CONTEXTS) {
        return NULL;
    }
    
    execution_context_capabilities_t *ctx_caps = &context_capabilities[context_count];
    ctx_caps->context_id = execution_ctx;
    ctx_caps->capability_count = 0;
    
    // Initialize capability IDs to 0
    for (uint32_t i = 0; i < MAX_CAPABILITIES_PER_CONTEXT; i++) {
        ctx_caps->capability_ids[i] = 0;
    }
    
    context_count++;
    return ctx_caps;
}

// ============================================================================
// CAPABILITY LIFECYCLE OPERATIONS
// ============================================================================

/**
 * capability_create - Create a new capability token
 * @resource_type: Type of resource (CAPABILITY_RESOURCE_*)
 * @permissions: Permission bitmask (CAPABILITY_PERM_*)
 * @resource_address: Physical address or handle of protected resource
 * @resource_size: Size of protected resource
 * 
 * Returns: New capability token with unique ID
 * Requirements: FR-2.2.1 - Capability tokens must provide secure resource access control
 */
capability_token_t capability_create(uint32_t resource_type, uint32_t permissions, 
                                   uint64_t resource_address, uint64_t resource_size)
{
    capability_token_t invalid_token = {0, 0, 0};
    
    // Check if system is initialized
    if (!capability_system_initialized) {
        fb_print("[capability] System not initialized\n");
        return invalid_token;
    }
    
    // Check if capability table is full
    if (capability_count >= MAX_CAPABILITIES) {
        fb_print("[capability] Capability table full\n");
        return invalid_token;
    }
    
    // Validate parameters
    if (resource_type == 0 || permissions == 0) {
        fb_print("[capability] Invalid parameters\n");
        return invalid_token;
    }
    
    // Create new capability
    capability_extended_t *cap = &capability_table[capability_count];
    
    // Initialize core token
    cap->token.id = next_capability_id++;
    cap->token.permissions = permissions;
    cap->token.resource_type = resource_type;
    
    // Initialize extended fields
    cap->state = CAPABILITY_STATE_ACTIVE;
    cap->owner_context = 0; // Will be set when bound to context
    cap->resource_address = resource_address;
    cap->resource_size = resource_size;
    cap->creation_time = 0; // TODO: Get actual system time
    cap->expiration_time = 0; // No expiration by default
    cap->reference_count = 0;
    cap->flags = 0;
    
    capability_count++;
    
    fb_print("[capability] Created capability ID=");
    fb_print_int(cap->token.id);
    fb_print(" type=");
    fb_print_int(resource_type);
    fb_print(" perms=0x");
    fb_print_hex(permissions);
    fb_print("\n");
    
    return cap->token;
}

/**
 * capability_validate - Validate a capability token
 * @token: Pointer to capability token to validate
 * 
 * Returns: 0 if valid, negative error code if invalid
 * Requirements: FR-2.2.4 - Capability system must prevent unauthorized resource access
 */
int capability_validate(const capability_token_t *token)
{
    if (token == NULL) {
        return CAPABILITY_ERROR_INVALID_TOKEN;
    }
    
    // Find capability in table
    capability_extended_t *cap = find_capability_by_id(token->id);
    if (cap == NULL) {
        return CAPABILITY_ERROR_NOT_FOUND;
    }
    
    // Check if capability matches token
    if (cap->token.permissions != token->permissions ||
        cap->token.resource_type != token->resource_type) {
        return CAPABILITY_ERROR_INVALID_TOKEN;
    }
    
    // Check capability state
    switch (cap->state) {
        case CAPABILITY_STATE_ACTIVE:
            return CAPABILITY_SUCCESS;
            
        case CAPABILITY_STATE_REVOKED:
            return CAPABILITY_ERROR_REVOKED;
            
        case CAPABILITY_STATE_EXPIRED:
            return CAPABILITY_ERROR_EXPIRED;
            
        case CAPABILITY_STATE_SUSPENDED:
            return CAPABILITY_ERROR_REVOKED; // Treat suspended as revoked for now
            
        default:
            return CAPABILITY_ERROR_INVALID_TOKEN;
    }
}

/**
 * capability_revoke - Revoke a capability by ID
 * @capability_id: ID of capability to revoke
 * 
 * Returns: 0 on success, negative error code on failure
 * Requirements: FR-2.2.3 - Capability revocation must immediately invalidate access rights
 */
int capability_revoke(uint64_t capability_id)
{
    // Find capability in table
    capability_extended_t *cap = find_capability_by_id(capability_id);
    if (cap == NULL) {
        return CAPABILITY_ERROR_NOT_FOUND;
    }
    
    // Mark as revoked
    cap->state = CAPABILITY_STATE_REVOKED;
    
    // Remove from all execution contexts
    for (uint32_t i = 0; i < context_count; i++) {
        execution_context_capabilities_t *ctx_caps = &context_capabilities[i];
        
        // Find and remove capability from this context
        for (uint32_t j = 0; j < ctx_caps->capability_count; j++) {
            if (ctx_caps->capability_ids[j] == capability_id) {
                // Shift remaining capabilities down
                for (uint32_t k = j; k < ctx_caps->capability_count - 1; k++) {
                    ctx_caps->capability_ids[k] = ctx_caps->capability_ids[k + 1];
                }
                ctx_caps->capability_ids[ctx_caps->capability_count - 1] = 0;
                ctx_caps->capability_count--;
                break;
            }
        }
    }
    
    fb_print("[capability] Revoked capability ID=");
    fb_print_int(capability_id);
    fb_print("\n");
    
    return CAPABILITY_SUCCESS;
}

// ============================================================================
// CAPABILITY VERIFICATION OPERATIONS
// ============================================================================

/**
 * capability_check_permission - Check if capability has required permission
 * @token: Pointer to capability token
 * @required_permission: Required permission bitmask
 * 
 * Returns: 0 if permission granted, negative error code if denied
 * Requirements: NFR-3.3 - Resource access must be mediated through capability tokens
 */
int capability_check_permission(const capability_token_t *token, uint32_t required_permission)
{
    if (token == NULL) {
        return CAPABILITY_ERROR_INVALID_TOKEN;
    }
    
    // First validate the token
    int validation_result = capability_validate(token);
    if (validation_result != CAPABILITY_SUCCESS) {
        return validation_result;
    }
    
    // Check if token has required permissions
    if ((token->permissions & required_permission) != required_permission) {
        fb_print("[capability] Permission denied: token has 0x");
        fb_print_hex(token->permissions);
        fb_print(" but requires 0x");
        fb_print_hex(required_permission);
        fb_print("\n");
        return CAPABILITY_ERROR_PERMISSION;
    }
    
    return CAPABILITY_SUCCESS;
}

/**
 * capability_check_resource_access - Check if capability allows resource access
 * @token: Pointer to capability token
 * @resource_address: Address of resource being accessed
 * @access_size: Size of access
 * @access_type: Type of access (read/write/execute)
 * 
 * Returns: 0 if access allowed, negative error code if denied
 * Requirements: NFR-3.1 - Capability system must prevent privilege escalation
 */
int capability_check_resource_access(const capability_token_t *token, uint64_t resource_address, 
                                    uint64_t access_size, uint32_t access_type)
{
    if (token == NULL) {
        return CAPABILITY_ERROR_INVALID_TOKEN;
    }
    
    // First check permissions
    int permission_result = capability_check_permission(token, access_type);
    if (permission_result != CAPABILITY_SUCCESS) {
        return permission_result;
    }
    
    // Find the extended capability for bounds checking
    capability_extended_t *cap = find_capability_by_id(token->id);
    if (cap == NULL) {
        return CAPABILITY_ERROR_NOT_FOUND;
    }
    
    // Check resource bounds to prevent buffer overflows and privilege escalation
    if (resource_address < cap->resource_address ||
        resource_address + access_size > cap->resource_address + cap->resource_size) {
        fb_print("[capability] Bounds violation: access [0x");
        fb_print_hex(resource_address);
        fb_print("-0x");
        fb_print_hex(resource_address + access_size);
        fb_print("] outside bounds [0x");
        fb_print_hex(cap->resource_address);
        fb_print("-0x");
        fb_print_hex(cap->resource_address + cap->resource_size);
        fb_print("]\n");
        return CAPABILITY_ERROR_PERMISSION;
    }
    
    fb_print("[capability] Access granted: 0x");
    fb_print_hex(resource_address);
    fb_print(" size=");
    fb_print_int(access_size);
    fb_print(" type=0x");
    fb_print_hex(access_type);
    fb_print("\n");
    
    return CAPABILITY_SUCCESS;
}

/**
 * capability_get_by_context - Get capability for execution context and resource type
 * @execution_ctx: ID of execution context
 * @resource_type: Type of resource needed
 * 
 * Returns: Pointer to capability token or NULL if not found
 * Requirements: FR-2.2.2 - Capability binding must associate permissions with execution contexts
 */
capability_token_t *capability_get_by_context(uint64_t execution_ctx, uint32_t resource_type)
{
    // Find context capabilities
    execution_context_capabilities_t *ctx_caps = find_context_capabilities(execution_ctx);
    if (ctx_caps == NULL) {
        return NULL;
    }
    
    // Search for capability with matching resource type
    for (uint32_t i = 0; i < ctx_caps->capability_count; i++) {
        uint64_t cap_id = ctx_caps->capability_ids[i];
        capability_extended_t *cap = find_capability_by_id(cap_id);
        
        if (cap != NULL && cap->token.resource_type == resource_type && 
            cap->state == CAPABILITY_STATE_ACTIVE) {
            return &cap->token;
        }
    }
    
    return NULL;
}

// ============================================================================
// CAPABILITY BINDING OPERATIONS
// ============================================================================

/**

/**
 * capability_bind_to_context - Bind capability to execution context
 * @execution_ctx: ID of execution context
 * @token: Pointer to capability token to bind
 * 
 * Returns: 0 on success, negative error code on failure
 * Requirements: FR-2.2.2 - Capability binding must associate permissions with execution contexts
 */
int capability_bind_to_context(uint64_t execution_ctx, const capability_token_t *token)
{
    if (execution_ctx == 0 || token == NULL) {
        return CAPABILITY_ERROR_INVALID_TOKEN;
    }
    
    // Validate the capability token
    int validation_result = capability_validate(token);
    if (validation_result != CAPABILITY_SUCCESS) {
        return validation_result;
    }
    
    // Find or create context capabilities
    execution_context_capabilities_t *ctx_caps = find_context_capabilities(execution_ctx);
    if (ctx_caps == NULL) {
        ctx_caps = create_context_capabilities(execution_ctx);
        if (ctx_caps == NULL) {
            return CAPABILITY_ERROR_SYSTEM_LIMIT;
        }
    }
    
    // Check if context already has this capability
    for (uint32_t i = 0; i < ctx_caps->capability_count; i++) {
        if (ctx_caps->capability_ids[i] == token->id) {
            return CAPABILITY_ERROR_ALREADY_EXISTS;
        }
    }
    
    // Check if context has room for more capabilities
    if (ctx_caps->capability_count >= MAX_CAPABILITIES_PER_CONTEXT) {
        return CAPABILITY_ERROR_SYSTEM_LIMIT;
    }
    
    // Add capability to context
    ctx_caps->capability_ids[ctx_caps->capability_count] = token->id;
    ctx_caps->capability_count++;
    
    // Update capability owner
    capability_extended_t *cap = find_capability_by_id(token->id);
    if (cap != NULL) {
        cap->owner_context = execution_ctx;
        cap->reference_count++;
    }
    
    fb_print("[capability] Bound capability ID=");
    fb_print_int(token->id);
    fb_print(" to context=");
    fb_print_int(execution_ctx);
    fb_print("\n");
    
    return CAPABILITY_SUCCESS;
}

/**
 * capability_unbind_from_context - Unbind capability from execution context
 * @execution_ctx: ID of execution context
 * @capability_id: ID of capability to unbind
 * 
 * Returns: 0 on success, negative error code on failure
 */
int capability_unbind_from_context(uint64_t execution_ctx, uint64_t capability_id)
{
    // Find context capabilities
    execution_context_capabilities_t *ctx_caps = find_context_capabilities(execution_ctx);
    if (ctx_caps == NULL) {
        return CAPABILITY_ERROR_NOT_FOUND;
    }
    
    // Find and remove capability from context
    for (uint32_t i = 0; i < ctx_caps->capability_count; i++) {
        if (ctx_caps->capability_ids[i] == capability_id) {
            // Shift remaining capabilities down
            for (uint32_t j = i; j < ctx_caps->capability_count - 1; j++) {
                ctx_caps->capability_ids[j] = ctx_caps->capability_ids[j + 1];
            }
            ctx_caps->capability_ids[ctx_caps->capability_count - 1] = 0;
            ctx_caps->capability_count--;
            
            // Update capability reference count
            capability_extended_t *cap = find_capability_by_id(capability_id);
            if (cap != NULL && cap->reference_count > 0) {
                cap->reference_count--;
                if (cap->reference_count == 0) {
                    cap->owner_context = 0;
                }
            }
            
            fb_print("[capability] Unbound capability ID=");
            fb_print_int(capability_id);
            fb_print(" from context=");
            fb_print_int(execution_ctx);
            fb_print("\n");
            
            return CAPABILITY_SUCCESS;
        }
    }
    
    return CAPABILITY_ERROR_NOT_FOUND;
}

// ============================================================================
// SYSTEM INITIALIZATION
// ============================================================================

/**
 * capability_system_init - Initialize the capability system
 * 
 * Requirements: System initialization for capability management
 */
void capability_system_init(void)
{
    // Initialize capability table
    for (uint32_t i = 0; i < MAX_CAPABILITIES; i++) {
        capability_table[i].token.id = 0;
        capability_table[i].token.permissions = 0;
        capability_table[i].token.resource_type = 0;
        capability_table[i].state = CAPABILITY_STATE_INVALID;
        capability_table[i].owner_context = 0;
        capability_table[i].resource_address = 0;
        capability_table[i].resource_size = 0;
        capability_table[i].creation_time = 0;
        capability_table[i].expiration_time = 0;
        capability_table[i].reference_count = 0;
        capability_table[i].flags = 0;
    }
    
    // Initialize context capabilities
    for (uint32_t i = 0; i < MAX_EXECUTION_CONTEXTS; i++) {
        context_capabilities[i].context_id = 0;
        context_capabilities[i].capability_count = 0;
        for (uint32_t j = 0; j < MAX_CAPABILITIES_PER_CONTEXT; j++) {
            context_capabilities[i].capability_ids[j] = 0;
        }
    }
    
    capability_count = 0;
    context_count = 0;
    next_capability_id = 1;
    capability_system_initialized = 1;
    
    fb_print("[capability] System initialized\n");
}

/**
 * capability_system_cleanup - Cleanup the capability system
 */
void capability_system_cleanup(void)
{
    capability_system_initialized = 0;
    capability_count = 0;
    context_count = 0;
    next_capability_id = 1;
    
    fb_print("[capability] System cleaned up\n");
}

/**
 * capability_get_stats - Get capability system statistics
 * @stats: Pointer to statistics structure to fill
 * 
 * Returns: 0 on success, negative error code on failure
 */
int capability_get_stats(capability_stats_t *stats)
{
    if (stats == NULL) {
        return CAPABILITY_ERROR_INVALID_TOKEN;
    }
    
    stats->total_capabilities = capability_count;
    stats->active_capabilities = 0;
    stats->revoked_capabilities = 0;
    stats->expired_capabilities = 0;
    
    // Count capabilities by state
    for (uint32_t i = 0; i < capability_count; i++) {
        switch (capability_table[i].state) {
            case CAPABILITY_STATE_ACTIVE:
                stats->active_capabilities++;
                break;
            case CAPABILITY_STATE_REVOKED:
                stats->revoked_capabilities++;
                break;
            case CAPABILITY_STATE_EXPIRED:
                stats->expired_capabilities++;
                break;
            default:
                break;
        }
    }
    
    // Calculate memory usage (rough estimate)
    stats->memory_usage = capability_count * sizeof(capability_extended_t) +
                         context_count * sizeof(execution_context_capabilities_t);
    
    return CAPABILITY_SUCCESS;
}