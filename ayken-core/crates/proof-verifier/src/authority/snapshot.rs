use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::types::{VerificationFinding, VerifierTrustRegistrySnapshot};
use serde_json::Value;

pub struct VerifierTrustRegistryValidation {
    pub findings: Vec<VerificationFinding>,
    pub recomputed_hash: String,
}

pub fn validate_verifier_trust_registry_snapshot(
    snapshot: &VerifierTrustRegistrySnapshot,
) -> Result<VerifierTrustRegistryValidation, VerifierRuntimeError> {
    let mut findings = Vec::new();
    if snapshot.registry_format_version == 0 {
        findings.push(VerificationFinding::error(
            "PV0900",
            "verifier trust registry registry_format_version must be non-zero",
        ));
    }
    if snapshot.registry_scope.trim().is_empty() {
        findings.push(VerificationFinding::error(
            "PV0900",
            "verifier trust registry registry_scope must not be empty",
        ));
    }
    if snapshot.root_verifier_ids.is_empty() {
        findings.push(VerificationFinding::error(
            "PV0903",
            "verifier trust registry must declare at least one explicit root verifier",
        ));
    }
    if snapshot.verifier_registry_snapshot_hash.is_empty() {
        findings.push(VerificationFinding::error(
            "PV0901",
            "verifier_registry_snapshot_hash must not be empty",
        ));
    } else if !is_prefixed_sha256(&snapshot.verifier_registry_snapshot_hash) {
        findings.push(VerificationFinding::error(
            "PV0901",
            "verifier_registry_snapshot_hash must use sha256:<64-hex> format",
        ));
    }

    let recomputed_hash = compute_verifier_trust_registry_snapshot_hash(snapshot)?;
    if !snapshot.verifier_registry_snapshot_hash.is_empty()
        && snapshot.verifier_registry_snapshot_hash != recomputed_hash
    {
        findings.push(VerificationFinding::error(
            "PV0902",
            "verifier_registry_snapshot_hash does not match canonical recomputed verifier trust registry snapshot hash",
        ));
    }

    for root_verifier_id in &snapshot.root_verifier_ids {
        if !snapshot.verifiers.contains_key(root_verifier_id) {
            findings.push(VerificationFinding::error(
                "PV0903",
                format!(
                    "explicit root verifier {root_verifier_id} is missing from verifier trust registry nodes"
                ),
            ));
        }
    }

    for (verifier_id, node) in &snapshot.verifiers {
        if node.verifier_id != *verifier_id {
            findings.push(VerificationFinding::error(
                "PV0904",
                format!(
                    "verifier trust node key {verifier_id} does not match node.verifier_id {}",
                    node.verifier_id
                ),
            ));
        }
        if !snapshot.public_keys.contains_key(&node.verifier_pubkey_id) {
            findings.push(VerificationFinding::error(
                "PV0904",
                format!(
                    "verifier trust node {verifier_id} references missing verifier public key {}",
                    node.verifier_pubkey_id
                ),
            ));
        }
    }

    for edge in &snapshot.delegation_edges {
        if !snapshot.verifiers.contains_key(&edge.parent_verifier_id) {
            findings.push(VerificationFinding::error(
                "PV0905",
                format!(
                    "delegation edge references missing parent verifier {}",
                    edge.parent_verifier_id
                ),
            ));
        }
        if !snapshot.verifiers.contains_key(&edge.delegate_verifier_id) {
            findings.push(VerificationFinding::error(
                "PV0905",
                format!(
                    "delegation edge references missing delegate verifier {}",
                    edge.delegate_verifier_id
                ),
            ));
        }
    }

    Ok(VerifierTrustRegistryValidation {
        findings,
        recomputed_hash,
    })
}

pub fn compute_verifier_trust_registry_snapshot_hash(
    snapshot: &VerifierTrustRegistrySnapshot,
) -> Result<String, VerifierRuntimeError> {
    let mut snapshot_value = serde_json::to_value(snapshot).map_err(|error| {
        VerifierRuntimeError::json("serialize verifier trust registry snapshot", error)
    })?;
    if let Value::Object(map) = &mut snapshot_value {
        map.remove("verifier_registry_snapshot_hash");
    }
    let bytes = canonicalize_json_value(&snapshot_value)?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

fn is_prefixed_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}
