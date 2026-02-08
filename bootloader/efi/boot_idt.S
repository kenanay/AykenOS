.section .text
.global boot_idt_install
.global boot_bp_handler
.global boot_pf_handler
.global boot_gp_handler
.global boot_ud_handler

# 0: halt in #BP handler (deterministic), 1: iretq to continue
.equ BP_MODE, 0

# Build a minimal IDT with #GP and #PF handlers.
# Uses KERNEL_CS selector (0x08) from bootloader GDT.
boot_idt_install:
    mov $'i', %al
    outb %al, $0xE9
    lea idt(%rip), %rdi

    # Install #BP (3)
    lea boot_bp_handler(%rip), %rax
    mov $3, %ecx
    call set_gate

    # Install #UD (6)
    lea boot_ud_handler(%rip), %rax
    mov $6, %ecx
    call set_gate

    # Install #GP (13)
    lea boot_gp_handler(%rip), %rax
    mov $13, %ecx
    call set_gate

    # Install #PF (14)
    lea boot_pf_handler(%rip), %rax
    mov $14, %ecx
    call set_gate

    lea idt(%rip), %rax
    mov %rax, idtr_base(%rip)
    movw $idt_end-idt-1, idtr_limit(%rip)
    lidt idtr(%rip)
    ret

# set_gate(rdi=idt_base, rcx=idx, rax=handler)
set_gate:
    mov $'g', %al
    outb %al, $0xE9
    mov %rcx, %rdx
    shl $4, %rdx                 # idx * 16
    lea (%rdi,%rdx), %rdx        # gate addr

    movw %ax, (%rdx)             # offset[15:0]
    mov %cs, %ax
    movw %ax, 2(%rdx)            # selector = current CS
    movb $0, 4(%rdx)             # IST = 0
    movb $0x8E, 5(%rdx)          # present, DPL0, interrupt gate

    shr $16, %rax
    movw %ax, 6(%rdx)            # offset[31:16]
    shr $16, %rax
    movl %eax, 8(%rdx)           # offset[63:32]
    movl $0, 12(%rdx)            # reserved
    ret

# ------------------------
# Breakpoint Handler (#3)
# Stack frame (ring0): [RIP][CS][RFLAGS]
# ------------------------
boot_bp_handler:
    cli
    mov $'!', %al
    outb %al, $0xE9
    mov $'B', %al
    outb %al, $0xE9

    # RIP
    mov $'R', %al
    outb %al, $0xE9
    mov (%rsp), %rax
    call dump_u64_hex_e9

    # CS (16-bit)
    mov $'S', %al
    outb %al, $0xE9
    xor %rax, %rax
    mov 8(%rsp), %ax
    call dump_u16_hex_e9

    # RSP
    mov $'p', %al
    outb %al, $0xE9
    mov %rsp, %rax
    call dump_u64_hex_e9

.if BP_MODE
    iretq
.else
1:  hlt
    jmp 1b
.endif

# ------------------------
# Page Fault Handler (#14)
# Stack frame (ring0): [ERR][RIP][CS][RFLAGS]
# ------------------------
boot_pf_handler:
    cli
    mov $'P', %al
    outb %al, $0xE9

    # CR2
    mov $'C', %al
    outb %al, $0xE9
    mov %cr2, %rax
    call dump_u64_hex_e9

    # RSP
    mov $'S', %al
    outb %al, $0xE9
    mov %rsp, %rax
    call dump_u64_hex_e9

    # Error code
    mov $'E', %al
    outb %al, $0xE9
    mov (%rsp), %rax
    call dump_u64_hex_e9

    # RIP
    mov $'R', %al
    outb %al, $0xE9
    mov 8(%rsp), %rax
    call dump_u64_hex_e9

2:  hlt
    jmp 2b

# ------------------------
# General Protection (#13)
# ------------------------
boot_gp_handler:
    cli
    mov $'G', %al
    outb %al, $0xE9

    # Error code
    mov $'E', %al
    outb %al, $0xE9
    mov (%rsp), %rax
    call dump_u64_hex_e9

    # RIP
    mov $'R', %al
    outb %al, $0xE9
    mov 8(%rsp), %rax
    call dump_u64_hex_e9

    # CS (16-bit)
    mov $'S', %al
    outb %al, $0xE9
    xor %rax, %rax
    mov 16(%rsp), %ax
    call dump_u16_hex_e9

3:  hlt
    jmp 3b

# ------------------------
# Invalid Opcode Handler (#6)
# ------------------------
boot_ud_handler:
    cli
    mov $'Z', %al
    outb %al, $0xE9
4:  hlt
    jmp 4b

# ------------------------
# Hex dump helpers
# ------------------------
dump_u64_hex_e9:
    lea .Lhex_digits_idt(%rip), %rbx
    mov $60, %ecx
4:
    mov %rax, %rdx
    shr %cl, %rdx
    and $0xF, %dl
    movzbq %dl, %r8
    mov (%rbx,%r8,1), %al
    outb %al, $0xE9
    sub $4, %ecx
    jns 4b
    ret

dump_u16_hex_e9:
    and $0xFFFF, %eax
    lea .Lhex_digits_idt(%rip), %rbx
    mov $12, %ecx
5:
    mov %rax, %rdx
    shr %cl, %rdx
    and $0xF, %dl
    movzbq %dl, %r8
    mov (%rbx,%r8,1), %al
    outb %al, $0xE9
    sub $4, %ecx
    jns 5b
    ret

.section .data
.align 16
idt:
    .zero 16 * 256
idt_end:

idtr:
idtr_limit: .word 0
idtr_base:  .quad 0

.section .rodata
.Lhex_digits_idt:
    .ascii "0123456789ABCDEF"
