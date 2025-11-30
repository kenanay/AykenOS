// kernel/mm/phys_mem.c
// ============================================================================
//  AykenOS Physical Memory Manager (bitmap tabanlı frame allocator)
//  Açıklamalı, klasik + gelişmiş fonksiyonlarla güncellenmiş tam sürüm
// ============================================================================

#include <stdint.h>
#include <stddef.h>
#include "include/mm.h"
#include "include/ayken.h"
#include "drivers/console/fb_console.h"

// ---------------------------------------------------------------------------
// EFI memory map entry (UEFI’nin EFI_MEMORY_DESCRIPTOR eşleniği)
// ---------------------------------------------------------------------------

typedef struct {
    uint32_t type;
    uint32_t pad;
    uint64_t phys_start;
    uint64_t virt_start;
    uint64_t num_pages;
    uint64_t attrib;
} ayken_efi_mmap_entry_t;

#define AYKEN_EFI_MEM_CONVENTIONAL   7   // UEFI: kullanılabilir RAM türü

// ---------------------------------------------------------------------------
// Bitmap: Her bit = 1 frame (4 KB)
//
// 0 = free
// 1 = used (ayrılmış)
//
// Toplam frame sayısı AYKEN_MAX_FRAMES kadar olabilir.
// ---------------------------------------------------------------------------

static uint8_t  g_frame_bitmap[AYKEN_MAX_FRAMES / 8];
static uint64_t g_total_frames = 0;
static uint64_t g_free_frames  = 0;

// Son alloc arama başlangıcı (performans için)
static uint64_t g_last_alloc_search_idx = 0;


// ---------------------------------------------------------------------------
// Bitmap yardımcı fonksiyonları (eski haliyle KORUNDU)
// ---------------------------------------------------------------------------

static inline void frame_set(uint64_t frame_idx)
{
    g_frame_bitmap[frame_idx / 8] |= (1u << (frame_idx % 8));
}

static inline void frame_clear(uint64_t frame_idx)
{
    g_frame_bitmap[frame_idx / 8] &= ~(1u << (frame_idx % 8));
}

static inline int frame_test(uint64_t frame_idx)
{
    return (g_frame_bitmap[frame_idx / 8] >> (frame_idx % 8)) & 1u;
}

static inline uint64_t addr_to_frame_idx(uint64_t phys_addr)
{
    return phys_addr / AYKEN_FRAME_SIZE;
}

static inline uint64_t frame_idx_to_addr(uint64_t idx)
{
    return idx * AYKEN_FRAME_SIZE;
}

// Tüm frame’leri “dolu” olarak işaretle, sonra usable RAM açılacak
static void bitmap_mark_all_used(void)
{
    for (uint64_t i = 0; i < (AYKEN_MAX_FRAMES / 8); ++i)
        g_frame_bitmap[i] = 0xFF;

    g_total_frames = 0;
    g_free_frames  = 0;
}



// ===========================================================================
//  FİZİKSEL BELLEK BAŞLATMA  (ESKİ KOD + açıklamalar)
// ===========================================================================

void phys_mem_init(void *efi_mem_map,
                   uint64_t desc_size,
                   uint64_t desc_count,
                   uint64_t kernel_phys_start,
                   uint64_t kernel_phys_end)
{
    fb_print("[phys_mem] Initializing physical memory manager...\n");

    // 1) Tüm frame’leri kapalı (used) yap
    bitmap_mark_all_used();

    uint8_t *mmap_ptr = (uint8_t *)efi_mem_map;

    // 2) UEFI memory map içindeki usable bölgeleri free olarak işaretle
    for (uint64_t i = 0; i < desc_count; ++i) {

        ayken_efi_mmap_entry_t *ent =
            (ayken_efi_mmap_entry_t *)(mmap_ptr + i * desc_size);

        if (ent->type != AYKEN_EFI_MEM_CONVENTIONAL)
            continue;

        uint64_t region_start = ent->phys_start;
        uint64_t region_size  = ent->num_pages * 4096ULL;
        uint64_t region_end   = region_start + region_size;

        // 4GB sınırı dışını yok say
        if (region_start >= AYKEN_MAX_PHYS_MEM)
            continue;
        if (region_end > AYKEN_MAX_PHYS_MEM)
            region_end = AYKEN_MAX_PHYS_MEM;

        uint64_t first_frame = addr_to_frame_idx(region_start);
        uint64_t last_frame  = addr_to_frame_idx(region_end - 1);

        // Bölgedeki tüm frame’leri free yap
        for (uint64_t f = first_frame; f <= last_frame && f < AYKEN_MAX_FRAMES; ++f) {
            frame_clear(f);   // 0 = free
            g_total_frames++;
            g_free_frames++;
        }
    }

    // 3) Kernel’in fiziksel adres aralığını rezerve et
    uint64_t k_start_frame = addr_to_frame_idx(kernel_phys_start);
    uint64_t k_end_frame   = addr_to_frame_idx(kernel_phys_end - 1);

    for (uint64_t f = k_start_frame; f <= k_end_frame && f < AYKEN_MAX_FRAMES; ++f) {
        if (!frame_test(f)) {
            frame_set(f);
            g_free_frames--;
        }
    }

    // 4) İlk 1MB’yi rezerve et (BIOS, APIC, EBDA vb.)
    for (uint64_t f = 0; f < addr_to_frame_idx(0x100000ULL); ++f) {
        if (!frame_test(f)) {
            frame_set(f);
            g_free_frames--;
        }
    }

    g_last_alloc_search_idx = 0;

    fb_print("[phys_mem] total frames: ");
    fb_print_hex64(g_total_frames);
    fb_print(", free: ");
    fb_print_hex64(g_free_frames);
    fb_print("\n");

    fb_print("[phys_mem] init done.\n");
}



// ===========================================================================
//  TEK FRAME ALLOCATION (eski sürüm — korunarak bırakıldı)
// ===========================================================================

uint64_t phys_alloc_frame(void)
{
    if (g_free_frames == 0)
        return 0;

    // 1) Son aramadan itibaren devam
    for (uint64_t i = g_last_alloc_search_idx; i < AYKEN_MAX_FRAMES; ++i) {
        if (!frame_test(i)) {
            frame_set(i);
            g_free_frames--;
            g_last_alloc_search_idx = i + 1;
            return frame_idx_to_addr(i);
        }
    }

    // 2) Baştan bir daha tarayalım
    for (uint64_t i = 0; i < g_last_alloc_search_idx; ++i) {
        if (!frame_test(i)) {
            frame_set(i);
            g_free_frames--;
            g_last_alloc_search_idx = i + 1;
            return frame_idx_to_addr(i);
        }
    }

    return 0; // OOM
}

void phys_free_frame(uint64_t phys_addr)
{
    uint64_t idx = addr_to_frame_idx(phys_addr);
    if (idx >= AYKEN_MAX_FRAMES)
        return;

    if (frame_test(idx)) {
        frame_clear(idx);
        g_free_frames++;

        if (idx < g_last_alloc_search_idx)
            g_last_alloc_search_idx = idx;
    }
}



// ===========================================================================
//  YENİ → MULTI-FRAME ALLOCATION (açıklamalı)
// ===========================================================================

/**
 * Birden fazla ardışık frame ayırır.
 * Bu, sayfa tabloları (PML4/PDPT/PD/PT) oluştururken VE
 * DMA gibi contiguous memory gerektiğinde çok önemlidir.
 *
 * @param count Kaç frame isteniyor (ör: 4 frame → 16KB)
 */
uint64_t phys_alloc_frames(uint64_t count)
{
    if (count == 0)
        return 0;

    if (count == 1)
        return phys_alloc_frame();

    uint64_t chain_start = 0;
    uint64_t chain_len   = 0;

    // Tüm frame bitmap’ini tara
    for (uint64_t i = 0; i < AYKEN_MAX_FRAMES; ++i)
    {
        if (!frame_test(i)) {
            // frame boş → zincir genişliyor
            if (chain_len == 0)
                chain_start = i;

            chain_len++;

            if (chain_len == count) {
                uint64_t phys_start_addr = frame_idx_to_addr(chain_start);

                // Tüm zinciri rezerve et
                for (uint64_t f = chain_start; f < chain_start + count; f++) {
                    frame_set(f);
                    g_free_frames--;
                }

                g_last_alloc_search_idx = chain_start + count;
                return phys_start_addr;
            }
        }
        else {
            // zincir bozuldu
            chain_len = 0;
        }
    }

    // Yer yok → OOM
    return 0;
}



/**
 * Birden fazla ardışık frame free et.
 */
void phys_free_frames(uint64_t phys_addr, uint64_t count)
{
    if (count == 0)
        return;

    uint64_t start_idx = addr_to_frame_idx(phys_addr);

    for (uint64_t i = 0; i < count; i++) {
        uint64_t idx = start_idx + i;
        if (idx >= AYKEN_MAX_FRAMES)
            return;

        if (frame_test(idx)) {
            frame_clear(idx);
            g_free_frames++;

            if (idx < g_last_alloc_search_idx)
                g_last_alloc_search_idx = idx;
        }
    }
}



// ===========================================================================
//  Debug Fonksiyonu
// ===========================================================================

int phys_frame_is_used(uint64_t phys_addr)
{
    uint64_t idx = addr_to_frame_idx(phys_addr);
    if (idx >= AYKEN_MAX_FRAMES)
        return -1; // geçersiz adres

    return frame_test(idx);
}



// ===========================================================================
//  İSTATİSTİK
// ===========================================================================

uint64_t phys_get_total_frames(void) { return g_total_frames; }
uint64_t phys_get_free_frames(void)  { return g_free_frames; }
