// kernel/fs/devfs.h - Ring3 DevFS Proxy Stub Interface
// AykenOS Phase 2.2 - Kernel DevFS Stub Conversion (Step B)
//
// This header defines the kernel DevFS stub interface that redirects
// all DevFS operations to the Ring3 DevFS library. The kernel no longer
// contains device management logic.
//
// Requirements:
// - FR-2.3.2: Device policy must execute entirely in Ring3
// - FR-2.3.3: Ring0 must provide only device access mechanism
// - Task 2.2.3.2: Convert kernel DevFS to Ring3 proxy (Step B)

#ifndef __KERNEL_FS_DEVFS_H
#define __KERNEL_FS_DEVFS_H

#include <stdint.h>
#include <stddef.h>

// ============================================================================
// KERNEL DEVFS STUB INTERFACE
// ============================================================================

/**
 * @brief Initialize DevFS system (stub - redirects to Ring3)
 * 
 * This stub function redirects DevFS initialization to the Ring3 DevFS library.
 * All device management policy is now handled in Ring3 userspace.
 * 
 * @return 0 on success, negative error code on failure
 */
int devfs_init(void);

/**
 * @brief Register a device (stub - redirects to Ring3)
 * 
 * This stub function redirects device registration to the Ring3 DevFS library.
 * Device management policy is now handled entirely in Ring3.
 * 
 * @param name Device name
 * @param ops Device operations (legacy - not used in Ring3 model)
 * @param device_data Device data (legacy - not used in Ring3 model)
 * @return 0 on success, negative error code on failure
 */
int devfs_register_device(const char *name, void *ops, void *device_data);

/**
 * @brief Read from a device (stub - redirects to Ring3)
 * 
 * This stub function redirects device read operations to the Ring3 DevFS library.
 * All device I/O policy is now handled in Ring3 with capability-based security.
 * 
 * @param dev_name Device name
 * @param buffer Buffer to store read data
 * @param size Number of bytes to read
 * @return Number of bytes read on success, negative error code on failure
 */
int devfs_read(const char *dev_name, void *buffer, uint32_t size);

/**
 * @brief Write to a device (stub - redirects to Ring3)
 * 
 * This stub function redirects device write operations to the Ring3 DevFS library.
 * All device I/O policy is now handled in Ring3 with capability-based security.
 * 
 * @param dev_name Device name
 * @param buffer Buffer containing data to write
 * @param size Number of bytes to write
 * @return Number of bytes written on success, negative error code on failure
 */
int devfs_write(const char *dev_name, const void *buffer, uint32_t size);

/**
 * @brief Perform device I/O control (stub - redirects to Ring3)
 * 
 * This stub function redirects device control operations to the Ring3 DevFS library.
 * All device control policy is now handled in Ring3 with capability-based security.
 * 
 * @param dev_name Device name
 * @param cmd Control command
 * @param arg Command argument
 * @return 0 on success, negative error code on failure
 */
int devfs_ioctl(const char *dev_name, uint32_t cmd, void *arg);

/**
 * @brief Close device access (stub - redirects to Ring3)
 * 
 * This stub function redirects device close operations to the Ring3 DevFS library.
 * All device resource management is now handled in Ring3.
 * 
 * @param dev_name Device name
 */
void devfs_close(const char *dev_name);

// ============================================================================
// LEGACY COMPATIBILITY FUNCTIONS
// ============================================================================

/**
 * @brief Legacy device read function (stub - redirects to Ring3)
 * 
 * This function provides compatibility with legacy kernel code that expects
 * the old devfs_device_read interface.
 */
int devfs_device_read(const char *dev_name, uint8_t *buffer, uint32_t size);

/**
 * @brief Legacy device write function (stub - redirects to Ring3)
 * 
 * This function provides compatibility with legacy kernel code that expects
 * the old devfs_device_write interface.
 */
int devfs_device_write(const char *dev_name, const uint8_t *buffer, uint32_t size);

/**
 * @brief Legacy device ioctl function (stub - redirects to Ring3)
 * 
 * This function provides compatibility with legacy kernel code that expects
 * the old devfs_device_ioctl interface.
 */
int devfs_device_ioctl(const char *dev_name, uint32_t cmd, void *arg);

/**
 * @brief Legacy device close function (stub - redirects to Ring3)
 * 
 * This function provides compatibility with legacy kernel code that expects
 * the old devfs_device_close interface.
 */
void devfs_device_close(const char *dev_name);

#endif // __KERNEL_FS_DEVFS_H