#include <stdint.h>
#include "gdt_idt.h"

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

// IDT Entry Structure
struct idt_entry {
    uint16_t offset_low;
    uint16_t selector;
    uint8_t ist;
    uint8_t type_attr;
    uint16_t offset_mid;
    uint32_t offset_high;
    uint32_t zero;
} __attribute__((packed));

// IDT Pointer Structure
struct idt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

// Global GDT (6 entries: null, kernel code, kernel data, user data, user code, tss)
static struct gdt_entry gdt[6] __attribute__((section(".data"))) = {
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
    // Entry 5: TSS Descriptor (two entries used, filled in gdt_init)
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
static inline void lidt(void *base, uint16_t size)
{
    struct {
        uint16_t length;
        void *base;
    } __attribute__((packed)) IDTR = { size, base };
    __asm__ volatile("lidt %0" : : "m"(IDTR));
}

// Load TSS using inline asm
static inline void ltr(uint16_t tss_selector)
{
    __asm__ volatile("ltr %0" : : "r"(tss_selector));
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
    
    // Load GDT (limit = 8 * 6 - 1 = 47, but we have 2 entries for TSS, so 8 * 7 - 1 = 55)
    struct gdt_ptr gdt_descriptor = {
        .limit = sizeof(gdt) + 8 - 1,  // TSS takes extra 8 bytes
        .base = (uint64_t)&gdt[0],
    };
    
    lgdt(&gdt_descriptor, gdt_descriptor.limit);
    
    // Load TSS (selector 5 << 3 = 0x28)
    ltr(GDT_TSS_SEL);
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
}

void idt_init(void)
{
    // IDT will be filled by interrupt setup; just load descriptor
    struct idt_ptr idt_descriptor = {
        .limit = sizeof(struct idt_entry) * 256 - 1,
        .base = (uint64_t)idt_table,
    };
    lidt(&idt_descriptor, idt_descriptor.limit);
}

static void idt_set_gate_local(uint8_t num, void (*handler)(void), uint8_t flags)
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
