# Implementation Plan: Minimal Ring3 ELF Loader

**Author:** Kenan AY  
**Phase:** 10 (Ring3 Execution)  
**Status:** In Progress  
**Version:** 3.0 (Updated based on actual implementation - 2026-02-28)

## Overview

This implementation plan follows a strict phased approach to incrementally prove Ring3 execution capability in AykenOS. The plan is divided into phases:

- **Phase 10-A1**: Ring3 Process Preparation ✅ COMPLETED
- **Phase 10-A2**: Real CPL3 Entry Proof 🔄 IN PROGRESS  
- **Phase 10-B**: Full ELF Parsing ✅ PARTIALLY COMPLETED
- **Phase 10-C**: Process Integration ✅ MOSTLY COMPLETED

## Current Implementation Status (2026-02-28)

### What Has Been Implemented

**Phase 10-A1: Ring3 Process Preparation** ✅ COMPLETE
- `jump_to_ring3()` entry point in `kernel/ring3_jump.c`
- `proc_create_user_process()` full implementation in `kernel/proc/proc.c`
- ELF validation helpers (private, not exported) in `kernel/elf/parser.c`
- User address space creation with PML4 allocation
- Kernel half copying with USER bit clearing (security enforcement)
- PT_LOAD segment loading with proper page flag derivation
- User stack allocation (2 pages at USER_STACK_TOP)
- Kernel stack (RSP0) allocation and mapping in user CR3
- Mailbox allocation and mapping at 0x700000 for scheduler bridge
- Process registration and PROC_READY state
- Minimal userspace program in assembly (`userspace/minimal/minimal.S`)
- ELF embedding tool (`tools/embed_elf.py`)

**Phase 10-B: ELF Parsing** ✅ PARTIALLY COMPLETE
- ELF64 header structures defined in `kernel/include/elf/elf64.h`
- Private ELF validation helpers in `kernel/elf/parser.c`:
  - `elf64_validate_minimal()` - validates magic, class, machine, bounds
  - `elf64_get_entry()` - extracts entry point
  - Both functions are STATIC (not exported to Ring0 surface)
- Full PT_LOAD iteration in `load_elf_image()` in `kernel/proc/proc.c`
- BSS zero-fill support (p_memsz > p_filesz)
- Page flag derivation from ELF p_flags
- Segment bounds validation

**Phase 10-C: Process Integration** ✅ MOSTLY COMPLETE
- PCB (proc_t) structure fully implemented
- PID assignment working
- Scheduler integration via `sched_add()`
- Process state management (PROC_READY, PROC_RUNNING, PROC_BLOCKED)
- Process table and `proc_find_by_pid()` lookup
- Init process creation and PID1 management

### What Remains To Be Done

**Phase 10-A2: Real CPL3 Entry** 🔄 IN PROGRESS
- TSS/GDT/IDT validation functions (Task 1.1, 1.2, 1.3, 1.4)
- `ring3_enter()` assembly function with IRETQ (Task 2.1)
- Marker emission macros for assembly (Task 2.2)
- #BP exception handler Ring3 detection (Task 2.3)
- Scheduler dispatch integration (Task 3.1)
- CI gate for CPL3 execution validation (Task 4)

**Phase 10-B: Full ELF Parsing** 📋 REMAINING
- Comprehensive error handling and cleanup tracking
- W^X enforcement validation
- Segment overlap detection
- Full property-based testing

**Phase 10-C: Process Integration** 📋 REMAINING
- Context switch path refinement (kernel ↔ user)
- Syscall entry path optimization
- Multi-process support testing

---

## Detailed Task Breakdown

### Phase 10-A1: Ring3 Process Preparation (COMPLETED) ✅

**Status:** IMPLEMENTED in `kernel/ring3_jump.c`, `kernel/proc/proc.c`, `kernel/elf/parser.c`

**Implementation Files:**
- `kernel/ring3_jump.c`: `jump_to_ring3()` entry point ✅
- `kernel/proc/proc.c`: `proc_create_user_process()` implementation ✅
- `kernel/elf/parser.c`: Private ELF validation helpers (static, not exported) ✅
- `userspace/minimal/minimal.S`: Minimal Ring3 test program ✅
- `userspace/minimal/Makefile`: Build system for minimal program ✅
- `tools/embed_elf.py`: ELF embedding tool ✅

**Completed Tasks:**

- [x] 1. Create minimal Ring3 userspace program
  - [x] 1.1 Create userspace/minimal directory structure
  - [x] 1.2 Write minimal Ring3 program (minimal.S) with syscall roundtrip test
  - [x] 1.3 Create linker script (user.ld) with single RX segment
  - [x] 1.4 Create Makefile for minimal userspace program
  - [x] 1.5 Write unit test for minimal ELF structure

- [x] 2. Create ELF embedding tool
  - [x] 2.1 Write Python script (tools/embed_elf.py)
  - [x] 2.2 Integrate into kernel build (Makefile)

- [x] 3. Implement minimal ELF validation (private helpers)
  - [x] 3.1 Create ELF header structures (kernel/include/elf/elf64.h)
  - [x] 3.2 Write minimal ELF validation (kernel/elf/parser.c - PRIVATE)
    - **IMPLEMENTED: `elf64_validate_minimal()` is STATIC (not exported)**
    - **Ring0 export policy: Only test entry points exported**
    - Validates magic, class, machine, bounds, program header table, segment file ranges
  - [x] 3.3 Write entry point extraction (PRIVATE helper)
    - **IMPLEMENTED: `elf64_get_entry()` is STATIC (not exported)**
  - [x] 3.4 Write property test for ELF magic validation
    - **IMPLEMENTED: `test_elf_magic_validation()` in parser.c**
  - [x] 3.5 Write property test for entry point extraction
    - **IMPLEMENTED: `test_entry_point_extraction()` in parser.c**

- [x] 4. Implement user address space creation
  - [x] 4.1 Create user address space structures (kernel/include/mm/user_as.h)
  - [x] 4.2 Implement PML4 allocation (kernel/mm/paging.c)
    - **IMPLEMENTED: `paging_create_user_pml4()`**
    - Copies kernel half (entries 256-511)
    - Clears USER bit on kernel mappings (security enforcement)

- [x] 5. Implement segment loading
  - [x] 5.1 Page flag derivation
    - **IMPLEMENTED: Inline in `load_elf_image()` in proc.c**
    - Derives PRESENT, USER, WRITABLE from p_flags
  - [x] 5.2 Segment address validation
    - **IMPLEMENTED: Bounds checking in `load_elf_image()`**
  - [x] 5.3 Full segment loader with PT_LOAD iteration
    - **IMPLEMENTED: `load_elf_image()` in proc.c**
    - Iterates all PT_LOAD segments
    - Handles p_filesz and p_memsz (BSS zero-fill)
    - Maps pages with correct flags

- [x] 6. Implement execution environment setup
  - [x] 6.1 Allocate user stack
    - **IMPLEMENTED: 2 pages at USER_STACK_TOP in `proc_create_user_process()`**
    - Stack pages mapped with USER | WRITABLE | PRESENT
  - [x] 6.2 Create initial CPU context
    - **IMPLEMENTED: Full context initialization in `proc_create_user_process()`**
    - RIP = entry point from ELF
    - RSP = USER_STACK_TOP - 8 (16-byte aligned)
    - CS = 0x23 (GDT_USER_CODE), SS = 0x1B (GDT_USER_DATA)
    - RFLAGS = 0x202 (IF=1, reserved bit 1)
    - RSP0 = kernel stack top (for Ring3→Ring0 transitions)

- [x] 7. Implement process creation and queueing
  - [x] 7.1 Implement `proc_create_user_process()` (kernel/proc/proc.c)
    - **FULLY IMPLEMENTED:**
    - Allocates PCB via `proc_alloc()`
    - Creates user address space via `paging_create_user_pml4()`
    - Loads ELF segments via `load_user_image()`
    - Allocates user stack (2 pages)
    - Allocates scratch page for diagnostics (RING3_CANARY_ADDR)
    - Allocates and maps mailbox at 0x700000 (SCHED_MAILBOX_VA)
    - Allocates kernel stack and sets RSP0
    - Maps kernel stack in user CR3 (for safe IRETQ)
    - Initializes full CPU context
    - Adds process to scheduler via `sched_add()`
    - Marks process PROC_READY

- [x] 8. Integrate into kernel boot
  - [x] 8.1 Implement `jump_to_ring3()` (kernel/ring3_jump.c)
    - **FULLY IMPLEMENTED:**
    - Validates embedded ELF magic
    - Calls `proc_create_user_process()`
    - Validates registration invariant (`proc_find_by_pid()`)
    - Validates runnable state invariant (PROC_READY)
    - Validates segment selectors (CS=0x23, SS=0x1B)
    - Validates RSP0 allocation
    - Emits markers: KERNEL_BEFORE_RING3, [[AYKEN_RING3_PREP_OK]], P10_SCHED_ARMED
  - [x] 8.2 Call from `kernel_late_init()` (kernel/kernel.c)
    - **IMPLEMENTED: Called during boot sequence**

- [x] 9. CI gate for preparation validation
  - [x] 9.1 Marker validation
    - Expected: `KERNEL_BEFORE_RING3` → `[[AYKEN_RING3_PREP_OK]]` → `P10_SCHED_ARMED`
    - Fail markers: `[[AYKEN_RING3_PREP_FAIL]] <reason>`

**Phase 10-A1 Proof:**
```
Markers: KERNEL_BEFORE_RING3 → [[AYKEN_RING3_PREP_OK]] → P10_SCHED_ARMED
Invariants: 
  - proc_find_by_pid(pid) == proc ✅
  - proc->state == PROC_READY ✅
  - proc->context.cs == 0x23 ✅
  - proc->context.ss == 0x1B ✅
  - proc->context.rsp0 != 0 ✅
  - proc->mailbox_pa != 0 ✅
  - Process queued for scheduler ✅
```

---

### Phase 10-A2: Real CPL3 Entry Proof (IN PROGRESS) 🔄

**Goal:** Prove actual CPL3 execution via scheduler dispatch → IRETQ → syscall roundtrip

**Prerequisites:**
- Phase 10-A1 completed ✅
- Scheduler dispatch path active (needs verification)
- TSS/GDT/IDT properly configured (needs validation)

**Current Status:** Preparation complete, waiting for scheduler dispatch implementation

- [x] 1. Validate GDT, IDT, and TSS configuration (CRITICAL prerequisite)
  - [x] 1.1 Implement `validate_gdt_user_segments()`
    - **STATUS:** Needs implementation
    - Verify GDT entry 3 (CS=0x23): DPL=3, present, code segment
    - Verify GDT entry 4 (SS=0x1B): DPL=3, present, data segment
    - _Requirements: 5.1, 5.6_

  - [x] 1.2 Implement `validate_idt_bp_gate()`
    - **STATUS:** Needs implementation
    - Verify IDT entry 3 (#BP): present bit set
    - **Note: DPL=3 is debugger-friendly/future-proof, not strictly required for INT3**
    - **Critical: Handler must use correct stack (TSS/RSP0)**
    - _Requirements: 5.1, 5.6_

  - [x] 1.3 Implement `validate_tss_for_ring3()`
    - **STATUS:** Needs implementation
    - Verify TSS structure is defined and initialized
    - Verify LTR (Load Task Register) has been called
    - Verify TSS.RSP0 points to valid kernel stack
    - **CRITICAL: Without proper TSS/RSP0, Ring3→Ring0 exception causes #DF → triple fault**
    - _Requirements: 5.1, 5.6_

  - [x] 1.4 Call all three validation functions before scheduler dispatch
    - **STATUS:** Needs implementation
    - Emit P10_TSS_OK marker after successful validation
    - _Note: This is a hidden dependency - Phase 10-A2 will fail silently without it_

- [ ] 2. Implement Ring3 entry assembly (IRETQ)
  - [ ] 2.1 Write ring3_enter assembly function (kernel/arch/x86_64/ring3_enter.S)
    - **STATUS:** Needs implementation
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
    - **STATUS:** Needs implementation
    - Define `EMIT_MARKER_ASM(marker_string)` macro
    - Direct serial port write to 0xE9
    - **CRITICAL: Preserve all registers (pushfq/popfq)**
    - _Requirements: 12.1, 12.2, 12.4_

  - [ ] 2.3 Implement #BP exception handler for Ring3 marker
    - **STATUS:** Needs implementation
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
    - **STATUS:** Needs implementation
    - Check if process is PROC_TYPE_USER
    - Load process context and CR3
    - Call ring3_enter (never returns on first entry)
    - _Requirements: 5.1, 5.2, 5.3_

- [ ] 4. Create CI gate for CPL3 execution validation
  - [ ] 4.1 Write marker extraction script (tools/ci/extract_markers.py)
  - [ ] 4.2 Write marker order validation (tools/ci/validate_marker_order_phase10a2.py)
    - Expected order: KERNEL_BEFORE_RING3, [[AYKEN_RING3_PREP_OK]], P10_SCHED_ARMED, P10_TSS_OK, P10_CR3_SWITCH, P10_RING3_ENTER, P10_RING3_USER_CODE
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
    12. ✓ Marker order correct? (7 markers)
    13. ✓ No triple fault? (Check QEMU output)

---

### Phase 10-B: Full ELF Parsing (PARTIALLY COMPLETED) ✅

**Status:** Basic ELF parsing implemented, comprehensive error handling remains

**What's Implemented:**
- ELF64 header validation (magic, class, machine, type)
- Program header table bounds validation
- Segment file range bounds validation
- PT_LOAD iteration
- BSS zero-fill (p_memsz > p_filesz)
- Page flag derivation from p_flags
- Basic segment loading

**What Remains:**
- [ ] 11. Implement comprehensive error handling
  - [ ] 11.1 Cleanup tracking structure
  - [ ] 11.2 Reverse-order cleanup on error
  - [ ] 11.3 Memory leak prevention

- [ ] 12. Implement W^X enforcement validation
  - [ ] 12.1 Reject segments with both PF_W and PF_X
  - [ ] 12.2 Validate NX bit correctness

- [ ] 13. Implement segment overlap detection
  - [ ] 13.1 Check for overlapping virtual address ranges
  - [ ] 13.2 Check for kernel space overlap

- [ ] 14. Implement property-based tests
  - [ ] 14.1 Property tests for all 30 correctness properties
  - [ ] 14.2 Randomized ELF generation
  - [ ] 14.3 Comprehensive input coverage

- [ ] 15. Checkpoint - Phase 10-B validation
  - All property tests pass
  - Error handling verified
  - W^X enforcement validated

---

### Phase 10-C: Process Integration (MOSTLY COMPLETED) ✅

**Status:** Core integration complete, refinement remains

**What's Implemented:**
- PCB (proc_t) structure with full fields
- PID assignment and process table
- Scheduler integration via `sched_add()`
- Process state management
- Init process (PID1) creation
- Mailbox allocation for scheduler bridge
- RSP0 allocation for Ring3→Ring0 transitions

**What Remains:**
- [ ] 17. Refine context switch path
  - [ ] 17.1 Optimize kernel ↔ user transitions
  - [ ] 17.2 Validate context preservation

- [ ] 18. Optimize syscall entry path
  - [ ] 18.1 INT 0x80 handler optimization
  - [ ] 18.2 Register preservation validation

- [ ] 19. Multi-process support testing
  - [ ] 19.1 Create multiple Ring3 processes
  - [ ] 19.2 Validate scheduler fairness
  - [ ] 19.3 Validate context isolation

- [ ] 20. Final checkpoint - Phase 10-C validation
  - Multiple processes run correctly
  - Context switches work reliably
  - Syscalls work from all processes

---

## Implementation Notes

### Ring0 Export Policy

**CRITICAL:** ELF parser functions are PRIVATE (static, not exported to Ring0 surface)
- `elf64_validate_minimal()` - STATIC
- `elf64_get_entry()` - STATIC
- Only test entry points (`elf_parser_run_validation()`) are exported

This follows AykenOS constitutional principle: minimize Ring0 export surface.

### Marker Sequences

**Phase 10-A1 (COMPLETE):**
```
KERNEL_BEFORE_RING3 → [[AYKEN_RING3_PREP_OK]] → P10_SCHED_ARMED
```

**Phase 10-A2 (PLANNED):**
```
KERNEL_BEFORE_RING3 → [[AYKEN_RING3_PREP_OK]] → P10_SCHED_ARMED → 
P10_TSS_OK → P10_CR3_SWITCH → P10_RING3_ENTER → P10_RING3_USER_CODE
```

### Critical Dependencies

1. **TSS/GDT/IDT Configuration:** Must be validated before Ring3 entry
2. **RSP0 Mapping:** Kernel stack must be mapped in user CR3
3. **Mailbox Mapping:** Required for scheduler bridge communication
4. **Scheduler Dispatch:** Must call `ring3_enter()` for user processes

### Debug Strategy

Use 13-point checklist for Phase 10-A2 to prevent triple fault:
1. TSS loaded
2. RSP0 valid
3. IDT #BP present
4. User segments DPL=3
5. PML4 USER bit correct
6. CR3 switch correct
7. NX flag correct
8. RIP canonical
9. Stack mapped
10. Stack RSP valid
11. Guard page unmapped
12. Marker order correct
13. No triple fault

---

**Author:** Kenan AY  
**Last Updated:** 2026-02-28  
**Next Review:** After Phase 10-A2 implementation
