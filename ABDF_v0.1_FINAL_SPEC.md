# ABDF v0.1 FINAL SPECIFICATION

**Authority**: Kenan AY - Architectural Steward  
**Status**: LOCKED - Constitutional Core Document  
**Version**: 0.1  
**Date**: 2026-05-03

---

## Purpose

ABDF (AykenOS Binary Data Format) is the typed, binary, zero-copy, CPU/GPU/UI compatible data format for AykenOS.

### ABDF's Mission

1. **Store data** in a structured, validated format
2. **Make data directly addressable** without deserialization
3. **Provide common memory contract** for CPU/GPU/AI/runtime

**ABDF is NOT a JSON alternative. ABDF is a hardware-level data protocol.**

---

## Immutable Rules

| Domain | Decision |
|--------|----------|
| **Endianness** | Little-endian |
| **Pointer** | FORBIDDEN |
| **Offset** | u64 |
| **Alignment** | 64 bytes |
| **Mutability** | Read-only snapshot |
| **String** | UTF-8 string pool |
| **ABI** | C ABI / repr(C) |
| **GPU Compatibility** | Segment-based |
| **UI Support** | Render graph metadata |

These rules address the core memory contract requirements identified in previous audits, particularly around endianness, alignment, offset, and mutability.

---

## Header Structure

```c
struct AbdfHeader {
    uint32_t magic;              // "ABDF" (0x46424441)
    uint16_t major_version;      // 0
    uint16_t minor_version;      // 1
    uint32_t flags;
    uint32_t segment_count;
    uint64_t segment_table_offset;
    uint64_t metadata_offset;
    uint64_t contract_offset;
    uint64_t total_size;
    uint64_t checksum;
};
```

**Size**: 64 bytes (aligned)

---

## Segment Descriptor

```c
struct AbdfSegment {
    uint32_t type;
    uint32_t flags;
    uint64_t offset;
    uint64_t length;
    uint64_t alignment;
    uint64_t checksum;
};
```

**Size**: 40 bytes

---

## Segment Types

| Code | Type | Purpose |
|------|------|---------|
| `0x01` | META | Metadata segment |
| `0x02` | STRING_POOL | UTF-8 string pool |
| `0x03` | SCHEMA | Schema definition |
| `0x04` | ROW_DATA | Row-oriented data |
| `0x05` | COLUMN_DATA | Column-oriented data |
| `0x06` | VECTOR | Vector data |
| `0x07` | TENSOR | Tensor data |
| `0x08` | GRAPH | Graph structure |
| `0x09` | UI_SCENE | UI scene graph |
| `0x0A` | UI_NODE | UI node data |
| `0x0B` | GPU_BUFFER | GPU buffer data |
| `0x0C` | GPU_TEXTURE | GPU texture data |
| `0x0D` | GPU_SHADER_META | GPU shader metadata |
| `0x0E` | EMBEDDING_INDEX | AI embedding index |

### Forward Compatibility Rule

**When an unknown segment type is encountered, legacy readers MUST skip it.**

This is the primary rule for forward compatibility.

---

## Memory Contract

```
endianness = LE
offset_width = 64
alignment = 64
mutability = READ_ONLY
consistency = SNAPSHOT
pointer_policy = FORBIDDEN
float = IEEE754
string = UTF8
```

### Phase 1 Constraint

**No in-place updates.** If an update is required, a new ABDF snapshot is generated.

---

## GPU Compatibility

GPU-compatible segments MUST enforce these rules:

1. **64-byte alignment**
2. **Contiguous memory**
3. **No pointers**
4. **Offset + length access only**
5. **Read-only snapshot**

### GPU Segments

- `GPU_BUFFER`
- `GPU_TEXTURE`
- `GPU_SHADER_META`
- `VECTOR`
- `TENSOR`

---

## UI Data Model

**ABDF does NOT directly render to screen.** UI is a renderable data graph.

### UI Segment Types

- `UI_SCENE` - Scene graph root
- `UI_NODE` - Individual UI nodes
- `STYLE_METADATA` - Style information
- `LAYOUT_METADATA` - Layout information
- `EVENT_METADATA` - Event binding metadata

### Example Model

```
ui.scene.main
  ├─ node.chart.cpu
  ├─ node.table.process
  └─ node.graph.memory
```

This structure aligns with the `ui.*` namespace concept in Semantic CLI.

---

## Validation Rules

An ABDF reader MUST reject a file if:

1. `magic != "ABDF"`
2. `endianness != LE`
3. Segment offset is not 64-byte aligned
4. `offset + length` exceeds `total_size`
5. Structure requires pointers/fixups
6. Checksum mismatch
7. Contract block is missing

---

## Relationship to BCIB

- **ABDF** = Data format and memory contract
- **BCIB** = Instruction format and execution contract

ABDF files are consumed by BCIB instructions. The runtime validates ABDF before execution.

---

## Constitutional Compliance

This specification is subject to:

- `NON_OVERRIDABLE.md` - Memory contract violations
- `PHASES.md` - Phase-based enforcement
- Memory safety rules (no pointer, no leak, no double-free)

---

## Implementation Priority

### Phase 1 (P4.4 - Development)

**Minimum Required Segments:**

1. `META`
2. `STRING_POOL`
3. `SCHEMA`
4. `ROW_DATA` or `COLUMN_DATA`
5. `GPU_BUFFER` (stub OK)
6. `UI_SCENE` (stub OK)

### Phase 2 (P4.5 - Stabilization)

- Full GPU segment support
- UI render graph validation
- Tensor/Vector optimization

### Phase 3 (P5 - Production)

- Zero-copy validation
- Hardware-accelerated checksums
- Advanced compression

---

## Checksum Algorithm

**Phase 1**: CRC64-ECMA

**Future**: Hardware-accelerated BLAKE3

---

## File Extension

`.abdf`

---

## MIME Type

`application/x-abdf`

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-05-03 | Initial locked specification |

---

**End of ABDF v0.1 Final Specification**

Source: `ABDF_v0.1_FINAL_SPEC.md`
