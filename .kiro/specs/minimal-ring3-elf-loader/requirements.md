# Requirements Document: Minimal Ring3 ELF Loader

**Author:** Kenan AY  
**Phase:** 10 (Ring3 Execution)  
**Status:** Draft  
**Version:** 1.0

## Introduction

This document specifies the requirements for implementing a minimal ELF loader in the AykenOS kernel that enables loading and executing Ring3 (userspace) programs from ELF binary format. This feature is foundational for userspace program execution and represents Phase 10 of AykenOS development.

The ELF loader will parse ELF64 binaries, map program segments into user address space, set up the execution environment, and transfer control to Ring3. This is a Ring0 mechanism-only component that provides the infrastructure for userspace execution without making policy decisions.

## Glossary

- **ELF**: Executable and Linkable Format, a standard binary format for executables
- **ELF_Loader**: The kernel component responsible for parsing and loading ELF binaries
- **Program_Header**: ELF metadata describing loadable segments (PT_LOAD)
- **User_Process**: A Ring3 process created from an ELF binary
- **User_Address_Space**: Virtual memory region (0x0000000000400000-0x00007FFFFFFFFFFF) for Ring3 programs (canonical lower half)
- **Kernel_Address_Space**: Virtual memory region (0xFFFF800000000000+) for Ring0 code (canonical upper half)
- **PML4**: Page Map Level 4, the root page table structure in x86_64
- **Ring0**: CPU privilege level 0 (kernel mode, supervisor)
- **Ring3**: CPU privilege level 3 (user mode, unprivileged)
- **Syscall_Interface**: The 1000-1010 syscall range for Ring3→Ring0 communication

## Requirements

### Requirement 1: ELF Binary Parsing

**User Story:** As a kernel developer, I want to parse ELF64 binaries, so that I can extract program segments and entry points for execution.

#### Acceptance Criteria

1. WHEN an ELF binary is provided, THE ELF_Loader SHALL validate the ELF magic number (0x7F 'E' 'L' 'F')
2. WHEN the ELF header is invalid, THE ELF_Loader SHALL return an error code without modifying system state
3. WHEN the ELF binary is valid, THE ELF_Loader SHALL extract the entry point address from e_entry field
4. WHEN program headers are present, THE ELF_Loader SHALL iterate through all PT_LOAD segments
5. WHEN a PT_LOAD segment is encountered, THE ELF_Loader SHALL extract p_vaddr, p_offset, p_filesz, and p_memsz fields

### Requirement 2: User Address Space Creation

**User Story:** As a kernel developer, I want to create isolated user address spaces, so that Ring3 programs execute in their own virtual memory context.

#### Acceptance Criteria

1. WHEN creating a user process, THE ELF_Loader SHALL allocate a new PML4 root page table
2. WHEN the PML4 is allocated, THE ELF_Loader SHALL copy kernel half mappings (PML4 entries 256-511) from kernel PML4
3. WHEN copying kernel mappings, THE ELF_Loader SHALL ensure kernel entries do NOT have USER bit set
4. WHEN copying kernel mappings, THE ELF_Loader SHALL preserve GLOBAL and NX bits as defined in kernel PML4
4. WHEN the user PML4 is created, THE ELF_Loader SHALL leave user half (PML4 entries 0-255) initially unmapped
5. WHEN PML4 allocation fails, THE ELF_Loader SHALL return an error without partial state
6. THE ELF_Loader SHALL store the PML4 physical address in the process control block

### Requirement 3: Program Segment Loading

**User Story:** As a kernel developer, I want to load ELF segments into user memory, so that the program code and data are accessible at runtime.

#### Acceptance Criteria

1. FOR ALL PT_LOAD segments, THE ELF_Loader SHALL allocate physical frames for the segment size (p_memsz)
2. WHEN allocating frames, THE ELF_Loader SHALL align allocations to 4KB page boundaries
3. WHEN a segment has file data (p_filesz > 0), THE ELF_Loader SHALL copy data from ELF binary at p_offset
4. WHEN p_memsz exceeds p_filesz, THE ELF_Loader SHALL zero-fill the remaining bytes (BSS section)
5. WHEN mapping segments, THE ELF_Loader SHALL use p_vaddr as the virtual address base
6. WHEN mapping user segments, THE ELF_Loader SHALL derive page flags from ELF segment flags (p_flags)
7. WHEN p_flags includes PF_W, THE ELF_Loader SHALL set WRITABLE flag
8. WHEN p_flags includes PF_X, THE ELF_Loader SHALL clear NX bit (executable)
9. WHEN p_flags excludes PF_X, THE ELF_Loader SHALL set NX bit (non-executable)
10. THE ELF_Loader SHALL always set USER and PRESENT flags for user segments
11. THE ELF_Loader SHALL reject segments with both PF_W and PF_X set (W^X enforcement)
12. WHEN segment loading fails, THE ELF_Loader SHALL deallocate all previously allocated frames and return an error

### Requirement 4: Execution Environment Setup

**User Story:** As a kernel developer, I want to set up the Ring3 execution environment, so that userspace programs can begin execution with proper CPU state.

#### Acceptance Criteria

1. WHEN preparing for Ring3 execution, THE ELF_Loader SHALL create a cpu_context_t structure with initial register state
2. WHEN setting the instruction pointer, THE ELF_Loader SHALL use the ELF entry point (e_entry) as RIP value
3. WHEN allocating the user stack, THE ELF_Loader SHALL allocate a guard page at 0x00007FFFFFFFD000 (unmapped)
4. WHEN allocating the user stack, THE ELF_Loader SHALL allocate a stack page at 0x00007FFFFFFFE000 with USER | WRITABLE | PRESENT | NX flags
5. WHEN setting RSP, THE ELF_Loader SHALL set it to 0x00007FFFFFFFFFF0 (16-byte aligned, top of stack)
6. WHEN setting segment selectors, THE ELF_Loader SHALL use Ring3 code segment (0x23) for CS and Ring3 data segment (0x1B) for DS/ES/SS
7. WHEN setting RFLAGS, THE ELF_Loader SHALL enable interrupts (IF=1), set IOPL=0, clear AC bit, and clear VM bit
8. WHEN setting RFLAGS, THE ELF_Loader SHALL validate that reserved bits are properly masked
9. THE ELF_Loader SHALL zero all general-purpose registers except RSP and RIP

### Requirement 5: Control Transfer to Ring3

**User Story:** As a kernel developer, I want to transfer control to Ring3, so that userspace programs execute at the correct privilege level.

#### Acceptance Criteria

1. WHEN loading user PML4, THE ELF_Loader SHALL load the user PML4 physical address into CR3 before Ring3 transition
2. WHEN loading CR3, THE ELF_Loader SHALL flush TLB (implicit via CR3 write)
3. WHEN transferring to Ring3, THE ELF_Loader SHALL use IRETQ instruction for privilege level transition
4. WHEN setting up IRETQ frame, THE ELF_Loader SHALL ensure RSP is 16-byte aligned before pushing frame
5. WHEN setting up IRETQ frame, THE ELF_Loader SHALL push SS, RSP, RFLAGS, CS, RIP in correct order on stack
6. WHEN IRETQ executes, THE ELF_Loader SHALL ensure CPU transitions from Ring0 (CPL=0) to Ring3 (CPL=3)
7. IF Ring3 entry fails, THEN THE ELF_Loader SHALL trigger a kernel panic with diagnostic information

### Requirement 6: Error Handling and Validation

**User Story:** As a kernel developer, I want robust error handling, so that invalid ELF binaries do not compromise system stability.

#### Acceptance Criteria

1. WHEN ELF validation fails, THE ELF_Loader SHALL return specific error codes (EINVAL, ENOMEM, ENOEXEC)
2. WHEN memory allocation fails, THE ELF_Loader SHALL maintain a list of allocated frames for cleanup
3. WHEN page mapping fails, THE ELF_Loader SHALL maintain a list of mapped pages for cleanup
4. WHEN cleanup is required, THE ELF_Loader SHALL execute cleanup in reverse allocation order
5. WHEN cleanup is required, THE ELF_Loader SHALL deallocate all tracked frames and unmap all tracked pages
5. WHEN segment addresses overlap kernel space (≥0xFFFF800000000000), THE ELF_Loader SHALL reject the binary
6. WHEN segment sizes exceed reasonable limits (>1GB per segment), THE ELF_Loader SHALL reject the binary
7. WHEN p_vaddr is not page-aligned, THE ELF_Loader SHALL align down to nearest 4KB boundary
8. THE ELF_Loader SHALL validate that all PT_LOAD segments fit within user address space (0x0000000000400000-0x00007FFFFFFFFFFF)
9. THE ELF_Loader SHALL validate that segment file range [p_offset, p_offset + p_filesz) fits within ELF blob size
10. THE ELF_Loader SHALL validate that program header table [e_phoff, e_phoff + e_phnum * e_phentsize) fits within ELF blob size

### Requirement 7: Integration with Process Management

**User Story:** As a kernel developer, I want the ELF loader to integrate with existing process structures, so that loaded programs are managed by the scheduler.

#### Acceptance Criteria

1. WHEN creating a user process, THE ELF_Loader SHALL allocate a process control block (PCB)
2. WHEN the PCB is created, THE ELF_Loader SHALL store the user PML4 address in the PCB
3. WHEN the PCB is created, THE ELF_Loader SHALL store the initial cpu_context_t in the PCB
4. WHEN the process is ready, THE ELF_Loader SHALL mark the process as RUNNABLE in the scheduler
5. THE ELF_Loader SHALL assign a unique process ID (PID) to the new process

### Requirement 8: Memory Management Integration

**User Story:** As a kernel developer, I want the ELF loader to use existing memory management primitives, so that memory allocation is consistent with kernel design.

#### Acceptance Criteria

1. WHEN allocating physical frames, THE ELF_Loader SHALL use the kernel physical memory allocator
2. WHEN creating page tables, THE ELF_Loader SHALL use the kernel page table management functions
3. WHEN mapping pages, THE ELF_Loader SHALL use the kernel virtual memory mapping interface
4. WHEN deallocating on error, THE ELF_Loader SHALL use the kernel deallocation functions
5. THE ELF_Loader SHALL NOT bypass kernel memory management primitives

### Requirement 9: Minimal ELF Support Scope

**User Story:** As a kernel developer, I want to support only essential ELF features, so that the loader remains minimal and maintainable.

#### Acceptance Criteria

1. THE ELF_Loader SHALL support ELF64 format only (64-bit binaries)
2. THE ELF_Loader SHALL support x86_64 architecture only (e_machine = EM_X86_64)
3. THE ELF_Loader SHALL support PT_LOAD segments only (ignore PT_DYNAMIC, PT_INTERP, etc.)
4. THE ELF_Loader SHALL support statically-linked binaries only (no dynamic linker support)
5. THE ELF_Loader SHALL NOT support ELF relocations or symbol resolution
6. THE ELF_Loader SHALL NOT support shared libraries or position-independent executables (PIE)

### Requirement 10: Testing and Validation

**User Story:** As a kernel developer, I want comprehensive testing, so that the ELF loader is reliable and correct.

#### Acceptance Criteria

1. WHEN testing, THE ELF_Loader SHALL correctly load a minimal "hello world" static binary
2. WHEN testing, THE ELF_Loader SHALL correctly reject invalid ELF magic numbers
3. WHEN testing, THE ELF_Loader SHALL correctly handle binaries with multiple PT_LOAD segments
4. WHEN testing, THE ELF_Loader SHALL correctly zero-fill BSS sections (p_memsz > p_filesz)
5. WHEN testing, THE ELF_Loader SHALL correctly reject binaries with kernel-space addresses
6. WHEN testing, THE ELF_Loader SHALL correctly clean up on allocation failures

### Requirement 11: Phased Implementation Strategy

**User Story:** As a kernel developer, I want a phased implementation approach, so that Ring3 execution is proven incrementally with minimal debugging complexity.

#### Acceptance Criteria

1. THE ELF_Loader implementation SHALL be divided into distinct phases (Phase 10-A, 10-B, 10-C)
2. WHEN implementing Phase 10-A, THE ELF_Loader SHALL support hardcoded ELF binary with single PT_LOAD segment
3. WHEN implementing Phase 10-A, THE ELF_Loader SHALL emit P10_RING3_ENTER marker upon successful Ring3 transition
4. WHEN implementing Phase 10-B, THE ELF_Loader SHALL support full PT_LOAD iteration and BSS zero-fill
5. WHEN implementing Phase 10-C, THE ELF_Loader SHALL integrate with PCB, scheduler, and PID assignment
6. THE ELF_Loader SHALL NOT attempt full implementation in a single phase

### Requirement 13: GDT Segment Selector Contract

**User Story:** As a kernel developer, I want explicit GDT segment selector definitions, so that Ring3 code and data segments are correctly configured.

#### Acceptance Criteria

1. THE Kernel GDT SHALL define user code selector 0x23 with DPL=3, present, executable, readable
2. THE Kernel GDT SHALL define user data selector 0x1B with DPL=3, present, writable, readable
3. THE ELF_Loader SHALL use selector 0x23 for Ring3 code segment (CS)
4. THE ELF_Loader SHALL use selector 0x1B for Ring3 data segments (SS, DS, ES)
5. THE GDT configuration SHALL be validated before Ring3 entry

### Requirement 14: RFLAGS Initialization Contract

**User Story:** As a kernel developer, I want deterministic RFLAGS initialization, so that Ring3 execution starts in a known CPU state.

#### Acceptance Criteria

1. WHEN initializing RFLAGS, THE ELF_Loader SHALL set it to 0x202 (IF=1, reserved bit 1 set, all other flags clear)
2. THE ELF_Loader SHALL NOT use input-derived RFLAGS values (deterministic initialization)
3. THE ELF_Loader SHALL ensure IOPL=0, VM=0, AC=0, NT=0, TF=0, DF=0 in initial RFLAGS

### Requirement 12: Runtime Marker Proof

**User Story:** As a kernel developer, I want runtime marker proof, so that Ring3 execution success is validated by CI gates.

#### Acceptance Criteria

1. WHEN Ring3 transition succeeds, THE ELF_Loader SHALL emit marker P10_RING3_ENTER (transition attempt proof)
2. WHEN CR3 is switched to user PML4, THE ELF_Loader SHALL emit marker P10_CR3_SWITCH
3. WHEN Ring3 user code begins execution, THE ELF_Loader SHALL ensure user code emits marker P10_RING3_USER_CODE (execution proof)
4. WHEN IRETQ executes successfully, THE ELF_Loader SHALL ensure marker order: KERNEL_BEFORE_RING3, P10_CR3_SWITCH, P10_RING3_ENTER, P10_RING3_USER_CODE
5. THE marker sequence SHALL be validated by CI gate gate_ring3_execution.sh
6. IF marker order is violated, THEN CI SHALL FAIL (fail-closed)
7. THE ELF_Loader SHALL emit markers using the existing marker infrastructure (serial output)