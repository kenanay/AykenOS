//! Proof chain records for canonical query submission.

use crate::canonical_query::CanonicalPlan;
use crate::canonical_query_lowering::LoweredCanonicalQuery;
use crate::gate_c::types::SubmissionId;
use crate::submission_validation::{submission_result_fingerprint, SubmissionCapability, SubmissionValidationReport};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofReplayBinding {
    pub canonical_plan_fingerprint: String,
    pub canonical_binding_fingerprint: String,
    pub bcib_sha256: String,
    pub submission_result_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofChainRecord {
    pub canonical_command: String,
    pub canonical_command_sha256: String,
    pub canonical_plan_fingerprint: String,
    pub canonical_binding_fingerprint: String,
    pub bcib_sha256: String,
    pub target_context_id: u64,
    pub submission_id: SubmissionId,
    pub required_capabilities: Vec<SubmissionCapability>,
    pub declared_capabilities: Vec<SubmissionCapability>,
    pub replay_binding: ProofReplayBinding,
    pub proof_chain_sha256: String,
}

pub fn build_proof_chain_record(
    canonical_command: &str,
    plan: &CanonicalPlan,
    lowered: &LoweredCanonicalQuery,
    validation: &SubmissionValidationReport,
    submission_id: SubmissionId,
) -> ProofChainRecord {
    let canonical_command_sha256 = sha256_hex(canonical_command.as_bytes());
    let replay_binding = ProofReplayBinding {
        canonical_plan_fingerprint: validation.canonical_plan_fingerprint.clone(),
        canonical_binding_fingerprint: validation.canonical_binding_fingerprint.clone(),
        bcib_sha256: validation.bcib_sha256.clone(),
        submission_result_fingerprint: submission_result_fingerprint(&submission_id),
    };

    let proof_chain_sha256 = {
        let payload = serde_json::to_vec(&(
            canonical_command,
            &validation.canonical_plan_fingerprint,
            &validation.canonical_binding_fingerprint,
            &validation.bcib_sha256,
            validation.target_context_id,
            &submission_id,
            &validation.required_capabilities,
            &validation.declared_capabilities,
            &replay_binding,
        ))
        .expect("proof chain payload must serialize deterministically");
        sha256_hex(&payload)
    };

    ProofChainRecord {
        canonical_command: canonical_command.to_string(),
        canonical_command_sha256,
        canonical_plan_fingerprint: plan.fingerprint_hex(),
        canonical_binding_fingerprint: plan.binding.fingerprint_hex(),
        bcib_sha256: lowered.bcib_sha256.clone(),
        target_context_id: validation.target_context_id,
        submission_id,
        required_capabilities: validation.required_capabilities.clone(),
        declared_capabilities: validation.declared_capabilities.clone(),
        replay_binding,
        proof_chain_sha256,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_query::parse_canonical_plan;
    use crate::canonical_query_lowering::lower_canonical_query_to_bcib;
    use crate::gate_c::types::SubmissionId;
    use crate::submission_validation::{
        SubmissionCapability, SubmissionValidator, SubmissionValidationInput,
    };

    fn build_record() -> ProofChainRecord {
        let plan = parse_canonical_plan("show data.users 42").unwrap();
        let lowered = lower_canonical_query_to_bcib(&plan).unwrap();
        let validator = SubmissionValidator::new();
        let validation = validator
            .validate(&SubmissionValidationInput {
                canonical_command: "show data.users 42".to_string(),
                plan: plan.clone(),
                lowered: lowered.clone(),
                target_context_id: 9,
                declared_capabilities: vec![SubmissionCapability::context_read(
                    "data.users",
                    "explicit show access",
                )],
                submission_surface_available: true,
            })
            .unwrap();

        build_proof_chain_record(
            "show data.users 42",
            &plan,
            &lowered,
            &validation,
            SubmissionId {
                id: "submit_123".to_string(),
                timestamp: 1642694400,
                fingerprint: Some("submission-fingerprint".to_string()),
            },
        )
    }

    #[test]
    fn proof_chain_record_binds_required_artifacts() {
        let record = build_record();

        assert_eq!(record.canonical_command, "show data.users 42");
        assert_eq!(record.target_context_id, 9);
        assert_eq!(record.submission_id.id, "submit_123");
        assert_eq!(record.required_capabilities.len(), 1);
        assert_eq!(record.replay_binding.bcib_sha256, record.bcib_sha256);
        assert_eq!(record.proof_chain_sha256.len(), 64);
    }

    #[test]
    fn proof_chain_record_is_deterministic() {
        let left = build_record();
        let right = build_record();

        assert_eq!(left, right);
    }
}
