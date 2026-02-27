// kernel/ring3_jump.c
// ============================================================================
// AykenOS Phase 10 Sprint A: Ring3 Entry Mechanism
// Constitutional: Real CPL3 execution proof
// Authority: ARCHITECTURE_FREEZE.md
// ============================================================================

#include <stdint.h>
#include "arch/x86_64/port_io.h"

// External user binary (embedded via linker)
extern uint8_t _binary_userspace_minimal_user_bin_start[];
extern uint8_t _binary_userspace_minimal_user_bin_end[];

// Debugcon marker output
static void debugcon_write(const char *s)
{
    if (!s) return;
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        s++;
    }
}

// Jump to Ring3 user code
// This is the constitutional proof of Ring3 execution
void jump_to_ring3(void)
{
    debugcon_write("[K][PHASE10] KERNEL_BEFORE_RING3\n");
    
    // User binary location
    uint64_t user_entry = 0x400000;  // User-space base (from linker.ld)
    uint64_t user_stack = 0x500000;  // User stack (1MB above entry)
    
    // Copy user binary to user-space region
    uint8_t *user_bin_start = _binary_userspace_minimal_user_bin_start;
    uint8_t *user_bin_end = _binary_userspace_minimal_user_bin_end;
    uint64_t user_bin_size = (uint64_t)(user_bin_end - user_bin_start);
    
    // Simple memcpy (no libc)
    uint8_t *dst = (uint8_t *)user_entry;
    uint8_t *src = user_bin_start;
    for (uint64_t i = 0; i < user_bin_size; i++) {
        dst[i] = src[i];
    }
    
    // GDT selectors (from gdt_idt.c)
    // Entry 3: User Data (DPL=3) → selector 0x18 | 3 = 0x1B
    // Entry 4: User Code (DPL=3) → selector 0x20 | 3 = 0x23
    uint64_t user_cs = 0x23;  // User code segment (RPL=3)
    uint64_t user_ds = 0x1B;  // User data segment (RPL=3)
    
    // Set user data segments
    __asm__ volatile (
        "movw %w0, %%ax\n"
        "mov %%ax, %%ds\n"
        "mov %%ax, %%es\n"
        "mov %%ax, %%fs\n"
        "mov %%ax, %%gs\n"
        :
        : "r"(user_ds)
        : "ax"
    );
    
    // Build iretq frame on kernel stack
    // Stack layout (top to bottom):
    //   SS (user data segment)
    //   RSP (user stack pointer)
    //   RFLAGS (IF=1, IOPL=0)
    //   CS (user code segment)
    //   RIP (user entry point)
    
    uint64_t rflags = 0x202;  // IF=1 (interrupts enabled), IOPL=0
    
    __asm__ volatile (
        "pushq %0\n"        // SS
        "pushq %1\n"        // RSP
        "pushq %2\n"        // RFLAGS
        "pushq %3\n"        // CS
        "pushq %4\n"        // RIP
        "iretq\n"
        :
        : "r"(user_ds), "r"(user_stack), "r"(rflags), "r"(user_cs), "r"(user_entry)
        : "memory"
    );
    
    // Never reached (iretq jumps to Ring3)
    __builtin_unreachable();
}
