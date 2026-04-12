// Minimal entry marker: emit 'K' to debugcon (0xE9) then halt.
// Note: int3 requires a valid stack for its exception frame.
.section .text.boot, "ax"
.global kmain
.type kmain, @function
.extern kmain_real
kmain:
    # BLOCK 2: Channel Proof - Test A (Raw byte channel test)
    # Write 'K' to debugcon (port 0xE9) at actual entry point
    mov $'K', %al
    out %al, $0xE9
    
    # Initialize COM1 (serial port 0x3F8) deterministically BEFORE write
    # CRITICAL: This makes serial fallback deterministic, not random
    
    # Disable interrupts (IER = 0)
    mov $0x00, %al
    mov $0x3F9, %dx  # IER port
    out %al, %dx
    
    # Set DLAB=1 to access divisor
    mov $0x80, %al
    mov $0x3FB, %dx  # LCR port
    out %al, %dx
    
    # Set divisor = 1 (115200 baud)
    mov $0x01, %al
    mov $0x3F8, %dx  # DLL port
    out %al, %dx
    mov $0x00, %al
    mov $0x3F9, %dx  # DLH port
    out %al, %dx
    
    # Set 8N1, DLAB=0
    mov $0x03, %al
    mov $0x3FB, %dx  # LCR port
    out %al, %dx
    
    # Enable FIFO, clear TX/RX
    mov $0xC7, %al
    mov $0x3FA, %dx  # FCR port
    out %al, %dx
    
    # Enable DTR, RTS
    mov $0x03, %al
    mov $0x3FC, %dx  # MCR port
    out %al, %dx
    
    # Poll TX empty bit (bit 5 of LSR)
    mov $0x3FD, %dx  # LSR port
1:
    in %dx, %al
    test $0x20, %al
    jz 1b
    
    # Write 'K' to COM1 (serial fallback) - NOW DETERMINISTIC
    mov $'K', %al
    mov $0x3F8, %dx
    out %al, %dx
    
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
