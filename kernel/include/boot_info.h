#ifndef AYKEN_BOOT_INFO_H
#define AYKEN_BOOT_INFO_H

#include <stdint.h>

// =============================================================
// Bootloader → Kernel bilgi aktarım yapısı
// AykenOS kernel bu struct ile boot edilir.
// =============================================================

typedef struct {

    // ---------------------------------------------------------
    // 1) MEMORY MAP (UEFI → kernel)
    // ---------------------------------------------------------
    uint64_t mem_map_addr;        // Memory map fiziksel adresi
    uint64_t mem_map_size;        // Map toplam byte uzunluğu
    uint64_t mem_desc_size;       // Descriptor boyutu
    uint64_t mem_desc_count;      // Descriptor adedi
    uint64_t uefi_map_key;        // UEFI memory map key (ExitBootServices için)
    uint32_t uefi_desc_ver;       // UEFI descriptor version

    // ---------------------------------------------------------
    // 2) KERNEL ELF FİZİKSEL ADRES ARALIĞI
    // Bootloader ELF yüklerken bu alanları hesaplar.
    // ---------------------------------------------------------
    uint64_t kernel_phys_start;   // Kernel ELF başlangıç fiziksel adres
    uint64_t kernel_phys_end;     // Kernel ELF bitiş fiziksel adres

    // ---------------------------------------------------------
    // 3) PML4 ROOT (paging_init için)
    // Bootloader paging hazırlıyorsa burayı doldurur.
    // (Şimdilik 0 bırakılabilir)
    // ---------------------------------------------------------
    uint64_t pml4_phys;

    // ---------------------------------------------------------
    // 4) FRAMEBUFFER (grafik ekran) – UEFI GOP tarafından doldurulur
    // Kernel, fb_console ile bu bilgileri kullanarak ekrana yazı basar.
    // ---------------------------------------------------------
    uint64_t fb_phys_addr;        // Framebuffer fiziksel adresi
    uint32_t fb_width;            // Çözünürlük yatay
    uint32_t fb_height;           // Çözünürlük dikey
    uint32_t fb_pitch;            // Her satırın byte uzunluğu
    uint32_t fb_bpp;              // Bits per pixel (genelde 32)

    // ---------------------------------------------------------
    // 5) İLERİDE: ACPI, RSDP, SMP (APIC), NVRAM, vb.
    // ---------------------------------------------------------

} ayken_boot_info_t;

#endif // AYKEN_BOOT_INFO_H
