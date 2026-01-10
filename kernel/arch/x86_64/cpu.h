#pragma once

#include <stdint.h>

// TSS (Task State Segment) structure for x86_64
typedef struct __attribute__((packed)) {
    uint32_t reserved1;
    uint64_t rsp0;      // Ring 0 stack pointer
    uint64_t rsp1;      // Ring 1 stack pointer  
    uint64_t rsp2;      // Ring 2 stack pointer
    uint64_t reserved2;
    uint64_t ist1;      // Interrupt Stack Table entries
    uint64_t ist2;
    uint64_t ist3;
    uint64_t ist4;
    uint64_t ist5;
    uint64_t ist6;
    uint64_t ist7;
    uint64_t reserved3;
    uint16_t reserved4;
    uint16_t iomap_base;
} tss_t;

void cpu_init(void);
void tss_init(void);
void tss_set_kernel_stack(uint64_t stack_ptr);

// Low-level interrupt flag helpers
static inline void enable_interrupts(void) { __asm__ volatile("sti" ::: "memory"); }
static inline void disable_interrupts(void) { __asm__ volatile("cli" ::: "memory"); }

// Context switch routines (implemented in assembly)
struct cpu_context;
void context_switch(struct cpu_context *old_ctx, struct cpu_context *new_ctx);
void switch_to_first(struct cpu_context *ctx);
