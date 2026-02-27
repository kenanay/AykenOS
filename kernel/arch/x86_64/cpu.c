// Basic CPU init placeholder
#include "cpu.h"
#include "gdt_idt.h"

extern char _tss_rsp0_stack_top[];
extern char _tss_ist1_stack_top[];

void cpu_init(void)
{
    // For now, nothing architecture specific besides ensuring interrupts are off
    disable_interrupts();
}

void tss_init(void)
{
    // Initialize TSS structure
    __builtin_memset(&kernel_tss, 0, sizeof(tss_entry_t));

    // Linker-reserved stacks are placed in kernel high-half virtual memory.
    kernel_tss.rsp0 = (uint64_t)_tss_rsp0_stack_top;

    // Dedicated IST stack for critical faults/syscalls (bypass rsp0 on entry).
    kernel_tss.ist1 = (uint64_t)_tss_ist1_stack_top;
    
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
