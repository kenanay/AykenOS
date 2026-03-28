// SPDX-License-Identifier: ASAL-1.0
// Diagnostic-only Phase 10-A2 user payload with a text-page retire witness.
//
// Purpose:
// - Prove whether early user stores retire on the already-grafted text leaf.
// - Avoid stack, mailbox, syscall, and exception-delivery dependencies.

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
    nop

.Lloop:
    pause
    jmp .Lloop
