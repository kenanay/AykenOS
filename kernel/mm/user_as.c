// kernel/mm/user_as.c
// ============================================================================
//  AykenOS User Address Space Management
//
//  Phase 10-A: Ring3 Execution - User Address Space Creation
//
//  This module provides mechanisms for creating isolated user address spaces
//  by allocating new PML4 root page tables and copying kernel half mappings.
//
//  Constitutional Compliance:
//   - Ring0 mechanism only (no policy decisions)
//   - Fail-closed design (reject invalid state immediately)
//   - Cleanup tracking for error handling (reverse order deallocation)
//   - Explicit USER bit clearing on kernel entries (security enforcement)
// ============================================================================

#include <stdint.h>
#include <stddef.h>
#include "../include/mm/user_as.h"
#include "../include/mm.h"
#include "../include/errno.h"

// Page table constants
#define PT_ENTRIES 512
#define KERNEL_HALF_START 256  // PML4 entries 256-511 are kernel half

// Page flag bits (from paging.c)
#define PAGE_PRESENT   (1ULL << 0)
#define PAGE_USER      (1ULL << 2)
#define PAGE_HUGE      (1ULL << 7)

// Maximum tracked allocations (Phase 10-A limit)
#define MAX_TRACKED_FRAMES 256
#define MAX_TRACKED_VADDRS 256

// Static storage for cleanup tracking (Phase 10-A: single process)
static uint64_t g_tracked_frames[MAX_TRACKED_FRAMES];
static uint64_t g_tracked_vaddrs[MAX_TRACKED_VADDRS];

/**
 * Create new user address space
 * 
 * Implementation notes:
 * 1. Allocate new PML4 frame
 * 2. Zero entire PML4 (all 512 entries)
 * 3. Copy kernel half (entries 256-511) from kernel PML4
 * 4. Mirror the current low-half kernel heap window supervisor-only so
 *    kernel-owned proc/stack metadata remains reachable under user CR3
 *    until the heap is promoted out of the low half
 * 5. For each copied kernel-half entry (256-511):
 *    - If entry is present: entry &= ~PAGE_USER (clear USER bit)
 *    - Preserve GLOBAL and NX bits as-is
 * 6. Store PML4 physical address in out_as->cr3_phys
 * 7. Store PML4 virtual address in out_as->pml4_virt
 * 
 * This ensures "trust no upstream state" principle for security.
 */
int user_as_create(user_as_t *out_as)
{
    if (!out_as) {
        return -EINVAL;
    }

    // Allocate new PML4 frame
    uint64_t new_pml4_phys = paging_alloc_page_table();
    if (!new_pml4_phys) {
        return -ENOMEM;
    }

    // Get virtual address for kernel access
    uint64_t *new_pml4_virt = (uint64_t *)paging_phys_to_virt(new_pml4_phys);

    // Zero entire PML4 (all 512 entries)
    for (int i = 0; i < PT_ENTRIES; i++) {
        new_pml4_virt[i] = 0;
    }

    // Get kernel PML4 physical address
    uint64_t kernel_pml4_phys = paging_get_kernel_pml4_phys();
    if (!kernel_pml4_phys) {
        // This should never happen if paging_init was called
        phys_free_frame(new_pml4_phys);
        return -EINVAL;
    }

    // Get kernel PML4 virtual address
    uint64_t *kernel_pml4_virt = (uint64_t *)paging_phys_to_virt(kernel_pml4_phys);

    // Copy kernel half (entries 256-511) from kernel PML4
    // For each copied entry: clear USER bit explicitly (security enforcement)
    for (int i = KERNEL_HALF_START; i < PT_ENTRIES; i++) {
        uint64_t entry = kernel_pml4_virt[i];
        
        // If entry is present, clear USER bit
        if (entry & PAGE_PRESENT) {
            entry &= ~PAGE_USER;  // Explicit clear: kernel entries MUST NOT have USER bit
        }
        
        // Copy entry (with USER bit cleared if present)
        new_pml4_virt[i] = entry;
    }

    if (paging_seed_user_kernel_heap_window(new_pml4_phys) != 0) {
        phys_free_frame(new_pml4_phys);
        return -EINVAL;
    }

    // Store PML4 addresses in output structure
    out_as->cr3_phys = new_pml4_phys;
    out_as->pml4_virt = new_pml4_virt;

    return 0;
}

/**
 * Initialize cleanup tracker
 */
void cleanup_tracker_init(cleanup_tracker_t *tracker)
{
    if (!tracker) {
        return;
    }

    // Use static storage for Phase 10-A
    tracker->frames = g_tracked_frames;
    tracker->frame_count = 0;
    tracker->frame_capacity = MAX_TRACKED_FRAMES;
    
    tracker->vaddrs = g_tracked_vaddrs;
    tracker->vaddr_count = 0;
    tracker->vaddr_capacity = MAX_TRACKED_VADDRS;
}

/**
 * Add allocated frame to cleanup tracker
 */
int cleanup_tracker_add_frame(cleanup_tracker_t *tracker, uint64_t phys_addr)
{
    if (!tracker) {
        return -EINVAL;
    }

    if (tracker->frame_count >= tracker->frame_capacity) {
        return -ENOMEM;  // Tracker full
    }

    tracker->frames[tracker->frame_count++] = phys_addr;
    return 0;
}

/**
 * Add mapped virtual address to cleanup tracker
 */
int cleanup_tracker_add_vaddr(cleanup_tracker_t *tracker, uint64_t vaddr)
{
    if (!tracker) {
        return -EINVAL;
    }

    if (tracker->vaddr_count >= tracker->vaddr_capacity) {
        return -ENOMEM;  // Tracker full
    }

    tracker->vaddrs[tracker->vaddr_count++] = vaddr;
    return 0;
}

/**
 * Cleanup user address space on error (reverse order)
 * 
 * Deallocates all tracked frames and unmaps all tracked pages
 * in reverse allocation order (last allocated, first freed).
 */
void user_as_cleanup(user_as_t *as, cleanup_tracker_t *tracker)
{
    if (!tracker) {
        return;
    }

    // Unmap all tracked virtual addresses (reverse order)
    for (size_t i = tracker->vaddr_count; i > 0; i--) {
        uint64_t vaddr = tracker->vaddrs[i - 1];
        paging_unmap(vaddr);
    }

    // Deallocate all tracked frames (reverse order)
    for (size_t i = tracker->frame_count; i > 0; i--) {
        uint64_t phys = tracker->frames[i - 1];
        phys_free_frame(phys);
    }

    // Deallocate user PML4 if allocated
    if (as && as->cr3_phys) {
        phys_free_frame(as->cr3_phys);
        as->cr3_phys = 0;
        as->pml4_virt = NULL;
    }

    // Reset tracker
    tracker->frame_count = 0;
    tracker->vaddr_count = 0;
}

static void user_as_destroy_table_recursive(uint64_t table_phys, uint32_t level)
{
    uint64_t *table;
    uint32_t i;

    if (table_phys == 0 || level == 0) {
        return;
    }

    table = (uint64_t *)paging_phys_to_virt(table_phys);
    if (!table) {
        return;
    }

    for (i = 0; i < PT_ENTRIES; ++i) {
        uint64_t entry = table[i];
        uint64_t child_phys;

        if ((entry & PAGE_PRESENT) == 0) {
            continue;
        }

        child_phys = entry & AYKEN_PTE_ADDR_MASK;
        if (child_phys == 0) {
            table[i] = 0;
            continue;
        }

        if (level == 1) {
            if (entry & PAGE_USER) {
                uint8_t *leaf = (uint8_t *)paging_phys_to_virt(child_phys);
                if (leaf) {
                    __builtin_memset(leaf, 0, AYKEN_FRAME_SIZE);
                }
                phys_free_frame(child_phys);
            }
            table[i] = 0;
            continue;
        }

        if (entry & PAGE_HUGE) {
            table[i] = 0;
            continue;
        }

        user_as_destroy_table_recursive(child_phys, level - 1);
        phys_free_frame(child_phys);
        table[i] = 0;
    }
}

void user_as_destroy_lower_half(user_as_t *as)
{
    uint64_t *pml4;
    uint32_t i;

    if (!as || as->cr3_phys == 0) {
        return;
    }

    pml4 = as->pml4_virt;
    if (!pml4) {
        pml4 = (uint64_t *)paging_phys_to_virt(as->cr3_phys);
        as->pml4_virt = pml4;
    }
    if (!pml4) {
        return;
    }

    for (i = 0; i < KERNEL_HALF_START; ++i) {
        uint64_t entry = pml4[i];
        uint64_t child_phys;

        if ((entry & PAGE_PRESENT) == 0) {
            continue;
        }

        child_phys = entry & AYKEN_PTE_ADDR_MASK;
        if (child_phys == 0) {
            pml4[i] = 0;
            continue;
        }

        user_as_destroy_table_recursive(child_phys, 3);
        phys_free_frame(child_phys);
        pml4[i] = 0;
    }
}

void user_as_destroy_root(user_as_t *as)
{
    if (!as) {
        return;
    }

    if (as->cr3_phys) {
        phys_free_frame(as->cr3_phys);
        as->cr3_phys = 0;
        as->pml4_virt = NULL;
    }
}

/**
 * Destroy user address space
 *
 * Destroy lower-half user mappings and page tables, then free the root PML4.
 */
void user_as_destroy(user_as_t *as)
{
    user_as_destroy_lower_half(as);
    user_as_destroy_root(as);
}
