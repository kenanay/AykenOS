#include <stdint.h>
#include "timer.h"
#include "port_io.h"
#include "interrupts.h"
#include "pic.h"
#include "gdt_idt.h"
#include "../../sched/sched.h"

#define PIT_CHANNEL0   0x40
#define PIT_COMMAND    0x43

static uint64_t tick_count = 0;

typedef struct irq_timer_frame {
    uint64_t r15, r14, r13, r12;
    uint64_t r11, r10, r9, r8;
    uint64_t rbp, rdi, rsi, rdx, rcx, rbx, rax;
    uint64_t rip, cs, rflags, rsp, ss;
} irq_timer_frame_t;

// C handler called from ASM stub (argument: pointer to saved IRQ frame on kernel stack)
void timer_isr_c(void *frame_ptr)
{
    irq_timer_frame_t *frame = (irq_timer_frame_t *)frame_ptr;
    tick_count++;
    
    // PHASE 4.5 test mode: frequent timer marker for deterministic visibility.
    static uint32_t t = 0;
    t++;
    if ((t % 2) == 0) {  // Every 2 ticks
        outb(0xE9, (uint8_t)'T');  // Timer marker
    }
    
    // Acknowledge IRQ0 before scheduling. If the scheduler switches context from
    // IRQ path, we still want PIC in-service state to be cleared deterministically.
    pic_send_eoi(0);

    // PHASE 4.5: Aggressive timer-driven preemption for validation.
    // Snapshot interrupted user context so IRQ-driven switch can restore
    // true user RIP/RSP instead of kernel scheduler frame state.
    extern proc_t *current_proc;
    if (current_proc && current_proc->type == PROC_TYPE_USER &&
        frame && ((frame->cs & 0x3) == 0x3)) {
        current_proc->context.r15 = frame->r15;
        current_proc->context.r14 = frame->r14;
        current_proc->context.r13 = frame->r13;
        current_proc->context.r12 = frame->r12;
        current_proc->context.rbx = frame->rbx;
        current_proc->context.rbp = frame->rbp;
        current_proc->context.rip = frame->rip;
        current_proc->context.rsp = frame->rsp;
        current_proc->context.rflags = frame->rflags;
        current_proc->context.cs = (uint16_t)frame->cs;
        current_proc->context.ss = (uint16_t)frame->ss;
        __asm__ volatile("mov %%cr3, %0" : "=r"(current_proc->context.cr3));

        // Tell context_switch.asm old user state is already snapshotted.
        sched_irq_user_ctx_saved = 1;

        // Defer context switch to IRQ ASM tail for clean stack discipline.
        sched_request_resched_irq();
    }
}

void timer_init(uint32_t frequency_hz)
{
    // DEBUG: Timer init entry
    outb(0xE9, (uint8_t)'[');
    outb(0xE9, (uint8_t)'T');
    outb(0xE9, (uint8_t)'M');
    outb(0xE9, (uint8_t)'R');
    outb(0xE9, (uint8_t)']');
    
    // Install ASM handler for IRQ0 (vector 32) using raw API
    extern void timer_isr_asm(void);
    idt_set_gate_raw(32, timer_isr_asm, 0x8E); // present, ring0 interrupt gate

    uint32_t divisor = 1193180 / (frequency_hz ? frequency_hz : 100);
    outb(PIT_COMMAND, 0x36); // channel 0, lobyte/hibyte, mode 3
    outb(PIT_CHANNEL0, divisor & 0xFF);
    outb(PIT_CHANNEL0, (divisor >> 8) & 0xFF);

    // DEBUG: Before clearing IRQ0 mask
    outb(0xE9, (uint8_t)'[');
    outb(0xE9, (uint8_t)'U');
    outb(0xE9, (uint8_t)'N');
    outb(0xE9, (uint8_t)'M');
    outb(0xE9, (uint8_t)'S');
    outb(0xE9, (uint8_t)'K');
    outb(0xE9, (uint8_t)']');

    pic_clear_mask(0); // enable timer IRQ
    
    // DEBUG: Timer init complete
    outb(0xE9, (uint8_t)'[');
    outb(0xE9, (uint8_t)'T');
    outb(0xE9, (uint8_t)'M');
    outb(0xE9, (uint8_t)'R');
    outb(0xE9, (uint8_t)'_');
    outb(0xE9, (uint8_t)'O');
    outb(0xE9, (uint8_t)'K');
    outb(0xE9, (uint8_t)']');
}

uint64_t timer_ticks(void)
{
    return tick_count;
}
