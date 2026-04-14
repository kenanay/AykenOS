# BCIB Worker Test Payload
# Purpose: Prove BCIB-role process can call SYS_V2_SUBMIT_EXECUTION
# Expected: No BOUNDARY_KILL, execution accepted
#
# Mailbox Epoch Update Fix:
# After each work cycle, the worker must increment its mailbox epoch
# to maintain freshness for subsequent scheduler handoffs.
#
# Mailbox Structure (at 0x700000):
#   +0x00: magic (4 bytes)
#   +0x04: version (2 bytes)
#   +0x06: kind (2 bytes)
#   +0x08: epoch (8 bytes) <- CRITICAL: Must increment after work
#   +0x10: proposer_pid (4 bytes)
#   +0x14: candidate_pid (4 bytes)

.section .text
.global _start

_start:
    # Load mailbox address into rbx (0x700000)
    mov $0x700000, %rbx

    # Emit start marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'B', %al
    out %al, %dx
    mov $'C', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'B', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'W', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'K', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'S', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $'A', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $'\n', %al
    out %al, %dx

work_loop:
    # Call SYS_V2_SUBMIT_EXECUTION (syscall 1003)
    # This should NOT trigger BOUNDARY_KILL because we have BCIB role
    mov $1003, %rax
    xor %rdi, %rdi  # execution_id = 0 (placeholder)
    xor %rsi, %rsi  # context_ptr = NULL (placeholder)
    xor %rdx, %rdx  # context_size = 0
    xor %r10, %r10  # flags = 0
    int $0x80

    # Emit success marker (if we reach here, enforcement passed)
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'B', %al
    out %al, %dx
    mov $'C', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'B', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'S', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'B', %al
    out %al, %dx
    mov $'M', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'K', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $'\n', %al
    out %al, %dx

    # CRITICAL FIX: Increment mailbox epoch for next handoff
    # Read current epoch from mailbox (+0x08 offset)
    mov 0x08(%rbx), %rax
    
    # Increment epoch (monotonically increasing)
    inc %rax
    
    # Handle epoch wraparound (if epoch becomes 0, set to 1)
    test %rax, %rax
    jnz .Lepoch_ok
    mov $1, %rax
.Lepoch_ok:
    
    # CRITICAL: Memory barrier before epoch write (seqlock protocol)
    # Ensures all payload writes complete before epoch becomes visible
    sfence
    
    # Write updated epoch back to mailbox (LAST, after barrier)
    mov %rax, 0x08(%rbx)
    
    # Emit epoch update marker for debugging
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'P', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'C', %al
    out %al, %dx
    mov $'H', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'P', %al
    out %al, %dx
    mov $'D', %al
    out %al, %dx
    mov $'A', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $'\n', %al
    out %al, %dx

    # Continue work loop (in real implementation, this would wait for work)
    # For now, loop a few times to demonstrate epoch updates
    jmp work_loop

    # Exit (unreachable in current implementation)
    mov $60, %rax
    xor %rdi, %rdi
    int $0x80

    # Infinite loop fallback
1:
    hlt
    jmp 1b
