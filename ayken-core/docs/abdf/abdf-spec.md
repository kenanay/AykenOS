# ABDF Format Specification v0.1

*(Ayken Binary Data Format)*  
**Status:** Draft (Faz 1)  
**Author:** Kenan AY
**License:** Open Spec – Source Required Attribution  

## 1. Introduction

ABDF (Ayken Binary Data Format) is a **binary, self-describing, extensible, and high-performance data storage format** designed for:

- CPU-efficient data access  
- GPU-friendly memory layout  
- AI/TinyLLM metadata integration  
- Real-time UI/runtime systems  
- High-performance data pipelines  

ABDF aims to unify multiple data forms into a **single, composable binary format**, supporting:

- tabular data  
- vector/tensor data  
- UI scene graphs  
- GPU buffer data  
- metadata structures  

ABDF is a foundation for **AykenOS runtime, CLI, AI-assisted workflows, and GPU compute pipelines**.

## 2. Design Principles

ABDF is designed according to the following principles:

### 2.1 Performance

- Linear, sequential layout  
- Minimal indirection  
- Zero-copy data access whenever possible  

### 2.2 Extensibility

- New segment types may be added  
- Backward compatibility maintained  

### 2.3 Self-Describing

- Header contains offsets to all core sections  
- Segment table describes all data blocks  

### 2.4 GPU & SIMD Friendly

- Aligned data blocks  
- Dense layouts  
- Predictable access patterns  

### 2.5 AI-Aware

- Rich metadata support  
- Typed vectors/tensors  
- Segment-level type information  

## 3. Binary Layout Overview

An ABDF file is composed of **four major sections** stored contiguously:

```
+---------------------+
| HEADER              |  fixed-size
+---------------------+
| SEGMENT TABLE       |  variable-size
+---------------------+
| STRING POOL         |  variable-size
+---------------------+
| DATA SECTION        |  variable-size
|  [Segment 0 Data]   |
|  [Segment 1 Data]   |
|  ...                |
+---------------------+
```

Each section begins at an **8-byte aligned offset**.

## 4. Header Format

The ABDF header is a fixed-size, 64-byte structure:

```c
struct AbdfHeader {
    uint32 magic;                  // "ABDF" = 0x41424446
    uint16 version_major;          // format major version
    uint16 version_minor;          // format minor version

    uint32 flags;                  // global configuration flags

    uint32 segment_count;          // number of segment descriptors

    uint64 segment_table_offset;   // absolute file offset
    uint64 string_pool_offset;     // absolute file offset
    uint64 data_section_offset;    // absolute file offset

    uint64 reserved0;
    uint64 reserved1;
};
```

### 4.1 Magic

ABDF files are identified by a 32-bit constant:

```
0x41424446   // ASCII 'A','B','D','F'
```

### 4.2 Versioning

Versioning follows semantic versioning:

```
major.minor
```

Format changes that break compatibility → major increment  
Additions/extensions → minor increment  

## 5. Segment Table

Segment table is an array of **SegmentDescriptor** structures.

Length:

```
segment_count * sizeof(SegmentDescriptor)
```

## 6. SegmentDescriptor Format

```c
struct SegmentDescriptor {
    uint8  kind;      // SegmentKind
    uint8  data_type; // AbdfType
    uint16 flags;     // compression, encoding, optional bits
    uint32 name_index; // index into string pool (optional)

    uint64 offset;    // relative to data_section_offset
    uint64 length;    // in bytes

    uint64 reserved;
};
```

### 6.1 Segment Kind

| Kind | Description |
|-----:|-------------|
| 0 | Tabular data |
| 1 | UI scene graph |
| 2 | GPU buffer |
| 3+ | Reserved for future extensions |

### 6.2 Data Type

Examples:

- Scalar(I32)  
- Vector(F32)  
- Tensor(F32, rank=2)  

### 6.3 Offset Rules

```
offset is relative to data_section_offset
```

## 7. String Pool

Contains all UTF-8 strings referenced in the file.

### 7.1 Organization

```
"<str1>\0<str2>\0<str3>\0..."
```

### 7.2 Indexing

`name_index` references string index (0-based)

## 8. Data Section

Contains **raw binary payloads**, each aligned to 8 bytes.

```
Segment 0 data
Segment 1 data
Segment 2 data
...
```

No framing inside the Data Section.  

## 9. Alignment & Padding

### 9.1 Global Alignment
8-byte aligned

### 9.2 Segment Alignment
8-byte aligned

### 9.3 Padding
Zero-filled

## 10. Compression & Encoding

Not included in version 0.1.

## 11. Versioning Strategy

- Never break struct size or field order  
- Add fields only to reserved regions  
- SegmentDescriptor size must remain stable  

## 12. Integrity & Validation

Required:

- magic must match  
- offsets aligned  
- non-overlapping segments  
- zero-padding enforced  

Optional:

- checksum  
- Merkle trees  
- signatures  

## 13. Example Layout

```
0000 HEADER
0040 SEGMENT TABLE
00A0 STRING POOL
0120 DATA SECTION
...
```

## 14. Future Extensions

- Columnar storage  
- Chunked streaming  
- Sparse tensor format  
- GPU mapping  
- Encryption  

## 15. Security Considerations

- validate offsets  
- prevent overflow  
- reject overlaps  

## 16. Philosophy

ABDF unifies:

- CPU compute  
- GPU compute  
- AI inference  
- UI runtime  

As **binary, typed, structured data streams**.

## 17. Changelog

v0.1 Initial Draft  

## 18. Appendix A — Size Summary

| Structure | Size |
|----------|------|
| Header | 64 bytes |
| SegmentDescriptor | 48 bytes |
| String pool | variable |
| Data section | variable |

## 19. Appendix B — Terminology

- Segment: logical block  
- Descriptor: metadata  
- Pool: compact strings  
- Tensor: N-D array  

## 20. License & Contributions

Open spec; maintain ABI stability.

---

