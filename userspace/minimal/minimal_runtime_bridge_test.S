// SPDX-License-Identifier: ASAL-1.0
// Phase-16 Task 5: Runtime_Bridge Syscall Proof Test
//
// This userspace program exercises Runtime_Bridge allowed syscalls:
// - SYS_V2_DEVICE_OPERATION (1012)
// - SYS_V2_EXTERNAL_CALL (1013)
// - SYS_V2_ABDF_OPERATION (1014)
//
// Expected behavior: All three syscalls should succeed and return to userspace.
// Markers emitted via SYS_V2_DEBUG_PUTCHAR (1010) for QEMU trace validation.

.section .text
.global _start

_start:
    // Mailbox epoch update (Phase 10 compatibility)
    mov $0x700000, %rbx
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit start marker: [U][RUNTIME_BRIDGE_TEST_START]
    lea .Lmarker_start(%rip), %r12
    call emit_marker

    // Test 1: SYS_V2_DEVICE_OPERATION (1012)
    // Emit BEFORE marker
    lea .Lmarker_device_before(%rip), %r12
    call emit_marker
    
    // syscall(1012, device_id=42, operation=2, buffer=0, size=4)
    mov $1012, %rax
    mov $42, %rdi          // device_id (unique marker)
    mov $2, %rsi           // operation: DEVICE_OP_WRITE
    xor %rdx, %rdx         // buffer (null for stub test)
    mov $4, %r10           // size
    int $0x80
    
    // Emit AFTER marker
    lea .Lmarker_device_after(%rip), %r12
    call emit_marker
    
    // Mailbox epoch update
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Test 2: SYS_V2_EXTERNAL_CALL (1013)
    // Emit BEFORE marker
    lea .Lmarker_external_before(%rip), %r12
    call emit_marker
    
    // syscall(1013, call_type=1, args=0, arg_count=2, flags=0)
    mov $1013, %rax
    mov $1, %rdi           // call_type: EXTERNAL_CALL_NETWORK
    xor %rsi, %rsi         // args (null for stub test)
    mov $2, %rdx           // arg_count
    xor %r10, %r10         // flags
    int $0x80
    
    // Emit AFTER marker
    lea .Lmarker_external_after(%rip), %r12
    call emit_marker
    
    // Mailbox epoch update
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Test 3: SYS_V2_ABDF_OPERATION (1014)
    // Emit BEFORE marker
    lea .Lmarker_abdf_before(%rip), %r12
    call emit_marker
    
    // syscall(1014, operation=1, handle_id=123, buffer=0, size=4)
    mov $1014, %rax
    mov $1, %rdi           // operation: ABDF_OP_READ
    mov $123, %rsi         // handle_id (unique marker)
    xor %rdx, %rdx         // buffer (null for stub test)
    mov $4, %r10           // size
    int $0x80
    
    // Emit AFTER marker
    lea .Lmarker_abdf_after(%rip), %r12
    call emit_marker
    
    // Mailbox epoch update
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)

    // Emit completion marker: [U][RUNTIME_BRIDGE_TEST_COMPLETE]
    lea .Lmarker_complete(%rip), %r12
    call emit_marker

.Lloop:
    // Keep mailbox epoch fresh while QEMU harness observes logs
    mov 8(%rbx), %rax
    inc %rax
    mov %rax, 8(%rbx)
    pause
    jmp .Lloop

// Helper: emit null-terminated string via SYS_V2_DEBUG_PUTCHAR (1010)
// Input: %r12 = pointer to string
emit_marker:
    push %r12
.Lemit_loop:
    movzbq (%r12), %rdi
    test %dil, %dil
    je .Lemit_done
    mov $1010, %rax        // SYS_V2_DEBUG_PUTCHAR
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
.Lmarker_start:
    .asciz "[U][RUNTIME_BRIDGE_TEST_START]\n"
.Lmarker_device_before:
    .asciz "[U][RUNTIME_BRIDGE_DEVICE_OP_BEFORE]\n"
.Lmarker_device_after:
    .asciz "[U][RUNTIME_BRIDGE_DEVICE_OP_AFTER]\n"
.Lmarker_external_before:
    .asciz "[U][RUNTIME_BRIDGE_EXTERNAL_CALL_BEFORE]\n"
.Lmarker_external_after:
    .asciz "[U][RUNTIME_BRIDGE_EXTERNAL_CALL_AFTER]\n"
.Lmarker_abdf_before:
    .asciz "[U][RUNTIME_BRIDGE_ABDF_OP_BEFORE]\n"
.Lmarker_abdf_after:
    .asciz "[U][RUNTIME_BRIDGE_ABDF_OP_AFTER]\n"
.Lmarker_complete:
    .asciz "[U][RUNTIME_BRIDGE_TEST_COMPLETE]\n"
