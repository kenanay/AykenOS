#pragma once

void cpu_init(void);

// Low-level interrupt flag helpers
static inline void enable_interrupts(void) { __asm__ volatile("sti" ::: "memory"); }
static inline void disable_interrupts(void) { __asm__ volatile("cli" ::: "memory"); }

// Context switch routines (implemented in assembly)
struct cpu_context;
void context_switch(struct cpu_context *old_ctx, struct cpu_context *new_ctx);
void switch_to_first(struct cpu_context *ctx);
