# USER Worker Test Payload
# Purpose: Poll execution inbox, pickup work, complete execution
# Expected: Receive execution from BCIB worker, complete successfully
#
# Execution Inbox Structure (at EXECUTION_INBOX_VA = 0x600000):
#   +0x00: magic (4 bytes) - 0x41584942 ("AXIB")
#   +0x04: version (4 bytes)
#   +0x08: state (4 bytes) - AXIB_STATE_READY = 1
#   +0x0C: reserved (4 bytes)
#   +0x10: execution_id (8 bytes)
#   +0x18: target_context_id (8 bytes)
#   +0x20: bcib_user_va (8 bytes)
#   +0x28: bcib_size (8 bytes)
#   +0x30: bcib_window_size (8 bytes)
#   +0x38: delivery_seq (8 bytes)
#   +0x40: flags (8 bytes)

.section .text
.global _start

_start:
    # Load inbox address into rbx (EXECUTION_INBOX_VA = 0x600000)
    mov $0x600000, %rbx

    # Emit start marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'S', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'R', %al
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

poll_inbox:
    # Read inbox state (+0x08 offset)
    mov 0x08(%rbx), %eax
    
    # Check if state == AXIB_STATE_READY (1)
    cmp $1, %eax
    jne poll_inbox
    
    # Inbox ready! Read execution_id (+0x10 offset)
    mov 0x10(%rbx), %rdi
    
    # Emit delivery marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'S', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'N', %al
    out %al, %dx
    mov $'B', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'X', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'A', %al
    out %al, %dx
    mov $'D', %al
    out %al, %dx
    mov $'Y', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $'\n', %al
    out %al, %dx
    
    # Call SYS_V2_COMPLETE_EXECUTION (syscall 1004)
    # rdi already contains execution_id
    mov $1004, %rax
    xor %rsi, %rsi              # result_code = 0 (success)
    xor %rdx, %rdx              # output_size = 0 (no output for now)
    xor %r10, %r10              # flags = 0
    int $0x80
    
    # Check return value (should be 0 for success)
    test %rax, %rax
    jnz complete_failed
    
    # Emit success marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'S', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'C', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'M', %al
    out %al, %dx
    mov $'P', %al
    out %al, %dx
    mov $'L', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $'E', %al
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
    
    # Emit pipeline done marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'X', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'C', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'N', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'P', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'P', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'L', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'N', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'D', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'N', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $'\n', %al
    out %al, %dx
    
    # Exit cleanly
    mov $60, %rax
    xor %rdi, %rdi
    int $0x80

complete_failed:
    # Emit failure marker
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'U', %al
    out %al, %dx
    mov $'S', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'C', %al
    out %al, %dx
    mov $'O', %al
    out %al, %dx
    mov $'M', %al
    out %al, %dx
    mov $'P', %al
    out %al, %dx
    mov $'L', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'T', %al
    out %al, %dx
    mov $'E', %al
    out %al, %dx
    mov $'_', %al
    out %al, %dx
    mov $'F', %al
    out %al, %dx
    mov $'A', %al
    out %al, %dx
    mov $'I', %al
    out %al, %dx
    mov $'L', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $'\n', %al
    out %al, %dx
    
    # Infinite loop fallback
1:
    hlt
    jmp 1b
