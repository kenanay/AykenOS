use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::types::{RegistrySnapshot, VerificationFinding};
use serde_json::Value;

pub struct RegistrySnapshotValidation {
    pub findings: Vec<VerificationFinding>,
    pub recomputed_hash: String,
}

pub fn validate_registry_snapshot(
    snapshot: &RegistrySnapshot,
) -> Result<RegistrySnapshotValidation, VerifierRuntimeError> {
    let mut findings = Vec::new();
    if snapshot.registry_format_version == 0 {
        findings.push(VerificationFinding::error(
            "PV0400",
            "registry_format_version must be non-zero",
        ));
    }
    if snapshot.registry_snapshot_hash.is_empty() {
        findings.push(VerificationFinding::error(
            "PV0401",
            "registry_snapshot_hash must not be empty",
        ));
    } else if !is_sha256_hex(&snapshot.registry_snapshot_hash) {
        findings.push(VerificationFinding::error(
            "PV0409",
            "registry_snapshot_hash must be a 64-character lowercase SHA-256 hex digest",
        ));
    }

    let recomputed_hash = compute_registry_snapshot_hash(snapshot)?;
    if !snapshot.registry_snapshot_hash.is_empty()
        && snapshot.registry_snapshot_hash != recomputed_hash
    {
        findings.push(VerificationFinding::error(
            "PV0410",
            "registry_snapshot_hash does not match canonical recomputed registry snapshot hash",
        ));
    }

    for (producer_id, entry) in &snapshot.producers {
        for key_id in entry
            .active_pubkey_ids
            .iter()
            .chain(entry.revoked_pubkey_ids.iter())
            .chain(entry.superseded_pubkey_ids.iter())
        {
            if !entry.public_keys.contains_key(key_id) {
                findings.push(VerificationFinding::error(
                    "PV0408",
                    format!(
                        "registry producer {producer_id} references key {key_id} without concrete public key material"
                    ),
                ));
            }
        }
    }

    Ok(RegistrySnapshotValidation {
        findings,
        recomputed_hash,
    })
}

pub fn compute_registry_snapshot_hash(
    snapshot: &RegistrySnapshot,
) -> Result<String, VerifierRuntimeError> {
    let mut snapshot_value = serde_json::to_value(snapshot)
        .map_err(|error| VerifierRuntimeError::json("serialize registry snapshot", error))?;
    if let Value::Object(map) = &mut snapshot_value {
        map.remove("registry_snapshot_hash");
    }
    let bytes = canonicalize_json_value(&snapshot_value)?;
    Ok(sha256_hex(&bytes))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}
