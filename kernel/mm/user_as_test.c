// kernel/mm/user_as_test.c
// ============================================================================
//  AykenOS User Address Space Tests (Phase 10-A)
//
//  Basic validation tests for user address space creation.
//  Property-based tests are optional for Phase 10-A and recommended for 10-B.
// ============================================================================

#include <stdint.h>
#include "../include/mm/user_as.h"
#include "../include/mm.h"
#include "../include/errno.h"
#include "../drivers/console/fb_console.h"

// Page flag bits
#define PAGE_PRESENT   (1ULL << 0)
#define PAGE_USER      (1ULL << 2)

// PML4 constants
#define PT_ENTRIES 512
#define KERNEL_HALF_START 256

/**
 * Test: User address space creation
 * 
 * Validates:
 * - PML4 allocation succeeds
 * - Kernel half (entries 256-511) is copied
 * - User half (entries 0-255) is initially unmapped
 * - Kernel entries do NOT have USER bit set
 */
static void test_user_as_creation(void)
{
    fb_print("[TEST] User Address Space Creation\n");

    user_as_t as;
    int result = user_as_create(&as);

    if (result != 0) {
        fb_print("[TEST] FAIL: user_as_create returned error: ");
        fb_print_hex64((uint64_t)result);
        fb_print("\n");
        return;
    }

    if (as.cr3_phys == 0) {
        fb_print("[TEST] FAIL: cr3_phys is 0\n");
        return;
    }

    if (as.pml4_virt == NULL) {
        fb_print("[TEST] FAIL: pml4_virt is NULL\n");
        return;
    }

    fb_print("[TEST] User PML4 allocated at phys=0x");
    fb_print_hex64(as.cr3_phys);
    fb_print("\n");

    // Get kernel PML4 for comparison
    uint64_t kernel_pml4_phys = paging_get_kernel_pml4_phys();
    uint64_t *kernel_pml4_virt = (uint64_t *)paging_phys_to_virt(kernel_pml4_phys);

    // Validate user half (entries 0-255) is initially unmapped
    int user_half_ok = 1;
    for (int i = 0; i < KERNEL_HALF_START; i++) {
        if (as.pml4_virt[i] != 0) {
            fb_print("[TEST] FAIL: User half entry ");
            fb_print_hex64((uint64_t)i);
            fb_print(" is not zero: 0x");
            fb_print_hex64(as.pml4_virt[i]);
            fb_print("\n");
            user_half_ok = 0;
            break;
        }
    }

    if (user_half_ok) {
        fb_print("[TEST] PASS: User half (entries 0-255) is initially unmapped\n");
    }

    // Validate kernel half (entries 256-511) is copied
    int kernel_half_ok = 1;
    int user_bit_clear_ok = 1;
    int present_count = 0;

    for (int i = KERNEL_HALF_START; i < PT_ENTRIES; i++) {
        uint64_t kernel_entry = kernel_pml4_virt[i];
        uint64_t user_entry = as.pml4_virt[i];

        // If kernel entry is present, check copy
        if (kernel_entry & PAGE_PRESENT) {
            present_count++;

            // Check if USER bit is cleared in user PML4
            if (user_entry & PAGE_USER) {
                fb_print("[TEST] FAIL: Kernel entry ");
                fb_print_hex64((uint64_t)i);
                fb_print(" has USER bit set in user PML4\n");
                user_bit_clear_ok = 0;
                break;
            }

            // Check if entry matches (except USER bit)
            uint64_t kernel_entry_no_user = kernel_entry & ~PAGE_USER;
            if (user_entry != kernel_entry_no_user) {
                fb_print("[TEST] FAIL: Kernel entry ");
                fb_print_hex64((uint64_t)i);
                fb_print(" mismatch\n");
                fb_print("  Kernel: 0x");
                fb_print_hex64(kernel_entry);
                fb_print("\n  User:   0x");
                fb_print_hex64(user_entry);
                fb_print("\n");
                kernel_half_ok = 0;
                break;
            }
        } else {
            // Non-present entries should remain non-present
            if (user_entry != 0) {
                fb_print("[TEST] FAIL: Non-present kernel entry ");
                fb_print_hex64((uint64_t)i);
                fb_print(" is non-zero in user PML4: 0x");
                fb_print_hex64(user_entry);
                fb_print("\n");
                kernel_half_ok = 0;
                break;
            }
        }
    }

    if (kernel_half_ok) {
        fb_print("[TEST] PASS: Kernel half (entries 256-511) copied correctly (");
        fb_print_hex64((uint64_t)present_count);
        fb_print(" present entries)\n");
    }

    if (user_bit_clear_ok) {
        fb_print("[TEST] PASS: USER bit cleared on all kernel entries\n");
    }

    // Cleanup
    user_as_destroy(&as);

    if (user_half_ok && kernel_half_ok && user_bit_clear_ok) {
        fb_print("[TEST] User Address Space Creation: PASS\n");
    } else {
        fb_print("[TEST] User Address Space Creation: FAIL\n");
    }
}

/**
 * Test: Cleanup tracker initialization
 */
static void test_cleanup_tracker(void)
{
    fb_print("[TEST] Cleanup Tracker Initialization\n");

    cleanup_tracker_t tracker;
    cleanup_tracker_init(&tracker);

    if (tracker.frames == NULL) {
        fb_print("[TEST] FAIL: frames array is NULL\n");
        return;
    }

    if (tracker.vaddrs == NULL) {
        fb_print("[TEST] FAIL: vaddrs array is NULL\n");
        return;
    }

    if (tracker.frame_count != 0) {
        fb_print("[TEST] FAIL: frame_count is not 0\n");
        return;
    }

    if (tracker.vaddr_count != 0) {
        fb_print("[TEST] FAIL: vaddr_count is not 0\n");
        return;
    }

    fb_print("[TEST] Cleanup Tracker Initialization: PASS\n");
}

/**
 * Run all user address space tests
 */
void test_user_as(void)
{
    fb_print("\n=== User Address Space Tests (Phase 10-A) ===\n");
    
    test_user_as_creation();
    test_cleanup_tracker();
    
    fb_print("=== User Address Space Tests Complete ===\n\n");
}
