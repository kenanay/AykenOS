/**
 * @file vfs_types.h
 * @brief Ring3 VFS Internal Types and Structures
 * 
 * This header defines internal types and structures used by the Ring3 VFS
 * implementation. These are not part of the public API but are shared
 * between different VFS implementation modules.
 * 
 * @author Kenan AY
 * @project AykenOS - Advanced AI-Integrated Operating System
 * @created January 3, 2026
 * @phase Phase 2.2 - Ring3 Runtime Development
 */

#ifndef USERSPACE_VFS_TYPES_H
#define USERSPACE_VFS_TYPES_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * Internal Configuration Constants
 * ======================================================================== */

#define VFS_MAX_OPEN_FILES      256     /**< Maximum number of open files */
#define VFS_MAX_PATH_LENGTH     1024    /**< Maximum path length */
#define VFS_MAX_FILENAME_LENGTH 255     /**< Maximum filename length */
#define VFS_BLOCK_SIZE          4096    /**< Standard block size */
#define VFS_MAX_MMAP_REGIONS    64      /**< Maximum memory-mapped regions */

/* ========================================================================
 * Capability System Integration
 * ======================================================================== */

/**
 * @brief Capability token for file system access
 * 
 * Represents a capability token that grants specific permissions
 * to file system resources. Used to integrate with the Ring0
 * capability system.
 */
typedef struct {
    uint64_t id;                /**< Unique capability identifier */
    uint32_t permissions;       /**< Permission flags */
    uint32_t resource_type;     /**< Type of resource (file, directory, etc.) */
    uint64_t resource_id;       /**< Specific resource identifier */
    uint64_t expiry_time;       /**< Capability expiration time */
} vfs_capability_token_t;

/**
 * @brief Capability permission flags
 */
typedef enum {
    VFS_CAP_READ    = 0x01,     /**< Read permission */
    VFS_CAP_WRITE   = 0x02,     /**< Write permission */
    VFS_CAP_EXECUTE = 0x04,     /**< Execute permission */
    VFS_CAP_CREATE  = 0x08,     /**< Create permission */
    VFS_CAP_DELETE  = 0x10,     /**< Delete permission */
    VFS_CAP_ADMIN   = 0x20      /**< Administrative permission */
} vfs_capability_perms_t;

/**
 * @brief Resource types for capability system
 */
typedef enum {
    VFS_RESOURCE_FILE       = 1,    /**< Regular file */
    VFS_RESOURCE_DIRECTORY  = 2,    /**< Directory */
    VFS_RESOURCE_DEVICE     = 3,    /**< Device file */
    VFS_RESOURCE_PIPE       = 4,    /**< Named pipe */
    VFS_RESOURCE_SOCKET     = 5     /**< Socket */
} vfs_resource_type_t;

/* ========================================================================
 * Memory Mapping Integration
 * ======================================================================== */

/**
 * @brief Memory-mapped region descriptor
 * 
 * Describes a memory region that has been mapped from Ring0
 * for file access. This is the primary mechanism for file I/O
 * in the Ring3 VFS system.
 */
typedef struct {
    void *virt_addr;            /**< Virtual address of mapped region */
    uint64_t phys_addr;         /**< Physical address (if known) */
    size_t size;                /**< Size of mapped region */
    uint32_t flags;             /**< Mapping flags */
    vfs_capability_token_t cap; /**< Associated capability token */
    bool in_use;                /**< Whether this mapping is active */
    uint32_t ref_count;         /**< Reference count for sharing */
} vfs_mmap_region_t;

/**
 * @brief Memory mapping flags
 */
typedef enum {
    VFS_MMAP_READ       = 0x01,     /**< Readable mapping */
    VFS_MMAP_WRITE      = 0x02,     /**< Writable mapping */
    VFS_MMAP_EXEC       = 0x04,     /**< Executable mapping */
    VFS_MMAP_SHARED     = 0x08,     /**< Shared mapping */
    VFS_MMAP_PRIVATE    = 0x10,     /**< Private mapping */
    VFS_MMAP_FIXED      = 0x20,     /**< Fixed address mapping */
    VFS_MMAP_ANONYMOUS  = 0x40      /**< Anonymous mapping */
} vfs_mmap_flags_t;

/* ========================================================================
 * File Descriptor Management
 * ======================================================================== */

/**
 * @brief Internal file descriptor structure
 * 
 * Represents an open file in the Ring3 VFS system. Contains all
 * necessary information to perform I/O operations through memory
 * mapping and capability tokens.
 */
typedef struct {
    int fd;                         /**< File descriptor number */
    char path[VFS_MAX_PATH_LENGTH]; /**< File path */
    uint32_t flags;                 /**< Open flags */
    uint64_t offset;                /**< Current file offset */
    uint64_t size;                  /**< File size */
    vfs_mmap_region_t *mmap;        /**< Associated memory mapping */
    vfs_capability_token_t cap;     /**< File access capability */
    bool in_use;                    /**< Whether this FD is active */
    uint32_t ref_count;             /**< Reference count */
    void *private_data;             /**< Implementation-specific data */
} vfs_file_descriptor_t;

/* ========================================================================
 * VFS Implementation Context
 * ======================================================================== */

/**
 * @brief VFS implementation context
 * 
 * Contains the runtime state and configuration for a VFS implementation.
 * This allows multiple VFS implementations to coexist and provides
 * isolation between different file system backends.
 */
typedef struct {
    char name[64];                              /**< Implementation name */
    uint32_t version;                           /**< Implementation version */
    
    /* File descriptor table */
    vfs_file_descriptor_t fds[VFS_MAX_OPEN_FILES];
    uint32_t next_fd;                           /**< Next available FD */
    
    /* Memory mapping table */
    vfs_mmap_region_t mmaps[VFS_MAX_MMAP_REGIONS];
    uint32_t next_mmap;                         /**< Next available mapping slot */
    
    /* Capability management */
    vfs_capability_token_t *(*acquire_capability)(const char *path, uint32_t perms);
    int (*release_capability)(vfs_capability_token_t *cap);
    int (*validate_capability)(vfs_capability_token_t *cap, uint32_t required_perms);
    
    /* Ring0 syscall interface */
    uint64_t (*sys_map_memory)(uint64_t virt, uint64_t phys, uint64_t flags);
    uint64_t (*sys_unmap_memory)(uint64_t virt, uint64_t size);
    uint64_t (*sys_capability_bind)(uint64_t execution_ctx, vfs_capability_token_t *token);
    uint64_t (*sys_capability_revoke)(uint64_t token_id);
    
    /* Implementation-specific data */
    void *private_data;
    size_t private_data_size;
    
    /* Statistics and debugging */
    struct {
        uint64_t files_opened;
        uint64_t files_closed;
        uint64_t bytes_read;
        uint64_t bytes_written;
        uint64_t mmap_operations;
        uint64_t capability_operations;
    } stats;
    
} vfs_context_t;

/* ========================================================================
 * Error Handling
 * ======================================================================== */

/**
 * @brief VFS error context
 * 
 * Provides detailed error information for debugging and diagnostics.
 */
typedef struct {
    int error_code;                     /**< Primary error code */
    int system_errno;                   /**< System errno if applicable */
    char message[256];                  /**< Human-readable error message */
    char function[64];                  /**< Function where error occurred */
    char file[128];                     /**< Source file where error occurred */
    int line;                           /**< Line number where error occurred */
    uint64_t timestamp;                 /**< Error timestamp */
} vfs_error_context_t;

/* ========================================================================
 * Utility Macros
 * ======================================================================== */

/**
 * @brief Set VFS error with context information
 */
#define VFS_SET_ERROR(ctx, code, msg) do { \
    (ctx)->error_code = (code); \
    snprintf((ctx)->message, sizeof((ctx)->message), "%s", (msg)); \
    snprintf((ctx)->function, sizeof((ctx)->function), "%s", __FUNCTION__); \
    snprintf((ctx)->file, sizeof((ctx)->file), "%s", __FILE__); \
    (ctx)->line = __LINE__; \
    (ctx)->timestamp = vfs_get_timestamp(); \
} while(0)

/**
 * @brief Check if file descriptor is valid
 */
#define VFS_FD_VALID(fd) ((fd) >= 0 && (fd) < VFS_MAX_OPEN_FILES)

/**
 * @brief Check if capability has required permissions
 */
#define VFS_CAP_HAS_PERM(cap, perm) (((cap)->permissions & (perm)) == (perm))

/**
 * @brief Align size to block boundary
 */
#define VFS_ALIGN_TO_BLOCK(size) (((size) + VFS_BLOCK_SIZE - 1) & ~(VFS_BLOCK_SIZE - 1))

/* ========================================================================
 * Forward Declarations
 * ======================================================================== */

struct userspace_vfs;
typedef struct userspace_vfs userspace_vfs_t;

/* ========================================================================
 * Internal Function Prototypes
 * ======================================================================== */

/**
 * @brief Get current timestamp for error reporting
 */
uint64_t vfs_get_timestamp(void);

/**
 * @brief Allocate a new file descriptor
 */
int vfs_alloc_fd(vfs_context_t *ctx);

/**
 * @brief Free a file descriptor
 */
void vfs_free_fd(vfs_context_t *ctx, int fd);

/**
 * @brief Allocate a memory mapping region
 */
vfs_mmap_region_t *vfs_alloc_mmap(vfs_context_t *ctx);

/**
 * @brief Free a memory mapping region
 */
void vfs_free_mmap(vfs_context_t *ctx, vfs_mmap_region_t *mmap);

#ifdef __cplusplus
}
#endif

#endif /* USERSPACE_VFS_TYPES_H */