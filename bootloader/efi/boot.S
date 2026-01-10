.section .text
.global ayken_jump_to_kernel

ayken_jump_to_kernel:
    # rdi = kernel_entry (ayken_kernel_entry_t)
    # rsi = boot_info pointer

    # ABI: 1. arg -> rdi
    mov %rsi, %rsi   # boot_info zaten rsi
    mov %rdi, %rax   # entry adresini rax'e al

    # Jump to kernel entry: kmain(boot_info)
    jmp *%rax
