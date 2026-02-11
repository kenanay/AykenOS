// Basic CPU init placeholder
#include "cpu.h"
#include "gdt_idt.h"
#include "../include/mm.h"
#include "../include/kheap.h"

void cpu_init(void)
{
    // For now, nothing architecture specific besides ensuring interrupts are off
    disable_interrupts();
}

void tss_init(void)
{
    // Initialize TSS structure
    __builtin_memset(&kernel_tss, 0, sizeof(tss_entry_t));
    
    // Allocate kernel stack for interrupts (4KB)
    uint64_t kernel_stack = (uint64_t)kmalloc(4096);
    kernel_tss.rsp0 = kernel_stack + 4096; // Stack grows down

    // Dedicated IST stack for critical faults/syscalls (bypass rsp0 on entry)
    static uint8_t ist1_stack[AYKEN_FRAME_SIZE] __attribute__((aligned(16)));
    kernel_tss.ist1 = (uint64_t)ist1_stack + sizeof(ist1_stack);
    
    // Set I/O map base beyond TSS limit to disable I/O permission checking
    // This allows all Ring3 processes to access all I/O ports
    kernel_tss.io_map_base = sizeof(tss_entry_t);
    
    // Install TSS in GDT and load TR register
    gdt_install_tss((uint64_t)&kernel_tss);
}

void tss_set_kernel_stack(uint64_t stack_ptr)
{
    kernel_tss.rsp0 = stack_ptr;
}
