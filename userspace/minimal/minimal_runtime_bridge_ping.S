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

    // Emit single ping marker via SYS_V2_DEBUG_PUTCHAR (1010)
    lea .Lmarker_ping(%rip), %rsi
.Lemit_loop:
    movzbl (%rsi), %edi
    test %edi, %edi
    je .Ldone
    
    // SYS_V2_DEBUG_PUTCHAR = 1010
    mov $1010, %rax
    int $0x80
    
    inc %rsi
    jmp .Lemit_loop

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
