// kernel/mm/paging.c
// ============================================================================
//  AykenOS Paging / Sanal Bellek Yöneticisi (x86_64, 4 seviye page table)
//
//  - Bootloader'dan gelen PML4 fiziksel adresini devralır
//  - CR3 kaydını yükler
//  - Yeni page table (PML4/PDPT/PD/PT) ayırır
//  - 4KB sayfa bazlı map / unmap işlemleri sağlar
//
//  Tasarım Notları:
//   * Şimdilik sadece 4KB sayfa kullanıyoruz (huge page yok).
//   * Tüm page table'lar fiziksel olarak 4KB frame içinde tutuluyor.
//   * Page table bellekleri phys_alloc_frame() ile ayrılıyor.
//   * Page table’lara erişim için higher-half mapping varsayımı:
//       virt = phys + KERNEL_VIRT_BASE
//     (Bootloader bu mapping’i kurmuş olmalı.)
// ============================================================================

#include <stdint.h>
#include "../include/mm.h"
#include "../include/ayken.h"
#include "../drivers/console/fb_console.h"

// ---------------------------------------------------------------------------
// x86_64 page table sabitleri ve flag'ler
// ---------------------------------------------------------------------------

typedef uint64_t ayken_pte_t;

#define AYKEN_PT_ENTRIES          512

#ifndef AYKEN_PTE_PRESENT
#define AYKEN_PTE_PRESENT         (1ULL << 0)
#define AYKEN_PTE_WRITABLE        (1ULL << 1)
#define AYKEN_PTE_USER            (1ULL << 2)
#define AYKEN_PTE_WRITE_THROUGH   (1ULL << 3)
#define AYKEN_PTE_CACHE_DISABLE   (1ULL << 4)
#define AYKEN_PTE_ACCESSED        (1ULL << 5)
#define AYKEN_PTE_DIRTY           (1ULL << 6)
#define AYKEN_PTE_HUGE            (1ULL << 7)
#define AYKEN_PTE_GLOBAL          (1ULL << 8)

// Tablo pointer'ları için kullanacağımız flags:
// Present + Writable (kernel space tablolar için yeterli)
#define AYKEN_PTE_TABLE_FLAGS     (AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE)

// Kernel page’leri için temel flag seti:
#define AYKEN_PTE_ADDR_MASK       0x000FFFFFFFFFF000ULL
#endif

#define AYKEN_PTE_KERNEL_FLAGS    (AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE | AYKEN_PTE_GLOBAL)

// Adresi entry'den çekmek için maske
// Eski isimlerle uyum için (istersen kullanabilirsin)
#define PAGE_PRESENT   AYKEN_PTE_PRESENT
#define PAGE_RW        AYKEN_PTE_WRITABLE
#define PAGE_USER      AYKEN_PTE_USER
#define PAGE_GLOBAL    AYKEN_PTE_GLOBAL
#define PAGE_ADDR_MASK AYKEN_PTE_ADDR_MASK

// Sanal adres → index hesaplayıcılar
#define PML4_INDEX(va)   (((va) >> 39) & 0x1FF)
#define PDPT_INDEX(va)   (((va) >> 30) & 0x1FF)
#define PD_INDEX(va)     (((va) >> 21) & 0x1FF)
#define PT_INDEX(va)     (((va) >> 12) & 0x1FF)


// ---------------------------------------------------------------------------
// Global durum
// ---------------------------------------------------------------------------

// Kernel PML4 fiziksel adresi ve sanal pointer'ı
static uint64_t   g_kernel_pml4_phys = 0;
static ayken_pte_t *g_kernel_pml4    = NULL;

// Higher-half mapping varsayımı:
//   virt = phys + KERNEL_VIRT_BASE
// Bootloader bu mapping'i kurmuş olmalı.
static inline void *phys_to_virt(uint64_t phys)
{
    return (void *)(phys + KERNEL_VIRT_BASE);
}

static inline uint64_t virt_to_phys(const void *virt)
{
    return ((uint64_t)virt - KERNEL_VIRT_BASE);
}

void *paging_phys_to_virt(uint64_t phys)
{
    return phys_to_virt(phys);
}

// CR3 yükleme helper
static inline void load_cr3(uint64_t phys_addr)
{
    __asm__ volatile ("mov %0, %%cr3" :: "r"(phys_addr) : "memory");
}

void paging_load_cr3(uint64_t phys_addr)
{
    load_cr3(phys_addr);
}


// ============================================================================
//  Yeni page table ayırma (4KB)
// ============================================================================

uint64_t paging_alloc_page_table(void)
{
    uint64_t phys = phys_alloc_frame();
    if (phys == 0) {
        fb_print("[AykenOS][paging] ERROR: phys_alloc_frame() failed for page table.\n");
        return 0;
    }

    ayken_pte_t *tbl = (ayken_pte_t *)phys_to_virt(phys);

    // Tüm girişleri temizle
    for (int i = 0; i < AYKEN_PT_ENTRIES; ++i)
        tbl[i] = 0;

    return phys;
}


// ============================================================================
//  Dahili yardımcı: Tablo getir/oluştur
//
//  Verilen üst seviye tabloda (PML4/PDPT/PD) index'e bakar:
//    - Eğer PRESENT ise var olan tablonun phys adresini alır.
//    - Değilse yeni bir page table ayırır, entry'yi doldurur.
//  Sonuçta alttaki tabloya sanal pointer döner.
// ============================================================================

static ayken_pte_t *get_or_create_table(ayken_pte_t *table,
                                        uint64_t index,
                                        uint64_t table_flags)
{
    if (!(table[index] & AYKEN_PTE_PRESENT)) {
        // Yeni bir frame al
        uint64_t phys = paging_alloc_page_table();
        if (!phys) {
            fb_print("[AykenOS][paging] ERROR: cannot alloc page table.\n");
            return NULL;
        }

        // Entry'ye yaz: adres + flags
        table[index] = (phys & AYKEN_PTE_ADDR_MASK) | table_flags;
    }

    uint64_t next_phys = table[index] & AYKEN_PTE_ADDR_MASK;
    return (ayken_pte_t *)phys_to_virt(next_phys);
}


// ============================================================================
//  paging_map_page
//
//  Belirtilen sanal adresi (virt_addr), fiziksel adres (phys_addr) ile
//  4KB sayfa olarak map eder.
//
//  flags: PTE tarafında eklenmesini istediğin ekstra bitler
//         Örn: AYKEN_PTE_USER vermek istiyorsan user-space page demektir.
// ============================================================================

static void paging_map_page_into_root(ayken_pte_t *root,
                                      uint64_t virt_addr,
                                      uint64_t phys_addr,
                                      uint64_t flags)
{
    if (!root) {
        fb_print("[AykenOS][paging] ERROR: invalid PML4 root.\n");
        return;
    }

    uint16_t i_pml4 = PML4_INDEX(virt_addr);
    uint16_t i_pdpt = PDPT_INDEX(virt_addr);
    uint16_t i_pd   = PD_INDEX(virt_addr);
    uint16_t i_pt   = PT_INDEX(virt_addr);

    uint64_t table_flags = AYKEN_PTE_TABLE_FLAGS;
    if (flags & AYKEN_PTE_USER)
        table_flags |= AYKEN_PTE_USER;

    ayken_pte_t *pdpt = get_or_create_table(root, i_pml4, table_flags);
    if (!pdpt) return;

    ayken_pte_t *pd = get_or_create_table(pdpt, i_pdpt, table_flags);
    if (!pd) return;

    ayken_pte_t *pt = get_or_create_table(pd, i_pd, table_flags);
    if (!pt) return;

    uint64_t entry_flags = AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE;
    if (flags & AYKEN_PTE_USER)
        entry_flags |= AYKEN_PTE_USER;
    else
        entry_flags |= AYKEN_PTE_GLOBAL;

    entry_flags |= (flags & ~(AYKEN_PTE_USER));

    pt[i_pt] = (phys_addr & AYKEN_PTE_ADDR_MASK) | entry_flags;
}

void paging_map_page(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags)
{
    if (g_kernel_pml4_phys == 0 || g_kernel_pml4 == NULL) {
        fb_print("[AykenOS][paging] ERROR: paging_init() not called.\n");
        return;
    }

    paging_map_page_into_root(g_kernel_pml4, virt_addr, phys_addr, flags);
}

void paging_map_page_in_pml4(uint64_t pml4_phys,
                             uint64_t virt_addr,
                             uint64_t phys_addr,
                             uint64_t flags)
{
    ayken_pte_t *root = (ayken_pte_t *)phys_to_virt(pml4_phys);
    paging_map_page_into_root(root, virt_addr, phys_addr, flags);
}


// ============================================================================
//  Eski API ile uyum: paging_map
//
//  Dışarıda eskiden kullanılan isimle fonksiyon sunuyoruz.
//  İçeride yeni paging_map_page() fonksiyonuna yönlendiriliyor.
// ============================================================================

void paging_map(uint64_t virt, uint64_t phys, uint64_t flags)
{
    // flags parametresini PTE flags olarak geçiyoruz.
    paging_map_page(virt, phys, flags);
}


// ============================================================================
//  paging_unmap
//
//  Verilen sanal adres için PT entry'yi sıfırlar,
//  ardından TLB flush (invlpg) yapar.
//  Şimdilik boşalan tabloları free etmiyoruz; ileride optimize edilebilir.
// ============================================================================

void paging_unmap(uint64_t virt)
{
    if (!g_kernel_pml4)
        return;

    uint16_t pml4_i = PML4_INDEX(virt);
    uint16_t pdpt_i = PDPT_INDEX(virt);
    uint16_t pd_i   = PD_INDEX(virt);
    uint16_t pt_i   = PT_INDEX(virt);

    ayken_pte_t pml4e = g_kernel_pml4[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) return;
    ayken_pte_t *pdpt = (ayken_pte_t *)phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);

    ayken_pte_t pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT)) return;
    ayken_pte_t *pd = (ayken_pte_t *)phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);

    ayken_pte_t pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT)) return;
    ayken_pte_t *pt = (ayken_pte_t *)phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);

    pt[pt_i] = 0;

    // TLB flush
    __asm__ volatile("invlpg (%0)" :: "r"(virt) : "memory");
}


// ============================================================================
//  paging_get_phys
//
//  Verilen sanal adresin map edildiği fiziksel adresi döner.
//  Map yoksa 0 döndürür.
// ============================================================================

uint64_t paging_get_phys(uint64_t virt)
{
    if (!g_kernel_pml4)
        return 0;

    uint16_t pml4_i = PML4_INDEX(virt);
    uint16_t pdpt_i = PDPT_INDEX(virt);
    uint16_t pd_i   = PD_INDEX(virt);
    uint16_t pt_i   = PT_INDEX(virt);

    ayken_pte_t pml4e = g_kernel_pml4[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) return 0;
    ayken_pte_t *pdpt = (ayken_pte_t *)phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);

    ayken_pte_t pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT)) return 0;
    ayken_pte_t *pd = (ayken_pte_t *)phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);

    ayken_pte_t pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT)) return 0;
    ayken_pte_t *pt = (ayken_pte_t *)phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);

    ayken_pte_t pte = pt[pt_i];
    if (!(pte & AYKEN_PTE_PRESENT)) return 0;

    return (pte & AYKEN_PTE_ADDR_MASK);
}

uint64_t paging_get_kernel_pml4_phys(void)
{
    return g_kernel_pml4_phys;
}

uint64_t paging_create_user_pml4(void)
{
    uint64_t new_pml4_phys = paging_alloc_page_table();
    if (!new_pml4_phys)
        return 0;

    ayken_pte_t *new_root = (ayken_pte_t *)phys_to_virt(new_pml4_phys);

    for (int i = AYKEN_PT_ENTRIES / 2; i < AYKEN_PT_ENTRIES; ++i) {
        new_root[i] = g_kernel_pml4[i];
    }

    return new_pml4_phys;
}


// ============================================================================
//  paging_init
//
//  Bootloader'dan gelen PML4 fiziksel adresini devralır ve CR3'e yükler.
//  Bu fonksiyon phys_mem_init'tan SONRA çağrılmalıdır.
//
//  Varsayım:
//    * Bootloader, kernel'i higher-half (KERNEL_VIRT_BASE + ...) adresine
//      map etmiş durumda.
//    * PML4 tablosu bu mapping'i içeriyor.
// ============================================================================

void paging_init(uint64_t pml4_phys)
{
    fb_print("[AykenOS][paging] Initializing paging...\n");

    if (pml4_phys == 0) {
        fb_print("[AykenOS][paging] ERROR: pml4_phys = 0.\n");
        return;
    }

    g_kernel_pml4_phys = pml4_phys;
    g_kernel_pml4      = (ayken_pte_t *)phys_to_virt(pml4_phys);

    load_cr3(pml4_phys);

    fb_print("[AykenOS][paging] PML4 at phys=0x");
    fb_print_hex64(pml4_phys);
    fb_print("\n");

    // Burada: identity map'i temizleyelim (örnek: ilk 1GB)
    paging_drop_identity_map(0x40000000ULL); // 1GB

    fb_print("[AykenOS][paging] Paging is now active (no identity map).\n");
}

// paging.c'nin sonlarına doğru eklenebilir

// [0, limit) aralığındaki identity map'leri kaldır
void paging_drop_identity_map(uint64_t limit_phys)
{
    fb_print("[paging] Dropping identity map up to 0x");
    fb_print_hex64(limit_phys);
    fb_print("\n");

    // 4KB adım
    for (uint64_t addr = 0; addr < limit_phys; addr += AYKEN_FRAME_SIZE) {
        // Sanal = fiziksel (identity)
        paging_unmap(addr);
    }

    // TLB global temizlik
    uint64_t cr3;
    __asm__ volatile("mov %%cr3, %0" : "=r"(cr3));
    __asm__ volatile("mov %0, %%cr3" :: "r"(cr3));
}
