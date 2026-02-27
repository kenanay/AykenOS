# Implementation Plan: Minimal Ring3 ELF Loader

**Author:** Kenan AY  
**Phase:** 10 (Ring3 Execution)  
**Status:** In Progress  
**Version:** 2.0 (Revised to match implementation reality)

## Overview

This implementation plan follows a strict phased approach to incrementally prove Ring3 execution capability in AykenOS. The plan is divided into phases:

- **Phase 10-A1**: Ring3 Process Preparation ✅ COMPLETED
- **Phase 10-A2**: Real CPL3 Entry Proof 📋 PLANNED
- **Phase 10-B**: Full ELF Parsing 📋 PLANNED
- **Phase 10-C**: Process Integration 📋 PLANNED

**CRITICAL: Phase 10-A split into two sub-phases based on implementation reality:**

### Phase 10-A1 (COMPLETED) ✅

**Goal:** Prepare Ring3 process and enqueue for scheduler

**Implementation:**
- `jump_to_ring3()` → `proc_create_user_process()` → process queue
- ELF loaded, address space mapped, context initialized
- Process registered and marked PROC_READY

**Markers:**
- `KERNEL_BEFORE_RING3`
- `[[AYKEN_RING3_PREP_OK]]`

**Proof:**
- `proc_find_by_pid(pid) == proc`
- `proc->state == PROC_READY`
- Process queued for scheduler dispatch

**Status:** NO CPL3 entry yet (scheduler dispatch not active)

### Phase 10-A2 (PLANNED) 📋

**Goal:** Prove actual CPL3 execution via scheduler dispatch → IRETQ → #BP

**Implementation:**
- Scheduler dispatch path active
- `ring3_enter()` assembly function
- IRETQ privilege transition
- #BP exception handler detects Ring3 origin

**Markers:**
- `P10_TSS_OK` (after TSS/GDT/IDT validation)
- `P10_CR3_SWITCH` (after CR3 load)
- `P10_RING3_ENTER` (before IRETQ)
- `P10_RING3_USER_CODE` (from #BP handler in Ring3)

**Proof:**
- CPL3 execution confirmed via #BP exception
- All markers in correct order
- No triple fault, no #GP/#PF

**Prerequisites:**
- Phase 10-A1 completed
- TSS/GDT/IDT properly configured
- Scheduler dispatch path implemented

---

## Tasks

### Phase 10-A1: Ring3 Process Preparation (COMPLETED) ✅

**Status:** IMPLEMENTED in `kernel/ring3_jump.c` and `kernel/proc/proc.c`

**Implementation Files:**
- `kernel/ring3_jump.c`: `jump_to_ring3()` entry point
- `kernel/proc/proc.c`: `proc_create_user_process()` implementation
- `kernel/elf/parser.c`: Private ELF validation helpers (static, not exported)
- `userspace/minimal/`: Minimal Ring3 test program
- `tools/embed_elf.py`: ELF embedding tool

**Completed Tasks:**

- [x] 1. Create minimal Ring3 userspace program
  - [x] 1.1 Create userspace/minimal directory structure
  - [x] 1.2 Write minimal Ring3 program (minimal.c) with INT3 at 0x400000
  - [x] 1.3 Create linker script (user.ld) with single RX segment
  - [x] 1.4 Create Makefile for minimal userspace program
  - [x] 1.5 Write unit test for minimal ELF structure

- [x] 2. Create ELF embedding tool
  - [x] 2.1 Write Python script (tools/embed_elf.py)
  - [x] 2.2 Integrate into kernel build (Makefile)

- [x] 3. Implement minimal ELF validation (private helpers)
  - [x] 3.1 Create ELF header structures (kernel/include/elf/elf64.h)
  - [x] 3.2 Write minimal ELF validation (kernel/elf/parser.c - PRIVATE)
    - **CRITICAL: `elf64_validate_minimal()` is STATIC (not exported)**
    - **Ring0 export policy: Only test entry points exported**
    - Validates magic, class, machine, bounds
  - [x] 3.3 Write entry point extraction (PRIVATE helper)
    - **CRITICAL: `elf64_get_entry()` is STATIC (not exported)**
  - [x] 3.4 Write property test for ELF magic validation (optional)
  - [x] 3.5 Write property test for entry point extraction (optional)

- [x] 4. Implement user address space creation
  - [x] 4.1 Create user address space structures (kernel/include/mm/user_as.h)
  - [x] 4.2 Implement PML4 allocation (kernel/mm/paging.c)
    - Implemented in `paging_create_user_pml4()`
    - Copies kernel half (entries 256-511)
    - Clears USER bit on kernel mappings

- [x] 5. Implement minimal segment loading
  - [x] 5.1 Page flag derivation (implemented in proc.c)
  - [x] 5.2 Segment address validation (implemented in proc.c)
  - [x] 5.3 Minimal segment loader (implemented in `load_user_image()`)

- [x] 6. Implement execution environment setup
  - [x] 6.1 Allocate user stack (implemented in proc_create_user_process)
    - Stack: 2 pages at USER_STACK_TOP
    - Guard page: Not explicitly unmapped (implicit)
  - [x] 6.2 Create initial CPU context (implemented in proc_create_user_process)
    - RIP = entry point
    - RSP = USER_STACK_TOP - 16
    - CS = 0x23, SS = 0x1B
    - RFLAGS = 0x202

- [x] 7. Implement process creation and queueing
  - [x] 7.1 Implement `proc_create_user_process()` (kernel/proc/proc.c)
    - Allocates PCB
    - Creates user address space
    - Loads ELF segments
    - Allocates stack
    - Initializes context
    - Marks process PROC_READY

- [x] 8. Integrate into kernel boot
  - [x] 8.1 Implement `jump_to_ring3()` (kernel/ring3_jump.c)
    - Calls `proc_create_user_process()`
    - Validates registration invariant
    - Validates runnable state invariant
    - Emits markers
  - [x] 8.2 Call from `kernel_late_init()` (kernel/kernel.c)

- [x] 9. CI gate for preparation validation
  - [x] 9.1 Marker validation
    - Expected: `KERNEL_BEFORE_RING3` → `[[AYKEN_RING3_PREP_OK]]`
    - Fail markers: `[[AYKEN_RING3_PREP_FAIL]] <reason>`

**Phase 10-A1 Proof:**
```
Markers: KERNEL_BEFORE_RING3 → [[AYKEN_RING3_PREP_OK]]
Invariants: 
  - proc_find_by_pid(pid) == proc
  - proc->state == PROC_READY
  - Process queued for scheduler
```

---

### Phase 10-A2: Real CPL3 Entry Proof (PLANNED) 📋

**Goal:** Prove actual CPL3 execution via scheduler dispatch → IRETQ → #BP exception

**Prerequisites:**
- Phase 10-A1 completed ✅
- Scheduler dispatch path active
- TSS/GDT/IDT properly configured

- [x] 1. Validate GDT, IDT, and TSS configuration (CRITICAL prerequisite)
  - [x] 1.1 Implement `validate_gdt_user_segments()`
    - Verify GDT entry 3 (CS=0x23): DPL=3, present, code segment
    - Verify GDT entry 4 (SS=0x1B): DPL=3, present, data segment
    - _Requirements: 5.1, 5.6_

  - [x] 1.2 Implement `validate_idt_bp_gate()`
    - Verify IDT entry 3 (#BP): present bit set
    - **Note: DPL=3 is debugger-friendly/future-proof, not strictly required for INT3**
    - **Critical: Handler must use correct stack (TSS/RSP0)**
    - _Requirements: 5.1, 5.6_

  - [x] 1.3 Implement `validate_tss_for_ring3()`
    - Verify TSS structure is defined and initialized
    - Verify LTR (Load Task Register) has been called
    - Verify TSS.RSP0 points to valid kernel stack
    - **CRITICAL: Without proper TSS/RSP0, Ring3→Ring0 exception causes #DF → triple fault**
    - _Requirements: 5.1, 5.6_

  - [x] 1.4 Call all three validation functions before scheduler dispatch
    - Emit P10_TSS_OK marker after successful validation
    - _Note: This is a hidden dependency - Phase 10-A2 will fail silently without it_

- [ ] 2. Implement Ring3 entry assembly (IRETQ)
  - [ ] 2.1 Write ring3_enter assembly function (kernel/arch/x86_64/ring3_enter.S)
    - Declare `ring3_enter(cpu_context_t *ctx, uint64_t user_cr3)` as noreturn
    - Save context pointer in RBX (callee-saved)
    - Load user CR3 into CR3 register (TLB flush implicit)
    - Emit P10_CR3_SWITCH marker (inline serial write, preserve registers)
    - Load all GPRs from context (except RSP, RIP)
    - **CRITICAL: Ensure RSP is 16-byte aligned BEFORE pushing IRETQ frame**
    - Push IRETQ frame: SS (0x1B), RSP, RFLAGS, CS (0x23), RIP
    - **Note: Stack grows down, push order is REVERSE of frame layout**
    - Emit P10_RING3_ENTER marker (BEFORE IRETQ)
    - Restore RBX from context
    - Execute IRETQ
    - Add UD2 after IRETQ (never reached)
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 12.1, 12.2, 12.4_

  - [ ] 2.2 Add marker emission macros (kernel/include/markers.h)
    - Define `EMIT_MARKER_ASM(marker_string)` macro
    - Direct serial port write to 0xE9
    - **CRITICAL: Preserve all registers (pushfq/popfq)**
    - _Requirements: 12.1, 12.2, 12.4_

  - [ ] 2.3 Implement #BP exception handler for Ring3 marker
    - Extend existing #BP (INT3) handler (kernel/arch/x86_64/interrupts.c)
    - **CRITICAL: Comprehensive Ring3 detection:**
      - Check CPL: `(frame->cs & 0x3) == 0x3`
      - Check SS: `(frame->ss & 0x3) == 0x3`
      - Check CS value: `frame->cs == 0x23`
      - Check SS value: `frame->ss == 0x1B`
      - Check RIP range: `0x400000 <= frame->rip < 0x00007FFFFFFFFFFF`
      - Check RIP canonical: upper bits sign extension of bit 47
      - All checks MUST pass
    - If Ring3: emit P10_RING3_USER_CODE marker
    - After marker: halt or panic (Phase 10-A2 proof complete)
    - If Ring0: handle as normal breakpoint
    - _Requirements: 12.3_

- [ ] 3. Integrate with scheduler dispatch
  - [ ] 3.1 Modify scheduler dispatch to call ring3_enter for user processes
    - Check if process is PROC_TYPE_USER
    - Load process context and CR3
    - Call ring3_enter (never returns on first entry)
    - _Requirements: 5.1, 5.2, 5.3_

- [ ] 4. Create CI gate for CPL3 execution validation
  - [ ] 4.1 Write marker extraction script (tools/ci/extract_markers.py)
  - [ ] 4.2 Write marker order validation (tools/ci/validate_marker_order_phase10a2.py)
    - Expected order: KERNEL_BEFORE_RING3, [[AYKEN_RING3_PREP_OK]], P10_TSS_OK, P10_CR3_SWITCH, P10_RING3_ENTER, P10_RING3_USER_CODE
  - [ ] 4.3 Write CI gate script (scripts/ci/gate_ring3_execution_phase10a2.sh)
  - [ ] 4.4 Integrate into CI workflow

- [ ] 5. Checkpoint - Phase 10-A2 validation
  - Ensure CI gate passes (all markers in correct order)
  - Verify Ring3 code executes (P10_RING3_USER_CODE marker)
  - Verify no #GP/#PF/triple fault
  - **CRITICAL: Phase 10-A2 CPL3 Entry Checklist (13 points):**
    1. ✓ TSS loaded? (LTR called, TSS descriptor valid)
    2. ✓ RSP0 valid? (TSS.RSP0 points to kernel stack)
    3. ✓ IDT #BP present? (Entry 3, present bit set)
    4. ✓ User segment DPL=3? (GDT entries 3 and 4)
    5. ✓ PML4 USER bit correct? (Entries 0-255 can have USER, 256-511 must NOT)
    6. ✓ CR3 switch correct? (User PML4 physical address loaded)
    7. ✓ NX flag correct? (NX=0 for executable, NX=1 for non-executable)
    8. ✓ RIP canonical? (0x400000 in canonical lower half)
    9. ✓ Stack mapped? (USER|WRITABLE|NX)
    10. ✓ Stack RSP valid? (within mapped page)
    11. ✓ Guard page unmapped? (if implemented)
    12. ✓ Marker order correct? (6 markers)
    13. ✓ No triple fault? (Check QEMU output)

---

### Phase 10-B: Full ELF Parsing (PLANNED) 📋

- [ ] 11. Implement full ELF validation
- [ ] 12. Implement PT_LOAD iteration
- [ ] 13. Implement full segment loading with BSS support
- [ ] 14. Implement comprehensive error handling and cleanup
- [ ] 15. Implement full ELF loader entry point
- [ ] 16. Checkpoint - Phase 10-B validation

---

### Phase 10-C: Process Integration (PLANNED) 📋

- [ ] 17. Implement process control block (PCB) - PARTIALLY DONE
- [ ] 18. Integrate ELF loader with PCB - DONE in Phase 10-A1
- [ ] 19. Integrate with scheduler - PARTIALLY DONE
- [ ] 20. Implement context switch path (kernel ↔ user)
- [ ] 21. Implement syscall entry path (Ring3 → Ring0)
- [ ] 22. Final checkpoint - Phase 10-C validation

---

## Notes

- **Phase 10-A1 is COMPLETE** - Process preparation and queueing works
- **Phase 10-A2 is NEXT** - Real CPL3 entry via scheduler dispatch
- **Ring0 Export Policy:** ELF parser helpers are PRIVATE (static, not exported)
- **Marker Sequence (10-A1):** KERNEL_BEFORE_RING3 → [[AYKEN_RING3_PREP_OK]]
- **Marker Sequence (10-A2):** + P10_TSS_OK → P10_CR3_SWITCH → P10_RING3_ENTER → P10_RING3_USER_CODE
- **Critical Path:** TSS/GDT/IDT validation MUST run before scheduler dispatch
- **Debug Strategy:** Use 13-point checklist for Phase 10-A2 (prevents triple fault)
