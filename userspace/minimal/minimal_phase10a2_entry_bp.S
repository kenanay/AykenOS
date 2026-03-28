// SPDX-License-Identifier: ASAL-1.0
// Diagnostic-only Phase 10-A2 user payload.
//
// Purpose:
// - Prove whether canonical Ring3 entry reaches the first user instruction.
// - Avoid all stack, mailbox, and syscall dependencies.

.section .text
.global _start

_start:
    int3

.Lloop:
    pause
    jmp .Lloop

