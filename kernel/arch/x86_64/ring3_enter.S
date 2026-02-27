// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Canonical Ring3 IRETQ Entry Helpers
// Authority: Phase 10-A2 Task 2
// Constitutional: Ring0 mechanism only (no policy)

.section .text
.global ring3_enter
.global ring3_enter_iretq

// Segment selectors (with RPL=3 baked in)
.set SEG_USER_DATA,   0x1B
.set SEG_USER_CODE,   0x23

// RFLAGS policy
.set RFLAGS_BASE,     0x202
.set RFLAGS_CLEARMASK, ((3 << 12) | (1 << 14) | (1 << 16) | (1 << 17) | (1 << 8))

#ifndef AYKEN_CR3_PCID
#define AYKEN_CR3_PCID 0
#endif

// Marker emission helper (debugcon, port 0xE9)
.macro EMIT_CSTR label
    leaq \label(%rip), %rsi
1:
    movzbq (%rsi), %rax
    testb %al, %al
    je 2f
    outb %al, $0xE9
    inc %rsi
    jmp 1b
2:
.endm

//
// ring3_enter_iretq(rip, rsp, rflags, user_cr3) noreturn
//
// RDI = user RIP
// RSI = user RSP
// RDX = user RFLAGS (will be sanitized)
// RCX = user CR3 (physical, page-aligned)
//
// This is the canonical Ring3 entry path used by scheduler ASM.
//
ring3_enter_iretq:
    // Preserve transition inputs in caller-saved registers only.
    mov %rsi, %r10            // r10 = user_rsp
    mov %rdi, %r11            // r11 = user_rip

    // Sanitize target user RFLAGS.
    mov %rdx, %rax
    andq $~RFLAGS_CLEARMASK, %rax
    orq $RFLAGS_BASE, %rax
    mov %rax, %rdx            // rdx = sanitized user rflags

    // Enforce CR3 policy (PCID-aware, fail-closed).
#if AYKEN_CR3_PCID == 1
    // Allow only: NOFLUSH[63], PML4 base[51:12], PCID[11:0].
    // Reserved bits [62:52] are always cleared.
    mov %rcx, %rax
    movabsq $0x800FFFFFFFFFF000, %r8
    andq %r8, %rcx
    andq $0x0000000000000FFF, %rax
    orq  %rax, %rcx
#else
    // PCID disabled: enforce 4K-aligned CR3 base, low bits zero.
    andq $0xFFFFFFFFFFFFF000, %rcx
#endif

    // Keep marker stream deterministic.
    cli

    // Validation marker: IF bit policy after sanitization.
    testq $0x200, %rdx
    jz .L_rflags_if_off
    EMIT_CSTR p10_rflags_if_on
    jmp .L_rflags_if_ok
.L_rflags_if_off:
    EMIT_CSTR p10_rflags_if_off
    ud2
.L_rflags_if_ok:

    // Switch to user CR3 and emit marker.
    mov %rcx, %cr3
    EMIT_CSTR p10_cr3_switch

    // Build IRETQ frame: SS, RSP, RFLAGS, CS, RIP (reverse push order).
    movq $SEG_USER_DATA, %rax
    pushq %rax
    pushq %r10
    pushq %rdx
    movq $SEG_USER_CODE, %rax
    pushq %rax
    pushq %r11

    // Enter marker immediately before transition.
    EMIT_CSTR p10_ring3_enter
    iretq

    // Never returns.
    ud2

//
// ring3_enter(rip, rsp, user_cr3) noreturn
//
// Compatibility wrapper for legacy C callsites.
// Uses default policy RFLAGS value and tails into canonical helper.
//
ring3_enter:
    mov %rdx, %rcx            // rcx = user_cr3
    movq $RFLAGS_BASE, %rdx   // rdx = baseline user rflags
    jmp ring3_enter_iretq

.section .rodata
p10_cr3_switch:
    .asciz "P10_CR3_SWITCH\n"
p10_ring3_enter:
    .asciz "P10_RING3_ENTER\n"
p10_rflags_if_on:
    .asciz "P10_RFLAGS_IF_ON\n"
p10_rflags_if_off:
    .asciz "P10_RFLAGS_IF_OFF\n"
