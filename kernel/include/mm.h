#ifndef AYKEN_MM_H
#define AYKEN_MM_H

#include <stdint.h>
#include "boot_info.h"
#include "ayken.h"

// -----------------------------------------------------------------------------
// FRAME ve FİZİKSEL BELLEK SINIRLARI
// -----------------------------------------------------------------------------

// 4KB frame standardı
#define AYKEN_FRAME_SIZE            4096ULL

// İleride 128GB RAM’e kadar çıkabilir; fakat şimdilik OS için 4GB sınır yeterli.
#define AYKEN_MAX_PHYS_MEM          (4ULL * 1024ULL * 1024ULL * 1024ULL)
#define AYKEN_MAX_FRAMES            (AYKEN_MAX_PHYS_MEM / AYKEN_FRAME_SIZE)

// Page table entry flag bits (x86_64)
#define AYKEN_PTE_PRESENT         (1ULL << 0)
#define AYKEN_PTE_WRITABLE        (1ULL << 1)
#define AYKEN_PTE_USER            (1ULL << 2)
#define AYKEN_PTE_GLOBAL          (1ULL << 8)
#define AYKEN_PTE_ADDR_MASK       0x000FFFFFFFFFF000ULL


// -----------------------------------------------------------------------------
// FİZİKSEL BELLEK BAŞLATMA
// -----------------------------------------------------------------------------

/**
 * EFI memory map üzerinden kullanılabilir fiziksel RAM’i analiz eder.
 * Frame bitmap/tablosunu oluşturur.
 *
 * @param efi_mem_map        EFI tarafından verilen memory map pointer'ı
 * @param desc_size          Her memory descriptor'ın boyutu
 * @param desc_count         Toplam descriptor sayısı
 * @param kernel_phys_start  Kernel fiziksel başlangıcı
 * @param kernel_phys_end    Kernel fiziksel sonu
 */
void phys_mem_init(void *efi_mem_map,
                   uint64_t desc_size,
                   uint64_t desc_count,
                   uint64_t kernel_phys_start,
                   uint64_t kernel_phys_end);


// -----------------------------------------------------------------------------
// FRAME ALLOKASYONU
// -----------------------------------------------------------------------------

/**
 * Tek bir fiziksel frame (4096 byte) ayırır.
 * @return fiziksel adres (başarısız olursa 0 dönebilir)
 */
uint64_t phys_alloc_frame(void);

/**
 * Ayrılmış bir frame’i boşaltır.
 */
void phys_free_frame(uint64_t phys_addr);


// -----------------------------------------------------------------------------
// PAGING (Sanal Bellek) Yönetimi – paging.c API
// -----------------------------------------------------------------------------
//
//  Not:
//    paging_init() → phys_mem_init() tamamlandıktan sonra çağrılmalıdır.
//    Çünkü page table belleklerini phys_alloc_frame() ile ayırıyoruz.
// -----------------------------------------------------------------------------

/**
 * Bootloader tarafından verilen PML4 fiziksel adresini devralır,
 * CR3'e yükler ve kernel'in page table kökünü ayarlar.
 */
void     paging_init(uint64_t boot_pml4_phys);

/**
 * Aktif CR3 kaydını verilen PML4 fiziksel adresiyle günceller.
 */
void     paging_load_cr3(uint64_t phys_addr);

/** Kernel PML4 fiziksel kök adresi. */
uint64_t paging_get_kernel_pml4_phys(void);

/** Yeni bir kullanıcı alanı PML4'ü oluşturur ve kernel yarım alanını kopyalar. */
uint64_t paging_create_user_pml4(void);

/**
 * Yeni bir 4KB page table (PML4/PDPT/PD/PT) ayırır ve sıfırlar.
 * @return fiziksel adres (başarısız olursa 0)
 */
uint64_t paging_alloc_page_table(void);

/**
 * Belirtilen sanal adresi (virt_addr), fiziksel adres (phys_addr)
 * ile verilen flag'lerle map eder.
 *
 * (Örn: AYKEN_PTE_USER → user-mode sayfa)
 */
void     paging_map_page(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags);

/** Belirtilen PML4 kökünde sayfa map'ler. */
void     paging_map_page_in_pml4(uint64_t pml4_phys,
                                 uint64_t virt_addr,
                                 uint64_t phys_addr,
                                 uint64_t flags);

/**
 * Eski API ile uyumluluk: paging_map() → paging_map_page() çağırır.
 */
void     paging_map(uint64_t virt, uint64_t phys, uint64_t flags);

/**
 * Bir sanal adresin map'ini kaldırır (PT entry = 0) ve TLB flush eder.
 */
void     paging_unmap(uint64_t virt);

/**
 * Bir sanal adresin hangi fiziksel adresle eşleştiğini döndürür.
 * Bulunamazsa 0 döndürür.
 */
uint64_t paging_get_phys(uint64_t virt);

/** Fiziksel adresi kernel sanal alanına çevirir (higher-half mapping). */
void    *paging_phys_to_virt(uint64_t phys);


// -----------------------------------------------------------------------------
// DURUM/İSTATİSTİK
// -----------------------------------------------------------------------------
uint64_t phys_get_total_frames(void);
uint64_t phys_get_free_frames(void);


// -----------------------------------------------------------------------------
// İLERİDE GEREKECEK OLANLAR (Multi-frame + Debug)
// -----------------------------------------------------------------------------

/**
 * Birden fazla ardışık frame alloc (örn: 4 KB değil, 16 KB istendiğinde).
 * Page table oluştururken, DMA buffer'larında vb. işe yarar.
 */
uint64_t phys_alloc_frames(uint64_t count);

/**
 * Birden fazla ardışık frame free etme.
 */
void phys_free_frames(uint64_t phys_addr, uint64_t count);

/**
 * Adresin gerçekten ayrılmış/boş olup olmadığını kontrol etmek için.
 * Debug için çok işe yarar.
 *
 * @return 1 → kullanılıyor, 0 → boş, -1 → geçersiz adres
 */
int phys_frame_is_used(uint64_t phys_addr);

#endif // AYKEN_MM_H
