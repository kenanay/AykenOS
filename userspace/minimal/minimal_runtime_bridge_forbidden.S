// SPDX-License-Identifier: ASAL-1.0
// Phase-16 Task 5: Runtime_Bridge Forbidden Path Proof
//
// This userspace program tests Runtime_Bridge fail-closed enforcement:
// - Runtime_Bridge role attempts forbidden syscall SYS_V2_SUBMIT_EXECUTION (1003)
// - Expected: Boundary kill with no continuation
// - Forbidden: Any execution after syscall attempt
//
// Markers emitted via SYS_V2_DEBUG_PUTCHAR (1010) for QEMU trace validation.

.section .text
.global _start

_start:
    // Mailbox epoch update (Phase 10 compatibility)
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit forbidden path marker: [U][RTB_FB]
    // Minimal marker for window hardening (Task 10B)
    lea .Lmarker_forbidden_before(%rip), %r12
    call emit_marker

    // Attempt forbidden syscall: SYS_V2_SUBMIT_EXECUTION (1003)
    // This MUST be denied for Runtime_Bridge role
    mov $1003, %rax
    xor %rdi, %rdi         // bcib_graph (null - invalid)
    xor %rsi, %rsi         // graph_size (0 - invalid)
    xor %rdx, %rdx         // context_id (0 - invalid)
    int $0x80
    
    // ⚠️ FORBIDDEN CONTINUATION ZONE ⚠️
    // If execution reaches here, fail-closed enforcement is BROKEN
    // This marker MUST NOT appear in valid trace
    lea .Lmarker_forbidden_after(%rip), %r12
    call emit_marker

.Lloop:
    // Keep mailbox epoch fresh (should never reach here)
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    pause
    jmp .Lloop

// Helper: emit null-terminated string via SYS_V2_DEBUG_PUTCHAR (1010)
// Input: %r12 = pointer to string
emit_marker:
    push %r12
.Lemit_loop:
    movzbq (%r12), %rdi
    test %dil, %dil
    je .Lemit_done
    mov $1010, %rax        // SYS_V2_DEBUG_PUTCHAR
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    inc %r12
    jmp .Lemit_loop
.Lemit_done:
    pop %r12
    ret

.section .rodata
.Lmarker_forbidden_before:
    .asciz "FB\n"
.Lmarker_forbidden_after:
    .asciz "FA\n"
