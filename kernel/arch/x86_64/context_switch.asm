; kernel/arch/x86_64/context_switch.asm

global context_switch
global switch_to_first
global kernel_first_entry
global kernel_iret_entry
global syscall_isr
extern syscall_handler
extern init_process_main
extern sched_irq_user_ctx_saved

; First kernel task entry: establish a clean frame then call C init
kernel_first_entry:
    mov al, 'J'
    out 0xE9, al
    mov rbp, 0          ; Clear frame pointer
    call init_process_main
.L_kernel_first_hang:
    hlt
    jmp .L_kernel_first_hang

; Interrupt-safe kernel entry point (IRET compatible)
kernel_iret_entry:
    mov al, 'J'
    out 0xE9, al
    
    ; CRITICAL: Stack alignment fix for C function call
    ; IRET pushed 5 qwords (40 bytes), so RSP % 16 = 8
    ; Need to align to 16 before calling C function
    sub rsp, 8          ; Align stack to 16 bytes
    
    mov rbp, 0          ; Clear frame pointer  
    call init_process_main
    
    ; If init_process_main returns, restore stack and halt
    add rsp, 8          ; Restore stack alignment
.L_iret_hang:
    hlt
    jmp .L_iret_hang

; void context_switch(cpu_context_t *old, cpu_context_t *new)
context_switch:
    ; If IRQ path already saved interrupted user context, do not overwrite
    ; old->rip/rsp/rflags/cs/ss with kernel scheduler frame values.
    mov eax, dword [rel sched_irq_user_ctx_saved]
    test eax, eax
    jz .L_ctx_save_old
    mov dword [rel sched_irq_user_ctx_saved], 0
    jmp .L_ctx_load_new

.L_ctx_save_old:
    ; Save old registers
    mov [rdi + 0], r15
    mov [rdi + 8], r14
    mov [rdi +16], r13
    mov [rdi +24], r12
    mov [rdi +32], rbx
    mov [rdi +40], rbp

    ; CRITICAL FIX: DO NOT save RIP from stack!
    ; RIP is set during process creation (proc_create_*), not during context switch
    ; [rsp] contains the return address to context_switch caller (kernel code)
    ; Saving it would overwrite the task's real RIP with a kernel address
    ; ❌ OLD BUGGY CODE (REMOVED):
    ;    mov rax, [rsp]
    ;    mov [rdi +48], rax
    ; ✅ RIP is already set in context, don't touch it during switch
    
    ; Save RSP/RFLAGS/CR3
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

.L_ctx_load_new:
    ; Load new registers
    mov r15, [rsi + 0]
    mov r14, [rsi + 8]
    mov r13, [rsi +16]
    mov r12, [rsi +24]
    mov rbx, [rsi +32]
    mov rbp, [rsi +40]

    ; ✅ DEBUG: Show RSI (new context pointer) and RIP value
    mov al, 'C'
    out 0xE9, al
    mov al, 'T'
    out 0xE9, al
    mov al, 'X'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Show RSI as 8 hex digits
    mov rax, rsi
    shr rax, 28
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d1
    add rax, 'A' - 10
    jmp .ctx_o1
.ctx_d1:
    add rax, '0'
.ctx_o1:
    out 0xE9, al
    mov rax, rsi
    shr rax, 24
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d2
    add rax, 'A' - 10
    jmp .ctx_o2
.ctx_d2:
    add rax, '0'
.ctx_o2:
    out 0xE9, al
    mov rax, rsi
    shr rax, 20
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d3
    add rax, 'A' - 10
    jmp .ctx_o3
.ctx_d3:
    add rax, '0'
.ctx_o3:
    out 0xE9, al
    mov rax, rsi
    shr rax, 16
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d4
    add rax, 'A' - 10
    jmp .ctx_o4
.ctx_d4:
    add rax, '0'
.ctx_o4:
    out 0xE9, al
    mov rax, rsi
    shr rax, 12
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d5
    add rax, 'A' - 10
    jmp .ctx_o5
.ctx_d5:
    add rax, '0'
.ctx_o5:
    out 0xE9, al
    mov rax, rsi
    shr rax, 8
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d6
    add rax, 'A' - 10
    jmp .ctx_o6
.ctx_d6:
    add rax, '0'
.ctx_o6:
    out 0xE9, al
    mov rax, rsi
    shr rax, 4
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d7
    add rax, 'A' - 10
    jmp .ctx_o7
.ctx_d7:
    add rax, '0'
.ctx_o7:
    out 0xE9, al
    mov rax, rsi
    and rax, 0xF
    cmp rax, 10
    jl .ctx_d8
    add rax, 'A' - 10
    jmp .ctx_o8
.ctx_d8:
    add rax, '0'
.ctx_o8:
    out 0xE9, al
    mov al, ' '
    out 0xE9, al
    
    ; ✅ CRITICAL FIX #2: Load RIP into R11 immediately (before any debug prints)
    ; R11 will hold the target RIP and MUST NOT be touched by debug code
    ; We use R11 instead of R15 because R15 is a callee-saved register that needs to be restored
    mov r11, [rsi +48]    ; rip -> R11 (PROTECTED)
    
    ; ✅ DEBUG: Show RIP value immediately after loading
    mov al, '@'
    out 0xE9, al
    mov al, '4'
    out 0xE9, al
    mov al, '8'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Show R11 as 8 hex digits
    mov rax, r11
    shr rax, 28
    and rax, 0xF
    cmp rax, 10
    jl .at48_d1
    add rax, 'A' - 10
    jmp .at48_o1
.at48_d1:
    add rax, '0'
.at48_o1:
    out 0xE9, al
    mov rax, r11
    shr rax, 24
    and rax, 0xF
    cmp rax, 10
    jl .at48_d2
    add rax, 'A' - 10
    jmp .at48_o2
.at48_d2:
    add rax, '0'
.at48_o2:
    out 0xE9, al
    mov rax, r11
    shr rax, 20
    and rax, 0xF
    cmp rax, 10
    jl .at48_d3
    add rax, 'A' - 10
    jmp .at48_o3
.at48_d3:
    add rax, '0'
.at48_o3:
    out 0xE9, al
    mov rax, r11
    shr rax, 16
    and rax, 0xF
    cmp rax, 10
    jl .at48_d4
    add rax, 'A' - 10
    jmp .at48_o4
.at48_d4:
    add rax, '0'
.at48_o4:
    out 0xE9, al
    mov rax, r11
    shr rax, 12
    and rax, 0xF
    cmp rax, 10
    jl .at48_d5
    add rax, 'A' - 10
    jmp .at48_o5
.at48_d5:
    add rax, '0'
.at48_o5:
    out 0xE9, al
    mov rax, r11
    shr rax, 8
    and rax, 0xF
    cmp rax, 10
    jl .at48_d6
    add rax, 'A' - 10
    jmp .at48_o6
.at48_d6:
    add rax, '0'
.at48_o6:
    out 0xE9, al
    mov rax, r11
    shr rax, 4
    and rax, 0xF
    cmp rax, 10
    jl .at48_d7
    add rax, 'A' - 10
    jmp .at48_o7
.at48_d7:
    add rax, '0'
.at48_o7:
    out 0xE9, al
    mov rax, r11
    and rax, 0xF
    cmp rax, 10
    jl .at48_d8
    add rax, 'A' - 10
    jmp .at48_o8
.at48_d8:
    add rax, '0'
.at48_o8:
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    
    mov rcx, [rsi +56]    ; rsp
    mov rdx, [rsi +64]    ; rflags
    mov r8,  [rsi +72]    ; cr3
    and r8, 0xFFFFFFFFFFFFF000
    movzx r9,  word [rsi +80]    ; cs (zero-extend)
    movzx r10, word [rsi +82]    ; ss (zero-extend)

    ; Debug: Show RFLAGS value
    mov al, 'F'
    out 0xE9, al
    mov al, 'L'
    out 0xE9, al
    mov al, 'A'
    out 0xE9, al
    mov al, 'G'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Show RFLAGS as 4 hex digits
    mov rax, rdx
    shr rax, 12
    and rax, 0xF
    cmp rax, 10
    jl .flag_digit1
    add rax, 'A' - 10
    jmp .flag_out1
.flag_digit1:
    add rax, '0'
.flag_out1:
    out 0xE9, al
    mov rax, rdx
    shr rax, 8
    and rax, 0xF
    cmp rax, 10
    jl .flag_digit2
    add rax, 'A' - 10
    jmp .flag_out2
.flag_digit2:
    add rax, '0'
.flag_out2:
    out 0xE9, al
    mov rax, rdx
    shr rax, 4
    and rax, 0xF
    cmp rax, 10
    jl .flag_digit3
    add rax, 'A' - 10
    jmp .flag_out3
.flag_digit3:
    add rax, '0'
.flag_out3:
    out 0xE9, al
    mov rax, rdx
    and rax, 0xF
    cmp rax, 10
    jl .flag_digit4
    add rax, 'A' - 10
    jmp .flag_out4
.flag_digit4:
    add rax, '0'
.flag_out4:
    out 0xE9, al
    mov al, ' '
    out 0xE9, al

    ; Debug: Test ring detection logic
    mov al, 'C'
    out 0xE9, al
    mov al, 'S'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Dump CS value (simple hex)
    mov rax, r9
    shr rax, 12
    and rax, 0xF
    cmp rax, 10
    jl .cs_digit1
    add rax, 'A' - 10
    jmp .cs_out1
.cs_digit1:
    add rax, '0'
.cs_out1:
    out 0xE9, al
    mov rax, r9
    shr rax, 8
    and rax, 0xF
    cmp rax, 10
    jl .cs_digit2
    add rax, 'A' - 10
    jmp .cs_out2
.cs_digit2:
    add rax, '0'
.cs_out2:
    out 0xE9, al
    mov rax, r9
    shr rax, 4
    and rax, 0xF
    cmp rax, 10
    jl .cs_digit3
    add rax, 'A' - 10
    jmp .cs_out3
.cs_digit3:
    add rax, '0'
.cs_out3:
    out 0xE9, al
    mov rax, r9
    and rax, 0xF
    cmp rax, 10
    jl .cs_digit4
    add rax, 'A' - 10
    jmp .cs_out4
.cs_digit4:
    add rax, '0'
.cs_out4:
    out 0xE9, al
    mov al, ' '
    out 0xE9, al
    
    ; If target is user mode, build IRET frame
    ; Test RPL bits instead of hardcoded values
    test r9w, 3          ; Check RPL bits (bit 0-1)
    jnz .L_ring3_ret     ; If RPL != 0, it's ring3
    
    ; Ring0 path marker
    mov al, '0'
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    jmp .L_ring0_ret

.L_ring3_ret:
    ; Ring3 path marker  
    mov al, '3'
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    
    ; ✅ R11 contains the target RIP - NEVER touch it after this point!
    
    ; Ring3: push SS, RSP, RFLAGS, CS, RIP then iretq
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
    
    ; Preempt validation mode: sanitize RFLAGS but keep IF enabled for IRQ0 preemption.
    ; Clear: IOPL(12-13), NT(14), RF(16), VM(17), TF(8)
    ; Set: IF(9), bit1(1)
    mov rax, rdx
    and rax, ~((3<<12) | (1<<14) | (1<<16) | (1<<17) | (1<<8))
    or rax, 0x202          ; IF=1, bit1=1
    mov rdx, rax
    
    ; ✅ CRITICAL: Debug prints BEFORE building IRET frame
    ; Show final RFLAGS (using RDX copy, not touching R11)
    mov al, 'I'
    out 0xE9, al
    mov al, 'R'
    out 0xE9, al
    mov al, 'E'
    out 0xE9, al
    mov al, 'T'
    out 0xE9, al
    mov al, '_'
    out 0xE9, al
    mov al, 'F'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Show RFLAGS as 4 hex digits (using RBX as temp, not RAX)
    mov rbx, rdx
    shr rbx, 12
    and rbx, 0xF
    cmp rbx, 10
    jl .iret_digit1
    add rbx, 'A' - 10
    jmp .iret_out1
.iret_digit1:
    add rbx, '0'
.iret_out1:
    mov al, bl
    out 0xE9, al
    mov rbx, rdx
    shr rbx, 8
    and rbx, 0xF
    cmp rbx, 10
    jl .iret_digit2
    add rbx, 'A' - 10
    jmp .iret_out2
.iret_digit2:
    add rbx, '0'
.iret_out2:
    mov al, bl
    out 0xE9, al
    mov rbx, rdx
    shr rbx, 4
    and rbx, 0xF
    cmp rbx, 10
    jl .iret_digit3
    add rbx, 'A' - 10
    jmp .iret_out3
.iret_digit3:
    add rbx, '0'
.iret_out3:
    mov al, bl
    out 0xE9, al
    mov rbx, rdx
    and rbx, 0xF
    cmp rbx, 10
    jl .iret_digit4
    add rbx, 'A' - 10
    jmp .iret_out4
.iret_digit4:
    add rbx, '0'
.iret_out4:
    mov al, bl
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    
    ; ✅ Show RIP value (using RBX as temp, NEVER touching R11!)
    mov al, 'R'
    out 0xE9, al
    mov al, 'I'
    out 0xE9, al
    mov al, 'P'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Show RIP as 16 hex digits (using RBX, not RAX!)
    mov rbx, r11
    shr rbx, 60
    and rbx, 0xF
    cmp rbx, 10
    jl .rip_d1
    add rbx, 'A' - 10
    jmp .rip_o1
.rip_d1:
    add rbx, '0'
.rip_o1:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 56
    and rbx, 0xF
    cmp rbx, 10
    jl .rip_d2
    add rbx, 'A' - 10
    jmp .rip_o2
.rip_d2:
    add rbx, '0'
.rip_o2:
    mov al, bl
    out 0xE9, al
    ; Show "..." for brevity
    mov al, '.'
    out 0xE9, al
    out 0xE9, al
    out 0xE9, al
    ; Show last 4 digits
    mov rbx, r11
    shr rbx, 12
    and rbx, 0xF
    cmp rbx, 10
    jl .rip_d13
    add rbx, 'A' - 10
    jmp .rip_o13
.rip_d13:
    add rbx, '0'
.rip_o13:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 8
    and rbx, 0xF
    cmp rbx, 10
    jl .rip_d14
    add rbx, 'A' - 10
    jmp .rip_o14
.rip_d14:
    add rbx, '0'
.rip_o14:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 4
    and rbx, 0xF
    cmp rbx, 10
    jl .rip_d15
    add rbx, 'A' - 10
    jmp .rip_o15
.rip_d15:
    add rbx, '0'
.rip_o15:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    and rbx, 0xF
    cmp rbx, 10
    jl .rip_d16
    add rbx, 'A' - 10
    jmp .rip_o16
.rip_d16:
    add rbx, '0'
.rip_o16:
    mov al, bl
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    
    ; ✅ CRITICAL DEBUG: Show R11 value BEFORE pushing to stack
    mov al, 'R'
    out 0xE9, al
    mov al, '1'
    out 0xE9, al
    mov al, '1'
    out 0xE9, al
    mov al, '='
    out 0xE9, al
    ; Show R11 as 8 hex digits (using RBX as temp)
    mov rbx, r11
    shr rbx, 28
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d1
    add rbx, 'A' - 10
    jmp .r11_o1
.r11_d1:
    add rbx, '0'
.r11_o1:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 24
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d2
    add rbx, 'A' - 10
    jmp .r11_o2
.r11_d2:
    add rbx, '0'
.r11_o2:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 20
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d3
    add rbx, 'A' - 10
    jmp .r11_o3
.r11_d3:
    add rbx, '0'
.r11_o3:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 16
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d4
    add rbx, 'A' - 10
    jmp .r11_o4
.r11_d4:
    add rbx, '0'
.r11_o4:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 12
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d5
    add rbx, 'A' - 10
    jmp .r11_o5
.r11_d5:
    add rbx, '0'
.r11_o5:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 8
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d6
    add rbx, 'A' - 10
    jmp .r11_o6
.r11_d6:
    add rbx, '0'
.r11_o6:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    shr rbx, 4
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d7
    add rbx, 'A' - 10
    jmp .r11_o7
.r11_d7:
    add rbx, '0'
.r11_o7:
    mov al, bl
    out 0xE9, al
    mov rbx, r11
    and rbx, 0xF
    cmp rbx, 10
    jl .r11_d8
    add rbx, 'A' - 10
    jmp .r11_o8
.r11_d8:
    add rbx, '0'
.r11_o8:
    mov al, bl
    out 0xE9, al
    mov al, 0x0A
    out 0xE9, al
    
    ; ✅ Build IRET frame - R11 is UNTOUCHED, contains correct RIP
    ; Restore callee-saved RBX after debug formatting that used RBX as scratch.
    mov rbx, [rsi +32]
    push r10                ; SS (user)
    push rcx                ; RSP
    push rdx                ; RFLAGS (sanitized)
    push r9                 ; CS (user)
    push r11                ; RIP (CORRECT VALUE from R11!)
    
    mov cr3, r8
    iretq

.L_ring0_ret:
    ; Ring0: normal return via RET
    mov cr3, r8
    mov rsp, rcx
    ; Restore callee-saved RBX after debug formatting that used RBX as scratch.
    mov rbx, [rsi +32]
    mov al, 'k'
    out 0xE9, al
    ; CRITICAL: Ensure IF=1 for Ring0 processes
    or rdx, 0x200          ; Set IF bit in RFLAGS
    push rdx
    popfq
    push r11                ; RIP is in R11, not RAX
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

    ; ✅ CRITICAL FIX #2: Load RIP into R11 immediately (protected from debug prints)
    ; We use R11 instead of R15 because R15 is a callee-saved register that needs to be restored
    mov r11, [rdi +48]      ; rip -> R11 (PROTECTED)
    mov rcx, [rdi +56]      ; rsp
    mov rdx, [rdi +64]      ; rflags
    mov r8,  [rdi +72]      ; cr3
    and r8, 0xFFFFFFFFFFFFF000
    movzx r9,  word [rdi +80]      ; cs (zero-extend)
    movzx r10, word [rdi +82]      ; ss (zero-extend)

    ; Test RPL bits instead of hardcoded values
    test r9w, 3          ; Check RPL bits (bit 0-1)
    jnz .L_first_ring3   ; If RPL != 0, it's ring3
    jmp .L_first_ring0
    
.L_first_ring3:
    ; ✅ R11 already contains the target RIP (loaded at the beginning)
    ; No need to move from RAX - R11 is protected from debug prints
    
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
    
    ; Preempt validation mode: sanitize RFLAGS but keep IF enabled for IRQ0 preemption.
    ; Clear: IOPL(12-13), NT(14), RF(16), VM(17), TF(8)
    ; Set: IF(9), bit1(1)
    mov rax, rdx
    and rax, ~((3<<12) | (1<<14) | (1<<16) | (1<<17) | (1<<8))
    or rax, 0x202          ; IF=1, bit1=1
    mov rdx, rax

    ; ✅ Build IRET frame with R11 (protected RIP value)
    push r10
    push rcx
    push rdx
    push r9
    push r11                ; RIP (from R11, not RAX!)
    mov cr3, r8
    iretq

.L_first_ring0:
    mov cr3, r8
    mov rsp, rcx
    mov al, 'k'
    out 0xE9, al
    ; Back to simple JMP for debugging - RIP is in R11
    jmp r11

; -----------------------------------------------------------------------------
; Syscall Interrupt Handler (INT 0x80)
; -----------------------------------------------------------------------------
; User ABI: RAX=num, RDI=arg1, RSI=arg2, RDX=arg3, R10=arg4
; C ABI:    RDI=num, RSI=arg1, RDX=arg2, RCX=arg3, R8 =arg4
; -----------------------------------------------------------------------------

; -----------------------------------------------------------------------------
; Timer Interrupt Handler (IRQ0 -> Vector 32)
; -----------------------------------------------------------------------------
; ASM stub that calls C handler - avoids Clang interrupt ABI issues
; -----------------------------------------------------------------------------
global timer_isr_asm
extern timer_isr_c
extern sched_take_resched
extern sched_yield_irq

timer_isr_asm:
    ; Save all registers (full context save for safety)
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    ; Capture real IRQ frame base before optional stack alignment.
    ; C side expects frame_ptr to start at saved r15.
    mov rdi, rsp
    
    ; ✅ CRITICAL: Dynamic stack alignment for C call
    ; CPU pushed interrupt frame (24 or 40 bytes depending on CPL change)
    ; We pushed 15 registers (120 bytes)
    ; Check if RSP % 16 == 0, if not, align it
    mov rax, rsp
    and rax, 0xF
    jz .aligned
    sub rsp, 8          ; Align to 16 bytes
    mov rbx, 1          ; Flag: we aligned
    jmp .call_c
.aligned:
    xor rbx, rbx        ; Flag: no alignment needed
.call_c:
    
    ; Call C handler
    call timer_isr_c
    
    ; Restore alignment if we adjusted it
    test rbx, rbx
    jz .no_restore
    add rsp, 8
.no_restore:
    ; Timer preempt path: switch at IRQ tail to keep C handler side-effect free.
    call sched_take_resched
    test eax, eax
    jz .no_irq_switch
    mov al, 'Y'
    out 0xE9, al
    call sched_yield_irq
.no_irq_switch:
    
    ; Restore all registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    
    ; Return from interrupt
    iretq

syscall_isr:
    ; Preserve original RAX syscall number before register shuffling.
    mov r11, rax
    
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
    
    ; CRITICAL: SysV ABI stack alignment fix
    ; CPU pushed 5 qwords (40 bytes) = RSP % 16 = 8
    ; We pushed 8 qwords (64 bytes) = still RSP % 16 = 8
    ; Need to align to 16 before call
    sub rsp, 8
    
    ; Prepare arguments for C function syscall_handler
    ; uint64_t syscall_handler(uint64_t num, uint64_t arg1, uint64_t arg2, uint64_t arg3, uint64_t arg4)
    mov r8, r10      ; arg4 (User R10 -> C R8)
    mov rcx, rdx     ; arg3 (User RDX -> C RCX)
    mov rdx, rsi     ; arg2 (User RSI -> C RDX)
    mov rsi, rdi     ; arg1 (User RDI -> C RSI)
    mov rdi, rax     ; num  (User RAX -> C RDI)

    call syscall_handler
    
    ; Restore stack alignment
    add rsp, 8
    
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
