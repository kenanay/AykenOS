/**
 * @file kernel/fs/vfs.h
 * @brief Kernel VFS Interface Header
 * 
 * This header defines the kernel VFS interface that acts as a proxy
 * to the Ring3 VFS implementation. All VFS operations are redirected
 * to Ring3 userspace, removing VFS policy code from Ring0.
 * 
 * @author Kenan AY
 * @project AykenOS - Advanced AI-Integrated Operating System
 * @created January 10, 2026
 * @phase Phase 2.2 - Ring3 Runtime Development
 * @task 2.2.1.2 - Convert kernel VFS to Ring3 proxy (Step B)
 */

#ifndef KERNEL_VFS_H
#define KERNEL_VFS_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// VFS DATA TYPES
// ============================================================================

/**
 * @brief Opaque file handle
 * 
 * Represents an open file in the VFS system. The actual implementation
 * is handled by Ring3 VFS, this is just an opaque handle.
 */
typedef struct vfs_file vfs_file_t;

/**
 * @brief File status information
 */
typedef struct {
    uint64_t size;          /**< File size in bytes */
    uint64_t blocks;        /**< Number of blocks allocated */
    uint32_t mode;          /**< File mode and permissions */
    uint32_t uid;           /**< User ID of owner */
    uint32_t gid;           /**< Group ID of owner */
    uint64_t atime;         /**< Last access time */
    uint64_t mtime;         /**< Last modification time */
    uint64_t ctime;         /**< Creation time */
} vfs_stat_t;

// ============================================================================
// VFS CONSTANTS
// ============================================================================

// File open flags
#define VFS_O_RDONLY    0x00    /**< Read only */
#define VFS_O_WRONLY    0x01    /**< Write only */
#define VFS_O_RDWR      0x02    /**< Read and write */
#define VFS_O_CREAT     0x04    /**< Create if not exists */
#define VFS_O_TRUNC     0x08    /**< Truncate on open */
#define VFS_O_APPEND    0x10    /**< Append mode */

// Seek whence values
#define VFS_SEEK_SET    0       /**< Seek from beginning */
#define VFS_SEEK_CUR    1       /**< Seek from current position */
#define VFS_SEEK_END    2       /**< Seek from end */

// Mount flags
#define VFS_MOUNT_RDONLY    0x01    /**< Read-only mount */
#define VFS_MOUNT_NOEXEC    0x02    /**< No execution from mount */
#define VFS_MOUNT_NOSUID    0x04    /**< No setuid/setgid */

// ============================================================================
// VFS INITIALIZATION
// ============================================================================

/**
 * @brief Initialize VFS system
 * 
 * Initializes the VFS system by setting up the Ring3 VFS proxy.
 * All subsequent VFS operations will be redirected to Ring3.
 * 
 * @return 0 on success, negative error code on failure
 */
int vfs_init(void);

// ============================================================================
// VFS FILE OPERATIONS
// ============================================================================

/**
 * @brief Open a file
 * 
 * Opens a file through the Ring3 VFS proxy. This function redirects
 * to the Ring3 VFS implementation which uses sys_v2_map_memory and
 * capability tokens for secure file access.
 * 
 * @param path File path to open
 * @param flags Open flags (VFS_O_*)
 * @return File handle on success, NULL on failure
 */
vfs_file_t *vfs_open(const char *path, int flags);

/**
 * @brief Read from a file
 * 
 * Reads data from an open file through the Ring3 VFS proxy.
 * Uses memory-mapped access for efficient I/O.
 * 
 * @param file File handle
 * @param buffer Buffer to store read data
 * @param size Number of bytes to read
 * @return Number of bytes read on success, negative error code on failure
 */
int vfs_read(vfs_file_t *file, void *buffer, uint64_t size);

/**
 * @brief Write to a file
 * 
 * Writes data to an open file through the Ring3 VFS proxy.
 * 
 * @param file File handle
 * @param buffer Buffer containing data to write
 * @param size Number of bytes to write
 * @return Number of bytes written on success, negative error code on failure
 */
int vfs_write(vfs_file_t *file, const void *buffer, uint64_t size);

/**
 * @brief Seek within a file
 * 
 * Changes the file position for subsequent read/write operations.
 * 
 * @param file File handle
 * @param offset Seek offset
 * @param whence Seek origin (VFS_SEEK_*)
 * @return 0 on success, negative error code on failure
 */
int vfs_seek(vfs_file_t *file, int64_t offset, int whence);

/**
 * @brief Close a file
 * 
 * Closes an open file and releases associated resources through
 * the Ring3 VFS proxy.
 * 
 * @param file File handle to close
 * @return 0 on success, negative error code on failure
 */
int vfs_close(vfs_file_t *file);

// ============================================================================
// VFS DIRECTORY OPERATIONS
// ============================================================================

/**
 * @brief Create a directory
 * 
 * Creates a directory through the Ring3 VFS proxy.
 * 
 * @param path Directory path to create
 * @param mode Directory permissions
 * @return 0 on success, negative error code on failure
 */
int vfs_mkdir(const char *path, int mode);

/**
 * @brief Remove a directory
 * 
 * Removes an empty directory through the Ring3 VFS proxy.
 * 
 * @param path Directory path to remove
 * @return 0 on success, negative error code on failure
 */
int vfs_rmdir(const char *path);

// ============================================================================
// VFS FILE SYSTEM OPERATIONS
// ============================================================================

/**
 * @brief Get file status
 * 
 * Retrieves file status information through the Ring3 VFS proxy.
 * 
 * @param path File path
 * @param stat Pointer to status structure to fill
 * @return 0 on success, negative error code on failure
 */
int vfs_stat(const char *path, vfs_stat_t *stat);

/**
 * @brief Remove a file
 * 
 * Removes a file through the Ring3 VFS proxy.
 * 
 * @param path File path to remove
 * @return 0 on success, negative error code on failure
 */
int vfs_unlink(const char *path);

// ============================================================================
// VFS MOUNT OPERATIONS
// ============================================================================

/**
 * @brief Mount a filesystem
 * 
 * Mounts a filesystem through the Ring3 VFS proxy.
 * 
 * @param device Device path
 * @param mount_point Mount point path
 * @param fs_type Filesystem type
 * @param flags Mount flags
 * @return 0 on success, negative error code on failure
 */
int vfs_mount(const char *device, const char *mount_point, const char *fs_type, int flags);

/**
 * @brief Unmount a filesystem
 * 
 * Unmounts a filesystem through the Ring3 VFS proxy.
 * 
 * @param mount_point Mount point path
 * @param flags Unmount flags
 * @return 0 on success, negative error code on failure
 */
int vfs_unmount(const char *mount_point, int flags);

#ifdef __cplusplus
}
#endif

#endif /* KERNEL_VFS_H */