#include "../include/vfs_mech.h"

#include <stddef.h>

#define K_VFS_MAX_MOUNTS   32u
#define K_VFS_MAX_HANDLES 256u
#define K_VFS_HANDLE_BASE 0x1000u

typedef struct k_vfs_mount_slot {
    int in_use;
    k_vfs_mount_id_t mount_id;
    uint64_t mount_tag;
    const k_vfs_driver_ops_t *ops;
    void *driver_ctx;
} k_vfs_mount_slot_t;

typedef struct k_vfs_handle_slot {
    int in_use;
    k_vfs_handle_t handle;
    k_vfs_mount_id_t mount_id;
    uint64_t object_id;
    uint64_t offset;
    uint32_t perms;
} k_vfs_handle_slot_t;

static k_vfs_mount_slot_t g_mount_slots[K_VFS_MAX_MOUNTS];
static k_vfs_handle_slot_t g_handle_slots[K_VFS_MAX_HANDLES];
static k_vfs_mount_id_t g_next_mount_id = 1;
static k_vfs_handle_t g_next_handle = K_VFS_HANDLE_BASE;

static uint64_t k_vfs_mount_name_tag(const char *mount_name)
{
    uint64_t tag = 1469598103934665603ULL;
    const unsigned char *p = (const unsigned char *)mount_name;

    while (p && *p) {
        tag ^= (uint64_t)(*p++);
        tag *= 1099511628211ULL;
    }
    return tag;
}

static void k_vfs_reset_state(void)
{
    uint32_t i;
    for (i = 0; i < K_VFS_MAX_MOUNTS; i++) {
        g_mount_slots[i].in_use = 0;
        g_mount_slots[i].mount_id = 0;
        g_mount_slots[i].mount_tag = 0;
        g_mount_slots[i].ops = NULL;
        g_mount_slots[i].driver_ctx = NULL;
    }
    for (i = 0; i < K_VFS_MAX_HANDLES; i++) {
        g_handle_slots[i].in_use = 0;
        g_handle_slots[i].handle = 0;
        g_handle_slots[i].mount_id = 0;
        g_handle_slots[i].object_id = 0;
        g_handle_slots[i].offset = 0;
        g_handle_slots[i].perms = 0;
    }
}

static k_vfs_mount_slot_t *k_vfs_find_mount(k_vfs_mount_id_t mount_id)
{
    uint32_t i;
    for (i = 0; i < K_VFS_MAX_MOUNTS; i++) {
        if (g_mount_slots[i].in_use && g_mount_slots[i].mount_id == mount_id) {
            return &g_mount_slots[i];
        }
    }
    return NULL;
}

static k_vfs_handle_slot_t *k_vfs_find_handle(k_vfs_handle_t handle)
{
    uint32_t i;
    for (i = 0; i < K_VFS_MAX_HANDLES; i++) {
        if (g_handle_slots[i].in_use && g_handle_slots[i].handle == handle) {
            return &g_handle_slots[i];
        }
    }
    return NULL;
}

static int k_vfs_validate_capability(const capability_token_t *cap, uint32_t needed_perm)
{
    if (!cap) {
        return K_VFS_ERR_INVALID;
    }
    if (cap->resource_type != CAPABILITY_RESOURCE_FILE) {
        return K_VFS_ERR_DENIED;
    }
    if ((cap->permissions & needed_perm) != needed_perm) {
        return K_VFS_ERR_DENIED;
    }
    return K_VFS_OK;
}

int k_vfs_mech_init(void)
{
    k_vfs_reset_state();
    g_next_mount_id = 1;
    g_next_handle = K_VFS_HANDLE_BASE;
    return K_VFS_OK;
}

int k_vfs_mount(const char *mount_name, const k_vfs_driver_ops_t *ops, void *driver_ctx, k_vfs_mount_id_t *mount_id_out)
{
    uint32_t i;

    if (!mount_name || !ops || (!ops->io_read && !ops->io_write)) {
        return K_VFS_ERR_INVALID;
    }

    for (i = 0; i < K_VFS_MAX_MOUNTS; i++) {
        if (!g_mount_slots[i].in_use) {
            g_mount_slots[i].in_use = 1;
            g_mount_slots[i].mount_id = g_next_mount_id++;
            g_mount_slots[i].mount_tag = k_vfs_mount_name_tag(mount_name);
            g_mount_slots[i].ops = ops;
            g_mount_slots[i].driver_ctx = driver_ctx;
            if (mount_id_out) {
                *mount_id_out = g_mount_slots[i].mount_id;
            }
            return K_VFS_OK;
        }
    }

    return K_VFS_ERR_FULL;
}

k_vfs_handle_t k_vfs_open(k_vfs_mount_id_t mount_id, uint64_t object_id, const capability_token_t *cap)
{
    uint32_t i;
    uint32_t io_perms;

    if (!k_vfs_find_mount(mount_id)) {
        return 0;
    }

    io_perms = CAPABILITY_PERM_READ | CAPABILITY_PERM_WRITE;
    if (k_vfs_validate_capability(cap, CAPABILITY_PERM_READ) != K_VFS_OK &&
        k_vfs_validate_capability(cap, CAPABILITY_PERM_WRITE) != K_VFS_OK) {
        return 0;
    }

    for (i = 0; i < K_VFS_MAX_HANDLES; i++) {
        if (!g_handle_slots[i].in_use) {
            g_handle_slots[i].in_use = 1;
            g_handle_slots[i].handle = g_next_handle++;
            g_handle_slots[i].mount_id = mount_id;
            g_handle_slots[i].object_id = object_id;
            g_handle_slots[i].offset = 0;
            g_handle_slots[i].perms = cap ? (cap->permissions & io_perms) : 0;
            return g_handle_slots[i].handle;
        }
    }

    return 0;
}

int k_vfs_close(k_vfs_handle_t handle)
{
    k_vfs_handle_slot_t *slot = k_vfs_find_handle(handle);
    k_vfs_mount_slot_t *mount;
    int rc = K_VFS_OK;

    if (!slot) {
        return K_VFS_ERR_NOT_FOUND;
    }

    mount = k_vfs_find_mount(slot->mount_id);
    if (mount && mount->ops && mount->ops->io_close) {
        rc = mount->ops->io_close(mount->driver_ctx, slot->object_id);
    }

    slot->in_use = 0;
    slot->handle = 0;
    slot->mount_id = 0;
    slot->object_id = 0;
    slot->offset = 0;
    slot->perms = 0;

    return rc;
}

int k_vfs_read(k_vfs_handle_t handle, void *buffer, uint64_t size)
{
    k_vfs_handle_slot_t *slot = k_vfs_find_handle(handle);
    k_vfs_mount_slot_t *mount;
    int rc;

    if (!slot || !buffer || size == 0) {
        return K_VFS_ERR_INVALID;
    }
    if ((slot->perms & CAPABILITY_PERM_READ) == 0) {
        return K_VFS_ERR_DENIED;
    }

    mount = k_vfs_find_mount(slot->mount_id);
    if (!mount || !mount->ops || !mount->ops->io_read) {
        return K_VFS_ERR_IO;
    }

    rc = mount->ops->io_read(mount->driver_ctx, slot->object_id, slot->offset, buffer, size);
    if (rc > 0) {
        slot->offset += (uint64_t)rc;
    }
    return rc;
}

int k_vfs_write(k_vfs_handle_t handle, const void *buffer, uint64_t size)
{
    k_vfs_handle_slot_t *slot = k_vfs_find_handle(handle);
    k_vfs_mount_slot_t *mount;
    int rc;

    if (!slot || !buffer || size == 0) {
        return K_VFS_ERR_INVALID;
    }
    if ((slot->perms & CAPABILITY_PERM_WRITE) == 0) {
        return K_VFS_ERR_DENIED;
    }

    mount = k_vfs_find_mount(slot->mount_id);
    if (!mount || !mount->ops || !mount->ops->io_write) {
        return K_VFS_ERR_IO;
    }

    rc = mount->ops->io_write(mount->driver_ctx, slot->object_id, slot->offset, buffer, size);
    if (rc > 0) {
        slot->offset += (uint64_t)rc;
    }
    return rc;
}
