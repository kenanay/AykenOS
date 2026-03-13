use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use crate::errors::VerifierRuntimeError;
use crate::types::{
    VerdictSubject, VerificationAuditEvent, VerificationReceipt, VerificationVerdict,
};
use serde_json::Value;

pub fn build_audit_event(
    subject: &VerdictSubject,
    verdict: VerificationVerdict,
    receipt: &VerificationReceipt,
    previous_event_hash: Option<String>,
) -> Result<VerificationAuditEvent, VerifierRuntimeError> {
    let receipt_hash = compute_receipt_hash(receipt)?;
    let mut event = VerificationAuditEvent {
        event_version: 1,
        event_type: "verification".to_string(),
        event_id: String::new(),
        event_time_utc: receipt.payload.verified_at_utc.clone(),
        verifier_node_id: receipt.payload.verifier_node_id.clone(),
        verifier_key_id: receipt.payload.verifier_key_id.clone(),
        bundle_id: subject.bundle_id.clone(),
        trust_overlay_hash: subject.trust_overlay_hash.clone(),
        policy_hash: subject.policy_hash.clone(),
        registry_snapshot_hash: subject.registry_snapshot_hash.clone(),
        verdict,
        receipt_hash,
        previous_event_hash,
    };
    event.event_id = format!("sha256:{}", compute_audit_event_hash(&event)?);
    Ok(event)
}

pub fn compute_audit_event_hash(
    event: &VerificationAuditEvent,
) -> Result<String, VerifierRuntimeError> {
    let mut event_value = serde_json::to_value(event)
        .map_err(|error| VerifierRuntimeError::json("serialize audit event", error))?;
    if let Value::Object(map) = &mut event_value {
        map.remove("event_id");
    }
    let bytes = canonicalize_json_value(&event_value)?;
    Ok(sha256_hex(&bytes))
}

pub fn compute_receipt_hash(receipt: &VerificationReceipt) -> Result<String, VerifierRuntimeError> {
    let bytes = canonicalize_json(receipt)?;
    Ok(sha256_hex(&bytes))
}
