// SPDX-License-Identifier: ASAL-1.0
// Diagnostic-only Phase 10-A2 user payload with a non-trapping retire witness.
//
// Purpose:
// - Prove whether the first user-side store retires after iretq.
// - Avoid mailbox, syscall, and exception-delivery dependencies.

.section .text
.global _start

.set RING3_CANARY_ADDR, 0x0000000000405000
.set RING3_CANARY_POST_LOW32, 0x44444444
.set RING3_CANARY_POST_HIGH32, 0x33333333

_start:
    mov $RING3_CANARY_ADDR, %ebx
    mov $RING3_CANARY_POST_LOW32, %eax
    mov $RING3_CANARY_POST_HIGH32, %edx
    shl $32, %rdx
    or %rdx, %rax
    mov %rax, (%rbx)
    nop

.Lloop:
    pause
    jmp .Lloop
