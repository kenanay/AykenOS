// SPDX-License-Identifier: ASAL-1.0
// Phase-16 Task 5: Execution Delivery Proof
//
// This userspace program tests the execution submission → delivery → completion pipeline:
// - Submit execution with minimal BCIB graph
// - Poll inbox until AXIB_STATE_READY
// - Complete execution with EXEC_COMPLETION_COMPLETED
// - Expected: Delivery markers visible, no fail-closed termination
//
// Markers emitted via SYS_V2_DEBUG_PUTCHAR (10) for QEMU trace validation.

.section .text
.global _start

// Syscall numbers
.set SYS_V2_DEBUG_PUTCHAR, 10
.set SYS_V2_COMPLETE_EXECUTION, 11
.set SYS_V2_SUBMIT_EXECUTION, 3

// Inbox constants
.set EXECUTION_INBOX_VA, 0x701000
.set EXECUTION_PAYLOAD_VA, 0x702000
.set AXIB_STATE_EMPTY, 0
.set AXIB_STATE_READY, 1
.set EXEC_COMPLETION_COMPLETED, 0

// Inbox structure offsets
.set INBOX_MAGIC, 0
.set INBOX_VERSION, 4
.set INBOX_STATE, 6
.set INBOX_DELIVERY_SEQ, 8
.set INBOX_EXECUTION_ID, 16
.set INBOX_TARGET_CONTEXT_ID, 24
.set INBOX_BCIB_USER_VA, 32
.set INBOX_BCIB_SIZE, 40

_start:
    // Mailbox epoch update (Phase 10 compatibility)
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit start marker
    lea .Lmarker_start(%rip), %r12
    call emit_marker

    // Prepare minimal BCIB graph (just a few bytes of dummy data)
    lea .Lbcib_graph(%rip), %r13
    mov $16, %r14                  // graph_size = 16 bytes
    mov $2, %r15                   // context_id = 2 (target worker PID)

    // Submit execution: SYS_V2_SUBMIT_EXECUTION (3)
    mov $SYS_V2_SUBMIT_EXECUTION, %rax
    mov %r13, %rdi                 // bcib_graph
    mov %r14, %rsi                 // graph_size
    mov %r15, %rdx                 // context_id
    int $0x80
    
    // Save execution_id
    mov %rax, %r15                 // r15 = execution_id

    // Emit submitted marker
    lea .Lmarker_submitted(%rip), %r12
    call emit_marker

    // Poll inbox until AXIB_STATE_READY
    mov $EXECUTION_INBOX_VA, %r13
    mov $1000, %rcx                // max poll iterations

.Lpoll_loop:
    // Read inbox state
    movzwq INBOX_STATE(%r13), %rax
    cmp $AXIB_STATE_READY, %rax
    je .Linbox_ready
    
    // Keep mailbox epoch fresh
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    
    pause
    dec %rcx
    jnz .Lpoll_loop
    
    // Timeout - emit timeout marker and exit
    lea .Lmarker_timeout(%rip), %r12
    call emit_marker
    jmp .Lexit

.Linbox_ready:
    // Inbox is ready - emit delivered marker
    lea .Lmarker_delivered(%rip), %r12
    call emit_marker

    // Complete execution: SYS_V2_COMPLETE_EXECUTION (11)
    mov $SYS_V2_COMPLETE_EXECUTION, %rax
    mov %r15, %rdi                 // execution_id
    mov $EXEC_COMPLETION_COMPLETED, %rsi  // completion_code
    int $0x80

    // Emit completed marker
    lea .Lmarker_completed(%rip), %r12
    call emit_marker

.Lexit:
    // Exit cleanly
    mov $9, %rax                   // SYS_V2_EXIT
    xor %rdi, %rdi                 // exit_code = 0
    int $0x80

.Lloop:
    pause
    jmp .Lloop

// Helper: emit null-terminated string via SYS_V2_DEBUG_PUTCHAR (10)
// Input: %r12 = pointer to string
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
.Lmarker_submitted:
    .asciz "SB\n"
.Lmarker_delivered:
    .asciz "DL\n"
.Lmarker_completed:
    .asciz "CP\n"
.Lmarker_timeout:
    .asciz "TO\n"

// Minimal BCIB graph (read-only data)
.Lbcib_graph:
    .byte 0x01, 0x02, 0x03, 0x04
    .byte 0x05, 0x06, 0x07, 0x08
    .byte 0x09, 0x0A, 0x0B, 0x0C
    .byte 0x0D, 0x0E, 0x0F, 0x10
