//! ABDF (Ayken Binary Data Format) Header
//!
//! Bu modül, her ABDF buffer'ının/binary dosyasının başında yer alan
//! düşük seviye header yapısını tanımlar.
//!
//! # Örnek Kullanım
//!
//! ```
//! use abdf::header::AbdfHeader;
//!
//! let mut header = AbdfHeader::new();
//! assert!(header.is_valid());
//! assert_eq!(header.segment_count, 0);
//!
//! header.increment_segment_count();
//! assert_eq!(header.segment_count, 1);
//! ```

/// ABDF header yapısı.
/// 
/// `#[repr(C)]` kullanarak C ile uyumlu, tahmin edilebilir bir layout elde ediyoruz.
/// Bu sayede:
/// - Farklı dillere/ortamlara (C, C++, Rust, Zig, vb.) köprü kurmak kolaylaşır
/// - Binary dump / hexdump üzerinde debug etmek daha öngörülebilir olur
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AbdfHeader {
    /// Magic bytes for ABDF, her zaman "ABDF" olmalıdır.
    pub magic: [u8; 4],

    /// Format version (u16). Faz 2 icin "0.2" -> 2; ileride degisebilir.
    pub version: u16,

    /// Global flag alanı:
    /// - sıkıştırma
    /// - şifreleme
    /// - özel modlar
    /// için kullanılabilir (ileride).
    pub flags: u16,

    /// Bu ABDF buffer'ında tanımlı segment sayısı.
    /// Segment descriptor listesi (segment table) ile uyumlu olmalıdır.
    pub segment_count: u32,
}

/// Magic bytes for ABDF files: "ABDF"
pub const ABDF_MAGIC: [u8; 4] = *b"ABDF";

/// Current ABDF format version (binary u16). Faz 2 icin "0.2" -> 2; format degisebilir.
pub const ABDF_VERSION: u16 = 2;

impl AbdfHeader {
    /// Varsayılan bir ABDF header oluşturur:
    /// - magic = "ABDF"
    /// - version = 2 (dokumantasyonda 0.2)
    /// - flags = 0
    /// - segment_count = 0
    pub fn new() -> Self {
        Self {
            magic: ABDF_MAGIC,
            version: ABDF_VERSION,
            flags: 0,
            segment_count: 0,
        }
    }

    /// Header'ın geçerli bir ABDF header'ı olup olmadığını kontrol eder.
    ///
    /// Şimdilik sadece `magic` alanını kontrol ediyoruz.
    /// İleride:
    /// - version aralığı
    /// - reserved alanlar
    /// da kontrol edilebilir.
    pub fn is_valid(&self) -> bool {
        self.magic == ABDF_MAGIC
    }

    /// Header içindeki segment sayısını arttırmak için yardımcı fonksiyon.
    /// Faz 1'de builder tarafından kullanılabilir.
    pub fn increment_segment_count(&mut self) {
        // saturating_add kullanılarak overflow durumunda değerin başa sarması engellenir,
        // bunun yerine u32::MAX değerinde sabit kalır.
        self.segment_count = self.segment_count.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_header_defaults() {
        let h = AbdfHeader::new();

        assert_eq!(h.magic, ABDF_MAGIC);
        assert_eq!(h.version, ABDF_VERSION);
        assert_eq!(h.flags, 0);
        assert_eq!(h.segment_count, 0);
        assert!(h.is_valid());
    }

    #[test]
    fn test_invalid_magic() {
        let mut h = AbdfHeader::new();
        h.magic = *b"XXXX";
        assert!(!h.is_valid());
    }

    #[test]
    fn test_increment_segment_count() {
        let mut h = AbdfHeader::new();
        assert_eq!(h.segment_count, 0);

        h.increment_segment_count();
        h.increment_segment_count();
        h.increment_segment_count();

        assert_eq!(h.segment_count, 3);
    }
}
