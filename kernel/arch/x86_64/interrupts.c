#include <stdint.h>
#include "interrupts.h"
#include "gdt_idt.h"
#include "port_io.h"

struct idt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

struct idt_entry idt_table[256];
struct idt_ptr idt_descriptor;

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
static void isr_bp_stub(void)
{
    __asm__ volatile(
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
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
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((interrupt))
static void isr_bp(struct interrupt_frame *frame)
{
    (void)frame;
    OUTC('B');
    HALT_FOREVER();
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

__attribute__((interrupt))
static void isr_df(struct interrupt_frame *frame, uint64_t error_code)
{
    dump_exc_common(8, error_code, frame, 0);
    HALT_FOREVER();
}

__attribute__((interrupt))
static void isr_ts(struct interrupt_frame *frame, uint64_t error_code)
{
    dump_exc_common(10, error_code, frame, 0);
    HALT_FOREVER();
}

__attribute__((interrupt))
static void isr_np(struct interrupt_frame *frame, uint64_t error_code)
{
    dump_exc_common(11, error_code, frame, 0);
    HALT_FOREVER();
}

__attribute__((interrupt))
static void isr_ss(struct interrupt_frame *frame, uint64_t error_code)
{
    dump_exc_common(12, error_code, frame, 0);
    HALT_FOREVER();
}

__attribute__((interrupt))
static void isr_gp(struct interrupt_frame *frame, uint64_t error_code)
{
    dump_exc_common(13, error_code, frame, 0);
    HALT_FOREVER();
}

__attribute__((interrupt))
static void isr_pf(struct interrupt_frame *frame, uint64_t error_code)
{
    dump_exc_common(14, error_code, frame, 1);
    HALT_FOREVER();
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

    // Install core exception handlers for the late IDT (early IDT gets wiped above)
    // Use trap gates for debug visibility; allow Ring3 INT3 with DPL=3.
    idt_set_gate(3,  (interrupt_handler_t)isr_bp_stub, 0xEF);
    idt_set_gate(6,  isr_ud, 0x8F);
    idt_set_gate(8,  (interrupt_handler_t)isr_df, 0x8F);
    idt_set_gate(10, (interrupt_handler_t)isr_ts, 0x8F);
    idt_set_gate(11, (interrupt_handler_t)isr_np, 0x8F);
    idt_set_gate(12, (interrupt_handler_t)isr_ss, 0x8F);
    idt_set_gate(13, (interrupt_handler_t)isr_gp, 0x8F);
    idt_set_gate(14, (interrupt_handler_t)isr_pf, 0x8F);

    // Route critical exceptions to IST1 to avoid stack issues
    // Keep BP on current stack to avoid IST setup dependency during debug
    idt_table[3].ist  = 0;
    idt_table[8].ist  = 1;
    idt_table[10].ist = 1;
    idt_table[11].ist = 1;
    idt_table[12].ist = 1;
    idt_table[13].ist = 1;
    idt_table[14].ist = 1;

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

    // Debug: prove install runs and which CS is used
    OUTC('E'); OUTC('C'); OUTC('='); DUMP_HEX16(cs); OUTC('\n');

    // Exceptions we care about early
    // Use trap gates (0x8F) so IF is preserved during debug
    idt_set_gate_selector(3,  (interrupt_handler_t)isr_bp_stub, 0x8F, cs);
    idt_set_gate_selector(6,  isr_ud, 0x8F, cs);
    idt_set_gate_selector(8,  (interrupt_handler_t)isr_df, 0x8F, cs);
    idt_set_gate_selector(10, (interrupt_handler_t)isr_ts, 0x8F, cs);
    idt_set_gate_selector(11, (interrupt_handler_t)isr_np, 0x8F, cs);
    idt_set_gate_selector(12, (interrupt_handler_t)isr_ss, 0x8F, cs);
    idt_set_gate_selector(13, (interrupt_handler_t)isr_gp, 0x8F, cs);
    idt_set_gate_selector(14, (interrupt_handler_t)isr_pf, 0x8F, cs);
    // Keep BP on current stack to avoid IST setup dependency during debug
    idt_table[3].ist  = 0;
    idt_table[8].ist  = 1;
    idt_table[10].ist = 1;
    idt_table[11].ist = 1;
    idt_table[12].ist = 1;
    idt_table[13].ist = 1;
    idt_table[14].ist = 1;

    // Debug: dump IDT[3] selector/type_attr/offset
    OUTC('S'); OUTC('3'); OUTC('='); DUMP_HEX16(idt_table[3].selector); OUTC(' ');
    OUTC('T'); OUTC('3'); OUTC('='); OUTC('0'); OUTC('x');
    {
        static const char *_hx = "0123456789ABCDEF";
        uint8_t ta = idt_table[3].type_attr;
        OUTC(_hx[(ta >> 4) & 0xF]);
        OUTC(_hx[ta & 0xF]);
    }
    OUTC(' ');
    OUTC('O'); OUTC('3'); OUTC('=');
    {
        uint64_t off = ((uint64_t)idt_table[3].offset_high << 32) |
                       ((uint64_t)idt_table[3].offset_mid  << 16) |
                       (uint64_t)idt_table[3].offset_low;
        DUMP_HEX64(off);
    }
    OUTC('\n');

    idt_descriptor.limit = sizeof(idt_table) - 1;
    idt_descriptor.base = (uint64_t)&idt_table[0];

    idt_init();

    // Debug: confirm IDTR contents after lidt
    struct { uint16_t limit; uint64_t base; } __attribute__((packed)) idtr;
    __asm__ volatile("sidt %0" : "=m"(idtr));
    OUTC('I'); OUTC('D'); OUTC('T'); OUTC('R'); OUTC('=');
    DUMP_HEX16(idtr.limit); OUTC(':'); DUMP_HEX64(idtr.base); OUTC('\n');
}
