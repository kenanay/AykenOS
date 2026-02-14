#pragma once
#include <stdint.h>

// CPU-pushed interrupt frame (ring0; rsp/ss may be undefined if no privilege change)
struct interrupt_frame {
    uint64_t rip;
    uint64_t cs;
    uint64_t rflags;
    uint64_t rsp;
    uint64_t ss;
};
typedef void (*interrupt_handler_t)(struct interrupt_frame *frame);

struct idt_entry {
    uint16_t offset_low;
    uint16_t selector;
    uint8_t ist;
    uint8_t type_attr;
    uint16_t offset_mid;
    uint32_t offset_high;
    uint32_t zero;
} __attribute__((packed));

struct idt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

void idt_set_gate(uint8_t num, interrupt_handler_t handler, uint8_t flags);
void interrupts_install(void);
void interrupts_install_early(void);
const struct idt_ptr *idt_get_descriptor(void);

// Syscall ISR (INT 0x80) - implemented in context_switch.asm
extern void syscall_isr(void);
