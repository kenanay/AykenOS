// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Canonical Ring3 IRETQ Entry Helpers
// Authority: Phase 10-A2 Task 2
// Constitutional: Ring0 mechanism only (no policy)

#include "../../include/ring3_contract.h"

.section .text
.global ring3_enter
.global ring3_enter_iretq
.global ring3_enter_post_cr3
.global ring3_enter_trampoline
.global ring3_enter_iret_trampoline
.global ring3_enter_alt_bridge_trampoline
.global ring3_enter_final_iret_trampoline

.extern sched_emit_ring3_frame_proof

.section .bss
ring3_transition_trace_emitted:
    .byte 0
ring3_transition_trace_emit_now:
    .byte 0
entry_diag_samples:
    .quad 0

.section .text

// Frozen selector / RFLAGS contract shared with C.
.set SEG_USER_DATA,   AYKEN_RING3_USER_DATA_SELECTOR
.set SEG_USER_CODE,   AYKEN_RING3_USER_CODE_SELECTOR
.set RFLAGS_BASE,     AYKEN_RING3_RFLAGS_BASE
.set RFLAGS_CLEARMASK, AYKEN_RING3_RFLAGS_CLEARMASK

#ifndef AYKEN_CR3_PCID
#define AYKEN_CR3_PCID 0
#endif

#ifndef AYKEN_RING3_IRETQ_DIAG_PROBE
#define AYKEN_RING3_IRETQ_DIAG_PROBE 0
#endif

#ifndef AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE
#define AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE 0
#endif

#ifndef AYKEN_RING3_POST_CR3_TEXT_PROBE
#define AYKEN_RING3_POST_CR3_TEXT_PROBE 0
#endif

#ifndef AYKEN_RING3_LOW_FETCH_STUB
#define AYKEN_RING3_LOW_FETCH_STUB 0
#endif

#ifndef AYKEN_RING3_CANONICAL_FETCH_STUB
#define AYKEN_RING3_CANONICAL_FETCH_STUB 0
#endif

#ifndef AYKEN_RING3_SKIP_CR3_PIVOT
#define AYKEN_RING3_SKIP_CR3_PIVOT 0
#endif

#ifndef AYKEN_RING3_SELF_RELOAD_CR3
#define AYKEN_RING3_SELF_RELOAD_CR3 0
#endif

#ifndef AYKEN_RING3_IRETQ_HEAD_PAD
#define AYKEN_RING3_IRETQ_HEAD_PAD 0
#endif

#ifndef AYKEN_RING3_IRETQ_MARKER_PREFIX
#define AYKEN_RING3_IRETQ_MARKER_PREFIX 0
#endif

#ifndef AYKEN_RING3_SPLIT_IRETQ_PAGE
#define AYKEN_RING3_SPLIT_IRETQ_PAGE 0
#endif

#ifndef AYKEN_DEBUG_SCHED
#define AYKEN_DEBUG_SCHED 0
#endif

#ifndef AYKEN_RING3_ALT_STAGEB_SOURCE
#define AYKEN_RING3_ALT_STAGEB_SOURCE 0
#endif

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
.macro FAIL_CLOSED
    ud2
1:
    hlt
    jmp 1b
.endm
#else
.macro FAIL_CLOSED
1:
    hlt
    jmp 1b
.endm
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

.macro EMIT_HEX64 reg
    mov \reg, %r9
    mov $16, %r8d
1:
    mov %r9, %rax
    shr $60, %rax
    and $0xF, %eax
    cmp $10, %al
    jb 2f
    add $('A' - 10), %al
    jmp 3f
2:
    add $'0', %al
3:
    outb %al, $0xE9
    shl $4, %r9
    decl %r8d
    jne 1b
.endm

.macro EMIT_POST_CR3_TEXT_PROBE
#if defined(AYKEN_RING3_POST_CR3_TEXT_PROBE) && (AYKEN_RING3_POST_CR3_TEXT_PROBE == 1)
    EMIT_CSTR p10_post_cr3_text_probe
    mov %cr3, %r10
    EMIT_HEX64 %r10
    EMIT_CSTR p10_post_cr3_text_probe_rip
    EMIT_HEX64 %r11
    EMIT_CSTR p10_post_cr3_text_probe_qword
    mov (%r11), %r10
    EMIT_HEX64 %r10
    EMIT_CSTR p10_newline
#endif
.endm

// Entry segmentation profiling macro (bounded sampling)
// CRITICAL: Uses only caller-saved registers to avoid clobbering user state
.macro EMIT_ENTRY_SEG_TSC label
#if defined(AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE) && (AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE == 1)
    // Check if we've already collected enough samples
    // Use memory-to-memory compare to avoid clobbering registers
    cmpq $3, entry_diag_samples(%rip)
    jge 1f
    // Save caller-saved registers we'll use
    pushq %rax
    pushq %rdx
    pushq %r9
    // Read TSC
    rdtsc
    shl $32, %rdx
    or %rdx, %rax
    mov %rax, %r9
    // Emit marker
    EMIT_CSTR \label
    EMIT_HEX64 %r9
    EMIT_CSTR p10_newline
    // Restore registers
    popq %r9
    popq %rdx
    popq %rax
1:
#endif
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
// Scheduler-side code owns all RIP/RSP/selector/mapping precondition checks.
// This helper narrows the post-CR3 window to the final privilege drop only.
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

    // Keep the verbose transition proof one-shot without touching restored
    // user callee-saved registers. MARK:SW/MARK:IRET and AYKEN_PERF markers
    // remain per-transition for the preempt contract.
    movb $0, ring3_transition_trace_emit_now(%rip)
    cmpb $0, ring3_transition_trace_emitted(%rip)
    jne .L_ring3_trace_state_loaded
    movb $1, ring3_transition_trace_emitted(%rip)
    movb $1, ring3_transition_trace_emit_now(%rip)
.L_ring3_trace_state_loaded:
    cli
    cmpb $0, ring3_transition_trace_emit_now(%rip)
    jz .L_skip_ring3_attempt_trace
    EMIT_CSTR p10_ring3_attempt
.L_skip_ring3_attempt_trace:

    // Validation marker: IF bit policy after sanitization.
    testq $0x200, %rdx
    jz .L_rflags_if_off
    cmpb $0, ring3_transition_trace_emit_now(%rip)
    jz .L_rflags_if_ok
    EMIT_CSTR p10_rflags_if_on
    EMIT_HEX64 %rdx
    EMIT_CSTR p10_newline
    jmp .L_rflags_if_ok
.L_rflags_if_off:
    EMIT_CSTR p10_rflags_if_off
    EMIT_HEX64 %rdx
    EMIT_CSTR p10_newline
    FAIL_CLOSED
.L_rflags_if_ok:

    // Build the final IRETQ frame before the CR3 pivot so the target root
    // never sees extra kernel stack writes. The target rsp0 still must remain
    // supervisor-readable for the final iretq pop sequence.
    movq $SEG_USER_DATA, %rax
    pushq %rax
    pushq %r10
    pushq %rdx
    movq $SEG_USER_CODE, %rax
    pushq %rax
    pushq %r11

#if AYKEN_DEBUG_SCHED == 1
    pushq %rcx
    leaq 8(%rsp), %rdi
    call sched_emit_ring3_frame_proof
    popq %rcx
#endif

    cmpb $0, ring3_transition_trace_emit_now(%rip)
    jz .L_skip_ring3_commit_trace
    EMIT_CSTR p10_ring3_commit
    // These are last reliable kernel-side markers; actual user-code proof
    // still comes from the Ring3 #BP path (`P10_RING3_USER_CODE`).
    EMIT_CSTR p10_cr3_switch
    
    // Dump PIC mask before P10_RING3_ENTER
    EMIT_CSTR p10_pic_mask
    inb $0x21, %al
    movzbq %al, %rdi
    EMIT_HEX64 %rdi
    inb $0xA1, %al
    movzbq %al, %rdi
    EMIT_HEX64 %rdi
    EMIT_CSTR p10_newline

    EMIT_CSTR p10_ring3_enter

    // Dump PIC mask after P10_RING3_ENTER
    EMIT_CSTR p10_pic_mask
    inb $0x21, %al
    movzbq %al, %rdi
    EMIT_HEX64 %rdi
    inb $0xA1, %al
    movzbq %al, %rdi
    EMIT_HEX64 %rdi
    EMIT_CSTR p10_newline
    movb $0, ring3_transition_trace_emit_now(%rip)
.L_skip_ring3_commit_trace:
#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    // Diagnostic-only fetch probe stays on this transition page; scheduler-side
    // C helpers must not extend the post-CR3 execution footprint.
    call ring3_enter_post_cr3_kernel_probe
#if defined(AYKEN_RING3_SECOND_CANONICAL_PROBE) && (AYKEN_RING3_SECOND_CANONICAL_PROBE == 1)
    movabsq $AYKEN_RING3_SECOND_CANONICAL_PROBE_VA, %rax
    invlpg (%rax)
#else
    movabsq $AYKEN_RING3_TRAMPOLINE_VA, %rax
#endif
    jmp *%rax
#else
#if defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1)
    EMIT_CSTR p10_canonical_fetch_stub
#if defined(AYKEN_RING3_SKIP_CR3_PIVOT) && (AYKEN_RING3_SKIP_CR3_PIVOT == 1)
    EMIT_CSTR p10_skip_cr3_pivot
#endif
#if defined(AYKEN_RING3_SELF_RELOAD_CR3) && (AYKEN_RING3_SELF_RELOAD_CR3 == 1)
    EMIT_CSTR p10_self_reload_cr3
#endif
    movabsq $AYKEN_RING3_CANONICAL_STAGE_A_VA, %rax
    jmp *%rax
#else
#if defined(AYKEN_RING3_LOW_FETCH_STUB) && (AYKEN_RING3_LOW_FETCH_STUB == 1)
    EMIT_CSTR p10_low_fetch_stub
    movabsq $AYKEN_RING3_TRAMPOLINE_VA, %rax
    jmp *%rax
#else
    jmp ring3_enter_post_cr3
#endif
#endif
#endif

#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
ring3_enter_post_cr3_kernel_probe:
    movb $'K', %al
    outb %al, $0xE9
    ret
#endif

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

#if defined(AYKEN_RING3_FETCH_PROBE) && (AYKEN_RING3_FETCH_PROBE == 1)
    .section .ring3_transition_probe, "ax", @progbits
    .p2align 12
ring3_enter_post_cr3:
    mov %rcx, %cr3
    mov %cr3, %rax
    cmp %rcx, %rax
    jne .L_probe_cr3_mismatch
    movb $'L', %al
    outb %al, $0xE9
    nop
    movb $'F', %al
    outb %al, $0xE9
1:
    jmp 1b
.L_probe_cr3_mismatch:
    movb $'M', %al
    outb %al, $0xE9
2:
    jmp 2b
    .section .text
#else
    .section .ring3_transition, "ax", @progbits
    .p2align 12
ring3_enter_post_cr3:
    // Canonical post-CR3 window: dedicated page, no adjacent kernel text.
    
    // PATCH H: Entry segmentation profiling - START
    EMIT_ENTRY_SEG_TSC diag_entry_seg_start
    
    // CR3 pivot
    mov %rcx, %cr3
    
    // PATCH H: After CR3 pivot
    EMIT_ENTRY_SEG_TSC diag_entry_seg_after_cr3
    
    EMIT_POST_CR3_TEXT_PROBE
    
    // PATCH H: After text proof (if enabled)
    EMIT_ENTRY_SEG_TSC diag_entry_seg_after_text_proof
    
    // PATCH H: Before IRET
    EMIT_ENTRY_SEG_TSC diag_entry_seg_before_iret
    
    // PATCH H: Increment sample counter (register-safe)
#if defined(AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE) && (AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE == 1)
    cmpq $3, entry_diag_samples(%rip)
    jge 1f
    incq entry_diag_samples(%rip)
1:
#endif
    
#if defined(AYKEN_RING3_IRETQ_DIAG_PROBE) && (AYKEN_RING3_IRETQ_DIAG_PROBE == 1)
    jmp 1f
1:
    nop
    movb $'J', %al
    outb %al, $0xE9
2:
    jmp 2b
#else
    iretq
#endif
    .section .text
#endif

    .section .ring3_trampoline, "ax", @progbits
    .p2align 12
ring3_enter_trampoline:
    // Dedicated low-half trampoline page for fetch-stub validation lanes.
#if defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1)
    /*
     * Canonical boundary-cross experiment:
     * enter this page before the CR3 pivot, jump to the last 3 bytes, and let
     * the first post-pivot fetch fall naturally onto the next page.
     */
    jmp .L_trampoline_pivot
    .org 0xFFD, 0x90
.L_trampoline_pivot:
#if defined(AYKEN_RING3_SKIP_CR3_PIVOT) && (AYKEN_RING3_SKIP_CR3_PIVOT == 1)
    nop
    nop
    nop
#elif defined(AYKEN_RING3_SELF_RELOAD_CR3) && (AYKEN_RING3_SELF_RELOAD_CR3 == 1)
    mov %cr3, %rcx
#else
    mov %rcx, %cr3
#endif
#else
    mov %rcx, %cr3
    EMIT_POST_CR3_TEXT_PROBE
#if defined(AYKEN_RING3_IRETQ_DIAG_PROBE) && (AYKEN_RING3_IRETQ_DIAG_PROBE == 1)
    jmp 1f
1:
    nop
    movb $'J', %al
    outb %al, $0xE9
2:
    jmp 2b
#else
    iretq
#endif
#endif
    .section .text

    .section .ring3_iret_trampoline, "ax", @progbits
    .p2align 12
ring3_enter_iret_trampoline:
#if defined(AYKEN_RING3_CANONICAL_FETCH_STUB) && (AYKEN_RING3_CANONICAL_FETCH_STUB == 1)
#if defined(AYKEN_RING3_SELF_RELOAD_CR3) && (AYKEN_RING3_SELF_RELOAD_CR3 == 1)
    mov %rcx, %cr3
#endif
#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
#if defined(AYKEN_RING3_ALT_STAGEB_SOURCE) && (AYKEN_RING3_ALT_STAGEB_SOURCE == 1)
    jmp .L_alt_stageb_unmapped_should_not_execute
.L_alt_stageb_unmapped_should_not_execute:
    ud2
#else
    /*
     * Split-page validation lane:
     * prove whether this page itself can execute under the alternate root
     * before jumping to a separate final-iretq page.
     */
    jmp 1f
1:
    nop
    movb $'J', %al
    outb %al, $0xE9
    movabsq $AYKEN_RING3_CANONICAL_STAGE_C_VA, %rax
    jmp *%rax
#endif
#else
    jmp .L_iret_resume
    .p2align 4
.L_iret_resume:
#endif
#endif
#if defined(AYKEN_RING3_IRETQ_DIAG_PROBE) && (AYKEN_RING3_IRETQ_DIAG_PROBE == 1)
    jmp 1f
1:
    nop
    movb $'J', %al
    outb %al, $0xE9
2:
    jmp 2b
#else
#if defined(AYKEN_RING3_IRETQ_HEAD_PAD) && (AYKEN_RING3_IRETQ_HEAD_PAD == 1)
    /*
     * Byte-bisection lane:
     * keep the first three executed bytes aligned with the working diag page,
     * then fall into the real iretq a few bytes later.
     */
    jmp 1f
1:
    nop
#endif
#if defined(AYKEN_RING3_IRETQ_MARKER_PREFIX) && (AYKEN_RING3_IRETQ_MARKER_PREFIX == 1)
    /*
     * Deeper byte-bisection lane:
     * reuse the working diag prefix through the debugcon write, then keep the
     * real iretq out of the first fetch/decode window.
     */
    jmp 2f
2:
    nop
    movb $'J', %al
    outb %al, $0xE9
    jmp .L_iret_far_resume
    .org 0x40, 0x90
.L_iret_far_resume:
#endif
    iretq
#endif
    .section .text

#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1) && \
    defined(AYKEN_RING3_ALT_STAGEB_SOURCE) && (AYKEN_RING3_ALT_STAGEB_SOURCE == 1)
    .section .ring3_alt_bridge_trampoline, "ax", @progbits
    .p2align 12
ring3_enter_alt_bridge_trampoline:
    /*
     * Alternate source-frame lane:
     * keep stage-B bytes identical while sourcing them from a different
     * executable page/frame.
     */
    jmp 1f
1:
    nop
    movb $'J', %al
    outb %al, $0xE9
    movabsq $AYKEN_RING3_CANONICAL_STAGE_C_VA, %rax
    jmp *%rax
    .section .text
#endif

#if defined(AYKEN_RING3_SPLIT_IRETQ_PAGE) && (AYKEN_RING3_SPLIT_IRETQ_PAGE == 1)
    .section .ring3_final_iret_trampoline, "ax", @progbits
    .p2align 12
ring3_enter_final_iret_trampoline:
    iretq
    .section .text
#endif

	.section .rodata
p10_cr3_switch:
    .asciz "P10_CR3_SWITCH\n"
p10_pic_mask:
    .asciz "PIC_MASK M/S: "
p10_ring3_attempt:
    .asciz "P10_RING3_ATTEMPT\n"
p10_ring3_commit:
    .asciz "P10_RING3_COMMIT\n"
p10_ring3_enter:
    .asciz "P10_RING3_ENTER\n"
p10_post_cr3_text_probe:
    .asciz "P10_POST_CR3_TEXT_PROBE CR3="
p10_post_cr3_text_probe_rip:
    .asciz " RIP="
p10_post_cr3_text_probe_qword:
    .asciz " Q="
p10_newline:
    .asciz "\n"
p10_canonical_fetch_stub:
    .asciz "P10_CANONICAL_FETCH_STUB\n"
p10_skip_cr3_pivot:
    .asciz "P10_SKIP_CR3_PIVOT\n"
p10_self_reload_cr3:
    .asciz "P10_SELF_RELOAD_CR3\n"
p10_low_fetch_stub:
    .asciz "P10_LOW_FETCH_STUB\n"
p10_rflags_if_on:
    .asciz "P10_RFLAGS_IF_ON\n"
p10_rflags_if_off:
    .asciz "P10_RFLAGS_IF_OFF\n"
diag_entry_seg_start:
    .asciz "DIAG_ENTRY_SEG_START="
diag_entry_seg_after_cr3:
    .asciz "DIAG_ENTRY_SEG_AFTER_CR3="
diag_entry_seg_after_text_proof:
    .asciz "DIAG_ENTRY_SEG_AFTER_TEXT_PROOF="
diag_entry_seg_before_iret:
    .asciz "DIAG_ENTRY_SEG_BEFORE_IRET="
