use crate::canonical::jcs::canonicalize_json;
use crate::errors::VerifierRuntimeError;
use crate::types::{
    VerdictSubject, VerificationReceipt, VerificationReceiptPayload, VerificationVerdict,
};

const DEFAULT_VERIFIER_NODE_ID: &str = "local-node";
const DEFAULT_VERIFIED_AT_UTC: &str = "1970-01-01T00:00:00Z";

pub fn build_receipt_payload(
    subject: &VerdictSubject,
    verdict: VerificationVerdict,
    verifier_node_id: &str,
    verifier_key_id: Option<String>,
    verified_at_utc: &str,
) -> VerificationReceiptPayload {
    VerificationReceiptPayload {
        receipt_version: 1,
        bundle_id: subject.bundle_id.clone(),
        trust_overlay_hash: subject.trust_overlay_hash.clone(),
        policy_hash: subject.policy_hash.clone(),
        registry_snapshot_hash: subject.registry_snapshot_hash.clone(),
        verifier_node_id: verifier_node_id.to_string(),
        verifier_key_id,
        verdict,
        verified_at_utc: verified_at_utc.to_string(),
    }
}

pub fn build_unsigned_receipt(payload: VerificationReceiptPayload) -> VerificationReceipt {
    VerificationReceipt {
        payload,
        verifier_signature_algorithm: None,
        verifier_signature: None,
    }
}

pub fn build_signed_receipt(
    payload: VerificationReceiptPayload,
    signature_algorithm: &str,
    signature: String,
) -> VerificationReceipt {
    VerificationReceipt {
        payload,
        verifier_signature_algorithm: Some(signature_algorithm.to_string()),
        verifier_signature: Some(signature),
    }
}

pub fn canonicalize_receipt_payload(
    payload: &VerificationReceiptPayload,
) -> Result<Vec<u8>, VerifierRuntimeError> {
    canonicalize_json(payload)
}

pub fn build_bootstrap_unsigned_receipt(
    subject: &VerdictSubject,
    verdict: VerificationVerdict,
) -> VerificationReceipt {
    let payload = build_receipt_payload(
        subject,
        verdict,
        DEFAULT_VERIFIER_NODE_ID,
        None,
        DEFAULT_VERIFIED_AT_UTC,
    );
    build_unsigned_receipt(payload)
}
