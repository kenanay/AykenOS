.section .text
.global _start

.equ SYS_V2_DEBUG_PUTCHAR, 1010

/*
 * First-retire probe for the BCIB bootstrap lane.
 * Keep the sequence stackless and memory-free so the first signal is whether
 * Ring3 can actually retire instructions and cross one syscall boundary.
 */
_start:
    mov $'A', %rdi
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

    mov $'B', %rdi
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

    mov $'C', %rdi
    mov $SYS_V2_DEBUG_PUTCHAR, %rax
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

.L_spin:
    jmp .L_spin
