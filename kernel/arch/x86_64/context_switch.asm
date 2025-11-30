; kernel/arch/x86_64/context_switch.asm

global context_switch
global switch_to_first

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

    ; Load new registers
    mov r15, [rsi + 0]
    mov r14, [rsi + 8]
    mov r13, [rsi +16]
    mov r12, [rsi +24]
    mov rbx, [rsi +32]
    mov rbp, [rsi +40]

    ; Load RIP, RSP, RFLAGS
    mov rax, [rsi +48]    ; rip
    mov rcx, [rsi +56]    ; rsp
    mov rdx, [rsi +64]    ; rflags
    mov r8,  [rsi +72]    ; cr3

    mov cr3, r8

    mov rsp, rcx
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

    mov rax, [rdi +48]    ; rip
    mov rcx, [rdi +56]    ; rsp
    mov rdx, [rdi +64]    ; rflags
    mov r8,  [rdi +72]    ; cr3

    mov cr3, r8
    mov rsp, rcx
    push rdx
    popfq
    push rax
    ret
