# Ayken Core API Documentation

## ABDF API

### Builder API

```rust
use abdf_builder::{AbdfBuilder, AbdfView, decode_abdf};
use abdf::segment::{SegmentKind, MetaContainer};

// Create builder
let mut builder = AbdfBuilder::new();

// Add strings to pool
let name_idx = builder.intern_string("users");
let type_idx = builder.intern_string("table/generic");

// Create meta container
let meta = MetaContainer {
    name_idx,
    type_idx,
    schema_idx: 0,
    permissions: 0,
    embedding_idx: 0,
};

// Add segment
let data = b"sample data";
builder.add_segment(SegmentKind::Tabular(meta), data);

// Build buffer
let buffer = builder.build();

// Decode buffer
let view = decode_abdf(&buffer)?;
```

### Reader API

```rust
// Access segment data
let segment_data = view.segment_data(0)?;

// Get segment metadata
let segment_kind = view.segment_kind(0)?;

// Get string from pool
let name = view.get_string(name_idx)?;
```

## BCIB API

### Instruction Creation

```rust
use bcib::{BcibInstruction, BcibOpcode};

// Create instructions
let select = BcibInstruction::new(
    BcibOpcode::CtxSelect, 
    0, // flags
    0, // container_id
    0, 0 // unused args
);

let query = BcibInstruction::new(
    BcibOpcode::DataQuery,
    0,
    0, // query string index
    0, 0
);
```

### Header Management

```rust
use bcib::BcibHeader;

let mut header = BcibHeader::new();
header.instruction_count = 2;
header.string_pool_offset = 32; // after header + instructions
```

## Type System

### ABDF Types

```rust
use abdf::types::{AbdfType, AbdfScalarType};

// Scalar types
let int_type = AbdfType::Scalar(AbdfScalarType::I32);
let float_type = AbdfType::Scalar(AbdfScalarType::F64);

// Complex types
let vector_type = AbdfType::Vector(AbdfScalarType::F32);
let tensor_type = AbdfType::Tensor {
    base: AbdfScalarType::F32,
    rank: 2,
};
```

## Error Handling

### Decode Errors

```rust
use abdf_builder::DecodeError;

match decode_abdf(&buffer) {
    Ok(view) => { /* use view */ },
    Err(DecodeError::BufferTooSmall) => { /* handle */ },
    Err(DecodeError::InvalidMagic) => { /* handle */ },
    Err(DecodeError::UnsupportedVersion) => { /* handle */ },
    // ... other errors
}
```

## Best Practices

1. **Always validate headers** before processing
2. **Use string interning** for repeated strings
3. **Align data** to 8-byte boundaries
4. **Check buffer bounds** when accessing segments
5. **Handle version compatibility** gracefully