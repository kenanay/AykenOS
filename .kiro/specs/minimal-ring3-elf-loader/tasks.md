# Implementation Plan: Minimal Ring3 ELF Loader

**Author:** Kenan AY  
**Phase:** 10 (Ring3 Execution)  
**Status:** Draft  
**Version:** 1.0

## Overview

This implementation plan follows a strict phased approach to incrementally prove Ring3 execution capability in AykenOS. The plan is divided into three phases:

- **Phase 10-A**: Ring3 Entry Proof (minimal, hardcoded ELF, single PT_LOAD)
- **Phase 10-B**: Full ELF Parsing (multi-segment, BSS, error handling)
- **Phase 10-C**: Process Integration (PCB, scheduler, syscalls)

Each phase builds on the previous phase and includes runtime marker validation through CI gates. The phased approach minimizes debugging complexity and provides incremental proof of correctness.

## Tasks

### Phase 10-A: Ring3 Entry Proof (Minimal)

- [x] 1. Create minimal Ring3 userspace program
  - [x] 1.1 Create userspace/minimal directory structure
    - Create `userspace/minimal/` directory
    - _Requirements: 11.2_

  - [x] 1.2 Write minimal Ring3 program (minimal.c)
    - Write `userspace/minimal/minimal.c` with entry point at 0x400000
    - Execute INT3 (breakpoint) instruction to trigger #BP exception
    - Kernel #BP handler will emit P10_RING3_USER_CODE marker
    - No writable data, no BSS (single RX segment only)
    - No HLT instruction (privileged, would cause #GP)
    - No direct I/O (OUT 0xE9 would cause #GP with IOPL=0)
    - _Requirements: 12.3_
    - _Note: Exception-based marker is the only viable Ring3 proof method in Phase 10-A (no syscall, no I/O privilege)_

  - [x] 1.3 Create linker script for Ring3 binary (user.ld)
    - Write `userspace/minimal/user.ld` with entry at 0x400000
    - Single PT_LOAD segment (code + rodata combined, RX flags only)
    - No writable sections (.data, .bss) - Phase 10-A uses RX-only segment
    - Static linking (no dynamic sections)
    - _Requirements: 9.4, 9.6_

  - [x] 1.4 Create Makefile for minimal userspace program
    - Write `userspace/minimal/Makefile`
    - Build with `clang -target x86_64-elf -ffreestanding -nostdlib -static`
    - Link with user.ld linker script
    - Output: `minimal.elf`
    - _Requirements: 9.1, 9.2, 9.4_

  - [x] 1.5 Write unit test for minimal ELF structure
    - Verify ELF magic, class, machine, entry point
    - Verify single PT_LOAD segment with correct flags
    - _Requirements: 10.1_

- [x] 2. Create ELF embedding tool
  - [x] 2.1 Write Python script to embed ELF as C array (tools/embed_elf.py)
    - Read ELF binary file
    - Generate C header with `const uint8_t embedded_elf[]` array
    - Generate size constant `const size_t embedded_elf_size`
    - _Requirements: 11.2_

  - [x] 2.2 Integrate embed_elf.py into kernel build
    - Add Makefile rule to generate `kernel/include/embedded_elf.h`
    - Depend on `userspace/minimal/minimal.elf`
    - _Requirements: 11.2_

- [x] 3. Implement minimal ELF validation (Phase 10-A scope)
  - [x] 3.1 Create ELF header structures (kernel/include/elf/elf64.h)
    - Define `elf64_ehdr_t` structure
    - Define `elf64_phdr_t` structure
    - Define ELF constants (magic, class, machine, PT_LOAD, PF_*)
    - _Requirements: 1.1, 1.3, 1.4, 1.5_

  - [x] 3.2 Write minimal ELF validation function (kernel/src/elf/parser.c)
    - Implement `elf64_validate_minimal(blob, size)` for Phase 10-A
    - Validate magic number (0x7F 'E' 'L' 'F')
    - Validate class (64-bit), machine (x86_64), type (ET_EXEC)
    - **CRITICAL: Validate program header table bounds** (e_phoff + e_phnum * e_phentsize <= size)
    - **CRITICAL: Validate segment file range bounds** (p_offset + p_filesz <= size for all PT_LOAD)
    - Return -EINVAL on invalid, -ENOEXEC on unsupported
    - _Requirements: 1.1, 1.2, 6.9, 6.10, 9.1, 9.2_
    - _Note: Bounds checks are mandatory even in Phase 10-A for fail-fast security_

  - [x] 3.3 Write entry point extraction function
    - Implement `elf64_get_entry(blob)` to extract e_entry field
    - _Requirements: 1.3_

  - [x] 3.4 Write property test for ELF magic validation
    - **Property 1: ELF Magic Validation**
    - **Validates: Requirements 1.1**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

  - [x] 3.5 Write property test for entry point extraction
    - **Property 3: Entry Point Extraction**
    - **Validates: Requirements 1.3**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

- [ ] 4. Implement user address space creation
  - [x] 4.1 Create user address space structures (kernel/include/mm/user_as.h)
    - Define `user_as_t` structure (cr3_phys, pml4_virt)
    - Define `cleanup_tracker_t` structure (frames, vaddrs, counts)
    - _Requirements: 2.1, 6.2, 6.3_

  - [x] 4.2 Implement PML4 allocation and kernel half copy (kernel/mm/user_as.c)
    - Implement `user_as_create(out_as)`
    - Allocate new PML4 frame
    - Zero entire PML4 (all 512 entries)
    - Copy kernel half (entries 256-511) from kernel PML4
    - For each copied entry: clear USER bit explicitly (entry &= ~PAGE_USER)
    - Preserve GLOBAL and NX bits as-is
    - Store PML4 physical and virtual addresses in out_as
    - Return -ENOMEM on allocation failure
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ] 4.3 Write property test for kernel mapping copy correctness
    - **Property 6: Kernel Mapping Copy Correctness**
    - **Validates: Requirements 2.2, 2.3**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

  - [ ] 4.4 Write property test for kernel mapping security (no USER bit)
    - **Property 7: Kernel Mapping Security (No USER Bit)**
    - **Validates: Requirements 2.3**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

  - [ ] 4.5 Write property test for user half initially unmapped
    - **Property 9: User Half Initially Unmapped**
    - **Validates: Requirements 2.5**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

- [ ] 5. Implement minimal segment loading (Phase 10-A: single PT_LOAD only)
  - [ ] 5.1 Write page flag derivation function (kernel/src/mm/user_as.c)
    - Implement `derive_page_flags(elf_flags)` as inline function
    - Set PRESENT | USER always
    - Set WRITABLE if PF_W
    - **CRITICAL: NX bit has INVERSE logic on x86-64:**
      - NX bit = 1 → NOT executable (page is non-executable)
      - NX bit = 0 → executable (page is executable)
      - If PF_X set: NX = 0 (clear NX bit, page is executable)
      - If PF_X not set: NX = 1 (set NX bit, page is non-executable)
    - Return 0 if both PF_W and PF_X (W^X violation)
    - _Requirements: 3.6, 3.7, 3.8, 3.9, 3.10, 3.11_
    - _Note: NX bit confusion is a common source of #GP faults_

  - [ ] 5.2 Implement segment address validation (kernel/src/elf/loader.c)
    - Implement `validate_segment_range(vaddr, size)`
    - Check vaddr >= 0x400000 (user space start)
    - Check vaddr + size <= 0x00007FFFFFFFFFFF (user space end)
    - Check vaddr < 0xFFFF800000000000 (no kernel overlap)
    - Check size <= 1GB (segment size limit)
    - Check for overflow (vaddr + size)
    - Return 0 on valid, -EINVAL on invalid
    - _Requirements: 6.6, 6.7, 6.9_

  - [ ] 5.3 Implement minimal segment loader for Phase 10-A (single PT_LOAD)
    - Implement `load_segment_minimal(as, elf_blob, phdr, tracker)`
    - Validate segment range
    - Calculate page-aligned base and bias (seg_page_base, seg_page_bias)
    - Calculate page count from p_memsz + bias
    - For each page: allocate frame, copy data (accounting for bias), map to user address
    - Track allocations in cleanup_tracker
    - Return -ENOMEM on allocation failure, -EINVAL on invalid segment
    - _Requirements: 3.1, 3.2, 3.3, 3.5, 6.8_

  - [ ] 5.4 Write property test for page flag derivation
    - **Property 15: Page Flag Derivation from ELF Flags**
    - **Validates: Requirements 3.6, 3.7, 3.8, 3.9, 3.10**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

  - [ ] 5.5 Write property test for W^X enforcement
    - **Property 16: W^X Enforcement**
    - **Validates: Requirements 3.11**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

  - [ ] 5.6 Write property test for segment load bias correctness
    - **Property 14: Segment Load Bias Correctness**
    - **Validates: Requirements 3.5, 6.8**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

- [ ] 6. Implement execution environment setup
  - [ ] 6.1 Allocate user stack with guard page (kernel/src/mm/user_as.c)
    - Implement `user_as_alloc_stack(as, tracker)`
    - Allocate guard page at 0x00007FFFFFFFD000 (unmapped, no mapping)
    - Allocate stack page at 0x00007FFFFFFFE000 with USER | WRITABLE | PRESENT | NX
    - Track stack page in cleanup_tracker
    - Return -ENOMEM on allocation failure
    - _Requirements: 4.3, 4.4_

  - [ ] 6.2 Create initial CPU context (kernel/src/elf/loader.c)
    - Implement `create_initial_context(ctx, entry_point)`
    - Zero all general-purpose registers
    - Set RIP = entry_point (from e_entry)
    - Set RSP = 0x00007FFFFFFFFFF0 (16-byte aligned, top of stack)
    - Set CS = 0x23 (Ring3 code segment, hardcoded in Phase 10-A)
    - Set SS = DS = ES = 0x1B (Ring3 data segment, hardcoded in Phase 10-A)
    - Set RFLAGS = 0x202 (IF=1, reserved bit 1, deterministic)
    - _Requirements: 4.1, 4.2, 4.5, 4.6, 4.7, 4.8, 4.9, 13.3, 13.4, 14.1, 14.2, 14.3_

  - [ ] 6.3 Write property test for RIP initialization
    - **Property 18: RIP Initialization**
    - **Validates: Requirements 4.2**
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

  - [ ] 6.4 Write unit test for stack allocation
    - Verify guard page is unmapped
    - Verify stack page has correct flags (USER | WRITABLE | NX)
    - Verify RSP is 16-byte aligned
    - _Requirements: 4.3, 4.4, 4.5_
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

- [ ] 7. Implement Ring3 entry assembly (IRETQ)
  - [ ] 7.1 Validate TSS and RSP0 configuration (CRITICAL prerequisite)
    - Verify TSS structure is defined and initialized
    - Verify LTR (Load Task Register) has been called
    - Verify TSS.RSP0 points to valid kernel stack
    - **CRITICAL: Without proper TSS/RSP0, Ring3→Ring0 exception causes #DF → triple fault**
    - Add validation function: `validate_tss_for_ring3()`
    - Call before first Ring3 entry attempt
    - _Requirements: 5.1, 5.6_
    - _Note: This is a hidden dependency - Phase 10-A will fail silently without it_

  - [ ] 7.2 Write ring3_enter assembly function (kernel/src/arch/x86_64/ring3_enter.S)
    - Declare `ring3_enter(cpu_context_t *ctx, uint64_t user_cr3)` as noreturn
    - Save context pointer in RBX (callee-saved, non-volatile)
    - Load user CR3 into CR3 register (TLB flush implicit)
    - Emit P10_CR3_SWITCH marker (inline, direct serial write to 0xE9)
    - Load all GPRs from context (except RSP, RIP)
    - **CRITICAL: Ensure RSP is 16-byte aligned BEFORE pushing IRETQ frame**
    - Push IRETQ frame in CORRECT order (bottom to top): SS (0x1B), RSP, RFLAGS, CS (0x23), RIP
    - **Note: Stack grows down, push order is REVERSE of frame layout**
    - Emit P10_RING3_ENTER marker (inline, direct serial write)
    - Restore RBX from context
    - Execute IRETQ
    - Add UD2 after IRETQ (never reached)
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 12.1, 12.2, 12.4_

  - [ ] 7.3 Add marker emission macros for assembly (kernel/include/markers.h)
    - Define `EMIT_MARKER_ASM(marker_string)` macro
    - Direct serial port write to 0xE9 (no C function calls)
    - Preserve all registers (use stack if needed)
    - _Requirements: 12.1, 12.2, 12.4_

  - [ ] 7.4 Implement #BP exception handler for Ring3 marker (kernel/src/arch/x86_64/interrupts.c)
    - Extend existing #BP (INT3) handler to detect Ring3 origin
    - **CRITICAL: Comprehensive Ring3 detection (not just CS check):**
      - Check CPL: `(frame->cs & 0x3) == 0x3`
      - Check SS: `(frame->ss & 0x3) == 0x3` (user data segment)
      - Check RIP range: `frame->rip >= 0x400000 && frame->rip < 0x00007FFFFFFFFFFF` (user space)
      - All three MUST pass to confirm Ring3 origin
    - If Ring3: emit P10_RING3_USER_CODE marker
    - After marker: halt or panic (Phase 10-A proof complete)
    - If Ring0: handle as normal breakpoint (debug use)
    - _Requirements: 12.3_
    - _Note: Exception-based marker is the ONLY viable Ring3 proof in Phase 10-A_
    - _Critical: This is how Ring3 execution is proven without syscall or I/O privilege_

  - [ ] 7.4 Write unit test for IRETQ frame setup
    - Verify frame is correctly ordered (SS, RSP, RFLAGS, CS, RIP)
    - Verify RSP is 16-byte aligned before frame push
    - _Requirements: 5.4, 5.5_
    - _Note: Optional for Phase 10-A, recommended for Phase 10-B_

- [ ] 8. Implement Phase 10-A loader entry point
  - [ ] 8.1 Write elf_load_process_phase_a function (kernel/src/elf/loader_phase_a.c)
    - Validate ELF header (minimal validation for Phase 10-A)
    - Extract entry point
    - Parse first PT_LOAD segment only (ignore others)
    - Create user address space
    - Load single segment
    - Allocate stack with guard page
    - Create initial CPU context
    - Call ring3_enter (never returns)
    - On error: cleanup and return error code
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 11.2, 11.3_

  - [ ] 8.2 Integrate Phase 10-A loader into kernel boot (kernel/src/kmain.c)
    - Include embedded_elf.h
    - Call elf_load_process_phase_a(embedded_elf, embedded_elf_size)
    - Emit KERNEL_BEFORE_RING3 marker before call
    - Handle error return (should not happen with valid embedded ELF)
    - _Requirements: 11.3, 12.4_

  - [ ]* 8.3 Write unit test for Phase 10-A loader
    - Test with minimal hardcoded ELF
    - Verify successful load (no error)
    - Verify context is correctly initialized
    - _Requirements: 10.1, 11.2_
    - _Note: Optional for Phase 10-A, focus on CI gate marker validation_

- [ ] 9. Create CI gate for Ring3 execution validation
  - [ ] 9.1 Write marker extraction script (tools/ci/extract_markers.py)
    - Parse QEMU serial output
    - Extract marker strings
    - Output one marker per line
    - _Requirements: 12.4, 12.5_

  - [ ] 9.2 Write marker order validation script (tools/ci/validate_marker_order.py)
    - Read markers from stdin or file
    - Validate expected order: KERNEL_BEFORE_RING3, P10_CR3_SWITCH, P10_RING3_ENTER, P10_RING3_USER_CODE
    - Exit 0 on success, exit 1 on failure
    - _Requirements: 12.4, 12.5_

  - [ ] 9.3 Write CI gate script (scripts/ci/gate_ring3_execution.sh)
    - Build kernel with KERNEL_PROFILE=validation
    - Run QEMU with serial output capture (timeout 10s)
    - Extract markers using extract_markers.py
    - Validate marker order using validate_marker_order.py
    - Exit 0 on pass, exit 1 on fail
    - _Requirements: 12.4, 12.5, 12.6_

  - [ ] 9.4 Integrate gate into CI workflow (.github/workflows/ci-freeze.yml)
    - Add gate_ring3_execution.sh to CI pipeline
    - Run after ci-gate-boundary
    - Fail-closed: CI fails if gate fails
    - _Requirements: 12.6_

- [ ] 10. Checkpoint - Phase 10-A validation
  - Ensure all Phase 10-A mandatory tests pass (bounds checks, marker order)
  - Ensure CI gate passes (markers in correct order: KERNEL_BEFORE_RING3, P10_CR3_SWITCH, P10_RING3_ENTER, P10_RING3_USER_CODE)
  - Verify Ring3 code executes (P10_RING3_USER_CODE marker emitted via #BP handler)
  - Verify no #GP faults (IOPL, HLT, segment violations)
  - Optional property tests can be deferred to Phase 10-B
  - **CRITICAL: Phase 10-A Ultra-Minimal Execution Checklist (12 points):**
    1. ✓ TSS loaded? (LTR called, TSS descriptor valid)
    2. ✓ RSP0 valid? (TSS.RSP0 points to kernel stack)
    3. ✓ IDT #BP present? (Entry 3, DPL=3, present bit set)
    4. ✓ User segment DPL=3? (GDT entries 3 and 4, CS=0x23, SS=0x1B)
    5. ✓ PML4 USER bit correct? (Entries 0-255 can have USER, 256-511 must NOT)
    6. ✓ CR3 switch correct? (User PML4 physical address loaded)
    7. ✓ NX flag correct? (NX=0 for executable, NX=1 for non-executable)
    8. ✓ RIP canonical? (0x400000 in canonical lower half)
    9. ✓ Stack mapped? (0x00007FFFFFFFE000 with USER|WRITABLE|NX)
    10. ✓ Guard page unmapped? (0x00007FFFFFFFD000 not present)
    11. ✓ Marker order correct? (KERNEL→CR3→RING3_ENTER→RING3_USER_CODE)
    12. ✓ No triple fault? (Check QEMU output, no reset loop)
  - **If any checklist item fails, Phase 10-A will fail silently or triple fault**
  - Ask user if questions arise before proceeding to Phase 10-B

### Phase 10-B: Full ELF Parsing

- [ ] 11. Implement full ELF validation
  - [ ] 11.1 Extend ELF validation to full spec (kernel/src/elf/parser.c)
    - Implement `elf64_validate(blob, size)` (full validation)
    - Validate program header table bounds (e_phoff + e_phnum * e_phentsize <= size)
    - Validate all PT_LOAD segment file ranges (p_offset + p_filesz <= size)
    - Return specific error codes (EINVAL, ENOEXEC)
    - _Requirements: 1.1, 1.2, 6.9, 6.10_

  - [ ] 11.2 Write property test for program header table bounds validation
    - **Property 29: Program Header Table Bounds Validation**
    - **Validates: Requirements 6.10**

  - [ ] 11.3 Write property test for segment file range validation
    - **Property 28: Segment File Range Validation**
    - **Validates: Requirements 6.9**

- [ ] 12. Implement PT_LOAD iteration
  - [ ] 12.1 Write PT_LOAD iterator (kernel/src/elf/parser.c)
    - Implement `elf64_iter_phdrs(blob, callback, ctx)`
    - Iterate through all program headers
    - Call callback for each PT_LOAD segment
    - Skip non-PT_LOAD segments
    - Return 0 on success, callback return value on early stop
    - _Requirements: 1.4, 9.3_

  - [ ] 12.2 Write property test for PT_LOAD iteration completeness
    - **Property 4: PT_LOAD Iteration Completeness**
    - **Validates: Requirements 1.4**

  - [ ] 12.3 Write property test for program header field extraction
    - **Property 5: Program Header Field Extraction**
    - **Validates: Requirements 1.5**

- [ ] 13. Implement full segment loading with BSS support
  - [ ] 13.1 Extend segment loader to support BSS (kernel/src/elf/loader.c)
    - Implement `load_segment(as, elf_blob, phdr, tracker)` (full version)
    - Handle p_memsz > p_filesz (BSS section)
    - Zero-fill BSS region (p_filesz to p_memsz)
    - Handle edge case: p_filesz <= seg_page_bias (first page is all BSS)
    - Track all allocations for cleanup
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ] 13.2 Write property test for data copy correctness
    - **Property 11: Data Copy Correctness**
    - **Validates: Requirements 3.3**

  - [ ] 13.3 Write property test for BSS zero-fill
    - **Property 12: BSS Zero-Fill**
    - **Validates: Requirements 3.4**

  - [ ] 13.4 Write unit test for BSS-only segment
    - Test segment where p_filesz = 0, p_memsz > 0
    - Verify entire segment is zeroed
    - _Requirements: 3.4, 10.4_

- [ ] 14. Implement comprehensive error handling and cleanup
  - [ ] 14.1 Implement cleanup tracker (kernel/src/mm/user_as.c)
    - Implement `cleanup_tracker_init(tracker)`
    - Implement `cleanup_tracker_add_frame(tracker, phys_addr)`
    - Implement `cleanup_tracker_add_vaddr(tracker, vaddr)`
    - Implement `user_as_cleanup(as, tracker)` (reverse order)
    - Deallocate all tracked frames
    - Unmap all tracked pages
    - Free tracker arrays
    - _Requirements: 6.2, 6.3, 6.4, 6.5_

  - [ ] 14.2 Add error handling to all allocation points
    - Check all allocation return values
    - Call user_as_cleanup on any failure
    - Return specific error codes (ENOMEM, EINVAL)
    - Ensure no partial state on error
    - _Requirements: 1.2, 2.6, 3.12, 6.1, 6.2_

  - [ ] 14.3 Write property test for cleanup completeness and reverse order
    - **Property 20: Cleanup Completeness and Reverse Order**
    - **Validates: Requirements 6.4, 6.5**

  - [ ] 14.4 Write unit test for cleanup on allocation failure
    - Simulate allocation failure mid-load
    - Verify all allocations are cleaned up
    - Verify no memory leaks
    - _Requirements: 10.6_

- [ ] 15. Implement full ELF loader entry point
  - [ ] 15.1 Write elf_load_process function (kernel/src/elf/loader.c)
    - Validate ELF header (full validation)
    - Extract entry point
    - Create user address space
    - Iterate all PT_LOAD segments and load each
    - Allocate stack with guard page
    - Create initial CPU context
    - Call ring3_enter (never returns)
    - On error: cleanup and return error code
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 11.4_

  - [ ] 15.2 Write unit test for multi-segment ELF loading
    - Test ELF with 3 PT_LOAD segments (code, rodata, data)
    - Verify all segments loaded with correct flags
    - _Requirements: 10.3_

  - [ ] 15.3 Write unit test for kernel address rejection
    - Test ELF with segment at kernel address (>= 0xFFFF800000000000)
    - Verify loader rejects with -EINVAL
    - Verify no partial state
    - _Requirements: 10.5_

- [ ] 16. Checkpoint - Phase 10-B validation
  - Ensure all Phase 10-B tests pass
  - Ensure property tests pass (100+ iterations each)
  - Verify multi-segment ELF loading works
  - Verify BSS zero-fill works
  - Verify error handling and cleanup work
  - Ask user if questions arise before proceeding to Phase 10-C

### Phase 10-C: Process Integration

- [ ] 17. Implement process control block (PCB)
  - [ ] 17.1 Create PCB structures (kernel/include/proc/process.h)
    - Define `pcb_t` structure (pid, state, address_space, context)
    - Define process states (RUNNABLE, RUNNING, BLOCKED, etc.)
    - _Requirements: 7.1, 7.2, 7.3_

  - [ ] 17.2 Implement PCB allocation (kernel/src/proc/process.c)
    - Implement `pcb_alloc()` to allocate and initialize PCB
    - Zero-initialize all fields
    - Return NULL on allocation failure
    - _Requirements: 7.1_

  - [ ] 17.3 Implement PID assignment (kernel/src/proc/process.c)
    - Implement `pcb_assign_pid(pcb)` to assign unique PID
    - Use atomic counter or PID allocator
    - _Requirements: 7.5_

  - [ ] 17.4 Write unit test for PCB allocation and PID assignment
    - Verify PCB is properly initialized
    - Verify PIDs are unique
    - _Requirements: 7.1, 7.5_

- [ ] 18. Integrate ELF loader with PCB
  - [ ] 18.1 Modify elf_load_process to create PCB (kernel/src/elf/loader.c)
    - Allocate PCB at start of loading
    - Store user address space in PCB
    - Store initial CPU context in PCB
    - Assign PID to PCB
    - Return PCB pointer on success
    - Cleanup PCB on error
    - _Requirements: 7.1, 7.2, 7.3, 7.5_

  - [ ] 18.2 Write unit test for PCB integration
    - Verify PCB is created during ELF load
    - Verify address space and context are stored in PCB
    - Verify PID is assigned
    - _Requirements: 7.1, 7.2, 7.3, 7.5_

- [ ] 19. Integrate with scheduler
  - [ ] 19.1 Implement scheduler enqueue function (kernel/src/sched/user_sched.c)
    - Implement `pcb_mark_runnable(pcb)` to enqueue process in scheduler
    - Add PCB to runnable queue
    - Set process state to RUNNABLE
    - _Requirements: 7.4_

  - [ ] 19.2 Modify elf_load_process to enqueue process (kernel/src/elf/loader.c)
    - Call pcb_mark_runnable after successful load
    - Do NOT call ring3_enter directly (scheduler will dispatch)
    - Return PCB pointer to caller
    - _Requirements: 7.4_

  - [ ] 19.3 Write integration test for scheduler integration
    - Load process via elf_load_process
    - Verify process is in runnable queue
    - Verify process state is RUNNABLE
    - _Requirements: 7.4_

- [ ] 20. Implement context switch path (kernel ↔ user)
  - [ ] 20.1 Implement user context switch (kernel/src/sched/context_switch.c)
    - Extend existing context_switch to support user processes
    - Load user CR3 when switching to user process
    - Save/restore user context
    - Use ring3_enter for initial entry to user process
    - _Requirements: 5.1, 5.2, 5.3_

  - [ ] 20.2 Write integration test for context switch
    - Create two user processes
    - Trigger context switch
    - Verify both processes execute
    - _Requirements: 5.1, 5.2, 5.3_

- [ ] 21. Implement syscall entry path (Ring3 → Ring0)
  - [ ] 21.1 Write syscall entry assembly (kernel/src/sys/syscall_entry.S)
    - Save user context on kernel stack
    - Switch to kernel CR3
    - Call syscall dispatcher
    - Restore user context
    - Return to Ring3 via IRETQ or SYSRET
    - _Requirements: Syscall interface (1000-1010)_

  - [ ] 21.2 Implement minimal syscall (sys_exit)
    - Implement `sys_v2_exit(int status)` (syscall 1000)
    - Mark process as terminated
    - Trigger scheduler
    - _Requirements: Syscall interface (1000-1010)_

  - [ ] 21.3 Write integration test for syscall
    - Load user process that calls sys_exit
    - Verify syscall executes
    - Verify process terminates
    - Emit P10_SYSCALL_HELLO marker for CI validation
    - _Requirements: Syscall interface (1000-1010)_

- [ ] 22. Final checkpoint - Phase 10-C validation
  - Ensure all Phase 10-C tests pass
  - Ensure PCB integration works
  - Ensure scheduler integration works
  - Ensure context switch works
  - Ensure syscall works
  - Verify CI gates pass
  - Ask user if questions arise

## Notes

- Tasks marked with `*` are optional property-based tests and can be deferred to Phase 10-B for faster Phase 10-A MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation at phase boundaries
- **Phase 10-A focuses on Ring3 transition proof with minimal complexity:**
  - Exception-based marker (INT3 → #BP handler) is the ONLY viable Ring3 proof method
  - No syscall infrastructure (deferred to Phase 10-C)
  - No I/O privilege (IOPL=0, OUT instruction would cause #GP)
  - No HLT instruction (privileged, would cause #GP)
  - Single RX segment (no writable data/BSS)
  - Mandatory bounds checks (phdr table, segment file range) for fail-fast security
  - **CRITICAL: TSS/RSP0 must be configured before Ring3 entry (hidden dependency)**
  - **CRITICAL: #BP handler must validate CPL, SS, and RIP range (not just CS)**
  - **CRITICAL: NX bit has inverse logic (NX=1 means NOT executable)**
  - **CRITICAL: IRETQ frame push order is reverse of layout (stack grows down)**
- **Phase 10-A Ultra-Minimal Execution Checklist (12 points) - see task 10**
  - Use this checklist to debug triple faults and silent failures
  - Each item is a common failure point that causes #GP or #DF
  - Validating all 12 points reduces debug time by ~70%
- Phase 10-B adds full ELF parsing, multi-segment support, BSS, and comprehensive error handling
- Phase 10-C integrates with process management and scheduler
- Property tests validate universal correctness properties (minimum 100 iterations) - recommended for Phase 10-B
- Unit tests validate specific examples and edge cases
- CI gates provide runtime validation through marker sequences
