; kernel/arch/x86_64/context_switch.asm

global context_switch
global switch_to_first
global kernel_first_entry
global syscall_isr
extern syscall_handler
extern init_process_main

; First kernel task entry: establish a clean frame then call C init
kernel_first_entry:
    mov al, 'J'
    out 0xE9, al
    mov rbp, 0
    call init_process_main
.L_kernel_first_hang:
    hlt
    jmp .L_kernel_first_hang

; void context_switch(cpu_context_t *old, cpu_context_t *new)
context_switch:
    ; Save old registers
    mov [rdi + 0], r15
    mov [rdi + 8], r14
    mov [rdi +16], r13
    mov [rdi +24], r12
    mov [rdi +32], rbx
    mov [rdi +40], rbp

    ; Save RIP/RSP/RFLAGS/CR3
    mov rax, [rsp]
    mov [rdi +48], rax
    mov [rdi +56], rsp
    pushfq
    pop rax
    mov [rdi +64], rax
    mov rax, cr3
    mov [rdi +72], rax

    ; Save current CS/SS
    mov ax, cs
    mov [rdi +80], ax
    mov ax, ss
    mov [rdi +82], ax

    ; Load new registers
    mov r15, [rsi + 0]
    mov r14, [rsi + 8]
    mov r13, [rsi +16]
    mov r12, [rsi +24]
    mov rbx, [rsi +32]
    mov rbp, [rsi +40]

    ; Load RIP, RSP, RFLAGS, CR3, CS, SS
    mov rax, [rsi +48]    ; rip
    mov rcx, [rsi +56]    ; rsp
    mov rdx, [rsi +64]    ; rflags
    mov r8,  [rsi +72]    ; cr3
    and r8, 0xFFFFFFFFFFFFF000
    movzx r9,  word [rsi +80]    ; cs (zero-extend)
    movzx r10, word [rsi +82]    ; ss (zero-extend)

    ; If target is user mode (CS == 0x23), build IRET frame
    cmp r9w, 0x23
    je .L_ring3_ret
    cmp r10w, 0x23
    je .L_ring3_ret
    jmp .L_ring0_ret

.L_ring3_ret:
    ; Ring3: push SS, RSP, RFLAGS, CS, RIP then iretq
    mov r11, rax
    mov al, '['
    out 0xE9, al
    mov al, 'U'
    out 0xE9, al
    mov al, ']'
    out 0xE9, al
    mov al, '['
    out 0xE9, al
    mov al, 'R'
    out 0xE9, al
    mov al, 'I'
    out 0xE9, al
    mov al, 'N'
    out 0xE9, al
    mov al, 'G'
    out 0xE9, al
    mov al, '3'
    out 0xE9, al
    mov al, '_'
    out 0xE9, al
    mov al, 'O'
    out 0xE9, al
    mov al, 'K'
    out 0xE9, al
    mov al, ']'
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    mov rax, r11
    or rdx, 0x200          ; IF=1
    or rdx, 0x2            ; bit1 must be 1
    push r10                ; SS (user)
    push rcx                ; RSP
    push rdx                ; RFLAGS
    push r9                 ; CS (user)
    push rax                ; RIP
    mov cr3, r8
    iretq

.L_ring0_ret:
    ; Ring0: normal return via RET
    mov cr3, r8
    mov rsp, rcx
    mov r11, rax
    mov al, 'k'
    out 0xE9, al
    mov rax, r11
    push rdx
    popfq
    push rax
    ret

; void switch_to_first(cpu_context_t *ctx)
switch_to_first:
    mov al, 't'
    out 0xE9, al
    mov r15, [rdi + 0]
    mov r14, [rdi + 8]
    mov r13, [rdi +16]
    mov r12, [rdi +24]
    mov rbx, [rdi +32]
    mov rbp, [rdi +40]

    mov rax, [rdi +48]      ; rip
    mov rcx, [rdi +56]      ; rsp
    mov rdx, [rdi +64]      ; rflags
    mov r8,  [rdi +72]      ; cr3
    and r8, 0xFFFFFFFFFFFFF000
    movzx r9,  word [rdi +80]      ; cs (zero-extend)
    movzx r10, word [rdi +82]      ; ss (zero-extend)

    cmp r9w, 0x23
    je .L_first_ring3
    cmp r10w, 0x23
    je .L_first_ring3
    jmp .L_first_ring0
    
.L_first_ring3:
    mov r11, rax
    mov al, '['
    out 0xE9, al
    mov al, 'U'
    out 0xE9, al
    mov al, ']'
    out 0xE9, al
    mov al, '['
    out 0xE9, al
    mov al, 'R'
    out 0xE9, al
    mov al, 'I'
    out 0xE9, al
    mov al, 'N'
    out 0xE9, al
    mov al, 'G'
    out 0xE9, al
    mov al, '3'
    out 0xE9, al
    mov al, '_'
    out 0xE9, al
    mov al, 'O'
    out 0xE9, al
    mov al, 'K'
    out 0xE9, al
    mov al, ']'
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    mov rax, r11
    or rdx, 0x200          ; IF=1
    or rdx, 0x2            ; bit1 must be 1

    push r10
    push rcx
    push rdx
    push r9
    push rax
    mov cr3, r8
    iretq

.L_first_ring0:
    mov cr3, r8
    mov rsp, rcx
    mov r11, rax
    mov al, 'k'
    out 0xE9, al
    mov rax, r11
    ; Bring-up path: avoid stack usage until we confirm stack is stable
    ; Keep IF as-is (scheduler disabled interrupts before switch)
    jmp rax

; -----------------------------------------------------------------------------
; Syscall Interrupt Handler (INT 0x80)
; -----------------------------------------------------------------------------
; User ABI: RAX=num, RDI=arg1, RSI=arg2, RDX=arg3, R10=arg4
; C ABI:    RDI=num, RSI=arg1, RDX=arg2, RCX=arg3, R8 =arg4
; -----------------------------------------------------------------------------
syscall_isr:
    ; Emit marker before touching the stack (diagnose CPL3->CPL0 switch)
    mov r11, rax
    mov al, '['
    out 0xE9, al
    mov al, 'U'
    out 0xE9, al
    mov al, ']'
    out 0xE9, al
    mov al, '['
    out 0xE9, al
    mov al, 'S'
    out 0xE9, al
    mov al, 'Y'
    out 0xE9, al
    mov al, 'S'
    out 0xE9, al
    mov al, 'C'
    out 0xE9, al
    mov al, 'A'
    out 0xE9, al
    mov al, 'L'
    out 0xE9, al
    mov al, 'L'
    out 0xE9, al
    mov al, '_'
    out 0xE9, al
    mov al, 'O'
    out 0xE9, al
    mov al, 'K'
    out 0xE9, al
    mov al, ']'
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al

    ; Save caller-saved registers that we want to preserve for the user
    push r11
    push r10
    push r9
    push r8
    push rcx
    push rdx
    push rsi
    push rdi
    mov rax, r11
    
    ; Prepare arguments for C function syscall_handler
    ; uint64_t syscall_handler(uint64_t num, uint64_t arg1, uint64_t arg2, uint64_t arg3, uint64_t arg4)
    
    mov r8, r10      ; arg4 (User R10 -> C R8)
    mov rcx, rdx     ; arg3 (User RDX -> C RCX)
    mov rdx, rsi     ; arg2 (User RSI -> C RDX)
    mov rsi, rdi     ; arg1 (User RDI -> C RSI)
    mov rdi, rax     ; num  (User RAX -> C RDI)

    call syscall_handler
    
    ; Restore registers
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9
    pop r10
    pop r11
    
    ; Return from interrupt (RAX holds the return value from syscall_handler)
    iretq
