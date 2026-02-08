// Minimal entry marker: emit 'K' to debugcon (0xE9) then halt.
// Note: int3 requires a valid stack for its exception frame.
.section .text.boot, "ax"
.global kmain
.type kmain, @function
.extern kmain_real
kmain:
    # Set a known-good temporary stack; keep 16-byte alignment
    leaq kernel_boot_stack_top(%rip), %rsp
    andq $~0xFULL, %rsp
    # Bootloader passes boot_info* in RDI (SysV ABI)
    call kmain_real
1:
    hlt
    jmp 1b
.section .bss
.align 16
kernel_boot_stack:
    .skip 4096
kernel_boot_stack_top:
.section .text.boot, "ax"
