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

    // Emit ping marker character-by-character via SYS_V2_DEBUG_PUTCHAR (1010)
    // Kernel-side atomic blocks prevent interleaving
    lea .Lmarker_ping(%rip), %rsi  // String pointer
    xor %rcx, %rcx                  // Character index

.Lprint_loop:
    movb (%rsi, %rcx, 1), %al      // Load character into AL
    test %al, %al                   // Check for null terminator
    jz .Ldone                       // Exit if null
    
    movzx %al, %rdi                 // Zero-extend to 64-bit for syscall arg
    mov $1010, %rax                 // SYS_V2_DEBUG_PUTCHAR
    int $0x80
    
    inc %rcx                        // Next character
    jmp .Lprint_loop

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
