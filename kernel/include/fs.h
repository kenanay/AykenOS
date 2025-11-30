// kernel/include/fs.h
#ifndef AYKEN_FS_H
#define AYKEN_FS_H

#include <stdint.h>
#include <stddef.h>

typedef enum {
    VFS_MODE_READ = 1
} vfs_mode_t;

typedef enum {
    VFS_SEEK_SET = 0,
    VFS_SEEK_CUR = 1,
    VFS_SEEK_END = 2
} vfs_seek_whence_t;

typedef struct vfs_file vfs_file_t;

// VFS functions
void vfs_init(void);
vfs_file_t *vfs_open(const char *path, vfs_mode_t mode);
int vfs_read(vfs_file_t *file, void *buffer, uint64_t size);
int vfs_seek(vfs_file_t *file, int64_t offset, vfs_seek_whence_t whence);
int vfs_close(vfs_file_t *file);

// DevFS functions
void devfs_init(void);
void devfs_register_device(const char *name, void *ops);

#endif // AYKEN_FS_H
