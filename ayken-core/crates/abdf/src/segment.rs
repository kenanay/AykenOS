//! ABDF Segment Model (v0.2)
//!
//! Bu modül, ABDF buffer'ı içindeki "segment" ve "container"
//! kavramlarını tanımlar. Meta-veri ve fiziksel veri ayrılmıştır.
//!
//! # Örnek Kullanım
//!
//! ```
//! use abdf::segment::{SegmentDescriptor, SegmentKind, MetaContainer};
//!
//! // Meta container oluştur
//! let meta = MetaContainer {
//!     name_idx: 0,
//!     type_idx: 1,
//!     schema_idx: 2,
//!     permissions: 0,
//!     embedding_idx: 0,
//! };
//!
//! // Segment kind oluştur
//! let kind = SegmentKind::Tabular(meta);
//! assert!(kind.is_tabular());
//!
//! // Segment descriptor oluştur
//! let desc = SegmentDescriptor::new(0, 1024, 4096);
//! assert_eq!(desc.offset, 1024);
//! assert_eq!(desc.length, 4096);
//! ```

/// Bir veri konteynerinin meta-verilerini tutan genel yapı.
/// İndeksler (idx), string pool'daki karşılıklarına işaret eder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaContainer {
    /// Konteynerin adı (örn: "users", "system_log")
    pub name_idx: u32,
    /// Konteynerin tipi (örn: "table/sql", "log/syslog")
    pub type_idx: u32,
    /// Veri şeması (örn: "id:int,name:string" veya JSON şeması)
    pub schema_idx: u32,
    /// İzinler (ileride kullanılacak - placeholder)
    pub permissions: u64,
    /// Anlamsal embedding verisi indeksi (ileride kullanılacak)
    pub embedding_idx: u32,
}

/// ABDF'de hangi tür segment olduğunu belirten enum (v0.2).
/// Bu yapı, segmentin anlamsal türünü ve ilişkili meta-verisini içerir.
/// Bu enum'lar bir "Meta Tablosu"nda saklanır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmentKind {
    /// Zengin meta-veriye sahip konteyner bazlı segmentler.
    Tabular(MetaContainer),
    Log(MetaContainer),
    Text(MetaContainer),
    UiScene(MetaContainer),
    GpuBuffer(MetaContainer),

    /// Basit, meta-verisiz ham byte verisi.
    Raw,
}

/// ABDF segment descriptor (v0.2).
///
/// Bu yapı, bir segment verisinin buffer içindeki fiziksel konumunu
/// ve meta-verisine olan bağlantısını belirtir.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentDescriptor {
    /// Meta tablosundaki `SegmentKind`'e işaret eden indeks.
    pub meta_idx: u32,

    /// Segment verisinin buffer içindeki byte offset'i.
    pub offset: u64,

    /// Segment verisinin byte cinsinden uzunluğu.
    pub length: u64,
}

impl SegmentDescriptor {
    /// Yeni bir segment descriptor oluşturur.
    pub fn new(meta_idx: u32, offset: u64, length: u64) -> Self {
        Self {
            meta_idx,
            offset,
            length,
        }
    }
}

impl SegmentKind {
    /// Bu segmentin tabular veri taşıyıp taşımadığını söyler.
    pub fn is_tabular(&self) -> bool {
        matches!(self, Self::Tabular(_))
    }

    /// Bu segment bir UI sahnesi mi?
    pub fn is_ui_scene(&self) -> bool {
        matches!(self, Self::UiScene(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_descriptor_v2() {
        let descriptor = SegmentDescriptor::new(
            5, // 5. meta kaydına işaret ediyor.
            1024, 4096,
        );

        assert_eq!(descriptor.meta_idx, 5);
        assert_eq!(descriptor.offset, 1024);
        assert_eq!(descriptor.length, 4096);
    }

    #[test]
    fn create_metacontainer_and_kind() {
        let meta = MetaContainer {
            name_idx: 0,   // "users"
            type_idx: 1,   // "table/generic"
            schema_idx: 2, // "id:int,name:string"
            permissions: 0,
            embedding_idx: 0,
        };

        let kind = SegmentKind::Tabular(meta);

        assert!(kind.is_tabular());
        if let SegmentKind::Tabular(m) = kind {
            assert_eq!(m.name_idx, 0);
            assert_eq!(m.schema_idx, 2);
        } else {
            panic!("Expected Tabular segment");
        }
    }
}
