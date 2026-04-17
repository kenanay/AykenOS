#include <stdint.h>
#include <stddef.h>
#include "timer.h"
#include "port_io.h"
#include "interrupts.h"
#include "pic.h"
#include "gdt_idt.h"
#include "../../sched/sched.h"
#include "../../sched/sched_mailbox.h"
#include "../../include/execution_slot.h"
#include "../../include/ayken_abi.h"
#include "../../include/ayken.h"

#define PIT_CHANNEL0   0x40
#define PIT_COMMAND    0x43
#define QEMU_DEBUG_EXIT_PORT 0xF4

#ifndef AYKEN_DEBUG_IRQ
#define AYKEN_DEBUG_IRQ 0
#endif

#ifndef AYKEN_DETERMINISTIC_EXIT
#define AYKEN_DETERMINISTIC_EXIT 0
#endif

#if defined(AYKEN_GATE45_PROOF) && (AYKEN_GATE45_PROOF == 1)
#ifndef AYKEN_GATE45_TARGET_PID
#define AYKEN_GATE45_TARGET_PID 3u
#endif
#endif

#if AYKEN_DEBUG_IRQ
#define TIMER_DBG_CHAR(ch) outb(0xE9, (uint8_t)(ch))
#else
#define TIMER_DBG_CHAR(ch) do { } while (0)
#endif

static volatile uint64_t tick_count = 0;
static volatile uint32_t timer_frequency_hz_value = 100;
static volatile uint32_t timer_initialized = 0;

static void timer_debugcon_write(const char *s)
{
    while (*s) {
        outb(0xE9, (uint8_t)(*s++));
    }
}

static void timer_debugcon_hex16(uint16_t value)
{
    static const char hex[] = "0123456789ABCDEF";
    for (int i = 3; i >= 0; --i) {
        outb(0xE9, (uint8_t)hex[(value >> (i * 4)) & 0xF]);
    }
}

static void timer_debugcon_hex64(uint64_t value)
{
    static const char hex[] = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        outb(0xE9, (uint8_t)hex[(value >> (i * 4)) & 0xF]);
    }
}

static void timer_maybe_exit_on_proof_done(void)
{
#if AYKEN_DETERMINISTIC_EXIT && defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    static uint8_t proof_done_emitted = 0;
    static uint32_t owner_user_ticks = 0;
    int proof_done = 0;
    extern proc_t *current_proc;

    if (current_proc &&
        current_proc->type == PROC_TYPE_USER &&
        (uint32_t)current_proc->pid == AYKEN_SCHED_OWNER_PID) {
        if (owner_user_ticks != UINT32_MAX) {
            owner_user_ticks++;
        }
    }

#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
#if defined(AYKEN_GATE45_PROOF) && (AYKEN_GATE45_PROOF == 1)
    if (current_proc &&
        current_proc->type == PROC_TYPE_USER &&
        (uint32_t)current_proc->pid == AYKEN_GATE45_TARGET_PID &&
        !sched_mailbox_gate4_epoch1_pending()) {
        proof_done = 1;
    }
#else
    if (current_proc &&
        current_proc->type == PROC_TYPE_USER &&
        (uint32_t)current_proc->pid == AYKEN_SCHED_OWNER_PID &&
        !sched_mailbox_gate4_epoch1_pending()) {
        proof_done = 1;
    }
#endif
#else
    // Perf harness deterministic exit: allow enough IRQ cadence before exit.
    if (owner_user_ticks >= 64u) {
        proof_done = 1;
    }
#endif

    if (!proof_done_emitted && proof_done) {
        proof_done_emitted = 1;
        timer_debugcon_write("[[AYKEN_PROOF_DONE]]\n");
        // Primary deterministic exit path for CI (if device is present):
        // qemu-system-x86_64 -device isa-debug-exit,iobase=0x501,iosize=0x04
        // Process exit code becomes (value << 1) | 1.
        outl(QEMU_DEBUG_EXIT_PORT, 0);
        // Fallback: ACPI poweroff (works when QEMU runs without -no-shutdown).
        outw(0x604, 0x2000);
    }
#endif
}

typedef struct irq_timer_frame {
    uint64_t r15, r14, r13, r12;
    uint64_t r11, r10, r9, r8;
    uint64_t rbp, rdi, rsi, rdx, rcx, rbx, rax;
    uint64_t rip, cs, rflags, rsp, ss;
} irq_timer_frame_t;

_Static_assert(offsetof(irq_timer_frame_t, r15) == IRQF_R15, "irq frame: r15");
_Static_assert(offsetof(irq_timer_frame_t, r14) == IRQF_R14, "irq frame: r14");
_Static_assert(offsetof(irq_timer_frame_t, r13) == IRQF_R13, "irq frame: r13");
_Static_assert(offsetof(irq_timer_frame_t, r12) == IRQF_R12, "irq frame: r12");
_Static_assert(offsetof(irq_timer_frame_t, r11) == IRQF_R11, "irq frame: r11");
_Static_assert(offsetof(irq_timer_frame_t, r10) == IRQF_R10, "irq frame: r10");
_Static_assert(offsetof(irq_timer_frame_t, r9) == IRQF_R9, "irq frame: r9");
_Static_assert(offsetof(irq_timer_frame_t, r8) == IRQF_R8, "irq frame: r8");
_Static_assert(offsetof(irq_timer_frame_t, rbp) == IRQF_RBP, "irq frame: rbp");
_Static_assert(offsetof(irq_timer_frame_t, rdi) == IRQF_RDI, "irq frame: rdi");
_Static_assert(offsetof(irq_timer_frame_t, rsi) == IRQF_RSI, "irq frame: rsi");
_Static_assert(offsetof(irq_timer_frame_t, rdx) == IRQF_RDX, "irq frame: rdx");
_Static_assert(offsetof(irq_timer_frame_t, rcx) == IRQF_RCX, "irq frame: rcx");
_Static_assert(offsetof(irq_timer_frame_t, rbx) == IRQF_RBX, "irq frame: rbx");
_Static_assert(offsetof(irq_timer_frame_t, rax) == IRQF_RAX, "irq frame: rax");
_Static_assert(offsetof(irq_timer_frame_t, rip) == IRQF_RIP, "irq frame: rip");
_Static_assert(offsetof(irq_timer_frame_t, cs) == IRQF_CS, "irq frame: cs");
_Static_assert(offsetof(irq_timer_frame_t, rflags) == IRQF_RFLAGS, "irq frame: rflags");
_Static_assert(offsetof(irq_timer_frame_t, rsp) == IRQF_RSP, "irq frame: rsp");
_Static_assert(offsetof(irq_timer_frame_t, ss) == IRQF_SS, "irq frame: ss");
_Static_assert(sizeof(irq_timer_frame_t) == IRQF_SIZE, "irq frame: size");

// C handler called from ASM stub (argument: pointer to saved IRQ frame on kernel stack)
void timer_isr_c(void *frame_ptr)
{
    irq_timer_frame_t *frame = (irq_timer_frame_t *)frame_ptr;
    execution_slot_guard_t slot_guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    tick_count++;
    
    // Phase 3A Layer 2: IRQ0 tick marker
    static uint8_t irq0_marker_emitted = 0;
    if (!irq0_marker_emitted && tick_count >= 1) {
        irq0_marker_emitted = 1;
        timer_debugcon_write("[[AYKEN_IRQ0_TICK]] count=");
        timer_debugcon_hex64(tick_count);
        timer_debugcon_write("\n");
    }

    execution_slot_enter_critical(&slot_guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_TIMEOUT_IRQ);
    execution_slot_process_timeouts_locked(tick_count);
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&slot_guard);

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    static uint8_t p10_tick_marker_emitted = 0;
    if (!p10_tick_marker_emitted && tick_count >= 1) {
        p10_tick_marker_emitted = 1;
        timer_debugcon_write("P10_TICK\n");
    }
    static uint8_t tick_marker_emitted = 0;
    if (!tick_marker_emitted && tick_count >= 10) {
        tick_marker_emitted = 1;
        timer_debugcon_write("[[AYKEN_TICK]]\n");
    }
#endif

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
    timer_maybe_exit_on_proof_done();

    // PHASE 4.5: Aggressive timer-driven preemption for validation.
    // Snapshot interrupted user context so IRQ-driven switch can restore
    // true user RIP/RSP instead of kernel scheduler frame state.
    extern proc_t *current_proc;
    if (current_proc && current_proc->type == PROC_TYPE_USER &&
        frame && ((frame->cs & 0x3) == 0x3)) {
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
        static uint8_t low_half_kheap_timer_runtime_proof_emitted = 0;
        if (!low_half_kheap_timer_runtime_proof_emitted) {
            low_half_kheap_timer_runtime_proof_emitted = 1;
            proc_emit_low_half_kheap_runtime_proof(current_proc, "timer_irq");
        }
#endif
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

        // Timer-driven mailbox validation is enabled in:
        // - transitional bootstrap-policy mode, or
        // - Gate-4 isolated policy proof mode.
        //
        // PERFORMANCE TEST: Temporarily disabled to isolate IRQ validation overhead
        // This is a diagnostic patch to confirm IRQ validation is the bottleneck
#if 0  // DISABLED FOR PERFORMANCE TEST
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
   ((defined(AYKEN_SCHED_BOOTSTRAP_POLICY) && (AYKEN_SCHED_BOOTSTRAP_POLICY == 1)) || \
    (defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)))
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
        // Gate-4/4.5 isolated proofs validate owner authority only.
        if ((uint32_t)current_proc->pid == AYKEN_SCHED_OWNER_PID) {
            sched_mailbox_validate_ring3(current_proc);
        }
#else
        sched_mailbox_validate_ring3(current_proc);
#endif
#endif
#endif  // DISABLED FOR PERFORMANCE TEST

        // Tell context_switch.asm old user state is already snapshotted.
        sched_irq_user_ctx_saved = 1;

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
        {
            static uint8_t snapshot_marker_emitted = 0;
            if (!snapshot_marker_emitted) {
                snapshot_marker_emitted = 1;
                timer_debugcon_write("P10_IRQ_SNAPSHOT_OK rip=");
                timer_debugcon_hex64(frame->rip);
                timer_debugcon_write(" rsp=");
                timer_debugcon_hex64(frame->rsp);
                timer_debugcon_write(" cs=");
                timer_debugcon_hex16((uint16_t)frame->cs);
                timer_debugcon_write("\n");
            }
        }
#endif

        // Defer context switch to IRQ ASM tail for clean stack discipline.
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
        static uint8_t sched_event_notify_marker_emitted = 0;
        if (!sched_event_notify_marker_emitted) {
            sched_event_notify_marker_emitted = 1;
            timer_debugcon_write("P10_SCHED_EVENT_NOTIFY\n");
        }
#endif
        if (sched_should_defer_irq_resched_on_ring3_entry(current_proc)) {
            return;
        }
        sched_request_resched_irq();
    }
}

void timer_init(uint32_t frequency_hz)
{
    uint32_t configured_frequency_hz = frequency_hz ? frequency_hz : 100;

    // DEBUG: Timer init entry
    TIMER_DBG_CHAR('[');
    TIMER_DBG_CHAR('T');
    TIMER_DBG_CHAR('M');
    TIMER_DBG_CHAR('R');
    TIMER_DBG_CHAR(']');
    
    // Install ASM handler for IRQ0 (vector 32) using raw API
    extern void timer_isr_asm(void);
    idt_set_gate_raw(32, timer_isr_asm, 0x8E); // present, ring0 interrupt gate

    timer_frequency_hz_value = configured_frequency_hz;
    timer_initialized = 1;

    uint32_t divisor = 1193180 / configured_frequency_hz;
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

uint32_t timer_is_initialized(void)
{
    return timer_initialized;
}

uint64_t timer_ticks(void)
{
    return tick_count;
}

uint32_t timer_frequency_hz(void)
{
    return timer_frequency_hz_value;
}

uint64_t timer_ticks_to_ms(uint64_t ticks)
{
    uint32_t hz = timer_frequency_hz_value;

    if (hz == 0) {
        return 0;
    }

    return (ticks * 1000ULL) / (uint64_t)hz;
}

uint64_t timer_ms_to_ticks_ceil(uint64_t ms)
{
    uint32_t hz = timer_frequency_hz_value;
    uint64_t numerator;

    if (hz == 0 || ms == 0) {
        return 0;
    }

    if (ms > (UINT64_MAX - 999ULL) / (uint64_t)hz) {
        return UINT64_MAX / 2ULL;
    }

    numerator = (ms * (uint64_t)hz) + 999ULL;
    return numerator / 1000ULL;
}
