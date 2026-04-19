// kernel/sys/syscall.c
// AykenOS Phase 2.5 - Execution-Centric Syscall Interface Only
//
// This file implements the final syscall interface with only the execution-centric
// syscalls. All POSIX-like syscalls have been removed as part of the architectural
// transformation to a data-centric, AI-native operating system.
//
// Requirements: AC-6 - Ring0 contains exactly 10 syscalls, no POSIX syscalls remain
// Phase-16: Integrated with boundary enforcement for BCIB/ABDF isolation

#include <stdint.h>
#include <stddef.h>
#include "../arch/x86_64/interrupts.h"
#include "../arch/x86_64/port_io.h"
#include "../drivers/console/fb_console.h"
#include "../sched/sched.h"
#include "syscall_v2.h"  // Include v2 syscall interface
#include "syscall_v2_hardened.h"  // Phase-16: Include hardened syscall interface

// Debug output via debugcon (port 0xE9)
static void debugcon_write(const char *s)
{
    uint64_t rflags;
    if (!s) return;
    __asm__ volatile("pushfq; pop %0; cli" : "=r"(rflags) : : "memory");
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        s++;
    }
    __asm__ volatile("push %0; popfq" : : "r"(rflags) : "memory", "cc");
}

// Helper to write integer to debugcon
static void debugcon_write_int(int64_t value)
{
    char buf[32];
    int idx = 0;

    if (value == 0) {
        outb(0xE9, '0');
        return;
    }

    if (value < 0) {
        outb(0xE9, '-');
        value = -value;
    }

    while (value > 0 && idx < (int)sizeof(buf)) {
        buf[idx++] = (char)('0' + (value % 10));
        value /= 10;
    }

    while (idx > 0) {
        outb(0xE9, (uint8_t)buf[--idx]);
    }
}

uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1,
                         uint64_t arg2, uint64_t arg3, uint64_t arg4);

// Basit INT 0x80 giriş noktası (frame pointer yok)
extern void syscall_isr(void);

void syscall_init(void)
{
    fb_print("[syscall] Installing INT 0x80 gate for execution-centric syscalls only.\n");
    
    // Debug: Before setting gate
    fb_print("[syscall] Before idt_set_gate: syscall_isr addr = ");
    fb_print_hex64((uint64_t)syscall_isr);
    fb_print("\n");
    
    idt_set_gate(0x80, (interrupt_handler_t)syscall_isr, 0xEE); // Present | DPL=3 | interrupt gate
    idt_table[0x80].ist = 0;  // CRITICAL: Use current kernel stack, not IST1

    struct idt_entry *e = &idt_table[0x80];
    uint64_t off = ((uint64_t)e->offset_high << 32) |
                   ((uint64_t)e->offset_mid << 16) |
                   (uint64_t)e->offset_low;
    fb_print("[idt80] sel=");
    fb_print_hex(e->selector);
    fb_print(" attr=");
    fb_print_hex(e->type_attr);
    fb_print(" off=");
    fb_print_hex64(off);
    fb_print("\n");
    
    // Debug: Verify gate was set correctly
    if (off == (uint64_t)syscall_isr) {
        fb_print("[syscall] INT 0x80 gate set correctly!\n");
    } else {
        fb_print("[syscall] ERROR: INT 0x80 gate offset mismatch!\n");
    }
}

// ============================================================================
// EXECUTION-CENTRIC SYSCALL DISPATCHER (Phase 2.5 - Final Implementation)
// ============================================================================
//
// This dispatcher implements the final syscall interface with only the
// execution-centric syscalls. All POSIX-like syscalls have been removed.
// 
// Syscall Numbering Plan (Final):
// - SYS_V2_BASE..SYS_V2_LAST range: Execution-centric (v2) syscalls (user space numbers)
// - 0..SYS_V2_MAX_INDEX range: Internal kernel mapping for v2 syscalls
// - All other ranges: Invalid (return -ENOSYS)
//
// Requirements: AC-6 - Only execution-centric syscalls remain

uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1,
                         uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    uint64_t result;

#if defined(AYKEN_RING3_ENTRY_MEM_PROFILE) && (AYKEN_RING3_ENTRY_MEM_PROFILE == 1)
    extern void entry_diag_record_c(uint32_t phase, uint32_t aux);
    static uint32_t entry_diag_first_syscall_seen = 0;
    static uint32_t entry_diag_kernel_reentry_seen = 0;
    
    // Phase 5: FIRST_FETCH (first syscall only)
    if (!entry_diag_first_syscall_seen) {
        entry_diag_record_c(5, (uint32_t)syscall_num);
        entry_diag_first_syscall_seen = 1;
    }
#endif

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    static uint8_t low_half_kheap_syscall_runtime_proof_emitted = 0;
    if (!low_half_kheap_syscall_runtime_proof_emitted &&
        current_proc != NULL &&
        current_proc->type == PROC_TYPE_USER) {
        low_half_kheap_syscall_runtime_proof_emitted = 1;
        proc_emit_low_half_kheap_runtime_proof(current_proc, "syscall_entry");
    }
#endif

    // Marker: syscall entry/return for Phase 10-A2 Task 3 roundtrip evidence.
    sched_perf_note_first_syscall_entry();
    
    // Make marker emission atomic as a block
    uint64_t rflags_marker;
    __asm__ volatile("pushfq; pop %0; cli" : "=r"(rflags_marker) : : "memory");

    // Phase-16: Emit BCIB_FORBIDDEN_BEFORE marker for BCIB contexts
    // MUST be before AYKEN_SYSCALL_ENTER for correct marker sequence
    if (current_proc && current_proc->execution_role == PROC_EXECUTION_ROLE_BCIB) {
        debugcon_write("BCIB_FORBIDDEN_BEFORE process_id=");
        debugcon_write_int((int64_t)current_proc->pid);
        debugcon_write("\n");
    }
    
    debugcon_write("[[AYKEN_SYSCALL_ENTER]] pid=");
    if (current_proc) {
        debugcon_write_int((int64_t)current_proc->pid);
    } else {
        debugcon_write("0");
    }
    debugcon_write("\n");
    debugcon_write("P10_SYSCALL_ENTER\n");
    __asm__ volatile("push %0; popfq" : : "r"(rflags_marker) : "memory", "cc");
    
    // Route based on Final Syscall Numbering Plan
    if (syscall_num >= SYS_V2_BASE && syscall_num <= SYS_V2_LAST) {
        // Execution-centric syscalls (v2) - Convert to internal index for hardened handler
        // Phase-16: Use hardened handler with boundary enforcement
        result = syscall_v2_hardened_handler(syscall_num - SYS_V2_BASE, arg1, arg2, arg3, arg4);
    } else {
        // Invalid syscall number - only the frozen v2 public range is valid
        fb_print("[syscall] ENOSYS: invalid syscall number ");
        fb_print_int(syscall_num);
        fb_print(" (valid range: ");
        fb_print_int(SYS_V2_BASE);
        fb_print("-");
        fb_print_int(SYS_V2_LAST);
        fb_print(" only)\n");
        result = (uint64_t)-38; // -ENOSYS
    }
    
    // Marker: Syscall return
    sched_perf_note_first_syscall_exit();
    
#if defined(AYKEN_RING3_ENTRY_MEM_PROFILE) && (AYKEN_RING3_ENTRY_MEM_PROFILE == 1)
    // Phase 7: KERNEL_REENTRY (first syscall return only)
    if (!entry_diag_kernel_reentry_seen) {
        entry_diag_record_c(7, 0);
        entry_diag_kernel_reentry_seen = 1;
    }
#endif
    
    __asm__ volatile("pushfq; pop %0; cli" : "=r"(rflags_marker) : : "memory");
    debugcon_write("[[AYKEN_SYSCALL_RETURN]]\n");
    debugcon_write("P10_SYSCALL_RETURN\n");
    __asm__ volatile("push %0; popfq" : : "r"(rflags_marker) : "memory", "cc");
    if (syscall_num == 1008 && result != 0) {
        // Capability negative-path enforcement marker (expected for fresh boot).
        debugcon_write("P10_CAP_ENFORCED\n");
    }
    
    return result;
}
