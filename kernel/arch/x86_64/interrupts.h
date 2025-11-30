#pragma once
#include <stdint.h>

struct interrupt_frame {
    uint64_t rip;
    uint64_t cs;
    uint64_t rflags;
    uint64_t rsp;
    uint64_t ss;
};

typedef void (*interrupt_handler_t)(struct interrupt_frame *frame);

void idt_set_gate(int num, interrupt_handler_t handler, uint8_t flags);
void interrupts_install(void);
