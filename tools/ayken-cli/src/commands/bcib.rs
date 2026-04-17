use crate::cli::{BcibArgs, BcibPathArgs, BcibTarget, BcibVerifyArgs};
use crate::commands::{
    risk,
    status::{self, LineageConfidence},
};
use crate::core::{error::AykenError, output, process};
use bcib::{BcibBuffer, BCIB_MAGIC, BCIB_VERSION};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct BcibHashStatus {
    path: String,
    sha256: String,
    note: &'static str,
}

#[derive(Serialize)]
struct BcibInspectStatus {
    path: String,
    bytes: u64,
    sha256: String,
    header_valid: bool,
    valid_structure: bool,
    valid_bcib: bool,
    declared_instruction_count: Option<u16>,
    decoded_instruction_count: Option<usize>,
    trailing_bytes: Option<u64>,
    decode_error: Option<String>,
    authority: &'static str,
    lineage: BcibInspectLineageStatus,
    risk: BcibInspectRiskStatus,
    note: &'static str,
}

#[derive(Serialize)]
struct BcibInspectLineageStatus {
    resolved: bool,
    tainted: bool,
    confidence: Option<LineageConfidence>,
    ancestor_distance: Option<usize>,
    nearest_verified_ancestor: Option<String>,
}

#[derive(Serialize)]
struct BcibInspectRiskStatus {
    level: &'static str,
    note: &'static str,
}

struct BcibStructureStatus {
    header_valid: bool,
    valid_structure: bool,
    declared_instruction_count: Option<u16>,
    trailing_bytes: Option<u64>,
}

pub fn run(args: BcibArgs, json: bool) -> Result<(), AykenError> {
    match args.target {
        BcibTarget::Verify(args) => run_verify(args, json),
        BcibTarget::Hash(args) => run_hash(args, json),
        BcibTarget::Inspect(args) => run_inspect(args, json),
    }
}

fn run_verify(args: BcibVerifyArgs, json: bool) -> Result<(), AykenError> {
    let mut command = vec![
        "run".to_string(),
        "--manifest-path".to_string(),
        "ayken-core/Cargo.toml".to_string(),
        "-p".to_string(),
        "proof-verifier".to_string(),
        "--bin".to_string(),
        "proof-verifier".to_string(),
        "--".to_string(),
        "verify".to_string(),
        "bundle".to_string(),
        args.bundle_path,
        "--policy".to_string(),
        args.policy,
        "--registry".to_string(),
        args.registry,
    ];

    if json {
        command.push("--json".to_string());
    }

    process::run_command_owned("cargo", &command, None, &[])
}

fn run_hash(args: BcibPathArgs, json: bool) -> Result<(), AykenError> {
    let path = PathBuf::from(args.path);
    ensure_regular_file(&path)?;
    let bytes = fs::read(&path)?;

    let status = BcibHashStatus {
        path: path.display().to_string(),
        sha256: sha256_hex(&bytes),
        note: "Local digest surface only; does not assert proof or closure authority.",
    };

    if json {
        output::print_json(&status)
    } else {
        println!("ayken bcib hash");
        println!("  path   : {}", status.path);
        println!("  sha256 : {}", status.sha256);
        println!("  note   : {}", status.note);
        Ok(())
    }
}

fn run_inspect(args: BcibPathArgs, json: bool) -> Result<(), AykenError> {
    let path = PathBuf::from(args.path);
    let metadata = ensure_regular_file(&path)?;
    let bytes = fs::read(&path)?;
    let structure = inspect_structure(&bytes);
    let sha256 = sha256_hex(&bytes);
    let authority = status::gather_authority_status();
    let advisory_risk = risk::compute_risk(&authority);
    let (valid_bcib, decoded_instruction_count, decode_error) = match BcibBuffer::decode(&bytes) {
        Ok(buffer) => (true, Some(buffer.len()), None),
        Err(err) => (false, None, Some(err.to_string())),
    };

    let status = BcibInspectStatus {
        path: path.display().to_string(),
        bytes: metadata.len(),
        sha256,
        header_valid: structure.header_valid,
        valid_structure: structure.valid_structure,
        valid_bcib,
        declared_instruction_count: structure.declared_instruction_count,
        decoded_instruction_count,
        trailing_bytes: structure.trailing_bytes,
        decode_error,
        authority: authority.effective_authority,
        lineage: BcibInspectLineageStatus {
            resolved: authority.lineage_resolved,
            tainted: authority.lineage_tainted,
            confidence: authority.lineage_confidence,
            ancestor_distance: authority.ancestor_distance,
            nearest_verified_ancestor: authority.nearest_verified_ancestor.clone(),
        },
        risk: BcibInspectRiskStatus {
            level: advisory_risk.risk_level,
            note: advisory_risk.note,
        },
        note: "Inspection is advisory; BCIB structure/decode signals are exposed alongside authority, lineage, and risk context without claiming execution safety or verification authority.",
    };

    if json {
        output::print_json(&status)
    } else {
        println!("ayken bcib inspect");
        println!("  path              : {}", status.path);
        println!("  bytes             : {}", status.bytes);
        println!("  sha256            : {}", status.sha256);
        println!("  header_valid      : {}", status.header_valid);
        println!("  valid_structure   : {}", status.valid_structure);
        println!("  valid_bcib        : {}", status.valid_bcib);
        println!(
            "  declared_instr    : {}",
            status
                .declared_instruction_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  decoded_instr     : {}",
            status
                .decoded_instruction_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  trailing_bytes    : {}",
            status
                .trailing_bytes
                .map(|count| count.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        if let Some(error) = &status.decode_error {
            println!("  decode_error      : {error}");
        }
        println!("  authority         : {}", status.authority);
        println!("  lineage.resolved  : {}", status.lineage.resolved);
        println!("  lineage.tainted   : {}", status.lineage.tainted);
        println!(
            "  lineage.confidence: {}",
            status
                .lineage
                .confidence
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  lineage.distance  : {}",
            status
                .lineage
                .ancestor_distance
                .map(|count| count.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  lineage.ancestor  : {}",
            status
                .lineage
                .nearest_verified_ancestor
                .as_deref()
                .unwrap_or("n/a")
        );
        println!("  risk.level        : {}", status.risk.level);
        println!("  risk.note         : {}", status.risk.note);
        println!("  note: {}", status.note);
        Ok(())
    }
}

fn ensure_regular_file(path: &Path) -> Result<fs::Metadata, AykenError> {
    let metadata = fs::metadata(path).map_err(|err| {
        AykenError::Io(format!("failed to read {} metadata: {err}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(AykenError::Policy(format!(
            "{} is not a regular file",
            path.display()
        )));
    }

    Ok(metadata)
}

fn inspect_structure(bytes: &[u8]) -> BcibStructureStatus {
    let header_size = mem::size_of::<bcib::BcibHeader>();
    if bytes.len() < header_size {
        return BcibStructureStatus {
            header_valid: false,
            valid_structure: false,
            declared_instruction_count: None,
            trailing_bytes: None,
        };
    }

    let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let declared_instruction_count = u16::from_le_bytes([bytes[6], bytes[7]]);
    let header_valid = magic == BCIB_MAGIC && version == BCIB_VERSION;
    let expected_size =
        header_size + mem::size_of::<bcib::BcibInstruction>() * declared_instruction_count as usize;
    let trailing_bytes = bytes.len().saturating_sub(expected_size) as u64;

    BcibStructureStatus {
        header_valid,
        valid_structure: header_valid && bytes.len() == expected_size,
        declared_instruction_count: Some(declared_instruction_count),
        trailing_bytes: Some(trailing_bytes),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}
