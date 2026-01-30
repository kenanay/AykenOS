# ABDF Metadata Specification v0.1

_(Ayken Binary Data Format - Metadata Layer)_  
**Status:** Draft  
**Author:** Kenan AY  
**License:** Open Spec – Source Required Attribution

## 1. Overview

This document describes the **metadata layer** for ABDF (Ayken Binary Data Format). While the main ABDF specification defines the binary layout and core structures, this metadata specification defines:

- Rich type system for AI/ML data
- Semantic annotations for segments
- Schema validation rules
- Compression and encoding metadata
- AI model deployment information

## 2. Metadata Architecture

ABDF metadata is stored within the format itself using **reserved segments** and **extended descriptors**:

```
ABDF File
├── Standard segments (data)
├── Metadata segments (schemas, types, annotations)
└── String pool (includes metadata strings)
```

## 3. Type System

### 3.1 Primitive Types

| Type Code | Name | Size | Description             |
| --------- | ---- | ---- | ----------------------- |
| 0x01      | I8   | 1    | Signed 8-bit integer    |
| 0x02      | U8   | 1    | Unsigned 8-bit integer  |
| 0x03      | I16  | 2    | Signed 16-bit integer   |
| 0x04      | U16  | 2    | Unsigned 16-bit integer |
| 0x05      | I32  | 4    | Signed 32-bit integer   |
| 0x06      | U32  | 4    | Unsigned 32-bit integer |
| 0x07      | I64  | 8    | Signed 64-bit integer   |
| 0x08      | U64  | 8    | Unsigned 64-bit integer |
| 0x09      | F16  | 2    | Half-precision float    |
| 0x0A      | F32  | 4    | Single-precision float  |
| 0x0B      | F64  | 8    | Double-precision float  |
| 0x0C      | BOOL | 1    | Boolean value           |

### 3.2 Composite Types

| Type Code | Name   | Description             |
| --------- | ------ | ----------------------- |
| 0x10      | VECTOR | 1D array of primitives  |
| 0x11      | MATRIX | 2D array of primitives  |
| 0x12      | TENSOR | N-D array of primitives |
| 0x13      | STRING | UTF-8 string            |
| 0x14      | STRUCT | Custom structure        |

## 4. AI Model Metadata

### 4.1 Model Descriptor

```c
struct ModelMetadata {
    uint32 model_type;        // Neural network type
    uint32 input_dims[4];     // Input tensor dimensions
    uint32 output_dims[4];    // Output tensor dimensions
    uint32 parameter_count;   // Total parameters
    uint32 quantization;      // Quantization scheme
    uint64 checksum;          // Model integrity check
};
```

### 4.2 Model Types

| Type | Description             |
| ---- | ----------------------- |
| 0x01 | Transformer (GPT-style) |
| 0x02 | CNN (Convolutional)     |
| 0x03 | RNN/LSTM                |
| 0x04 | Custom/Hybrid           |

## 5. Compression Metadata

### 5.1 Compression Schemes

| Scheme | Code | Description          |
| ------ | ---- | -------------------- |
| None   | 0x00 | No compression       |
| LZ4    | 0x01 | Fast compression     |
| ZSTD   | 0x02 | Balanced compression |
| Custom | 0xFF | Domain-specific      |

### 5.2 Compression Descriptor

```c
struct CompressionMetadata {
    uint8  scheme;           // Compression algorithm
    uint8  level;            // Compression level (0-9)
    uint16 flags;            // Algorithm-specific flags
    uint64 original_size;    // Uncompressed size
    uint64 compressed_size;  // Compressed size
    uint32 checksum;         // Integrity check
};
```

## 6. Schema Validation

### 6.1 Schema Segment

Special segment type (kind=0xFE) containing JSON schema for validation:

```json
{
  "version": "1.0",
  "segments": {
    "model_weights": {
      "type": "tensor",
      "dtype": "f32",
      "shape": [1024, 768],
      "required": true
    },
    "tokenizer": {
      "type": "struct",
      "fields": {
        "vocab_size": "u32",
        "tokens": "string[]"
      }
    }
  }
}
```

## 7. Semantic Annotations

### 7.1 Annotation Types

| Type           | Description          |
| -------------- | -------------------- |
| `model.layer`  | Neural network layer |
| `data.feature` | Feature vector       |
| `ui.component` | UI element           |
| `gpu.buffer`   | GPU memory buffer    |

### 7.2 Annotation Format

Stored in string pool with special prefix:

```
@annotation:type:value
```

Example:

```
@model.layer:transformer.attention.0
@data.feature:embedding.token
```

## 8. Version Compatibility

### 8.1 Metadata Versioning

```c
struct MetadataHeader {
    uint16 metadata_version;  // Metadata spec version
    uint16 schema_version;    // Schema version
    uint32 compatibility;     // Compatibility flags
};
```

### 8.2 Compatibility Matrix

| Metadata v1.0 | Schema v1.0 | Schema v1.1 |
| ------------- | ----------- | ----------- |
| Reader v1.0   | ✅          | ⚠️          |
| Reader v1.1   | ✅          | ✅          |

## 9. Usage Examples

### 9.1 AI Model Package

```
Segments:
├── model_weights (tensor, f32, compressed)
├── tokenizer_vocab (string array)
├── config (struct, JSON)
└── metadata (schema + annotations)
```

### 9.2 GPU Buffer

```
Segments:
├── vertex_data (vector, f32, GPU-aligned)
├── index_data (vector, u32)
└── shader_metadata (struct)
```

## 10. Implementation Notes

- Metadata segments are optional
- Readers should gracefully handle missing metadata
- Schema validation is recommended but not required
- Annotations enable rich tooling and debugging

## 11. Future Extensions

- Binary schema format (faster than JSON)
- Cryptographic signatures for metadata
- Distributed metadata (external references)
- Real-time metadata updates

---

**ABDF Metadata v0.1**  
_Rich semantic layer for high-performance binary data_
