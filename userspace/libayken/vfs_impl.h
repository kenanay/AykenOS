/**
 * @file vfs_impl.h
 * @brief Ring3 VFS Implementation Interface
 * 
 * This header defines the interface for creating VFS implementations
 * that work with the Ring3 VFS library. It provides the framework
 * for implementing different file system backends.
 * 
 * @author Kenan AY
 * @project AykenOS - Advanced AI-Integrated Operating System
 * @created January 3, 2026
 * @phase Phase 2.2 - Ring3 Runtime Development
 */

#ifndef USERSPACE_VFS_IMPL_H
#define USERSPACE_VFS_IMPL_H

#include "vfs.h"
#include "vfs_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * VFS Implementation Factory
 * ======================================================================== */

/**
 * @brief VFS implementation factory function type
 * 
 * Factory functions create and initialize VFS implementations.
 * Each implementation provides its own factory function.
 */
typedef userspace_vfs_t *(*vfs_factory_fn_t)(vfs_context_t *ctx);

/**
 * @brief VFS implementation descriptor
 * 
 * Describes a VFS implementation that can be registered with the system.
 */
typedef struct {
    const char *name;               /**< Implementation name */
    const char *description;        /**< Human-readable description */
    uint32_t version;               /**< Implementation version */
    vfs_factory_fn_t factory;       /**< Factory function */
    uint32_t capabilities;          /**< Implementation capabilities */
    size_t context_size;            /**< Required context size */
} vfs_impl_descriptor_t;

/**
 * @brief VFS implementation capabilities
 */
typedef enum {
    VFS_IMPL_CAP_READ       = 0x01,     /**< Supports read operations */
    VFS_IMPL_CAP_WRITE      = 0x02,     /**< Supports write operations */
    VFS_IMPL_CAP_CREATE     = 0x04,     /**< Supports file creation */
    VFS_IMPL_CAP_DELETE     = 0x08,     /**< Supports file deletion */
    VFS_IMPL_CAP_DIRECTORY  = 0x10,     /**< Supports directories */
    VFS_IMPL_CAP_METADATA   = 0x20,     /**< Supports metadata operations */
    VFS_IMPL_CAP_MMAP       = 0x40,     /**< Supports memory mapping */
    VFS_IMPL_CAP_CAPABILITY = 0x80      /**< Supports capability system */
} vfs_impl_capabilities_t;

/* ========================================================================
 * Built-in VFS Implementations
 * ======================================================================== */

/**
 * @brief Create a memory-based VFS implementation
 * 
 * Creates a VFS implementation that stores files in memory.
 * Useful for temporary files and testing.
 * 
 * @param ctx VFS context to use
 * @return Pointer to VFS implementation, NULL on error
 */
userspace_vfs_t *vfs_create_memory_impl(vfs_context_t *ctx);

/**
 * @brief Create a Ring0 proxy VFS implementation
 * 
 * Creates a VFS implementation that proxies operations to Ring0
 * through the new syscall interface. This is the primary implementation
 * for accessing real file systems.
 * 
 * @param ctx VFS context to use
 * @return Pointer to VFS implementation, NULL on error
 */
userspace_vfs_t *vfs_create_ring0_proxy_impl(vfs_context_t *ctx);

/**
 * @brief Create a capability-based VFS implementation
 * 
 * Creates a VFS implementation that uses the capability system
 * for all file access operations. Provides enhanced security.
 * 
 * @param ctx VFS context to use
 * @return Pointer to VFS implementation, NULL on error
 */
userspace_vfs_t *vfs_create_capability_impl(vfs_context_t *ctx);

/**
 * @brief Create a layered VFS implementation
 * 
 * Creates a VFS implementation that can layer multiple backends,
 * allowing for union mounts and overlay file systems.
 * 
 * @param ctx VFS context to use
 * @param base_impl Base VFS implementation
 * @param overlay_impl Overlay VFS implementation
 * @return Pointer to VFS implementation, NULL on error
 */
userspace_vfs_t *vfs_create_layered_impl(vfs_context_t *ctx, 
                                          userspace_vfs_t *base_impl,
                                          userspace_vfs_t *overlay_impl);

/* ========================================================================
 * VFS Implementation Helpers
 * ======================================================================== */

/**
 * @brief Initialize a VFS context
 * 
 * Initializes a VFS context with default values and sets up
 * internal data structures.
 * 
 * @param ctx Context to initialize
 * @param name Implementation name
 * @return 0 on success, -1 on error
 */
int vfs_init_context(vfs_context_t *ctx, const char *name);

/**
 * @brief Cleanup a VFS context
 * 
 * Cleans up a VFS context and releases all associated resources.
 * 
 * @param ctx Context to cleanup
 */
void vfs_cleanup_context(vfs_context_t *ctx);

/**
 * @brief Register syscall interface functions
 * 
 * Registers the Ring0 syscall interface functions with a VFS context.
 * This allows the VFS implementation to communicate with Ring0.
 * 
 * @param ctx VFS context
 * @param sys_map_memory Pointer to sys_v2_map_memory function
 * @param sys_unmap_memory Pointer to sys_v2_unmap_memory function
 * @param sys_capability_bind Pointer to sys_v2_capability_bind function
 * @param sys_capability_revoke Pointer to sys_v2_capability_revoke function
 * @return 0 on success, -1 on error
 */
int vfs_register_syscall_interface(vfs_context_t *ctx,
                                   uint64_t (*sys_map_memory)(uint64_t, uint64_t, uint64_t),
                                   uint64_t (*sys_unmap_memory)(uint64_t, uint64_t),
                                   uint64_t (*sys_capability_bind)(uint64_t, vfs_capability_token_t*),
                                   uint64_t (*sys_capability_revoke)(uint64_t));

/**
 * @brief Register capability interface functions
 * 
 * Registers capability management functions with a VFS context.
 * 
 * @param ctx VFS context
 * @param acquire_capability Function to acquire capabilities
 * @param release_capability Function to release capabilities
 * @param validate_capability Function to validate capabilities
 * @return 0 on success, -1 on error
 */
int vfs_register_capability_interface(vfs_context_t *ctx,
                                      vfs_capability_token_t *(*acquire_capability)(const char*, uint32_t),
                                      int (*release_capability)(vfs_capability_token_t*),
                                      int (*validate_capability)(vfs_capability_token_t*, uint32_t));

/* ========================================================================
 * VFS Implementation Base Class
 * ======================================================================== */

/**
 * @brief Base VFS implementation structure
 * 
 * Provides a base implementation that other VFS implementations can
 * inherit from. Contains common functionality and default implementations.
 */
typedef struct {
    userspace_vfs_t vfs;            /**< Public VFS interface */
    vfs_context_t *ctx;             /**< Associated context */
    const char *impl_name;          /**< Implementation name */
    uint32_t impl_version;          /**< Implementation version */
    uint32_t impl_capabilities;     /**< Implementation capabilities */
    
    /* Implementation-specific function pointers */
    int (*init)(struct vfs_base_impl *impl);
    void (*cleanup)(struct vfs_base_impl *impl);
    int (*configure)(struct vfs_base_impl *impl, const char *key, const void *value);
    
    /* Private data */
    void *private_data;
    size_t private_data_size;
    
} vfs_base_impl_t;

/**
 * @brief Initialize a base VFS implementation
 * 
 * Initializes the base VFS implementation structure with default
 * function implementations.
 * 
 * @param impl Base implementation to initialize
 * @param ctx VFS context to associate
 * @param name Implementation name
 * @return 0 on success, -1 on error
 */
int vfs_init_base_impl(vfs_base_impl_t *impl, vfs_context_t *ctx, const char *name);

/**
 * @brief Cleanup a base VFS implementation
 * 
 * Cleans up the base VFS implementation and releases resources.
 * 
 * @param impl Base implementation to cleanup
 */
void vfs_cleanup_base_impl(vfs_base_impl_t *impl);

/* ========================================================================
 * VFS Implementation Registry
 * ======================================================================== */

/**
 * @brief Register a VFS implementation
 * 
 * Registers a VFS implementation with the global registry.
 * Registered implementations can be created by name.
 * 
 * @param descriptor Implementation descriptor
 * @return 0 on success, -1 on error
 */
int vfs_register_impl(const vfs_impl_descriptor_t *descriptor);

/**
 * @brief Unregister a VFS implementation
 * 
 * Removes a VFS implementation from the global registry.
 * 
 * @param name Implementation name to unregister
 * @return 0 on success, -1 on error
 */
int vfs_unregister_impl(const char *name);

/**
 * @brief Create a VFS implementation by name
 * 
 * Creates a VFS implementation using a registered factory function.
 * 
 * @param name Implementation name
 * @param ctx VFS context to use
 * @return Pointer to VFS implementation, NULL on error
 */
userspace_vfs_t *vfs_create_impl(const char *name, vfs_context_t *ctx);

/**
 * @brief List available VFS implementations
 * 
 * Returns an array of available VFS implementation names.
 * 
 * @param names Array to store implementation names
 * @param max_names Maximum number of names to return
 * @return Number of implementations returned
 */
int vfs_list_impls(const char **names, int max_names);

/* ========================================================================
 * VFS Implementation Utilities
 * ======================================================================== */

/**
 * @brief Validate a file path
 * 
 * Validates that a file path is well-formed and safe to use.
 * 
 * @param path Path to validate
 * @return 0 if valid, -1 if invalid
 */
int vfs_validate_path(const char *path);

/**
 * @brief Normalize a file path
 * 
 * Normalizes a file path by resolving relative components
 * and removing redundant separators.
 * 
 * @param path Input path
 * @param normalized Buffer for normalized path
 * @param size Size of normalized buffer
 * @return 0 on success, -1 on error
 */
int vfs_normalize_path(const char *path, char *normalized, size_t size);

/**
 * @brief Convert VFS flags to capability permissions
 * 
 * Converts VFS open flags to capability permission flags.
 * 
 * @param vfs_flags VFS flags (VFS_O_* constants)
 * @return Capability permission flags
 */
uint32_t vfs_flags_to_capability_perms(int vfs_flags);

/**
 * @brief Convert capability permissions to memory mapping flags
 * 
 * Converts capability permissions to memory mapping flags.
 * 
 * @param cap_perms Capability permissions
 * @return Memory mapping flags
 */
uint32_t vfs_capability_perms_to_mmap_flags(uint32_t cap_perms);

#ifdef __cplusplus
}
#endif

#endif /* USERSPACE_VFS_IMPL_H */