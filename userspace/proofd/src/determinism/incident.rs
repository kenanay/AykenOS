use serde::{Deserialize, Serialize};

use super::contract::VerificationDeterminismContractArtifact;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct VerificationDeterminismIncident {
    #[serde(rename = "type")]
    pub incident_type: &'static str,
    pub run_id: String,
    pub request_fingerprint: String,
    pub expected_hash: String,
    pub observed_hash: String,
    pub surface: String,
    pub severity: &'static str,
    pub status: &'static str,
}

fn determine_determinism_violation_surface(
    expected: &VerificationDeterminismContractArtifact,
    observed: &VerificationDeterminismContractArtifact,
) -> String {
    if expected.contract.authority_hash != observed.contract.authority_hash {
        "authority".to_string()
    } else if expected.contract.context_hash != observed.contract.context_hash {
        "context".to_string()
    } else {
        "verification".to_string()
    }
}

pub(crate) fn build_verification_determinism_incident(
    source_run_id: &str,
    expected: &VerificationDeterminismContractArtifact,
    observed: &VerificationDeterminismContractArtifact,
) -> VerificationDeterminismIncident {
    VerificationDeterminismIncident {
        incident_type: "determinism_incident",
        run_id: source_run_id.to_string(),
        request_fingerprint: observed.contract.request_fingerprint.clone(),
        expected_hash: expected.artifact_hash.clone(),
        observed_hash: observed.artifact_hash.clone(),
        surface: determine_determinism_violation_surface(expected, observed),
        severity: "critical",
        status: "DETERMINISM_VIOLATION",
    }
}
