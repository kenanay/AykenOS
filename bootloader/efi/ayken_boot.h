#ifndef AYKEN_BOOT_H
#define AYKEN_BOOT_H

#include <efi.h>
#include <efilib.h>

// Kernel tarafındaki struct’ı BURADAN çekiyoruz
#include "../../kernel/include/boot_info.h"
#include "../../kernel/include/boot_flags.h"

typedef void (*ayken_kernel_entry_t)(ayken_boot_info_t *boot);

EFI_STATUS ayken_load_memory_map(EFI_SYSTEM_TABLE *SystemTable,
                                 ayken_boot_info_t *out);

EFI_STATUS ayken_setup_framebuffer(EFI_SYSTEM_TABLE *SystemTable,
                                   ayken_boot_info_t *boot);

EFI_STATUS ayken_load_kernel_elf(EFI_HANDLE ImageHandle,
                                 EFI_SYSTEM_TABLE *SystemTable,
                                 ayken_boot_info_t *boot_info,
                                 UINT64 *kernel_entry);

EFI_STATUS ayken_setup_paging(EFI_SYSTEM_TABLE *SystemTable,
                              ayken_boot_info_t *boot_info);

EFI_STATUS ayken_map_identity_range(EFI_SYSTEM_TABLE *SystemTable,
                                    uint64_t pml4_phys,
                                    uint64_t phys_start,
                                    uint64_t size);


void ayken_load_cr3(uint64_t phys_addr);
void EFIAPI ayken_jump_to_kernel(ayken_kernel_entry_t entry,
                                 ayken_boot_info_t *boot);

__attribute__((ms_abi))
void ayken_jump_to_kernel_raw(void);

extern volatile uint64_t g_handoff_entry;
extern volatile uint64_t g_handoff_boot;
extern volatile uint64_t g_handoff_stack;
extern volatile uint64_t g_handoff_cr3;

#endif
