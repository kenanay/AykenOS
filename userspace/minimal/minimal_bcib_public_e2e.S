// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Validation-only public BCIB boundary witness.
//
// This Ring3 payload calls the frozen public ABI:
//   1003: submit_execution(payload, size, self context)
//   1004: wait_result(execution_id, nonblocking)
//
// The matching validation build names this process "bcib_worker" and enables
// deterministic stub completion. This proves public result publication, not
// real BCIB interpretation or production completion behavior.

.set SYS_V2_SUBMIT_EXECUTION, 1003
.set SYS_V2_WAIT_RESULT, 1004
.set SYS_V2_DEBUG_PUTCHAR, 1010
.set SELF_CONTEXT_ID, 2
.set ESYS_V2_RESOURCE_BUSY, -8
.set AYKEN_EXECUTION_OUTPUT_MAGIC, 0x54554f41
.set AYKEN_EXECUTION_OUTPUT_VERSION, 1
.set EXPECTED_PAYLOAD_BYTES, 8
.set EXPECTED_STUB_RESULT, 0xDEADBEEFCAFEBABE

.section .text
.global _start

_start:
    // Submit fixed bytes already resident in the Ring3 text/rodata image.
    mov $SYS_V2_SUBMIT_EXECUTION, %rax
    lea .Lbcib_input(%rip), %rdi
    mov $8, %rsi
    mov $SELF_CONTEXT_ID, %rdx
    xor %r10, %r10
    int $0x80

    test %rax, %rax
    jz .Lfail
    test %rax, %rax
    js .Lfail
    mov %rax, %r12

.Lwait:
    // Keep the strict scheduler mailbox publication monotonically fresh.
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    mov $SYS_V2_WAIT_RESULT, %rax
    mov %r12, %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

    test %rax, %rax
    jns .Lcheck_result
    cmp $ESYS_V2_RESOURCE_BUSY, %rax
    jne .Lfail
    pause
    jmp .Lwait

.Lcheck_result:
    mov %rax, %r13
    cmpl $AYKEN_EXECUTION_OUTPUT_MAGIC, 0(%r13)
    jne .Lfail
    cmpl $AYKEN_EXECUTION_OUTPUT_VERSION, 4(%r13)
    jne .Lfail
    cmpq $EXPECTED_PAYLOAD_BYTES, 16(%r13)
    jne .Lfail
    movabs $EXPECTED_STUB_RESULT, %rax
    cmpq %rax, 48(%r13)
    jne .Lfail

    lea .Lok_marker(%rip), %r14
    call .Lemit_marker
    jmp .Lhalt

.Lfail:
    lea .Lfail_marker(%rip), %r14
    call .Lemit_marker

.Lhalt:
    pause
    jmp .Lhalt

.Lemit_marker:
    movzbq (%r14), %rdi
    test %dil, %dil
    je .Lemit_done
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    inc %r14
    jmp .Lemit_marker
.Lemit_done:
    ret

.section .rodata
.Lbcib_input:
    .byte 'B', 'C', 'I', 'B', 0x17, 0x03, 0x00, 0x01
.Lok_marker:
    // Existing debug_putchar contract synthesizes an atomic kernel marker
    // after this Ring3-only post-result witness has been fully observed.
    .asciz "[U][SYSCALL_OK]\n"
.Lfail_marker:
    .asciz "[[AYKEN_PUBLIC_EXEC_E2E_USER_FAIL]]\n"
