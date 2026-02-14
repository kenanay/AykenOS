// kernel/fs/devfs.h - Ring0 DevFS mechanism surface
#ifndef __KERNEL_FS_DEVFS_H
#define __KERNEL_FS_DEVFS_H

// Ring0 exposes only mechanism-level init hook.
int devfs_init(void);

#endif // __KERNEL_FS_DEVFS_H
