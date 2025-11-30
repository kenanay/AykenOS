#include <stdint.h>
#include "interrupts.h"
#include "gdt_idt.h"

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

struct idt_entry idt_table[256];
struct idt_ptr idt_descriptor;

void idt_set_gate(int num, interrupt_handler_t handler, uint8_t flags)
{
    uint64_t addr = (uint64_t)handler;
    idt_table[num].offset_low = addr & 0xFFFF;
    idt_table[num].selector = 0x08; // kernel code segment
    idt_table[num].ist = 0;
    idt_table[num].type_attr = flags;
    idt_table[num].offset_mid = (addr >> 16) & 0xFFFF;
    idt_table[num].offset_high = (addr >> 32) & 0xFFFFFFFF;
    idt_table[num].zero = 0;
}

void interrupts_install(void)
{
    // zero-out IDT
    for (int i = 0; i < 256; ++i) {
        idt_table[i].offset_low = 0;
        idt_table[i].selector = 0;
        idt_table[i].ist = 0;
        idt_table[i].type_attr = 0;
        idt_table[i].offset_mid = 0;
        idt_table[i].offset_high = 0;
        idt_table[i].zero = 0;
    }

    idt_descriptor.limit = sizeof(idt_table) - 1;
    idt_descriptor.base = (uint64_t)&idt_table[0];

    idt_init();
}
