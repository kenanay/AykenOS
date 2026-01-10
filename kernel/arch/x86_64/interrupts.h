#pragma once
#include <stdint.h>

struct interrupt_frame;
typedef void (*interrupt_handler_t)(struct interrupt_frame *frame);

void idt_set_gate(uint8_t num, interrupt_handler_t handler, uint8_t flags);
void interrupts_install(void);

// Syscall ISR (INT 0x80) - implemented in syscall_isr.S
extern void syscall_isr(struct interrupt_frame *frame);
