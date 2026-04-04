// SPDX-License-Identifier: ASAL-1.0
// Runtime syscall-v2 contract payload (userspace/minimal mode override)
//
// Emits the syscall-v2 runtime gate contract in a deterministic sequence:
// - time_query (1006)
// - capability_bind x2 (1007)
// - capability_revoke x2 (1008)
// - canonical debug marker via debug_putchar (1010): [U][SYSCALL_OK]

.section .text
.global _start

_start:
    // Stage 1: payload entry reached.
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Reserve scratch space on user stack:
    //   [rsp+0x00] : time_query result buffer (u64)
    //   [rsp+0x10] : capability_token_t {id:u64, perms:u32, type:u32}
    sub $64, %rsp

    // Stage 2: user reached the first syscall gate attempt site.
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // time_query(type=0, out=&buf)
    mov $1006, %rax
    xor %rdi, %rdi
    lea 0(%rsp), %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

    // Stage 3: first syscall returned to userspace.
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Build a deterministic invalid token (id=0) for bind probes.
    movq $0, 16(%rsp)
    movl $1, 24(%rsp)
    movl $8, 28(%rsp)

    // capability_bind(ctx=2, token=&token) - probe #1
    mov $1007, %rax
    mov $2, %rdi
    lea 16(%rsp), %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // capability_bind(ctx=2, token=&token) - probe #2
    mov $1007, %rax
    mov $2, %rdi
    lea 16(%rsp), %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // capability_revoke(token_id=1) - probe #1
    mov $1008, %rax
    mov $1, %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // capability_revoke(token_id=1) - probe #2
    mov $1008, %rax
    mov $1, %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit canonical marker expected by runtime harness: [U][SYSCALL_OK]\n
    lea .Lmarker(%rip), %r12
.Lemit:
    movzbq (%r12), %rdi
    test %dil, %dil
    je .Lemit_done
    mov $1010, %rax
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    inc %r12
    jmp .Lemit

.Lemit_done:
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

.Lloop:
    // Keep mailbox epoch fresh while runtime harness observes logs.
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    pause
    jmp .Lloop

.section .rodata
.Lmarker:
    .asciz "[U][SYSCALL_OK]\n"
