use serde_json::{json, Value};

use super::contract::VerificationDeterminismContractArtifact;
use super::incident::{build_verification_determinism_incident, VerificationDeterminismIncident};

pub(crate) struct ReplayComparison {
    pub status_code: u16,
    pub response: Value,
    pub incident: Option<VerificationDeterminismIncident>,
}

pub(crate) fn compare_contracts(
    source_run_id: &str,
    request_fingerprint: &str,
    expected: &VerificationDeterminismContractArtifact,
    recomputed: &VerificationDeterminismContractArtifact,
) -> ReplayComparison {
    let matches_original = expected.artifact_hash == recomputed.artifact_hash;
    let mut response = json!({
        "status": if matches_original { "ok" } else { "DETERMINISM_VIOLATION" },
        "source_run_id": source_run_id,
        "request_fingerprint": request_fingerprint,
        "expected_artifact_hash": expected.artifact_hash,
        "recomputed_artifact_hash": recomputed.artifact_hash,
        "matches_original": matches_original,
        "contract": recomputed,
        "comparison_scope": "verification_determinism_contract",
    });

    if matches_original {
        return ReplayComparison {
            status_code: 200,
            response,
            incident: None,
        };
    }

    let incident = build_verification_determinism_incident(source_run_id, expected, recomputed);
    if let Some(object) = response.as_object_mut() {
        object.insert(
            "incident".to_string(),
            serde_json::to_value(&incident).unwrap_or_else(|_| json!({})),
        );
    }

    ReplayComparison {
        status_code: 409,
        response,
        incident: Some(incident),
    }
}
