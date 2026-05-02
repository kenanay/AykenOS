# Bugfix Requirements Document

## Introduction

The ABDF Hardware Contract document (`_ayken/specs/ABDF_HARDWARE_CONTRACT.md`) contains critical technical inconsistencies and undefined behaviors that could lead to implementation errors, incorrect assumptions about hardware capabilities, and runtime failures. These issues affect the core memory contract, string handling, checksum validation, GPU capabilities, immutability scope, and alignment guarantees. This bugfix addresses six specific technical contradictions plus one boundary hardening addition that must be resolved before implementation begins.

## Bug Analysis

### Current Behavior (Defect)

**1. String Pool Representation**

1.1 WHEN the String Pool section is read THEN the system documents "UTF-8 Data (null-terminated)" which contradicts the earlier design decision that strings are "offset + length, NOT null-terminated"

**2. Checksum Validation Scope**

1.2 WHEN the Header Structure defines `checksum: u64` with comment "XXH3 hash" THEN the system does not specify what data the checksum covers (segments, string pool, header fields, or combinations thereof)

**3. GPU Zero-Copy Capability Claims**

1.3 WHEN the GPU Buffer Segment section states "GPU buffer data is **directly mappable** to GPU memory" THEN the system presents this as a guaranteed behavior across all hardware platforms without acknowledging hardware limitations or fallback requirements

**4. Immutability Scope Ambiguity**

1.4 WHEN the Constitutional Note states "This contract is **immutable** in Phase 1" THEN the system does not distinguish between the core memory contract (which must be immutable) and segment type definitions (UI/GPU hooks which are extensible)

**5. Struct Size Validation Missing**

1.5 WHEN the Segment Directory Entry claims to be 32 bytes via `#[repr(C, align(32))]` THEN the system does not include compile-time static assertions to guarantee this size invariant

**6. Alignment Requirement Conflation**

1.6 WHEN the Alignment Rules section states "All offsets MUST be 64B aligned" with reason "SIMD (AVX-512), GPU cache lines, mmap page alignment" THEN the system conflates segment alignment (64B for SIMD/GPU) with mmap page alignment requirements (typically 4KB), which serve different purposes

### Expected Behavior (Correct)

**1. String Pool Representation**

2.1 WHEN the String Pool section is read THEN the system SHALL document strings as "offset + length" representation WITHOUT null-termination, consistent with the original design decision

**2. Checksum Validation Scope**

2.2 WHEN the Header Structure defines `checksum: u64` THEN the system SHALL explicitly specify the checksum scope (e.g., "covers all data from segment directory through end of file, excluding the checksum field itself")

**3. GPU Zero-Copy Capability Claims**

2.3 WHEN the GPU Buffer Segment section describes GPU mapping capabilities THEN the system SHALL state this as an "optimization target with fallback" rather than guaranteed behavior, acknowledging hardware-specific limitations

**4. Immutability Scope Clarification**

2.4 WHEN the Constitutional Note describes immutability THEN the system SHALL distinguish between "core memory contract is immutable" (header, alignment, endianness) and "segment type extensions are versioned" (UI/GPU hooks can be extended)

**5. Struct Size Validation**

2.5 WHEN the Segment Directory Entry structure is defined THEN the system SHALL include a compile-time static assertion `assert_eq!(size_of::<SegmentEntry>(), 32)` to guarantee the size invariant

**6. Alignment Requirement Separation**

2.6 WHEN the Alignment Rules section describes alignment requirements THEN the system SHALL separate segment alignment (64B for SIMD/GPU performance) from mmap page alignment (OS-specific, typically 4KB for file mapping), clarifying their distinct purposes

**7. ABDF-BCIB Boundary Contract**

2.7 WHEN BCIB references ABDF data structures THEN the system SHALL document that BCIB MUST NOT store raw pointers to ABDF memory, and MUST reference ABDF only through stable identifiers (object_id, segment_index, string_index, offset+length), preserving ABDF's pointer-free memory model

### Unchanged Behavior (Regression Prevention)

**3. Core Contract Structure**

3.1 WHEN the ABDF binary layout is accessed THEN the system SHALL CONTINUE TO maintain 64-byte alignment for all major sections (header, segment directory, string pool, data segments)

**4. Memory Safety Guarantees**

3.2 WHEN ABDF files are used in concurrent scenarios THEN the system SHALL CONTINUE TO enforce immutability guarantees for safe concurrent reads

**5. Hardware-Ready Design Philosophy**

3.3 WHEN ABDF is designed for hardware integration THEN the system SHALL CONTINUE TO prioritize zero-copy access patterns for CPU, GPU, and UI subsystems

**6. Phase 1 Scope Boundaries**

3.4 WHEN UI and GPU segment types are defined THEN the system SHALL CONTINUE TO treat them as "type definitions only" with no implementation required in Phase 1

**7. Endianness Policy**

3.5 WHEN ABDF files are created or parsed THEN the system SHALL CONTINUE TO enforce little-endian only representation with no runtime conversion

**8. Segment Type Definitions**

3.6 WHEN segment types are referenced (ROW, COLUMN, VECTOR, UI_SCENE, GPU_BUFFER) THEN the system SHALL CONTINUE TO use the existing type constants and structure definitions
