#include "serial.h"
#include "../../arch/x86_64/port_io.h"

#define COM1 0x3F8

void serial_init_com1(void) {
    outb(COM1 + 1, 0x00);    // Disable interrupts
    outb(COM1 + 3, 0x80);    // Enable DLAB
    outb(COM1 + 0, 0x03);    // Divisor low  (38400 baud if base clock 115200)
    outb(COM1 + 1, 0x00);    // Divisor high
    outb(COM1 + 3, 0x03);    // 8 bits, no parity, one stop bit
    outb(COM1 + 2, 0xC7);    // Enable FIFO, clear, 14-byte threshold
    outb(COM1 + 4, 0x0B);    // IRQs enabled, RTS/DSR set
}

static int serial_can_tx(void) {
    return (inb(COM1 + 5) & 0x20) != 0;
}

void serial_write_char(char c) {
    while (!serial_can_tx()) { }
    outb(COM1, (uint8_t)c);
}

void serial_write(const char *s) {
    if (!s) return;
    while (*s) serial_write_char(*s++);
}