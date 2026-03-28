// SPDX-License-Identifier: ASAL-1.0
// Diagnostic-only Phase 10-A2 user payload without an initial trap.
//
// Purpose:
// - Prove whether direct Ring3 entry executes normal user instructions.
// - Avoid stack, mailbox, syscall, and exception-delivery dependencies.

.section .text
.global _start

_start:
    xor %eax, %eax
    inc %eax
    nop

.Lloop:
    pause
    jmp .Lloop
