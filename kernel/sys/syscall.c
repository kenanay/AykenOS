// kernel/sys/syscall.c
// AykenOS Phase 2.5 - Execution-Centric Syscall Interface Only
//
// This file implements the final syscall interface with only the 11 execution-centric
// syscalls. All POSIX-like syscalls have been removed as part of the architectural
// transformation to a data-centric, AI-native operating system.
//
// Requirements: AC-6 - Ring0 contains exactly 11 syscalls, no POSIX syscalls remain

#include <stdint.h>
#include <stddef.h>
#include "../arch/x86_64/interrupts.h"
#include "../drivers/console/fb_console.h"
#include "syscall_v2.h"  // Include v2 syscall interface

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
    fb_print("[idt80] sel=");
    fb_print_hex(0x08);
    fb_print(" attr=");
    fb_print_hex(0xEE);
    fb_print(" off=");
    fb_print_hex64((uint64_t)syscall_isr);
    fb_print("\n");
    
    // Debug: Verify gate was set correctly
    fb_print("[syscall] INT 0x80 gate configured.\n");
}

// ============================================================================
// EXECUTION-CENTRIC SYSCALL DISPATCHER (Phase 2.5 - Final Implementation)
// ============================================================================
//
// This dispatcher implements the final syscall interface with only the 11
// execution-centric syscalls. All POSIX-like syscalls have been removed.
// 
// Syscall Numbering Plan (Final):
// - 1000-1010 range: Execution-centric (v2) syscalls (user space numbers)
// - 0-10 range: Internal kernel mapping for v2 syscalls
// - All other ranges: Invalid (return -ENOSYS)
//
// Requirements: AC-6 - Only 11 execution-centric syscalls remain

uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1,
                         uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    uint64_t result;
    
    // Route based on Final Syscall Numbering Plan
    if (syscall_num >= SYS_V2_BASE && syscall_num <= SYS_V2_LAST) {
        // Execution-centric syscalls (v2) - Convert to internal index range.
        result = syscall_v2_handler(syscall_num - SYS_V2_BASE, arg1, arg2, arg3, arg4);
    } else {
        // Invalid syscall number - only SYS_V2_BASE..SYS_V2_LAST is valid.
        fb_print("[syscall] ENOSYS: invalid syscall number ");
        fb_print_int(syscall_num);
        fb_print(" (valid range: ");
        fb_print_int(SYS_V2_BASE);
        fb_print("-");
        fb_print_int(SYS_V2_LAST);
        fb_print(" only)\n");
        result = (uint64_t)-38; // -ENOSYS
    }
    return result;
}
