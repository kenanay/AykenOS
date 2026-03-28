// SPDX-License-Identifier: ASAL-1.0
// Diagnostic-only Phase 10-A2 user payload.
//
// Purpose:
// - Prove whether canonical Ring3 entry plus the first mailbox access works.
// - Avoid the int 0x80 path so failures isolate user mapping/mailbox continuity.

.section .text
.global _start

_start:
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    int3

.Lloop:
    pause
    jmp .Lloop
