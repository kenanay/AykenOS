# Design Document: Minimal Ring3 ELF Loader

**Author:** Kenan AY  
**Phase:** 10 (Ring3 Execution)  
**Status:** Draft  
**Version:** 1.0

## Overview

This document specifies the design for a minimal ELF64 loader in the AykenOS kernel that enables loading and executing Ring3 (userspace) programs. The loader is a Ring0 mechanism-only component that parses ELF binaries, creates isolated user address spaces, maps program segments, and transfers control to Ring3.

The design follows AykenOS constitutional principles:
- Ring0 provides mechanisms only (no policy decisions)
- Phased implementation (10-A, 10-B, 10-C) for incremental proof
- Runtime marker validation for CI gates
- W^X enforcement and security-first design
- Deterministic, reproducible behavior

## Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    ELF Loader Entry Point                    │
│                  elf_load_process(blob, size)                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Phase 1: ELF Validation                    │
│  - Validate ELF magic (0x7F 'E' 'L' 'F')                    │
│  - Check ELF class (64-bit), endianness, machine (x86_64)  │
│  - Extract entry point (e_entry)                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Phase 2: User Address Space Creation            │
│  - Allocate new PML4 root page table                        │
│  - Copy kernel half (entries 256-511) from kernel PML4      │
│  - Preserve GLOBAL/NX bits, ensure no USER bit on kernel    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│               Phase 3: Program Segment Loading               │
│  - Iterate PT_LOAD segments                                 │
│  - Allocate physical frames (4KB aligned)                   │
│  - Copy p_filesz bytes from ELF                             │
│  - Zero-fill BSS (p_memsz - p_filesz)                       │
│  - Map to user virtual address (p_vaddr)                    │
│  - Derive flags from p_flags (W^X enforcement)              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│            Phase 4: Execution Environment Setup              │
│  - Allocate guard page (0x00007FFFFFFFD000, unmapped)       │
│  - Allocate stack page (0x00007FFFFFFFE000, RW|USER|NX)     │
│  - Create cpu_context_t with initial state                  │
│  - Set RIP = e_entry, RSP = 0x00007FFFFFFFFFF0              │
│  - Set CS=0x23 (Ring3 code), SS=0x1B (Ring3 data)           │
│  - Set RFLAGS (IF=1, IOPL=0, AC=0, VM=0)                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Phase 5: Control Transfer to Ring3              │
│  - Load user PML4 into CR3 (TLB flush implicit)             │
│  - Emit marker: P10_CR3_SWITCH (after CR3 write)            │
│  - Prepare IRETQ frame (SS, RSP, RFLAGS, CS, RIP)           │
│  - Execute IRETQ (Ring0 → Ring3 transition)                 │
│  - Emit marker: P10_RING3_ENTER                             │
└─────────────────────────────────────────────────────────────┘
```

### Address Space Layout

```
Canonical Lower Half (User Space):
┌─────────────────────────────────────────────────────────────┐
│ 0x0000000000000000 - 0x00000000003FFFFF : Reserved          │
│ 0x0000000000400000 - 0x00007FFFFFFFFFFF : User Space        │
│   ├─ 0x0000000000400000 : Program base (typical)           │
│   ├─ ...                : Program segments (code, data)     │
│   ├─ 0x00007FFFFFFFD000 : Stack guard page (unmapped)      │
│   └─ 0x00007FFFFFFFE000 : Stack page (RW|USER|NX)          │
│       RSP = 0x00007FFFFFFFFFF0 (16-byte aligned)            │
└─────────────────────────────────────────────────────────────┘

Non-Canonical Gap:
┌─────────────────────────────────────────────────────────────┐
│ 0x0000800000000000 - 0xFFFF7FFFFFFFFFFF : Invalid          │
└─────────────────────────────────────────────────────────────┘

Canonical Upper Half (Kernel Space):
┌─────────────────────────────────────────────────────────────┐
│ 0xFFFF800000000000 - 0xFFFFFFFFFFFFFFFF : Kernel Space     │
│   ├─ Kernel code, data, heap                                │
│   └─ Kernel mappings (copied to user PML4, no USER bit)    │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. ELF Parser (`kernel/include/elf/elf64.h`, `kernel/src/elf/parser.c`)

**Purpose:** Parse and validate ELF64 binary format.

**Data Structures:**

```c
// ELF64 Header (standard)
typedef struct {
    uint8_t  e_ident[16];     // Magic, class, endian, version
    uint16_t e_type;          // Object file type (ET_EXEC)
    uint16_t e_machine;       // Architecture (EM_X86_64)
    uint32_t e_version;       // ELF version
    uint64_t e_entry;         // Entry point address
    uint64_t e_phoff;         // Program header offset
    uint64_t e_shoff;         // Section header offset (unused)
    uint32_t e_flags;         // Processor-specific flags
    uint16_t e_ehsize;        // ELF header size
    uint16_t e_phentsize;     // Program header entry size
    uint16_t e_phnum;         // Program header count
    uint16_t e_shentsize;     // Section header entry size (unused)
    uint16_t e_shnum;         // Section header count (unused)
    uint16_t e_shstrndx;      // Section name string table index (unused)
} elf64_ehdr_t;

// ELF64 Program Header
typedef struct {
    uint32_t p_type;          // Segment type (PT_LOAD)
    uint32_t p_flags;         // Segment flags (PF_R, PF_W, PF_X)
    uint64_t p_offset;        // Offset in file
    uint64_t p_vaddr;         // Virtual address
    uint64_t p_paddr;         // Physical address (unused)
    uint64_t p_filesz;        // Size in file
    uint64_t p_memsz;         // Size in memory
    uint64_t p_align;         // Alignment
} elf64_phdr_t;

// ELF Constants
#define ELF_MAGIC_0     0x7F
#define ELF_MAGIC_1     'E'
#define ELF_MAGIC_2     'L'
#define ELF_MAGIC_3     'F'
#define ELF_CLASS_64    2
#define ELF_DATA_LSB    1
#define ET_EXEC         2
#define EM_X86_64       62
#define PT_LOAD         1
#define PF_X            0x1
#define PF_W            0x2
#define PF_R            0x4
```

**Functions:**

```c
// Validate ELF header
// Returns: 0 on success, -EINVAL on invalid ELF, -ENOEXEC on unsupported format
int elf64_validate(const uint8_t *blob, size_t size);

// Get entry point from ELF
uint64_t elf64_get_entry(const uint8_t *blob);

// Iterate program headers
// Callback returns 0 to continue, non-zero to stop
typedef int (*elf64_phdr_cb_t)(const elf64_phdr_t *phdr, void *ctx);
int elf64_iter_phdrs(const uint8_t *blob, elf64_phdr_cb_t cb, void *ctx);
```

### 2. User Address Space Manager (`kernel/src/mm/user_as.c`)

**Purpose:** Create and manage user address spaces (PML4 + page tables).

**Data Structures:**

```c
// User address space descriptor
typedef struct {
    uint64_t cr3_phys;        // Physical address of PML4
    uint64_t *pml4_virt;      // Virtual address of PML4 (for kernel access)
} user_as_t;

// Cleanup tracking
typedef struct {
    uint64_t *frames;         // Array of allocated physical frames
    size_t frame_count;       // Number of allocated frames
    uint64_t *vaddrs;         // Array of mapped virtual addresses
    size_t vaddr_count;       // Number of mapped pages
} cleanup_tracker_t;
```

**Functions:**

```c
// Create new user address space
// Allocates PML4, copies kernel half (entries 256-511)
// Ensures kernel entries do NOT have USER bit set (explicit clear)
// Returns: 0 on success, -ENOMEM on allocation failure
int user_as_create(user_as_t *out_as);

// Implementation notes for user_as_create:
// 1. Allocate new PML4 frame
// 2. Zero entire PML4 (all 512 entries)
// 3. Copy kernel half (entries 256-511) from kernel PML4
// 4. For each copied entry (256-511):
//    - If entry is present: entry &= ~PAGE_USER (clear USER bit)
//    - Preserve GLOBAL and NX bits as-is
// 5. Store PML4 physical address in out_as->cr3_phys
// 6. Store PML4 virtual address in out_as->pml4_virt
// This ensures "trust no upstream state" principle for security

// Map physical frame to user virtual address
// Derives page flags from ELF segment flags (p_flags)
// Enforces W^X: rejects PF_W + PF_X combination
// Returns: 0 on success, -EINVAL on invalid flags, -ENOMEM on allocation failure
int user_as_map(user_as_t *as, uint64_t vaddr, uint64_t phys, uint32_t elf_flags,
                cleanup_tracker_t *tracker);

// Cleanup on error (reverse allocation order)
void user_as_cleanup(user_as_t *as, cleanup_tracker_t *tracker);

// Destroy user address space
void user_as_destroy(user_as_t *as);
```

**Page Flag Derivation:**

```c
// Derive x86_64 page flags from ELF segment flags
// Returns 0 on invalid flag combination (caller must check and return -EINVAL)
static inline uint64_t derive_page_flags(uint32_t elf_flags) {
    uint64_t flags = PAGE_PRESENT | PAGE_USER;
    
    // Writable if PF_W set
    if (elf_flags & PF_W) {
        flags |= PAGE_WRITABLE;
    }
    
    // CRITICAL: NX bit has INVERSE logic on x86-64
    // NX bit = 1 → NOT executable (page is non-executable)
    // NX bit = 0 → executable (page is executable)
    // 
    // Executable if PF_X set (clear NX bit → NX = 0)
    // Non-executable if PF_X not set (set NX bit → NX = 1)
    if (!(elf_flags & PF_X)) {
        flags |= PAGE_NX;  // Set NX bit (page is non-executable)
    }
    // else: NX bit remains 0 (page is executable)
    
    // W^X enforcement: reject if both W and X
    if ((elf_flags & PF_W) && (elf_flags & PF_X)) {
        return 0; // Invalid combination - caller must check and return error
    }
    
    return flags;
}

// Caller usage:
// uint64_t flags = derive_page_flags(phdr->p_flags);
// if (flags == 0) {
//     return -EINVAL; // W^X violation
// }
```

### 3. Segment Loader (`kernel/src/elf/loader.c`)

**Purpose:** Load ELF segments into user address space.

**Functions:**

```c
// Load single PT_LOAD segment
// Allocates frames, copies data, zero-fills BSS, maps to user address
// Returns: 0 on success, -ENOMEM on allocation failure, -EINVAL on invalid segment
int load_segment(user_as_t *as, const uint8_t *elf_blob, const elf64_phdr_t *phdr,
                 cleanup_tracker_t *tracker);

// Validate segment address range
// Ensures segment fits in user space (0x400000 - 0x00007FFFFFFFFFFF)
// Rejects kernel space overlap (>= 0xFFFF800000000000)
// Rejects oversized segments (> 1GB)
// Returns: 0 on valid, -EINVAL on invalid
int validate_segment_range(uint64_t vaddr, uint64_t size);
```

**Segment Loading Algorithm:**

```
1. Validate segment range (user space only, < 1GB)
2. Calculate page-aligned base and bias:
   seg_page_base = p_vaddr & ~0xFFF  (align down to 4KB)
   seg_page_bias = p_vaddr & 0xFFF   (offset within first page)
3. Calculate page count: 
   total_size = seg_page_bias + p_memsz
   page_count = (total_size + 4095) / 4096
4. Edge case: If p_filesz <= seg_page_bias, first page contains only zero-filled data
5. For each page (i = 0 to page_count - 1):
   a. Allocate physical frame
   b. Track frame in cleanup_tracker
   c. Calculate copy parameters for this page:
      - If i == 0 (first page):
          dst_offset = seg_page_bias
          src_offset = 0
          available_space = PAGE_SIZE - seg_page_bias
      - Else (subsequent pages):
          dst_offset = 0
          src_offset = (i * PAGE_SIZE) - seg_page_bias
          available_space = PAGE_SIZE
   d. If within p_filesz range:
      - bytes_to_copy = min(available_space, p_filesz - src_offset)
      - Copy data from ELF blob at (p_offset + src_offset) to (frame + dst_offset)
      - If bytes_to_copy < available_space: zero-fill remainder
   e. If beyond p_filesz (BSS):
      - Zero-fill entire available_space starting at dst_offset
   f. Map frame to virtual address (seg_page_base + i * PAGE_SIZE)
   g. Track mapping in cleanup_tracker
6. On error: call user_as_cleanup() in reverse order
```

**Critical:** The bias calculation ensures that unaligned p_vaddr values are correctly handled. The first page may have a non-zero offset, and subsequent pages account for this bias when calculating source offsets.

### 4. Ring3 Entry (`kernel/src/arch/x86_64/ring3_enter.S`)

**Purpose:** Transfer control from Ring0 to Ring3 using IRETQ.

**CRITICAL PREREQUISITE: TSS and RSP0 Configuration**

Ring3→Ring0 exception handling (including INT3 for marker proof) requires proper TSS configuration:
- TSS structure must be defined and initialized
- LTR (Load Task Register) must have been called
- TSS.RSP0 must point to a valid kernel stack
- **Without proper TSS/RSP0, any Ring3 exception causes #DF → triple fault**

This is a hidden dependency that Phase 10-A MUST validate before first Ring3 entry.

**Assembly Interface:**

```asm
; ring3_enter(cpu_context_t *ctx, uint64_t user_cr3)
; Never returns (transitions to Ring3)
; Declared as __attribute__((noreturn)) in C
; Note: Phase 10-A ignores context segment selector fields (CS, SS) and uses fixed 0x23/0x1B
global ring3_enter
ring3_enter:
    ; Save context pointer in non-volatile register (callee-saved)
    ; Note: This function never returns, so RBX restore is not needed
    mov rbx, rdi
    
    ; Load user CR3 (TLB flush implicit, PCID disabled in Phase 10-A)
    ; AykenOS Phase 10-A assumes PCID disabled; CR3 write performs full TLB flush
    mov rax, rsi
    mov cr3, rax
    
    ; Emit marker: P10_CR3_SWITCH (inline, after CR3 write)
    ; Direct serial port write to 0xE9 (no C call to preserve stack alignment)
    ; (marker implementation: see marker macro)
    
    ; Load context registers (use rbx as base pointer)
    mov rax, [rbx + CTX_RAX]
    mov rcx, [rbx + CTX_RCX]
    mov rdx, [rbx + CTX_RDX]
    mov rsi, [rbx + CTX_RSI]
    mov rdi, [rbx + CTX_RDI]
    mov rbp, [rbx + CTX_RBP]
    mov r8,  [rbx + CTX_R8]
    mov r9,  [rbx + CTX_R9]
    mov r10, [rbx + CTX_R10]
    mov r11, [rbx + CTX_R11]
    mov r12, [rbx + CTX_R12]
    mov r13, [rbx + CTX_R13]
    mov r14, [rbx + CTX_R14]
    mov r15, [rbx + CTX_R15]
    
    ; Prepare IRETQ frame
    ; Current RSP must be 16-byte aligned
    ; IRETQ frame is 5 qwords (40 bytes)
    ; After 5 pushes, RSP will be (RSP - 40)
    ; To maintain 16-byte alignment after frame: RSP must be (16n + 8) before pushes
    ; Adjust RSP if needed (padding remains on kernel stack, not transferred to Ring3)
    mov r11, rsp
    and r11, 0xF
    cmp r11, 8
    je .aligned
    sub rsp, 8          ; Add padding if not at (16n + 8)
.aligned:
    
    ; Push IRETQ frame (bottom to top):
    push qword 0x1B                ; SS (Ring3 data segment)
    push qword [rbx + CTX_RSP]     ; RSP (user stack pointer)
    push qword [rbx + CTX_RFLAGS]  ; RFLAGS
    push qword 0x23                ; CS (Ring3 code segment)
    push qword [rbx + CTX_RIP]     ; RIP (entry point)
    
    ; Emit marker: P10_RING3_ENTER (inline, before IRETQ)
    ; Direct serial port write to 0xE9 (no C call)
    ; Note: This marker indicates transition attempt, not execution proof
    ; For execution proof, Ring3 code should emit P10_RING3_USER_CODE marker
    ; (marker implementation: see marker macro)
    
    ; Restore rbx (will be overwritten by context value)
    mov rbx, [rbx + CTX_RBX]
    
    ; Execute IRETQ (Ring0 → Ring3 transition)
    iretq
    
    ; Never reached (noreturn function)
    ud2
```

### 5. Process Control Block (Phase 10-C)

**Purpose:** Integrate ELF loader with process management and scheduler.

**Data Structures:**

```c
// Process Control Block (minimal for Phase 10-C)
typedef struct {
    uint32_t pid;             // Process ID
    uint32_t state;           // RUNNABLE, RUNNING, BLOCKED, etc.
    user_as_t address_space;  // User address space (CR3)
    cpu_context_t context;    // Saved CPU context
} pcb_t;
```

**Functions:**

```c
// Allocate and initialize PCB
pcb_t *pcb_alloc(void);

// Assign unique PID
void pcb_assign_pid(pcb_t *pcb);

// Mark process as runnable (enqueue in scheduler)
void pcb_mark_runnable(pcb_t *pcb);
```

## Data Models

### ELF Binary Format

The loader supports a minimal subset of ELF64:
- **Format:** ELF64 (64-bit)
- **Architecture:** x86_64 (EM_X86_64 = 62)
- **Type:** Executable (ET_EXEC = 2)
- **Linking:** Static only (no dynamic linker, no relocations)
- **Segments:** PT_LOAD only (ignore PT_DYNAMIC, PT_INTERP, etc.)
- **Position:** Non-PIE (fixed addresses)

### Page Table Structure

```
PML4 (Page Map Level 4):
  - 512 entries (8 bytes each)
  - Entries 0-255: User half (canonical lower)
  - Entries 256-511: Kernel half (canonical upper)
  
User PML4 Creation:
  1. Allocate new PML4 frame
  2. Zero entries 0-255 (user half)
  3. Copy entries 256-511 from kernel PML4 (kernel half)
  4. Ensure kernel entries have GLOBAL bit, NX bit preserved
  5. Ensure kernel entries do NOT have USER bit set
```

### CPU Context

```c
// CPU context for Ring3 entry (matches ayken_abi.h)
typedef struct {
    uint64_t rax, rbx, rcx, rdx;
    uint64_t rsi, rdi, rbp, rsp;
    uint64_t r8, r9, r10, r11;
    uint64_t r12, r13, r14, r15;
    uint64_t rip;
    uint64_t rflags;
    uint16_t cs, ss, ds, es, fs, gs;
} cpu_context_t;

// Initial Ring3 context:
//   RIP = e_entry (from ELF)
//   RSP = 0x00007FFFFFFFFFF0 (16-byte aligned)
//   CS = 0x23 (Ring3 code segment, DPL=3) - hardcoded in Phase 10-A
//   SS = DS = ES = 0x1B (Ring3 data segment, DPL=3) - hardcoded in Phase 10-A
//   RFLAGS = 0x202 (IF=1, reserved bit 1 set, deterministic initialization)
//   All GPRs = 0 (except RSP, RIP)
//
// Note: Phase 10-A ignores context CS/SS fields and uses fixed 0x23/0x1B values
// GDT Contract: Kernel GDT MUST define selectors 0x23 and 0x1B with DPL=3
```

### GDT Segment Selector Contract

The ELF loader requires specific GDT entries for Ring3 execution:

**Required GDT Entries:**
- **Selector 0x23** (index 4, RPL=3): User code segment
  - DPL=3, Present=1, Executable=1, Readable=1
  - Base=0, Limit=0xFFFFFFFF, Granularity=4KB, Long mode=1
  
- **Selector 0x1B** (index 3, RPL=3): User data segment
  - DPL=3, Present=1, Writable=1, Readable=1
  - Base=0, Limit=0xFFFFFFFF, Granularity=4KB

**Validation:** The kernel MUST validate GDT configuration before Ring3 entry. If selectors are not properly configured, Ring3 transition will fail with #GP fault.

### RFLAGS Initialization

**Deterministic RFLAGS:** Phase 10-A uses fixed RFLAGS value for deterministic initialization:

```c
// RFLAGS = 0x202
// Bit 1: Reserved (always 1)
// Bit 9: IF (Interrupt Enable) = 1
// All other bits: 0
//   IOPL = 0 (bits 12-13)
//   VM = 0 (bit 17)
//   AC = 0 (bit 18)
//   NT = 0 (bit 14)
//   TF = 0 (bit 8)
//   DF = 0 (bit 10)

ctx->rflags = 0x202;
```

**Rationale:** Fixed RFLAGS ensures deterministic Ring3 entry state. Input-derived RFLAGS values are not used to prevent undefined behavior.


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing all acceptance criteria, the following properties were identified as testable. Redundant properties have been eliminated through reflection:

**Redundancy Analysis:**
- Properties 3.7, 3.8, 3.9 (individual flag checks) are subsumed by Property 3.6 (comprehensive flag derivation)
- Property 10.2 (reject invalid magic) is covered by Property 1.1 (magic validation)
- Property 10.4 (BSS zero-fill) is covered by Property 3.4
- Property 10.5 (reject kernel addresses) is covered by Property 6.6
- Property 10.6 (cleanup on failure) is covered by Properties 3.12, 6.4, 6.5

### Core Correctness Properties

**Property 1: ELF Magic Validation**
*For any* byte sequence, the ELF validator should accept it if and only if the first four bytes are 0x7F, 'E', 'L', 'F'.
**Validates: Requirements 1.1**

**Property 2: Error Return Without State Modification**
*For any* invalid ELF header, validation should return an error code and leave system state unchanged (no allocations, no modifications).
**Validates: Requirements 1.2**

**Property 3: Entry Point Extraction**
*For any* valid ELF binary, the extracted entry point should equal the e_entry field from the ELF header.
**Validates: Requirements 1.3**

**Property 4: PT_LOAD Iteration Completeness**
*For any* ELF binary with N PT_LOAD segments, iteration should visit exactly N segments in order.
**Validates: Requirements 1.4**

**Property 5: Program Header Field Extraction**
*For any* PT_LOAD segment, the extracted fields (p_vaddr, p_offset, p_filesz, p_memsz) should match the values in the program header.
**Validates: Requirements 1.5**

**Property 6: Kernel Mapping Copy Correctness**
*For any* user PML4, all present entries 256-511 should match the corresponding kernel PML4 entries, except that the USER bit must be clear (security enforcement). Non-present entries should remain non-present.
**Validates: Requirements 2.2, 2.3**

**Property 7: Kernel Mapping Security (No USER Bit)**
*For any* user PML4, all entries 256-511 (kernel half) should have the USER bit clear.
**Validates: Requirements 2.3**

**Property 8: Kernel Mapping Bit Preservation**
*For any* user PML4, all entries 256-511 should preserve GLOBAL and NX bits from the kernel PML4.
**Validates: Requirements 2.4**

**Property 9: User Half Initially Unmapped**
*For any* newly created user PML4, all entries 0-255 (user half) should be zero (unmapped).
**Validates: Requirements 2.5**

**Property 10: Frame Allocation Alignment**
*For any* allocated physical frame, the address should be aligned to 4KB (address % 4096 == 0).
**Validates: Requirements 3.2**

**Property 11: Data Copy Correctness**
*For any* segment with p_filesz > 0, the copied data in memory should match the data in the ELF binary at p_offset for p_filesz bytes.
**Validates: Requirements 3.3**

**Property 12: BSS Zero-Fill**
*For any* segment where p_memsz > p_filesz, the bytes from p_filesz to p_memsz should all be zero.
**Validates: Requirements 3.4**

**Property 13: Virtual Address Mapping**
*For any* loaded segment, the virtual address of the mapped memory should equal p_vaddr (aligned down to 4KB), and the segment data should start at the correct offset within the first page (p_vaddr & 0xFFF).
**Validates: Requirements 3.5**

**Property 14: Segment Load Bias Correctness**
*For any* segment with unaligned p_vaddr, the first page should contain the segment data starting at offset (p_vaddr & 0xFFF), and subsequent pages should account for this bias when calculating source offsets from the ELF binary.
**Validates: Requirements 3.5, 6.8**

**Property 15: Page Flag Derivation from ELF Flags**
*For any* user segment, the page flags should be derived as follows:
- PRESENT and USER always set
- WRITABLE set if and only if p_flags includes PF_W
- NX clear if and only if p_flags includes PF_X
- NX set if and only if p_flags excludes PF_X
**Validates: Requirements 3.6, 3.7, 3.8, 3.9, 3.10**

**Property 16: W^X Enforcement**
*For any* segment where p_flags has both PF_W and PF_X set, the loader should reject the segment with an error.
**Validates: Requirements 3.11**

**Property 17: Cleanup on Segment Loading Failure**
*For any* segment loading operation that fails, all previously allocated frames and mapped pages should be deallocated before returning an error.
**Validates: Requirements 3.12**

**Property 18: RIP Initialization**
*For any* ELF binary, the initial cpu_context_t should have RIP equal to the e_entry field.
**Validates: Requirements 4.2**

**Property 19: Error Code Correctness**
*For any* validation failure, the returned error code should be one of EINVAL (invalid format), ENOMEM (allocation failure), or ENOEXEC (unsupported format).
**Validates: Requirements 6.1**

**Property 20: Cleanup Completeness and Reverse Order**
*For any* cleanup operation, all tracked frames should be deallocated and all tracked pages should be unmapped, in reverse allocation order (last allocated, first freed).
**Validates: Requirements 6.4, 6.5**

**Property 21: Kernel Space Address Rejection**
*For any* segment with p_vaddr >= 0xFFFF800000000000, the loader should reject the binary with an error.
**Validates: Requirements 6.6**

**Property 22: Segment Size Limit**
*For any* segment with p_memsz > 1GB (1073741824 bytes), the loader should reject the binary with an error.
**Validates: Requirements 6.7**

**Property 23: Address Alignment**
*For any* segment with unaligned p_vaddr, the mapped virtual address should be aligned down to the nearest 4KB boundary (p_vaddr & ~0xFFF).
**Validates: Requirements 6.8**

**Property 24: User Address Space Range Validation**
*For any* segment, the address range [p_vaddr, p_vaddr + p_memsz) should fit entirely within the user address space (0x0000000000400000 to 0x00007FFFFFFFFFFF).
**Validates: Requirements 6.9**

**Property 25: Non-ELF64 Rejection**
*For any* binary where e_ident[EI_CLASS] != ELFCLASS64, the loader should reject the binary with an error.
**Validates: Requirements 9.1**

**Property 26: Non-x86_64 Rejection**
*For any* binary where e_machine != EM_X86_64, the loader should reject the binary with an error.
**Validates: Requirements 9.2**

**Property 27: Non-PT_LOAD Segment Ignoring**
*For any* program header with p_type != PT_LOAD, the loader should skip the segment (not load it).
**Validates: Requirements 9.3**

**Property 28: Segment File Range Validation**
*For any* PT_LOAD segment, the file range [p_offset, p_offset + p_filesz) must fit entirely within the ELF blob size.
**Validates: Requirements 6.9**

**Property 29: Program Header Table Bounds Validation**
*For any* ELF binary, the program header table range [e_phoff, e_phoff + e_phnum * e_phentsize) must fit entirely within the ELF blob size.
**Validates: Requirements 6.10**

**Property 30: Ring3 User Code Execution Proof**
*For any* successful Ring3 transition, the user code must emit the P10_RING3_USER_CODE marker, proving actual execution (not just transition attempt).
**Validates: Requirements 12.3**

## Error Handling

### Error Codes

The ELF loader uses standard errno codes:

- **EINVAL** (22): Invalid ELF format (bad magic, invalid fields, constraint violations)
- **ENOMEM** (12): Memory allocation failure (out of physical memory)
- **ENOEXEC** (8): Unsupported executable format (wrong architecture, wrong class)

### Error Handling Strategy

**Fail-Fast Validation:**
1. Validate ELF magic and header before any allocations
2. Validate segment addresses and sizes before loading
3. Reject invalid binaries immediately with specific error codes

**Cleanup on Failure:**
1. Maintain cleanup tracker throughout loading process
2. Track all allocated frames in order
3. Track all mapped pages in order
4. On error, cleanup in reverse allocation order
5. Ensure no memory leaks or partial state

**Panic on Catastrophic Failure:**
- If Ring3 entry fails after successful loading, trigger kernel panic
- Panic includes diagnostic information (CR3, RIP, RFLAGS, segment selectors)
- Panic is fail-safe: system halts rather than continuing in undefined state

### Validation Checks

**ELF Header Validation:**
```c
// Check magic number
if (ehdr->e_ident[0] != 0x7F || ehdr->e_ident[1] != 'E' ||
    ehdr->e_ident[2] != 'L' || ehdr->e_ident[3] != 'F') {
    return -EINVAL;
}

// Check class (64-bit)
if (ehdr->e_ident[EI_CLASS] != ELFCLASS64) {
    return -ENOEXEC;
}

// Check architecture (x86_64)
if (ehdr->e_machine != EM_X86_64) {
    return -ENOEXEC;
}

// Check type (executable)
if (ehdr->e_type != ET_EXEC) {
    return -ENOEXEC;
}

// Check program header table bounds (critical security check)
uint64_t phdr_end = ehdr->e_phoff + (ehdr->e_phnum * ehdr->e_phentsize);
if (phdr_end > elf_size) {
    return -EINVAL; // Program header table extends beyond blob
}
```

**Segment Validation:**
```c
// Check for overflow in segment range calculation
if (phdr->p_vaddr > UINT64_MAX - phdr->p_memsz) {
    return -EINVAL; // Overflow would occur
}

// Check segment file range bounds (critical security check)
if (phdr->p_filesz > 0) {
    if (phdr->p_offset > UINT64_MAX - phdr->p_filesz) {
        return -EINVAL; // File range overflow
    }
    uint64_t file_end = phdr->p_offset + phdr->p_filesz;
    if (file_end > elf_size) {
        return -EINVAL; // Segment data extends beyond blob
    }
}

// Check kernel space overlap
if (phdr->p_vaddr >= 0xFFFF800000000000ULL) {
    return -EINVAL;
}

// Check segment size limit (1GB)
if (phdr->p_memsz > (1ULL << 30)) {
    return -EINVAL;
}

// Check user address space range (no overflow now)
// AykenOS design decision: Lower 4MB reserved, user programs load at >= 0x400000
uint64_t seg_end = phdr->p_vaddr + phdr->p_memsz;
if (phdr->p_vaddr < 0x400000 || seg_end > 0x00007FFFFFFFFFFFULL) {
    return -EINVAL;
}

// Check W^X enforcement (reject segments with both W and X)
if ((phdr->p_flags & PF_W) && (phdr->p_flags & PF_X)) {
    return -EINVAL;
}
```

## Testing Strategy

### Dual Testing Approach

The ELF loader requires both unit tests and property-based tests for comprehensive coverage:

**Unit Tests:**
- Specific examples (minimal ELF binary, multi-segment binary)
- Edge cases (empty segments, BSS-only segments, guard page access)
- Error conditions (invalid magic, kernel addresses, oversized segments)
- Integration points (PCB creation, scheduler integration in Phase 10-C)

**Property-Based Tests:**
- Universal properties across all inputs (see Correctness Properties section)
- Randomized ELF generation (valid and invalid)
- Comprehensive input coverage (all flag combinations, all address ranges)
- Minimum 100 iterations per property test

### Property-Based Testing Configuration

**Library:** Use existing property-based testing library for C (e.g., theft, or custom generator)

**Test Configuration:**
- Minimum 100 iterations per property test
- Each test tagged with feature name and property number
- Tag format: `/* Feature: minimal-ring3-elf-loader, Property N: <property_text> */`

**Example Property Test:**

```c
/* Feature: minimal-ring3-elf-loader, Property 1: ELF Magic Validation */
void test_property_elf_magic_validation(void) {
    for (int i = 0; i < 100; i++) {
        // Generate random byte sequence
        uint8_t blob[16];
        random_bytes(blob, 16);
        
        // Set magic if this iteration should pass
        bool should_pass = (i % 2 == 0);
        if (should_pass) {
            blob[0] = 0x7F;
            blob[1] = 'E';
            blob[2] = 'L';
            blob[3] = 'F';
        }
        
        // Validate
        int result = elf64_validate(blob, 16);
        
        // Check property
        if (should_pass) {
            assert(result == 0);
        } else {
            assert(result < 0);
        }
    }
}
```

### Unit Test Examples

**Test 1: Minimal ELF Binary (Phase 10-A)**
```c
void test_minimal_elf_load(void) {
    // Hardcoded minimal ELF with single PT_LOAD
    extern const uint8_t minimal_elf[];
    extern const size_t minimal_elf_size;
    
    // Load process
    pcb_t *pcb = NULL;
    int result = elf_load_process(minimal_elf, minimal_elf_size, &pcb);
    
    // Verify success
    assert(result == 0);
    assert(pcb != NULL);
    assert(pcb->address_space.cr3_phys != 0);
    assert(pcb->context.rip == 0x400000); // Expected entry point
}
```

**Test 2: Invalid Magic Rejection**
```c
void test_invalid_magic_rejection(void) {
    uint8_t invalid_elf[64] = {0};
    invalid_elf[0] = 0x7F;
    invalid_elf[1] = 'X'; // Invalid
    invalid_elf[2] = 'Y';
    invalid_elf[3] = 'Z';
    
    int result = elf64_validate(invalid_elf, 64);
    assert(result == -EINVAL);
}
```

**Test 3: Multi-Segment Loading (Phase 10-B)**
```c
void test_multi_segment_load(void) {
    // ELF with 3 PT_LOAD segments (code, rodata, data)
    extern const uint8_t multi_seg_elf[];
    extern const size_t multi_seg_elf_size;
    
    pcb_t *pcb = NULL;
    int result = elf_load_process(multi_seg_elf, multi_seg_elf_size, &pcb);
    
    assert(result == 0);
    // Verify all segments loaded
    // Verify correct flags (RX, R, RW)
}
```

**Test 4: BSS Zero-Fill**
```c
void test_bss_zero_fill(void) {
    // ELF with BSS section (p_memsz > p_filesz)
    extern const uint8_t bss_elf[];
    extern const size_t bss_elf_size;
    
    pcb_t *pcb = NULL;
    int result = elf_load_process(bss_elf, bss_elf_size, &pcb);
    
    assert(result == 0);
    // Verify BSS region is zeroed
    // (requires reading user memory after load)
}
```

**Test 5: Kernel Address Rejection**
```c
void test_kernel_address_rejection(void) {
    // ELF with segment at kernel address
    uint8_t kernel_addr_elf[512];
    create_elf_with_kernel_addr(kernel_addr_elf, 0xFFFF800000000000ULL);
    
    pcb_t *pcb = NULL;
    int result = elf_load_process(kernel_addr_elf, 512, &pcb);
    
    assert(result == -EINVAL);
    assert(pcb == NULL); // No partial state
}
```

**Test 6: Cleanup on Allocation Failure**
```c
void test_cleanup_on_failure(void) {
    // Simulate allocation failure mid-load
    set_alloc_failure_after(5); // Fail after 5 allocations
    
    extern const uint8_t multi_seg_elf[];
    extern const size_t multi_seg_elf_size;
    
    pcb_t *pcb = NULL;
    int result = elf_load_process(multi_seg_elf, multi_seg_elf_size, &pcb);
    
    assert(result == -ENOMEM);
    assert(pcb == NULL);
    // Verify all allocations cleaned up (no leaks)
    assert(get_allocated_frame_count() == 0);
}
```

### CI Gate Integration

**Gate: gate_ring3_execution.sh**

**Purpose:** Validate Ring3 execution success through runtime markers.

**Validation:**
1. Build kernel with Phase 10-A ELF loader
2. Run in QEMU with serial output capture
3. Extract markers from serial output
4. Validate marker sequence:
   - KERNEL_BEFORE_RING3 (existing)
   - P10_CR3_SWITCH (new)
   - P10_RING3_ENTER (new)
5. Fail if markers missing or out of order

**Implementation:**
```bash
#!/bin/bash
# scripts/ci/gate_ring3_execution.sh

set -e

# Build kernel
make clean
make KERNEL_PROFILE=validation

# Run QEMU with serial capture
timeout 10s qemu-system-x86_64 \
    -drive format=raw,file=efi.img \
    -bios firmware/OVMF.fd \
    -serial file:qemu_serial.log \
    -nographic \
    || true

# Extract markers
python3 tools/ci/extract_markers.py qemu_serial.log > markers.txt

# Validate sequence
if ! grep -q "KERNEL_BEFORE_RING3" markers.txt; then
    echo "ERROR: Missing KERNEL_BEFORE_RING3 marker"
    exit 1
fi

if ! grep -q "P10_CR3_SWITCH" markers.txt; then
    echo "ERROR: Missing P10_CR3_SWITCH marker"
    exit 1
fi

if ! grep -q "P10_RING3_ENTER" markers.txt; then
    echo "ERROR: Missing P10_RING3_ENTER marker"
    exit 1
fi

# Validate order
python3 tools/ci/validate_marker_order.py markers.txt \
    KERNEL_BEFORE_RING3 P10_CR3_SWITCH P10_RING3_ENTER

echo "✓ Ring3 execution gate PASSED"
```

### Test Coverage Goals

- **Unit Test Coverage:** >80% line coverage for ELF loader code
- **Property Test Coverage:** All 27 correctness properties implemented
- **Integration Test Coverage:** All phases (10-A, 10-B, 10-C) validated
- **CI Gate Coverage:** Runtime marker validation for Ring3 execution

## Phased Implementation Plan

### Phase 10-A: Ring3 Entry Proof (Minimal)

**Goal:** Prove Ring3 execution works with minimal complexity. This phase focuses on Ring3 transition proof, NOT full ELF parsing.

**Scope:**
- Hardcoded ELF binary (embedded in kernel as const uint8_t array)
- Minimal ELF parsing (validate magic, extract e_entry, parse single PT_LOAD)
- Single PT_LOAD segment (code only, RX flags)
- Minimal PML4 creation (kernel half copy, user half empty)
- Stack allocation (with guard page)
- CR3 switch + IRETQ (with correct register preservation)
- Runtime markers (P10_CR3_SWITCH, P10_RING3_ENTER)
- Inline marker emission (direct serial port write, no C calls)

**Important:** Phase 10-A validates Ring3 transition mechanism, not comprehensive ELF loading. Full ELF parsing (multi-segment, BSS, error handling) is deferred to Phase 10-B.

**Deliverables:**
- `kernel/src/elf/loader_phase_a.c` (minimal loader with basic ELF validation)
- `kernel/src/arch/x86_64/ring3_enter.S` (IRETQ assembly with correct register handling)
- `userspace/minimal/minimal.S` (minimal Ring3 assembly stub for syscall roundtrip test - Phase 10-A2 Task 3)
- `userspace/minimal/user.ld` (linker script for static Ring3 binary)
- `userspace/minimal/Makefile` (build minimal ELF from assembly source)
- `tools/embed_elf.py` (convert ELF to C array)
- `scripts/ci/gate_ring3_execution.sh` (CI gate)
- CI gate PASS

**Success Criteria:**
- Markers appear in correct order (KERNEL_BEFORE_RING3, P10_CR3_SWITCH, P10_RING3_ENTER, P10_RING3_USER_CODE)
- P10_RING3_USER_CODE marker proves Ring3 code execution (not just transition attempt)
- No triple fault
- No kernel panic
- CI gate passes
- Ring3 code executes (proven by user-space marker)

### Phase 10-B: Full ELF Parsing

**Goal:** Support real ELF binaries with multiple segments.

**Scope:**
- Full ELF validation (magic, class, machine, type)
- PT_LOAD iteration (all segments)
- Multi-segment loading (code, rodata, data, bss)
- BSS zero-fill
- W^X enforcement
- Comprehensive error handling
- Cleanup tracking

**Deliverables:**
- `kernel/include/elf/elf64.h` (ELF structures)
- `kernel/src/elf/parser.c` (ELF parsing)
- `kernel/src/elf/loader.c` (full loader)
- `kernel/src/mm/user_as.c` (user address space)
- Unit tests for all properties
- Property-based tests

**Success Criteria:**
- Load multi-segment ELF binaries
- Correct flag derivation (RX, R, RW)
- BSS correctly zeroed
- Invalid binaries rejected
- Cleanup on error works

### Phase 10-C: Process Integration

**Goal:** Integrate ELF loader with process management and scheduler.

**Scope:**
- PCB allocation and initialization
- PID assignment
- Scheduler integration (RUNNABLE state)
- Context switch path (kernel ↔ user)
- Syscall entry path (Ring3 → Ring0)
- Minimal syscall (e.g., sys_exit)

**Deliverables:**
- `kernel/src/proc/process.c` (PCB management)
- `kernel/src/sched/user_sched.c` (user process scheduling)
- `kernel/src/sys/syscall_entry.S` (syscall entry)
- Integration tests
- Syscall validation marker (P10_SYSCALL_HELLO)

**Success Criteria:**
- Loaded process runs as scheduled process
- Context switch works (kernel ↔ user)
- Syscall works (Ring3 → Ring0 → Ring3)
- Multiple processes can be loaded (future)

---

**Author:** Kenan AY  
**Last Updated:** 2026-02-22  
**Next Review:** After Phase 10-A implementation
