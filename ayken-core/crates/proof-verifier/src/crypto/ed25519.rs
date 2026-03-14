use crate::errors::VerifierRuntimeError;
use crate::types::{KeyStatus, ResolvedSigner, SignatureEnvelope, VerificationFinding};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::collections::BTreeMap;

pub const ALLOWED_SIGNATURE_ALGORITHM: &str = "ed25519";
const ALLOWED_BUNDLE_ID_ALGORITHM: &str = "sha256";

pub fn verify_detached_signatures(
    bundle_id: &str,
    signature_envelope: &SignatureEnvelope,
    resolved_signers: &[ResolvedSigner],
) -> Vec<VerificationFinding> {
    let mut findings = Vec::new();

    if !signature_envelope
        .bundle_id_algorithm
        .eq_ignore_ascii_case(ALLOWED_BUNDLE_ID_ALGORITHM)
    {
        findings.push(VerificationFinding::error(
            "PV0600",
            "signature envelope bundle_id_algorithm is not allowlisted",
        ));
    }

    let signer_index = resolved_signer_index(resolved_signers);
    for signature in &signature_envelope.signatures {
        if !signature
            .signature_algorithm
            .eq_ignore_ascii_case(ALLOWED_SIGNATURE_ALGORITHM)
        {
            findings.push(VerificationFinding::error(
                "PV0601",
                "signature entry uses a non-allowlisted detached signature algorithm",
            ));
            continue;
        }

        let key = (
            signature.signer_id.as_str(),
            signature.producer_pubkey_id.as_str(),
        );
        let Some(resolved_signer) = signer_index.get(&key) else {
            findings.push(VerificationFinding::error(
                "PV0602",
                "signature entry could not be matched to a resolved signer",
            ));
            continue;
        };

        let Some(public_key) = &resolved_signer.public_key else {
            findings.push(VerificationFinding::error(
                "PV0603",
                "resolved signer does not expose concrete public key material",
            ));
            continue;
        };

        if !public_key
            .algorithm
            .eq_ignore_ascii_case(ALLOWED_SIGNATURE_ALGORITHM)
        {
            findings.push(VerificationFinding::error(
                "PV0604",
                "registry public key algorithm is not allowlisted for detached signature verification",
            ));
            continue;
        }

        if matches!(resolved_signer.status, KeyStatus::Unknown) {
            findings.push(VerificationFinding::error(
                "PV0605",
                "resolved signer key state is unknown and cannot be used for signature verification",
            ));
            continue;
        }

        if let Err(finding) = verify_ed25519_bytes(
            &public_key.public_key,
            &signature.signature,
            bundle_id.as_bytes(),
            "PV0610",
            "detached signature verification failed for resolved signer",
        ) {
            findings.push(finding);
        }
    }

    findings
}

pub fn sign_ed25519_bytes(
    private_key_material: &str,
    payload: &[u8],
) -> Result<String, VerifierRuntimeError> {
    let private_key_bytes =
        decode_base64_config_material(private_key_material, "receipt signer private key")?;
    let signing_key = signing_key_from_bytes(&private_key_bytes)?;
    let signature = signing_key.sign(payload);
    Ok(format!("base64:{}", STANDARD.encode(signature.to_bytes())))
}

pub fn verify_ed25519_bytes(
    public_key_material: &str,
    signature_material: &str,
    payload: &[u8],
    invalid_signature_code: &str,
    invalid_signature_message: &str,
) -> Result<(), VerificationFinding> {
    let public_key_bytes =
        decode_base64_material(public_key_material, "PV0606", "registry public key")?;
    let signature_bytes =
        decode_base64_material(signature_material, "PV0607", "detached signature")?;
    let verifying_key = verifying_key_from_bytes(&public_key_bytes)?;
    let detached_signature = detached_signature_from_bytes(&signature_bytes)?;

    verifying_key
        .verify(payload, &detached_signature)
        .map_err(|_| VerificationFinding::error(invalid_signature_code, invalid_signature_message))
}

pub fn is_allowed_signature_algorithm(value: &str) -> bool {
    value.eq_ignore_ascii_case(ALLOWED_SIGNATURE_ALGORITHM)
}

fn resolved_signer_index<'a>(
    resolved_signers: &'a [ResolvedSigner],
) -> BTreeMap<(&'a str, &'a str), &'a ResolvedSigner> {
    let mut index = BTreeMap::new();
    for signer in resolved_signers {
        index.insert(
            (
                signer.signer_id.as_str(),
                signer.producer_pubkey_id.as_str(),
            ),
            signer,
        );
    }
    index
}

fn decode_base64_material(
    value: &str,
    code: &str,
    label: &str,
) -> Result<Vec<u8>, VerificationFinding> {
    let encoded = value.strip_prefix("base64:").unwrap_or(value);
    STANDARD.decode(encoded).map_err(|_| {
        VerificationFinding::error(code, format!("{label} is not valid base64 material"))
    })
}

fn decode_base64_config_material(
    value: &str,
    label: &str,
) -> Result<Vec<u8>, VerifierRuntimeError> {
    let encoded = value.strip_prefix("base64:").unwrap_or(value);
    STANDARD
        .decode(encoded)
        .map_err(|_| VerifierRuntimeError::config(format!("{label} is not valid base64 material")))
}

fn verifying_key_from_bytes(bytes: &[u8]) -> Result<VerifyingKey, VerificationFinding> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        VerificationFinding::error(
            "PV0608",
            "registry public key is not 32-byte Ed25519 material",
        )
    })?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| {
        VerificationFinding::error(
            "PV0608",
            "registry public key bytes are not a valid Ed25519 verifying key",
        )
    })
}

fn detached_signature_from_bytes(bytes: &[u8]) -> Result<Signature, VerificationFinding> {
    Signature::from_slice(bytes).map_err(|_| {
        VerificationFinding::error(
            "PV0609",
            "detached signature bytes are not valid Ed25519 signature material",
        )
    })
}

fn signing_key_from_bytes(bytes: &[u8]) -> Result<SigningKey, VerifierRuntimeError> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        VerifierRuntimeError::config("receipt signer private key is not 32-byte Ed25519 material")
    })?;
    Ok(SigningKey::from_bytes(&bytes))
}
