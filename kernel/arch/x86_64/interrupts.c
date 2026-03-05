#include <stdint.h>
#include "interrupts.h"
#include "gdt_idt.h"
#include "port_io.h"
#include "../../sched/sched.h"

struct idt_entry idt_table[256];
struct idt_ptr idt_descriptor;
volatile uint32_t phase10_ring3_user_code_seen = 0;

// Early debugcon output (QEMU port 0xE9) — use macros to avoid calls in ISRs
#define OUTC(ch) do { \
    uint8_t __oc = (uint8_t)(ch); \
    __asm__ volatile("outb %0, $0xE9" : : "a"(__oc)); \
} while (0)

#define DUMP_HEX64(v) do { \
    static const char *_hex = "0123456789ABCDEF"; \
    uint64_t __v = (uint64_t)(v); \
    for (int _i = 15; _i >= 0; --_i) { \
        uint8_t __hc = (uint8_t)_hex[(__v >> (_i * 4)) & 0xF]; \
        OUTC(__hc); \
    } \
} while (0)

#define DUMP_HEX16(v) do { \
    static const char *_hex = "0123456789ABCDEF"; \
    uint16_t __v = (uint16_t)(v); \
    for (int _i = 3; _i >= 0; --_i) { \
        uint8_t __hc = (uint8_t)_hex[(__v >> (_i * 4)) & 0xF]; \
        OUTC(__hc); \
    } \
} while (0)

#define HALT_FOREVER() do { \
    for (;;) __asm__ volatile("cli; hlt"); \
} while (0)

__attribute__((unused))
static void dump_exc_common(uint8_t vec, uint64_t err, const struct interrupt_frame *frame, int has_cr2)
{
    uint64_t cr2 = 0;
    if (has_cr2) {
        __asm__ volatile("mov %%cr2, %0" : "=r"(cr2));
    }
    OUTC('['); OUTC('K'); OUTC(']'); OUTC('['); OUTC('E'); OUTC('X'); OUTC('C'); OUTC(']'); OUTC(' ');
    OUTC('v'); OUTC('e'); OUTC('c'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX16(vec);
    OUTC(' '); OUTC('e'); OUTC('r'); OUTC('r'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(err);
    OUTC(' '); OUTC('r'); OUTC('i'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(frame->rip);
    OUTC(' '); OUTC('r'); OUTC('s'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(frame->rsp);
    OUTC(' '); OUTC('c'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX16((uint16_t)frame->cs);
    OUTC(' '); OUTC('s'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX16((uint16_t)frame->ss);
    if (has_cr2) {
        OUTC(' '); OUTC('c'); OUTC('r'); OUTC('2'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(cr2);
    }
    OUTC('\n');
}

__attribute__((naked))
static void isr_pf_stub(void)
{
    __asm__ volatile(
        "movb $'F', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_gp_stub(void)
{
    __asm__ volatile(
        "movb $'G', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_df_stub(void)
{
    __asm__ volatile(
        "movb $'D', %al\n"
        "outb %al, $0xE9\n"
        "movb $'F', %al\n"
        "outb %al, $0xE9\n"
        "movb $'!', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((interrupt))
static void isr_bp(struct interrupt_frame *frame)
{
    const uint16_t cs = (uint16_t)frame->cs;
    const uint16_t ss = (uint16_t)frame->ss;
    const uint64_t rip = frame->rip;
    const uint64_t upper = rip >> 48;
    const uint64_t sign = (rip >> 47) & 1ULL;
    const int rip_canonical = sign ? (upper == 0xFFFFULL) : (upper == 0x0000ULL);
    const int is_ring3_bp =
        ((cs & 0x3u) == 0x3u) &&
        ((ss & 0x3u) == 0x3u) &&
        (cs == GDT_USER_CODE) &&
        (ss == GDT_USER_DATA) &&
        (rip >= 0x0000000000400000ULL) &&
        (rip < 0x00007FFFFFFFFFFFULL) &&
        rip_canonical;

    if (is_ring3_bp) {
        phase10_ring3_user_code_seen = 1u;
        // Source anchor token for runtime-marker-contract: P10_RING3_USER_CODE
        // ISR-safe marker emission: no helper calls in interrupt context.
        OUTC('P'); OUTC('1'); OUTC('0'); OUTC('_');
        OUTC('R'); OUTC('I'); OUTC('N'); OUTC('G'); OUTC('3'); OUTC('_');
        OUTC('U'); OUTC('S'); OUTC('E'); OUTC('R'); OUTC('_');
        OUTC('C'); OUTC('O'); OUTC('D'); OUTC('E'); OUTC('\n');
        HALT_FOREVER();
    }

    // Ring0 breakpoint: keep debug behavior and return.
    OUTC('B'); OUTC('P'); OUTC('!'); OUTC('\n');
}

__attribute__((interrupt))
static void isr_ud(struct interrupt_frame *frame)
{
    OUTC('['); OUTC('E'); OUTC('X'); OUTC(']'); OUTC('['); OUTC('#'); OUTC('U'); OUTC('D'); OUTC(']'); OUTC(' ');
    OUTC('r'); OUTC('i'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX64(frame->rip);
    OUTC(' '); OUTC('c'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX16((uint16_t)frame->cs);
    OUTC(' '); OUTC('r'); OUTC('s'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX64(frame->rsp);
    OUTC(' '); OUTC('r'); OUTC('f'); OUTC('l'); OUTC('a'); OUTC('g'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX64(frame->rflags);
    OUTC('\n');
    HALT_FOREVER();
}

__attribute__((naked))
static void isr_ts_stub(void)
{
    __asm__ volatile(
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'!', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_np_stub(void)
{
    __asm__ volatile(
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'P', %al\n"
        "outb %al, $0xE9\n"
        "movb $'!', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_ss_stub(void)
{
    __asm__ volatile(
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'!', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((interrupt))
static void isr_gp(struct interrupt_frame *frame, uint64_t error_code)
{
    (void)error_code;
    // CRITICAL: GP fault marker - ASM safe, no C calls
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'G'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'P'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'!'), "Nd"(0xE9));
    
    // Show RIP where GP occurred (simple hex dump)
    uint64_t rip = frame->rip;
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nibble = (rip >> i) & 0xF;
        uint8_t ch = (nibble < 10) ? ('0' + nibble) : ('A' + nibble - 10);
        __asm__ volatile("outb %0, %1" : : "a"(ch), "Nd"(0xE9));
    }
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'\n'), "Nd"(0xE9));
    
    // Halt forever - no C calls
    __asm__ volatile("cli; 1: hlt; jmp 1b");
}

__attribute__((interrupt))
static void isr_pf(struct interrupt_frame *frame, uint64_t error_code)
{
    uint64_t cr2 = 0;
    __asm__ volatile("mov %%cr2, %0" : "=r"(cr2));

    // CRITICAL: Page fault marker - keep ASM-safe emission.
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'P'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'F'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'!'), "Nd"(0xE9));

    // Fault RIP (kept first for compatibility with existing logs/tools).
    uint64_t rip = frame->rip;
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nibble = (rip >> i) & 0xF;
        uint8_t ch = (nibble < 10) ? ('0' + nibble) : ('A' + nibble - 10);
        __asm__ volatile("outb %0, %1" : : "a"(ch), "Nd"(0xE9));
    }
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)' '), "Nd"(0xE9));
    OUTC('C'); OUTC('R'); OUTC('2'); OUTC('='); DUMP_HEX64(cr2);
    OUTC(' ');
    OUTC('E'); OUTC('R'); OUTC('R'); OUTC('='); DUMP_HEX64(error_code);
    OUTC(' ');
    OUTC('C'); OUTC('S'); OUTC('='); DUMP_HEX16((uint16_t)frame->cs);
    OUTC(' ');
    OUTC('S'); OUTC('S'); OUTC('='); DUMP_HEX16((uint16_t)frame->ss);
    OUTC(' ');
    OUTC('R'); OUTC('S'); OUTC('P'); OUTC('='); DUMP_HEX64(frame->rsp);
    OUTC(' ');
    OUTC('P'); OUTC('I'); OUTC('D'); OUTC('=');
    if (current_proc) {
        DUMP_HEX64((uint64_t)(uint32_t)current_proc->pid);
        OUTC(' ');
        OUTC('P'); OUTC('C'); OUTC('S'); OUTC('='); DUMP_HEX16(current_proc->context.cs);
        OUTC(' ');
        OUTC('P'); OUTC('R'); OUTC('I'); OUTC('P'); OUTC('='); DUMP_HEX64(current_proc->context.rip);
        OUTC(' ');
        OUTC('P'); OUTC('R'); OUTC('S'); OUTC('P'); OUTC('='); DUMP_HEX64(current_proc->context.rsp);
        OUTC(' ');
        OUTC('P'); OUTC('C'); OUTC('R'); OUTC('3'); OUTC('='); DUMP_HEX64(current_proc->context.cr3);
    } else {
        OUTC('N'); OUTC('U'); OUTC('L'); OUTC('L');
    }
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'\n'), "Nd"(0xE9));

    // Halt forever - no recovery from early PF in validation path.
    __asm__ volatile("cli; 1: hlt; jmp 1b");
}

void idt_set_gate(uint8_t num, interrupt_handler_t handler, uint8_t flags)
{
    uint64_t addr = (uint64_t)handler;
    idt_table[num].offset_low = addr & 0xFFFF;
    idt_table[num].selector = GDT_KERNEL_CODE; // kernel code segment
    idt_table[num].ist = 0;
    idt_table[num].type_attr = flags;
    idt_table[num].offset_mid = (addr >> 16) & 0xFFFF;
    idt_table[num].offset_high = (addr >> 32) & 0xFFFFFFFF;
    idt_table[num].zero = 0;
}

static void idt_set_gate_selector(uint8_t num, interrupt_handler_t handler, uint8_t flags, uint16_t selector)
{
    uint64_t addr = (uint64_t)handler;
    idt_table[num].offset_low = addr & 0xFFFF;
    idt_table[num].selector = selector;
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

    // Install core exception handlers for the late IDT (early IDT gets wiped above).
    // INT3 uses interrupt gate (DPL=3) to keep marker emission deterministic.
    idt_set_gate(3,  (interrupt_handler_t)isr_bp, 0xEE);
    idt_set_gate(6,  isr_ud, 0x8F);
    idt_set_gate(8,  (interrupt_handler_t)isr_df_stub, 0x8F);
    idt_set_gate(10, (interrupt_handler_t)isr_ts_stub, 0x8F);
    idt_set_gate(11, (interrupt_handler_t)isr_np_stub, 0x8F);
    idt_set_gate(12, (interrupt_handler_t)isr_ss_stub, 0x8F);
/* Validation builds use verbose #GP/#PF handlers to surface fault RIP quickly. */
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    idt_set_gate(13, (interrupt_handler_t)isr_gp, 0x8F);
    idt_set_gate(14, (interrupt_handler_t)isr_pf, 0x8F);
#else
    idt_set_gate(13, (interrupt_handler_t)isr_gp_stub, 0x8F);
    idt_set_gate(14, (interrupt_handler_t)isr_pf_stub, 0x8F);
#endif

    // Keep current-stack delivery for diagnostic consistency during bring-up.
    idt_table[3].ist  = 0;
    idt_table[8].ist  = 0;
    idt_table[10].ist = 0;
    idt_table[11].ist = 0;
    idt_table[12].ist = 0;
    idt_table[13].ist = 0;
    idt_table[14].ist = 0;

    idt_descriptor.limit = sizeof(idt_table) - 1;
    idt_descriptor.base = (uint64_t)&idt_table[0];

    idt_init();
}

void interrupts_install_early(void)
{
    uint16_t cs = 0;
    __asm__ volatile("mov %%cs, %0" : "=r"(cs));

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

    // Exceptions we care about early.
    // CRITICAL: INT3 (#BP) is DPL=3 interrupt gate for deterministic marker path.
    idt_set_gate_selector(3,  (interrupt_handler_t)isr_bp, 0xEE, cs);
    idt_set_gate_selector(6,  isr_ud, 0x8F, cs);
    idt_set_gate_selector(8,  (interrupt_handler_t)isr_df_stub, 0x8F, cs);
    idt_set_gate_selector(10, (interrupt_handler_t)isr_ts_stub, 0x8F, cs);
    idt_set_gate_selector(11, (interrupt_handler_t)isr_np_stub, 0x8F, cs);
    idt_set_gate_selector(12, (interrupt_handler_t)isr_ss_stub, 0x8F, cs);
/* Validation builds use verbose #GP/#PF handlers to surface fault RIP quickly. */
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    idt_set_gate_selector(13, (interrupt_handler_t)isr_gp, 0x8F, cs);
    idt_set_gate_selector(14, (interrupt_handler_t)isr_pf, 0x8F, cs);
#else
    idt_set_gate_selector(13, (interrupt_handler_t)isr_gp_stub, 0x8F, cs);
    idt_set_gate_selector(14, (interrupt_handler_t)isr_pf_stub, 0x8F, cs);
#endif
    // Keep current-stack delivery for diagnostic consistency during bring-up.
    idt_table[3].ist  = 0;
    idt_table[8].ist  = 0;
    idt_table[10].ist = 0;
    idt_table[11].ist = 0;
    idt_table[12].ist = 0;
    idt_table[13].ist = 0;
    idt_table[14].ist = 0;

    idt_descriptor.limit = sizeof(idt_table) - 1;
    idt_descriptor.base = (uint64_t)&idt_table[0];

    idt_init();

}
