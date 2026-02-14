#ifndef KERNEL_VFS_H
#define KERNEL_VFS_H

/*
 * Compatibility shim: Ring0 VFS surface is defined in kernel/include/vfs_mech.h.
 * Keep this header for legacy include paths while canonicalizing mechanism APIs.
 */
#include "../include/vfs_mech.h"

#endif /* KERNEL_VFS_H */
