#include <efi.h>
#include "efilib.h"
#include "ayken_boot.h"
#include "elf_loader.h"

EFI_HANDLE gImageHandle;
EFI_SYSTEM_TABLE *gST;

EFI_STATUS EFIAPI efi_main(EFI_HANDLE ImageHandle,
                           EFI_SYSTEM_TABLE *SystemTable)
{
    gImageHandle = ImageHandle;
    gST = SystemTable;

    InitializeLib(ImageHandle, SystemTable);

    Print(L"AykenOS UEFI Bootloader\n");

    /* Tüm alanları deterministik başlatmak için sıfırla */
    ayken_boot_info_t boot = {0};
    EFI_STATUS Status;

    /* ABI ve flags başlat */
    boot.abi_version = AYKEN_BOOT_ABI_VERSION;
    boot.flags = 0;

    // 1) Memory map
    Status = ayken_load_memory_map(SystemTable, &boot);
    if (EFI_ERROR(Status)) {
        Print(L"[ERR] Memory map alinamadi!\n");
        return Status;
    }

    // 2) Framebuffer / GOP
    Status = ayken_setup_framebuffer(SystemTable, &boot);
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
    if (EFI_ERROR(Status)) {
        Print(L"[ERR] Kernel ELF yukleme hatasi!\n");
        return Status;
    }

    Print(L"[OK] Kernel yuklendi. Entry = 0x%lx\n", kernel_entry);

    // 4) Paging: higher-half PML4 kur
    Status = ayken_setup_paging(SystemTable, &boot);
    if (EFI_ERROR(Status)) {
        Print(L"[WARN] Paging setup basarisiz, pml4_phys=0.\n");
    }
    else {
        /* Paging başarılı ise pml4_phys ayarlandı — flag setle */
        if (boot.pml4_phys != 0)
            boot.flags |= AYKEN_BOOT_FLAG_PAGING_READY;
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

    // 6) Kernel'e zipla
    ayken_jump_to_kernel((ayken_kernel_entry_t)kernel_entry, &boot);

    return EFI_SUCCESS;
}
