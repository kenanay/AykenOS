#pragma once

#include <stdint.h>
#include "../../include/gdt_idt.h"
#include "interrupts.h"

// Additional arch-specific functions
void gdt_install_tss(uint64_t tss_addr);
void isr_init_stubs(void);
void idt_set_gate_raw(uint8_t num, void (*handler)(void), uint8_t flags);
