//! ABDF Builder Crate (v0.2)
//!
//! Bu crate, ABDF formatındaki binary buffer'ları oluşturmak (encode)
//! ve sonradan geri okumak (decode) için yüksek seviyeli bir API sağlar.
//!
//! # Örnek Kullanım
//!
//! ```
//! use abdf_builder::{AbdfBuilder, decode_abdf};
//! use abdf::segment::{SegmentKind, MetaContainer};
//!
//! // Builder oluştur
//! let mut builder = AbdfBuilder::new();
//!
//! // String pool'a ekle
//! let name_idx = builder.intern_string("test");
//! let type_idx = builder.intern_string("table");
//!
//! // Meta container oluştur
//! let meta = MetaContainer {
//!     name_idx,
//!     type_idx,
//!     schema_idx: 0,
//!     permissions: 0,
//!     embedding_idx: 0,
//! };
//!
//! // Segment ekle
//! let data = b"test data";
//! builder.add_segment(SegmentKind::Tabular(meta), data);
//!
//! // Build ve decode
//! let buffer = builder.build();
//! let view = decode_abdf(&buffer).unwrap();
//! assert_eq!(view.segments.len(), 1);
//! ```

use abdf::header::{AbdfHeader, ABDF_VERSION};
use abdf::segment::{MetaContainer, SegmentDescriptor, SegmentKind};
use std::convert::TryFrom;

// --- Serialization Structures ---

/// `SegmentKind` enum'unun diskte saklanabilir (serializable) hali.
/// `#[repr(C)]` olduğu için byte-level kopyalamaya uygundur.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RawSegmentKind {
    /// Hangi tür segment olduğunu belirten etiket.
    /// 0: Raw, 1: Tabular, 2: Log, 3: Text, 4: UiScene, 5: GpuBuffer
    kind_tag: u32,
    /// `MetaContainer`'dan gelen name_idx alanı.
    name_idx: u32,
    /// `MetaContainer`'dan gelen type_idx alanı.
    type_idx: u32,
    /// `MetaContainer`'dan gelen schema_idx alanı.
    schema_idx: u32,
    /// `MetaContainer`'dan gelen permissions alanı.
    permissions: u64,
    /// `MetaContainer`'dan gelen embedding_idx alanı.
    embedding_idx: u32,
    _padding: u32, // 8-byte alignment için (gelecekte ek meta alanları için ayrılmıştır)
}

// Kind tags for RawSegmentKind
const KIND_TAG_RAW: u32 = 0;
const KIND_TAG_TABULAR: u32 = 1;
const KIND_TAG_LOG: u32 = 2;
const KIND_TAG_TEXT: u32 = 3;
const KIND_TAG_UI_SCENE: u32 = 4;
const KIND_TAG_GPU_BUFFER: u32 = 5;

/// `SegmentKind` -> `RawSegmentKind` dönüşümü (serialization için).
impl From<&SegmentKind> for RawSegmentKind {
    fn from(kind: &SegmentKind) -> Self {
        let mut raw = RawSegmentKind {
            kind_tag: 0,
            name_idx: 0,
            type_idx: 0,
            schema_idx: 0,
            permissions: 0,
            embedding_idx: 0,
            _padding: 0,
        };

        let (tag, meta_opt) = match kind {
            SegmentKind::Raw => (KIND_TAG_RAW, None),
            SegmentKind::Tabular(m) => (KIND_TAG_TABULAR, Some(m)),
            SegmentKind::Log(m) => (KIND_TAG_LOG, Some(m)),
            SegmentKind::Text(m) => (KIND_TAG_TEXT, Some(m)),
            SegmentKind::UiScene(m) => (KIND_TAG_UI_SCENE, Some(m)),
            SegmentKind::GpuBuffer(m) => (KIND_TAG_GPU_BUFFER, Some(m)),
        };

        raw.kind_tag = tag;
        if let Some(meta) = meta_opt {
            raw.name_idx = meta.name_idx;
            raw.type_idx = meta.type_idx;
            raw.schema_idx = meta.schema_idx;
            raw.permissions = meta.permissions;
            raw.embedding_idx = meta.embedding_idx;
        }
        raw
    }
}

/// `RawSegmentKind` -> `SegmentKind` dönüşümü (deserialization için).
impl TryFrom<&RawSegmentKind> for SegmentKind {
    type Error = DecodeError;

    fn try_from(raw: &RawSegmentKind) -> Result<Self, Self::Error> {
        let meta = MetaContainer {
            name_idx: raw.name_idx,
            type_idx: raw.type_idx,
            schema_idx: raw.schema_idx,
            permissions: raw.permissions,
            embedding_idx: raw.embedding_idx,
        };

        match raw.kind_tag {
            KIND_TAG_RAW => Ok(SegmentKind::Raw),
            KIND_TAG_TABULAR => Ok(SegmentKind::Tabular(meta)),
            KIND_TAG_LOG => Ok(SegmentKind::Log(meta)),
            KIND_TAG_TEXT => Ok(SegmentKind::Text(meta)),
            KIND_TAG_UI_SCENE => Ok(SegmentKind::UiScene(meta)),
            KIND_TAG_GPU_BUFFER => Ok(SegmentKind::GpuBuffer(meta)),
            _ => Err(DecodeError::InvalidSegmentKindTag),
        }
    }
}


/// 8 byte alignment helper.
fn align_to8(len: usize) -> usize {
    (len + 7) & !7
}

/// Yeni nesil ABDF builder (v0.2).
/// Meta-veri tablosunu destekler.
#[derive(Debug)]
pub struct AbdfBuilder {
    header: AbdfHeader,
    segments: Vec<SegmentDescriptor>,
    meta_table: Vec<SegmentKind>,
    string_pool: Vec<String>,
    data: Vec<u8>,
}

impl Default for AbdfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AbdfBuilder {
    /// Yeni bir boş ABDF builder oluşturur.
    pub fn new() -> Self {
        let mut header = AbdfHeader::new();
        header.version = ABDF_VERSION; // Ensure correct version
        Self {
            header,
            segments: Vec::new(),
            meta_table: Vec::new(),
            string_pool: Vec::new(),
            data: Vec::new(),
        }
    }

    /// String pool'a bir string ekler veya varsa index'ini döner.
    pub fn intern_string<S: AsRef<str>>(&mut self, s: S) -> u32 {
        let s_ref = s.as_ref();
        if let Some(pos) = self.string_pool.iter().position(|r| r == s_ref) {
            return pos as u32;
        }
        let idx = self.string_pool.len();
        self.string_pool.push(s_ref.to_string());
        idx as u32
    }
    
    /// Yeni bir segment ve ilişkili meta-veriyi ekler.
    pub fn add_segment(&mut self, kind: SegmentKind, bytes: &[u8]) -> u32 {
        // 1. Meta-veriyi meta tablosuna ekle ve index'ini al (meta_idx).
        let meta_idx = self.meta_table.len() as u32;
        self.meta_table.push(kind);

        // 2. Veriyi data bölümüne ekle ve offset/length'i al.
        let offset = self.data.len() as u64;
        let length = bytes.len() as u64;
        self.data.extend_from_slice(bytes);
        // Data alignment'ı burada yapmıyoruz, build aşamasında genel hizalama var.

        // 3. Segment descriptor'ı oluştur ve listeye ekle.
        let desc = SegmentDescriptor::new(meta_idx, offset, length);
        self.segments.push(desc);

        // 4. Header'daki segment sayısını güncelle.
        self.header.increment_segment_count();

        // Segment'in ana segment tablosundaki index'ini döndür.
        (self.segments.len() - 1) as u32
    }

    /// Builder'dan çalışır bir ABDF binary buffer'ı üretir.
    ///
    /// Layout (v0.2):
    /// \[Header\] \[Segment Table\] \[Meta Table\] \[String Pool\] \[Data Section\]
    pub fn build(mut self) -> Vec<u8> {
        use std::{mem, ptr};

        // 1. Header'daki segment sayısını doğrula.
        self.header.segment_count = self.segments.len() as u32;

        // 2. Tüm bölümlerin boyutlarını hesapla.
        let header_size = mem::size_of::<AbdfHeader>();

        let seg_desc_size = mem::size_of::<SegmentDescriptor>();
        let segment_table_size = self.segments.len() * seg_desc_size;
        
        let raw_kind_size = mem::size_of::<RawSegmentKind>();
        let meta_table_size = self.meta_table.len() * raw_kind_size;

        // String pool'u byte buffer'ına çevir (her string null-terminated).
        let string_pool_bytes: Vec<u8> = self
            .string_pool
            .iter()
            .flat_map(|s| s.bytes().chain(std::iter::once(0)))
            .collect();
        let string_pool_size = string_pool_bytes.len();
        
        let data_size = self.data.len();

        // 3. Toplam buffer boyutunu hizalamaları dikkate alarak hesapla.
        let total_size = align_to8(header_size)
            + align_to8(segment_table_size)
            + align_to8(meta_table_size)
            + align_to8(string_pool_size)
            + align_to8(data_size);

        let mut buf = vec![0u8; total_size];

        // 4. Bölümleri sırayla buffer'a yaz.
        
        // Header
        unsafe {
            ptr::copy_nonoverlapping(&self.header as *const _ as *const u8, buf.as_mut_ptr(), header_size);
        }
        let mut current_offset = align_to8(header_size);

        // Segment Table
        unsafe {
            let mut ptr = buf.as_mut_ptr().add(current_offset);
            for seg in &self.segments {
                ptr::copy_nonoverlapping(seg as *const _ as *const u8, ptr, seg_desc_size);
                ptr = ptr.add(seg_desc_size);
            }
        }
        current_offset += align_to8(segment_table_size);

        // Meta Table
        unsafe {
            let mut ptr = buf.as_mut_ptr().add(current_offset);
            for kind in &self.meta_table {
                let raw_kind = RawSegmentKind::from(kind);
                ptr::copy_nonoverlapping(&raw_kind as *const _ as *const u8, ptr, raw_kind_size);
                ptr = ptr.add(raw_kind_size);
            }
        }
        current_offset += align_to8(meta_table_size);
        
        // String Pool
        buf[current_offset..current_offset + string_pool_size].copy_from_slice(&string_pool_bytes);
        let data_offset = current_offset + align_to8(string_pool_size);

        // Data Section
        buf[data_offset..data_offset + data_size].copy_from_slice(&self.data);

        buf
    }
}

/// Decode edilmiş ABDF buffer'ı için salt-okunur bir görünüm (v0.2).
#[derive(Debug)]
pub struct AbdfView<'a> {
    pub header: &'a AbdfHeader,
    pub segments: &'a [SegmentDescriptor],
    pub meta_table: Vec<SegmentKind>,
    pub string_pool: Vec<String>,
    pub data_section: &'a [u8],
}

impl<'a> AbdfView<'a> {
    /// Belirli bir segmentin data slice'ını döner.
    pub fn segment_data(&self, segment_idx: usize) -> Option<&'a [u8]> {
        let seg = self.segments.get(segment_idx)?;
        let start = seg.offset as usize;
        let end = start + seg.length as usize;
        self.data_section.get(start..end)
    }

    /// Bir segmentin `SegmentKind` bilgisine erişir.
    pub fn segment_kind(&self, segment_idx: usize) -> Option<&SegmentKind> {
        let seg = self.segments.get(segment_idx)?;
        self.meta_table.get(seg.meta_idx as usize)
    }

    /// Bir string index'ini kullanarak string pool'dan string'e erişir.
    pub fn get_string(&self, string_idx: u32) -> Option<&str> {
        self.string_pool.get(string_idx as usize).map(|s| s.as_str())
    }
    
    /// Bir segmentin adını (string pool'dan) döner.
    pub fn segment_name(&self, segment_idx: usize) -> Option<&str> {
        let kind = self.segment_kind(segment_idx)?;
        let meta = match kind {
            SegmentKind::Tabular(m) | SegmentKind::Log(m) | SegmentKind::Text(m) |
            SegmentKind::UiScene(m) | SegmentKind::GpuBuffer(m) => Some(m),
            SegmentKind::Raw => None,
        }?;
        self.get_string(meta.name_idx)
    }
}

/// Decode hataları için hata türü.
#[derive(Debug)]
pub enum DecodeError {
    BufferTooSmall,
    InvalidMagic,
    UnsupportedVersion,
    InvalidSegmentKindTag,
    CorruptLayout,
    MetaTableSizeMismatch,
    StringPoolFormat,
    Utf8(std::str::Utf8Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::BufferTooSmall => write!(f, "Buffer too small"),
            DecodeError::InvalidMagic => write!(f, "Invalid magic number"),
            DecodeError::UnsupportedVersion => write!(f, "Unsupported version"),
            DecodeError::InvalidSegmentKindTag => write!(f, "Invalid segment kind tag"),
            DecodeError::CorruptLayout => write!(f, "Corrupt layout"),
            DecodeError::MetaTableSizeMismatch => write!(f, "Meta table size mismatch"),
            DecodeError::StringPoolFormat => write!(f, "String pool format error"),
            DecodeError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

/// Bir ABDF buffer'ını decode edip `AbdfView` döner (v0.2).
pub fn decode_abdf(buf: &[u8]) -> Result<AbdfView<'_>, DecodeError> {
    use std::{mem, slice, str};

    let file_len = buf.len();
    let header_size = mem::size_of::<AbdfHeader>();

    if file_len < header_size {
        return Err(DecodeError::BufferTooSmall);
    }

    // 1) Header'ı oku ve doğrula.
    let header: &AbdfHeader = unsafe { &*(buf.as_ptr() as *const AbdfHeader) };
    if !header.is_valid() {
        return Err(DecodeError::InvalidMagic);
    }
    if header.version != ABDF_VERSION {
        return Err(DecodeError::UnsupportedVersion);
    }
    
    let mut current_offset = align_to8(header_size);

    // 2) Segment Table'ı oku.
    let seg_count = header.segment_count as usize;
    let seg_size = mem::size_of::<SegmentDescriptor>();
    let seg_table_size = seg_count
        .checked_mul(seg_size)
        .ok_or(DecodeError::CorruptLayout)?;
    if current_offset + seg_table_size > file_len {
        return Err(DecodeError::CorruptLayout);
    }
    let segments: &[SegmentDescriptor] = unsafe {
        slice::from_raw_parts(buf.as_ptr().add(current_offset) as *const SegmentDescriptor, seg_count)
    };
    current_offset += align_to8(seg_table_size);

    // 3) Meta Table'ı oku.
    let raw_kind_size = mem::size_of::<RawSegmentKind>();
    let meta_table_size = seg_count
        .checked_mul(raw_kind_size)
        .ok_or(DecodeError::CorruptLayout)?; // Her segment için bir meta vardır.
    if current_offset + meta_table_size > file_len {
        return Err(DecodeError::CorruptLayout);
    }
    let raw_kinds: &[RawSegmentKind] = unsafe {
        slice::from_raw_parts(buf.as_ptr().add(current_offset) as *const RawSegmentKind, seg_count)
    };
    let meta_table: Vec<SegmentKind> = raw_kinds
        .iter()
        .map(SegmentKind::try_from)
        .collect::<Result<_, _>>()?;
    if meta_table.len() != seg_count {
        return Err(DecodeError::MetaTableSizeMismatch);
    }
    current_offset += align_to8(meta_table_size);

    // 4) String Pool ve Data Section'ı ayır.
    // Bu mantık, `build` fonksiyonundaki sıralı yerleşime dayanır.
    // Data bölümünün boyutu, segmentlerin en büyük `offset + length`'inden anlaşılır.
    let mut data_section_content_size: u64 = 0;
    for s in segments {
        let end = s
            .offset
            .checked_add(s.length)
            .ok_or(DecodeError::CorruptLayout)?;
        if end > data_section_content_size {
            data_section_content_size = end;
        }
    }

    let data_section_total_size = align_to8(
        usize::try_from(data_section_content_size).map_err(|_| DecodeError::CorruptLayout)?,
    );
    if file_len < data_section_total_size {
        return Err(DecodeError::CorruptLayout);
    }
	let data_section_start = file_len - data_section_total_size;
	let string_pool_end = data_section_start;

	if current_offset > string_pool_end {
		return Err(DecodeError::CorruptLayout);
	}

	let string_pool_bytes = &buf[current_offset..string_pool_end];
    let data_section = &buf[data_section_start..];

    // Segmentlerin offset+length'i data_section sınırını aşmamalı.
    for seg in segments {
        let end = seg
            .offset
            .checked_add(seg.length)
            .ok_or(DecodeError::CorruptLayout)? as usize;
        if end > data_section.len() {
            return Err(DecodeError::CorruptLayout);
        }
    }

    // 5) String Pool'u parse et.
    let mut string_pool = Vec::new();
    if !string_pool_bytes.is_empty() {
        // Son null byte'ı handle etmek için `trim_end`
        for s in string_pool_bytes.split(|&b| b == 0).filter(|s| !s.is_empty()) {
            let decoded_str = str::from_utf8(s).map_err(DecodeError::Utf8)?;
            string_pool.push(decoded_str.to_string());
        }
    }
    
    Ok(AbdfView {
        header,
        segments,
        meta_table,
        string_pool,
        data_section,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use abdf::segment::MetaContainer;

    #[test]
    fn test_v2_build_and_decode_cycle() {
        let mut builder = AbdfBuilder::new();

        // Stringleri intern et
        let users_name = builder.intern_string("users");
        let table_type = builder.intern_string("table/generic");
        let schema_str = builder.intern_string("id:u64,name:string");
        
        let syslog_name = builder.intern_string("syslog");
        let log_type = builder.intern_string("log/syslog");
        let log_schema = builder.intern_string("ts:u64,level:u8,msg:string");

        // Segment 1: Tabular data
        let user_data = b"record1_data_here_record2_data_here";
        let user_meta = MetaContainer {
            name_idx: users_name,
            type_idx: table_type,
            schema_idx: schema_str,
            permissions: 0,
            embedding_idx: 0,
        };
        builder.add_segment(SegmentKind::Tabular(user_meta), user_data);
        
        // Segment 2: Log data
        let log_data = b"some_log_entries";
         let log_meta = MetaContainer {
            name_idx: syslog_name,
            type_idx: log_type,
            schema_idx: log_schema,
            permissions: 0,
            embedding_idx: 0,
        };
        builder.add_segment(SegmentKind::Log(log_meta), log_data);

        // Segment 3: Raw data
        let raw_data = &[0xDE, 0xAD, 0xBE, 0xEF];
        builder.add_segment(SegmentKind::Raw, raw_data);

        // Build
        let buffer = builder.build();

        // Decode
        let view = decode_abdf(&buffer).expect("Decode failed");

        // Assertions
        assert_eq!(view.header.version, ABDF_VERSION);
        assert_eq!(view.segments.len(), 3);
        assert_eq!(view.meta_table.len(), 3);

        // Check Segment 1 (Users)
        assert_eq!(view.segment_name(0), Some("users"));
        assert_eq!(view.segment_data(0), Some(user_data.as_slice()));
        if let Some(SegmentKind::Tabular(meta)) = view.segment_kind(0) {
            assert_eq!(view.get_string(meta.schema_idx), Some("id:u64,name:string"));
        } else {
            panic!("Segment 0 should be tabular");
        }

        // Check Segment 2 (Syslog)
        assert_eq!(view.segment_name(1), Some("syslog"));
        assert_eq!(view.segment_data(1), Some(log_data.as_slice()));
        assert!(matches!(view.segment_kind(1), Some(SegmentKind::Log(_))));
        
        // Check Segment 3 (Raw)
        assert_eq!(view.segment_name(2), None); // Raw segment has no meta container
        assert_eq!(view.segment_data(2), Some(raw_data.as_slice()));
        assert!(matches!(view.segment_kind(2), Some(SegmentKind::Raw)));
    }
}
