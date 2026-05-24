// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Validation-only timeout-versus-completion race witness.
//
// The Ring3 worker submits a bounded fixture to itself and waits until
// delivery has placed the slot in RUNNING. The validation harness assigns a
// fixed logical deadline at delivery; while this worker continues polling,
// the timer IRQ must win terminalization. A delayed public
// complete_execution(1011) must then fail closed with INVALID_STATE.

.set SYS_V2_SUBMIT_EXECUTION, 1003
.set SYS_V2_WAIT_RESULT, 1004
.set SYS_V2_DEBUG_PUTCHAR, 1010
.set SYS_V2_COMPLETE_EXECUTION, 1011
.set SELF_CONTEXT_ID, 2
.set ESYS_V2_TIMEOUT, -6
.set ESYS_V2_INVALID_STATE, -10
.set EXEC_COMPLETION_COMPLETED, 0
.set ESYS_V2_RESOURCE_BUSY, -8

.set EXECUTION_INBOX_VA, 0x701000
.set EXECUTION_PAYLOAD_VA, 0x702000
.set AYKEN_EXECUTION_INBOX_MAGIC, 0x42495841
.set AYKEN_EXECUTION_INBOX_VERSION, 1
.set AXIB_STATE_READY, 1
.set FIXTURE_SIZE, 8

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

.Lobserve_timeout:
    // Nonblocking polling leaves Ring3 running while a real timer IRQ races
    // the validation-only logical deadline assigned to the RUNNING slot.
    call .Lpublish_mailbox_tick
    mov $SYS_V2_WAIT_RESULT, %rax
    mov %r12, %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    cmp $ESYS_V2_TIMEOUT, %rax
    je .Llate_completion
    cmp $ESYS_V2_RESOURCE_BUSY, %rax
    jne .Lfail
    pause
    jmp .Lobserve_timeout

.Llate_completion:
    // A late worker completion must not mutate the IRQ-won TIMEOUT terminal.
    mov $SYS_V2_COMPLETE_EXECUTION, %rax
    mov %r12, %rdi
    mov $EXEC_COMPLETION_COMPLETED, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    cmp $ESYS_V2_INVALID_STATE, %rax
    jne .Lfail

    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    mov $'R', %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    jmp .Lhalt

.Lfail:
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    mov $'F', %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

.Lhalt:
    pause
    jmp .Lhalt

.Lpublish_mailbox_tick:
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    ret

.section .rodata
.Lbcib_fixture:
    .byte 'B', 'C', 'I', 'B', 0x17, 0x31, 0x00, 0x01
