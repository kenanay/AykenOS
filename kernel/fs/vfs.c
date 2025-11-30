// kernel/fs/vfs.c
// Virtual File System stub implementation

#include <stddef.h>
#include <stdint.h>

void vfs_init(void)
{
    // TODO: VFS initialization
    // - Mount root filesystem
    // - Register filesystem drivers
}

void *vfs_open(const char *path)
{
    // TODO: Open file
    (void)path;
    return NULL;
}

int vfs_read(void *file, void *buffer, uint64_t size)
{
    // TODO: Read from file
    (void)file;
    (void)buffer;
    (void)size;
    return -1;
}

int vfs_close(void *file)
{
    // TODO: Close file
    (void)file;
    return -1;
}
