// SPDX-License-Identifier: ASAL-1.0
// Diagnostic-only Phase 10-A2 user payload with a first-retire witness.
//
// Purpose:
// - Retire one user-side store on the text leaf.
// - Return immediately via #BP so kernel can read the witness without timer IRQs.

.section .text
.global _start

.set TEXT_WITNESS_ADDR, 0x0000000000400100
.set TEXT_WITNESS_LOW32, 0x57544E53
.set TEXT_WITNESS_HIGH32, 0x54585431

_start:
    mov $TEXT_WITNESS_ADDR, %ebx
    mov $TEXT_WITNESS_LOW32, %eax
    mov $TEXT_WITNESS_HIGH32, %edx
    shl $32, %rdx
    or %rdx, %rax
    mov %rax, (%rbx)
    int3

.Lloop:
    pause
    jmp .Lloop
