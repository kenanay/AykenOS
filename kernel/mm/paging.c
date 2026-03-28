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
#include <stddef.h>
#include "../include/mm.h"
#include "../include/ayken.h"
#include "../drivers/console/fb_console.h"
#include "../arch/x86_64/port_io.h"

// Forward declaration
static void paging_drop_identity_map(uint64_t limit_phys);
static inline void paging_dbg(char c)
{
    outb(0xE9, (uint8_t)c);
}

#ifndef AYKEN_SHARE_KERNEL_UPPER_HALF
#define AYKEN_SHARE_KERNEL_UPPER_HALF 0
#endif

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
#define PAGING_DIAG_WATCH_PHYS_LO 0x000000000010A000ULL
#define PAGING_DIAG_WATCH_PHYS_HI 0x000000000010E000ULL

static inline void paging_diag_emit_char(char c)
{
    outb(0xE9, (uint8_t)c);
}

static void paging_diag_emit_text(const char *text)
{
    if (!text) {
        return;
    }

    while (*text) {
        paging_diag_emit_char(*text++);
    }
}

static void paging_diag_emit_hex64(uint64_t value)
{
    static const char hex[] = "0123456789ABCDEF";

    for (int shift = 60; shift >= 0; shift -= 4) {
        paging_diag_emit_char(hex[(value >> shift) & 0xFULL]);
    }
}

static int paging_diag_watch_phys(uint64_t phys)
{
    uint64_t page = phys & AYKEN_PTE_ADDR_MASK;

    return page >= PAGING_DIAG_WATCH_PHYS_LO && page < PAGING_DIAG_WATCH_PHYS_HI;
}

static void paging_diag_log_alloc(uint64_t phys)
{
    if (!paging_diag_watch_phys(phys)) {
        return;
    }

    paging_diag_emit_text("PTAL P=");
    paging_diag_emit_hex64(phys & AYKEN_PTE_ADDR_MASK);
    paging_diag_emit_text("\n");
}

static void __attribute__((unused)) paging_diag_log_share_upper_half(uint64_t root_phys)
{
    paging_diag_emit_text("P10_SHARE_KERNEL_UPPER_HALF R=");
    paging_diag_emit_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
    paging_diag_emit_text("\n");
}

static void paging_diag_log_table_write(const char *tag,
                                        uint64_t root_phys,
                                        uint64_t table_phys,
                                        uint64_t index,
                                        uint64_t old_entry,
                                        uint64_t new_entry,
                                        uint64_t virt_addr,
                                        uint64_t phys_addr,
                                        uint64_t flags)
{
    uint64_t table_page = table_phys & AYKEN_PTE_ADDR_MASK;
    uint64_t old_phys = old_entry & AYKEN_PTE_ADDR_MASK;
    uint64_t new_phys = new_entry & AYKEN_PTE_ADDR_MASK;

    if (!paging_diag_watch_phys(table_page) &&
        !paging_diag_watch_phys(old_phys) &&
        !paging_diag_watch_phys(new_phys)) {
        return;
    }

    paging_diag_emit_text("PTWR T=");
    paging_diag_emit_text(tag);
    paging_diag_emit_text(" R=");
    paging_diag_emit_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
    paging_diag_emit_text(" B=");
    paging_diag_emit_hex64(table_page);
    paging_diag_emit_text(" I=");
    paging_diag_emit_hex64(index);
    paging_diag_emit_text(" O=");
    paging_diag_emit_hex64(old_entry);
    paging_diag_emit_text(" N=");
    paging_diag_emit_hex64(new_entry);
    paging_diag_emit_text(" V=");
    paging_diag_emit_hex64(virt_addr);
    paging_diag_emit_text(" P=");
    paging_diag_emit_hex64(phys_addr & AYKEN_PTE_ADDR_MASK);
    paging_diag_emit_text(" F=");
    paging_diag_emit_hex64(flags);
    paging_diag_emit_text("\n");

    if ((old_entry & AYKEN_PTE_PRESENT) != 0 && new_entry != old_entry) {
        paging_diag_emit_text("PTOV T=");
        paging_diag_emit_text(tag);
        paging_diag_emit_text(" R=");
        paging_diag_emit_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
        paging_diag_emit_text(" B=");
        paging_diag_emit_hex64(table_page);
        paging_diag_emit_text(" I=");
        paging_diag_emit_hex64(index);
        paging_diag_emit_text(" O=");
        paging_diag_emit_hex64(old_entry);
        paging_diag_emit_text(" N=");
        paging_diag_emit_hex64(new_entry);
        paging_diag_emit_text(" V=");
        paging_diag_emit_hex64(virt_addr);
        paging_diag_emit_text("\n");
    }
}
#else
static inline void paging_diag_log_alloc(uint64_t phys)
{
    (void)phys;
}

static inline void paging_diag_log_share_upper_half(uint64_t root_phys)
{
    (void)root_phys;
}

static inline void paging_diag_log_table_write(const char *tag,
                                               uint64_t root_phys,
                                               uint64_t table_phys,
                                               uint64_t index,
                                               uint64_t old_entry,
                                               uint64_t new_entry,
                                               uint64_t virt_addr,
                                               uint64_t phys_addr,
                                               uint64_t flags)
{
    (void)tag;
    (void)root_phys;
    (void)table_phys;
    (void)index;
    (void)old_entry;
    (void)new_entry;
    (void)virt_addr;
    (void)phys_addr;
    (void)flags;
}
#endif

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
#define AYKEN_PTE_READ_ONLY       (1ULL << 9)
#define AYKEN_PTE_NO_GLOBAL       (1ULL << 10)
#define AYKEN_PTE_NO_EXEC         (1ULL << 63)

// Tablo pointer'ları için kullanacağımız flags:
// Present + Writable (kernel space tablolar için yeterli)
#define AYKEN_PTE_TABLE_FLAGS     (AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE)

// Kernel page’leri için temel flag seti:
#define AYKEN_PTE_ADDR_MASK       0x000FFFFFFFFFF000ULL
#endif

#ifndef AYKEN_PTE_TABLE_FLAGS
#define AYKEN_PTE_TABLE_FLAGS     (AYKEN_PTE_PRESENT | AYKEN_PTE_WRITABLE)
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
    if (phys < AYKEN_IDENTITY_MAP_SIZE) {
        uint64_t active_cr3 = 0;

        if (g_kernel_pml4_phys == 0) {
            return (void *)(uintptr_t)phys;
        }

        __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
        if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (g_kernel_pml4_phys & AYKEN_PTE_ADDR_MASK)) {
            return (void *)(uintptr_t)phys;
        }
    }
    return (void *)(phys + KERNEL_VIRT_BASE);
}

static inline uint64_t virt_to_phys(const void *virt) __attribute__((unused));
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

static inline void invalidate_if_active(uint64_t target_pml4_phys, uint64_t virt_addr)
{
    uint64_t active_cr3 = 0;

    if (target_pml4_phys == 0) {
        return;
    }

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (target_pml4_phys & AYKEN_PTE_ADDR_MASK)) {
        __asm__ volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
    }
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

    paging_diag_log_alloc(phys);

    return phys;
}

uint64_t paging_alloc_page_table_high(void)
{
    uint64_t phys = phys_alloc_frame_high();
    if (phys == 0) {
        fb_print("[AykenOS][paging] ERROR: phys_alloc_frame_high() failed for page table.\n");
        return 0;
    }

    ayken_pte_t *tbl = (ayken_pte_t *)phys_to_virt(phys);

    for (int i = 0; i < AYKEN_PT_ENTRIES; ++i) {
        tbl[i] = 0;
    }

    paging_diag_log_alloc(phys);

    return phys;
}

static uint64_t paging_alloc_page_table_for_root(uint64_t root_phys)
{
    uint64_t kernel_root = g_kernel_pml4_phys & AYKEN_PTE_ADDR_MASK;
    uint64_t target_root = root_phys & AYKEN_PTE_ADDR_MASK;

    /*
     * Keep authored user/alternate roots out of the low-phys table class.
     * Kernel-root allocations can stay on the default path.
     */
    if (target_root != 0 && target_root != kernel_root) {
        return paging_alloc_page_table_high();
    }
    return paging_alloc_page_table();
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
                                        uint64_t table_phys,
                                        uint64_t root_phys,
                                        uint64_t virt_addr,
                                        uint64_t index,
                                        uint64_t table_flags,
                                        uint64_t *next_table_phys_out)
{
    if (!(table[index] & AYKEN_PTE_PRESENT)) {
        ayken_pte_t old_entry = table[index];
        // Yeni bir frame al
        uint64_t phys = paging_alloc_page_table_for_root(root_phys);
        if (!phys) {
            fb_print("[AykenOS][paging] ERROR: cannot alloc page table.\n");
            return NULL;
        }

        // Entry'ye yaz: adres + flags
        table[index] = (phys & AYKEN_PTE_ADDR_MASK) | table_flags;
        paging_diag_log_table_write(
            "GOC",
            root_phys,
            table_phys,
            index,
            old_entry,
            table[index],
            virt_addr,
            phys,
            table_flags);
    } else if ((table_flags & AYKEN_PTE_USER) &&
               !(table[index] & AYKEN_PTE_USER)) {
        ayken_pte_t old_entry = table[index];
        // Mixed low-half trees may hold both user leaves and supervisor-only
        // kernel leaves. Parent entries must be user-visible when any child
        // user mapping exists; leaf permissions still enforce supervisor-only
        // access for kernel heap pages.
        table[index] |= AYKEN_PTE_USER;
        paging_diag_log_table_write(
            "GOU",
            root_phys,
            table_phys,
            index,
            old_entry,
            table[index],
            virt_addr,
            table[index] & AYKEN_PTE_ADDR_MASK,
            table_flags);
    }

    uint64_t next_phys = table[index] & AYKEN_PTE_ADDR_MASK;
    if (next_table_phys_out) {
        *next_table_phys_out = next_phys;
    }
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
                                      uint64_t root_phys,
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
    uint64_t pdpt_phys = 0;
    uint64_t pd_phys = 0;
    uint64_t pt_phys = 0;

    uint64_t table_flags = AYKEN_PTE_TABLE_FLAGS;
    if (flags & AYKEN_PTE_USER)
        table_flags |= AYKEN_PTE_USER;

    ayken_pte_t *pdpt = get_or_create_table(
        root,
        root_phys,
        root_phys,
        virt_addr,
        i_pml4,
        table_flags,
        &pdpt_phys);
    if (!pdpt) return;

    ayken_pte_t *pd = get_or_create_table(
        pdpt,
        pdpt_phys,
        root_phys,
        virt_addr,
        i_pdpt,
        table_flags,
        &pd_phys);
    if (!pd) return;

    ayken_pte_t *pt = get_or_create_table(
        pd,
        pd_phys,
        root_phys,
        virt_addr,
        i_pd,
        table_flags,
        &pt_phys);
    if (!pt) return;

    uint64_t entry_flags = AYKEN_PTE_PRESENT;
    if (!(flags & AYKEN_PTE_READ_ONLY) &&
        ((flags & AYKEN_PTE_WRITABLE) || !(flags & AYKEN_PTE_USER))) {
        entry_flags |= AYKEN_PTE_WRITABLE;
    }
    if (flags & AYKEN_PTE_USER)
        entry_flags |= AYKEN_PTE_USER;
    else if ((flags & AYKEN_PTE_NO_GLOBAL) == 0)
        entry_flags |= AYKEN_PTE_GLOBAL;

    entry_flags |= (flags & ~(AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_GLOBAL));

    {
        ayken_pte_t old_entry = pt[i_pt];
        ayken_pte_t new_entry = (phys_addr & AYKEN_PTE_ADDR_MASK) | entry_flags;

        pt[i_pt] = new_entry;
        paging_diag_log_table_write(
            "MAP",
            root_phys,
            pt_phys,
            i_pt,
            old_entry,
            new_entry,
            virt_addr,
            phys_addr,
            flags);
    }
}

void paging_map_page(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags)
{
    if (g_kernel_pml4_phys == 0 || g_kernel_pml4 == NULL) {
        fb_print("[AykenOS][paging] ERROR: paging_init() not called.\n");
        return;
    }

    paging_map_page_into_root(
        g_kernel_pml4, g_kernel_pml4_phys, virt_addr, phys_addr, flags);
    invalidate_if_active(g_kernel_pml4_phys, virt_addr);
}

void paging_map_page_in_pml4(uint64_t pml4_phys,
                             uint64_t virt_addr,
                             uint64_t phys_addr,
                             uint64_t flags)
{
    ayken_pte_t *root = (ayken_pte_t *)phys_to_virt(pml4_phys);
    paging_map_page_into_root(root, pml4_phys, virt_addr, phys_addr, flags);
    invalidate_if_active(pml4_phys, virt_addr);
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

static void paging_unmap_from_root(ayken_pte_t *root,
                                   uint64_t virt,
                                   uint64_t target_pml4_phys)
{
    uint16_t pml4_i;
    uint16_t pdpt_i;
    uint16_t pd_i;
    uint16_t pt_i;
    ayken_pte_t pml4e;
    ayken_pte_t *pdpt;
    ayken_pte_t pdpte;
    ayken_pte_t *pd;
    ayken_pte_t pde;
    ayken_pte_t *pt;
    uint64_t pdpt_phys;
    uint64_t pd_phys;
    uint64_t pt_phys;
    uint64_t current_cr3 = 0;

    if (!root) {
        return;
    }

    pml4_i = PML4_INDEX(virt);
    pdpt_i = PDPT_INDEX(virt);
    pd_i = PD_INDEX(virt);
    pt_i = PT_INDEX(virt);

    pml4e = root[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) return;
    pdpt_phys = pml4e & AYKEN_PTE_ADDR_MASK;
    pdpt = (ayken_pte_t *)phys_to_virt(pdpt_phys);

    pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT)) return;
    pd_phys = pdpte & AYKEN_PTE_ADDR_MASK;
    pd = (ayken_pte_t *)phys_to_virt(pd_phys);

    pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT)) return;
    pt_phys = pde & AYKEN_PTE_ADDR_MASK;
    pt = (ayken_pte_t *)phys_to_virt(pt_phys);

    {
        ayken_pte_t old_entry = pt[pt_i];
        pt[pt_i] = 0;
        paging_diag_log_table_write(
            "UNM",
            target_pml4_phys,
            pt_phys,
            pt_i,
            old_entry,
            0,
            virt,
            old_entry & AYKEN_PTE_ADDR_MASK,
            0);
    }

    __asm__ volatile("mov %%cr3, %0" : "=r"(current_cr3));
    if ((current_cr3 & AYKEN_PTE_ADDR_MASK) == (target_pml4_phys & AYKEN_PTE_ADDR_MASK)) {
        __asm__ volatile("invlpg (%0)" :: "r"(virt) : "memory");
    }
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
    if (!g_kernel_pml4) {
        return;
    }

    paging_unmap_from_root(g_kernel_pml4, virt, g_kernel_pml4_phys);
}

void paging_unmap_in_pml4(uint64_t pml4_phys, uint64_t virt)
{
    ayken_pte_t *root;

    if (!pml4_phys) {
        return;
    }

    root = (ayken_pte_t *)phys_to_virt(pml4_phys);
    paging_unmap_from_root(root, virt, pml4_phys);
}


// ============================================================================
//  paging_get_phys
//
//  Verilen sanal adresin map edildiği fiziksel adresi döner.
//  Map yoksa 0 döndürür.
// ============================================================================

uint64_t paging_get_phys(uint64_t virt)
{
    return paging_get_pte(virt) & AYKEN_PTE_ADDR_MASK;
}

uint64_t paging_get_pte(uint64_t virt)
{
    if (!g_kernel_pml4_phys)
        return 0;
    return paging_get_pte_in_pml4(g_kernel_pml4_phys, virt);
}

uint64_t paging_get_pte_in_pml4(uint64_t pml4_phys, uint64_t virt)
{
    ayken_pte_t *root;
    uint16_t pml4_i;
    uint16_t pdpt_i;
    uint16_t pd_i;
    uint16_t pt_i;
    ayken_pte_t pml4e;
    ayken_pte_t *pdpt;
    ayken_pte_t pdpte;
    ayken_pte_t *pd;
    ayken_pte_t pde;
    ayken_pte_t *pt;
    ayken_pte_t pte = 0;
    uint64_t active_cr3;
    uint64_t kernel_cr3;
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;

    if (!pml4_phys) {
        return 0;
    }

    pml4_phys &= AYKEN_PTE_ADDR_MASK;
    kernel_cr3 = g_kernel_pml4_phys & AYKEN_PTE_ADDR_MASK;
    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if (kernel_cr3 != 0 &&
        ((active_cr3 & AYKEN_PTE_ADDR_MASK) != kernel_cr3)) {
        __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
        __asm__ volatile("cli");
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
        switched_to_kernel_cr3 = 1;
    }

    root = (ayken_pte_t *)phys_to_virt(pml4_phys);
    pml4_i = PML4_INDEX(virt);
    pdpt_i = PDPT_INDEX(virt);
    pd_i = PD_INDEX(virt);
    pt_i = PT_INDEX(virt);

    pml4e = root[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) goto out;
    pdpt = (ayken_pte_t *)phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);

    pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT)) goto out;
    pd = (ayken_pte_t *)phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);

    pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT)) goto out;
    pt = (ayken_pte_t *)phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);

    pte = pt[pt_i];
    if (!(pte & AYKEN_PTE_PRESENT)) {
        pte = 0;
        goto out;
    }

out:
    if (switched_to_kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
        if (saved_rflags & (1ULL << 9)) {
            __asm__ volatile("sti");
        }
    }

    return pte;
}

uint64_t paging_get_kernel_pml4_phys(void)
{
    return g_kernel_pml4_phys;
}

static void free_cloned_table_recursive(uint64_t table_phys, uint32_t level)
{
    ayken_pte_t *table;

    if (table_phys == 0 || level == 0) {
        return;
    }

    table = (ayken_pte_t *)phys_to_virt(table_phys & AYKEN_PTE_ADDR_MASK);
    if (!table) {
        return;
    }

    for (uint32_t i = 0; i < AYKEN_PT_ENTRIES; ++i) {
        ayken_pte_t entry = table[i];
        uint64_t child_phys;

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            continue;
        }

        child_phys = entry & AYKEN_PTE_ADDR_MASK;
        if (child_phys == 0) {
            table[i] = 0;
            continue;
        }

        if (level == 1 || (entry & (1ULL << 7))) {
            table[i] = 0;
            continue;
        }

        free_cloned_table_recursive(child_phys, level - 1);
        phys_free_frame(child_phys);
        table[i] = 0;
    }
}

static void free_cloned_kernel_half_in_root(ayken_pte_t *root)
{
    if (!root) {
        return;
    }

    for (uint32_t i = AYKEN_PT_ENTRIES / 2; i < AYKEN_PT_ENTRIES; ++i) {
        ayken_pte_t entry = root[i];
        uint64_t child_phys;

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            continue;
        }

        child_phys = entry & AYKEN_PTE_ADDR_MASK;
        if (child_phys == 0) {
            root[i] = 0;
            continue;
        }

        if ((entry & (1ULL << 7)) == 0) {
            free_cloned_table_recursive(child_phys, 3);
            phys_free_frame(child_phys);
        }
        root[i] = 0;
    }
}

static uint64_t clone_kernel_table_recursive(uint64_t table_phys,
                                             uint32_t level,
                                             uint64_t root_phys)
{
    ayken_pte_t *src;
    ayken_pte_t *dst;
    uint64_t cloned_phys;

    if (table_phys == 0 || level == 0) {
        return 0;
    }

    /*
     * User-root upper-half clones must stay out of the low-phys page-table
     * class. The root frame was already moved high; keep child tables in the
     * same high-phys band so hardware fetch sees a consistent upper subtree.
     */
    cloned_phys = paging_alloc_page_table_high();
    if (cloned_phys == 0) {
        return 0;
    }

    src = (ayken_pte_t *)phys_to_virt(table_phys & AYKEN_PTE_ADDR_MASK);
    dst = (ayken_pte_t *)phys_to_virt(cloned_phys);
    if (!src || !dst) {
        phys_free_frame(cloned_phys);
        return 0;
    }

    for (uint32_t i = 0; i < AYKEN_PT_ENTRIES; ++i) {
        ayken_pte_t entry = src[i];

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            dst[i] = 0;
            continue;
        }

        if (level == 1 || (entry & (1ULL << 7))) {
            ayken_pte_t old_entry = dst[i];
            ayken_pte_t new_entry = entry & ~AYKEN_PTE_USER;

            dst[i] = new_entry;
            paging_diag_log_table_write(
                "CLN",
                root_phys,
                cloned_phys,
                i,
                old_entry,
                new_entry,
                0,
                new_entry & AYKEN_PTE_ADDR_MASK,
                entry);
            continue;
        }

        {
            uint64_t child_phys = entry & AYKEN_PTE_ADDR_MASK;
            uint64_t cloned_child_phys =
                clone_kernel_table_recursive(child_phys, level - 1, root_phys);
            uint64_t entry_flags = (entry & ~AYKEN_PTE_ADDR_MASK) & ~AYKEN_PTE_USER;
            ayken_pte_t old_entry = dst[i];

            if (cloned_child_phys == 0) {
                free_cloned_table_recursive(cloned_phys, level);
                phys_free_frame(cloned_phys);
                return 0;
            }

            dst[i] = cloned_child_phys | entry_flags;
            paging_diag_log_table_write(
                "CLN",
                root_phys,
                cloned_phys,
                i,
                old_entry,
                dst[i],
                0,
                cloned_child_phys,
                entry_flags);
        }
    }

    return cloned_phys;
}

uint64_t paging_create_user_pml4(void)
{
    uint64_t new_pml4_phys = paging_alloc_page_table_high();
    if (!new_pml4_phys)
        return 0;

    ayken_pte_t *new_root = (ayken_pte_t *)phys_to_virt(new_pml4_phys);
    if (!new_root) {
        phys_free_frame(new_pml4_phys);
        return 0;
    }

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_SHARE_KERNEL_UPPER_HALF == 1)
    paging_diag_log_share_upper_half(new_pml4_phys);
    for (uint32_t i = AYKEN_PT_ENTRIES / 2; i < AYKEN_PT_ENTRIES; ++i) {
        ayken_pte_t entry = g_kernel_pml4[i];
        ayken_pte_t old_entry = new_root[i];

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            new_root[i] = 0;
            continue;
        }

        new_root[i] = entry & ~AYKEN_PTE_USER;
        paging_diag_log_table_write(
            "SHR",
            new_pml4_phys,
            new_pml4_phys,
            i,
            old_entry,
            new_root[i],
            0,
            new_root[i] & AYKEN_PTE_ADDR_MASK,
            entry);
    }

    return new_pml4_phys;
#endif

    for (uint32_t i = AYKEN_PT_ENTRIES / 2; i < AYKEN_PT_ENTRIES; ++i) {
        ayken_pte_t entry = g_kernel_pml4[i];

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            new_root[i] = 0;
            continue;
        }

        if (entry & (1ULL << 7)) {
            ayken_pte_t old_entry = new_root[i];

            new_root[i] = entry & ~AYKEN_PTE_USER;
            paging_diag_log_table_write(
                "CRT",
                new_pml4_phys,
                new_pml4_phys,
                i,
                old_entry,
                new_root[i],
                0,
                new_root[i] & AYKEN_PTE_ADDR_MASK,
                entry);
            continue;
        }

        {
            uint64_t child_phys = entry & AYKEN_PTE_ADDR_MASK;
            uint64_t cloned_child_phys =
                clone_kernel_table_recursive(child_phys, 3, new_pml4_phys);
            uint64_t entry_flags = (entry & ~AYKEN_PTE_ADDR_MASK) & ~AYKEN_PTE_USER;
            ayken_pte_t old_entry = new_root[i];

            if (cloned_child_phys == 0) {
                free_cloned_kernel_half_in_root(new_root);
                phys_free_frame(new_pml4_phys);
                return 0;
            }

            new_root[i] = cloned_child_phys | entry_flags;
            paging_diag_log_table_write(
                "CRT",
                new_pml4_phys,
                new_pml4_phys,
                i,
                old_entry,
                new_root[i],
                0,
                cloned_child_phys,
                entry_flags);
        }
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
    paging_dbg('P');
    fb_print("[AykenOS][paging] Initializing paging...\n");

    if (pml4_phys == 0) {
        paging_dbg('0');
        fb_print("[AykenOS][paging] ERROR: pml4_phys = 0.\n");
        return;
    }

    g_kernel_pml4_phys = pml4_phys;
    g_kernel_pml4      = (ayken_pte_t *)phys_to_virt(pml4_phys);

    paging_dbg('1');
    load_cr3(pml4_phys);
    paging_dbg('2');

    fb_print("[AykenOS][paging] PML4 at phys=0x");
    fb_print_hex64(pml4_phys);
    fb_print("\n");

    // Burada: identity map'i temizleyelim (örnek: ilk 1GB)
    paging_dbg('3');
    // paging_drop_identity_map(0x01000000ULL); // debug: geçici olarak kapalı
    paging_dbg('4');

    fb_print("[AykenOS][paging] Paging is now active (no identity map).\n");
}

// paging.c'nin sonlarına doğru eklenebilir

// [0, limit) aralığındaki identity map'leri kaldır
void paging_drop_identity_map(uint64_t limit_phys) __attribute__((unused));
void paging_drop_identity_map(uint64_t limit_phys)
{
    paging_dbg('d');
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
    paging_dbg('e');
}
