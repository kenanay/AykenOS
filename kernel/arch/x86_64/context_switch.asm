; kernel/arch/x86_64/context_switch.asm

global context_switch
global switch_to_first
global syscall_isr
extern syscall_handler

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
    mov r9w, [rsi +80]    ; cs (word)
    mov r10w,[rsi +82]    ; ss (word)

    ; Load CR3 (PML4)
    mov cr3, r8

    ; Set new RSP
    mov rsp, rcx

    ; If target is user mode (CS == 0x23), build IRET frame
    cmp r9w, 0x23
    jne .L_ring0_ret

    ; Ring3: push SS, RSP, RFLAGS, CS, RIP then iretq
    push r10                ; SS (user)
    push rcx                ; RSP
    push rdx                ; RFLAGS
    push r9                 ; CS (user)
    push rax                ; RIP
    iretq

.L_ring0_ret:
    ; Ring0: normal return via RET
    push rdx
    popfq
    push rax
    ret

; void switch_to_first(cpu_context_t *ctx)
switch_to_first:
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
    mov r9w, [rdi +80]      ; cs
    mov r10w,[rdi +82]      ; ss

    mov cr3, r8
    mov rsp, rcx

    cmp r9w, 0x23
    jne .L_first_ring0

    push r10
    push rcx
    push rdx
    push r9
    push rax
    iretq

.L_first_ring0:
    push rdx
    popfq
    push rax
    ret

; -----------------------------------------------------------------------------
; Syscall Interrupt Handler (INT 0x80)
; -----------------------------------------------------------------------------
; User ABI: RAX=num, RDI=arg1, RSI=arg2, RDX=arg3, R10=arg4
; C ABI:    RDI=num, RSI=arg1, RDX=arg2, RCX=arg3, R8 =arg4
; -----------------------------------------------------------------------------
syscall_isr:
    ; Save caller-saved registers that we want to preserve for the user
    push r11
    push r10
    push r9
    push r8
    push rcx
    push rdx
    push rsi
    push rdi
    
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