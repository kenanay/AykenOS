use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json;
use crate::errors::VerifierRuntimeError;
use crate::overlay::producer::load_producer;
use crate::overlay::signature_envelope::load_signature_envelope;
use crate::types::{LoadedBundle, OverlayState, VerificationFinding};

pub fn verify_overlay(
    bundle: &LoadedBundle,
    expected_bundle_id: &str,
) -> Result<OverlayState, VerifierRuntimeError> {
    let producer = load_producer(&bundle.producer_path)?;
    let signature_envelope = load_signature_envelope(&bundle.signature_envelope_path)?;
    let mut findings = Vec::new();

    if signature_envelope.bundle_id != expected_bundle_id {
        findings.push(VerificationFinding::error(
            "PV0300",
            "signature envelope bundle_id does not match portable bundle_id",
        ));
    }

    if signature_envelope.signatures.is_empty() {
        findings.push(VerificationFinding::error(
            "PV0301",
            "signature envelope contains no signatures",
        ));
    }

    for signature in &signature_envelope.signatures {
        if signature.signer_id.is_empty() || signature.producer_pubkey_id.is_empty() {
            findings.push(VerificationFinding::error(
                "PV0302",
                "signature envelope contains incomplete signer metadata",
            ));
        }
        if signature.signature_algorithm.is_empty() || signature.signature.is_empty() {
            findings.push(VerificationFinding::error(
                "PV0303",
                "signature envelope contains empty algorithm or signature bytes",
            ));
        }
    }

    let producer_bytes = canonicalize_json(&producer)?;
    let envelope_bytes = canonicalize_json(&signature_envelope)?;
    let mut material = Vec::new();
    material.extend_from_slice(&producer_bytes);
    material.extend_from_slice(&envelope_bytes);
    let trust_overlay_hash = sha256_hex(&material);

    Ok(OverlayState {
        producer,
        signature_envelope,
        trust_overlay_hash,
        findings,
    })
}
