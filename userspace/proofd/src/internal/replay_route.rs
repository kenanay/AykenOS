use crate::determinism::artifacts::{
    write_canonical_json_file_if_absent_or_same, write_canonical_json_value_if_absent_or_same,
};
use crate::determinism::contract::{
    build_verification_determinism_contract, VerificationDeterminismContractArtifact,
};
use crate::determinism::replay_engine::compare_contracts;
use crate::{
    compute_verify_bundle_request_fingerprint, load_json_from_path, map_receipt_mode,
    map_receipt_signer_config, parse_replay_determinism_request,
    validate_replay_determinism_request, DiagnosticsResponse, ServiceError,
    VERIFICATION_DETERMINISM_CONTRACT_FILE, VERIFICATION_DETERMINISM_INCIDENT_FILE,
    VERIFICATION_DETERMINISM_REPLAY_REPORT_FILE,
};
use proof_verifier::types::{AuditMode, VerifyRequest};
use proof_verifier::{verify_bundle, RegistrySnapshot, TrustPolicy};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(crate) fn handle_internal_replay(raw_body: &[u8], evidence_dir: &Path) -> DiagnosticsResponse {
    match replay_determinism_request(raw_body, evidence_dir) {
        Ok((status_code, value)) => DiagnosticsResponse {
            status_code,
            body: serde_json::to_vec_pretty(&value).unwrap_or_else(|_| b"{}".to_vec()),
            content_type: "application/json; charset=utf-8",
        },
        Err(error) => crate::error_response(error),
    }
}

fn replay_determinism_request(
    raw_body: &[u8],
    evidence_dir: &Path,
) -> Result<(u16, Value), ServiceError> {
    let request = parse_replay_determinism_request(raw_body)?;
    validate_replay_determinism_request(&request)?;

    let request_fingerprint = compute_verify_bundle_request_fingerprint(&request.verify_request)?;
    let policy_path = PathBuf::from(&request.verify_request.policy_path);
    let registry_path = PathBuf::from(&request.verify_request.registry_path);
    let bundle_path = PathBuf::from(&request.verify_request.bundle_path);
    let policy = load_json_from_path::<TrustPolicy>(&policy_path, "invalid_policy_json")?;
    let registry =
        load_json_from_path::<RegistrySnapshot>(&registry_path, "invalid_registry_json")?;
    let receipt_mode = map_receipt_mode(request.verify_request.receipt_mode.as_ref());
    let receipt_signer = request
        .verify_request
        .receipt_signer
        .as_ref()
        .map(map_receipt_signer_config);
    let verify_request = VerifyRequest {
        bundle_path: &bundle_path,
        policy: &policy,
        registry_snapshot: &registry,
        receipt_mode,
        receipt_signer: receipt_signer.as_ref(),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    let outcome = verify_bundle(&verify_request)
        .map_err(|_| ServiceError::Runtime("verifier_runtime_failure"))?;
    let recomputed_contract =
        build_verification_determinism_contract(&registry, &outcome, &request_fingerprint)?;

    if let Some(source_run_id) = request.source_run_id.as_deref() {
        let run_dir = evidence_dir.join(source_run_id);
        let expected_contract = load_json_from_path::<VerificationDeterminismContractArtifact>(
            &run_dir.join(VERIFICATION_DETERMINISM_CONTRACT_FILE),
            "determinism_contract_not_found",
        )?;
        let comparison = compare_contracts(
            source_run_id,
            &request_fingerprint,
            &expected_contract,
            &recomputed_contract,
        );

        write_canonical_json_value_if_absent_or_same(
            &run_dir.join(VERIFICATION_DETERMINISM_REPLAY_REPORT_FILE),
            &comparison.response,
            "determinism_replay_report_write_failed",
            "determinism_replay_report_bytes_conflict",
        )?;

        if let Some(incident) = comparison.incident {
            write_canonical_json_file_if_absent_or_same(
                &run_dir.join(VERIFICATION_DETERMINISM_INCIDENT_FILE),
                &incident,
                "determinism_incident_write_failed",
                "determinism_incident_bytes_conflict",
            )?;
        }

        return Ok((comparison.status_code, comparison.response));
    }

    Ok((
        200,
        json!({
            "status": "ok",
            "request_fingerprint": request_fingerprint,
            "recomputed_artifact_hash": recomputed_contract.artifact_hash,
            "contract": recomputed_contract,
        }),
    ))
}
