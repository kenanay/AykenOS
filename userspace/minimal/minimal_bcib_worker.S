# userspace/minimal/minimal_bcib_worker.S
# Phase-16 Task 5: BCIB Worker Payload - Execution Pipeline Validation
#
# Purpose: Submit minimal BCIB graph via SYS_V2_SUBMIT_EXECUTION
# Role: PROC_EXECUTION_ROLE_BCIB (assigned by kernel at process creation)
# Target: USER worker process (PID 2) for execution delivery
#
# Marker Flow:
# - BW_START: Worker started
# - BW_SUBMIT_OK: Submission succeeded
# - BW_SUBMIT_FAIL: Submission failed
# - BW_EXIT: Worker exiting
#
# Constitutional Compliance:
# - No role escalation
# - No enforcement bypass
# - Fail-closed on submission failure

.section .text
.global _start

# Syscall numbers (from shared/abi/ayken_abi.h)
.equ SYS_WRITE, 1
.equ SYS_EXIT, 60
.equ SYS_V2_SUBMIT_EXECUTION, 1003

# Debugcon port for marker emission
.equ DEBUGCON_PORT, 0xE9

_start:
    # Emit BW_START marker
    leaq marker_bw_start(%rip), %rsi
    call emit_marker
    
    # Prepare submit_execution syscall arguments
    # syscall(SYS_V2_SUBMIT_EXECUTION, bcib_graph_ptr, graph_size, target_pid, flags)
    movq $SYS_V2_SUBMIT_EXECUTION, %rax
    leaq minimal_bcib_graph(%rip), %rdi      # arg1: bcib_graph_ptr
    movq minimal_bcib_graph_size(%rip), %rsi # arg2: graph_size
    movq $2, %rdx                            # arg3: target_pid (USER worker PID 2)
    xorq %r10, %r10                          # arg4: flags (0)
    
    # Execute syscall
    int $0x80
    
    # Check return value (execution_id in rax)
    # Success: execution_id > 0
    # Failure: execution_id == 0 or negative
    testq %rax, %rax
    jle submit_failed
    
submit_success:
    # Emit BW_SUBMIT_OK marker
    leaq marker_bw_submit_ok(%rip), %rsi
    call emit_marker
    jmp exit_worker
    
submit_failed:
    # Emit BW_SUBMIT_FAIL marker
    leaq marker_bw_submit_fail(%rip), %rsi
    call emit_marker
    jmp exit_worker
    
exit_worker:
    # Emit BW_EXIT marker
    leaq marker_bw_exit(%rip), %rsi
    call emit_marker
    
    # Exit cleanly
    movq $SYS_EXIT, %rax
    xorq %rdi, %rdi
    syscall

# ============================================================================
# Helper: Emit marker to debugcon
# Input: rsi = marker string address
# ============================================================================
emit_marker:
    pushq %rax
    pushq %rdx
    pushq %rsi
    
.emit_loop:
    lodsb                    # Load byte from [rsi] into al, increment rsi
    testb %al, %al           # Check for null terminator
    jz .emit_done
    movw $DEBUGCON_PORT, %dx
    outb %al, %dx            # Write character to debugcon
    jmp .emit_loop
    
.emit_done:
    # Emit newline
    movb $10, %al
    movw $DEBUGCON_PORT, %dx
    outb %al, %dx
    
    popq %rsi
    popq %rdx
    popq %rax
    ret

# ============================================================================
# Markers (Audit Trail)
# ============================================================================
.section .rodata

marker_bw_start:
    .asciz "[U][BW_START]"

marker_bw_submit_ok:
    .asciz "[U][BW_SUBMIT_OK]"

marker_bw_submit_fail:
    .asciz "[U][BW_SUBMIT_FAIL]"

marker_bw_exit:
    .asciz "[U][BW_EXIT]"

# ============================================================================
# Minimal BCIB Graph (Placeholder for Phase 1)
# ============================================================================
# This is a minimal placeholder graph for Phase 1 validation
# Real BCIB graph structure will be implemented in later phases

minimal_bcib_graph:
    .quad 0x4243494247525048  # Magic: "BCIBGRPH" (BCIB Graph)
    .quad 1                   # Version: 1
    .quad 0                   # Node count: 0 (minimal graph)
    .quad 0                   # Edge count: 0 (minimal graph)

minimal_bcib_graph_size:
    .quad 32  # Size of minimal graph (4 * 8 bytes)
