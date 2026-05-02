# Implementation Plan

## Phase 1: Exploration - Document Bug Condition

- [x] 1. Document inconsistency exploration check (BEFORE implementing fix)
  - **Property 1: Bug Condition** - Six Original Technical Inconsistencies + Boundary Hardening Need
  - **CRITICAL**: This check MUST FAIL on unfixed document - failure confirms the bugs exist
  - **DO NOT attempt to fix the document when inconsistencies are found**
  - **NOTE**: This check encodes the expected correct behavior - it will validate the fix when it passes after implementation
  - **GOAL**: Surface the six specific documentation inconsistencies plus one boundary hardening need that demonstrate the bugs exist
  - **Scoped Validation Approach**: Check each of the six concrete issues plus boundary contract identified in the bug report
  - Validate Issue 1: String Pool section contains "UTF-8 Data (null-terminated)" contradicting earlier "NOT null-terminated" design
  - Validate Issue 2: Checksum field `checksum: u64, // XXH3 hash` lacks scope definition (what data is hashed)
  - Validate Issue 3: GPU Buffer section states "directly mappable" without acknowledging hardware limitations or fallback
  - Validate Issue 4: Constitutional Note states "immutable" without distinguishing core contract vs. segment type extensions
  - Validate Issue 5: SegmentEntry struct lacks compile-time static assertions for size validation
  - Validate Issue 6: Alignment Rules conflate 64B segment alignment with mmap page alignment
  - Validate Issue 7: ABDF-BCIB boundary contract is not documented (pointer-free guarantee missing)
  - Run validation on UNFIXED document at `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
  - **EXPECTED OUTCOME**: Validation FAILS (this is correct - it proves the bugs exist)
  - Document each inconsistency found with exact line numbers and contradictory text
  - Mark task complete when validation is run, failures are documented, and all seven issues are confirmed
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.7_

## Phase 2: Preservation - Document Baseline Behavior

- [x] 2. Document preservation baseline check (BEFORE implementing fix)
  - **Property 2: Preservation** - Non-Buggy Sections Remain Unchanged
  - **IMPORTANT**: Follow observation-first methodology
  - Observe current content in UNFIXED document for all sections NOT related to the six identified issues
  - Document baseline content for preservation validation:
    - Binary Layout diagram and structure
    - Header Structure field definitions (except checksum comment)
    - Memory Contract Flags definitions
    - All Segment Type constants (ROW, COLUMN, VECTOR, UI_SCENE, GPU_BUFFER)
    - UI Segment Structure definitions
    - Vector/Tensor Support definitions
    - Memory Safety Contract section
    - CPU ↔ GPU Bridge section
    - Phase 1 Checklist items
    - Success Criteria section
    - Critical Warnings section
  - Create baseline snapshot or checksum of non-buggy sections
  - **EXPECTED OUTCOME**: Baseline documented successfully (confirms what must be preserved)
  - Mark task complete when baseline is documented and ready for diff comparison
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

## Phase 3: Implementation - Apply Documentation Fixes

- [x] 3. Fix ABDF Hardware Contract documentation

  - [x] 3.1 Correct String Pool representation (Issue 1)
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Section**: "🧵 String Pool (Zero-Copy)"
    - **Current**: Diagram shows "UTF-8 Data (null-terminated)" with example `"hello\0world\0..."`
    - **Change**: Replace with "UTF-8 Data (offset + length)" representation
    - Update diagram to show raw UTF-8 bytes without null terminators
    - Add critical clarifications:
      - Null bytes are allowed ONLY as payload data if explicitly included in length field
      - Empty strings are valid (length = 0)
      - Offset must point to valid position within string pool even for empty strings
      - `offset + length` MUST NOT exceed `string_pool_size` (bounds check)
      - All STRING types in metadata segments MUST reference string pool using offset+length
    - **Rationale**: Aligns with original design decision, prevents parser ambiguity, ensures memory safety
    - _Bug_Condition: isBugCondition(section) where section.text CONTAINS "null-terminated" AND section.name == "String Pool"_
    - _Expected_Behavior: String Pool section documents offset+length representation WITHOUT null-termination_
    - _Preservation: Binary layout, header structure, segment types remain unchanged_
    - _Requirements: 1.1, 2.1_

  - [x] 3.2 Define checksum validation scope (Issue 2)
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Section**: "🔒 Header Structure (64 bytes)", `checksum: u64` field
    - **Current**: `checksum: u64, // XXH3 hash`
    - **Change**: Update comment to `checksum: u64, // XXH3-64 hash of bytes [64..total_size), excludes checksum field itself`
    - Add new subsection "### Checksum Validation" with:
      - Exact byte range: `[64..total_size)`
      - Exclusion: checksum field itself
      - Algorithm: XXH3-64
      - Checksum MUST be computed after all alignment padding is applied
      - Checksum MUST be computed on final byte representation
      - Checksum MUST be computed with all reserved fields zeroed
    - **Rationale**: Provides unambiguous specification, ensures deterministic results
    - _Bug_Condition: isBugCondition(field) where field.name == "checksum" AND NOT field.defines_scope_
    - _Expected_Behavior: Checksum field includes explicit scope definition covering bytes [64..total_size)_
    - _Preservation: Header structure layout, field sizes, alignment remain unchanged_
    - _Requirements: 1.2, 2.2_

  - [x] 3.3 Clarify GPU zero-copy as optimization target (Issue 3)
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Section**: "🖥️ GPU Buffer Segment (Phase 1 Hook)"
    - **Current**: "**Critical**: GPU buffer data is **directly mappable** to GPU memory."
    - **Change**: Replace with "**Critical**: GPU buffer data is designed for **direct mapping to GPU memory** (optimization target). Implementations MUST provide fallback to staged upload/copy on hardware without direct mapping support."
    - Add note acknowledging hardware-dependent behavior
    - Add requirement: Fallback MUST preserve byte-exact data representation (no format conversion)
    - **Rationale**: Sets realistic expectations, ensures data integrity across GPU access paths
    - _Bug_Condition: isBugCondition(section) where section.text CONTAINS "directly mappable" AND NOT section.acknowledges_fallback_
    - _Expected_Behavior: GPU section describes direct mapping as optimization target with mandatory fallback_
    - _Preservation: GPU segment structure, buffer types, data layout remain unchanged_
    - _Requirements: 1.3, 2.3_

  - [x] 3.4 Separate immutability scope (core vs extensions) (Issue 4)
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Section**: "📚 References", "Constitutional Note"
    - **Current**: "**Constitutional Note**: This contract is **immutable** in Phase 1. Changes require architectural review."
    - **Change**: Replace with detailed breakdown:
      - "**Immutable Core Contract**: Header structure, endianness, alignment rules, and pointer-free memory model are immutable."
      - "**Versioned Extensions**: New segment types, flags, and metadata schemas can be added through versioning without breaking the core contract."
      - Existing segment types MUST NOT change binary layout across versions
      - New segment types MUST use new type identifiers (cannot reuse existing type IDs)
    - **Rationale**: Clarifies what can/cannot change, guarantees backward compatibility
    - _Bug_Condition: isBugCondition(note) where note.text == "immutable" AND NOT note.distinguishes_core_vs_extensions_
    - _Expected_Behavior: Constitutional Note distinguishes immutable core from versioned extensions_
    - _Preservation: Phase 1 scope, success criteria, checklist remain unchanged_
    - _Requirements: 1.4, 2.4_

  - [x] 3.5 Add SegmentEntry static assertions (Issue 5)
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Section**: "📊 Segment Directory Entry (32 bytes)", after struct definition
    - **Current**: Struct definition with `#[repr(C, align(32))]` but no compile-time validation
    - **Change**: Add new subsection "### Compile-Time Validation" with:
      ```rust
      // Static assertions to guarantee struct invariants
      const _: () = assert!(core::mem::size_of::<SegmentEntry>() == 32);
      const _: () = assert!(core::mem::align_of::<SegmentEntry>() == 32);
      ```
    - **Rationale**: Ensures struct size invariant is validated at compile time
    - _Bug_Condition: isBugCondition(struct) where struct.name == "SegmentEntry" AND NOT struct.includes_static_assertions_
    - _Expected_Behavior: SegmentEntry includes compile-time static assertions for size and alignment_
    - _Preservation: Struct field definitions, layout, segment types remain unchanged_
    - _Requirements: 1.5, 2.5_

  - [x] 3.6 Separate alignment requirements (segment vs mmap) (Issue 6)
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Section**: "⚙️ Alignment Rules (IMMUTABLE)"
    - **Current**: Single rule "All offsets MUST be 64B aligned" with reason "SIMD (AVX-512), GPU cache lines, mmap page alignment"
    - **Change**: Split into two distinct rules:
      - **Rule 1**: "All ABDF segment offsets MUST be 64B aligned (Reason: SIMD/AVX-512 operations, GPU cache line optimization)"
      - **Rule 2**: "ABDF file base address for mmap SHOULD be page-aligned (Reason: OS-level requirement, typically 4KB, handled by OS/runtime, not part of ABDF format)"
    - Add clarifications:
      - Segment alignment is ABDF format concern, mmap alignment is OS/runtime concern
      - Segment size does NOT need to be multiple of 64B. Only segment start offset must be aligned.
      - **Versioning Note**: This specification defines 64B alignment for v1.0. Earlier ABDF implementations (v0.1) used 8B alignment. Parsers SHOULD support both via version field detection for backward compatibility.
    - **Rationale**: Separates internal format from OS requirements, clarifies offset vs size, provides backward compatibility
    - _Bug_Condition: isBugCondition(rules) where rules.conflates_segment_and_mmap_alignment_
    - _Expected_Behavior: Alignment Rules separate 64B segment alignment from OS-level mmap page alignment_
    - _Preservation: 64B alignment requirement, endianness policy, memory contract remain unchanged_
    - _Requirements: 1.6, 2.6_

  - [x] 3.7 Add ABDF↔BCIB boundary contract
    - **File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - **Location**: New section after "⚙️ Alignment Rules (IMMUTABLE)"
    - **Change**: Add new section "### 🔗 ABDF-BCIB Integration Contract" with:
      - **Pointer-Free Guarantee**: BCIB MUST NOT store raw pointers to ABDF memory
      - **Stable Identifiers Only**: BCIB references ABDF only through:
        - `object_id` (stable object identifier)
        - `segment_index` (segment directory index)
        - `string_index` (string pool entry index)
        - `offset + length` (bounds-checked offset pairs)
      - **No Layout Reinterpretation**: BCIB MUST NOT reinterpret ABDF memory layout outside the ABDF contract
      - **Memory Safety Preservation**: BCIB operations MUST preserve ABDF's pointer-free memory model
    - **Rationale**: Ensures ABDF's pointer-free memory contract is not violated by BCIB's execution model
    - _Expected_Behavior: ABDF-BCIB boundary contract prevents pointer-based coupling_
    - _Preservation: All existing ABDF specifications remain unchanged_
    - _Requirements: 2.7_

  - [x] 3.8 Verify bug condition exploration check now passes
    - **Property 1: Expected Behavior** - Six Original Inconsistencies + Boundary Hardening Resolved
    - **IMPORTANT**: Re-run the SAME validation from task 1 - do NOT create a new validation
    - The validation from task 1 encodes the expected correct behavior
    - When this validation passes, it confirms the expected behavior is satisfied
    - Run validation on FIXED document at `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`
    - Verify Issue 1: String Pool section now documents offset+length WITHOUT null-termination
    - Verify Issue 2: Checksum field now includes explicit scope definition
    - Verify Issue 3: GPU section now describes direct mapping as optimization target with fallback
    - Verify Issue 4: Constitutional Note now distinguishes core contract vs. extensions
    - Verify Issue 5: SegmentEntry now includes static assertions
    - Verify Issue 6: Alignment Rules now separate segment and mmap alignment
    - Verify Issue 7: ABDF-BCIB boundary contract is documented
    - **EXPECTED OUTCOME**: Validation PASSES (confirms bugs are fixed)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_

  - [x] 3.9 Verify preservation check still passes (diff-based validation)
    - **Property 2: Preservation** - Non-Buggy Sections Unchanged
    - **IMPORTANT**: Re-run the SAME preservation check from task 2 - do NOT create new checks
    - Generate unified diff between original and corrected document
    - Verify all changes are limited to the seven identified sections (tasks 3.1-3.7)
    - Verify unchanged sections:
      - Binary Layout diagram structure
      - Header Structure field definitions (except checksum comment)
      - Memory Contract Flags bit definitions
      - All Segment Type constants and values
      - UI Segment Structure field layouts
      - Vector/Tensor Support definitions
      - Memory Safety Contract guarantees
      - CPU ↔ GPU Bridge flow
      - Phase 1 Checklist items
      - Success Criteria list
      - Critical Warnings content
    - **EXPECTED OUTCOME**: Diff shows ONLY the seven targeted changes (confirms no regressions)
    - Confirm all non-buggy sections remain byte-identical to baseline
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 4. Final review and validation checkpoint
  - Review all seven documentation corrections for internal consistency
  - Verify the corrected document is internally consistent (no new contradictions)
  - Verify all cross-references between sections remain valid
  - Verify the document still serves as complete specification for ABDF implementation
  - Verify the corrected document maintains "hardware-ready" design philosophy
  - Verify Phase 1 scope boundaries remain clear and actionable
  - Generate final diff summary showing only the seven targeted changes
  - Ensure all tasks are complete and all requirements are satisfied
  - Record unresolved questions, if any, in final review notes
