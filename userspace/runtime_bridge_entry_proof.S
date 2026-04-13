# Ultra-minimal payload: Assembly version with embedded marker
# This CANNOT be optimized away by compiler

.section .rodata
marker_string:
    .ascii "[RB_PAYLOAD_V1_ENTRY]\n\0"

.section .text
.global _start

_start:
    # Print marker using SYS_V2_DEBUG_PUTCHAR (1010)
    # rax = syscall number, rdi = character
    
    # Load marker address
    lea marker_string(%rip), %rsi
    
print_loop:
    # Load character
    movzbl (%rsi), %edi
    
    # Check for null terminator
    test %edi, %edi
    jz done_printing
    
    # Syscall: SYS_V2_DEBUG_PUTCHAR = 1010
    mov $1010, %rax
    int $0x80
    
    # Next character
    inc %rsi
    jmp print_loop

done_printing:
    # Infinite loop
    pause
    jmp done_printing
