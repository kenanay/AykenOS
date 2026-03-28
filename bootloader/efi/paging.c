#include <efi.h>
#include <efilib.h>
#include <stdint.h>
#include "ayken_boot.h"
#include "../../shared/abi/kernel_limits.h"
#include "elf_loader.h"

#define PAGE_SIZE            0x1000ULL
#define PAGE_ENTRIES         512ULL
#define IDENTITY_MAP_SIZE    AYKEN_IDENTITY_MAP_SIZE

#define PTE_PRESENT          (1ULL << 0)
#define PTE_WRITABLE         (1ULL << 1)
#define PTE_USER             (1ULL << 2)
#define PTE_GLOBAL           (1ULL << 8)
#define PTE_ADDR_MASK        0x000FFFFFFFFFF000ULL

// debugcon (QEMU 0xE9)
static void debugcon_write(const char *s)
{
    while (s && *s) {
        __asm__ volatile("outb %0, %1" : : "a"(*s), "Nd"(0xE9));
        s++;
    }
}

static void debugcon_hex_u64(uint64_t v)
{
    const char *hex = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        uint8_t c = (uint8_t)hex[(v >> (i * 4)) & 0xF];
        __asm__ volatile("outb %0, %1" : : "a"(c), "Nd"(0xE9));
    }
}

static void debugcon_u64(const char *tag, uint64_t v)
{
    debugcon_write(tag);
    debugcon_write("0x");
    debugcon_hex_u64(v);
    debugcon_write("\n");
}

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

static void debugcon_walk_va(EFI_PHYSICAL_ADDRESS pml4_phys, uint64_t va)
{
    uint64_t *pml4 = phys_to_virt(pml4_phys);
    uint16_t i_pml4 = (va >> 39) & 0x1FF;
    uint16_t i_pdpt = (va >> 30) & 0x1FF;
    uint16_t i_pd   = (va >> 21) & 0x1FF;
    uint16_t i_pt   = (va >> 12) & 0x1FF;

    uint64_t pml4e = pml4[i_pml4];
    if (!(pml4e & PTE_PRESENT)) return;
    uint64_t *pdpt = phys_to_virt(pml4e & PTE_ADDR_MASK);
    uint64_t pdpte = pdpt[i_pdpt];
    if (!(pdpte & PTE_PRESENT)) return;
    if (pdpte & (1ULL << 7)) return;
    uint64_t *pd = phys_to_virt(pdpte & PTE_ADDR_MASK);
    uint64_t pde = pd[i_pd];
    if (!(pde & PTE_PRESENT)) return;
    if (pde & (1ULL << 7)) return;
    uint64_t *pt = phys_to_virt(pde & PTE_ADDR_MASK);
    uint64_t pte = pt[i_pt];

    debugcon_u64("[B][PHDR0_PTE]=", pte);
}

static EFI_STATUS alloc_page_table(EFI_SYSTEM_TABLE *SystemTable,
                                   EFI_PHYSICAL_ADDRESS *out_phys)
{
    // Keep page tables below identity-mapped region to avoid CR3 faults.
    EFI_PHYSICAL_ADDRESS addr = IDENTITY_MAP_SIZE - PAGE_SIZE;
    EFI_STATUS Status = SystemTable->BootServices->AllocatePages(
        AllocateMaxAddress, EfiLoaderData, 1, &addr);
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
    if (virt == 0xFFFFFFFF80000000ULL) {
        debugcon_u64("[B][MAP_ENTRY_PA]=", phys);
    }
    pt[i_pt] = (phys & PTE_ADDR_MASK) | entry_flags;
    if (virt == 0xFFFFFFFF80000000ULL) {
        debugcon_u64("[B][MAP_ENTRY_PTE]=", pt[i_pt]);
    }
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

static EFI_STATUS map_kernel_heap_window(EFI_SYSTEM_TABLE *SystemTable,
                                         EFI_PHYSICAL_ADDRESS pml4_phys)
{
    EFI_BOOT_SERVICES *BS = SystemTable->BootServices;
    uint64_t heap_pages = align_up(AYKEN_KHEAP_INITIAL_SIZE) / PAGE_SIZE;

    for (uint64_t page = 0; page < heap_pages; ++page) {
        EFI_PHYSICAL_ADDRESS heap_phys = 0xFFFFFFFFULL;
        EFI_STATUS Status = uefi_call_wrapper(
            BS->AllocatePages, 4, AllocateMaxAddress, EfiLoaderData, 1, &heap_phys);
        if (EFI_ERROR(Status)) {
            return Status;
        }

        SetMem((void *)(uintptr_t)heap_phys, PAGE_SIZE, 0);
        Status = map_page(SystemTable,
                          pml4_phys,
                          AYKEN_KHEAP_START + (page * PAGE_SIZE),
                          (uint64_t)heap_phys,
                          PTE_PRESENT | PTE_WRITABLE);
        if (EFI_ERROR(Status)) {
            return Status;
        }
    }

    return EFI_SUCCESS;
}

void ayken_load_cr3(uint64_t phys_addr)
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

    // 2) Higher-half kernel mapping (ELF PT_LOAD segments)
    UINT64 ph_count = ayken_get_phdr_count();
    const ayken_phdr_t *phdrs = ayken_get_phdrs();
    UINT64 load_base = ayken_get_load_base_phys();
    UINT64 min_vaddr = ayken_get_min_vaddr();

    if (ph_count > 0 && phdrs && load_base != 0) {
        debugcon_u64("[B][PHDR_LOAD_BASE]=", load_base);
        debugcon_u64("[B][PHDR_MIN_VA]=", min_vaddr);
        debugcon_u64("[B][PHDR_COUNT]=", ph_count);
        for (UINT64 i = 0; i < ph_count; ++i) {
            UINT64 seg_vaddr = phdrs[i].vaddr;
            UINT64 seg_memsz = phdrs[i].memsz;
            if (seg_memsz == 0)
                continue;

            UINT64 seg_va0 = align_down(seg_vaddr);
            UINT64 seg_delta = seg_vaddr - min_vaddr;
            UINT64 seg_pa0 = align_down(load_base + seg_delta);
            UINT64 va_bias = seg_vaddr - seg_va0;
            UINT64 map_size = align_up(seg_memsz + va_bias);
            if (i == 0) {
                debugcon_u64("[B][PHDR0_VA]=", seg_vaddr);
                debugcon_u64("[B][PHDR0_PA]=", seg_pa0);
            }

            Status = map_range(
                SystemTable,
                pml4_phys,
                seg_va0,
                seg_pa0,
                map_size,
                PTE_PRESENT | PTE_WRITABLE
            );
            if (EFI_ERROR(Status)) {
                boot_info->pml4_phys = 0;
                return Status;
            }
            if (i == 0) {
                debugcon_walk_va(pml4_phys, seg_vaddr);
            }
        }
    } else if (boot_info->kernel_map_size != 0) {
        // Fallback to legacy single-range mapping
        uint64_t k_virt_base = align_down(boot_info->kernel_virt_base);
        uint64_t k_phys_base = align_down(boot_info->kernel_phys_base);
        uint64_t k_size      = align_up(boot_info->kernel_map_size);

        Status = map_range(
            SystemTable,
            pml4_phys,
            k_virt_base,
            k_phys_base,
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

    Status = map_kernel_heap_window(SystemTable, pml4_phys);
    if (EFI_ERROR(Status)) {
        boot_info->pml4_phys = 0;
        return Status;
    }

    // 4) Expose PML4 to kernel (CR3 is loaded after ExitBootServices)
    boot_info->pml4_phys = pml4_phys;
    return EFI_SUCCESS;
}

EFI_STATUS ayken_map_identity_range(EFI_SYSTEM_TABLE *SystemTable,
                                    uint64_t pml4_phys,
                                    uint64_t phys_start,
                                    uint64_t size)
{
    if (pml4_phys == 0 || size == 0)
        return EFI_INVALID_PARAMETER;

    uint64_t phys_aligned = align_down(phys_start);
    uint64_t size_aligned = align_up((phys_start - phys_aligned) + size);
    return map_range(SystemTable,
                     (EFI_PHYSICAL_ADDRESS)pml4_phys,
                     phys_aligned,
                     phys_aligned,
                     size_aligned,
                     PTE_PRESENT | PTE_WRITABLE);
}
