#include <efi.h>
#include <efilib.h>
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

    ayken_boot_info_t boot;
    EFI_STATUS Status;

    // 1) Memory map
    Status = ayken_load_memory_map(SystemTable, &boot);
    if (EFI_ERROR(Status)) {
        Print(L"[ERR] Memory map alınamadı!\n");
        return Status;
    }

    // 2) Framebuffer / GOP
    Status = ayken_setup_framebuffer(SystemTable, &boot);
    if (EFI_ERROR(Status)) {
        Print(L"[WARN] GOP bulunamadı, framebuffer yok.\n");
    } else {
        Print(L"[OK] Framebuffer: %ux%u\n",
              boot.fb_width, boot.fb_height);
    }

    // 3) Paging (şimdilik basit)
    Status = ayken_setup_paging(SystemTable, &boot);
    if (EFI_ERROR(Status)) {
        Print(L"[WARN] Paging setup başarısız, pml4_phys=0.\n");
    }

    // 4) Kernel ELF'i yükle
    UINT64 kernel_entry = 0;
    Status = elf_load_kernel(
        ImageHandle, SystemTable,
        L"kernel.elf",
        &boot, &kernel_entry
    );
    if (EFI_ERROR(Status)) {
        Print(L"[ERR] Kernel ELF yükleme hatası!\n");
        return Status;
    }

    Print(L"[OK] Kernel yüklendi. Entry = 0x%lx\n", kernel_entry);

    // 5) Kernel'e zıpla
    ayken_jump_to_kernel((ayken_kernel_entry_t)kernel_entry, &boot);

    return EFI_SUCCESS;
}
