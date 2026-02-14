#include "devfs.h"
#include "../drivers/console/fb_console.h"

int devfs_init(void)
{
    fb_print("[kernel/devfs] Ring0 mechanism active (Ring3 policy-only)\n");
    return 0;
}
