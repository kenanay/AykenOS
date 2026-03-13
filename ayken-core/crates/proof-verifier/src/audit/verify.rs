use crate::audit::ledger::load_audit_events;
use crate::audit::schema::{compute_audit_event_hash, compute_receipt_hash};
use crate::errors::VerifierRuntimeError;
use crate::receipt::verify::{verify_signed_receipt, verify_signed_receipt_with_authority};
use crate::types::{
    ReceiptVerifierKey, VerdictSubject, VerificationAuditEvent, VerificationFinding,
    VerificationReceipt, VerifierTrustRegistrySnapshot,
};
use std::collections::BTreeMap;
use std::path::Path;

pub struct AuditReceiptBinding<'a> {
    pub receipt: &'a VerificationReceipt,
    pub verifier_key: &'a ReceiptVerifierKey,
    pub verifier_registry: Option<&'a VerifierTrustRegistrySnapshot>,
}

pub fn verify_audit_ledger(
    ledger_path: &Path,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let events = load_audit_events(ledger_path)?;
    let mut findings = Vec::new();
    let mut previous_event_id: Option<String> = None;

    for event in &events {
        findings.extend(validate_event_shape(event));

        let expected_event_id = format!("sha256:{}", compute_audit_event_hash(event)?);
        if event.event_id != expected_event_id {
            findings.push(VerificationFinding::error(
                "PV0801",
                "audit event_id does not match canonical recomputed audit event hash",
            ));
        }

        if event.previous_event_hash != previous_event_id {
            findings.push(VerificationFinding::error(
                "PV0802",
                "audit ledger previous_event_hash does not match prior event identity",
            ));
        }

        previous_event_id = Some(event.event_id.clone());
    }

    Ok(findings)
}

pub fn verify_audit_event_against_receipt(
    event: &VerificationAuditEvent,
    receipt: &VerificationReceipt,
    verifier_key: &ReceiptVerifierKey,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let mut findings = validate_event_against_receipt_binding(event, receipt)?;
    let expected_subject = VerdictSubject {
        bundle_id: event.bundle_id.clone(),
        trust_overlay_hash: event.trust_overlay_hash.clone(),
        policy_hash: event.policy_hash.clone(),
        registry_snapshot_hash: event.registry_snapshot_hash.clone(),
    };
    findings.extend(verify_signed_receipt(
        receipt,
        &expected_subject,
        verifier_key,
    )?);

    Ok(findings)
}

pub fn verify_audit_event_against_receipt_with_authority(
    event: &VerificationAuditEvent,
    receipt: &VerificationReceipt,
    verifier_key: &ReceiptVerifierKey,
    verifier_registry: &VerifierTrustRegistrySnapshot,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let mut findings = validate_event_against_receipt_binding(event, receipt)?;

    let expected_subject = VerdictSubject {
        bundle_id: event.bundle_id.clone(),
        trust_overlay_hash: event.trust_overlay_hash.clone(),
        policy_hash: event.policy_hash.clone(),
        registry_snapshot_hash: event.registry_snapshot_hash.clone(),
    };
    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &expected_subject,
        verifier_key,
        verifier_registry,
    )?;
    findings.extend(distributed.findings);
    Ok(findings)
}

pub fn verify_audit_ledger_with_receipts(
    ledger_path: &Path,
    bindings: &BTreeMap<String, AuditReceiptBinding<'_>>,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let events = load_audit_events(ledger_path)?;
    let mut findings = verify_audit_ledger(ledger_path)?;

    for event in &events {
        let Some(binding) = bindings.get(&event.receipt_hash) else {
            findings.push(VerificationFinding::error(
                "PV0807",
                "audit ledger is missing receipt binding material for receipt_hash",
            ));
            continue;
        };
        if let Some(verifier_registry) = binding.verifier_registry {
            findings.extend(verify_audit_event_against_receipt_with_authority(
                event,
                binding.receipt,
                binding.verifier_key,
                verifier_registry,
            )?);
        } else {
            findings.extend(verify_audit_event_against_receipt(
                event,
                binding.receipt,
                binding.verifier_key,
            )?);
        }
    }

    Ok(findings)
}

fn validate_event_against_receipt_binding(
    event: &VerificationAuditEvent,
    receipt: &VerificationReceipt,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let mut findings = Vec::new();
    let expected_receipt_hash = compute_receipt_hash(receipt)?;
    if event.receipt_hash != expected_receipt_hash {
        findings.push(VerificationFinding::error(
            "PV0803",
            "audit event receipt_hash does not match canonical recomputed receipt hash",
        ));
    }

    if event.bundle_id != receipt.payload.bundle_id
        || event.trust_overlay_hash != receipt.payload.trust_overlay_hash
        || event.policy_hash != receipt.payload.policy_hash
        || event.registry_snapshot_hash != receipt.payload.registry_snapshot_hash
        || event.verdict != receipt.payload.verdict
    {
        findings.push(VerificationFinding::error(
            "PV0805",
            "audit event subject tuple does not match receipt payload",
        ));
    }

    if event.verifier_node_id != receipt.payload.verifier_node_id
        || event.verifier_key_id != receipt.payload.verifier_key_id
    {
        findings.push(VerificationFinding::error(
            "PV0806",
            "audit event verifier identity does not match receipt payload",
        ));
    }

    Ok(findings)
}

fn validate_event_shape(event: &VerificationAuditEvent) -> Vec<VerificationFinding> {
    let mut findings = Vec::new();
    if event.event_version != 1 {
        findings.push(VerificationFinding::error(
            "PV0804",
            "audit event_version is unsupported",
        ));
    }
    if event.event_type != "verification" {
        findings.push(VerificationFinding::error(
            "PV0804",
            "audit event_type must be verification",
        ));
    }
    if !event.event_id.starts_with("sha256:") {
        findings.push(VerificationFinding::error(
            "PV0804",
            "audit event_id must use sha256: prefix",
        ));
    }
    if !is_sha256_hex(&event.receipt_hash) {
        findings.push(VerificationFinding::error(
            "PV0804",
            "audit receipt_hash must be a 64-character lowercase SHA-256 hex digest",
        ));
    }
    findings
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}
