#include <stdint.h>
#include "timer.h"
#include "port_io.h"
#include "interrupts.h"
#include "pic.h"
#include "../../sched/sched.h"

#define PIT_CHANNEL0   0x40
#define PIT_COMMAND    0x43

static uint64_t tick_count = 0;

__attribute__((interrupt))
static void timer_isr(struct interrupt_frame *frame)
{
    (void)frame;
    tick_count++;
    sched_yield();
    pic_send_eoi(0);
}

void timer_init(uint32_t frequency_hz)
{
    // Install handler for IRQ0 (vector 32)
    idt_set_gate(32, timer_isr, 0x8E); // present, ring0 interrupt gate

    uint32_t divisor = 1193180 / (frequency_hz ? frequency_hz : 100);
    outb(PIT_COMMAND, 0x36); // channel 0, lobyte/hibyte, mode 3
    outb(PIT_CHANNEL0, divisor & 0xFF);
    outb(PIT_CHANNEL0, (divisor >> 8) & 0xFF);

    pic_clear_mask(0); // enable timer IRQ
}

uint64_t timer_ticks(void)
{
    return tick_count;
}
