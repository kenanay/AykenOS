// kernel/sys/syscall.c
// System call stub implementation

#include <stdint.h>
#include "../arch/x86_64/interrupts.h"
#include "../drivers/console/fb_console.h"

// Basit INT 0x80 giriş noktası
__attribute__((interrupt)) void syscall_isr(struct interrupt_frame *frame)
{
    (void)frame;

    register uint64_t num asm("rax");
    register uint64_t arg1 asm("rdi");
    register uint64_t arg2 asm("rsi");
    register uint64_t arg3 asm("rdx");
    register uint64_t arg4 asm("r10");

    uint64_t ret = syscall_handler(num, arg1, arg2, arg3, arg4);
    __asm__ volatile("mov %0, %%rax" :: "r"(ret) : "rax");
}

void syscall_init(void)
{
    fb_print("[syscall] installing INT 0x80 gate.\n");
    idt_set_gate(0x80, syscall_isr, 0xEE); // Present | DPL=3 | interrupt gate
}

// Syscall handler (will be called from assembly)
uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1,
                         uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    // TODO: Implement syscall dispatch
    (void)syscall_num;
    (void)arg1;
    (void)arg2;
    (void)arg3;
    (void)arg4;

    return (uint64_t)-1; // ENOSYS
}
