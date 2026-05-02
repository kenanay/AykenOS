# ABDF Hardware-Level Contract

**Authority**: Kenan AY - Architectural Steward  
**Status**: CONSTITUTIONAL - Phase 1 Foundation  
**Version**: 1.0.0

## 🎯 Purpose

ABDF must be **hardware-ready** from day one:
- CPU: zero-copy mmap
- GPU: direct buffer mapping
- UI: native rendering pipeline
- Network: zero-serialization transfer

## 📐 Binary Layout (Immutable)

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

## 🔒 Header Structure (64 bytes)

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
    checksum: u64,            // XXH3-64 hash of bytes [64..total_size), excludes checksum field itself
    
    reserved: [u8; 16],       // future use
}
```

### Checksum Validation

**Scope**:
- **Byte range**: `[64..total_size)` - from end of header through end of file
- **Exclusion**: The `checksum` field itself (bytes 48-55 in header) is excluded from computation
- **Algorithm**: XXH3-64 (64-bit variant of XXHash3)

**Computation Rules**:
- **Checksum MUST be computed after all alignment padding is applied** - padding bytes are included in hash
- **Checksum MUST be computed on the final byte representation** - no intermediate transformations
- **All reserved fields MUST be zeroed before checksum computation** - prevents non-deterministic results from uninitialized memory
- **Checksum field itself MUST be set to zero during computation** - then replaced with computed value

**Determinism Guarantee**:
- Given identical segment data, string pool, and metadata, two independent writers MUST produce identical checksums
- Checksum computation MUST NOT depend on:
  - Writer implementation details
  - Compiler padding behavior (all padding explicit)
  - Memory initialization state (reserved fields zeroed)
  - Timestamp or random data

**Validation Failure**:
- **Parsers MUST reject** ABDF files where computed checksum does not match header checksum field
- **No partial load permitted** - checksum mismatch is a fatal error

### Memory Contract Flags (bits)

```
bit 0: READ_ONLY       - immutable after creation
bit 1: SNAPSHOT        - point-in-time consistency
bit 2: THREAD_SAFE     - concurrent read safe
bit 3: GPU_MAPPABLE    - can be mapped to GPU
bit 4: MMAP_FRIENDLY   - safe for mmap
bit 5-15: reserved
```

## 📊 Segment Directory Entry (32 bytes)

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

### Compile-Time Validation

The SegmentEntry structure MUST be validated at compile-time to guarantee binary layout invariants required by the ABDF contract.

```rust
// Static assertions to guarantee struct invariants
const _: () = assert!(core::mem::size_of::<SegmentEntry>() == 32);
const _: () = assert!(core::mem::align_of::<SegmentEntry>() == 32);
```

**Validation Guarantees**:
- **Any compiler/platform producing a different layout MUST fail compilation**
- **No runtime validation is sufficient** - layout must be enforced at compile-time
- **This guarantees**:
  - Deterministic binary layout
  - Cross-compiler consistency
  - ABI stability for mmap and zero-copy access

### Segment Types (Immutable)

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

## 🎨 UI Segment Structure (Phase 1 Hook)

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

**Critical**: UI segments are **data-only**. No rendering logic in ABDF.

## 🖥️ GPU Buffer Segment (Phase 1 Hook)

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

**Critical**: GPU buffer data is designed for **direct mapping to GPU memory** as an optimization target. Implementations MUST provide fallback to staged upload/copy on hardware, driver, OS, or runtime paths where direct mapping is not supported.

### GPU Mapping Semantics

- **Direct GPU mapping is hardware-dependent**, not guaranteed by the ABDF file alone
- **ABDF guarantees**: buffer's binary layout, alignment, offset model, and byte representation
- **Runtime/GPU backend decides**: whether the segment can be bound directly or must be staged
- **Fallback staged upload/copy MUST preserve byte-exact ABDF data representation**
- **Fallback MUST NOT perform**:
  - Format conversion
  - Endian conversion
  - Normalization
  - Reshaping
  - Compression
  - Semantic reinterpretation
- **If neither direct mapping nor byte-exact staged upload is possible**, the runtime MUST reject the GPU operation

### GPU-Ready Data Layout

```
Example: Vertex Buffer
┌────────────────────────────────────┐
│ GPUBufferSegment (64B)             │
├────────────────────────────────────┤
│ Vertex Data (64B aligned)          │
│   [x, y, z, w] (16B)               │ ← SIMD aligned
│   [r, g, b, a] (16B)               │ ← SIMD aligned
│   [u, v, _, _] (16B)               │ ← padding for alignment
│   ...                              │
└────────────────────────────────────┘
```

## 🔢 Vector/Tensor Support (AI/GPU)

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

**Critical**: Vector data is **contiguous and aligned** for SIMD/GPU.

## 🧵 String Pool (Zero-Copy)

```rust
// String pool entry
struct StringEntry {
    offset: u32,              // from pool start
    length: u32,              // UTF-8 byte count
}

// String pool layout
┌────────────────────────────────────┐
│ Entry Count (u32)                  │
├────────────────────────────────────┤
│ StringEntry[0]                     │
│ StringEntry[1]                     │
│ ...                                │
├────────────────────────────────────┤
│ UTF-8 Data (offset + length)       │
│ Raw UTF-8 bytes without terminators│
└────────────────────────────────────┘
```

### String Representation Rules

**Core Model**:
- Strings are represented as `offset + length` pairs
- **No null terminators required** - length field defines string boundaries
- **Null bytes allowed** ONLY as payload data if explicitly included in length field

**Edge Cases**:
- **Empty strings are valid** (length = 0)
- **Offset must point to valid position** within string pool even for empty strings (typically points to end of pool or any valid byte)

**Bounds Safety**:
- **Bounds check required**: `offset + length` MUST NOT exceed `string_pool_size`
- **Parsers MUST reject** ABDF files where any StringEntry violates bounds
- **Validation failure behavior**: Parser returns error, file is rejected (no partial load)

**Encoding Determinism**:
- **UTF-8 encoding required** - no other encodings permitted
- **No normalization applied** - parsers MUST treat string data as raw byte sequences
- **String equality is byte-level equality** - not semantic or Unicode normalization-aware
- **Rationale**: Ensures deterministic checksums and reproducible file generation

**Cross-Segment Consistency**:
- **Unified representation**: All STRING types in metadata segments MUST reference string pool using offset+length
- **No inline strings**: String data MUST NOT be embedded directly in other segments
- **BCIB references**: BCIB instructions reference strings via `string_index` (index into StringEntry array), not raw offsets

## ⚙️ Alignment Rules (IMMUTABLE)

1. **ABDF segment start offsets MUST be 64B aligned**
   - Format-level requirement
   - Reason: SIMD/AVX-512 access, GPU cache-line optimization, deterministic segment addressing

2. **Segment size does NOT need to be a multiple of 64B**
   - Only the segment start offset is alignment-constrained
   - Writer MUST add padding before the next segment when needed
   - Padding bytes are part of the final byte representation and are included in checksum computation

3. **mmap base alignment is an OS/runtime concern**
   - ABDF does not require file offsets to equal OS page alignment
   - mmap base address SHOULD be page-aligned where required by the host OS/runtime
   - Typical page alignment is 4KB, but this is not part of the ABDF binary format

4. **Endianness: Little-Endian ONLY**
   - No runtime conversion
   - Cross-platform writers MUST emit LE bytes

### Alignment Versioning Note

- ABDF v1.0 defines 64B segment start alignment
- Earlier ABDF v0.1 implementations used 8B alignment
- Parsers SHOULD detect alignment policy through the `version` and `alignment` header fields
- Parsers MUST NOT silently reinterpret v1.0 files using v0.1 alignment rules

## 🔗 ABDF-BCIB Integration Contract

BCIB may orchestrate ABDF execution, but it MUST NOT violate ABDF's pointer-free memory contract.

### Pointer-Free Guarantee

- **BCIB MUST NOT store raw pointers to ABDF memory**
- **BCIB MUST NOT persist process-local, mmap-local, GPU-local, or runtime-local addresses**
- **BCIB MUST NOT require pointer fix-up** during load, replay, migration, or validation

### Stable Identifiers Only

BCIB may reference ABDF only through stable, bounds-checked identifiers:

- `object_id` — stable ABDF object identifier
- `segment_index` — index into the ABDF segment directory
- `string_index` — index into the ABDF StringEntry array
- `offset + length` — explicit byte range validated against the referenced segment or string pool

### No Layout Reinterpretation

- **BCIB MUST NOT reinterpret ABDF memory outside the ABDF contract**
- **BCIB MUST NOT infer alternate struct layouts, padding rules, endian rules, or segment schemas**
- **BCIB MUST use ABDF-declared segment metadata and validation rules** as the only layout authority

### Memory Safety Preservation

- **BCIB operations MUST preserve ABDF's pointer-free memory model**
- **Any BCIB instruction referencing an invalid object, segment, string entry, or byte range MUST fail closed**
- **No partial execution is permitted** after ABDF reference validation failure

## 🔐 Memory Safety Contract

### Immutability Guarantee

```rust
// ABDF files are IMMUTABLE after creation
// Modifications = new ABDF file + BCIB diff
```

### Concurrency Model

```rust
// Multiple readers: SAFE (READ_ONLY flag)
// Single writer: SAFE (new file)
// Reader + Writer: SAFE (copy-on-write)
```

### mmap Safety

```rust
// ABDF can be mmap'd directly:
let file = File::open("data.abdf")?;
let mmap = unsafe { Mmap::map(&file)? };
let header = unsafe { &*(mmap.as_ptr() as *const ABDFHeader) };

// Zero-copy access:
let segment = &mmap[segment_offset..segment_offset + segment_size];
```

## 🚀 CPU ↔ GPU Bridge

### Zero-Copy Path

```
ABDF File (disk)
    ↓ mmap
CPU Memory (virtual)
    ↓ GPU mapping (no copy)
GPU Memory (physical)
    ↓ shader access
GPU Compute/Render
```

### BCIB Orchestration

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

## 📋 Phase 1 Checklist

### ABDF Core (MUST HAVE)
- [ ] Header parsing (magic, version, alignment)
- [ ] Segment directory iteration
- [ ] String pool access
- [ ] Offset validation (64B alignment)
- [ ] Endianness check (LE only)
- [ ] Memory contract flags

### ABDF Extended (MUST HAVE)
- [ ] ROW segment read
- [ ] COLUMN segment read
- [ ] VECTOR segment read (GPU-ready)
- [ ] METADATA segment read

### ABDF Hooks (PHASE 1 DEFINITION ONLY)
- [ ] UI_SCENE type definition
- [ ] UI_WIDGET type definition
- [ ] UI_LAYOUT type definition
- [ ] GPU_BUFFER type definition
- [ ] GPU_TEXTURE type definition (placeholder)

**Critical**: Hooks are **type definitions only**. No implementation in Phase 1.

## 🎯 Success Criteria

### Phase 1 Done When:
1. ✅ ABDF file can be mmap'd and parsed
2. ✅ All segments are 64B aligned
3. ✅ String pool is zero-copy accessible
4. ✅ Vector data is SIMD-ready
5. ✅ UI/GPU segment types are defined
6. ✅ Memory contract flags are enforced

### Phase 3 Ready When:
- UI segments can be rendered (implementation)
- GPU segments can be mapped (implementation)
- BCIB can orchestrate CPU/GPU/UI (implementation)

## 🔥 Critical Warnings

### ❌ DO NOT
- Add runtime endianness conversion
- Allow unaligned segments
- Implement UI/GPU logic in Phase 1
- Break 64B alignment for "optimization"

### ✅ DO
- Keep ABDF pure data
- Enforce alignment at write time
- Define UI/GPU types now, implement later
- Think hardware-first

## 📚 References

- `BCIB_HARDWARE_CONTRACT.md` (next)
- `ABDF_BCIB_EXECUTION_FLOW.md` (next)
- `MINIMAL_RUNTIME_SKELETON.md` (next)

---

**Constitutional Note**:

### Immutable Core Contract

The following ABDF core contract elements are **immutable** in Phase 1:

- Header structure and field order
- Endianness policy: Little-Endian only
- Alignment policy: 64B segment alignment
- Pointer-free memory model
- Offset-based reference model
- Existing segment type identifiers and binary layouts

Changes to these elements require architectural review and a major version decision.

### Versioned Extensions

The following may evolve through versioned extension **without breaking** the immutable core contract:

- New segment types
- New segment flags
- New metadata schemas
- New optional extension records

**Extension Rules**:
- **Existing segment types MUST NOT change binary layout across versions**
- **New segment types MUST use new type identifiers**
- **Existing type identifiers MUST NOT be reused or redefined**
- **Unknown segment types MUST remain safely skippable** through `offset + size`
