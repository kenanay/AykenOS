#ifndef AYKEN_USER_AS_H
#define AYKEN_USER_AS_H

#include <stdint.h>
#include <stddef.h>

// User address space descriptor
typedef struct {
    uint64_t cr3_phys;        // Physical address of PML4
    uint64_t *pml4_virt;      // Virtual address of PML4 (for kernel access)
} user_as_t;

// Cleanup tracking for error handling
typedef struct {
    uint64_t *frames;         // Array of allocated physical frames
    size_t frame_count;       // Number of allocated frames
    size_t frame_capacity;    // Capacity of frames array
    uint64_t *vaddrs;         // Array of mapped virtual addresses
    size_t vaddr_count;       // Number of mapped pages
    size_t vaddr_capacity;    // Capacity of vaddrs array
} cleanup_tracker_t;

/**
 * Create new user address space
 * Allocates a fresh PML4 root, copies the kernel half (entries 256-511),
 * and seeds the temporary supervisor-only low-half heap compatibility
 * window required by the current kmalloc/proc metadata placement.
 * Ensures copied kernel-half entries do NOT have USER bit set.
 * 
 * @param out_as Output parameter for user address space descriptor
 * @return 0 on success, -ENOMEM on allocation failure
 */
int user_as_create(user_as_t *out_as);

/**
 * Initialize cleanup tracker
 * 
 * @param tracker Cleanup tracker to initialize
 */
void cleanup_tracker_init(cleanup_tracker_t *tracker);

/**
 * Add allocated frame to cleanup tracker
 * 
 * @param tracker Cleanup tracker
 * @param phys_addr Physical address of allocated frame
 * @return 0 on success, -ENOMEM if tracker is full
 */
int cleanup_tracker_add_frame(cleanup_tracker_t *tracker, uint64_t phys_addr);

/**
 * Add mapped virtual address to cleanup tracker
 * 
 * @param tracker Cleanup tracker
 * @param vaddr Virtual address of mapped page
 * @return 0 on success, -ENOMEM if tracker is full
 */
int cleanup_tracker_add_vaddr(cleanup_tracker_t *tracker, uint64_t vaddr);

/**
 * Cleanup user address space on error (reverse order)
 * Deallocates all tracked frames and unmaps all tracked pages
 * 
 * @param as User address space descriptor
 * @param tracker Cleanup tracker with allocations to clean up
 */
void user_as_cleanup(user_as_t *as, cleanup_tracker_t *tracker);

/**
 * Destroy all user-half mappings, user-owned leaf frames, and page-table
 * hierarchy below the root PML4. Kernel-half entries are preserved.
 *
 * @param as User address space descriptor
 */
void user_as_destroy_lower_half(user_as_t *as);

/**
 * Destroy only the root PML4 frame for a user address space.
 *
 * @param as User address space descriptor
 */
void user_as_destroy_root(user_as_t *as);

/**
 * Destroy user address space
 * 
 * @param as User address space descriptor
 */
void user_as_destroy(user_as_t *as);

#endif // AYKEN_USER_AS_H
