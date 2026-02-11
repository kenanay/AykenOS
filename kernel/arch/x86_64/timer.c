#include <stdint.h>
#include <stddef.h>
#include "timer.h"
#include "port_io.h"
#include "interrupts.h"
#include "pic.h"
#include "gdt_idt.h"
#include "../../sched/sched.h"

#define PIT_CHANNEL0   0x40
#define PIT_COMMAND    0x43

#ifndef AYKEN_DEBUG_IRQ
#define AYKEN_DEBUG_IRQ 0
#endif

#if AYKEN_DEBUG_IRQ
#define TIMER_DBG_CHAR(ch) outb(0xE9, (uint8_t)(ch))
#else
#define TIMER_DBG_CHAR(ch) do { } while (0)
#endif

static uint64_t tick_count = 0;

typedef struct irq_timer_frame {
    uint64_t r15, r14, r13, r12;
    uint64_t r11, r10, r9, r8;
    uint64_t rbp, rdi, rsi, rdx, rcx, rbx, rax;
    uint64_t rip, cs, rflags, rsp, ss;
} irq_timer_frame_t;

_Static_assert(offsetof(irq_timer_frame_t, r15) == 0, "irq frame: r15");
_Static_assert(offsetof(irq_timer_frame_t, r11) == 32, "irq frame: r11");
_Static_assert(offsetof(irq_timer_frame_t, rbp) == 64, "irq frame: rbp");
_Static_assert(offsetof(irq_timer_frame_t, rax) == 112, "irq frame: rax");
_Static_assert(offsetof(irq_timer_frame_t, rip) == 120, "irq frame: rip");
_Static_assert(offsetof(irq_timer_frame_t, cs) == 128, "irq frame: cs");
_Static_assert(offsetof(irq_timer_frame_t, rflags) == 136, "irq frame: rflags");
_Static_assert(offsetof(irq_timer_frame_t, rsp) == 144, "irq frame: rsp");
_Static_assert(offsetof(irq_timer_frame_t, ss) == 152, "irq frame: ss");
_Static_assert(sizeof(irq_timer_frame_t) == 160, "irq frame: size");

// C handler called from ASM stub (argument: pointer to saved IRQ frame on kernel stack)
void timer_isr_c(void *frame_ptr)
{
    irq_timer_frame_t *frame = (irq_timer_frame_t *)frame_ptr;
    tick_count++;

#if AYKEN_DEBUG_IRQ
    // Validation profile marker cadence.
    static uint32_t t = 0;
    t++;
    if ((t % 2) == 0) {  // Every 2 ticks
        TIMER_DBG_CHAR('T');
    }
#endif
    
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
    TIMER_DBG_CHAR('[');
    TIMER_DBG_CHAR('T');
    TIMER_DBG_CHAR('M');
    TIMER_DBG_CHAR('R');
    TIMER_DBG_CHAR(']');
    
    // Install ASM handler for IRQ0 (vector 32) using raw API
    extern void timer_isr_asm(void);
    idt_set_gate_raw(32, timer_isr_asm, 0x8E); // present, ring0 interrupt gate

    uint32_t divisor = 1193180 / (frequency_hz ? frequency_hz : 100);
    outb(PIT_COMMAND, 0x36); // channel 0, lobyte/hibyte, mode 3
    outb(PIT_CHANNEL0, divisor & 0xFF);
    outb(PIT_CHANNEL0, (divisor >> 8) & 0xFF);

    // DEBUG: Before clearing IRQ0 mask
    TIMER_DBG_CHAR('[');
    TIMER_DBG_CHAR('U');
    TIMER_DBG_CHAR('N');
    TIMER_DBG_CHAR('M');
    TIMER_DBG_CHAR('S');
    TIMER_DBG_CHAR('K');
    TIMER_DBG_CHAR(']');

    pic_clear_mask(0); // enable timer IRQ
    
    // DEBUG: Timer init complete
    TIMER_DBG_CHAR('[');
    TIMER_DBG_CHAR('T');
    TIMER_DBG_CHAR('M');
    TIMER_DBG_CHAR('R');
    TIMER_DBG_CHAR('_');
    TIMER_DBG_CHAR('O');
    TIMER_DBG_CHAR('K');
    TIMER_DBG_CHAR(']');
}

uint64_t timer_ticks(void)
{
    return tick_count;
}
