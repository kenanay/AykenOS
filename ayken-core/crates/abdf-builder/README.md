# ABDF Builder

ABDF formatında veri oluşturma ve okuma için yüksek seviyeli API.

## Özellikler

- **Builder Pattern**: Kolay ABDF buffer oluşturma
- **Decoder**: Zero-copy veri erişimi
- **String Pool**: Otomatik string interning
- **Meta-data**: Zengin segment meta-verisi
- **Type Safety**: Compile-time güvenlik

## Kullanım

### ABDF Buffer Oluşturma

```rust
use abdf_builder::AbdfBuilder;
use abdf::segment::{SegmentKind, MetaContainer};

let mut builder = AbdfBuilder::new();

// String pool'a ekle
let name_idx = builder.intern_string("users");
let type_idx = builder.intern_string("table/generic");

// Meta-data oluştur
let meta = MetaContainer {
    name_idx,
    type_idx,
    schema_idx: 0,
    permissions: 0,
    embedding_idx: 0,
};

// Segment ekle
let data = b"sample data";
builder.add_segment(SegmentKind::Tabular(meta), data);

// Buffer oluştur
let buffer = builder.build();
```

### ABDF Buffer Okuma

```rust
use abdf_builder::decode_abdf;

let view = decode_abdf(&buffer)?;

// Segment verilerine eriş
let segment_data = view.segment_data(0)?;
let segment_name = view.segment_name(0)?;
let segment_kind = view.segment_kind(0)?;
```

## API Dokümantasyonu

Detaylı API dokümantasyonu için [Core API](../../docs/api/core-api.md) dosyasına bakın.

## Lisans

AykenOS Project - Open Source