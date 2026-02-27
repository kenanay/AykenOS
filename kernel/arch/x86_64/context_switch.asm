; kernel/arch/x86_64/context_switch.asm

global context_switch
global switch_to_first
global kernel_first_entry
global kernel_iret_entry
global syscall_isr
global timer_isr_asm

extern syscall_handler
extern init_process_main
extern sched_irq_user_ctx_saved
extern timer_isr_c
extern sched_take_resched
extern sched_yield_irq
extern ring3_enter_iretq

; cpu_context_t ABI offsets (single source of truth)
%include "ayken_abi.inc"

; timer_isr_asm pushes 15 GPRs before CPU IRQ frame fields (RIP at +120).
%assign IRQ_PUSHED_GPRS 15
%if (IRQ_PUSHED_GPRS * 8) != IRQF_RIP
%error "IRQ frame ABI drift: push-count no longer matches IRQF_RIP"
%endif
%if IRQF_SIZE != 160
%error "IRQ frame ABI drift: unexpected IRQF_SIZE"
%endif

%ifdef AYKEN_DEBUG_SCHED
%if AYKEN_DEBUG_SCHED
%define DEBUG_SCHED 1
%else
%define DEBUG_SCHED 0
%endif
%else
%define DEBUG_SCHED 0
%endif

%macro DBG_CHAR 1
%if DEBUG_SCHED
    mov al, %1
    out 0xE9, al
%endif
%endmacro

%macro DBG_ASSERT_RSP_ALIGNED 1
%if DEBUG_SCHED
    mov rax, rsp
    and rax, 0xF
    jz %%ok
    DBG_CHAR '['
    DBG_CHAR 'A'
    DBG_CHAR 'L'
    DBG_CHAR 'N'
    DBG_CHAR %1
    DBG_CHAR ']'
%%halt:
    cli
    hlt
    jmp %%halt
%%ok:
%endif
%endmacro

; First kernel task entry: establish a clean frame then call C init
kernel_first_entry:
    DBG_CHAR 'J'
    ; switch_to_first ring0 path enters via JMP with RSP % 16 = 8.
    ; Align to 16 before C call to satisfy SysV ABI.
    sub rsp, 8
    DBG_ASSERT_RSP_ALIGNED '1'
    mov rbp, 0
    call init_process_main
    add rsp, 8
.L_kernel_first_hang:
    hlt
    jmp .L_kernel_first_hang

; Interrupt-safe kernel entry point (IRET compatible)
kernel_iret_entry:
    DBG_CHAR 'J'

    ; IRET pushed 5 qwords (40 bytes), so RSP % 16 = 8.
    ; Align stack to 16 before C call.
    sub rsp, 8
    DBG_ASSERT_RSP_ALIGNED '2'

    mov rbp, 0
    call init_process_main

    ; If init_process_main returns, restore stack and halt.
    add rsp, 8
.L_iret_hang:
    hlt
    jmp .L_iret_hang

; void context_switch(cpu_context_t *old, cpu_context_t *new)
context_switch:
    ; If IRQ path already snapshotted interrupted user context, do not overwrite
    ; old->rip/rsp/rflags/cs/ss with scheduler-frame values.
    mov eax, dword [rel sched_irq_user_ctx_saved]
    test eax, eax
    jz .L_ctx_save_old
    mov dword [rel sched_irq_user_ctx_saved], 0
    jmp .L_ctx_load_new

.L_ctx_save_old:
    ; Save old callee-saved regs.
    mov [rdi + CTX_R15], r15
    mov [rdi + CTX_R14], r14
    mov [rdi + CTX_R13], r13
    mov [rdi + CTX_R12], r12
    mov [rdi + CTX_RBX], rbx
    mov [rdi + CTX_RBP], rbp

    ; Save RSP/RFLAGS/CR3 and current CS/SS.
    mov [rdi + CTX_RSP], rsp
    pushfq
    pop rax
    mov [rdi + CTX_RFLAGS], rax
    mov rax, cr3
    mov [rdi + CTX_CR3], rax

    mov ax, cs
    mov [rdi + CTX_CS], ax
    mov ax, ss
    mov [rdi + CTX_SS], ax

.L_ctx_load_new:
    ; Load next callee-saved regs.
    mov r15, [rsi + CTX_R15]
    mov r14, [rsi + CTX_R14]
    mov r13, [rsi + CTX_R13]
    mov r12, [rsi + CTX_R12]
    mov rbx, [rsi + CTX_RBX]
    mov rbp, [rsi + CTX_RBP]

    ; Load target context core fields.
    mov r11, [rsi + CTX_RIP]
    mov rcx, [rsi + CTX_RSP]
    mov rdx, [rsi + CTX_RFLAGS]
    mov r8,  [rsi + CTX_CR3]
    movzx r9,  word [rsi + CTX_CS]
    movzx r10, word [rsi + CTX_SS]

    ; If target is ring3, return with IRETQ frame.
    test r9w, 3
    jnz .L_ring3_ret

.L_ring0_ret:
    ; Ring0 return path: restore CR3/RSP/RFLAGS then RET to RIP.
    mov cr3, r8
    mov rsp, rcx
    or rdx, 0x200
    push rdx
    popfq
    push r11
    ret

.L_ring3_ret:
    ; Canonical Ring3 entry path: all CR3/RFLAGS/IRETQ semantics live in ring3_enter.S.
    mov rdi, r11 ; rip
    mov rsi, rcx ; rsp
    mov rcx, r8  ; user cr3
    jmp ring3_enter_iretq

; void switch_to_first(cpu_context_t *ctx)
switch_to_first:
    DBG_CHAR 't'

    mov r15, [rdi + CTX_R15]
    mov r14, [rdi + CTX_R14]
    mov r13, [rdi + CTX_R13]
    mov r12, [rdi + CTX_R12]
    mov rbx, [rdi + CTX_RBX]
    mov rbp, [rdi + CTX_RBP]

    mov r11, [rdi + CTX_RIP]
    mov rcx, [rdi + CTX_RSP]
    mov rdx, [rdi + CTX_RFLAGS]
    mov r8,  [rdi + CTX_CR3]
    movzx r9,  word [rdi + CTX_CS]
    movzx r10, word [rdi + CTX_SS]

    test r9w, 3
    jnz .L_first_ring3

.L_first_ring0:
    mov cr3, r8
    mov rsp, rcx
    jmp r11

.L_first_ring3:
    ; Canonical Ring3 entry path shared with context_switch().
    mov rdi, r11 ; rip
    mov rsi, rcx ; rsp
    mov rcx, r8  ; user cr3
    jmp ring3_enter_iretq

; -----------------------------------------------------------------------------
; Timer Interrupt Handler (IRQ0 -> Vector 32)
; -----------------------------------------------------------------------------
timer_isr_asm:
    ; Save full GP register context.
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
    mov rdi, rsp

    ; Align stack dynamically before C call.
    mov rax, rsp
    and rax, 0xF
    jz .L_irq_aligned
    sub rsp, 8
    mov rbx, 1
    jmp .L_irq_call_c
.L_irq_aligned:
    xor rbx, rbx
.L_irq_call_c:
    DBG_ASSERT_RSP_ALIGNED 'T'
    call timer_isr_c

    test rbx, rbx
    jz .L_irq_no_restore
    add rsp, 8
.L_irq_no_restore:

    ; IRQ-tail reschedule/switch path.
    call sched_take_resched
    test eax, eax
    jz .L_irq_no_switch
    DBG_CHAR 'Y'
    call sched_yield_irq
.L_irq_no_switch:

    ; Restore full GP context and return from interrupt.
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
    iretq

; -----------------------------------------------------------------------------
; Syscall Interrupt Handler (INT 0x80)
; -----------------------------------------------------------------------------
; User ABI: RAX=num, RDI=arg1, RSI=arg2, RDX=arg3, R10=arg4
; C ABI:    RDI=num, RSI=arg1, RDX=arg2, RCX=arg3, R8 =arg4
; -----------------------------------------------------------------------------
syscall_isr:
    ; Preserve original RAX syscall number before register shuffling.
    mov r11, rax

    ; Save caller-saved registers we preserve for user return.
    push r11
    push r10
    push r9
    push r8
    push rcx
    push rdx
    push rsi
    push rdi
    mov rax, r11

    ; SysV ABI alignment before C call.
    sub rsp, 8

    ; uint64_t syscall_handler(uint64_t num, uint64_t arg1,
    ;                          uint64_t arg2, uint64_t arg3, uint64_t arg4)
    mov r8,  r10
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax
    DBG_ASSERT_RSP_ALIGNED 'S'

    call syscall_handler

    add rsp, 8

    ; Restore preserved registers.
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9
    pop r10
    pop r11

    ; Return to user; RAX is syscall return value.
    iretq
