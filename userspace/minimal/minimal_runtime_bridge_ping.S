// SPDX-License-Identifier: ASAL-1.0
// Phase-16 Task 5: Ultra-Minimal Runtime_Bridge Ping Test
//
// Purpose: Prove userspace execution with single marker emission
// Expected: [U][RUNTIME_BRIDGE_PING] marker in QEMU trace

.section .text
.global _start

_start:
    // Mailbox epoch update (Phase 10 compatibility)
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit single ping marker via SYS_V2_DEBUG_WRITE_STR (1015)
    // This is atomic - entire string written in one syscall
    lea .Lmarker_ping(%rip), %rdi  // arg1: string pointer
    mov $25, %rsi                   // arg2: length (25 bytes)
    mov $1015, %rax                 // SYS_V2_DEBUG_WRITE_STR
    int $0x80

.Ldone:
    // Keep mailbox epoch fresh
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    pause
    jmp .Ldone

.section .rodata
.Lmarker_ping:
    .asciz "[U][RUNTIME_BRIDGE_PING]\n"
