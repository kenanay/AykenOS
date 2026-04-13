# Ultra-minimal Runtime_Bridge ping test
# Purpose: Prove userspace payload executes
# Single marker: [RB_PING]

.section .text
.global _start

_start:
    # Emit single marker via debug_putchar (syscall 1010)
    # String: "[RB_PING]\n"
    
    mov $1010, %rax          # SYS_V2_DEBUG_PUTCHAR
    mov $'[', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'R', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'B', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'_', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'P', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'I', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'N', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $'G', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $']', %rdi
    int $0x80
    
    mov $1010, %rax
    mov $10, %rdi            # newline
    int $0x80

.loop:
    jmp .loop                # Infinite loop

.section .note.GNU-stack,"",@progbits
