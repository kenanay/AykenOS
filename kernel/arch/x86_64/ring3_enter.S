// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Ring3 Transition Mechanism (x86_64)
// Authority: Phase 10-A2 Task 2
// Constitutional: Ring0 mechanism only (no policy)

.section .text
.global ring3_enter

// Segment selectors (GDT offsets)
.set SEG_KERNEL_CODE, 0x08
.set SEG_KERNEL_DATA, 0x10
.set SEG_USER_CODE,   0x18  // DPL=3
.set SEG_USER_DATA,   0x20  // DPL=3

// RFLAGS bits
.set RFLAGS_IF,       0x200  // Interrupt enable
.set RFLAGS_RESERVED, 0x2    // Reserved bit (always 1)

//
// ring3_enter(uint64_t rip, uint64_t rsp, uint64_t user_cr3)
//
// Transition from Ring0 to Ring3 via IRETQ.
//
// Parameters:
//   RDI = user RIP (entry point)
//   RSI = user RSP (stack pointer)
//   RDX = user CR3 (page table physical address)
//
// Preconditions (MUST be validated by caller):
//   - user_cr3 is valid physical address
//   - user RIP is canonical
//   - user RSP is canonical and aligned
//   - TSS.RSP0 is set to valid kernel stack
//   - User page tables are mapped
//
// Postconditions:
//   - CPU in Ring3
//   - CS = SEG_USER_CODE | 3
//   - SS = SEG_USER_DATA | 3
//   - RFLAGS.IF = 1 (interrupts enabled)
//   - CR3 = user_cr3
//
// Failure modes:
//   - Triple fault if preconditions violated
//   - General Protection Fault if segment DPL mismatch
//   - Page Fault if user page tables invalid
//
ring3_enter:
    // Marker: entering Ring3 transition
    // (Visible in QEMU trace: mov $0x5233, %eax)
    mov $0x5233, %eax  // "R3" marker (0x52='R', 0x33='3')

    // Save parameters (IRETQ will clobber registers)
    mov %rdi, %r8      // r8 = user_rip
    mov %rsi, %r9      // r9 = user_rsp
    mov %rdx, %r10     // r10 = user_cr3

    // Switch to user page tables
    // CRITICAL: This must happen before IRETQ
    mov %r10, %cr3

    // Marker: CR3 switched
    mov $0x4352, %eax  // "CR" marker (0x43='C', 0x52='R')

    // Build IRETQ stack frame (5 qwords):
    //   [RSP+32] SS      (user data segment | RPL=3)
    //   [RSP+24] RSP     (user stack pointer)
    //   [RSP+16] RFLAGS  (IF=1, reserved=1)
    //   [RSP+8]  CS      (user code segment | RPL=3)
    //   [RSP+0]  RIP     (user entry point)

    // Push SS (user data segment with RPL=3)
    mov $SEG_USER_DATA, %rax
    or $3, %rax        // Set RPL=3 (Ring3)
    push %rax

    // Push user RSP
    push %r9

    // Push RFLAGS (IF=1, reserved=1)
    mov $(RFLAGS_IF | RFLAGS_RESERVED), %rax
    push %rax

    // Push CS (user code segment with RPL=3)
    mov $SEG_USER_CODE, %rax
    or $3, %rax        // Set RPL=3 (Ring3)
    push %rax

    // Push user RIP
    push %r8

    // Marker: IRETQ frame ready
    mov $0x4952, %eax  // "IR" marker (0x49='I', 0x52='R')

    // Transition to Ring3
    // IRETQ will:
    //   1. Pop RIP, CS, RFLAGS, RSP, SS
    //   2. Switch to Ring3 (CPL=3)
    //   3. Enable interrupts (RFLAGS.IF=1)
    //   4. Jump to user RIP
    iretq

    // UNREACHABLE: If we get here, something is very wrong
    ud2
