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
    uint8_t a1 = inb(PIC1_DATA);
    uint8_t a2 = inb(PIC2_DATA);

    outb(PIC1_COMMAND, 0x11);
    outb(PIC2_COMMAND, 0x11);
    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);
    outb(PIC1_DATA, 4);
    outb(PIC2_DATA, 2);
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);

    outb(PIC1_DATA, a1);
    outb(PIC2_DATA, a2);

    master_mask = a1;
    slave_mask = a2;
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
    } else {
        irq -= 8;
        slave_mask &= ~(1 << irq);
        outb(PIC2_DATA, slave_mask);
    }
}
