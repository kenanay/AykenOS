.section .text
.global ayken_jump_to_kernel_raw
.extern boot_idt_install
.extern g_handoff_entry
.extern g_handoff_boot
.extern g_handoff_stack
.extern g_handoff_cr3

# MS x64 ABI (UEFI/PE-COFF):
#   RCX = entry (function pointer)
#   RDX = boot_info pointer
#   R8  = stack_top
#   R9  = cr3
#
# Kernel (SysV AMD64 ABI) expects:
#   RDI = boot_info
ayken_jump_to_kernel_raw:
    cli
    mov $'A', %al
    outb %al, $0xE9

    lea g_handoff_entry(%rip), %rsi
    mov $'p', %al
    outb %al, $0xE9
    mov (%rsi), %r11
    mov $'q', %al
    outb %al, $0xE9
    mov g_handoff_boot(%rip), %rdi
    mov g_handoff_stack(%rip), %r10
    mov g_handoff_cr3(%rip), %rax

    # install IDT while still on UEFI tables
    push %r11
    push %rdi
    push %r10
    push %rax
    call boot_idt_install
    pop %rax
    pop %r10
    pop %rdi
    pop %r11
    mov $'I', %al
    outb %al, $0xE9

    mov %r10, %rsp           # kernel stack
    mov $'B', %al
    outb %al, $0xE9

    # Jump to kernel entry point (CR3 already switched in C code)
    jmp *%r11

.data
.align 8
.global g_handoff_entry
g_handoff_entry:
    .quad 0
.global g_handoff_boot
g_handoff_boot:
    .quad 0
.global g_handoff_stack
g_handoff_stack:
    .quad 0
.global g_handoff_cr3
g_handoff_cr3:
    .quad 0
