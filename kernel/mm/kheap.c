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
#include "../arch/x86_64/port_io.h"

// ---------------------------------------------------------------------------
// Heap adres aralığı
// ---------------------------------------------------------------------------
//
// Not: KERNEL_VIRT_BASE ayken.h içinde tanımlı olmalı.
// Örn: #define KERNEL_VIRT_BASE 0xFFFFFFFF80000000ULL
//
// Burada heap'i kernel sanal adres alanında biraz yukarıdan başlatıyoruz.

// Alignment (8 veya 16 byte yeterli)
#define KHEAP_ALIGN        16ULL

static inline uint64_t align_up(uint64_t x, uint64_t a)
{
    return (x + a - 1) & ~(a - 1);
}

static inline void kheap_dbg(char c)
{
    outb(0xE9, (uint8_t)c);
}

static void kheap_dump_mapping(const char *tag, uint64_t va)
{
    uint64_t pml4_phys = paging_get_kernel_pml4_phys();
    uint64_t active_cr3 = 0;
    uint64_t pml4e = 0;
    uint64_t pdpte = 0;
    uint64_t pde = 0;
    uint64_t pte = 0;
    uint64_t *pml4 = NULL;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));

    if (pml4_phys != 0) {
        pml4 = (uint64_t *)paging_phys_to_virt(pml4_phys);
    }

    if (pml4 != NULL) {
        uint16_t pml4_i = (uint16_t)((va >> 39) & 0x1FF);
        uint16_t pdpt_i = (uint16_t)((va >> 30) & 0x1FF);
        uint16_t pd_i = (uint16_t)((va >> 21) & 0x1FF);
        uint16_t pt_i = (uint16_t)((va >> 12) & 0x1FF);

        pml4e = pml4[pml4_i];
        if (pml4e & AYKEN_PTE_PRESENT) {
            uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);
            if (pdpt != NULL) {
                pdpte = pdpt[pdpt_i];
                if ((pdpte & AYKEN_PTE_PRESENT) && ((pdpte & (1ULL << 7)) == 0)) {
                    uint64_t *pd = (uint64_t *)paging_phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);
                    if (pd != NULL) {
                        pde = pd[pd_i];
                        if ((pde & AYKEN_PTE_PRESENT) && ((pde & (1ULL << 7)) == 0)) {
                            uint64_t *pt = (uint64_t *)paging_phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);
                            if (pt != NULL) {
                                pte = pt[pt_i];
                            }
                        }
                    }
                }
            }
        }
    }

    fb_print("[kheap] ");
    fb_print(tag ? tag : "mapping");
    fb_print(" va=");
    fb_print_hex64(va);
    fb_print(" cr3=");
    fb_print_hex64(active_cr3);
    fb_print(" root=");
    fb_print_hex64(pml4_phys);
    fb_print(" pml4e=");
    fb_print_hex64(pml4e);
    fb_print(" pdpte=");
    fb_print_hex64(pdpte);
    fb_print(" pde=");
    fb_print_hex64(pde);
    fb_print(" pte=");
    fb_print_hex64(pte);
    fb_print("\n");
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
    kheap_dbg('h');
    fb_print("[kheap] Initializing kernel heap...\n");

    // 1) Heap aralığını sayfalara böl
    uint64_t heap_pages = AYKEN_KHEAP_INITIAL_SIZE / AYKEN_FRAME_SIZE;
    if (AYKEN_KHEAP_INITIAL_SIZE % AYKEN_FRAME_SIZE)
        heap_pages++;

    uint64_t cur_virt = AYKEN_KHEAP_START;

    for (uint64_t i = 0; i < heap_pages; ++i) {
        if (paging_get_phys(cur_virt) != 0) {
            cur_virt += AYKEN_FRAME_SIZE;
            continue;
        }
        if (i == 0)
            kheap_dbg('1');
        uint64_t phys = phys_alloc_frame();
        if (!phys) {
            kheap_dbg('!');
            fb_print("[kheap] ERROR: phys_alloc_frame failed while setting up heap.\n");
            return;
        }

        // Kernel sayfası: varsayılan flags = 0 → paging_map_page içinde
        // AYKEN_PTE_KERNEL_FLAGS eklenecek.
        paging_map_page(cur_virt, phys, 0);

        cur_virt += AYKEN_FRAME_SIZE;
    }

    kheap_dbg('m');
    if (paging_get_phys(AYKEN_KHEAP_START) == 0) { kheap_dbg('X');
        for (;;) __asm__ volatile("hlt"); }
    kheap_dump_mapping("post-map", AYKEN_KHEAP_START);
    paging_load_cr3(paging_get_kernel_pml4_phys());
    kheap_dump_mapping("post-cr3", AYKEN_KHEAP_START);
    kheap_dbg('M');
    kheap_dbg('a');
    volatile uint64_t *p = (volatile uint64_t *)AYKEN_KHEAP_START;
    volatile uint64_t tmp = *p;
    (void)tmp;
    kheap_dbg('A');
    kheap_dbg('b');
    *p = 0x1122334455667788ULL;
    kheap_dbg('B');
    // 2) Tek büyük boş blok oluştur
    kheap_head = (kheap_block_t *)AYKEN_KHEAP_START;
    kheap_head->size = (heap_pages * AYKEN_FRAME_SIZE) - sizeof(kheap_block_t);
    kheap_head->free = 1;
    kheap_head->next = NULL;

    kheap_dbg('H');
    fb_print("[kheap] Heap initialized at ");
    fb_print_hex64((uint64_t)AYKEN_KHEAP_START);
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
