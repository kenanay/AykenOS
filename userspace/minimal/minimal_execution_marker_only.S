// SPDX-License-Identifier: ASAL-1.0
// Phase-16 Task 5: Userspace Execution Marker Proof
//
// Minimal test: verify userspace can execute and emit markers
// - No submit_execution (avoids BCIB role requirement)
// - Just emit markers and exit cleanly
// - Purpose: validate manual QEMU lane userspace execution

.section .text
.global _start

// Syscall numbers
.set SYS_V2_EXIT, 1009
.set SYS_V2_DEBUG_PUTCHAR, 1010

_start:
    // Mailbox epoch update (Phase 10 compatibility)
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit start marker
    lea .Lmarker_start(%rip), %r12
    call emit_marker

    // Emit OK marker
    lea .Lmarker_ok(%rip), %r12
    call emit_marker

    // Exit cleanly
    mov $SYS_V2_EXIT, %rax
    xor %rdi, %rdi
    int $0x80

.Lloop:
    pause
    jmp .Lloop

// Helper: emit null-terminated string via SYS_V2_DEBUG_PUTCHAR (1010)
emit_marker:
    push %r12
.Lemit_loop:
    movzbq (%r12), %rdi
    test %dil, %dil
    je .Lemit_done
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
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
.Lmarker_start:
    .asciz "ST\n"
.Lmarker_ok:
    .asciz "OK\n"
