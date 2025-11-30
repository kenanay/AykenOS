// kernel/include/fs.h
#ifndef AYKEN_FS_H
#define AYKEN_FS_H

#include <stdint.h>

// VFS functions
void vfs_init(void);
void *vfs_open(const char *path);
int vfs_read(void *file, void *buffer, uint64_t size);
int vfs_close(void *file);

// DevFS functions
void devfs_init(void);
void devfs_register_device(const char *name, void *ops);

#endif // AYKEN_FS_H
