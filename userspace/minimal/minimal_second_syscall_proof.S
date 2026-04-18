/*
 * AykenOS Second Syscall Proof - Minimal Userspace Payload
 * 
 * Purpose: Generate three consecutive syscalls to verify boundary_init_done
 * flag behavior for scheduler-primary-regression-rca investigation.
 * 
 * Expected kernel behavior:
 *   1st syscall: DIAG_BOUNDARY_INIT_ENTER (init path)
 *   2nd syscall: DIAG_BOUNDARY_INIT_SKIPPED (skip path)
 *   3rd syscall: DIAG_BOUNDARY_INIT_SKIPPED (skip path)
 * 
 * This payload uses SYS_V2_DEBUG_PUTCHAR (1010) which is minimal overhead
 * and safe for diagnostic purposes.
 */

.section .text
.global _start

_start:
    /* START marker - anchor point for log analysis */
    movq    $1010, %rax         # SYS_V2_DEBUG_PUTCHAR
    movq    $0x53, %rdi         # 'S' - START marker
    xorq    %rsi, %rsi
    xorq    %rdx, %rdx
    xorq    %r10, %r10
    int     $0x80
    
    /* 1st syscall - should trigger init path */
    movq    $1010, %rax         # SYS_V2_DEBUG_PUTCHAR
    movq    $0x41, %rdi         # 'A'
    xorq    %rsi, %rsi
    xorq    %rdx, %rdx
    xorq    %r10, %r10
    int     $0x80
    
    /* 2nd syscall - should trigger skip path */
    movq    $1010, %rax         # SYS_V2_DEBUG_PUTCHAR
    movq    $0x42, %rdi         # 'B'
    xorq    %rsi, %rsi
    xorq    %rdx, %rdx
    xorq    %r10, %r10
    int     $0x80
    
    /* 3rd syscall - should trigger skip path */
    movq    $1010, %rax         # SYS_V2_DEBUG_PUTCHAR
    movq    $0x43, %rdi         # 'C'
    xorq    %rsi, %rsi
    xorq    %rdx, %rdx
    xorq    %r10, %r10
    int     $0x80
    
    /* Halt forever - test complete */
.Lhalt_loop:
    pause
    jmp     .Lhalt_loop

.section .note.GNU-stack,"",@progbits
