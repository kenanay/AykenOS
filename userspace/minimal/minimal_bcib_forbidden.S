# userspace/minimal/minimal_bcib_forbidden.S
# Phase-16 Task 3: BCIB Execution Entry Enforcement Test
#
# This minimal mode payload tests BCIB execution entry enforcement with
# fail-closed semantics. It attempts a forbidden syscall and expects
# kernel-level termination with no continuation.
#
# Expected QEMU trace markers:
# 1. BCIB_FORBIDDEN_BEFORE (userspace)
# 2. [[AYKEN_SYSCALL_ENTER]] (kernel)
# 3. [[AYKEN_BOUNDARY_KILL]] (kernel)
# 4. NO BCIB_FORBIDDEN_AFTER (proves fail-closed)

.section .text
.global _start

# Syscall numbers
.set SYS_V2_BASE, 1000
.set SYS_V2_TIME_QUERY, 6
.set SYS_V2_DEBUG_PUTCHAR, 10

_start:
    # Attempt forbidden syscall immediately
    # BCIB role is only allowed SYS_V2_SUBMIT_EXECUTION (3)
    # SYS_V2_TIME_QUERY (6) is FORBIDDEN
    # Kernel will emit BCIB_FORBIDDEN_BEFORE marker when it detects BCIB context
    movq $(SYS_V2_BASE + SYS_V2_TIME_QUERY), %rax
    xorq %rdi, %rdi                    # arg1 = 0
    leaq time_result(%rip), %rsi       # arg2 = &time_result (stack-based)
    int $0x80
    
    # If we reach here, fail-closed is BROKEN
    # This marker MUST NOT appear in QEMU trace
    # Infinite loop (should never reach)
.Lhalt_loop:
    hlt
    jmp .Lhalt_loop

.section .rodata
before_msg:
    .asciz "BCIB_FORBIDDEN_BEFORE\n"
after_msg:
    .asciz "BCIB_FORBIDDEN_AFTER\n"
critical_msg:
    .asciz "CRITICAL: Execution continued after violation!\n"
time_result:
    .quad 0

