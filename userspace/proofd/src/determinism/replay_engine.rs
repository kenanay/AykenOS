use serde_json::{json, Value};

use super::contract::VerificationDeterminismContractArtifact;
use super::fingerprint::canonical_hash_prefixed;
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
    let expected_contract_hash = canonical_hash_prefixed(
        &expected.contract,
        "determinism_contract_hash_recompute_failed",
    );
    let expected_artifact_hash_valid =
        expected_contract_hash.as_ref().ok() == Some(&expected.artifact_hash);
    let matches_original =
        expected_artifact_hash_valid && expected.artifact_hash == recomputed.artifact_hash;
    let mut response = json!({
        "status": if matches_original { "ok" } else { "DETERMINISM_VIOLATION" },
        "source_run_id": source_run_id,
        "request_fingerprint": request_fingerprint,
        "expected_artifact_hash": expected.artifact_hash,
        "expected_artifact_hash_valid": expected_artifact_hash_valid,
        "recomputed_artifact_hash": recomputed.artifact_hash,
        "matches_original": matches_original,
        "contract": recomputed,
        "comparison_scope": "verification_determinism_contract",
    });

    if let Some(object) = response.as_object_mut() {
        if let Ok(hash) = expected_contract_hash {
            object.insert("expected_recomputed_artifact_hash".to_string(), json!(hash));
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::determinism::contract::{
        VerificationDeterminismContractArtifact, VerificationDeterminismContractCore,
    };

    fn sample_artifact() -> VerificationDeterminismContractArtifact {
        let contract = VerificationDeterminismContractCore {
            contract_version: 1,
            request_fingerprint: "sha256:req".to_string(),
            verdict: "accept".to_string(),
            subject_hash: "sha256:subject".to_string(),
            context_hash: "sha256:context".to_string(),
            authority_hash: "sha256:authority".to_string(),
            findings_hash: "sha256:findings".to_string(),
            receipt_payload_hash: None,
        };
        let artifact_hash = canonical_hash_prefixed(&contract, "test_hash_compute_failed")
            .expect("compute artifact hash");
        VerificationDeterminismContractArtifact {
            contract,
            artifact_hash,
        }
    }

    #[test]
    fn compare_contracts_rejects_tampered_expected_artifact_hash() {
        let mut expected = sample_artifact();
        let recomputed = expected.clone();
        expected.artifact_hash = recomputed.artifact_hash.clone();
        expected.contract.verdict = "reject".to_string();

        let comparison = compare_contracts("run-test", "sha256:req", &expected, &recomputed);

        assert_eq!(comparison.status_code, 409);
        assert_eq!(
            comparison
                .response
                .get("expected_artifact_hash_valid")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            comparison
                .response
                .get("matches_original")
                .and_then(Value::as_bool),
            Some(false)
        );
    }
}
