// kernel/include/gdt_idt.h
#ifndef AYKEN_GDT_IDT_H
#define AYKEN_GDT_IDT_H

#include <stdint.h>
#include "ring3_contract.h"

// GDT Segment Selectors
#define GDT_KERNEL_CODE 0x08   // Selector 1 << 3
#define GDT_KERNEL_DATA 0x10   // Selector 2 << 3
#define GDT_USER_DATA   AYKEN_RING3_USER_DATA_SELECTOR
#define GDT_USER_CODE   AYKEN_RING3_USER_CODE_SELECTOR
#define GDT_TSS_SEL     0x28   // Selector 5 << 3

// TSS Entry
typedef struct __attribute__((packed)) {
    uint32_t reserved0;
    uint64_t rsp0;          // Kernel RSP for Ring 0 (set when context switching to Ring3)
    uint64_t rsp1;
    uint64_t rsp2;
    uint64_t reserved1;
    uint64_t ist1;          // Interrupt Stack Table
    uint64_t ist2;
    uint64_t ist3;
    uint64_t ist4;
    uint64_t ist5;
    uint64_t ist6;
    uint64_t ist7;
    uint64_t reserved2;
    uint16_t reserved3;
    uint16_t io_map_base;
} tss_entry_t;

// Extern TSS from gdt_idt.c
extern tss_entry_t kernel_tss;

// API
void gdt_init(void);
void idt_init(void);

// Update TSS RSP0 when setting kernel stack for Ring3 processes
static inline void gdt_set_kernel_stack(uint64_t rsp0)
{
    kernel_tss.rsp0 = rsp0;
}

#endif // AYKEN_GDT_IDT_H
