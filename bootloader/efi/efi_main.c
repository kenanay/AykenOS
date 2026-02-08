#include <efi.h>
#include "efilib.h"
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

static void debugcon_status(const char *tag, EFI_STATUS st)
{
    debugcon_write(tag);
    debugcon_write(" status=0x");
    debugcon_hex_u64((UINT64)st);
    debugcon_write("\n");
}

EFI_HANDLE gImageHandle;
EFI_SYSTEM_TABLE *gST;

EFI_STATUS EFIAPI efi_main(EFI_HANDLE ImageHandle,
                           EFI_SYSTEM_TABLE *SystemTable)
{
    gImageHandle = ImageHandle;
    gST = SystemTable;

    InitializeLib(ImageHandle, SystemTable);

    debugcon_write("[B][UEFI_BOOT_START] efi_main entry\n");
    debugcon_write("[B][INIT_LIB_OK]\n");
    Print(L"AykenOS UEFI Bootloader\n");

    /* Tüm alanları deterministik başlatmak için sıfırla */
    ayken_boot_info_t boot = {0};
    EFI_STATUS Status;

    /* ABI ve flags başlat */
    boot.abi_version = AYKEN_BOOT_ABI_VERSION;
    boot.flags = 0;

    // 1) Memory map
    Status = ayken_load_memory_map(SystemTable, &boot);
    debugcon_status("[B][MEMMAP]", Status);
    if (EFI_ERROR(Status)) {
        Print(L"[ERR] Memory map alinamadi!\n");
        return Status;
    }

    // 2) Framebuffer / GOP
    Status = ayken_setup_framebuffer(SystemTable, &boot);
    debugcon_status("[B][GOP]", Status);
    if (EFI_ERROR(Status)) {
        Print(L"[WARN] GOP bulunamadi, framebuffer yok.\n");
    } else {
        Print(L"[OK] Framebuffer: %ux%u\n",
              boot.fb_width, boot.fb_height);
        /* Framebuffer geçerli ise flag setle */
        boot.flags |= AYKEN_BOOT_FLAG_FB_VALID;
    }

    // 3) Kernel ELF'i yukle
    UINT64 kernel_entry = 0;
    Status = elf_load_kernel(
        ImageHandle, SystemTable,
        L"kernel.elf",
        &boot, &kernel_entry
    );
    debugcon_status("[B][ELF_LOAD]", Status);
    if (EFI_ERROR(Status)) {
        debugcon_write("[B][KERNEL_ELF_LOAD_FAIL]\n");
        Print(L"[ERR] Kernel ELF yukleme hatasi!\n");
        return Status;
    }

    debugcon_write("[B][KERNEL_ELF_LOADED]\n");
    Print(L"[OK] Kernel yuklendi. Entry = 0x%lx\n", kernel_entry);

    // 4) Paging: higher-half PML4 kur
    Status = ayken_setup_paging(SystemTable, &boot);
    debugcon_status("[B][PAGING]", Status);
    if (EFI_ERROR(Status)) {
        Print(L"[WARN] Paging setup basarisiz, pml4_phys=0.\n");
    }
    else {
        /* Paging başarılı ise pml4_phys ayarlandı — flag setle */
        if (boot.pml4_phys != 0)
            boot.flags |= AYKEN_BOOT_FLAG_PAGING_READY;
    }

    // 4.1) Ensure bootloader image is identity-mapped in new page tables
    EFI_LOADED_IMAGE_PROTOCOL *loaded = NULL;
    EFI_GUID loaded_image_guid = EFI_LOADED_IMAGE_PROTOCOL_GUID;
    Status = SystemTable->BootServices->HandleProtocol(
        ImageHandle, &loaded_image_guid, (void**)&loaded);
    debugcon_status("[B][LOADED_IMAGE]", Status);
    if (!EFI_ERROR(Status) && loaded != NULL && boot.pml4_phys != 0) {
        UINT64 img_base = (UINT64)(uintptr_t)loaded->ImageBase;
        UINT64 img_size = (UINT64)loaded->ImageSize;
        UINT64 pe_size = 0;
        UINT64 eff_size = img_size;
        UINT8 *base8 = (UINT8 *)(uintptr_t)img_base;

        /* Read PE OptionalHeader.SizeOfImage (PE32+) */
        if (base8[0] == 'M' && base8[1] == 'Z') {
            UINT32 e_lfanew = *(UINT32 *)(base8 + 0x3C);
            if (e_lfanew >= 0x40) {
                UINT8 *pe = base8 + e_lfanew;
                if (pe[0] == 'P' && pe[1] == 'E' && pe[2] == 0 && pe[3] == 0) {
                    UINT8 *opt = pe + 4 + 20; /* signature + COFF header */
                    UINT16 magic = *(UINT16 *)opt;
                    if (magic == 0x20B) { /* PE32+ */
                        pe_size = *(UINT32 *)(opt + 56);
                        if (pe_size > eff_size)
                            eff_size = pe_size;
                    }
                }
            }
        }

        UINT64 map_start = img_base & ~0xFFFULL;
        UINT64 map_end = (img_base + eff_size + 0xFFFULL) & ~0xFFFULL;
        UINT64 map_size = (map_end > map_start) ? (map_end - map_start) : 0;
        debugcon_write("[B][IMG_BASE]=0x");
        debugcon_hex_u64(img_base);
        debugcon_write("\n");
        debugcon_write("[B][IMG_SIZE]=0x");
        debugcon_hex_u64(img_size);
        debugcon_write("\n");
        debugcon_write("[B][PE_SIZEOFIMAGE]=0x");
        debugcon_hex_u64(pe_size);
        debugcon_write("\n");
        debugcon_write("[B][IMG_EFF_SIZE]=0x");
        debugcon_hex_u64(eff_size);
        debugcon_write("\n");

        debugcon_write("[B][IMG_MAP_START]=0x");
        debugcon_hex_u64(map_start);
        debugcon_write("\n");
        debugcon_write("[B][IMG_MAP_SIZE]=0x");
        debugcon_hex_u64(map_size);
        debugcon_write("\n");

        Status = ayken_map_identity_range(SystemTable, boot.pml4_phys, map_start, map_size);
        debugcon_status("[B][MAP_IMG]", Status);
    }

    // 5) ACPI / SMP (APIC) tespiti — ConfigurationTable içindeki RSDP'yi tara
    for (UINTN i = 0; i < SystemTable->NumberOfTableEntries; ++i) {
        void *vend = SystemTable->ConfigurationTable[i].VendorTable;
        if (!vend) continue;
        if (CompareGuid(&SystemTable->ConfigurationTable[i].VendorGuid, &AcpiTableGuid) == 0 ||
            (vend && CompareMem(vend, "RSD PTR ", 8) == 0)) {
            /* RSDP bulundu */
            boot.flags |= AYKEN_BOOT_FLAG_ACPI_PRESENT;

            /* RSDP içinden XSDT/RSDT adresini alıp APIC tablosunu ara */
            uint8_t *rsdp = (uint8_t*)vend;
            uint8_t revision = rsdp[15];
            uint64_t xsdt_addr = 0;
            uint32_t rsdt_addr = 0;
            if (revision >= 2) {
                xsdt_addr = *(uint64_t*)(rsdp + 24);
            }
            rsdt_addr = *(uint32_t*)(rsdp + 16);

            /* Helper: ACPI table header */
            typedef struct {
                char sig[4];
                uint32_t length;
                uint8_t rev;
                uint8_t checksum;
                char oemid[6];
                char oemtableid[8];
                uint32_t oemrev;
                uint32_t creatorid;
                uint32_t createrev;
            } acpi_hdr_t;

            if (xsdt_addr) {
                acpi_hdr_t *xsdt = (acpi_hdr_t*)(uintptr_t)xsdt_addr;
                if (xsdt && xsdt->length > sizeof(acpi_hdr_t)) {
                    uint32_t entries = (xsdt->length - sizeof(acpi_hdr_t)) / 8;
                    uint64_t *ents = (uint64_t*)((char*)xsdt + sizeof(acpi_hdr_t));
                    for (uint32_t e = 0; e < entries; ++e) {
                        acpi_hdr_t *h = (acpi_hdr_t*)(uintptr_t)ents[e];
                        if (h && CompareMem(h->sig, "APIC", 4) == 0) {
                            boot.flags |= AYKEN_BOOT_FLAG_SMP_AVAILABLE;
                            break;
                        }
                    }
                }
            } else if (rsdt_addr) {
                acpi_hdr_t *rsdt = (acpi_hdr_t*)(uintptr_t)rsdt_addr;
                if (rsdt && rsdt->length > sizeof(acpi_hdr_t)) {
                    uint32_t entries = (rsdt->length - sizeof(acpi_hdr_t)) / 4;
                    uint32_t *ents = (uint32_t*)((char*)rsdt + sizeof(acpi_hdr_t));
                    for (uint32_t e = 0; e < entries; ++e) {
                        acpi_hdr_t *h = (acpi_hdr_t*)(uintptr_t)ents[e];
                        if (h && CompareMem(h->sig, "APIC", 4) == 0) {
                            boot.flags |= AYKEN_BOOT_FLAG_SMP_AVAILABLE;
                            break;
                        }
                    }
                }
            }

            break; /* bir RSDP bulmak yeterli */
        }
    }

    // 6) Refresh memory map before ExitBootServices
    Status = ayken_load_memory_map(SystemTable, &boot);
    debugcon_status("[B][MEMMAP_FINAL]", Status);
    if (EFI_ERROR(Status)) {
        debugcon_write("[B][MEMMAP_FINAL_FAIL]\n");
        return Status;
    }

    // 7) Kernel'e zipla
    debugcon_write("[B][ABOUT_TO_JUMP_KERNEL]\n");

    // Move boot_info into low (identity-mapped) memory before jump
    EFI_PHYSICAL_ADDRESS boot_phys = (EFI_PHYSICAL_ADDRESS)((1ULL << 30) - 0x1000ULL);
    Status = SystemTable->BootServices->AllocatePages(AllocateMaxAddress, EfiLoaderData, 1, &boot_phys);
    if (EFI_ERROR(Status)) {
        Status = SystemTable->BootServices->AllocatePages(AllocateAnyPages, EfiLoaderData, 1, &boot_phys);
    }
    if (!EFI_ERROR(Status)) {
        SystemTable->BootServices->CopyMem((void*)(UINTN)boot_phys, &boot, sizeof(boot));
        debugcon_write("[B][BOOTINFO_PHYS]=0x");
        debugcon_hex_u64((UINT64)boot_phys);
        debugcon_write("\n");
        ayken_boot_info_t *boot_ptr = (ayken_boot_info_t*)(UINTN)boot_phys;
        ayken_jump_to_kernel((ayken_kernel_entry_t)kernel_entry, boot_ptr);
        debugcon_write("[B][JUMP_RETURNED]\n");
        return EFI_SUCCESS;
    }

    // Fallback: jump with stack-allocated boot_info
    ayken_jump_to_kernel((ayken_kernel_entry_t)kernel_entry, &boot);
    debugcon_write("[B][JUMP_RETURNED]\n");

    return EFI_SUCCESS;
}
