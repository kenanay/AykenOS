/**
 * @file vfs_kernel_interface.h
 * @brief Ring3 VFS Kernel Interface Functions
 * 
 * This header defines the Ring3 VFS functions that correspond to the
 * kernel VFS interface. These functions are called by the kernel VFS
 * stubs to redirect operations to Ring3 userspace implementations.
 * 
 * @author Kenan AY
 * @project AykenOS - Advanced AI-Integrated Operating System
 * @created January 3, 2026
 * @phase Phase 2.2 - Ring3 Runtime Development
 */

#ifndef USERSPACE_VFS_KERNEL_INTERFACE_H
#define USERSPACE_VFS_KERNEL_INTERFACE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * Forward Declarations
 * ======================================================================== */

/**
 * @brief Opaque file handle for kernel compatibility
 * 
 * This structure maintains compatibility with the kernel VFS interface
 * while redirecting operations to Ring3 implementations.
 */
typedef struct userspace_vfs_file userspace_vfs_file_t;

/**
 * @brief File access modes (kernel compatibility)
 */
typedef enum {
    USERSPACE_VFS_MODE_READ = 1
} userspace_vfs_mode_t;

/**
 * @brief Seek whence values (kernel compatibility)
 */
typedef enum {
    USERSPACE_VFS_SEEK_SET = 0,
    USERSPACE_VFS_SEEK_CUR = 1,
    USERSPACE_VFS_SEEK_END = 2
} userspace_vfs_seek_whence_t;

/* ========================================================================
 * Ring3 VFS Functions (Kernel Interface Compatibility)
 * ======================================================================== */

/**
 * @brief Initialize the Ring3 VFS system
 * 
 * Initializes the Ring3 VFS system and sets up communication with
 * Ring0 through the new syscall interface. This function is called
 * by the kernel VFS initialization stub.
 * 
 * @return 0 on success, -1 on error
 */
int userspace_vfs_init(void);

/**
 * @brief Open a file through Ring3 VFS
 * 
 * Opens a file using the Ring3 VFS implementation. This function
 * is called by the kernel vfs_open stub to redirect file operations
 * to userspace.
 * 
 * Uses sys_v2_map_memory and capability tokens to establish access
 * to the file through Ring0 mechanisms.
 * 
 * @param path File path to open
 * @param mode Access mode (read-only for now)
 * @return Pointer to file handle on success, NULL on error
 */
userspace_vfs_file_t *userspace_vfs_open(const char *path, userspace_vfs_mode_t mode);

/**
 * @brief Read data from a file through Ring3 VFS
 * 
 * Reads data from an open file using the Ring3 VFS implementation.
 * This function is called by the kernel vfs_read stub.
 * 
 * Uses memory-mapped access established during open() to read
 * file data efficiently.
 * 
 * @param file File handle returned by userspace_vfs_open
 * @param buffer Buffer to store read data
 * @param size Number of bytes to read
 * @return Number of bytes read on success, -1 on error
 */
int userspace_vfs_read(userspace_vfs_file_t *file, void *buffer, uint64_t size);

/**
 * @brief Seek within a file through Ring3 VFS
 * 
 * Changes the file position for subsequent read operations.
 * This function is called by the kernel vfs_seek stub.
 * 
 * @param file File handle
 * @param offset Seek offset
 * @param whence Seek origin (SET/CUR/END)
 * @return 0 on success, -1 on error
 */
int userspace_vfs_seek(userspace_vfs_file_t *file, int64_t offset, userspace_vfs_seek_whence_t whence);

/**
 * @brief Close a file through Ring3 VFS
 * 
 * Closes an open file and releases associated resources.
 * This function is called by the kernel vfs_close stub.
 * 
 * Unmaps any memory regions and releases capability tokens.
 * 
 * @param file File handle to close
 * @return 0 on success, -1 on error
 */
int userspace_vfs_close(userspace_vfs_file_t *file);

/* ========================================================================
 * Ring3 DevFS Functions (Kernel Interface Compatibility)
 * ======================================================================== */

/**
 * @brief Initialize the Ring3 DevFS system
 * 
 * Initializes the Ring3 DevFS proxy system. This function is called
 * by the kernel devfs_init stub.
 * 
 * @return 0 on success, -1 on error
 */
int userspace_devfs_init(void);

/**
 * @brief Register a device through Ring3 DevFS
 * 
 * Registers a device with the Ring3 DevFS proxy. This function is
 * called by the kernel devfs_register_device stub.
 * 
 * @param name Device name
 * @param ops Device operations (will be proxied)
 * @param device_data Device-specific data
 * @return 0 on success, -1 on error
 */
int userspace_devfs_register_device(const char *name, void *ops, void *device_data);

/**
 * @brief Read from a device through Ring3 DevFS
 * 
 * Reads data from a device using capability-based access.
 * 
 * @param dev_name Device name
 * @param buffer Buffer to store read data
 * @param size Number of bytes to read
 * @return Number of bytes read on success, -1 on error
 */
int userspace_devfs_device_read(const char *dev_name, uint8_t *buffer, uint32_t size);

/**
 * @brief Write to a device through Ring3 DevFS
 * 
 * Writes data to a device using capability-based access.
 * 
 * @param dev_name Device name
 * @param buffer Buffer containing data to write
 * @param size Number of bytes to write
 * @return Number of bytes written on success, -1 on error
 */
int userspace_devfs_device_write(const char *dev_name, const uint8_t *buffer, uint32_t size);

/**
 * @brief Perform device I/O control through Ring3 DevFS
 * 
 * Performs device-specific control operations.
 * 
 * @param dev_name Device name
 * @param cmd Control command
 * @param arg Command argument
 * @return 0 on success, -1 on error
 */
int userspace_devfs_device_ioctl(const char *dev_name, uint32_t cmd, void *arg);

/**
 * @brief Close device access through Ring3 DevFS
 * 
 * Closes device access and releases associated resources.
 * 
 * @param dev_name Device name
 */
void userspace_devfs_device_close(const char *dev_name);

#ifdef __cplusplus
}
#endif

#endif /* USERSPACE_VFS_KERNEL_INTERFACE_H */