// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Ring3 Jump Implementation (Phase 10-A2 Task 2)
// Authority: Phase 10 specification
// Constitutional: Ring0 mechanism only (no policy)

#include <stdint.h>
#include "arch/x86_64/port_io.h"
#include "drivers/console/fb_console.h"
#include "embedded_elf.h"
#include "ring3_jump.h"
#include "mm/user_as.h"
#include "mm.h"
#include "errno.h"

// Page flags (from mm.h)
#define AYKEN_PTE_PRESENT  (1ULL << 0)
#define AYKEN_PTE_WRITE    (1ULL << 1)
#define AYKEN_PTE_USER     (1ULL << 2)

// Debug output via debugcon (port 0xE9)
static void debugcon_write(const char *s)
{
    if (!s) return;
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        s++;
    }
}

static void debugcon_hex(uint64_t val)
{
    char buf[19]; // "0x" + 16 hex digits + null
    buf[0] = '0';
    buf[1] = 'x';
    for (int i = 15; i >= 0; i--) {
        uint8_t nibble = (val >> (i * 4)) & 0xF;
        buf[17 - i] = (nibble < 10) ? ('0' + nibble) : ('A' + nibble - 10);
    }
    buf[18] = '\0';
    debugcon_write(buf);
}

static void halt_forever(void)
{
    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static void ring3_panic(const char *marker, const char *msg)
{
    if (marker) {
        debugcon_write(marker);
        debugcon_write("\n");
    }
    if (msg) {
        fb_print(msg);
        fb_print("\n");
        debugcon_write(msg);
        debugcon_write("\n");
    }
    halt_forever();
}

// Validate canonical address (x86_64: bits 63-48 must be sign extension of bit 47)
static int is_canonical(uint64_t addr)
{
    uint64_t sign_bit = (addr >> 47) & 1;
    uint64_t high_bits = (addr >> 48) & 0xFFFF;
    return (sign_bit == 0 && high_bits == 0) || (sign_bit == 1 && high_bits == 0xFFFF);
}

// Phase 10-A2 Task 2: Direct Ring3 transition test
// This bypasses the scheduler for initial validation
void jump_to_ring3(void)
{
    debugcon_write("[K][PHASE10] KERNEL_BEFORE_RING3\n");
    fb_print("[PHASE10] Preparing Ring3 transition...\n");

    // Step 1: Validate embedded ELF
    if (!embedded_elf || embedded_elf_size == 0) {
        ring3_panic("[[AYKEN_RING3_FAIL]] no_elf",
                    "[PANIC] Phase10: No embedded ELF found.");
    }

    debugcon_write("[K][PHASE10] ELF_FOUND size=");
    debugcon_hex(embedded_elf_size);
    debugcon_write("\n");

    // Step 2: Parse ELF header (minimal validation)
    // ELF magic: 0x7F 'E' 'L' 'F'
    if (embedded_elf[0] != 0x7F || embedded_elf[1] != 'E' ||
        embedded_elf[2] != 'L' || embedded_elf[3] != 'F') {
        ring3_panic("[[AYKEN_RING3_FAIL]] bad_elf_magic",
                    "[PANIC] Phase10: Invalid ELF magic.");
    }

    // Extract entry point (offset 0x18 in ELF64 header)
    uint64_t user_rip = *(uint64_t *)(embedded_elf + 0x18);
    
    debugcon_write("[K][PHASE10] ELF_ENTRY rip=");
    debugcon_hex(user_rip);
    debugcon_write("\n");

    // Step 3: Validate entry point
    if (!is_canonical(user_rip)) {
        ring3_panic("[[AYKEN_RING3_FAIL]] non_canonical_rip",
                    "[PANIC] Phase10: Non-canonical user RIP.");
    }

    // Step 4: Allocate user address space
    user_as_t user_as;
    int ret = user_as_create(&user_as);
    if (ret != 0) {
        ring3_panic("[[AYKEN_RING3_FAIL]] no_user_as",
                    "[PANIC] Phase10: Failed to create user address space.");
    }

    debugcon_write("[K][PHASE10] USER_AS_CREATED cr3=");
    debugcon_hex(user_as.cr3_phys);
    debugcon_write("\n");

    // Step 5: Map user stack (1 page at 0x7FFFFFFFE000)
    uint64_t user_stack_base = 0x7FFFFFFFE000ULL;
    uint64_t user_stack_top = user_stack_base + 0x1000; // 4KB stack
    
    // Allocate physical frame for stack
    uint64_t stack_phys = phys_alloc_frame();
    if (!stack_phys) {
        ring3_panic("[[AYKEN_RING3_FAIL]] stack_alloc_failed",
                    "[PANIC] Phase10: Failed to allocate stack frame.");
    }

    // Map stack page in user address space
    paging_map_page_in_pml4(user_as.cr3_phys, user_stack_base, stack_phys,
                            AYKEN_PTE_PRESENT | AYKEN_PTE_WRITE | AYKEN_PTE_USER);

    debugcon_write("[K][PHASE10] USER_STACK_MAPPED base=");
    debugcon_hex(user_stack_base);
    debugcon_write(" phys=");
    debugcon_hex(stack_phys);
    debugcon_write("\n");

    // Step 6: Map user code (minimal: 1 page at entry point)
    uint64_t code_page = user_rip & ~0xFFFULL; // Align to page boundary
    
    // Allocate physical frame for code
    uint64_t code_phys = phys_alloc_frame();
    if (!code_phys) {
        ring3_panic("[[AYKEN_RING3_FAIL]] code_alloc_failed",
                    "[PANIC] Phase10: Failed to allocate code frame.");
    }

    // Map code page in user address space (read-only for now)
    paging_map_page_in_pml4(user_as.cr3_phys, code_page, code_phys,
                            AYKEN_PTE_PRESENT | AYKEN_PTE_USER);

    debugcon_write("[K][PHASE10] USER_CODE_MAPPED page=");
    debugcon_hex(code_page);
    debugcon_write(" phys=");
    debugcon_hex(code_phys);
    debugcon_write("\n");

    // Step 7: Copy minimal code to user page
    // For now, just put a HLT instruction (0xF4) to test transition
    uint8_t *code_virt = (uint8_t *)paging_phys_to_virt(code_phys);
    code_virt[0] = 0xF4; // HLT instruction
    
    debugcon_write("[K][PHASE10] USER_CODE_INITIALIZED (HLT stub)\n");

    // Step 8: Validate user RSP
    uint64_t user_rsp = user_stack_top - 16; // 16-byte alignment required
    
    if (!is_canonical(user_rsp)) {
        ring3_panic("[[AYKEN_RING3_FAIL]] non_canonical_rsp",
                    "[PANIC] Phase10: Non-canonical user RSP.");
    }

    if ((user_rsp & 0xF) != 0) {
        ring3_panic("[[AYKEN_RING3_FAIL]] misaligned_rsp",
                    "[PANIC] Phase10: Misaligned user RSP.");
    }

    // Step 9: Validate CR3
    if (user_as.cr3_phys == 0 || (user_as.cr3_phys & 0xFFF) != 0) {
        ring3_panic("[[AYKEN_RING3_FAIL]] invalid_cr3",
                    "[PANIC] Phase10: Invalid user CR3.");
    }

    // Step 10: Final marker before transition
    debugcon_write("[K][PHASE10] PRECONDITIONS_VALID\n");
    debugcon_write("[K][PHASE10] TRANSITION_PARAMS:\n");
    debugcon_write("  RIP=");
    debugcon_hex(code_page); // Use page base for now
    debugcon_write("\n  RSP=");
    debugcon_hex(user_rsp);
    debugcon_write("\n  CR3=");
    debugcon_hex(user_as.cr3_phys);
    debugcon_write("\n");

    fb_print("[PHASE10] Entering Ring3...\n");
    debugcon_write("[K][PHASE10] ENTERING_RING3\n");

    // Step 11: Transition to Ring3
    // NOTE: This function does NOT return
    ring3_enter(code_page, user_rsp, user_as.cr3_phys);

    // UNREACHABLE
    ring3_panic("[[AYKEN_RING3_FAIL]] returned_from_ring3",
                "[PANIC] Phase10: Returned from ring3_enter (impossible).");
}
