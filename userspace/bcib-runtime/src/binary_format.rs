/// BCIB v3 binary header and section layout parser.
///
/// Implements the on-disk/in-memory binary format defined in design.md:
///
/// ```text
/// ┌─────────────────────────────────────────────────────┐
/// │  Header (16 bytes)                                  │
/// │    magic:         [u8; 4]  = b"BCIB"                │
/// │    version:       u16      = 0x0003 (v3)            │
/// │    flags:         u16                               │
/// │    section_count: u16                               │
/// │    reserved:      [u8; 4]                           │
/// ├─────────────────────────────────────────────────────┤
/// │  Section Table (section_count × 8 bytes)            │
/// │    section_id: u16                                  │
/// │    offset:     u32                                  │
/// │    length:     u16                                  │
/// ├─────────────────────────────────────────────────────┤
/// │  Instruction Section  (section_id = 0x01)           │
/// │  Capability Section   (section_id = 0x02)           │
/// │  Cost Hint Section    (section_id = 0x03, optional) │
/// └─────────────────────────────────────────────────────┘
/// ```
///
/// Requirements: 12.1, 12.2, 16.1

use crate::types::BcibError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Expected magic bytes at offset 0.
pub const BCIB_MAGIC: [u8; 4] = *b"BCIB";

/// BCIB v3 version number.
pub const BCIB_VERSION_V3: u16 = 0x0003;

/// BCIB v0.2 version number (backward-compat path, Requirement 12.2).
pub const BCIB_VERSION_V02: u16 = 0x0002;

/// Size of the fixed header in bytes.
pub const HEADER_SIZE: usize = 16;

/// Size of a single section table entry in bytes.
pub const SECTION_ENTRY_SIZE: usize = 8;

// ---------------------------------------------------------------------------
// SectionId enum
// ---------------------------------------------------------------------------

/// Well-known section identifiers (design.md §BCIB Binary Format).
///
/// Requirements: 12.1 — opcode registry as single source of truth;
/// section IDs are part of the same canonical registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SectionId {
    /// Instruction bytecode section.
    Instructions = 0x01,
    /// Required capability descriptors section.
    Capabilities = 0x02,
    /// Optional cost hints section.
    CostHints = 0x03,
}

impl SectionId {
    /// Convert a raw `u16` to a `SectionId`.
    ///
    /// Returns `None` for unknown section IDs (fail-closed: callers decide
    /// whether to skip or reject unknown sections).
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x01 => Some(SectionId::Instructions),
            0x02 => Some(SectionId::Capabilities),
            0x03 => Some(SectionId::CostHints),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// BcibHeader
// ---------------------------------------------------------------------------

/// Parsed representation of the 16-byte BCIB binary header.
///
/// Layout (little-endian):
/// - bytes  0–3 : magic `b"BCIB"`
/// - bytes  4–5 : version (u16 LE)
/// - bytes  6–7 : flags   (u16 LE)
/// - bytes  8–9 : section_count (u16 LE)
/// - bytes 10–13: reserved ([u8; 4])
///
/// The two trailing bytes (14–15) complete the 16-byte block and are
/// absorbed into `reserved` as a 6-byte field in the raw layout; here we
/// store only the 4-byte reserved field and the 2-byte section_count
/// separately, matching the design spec exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcibHeader {
    /// Must equal `b"BCIB"`.
    pub magic: [u8; 4],
    /// BCIB format version (0x0003 for v3, 0x0002 for v0.2).
    pub version: u16,
    /// Format flags (reserved for future use; currently 0).
    pub flags: u16,
    /// Number of entries in the section table that follows the header.
    pub section_count: u16,
    /// Reserved bytes (must be zero; ignored on read).
    pub reserved: [u8; 4],
}

// ---------------------------------------------------------------------------
// SectionEntry
// ---------------------------------------------------------------------------

/// A single entry in the section table (8 bytes each).
///
/// Layout (little-endian):
/// - bytes 0–1 : section_id (u16 LE)
/// - bytes 2–5 : offset     (u32 LE) — byte offset from start of buffer
/// - bytes 6–7 : length     (u16 LE) — byte length of section data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionEntry {
    /// Raw section identifier; use `SectionId::from_u16()` to interpret.
    pub section_id: u16,
    /// Byte offset from the start of the BCIB buffer to this section's data.
    pub offset: u32,
    /// Byte length of this section's data.
    pub length: u16,
}

// ---------------------------------------------------------------------------
// parse_header
// ---------------------------------------------------------------------------

/// Parse and validate the 16-byte BCIB header from `data`.
///
/// Validation rules (Requirement 16.1):
/// - `data` must be at least `HEADER_SIZE` (16) bytes.
/// - `magic` must equal `b"BCIB"` → `BCIB_ERR_INVALID_GRAPH` on mismatch.
/// - `version` must be `0x0003` (v3) or `0x0002` (v0.2) →
///   `BCIB_ERR_UNSUPPORTED_VERSION` for any other value (Requirement 12.2).
///
/// Returns the parsed `BcibHeader` on success.
pub fn parse_header(data: &[u8]) -> Result<BcibHeader, BcibError> {
    if data.len() < HEADER_SIZE {
        return Err(BcibError::InvalidGraph(
            "buffer too short to contain BCIB header (need 16 bytes)",
        ));
    }

    // Safety: we verified data.len() >= 16 above; all slice accesses are
    // within bounds. No panics, no unsafe code.
    let magic: [u8; 4] = [data[0], data[1], data[2], data[3]];
    if magic != BCIB_MAGIC {
        return Err(BcibError::InvalidGraph("invalid BCIB magic bytes"));
    }

    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != BCIB_VERSION_V3 && version != BCIB_VERSION_V02 {
        return Err(BcibError::UnsupportedVersion(
            "BCIB version not supported; expected 0x0003 (v3) or 0x0002 (v0.2)",
        ));
    }

    let flags = u16::from_le_bytes([data[6], data[7]]);
    let section_count = u16::from_le_bytes([data[8], data[9]]);
    let reserved: [u8; 4] = [data[10], data[11], data[12], data[13]];
    // bytes 14–15 are the tail of the 16-byte header block; they are not
    // assigned a semantic field in the spec and are silently ignored.

    Ok(BcibHeader {
        magic,
        version,
        flags,
        section_count,
        reserved,
    })
}

// ---------------------------------------------------------------------------
// parse_section_table
// ---------------------------------------------------------------------------

/// Parse the section table that immediately follows the 16-byte header.
///
/// `header.section_count` entries are read, each 8 bytes wide.
/// The function validates that the buffer is large enough to hold the
/// complete section table (Requirement 16.1 — section layout integrity).
///
/// Returns a `Vec<SectionEntry>` on success.
pub fn parse_section_table(
    data: &[u8],
    header: &BcibHeader,
) -> Result<Vec<SectionEntry>, BcibError> {
    let count = header.section_count as usize;
    let table_size = count
        .checked_mul(SECTION_ENTRY_SIZE)
        .ok_or(BcibError::InvalidGraph("section table size overflow"))?;

    let table_end = HEADER_SIZE
        .checked_add(table_size)
        .ok_or(BcibError::InvalidGraph("section table end offset overflow"))?;

    if data.len() < table_end {
        return Err(BcibError::InvalidGraph(
            "buffer too short to contain declared section table",
        ));
    }

    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let base = HEADER_SIZE + i * SECTION_ENTRY_SIZE;
        // Each entry: section_id(2) + offset(4) + length(2) = 8 bytes
        let section_id = u16::from_le_bytes([data[base], data[base + 1]]);
        let offset = u32::from_le_bytes([
            data[base + 2],
            data[base + 3],
            data[base + 4],
            data[base + 5],
        ]);
        let length = u16::from_le_bytes([data[base + 6], data[base + 7]]);

        // Validate that the section data region is within the buffer.
        // Fail-fast: return immediately on first out-of-bounds section.
        let section_end = (offset as usize)
            .checked_add(length as usize)
            .ok_or(BcibError::InvalidGraph("section data range overflow"))?;

        if section_end > data.len() {
            return Err(BcibError::InvalidGraph(
                "section data extends beyond end of buffer",
            ));
        }

        entries.push(SectionEntry {
            section_id,
            offset,
            length,
        });
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// parse_header_and_sections  (single-pass combined decode)
// ---------------------------------------------------------------------------

/// Parse the BCIB header **and** section table in a single pass over `data`.
///
/// This is the preferred entry point for the verification pipeline
/// (Requirement 19.2 — minimize decode overhead).  It avoids the two-call
/// overhead of `parse_header()` followed by `parse_section_table()` by
/// performing both operations in one sequential read of the buffer.
///
/// Fail-fast semantics: the first validation failure returns immediately;
/// no further work is performed after an error is detected
/// (Requirement 19.1 — `BCIB_ERR_INVALID_GRAPH` early exit).
///
/// Returns `(BcibHeader, Vec<SectionEntry>)` on success.
#[inline]
pub fn parse_header_and_sections(data: &[u8]) -> Result<(BcibHeader, Vec<SectionEntry>), BcibError> {
    // --- Pass 1: header (bytes 0..16) ---
    // Fail-fast: any header error returns before touching the section table.
    let header = parse_header(data)?;

    // --- Pass 2: section table (bytes 16..16 + section_count*8) ---
    // Continues from where the header left off; no re-reading of header bytes.
    let sections = parse_section_table(data, &header)?;

    Ok((header, sections))
}

// ---------------------------------------------------------------------------
// Tests (Task 3.1)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helper: build a minimal valid v3 BCIB buffer
    // -----------------------------------------------------------------------

    /// Builds a minimal valid v3 BCIB buffer with one section entry.
    ///
    /// Layout:
    ///   [0..16]  header
    ///   [16..24] section table (1 entry × 8 bytes)
    ///   [24..26] section data  (2 bytes of dummy payload)
    fn minimal_v3_buffer() -> Vec<u8> {
        let mut buf = Vec::with_capacity(26);

        // Header (16 bytes)
        buf.extend_from_slice(b"BCIB");                    // magic
        buf.extend_from_slice(&BCIB_VERSION_V3.to_le_bytes()); // version
        buf.extend_from_slice(&0u16.to_le_bytes());        // flags
        buf.extend_from_slice(&1u16.to_le_bytes());        // section_count = 1
        buf.extend_from_slice(&[0u8; 4]);                  // reserved
        buf.extend_from_slice(&[0u8; 2]);                  // header tail (bytes 14-15)

        // Section table entry (8 bytes): Instructions section at offset 24, length 2
        buf.extend_from_slice(&(SectionId::Instructions as u16).to_le_bytes()); // section_id
        buf.extend_from_slice(&24u32.to_le_bytes());       // offset
        buf.extend_from_slice(&2u16.to_le_bytes());        // length

        // Section data (2 bytes dummy payload)
        buf.extend_from_slice(&[0x00, 0x01]);

        buf
    }

    // -----------------------------------------------------------------------
    // parse_header tests (Requirement 16.1, 12.2)
    // -----------------------------------------------------------------------

    /// Valid v3 header parses successfully.
    #[test]
    fn parse_header_valid_v3() {
        let buf = minimal_v3_buffer();
        let header = parse_header(&buf).expect("valid v3 header should parse");
        assert_eq!(header.magic, *b"BCIB");
        assert_eq!(header.version, BCIB_VERSION_V3);
        assert_eq!(header.flags, 0);
        assert_eq!(header.section_count, 1);
    }

    /// Valid v0.2 header parses successfully (backward-compat, Requirement 12.2).
    #[test]
    fn parse_header_valid_v02() {
        let mut buf = minimal_v3_buffer();
        // Overwrite version bytes with v0.2
        let v02_bytes = BCIB_VERSION_V02.to_le_bytes();
        buf[4] = v02_bytes[0];
        buf[5] = v02_bytes[1];

        let header = parse_header(&buf).expect("valid v0.2 header should parse");
        assert_eq!(header.version, BCIB_VERSION_V02);
    }

    /// Buffer shorter than 16 bytes → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn parse_header_too_short() {
        let result = parse_header(&[0u8; 8]);
        assert!(matches!(result, Err(BcibError::InvalidGraph(_))));
    }

    /// Wrong magic bytes → `BCIB_ERR_INVALID_GRAPH` (Requirement 16.1).
    #[test]
    fn parse_header_bad_magic() {
        let mut buf = minimal_v3_buffer();
        buf[0] = b'X'; // corrupt magic
        let result = parse_header(&buf);
        assert!(
            matches!(result, Err(BcibError::InvalidGraph(_))),
            "bad magic must produce BCIB_ERR_INVALID_GRAPH"
        );
    }

    /// Unsupported version (e.g. 0x0001) → `BCIB_ERR_UNSUPPORTED_VERSION`
    /// (Requirement 12.2).
    #[test]
    fn parse_header_unsupported_version() {
        let mut buf = minimal_v3_buffer();
        let bad_version: u16 = 0x0001;
        let bytes = bad_version.to_le_bytes();
        buf[4] = bytes[0];
        buf[5] = bytes[1];

        let result = parse_header(&buf);
        assert!(
            matches!(result, Err(BcibError::UnsupportedVersion(_))),
            "unsupported version must produce BCIB_ERR_UNSUPPORTED_VERSION"
        );
    }

    /// Version 0x0004 (future, unknown) → `BCIB_ERR_UNSUPPORTED_VERSION`.
    #[test]
    fn parse_header_future_version_rejected() {
        let mut buf = minimal_v3_buffer();
        let future: u16 = 0x0004;
        let bytes = future.to_le_bytes();
        buf[4] = bytes[0];
        buf[5] = bytes[1];

        assert!(matches!(
            parse_header(&buf),
            Err(BcibError::UnsupportedVersion(_))
        ));
    }

    // -----------------------------------------------------------------------
    // parse_section_table tests (Requirement 16.1)
    // -----------------------------------------------------------------------

    /// Valid section table with one Instructions entry parses correctly.
    #[test]
    fn parse_section_table_valid_one_entry() {
        let buf = minimal_v3_buffer();
        let header = parse_header(&buf).unwrap();
        let entries = parse_section_table(&buf, &header).expect("valid section table");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].section_id, SectionId::Instructions as u16);
        assert_eq!(entries[0].offset, 24);
        assert_eq!(entries[0].length, 2);
    }

    /// Zero sections → empty section table (valid).
    #[test]
    fn parse_section_table_zero_sections() {
        let mut buf = minimal_v3_buffer();
        // Set section_count to 0
        buf[8] = 0;
        buf[9] = 0;
        let header = parse_header(&buf).unwrap();
        let entries = parse_section_table(&buf, &header).expect("zero sections is valid");
        assert!(entries.is_empty());
    }

    /// Buffer too short for declared section table → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn parse_section_table_truncated_buffer() {
        let buf = minimal_v3_buffer();
        let header = parse_header(&buf).unwrap();
        // Truncate buffer so section table doesn't fit
        let truncated = &buf[..HEADER_SIZE + 4]; // only 4 bytes of the 8-byte entry
        let result = parse_section_table(truncated, &header);
        assert!(matches!(result, Err(BcibError::InvalidGraph(_))));
    }

    /// Section data extends beyond buffer end → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn parse_section_table_section_data_out_of_bounds() {
        let mut buf = minimal_v3_buffer();
        // Set section length to something larger than the buffer
        let big_length: u16 = 0xFFFF;
        let bytes = big_length.to_le_bytes();
        buf[22] = bytes[0]; // length field of first section entry
        buf[23] = bytes[1];

        let header = parse_header(&buf).unwrap();
        let result = parse_section_table(&buf, &header);
        assert!(matches!(result, Err(BcibError::InvalidGraph(_))));
    }

    // -----------------------------------------------------------------------
    // SectionId tests
    // -----------------------------------------------------------------------

    #[test]
    fn section_id_round_trip() {
        assert_eq!(SectionId::from_u16(0x01), Some(SectionId::Instructions));
        assert_eq!(SectionId::from_u16(0x02), Some(SectionId::Capabilities));
        assert_eq!(SectionId::from_u16(0x03), Some(SectionId::CostHints));
        assert_eq!(SectionId::from_u16(0x00), None);
        assert_eq!(SectionId::from_u16(0xFF), None);
    }

    // -----------------------------------------------------------------------
    // parse_header_and_sections tests (Requirement 19.1, 19.2)
    // -----------------------------------------------------------------------

    /// Combined single-pass parse produces the same result as calling
    /// parse_header + parse_section_table separately.
    #[test]
    fn parse_header_and_sections_matches_separate_calls() {
        let buf = minimal_v3_buffer();
        let (combined_header, combined_sections) =
            parse_header_and_sections(&buf).expect("combined parse should succeed");

        let sep_header = parse_header(&buf).unwrap();
        let sep_sections = parse_section_table(&buf, &sep_header).unwrap();

        assert_eq!(combined_header, sep_header);
        assert_eq!(combined_sections, sep_sections);
    }

    /// Fail-fast: bad magic causes immediate return from combined parse —
    /// no section table work is attempted (Requirement 19.1).
    #[test]
    fn parse_header_and_sections_fail_fast_bad_magic() {
        let mut buf = minimal_v3_buffer();
        buf[0] = b'X'; // corrupt magic
        let result = parse_header_and_sections(&buf);
        assert!(
            matches!(result, Err(BcibError::InvalidGraph(_))),
            "bad magic must produce BCIB_ERR_INVALID_GRAPH immediately"
        );
    }

    /// Fail-fast: unsupported version causes immediate return (Requirement 19.1).
    #[test]
    fn parse_header_and_sections_fail_fast_bad_version() {
        let mut buf = minimal_v3_buffer();
        let bad: u16 = 0x0001;
        let bytes = bad.to_le_bytes();
        buf[4] = bytes[0];
        buf[5] = bytes[1];
        let result = parse_header_and_sections(&buf);
        assert!(matches!(result, Err(BcibError::UnsupportedVersion(_))));
    }

    /// Fail-fast: truncated section table causes early return (Requirement 19.1).
    #[test]
    fn parse_header_and_sections_fail_fast_truncated_section_table() {
        let buf = minimal_v3_buffer();
        // Truncate so section table doesn't fit
        let truncated = &buf[..HEADER_SIZE + 4];
        let result = parse_header_and_sections(truncated);
        assert!(matches!(result, Err(BcibError::InvalidGraph(_))));
    }
}
