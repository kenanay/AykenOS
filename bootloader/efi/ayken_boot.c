#include "ayken_boot.h"
#include "elf_loader.h"

// Early debugcon output (QEMU port 0xE9)
static void debugcon_write(const char *s)
{
    if (!s) return;
    while (*s) {
        __asm__ volatile("outb %0, %1" : : "a"(*s), "Nd"(0xE9));
        s++;
    }
}

static void debugcon_hex_u64(UINT64 v)
{
    const char *hex = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        UINT8 c = (UINT8)hex[(v >> (i * 4)) & 0xF];
        __asm__ volatile("outb %0, %1" : : "a"(c), "Nd"(0xE9));
    }
}

static void debugcon_hex_u8(UINT8 v)
{
    const char *hex = "0123456789ABCDEF";
    UINT8 hi = (UINT8)hex[(v >> 4) & 0xF];
    UINT8 lo = (UINT8)hex[v & 0xF];
    __asm__ volatile("outb %0, %1" : : "a"(hi), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"(lo), "Nd"(0xE9));
}

static void debugcon_status(const char *tag, EFI_STATUS st)
{
    debugcon_write(tag);
    debugcon_write(" status=0x");
    debugcon_hex_u64((UINT64)st);
    debugcon_write("\n");
}

static void debugcon_u64(const char *tag, UINT64 v)
{
    debugcon_write(tag);
    debugcon_write("0x");
    debugcon_hex_u64(v);
    debugcon_write("\n");
}

static void debugcon_pt_entry(const char *tag, UINT64 idx, UINT64 entry)
{
    uint8_t present = (entry & 1) ? (uint8_t)'1' : (uint8_t)'0';
    debugcon_write(tag);
    debugcon_write(" idx=0x");
    debugcon_hex_u64(idx);
    debugcon_write(" val=0x");
    debugcon_hex_u64(entry);
    debugcon_write(" P=");
    __asm__ volatile("outb %0, %1" : : "a"(present), "Nd"(0xE9));
    debugcon_write("\n");
}

static void debugcon_page_walk(const char *tag, UINT64 pml4_phys, UINT64 va)
{
    const UINT64 ADDR_MASK = 0x000FFFFFFFFFF000ULL;
    UINT64 *pml4 = (UINT64 *)(uintptr_t)pml4_phys;
    UINT64 i_pml4 = (va >> 39) & 0x1FF;
    UINT64 i_pdpt = (va >> 30) & 0x1FF;
    UINT64 i_pd   = (va >> 21) & 0x1FF;
    UINT64 i_pt   = (va >> 12) & 0x1FF;

    debugcon_write(tag);
    debugcon_write(" VA=0x");
    debugcon_hex_u64(va);
    debugcon_write("\n");

    UINT64 pml4e = pml4[i_pml4];
    debugcon_pt_entry("[B][PTW][PML4E]", i_pml4, pml4e);
    if (!(pml4e & 1)) return;

    UINT64 *pdpt = (UINT64 *)(uintptr_t)(pml4e & ADDR_MASK);
    UINT64 pdpte = pdpt[i_pdpt];
    debugcon_pt_entry("[B][PTW][PDPTE]", i_pdpt, pdpte);
    if (!(pdpte & 1)) return;
    if (pdpte & (1ULL << 7)) return;

    UINT64 *pd = (UINT64 *)(uintptr_t)(pdpte & ADDR_MASK);
    UINT64 pde = pd[i_pd];
    debugcon_pt_entry("[B][PTW][PDE] ", i_pd, pde);
    if (!(pde & 1)) return;
    if (pde & (1ULL << 7)) return;

    UINT64 *pt = (UINT64 *)(uintptr_t)(pde & ADDR_MASK);
    UINT64 pte = pt[i_pt];
    debugcon_pt_entry("[B][PTW][PTE] ", i_pt, pte);
}

static int resolve_va_to_phys(UINT64 pml4_phys, UINT64 va, UINT64 *out_phys)
{
    const UINT64 ADDR_MASK = 0x000FFFFFFFFFF000ULL;
    UINT64 *pml4 = (UINT64 *)(uintptr_t)pml4_phys;
    UINT64 i_pml4 = (va >> 39) & 0x1FF;
    UINT64 i_pdpt = (va >> 30) & 0x1FF;
    UINT64 i_pd   = (va >> 21) & 0x1FF;
    UINT64 i_pt   = (va >> 12) & 0x1FF;

    UINT64 pml4e = pml4[i_pml4];
    if (!(pml4e & 1)) return 0;
    UINT64 *pdpt = (UINT64 *)(uintptr_t)(pml4e & ADDR_MASK);
    UINT64 pdpte = pdpt[i_pdpt];
    if (!(pdpte & 1)) return 0;
    if (pdpte & (1ULL << 7)) {
        UINT64 phys = (pdpte & 0x000FFFFFC0000000ULL) | (va & 0x3FFFFFFFULL);
        *out_phys = phys;
        return 1;
    }
    UINT64 *pd = (UINT64 *)(uintptr_t)(pdpte & ADDR_MASK);
    UINT64 pde = pd[i_pd];
    if (!(pde & 1)) return 0;
    if (pde & (1ULL << 7)) {
        UINT64 phys = (pde & 0x000FFFFFFFE00000ULL) | (va & 0x1FFFFFULL);
        *out_phys = phys;
        return 1;
    }
    UINT64 *pt = (UINT64 *)(uintptr_t)(pde & ADDR_MASK);
    UINT64 pte = pt[i_pt];
    if (!(pte & 1)) return 0;
    *out_phys = (pte & ADDR_MASK) | (va & 0xFFFULL);
    return 1;
}

static void debugcon_dump_phys(const char *tag, UINT64 phys, UINTN len)
{
    debugcon_write(tag);
    debugcon_write(" PA=0x");
    debugcon_hex_u64(phys);
    debugcon_write(" bytes=");
    UINT8 *p = (UINT8 *)(uintptr_t)phys;
    for (UINTN i = 0; i < len; ++i) {
        debugcon_hex_u8(p[i]);
    }
    debugcon_write("\n");
}

// Eğer gImageHandle / gST global kullanıyorsan, efi_main.c'de tanımlıdır:
extern EFI_HANDLE gImageHandle;
extern EFI_SYSTEM_TABLE *gST;

static EFI_PHYSICAL_ADDRESS g_kernel_stack_phys = 0;
static UINTN g_kernel_stack_pages = 0;

// ---------------------------------------------------------
// 1) Memory Map
// ---------------------------------------------------------
EFI_STATUS ayken_load_memory_map(EFI_SYSTEM_TABLE *SystemTable,
                                 ayken_boot_info_t *out)
{
    EFI_STATUS Status;
    debugcon_write("[B][JUMP_FUNC_ENTER]\n");
    UINTN map_size = 0;
    UINTN desc_size = 0;
    UINTN map_key = 0;
    UINT32 desc_ver = 0;
    EFI_MEMORY_DESCRIPTOR *map = NULL;

    // Boyut öğren
    Status = SystemTable->BootServices->GetMemoryMap(
        &map_size, map, &map_key, &desc_size, &desc_ver);
    if (Status != EFI_BUFFER_TOO_SMALL || desc_size == 0) {
        return Status;
    }

    map_size += desc_size * 4;

    Status = SystemTable->BootServices->AllocatePool(
        EfiLoaderData, map_size, (void**)&map);
    if (EFI_ERROR(Status)) return Status;

    Status = SystemTable->BootServices->GetMemoryMap(
        &map_size, map, &map_key, &desc_size, &desc_ver);
    if (EFI_ERROR(Status)) return Status;

    out->mem_map_addr   = (uint64_t)map;
    out->mem_map_size   = map_size;
    out->mem_desc_size  = desc_size;
    out->mem_desc_count = map_size / desc_size;
    out->uefi_map_key   = map_key;      // KRİTİK: ExitBootServices için
    out->uefi_desc_ver  = desc_ver;     // Descriptor version

    return EFI_SUCCESS;
}

// ---------------------------------------------------------
// 2) Framebuffer / GOP (Graphics Output Protocol)
// ---------------------------------------------------------
EFI_STATUS ayken_setup_framebuffer(EFI_SYSTEM_TABLE *SystemTable,
                                   ayken_boot_info_t *boot)
{
    EFI_STATUS Status;
    debugcon_write("[B][JUMP_FUNC_ENTER]\n");
    EFI_GRAPHICS_OUTPUT_PROTOCOL *gop = NULL;

    // GOP GUID
    EFI_GUID gopGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;

    Status = SystemTable->BootServices->LocateProtocol(
        &gopGuid,
        NULL,
        (void**)&gop
    );
    if (EFI_ERROR(Status) || gop == NULL) {
        // Grafik mod bulunamazsa framebuffer'ı 0 bırak
        boot->fb_phys_addr = 0;
        boot->fb_width     = 0;
        boot->fb_height    = 0;
        boot->fb_pitch     = 0;
        boot->fb_bpp       = 0;
        return Status;
    }

    // Framebuffer temel bilgileri
    boot->fb_phys_addr = gop->Mode->FrameBufferBase;
    boot->fb_width     = gop->Mode->Info->HorizontalResolution;
    boot->fb_height    = gop->Mode->Info->VerticalResolution;
    boot->fb_pitch     = gop->Mode->Info->PixelsPerScanLine * 4; // 4 byte/pixel varsayıyoruz
    boot->fb_bpp       = 32;

    // İleride PixelFormat'e göre farklı BPP/format desteği eklenebilir.

    return EFI_SUCCESS;
}

// ---------------------------------------------------------
// 3) Paging - gerçek implementasyon paging.c'de bulunur
// (Bu dosya ayken_setup_paging fonksiyonunu çağırır,
//  ancak tanımı paging.c'de 4000+ satırlık implementasyonla yapılır)

// ---------------------------------------------------------
// 4) Kernel'e zıplama (ExitBootServices ile)
// ---------------------------------------------------------
__attribute__((noinline))
void EFIAPI ayken_jump_to_kernel(ayken_kernel_entry_t entry,
                                 ayken_boot_info_t *boot)
{
    EFI_STATUS Status;

    debugcon_write("[B][JUMP_FUNC_ENTER]\n");

    // Final memmap + ExitBootServices retry loop
    if (g_kernel_stack_phys == 0) {
        EFI_PHYSICAL_ADDRESS stack_phys = (EFI_PHYSICAL_ADDRESS)((1ULL << 30) - 0x20000ULL);
        UINTN pages = 32;
        UINT64 stack_size = (UINT64)pages * 0x1000ULL;
        EFI_PHYSICAL_ADDRESS max_addr = stack_phys;
        if (boot) {
            EFI_PHYSICAL_ADDRESS boot_phys = (EFI_PHYSICAL_ADDRESS)(uintptr_t)boot;
            if (boot_phys > (stack_size + 0x2000ULL)) {
                max_addr = boot_phys - stack_size - 0x1000ULL;
            }
        }
        stack_phys = max_addr;
        EFI_STATUS S = gST->BootServices->AllocatePages(AllocateMaxAddress, EfiLoaderData, pages, &stack_phys);
        if (EFI_ERROR(S)) {
            pages = 8;
            stack_size = (UINT64)pages * 0x1000ULL;
            max_addr = (EFI_PHYSICAL_ADDRESS)((1ULL << 30) - 0x20000ULL);
            if (boot) {
                EFI_PHYSICAL_ADDRESS boot_phys = (EFI_PHYSICAL_ADDRESS)(uintptr_t)boot;
                if (boot_phys > (stack_size + 0x2000ULL)) {
                    max_addr = boot_phys - stack_size - 0x1000ULL;
                }
            }
            stack_phys = max_addr;
            S = gST->BootServices->AllocatePages(AllocateMaxAddress, EfiLoaderData, pages, &stack_phys);
        }
        if (!EFI_ERROR(S)) {
            g_kernel_stack_phys = stack_phys;
            g_kernel_stack_pages = pages;
            debugcon_u64("[B][STACK_PHYS]=", (UINT64)stack_phys);
            debugcon_u64("[B][STACK_PAGES]=", (UINT64)g_kernel_stack_pages);
        } else {
            debugcon_status("[B][STACK_ALLOC_FAIL]", S);
        }
    }

    UINTN map_key = 0;
    UINTN map_size = boot->mem_map_size;
    UINTN desc_size = boot->mem_desc_size;
    UINT32 desc_ver = boot->uefi_desc_ver;

    for (int attempt = 0; attempt < 4; ++attempt) {
        debugcon_write("[B][EBS_PRE_MEMMAP]\n");

        map_size = boot->mem_map_size;
        desc_size = boot->mem_desc_size;

        Status = gST->BootServices->GetMemoryMap(
            &map_size,
            (EFI_MEMORY_DESCRIPTOR*)(uintptr_t)boot->mem_map_addr,
            &map_key,
            &desc_size,
            &desc_ver);
        debugcon_status("[B][EBS_MEMMAP_RET]", Status);

        if (Status == EFI_BUFFER_TOO_SMALL) {
            void *new_map = NULL;
            UINTN new_size = map_size + desc_size * 8;

            Status = gST->BootServices->AllocatePool(EfiLoaderData, new_size, &new_map);
            debugcon_status("[B][EBS_MAP_REALLOC]", Status);
            if (EFI_ERROR(Status))
                break;

            boot->mem_map_addr = (uint64_t)(uintptr_t)new_map;
            boot->mem_map_size = new_size;
            continue;
        }

        if (EFI_ERROR(Status))
            break;

        boot->uefi_map_key = map_key;
        boot->mem_map_size = map_size;
        boot->mem_desc_size = desc_size;
        boot->mem_desc_count = map_size / desc_size;
        boot->uefi_desc_ver = desc_ver;

        debugcon_write("[B][EBS_CALL]\n");
        Status = gST->BootServices->ExitBootServices(gImageHandle, map_key);
        debugcon_status("[B][EBS_RET]", Status);

        if (!EFI_ERROR(Status))
            break;

        if (Status != EFI_INVALID_PARAMETER)
            break;
    }

    if (EFI_ERROR(Status)) {
        debugcon_status("[B][EBS_FAIL]", Status);
        for (;;) {
            __asm__ __volatile__("hlt");
        }
    }

    // UEFIden cikildi, artik kernelin kontrolundeyiz
    UINT64 stack_top = 0;
    if (g_kernel_stack_phys != 0) {
        stack_top = (UINT64)g_kernel_stack_phys + (UINT64)g_kernel_stack_pages * 0x1000ULL;
        stack_top &= ~0xFULL;
        debugcon_u64("[B][STACK_TOP]=", stack_top);
    } else {
        debugcon_write("[B][STACK_MISSING]\n");
    }

    debugcon_write("[B][CR3_SET]\n");
    debugcon_write("[B][JUMP_NOW]\n");
debugcon_u64("[B][JUMP_ENTRY]=", (UINT64)(uintptr_t)entry);
debugcon_u64("[B][JUMP_BOOT]=", (UINT64)(uintptr_t)boot);
debugcon_u64("[B][JUMP_CR3]=", (UINT64)boot->pml4_phys);

    debugcon_u64("[B][PML4_PHYS]=", (UINT64)boot->pml4_phys);
    debugcon_u64("[B][STACK_TOUCH]=", (stack_top != 0) ? (stack_top - 8) : 0);
    debugcon_u64("[B][ENTRY]=", (UINT64)(uintptr_t)entry);
    if (boot->pml4_phys != 0) {
        if (stack_top != 0) {
            debugcon_page_walk("[B][PTW][STACK]", boot->pml4_phys, stack_top - 8);
        }
        debugcon_page_walk("[B][PTW][ENTRY]", boot->pml4_phys, (UINT64)(uintptr_t)entry);

        UINT64 entry_pa = 0;
        if (resolve_va_to_phys(boot->pml4_phys, (UINT64)(uintptr_t)entry, &entry_pa)) {
            debugcon_dump_phys("[B][ENTRY_BYTES]", entry_pa, 16);
            debugcon_dump_phys("[B][ENTRY_BYTES+1P]", entry_pa + 0x1000, 16);
        } else {
            debugcon_write("[B][ENTRY_BYTES_FAIL]\n");
        }
    }
    UINT64 entry_val = (UINT64)(uintptr_t)entry;
    UINT64 boot_val  = (UINT64)(uintptr_t)boot;
    UINT64 cr3_val   = (UINT64)boot->pml4_phys;
    debugcon_u64("[B][WRAP_ENTRY]=", entry_val);
    debugcon_u64("[B][BOOT_KENTRY]=", (UINT64)boot->kernel_entry);
    debugcon_u64("[B][OFF_KENTRY]=", (UINT64)__builtin_offsetof(ayken_boot_info_t, kernel_entry));
    debugcon_u64("[B][OFF_PML4]=", (UINT64)__builtin_offsetof(ayken_boot_info_t, pml4_phys));
    debugcon_u64("[B][WRAP_BOOT]=", boot_val);
    debugcon_u64("[B][WRAP_STK]=", stack_top);
    debugcon_u64("[B][WRAP_CR3]=", cr3_val);
    debugcon_u64("[B][CALL_ENTRY]=", entry_val);
    debugcon_u64("[B][CALL_BOOT]=", boot_val);
    debugcon_u64("[B][CALL_STK]=", stack_top);
    debugcon_u64("[B][CALL_CR3]=", cr3_val);
    g_handoff_entry = entry_val;
    g_handoff_boot = boot_val;
    g_handoff_stack = stack_top;
    g_handoff_cr3 = cr3_val;
    
    // Switch to kernel page tables before jumping to kernel
    // This ensures we're running on the correct page tables when we jump
    __asm__ volatile("mov %0, %%cr3" : : "r"(cr3_val) : "memory");
    
    ayken_jump_to_kernel_raw();
    __builtin_unreachable();
}
