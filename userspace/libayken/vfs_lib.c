/**
 * @file vfs_lib.c
 * @brief Ring3 VFS Library Main Implementation (Simplified)
 * 
 * This file provides the main VFS library implementation that integrates
 * the Ring0 proxy VFS with the existing kernel interface. It implements
 * the complete Ring3 VFS using new syscalls as specified in task 2.2.1.3.
 * 
 * @author Kenan AY
 * @project AykenOS - Advanced AI-Integrated Operating System
 * @created January 10, 2026
 * @phase Phase 2.2 - Ring3 Runtime Development
 * @task 2.2.1.3 - Implement Ring3 VFS using new syscalls (Step C)
 */

#include "vfs.h"
#include "vfs_kernel_interface.h"
#include <stddef.h>
#include <stdint.h>

/* ========================================================================
 * Global VFS State (Simplified)
 * ======================================================================== */

/**
 * @brief Global VFS implementation pointer
 */
userspace_vfs_t *g_userspace_vfs = NULL;

/**
 * @brief VFS initialization flag
 */
static int g_vfs_initialized = 0;

/* ========================================================================
 * External Functions from Ring0 Proxy
 * ======================================================================== */

// These functions are implemented in vfs_ring0_proxy.c
extern userspace_vfs_t *vfs_create_ring0_proxy_impl(vfs_context_t *ctx);
extern int userspace_open(const char *path, int flags);
extern int userspace_read(int fd, void *buf, size_t count);
extern int userspace_write(int fd, const void *buf, size_t count);
extern int userspace_close(int fd);

/* ========================================================================
 * VFS Library Management Functions
 * ======================================================================== */

/**
 * @brief Initialize the Ring3 VFS library
 */
int vfs_init(void)
{
    if (g_vfs_initialized) {
        return 0; // Already initialized
    }
    
    // Create Ring0 proxy VFS implementation
    g_userspace_vfs = vfs_create_ring0_proxy_impl(NULL);
    if (!g_userspace_vfs) {
        return -1;
    }
    
    g_vfs_initialized = 1;
    return 0;
}

/**
 * @brief Shutdown the Ring3 VFS library
 */
int vfs_shutdown(void)
{
    if (!g_vfs_initialized) {
        return 0;
    }
    
    // Reset global state
    g_userspace_vfs = NULL;
    g_vfs_initialized = 0;
    
    return 0;
}

/**
 * @brief Get the current VFS implementation
 */
userspace_vfs_t *vfs_get_implementation(void)
{
    return g_userspace_vfs;
}

/* ========================================================================
 * Kernel Interface Compatibility Functions
 * ======================================================================== */

/**
 * @brief Initialize Ring3 VFS system (kernel interface)
 */
int userspace_vfs_init(void)
{
    return vfs_init();
}

/**
 * @brief Open a file (kernel interface)
 */
userspace_vfs_file_t *userspace_vfs_open(const char *path, userspace_vfs_mode_t mode)
{
    if (!path) {
        return NULL;
    }
    
    // Ensure VFS is initialized
    if (!g_vfs_initialized) {
        if (vfs_init() != 0) {
            return NULL;
        }
    }
    
    // Convert mode to VFS flags
    int vfs_flags = 0;
    switch (mode) {
        case USERSPACE_VFS_MODE_READ:
            vfs_flags = VFS_MODE_READ;
            break;
        default:
            return NULL;
    }
    
    // Open file using Ring0 proxy
    int fd = userspace_open(path, vfs_flags);
    if (fd < 0) {
        return NULL;
    }
    
    // Return file descriptor as opaque handle
    return (userspace_vfs_file_t *)(uintptr_t)fd;
}

/**
 * @brief Read from a file (kernel interface)
 */
int userspace_vfs_read(userspace_vfs_file_t *file_handle, void *buffer, uint64_t size)
{
    if (!file_handle || !buffer || size == 0) {
        return -1;
    }
    
    // Convert handle back to file descriptor
    int fd = (int)(uintptr_t)file_handle;
    
    return userspace_read(fd, buffer, (size_t)size);
}

/**
 * @brief Seek within a file (kernel interface)
 */
int userspace_vfs_seek(userspace_vfs_file_t *file_handle, int64_t offset, 
                       userspace_vfs_seek_whence_t whence)
{
    if (!file_handle) {
        return -1;
    }
    
    // Convert handle back to file descriptor
    int fd = (int)(uintptr_t)file_handle;
    
    // Convert whence values
    int vfs_whence;
    switch (whence) {
        case USERSPACE_VFS_SEEK_SET:
            vfs_whence = VFS_SEEK_SET;
            break;
        case USERSPACE_VFS_SEEK_CUR:
            vfs_whence = VFS_SEEK_CUR;
            break;
        case USERSPACE_VFS_SEEK_END:
            vfs_whence = VFS_SEEK_END;
            break;
        default:
            return -1;
    }
    
    // Perform seek using VFS implementation
    if (g_userspace_vfs && g_userspace_vfs->seek) {
        int64_t result = g_userspace_vfs->seek(fd, offset, vfs_whence);
        return (result >= 0) ? 0 : -1;
    }
    
    return -1;
}

/**
 * @brief Close a file (kernel interface)
 */
int userspace_vfs_close(userspace_vfs_file_t *file_handle)
{
    if (!file_handle) {
        return -1;
    }
    
    // Convert handle back to file descriptor
    int fd = (int)(uintptr_t)file_handle;
    
    return userspace_close(fd);
}