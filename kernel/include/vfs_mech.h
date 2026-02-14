#ifndef AYKEN_VFS_MECH_H
#define AYKEN_VFS_MECH_H

#include <stdint.h>

#include "capability.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t k_vfs_mount_id_t;
typedef uint64_t k_vfs_handle_t;

typedef struct k_vfs_driver_ops {
    int (*io_read)(void *driver_ctx, uint64_t object_id, uint64_t offset, void *buffer, uint64_t size);
    int (*io_write)(void *driver_ctx, uint64_t object_id, uint64_t offset, const void *buffer, uint64_t size);
    int (*io_close)(void *driver_ctx, uint64_t object_id);
} k_vfs_driver_ops_t;

#define K_VFS_OK            0
#define K_VFS_ERR_INVALID  -1
#define K_VFS_ERR_DENIED   -2
#define K_VFS_ERR_FULL     -3
#define K_VFS_ERR_NOT_FOUND -4
#define K_VFS_ERR_IO       -5

int k_vfs_mech_init(void);
int k_vfs_mount(const char *mount_name, const k_vfs_driver_ops_t *ops, void *driver_ctx, k_vfs_mount_id_t *mount_id_out);
k_vfs_handle_t k_vfs_open(k_vfs_mount_id_t mount_id, uint64_t object_id, const capability_token_t *cap);
int k_vfs_close(k_vfs_handle_t handle);
int k_vfs_read(k_vfs_handle_t handle, void *buffer, uint64_t size);
int k_vfs_write(k_vfs_handle_t handle, const void *buffer, uint64_t size);

#ifdef __cplusplus
}
#endif

#endif // AYKEN_VFS_MECH_H
