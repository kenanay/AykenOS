// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Minimal Ring3 Test Stub (Phase 10-A2 Task 2)
// Authority: Phase 10 specification
// Constitutional: Ring3 test code only

.section .text
.global _start

_start:
    // Phase 10-A2 Task 3: Ring3 -> int 0x80 -> Ring0 -> Ring3 roundtrip.
    // Use SYS_V2_CAPABILITY_REVOKE (1008). Token 1 should be denied in fresh boot.
    mov $1008, %rax
    mov $1, %rdi
    xor %rsi, %rsi
    xor %rdx, %rdx
    xor %r10, %r10
    int $0x80

    // Task 10-A2 proof path: trigger #BP from Ring3 after syscall return.
    int3

    // If #BP handler returns unexpectedly, stay in Ring3 with non-privileged ops.
.Lloop:
    pause
    jmp .Lloop

// Unreachable
.Lpanic:
    ud2                   // Invalid opcode (should never reach)
