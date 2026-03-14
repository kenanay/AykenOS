use crate::crypto::ed25519::{is_allowed_signature_algorithm, sign_ed25519_bytes};
use crate::errors::VerifierRuntimeError;
use crate::receipt::schema::{
    build_bootstrap_unsigned_receipt, build_receipt_payload, build_signed_receipt,
    canonicalize_receipt_payload,
};
use crate::types::{ReceiptSignerConfig, VerdictSubject, VerificationReceipt, VerificationVerdict};

pub fn emit_unsigned_receipt(
    subject: &VerdictSubject,
    verdict: VerificationVerdict,
) -> VerificationReceipt {
    build_bootstrap_unsigned_receipt(subject, verdict)
}

pub fn emit_signed_receipt(
    subject: &VerdictSubject,
    verdict: VerificationVerdict,
    signer: &ReceiptSignerConfig,
) -> Result<VerificationReceipt, VerifierRuntimeError> {
    if !is_allowed_signature_algorithm(&signer.signature_algorithm) {
        return Err(VerifierRuntimeError::config(
            "receipt signer signature_algorithm is not allowlisted",
        ));
    }

    let payload = build_receipt_payload(
        subject,
        verdict,
        &signer.verifier_node_id,
        Some(signer.verifier_key_id.clone()),
        &signer.verified_at_utc,
    );
    let payload_bytes = canonicalize_receipt_payload(&payload)?;
    let signature = sign_ed25519_bytes(&signer.private_key, &payload_bytes)?;

    Ok(build_signed_receipt(
        payload,
        &signer.signature_algorithm.to_ascii_lowercase(),
        signature,
    ))
}
