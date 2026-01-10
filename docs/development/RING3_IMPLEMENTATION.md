# Ring3 Transition Implementation Summary

**Oluşturan:** Kenan AY  
**Oluşturma Tarihi:** 01.01.2026

**Date:** January 1, 2026  
**Status:** IMPLEMENTED IN CODE — RUNTIME TEST PENDING
**Completion Ratio:** Code changes applied; QEMU/interrupt/syscall verification pending

## Changes Made

### 1. GDT Setup (kernel/arch/x86_64/gdt_idt.c)

**Before:** Minimal placeholder, no Ring3 support

**After:** Full Ring0/Ring3 GDT with TSS
- 6 GDT entries:
  - Entry 0: Null descriptor (required)
  - Entry 1: Ring0 Code (CS=0x08, L=64-bit, DPL=0)
  - Entry 2: Ring0 Data (SS=0x10, DPL=0)
  - Entry 3: Ring3 Data (SS=0x1B, DPL=3)
  - Entry 4: Ring3 Code (CS=0x23, L=64-bit, DPL=3)
  - Entry 5: TSS Descriptor (2 entries for 16-byte TSS structure)

- TSS (Task State Segment):
  - RSP0: Kernel stack pointer (updated at context switch)
  - IST1-7: Interrupt Stack Tables (reserved for future use)
  - IO Map Base: Set to sizeof(TSS)

**Key Functions:**
- `gdt_init()`: Initializes GDT, TSS, and loads with LGDT
- `ltr(uint16_t)`: Loads Task Register (TR) with TSS selector
- `gdt_set_kernel_stack(uint64_t rsp0)`: Inline function to update TSS.RSP0

### 2. Context Switch Assembly (kernel/arch/x86_64/context_switch.asm)

**Before:** Only CR3/RIP/RSP/RFLAGS, no privilege level change (Ring0 only)

**After:** Full IRET support with Ring0/Ring3 handling

**Changes:**
- Added CS and SS fields to saved context (offsets 80, 82)
- Detects privilege level from CS selector
- **Ring3:** Full IRET frame (SS:RSP, RFLAGS, CS:RIP)
- **Ring0:** Simple RET for efficiency

**New Code Paths:**
```asm
.ring3_iret:
    push r10                ; SS (Ring3 stack segment)
    push rcx                ; RSP
    push rdx                ; RFLAGS
    push r9                 ; CS (Ring3 code segment)
    push rax                ; RIP
    iretq                   ; Return to Ring3 (privilege drop)

.ring0_return:
    push rdx
    popfq
    push rax
    ret                     ; Return to Ring0
```

### 3. CPU Context Structure (kernel/include/proc.h)

**Before:**
```c
typedef struct cpu_context {
    uint64_t r15, r14, r13, r12;
    uint64_t rbx, rbp;
    uint64_t rip;
    uint64_t rsp;
    uint64_t rflags;
    uint64_t cr3;
} cpu_context_t;
```

**After:**
```c
typedef struct cpu_context {
    // Callee-saved general registers
    uint64_t r15, r14, r13, r12;
    uint64_t rbx, rbp;
    
    // Instruction pointer, stack pointer, flags
    uint64_t rip;
    uint64_t rsp;
    uint64_t rflags;
    
    // Memory management
    uint64_t cr3;
    
    // Ring3 context (for privilege level transitions)
    uint16_t cs;            // Code segment selector
    uint16_t ss;            // Stack segment selector
    uint64_t rsp0;          // Kernel stack RSP0 (for Ring0 when interrupted)
} cpu_context_t;
```

### 4. Process Allocation (kernel/proc/proc.c)

**Before:** No segment selector or privilege level setup

**After:**
- Ring0 processes: CS=0x08, SS=0x10
- Ring3 processes: CS=0x23, SS=0x1B
- Kernel stack (RSP0) allocated for Ring3 processes (4KB)

### 5. User Process Creation (kernel/proc/proc.c)

**Before:** Only user space stack setup

**After:**
- User space stack: 2 pages at USER_STACK_TOP (USER_STACK_BASE)
- Kernel stack: 1 page (4KB) allocated, RSP0 set to top
- Both stacks properly isolated in separate address spaces

### 6. Scheduler Updates (kernel/sched/sched.c)

**Changes:**
- `sched_start()`: Update TSS.RSP0 when first process starts
- `sched_yield()`: Update TSS.RSP0 on every context switch
- Calls `gdt_set_kernel_stack()` for Ring3 processes

**Logic:**
```c
if (current_proc->context.rsp0) {
    gdt_set_kernel_stack(current_proc->context.rsp0);
}
```

### 7. GDT IDT Header (kernel/include/gdt_idt.h)

**New File:** Complete Ring0/Ring3 GDT interface

**Exports:**
- Segment selectors (GDT_KERNEL_CODE, GDT_USER_CODE, etc.)
- TSS structure definition
- `gdt_set_kernel_stack()` inline function

### 8. Kernel Init (kernel/kernel.c)

**Before:** No `idt_init()` call

**After:**
```c
cpu_init();
gdt_init();       // NEW: Initialize GDT with TSS
idt_init();       // NEW: Load IDT
interrupts_install();
```

## Architecture Verification

### Ring3→Ring0 Transition (Interrupt/Syscall)

**Flow:**
1. User code (Ring3) executes INT 0x80 or HW interrupt
2. CPU checks gate descriptor DPL (≤ IOPL) → Gate is DPL=0, CPL=3 OK
3. CPU loads TSS.RSP0 (set by scheduler)
4. CPU pushes old (Ring3) SS, RSP, RFLAGS, CS, RIP onto kernel stack
5. CPU sets CPL=0, loads new CS/SS (Ring0)
6. Handler executes in Ring0 on kernel stack
7. Handler returns via IRET

**Kernel Stack Layout:**
```
[RSP0]     ← TSS.RSP0 (set by scheduler)
[interrupt frame pushed by CPU]
[handler local vars]
```

**User Stack:** Untouched during interrupt (Ring0 uses kernel stack)

### Ring0→Ring3 Transition (Context Switch)

**Flow:**
1. IRET in context_switch.asm detects CS=0x23 (Ring3)
2. IRET pops Ring3 state (SS, RSP, RFLAGS, CS, RIP) from kernel stack
3. CPU sets CPL=3, switches to Ring3 stack
4. User code continues at RIP with Ring3 privileges

**Key:** CPL (Current Privilege Level) determined by low 2 bits of CS
- CS = 0x08 → CPL=0 (Ring0)
- CS = 0x23 → CPL=3 (Ring3)

## Correctness Checks

### GDT Descriptor Fields
- **P (Present):** 1 for all valid entries
- **DPL:** 0 for Ring0, 3 for Ring3
- **Type:** 0x0B (executable), 0x03 (data), 0x09 (TSS)
- **G (Granularity):** 1 (4K pages)
- **L (Long):** 1 for 64-bit code segments
- **D (Default):** Varies (1 for 32-bit data, 0 for TSS)

### TSS Descriptor
- **Length:** sizeof(TSS) - 1 = 103
- **Access:** 0x89 (P=1, DPL=0, Type=0x09)
- **Base:** 64-bit pointer split across 4 fields

### IRET Frame (Ring3)
```
[RSP+32]  SS      (Ring3 0x1B)
[RSP+24]  RSP     (Ring3 user stack)
[RSP+16]  RFLAGS  
[RSP+8]   CS      (Ring3 0x23)
[RSP+0]   RIP     (user code address)
```

## Testing Checklist

- [ ] Compile without errors
- [ ] Boot to scheduler startup
- [ ] Timer interrupt triggers sched_yield
- [ ] User process created with cs=0x23, ss=0x1B
- [ ] Context switch loads proper segment selectors
- [ ] Syscall INT 0x80 accessible from Ring3
- [ ] Return from syscall via IRET lands back in Ring3
- [ ] Memory isolation: user code can't access kernel pages
- [ ] Interrupt during Ring3 uses kernel stack (RSP0)

## Impact

**Before:** User mode execution was not possible under the scheduler.

**After (code):** Context switch and IRET-based Ring3 entry are implemented in source; selector values set to `CS=0x23` / `SS=0x1B`. Runtime behavior (interrupts/syscalls from Ring3) is **pending verification** in QEMU or real hardware.

**Faz 1 Completion (code-level):** ~80% — critical code changes applied, final verification pending.
 - Ring3 transition: Implemented in code; QEMU test pending
- sched_add_task(): ✅ DONE (was already fixed)
- DevFS framework: ⚠️ TODO
- Build environment: ⚠️ TODO
- BCIB implementation: ⚠️ TODO (Faz 2)

## Files Modified

1. `kernel/arch/x86_64/gdt_idt.c` - Complete GDT/TSS implementation
2. `kernel/arch/x86_64/context_switch.asm` - IRET-based Ring3 support
3. `kernel/include/proc.h` - Added cs, ss, rsp0 to cpu_context_t
4. `kernel/proc/proc.c` - Ring3 context setup, kernel stack allocation
5. `kernel/sched/sched.c` - TSS.RSP0 update at context switch
6. `kernel/include/gdt_idt.h` - NEW: GDT interface header
7. `kernel/kernel.c` - Added idt_init() call

---

## Next Steps (for Faz 1 completion)

Not: Bu belge 01.01.2026 tarihinde hazırlanmıştır. Yapılan değişiklikler kod tabanına uygulanmıştır; gerçek doğrulama için lütfen proje derlenip QEMU üzerinde test yapınız. Derleme için WSL2 önerilir — ayrıntılar `README.md` içindedir.

1. **DevFS Framework** (1-2 days)
   - Device registry
   - /dev node mounting
   - Stub drivers

2. **Build Environment** (1-2 hours)
   - WSL 2 or Docker
   - Cross-compiler verification

3. **Integration Testing**
   - Full boot with Ring3 process
   - Syscall from user space
   - Context switching between Ring3 processes

