// kernel/fs/devfs.c
// Device Filesystem stub implementation

#include <stddef.h>

void devfs_init(void)
{
    // TODO: DevFS initialization
    // - Create /dev directory
    // - Register device nodes
}

void devfs_register_device(const char *name, void *ops)
{
    // TODO: Register device
    (void)name;
    (void)ops;
}
