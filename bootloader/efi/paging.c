#include <efi.h>
#include <efilib.h>
#include <stdint.h>
#include "ayken_boot.h"
#include "../../kernel/include/ayken.h"

#define PAGE_SIZE            0x1000ULL
#define PAGE_ENTRIES         512ULL
#define IDENTITY_MAP_SIZE    (1ULL << 30) // 1GB

#define PTE_PRESENT          (1ULL << 0)
#define PTE_WRITABLE         (1ULL << 1)
#define PTE_USER             (1ULL << 2)
#define PTE_GLOBAL           (1ULL << 8)
#define PTE_ADDR_MASK        0x000FFFFFFFFFF000ULL

static inline uint64_t align_down(uint64_t v) { return v & ~(PAGE_SIZE - 1); }
static inline uint64_t align_up(uint64_t v)   { return (v + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1); }

static inline uint64_t table_flags_for(uint64_t leaf_flags)
{
    uint64_t f = PTE_PRESENT | PTE_WRITABLE;
    if (leaf_flags & PTE_USER)
        f |= PTE_USER;
    return f;
}

static inline uint64_t *phys_to_virt(EFI_PHYSICAL_ADDRESS phys)
{
    return (uint64_t *)(uintptr_t)phys;
}

static EFI_STATUS alloc_page_table(EFI_SYSTEM_TABLE *SystemTable,
                                   EFI_PHYSICAL_ADDRESS *out_phys)
{
    EFI_PHYSICAL_ADDRESS addr = 0;
    EFI_STATUS Status = SystemTable->BootServices->AllocatePages(
        AllocateAnyPages, EfiLoaderData, 1, &addr);
    if (EFI_ERROR(Status))
        return Status;

    SetMem((void *)(uintptr_t)addr, PAGE_SIZE, 0);
    *out_phys = addr;
    return EFI_SUCCESS;
}

static EFI_STATUS get_or_create_table(EFI_SYSTEM_TABLE *SystemTable,
                                      uint64_t *table,
                                      uint16_t idx,
                                      uint64_t leaf_flags,
                                      EFI_PHYSICAL_ADDRESS *out_phys)
{
    if (!(table[idx] & PTE_PRESENT)) {
        EFI_PHYSICAL_ADDRESS new_tbl = 0;
        EFI_STATUS Status = alloc_page_table(SystemTable, &new_tbl);
        if (EFI_ERROR(Status))
            return Status;
        table[idx] = (new_tbl & PTE_ADDR_MASK) | table_flags_for(leaf_flags);
    }

    if (out_phys)
        *out_phys = table[idx] & PTE_ADDR_MASK;
    return EFI_SUCCESS;
}

static EFI_STATUS map_page(EFI_SYSTEM_TABLE *SystemTable,
                           EFI_PHYSICAL_ADDRESS pml4_phys,
                           uint64_t virt,
                           uint64_t phys,
                           uint64_t flags)
{
    uint64_t *pml4 = phys_to_virt(pml4_phys);
    uint16_t i_pml4 = (virt >> 39) & 0x1FF;
    uint16_t i_pdpt = (virt >> 30) & 0x1FF;
    uint16_t i_pd   = (virt >> 21) & 0x1FF;
    uint16_t i_pt   = (virt >> 12) & 0x1FF;

    EFI_PHYSICAL_ADDRESS pdpt_phys = 0;
    EFI_STATUS Status = get_or_create_table(SystemTable, pml4, i_pml4, flags, &pdpt_phys);
    if (EFI_ERROR(Status)) return Status;

    uint64_t *pdpt = phys_to_virt(pdpt_phys);
    EFI_PHYSICAL_ADDRESS pd_phys = 0;
    Status = get_or_create_table(SystemTable, pdpt, i_pdpt, flags, &pd_phys);
    if (EFI_ERROR(Status)) return Status;

    uint64_t *pd = phys_to_virt(pd_phys);
    EFI_PHYSICAL_ADDRESS pt_phys = 0;
    Status = get_or_create_table(SystemTable, pd, i_pd, flags, &pt_phys);
    if (EFI_ERROR(Status)) return Status;

    uint64_t *pt = phys_to_virt(pt_phys);
    uint64_t entry_flags = flags | PTE_PRESENT;
    if (!(flags & PTE_USER))
        entry_flags |= PTE_GLOBAL;
    pt[i_pt] = (phys & PTE_ADDR_MASK) | entry_flags;
    return EFI_SUCCESS;
}

static EFI_STATUS map_range(EFI_SYSTEM_TABLE *SystemTable,
                            EFI_PHYSICAL_ADDRESS pml4_phys,
                            uint64_t virt_start,
                            uint64_t phys_start,
                            uint64_t size,
                            uint64_t flags)
{
    if (size == 0)
        return EFI_SUCCESS;

    uint64_t pages = align_up(size) / PAGE_SIZE;
    for (uint64_t page = 0; page < pages; ++page) {
        uint64_t va = virt_start + page * PAGE_SIZE;
        uint64_t pa = phys_start + page * PAGE_SIZE;
        EFI_STATUS Status = map_page(SystemTable, pml4_phys, va, pa, flags);
        if (EFI_ERROR(Status))
            return Status;
    }
    return EFI_SUCCESS;
}

static inline void load_cr3(uint64_t phys_addr)
{
    __asm__ __volatile__("mov %0, %%cr3" :: "r"(phys_addr) : "memory");
}

EFI_STATUS ayken_setup_paging(EFI_SYSTEM_TABLE *SystemTable,
                              ayken_boot_info_t *boot_info)
{
    EFI_PHYSICAL_ADDRESS pml4_phys = 0;
    EFI_STATUS Status = alloc_page_table(SystemTable, &pml4_phys);
    if (EFI_ERROR(Status)) {
        boot_info->pml4_phys = 0;
        return Status;
    }

    // 1) Identity map (at least first 1GB) so bootloader/UEFI data keeps working
    Status = map_range(SystemTable, pml4_phys, 0, 0, IDENTITY_MAP_SIZE,
                       PTE_PRESENT | PTE_WRITABLE);
    if (EFI_ERROR(Status)) {
        boot_info->pml4_phys = 0;
        return Status;
    }

    // 2) Higher-half kernel mapping
    if (boot_info->kernel_phys_end > boot_info->kernel_phys_start) {
        uint64_t k_phys_start = align_down(boot_info->kernel_phys_start);
        uint64_t k_phys_end   = align_up(boot_info->kernel_phys_end);
        uint64_t k_size       = k_phys_end - k_phys_start;

        Status = map_range(
            SystemTable,
            pml4_phys,
            KERNEL_VIRT_BASE + k_phys_start,
            k_phys_start,
            k_size,
            PTE_PRESENT | PTE_WRITABLE
        );
        if (EFI_ERROR(Status)) {
            boot_info->pml4_phys = 0;
            return Status;
        }
    }

    // 3) Framebuffer mapping (higher-half mirror)
    if (boot_info->fb_phys_addr != 0 && boot_info->fb_pitch != 0 && boot_info->fb_height != 0) {
        uint64_t fb_size = (uint64_t)boot_info->fb_pitch * (uint64_t)boot_info->fb_height;
        uint64_t fb_phys_start = align_down(boot_info->fb_phys_addr);
        uint64_t fb_phys_end   = align_up(boot_info->fb_phys_addr + fb_size);
        uint64_t fb_size_aligned = fb_phys_end - fb_phys_start;

        Status = map_range(
            SystemTable,
            pml4_phys,
            KERNEL_VIRT_BASE + fb_phys_start,
            fb_phys_start,
            fb_size_aligned,
            PTE_PRESENT | PTE_WRITABLE
        );
        if (EFI_ERROR(Status)) {
            boot_info->pml4_phys = 0;
            return Status;
        }
    }

    // 4) Load new CR3 and expose to kernel
    load_cr3(pml4_phys);
    boot_info->pml4_phys = pml4_phys;
    return EFI_SUCCESS;
}
