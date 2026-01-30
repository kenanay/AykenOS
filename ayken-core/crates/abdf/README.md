# ABDF (Ayken Binary Data Format)

Yüksek performanslı, self-describing binary veri formatı.

## Özellikler

- **CPU ve GPU dostu**: Linear layout, minimal indirection
- **Extensible**: Yeni segment türleri eklenebilir
- **Self-describing**: Header tüm bölümlerin offsetlerini içerir
- **AI-aware**: Zengin metadata desteği
- **Type-safe**: Güçlü tip sistemi

## Kullanım

```rust
use abdf::header::AbdfHeader;
use abdf::segment::{SegmentDescriptor, SegmentKind, MetaContainer};
use abdf::types::{AbdfType, AbdfScalarType};

// Header oluştur
let header = AbdfHeader::new();

// Segment descriptor oluştur
let descriptor = SegmentDescriptor::new(0, 1024, 4096);

// Meta container oluştur
let meta = MetaContainer {
    name_idx: 0,
    type_idx: 1,
    schema_idx: 2,
    permissions: 0,
    embedding_idx: 0,
};

let segment_kind = SegmentKind::Tabular(meta);
```

## Format Spesifikasyonu

Detaylı format spesifikasyonu için [ABDF Spec](../../docs/abdf/abdf-spec.md) dosyasına bakın.

## Lisans

AykenOS Project - Open Source