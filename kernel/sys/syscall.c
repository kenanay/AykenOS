// kernel/sys/syscall.c
// AykenOS Phase 2.5 - Execution-Centric Syscall Interface Only
//
// This file implements the final syscall interface with only the execution-centric
// syscalls. All POSIX-like syscalls have been removed as part of the architectural
// transformation to a data-centric, AI-native operating system.
//
// Requirements: AC-6 - Ring0 contains exactly 10 syscalls, no POSIX syscalls remain

#include <stdint.h>
#include <stddef.h>
#include "../arch/x86_64/interrupts.h"
#include "../arch/x86_64/port_io.h"
#include "../drivers/console/fb_console.h"
#include "../sched/sched.h"
#include "syscall_v2.h"  // Include v2 syscall interface

// Debug output via debugcon (port 0xE9)
static void debugcon_write(const char *s)
{
    if (!s) return;
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        s++;
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
// - 1000-1011 range: Execution-centric (v2) syscalls (user space numbers)
// - 0-11 range: Internal kernel mapping for v2 syscalls
// - All other ranges: Invalid (return -ENOSYS)
//
// Requirements: AC-6 - Only execution-centric syscalls remain

uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1,
                         uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    uint64_t result;

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
    debugcon_write("[[AYKEN_SYSCALL_ENTER]]\n");
    debugcon_write("P10_SYSCALL_ENTER\n");
    
    // Route based on Final Syscall Numbering Plan
    if (syscall_num >= 1000 && syscall_num <= 1011) {
        // Execution-centric syscalls (v2) - Convert to 0-11 range for v2 handler
        result = syscall_v2_handler(syscall_num - 1000, arg1, arg2, arg3, arg4);
    } else {
        // Invalid syscall number - only 1000-1011 range is valid
        fb_print("[syscall] ENOSYS: invalid syscall number ");
        fb_print_int(syscall_num);
        fb_print(" (valid range: 1000-1011 only)\n");
        result = (uint64_t)-38; // -ENOSYS
    }
    
    // Marker: Syscall return
    debugcon_write("[[AYKEN_SYSCALL_RETURN]]\n");
    debugcon_write("P10_SYSCALL_RETURN\n");
    if (syscall_num == 1008 && result != 0) {
        // Capability negative-path enforcement marker (expected for fresh boot).
        debugcon_write("P10_CAP_ENFORCED\n");
    }
    
    return result;
}
