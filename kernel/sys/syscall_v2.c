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

// ============================================================================
// MEMORY MANAGEMENT SYSCALLS
// ============================================================================
//
// These syscalls provide memory mapping mechanisms without policy decisions.
// Ring3 components determine what should be mapped and when.

uint64_t sys_v2_map_memory(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags)
{
    // Validate parameters
    if (virt_addr == 0 || phys_addr == 0) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // SECURITY ENFORCEMENT: Check if current execution context has memory capability
    // This prevents privilege escalation by requiring explicit capability grants
    extern proc_t *current_proc;
    if (current_proc != NULL) {
        uint64_t execution_ctx = current_proc->pid;
        capability_token_t *memory_cap = capability_get_by_context(execution_ctx, CAPABILITY_RESOURCE_MEMORY);
        
        if (memory_cap == NULL) {
            fb_print("[syscall_v2] map_memory: DENIED - No memory capability for context ");
            fb_print_int(execution_ctx);
            fb_print("\n");
            return ESYS_V2_NO_CAPABILITY;
        }
        
        // Check if capability allows memory mapping
        int access_result = capability_check_resource_access(memory_cap, phys_addr, 4096, 
                                                           CAPABILITY_PERM_READ | CAPABILITY_PERM_WRITE);
        if (access_result != CAPABILITY_SUCCESS) {
            fb_print("[syscall_v2] map_memory: DENIED - Capability check failed\n");
            return ESYS_V2_NO_PERMISSION;
        }
        
        fb_print("[syscall_v2] map_memory: GRANTED via capability ID=");
        fb_print_int(memory_cap->id);
        fb_print("\n");
    }
    
    // TODO: Implement actual memory mapping using paging system
    // For now, return success to allow testing of the interface
    fb_print("[syscall_v2] map_memory: virt=0x");
    fb_print_hex(virt_addr);
    fb_print(" phys=0x");
    fb_print_hex(phys_addr);
    fb_print(" flags=0x");
    fb_print_hex(flags);
    fb_print("\n");
    
    return ESYS_V2_SUCCESS;
}

uint64_t sys_v2_unmap_memory(uint64_t virt_addr, uint64_t size)
{
    // Validate parameters
    if (virt_addr == 0 || size == 0) {
        return ESYS_V2_INVALID_PARAM;
    }
    
    // TODO: Implement actual memory unmapping
    // For now, return success to allow testing of the interface
    fb_print("[syscall_v2] unmap_memory: virt=0x");
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
    exec_slot_t *slot = NULL;
    uint64_t owner_pid = 0;
    uint64_t execution_id;

    // Validate parameters
    if (bcib_graph == NULL || graph_size == 0 || context_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    extern proc_t *current_proc;
    if (current_proc != NULL && current_proc->pid > 0) {
        owner_pid = (uint64_t)current_proc->pid;
    }

    execution_slot_enter_critical(&slot_guard);

    slot = execution_slot_alloc_locked(owner_pid, context_id);
    if (!slot) {
        execution_slot_exit_critical(&slot_guard);
        return ESYS_V2_RESOURCE_BUSY;
    }

    slot->created_tick = timer_ticks();
    slot->bcib_size = graph_size;

    // Kernel-owned BCIB backing copy is a later slice. This first submit path
    // activates slot lifecycle anchoring and READY queue visibility only.
    if (execution_slot_transition_locked(slot, EXEC_SLOT_CREATED, EXEC_SLOT_READY) != 0) {
        execution_slot_release_locked(slot);
        execution_slot_exit_critical(&slot_guard);
        return ESYS_V2_RESOURCE_BUSY;
    }

    if (execution_slot_enqueue_locked(slot) != 0) {
        execution_slot_release_locked(slot);
        execution_slot_exit_critical(&slot_guard);
        return ESYS_V2_RESOURCE_BUSY;
    }

    execution_id = slot->execution_id;
    execution_slot_exit_critical(&slot_guard);

    fb_print("[syscall_v2] submit_execution: graph=0x");
    fb_print_hex((uint64_t)bcib_graph);
    fb_print(" size=");
    fb_print_int(graph_size);
    fb_print(" ctx=");
    fb_print_int(context_id);
    fb_print(" exec_id=");
    fb_print_int(execution_id);
    fb_print("\n");
    
    return execution_id;
}

uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    uint64_t owner_pid = 0;
    uint64_t result = ESYS_V2_RESOURCE_BUSY;

    // Validate parameters
    if (execution_id == 0) {
        return ESYS_V2_INVALID_PARAM;
    }

    (void)timeout_ms;

    extern proc_t *current_proc;
    if (current_proc != NULL && current_proc->pid > 0) {
        owner_pid = (uint64_t)current_proc->pid;
    }

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
        result = ESYS_V2_SUCCESS;
        break;
    case EXEC_SLOT_TIMEOUT:
        result = ESYS_V2_TIMEOUT;
        break;
    case EXEC_SLOT_FAILED:
    case EXEC_SLOT_ABORTED:
        result = ESYS_V2_CONTEXT_ERROR;
        break;
    case EXEC_SLOT_CREATED:
    case EXEC_SLOT_READY:
    case EXEC_SLOT_RUNNING:
    default:
        result = ESYS_V2_RESOURCE_BUSY;
        break;
    }

    execution_slot_exit_critical(&slot_guard);
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
    fb_print("[syscall_v2] Process exit requested (code=");
    fb_print_int(exit_code);
    fb_print(")\n");
    
    // TODO: Implement proper process termination
    // For now, yield to scheduler (similar to v1 exit)
    while (1) {
        sched_yield();
    }
    
    return ESYS_V2_SUCCESS; // Never reached
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
