// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Validation-only Ring3 BCIB worker completion witness.
//
// The fixed fixture encodes one bounded literal-result operation. The Ring3
// worker submits it to its own execution context, waits for kernel delivery,
// reads the delivered bytes, publishes a v1 output window, and terminates the
// execution through the frozen public complete_execution(1011) endpoint.

.set SYS_V2_SUBMIT_EXECUTION, 1003
.set SYS_V2_WAIT_RESULT, 1004
.set SYS_V2_DEBUG_PUTCHAR, 1010
.set SYS_V2_COMPLETE_EXECUTION, 1011
.set SELF_CONTEXT_ID, 2
.set ESYS_V2_RESOURCE_BUSY, -8
.set EXEC_COMPLETION_COMPLETED, 0

.set EXECUTION_INBOX_VA, 0x701000
.set EXECUTION_PAYLOAD_VA, 0x702000
.set EXECUTION_OUTPUT_VA, 0x900000
.set AYKEN_EXECUTION_INBOX_MAGIC, 0x42495841
.set AYKEN_EXECUTION_INBOX_VERSION, 1
.set AXIB_STATE_READY, 1
.set AYKEN_EXECUTION_OUTPUT_MAGIC, 0x54554f41
.set AYKEN_EXECUTION_OUTPUT_VERSION, 1
.set FIXTURE_SIZE, 16
.set EXPECTED_RESULT_BYTES, 8
.set EXPECTED_LITERAL, 0x1122334455667788

.section .text
.global _start

_start:
    mov $SYS_V2_SUBMIT_EXECUTION, %rax
    lea .Lbcib_fixture(%rip), %rdi
    mov $FIXTURE_SIZE, %rsi
    mov $SELF_CONTEXT_ID, %rdx
    xor %r10, %r10
    int $0x80

    test %rax, %rax
    jle .Lfail
    mov %rax, %r12

.Lwait_delivery:
    call .Lpublish_mailbox_tick

    mov $EXECUTION_INBOX_VA, %rbx
    cmpl $AYKEN_EXECUTION_INBOX_MAGIC, 0(%rbx)
    jne .Lfail
    cmpw $AYKEN_EXECUTION_INBOX_VERSION, 4(%rbx)
    jne .Lfail
    cmpw $AXIB_STATE_READY, 6(%rbx)
    jne .Lwait_delivery
    cmpq %r12, 16(%rbx)
    jne .Lfail
    cmpq $SELF_CONTEXT_ID, 24(%rbx)
    jne .Lfail
    cmpq $EXECUTION_PAYLOAD_VA, 32(%rbx)
    jne .Lfail
    cmpq $FIXTURE_SIZE, 40(%rbx)
    jne .Lfail

    // Execute the bounded literal-result fixture from delivered BCIB bytes.
    mov $EXECUTION_PAYLOAD_VA, %r15
    cmpl $0x42494342, 0(%r15) // "BCIB"
    jne .Lfail
    cmpb $0x17, 4(%r15)       // Phase-17 fixture family
    jne .Lfail
    cmpb $0x11, 5(%r15)       // literal-result operation
    jne .Lfail
    cmpw $0x0001, 6(%r15)     // fixture version
    jne .Lfail
    mov 8(%r15), %r14

    mov $EXECUTION_OUTPUT_VA, %r13
    movl $AYKEN_EXECUTION_OUTPUT_MAGIC, 0(%r13)
    movl $AYKEN_EXECUTION_OUTPUT_VERSION, 4(%r13)
    movl $0, 8(%r13)
    movl $0, 12(%r13)
    movq $EXPECTED_RESULT_BYTES, 16(%r13)
    movq $0, 24(%r13)
    movq $0, 32(%r13)
    movq $0, 40(%r13)
    mov %r14, 48(%r13)

    mov $SYS_V2_COMPLETE_EXECUTION, %rax
    mov %r12, %rdi
    mov $EXEC_COMPLETION_COMPLETED, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    test %rax, %rax
    jne .Lfail

.Lwait_result:
    call .Lpublish_mailbox_tick
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
    jmp .Lwait_result

.Lcheck_result:
    mov %rax, %r13
    cmpl $AYKEN_EXECUTION_OUTPUT_MAGIC, 0(%r13)
    jne .Lfail
    cmpl $AYKEN_EXECUTION_OUTPUT_VERSION, 4(%r13)
    jne .Lfail
    cmpq $EXPECTED_RESULT_BYTES, 16(%r13)
    jne .Lfail
    movabs $EXPECTED_LITERAL, %rax
    cmpq %rax, 48(%r13)
    jne .Lfail

    /*
     * Keep the final witness atomic under timer-driven debug output. Public
     * submit/complete/wait markers already establish the syscall sequence;
     * this character states that Ring3 also checked the mapped literal.
     */
    mov $'W', %rdi
    call .Lemit_putchar
    jmp .Lhalt

.Lfail:
    mov $'F', %rdi
    call .Lemit_putchar

.Lhalt:
    pause
    jmp .Lhalt

.Lpublish_mailbox_tick:
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    ret

.Lemit_putchar:
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    ret

.section .rodata
.Lbcib_fixture:
    .byte 'B', 'C', 'I', 'B', 0x17, 0x11, 0x01, 0x00
    .quad EXPECTED_LITERAL
