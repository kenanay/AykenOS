#ifndef AYKEN_BOOT_INFO_H
#define AYKEN_BOOT_INFO_H

#include <stdint.h>

typedef struct {
    uint32_t abi_version;
    uint32_t flags;
    uint64_t mem_map_addr;
    uint64_t mem_map_size;
    uint64_t mem_desc_size;
    uint64_t mem_desc_count;
    uint64_t uefi_map_key;
    uint32_t uefi_desc_ver;
    uint64_t kernel_phys_start;
    uint64_t kernel_phys_end;
    uint64_t kernel_virt_base;
    uint64_t kernel_phys_base;
    uint64_t kernel_map_size;
    uint64_t kernel_entry;
    uint64_t pml4_phys;
    uint64_t fb_phys_addr;
    uint32_t fb_width;
    uint32_t fb_height;
    uint32_t fb_pitch;
    uint32_t fb_bpp;
} ayken_boot_info_t;

_Static_assert(sizeof(ayken_boot_info_t) == 136, "ayken_boot_info_t must be 136 bytes");

#endif
