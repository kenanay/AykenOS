use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use crate::errors::VerifierRuntimeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct VerificationDiversityLedgerEntry {
    pub ledger_version: u32,
    pub entry_id: String,
    pub run_id: String,
    #[serde(deserialize_with = "deserialize_u64_like")]
    pub timestamp_unix_ns: u64,
    pub subject_bundle_id: String,
    pub verification_context_id: String,
    pub verification_node_id: String,
    pub verifier_id: String,
    pub authority_chain_id: String,
    pub lineage_id: String,
    #[serde(default)]
    pub execution_cluster_id: Option<String>,
    pub verdict: String,
    pub receipt_hash: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct VerificationDiversityLedgerDocument {
    pub entries: Vec<VerificationDiversityLedgerEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LedgerDocument {
    Entries(Vec<VerificationDiversityLedgerEntry>),
    Wrapped {
        entries: Vec<VerificationDiversityLedgerEntry>,
    },
}

pub fn load_diversity_ledger_entries(
    path: &Path,
) -> Result<Vec<VerificationDiversityLedgerEntry>, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read ledger at {}: {error}", path.display()))?;
    let document: LedgerDocument = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse ledger at {}: {error}", path.display()))?;
    let mut entries = match document {
        LedgerDocument::Entries(entries) => entries,
        LedgerDocument::Wrapped { entries } => entries,
    };
    sort_diversity_ledger_entries(&mut entries);
    Ok(entries)
}

pub fn write_diversity_ledger_entries(
    path: &Path,
    entries: &[VerificationDiversityLedgerEntry],
) -> Result<(), String> {
    let mut sorted_entries = entries.to_vec();
    sort_diversity_ledger_entries(&mut sorted_entries);
    let document = VerificationDiversityLedgerDocument {
        entries: sorted_entries,
    };
    let bytes = canonicalize_json(&document)
        .map_err(|error| format!("failed to canonicalize ledger for {}: {error}", path.display()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create ledger parent {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes)
        .map_err(|error| format!("failed to write ledger {}: {error}", path.display()))
}

pub fn sort_diversity_ledger_entries(entries: &mut [VerificationDiversityLedgerEntry]) {
    entries.sort_by(|left, right| {
        left.timestamp_unix_ns
            .cmp(&right.timestamp_unix_ns)
            .then_with(|| left.entry_id.cmp(&right.entry_id))
            .then_with(|| left.receipt_hash.cmp(&right.receipt_hash))
    });
}

pub fn compute_diversity_ledger_entry_id(
    entry: &VerificationDiversityLedgerEntry,
) -> Result<String, String> {
    let mut value = serde_json::to_value(entry)
        .map_err(|error| format!("failed to serialize VDL entry for hashing: {error}"))?;
    if let Value::Object(map) = &mut value {
        map.remove("entry_id");
    }
    let bytes = canonicalize_json_value(&value)
        .map_err(|error| format!("failed to canonicalize VDL entry for hashing: {error}"))?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

pub fn validate_diversity_ledger_entry(
    entry: &VerificationDiversityLedgerEntry,
) -> Result<(), String> {
    if entry.ledger_version != 1 {
        return Err(format!(
            "unsupported ledger_version {} for entry {}",
            entry.ledger_version, entry.entry_id
        ));
    }
    for (label, value) in [
        ("entry_id", entry.entry_id.as_str()),
        ("run_id", entry.run_id.as_str()),
        ("subject_bundle_id", entry.subject_bundle_id.as_str()),
        (
            "verification_context_id",
            entry.verification_context_id.as_str(),
        ),
        (
            "verification_node_id",
            entry.verification_node_id.as_str(),
        ),
        ("verifier_id", entry.verifier_id.as_str()),
        ("authority_chain_id", entry.authority_chain_id.as_str()),
        ("lineage_id", entry.lineage_id.as_str()),
        ("verdict", entry.verdict.as_str()),
        ("receipt_hash", entry.receipt_hash.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{label} must not be empty for entry {}", entry.entry_id));
        }
    }
    if entry.timestamp_unix_ns == 0 {
        return Err(format!(
            "timestamp_unix_ns must be non-zero for entry {}",
            entry.entry_id
        ));
    }
    if !is_lower_hex_digest(&entry.receipt_hash) {
        return Err(format!(
            "receipt_hash must be a 64-character lowercase SHA-256 hex digest for entry {}",
            entry.entry_id
        ));
    }
    let expected_entry_id = compute_diversity_ledger_entry_id(entry)?;
    if entry.entry_id != expected_entry_id {
        return Err(format!(
            "entry_id does not match canonical content-addressed identity for entry {}",
            entry.entry_id
        ));
    }
    Ok(())
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn deserialize_u64_like<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64Like {
        Int(u64),
        String(String),
    }

    match U64Like::deserialize(deserializer)? {
        U64Like::Int(value) => Ok(value),
        U64Like::String(value) => value.parse::<u64>().map_err(serde::de::Error::custom),
    }
}

#[allow(dead_code)]
fn _error_type_marker(_: VerifierRuntimeError) {}
