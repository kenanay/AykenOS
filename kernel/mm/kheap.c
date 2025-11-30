// kernel/mm/kheap.c
// ============================================================================
//  AykenOS Kernel Heap (kmalloc / kfree)
//
//  - KHEAP_START adresinden itibaren belirli bir sanal aralığı "heap" olarak
//    kullanır.
//  - Bu aralığı phys_alloc_frame() + paging_map_page() ile fiziksel RAM'e map eder.
//  - Üzerinde basit bir free-list tabanlı allocator (first-fit) çalışır.
// ============================================================================

#include <stdint.h>
#include <stddef.h>
#include "../include/ayken.h"
#include "../include/mm.h"
#include "../drivers/console/fb_console.h"

// ---------------------------------------------------------------------------
// Heap adres aralığı
// ---------------------------------------------------------------------------
//
// Not: KERNEL_VIRT_BASE ayken.h içinde tanımlı olmalı.
// Örn: #define KERNEL_VIRT_BASE 0xFFFFFFFF80000000ULL
//
// Burada heap'i kernel sanal adres alanında biraz yukarıdan başlatıyoruz.

#define KHEAP_START        (KERNEL_VIRT_BASE + 0x01000000ULL)   // +16MB offset
#define KHEAP_INITIAL_SIZE (16ULL * 1024ULL * 1024ULL)          // 16 MiB heap

// Alignment (8 veya 16 byte yeterli)
#define KHEAP_ALIGN        16ULL

static inline uint64_t align_up(uint64_t x, uint64_t a)
{
    return (x + a - 1) & ~(a - 1);
}

// ---------------------------------------------------------------------------
// Heap blok yapısı
// ---------------------------------------------------------------------------
//
// [block_header][kullanıcı verisi ... ]
//
// block_header:
//   size = bu bloktaki "veri" alanının uzunluğu (header hariç)
//   free = 1 ise boş, 0 ise dolu
//   next = linked list'te bir sonraki blok
// ---------------------------------------------------------------------------

typedef struct kheap_block {
    uint64_t size;
    int      free;
    struct kheap_block *next;
} kheap_block_t;

static kheap_block_t *kheap_head = NULL;


// ============================================================================
//  kheap_init
//  - KHEAP_START'tan itibaren KHEAP_INITIAL_SIZE kadar sanal alanı
//    fiziksel frame'lerle doldurur ve tek büyük boş blok oluşturur.
// ============================================================================

void kheap_init(void)
{
    fb_print("[kheap] Initializing kernel heap...\n");

    // 1) Heap aralığını sayfalara böl
    uint64_t heap_pages = KHEAP_INITIAL_SIZE / AYKEN_FRAME_SIZE;
    if (KHEAP_INITIAL_SIZE % AYKEN_FRAME_SIZE)
        heap_pages++;

    uint64_t cur_virt = KHEAP_START;

    for (uint64_t i = 0; i < heap_pages; ++i) {
        uint64_t phys = phys_alloc_frame();
        if (!phys) {
            fb_print("[kheap] ERROR: phys_alloc_frame failed while setting up heap.\n");
            return;
        }

        // Kernel sayfası: varsayılan flags = 0 → paging_map_page içinde
        // AYKEN_PTE_KERNEL_FLAGS eklenecek.
        paging_map_page(cur_virt, phys, 0);

        cur_virt += AYKEN_FRAME_SIZE;
    }

    // 2) Tek büyük boş blok oluştur
    kheap_head = (kheap_block_t *)KHEAP_START;
    kheap_head->size = (heap_pages * AYKEN_FRAME_SIZE) - sizeof(kheap_block_t);
    kheap_head->free = 1;
    kheap_head->next = NULL;

    fb_print("[kheap] Heap initialized at ");
    fb_print_hex64((uint64_t)KHEAP_START);
    fb_print(" size=");
    fb_print_hex64(kheap_head->size);
    fb_print("\n");
}


// ============================================================================
//  kmalloc
//  - Basit first-fit algoritması
//  - Gerekirse blokları böler
// ============================================================================

void *kmalloc(uint64_t size)
{
    if (size == 0 || !kheap_head)
        return NULL;

    // Alignment uygulayalım
    size = align_up(size, KHEAP_ALIGN);

    kheap_block_t *current = kheap_head;

    while (current) {
        if (current->free && current->size >= size) {
            // Gerekirse bloğu böl
            uint64_t remaining = current->size - size;

            if (remaining > sizeof(kheap_block_t) + KHEAP_ALIGN) {
                // Yeni bir blok oluştur
                uint8_t *block_end = (uint8_t *)current + sizeof(kheap_block_t) + size;
                kheap_block_t *new_block = (kheap_block_t *)block_end;

                new_block->size = remaining - sizeof(kheap_block_t);
                new_block->free = 1;
                new_block->next = current->next;

                current->size = size;
                current->next = new_block;
            }

            current->free = 0;

            // Kullanıcıya dönecek adres: header'dan sonraki alan
            return (void *)((uint8_t *)current + sizeof(kheap_block_t));
        }

        current = current->next;
    }

    // Şimdilik heap genişletmiyoruz; ileride "heap grow" eklenebilir.
    fb_print("[kheap] WARNING: kmalloc out of memory.\n");
    return NULL;
}


// ============================================================================
//  kfree
//  - Bloğu free yapar, bitişik boş bloklarla birleştirir (coalesce).
// ============================================================================

void kfree(void *ptr)
{
    if (!ptr)
        return;

    // Pointer'ı header'a geri çek
    kheap_block_t *block = (kheap_block_t *)((uint8_t *)ptr - sizeof(kheap_block_t));
    block->free = 1;

    // Bitişik boş blokları birleştir
    kheap_block_t *current = kheap_head;

    while (current) {
        if (current->free) {
            // Sonraki blok da boşsa birleştir
            while (current->next && current->next->free) {
                current->size += sizeof(kheap_block_t) + current->next->size;
                current->next = current->next->next;
            }
        }
        current = current->next;
    }
}
