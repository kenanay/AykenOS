// userspace/libayken/vfs.h
// AykenOS Phase 2.2 - Ring3 VFS Library Interface
//
// This header defines the Ring3 VFS interface that moves file system operations
// from Ring0 to Ring3, using capability-based security and execution-centric
// syscalls for mechanism-only operations in the kernel.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 10, 2026

#ifndef AYKEN_USERSPACE_VFS_H
#define AYKEN_USERSPACE_VFS_H

#include <stdint.h>
#include <stddef.h>
#include "../../kernel/include/capability.h"

// Define missing types for VFS interface
typedef int64_t ssize_t;
typedef int64_t off_t;

// ============================================================================
// VFS OPERATION FLAGS
// ============================================================================

// File access modes
#define VFS_MODE_READ       0x01    // Read access
#define VFS_MODE_WRITE      0x02    // Write access
#define VFS_MODE_EXECUTE    0x04    // Execute access
#define VFS_MODE_CREATE     0x08    // Create if not exists
#define VFS_MODE_TRUNCATE   0x10    // Truncate on open
#define VFS_MODE_APPEND     0x20    // Append mode
#define VFS_MODE_EXCLUSIVE  0x40    // Exclusive access

// Convenience combinations
#define VFS_READ            VFS_MODE_READ
#define VFS_WRITE           VFS_MODE_WRITE
#define VFS_READ_WRITE      (VFS_MODE_READ | VFS_MODE_WRITE)
#define VFS_CREATE_WRITE    (VFS_MODE_CREATE | VFS_MODE_WRITE)

// Seek origins
#define VFS_SEEK_SET        0       // Seek from beginning
#define VFS_SEEK_CUR        1       // Seek from current position
#define VFS_SEEK_END        2       // Seek from end

// File types
#define VFS_TYPE_REGULAR    0x01    // Regular file
#define VFS_TYPE_DIRECTORY  0x02    // Directory
#define VFS_TYPE_SYMLINK    0x04    // Symbolic link
#define VFS_TYPE_DEVICE     0x08    // Device file
#define VFS_TYPE_PIPE       0x10    // Named pipe
#define VFS_TYPE_SOCKET     0x20    // Socket

// ============================================================================
// VFS DATA STRUCTURES
// ============================================================================

/**
 * vfs_file_info_t - File information structure
 */
typedef struct vfs_file_info {
    char name[256];                 // File name
    uint64_t size;                  // File size in bytes
    uint64_t creation_time;         // Creation timestamp
    uint64_t modification_time;     // Last modification timestamp
    uint64_t access_time;           // Last access timestamp
    uint32_t type;                  // File type (VFS_TYPE_*)
    uint32_t permissions;           // File permissions
    uint64_t inode;                 // Inode number
    uint32_t link_count;            // Hard link count
    uint32_t block_size;            // Preferred block size
    uint64_t blocks;                // Number of blocks allocated
} vfs_file_info_t;

/**
 * vfs_directory_entry_t - Directory entry structure
 */
typedef struct vfs_directory_entry {
    char name[256];                 // Entry name
    uint64_t inode;                 // Inode number
    uint32_t type;                  // Entry type (VFS_TYPE_*)
    uint32_t name_length;           // Length of name
    struct vfs_directory_entry *next; // Next entry in list
} vfs_directory_entry_t;

/**
 * vfs_mount_info_t - Mount point information
 */
typedef struct vfs_mount_info {
    char device[128];               // Device path
    char mount_point[256];          // Mount point path
    char filesystem_type[64];       // Filesystem type
    uint32_t flags;                 // Mount flags
    uint64_t total_space;           // Total space in bytes
    uint64_t free_space;            // Free space in bytes
    uint64_t available_space;       // Available space for non-root
} vfs_mount_info_t;

// ============================================================================
// RING3 VFS INTERFACE STRUCTURE
// ============================================================================

/**
 * userspace_vfs_t - Ring3 VFS interface structure
 * 
 * This structure contains function pointers for all VFS operations that
 * will be implemented in Ring3. The kernel VFS will become a proxy that
 * calls these Ring3 implementations.
 */
typedef struct userspace_vfs {
    // ========================================================================
    // FILE OPERATIONS
    // ========================================================================
    
    /**
     * open - Open a file or directory
     * @path: Path to the file/directory
     * @flags: Access flags (VFS_MODE_*)
     * @returns: File descriptor on success, negative error code on failure
     */
    int (*open)(const char *path, uint32_t flags);
    
    /**
     * close - Close a file descriptor
     * @fd: File descriptor to close
     * @returns: 0 on success, negative error code on failure
     */
    int (*close)(int fd);
    
    /**
     * read - Read data from a file
     * @fd: File descriptor
     * @buffer: Buffer to read into
     * @count: Number of bytes to read
     * @returns: Number of bytes read on success, negative error code on failure
     */
    ssize_t (*read)(int fd, void *buffer, size_t count);
    
    /**
     * write - Write data to a file
     * @fd: File descriptor
     * @buffer: Buffer containing data to write
     * @count: Number of bytes to write
     * @returns: Number of bytes written on success, negative error code on failure
     */
    ssize_t (*write)(int fd, const void *buffer, size_t count);
    
    /**
     * seek - Change file position
     * @fd: File descriptor
     * @offset: Offset to seek to
     * @whence: Seek origin (VFS_SEEK_*)
     * @returns: New file position on success, negative error code on failure
     */
    off_t (*seek)(int fd, off_t offset, int whence);
    
    /**
     * truncate - Truncate a file to specified length
     * @fd: File descriptor
     * @length: New file length
     * @returns: 0 on success, negative error code on failure
     */
    int (*truncate)(int fd, off_t length);
    
    /**
     * sync - Synchronize file data to storage
     * @fd: File descriptor
     * @returns: 0 on success, negative error code on failure
     */
    int (*sync)(int fd);
    
    // ========================================================================
    // DIRECTORY OPERATIONS
    // ========================================================================
    
    /**
     * mkdir - Create a directory
     * @path: Path of directory to create
     * @mode: Directory permissions
     * @returns: 0 on success, negative error code on failure
     */
    int (*mkdir)(const char *path, uint32_t mode);
    
    /**
     * rmdir - Remove a directory
     * @path: Path of directory to remove
     * @returns: 0 on success, negative error code on failure
     */
    int (*rmdir)(const char *path);
    
    /**
     * readdir - Read directory entries
     * @fd: Directory file descriptor
     * @entries: Pointer to store directory entries
     * @returns: Number of entries read on success, negative error code on failure
     */
    int (*readdir)(int fd, vfs_directory_entry_t **entries);
    
    /**
     * rewinddir - Reset directory reading position
     * @fd: Directory file descriptor
     * @returns: 0 on success, negative error code on failure
     */
    int (*rewinddir)(int fd);
    
    // ========================================================================
    // FILE SYSTEM OPERATIONS
    // ========================================================================
    
    /**
     * stat - Get file information
     * @path: Path to file
     * @info: Pointer to store file information
     * @returns: 0 on success, negative error code on failure
     */
    int (*stat)(const char *path, vfs_file_info_t *info);
    
    /**
     * fstat - Get file information by descriptor
     * @fd: File descriptor
     * @info: Pointer to store file information
     * @returns: 0 on success, negative error code on failure
     */
    int (*fstat)(int fd, vfs_file_info_t *info);
    
    /**
     * unlink - Remove a file
     * @path: Path to file to remove
     * @returns: 0 on success, negative error code on failure
     */
    int (*unlink)(const char *path);
    
    /**
     * rename - Rename/move a file
     * @old_path: Current path
     * @new_path: New path
     * @returns: 0 on success, negative error code on failure
     */
    int (*rename)(const char *old_path, const char *new_path);
    
    /**
     * link - Create a hard link
     * @target: Target file path
     * @link_path: Link path to create
     * @returns: 0 on success, negative error code on failure
     */
    int (*link)(const char *target, const char *link_path);
    
    /**
     * symlink - Create a symbolic link
     * @target: Target path
     * @link_path: Symbolic link path to create
     * @returns: 0 on success, negative error code on failure
     */
    int (*symlink)(const char *target, const char *link_path);
    
    /**
     * readlink - Read symbolic link target
     * @path: Symbolic link path
     * @buffer: Buffer to store target path
     * @buffer_size: Size of buffer
     * @returns: Length of target path on success, negative error code on failure
     */
    ssize_t (*readlink)(const char *path, char *buffer, size_t buffer_size);
    
    // ========================================================================
    // MOUNT OPERATIONS
    // ========================================================================
    
    /**
     * mount - Mount a filesystem
     * @device: Device path
     * @mount_point: Mount point path
     * @filesystem_type: Filesystem type string
     * @flags: Mount flags
     * @options: Mount options string
     * @returns: 0 on success, negative error code on failure
     */
    int (*mount)(const char *device, const char *mount_point, 
                 const char *filesystem_type, uint32_t flags, 
                 const char *options);
    
    /**
     * unmount - Unmount a filesystem
     * @mount_point: Mount point path
     * @flags: Unmount flags
     * @returns: 0 on success, negative error code on failure
     */
    int (*unmount)(const char *mount_point, uint32_t flags);
    
    /**
     * get_mount_info - Get mount point information
     * @mount_point: Mount point path
     * @info: Pointer to store mount information
     * @returns: 0 on success, negative error code on failure
     */
    int (*get_mount_info)(const char *mount_point, vfs_mount_info_t *info);
    
    // ========================================================================
    // CAPABILITY INTEGRATION
    // ========================================================================
    
    /**
     * request_file_capability - Request capability for file access
     * @path: File path
     * @permissions: Requested permissions (CAPABILITY_PERM_*)
     * @returns: Capability token on success, invalid token on failure
     */
    capability_token_t (*request_file_capability)(const char *path, uint32_t permissions);
    
    /**
     * bind_file_capability - Bind file capability to current context
     * @capability: Capability token to bind
     * @returns: 0 on success, negative error code on failure
     */
    int (*bind_file_capability)(const capability_token_t *capability);
    
    /**
     * revoke_file_capability - Revoke file capability
     * @capability_id: Capability ID to revoke
     * @returns: 0 on success, negative error code on failure
     */
    int (*revoke_file_capability)(uint64_t capability_id);
    
    // ========================================================================
    // ADVANCED OPERATIONS
    // ========================================================================
    
    /**
     * ioctl - Device-specific I/O control
     * @fd: File descriptor
     * @request: Request code
     * @arg: Request argument
     * @returns: Request-specific value on success, negative error code on failure
     */
    int (*ioctl)(int fd, uint32_t request, void *arg);
    
    /**
     * mmap_file - Memory map a file
     * @fd: File descriptor
     * @offset: File offset
     * @length: Mapping length
     * @flags: Mapping flags
     * @returns: Mapped address on success, NULL on failure
     */
    void* (*mmap_file)(int fd, off_t offset, size_t length, uint32_t flags);
    
    /**
     * munmap_file - Unmap a file mapping
     * @addr: Mapped address
     * @length: Mapping length
     * @returns: 0 on success, negative error code on failure
     */
    int (*munmap_file)(void *addr, size_t length);
    
    /**
     * get_filesystem_stats - Get filesystem statistics
     * @path: Path within filesystem
     * @stats: Pointer to store statistics
     * @returns: 0 on success, negative error code on failure
     */
    int (*get_filesystem_stats)(const char *path, vfs_mount_info_t *stats);
    
} userspace_vfs_t;

// ============================================================================
// VFS CONTEXT MANAGEMENT
// ============================================================================

/**
 * vfs_context_t - VFS operation context
 */
typedef struct vfs_context {
    uint64_t execution_context_id;  // Associated execution context
    capability_token_t *capabilities; // Bound capabilities
    uint32_t capability_count;      // Number of capabilities
    uint32_t flags;                 // Context flags
    void *private_data;             // Implementation-specific data
} vfs_context_t;

// ============================================================================
// RING3 VFS LIBRARY FUNCTIONS
// ============================================================================

/**
 * get_userspace_vfs - Get the Ring3 VFS interface
 * @returns: Pointer to VFS interface structure
 */
userspace_vfs_t* get_userspace_vfs(void);

/**
 * vfs_init_userspace - Initialize Ring3 VFS library
 * @returns: 0 on success, negative error code on failure
 */
int vfs_init_userspace(void);

/**
 * vfs_cleanup_userspace - Cleanup Ring3 VFS library
 */
void vfs_cleanup_userspace(void);

/**
 * vfs_create_context - Create VFS operation context
 * @execution_context_id: Associated execution context
 * @returns: VFS context on success, NULL on failure
 */
vfs_context_t* vfs_create_context(uint64_t execution_context_id);

/**
 * vfs_destroy_context - Destroy VFS operation context
 * @context: VFS context to destroy
 */
void vfs_destroy_context(vfs_context_t *context);

/**
 * vfs_set_current_context - Set current VFS context
 * @context: VFS context to set as current
 * @returns: 0 on success, negative error code on failure
 */
int vfs_set_current_context(vfs_context_t *context);

/**
 * vfs_get_current_context - Get current VFS context
 * @returns: Current VFS context, NULL if none set
 */
vfs_context_t* vfs_get_current_context(void);

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * vfs_path_normalize - Normalize a file path
 * @path: Input path
 * @normalized: Buffer for normalized path
 * @buffer_size: Size of buffer
 * @returns: 0 on success, negative error code on failure
 */
int vfs_path_normalize(const char *path, char *normalized, size_t buffer_size);

/**
 * vfs_path_join - Join path components
 * @base: Base path
 * @component: Path component to join
 * @result: Buffer for result path
 * @buffer_size: Size of buffer
 * @returns: 0 on success, negative error code on failure
 */
int vfs_path_join(const char *base, const char *component, 
                  char *result, size_t buffer_size);

/**
 * vfs_path_dirname - Get directory name from path
 * @path: Input path
 * @dirname: Buffer for directory name
 * @buffer_size: Size of buffer
 * @returns: 0 on success, negative error code on failure
 */
int vfs_path_dirname(const char *path, char *dirname, size_t buffer_size);

/**
 * vfs_path_basename - Get base name from path
 * @path: Input path
 * @basename: Buffer for base name
 * @buffer_size: Size of buffer
 * @returns: 0 on success, negative error code on failure
 */
int vfs_path_basename(const char *path, char *basename, size_t buffer_size);

/**
 * vfs_is_absolute_path - Check if path is absolute
 * @path: Path to check
 * @returns: 1 if absolute, 0 if relative
 */
int vfs_is_absolute_path(const char *path);

// ============================================================================
// ERROR CODES
// ============================================================================

// VFS-specific error codes (negative values)
#define VFS_SUCCESS             0       // Operation successful
#define VFS_ERROR_INVALID_PATH  -1      // Invalid path
#define VFS_ERROR_NOT_FOUND     -2      // File/directory not found
#define VFS_ERROR_PERMISSION    -3      // Permission denied
#define VFS_ERROR_EXISTS        -4      // File/directory already exists
#define VFS_ERROR_NOT_DIR       -5      // Not a directory
#define VFS_ERROR_IS_DIR        -6      // Is a directory
#define VFS_ERROR_NOT_EMPTY     -7      // Directory not empty
#define VFS_ERROR_NO_SPACE      -8      // No space left on device
#define VFS_ERROR_READ_ONLY     -9      // Read-only filesystem
#define VFS_ERROR_TOO_MANY_LINKS -10    // Too many symbolic links
#define VFS_ERROR_NAME_TOO_LONG -11     // File name too long
#define VFS_ERROR_INVALID_FD    -12     // Invalid file descriptor
#define VFS_ERROR_IO            -13     // I/O error
#define VFS_ERROR_NO_MEMORY     -14     // Out of memory
#define VFS_ERROR_BUSY          -15     // Resource busy
#define VFS_ERROR_CROSS_DEVICE  -16     // Cross-device link
#define VFS_ERROR_NOT_SUPPORTED -17     // Operation not supported
#define VFS_ERROR_CAPABILITY    -18     // Capability error

// ============================================================================
// INTEGRATION WITH KERNEL VFS
// ============================================================================

/**
 * vfs_register_userspace_handler - Register Ring3 VFS handler with kernel
 * @vfs: Ring3 VFS interface to register
 * @returns: 0 on success, negative error code on failure
 */
int vfs_register_userspace_handler(userspace_vfs_t *vfs);

/**
 * vfs_unregister_userspace_handler - Unregister Ring3 VFS handler
 * @returns: 0 on success, negative error code on failure
 */
int vfs_unregister_userspace_handler(void);

// ============================================================================
// PERFORMANCE AND DEBUGGING
// ============================================================================

/**
 * vfs_get_stats - Get VFS operation statistics
 * @stats: Buffer to store statistics
 * @returns: 0 on success, negative error code on failure
 */
typedef struct vfs_stats {
    uint64_t open_calls;
    uint64_t read_calls;
    uint64_t write_calls;
    uint64_t close_calls;
    uint64_t total_bytes_read;
    uint64_t total_bytes_written;
    uint64_t cache_hits;
    uint64_t cache_misses;
} vfs_stats_t;

int vfs_get_stats(vfs_stats_t *stats);

/**
 * vfs_reset_stats - Reset VFS statistics
 */
void vfs_reset_stats(void);

/**
 * vfs_enable_debug - Enable VFS debug logging
 * @level: Debug level (0=off, 1=basic, 2=verbose)
 */
void vfs_enable_debug(int level);

#endif // AYKEN_USERSPACE_VFS_H