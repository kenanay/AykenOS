#ifndef AYKEN_BOOT_H
#define AYKEN_BOOT_H

#include <efi.h>
#include <efilib.h>

// Kernel tarafındaki struct’ı BURADAN çekiyoruz
#include "../../kernel/include/boot_info.h"

typedef void (*ayken_kernel_entry_t)(ayken_boot_info_t *boot);

EFI_STATUS ayken_load_memory_map(EFI_SYSTEM_TABLE *SystemTable,
                                 ayken_boot_info_t *out);

EFI_STATUS ayken_load_kernel_elf(EFI_HANDLE ImageHandle,
                                 EFI_SYSTEM_TABLE *SystemTable,
                                 ayken_boot_info_t *boot_info,
                                 UINT64 *kernel_entry);

EFI_STATUS ayken_setup_paging(EFI_SYSTEM_TABLE *SystemTable,
                              ayken_boot_info_t *boot_info);

void ayken_jump_to_kernel(ayken_kernel_entry_t entry,
                          ayken_boot_info_t *boot);

#endif

