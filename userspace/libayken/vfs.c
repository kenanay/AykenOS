// userspace/libayken/vfs.c
// AykenOS Phase 2.2 - Ring3 VFS Library Implementation
//
// This file implements the Ring3 VFS interface that moves file system operations
// from Ring0 to Ring3. This is Step A (API Design) - basic structure and stubs.
// Full implementation will be completed in Steps B and C.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 10, 2026

#include "vfs.h"
#include "vfs_kernel_interface.h"
#include "../../shared/abi/syscall_v2.h"
#include "../../shared/abi/capability.h"
#include <stddef.h>

// Simple implementations for missing standard library functions
static size_t strlen(const char *str) {
    size_t len = 0;
    if (str) {
        while (str[len]) len++;
    }
    return len;
}

static char *strcpy(char *dest, const char *src) {
    char *d = dest;
    if (dest && src) {
        while ((*d++ = *src++));
    }
    return dest;
}

static char *strncpy(char *dest, const char *src, size_t n) {
    size_t i;
    if (dest && src) {
        for (i = 0; i < n && src[i] != '\0'; i++) {
            dest[i] = src[i];
        }
        for (; i < n; i++) {
            dest[i] = '\0';
        }
    }
    return dest;
}

static void *memset(void *s, int c, size_t n) {
    unsigned char *p = (unsigned char *)s;
    if (p) {
        while (n--) {
            *p++ = (unsigned char)c;
        }
    }
    return s;
}

static void *vfs_heap_alloc(size_t size) {
    // Simplified malloc - in real implementation would use proper allocator
    static char heap[64 * 1024]; // 64KB heap
    static size_t heap_pos = 0;
    
    if (heap_pos + size > sizeof(heap)) {
        return NULL; // Out of memory
    }
    
    void *ptr = &heap[heap_pos];
    heap_pos += size;
    return ptr;
}

static void vfs_heap_free(void *ptr) {
    // Simplified free - in real implementation would use proper allocator
    (void)ptr; // No-op for now
}

// ============================================================================
// SYSCALL INTERFACE
// ============================================================================

/**
 * @brief Perform syscall to Ring0
 */
static inline uint64_t syscall(uint64_t syscall_num, uint64_t arg1, uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    uint64_t result;
    __asm__ volatile (
        "int $0x80"
        : "=a" (result)
        : "a" (syscall_num), "b" (arg1), "c" (arg2), "d" (arg3), "S" (arg4)
        : "memory"
    );
    return result;
}

// ============================================================================
// MISSING TYPE DEFINITIONS
// ============================================================================

// Global VFS pointer for Ring0 proxy integration
userspace_vfs_t *g_userspace_vfs = NULL;

// Global VFS interface instance
static userspace_vfs_t g_vfs_interface;

// Current VFS context
static vfs_context_t *g_current_context = NULL;

// VFS statistics
static vfs_stats_t g_vfs_stats = {0};

// Debug level
static int g_debug_level = 0;

// File descriptor table for Ring3 VFS
#define MAX_VFS_FDS 256
static struct {
    int in_use;
    char path[256];
    uint32_t flags;
    off_t position;
    capability_token_t capability;
} g_fd_table[MAX_VFS_FDS];

static int g_next_fd = 3; // Start after stdin/stdout/stderr

// ============================================================================
// FORWARD DECLARATIONS
// ============================================================================

// File operations
static int vfs_open_impl(const char *path, uint32_t flags);
static int vfs_close_impl(int fd);
static ssize_t vfs_read_impl(int fd, void *buffer, size_t count);
static ssize_t vfs_write_impl(int fd, const void *buffer, size_t count);
static off_t vfs_seek_impl(int fd, off_t offset, int whence);
static int vfs_truncate_impl(int fd, off_t length);
static int vfs_sync_impl(int fd);

// Directory operations
static int vfs_mkdir_impl(const char *path, uint32_t mode);
static int vfs_rmdir_impl(const char *path);
static int vfs_readdir_impl(int fd, vfs_directory_entry_t **entries);
static int vfs_rewinddir_impl(int fd);

// File system operations
static int vfs_stat_impl(const char *path, vfs_file_info_t *info);
static int vfs_fstat_impl(int fd, vfs_file_info_t *info);
static int vfs_unlink_impl(const char *path);
static int vfs_rename_impl(const char *old_path, const char *new_path);
static int vfs_link_impl(const char *target, const char *link_path);
static int vfs_symlink_impl(const char *target, const char *link_path);
static ssize_t vfs_readlink_impl(const char *path, char *buffer, size_t buffer_size);

// Mount operations
static int vfs_mount_impl(const char *device, const char *mount_point, 
                         const char *filesystem_type, uint32_t flags, 
                         const char *options);
static int vfs_unmount_impl(const char *mount_point, uint32_t flags);
static int vfs_get_mount_info_impl(const char *mount_point, vfs_mount_info_t *info);

// Capability operations
static capability_token_t vfs_request_file_capability_impl(const char *path, uint32_t permissions);
static int vfs_bind_file_capability_impl(const capability_token_t *capability);
static int vfs_revoke_file_capability_impl(uint64_t capability_id);

// Advanced operations
static int vfs_ioctl_impl(int fd, uint32_t request, void *arg);
static void* vfs_mmap_file_impl(int fd, off_t offset, size_t length, uint32_t flags);
static int vfs_munmap_file_impl(void *addr, size_t length);
static int vfs_get_filesystem_stats_impl(const char *path, vfs_mount_info_t *stats);

// ============================================================================
// VFS INTERFACE INITIALIZATION
// ============================================================================

/**
 * vfs_init_userspace - Initialize Ring3 VFS library
 */
int vfs_init_userspace(void)
{
    // Initialize file descriptor table
    for (int i = 0; i < MAX_VFS_FDS; i++) {
        g_fd_table[i].in_use = 0;
        g_fd_table[i].path[0] = '\0';
        g_fd_table[i].flags = 0;
        g_fd_table[i].position = 0;
        g_fd_table[i].capability.id = 0;
    }
    
    // Initialize VFS interface function pointers
    g_vfs_interface.open = vfs_open_impl;
    g_vfs_interface.close = vfs_close_impl;
    g_vfs_interface.read = vfs_read_impl;
    g_vfs_interface.write = vfs_write_impl;
    g_vfs_interface.seek = vfs_seek_impl;
    g_vfs_interface.truncate = vfs_truncate_impl;
    g_vfs_interface.sync = vfs_sync_impl;
    
    g_vfs_interface.mkdir = vfs_mkdir_impl;
    g_vfs_interface.rmdir = vfs_rmdir_impl;
    g_vfs_interface.readdir = vfs_readdir_impl;
    g_vfs_interface.rewinddir = vfs_rewinddir_impl;
    
    g_vfs_interface.stat = vfs_stat_impl;
    g_vfs_interface.fstat = vfs_fstat_impl;
    g_vfs_interface.unlink = vfs_unlink_impl;
    g_vfs_interface.rename = vfs_rename_impl;
    g_vfs_interface.link = vfs_link_impl;
    g_vfs_interface.symlink = vfs_symlink_impl;
    g_vfs_interface.readlink = vfs_readlink_impl;
    
    g_vfs_interface.mount = vfs_mount_impl;
    g_vfs_interface.unmount = vfs_unmount_impl;
    g_vfs_interface.get_mount_info = vfs_get_mount_info_impl;
    
    g_vfs_interface.request_file_capability = vfs_request_file_capability_impl;
    g_vfs_interface.bind_file_capability = vfs_bind_file_capability_impl;
    g_vfs_interface.revoke_file_capability = vfs_revoke_file_capability_impl;
    
    g_vfs_interface.ioctl = vfs_ioctl_impl;
    g_vfs_interface.mmap_file = vfs_mmap_file_impl;
    g_vfs_interface.munmap_file = vfs_munmap_file_impl;
    g_vfs_interface.get_filesystem_stats = vfs_get_filesystem_stats_impl;
    
    // Reset statistics
    memset(&g_vfs_stats, 0, sizeof(g_vfs_stats));
    
    return VFS_SUCCESS;
}

/**
 * vfs_cleanup_userspace - Cleanup Ring3 VFS library
 */
void vfs_cleanup_userspace(void)
{
    // Close all open file descriptors
    for (int i = 0; i < MAX_VFS_FDS; i++) {
        if (g_fd_table[i].in_use) {
            vfs_close_impl(i);
        }
    }
    
    // Cleanup current context
    if (g_current_context) {
        vfs_destroy_context(g_current_context);
        g_current_context = NULL;
    }
}

/**
 * get_userspace_vfs - Get the Ring3 VFS interface
 */
userspace_vfs_t* get_userspace_vfs(void)
{
    return &g_vfs_interface;
}

// ============================================================================
// CONTEXT MANAGEMENT
// ============================================================================

/**
 * vfs_create_context - Create VFS operation context
 */
vfs_context_t* vfs_create_context(uint64_t execution_context_id)
{
    vfs_context_t *context = vfs_heap_alloc(sizeof(vfs_context_t));
    if (!context) {
        return NULL;
    }
    
    context->execution_context_id = execution_context_id;
    context->capabilities = NULL;
    context->capability_count = 0;
    context->flags = 0;
    context->private_data = NULL;
    
    return context;
}

/**
 * vfs_destroy_context - Destroy VFS operation context
 */
void vfs_destroy_context(vfs_context_t *context)
{
    if (!context) return;
    
    // Revoke all capabilities
    for (uint32_t i = 0; i < context->capability_count; i++) {
        if (context->capabilities[i].id != 0) {
            syscall(SYS_V2_CAPABILITY_REVOKE + 1000, context->capabilities[i].id, 0, 0, 0);
        }
    }
    
    if (context->capabilities) {
        vfs_heap_free(context->capabilities);
    }
    
    if (context->private_data) {
        vfs_heap_free(context->private_data);
    }
    
    vfs_heap_free(context);
}

/**
 * vfs_set_current_context - Set current VFS context
 */
int vfs_set_current_context(vfs_context_t *context)
{
    g_current_context = context;
    return VFS_SUCCESS;
}

/**
 * vfs_get_current_context - Get current VFS context
 */
vfs_context_t* vfs_get_current_context(void)
{
    return g_current_context;
}

// ============================================================================
// FILE OPERATIONS IMPLEMENTATION (STUBS FOR STEP A)
// ============================================================================

/**
 * vfs_open_impl - Open a file or directory
 */
static int vfs_open_impl(const char *path, uint32_t flags)
{
    if (!path) {
        return VFS_ERROR_INVALID_PATH;
    }
    
    // Find available file descriptor
    int fd = -1;
    for (int i = g_next_fd; i < MAX_VFS_FDS; i++) {
        if (!g_fd_table[i].in_use) {
            fd = i;
            break;
        }
    }
    
    if (fd == -1) {
        return VFS_ERROR_NO_MEMORY; // No available FDs
    }
    
    // TODO: Step B - Implement actual file opening via Ring0 mechanisms
    // For now, just set up the FD table entry
    g_fd_table[fd].in_use = 1;
    strncpy(g_fd_table[fd].path, path, sizeof(g_fd_table[fd].path) - 1);
    g_fd_table[fd].path[sizeof(g_fd_table[fd].path) - 1] = '\0';
    g_fd_table[fd].flags = flags;
    g_fd_table[fd].position = 0;
    
    // Request file capability
    g_fd_table[fd].capability = vfs_request_file_capability_impl(path, 
        (flags & VFS_MODE_WRITE) ? CAPABILITY_PERM_READ_WRITE : CAPABILITY_PERM_READ);
    
    g_vfs_stats.open_calls++;
    
    if (g_debug_level >= 1) {
        // Debug output would go here
    }
    
    return fd;
}

/**
 * vfs_close_impl - Close a file descriptor
 */
static int vfs_close_impl(int fd)
{
    if (fd < 0 || fd >= MAX_VFS_FDS || !g_fd_table[fd].in_use) {
        return VFS_ERROR_INVALID_FD;
    }
    
    // Revoke capability
    if (g_fd_table[fd].capability.id != 0) {
        vfs_revoke_file_capability_impl(g_fd_table[fd].capability.id);
    }
    
    // Clear FD table entry
    g_fd_table[fd].in_use = 0;
    g_fd_table[fd].path[0] = '\0';
    g_fd_table[fd].flags = 0;
    g_fd_table[fd].position = 0;
    g_fd_table[fd].capability.id = 0;
    
    g_vfs_stats.close_calls++;
    
    return VFS_SUCCESS;
}

/**
 * vfs_read_impl - Read data from a file
 */
static ssize_t vfs_read_impl(int fd, void *buffer, size_t count)
{
    if (fd < 0 || fd >= MAX_VFS_FDS || !g_fd_table[fd].in_use) {
        return VFS_ERROR_INVALID_FD;
    }
    
    if (!buffer || count == 0) {
        return VFS_ERROR_INVALID_PATH; // Invalid parameters
    }
    
    // TODO: Step B - Implement actual file reading via Ring0 mechanisms
    // For now, return stub implementation
    
    g_vfs_stats.read_calls++;
    g_vfs_stats.total_bytes_read += count;
    
    // Stub: return 0 bytes read
    return 0;
}

/**
 * vfs_write_impl - Write data to a file
 */
static ssize_t vfs_write_impl(int fd, const void *buffer, size_t count)
{
    if (fd < 0 || fd >= MAX_VFS_FDS || !g_fd_table[fd].in_use) {
        return VFS_ERROR_INVALID_FD;
    }
    
    if (!buffer || count == 0) {
        return 0;
    }
    
    // Check write permissions
    if (!(g_fd_table[fd].flags & VFS_MODE_WRITE)) {
        return VFS_ERROR_PERMISSION;
    }
    
    // TODO: Step B - Implement actual file writing via Ring0 mechanisms
    // For now, return stub implementation
    
    g_vfs_stats.write_calls++;
    g_vfs_stats.total_bytes_written += count;
    
    // Stub: return bytes written
    return count;
}

/**
 * vfs_seek_impl - Change file position
 */
static off_t vfs_seek_impl(int fd, off_t offset, int whence)
{
    if (fd < 0 || fd >= MAX_VFS_FDS || !g_fd_table[fd].in_use) {
        return VFS_ERROR_INVALID_FD;
    }
    
    // TODO: Step B - Implement actual seeking via Ring0 mechanisms
    // For now, just update position in FD table
    
    switch (whence) {
    case VFS_SEEK_SET:
        g_fd_table[fd].position = offset;
        break;
    case VFS_SEEK_CUR:
        g_fd_table[fd].position += offset;
        break;
    case VFS_SEEK_END:
        // TODO: Get file size and set position
        g_fd_table[fd].position = offset; // Stub
        break;
    default:
        return VFS_ERROR_INVALID_PATH; // Invalid whence
    }
    
    return g_fd_table[fd].position;
}

// ============================================================================
// CAPABILITY OPERATIONS IMPLEMENTATION (STUBS FOR STEP A)
// ============================================================================

/**
 * vfs_request_file_capability_impl - Request capability for file access
 */
static capability_token_t vfs_request_file_capability_impl(const char *path, uint32_t permissions)
{
    capability_token_t token = {0};
    
    if (!path) {
        return token; // Invalid token
    }
    
    // TODO: Step B - Implement actual capability request
    // For now, create a stub capability
    token.id = 0; // Will be assigned by syscall
    token.permissions = permissions;
    token.resource_type = CAPABILITY_RESOURCE_FILE;
    
    return token;
}

/**
 * vfs_bind_file_capability_impl - Bind file capability to current context
 */
static int vfs_bind_file_capability_impl(const capability_token_t *capability)
{
    if (!capability || !g_current_context) {
        return VFS_ERROR_CAPABILITY;
    }
    
    // TODO: Step B - Implement actual capability binding
    uint64_t result = syscall(SYS_V2_CAPABILITY_BIND + 1000, 
                             g_current_context->execution_context_id, 
                             (uint64_t)capability, 0, 0);
    
    if (result == ESYS_V2_SUCCESS) {
        return VFS_SUCCESS;
    } else {
        return VFS_ERROR_CAPABILITY;
    }
}

/**
 * vfs_revoke_file_capability_impl - Revoke file capability
 */
static int vfs_revoke_file_capability_impl(uint64_t capability_id)
{
    if (capability_id == 0) {
        return VFS_ERROR_CAPABILITY;
    }
    
    uint64_t result = syscall(SYS_V2_CAPABILITY_REVOKE + 1000, capability_id, 0, 0, 0);
    
    if (result == ESYS_V2_SUCCESS) {
        return VFS_SUCCESS;
    } else {
        return VFS_ERROR_CAPABILITY;
    }
}

// ============================================================================
// STUB IMPLEMENTATIONS FOR OTHER OPERATIONS
// ============================================================================
// These will be fully implemented in Steps B and C

static int vfs_truncate_impl(int fd, off_t length) {
    (void)fd; (void)length;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_sync_impl(int fd) {
    (void)fd;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_mkdir_impl(const char *path, uint32_t mode) {
    (void)path; (void)mode;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_rmdir_impl(const char *path) {
    (void)path;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_readdir_impl(int fd, vfs_directory_entry_t **entries) {
    (void)fd; (void)entries;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_rewinddir_impl(int fd) {
    (void)fd;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_stat_impl(const char *path, vfs_file_info_t *info) {
    (void)path; (void)info;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_fstat_impl(int fd, vfs_file_info_t *info) {
    (void)fd; (void)info;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_unlink_impl(const char *path) {
    (void)path;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_rename_impl(const char *old_path, const char *new_path) {
    (void)old_path; (void)new_path;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_link_impl(const char *target, const char *link_path) {
    (void)target; (void)link_path;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_symlink_impl(const char *target, const char *link_path) {
    (void)target; (void)link_path;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static ssize_t vfs_readlink_impl(const char *path, char *buffer, size_t buffer_size) {
    (void)path; (void)buffer; (void)buffer_size;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_mount_impl(const char *device, const char *mount_point, 
                         const char *filesystem_type, uint32_t flags, 
                         const char *options) {
    (void)device; (void)mount_point; (void)filesystem_type; (void)flags; (void)options;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_unmount_impl(const char *mount_point, uint32_t flags) {
    (void)mount_point; (void)flags;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_get_mount_info_impl(const char *mount_point, vfs_mount_info_t *info) {
    (void)mount_point; (void)info;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_ioctl_impl(int fd, uint32_t request, void *arg) {
    (void)fd; (void)request; (void)arg;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static void* vfs_mmap_file_impl(int fd, off_t offset, size_t length, uint32_t flags) {
    (void)fd; (void)offset; (void)length; (void)flags;
    return NULL; // TODO: Implement in Step B
}

static int vfs_munmap_file_impl(void *addr, size_t length) {
    (void)addr; (void)length;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

static int vfs_get_filesystem_stats_impl(const char *path, vfs_mount_info_t *stats) {
    (void)path; (void)stats;
    return VFS_ERROR_NOT_SUPPORTED; // TODO: Implement in Step B
}

// ============================================================================
// UTILITY FUNCTIONS IMPLEMENTATION
// ============================================================================

/**
 * vfs_path_normalize - Normalize a file path
 */
int vfs_path_normalize(const char *path, char *normalized, size_t buffer_size)
{
    if (!path || !normalized || buffer_size == 0) {
        return VFS_ERROR_INVALID_PATH;
    }
    
    // Simple normalization - remove double slashes and trailing slashes
    size_t len = strlen(path);
    if (len >= buffer_size) {
        return VFS_ERROR_NAME_TOO_LONG;
    }
    
    strcpy(normalized, path);
    
    // TODO: Implement full path normalization (., .., multiple slashes)
    
    return VFS_SUCCESS;
}

/**
 * vfs_is_absolute_path - Check if path is absolute
 */
int vfs_is_absolute_path(const char *path)
{
    if (!path || path[0] == '\0') {
        return 0;
    }
    
    return (path[0] == '/');
}

// ============================================================================
// STATISTICS AND DEBUGGING
// ============================================================================

/**
 * vfs_get_stats - Get VFS operation statistics
 */
int vfs_get_stats(vfs_stats_t *stats)
{
    if (!stats) {
        return VFS_ERROR_INVALID_PATH;
    }
    
    *stats = g_vfs_stats;
    return VFS_SUCCESS;
}

/**
 * vfs_reset_stats - Reset VFS statistics
 */
void vfs_reset_stats(void)
{
    memset(&g_vfs_stats, 0, sizeof(g_vfs_stats));
}

/**
 * vfs_enable_debug - Enable VFS debug logging
 */
void vfs_enable_debug(int level)
{
    g_debug_level = level;
}

// ============================================================================
// KERNEL INTEGRATION STUBS
// ============================================================================

/**
 * vfs_register_userspace_handler - Register Ring3 VFS handler with kernel
 */
int vfs_register_userspace_handler(userspace_vfs_t *vfs)
{
    (void)vfs;
    // TODO: Step B - Implement kernel registration
    return VFS_SUCCESS;
}

/**
 * vfs_unregister_userspace_handler - Unregister Ring3 VFS handler
 */
int vfs_unregister_userspace_handler(void)
{
    // TODO: Step B - Implement kernel unregistration
    return VFS_SUCCESS;
}

// ============================================================================
// KERNEL INTEGRATION FUNCTIONS
// ============================================================================

/**
 * userspace_vfs_init - Initialize Ring3 VFS system (kernel interface)
 */
int userspace_vfs_init(void)
{
    return vfs_init_userspace();
}

/**
 * userspace_vfs_open - Open file via Ring3 VFS (kernel interface)
 */
userspace_vfs_file_t *userspace_vfs_open(const char *path, userspace_vfs_mode_t mode)
{
    if (!path) {
        return NULL;
    }
    
    // Convert mode to VFS flags
    uint32_t flags = 0;
    if (mode & USERSPACE_VFS_MODE_READ) flags |= VFS_MODE_READ;
    
    int fd = vfs_open_impl(path, flags);
    if (fd < 0) {
        return NULL;
    }
    
    // Return file descriptor as opaque handle
    return (userspace_vfs_file_t *)(uintptr_t)fd;
}

/**
 * userspace_vfs_read - Read from file via Ring3 VFS (kernel interface)
 */
int userspace_vfs_read(userspace_vfs_file_t *file, void *buffer, uint64_t size)
{
    if (!file || !buffer || size == 0) {
        return -1;
    }
    
    int fd = (int)(uintptr_t)file;
    return (int)vfs_read_impl(fd, buffer, size);
}

/**
 * userspace_vfs_seek - Seek in file via Ring3 VFS (kernel interface)
 */
int userspace_vfs_seek(userspace_vfs_file_t *file, int64_t offset, userspace_vfs_seek_whence_t whence)
{
    if (!file) {
        return -1;
    }
    
    int fd = (int)(uintptr_t)file;
    
    // Convert whence to VFS constants
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
    
    off_t result = vfs_seek_impl(fd, offset, vfs_whence);
    return (result >= 0) ? 0 : -1;
}

/**
 * userspace_vfs_close - Close file via Ring3 VFS (kernel interface)
 */
int userspace_vfs_close(userspace_vfs_file_t *file)
{
    if (!file) {
        return -1;
    }
    
    int fd = (int)(uintptr_t)file;
    return vfs_close_impl(fd);
}
