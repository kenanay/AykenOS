#include <stdint.h>
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

extern struct idt_entry idt_table[256];
extern struct idt_ptr idt_descriptor;

static inline void lidt(void *base, uint16_t size)
{
    struct {
        uint16_t length;
        void *base;
    } __attribute__((packed)) IDTR = { size, base };
    __asm__ volatile("lidt %0" : : "m"(IDTR));
}

void gdt_init(void)
{
    // GDT setup placeholder (UEFI bootloader already provided a basic GDT)
}

void idt_init(void)
{
    // IDT will be filled by interrupt setup; just load descriptor
    lidt(idt_table, sizeof(struct idt_entry) * 256 - 1);
}

void isr_init_stubs(void)
{
    // Nothing to do here for now; handlers are registered explicitly.
}
