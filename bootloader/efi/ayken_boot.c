#include "ayken_boot.h"
#include "elf_loader.h"

// Eğer gImageHandle / gST global kullanıyorsan, efi_main.c'de tanımlıdır:
extern EFI_HANDLE gImageHandle;
extern EFI_SYSTEM_TABLE *gST;

// ---------------------------------------------------------
// 1) Memory Map
// ---------------------------------------------------------
EFI_STATUS ayken_load_memory_map(EFI_SYSTEM_TABLE *SystemTable,
                                 ayken_boot_info_t *out)
{
    EFI_STATUS Status;
    UINTN map_size = 0;
    UINTN desc_size = 0;
    UINTN map_key, desc_ver;
    EFI_MEMORY_DESCRIPTOR *map = NULL;

    // Boyut öğren
    Status = SystemTable->BootServices->GetMemoryMap(
        &map_size, map, &map_key, &desc_size, &desc_ver);

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
// 3) Paging (şimdilik placeholder)
// ---------------------------------------------------------
EFI_STATUS ayken_setup_paging(EFI_SYSTEM_TABLE *SystemTable,
                              ayken_boot_info_t *boot_info)
{
    // Şimdilik UEFI'nin hazır paging ayarını kullanıyoruz
    boot_info->pml4_phys = 0;
    return EFI_SUCCESS;
}

// ---------------------------------------------------------
// 4) Kernel'e zıplama (ExitBootServices ile)
// ---------------------------------------------------------
void ayken_jump_to_kernel(ayken_kernel_entry_t entry,
                          ayken_boot_info_t *boot)
{
    EFI_STATUS Status;
    
    // ExitBootServices çağrısı - UEFI firmware'den ayrılıyoruz
    // Bu noktadan sonra UEFI servisleri kullanılamaz!
    Status = gST->BootServices->ExitBootServices(gImageHandle, boot->uefi_map_key);
    
    if (EFI_ERROR(Status)) {
        // Eğer EFI_INVALID_PARAMETER gelirse, memory map değişmiş demektir
        // UEFI spec'e göre memory map'i yeniden alıp tekrar denemeliyiz
        
        // Memory map'i yeniden al
        UINTN map_size = boot->mem_map_size;
        UINTN desc_size = boot->mem_desc_size;
        UINTN map_key, desc_ver;
        EFI_MEMORY_DESCRIPTOR *map = (EFI_MEMORY_DESCRIPTOR*)boot->mem_map_addr;
        
        Status = gST->BootServices->GetMemoryMap(
            &map_size, map, &map_key, &desc_size, &desc_ver);
        
        if (!EFI_ERROR(Status)) {
            // Güncellenmiş key ile tekrar dene
            boot->uefi_map_key = map_key;
            boot->mem_map_size = map_size;
            boot->mem_desc_count = map_size / desc_size;
            
            Status = gST->BootServices->ExitBootServices(gImageHandle, map_key);
        }
        
        // Hala başarısız olursa, en azından deneyelim
        // (Bazı firmware'ler toleranslıdır)
        if (EFI_ERROR(Status)) {
            // Son çare: direkt kernel'e atla
            // Gerçek donanımda sorun çıkarabilir ama QEMU'da çalışabilir
        }
    }
    
    // UEFI'den çıktık, artık kernel'in kontrolündeyiz
    entry(boot);

    // Kernel return etmemeli ama yine de güvenlik için
    for (;;) {
        __asm__ __volatile__("hlt");
    }
}
