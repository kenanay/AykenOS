use crate::audit::verify::{
    verify_audit_event_against_receipt, verify_audit_event_against_receipt_with_authority,
    verify_audit_ledger, verify_audit_ledger_with_receipts, AuditReceiptBinding,
};
use crate::authority::parity::{
    compare_authority_resolution, compare_cross_node_parity, CrossNodeParityInput,
    CrossNodeParityStatus,
};
use crate::authority::resolution::resolve_verifier_authority;
use crate::authority::snapshot::compute_verifier_trust_registry_snapshot_hash;
use crate::receipt::verify::{
    verify_signed_receipt, verify_signed_receipt_with_authority,
    verify_signed_receipt_with_resolved_authority,
};
use crate::types::SignatureEnvelope;
use crate::types::{
    AuditMode, ReceiptMode, VerificationFinding, VerificationVerdict, VerifierAuthorityNode,
    VerifierAuthorityResolution, VerifierAuthorityResolutionClass, VerifierAuthorityState,
    VerifierDelegationEdge, VerifyRequest,
};
use crate::verify_bundle;
use std::collections::BTreeMap;

use super::fixtures::create_fixture_bundle;

#[test]
fn verify_bundle_builds_subject_and_signed_receipt_from_fixture() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should not fail at runtime");

    assert_eq!(outcome.verdict, VerificationVerdict::Trusted);
    assert_eq!(outcome.subject.bundle_id.len(), 64);
    assert_eq!(outcome.subject.trust_overlay_hash.len(), 64);
    assert_eq!(
        outcome.subject.registry_snapshot_hash,
        fixture.registry.registry_snapshot_hash
    );
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    assert_eq!(
        receipt.payload.verifier_node_id,
        fixture.receipt_signer.verifier_node_id
    );
    assert_eq!(
        receipt.payload.verifier_key_id.as_deref(),
        Some(fixture.receipt_signer.verifier_key_id.as_str())
    );
    assert_eq!(
        receipt.verifier_signature_algorithm.as_deref(),
        Some("ed25519")
    );
    let receipt_findings =
        verify_signed_receipt(receipt, &outcome.subject, &fixture.receipt_verifier_key)
            .expect("signed receipt verification should not fail at runtime");
    assert!(receipt_findings.is_empty());
    assert!(outcome
        .findings
        .iter()
        .all(|finding| finding.severity != crate::types::FindingSeverity::Error));
}

#[test]
fn verify_signed_receipt_binds_to_current_verifier_authority() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .expect("receipt authority binding should not fail at runtime");

    assert_eq!(
        distributed.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityResolvedDelegated
    );
    assert!(distributed
        .authority_resolution
        .authority_chain_id
        .as_deref()
        .map(|value| value.starts_with("sha256:"))
        .unwrap_or(false));
    assert!(distributed.findings.is_empty());
}

#[test]
fn verify_bundle_fails_closed_when_required_path_is_missing() {
    let fixture = create_fixture_bundle();
    std::fs::remove_file(fixture.root.join("manifest.json")).expect("manifest should be removable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("missing path should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0100"));
}

#[test]
fn verify_bundle_rejects_tampered_detached_signature() {
    let fixture = create_fixture_bundle();
    let signature_path = fixture.root.join("signatures/signature-envelope.json");
    let mut envelope: SignatureEnvelope = serde_json::from_slice(
        &std::fs::read(&signature_path).expect("signature envelope should exist"),
    )
    .expect("signature envelope fixture should parse");
    envelope.signatures[0].signature = "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_string();
    std::fs::write(
        &signature_path,
        serde_json::to_vec(&envelope).expect("tampered signature envelope should serialize"),
    )
    .expect("tampered signature envelope should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("tampered signature should produce deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0610"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_binding_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["ledger_root_hash"] = serde_json::Value::String("f".repeat(64));
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("proof manifest drift should produce deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0214" || finding.code == "PV0231"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_contract_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["mode"] = serde_json::Value::String("wrong-mode".to_string());
    proof_manifest["signature_mode"] = serde_json::Value::String("bootstrap-none".to_string());
    proof_manifest["signer_sig"] = serde_json::Value::String("base64:AAAA".to_string());
    proof_manifest["final_state_hash"] = serde_json::Value::String("not-a-digest".to_string());
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request)
        .expect("proof manifest contract drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0245"));
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0247"));
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0252"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_signature_mode_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["signature_mode"] = serde_json::Value::String("detached".to_string());
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request)
        .expect("proof manifest signature_mode drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0246"));
}

#[test]
fn verify_bundle_rejects_replay_trace_binding_drift() {
    let fixture = create_fixture_bundle();
    let replay_report_path = fixture.root.join("reports/replay_report.json");
    let mut replay_report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&replay_report_path).expect("replay report should exist"),
    )
    .expect("replay report should parse");
    replay_report["replay_execution_trace_hash"] = serde_json::Value::String("f".repeat(64));
    std::fs::write(
        &replay_report_path,
        serde_json::to_vec(&replay_report).expect("tampered replay report should serialize"),
    )
    .expect("tampered replay report should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request)
        .expect("replay trace binding drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0251"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_event_count_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["event_count"] = serde_json::Value::Number(serde_json::Number::from(7u64));
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("event_count drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0239"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_violation_count_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["violation_count"] = serde_json::Value::Number(serde_json::Number::from(3u64));
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("violation_count drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0240"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_proof_hash_shape_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["proof_hash"] = serde_json::Value::String("not-a-digest".to_string());
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("proof_hash shape drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0252"));
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0214"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_replay_result_hash_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["replay_result_hash"] = serde_json::Value::String("f".repeat(64));
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("replay_result_hash drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0237"));
}

#[test]
fn verify_bundle_rejects_proof_manifest_config_and_kernel_hash_shape_drift() {
    let fixture = create_fixture_bundle();
    let proof_manifest_path = fixture.root.join("reports/proof_manifest.json");
    let mut proof_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&proof_manifest_path).expect("proof manifest should exist"),
    )
    .expect("proof manifest should parse");
    proof_manifest["config_hash"] = serde_json::Value::String("broken-config-digest".to_string());
    proof_manifest["kernel_image_hash"] =
        serde_json::Value::String("broken-kernel-digest".to_string());
    std::fs::write(
        &proof_manifest_path,
        serde_json::to_vec(&proof_manifest).expect("tampered proof manifest should serialize"),
    )
    .expect("tampered proof manifest should be writable");

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request)
        .expect("config/kernel hash shape drift should be deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0252"));
}

#[test]
fn verify_bundle_rejects_registry_snapshot_hash_drift() {
    let fixture = create_fixture_bundle();
    let mut registry = fixture.registry.clone();
    registry.registry_snapshot_hash = "f".repeat(64);

    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome =
        verify_bundle(&request).expect("registry hash drift should produce deterministic invalid");

    assert_eq!(outcome.verdict, VerificationVerdict::Invalid);
    assert!(outcome
        .findings
        .iter()
        .any(|finding| finding.code == "PV0410"));
    assert_eq!(
        outcome.subject.registry_snapshot_hash,
        fixture.registry.registry_snapshot_hash
    );
}

#[test]
fn verify_signed_receipt_rejects_tampered_signature() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let mut receipt = outcome.receipt.expect("signed receipt should exist");
    receipt.verifier_signature =
        Some("base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_string());

    let findings = verify_signed_receipt(&receipt, &outcome.subject, &fixture.receipt_verifier_key)
        .expect("tampered receipt verification should not fail at runtime");

    assert!(findings.iter().any(|finding| finding.code == "PV0708"));
}

#[test]
fn verify_signed_receipt_rejects_subject_mismatch() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut mismatched_subject = outcome.subject.clone();
    mismatched_subject.registry_snapshot_hash = "f".repeat(64);

    let findings =
        verify_signed_receipt(receipt, &mismatched_subject, &fixture.receipt_verifier_key)
            .expect("receipt subject mismatch should not fail at runtime");

    assert!(findings.iter().any(|finding| finding.code == "PV0701"));
}

#[test]
fn verify_signed_receipt_rejects_verifier_authority_key_material_mismatch() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry
        .public_keys
        .get_mut("receipt-ed25519-key-2026-03-a")
        .expect("receipt verifier trust registry public key should exist")
        .public_key = "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("tampered verifier registry hash should recompute");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &verifier_registry,
    )
    .expect("authority-bound receipt verification should not fail at runtime");

    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0718"));
}

#[test]
fn verify_signed_receipt_rejects_missing_authority_chain_id_from_resolved_authority() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let forged_resolution = VerifierAuthorityResolution {
        result_class: VerifierAuthorityResolutionClass::AuthorityResolvedDelegated,
        requested_verifier_id: "node-b".to_string(),
        requested_authority_scope: vec!["distributed-receipt-issuer".to_string()],
        authority_chain: vec!["root-verifier-a".to_string(), "node-b".to_string()],
        authority_chain_id: None,
        effective_authority_scope: vec!["distributed-receipt-issuer".to_string()],
        verifier_registry_snapshot_hash: fixture
            .verifier_registry
            .verifier_registry_snapshot_hash
            .clone(),
        findings: Vec::<VerificationFinding>::new(),
    };

    let distributed = verify_signed_receipt_with_resolved_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
        forged_resolution,
    )
    .expect("forged missing chain id should still verify deterministically");

    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0713"));
}

#[test]
fn verify_signed_receipt_rejects_historical_only_verifier_authority() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry
        .verifiers
        .get_mut("node-b")
        .expect("verifier node-b should exist")
        .authority_state = VerifierAuthorityState::HistoricalOnly;
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("historical verifier registry hash should recompute");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &verifier_registry,
    )
    .expect("historical-only authority binding should not fail at runtime");

    assert_eq!(
        distributed.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityHistoricalOnly
    );
    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0711"));
}

#[test]
fn verify_signed_receipt_rejects_revoked_verifier_authority() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry
        .verifiers
        .get_mut("node-b")
        .expect("verifier node-b should exist")
        .authority_state = VerifierAuthorityState::Revoked;
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("revoked verifier registry hash should recompute");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &verifier_registry,
    )
    .expect("revoked authority binding should not fail at runtime");

    assert_eq!(
        distributed.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityNoValidChain
    );
    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0712"));
}

#[test]
fn verify_signed_receipt_rejects_orphan_verifier_outside_root_set() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry.delegation_edges.clear();
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("orphan verifier registry hash should recompute");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &verifier_registry,
    )
    .expect("orphan authority binding should not fail at runtime");

    assert_eq!(
        distributed.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityNoValidChain
    );
    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0712"));
}

#[test]
fn verify_signed_receipt_rejects_authority_scope_mismatch() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry
        .verifiers
        .get_mut("node-b")
        .expect("verifier node-b should exist")
        .authority_scope = vec!["parity-reporter".to_string()];
    verifier_registry
        .delegation_edges
        .get_mut(0)
        .expect("delegation edge should exist")
        .delegated_scope = vec!["parity-reporter".to_string()];
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("scope-mismatch verifier registry hash should recompute");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &verifier_registry,
    )
    .expect("scope mismatch binding should not fail at runtime");

    assert_eq!(
        distributed.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityNoValidChain
    );
    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0712"));
}

#[test]
fn verify_signed_receipt_rejects_verifier_authority_algorithm_drift() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };

    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry
        .public_keys
        .get_mut("receipt-ed25519-key-2026-03-a")
        .expect("receipt verifier trust registry public key should exist")
        .algorithm = "rsa".to_string();
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("algorithm-drift verifier registry hash should recompute");

    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &verifier_registry,
    )
    .expect("algorithm drift binding should not fail at runtime");

    assert!(distributed
        .findings
        .iter()
        .any(|finding| finding.code == "PV0717"));
}

#[test]
fn verify_bundle_appends_audit_event_and_verifies_chain() {
    let fixture = create_fixture_bundle();
    let ledger_path = fixture.root.join("audit/verification_audit_ledger.jsonl");
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::Append,
        audit_ledger_path: Some(&ledger_path),
    };

    let outcome = verify_bundle(&request).expect("audit append should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let audit_event = outcome
        .audit_event
        .as_ref()
        .expect("audit event should be returned");

    let ledger_findings =
        verify_audit_ledger(&ledger_path).expect("audit ledger verification should not fail");
    assert!(ledger_findings.is_empty());

    let receipt_findings = verify_audit_event_against_receipt_with_authority(
        audit_event,
        receipt,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .expect("audit event vs receipt verification should not fail");
    assert!(receipt_findings.is_empty());

    let mut bindings = BTreeMap::new();
    bindings.insert(
        audit_event.receipt_hash.clone(),
        AuditReceiptBinding {
            receipt,
            verifier_key: &fixture.receipt_verifier_key,
            verifier_registry: Some(&fixture.verifier_registry),
        },
    );
    let full_findings = verify_audit_ledger_with_receipts(&ledger_path, &bindings)
        .expect("full audit ledger verification should not fail");
    assert!(full_findings.is_empty());
}

#[test]
fn verify_audit_ledger_rejects_tampered_chain() {
    let fixture = create_fixture_bundle();
    let ledger_path = fixture.root.join("audit/verification_audit_ledger.jsonl");
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::Append,
        audit_ledger_path: Some(&ledger_path),
    };

    verify_bundle(&request).expect("first audit append should succeed");
    verify_bundle(&request).expect("second audit append should succeed");

    let raw = std::fs::read_to_string(&ledger_path).expect("audit ledger should exist");
    let mut lines: Vec<serde_json::Value> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("audit event should parse"))
        .collect();
    lines[1]["previous_event_hash"] =
        serde_json::Value::String(format!("sha256:{}", "f".repeat(64)));
    let rewritten = lines
        .into_iter()
        .map(|value| serde_json::to_string(&value).expect("audit event should serialize"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&ledger_path, rewritten).expect("tampered audit ledger should be writable");

    let findings =
        verify_audit_ledger(&ledger_path).expect("tampered audit ledger should still parse");
    assert!(findings.iter().any(|finding| finding.code == "PV0802"));
}

#[test]
fn verify_audit_event_rejects_receipt_hash_mismatch() {
    let fixture = create_fixture_bundle();
    let ledger_path = fixture.root.join("audit/verification_audit_ledger.jsonl");
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::Append,
        audit_ledger_path: Some(&ledger_path),
    };

    let outcome = verify_bundle(&request).expect("audit append should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let mut audit_event = outcome
        .audit_event
        .clone()
        .expect("audit event should be returned");
    audit_event.receipt_hash = "f".repeat(64);

    let findings =
        verify_audit_event_against_receipt(&audit_event, receipt, &fixture.receipt_verifier_key)
            .expect("audit event mismatch verification should not fail");
    assert!(findings.iter().any(|finding| finding.code == "PV0803"));
}

#[test]
fn verify_audit_event_rejects_tampered_receipt_signature() {
    let fixture = create_fixture_bundle();
    let ledger_path = fixture.root.join("audit/verification_audit_ledger.jsonl");
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::Append,
        audit_ledger_path: Some(&ledger_path),
    };

    let outcome = verify_bundle(&request).expect("audit append should succeed");
    let mut receipt = outcome.receipt.expect("signed receipt should exist");
    let audit_event = outcome
        .audit_event
        .as_ref()
        .expect("audit event should be returned");
    receipt.verifier_signature =
        Some("base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_string());

    let findings =
        verify_audit_event_against_receipt(audit_event, &receipt, &fixture.receipt_verifier_key)
            .expect("tampered audit receipt verification should not fail");

    assert!(findings.iter().any(|finding| finding.code == "PV0708"));
}

#[test]
fn verify_bundle_rejects_audit_append_without_signed_receipt() {
    let fixture = create_fixture_bundle();
    let ledger_path = fixture.root.join("audit/verification_audit_ledger.jsonl");
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitUnsigned,
        receipt_signer: None,
        audit_mode: AuditMode::Append,
        audit_ledger_path: Some(&ledger_path),
    };

    let error = verify_bundle(&request).expect_err("unsigned receipt audit append must fail");
    assert!(error
        .to_string()
        .contains("audit append requires a signed verification receipt"));
}

#[test]
fn resolve_verifier_authority_builds_deterministic_chain_id() {
    let fixture = create_fixture_bundle();

    let resolution = resolve_verifier_authority(
        &fixture.verifier_registry,
        &fixture.authority_requested_verifier_id,
        &fixture.authority_requested_scope,
    )
    .expect("authority resolution should not fail at runtime");

    assert_eq!(
        resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityResolvedDelegated
    );
    assert_eq!(
        resolution.authority_chain,
        vec!["root-verifier-a".to_string(), "node-b".to_string()]
    );
    assert_eq!(
        resolution
            .authority_chain_id
            .as_deref()
            .map(|value| value.starts_with("sha256:")),
        Some(true),
    );
    assert_eq!(
        resolution.effective_authority_scope,
        fixture.authority_requested_scope
    );
    assert!(resolution.findings.is_empty());
}

#[test]
fn compare_authority_resolution_reports_chain_id_equality() {
    let fixture = create_fixture_bundle();
    let resolution = resolve_verifier_authority(
        &fixture.verifier_registry,
        &fixture.authority_requested_verifier_id,
        &fixture.authority_requested_scope,
    )
    .expect("authority resolution should succeed");
    let mut different_resolution = resolution.clone();
    different_resolution.authority_chain_id = Some(format!("sha256:{}", "f".repeat(64)));

    let same = compare_authority_resolution(&resolution, &resolution);
    assert_eq!(same.authority_chain_id_equal, Some(true));

    let different = compare_authority_resolution(&resolution, &different_resolution);
    assert_eq!(different.authority_chain_id_equal, Some(false));
}

#[test]
fn compare_cross_node_parity_reports_match_for_equal_authority_chain_id() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .expect("authority-bound receipt verification should succeed");

    let parity = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: "sha256:context-a",
            authority_resolution: &distributed.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-b",
            subject: &outcome.subject,
            verification_context_id: "sha256:context-a",
            authority_resolution: &distributed.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );

    assert_eq!(parity.parity_status, CrossNodeParityStatus::ParityMatch);
    assert_eq!(parity.authority_chain_id_equal, Some(true));
}

#[test]
fn compare_cross_node_parity_reports_verifier_mismatch_for_different_authority_chain_id() {
    let fixture = create_fixture_bundle();
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    let outcome = verify_bundle(&request).expect("fixture verification should succeed");
    let receipt = outcome
        .receipt
        .as_ref()
        .expect("signed receipt should exist");
    let distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .expect("baseline authority-bound receipt verification should succeed");

    let mut alternate_registry = fixture.verifier_registry.clone();
    alternate_registry.verifier_registry_epoch = 2;
    alternate_registry.root_verifier_ids = vec!["root-verifier-c".to_string()];
    alternate_registry.verifiers.insert(
        "root-verifier-c".to_string(),
        VerifierAuthorityNode {
            verifier_id: "root-verifier-c".to_string(),
            verifier_pubkey_id: "root-verifier-c-ed25519-key-2026-03-a".to_string(),
            authority_scope: vec![
                "context-distributor".to_string(),
                "distributed-receipt-issuer".to_string(),
                "parity-reporter".to_string(),
            ],
            authority_state: VerifierAuthorityState::Current,
        },
    );
    alternate_registry.public_keys.insert(
        "root-verifier-c-ed25519-key-2026-03-a".to_string(),
        crate::types::VerifierTrustRegistryPublicKey {
            algorithm: "ed25519".to_string(),
            public_key: fixture.receipt_verifier_key.public_key.clone(),
        },
    );
    alternate_registry.delegation_edges = vec![VerifierDelegationEdge {
        parent_verifier_id: "root-verifier-c".to_string(),
        delegate_verifier_id: "node-b".to_string(),
        delegated_scope: vec!["distributed-receipt-issuer".to_string()],
    }];
    alternate_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&alternate_registry)
            .expect("alternate verifier registry hash should recompute");
    let alternate_distributed = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &alternate_registry,
    )
    .expect("alternate authority-bound receipt verification should succeed");

    let parity = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: "sha256:context-a",
            authority_resolution: &distributed.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-c",
            subject: &outcome.subject,
            verification_context_id: "sha256:context-a",
            authority_resolution: &alternate_distributed.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );

    assert_eq!(
        parity.parity_status,
        CrossNodeParityStatus::ParityVerifierMismatch
    );
    assert_eq!(parity.authority_chain_id_equal, Some(false));
}

#[test]
fn resolve_verifier_authority_rejects_depth_overflow_as_distinct_class() {
    let fixture = create_fixture_bundle();
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry.delegation_edges.clear();
    verifier_registry.verifiers.remove("node-b");
    verifier_registry
        .public_keys
        .remove("receipt-ed25519-key-2026-03-a");

    let mut parent = "root-verifier-a".to_string();
    for index in 1..=9 {
        let verifier_id = format!("deep-node-{index}");
        let key_id = format!("deep-node-{index}-ed25519-key-2026-03-a");
        verifier_registry.verifiers.insert(
            verifier_id.clone(),
            VerifierAuthorityNode {
                verifier_id: verifier_id.clone(),
                verifier_pubkey_id: key_id.clone(),
                authority_scope: vec!["distributed-receipt-issuer".to_string()],
                authority_state: VerifierAuthorityState::Current,
            },
        );
        verifier_registry.public_keys.insert(
            key_id,
            crate::types::VerifierTrustRegistryPublicKey {
                algorithm: "ed25519".to_string(),
                public_key: fixture.receipt_verifier_key.public_key.clone(),
            },
        );
        verifier_registry
            .delegation_edges
            .push(VerifierDelegationEdge {
                parent_verifier_id: parent.clone(),
                delegate_verifier_id: verifier_id.clone(),
                delegated_scope: vec!["distributed-receipt-issuer".to_string()],
            });
        parent = verifier_id;
    }
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("deep verifier registry hash should recompute");

    let resolution = resolve_verifier_authority(
        &verifier_registry,
        "deep-node-9",
        &fixture.authority_requested_scope,
    )
    .expect("depth-overflow authority resolution should still complete deterministically");

    assert_eq!(
        resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityGraphDepthExceeded
    );
    assert!(resolution
        .findings
        .iter()
        .any(|finding| finding.code == "PV0911"));
}

#[test]
fn resolve_verifier_authority_rejects_ambiguous_parent_chains() {
    let fixture = create_fixture_bundle();
    let mut verifier_registry = fixture.verifier_registry.clone();
    verifier_registry
        .root_verifier_ids
        .push("root-verifier-b".to_string());
    verifier_registry.verifiers.insert(
        "root-verifier-b".to_string(),
        VerifierAuthorityNode {
            verifier_id: "root-verifier-b".to_string(),
            verifier_pubkey_id: "root-verifier-b-ed25519-key-2026-03-a".to_string(),
            authority_scope: vec!["distributed-receipt-issuer".to_string()],
            authority_state: VerifierAuthorityState::Current,
        },
    );
    verifier_registry.public_keys.insert(
        "root-verifier-b-ed25519-key-2026-03-a".to_string(),
        crate::types::VerifierTrustRegistryPublicKey {
            algorithm: "ed25519".to_string(),
            public_key: fixture.receipt_verifier_key.public_key.clone(),
        },
    );
    verifier_registry
        .delegation_edges
        .push(VerifierDelegationEdge {
            parent_verifier_id: "root-verifier-b".to_string(),
            delegate_verifier_id: "node-b".to_string(),
            delegated_scope: vec!["distributed-receipt-issuer".to_string()],
        });
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry)
            .expect("ambiguous verifier registry hash should recompute");

    let resolution = resolve_verifier_authority(
        &verifier_registry,
        &fixture.authority_requested_verifier_id,
        &fixture.authority_requested_scope,
    )
    .expect("ambiguous authority resolution should still complete deterministically");

    assert_eq!(
        resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityGraphAmbiguous
    );
    assert!(resolution
        .findings
        .iter()
        .any(|finding| finding.code == "PV0909"));
}
