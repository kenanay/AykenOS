use crate::{
    build_default_context_rules_object, compute_context_rules_hash, verdict_label, ServiceError,
    VERIFICATION_CONTEXT_VERIFIER_CONTRACT_VERSION,
};
use proof_verifier::registry::snapshot::compute_registry_snapshot_hash;
use proof_verifier::verification_context_object::{
    compute_verification_context_id, VerificationContextObject,
};
use proof_verifier::{types::VerificationOutcome, RegistrySnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::fingerprint::{canonical_hash_findings_prefixed, canonical_hash_prefixed};

const REPLAY_DETERMINISM_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct VerificationDeterminismContractCore {
    pub contract_version: u32,
    pub request_fingerprint: String,
    pub verdict: String,
    pub subject_hash: String,
    pub context_hash: String,
    pub authority_hash: String,
    pub findings_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_payload_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct VerificationDeterminismContractArtifact {
    pub contract: VerificationDeterminismContractCore,
    pub artifact_hash: String,
}

pub(crate) fn build_verification_context_material(
    outcome: &VerificationOutcome,
) -> Result<(Value, VerificationContextObject), ServiceError> {
    let context_rules = build_default_context_rules_object();
    let mut context = VerificationContextObject {
        context_version: 1,
        verification_context_id: String::new(),
        policy_hash: outcome.subject.policy_hash.clone(),
        registry_snapshot_hash: outcome.subject.registry_snapshot_hash.clone(),
        verifier_contract_version: VERIFICATION_CONTEXT_VERIFIER_CONTRACT_VERSION.to_string(),
        context_rules_hash: compute_context_rules_hash(&context_rules)?,
        context_epoch: None,
        historical_cutoff_utc: None,
        policy_snapshot_ref: None,
        registry_snapshot_ref: None,
        time_semantics_mode: None,
    };
    context.verification_context_id = compute_verification_context_id(&context)
        .map_err(|_| ServiceError::Runtime("verification_context_id_compute_failed"))?;
    Ok((context_rules, context))
}

pub(crate) fn build_verification_determinism_contract(
    registry: &RegistrySnapshot,
    outcome: &VerificationOutcome,
    request_fingerprint: &str,
) -> Result<VerificationDeterminismContractArtifact, ServiceError> {
    let (_, context) = build_verification_context_material(outcome)?;
    let authority_hash = format!(
        "sha256:{}",
        compute_registry_snapshot_hash(registry)
            .map_err(|_| ServiceError::Runtime("determinism_authority_hash_compute_failed"))?
    );
    let contract = VerificationDeterminismContractCore {
        contract_version: REPLAY_DETERMINISM_CONTRACT_VERSION,
        request_fingerprint: request_fingerprint.to_string(),
        verdict: verdict_label(&outcome.verdict).to_string(),
        subject_hash: canonical_hash_prefixed(
            &outcome.subject,
            "determinism_subject_hash_compute_failed",
        )?,
        context_hash: canonical_hash_prefixed(&context, "determinism_context_hash_compute_failed")?,
        authority_hash,
        findings_hash: canonical_hash_findings_prefixed(
            &outcome.findings,
            "determinism_findings_hash_compute_failed",
        )?,
        receipt_payload_hash: outcome
            .receipt
            .as_ref()
            .map(|receipt| {
                canonical_hash_prefixed(
                    &receipt.payload,
                    "determinism_receipt_payload_hash_compute_failed",
                )
            })
            .transpose()?,
    };
    let artifact_hash =
        canonical_hash_prefixed(&contract, "determinism_contract_hash_compute_failed")?;

    Ok(VerificationDeterminismContractArtifact {
        contract,
        artifact_hash,
    })
}
