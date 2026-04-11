// kernel/sys/syscall_v2.c
// AykenOS Phase 2.1 - Execution-Centric Syscall Implementation
//
// This file implements the execution-centric syscall interface that provides
// mechanism-only implementations, delegating policy decisions to Ring3 components.
// This aligns with AykenOS's philosophy of minimal Ring0 and capability-based security.
//
// Requirements: FR-2.1.1, FR-2.1.2 - Execution-centric syscalls with capability system

#include "syscall_v2.h"
#include "../include/sys_v2_abi_lock.h"
#include "../drivers/console/fb_console.h"
#include "../include/execution_slot.h"
#include "../include/proc.h"
#include "../sched/sched.h"
#include "../arch/x86_64/cpu.h"
#include "../arch/x86_64/timer.h"
#include "../arch/x86_64/port_io.h"
#include "../include/gdt_idt.h"
#include "../include/mm.h"
#include "../include/capability.h"
#include "../include/barrier.h"
#include "../include/alias_registry.h"
#include "../include/errno.h"
#include <stddef.h>

// ============================================================================
// GLOBAL STATE (Minimal Ring0 State)
// ============================================================================
//
// Ring0 maintains only the minimal state necessary for mechanism implementation.
// All policy decisions and complex state management are delegated to Ring3.

static uint8_t debug_putchar_marker_progress[MAX_PROCS];

// Gate-3: Ring3 runtime validation marker tracking
static uint8_t gate3_ring3_marker_progress[MAX_PROCS];

#define SYSCALL_V2_USER_MARKER "[U][SYSCALL_OK]"
#define SYSCALL_V2_KERNEL_MARKER "[[AYKEN_SYSCALL_V2_OK]]\n"

// Gate-3: Ring3 runtime proof marker
#define GATE3_RING3_USER_MARKER "R3OK"
#define GATE3_RING3_KERNEL_MARKER "[[AYKEN_RING3_OK]]\n"

static void sys_v2_debugcon_write_string(const char *text)
{
    if (!text) {
        return;
    }
    while (*text) {
        outb(0xE9, (uint8_t)*text++);
    }
}

static void sys_v2_debug_putchar_note_marker(uint8_t character)
{
    extern proc_t *current_proc;
    const char *expected = SYSCALL_V2_USER_MARKER;
    const char *gate3_expected = GATE3_RING3_USER_MARKER;
    int pid_slot;
    uint8_t progress, gate3_progress;

    if (!current_proc || current_proc->pid <= 0 || current_proc->pid > MAX_PROCS) {
        return;
    }

    pid_slot = current_proc->pid - 1;
    progress = debug_putchar_marker_progress[pid_slot];
    gate3_progress = gate3_ring3_marker_progress[pid_slot];

    // Track original syscall marker
    if ((char)character == expected[progress]) {
        progress++;
    } else if ((char)character == expected[0]) {
        progress = 1;
    } else {
        progress = 0;
    }

    if (expected[progress] == '\0') {
        /* Emit a deterministic kernel-origin marker for hosted CI parsing. */
        sys_v2_debugcon_write_string(SYSCALL_V2_KERNEL_MARKER);
        progress = 0;
    }

    // Gate-3: Track Ring3 runtime proof marker
    if ((char)character == gate3_expected[gate3_progress]) {
        gate3_progress++;
    } else if ((char)character == gate3_expected[0]) {
        gate3_progress = 1;
    } else {
        gate3_progress = 0;
    }

    if (gate3_expected[gate3_progress] == '\0') {
        /* Gate-3: Emit Ring3 runtime proof marker */
        sys_v2_debugcon_write_string(GATE3_RING3_KERNEL_MARKER);
        gate3_progress = 0;
    }

    debug_putchar_marker_progress[pid_slot] = progress;
    gate3_ring3_marker_progress[pid_slot] = gate3_progress;
}

typedef uint64_t (*sys_v2_dispatch_fn_t)(uint64_t, uint64_t, uint64_t, uint64_t);

static uint64_t sys_v2_dispatch_map_memory(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a4;
    return sys_v2_map_memory(a1, a2, a3);
}

static uint64_t sys_v2_dispatch_unmap_memory(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_unmap_memory(a1, a2);
}

static uint64_t sys_v2_dispatch_switch_context(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_switch_context(a1, a2);
}

static uint64_t sys_v2_dispatch_submit_execution(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a4;
    return sys_v2_submit_execution((void *)a1, a2, a3);
}

static uint64_t sys_v2_dispatch_wait_result(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_wait_result(a1, a2);
}

static uint64_t sys_v2_dispatch_interrupt_return(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_interrupt_return(a1, a2);
}

static uint64_t sys_v2_dispatch_time_query(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_time_query(a1, (uint64_t *)a2);
}

static uint64_t sys_v2_dispatch_capability_bind(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_capability_bind(a1, (capability_token_t *)a2);
}

static uint64_t sys_v2_dispatch_capability_revoke(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a2;
    (void)a3;
    (void)a4;
    return sys_v2_capability_revoke(a1);
}

static uint64_t sys_v2_dispatch_exit(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a2;
    (void)a3;
    (void)a4;
    return sys_v2_exit(a1);
}

static uint64_t sys_v2_dispatch_debug_putchar(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a2;
    (void)a3;
    (void)a4;
    return sys_v2_debug_putchar(a1);
}

static uint64_t sys_v2_dispatch_complete_execution(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a3;
    (void)a4;
    return sys_v2_complete_execution(a1, a2);
}

static uint64_t sys_v2_dispatch_device_operation(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    return sys_v2_device_operation(a1, a2, (uint64_t *)a3, a4);
}

static uint64_t sys_v2_dispatch_external_call(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    (void)a4;
    return sys_v2_external_call(a1, (uint64_t *)a2, a3);
}

static uint64_t sys_v2_dispatch_abdf_operation(uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4)
{
    return sys_v2_abdf_operation(a1, a2, (uint64_t *)a3, a4);
}

static const sys_v2_dispatch_fn_t sys_v2_dispatch_table[SYS_V2_NR] = {
    [SYS_V2_MAP_MEMORY] = sys_v2_dispatch_map_memory,
    [SYS_V2_UNMAP_MEMORY] = sys_v2_dispatch_unmap_memory,
    [SYS_V2_SWITCH_CONTEXT] = sys_v2_dispatch_switch_context,
    [SYS_V2_SUBMIT_EXECUTION] = sys_v2_dispatch_submit_execution,
    [SYS_V2_WAIT_RESULT] = sys_v2_dispatch_wait_result,
    [SYS_V2_INTERRUPT_RETURN] = sys_v2_dispatch_interrupt_return,
    [SYS_V2_TIME_QUERY] = sys_v2_dispatch_time_query,
    [SYS_V2_CAPABILITY_BIND] = sys_v2_dispatch_capability_bind,
    [SYS_V2_CAPABILITY_REVOKE] = sys_v2_dispatch_capability_revoke,
    [SYS_V2_EXIT] = sys_v2_dispatch_exit,
    [SYS_V2_DEBUG_PUTCHAR] = sys_v2_dispatch_debug_putchar,
    [SYS_V2_COMPLETE_EXECUTION] = sys_v2_dispatch_complete_execution,
    [SYS_V2_DEVICE_OPERATION] = sys_v2_dispatch_device_operation,
    [SYS_V2_EXTERNAL_CALL] = sys_v2_dispatch_external_call,
    [SYS_V2_ABDF_OPERATION] = sys_v2_dispatch_abdf_operation,
};

_Static_assert(sizeof(sys_v2_dispatch_table) / sizeof(sys_v2_dispatch_table[0]) == SYS_V2_NR,
               "Dispatch table size does not match SYS_V2_NR");

/*
 * Constitutional syscall-exit contract:
 * all v2 handler exits must flow through deferred preemption completion.
 */
static inline uint64_t sys_v2_finalize_result(uint64_t result)
{
    if (sched_take_resched()) {
        sched_yield();
    }
    return result;
}

static int sys_v2_buffer_span_is_mapped(const void *buffer, uint64_t size)
{
    uint64_t start;
    uint64_t end_inclusive;
    uint64_t page;

    if (!buffer || size == 0) {
        return 0;
    }

    start = (uint64_t)buffer;
    if (start > UINT64_MAX - (size - 1)) {
        return 0;
    }

    end_inclusive = start + size - 1;
    page = start & ~(AYKEN_FRAME_SIZE - 1);

    for (;;) {
        if (paging_get_phys(page) == 0) {
            return 0;
        }
        if (page >= (end_inclusive & ~(AYKEN_FRAME_SIZE - 1))) {
            break;
        }
        if (page > UINT64_MAX - AYKEN_FRAME_SIZE) {
            return 0;
        }
        page += AYKEN_FRAME_SIZE;
    }

    return 1;
}

static proc_t *sys_v2_resolve_live_user_context(uint64_t context_id)
{
    proc_t *target_proc;

    if (context_id == 0 || context_id > (uint64_t)INT32_MAX) {
        return NULL;
    }

    target_proc = proc_find_by_pid((int)context_id);
    if (!target_proc) {
        return NULL;
    }

    if (target_proc->type != PROC_TYPE_USER || target_proc->state == PROC_ZOMBIE) {
        return NULL;
    }

    return target_proc;
}

static int sys_v2_completion_code_to_state(uint64_t completion_code,
                                           exec_slot_state_t *next_state)
{
    if (!next_state) {
        return -1;
    }

    switch (completion_code) {
    case EXEC_COMPLETION_COMPLETED:
        *next_state = EXEC_SLOT_COMPLETED;
        return 0;
    case EXEC_COMPLETION_FAILED:
        *next_state = EXEC_SLOT_FAILED;
        return 0;
    default:
        return -1;
    }
}

static int sys_v2_translate_map_access(uint64_t flags,
                                       uint64_t *pte_flags,
                                       uint32_t *capability_permissions)
{
    uint32_t access_bits = (uint32_t)(flags & (CAP_PERM_READ | CAP_PERM_WRITE | CAP_PERM_EXECUTE));

    if (!pte_flags || !capability_permissions) {
        return -1;
    }

    if (access_bits == 0 || (flags & ~(uint64_t)(CAP_PERM_READ | CAP_PERM_WRITE | CAP_PERM_EXECUTE)) != 0) {
        return -1;
    }

    *pte_flags = AYKEN_PTE_USER;
    if ((access_bits & CAP_PERM_WRITE) != 0) {
        *pte_flags |= AYKEN_PTE_WRITABLE;
    } else {
        *pte_flags |= AYKEN_PTE_READ_ONLY;
    }
    if ((access_bits & CAP_PERM_EXECUTE) == 0) {
        *pte_flags |= AYKEN_PTE_NO_EXEC;
    }

    *capability_permissions = access_bits;
    return 0;
}

static int sys_v2_generic_mapping_matches(uint64_t pte,
                                          uint64_t phys_addr,
                                          uint64_t pte_flags)
{
    if (pte == 0) {
        return 0;
    }
    if ((pte & AYKEN_PTE_ADDR_MASK) != phys_addr) {
        return 0;
    }
    if ((pte & AYKEN_PTE_USER) == 0) {
        return 0;
    }
    if ((pte_flags & AYKEN_PTE_WRITABLE) != 0) {
        if ((pte & AYKEN_PTE_WRITABLE) == 0) {
            return 0;
        }
    } else if ((pte & AYKEN_PTE_WRITABLE) != 0) {
        return 0;
    }
    if ((pte_flags & AYKEN_PTE_NO_EXEC) != 0) {
        if ((pte & AYKEN_PTE_NO_EXEC) == 0) {
            return 0;
        }
    } else if ((pte & AYKEN_PTE_NO_EXEC) != 0) {
        return 0;
    }

    return 1;
}

static int sys_v2_result_mapping_matches(uint64_t pte,
                                         uint64_t phys_addr,
                                         uint64_t map_flags)
{
    (void)map_flags;

    if (pte == 0) {
        return 0;
    }

    if ((pte & AYKEN_PTE_ADDR_MASK) != phys_addr) {
        return 0;
    }
    if ((pte & AYKEN_PTE_USER) == 0) {
        return 0;
    }
    if ((pte & AYKEN_PTE_WRITABLE) != 0) {
        return 0;
    }
    if ((pte & AYKEN_PTE_NO_EXEC) == 0) {
        return 0;
    }

    return 1;
}

static uint32_t sys_v2_result_frame_count_for_size(uint64_t result_size)
{
    if (result_size == 0) {
        return 0;
    }

    return (uint32_t)((result_size + (AYKEN_FRAME_SIZE - 1)) / AYKEN_FRAME_SIZE);
}

static void sys_v2_invalidate_local_page_if_active(uint64_t pml4_phys, uint64_t virt_addr)
{
    uint64_t active_cr3 = 0;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (pml4_phys & AYKEN_PTE_ADDR_MASK)) {
        __asm__ volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
    }
}

static int sys_v2_map_result_for_wait_locked(exec_slot_t *slot,
                                             proc_t *caller_proc,
                                             uint64_t *mapped_result_va)
{
    uint64_t desired_va;
    uint64_t desired_hash_va;
    uint64_t map_flags;
    uint32_t frame_count;
    uint32_t i;
    uint8_t mapped_pages[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES] = {0};
    uint8_t hash_mapped = 0;

    if (!slot || !caller_proc || !mapped_result_va) {
        return -1;
    }

    if (caller_proc->type != PROC_TYPE_USER || caller_proc->pml4_phys == 0) {
        return -1;
    }

    if (execution_slot_prepare_result_locked(slot) != 0) {
        return -1;
    }

    desired_va = slot->mapped_result_va;
    if (desired_va == 0) {
        desired_va = execution_slot_result_va_locked(slot);
    }
    if (desired_va == 0 ||
        slot->result_size == 0 ||
        slot->result_frame_count == 0 ||
        slot->hash_frame == 0 ||
        slot->hash_size != sizeof(ayken_execution_result_hash_v1_t) ||
        slot->hashed_size != slot->result_size) {
        return -1;
    }
    desired_hash_va = slot->mapped_hash_va;
    if (desired_hash_va == 0) {
        desired_hash_va = execution_slot_result_hash_va_locked(slot);
    }
    if (desired_hash_va == 0) {
        return -1;
    }

    map_flags = slot->result_map_flags;
    if (map_flags == 0) {
        map_flags = AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_EXEC;
    }

    frame_count = sys_v2_result_frame_count_for_size(slot->result_size);
    if (frame_count == 0 ||
        frame_count > AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES ||
        slot->result_frame_count != frame_count) {
        return -1;
    }

    for (i = 0; i < frame_count; ++i) {
        uint64_t page_va = desired_va + ((uint64_t)i * AYKEN_FRAME_SIZE);
        uint64_t page_phys = slot->result_frames[i];
        uint64_t pte;

        if (page_phys == 0) {
            return -1;
        }

        pte = paging_get_pte_in_pml4(caller_proc->pml4_phys, page_va);
        if (pte != 0 && !sys_v2_result_mapping_matches(pte, page_phys, map_flags)) {
            return -1;
        }
    }
    {
        uint64_t hash_pte = paging_get_pte_in_pml4(caller_proc->pml4_phys, desired_hash_va);

        if (hash_pte != 0 && !sys_v2_result_mapping_matches(hash_pte, slot->hash_frame, map_flags)) {
            return -1;
        }
    }

    for (i = 0; i < frame_count; ++i) {
        uint64_t page_va = desired_va + ((uint64_t)i * AYKEN_FRAME_SIZE);
        uint64_t page_phys = slot->result_frames[i];
        uint64_t pte = paging_get_pte_in_pml4(caller_proc->pml4_phys, page_va);

        if (pte != 0) {
            continue;
        }

        paging_map_page_in_pml4(caller_proc->pml4_phys,
                                page_va,
                                page_phys,
                                map_flags);
        sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys, page_va);
        mapped_pages[i] = 1;

        pte = paging_get_pte_in_pml4(caller_proc->pml4_phys, page_va);
        if (!sys_v2_result_mapping_matches(pte, page_phys, map_flags)) {
            uint32_t rollback_index;
            for (rollback_index = 0; rollback_index <= i; ++rollback_index) {
                if (!mapped_pages[rollback_index]) {
                    continue;
                }
                paging_unmap_in_pml4(caller_proc->pml4_phys,
                                     desired_va + ((uint64_t)rollback_index * AYKEN_FRAME_SIZE));
                sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys,
                                                       desired_va + ((uint64_t)rollback_index * AYKEN_FRAME_SIZE));
            }
            return -1;
        }
    }

    {
        uint64_t hash_pte = paging_get_pte_in_pml4(caller_proc->pml4_phys, desired_hash_va);

        if (hash_pte == 0) {
            paging_map_page_in_pml4(caller_proc->pml4_phys,
                                    desired_hash_va,
                                    slot->hash_frame,
                                    map_flags);
            sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys, desired_hash_va);
            hash_mapped = 1;

            hash_pte = paging_get_pte_in_pml4(caller_proc->pml4_phys, desired_hash_va);
            if (!sys_v2_result_mapping_matches(hash_pte, slot->hash_frame, map_flags)) {
                for (i = 0; i < frame_count; ++i) {
                    if (!mapped_pages[i]) {
                        continue;
                    }
                    paging_unmap_in_pml4(caller_proc->pml4_phys,
                                         desired_va + ((uint64_t)i * AYKEN_FRAME_SIZE));
                    sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys,
                                                           desired_va + ((uint64_t)i * AYKEN_FRAME_SIZE));
                }
                paging_unmap_in_pml4(caller_proc->pml4_phys, desired_hash_va);
                sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys, desired_hash_va);
                return -1;
            }
        }
    }

    if (execution_slot_record_result_mapping_locked(slot,
                                                    desired_va,
                                                    desired_hash_va,
                                                    map_flags) != 0) {
        for (i = 0; i < frame_count; ++i) {
            if (!mapped_pages[i]) {
                continue;
            }
            paging_unmap_in_pml4(caller_proc->pml4_phys,
                                 desired_va + ((uint64_t)i * AYKEN_FRAME_SIZE));
            sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys,
                                                   desired_va + ((uint64_t)i * AYKEN_FRAME_SIZE));
        }
        if (hash_mapped) {
            paging_unmap_in_pml4(caller_proc->pml4_phys, desired_hash_va);
            sys_v2_invalidate_local_page_if_active(caller_proc->pml4_phys, desired_hash_va);
        }
        return -1;
    }

    *mapped_result_va = desired_va;
    return 0;
}

// ============================================================================
// MEMORY MANAGEMENT SYSCALLS
// ============================================================================
//
// These syscalls provide memory mapping mechanisms without policy decisions.
// Ring3 components determine what should be mapped and when.

uint64_t sys_v2_map_memory(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags)
{
    capability_token_t *memory_cap;
    uint64_t execution_ctx;
    uint64_t pte_flags;
    uint32_t capability_permissions;
    uint64_t existing_pte;
    uint64_t map_id = 0;
    proc_t *current;
    int access_result;
#if defined(AYKEN_VALIDATION)
    int alias_record_result;
#endif

    if (virt_addr == 0 || phys_addr == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    if ((virt_addr & (AYKEN_FRAME_SIZE - 1)) != 0 ||
        (phys_addr & (AYKEN_FRAME_SIZE - 1)) != 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    if (sys_v2_translate_map_access(flags, &pte_flags, &capability_permissions) != 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    extern proc_t *current_proc;
    current = current_proc;
    if (current == NULL || current->type != PROC_TYPE_USER || current->pml4_phys == 0) {
        return ESYS_V2_NO_CAPABILITY;
    }

    /* FREEZE INVARIANT: Reject mapping if teardown has started
     * smp_rmb(): read teardown_started after all prior writes are visible
     * This prevents TOCTOU where Core 1 starts teardown while Core 2 is still mapping.
     */
#if defined(AYKEN_VALIDATION)
    smp_rmb();
    if (current->teardown_started == 1) {
        return ESYS_V2_INVALID_PARAM;
    }
#endif

    execution_ctx = (uint64_t)current->pid;
    memory_cap = capability_get_by_context(execution_ctx, CAPABILITY_RESOURCE_MEMORY);
    if (memory_cap == NULL) {
        fb_print("[syscall_v2] map_memory: DENIED - No memory capability for context ");
        fb_print_int(execution_ctx);
        fb_print("\n");
        return ESYS_V2_NO_CAPABILITY;
    }

    access_result = capability_check_resource_access(memory_cap,
                                                     phys_addr,
                                                     AYKEN_FRAME_SIZE,
                                                     capability_permissions);
    if (access_result != CAPABILITY_SUCCESS) {
        fb_print("[syscall_v2] map_memory: DENIED - Capability check failed\n");
        return ESYS_V2_NO_PERMISSION;
    }

    existing_pte = paging_get_pte_in_pml4(current->pml4_phys, virt_addr);
    if (existing_pte != 0 || proc_find_generic_mapping(current, virt_addr) != NULL) {
        return ESYS_V2_RESOURCE_BUSY;
    }

    paging_map_page_in_pml4(current->pml4_phys, virt_addr, phys_addr, pte_flags);
    sys_v2_invalidate_local_page_if_active(current->pml4_phys, virt_addr);

    existing_pte = paging_get_pte_in_pml4(current->pml4_phys, virt_addr);
    if (!sys_v2_generic_mapping_matches(existing_pte, phys_addr, pte_flags)) {
        paging_unmap_in_pml4(current->pml4_phys, virt_addr);
        return ESYS_V2_RESOURCE_BUSY;
    }

    if (proc_record_generic_mapping(current,
                                    virt_addr,
                                    phys_addr,
                                    flags,
                                    memory_cap->id,
                                    1,
                                    &map_id) != 0) {
        paging_unmap_in_pml4(current->pml4_phys, virt_addr);
        return ESYS_V2_RESOURCE_BUSY;
    }

#if defined(AYKEN_VALIDATION)
    /* TRANSACTIONAL CONTRACT: Registry record must succeed for mapping to be committed.
     * If alias_registry_record() fails, we MUST rollback the PTE to maintain
     * registry-page-table consistency. Partial commit is forbidden.
     * 
     * ROLLBACK DOĞRULAMA: After rollback, we verify PTE is actually zero to ensure
     * complete rollback. Partial rollback (PTE deleted but wrong error code) is more
     * dangerous than no rollback — it makes the system appear "clean" when it's not.
     */
    alias_record_result = alias_registry_record(&current->alias_reg, phys_addr, virt_addr);
    if (alias_record_result != 0) {
        /* Rollback: unmap PTE and remove from mapping_ledger */
        paging_unmap_in_pml4(current->pml4_phys, virt_addr);
        sys_v2_invalidate_local_page_if_active(current->pml4_phys, virt_addr);
        
        /* Remove from mapping_ledger */
        proc_mapping_entry_t removed_entry = {0};
        proc_remove_generic_mapping(current, virt_addr, &removed_entry);
        
        /* ROLLBACK VERIFICATION: Ensure PTE is actually zero after rollback */
        existing_pte = paging_get_pte_in_pml4(current->pml4_phys, virt_addr);
        if (existing_pte != 0) {
            /* KERNEL.SAFETY.CRITICAL: Rollback failed — system state is inconsistent */
            fb_print("[CRITICAL] sys_v2_map_memory: PTE rollback failed! va=0x");
            fb_print_hex(virt_addr);
            fb_print(" pte=0x");
            fb_print_hex(existing_pte);
            fb_print("\n");
        }
        
        /* Return appropriate error code based on alias_registry_record() result */
        if (alias_record_result == -ENOMEM) {
            return ESYS_V2_RESOURCE_BUSY;
        } else {
            /* -EINVAL or other error */
            return ESYS_V2_INVALID_PARAM;
        }
    }
#endif

    fb_print("[syscall_v2] map_memory: ctx=");
    fb_print_int(execution_ctx);
    fb_print(" virt=0x");
    fb_print_hex(virt_addr);
    fb_print(" phys=0x");
    fb_print_hex(phys_addr);
    fb_print(" map_id=");
    fb_print_int(map_id);
    fb_print(" cap_id=");
    fb_print_int(memory_cap->id);
    fb_print("\n");

    return ESYS_V2_SUCCESS;
}

uint64_t sys_v2_unmap_memory(uint64_t virt_addr, uint64_t size)
{
    proc_t *current;
    uint64_t page_count;
    uint64_t page;
    uint64_t execution_ctx;
    capability_token_t *memory_cap;

    if (virt_addr == 0 || size == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    if ((virt_addr & (AYKEN_FRAME_SIZE - 1)) != 0 ||
        (size & (AYKEN_FRAME_SIZE - 1)) != 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    extern proc_t *current_proc;
    current = current_proc;
    if (current == NULL || current->type != PROC_TYPE_USER || current->pml4_phys == 0) {
        return ESYS_V2_NO_CAPABILITY;
    }

    execution_ctx = (uint64_t)current->pid;
    memory_cap = capability_get_by_context(execution_ctx, CAPABILITY_RESOURCE_MEMORY);
    if (memory_cap == NULL) {
        return ESYS_V2_NO_CAPABILITY;
    }

    page_count = size / AYKEN_FRAME_SIZE;
    for (page = 0; page < page_count; ++page) {
        uint64_t page_va = virt_addr + (page * AYKEN_FRAME_SIZE);
        proc_mapping_entry_t *entry = proc_find_generic_mapping(current, page_va);

        if (entry == NULL || entry->owner_pid != execution_ctx) {
            return ESYS_V2_NO_PERMISSION;
        }
        if (entry->page_count != 1) {
            return ESYS_V2_INVALID_STATE;
        }
        if (capability_context_has_capability(execution_ctx, entry->capability_id) != CAPABILITY_SUCCESS) {
            return ESYS_V2_NO_CAPABILITY;
        }
    }

    for (page = 0; page < page_count; ++page) {
        uint64_t page_va = virt_addr + (page * AYKEN_FRAME_SIZE);
        proc_mapping_entry_t removed_entry = {0};
        paging_unmap_in_pml4(current->pml4_phys, page_va);
        sys_v2_invalidate_local_page_if_active(current->pml4_phys, page_va);
        if (proc_remove_generic_mapping(current, page_va, &removed_entry) != 0) {
            return ESYS_V2_INVALID_STATE;
        }
    }

    fb_print("[syscall_v2] unmap_memory: ctx=");
    fb_print_int(execution_ctx);
    fb_print(" virt=0x");
    fb_print_hex(virt_addr);
    fb_print(" size=0x");
    fb_print_hex(size);
    fb_print("\n");

    return ESYS_V2_SUCCESS;
}

// ============================================================================
// CONTEXT MANAGEMENT SYSCALLS
// ============================================================================
//
// Context switching mechanism - Ring0 provides the low-level mechanism,
// Ring3 determines scheduling policy and context selection.

uint64_t sys_v2_switch_context(uint64_t old_ctx_id, uint64_t new_ctx_id)
{
    // Validate context IDs
    if (old_ctx_id == 0 || new_ctx_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // SECURITY ENFORCEMENT: Check if current execution context has execution capability
    // This prevents privilege escalation by requiring explicit capability grants for context switching
    extern proc_t *current_proc;
    if (current_proc != NULL) {
        uint64_t execution_ctx = current_proc->pid;
        capability_token_t *exec_cap = capability_get_by_context(execution_ctx, CAPABILITY_RESOURCE_EXECUTION);
        
        if (exec_cap == NULL) {
            fb_print("[syscall_v2] switch_context: DENIED - No execution capability for context ");
            fb_print_int(execution_ctx);
            fb_print("\n");
            return ESYS_V2_NO_CAPABILITY;
        }
        
        // Check if capability allows context switching
        int permission_result = capability_check_permission(exec_cap, CAPABILITY_PERM_EXECUTE);
        if (permission_result != CAPABILITY_SUCCESS) {
            fb_print("[syscall_v2] switch_context: DENIED - Insufficient permissions\n");
            return ESYS_V2_NO_PERMISSION;
        }
        
        fb_print("[syscall_v2] switch_context: GRANTED via capability ID=");
        fb_print_int(exec_cap->id);
        fb_print("\n");
    }
    
    // Ring0 mechanism: Look up process structures by context ID (PID)
    proc_t *old_proc = proc_find_by_pid((int)old_ctx_id);
    proc_t *new_proc = proc_find_by_pid((int)new_ctx_id);
    
    // Validate that both processes exist
    if (!old_proc) {
        fb_print("[syscall_v2] switch_context: old context ");
        fb_print_int(old_ctx_id);
        fb_print(" not found\n");
        return ESYS_V2_CONTEXT_ERROR;
    }
    
    if (!new_proc) {
        fb_print("[syscall_v2] switch_context: new context ");
        fb_print_int(new_ctx_id);
        fb_print(" not found\n");
        return ESYS_V2_CONTEXT_ERROR;
    }
    
    // Ring0 mechanism: Validate process states
    if (new_proc->state != PROC_READY && new_proc->state != PROC_RUNNING) {
        fb_print("[syscall_v2] switch_context: new context ");
        fb_print_int(new_ctx_id);
        fb_print(" not ready (state=");
        fb_print_int(new_proc->state);
        fb_print(")\n");
        return ESYS_V2_CONTEXT_ERROR;
    }
    
    // Ring0 mechanism: Disable interrupts during context switch
    disable_interrupts();
    
    // Ring0 mechanism: Update process states
    if (old_proc->state == PROC_RUNNING) {
        old_proc->state = PROC_READY;
    }
    new_proc->state = PROC_RUNNING;
    
    // Ring0 mechanism: Update global current process pointer
    extern proc_t *current_proc;
    current_proc = new_proc;
    
    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions
    if (new_proc->context.rsp0) {
        gdt_set_kernel_stack(new_proc->context.rsp0);
    }
    
    // Ring0 mechanism: Load new process page tables
    paging_load_cr3(new_proc->context.cr3);
    
    // Ring0 mechanism: Perform the actual context switch
    context_switch(&old_proc->context, &new_proc->context);
    
    // Ring0 mechanism: Re-enable interrupts
    enable_interrupts();
    
    fb_print("[syscall_v2] switch_context: switched from ");
    fb_print_int(old_ctx_id);
    fb_print(" to ");
    fb_print_int(new_ctx_id);
    fb_print("\n");
    
    return ESYS_V2_SUCCESS;
}

// ============================================================================
// EXECUTION MANAGEMENT SYSCALLS
// ============================================================================
//
// BCIB execution submission and result waiting mechanisms.
// Ring0 provides execution tracking, Ring3 provides execution logic.

uint64_t sys_v2_submit_execution(void *bcib_graph, uint64_t graph_size, uint64_t context_id)
{
    execution_slot_guard_t slot_guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    exec_slot_t *slot = NULL;
    proc_t *target_proc = NULL;
    uint64_t owner_pid = 0;
    uint64_t execution_id;

    // Validate parameters
    if (bcib_graph == NULL || graph_size == 0 || context_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    if (graph_size > AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE) {
        return ESYS_V2_INVALID_PARAM;
    }

    if (!sys_v2_buffer_span_is_mapped(bcib_graph, graph_size)) {
        return ESYS_V2_INVALID_PARAM;
    }

    target_proc = sys_v2_resolve_live_user_context(context_id);
    if (!target_proc) {
        return ESYS_V2_CONTEXT_ERROR;
    }

    extern proc_t *current_proc;
    if (current_proc != NULL && current_proc->pid > 0) {
        owner_pid = (uint64_t)current_proc->pid;
    }

    execution_slot_enter_critical(&slot_guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_SUBMIT);

    slot = execution_slot_alloc_locked(owner_pid, context_id);
    if (!slot) {
        execution_slot_trace_scope_exit(&trace_scope);
        execution_slot_exit_critical(&slot_guard);
        return ESYS_V2_RESOURCE_BUSY;
    }

    slot->created_tick = timer_ticks();
    if (execution_slot_store_bcib_locked(slot, bcib_graph, graph_size) != 0) {
        execution_slot_release_locked(slot);
        execution_slot_trace_scope_exit(&trace_scope);
        execution_slot_exit_critical(&slot_guard);
        return ESYS_V2_RESOURCE_BUSY;
    }

    execution_slot_require_transition_locked(slot,
                                             EXEC_SLOT_CREATED,
                                             EXEC_SLOT_READY,
                                             "sys_v2_submit_execution");

    if (execution_slot_enqueue_locked(slot) != 0) {
        execution_slot_release_locked(slot);
        execution_slot_trace_scope_exit(&trace_scope);
        execution_slot_exit_critical(&slot_guard);
        return ESYS_V2_RESOURCE_BUSY;
    }

    execution_id = slot->execution_id;
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&slot_guard);

    fb_print("[syscall_v2] submit_execution: graph=0x");
    fb_print_hex((uint64_t)bcib_graph);
    fb_print(" size=");
    fb_print_int(graph_size);
    fb_print(" ctx=");
    fb_print_int(context_id);
    fb_print(" target_pid=");
    fb_print_int(target_proc->pid);
    fb_print(" exec_id=");
    fb_print_int(execution_id);
    fb_print("\n");
    
    return execution_id;
}

uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms)
{
    uint64_t owner_pid = 0;

    // Validate parameters
    if (execution_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    extern proc_t *current_proc;
    if (current_proc != NULL && current_proc->pid > 0) {
        owner_pid = (uint64_t)current_proc->pid;
    }

    for (;;) {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        uint64_t now_tick;
        uint64_t deadline_delta;
        uint64_t mapped_result_va = 0;
        void *wait_obj = NULL;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(execution_id);
        if (!slot) {
            execution_slot_exit_critical(&slot_guard);
            return ESYS_V2_CONTEXT_ERROR;
        }

        if (owner_pid != 0 && slot->owner_pid != 0 && slot->owner_pid != owner_pid) {
            execution_slot_exit_critical(&slot_guard);
            return ESYS_V2_NO_PERMISSION;
        }

        switch (slot->state) {
        case EXEC_SLOT_COMPLETED:
        case EXEC_SLOT_RESULT_MAPPED:
        {
            execution_slot_trace_scope_t trace_scope = {0};

            if (owner_pid == 0 || current_proc == NULL ||
                current_proc->type != PROC_TYPE_USER ||
                slot->owner_pid == 0 ||
                slot->owner_pid != owner_pid) {
                execution_slot_exit_critical(&slot_guard);
                return ESYS_V2_NO_PERMISSION;
            }

            execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_WAIT_RESULT);
            if (sys_v2_map_result_for_wait_locked(slot,
                                                  current_proc,
                                                  &mapped_result_va) != 0) {
                execution_slot_trace_scope_exit(&trace_scope);
                execution_slot_exit_critical(&slot_guard);
                return ESYS_V2_RESOURCE_BUSY;
            }
            execution_slot_trace_scope_exit(&trace_scope);

            execution_slot_exit_critical(&slot_guard);
            return mapped_result_va;
        }
        case EXEC_SLOT_TIMEOUT:
            execution_slot_exit_critical(&slot_guard);
            return ESYS_V2_TIMEOUT;
        case EXEC_SLOT_FAILED:
        case EXEC_SLOT_ABORTED:
            execution_slot_exit_critical(&slot_guard);
            return ESYS_V2_CONTEXT_ERROR;
        case EXEC_SLOT_CREATED:
        case EXEC_SLOT_READY:
        case EXEC_SLOT_RUNNING:
        default:
            if (timeout_ms == 0 || owner_pid == 0 || current_proc == NULL) {
                execution_slot_exit_critical(&slot_guard);
                return ESYS_V2_RESOURCE_BUSY;
            }

            if (slot->deadline_tick == 0) {
                now_tick = timer_ticks();
                deadline_delta = timer_ms_to_ticks_ceil(timeout_ms);
                if (deadline_delta == 0) {
                    deadline_delta = 1;
                }
                if (now_tick > UINT64_MAX - deadline_delta) {
                    slot->deadline_tick = UINT64_MAX;
                } else {
                    slot->deadline_tick = now_tick + deadline_delta;
                }
            }

            wait_obj = &slot->wait_key;
            execution_slot_exit_critical(&slot_guard);
            proc_block_current(wait_obj);
            break;
        }
    }
}

uint64_t sys_v2_complete_execution(uint64_t execution_id, uint64_t completion_code)
{
    execution_slot_guard_t slot_guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    exec_slot_t *slot = NULL;
    exec_slot_state_t next_state;
    proc_t *caller_proc;
    uint64_t caller_pid;
    uint64_t result = ESYS_V2_SUCCESS;

    if (execution_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    if (sys_v2_completion_code_to_state(completion_code, &next_state) != 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    extern proc_t *current_proc;
    caller_proc = current_proc;
    if (!caller_proc || caller_proc->pid <= 0 || caller_proc->type != PROC_TYPE_USER) {
        return ESYS_V2_PERMISSION_DENIED;
    }
    caller_pid = (uint64_t)caller_proc->pid;

    execution_slot_enter_critical(&slot_guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_COMPLETE);

    slot = execution_slot_find_locked(execution_id);
    if (!slot) {
        result = ESYS_V2_INVALID_ID;
        goto done;
    }

    if (slot->state != EXEC_SLOT_RUNNING) {
        result = ESYS_V2_INVALID_STATE;
        goto done;
    }

    if (caller_proc->active_execution_id == 0 ||
        caller_proc->active_execution_id != execution_id ||
        slot->target_context_id != caller_pid) {
        result = ESYS_V2_PERMISSION_DENIED;
        goto done;
    }

    if (next_state == EXEC_SLOT_COMPLETED) {
        if (execution_slot_validate_output_locked(slot, NULL) != 0 ||
            execution_slot_prepare_result_locked(slot) != 0) {
            execution_slot_require_finish_locked(slot,
                                                 EXEC_SLOT_FAILED,
                                                 "sys_v2_complete_execution:prepare_result_failed");
            result = ESYS_V2_INVALID_STATE;
            goto done;
        }
    }

    execution_slot_require_finish_locked(slot, next_state, "sys_v2_complete_execution");

done:
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&slot_guard);

    if (result == ESYS_V2_SUCCESS) {
        fb_print("[syscall_v2] complete_execution: exec_id=");
        fb_print_int(execution_id);
        fb_print(" caller_pid=");
        fb_print_int(caller_pid);
        fb_print(" state=");
        fb_print(next_state == EXEC_SLOT_COMPLETED ? "COMPLETED" : "FAILED");
        fb_print("\n");
    }

    return result;
}

// ============================================================================
// INTERRUPT MANAGEMENT SYSCALLS
// ============================================================================
//
// Interrupt handling return mechanism for Ring3 interrupt handlers.

uint64_t sys_v2_interrupt_return(uint64_t interrupt_id, uint64_t result_code)
{
    // Validate parameters
    if (interrupt_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // TODO: Implement actual interrupt return mechanism
    fb_print("[syscall_v2] interrupt_return: int_id=");
    fb_print_int(interrupt_id);
    fb_print(" result=");
    fb_print_int(result_code);
    fb_print("\n");
    
    return ESYS_V2_SUCCESS;
}

// ============================================================================
// TIME MANAGEMENT SYSCALLS
// ============================================================================
//
// Time query mechanism - provides access to system time without policy.

uint64_t sys_v2_time_query(uint64_t query_type, uint64_t *result_buffer)
{
    if (result_buffer == NULL) {
        return ESYS_V2_INVALID_PARAM;
    }

    switch (query_type) {
    case TIME_QUERY_MONOTONIC:
        *result_buffer = timer_ticks();
        return ESYS_V2_SUCCESS;
    case TIME_QUERY_UPTIME:
        *result_buffer = timer_ticks_to_ms(timer_ticks());
        return ESYS_V2_SUCCESS;
    default:
        return ESYS_V2_INVALID_PARAM;
    }
}

// ============================================================================
// CAPABILITY MANAGEMENT SYSCALLS
// ============================================================================
//
// Capability token binding and revocation mechanisms.
// Ring0 provides token validation, Ring3 provides capability policy.

uint64_t sys_v2_capability_bind(uint64_t execution_ctx_id, capability_token_t *token)
{
    // Validate parameters
    if (execution_ctx_id == 0 || token == NULL) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // SECURITY ENFORCEMENT: Use capability manager for secure binding
    // This prevents privilege escalation by validating tokens and enforcing permissions
    int bind_result = capability_bind_to_context(execution_ctx_id, token);
    
    // Convert capability manager error codes to syscall error codes
    switch (bind_result) {
        case CAPABILITY_SUCCESS:
            fb_print("[syscall_v2] capability_bind: GRANTED ctx=");
            fb_print_int(execution_ctx_id);
            fb_print(" cap_id=");
            fb_print_int(token->id);
            fb_print(" perms=0x");
            fb_print_hex(token->permissions);
            fb_print(" type=");
            fb_print_int(token->resource_type);
            fb_print("\n");
            return token->id;
            
        case CAPABILITY_ERROR_INVALID_TOKEN:
            fb_print("[syscall_v2] capability_bind: DENIED - Invalid token\n");
            return ESYS_V2_NO_CAPABILITY;
            
        case CAPABILITY_ERROR_NOT_FOUND:
            fb_print("[syscall_v2] capability_bind: DENIED - Token not found\n");
            return ESYS_V2_NO_CAPABILITY;
            
        case CAPABILITY_ERROR_REVOKED:
            fb_print("[syscall_v2] capability_bind: DENIED - Token revoked\n");
            return ESYS_V2_NO_CAPABILITY;
            
        case CAPABILITY_ERROR_EXPIRED:
            fb_print("[syscall_v2] capability_bind: DENIED - Token expired\n");
            return ESYS_V2_NO_CAPABILITY;
            
        case CAPABILITY_ERROR_ALREADY_EXISTS:
            fb_print("[syscall_v2] capability_bind: DENIED - Already bound\n");
            return ESYS_V2_RESOURCE_BUSY;
            
        case CAPABILITY_ERROR_SYSTEM_LIMIT:
            fb_print("[syscall_v2] capability_bind: DENIED - System limit reached\n");
            return ESYS_V2_NO_MEMORY;
            
        default:
            fb_print("[syscall_v2] capability_bind: DENIED - Unknown error ");
            fb_print_int(bind_result);
            fb_print("\n");
            return ESYS_V2_NO_PERMISSION;
    }
}

uint64_t sys_v2_capability_revoke(uint64_t token_id)
{
    // Validate parameters
    if (token_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // SECURITY ENFORCEMENT: Use capability manager for secure revocation
    // This ensures proper cleanup and prevents use-after-revoke attacks
    int revoke_result = capability_revoke(token_id);
    
    // Convert capability manager error codes to syscall error codes
    switch (revoke_result) {
        case CAPABILITY_SUCCESS:
            fb_print("[syscall_v2] capability_revoke: GRANTED token_id=");
            fb_print_int(token_id);
            fb_print("\n");
            return ESYS_V2_SUCCESS;
            
        case CAPABILITY_ERROR_NOT_FOUND:
            fb_print("[syscall_v2] capability_revoke: DENIED - Token not found\n");
            return ESYS_V2_NO_CAPABILITY;
            
        case CAPABILITY_ERROR_REVOKED:
            fb_print("[syscall_v2] capability_revoke: DENIED - Already revoked\n");
            return ESYS_V2_NO_CAPABILITY;
            
        default:
            fb_print("[syscall_v2] capability_revoke: DENIED - Unknown error ");
            fb_print_int(revoke_result);
            fb_print("\n");
            return ESYS_V2_NO_PERMISSION;
    }
}

// ============================================================================
// PROCESS MANAGEMENT SYSCALLS
// ============================================================================
//
// Process termination mechanism - Ring0 provides termination,
// Ring3 provides cleanup policy.

uint64_t sys_v2_exit(uint64_t exit_code)
{
    execution_slot_guard_t slot_guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    uint64_t result_vas[AYKEN_MAX_EXECUTION_SLOTS] = {0};
    uint64_t hash_vas[AYKEN_MAX_EXECUTION_SLOTS] = {0};
    uint32_t result_count = 0;
    proc_t *exiting_proc;
    uint64_t exiting_pid;

    extern proc_t *current_proc;
    exiting_proc = current_proc;
    if (!exiting_proc || exiting_proc->pid <= 0) {
        return ESYS_V2_INVALID_STATE;
    }
    exiting_pid = (uint64_t)exiting_proc->pid;

    if ((uint32_t)exiting_pid == sched_active_owner_pid()) {
        fb_print("[syscall_v2] scheduler-owner exit denied (pid=");
        fb_print_int(exiting_pid);
        fb_print(" code=");
        fb_print_int(exit_code);
        fb_print(")\n");
        return ESYS_V2_PERMISSION_DENIED;
    }

    fb_print("[syscall_v2] Process exit requested (pid=");
    fb_print_int(exiting_pid);
    fb_print(" code=");
    fb_print_int(exit_code);
    fb_print(")\n");

#if defined(AYKEN_VALIDATION)
    /* FREEZE INVARIANT: Set teardown_started flag with proper memory ordering
     * 
     * Memory ordering contract:
     * 1. smp_wmb(): Ensure all prior alias_registry_record() writes are visible
     *    before teardown_started is set. This prevents verifier from seeing
     *    partial registry state.
     * 2. teardown_started = 1: Set the freeze flag
     * 3. smp_mb(): Full barrier to ensure teardown_started write is globally
     *    visible before any subsequent operations. This prevents other cores
     *    from continuing to map while teardown is in progress.
     * 
     * Happens-before relationship:
     * - All alias_registry_record() writes happen-before teardown_started=1
     * - teardown_started=1 happens-before verifier snapshot
     * 
     * Without these barriers: Core 1 starts teardown, Core 2 still writes to
     * registry → verifier sees inconsistent snapshot → false negative.
     */
    smp_wmb();  /* alias_registry_record() writes happen-before teardown_started=1 */
    exiting_proc->teardown_started = 1;
    smp_mb();   /* teardown_started=1 globally visible before proceeding */
#endif

    execution_slot_enter_critical(&slot_guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_EXIT);
    result_count = execution_slot_prepare_process_exit_locked(exiting_pid,
                                                              result_vas,
                                                              hash_vas,
                                                              AYKEN_MAX_EXECUTION_SLOTS);
    exiting_proc->active_execution_id = 0;
    exiting_proc->wait_obj = NULL;
    exiting_proc->state = PROC_ZOMBIE;
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&slot_guard);

    proc_teardown_exit_surfaces(exiting_proc, result_vas, hash_vas, result_count);

#if defined(AYKEN_VALIDATION)
    /* ALIAS TEARDOWN PHASE: Alias eşlemelerini temizle ve doğrula
     * 
     * Çağrı sırası (LLD contract):
     * 1. Canonical teardown: proc_teardown_exit_surfaces() — canonical VA'ları temizler
     *    (mapping_ledger üzerinden)
     * 2. PROC_ZOMBIE state set edildi — teardown tamamlandı
     * 3. teardown_started = 1 set edildi — FREEZE INVARIANT aktif
     * 4. Alias teardown: exit_teardown_alias_phase() — alias VA'ları temizler
     *    (alias_reg üzerinden)
     * 
     * CANONICAL/ALIAS MEKANİK SINIR: Aynı phys frame'i paylaşan canonical VA ile
     * alias VA ayrıştırması veri-model düzeyinde mekanik olarak tanımlanmıştır:
     * 
     * - Canonical VA'lar: mapping_ledger'da kayıtlı, proc_teardown_exit_surfaces()
     *   içinde user_as_destroy_lower_half() tarafından temizlenir
     * - Alias VA'lar: alias_reg'de kayıtlı, exit_teardown_alias_phase() tarafından
     *   temizlenir
     * 
     * Bu ayrım kod seviyesinde mekanik olmalı: alias_reg döngüsü ve mapping_ledger
     * döngüsü aynı fonksiyonda birleştirilmemeli, ayrı scope'larda tutulmalı.
     * Canonical VA yanlışlıkla silinirse test geçer ama veri modeli sessizce bozulur
     * — bu sessiz veri kaybıdır.
     * 
     * FREEZE INVARIANT DOĞRULAMA: teardown_started = 1 set edildiği doğrulanmıştır
     * (bkz. yukarıda smp_wmb() + teardown_started=1 + smp_mb() sırası). Bu noktada
     * sys_v2_map_memory() bu proc için -EINVAL döner ve yeni alias kaydı gelmez;
     * verifier penceresi temizdir.
     * 
     * Validates: Requirements 4.1, 6.6, 7.1, 7.2, 7.3
     */
    exit_teardown_alias_phase(exiting_proc);
#endif

    sched_remove_process_everywhere(exiting_proc);

    execution_slot_enter_critical(&slot_guard);
    execution_slot_release_owned_by_owner_locked(exiting_pid);
    execution_slot_exit_critical(&slot_guard);

    sched_exit_current();
}

// ============================================================================
// DEBUG SYSCALLS (Ring3 Heartbeat)
// ============================================================================
//
// Debug character output - allows Ring3 to send heartbeat signals to Ring0
// for debugging and validation purposes. This bypasses the I/O privilege
// restriction that prevents Ring3 from using outb directly.

uint64_t sys_v2_debug_putchar(uint64_t character)
{
    uint8_t out_char;

    // Validate character is printable or control character
    if (character > 255) {
        return ESYS_V2_INVALID_PARAM;
    }

    out_char = (uint8_t)character;

    // Output character to debugcon (0xE9 port)
    outb(0xE9, out_char);

    // Reconstruct canonical marker per PID to avoid cross-process interleaving flake.
    sys_v2_debug_putchar_note_marker(out_char);

    return ESYS_V2_SUCCESS;
}

// ============================================================================
// SYSCALL DISPATCHER
// ============================================================================
//
// Routes execution-centric syscalls to their appropriate handlers.
// Provides consistent error handling and logging for debugging.

uint64_t syscall_v2_handler(uint64_t syscall_num, uint64_t arg1,
                            uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    sys_v2_dispatch_fn_t dispatch_fn;

    // Validate internal syscall index before dispatch table access.
    if (syscall_num >= SYS_V2_NR) {
        fb_print("[syscall_v2] ENOSYS: invalid v2 syscall ");
        fb_print_int(syscall_num);
        fb_print("\n");
        return sys_v2_finalize_result(ESYS_V2_INVALID_SYSCALL);
    }

    dispatch_fn = sys_v2_dispatch_table[syscall_num];
    if (!dispatch_fn) {
        fb_print("[syscall_v2] ENOSYS: unimplemented v2 syscall ");
        fb_print_int(syscall_num);
        fb_print("\n");
        return sys_v2_finalize_result(ESYS_V2_NOT_IMPLEMENTED);
    }
    return sys_v2_finalize_result(dispatch_fn(arg1, arg2, arg3, arg4));
}

// ============================================================================
// Phase-16 Runtime Bridge Syscalls
// ============================================================================
//
// **CRITICAL WARNING: THESE ARE STUB IMPLEMENTATIONS**
//
// These handlers return mock data and do NOT integrate with real subsystems:
// - Device operations: Return 0xDEADBEEF (not real DevFS)
// - External calls: Log only (no real external handler)
// - ABDF operations: Return mock data (not real ABDF substrate)
//
// **Production Requirements:**
// 1. Integrate sys_v2_device_operation with real DevFS
// 2. Integrate sys_v2_external_call with real external handler
// 3. Integrate sys_v2_abdf_operation with real ABDF substrate
// 4. Generate QEMU trace evidence showing real execution
//
// **Current Status:** ARCHITECTURAL SKELETON WITH MOCK DATA

/**
 * sys_v2_device_operation - Device operation syscall for Runtime_Bridge
 * 
 * Allows Runtime_Bridge to perform device operations with capability validation.
 * This is the ONLY approved path for BCIB to interact with devices.
 * 
 * @device_id: Device identifier
 * @operation: Operation type (read, write, status query)
 * @buffer: Data buffer for operation
 * @buffer_size: Size of data buffer
 * 
 * Returns: Operation result or error code
 */
uint64_t sys_v2_device_operation(uint64_t device_id, uint64_t operation, 
                                 uint64_t *buffer, uint64_t buffer_size) {
    // Validate parameters
    if (!buffer || buffer_size == 0 || buffer_size > 4096) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // Validate device_id range
    if (device_id >= 256) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // Get current process for capability validation
    extern proc_t *current_proc;
    if (!current_proc) {
        return ESYS_V2_CONTEXT_ERROR;
    }
    
    // Validate caller has device capability
    // In full implementation, this would check capability_token_t
    // For now, we validate execution role
    if (current_proc->execution_role != PROC_EXECUTION_ROLE_RUNTIME_BRIDGE) {
        fb_print("[syscall_v2] Device operation denied: not Runtime_Bridge\n");
        return ESYS_V2_PERMISSION_DENIED;
    }
    
    // Operation types
    #define DEVICE_OP_READ   1
    #define DEVICE_OP_WRITE  2
    #define DEVICE_OP_STATUS 3
    
    switch (operation) {
        case DEVICE_OP_READ:
            // Simulate device read - in real implementation, this would
            // interact with device drivers through DevFS
            buffer[0] = 0xDEADBEEF;  // Mock device data
            buffer[1] = device_id;
            return ESYS_V2_SUCCESS;
            
        case DEVICE_OP_WRITE:
            // Simulate device write
            fb_print("[syscall_v2] Device write to device ");
            fb_print_int(device_id);
            fb_print("\n");
            return ESYS_V2_SUCCESS;
            
        case DEVICE_OP_STATUS:
            // Return device status
            buffer[0] = 0x1;  // Device ready
            return ESYS_V2_SUCCESS;
            
        default:
            return ESYS_V2_INVALID_PARAM;
    }
}

/**
 * sys_v2_external_call - External call syscall for Runtime_Bridge
 * 
 * Allows Runtime_Bridge to perform external calls with capability validation.
 * This enables BCIB to interact with external systems through controlled interface.
 * 
 * @call_id: External call identifier
 * @args: Argument array
 * @arg_count: Number of arguments
 * 
 * Returns: Call result or error code
 */
uint64_t sys_v2_external_call(uint64_t call_id, uint64_t *args, uint64_t arg_count) {
    // Validate parameters
    if (!args || arg_count == 0 || arg_count > 8) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // Validate call_id range
    if (call_id >= 1024) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // Get current process for capability validation
    extern proc_t *current_proc;
    if (!current_proc) {
        return ESYS_V2_CONTEXT_ERROR;
    }
    
    // Validate caller has external call capability
    if (current_proc->execution_role != PROC_EXECUTION_ROLE_RUNTIME_BRIDGE) {
        fb_print("[syscall_v2] External call denied: not Runtime_Bridge\n");
        return ESYS_V2_PERMISSION_DENIED;
    }
    
    // External call types
    #define EXTERNAL_CALL_NETWORK  1
    #define EXTERNAL_CALL_IPC      2
    #define EXTERNAL_CALL_TIMER    3
    
    switch (call_id) {
        case EXTERNAL_CALL_NETWORK:
            // Simulate network call
            fb_print("[syscall_v2] External network call\n");
            return ESYS_V2_SUCCESS;
            
        case EXTERNAL_CALL_IPC:
            // Simulate IPC call
            fb_print("[syscall_v2] External IPC call\n");
            return ESYS_V2_SUCCESS;
            
        case EXTERNAL_CALL_TIMER:
            // Simulate timer call
            fb_print("[syscall_v2] External timer call\n");
            return ESYS_V2_SUCCESS;
            
        default:
            // Unknown external call
            fb_print("[syscall_v2] Unknown external call: ");
            fb_print_int(call_id);
            fb_print("\n");
            return ESYS_V2_NOT_IMPLEMENTED;
    }
}

/**
 * sys_v2_abdf_operation - ABDF operation syscall for Runtime_Bridge
 * 
 * Allows Runtime_Bridge to perform ABDF operations with capability validation.
 * This is the ONLY approved path for BCIB to interact with ABDF data substrate.
 * 
 * @operation_type: Operation type (read, write, create, revoke)
 * @handle_id: ABDF handle identifier
 * @data: Data buffer for operation
 * @data_size: Size of data buffer
 * 
 * Returns: Operation result or error code
 */
uint64_t sys_v2_abdf_operation(uint64_t operation_type, uint64_t handle_id,
                               uint64_t *data, uint64_t data_size) {
    // Validate parameters
    if (!data || data_size == 0 || data_size > 8192) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // Get current process for capability validation
    extern proc_t *current_proc;
    if (!current_proc) {
        return ESYS_V2_CONTEXT_ERROR;
    }
    
    // Validate caller has ABDF capability
    if (current_proc->execution_role != PROC_EXECUTION_ROLE_RUNTIME_BRIDGE) {
        fb_print("[syscall_v2] ABDF operation denied: not Runtime_Bridge\n");
        return ESYS_V2_PERMISSION_DENIED;
    }
    
    // ABDF operation types
    #define ABDF_OP_READ    1
    #define ABDF_OP_WRITE   2
    #define ABDF_OP_CREATE  3
    #define ABDF_OP_REVOKE  4
    
    switch (operation_type) {
        case ABDF_OP_READ:
            // Simulate ABDF read
            data[0] = 0xABDF0000 | (handle_id & 0xFFFF);
            data[1] = 0x12345678;  // Mock ABDF data
            fb_print("[syscall_v2] ABDF read handle ");
            fb_print_int(handle_id);
            fb_print("\n");
            return ESYS_V2_SUCCESS;
            
        case ABDF_OP_WRITE:
            // Simulate ABDF write (append-only)
            fb_print("[syscall_v2] ABDF write handle ");
            fb_print_int(handle_id);
            fb_print("\n");
            return ESYS_V2_SUCCESS;
            
        case ABDF_OP_CREATE:
            // Simulate ABDF handle creation
            data[0] = 0xABDF0000 | ((handle_id + 1) & 0xFFFF);  // New handle
            fb_print("[syscall_v2] ABDF create new handle\n");
            return ESYS_V2_SUCCESS;
            
        case ABDF_OP_REVOKE:
            // Simulate ABDF handle revocation
            fb_print("[syscall_v2] ABDF revoke handle ");
            fb_print_int(handle_id);
            fb_print("\n");
            return ESYS_V2_SUCCESS;
            
        default:
            return ESYS_V2_INVALID_PARAM;
    }
}
