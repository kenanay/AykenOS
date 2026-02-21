#include "port_io.h"
#include "pic.h"

#define PIC1            0x20
#define PIC2            0xA0
#define PIC1_COMMAND    PIC1
#define PIC1_DATA       (PIC1+1)
#define PIC2_COMMAND    PIC2
#define PIC2_DATA       (PIC2+1)

#define PIC_EOI         0x20

static uint8_t master_mask = 0xFF;
static uint8_t slave_mask = 0xFF;

void pic_init(void)
{
    // Remap PIC to 0x20-0x2F
    outb(PIC1_COMMAND, 0x11);
    outb(PIC2_COMMAND, 0x11);
    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);
    outb(PIC1_DATA, 4);
    outb(PIC2_DATA, 2);
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);

    // CRITICAL: Start with all IRQs masked, then selectively enable
    // Don't restore old masks - they might have all IRQs disabled
    outb(PIC1_DATA, 0xFF);  // Mask all master IRQs initially
    outb(PIC2_DATA, 0xFF);  // Mask all slave IRQs initially

    master_mask = 0xFF;
    slave_mask = 0xFF;
}

void pic_send_eoi(uint8_t irq)
{
    if (irq >= 8) {
        outb(PIC2_COMMAND, PIC_EOI);
    }
    outb(PIC1_COMMAND, PIC_EOI);
}

void pic_set_mask(uint8_t irq)
{
    if (irq < 8) {
        master_mask |= (1 << irq);
        outb(PIC1_DATA, master_mask);
    } else {
        irq -= 8;
        slave_mask |= (1 << irq);
        outb(PIC2_DATA, slave_mask);
    }
}

void pic_clear_mask(uint8_t irq)
{
    if (irq < 8) {
        master_mask &= ~(1 << irq);
        outb(PIC1_DATA, master_mask);
        
        // DEBUG: IRQ0 unmask
        if (irq == 0) {
            outb(0xE9, (uint8_t)'[');
            outb(0xE9, (uint8_t)'I');
            outb(0xE9, (uint8_t)'R');
            outb(0xE9, (uint8_t)'Q');
            outb(0xE9, (uint8_t)'0');
            outb(0xE9, (uint8_t)'_');
            outb(0xE9, (uint8_t)'O');
            outb(0xE9, (uint8_t)'N');
            outb(0xE9, (uint8_t)']');
        }
    } else {
        irq -= 8;
        slave_mask &= ~(1 << irq);
        outb(PIC2_DATA, slave_mask);
    }
}
