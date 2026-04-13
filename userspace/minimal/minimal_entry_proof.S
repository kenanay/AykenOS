# Ultra-minimal userspace entry proof
# ONLY proves that userspace payload started executing
# No complex logic - just a unique fingerprint marker

.section .rodata
entry_marker:
    .ascii "[RB_PAYLOAD_V1_ENTRY]\n\0"

.section .text
.global _start

_start:
    # Print entry marker using SYS_V2_DEBUG_PUTCHAR (1010)
    lea entry_marker(%rip), %rsi
    
print_loop:
    movzbl (%rsi), %edi
    test %edi, %edi
    jz done_printing
    
    # SYS_V2_DEBUG_PUTCHAR = 1010
    mov $1010, %rax
    int $0x80
    
    inc %rsi
    jmp print_loop

done_printing:
    # Infinite loop to keep process alive
1:
    pause
    jmp 1b
