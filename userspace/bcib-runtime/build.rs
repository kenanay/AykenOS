/// Build script — generates BCIB v0.2 golden fixture binary files.
///
/// This script runs at compile time and writes the fixture `.bcib` files
/// into `tests/fixtures/`. The files are committed to the repository so
/// CI can verify them without running the build script again, but the
/// script ensures they are always regenerated correctly if missing.
///
/// Requirements: 12.3, 12.4 — golden fixtures must be defined and validated
/// in CI; fixture mismatch must cause CI FAIL.

use std::fs;
use std::path::Path;

fn main() {
    let fixtures_dir = Path::new("tests/fixtures");
    fs::create_dir_all(fixtures_dir).expect("failed to create tests/fixtures directory");

    // Generate each fixture
    write_fixture(fixtures_dir, "nop_end.bcib", &build_nop_end());
    write_fixture(fixtures_dir, "data_create_query.bcib", &build_data_create_query());
    write_fixture(fixtures_dir, "data_add.bcib", &build_data_add());
    write_fixture(fixtures_dir, "ui_render.bcib", &build_ui_render());
    write_fixture(fixtures_dir, "ai_ask.bcib", &build_ai_ask());
    write_fixture(fixtures_dir, "invalid_magic.bcib", &build_invalid_magic());
    write_fixture(fixtures_dir, "unsupported_version.bcib", &build_unsupported_version());

    // Tell Cargo to re-run this script only if it changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn write_fixture(dir: &Path, name: &str, data: &[u8]) {
    let path = dir.join(name);
    fs::write(&path, data)
        .unwrap_or_else(|e| panic!("failed to write fixture {}: {}", name, e));
}

// ---------------------------------------------------------------------------
// Binary format helpers
// ---------------------------------------------------------------------------

/// BCIB v0.2 version number (little-endian).
const VERSION_V02: u16 = 0x0002;
/// BCIB v3 version number (little-endian).
const VERSION_V3: u16 = 0x0003;
/// Instructions section ID.
const SECTION_INSTRUCTIONS: u16 = 0x0001;
/// Header size in bytes.
const HEADER_SIZE: usize = 16;
/// Section entry size in bytes.
const SECTION_ENTRY_SIZE: usize = 8;

/// Build a complete BCIB binary buffer.
///
/// Layout:
///   [0..16]                header
///   [16..16+8*n]           section table (n entries)
///   [16+8*n..]             section data
///
/// `version`: BCIB version field (0x0002 or 0x0003)
/// `instr_bytes`: raw instruction section payload
fn build_bcib(version: u16, instr_bytes: &[u8]) -> Vec<u8> {
    let instr_len = instr_bytes.len();
    // Instruction section starts immediately after header + section table (1 entry)
    let instr_offset = (HEADER_SIZE + SECTION_ENTRY_SIZE) as u32; // 24

    let mut buf = Vec::with_capacity(HEADER_SIZE + SECTION_ENTRY_SIZE + instr_len);

    // --- Header (16 bytes) ---
    buf.extend_from_slice(b"BCIB");                          // magic [0..4]
    buf.extend_from_slice(&version.to_le_bytes());           // version [4..6]
    buf.extend_from_slice(&0u16.to_le_bytes());              // flags [6..8]
    buf.extend_from_slice(&1u16.to_le_bytes());              // section_count=1 [8..10]
    buf.extend_from_slice(&[0u8; 4]);                        // reserved [10..14]
    buf.extend_from_slice(&[0u8; 2]);                        // tail bytes [14..16]

    // --- Section table entry (8 bytes) ---
    buf.extend_from_slice(&SECTION_INSTRUCTIONS.to_le_bytes()); // section_id [16..18]
    buf.extend_from_slice(&instr_offset.to_le_bytes());          // offset [18..22]
    buf.extend_from_slice(&(instr_len as u16).to_le_bytes());    // length [22..24]

    // --- Instruction data ---
    buf.extend_from_slice(instr_bytes);

    buf
}

/// Encode a single instruction: opcode(1) + operand_count(1) + operands(n×4 LE).
fn encode_instr(opcode: u8, operands: &[u32]) -> Vec<u8> {
    let mut bytes = vec![opcode, operands.len() as u8];
    for &op in operands {
        bytes.extend_from_slice(&op.to_le_bytes());
    }
    bytes
}

// ---------------------------------------------------------------------------
// Fixture builders
// ---------------------------------------------------------------------------

/// Fixture 1: nop_end.bcib
/// Minimal v0.2 program: Nop (0x00) + End (0x01)
/// Both are Pure opcodes — no capability required.
/// Expected: parse OK, plan OK
fn build_nop_end() -> Vec<u8> {
    let mut instrs = Vec::new();
    instrs.extend(encode_instr(0x00 /* Nop */, &[]));
    instrs.extend(encode_instr(0x01 /* End */, &[]));
    build_bcib(VERSION_V02, &instrs)
}

/// Fixture 2: data_create_query.bcib
/// v0.2 data operations: DataCreate (0x10) + DataQuery (0x12) + End (0x01)
/// DataCreate and DataQuery are DataMutating — capability required at plan time.
/// Expected: parse OK, plan OK (with capability token)
fn build_data_create_query() -> Vec<u8> {
    let mut instrs = Vec::new();
    instrs.extend(encode_instr(0x10 /* DataCreate */, &[]));
    instrs.extend(encode_instr(0x12 /* DataQuery */, &[]));
    instrs.extend(encode_instr(0x01 /* End */, &[]));
    build_bcib(VERSION_V02, &instrs)
}

/// Fixture 3: data_add.bcib
/// v0.2 data mutation: DataCreate (0x10) + DataAdd (0x11) + End (0x01)
/// Both DataCreate and DataAdd are DataMutating — capability required.
/// Expected: parse OK, plan OK (with capability token)
fn build_data_add() -> Vec<u8> {
    let mut instrs = Vec::new();
    instrs.extend(encode_instr(0x10 /* DataCreate */, &[]));
    instrs.extend(encode_instr(0x11 /* DataAdd */, &[]));
    instrs.extend(encode_instr(0x01 /* End */, &[]));
    build_bcib(VERSION_V02, &instrs)
}

/// Fixture 4: ui_render.bcib
/// v0.2 UI operation: UiRender (0x20) + End (0x01)
/// UiRender is DataMutating — capability required.
/// Expected: parse OK, plan OK (with capability token)
fn build_ui_render() -> Vec<u8> {
    let mut instrs = Vec::new();
    instrs.extend(encode_instr(0x20 /* UiRender */, &[]));
    instrs.extend(encode_instr(0x01 /* End */, &[]));
    build_bcib(VERSION_V02, &instrs)
}

/// Fixture 5: ai_ask.bcib
/// v0.2 AI operation: AiAsk (0x30) + End (0x01)
/// AiAsk is External — capability required.
/// Expected: parse OK, plan OK (with capability token)
fn build_ai_ask() -> Vec<u8> {
    let mut instrs = Vec::new();
    instrs.extend(encode_instr(0x30 /* AiAsk */, &[]));
    instrs.extend(encode_instr(0x01 /* End */, &[]));
    build_bcib(VERSION_V02, &instrs)
}

/// Fixture 6: invalid_magic.bcib
/// Negative test: magic bytes are "XBIB" instead of "BCIB".
/// Expected: parse FAIL → BCIB_ERR_INVALID_GRAPH
fn build_invalid_magic() -> Vec<u8> {
    let mut buf = build_bcib(VERSION_V02, &encode_instr(0x00, &[]));
    // Corrupt the first magic byte
    buf[0] = b'X';
    buf
}

/// Fixture 7: unsupported_version.bcib
/// Negative test: version field is 0x0004 (future, unsupported).
/// Expected: parse FAIL → BCIB_ERR_UNSUPPORTED_VERSION
fn build_unsupported_version() -> Vec<u8> {
    let mut buf = build_bcib(VERSION_V3, &encode_instr(0x00, &[]));
    // Overwrite version with 0x0004
    let v: u16 = 0x0004;
    let bytes = v.to_le_bytes();
    buf[4] = bytes[0];
    buf[5] = bytes[1];
    buf
}
