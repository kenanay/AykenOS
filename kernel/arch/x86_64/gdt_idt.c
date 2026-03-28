#include <stdint.h>
#include <stddef.h>
#include "gdt_idt.h"
#include "interrupts.h"

_Static_assert(offsetof(tss_entry_t, rsp0) == 0x04, "tss.rsp0 offset must match x86_64 spec");
_Static_assert(offsetof(tss_entry_t, ist1) == 0x24, "tss.ist1 offset must match x86_64 spec");
_Static_assert(sizeof(tss_entry_t) == 0x68, "tss size must match x86_64 spec");
_Static_assert(GDT_USER_DATA == ((3u << 3) | 3u), "user data selector must match GDT entry 3");
_Static_assert(GDT_USER_CODE == ((4u << 3) | 3u), "user code selector must match GDT entry 4");

// GDT Entry Structure (for 64-bit)
struct gdt_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t base_mid;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
} __attribute__((packed));

// TSS Descriptor (16 bytes in 64-bit mode)
struct tss_descriptor {
    uint16_t length;
    uint16_t base_low;
    uint8_t base_mid;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
    uint32_t base_ext;
    uint32_t reserved;
} __attribute__((packed));

// GDT Pointer Structure
struct gdt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

// Global GDT (7 entries: null, kernel code, kernel data, user data, user code, tss low, tss high)
static struct gdt_entry gdt[7] __attribute__((section(".data"))) = {
    // Entry 0: Null descriptor (required)
    {
        .limit_low = 0,
        .base_low = 0,
        .base_mid = 0,
        .access = 0,
        .granularity = 0,
        .base_high = 0,
    },
    // Entry 1: Kernel Code (DPL=0, Executable)
    {
        .limit_low = 0xFFFF,
        .base_low = 0,
        .base_mid = 0,
        .access = 0x9B,        // P=1, DPL=0, Type=0x0B (exec)
        .granularity = 0xA0,   // G=1 (4K pages), L=1 (64-bit)
        .base_high = 0,
    },
    // Entry 2: Kernel Data (DPL=0)
    {
        .limit_low = 0xFFFF,
        .base_low = 0,
        .base_mid = 0,
        .access = 0x93,        // P=1, DPL=0, Type=0x03 (data)
        .granularity = 0xC0,   // G=1, D=1 (32-bit default)
        .base_high = 0,
    },
    // Entry 3: User Data (DPL=3)
    {
        .limit_low = 0xFFFF,
        .base_low = 0,
        .base_mid = 0,
        .access = 0xF3,        // P=1, DPL=3, Type=0x03 (data)
        .granularity = 0xC0,   // G=1, D=1
        .base_high = 0,
    },
    // Entry 4: User Code (DPL=3, Executable)
    {
        .limit_low = 0xFFFF,
        .base_low = 0,
        .base_mid = 0,
        .access = 0xFB,        // P=1, DPL=3, Type=0x0B (exec)
        .granularity = 0xA0,   // G=1, L=1 (64-bit)
        .base_high = 0,
    },
    // Entry 5: TSS Descriptor (low 8 bytes; high 8 bytes is entry 6, filled in gdt_init)
    {
        .limit_low = 0,
        .base_low = 0,
        .base_mid = 0,
        .access = 0,
        .granularity = 0,
        .base_high = 0,
    },
};

// Global TSS (used for Ring 0 kernel stack and interrupt handling)
tss_entry_t kernel_tss __attribute__((section(".data"))) = {
    .reserved0 = 0,
    .rsp0 = 0,  // Will be set at runtime
    .rsp1 = 0,
    .rsp2 = 0,
    .reserved1 = 0,
    .ist1 = 0,
    .ist2 = 0,
    .ist3 = 0,
    .ist4 = 0,
    .ist5 = 0,
    .ist6 = 0,
    .ist7 = 0,
    .reserved2 = 0,
    .reserved3 = 0,
    .io_map_base = sizeof(tss_entry_t),  // IO Map not used for now
};

extern struct idt_entry idt_table[256];

// Load GDT using inline asm
static inline void lgdt(void *base, uint16_t size)
{
    struct {
        uint16_t length;
        void *base;
    } __attribute__((packed)) GDTR = { size, base };
    __asm__ volatile("lgdt %0" : : "m"(GDTR));
    
    // Reload code segment and stack segment
    __asm__ volatile(
        "pushq $0x08\n"          // Push new code segment selector (GDT_KERNEL_CODE = 0x08)
        "leaq 1f(%%rip), %%rax\n"
        "pushq %%rax\n"
        "lretq\n"
        "1:\n"
        "movq $0x10, %%rax\n"    // GDT_KERNEL_DATA = 0x10
        "movq %%rax, %%ds\n"
        "movq %%rax, %%es\n"
        "movq %%rax, %%ss\n"
        : : : "rax"
    );
}

// Load IDT using inline asm
static inline void lidt_ptr(const struct idt_ptr *idtr)
{
    __asm__ volatile("lidt %0" : : "m"(*idtr));
}

// Load TSS using inline asm
static inline void ltr(uint16_t tss_selector)
{
    __asm__ volatile("ltr %0" : : "r"(tss_selector));
}

static inline void dbg_outc(uint8_t c)
{
    __asm__ volatile("outb %0, $0xE9" : : "a"(c));
}

static inline void dbg_dump_hex16(uint16_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    dbg_outc((uint8_t)hex[(v >> 12) & 0xF]);
    dbg_outc((uint8_t)hex[(v >> 8) & 0xF]);
    dbg_outc((uint8_t)hex[(v >> 4) & 0xF]);
    dbg_outc((uint8_t)hex[v & 0xF]);
}

static inline void dbg_dump_tr(void)
{
    uint16_t tr = 0;
    __asm__ volatile("str %0" : "=r"(tr));
    dbg_outc((uint8_t)'L');
    dbg_outc((uint8_t)'T');
    dbg_outc((uint8_t)'R');
    dbg_outc((uint8_t)'=');
    dbg_dump_hex16(tr);
    dbg_outc((uint8_t)'\n');
}

void gdt_init(void)
{
    // Set up TSS descriptor (GDT entry 5)
    // TSS descriptors in 64-bit are 16 bytes (2 GDT entries)
    uint64_t tss_base = (uint64_t)&kernel_tss;
    uint32_t tss_limit = sizeof(tss_entry_t) - 1;
    
    struct tss_descriptor *tss_desc = (struct tss_descriptor *)&gdt[5];
    tss_desc->length = tss_limit & 0xFFFF;
    tss_desc->base_low = tss_base & 0xFFFF;
    tss_desc->base_mid = (tss_base >> 16) & 0xFF;
    tss_desc->access = 0x89;   // P=1, DPL=0, Type=0x09 (TSS, not busy)
    tss_desc->granularity = 0x00;  // No granularity bit for TSS
    tss_desc->base_high = (tss_base >> 24) & 0xFF;
    tss_desc->base_ext = (tss_base >> 32) & 0xFFFFFFFF;
    tss_desc->reserved = 0;
    
    // Load GDT (7 entries total; TSS uses entries 5 and 6)
    struct gdt_ptr gdt_descriptor = {
        .limit = sizeof(gdt) - 1,
        .base = (uint64_t)&gdt[0],
    };
    
    lgdt((void *)&gdt[0], gdt_descriptor.limit);
    
    // Load TSS (selector 5 << 3 = 0x28)
    ltr(GDT_TSS_SEL);
    dbg_dump_tr();
}

void gdt_install_tss(uint64_t tss_addr)
{
    // Update TSS descriptor with new address
    struct tss_descriptor *tss_desc = (struct tss_descriptor *)&gdt[5];
    uint32_t tss_limit = sizeof(tss_entry_t) - 1;
    
    tss_desc->length = tss_limit & 0xFFFF;
    tss_desc->base_low = tss_addr & 0xFFFF;
    tss_desc->base_mid = (tss_addr >> 16) & 0xFF;
    tss_desc->access = 0x89;   // P=1, DPL=0, Type=0x09 (TSS, not busy)
    tss_desc->granularity = 0x00;
    tss_desc->base_high = (tss_addr >> 24) & 0xFF;
    tss_desc->base_ext = (tss_addr >> 32) & 0xFFFFFFFF;
    tss_desc->reserved = 0;
    
    // Load TSS register
    ltr(GDT_TSS_SEL);
    dbg_dump_tr();
}

void idt_init(void)
{
    // IDT descriptor is prepared by interrupts_install[_early] before idt_init().
    lidt_ptr(&idt_descriptor);
}

void idt_set_gate_raw(uint8_t num, void (*handler)(void), uint8_t flags)
{
    uint64_t handler_addr = (uint64_t)handler;
    
    idt_table[num].offset_low = handler_addr & 0xFFFF;
    idt_table[num].selector = GDT_KERNEL_CODE;
    idt_table[num].ist = 0;
    idt_table[num].type_attr = flags;
    idt_table[num].offset_mid = (handler_addr >> 16) & 0xFFFF;
    idt_table[num].offset_high = (handler_addr >> 32) & 0xFFFFFFFF;
    idt_table[num].zero = 0;
}
