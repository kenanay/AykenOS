pub mod audit;
pub mod authority;
pub mod bundle;
pub mod canonical;
pub mod crypto;
pub mod errors;
pub mod overlay;
pub mod policy;
pub mod portable_core;
pub mod receipt;
pub mod registry;
pub mod testing;
pub mod types;
pub mod verdict;

use audit::ledger::append_verification_audit_event;
use bundle::checksums::load_checksums;
use bundle::layout::validate_bundle_layout;
use bundle::loader::load_bundle;
use bundle::manifest::load_manifest;
use crypto::verify_detached_signatures;
use errors::VerifierRuntimeError;
use overlay::overlay_validator::verify_overlay;
use policy::policy_engine::{compute_policy_hash, evaluate_policy};
use portable_core::checksum_validator::validate_portable_checksums;
use portable_core::identity::recompute_bundle_id;
use portable_core::proof_chain_validator::validate_proof_chain;
use receipt::receipt_emitter::{emit_signed_receipt, emit_unsigned_receipt};
use registry::resolver::resolve_signers;
use types::{
    AuditMode, FindingSeverity, PolicyDecision, ReceiptMode, VerificationFinding,
    VerificationOutcome, VerificationVerdict, VerifyRequest,
};
use verdict::subject::build_verdict_subject;
use verdict::verdict_engine::build_outcome;

pub use errors::VerifierRuntimeError as Error;
pub use types::{
    ChecksumsFile, DetachedSignature, DistributedReceiptVerification, KeyStatus, LoadedBundle,
    Manifest, ProducerDeclaration, ReceiptSignerConfig, ReceiptVerifierKey, RegistryEntry,
    RegistryPublicKey, RegistrySnapshot, ResolvedSigner, SignatureEnvelope, SignatureRequirement,
    TrustPolicy, VerdictSubject, VerificationAuditEvent, VerificationReceipt,
    VerificationReceiptPayload, VerificationVerdict as Verdict, VerifierAuthorityNode,
    VerifierAuthorityResolution, VerifierAuthorityResolutionClass, VerifierAuthorityState,
    VerifierDelegationEdge, VerifierTrustRegistryPublicKey, VerifierTrustRegistrySnapshot,
};

pub fn verify_bundle(
    request: &VerifyRequest<'_>,
) -> Result<VerificationOutcome, VerifierRuntimeError> {
    let loaded_bundle = load_bundle(request.bundle_path);
    let policy_hash = compute_policy_hash(request.policy)?;
    let mut findings = Vec::new();
    let mut bundle_id = String::new();
    let mut trust_overlay_hash = String::new();
    let mut registry_snapshot_hash = request.registry_snapshot.registry_snapshot_hash.clone();

    findings.extend(validate_bundle_layout(&loaded_bundle));
    if has_errors(&findings) {
        return finalize_outcome(
            VerificationVerdict::Invalid,
            &bundle_id,
            &trust_overlay_hash,
            &policy_hash,
            &registry_snapshot_hash,
            request,
            findings,
        );
    }

    let manifest = load_manifest(&loaded_bundle.manifest_path)?;
    let checksums = load_checksums(&loaded_bundle.checksums_path)?;

    findings.extend(validate_portable_checksums(&loaded_bundle, &checksums)?);
    findings.extend(validate_proof_chain(&loaded_bundle)?);

    bundle_id = recompute_bundle_id(&manifest, &checksums)?;
    if bundle_id != manifest.bundle_id {
        findings.push(error_finding(
            "PV0203",
            "recomputed bundle_id does not match manifest.bundle_id",
        ));
    }

    let overlay_state = verify_overlay(&loaded_bundle, &bundle_id)?;
    trust_overlay_hash = overlay_state.trust_overlay_hash.clone();
    findings.extend(overlay_state.findings.iter().cloned());

    let registry_resolution = resolve_signers(
        request.registry_snapshot,
        &overlay_state.producer,
        &overlay_state.signature_envelope,
    )?;
    registry_snapshot_hash = registry_resolution.registry_snapshot_hash.clone();
    findings.extend(registry_resolution.findings.iter().cloned());
    findings.extend(verify_detached_signatures(
        &bundle_id,
        &overlay_state.signature_envelope,
        &registry_resolution.resolved_signers,
    ));

    let policy_decision = evaluate_policy(
        request.policy,
        &overlay_state.producer,
        &registry_resolution.resolved_signers,
    )?;
    findings.extend(policy_decision.findings.iter().cloned());

    let verdict = derive_verdict(&findings, &policy_decision);
    finalize_outcome(
        verdict,
        &bundle_id,
        &trust_overlay_hash,
        &policy_hash,
        &registry_snapshot_hash,
        request,
        findings,
    )
}

fn finalize_outcome(
    verdict: VerificationVerdict,
    bundle_id: &str,
    trust_overlay_hash: &str,
    policy_hash: &str,
    registry_snapshot_hash: &str,
    request: &VerifyRequest<'_>,
    findings: Vec<VerificationFinding>,
) -> Result<VerificationOutcome, VerifierRuntimeError> {
    let subject = build_verdict_subject(
        bundle_id,
        trust_overlay_hash,
        policy_hash,
        registry_snapshot_hash,
    );
    let receipt = match request.receipt_mode {
        ReceiptMode::None => None,
        ReceiptMode::EmitUnsigned => Some(emit_unsigned_receipt(&subject, verdict.clone())),
        ReceiptMode::EmitSigned => {
            let signer = request.receipt_signer.ok_or_else(|| {
                VerifierRuntimeError::config(
                    "receipt_mode=EmitSigned requires receipt_signer configuration",
                )
            })?;
            Some(emit_signed_receipt(&subject, verdict.clone(), signer)?)
        }
    };
    let audit_event = match request.audit_mode {
        AuditMode::None => None,
        AuditMode::Append => {
            let ledger_path = request.audit_ledger_path.ok_or_else(|| {
                VerifierRuntimeError::config(
                    "audit_mode=Append requires audit_ledger_path configuration",
                )
            })?;
            let receipt = receipt.as_ref().ok_or_else(|| {
                VerifierRuntimeError::config(
                    "audit_mode=Append requires receipt emission before audit append",
                )
            })?;
            Some(append_verification_audit_event(
                ledger_path,
                &subject,
                verdict.clone(),
                receipt,
            )?)
        }
    };
    Ok(build_outcome(
        verdict,
        subject,
        findings,
        receipt,
        audit_event,
    ))
}

fn derive_verdict(
    findings: &[VerificationFinding],
    policy_decision: &PolicyDecision,
) -> VerificationVerdict {
    if has_errors(findings) {
        return VerificationVerdict::Invalid;
    }

    policy_decision.verdict.clone()
}

fn has_errors(findings: &[VerificationFinding]) -> bool {
    findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Error)
}

fn error_finding(code: &str, message: &str) -> VerificationFinding {
    VerificationFinding::error(code, message)
}
