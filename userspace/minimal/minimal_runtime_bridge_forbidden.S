# userspace/minimal/minimal_runtime_bridge_forbidden.S
# Phase-16 Task 5: Runtime_Bridge Forbidden Execution Entry Test
#
# This minimal mode payload tests Runtime_Bridge execution entry enforcement with
# fail-closed semantics. It attempts a forbidden syscall and expects
# kernel-level termination with no continuation.

.section .text
.global _start

_start:
    # 1. Print BEFORE marker using allowed SYS_V2_DEBUG_PUTCHAR (1010)
    lea .Lmarker_forbidden_before(%rip), %r12
    call emit_marker

    # 2. Attempt forbidden syscall: SYS_V2_SUBMIT_EXECUTION (1003)
    # This is NOT allowed for RUNTIME_BRIDGE role.
    mov $1003, %rax
    xor %rdi, %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    int $0x80

    # 3. If we reach here, boundary kill failed
    # Print failure marker
    lea .Lmarker_forbidden_after(%rip), %r12
    call emit_marker

.Lhalt_loop:
    hlt
    jmp .Lhalt_loop

# Helper: emit null-terminated string via SYS_V2_DEBUG_PUTCHAR (1010)
# Input: %r12 = pointer to string
emit_marker:
    push %r12
.Lemit_loop:
    movzbq (%r12), %rdi
    test %dil, %dil
    je .Lemit_done
    mov $1010, %rax        # SYS_V2_DEBUG_PUTCHAR
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
.Lmarker_forbidden_before:
    .asciz "[U][RUNTIME_BRIDGE_FORBIDDEN_BEFORE]\n"
.Lmarker_forbidden_after:
    .asciz "[U][RUNTIME_BRIDGE_FORBIDDEN_AFTER]\n"
