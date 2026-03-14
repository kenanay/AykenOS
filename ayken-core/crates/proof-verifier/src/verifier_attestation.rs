use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use crate::crypto::ed25519::{is_allowed_signature_algorithm, verify_ed25519_bytes};
use crate::types::{VerificationFinding, VerifierTrustRegistrySnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierAttestation {
    pub attestation_version: u32,
    pub verifier_id: String,
    pub verifier_pubkey_id: String,
    pub verifier_registry_ref: String,
    #[serde(deserialize_with = "deserialize_u64_like")]
    pub verifier_key_epoch: u64,
    pub verifier_contract_version: String,
    pub attestation_signature_algorithm: String,
    pub attestation_signature: String,
    #[serde(default)]
    pub attested_at_utc: Option<String>,
}

pub fn load_verifier_attestation(path: &Path) -> Result<VerifierAttestation, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read verifier attestation at {}: {error}",
            path.display()
        )
    })?;
    let attestation: VerifierAttestation = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse verifier attestation at {}: {error}",
            path.display()
        )
    })?;
    validate_verifier_attestation_shape(&attestation)?;
    Ok(attestation)
}

pub fn write_verifier_attestation(
    path: &Path,
    attestation: &VerifierAttestation,
) -> Result<(), String> {
    validate_verifier_attestation_shape(attestation)?;
    let bytes = canonicalize_json(attestation).map_err(|error| {
        format!(
            "failed to canonicalize verifier attestation for {}: {error}",
            path.display()
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create verifier attestation parent {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to write verifier attestation {}: {error}",
            path.display()
        )
    })
}

pub fn validate_verifier_attestation_shape(
    attestation: &VerifierAttestation,
) -> Result<(), String> {
    if attestation.attestation_version != 1 {
        return Err(format!(
            "unsupported attestation_version {} for verifier attestation",
            attestation.attestation_version
        ));
    }
    for (label, value) in [
        ("verifier_id", attestation.verifier_id.as_str()),
        (
            "verifier_pubkey_id",
            attestation.verifier_pubkey_id.as_str(),
        ),
        (
            "verifier_registry_ref",
            attestation.verifier_registry_ref.as_str(),
        ),
        (
            "verifier_contract_version",
            attestation.verifier_contract_version.as_str(),
        ),
        (
            "attestation_signature_algorithm",
            attestation.attestation_signature_algorithm.as_str(),
        ),
        (
            "attestation_signature",
            attestation.attestation_signature.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "{label} must not be empty for verifier attestation"
            ));
        }
    }
    if attestation.verifier_key_epoch == 0 {
        return Err("verifier_key_epoch must be non-zero for verifier attestation".to_string());
    }
    if !is_allowed_signature_algorithm(&attestation.attestation_signature_algorithm) {
        return Err(format!(
            "unsupported attestation_signature_algorithm {} for verifier attestation",
            attestation.attestation_signature_algorithm
        ));
    }
    Ok(())
}

pub fn compute_verifier_attestation_ref(
    attestation: &VerifierAttestation,
) -> Result<String, String> {
    let bytes = canonicalize_json(attestation)
        .map_err(|error| format!("failed to canonicalize verifier attestation ref: {error}"))?;
    Ok(format!("cas:sha256:{}", sha256_hex(&bytes)))
}

pub fn canonicalize_verifier_attestation_payload(
    attestation: &VerifierAttestation,
) -> Result<Vec<u8>, String> {
    let mut value = serde_json::to_value(attestation).map_err(|error| {
        format!("failed to serialize verifier attestation payload for hashing: {error}")
    })?;
    if let Value::Object(map) = &mut value {
        map.remove("attestation_signature");
    }
    canonicalize_json_value(&value)
        .map_err(|error| format!("failed to canonicalize verifier attestation payload: {error}"))
}

pub fn verify_verifier_attestation(
    attestation: &VerifierAttestation,
    verifier_registry: &VerifierTrustRegistrySnapshot,
) -> Result<Vec<VerificationFinding>, String> {
    validate_verifier_attestation_shape(attestation)?;
    let mut findings = Vec::new();

    if attestation.verifier_registry_ref != verifier_registry.registry_scope {
        findings.push(VerificationFinding::error(
            "PV1101",
            "verifier attestation registry_ref does not match verifier trust registry scope",
        ));
    }

    let Some(node) = verifier_registry.verifiers.get(&attestation.verifier_id) else {
        findings.push(VerificationFinding::error(
            "PV1102",
            "verifier attestation verifier_id is missing from verifier trust registry",
        ));
        return Ok(findings);
    };

    if node.verifier_pubkey_id != attestation.verifier_pubkey_id {
        findings.push(VerificationFinding::error(
            "PV1103",
            "verifier attestation verifier_pubkey_id does not match verifier trust registry node",
        ));
    }

    let Some(public_key) = verifier_registry
        .public_keys
        .get(&attestation.verifier_pubkey_id)
    else {
        findings.push(VerificationFinding::error(
            "PV1104",
            "verifier attestation verifier_pubkey_id is missing from verifier trust registry public_keys",
        ));
        return Ok(findings);
    };

    if !public_key
        .algorithm
        .eq_ignore_ascii_case(&attestation.attestation_signature_algorithm)
    {
        findings.push(VerificationFinding::error(
            "PV1105",
            "verifier attestation signature algorithm does not match verifier trust registry public key algorithm",
        ));
    }

    let payload_bytes = canonicalize_verifier_attestation_payload(attestation)?;
    if let Err(finding) = verify_ed25519_bytes(
        &public_key.public_key,
        &attestation.attestation_signature,
        &payload_bytes,
        "PV1106",
        "verifier attestation detached signature verification failed",
    ) {
        findings.push(finding);
    }

    Ok(findings)
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

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_verifier_attestation_payload, compute_verifier_attestation_ref,
        validate_verifier_attestation_shape, verify_verifier_attestation, VerifierAttestation,
    };
    use crate::crypto::ed25519::sign_ed25519_bytes;
    use crate::testing::fixtures::create_fixture_bundle;
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    fn sample_attestation() -> (VerifierAttestation, crate::VerifierTrustRegistrySnapshot) {
        let fixture = create_fixture_bundle();
        let mut attestation = VerifierAttestation {
            attestation_version: 1,
            verifier_id: fixture.receipt_signer.verifier_node_id.clone(),
            verifier_pubkey_id: fixture.receipt_signer.verifier_key_id.clone(),
            verifier_registry_ref: fixture.verifier_registry.registry_scope.clone(),
            verifier_key_epoch: u64::from(fixture.verifier_registry.verifier_registry_epoch),
            verifier_contract_version: "phase12-context-v1".to_string(),
            attestation_signature_algorithm: "ed25519".to_string(),
            attestation_signature: String::new(),
            attested_at_utc: Some("2026-03-14T00:00:00Z".to_string()),
        };
        let payload = canonicalize_verifier_attestation_payload(&attestation)
            .expect("canonicalize attestation payload");
        attestation.attestation_signature =
            sign_ed25519_bytes(&fixture.receipt_signer.private_key, &payload)
                .expect("sign attestation payload");
        (attestation, fixture.verifier_registry)
    }

    #[test]
    fn verifier_attestation_validation_accepts_signed_fixture() {
        let (attestation, registry) = sample_attestation();
        validate_verifier_attestation_shape(&attestation).expect("shape should validate");
        let findings =
            verify_verifier_attestation(&attestation, &registry).expect("verify attestation");
        assert!(findings.is_empty());
        let attestation_ref =
            compute_verifier_attestation_ref(&attestation).expect("compute attestation ref");
        assert!(attestation_ref.starts_with("cas:sha256:"));
    }

    #[test]
    fn verifier_attestation_validation_rejects_signature_drift() {
        let (mut attestation, registry) = sample_attestation();
        attestation.attestation_signature = format!("base64:{}", STANDARD.encode([0u8; 64]));
        let findings =
            verify_verifier_attestation(&attestation, &registry).expect("verify attestation");
        assert!(findings.iter().any(|finding| finding.code == "PV1106"));
    }
}
