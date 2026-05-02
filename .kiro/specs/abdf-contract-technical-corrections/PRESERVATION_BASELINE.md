# ABDF Hardware Contract - Preservation Baseline

**Date**: 2026-05-02  
**Source**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md` (UNFIXED)  
**Purpose**: Establish baseline for diff validation - ensure ONLY 7 targeted bug fixes are applied

## Preservation Scope

All sections NOT related to the 7 identified bug fixes MUST remain byte-identical.

## Baseline Snapshot

### 1. Binary Layout Diagram (MUST NOT CHANGE)

```
┌─────────────────────────────────────────┐
│ ABDF Header (64B aligned)               │ offset: 0
├─────────────────────────────────────────┤
│ Segment Directory (64B aligned)         │ offset: 64
├─────────────────────────────────────────┤
│ String Pool (64B aligned)               │ offset: N * 64
├─────────────────────────────────────────┤
│ Data Segments (each 64B aligned)        │ offset: M * 64
│   - ROW segments                        │
│   - COLUMN segments                     │
│   - VECTOR segments (GPU-ready)         │
│   - UI_SCENE segments                   │
│   - GPU_BUFFER segments                 │
└─────────────────────────────────────────┘
```

**Preservation**: Structure, offsets, segment list unchanged

### 2. Header Structure Fields (EXCEPT checksum comment)

```rust
#[repr(C, align(64))]
struct ABDFHeader {
    magic: [u8; 4],           // "ABDF"
    version: u32,             // 0x00010000 = 1.0.0
    endianness: u8,           // 0x01 = LE (ONLY)
    alignment: u8,            // 64 (IMMUTABLE)
    flags: u16,               // memory contract flags
    
    segment_count: u32,       // number of segments
    segment_dir_offset: u64,  // always 64
    
    string_pool_offset: u64,  // offset to string pool
    string_pool_size: u64,    // size in bytes
    
    total_size: u64,          // total file size
    checksum: u64,            // XXH3 hash  ← ONLY THIS COMMENT CHANGES
    
    reserved: [u8; 16],       // future use
}
```

**Preservation**: All field names, types, sizes, comments (except checksum) unchanged

### 3. Memory Contract Flags (MUST NOT CHANGE)

```
bit 0: READ_ONLY       - immutable after creation
bit 1: SNAPSHOT        - point-in-time consistency
bit 2: THREAD_SAFE     - concurrent read safe
bit 3: GPU_MAPPABLE    - can be mapped to GPU
bit 4: MMAP_FRIENDLY   - safe for mmap
bit 5-15: reserved
```

**Preservation**: Bit assignments, descriptions unchanged

### 4. Segment Directory Entry Structure (EXCEPT static assertions addition)

```rust
#[repr(C, align(32))]
struct SegmentEntry {
    segment_type: u32,        // ROW, COLUMN, VECTOR, UI_SCENE, GPU_BUFFER
    flags: u32,               // segment-specific flags
    offset: u64,              // from file start (64B aligned)
    size: u64,                // segment size in bytes
    element_count: u64,       // number of elements
}
```

**Preservation**: Struct definition unchanged (new subsection added after)

### 5. Segment Type Constants (MUST NOT CHANGE)

```rust
const SEGMENT_ROW: u32        = 0x0001;
const SEGMENT_COLUMN: u32     = 0x0002;
const SEGMENT_VECTOR: u32     = 0x0003;
const SEGMENT_METADATA: u32   = 0x0004;
const SEGMENT_UI_SCENE: u32   = 0x0100;  // UI hook
const SEGMENT_UI_WIDGET: u32  = 0x0101;  // UI hook
const SEGMENT_UI_LAYOUT: u32  = 0x0102;  // UI hook
const SEGMENT_GPU_BUFFER: u32 = 0x0200;  // GPU hook
const SEGMENT_GPU_TEXTURE: u32= 0x0201;  // GPU hook (future)
```

**Preservation**: All constants, values, comments unchanged

### 6. UI Segment Structures (MUST NOT CHANGE)

```rust
// UI_SCENE segment layout
#[repr(C, align(64))]
struct UISceneSegment {
    scene_id: u64,
    widget_count: u32,
    layout_type: u32,         // FLEX, GRID, ABSOLUTE
    
    // Offsets to child segments
    widget_offset: u64,       // → UI_WIDGET segments
    layout_offset: u64,       // → UI_LAYOUT segment
    style_offset: u64,        // → METADATA segment
}

// UI_WIDGET segment layout
#[repr(C, align(64))]
struct UIWidgetSegment {
    widget_type: u32,         // BUTTON, TEXT, IMAGE, etc.
    flags: u32,               // VISIBLE, ENABLED, etc.
    
    bounds: [f32; 4],         // x, y, width, height
    transform: [f32; 16],     // 4x4 matrix (GPU-ready)
    
    data_offset: u64,         // widget-specific data
    event_handler_id: u64,    // BCIB function reference
}
```

**Preservation**: All field definitions, types, comments unchanged

### 7. GPU Buffer Segment Structure (EXCEPT "directly mappable" text)

```rust
// GPU_BUFFER segment layout
#[repr(C, align(64))]
struct GPUBufferSegment {
    buffer_type: u32,         // VERTEX, INDEX, UNIFORM, STORAGE
    usage: u32,               // STATIC, DYNAMIC, STREAM
    
    element_size: u32,        // bytes per element
    element_count: u32,       // number of elements
    
    data_offset: u64,         // → actual buffer data
    stride: u32,              // for vertex buffers
    format: u32,              // data format (f32, u32, etc.)
}
```

**Preservation**: Struct definition unchanged (Critical note text changes)

### 8. Vector/Tensor Support (MUST NOT CHANGE)

```rust
// VECTOR segment layout
#[repr(C, align(64))]
struct VectorSegment {
    dtype: u32,               // F32, F64, I32, etc.
    rank: u32,                // 1D, 2D, 3D, etc.
    
    shape: [u64; 8],          // dimensions (max 8D)
    strides: [u64; 8],        // for non-contiguous data
    
    data_offset: u64,         // → raw data (64B aligned)
    data_size: u64,           // total bytes
}
```

**Preservation**: All field definitions unchanged

### 9. String Pool Entry Structure (EXCEPT diagram and text)

```rust
// String pool entry
struct StringEntry {
    offset: u32,              // from pool start
    length: u32,              // UTF-8 byte count
}
```

**Preservation**: Struct definition unchanged (diagram changes)

### 10. Alignment Rules Section (EXCEPT rule text)

**Current Rules**:
1. All offsets MUST be 64B aligned
2. All segments MUST start at 64B boundary
3. Endianness: Little-Endian ONLY

**Preservation**: Rules 2 and 3 unchanged, Rule 1 text changes

### 11. Memory Safety Contract (MUST NOT CHANGE)

**Immutability Guarantee**:
```rust
// ABDF files are IMMUTABLE after creation
// Modifications = new ABDF file + BCIB diff
```

**Concurrency Model**:
```rust
// Multiple readers: SAFE (READ_ONLY flag)
// Single writer: SAFE (new file)
// Reader + Writer: SAFE (copy-on-write)
```

**mmap Safety**:
```rust
// ABDF can be mmap'd directly:
let file = File::open("data.abdf")?;
let mmap = unsafe { Mmap::map(&file)? };
let header = unsafe { &*(mmap.as_ptr() as *const ABDFHeader) };

// Zero-copy access:
let segment = &mmap[segment_offset..segment_offset + segment_size];
```

**Preservation**: All code examples, comments unchanged

### 12. CPU ↔ GPU Bridge (MUST NOT CHANGE)

**Zero-Copy Path**:
```
ABDF File (disk)
    ↓ mmap
CPU Memory (virtual)
    ↓ GPU mapping (no copy)
GPU Memory (physical)
    ↓ shader access
GPU Compute/Render
```

**BCIB Orchestration**:
```rust
// BCIB instruction:
OP_GPU_BUFFER_CREATE {
    abdf_segment_id: 42,      // → GPU_BUFFER segment
    gpu_usage: VERTEX_BUFFER,
}

// Runtime:
// 1. Locate segment in ABDF
// 2. Map to GPU memory (zero-copy)
// 3. Return GPU handle
```

**Preservation**: All diagrams, code examples unchanged

### 13. Phase 1 Checklist (MUST NOT CHANGE)

**ABDF Core (MUST HAVE)**:
- Header parsing (magic, version, alignment)
- Segment directory iteration
- String pool access
- Offset validation (64B alignment)
- Endianness check (LE only)
- Memory contract flags

**ABDF Extended (MUST HAVE)**:
- ROW segment read
- COLUMN segment read
- VECTOR segment read (GPU-ready)
- METADATA segment read

**ABDF Hooks (PHASE 1 DEFINITION ONLY)**:
- UI_SCENE type definition
- UI_WIDGET type definition
- UI_LAYOUT type definition
- GPU_BUFFER type definition
- GPU_TEXTURE type definition (placeholder)

**Preservation**: All checklist items unchanged

### 14. Success Criteria (MUST NOT CHANGE)

**Phase 1 Done When**:
1. ✅ ABDF file can be mmap'd and parsed
2. ✅ All segments are 64B aligned
3. ✅ String pool is zero-copy accessible
4. ✅ Vector data is SIMD-ready
5. ✅ UI/GPU segment types are defined
6. ✅ Memory contract flags are enforced

**Phase 3 Ready When**:
- UI segments can be rendered (implementation)
- GPU segments can be mapped (implementation)
- BCIB can orchestrate CPU/GPU/UI (implementation)

**Preservation**: All criteria unchanged

### 15. Critical Warnings (MUST NOT CHANGE)

**❌ DO NOT**:
- Add runtime endianness conversion
- Allow unaligned segments
- Implement UI/GPU logic in Phase 1
- Break 64B alignment for "optimization"

**✅ DO**:
- Keep ABDF pure data
- Enforce alignment at write time
- Define UI/GPU types now, implement later
- Think hardware-first

**Preservation**: All warnings unchanged

### 16. References Section (EXCEPT Constitutional Note)

**References**:
- `BCIB_HARDWARE_CONTRACT.md` (next)
- `ABDF_BCIB_EXECUTION_FLOW.md` (next)
- `MINIMAL_RUNTIME_SKELETON.md` (next)

**Preservation**: Reference list unchanged (Constitutional Note text changes)

## Validation Checksum

**File**: `_ayken/specs/ABDF_HARDWARE_CONTRACT.md`  
**Baseline State**: UNFIXED (contains 7 bugs)

**Sections with ZERO changes allowed**:
- Binary Layout diagram
- Header field definitions (except checksum comment)
- Memory Contract Flags
- Segment Type constants
- UI Segment structures
- Vector/Tensor structures
- Memory Safety Contract
- CPU ↔ GPU Bridge
- Phase 1 Checklist
- Success Criteria
- Critical Warnings
- References list

**Sections with TARGETED changes only**:
1. String Pool: diagram + clarifications
2. Header Structure: checksum comment + new subsection
3. GPU Buffer Segment: Critical note text
4. Segment Directory Entry: new subsection after struct
5. Alignment Rules: rule 1 text + new rule 2
6. Constitutional Note: text replacement
7. New section: ABDF-BCIB Integration Contract

## Diff Validation Rules

After fix implementation, run:
```bash
diff -u _ayken/specs/ABDF_HARDWARE_CONTRACT.md.baseline _ayken/specs/ABDF_HARDWARE_CONTRACT.md
```

**Expected diff characteristics**:
- Exactly 7 targeted sections modified
- No changes to preserved sections
- No wording improvements outside bug fixes
- No refactoring or reorganization
- No additional clarifications beyond bug fixes

**Failure conditions**:
- Any preserved section shows changes
- More than 7 sections modified
- Structural changes to document
- Additional "improvements" added

## Baseline Complete

This baseline establishes the preservation contract. Any changes beyond the 7 targeted bug fixes constitute a violation of the bugfix scope.
