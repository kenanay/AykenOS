use proof_verifier::receipt::verify::{
    verify_signed_receipt, verify_signed_receipt_with_authority,
};
use proof_verifier::testing::fixtures::create_fixture_bundle;
use proof_verifier::types::FindingSeverity;
use proof_verifier::{VerdictSubject, VerificationReceipt};
use proofd::{route_request, route_request_with_body};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const OBSERVABILITY_ROOT_ENDPOINTS: &[(&str, &str)] = &[
    ("/diagnostics/incidents", "parity_determinism_incidents.json"),
    ("/diagnostics/parity", "parity_report.json"),
    ("/diagnostics/drift", "parity_drift_attribution_report.json"),
    ("/diagnostics/convergence", "parity_convergence_report.json"),
    ("/diagnostics/failure-matrix", "failure_matrix.json"),
    (
        "/diagnostics/authority-topology",
        "parity_authority_drift_topology.json",
    ),
    (
        "/diagnostics/authority-suppression",
        "parity_authority_suppression_report.json",
    ),
    ("/diagnostics/graph", "parity_incident_graph.json"),
];

const FORBIDDEN_OBSERVABILITY_FIELDS: &[&str] = &[
    "autorecovery",
    "autoquarantine",
    "acceptedauthority",
    "acceptauthority",
    "commit",
    "commitclusterstate",
    "committedcluster",
    "elect",
    "executionoverride",
    "forceaccept",
    "mitigation",
    "nodepriority",
    "override",
    "promote",
    "quarantine",
    "recommendedaction",
    "recommendedactions",
    "resolvetruth",
    "routinghint",
    "retry",
    "selectedtruth",
    "selectwinner",
    "suppressnode",
    "triggerreplayadmission",
    "verificationweight",
    "winningverdict",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HarnessMode {
    ServiceContract,
    ObservabilityBoundary,
}

struct HarnessArgs {
    mode: HarnessMode,
    evidence_root: PathBuf,
    run_id: String,
    out_dir: PathBuf,
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("ERROR: {error}");
            process::exit(3);
        }
    }
}

fn run() -> Result<i32, String> {
    let args = parse_args()?;
    fs::create_dir_all(&args.out_dir)
        .map_err(|error| format!("failed to create {}: {error}", args.out_dir.display()))?;
    Ok(match args.mode {
        HarnessMode::ServiceContract => {
            run_service_contract_gate(&args.evidence_root, &args.run_id, &args.out_dir)
        }
        HarnessMode::ObservabilityBoundary => {
            run_observability_boundary_gate(&args.evidence_root, &args.run_id, &args.out_dir)
        }
    })
}

fn parse_args() -> Result<HarnessArgs, String> {
    let mut args = env::args().skip(1);
    let mode = match args.next().as_deref() {
        Some("service-contract") => HarnessMode::ServiceContract,
        Some("observability-boundary") => HarnessMode::ObservabilityBoundary,
        Some(other) => return Err(format!("unknown mode: {other}")),
        None => {
            return Err(
                "missing mode (expected service-contract or observability-boundary)"
                    .to_string(),
            )
        }
    };

    let mut evidence_root: Option<PathBuf> = None;
    let mut run_id: Option<String> = None;
    let mut out_dir: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--evidence-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --evidence-root".to_string())?;
                evidence_root = Some(PathBuf::from(value));
            }
            "--run-id" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --run-id".to_string())?;
                run_id = Some(value);
            }
            "--out-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --out-dir".to_string())?;
                out_dir = Some(PathBuf::from(value));
            }
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    Ok(HarnessArgs {
        mode,
        evidence_root: evidence_root
            .ok_or_else(|| "missing required --evidence-root".to_string())?,
        run_id: run_id.ok_or_else(|| "missing required --run-id".to_string())?,
        out_dir: out_dir.ok_or_else(|| "missing required --out-dir".to_string())?,
    })
}

fn run_service_contract_gate(evidence_root: &Path, run_id: &str, out_dir: &Path) -> i32 {
    match build_service_contract_artifacts(evidence_root, run_id, out_dir) {
        Ok(code) => code,
        Err(error) => {
            let violations = vec![error];
            write_json(
                out_dir.join("proofd_endpoint_contract.json"),
                &json!({
                    "status": "FAIL",
                    "mode": "phase12_proofd_service_gate_execution_slice",
                    "run_id": run_id,
                    "endpoint_count": 0,
                    "endpoint_checks": [],
                }),
            );
            write_json(
                out_dir.join("proofd_service_report.json"),
                &json!({
                    "status": "FAIL",
                    "gate": "proofd-service",
                    "mode": "phase12_proofd_service_gate_execution_slice",
                    "service_mode": "verification_execution_and_read_only_diagnostics",
                    "run_count": 0,
                    "run_id": run_id,
                    "root_passthrough_ok": false,
                    "run_scoped_passthrough_ok": false,
                    "deterministic_repeated_read_ok": false,
                    "deterministic_repeated_execution_ok": false,
                    "verification_execution_active": false,
                    "explicit_policy_binding_active": false,
                    "explicit_registry_binding_active": false,
                    "receipt_emission_active": false,
                    "endpoint_contract_path": "proofd_endpoint_contract.json",
                    "violations": violations,
                    "violations_count": 1,
                }),
            );
            write_json(
                out_dir.join("proofd_receipt_report.json"),
                &json!({
                    "status": "FAIL",
                    "gate": "proofd-service",
                    "mode": "phase12_proofd_receipt_execution_slice",
                    "receipt_boundary_preserved": false,
                    "receipt_emission_active": false,
                    "receipt_endpoint_exposed": false,
                    "proofd_recomputes_receipts": false,
                    "proofd_reinterprets_receipts": false,
                    "closure_complete": false,
                    "reason": "proofd_service_contract_generation_failed",
                }),
            );
            write_json(
                out_dir.join("proofd_receipt_verification_report.json"),
                &json!({
                    "status": "FAIL",
                    "gate": "proofd-service",
                    "mode": "phase12_proofd_receipt_final_hardening",
                    "signed_receipt_verified": false,
                    "receipt_authority_verified": false,
                    "request_bound_timestamp_preserved": false,
                    "receipt_boundary_preserved": false,
                }),
            );
            write_json(
                out_dir.join("proofd_repeated_execution_report.json"),
                &json!({
                    "status": "FAIL",
                    "gate": "proofd-service",
                    "mode": "phase12_proofd_repeated_execution_final_hardening",
                    "repeated_response_equal": false,
                    "repeated_receipt_bytes_equal": false,
                    "repeated_run_manifest_equal": false,
                    "diagnostics_artifacts_unchanged": false,
                    "run_artifact_merge_detected": false,
                }),
            );
            write_json(
                out_dir.join("report.json"),
                &json!({
                    "gate": "proofd-service",
                    "mode": "phase12_proofd_service_gate_execution_slice",
                    "verdict": "FAIL",
                    "violations": violations,
                    "violations_count": 1,
                }),
            );
            write_violations(out_dir.join("violations.txt"), &violations);
            2
        }
    }
}

fn run_observability_boundary_gate(evidence_root: &Path, run_id: &str, out_dir: &Path) -> i32 {
    match build_observability_boundary_artifacts(evidence_root, run_id, out_dir) {
        Ok(code) => code,
        Err(error) => {
            let violations = vec![error];
            write_json(
                out_dir.join("proofd_observability_boundary_report.json"),
                &json!({
                    "status": "FAIL",
                    "gate": "proofd-observability-boundary",
                    "mode": "phase13_proofd_observability_boundary",
                    "artifact_backed_ok": false,
                    "read_only_namespace_ok": false,
                    "unsupported_query_fail_closed_ok": false,
                    "allowed_incident_filter_ok": false,
                    "payload_non_authoritative_ok": false,
                    "payload_control_plane_free_ok": false,
                    "violations": violations,
                    "violations_count": 1,
                }),
            );
            write_json(
                out_dir.join("proofd_observability_negative_matrix.json"),
                &json!({
                    "status": "FAIL",
                    "gate": "proofd-observability-boundary",
                    "case_count": 0,
                    "cases": [],
                }),
            );
            write_json(
                out_dir.join("report.json"),
                &json!({
                    "gate": "proofd-observability-boundary",
                    "mode": "phase13_proofd_observability_boundary",
                    "verdict": "FAIL",
                    "violations": violations,
                    "violations_count": 1,
                }),
            );
            write_violations(out_dir.join("violations.txt"), &violations);
            2
        }
    }
}

fn build_service_contract_artifacts(
    evidence_root: &Path,
    run_id: &str,
    out_dir: &Path,
) -> Result<i32, String> {
    let run_dir = evidence_root.join(run_id);
    if !run_dir.is_dir() {
        return Err(format!("missing run directory {}", run_dir.display()));
    }

    let root_endpoint_files = [
        ("/diagnostics/parity", "parity_report.json"),
        (
            "/diagnostics/incidents",
            "parity_determinism_incidents.json",
        ),
        ("/diagnostics/drift", "parity_drift_attribution_report.json"),
        ("/diagnostics/convergence", "parity_convergence_report.json"),
        ("/diagnostics/failure-matrix", "failure_matrix.json"),
        (
            "/diagnostics/authority-topology",
            "parity_authority_drift_topology.json",
        ),
        (
            "/diagnostics/authority-suppression",
            "parity_authority_suppression_report.json",
        ),
        ("/diagnostics/graph", "parity_incident_graph.json"),
    ];
    let run_endpoint_files = [
        (
            format!("/diagnostics/runs/{run_id}/parity"),
            "parity_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/incidents"),
            "parity_determinism_incidents.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/drift"),
            "parity_drift_attribution_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/convergence"),
            "parity_convergence_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/failure-matrix"),
            "failure_matrix.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/authority-topology"),
            "parity_authority_drift_topology.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/authority-suppression"),
            "parity_authority_suppression_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/graph"),
            "parity_incident_graph.json",
        ),
    ];

    let mut violations = Vec::new();
    let mut endpoint_checks = Vec::new();
    let mut root_passthrough_ok = true;
    let mut run_scoped_passthrough_ok = true;
    let mut verification_execution_active = false;
    let mut explicit_policy_binding_active = false;
    let mut explicit_registry_binding_active = false;
    let mut receipt_emission_active = false;

    let (health_status, health_body) = route_json("/healthz", evidence_root)?;
    let health_ok = health_status == 200
        && health_body
            .get("status")
            .and_then(Value::as_str)
            .is_some_and(|status| status == "ok");
    if !health_ok {
        violations.push("healthz_contract_mismatch".to_string());
    }
    endpoint_checks.push(json!({
        "endpoint": "/healthz",
        "scope": "root",
        "status": pass_fail(health_ok),
    }));

    for (endpoint, filename) in root_endpoint_files {
        let expected = read_json_file(&evidence_root.join(filename))?;
        let (status_code, payload) = route_json(endpoint, evidence_root)?;
        let passed = status_code == 200 && payload == expected;
        if !passed {
            violations.push(format!("root_passthrough_mismatch:{endpoint}"));
            root_passthrough_ok = false;
        }
        endpoint_checks.push(json!({
            "endpoint": endpoint,
            "artifact": filename,
            "scope": "root",
            "status": pass_fail(passed),
        }));
    }

    for (endpoint, filename) in &run_endpoint_files {
        let expected = read_json_file(&run_dir.join(filename))?;
        let (status_code, payload) = route_json(endpoint, evidence_root)?;
        let passed = status_code == 200 && payload == expected;
        if !passed {
            violations.push(format!("run_passthrough_mismatch:{endpoint}"));
            run_scoped_passthrough_ok = false;
        }
        endpoint_checks.push(json!({
            "endpoint": endpoint,
            "artifact": filename,
            "scope": "run",
            "status": pass_fail(passed),
        }));
    }

    let (runs_status, runs_body) = route_json("/diagnostics/runs", evidence_root)?;
    let runs_ok = runs_status == 200
        && runs_body
            .get("run_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count == 1)
        && runs_body
            .get("runs")
            .and_then(Value::as_array)
            .and_then(|runs| runs.first())
            .and_then(|run| run.get("run_id"))
            .and_then(Value::as_str)
            .is_some_and(|found| found == run_id);
    if !runs_ok {
        violations.push("runs_index_contract_mismatch".to_string());
    }
    endpoint_checks.push(json!({
        "endpoint": "/diagnostics/runs",
        "scope": "root",
        "status": pass_fail(runs_ok),
    }));

    let expected_run_summary = json!({
        "run_id": run_id,
        "artifacts": list_json_artifacts(&run_dir)?,
    });
    let (run_summary_status, run_summary_body) =
        route_json(&format!("/diagnostics/runs/{run_id}"), evidence_root)?;
    let run_summary_ok = run_summary_status == 200 && run_summary_body == expected_run_summary;
    if !run_summary_ok {
        violations.push("run_summary_contract_mismatch".to_string());
    }
    endpoint_checks.push(json!({
        "endpoint": format!("/diagnostics/runs/{run_id}"),
        "scope": "run",
        "status": pass_fail(run_summary_ok),
    }));

    let (_, first_parity) = route_json("/diagnostics/parity", evidence_root)?;
    let (_, second_parity) = route_json("/diagnostics/parity", evidence_root)?;
    let deterministic_repeated_read_ok = first_parity == second_parity;
    if !deterministic_repeated_read_ok {
        violations.push("repeated_read_determinism_failed".to_string());
    }

    let fixture = create_fixture_bundle();
    let policy_path = fixture.root.join("proofd-policy.json");
    let registry_path = fixture.root.join("proofd-registry.json");
    write_json(
        policy_path.clone(),
        &serde_json::to_value(&fixture.policy).unwrap_or_else(|_| json!({})),
    );
    write_json(
        registry_path.clone(),
        &serde_json::to_value(&fixture.registry).unwrap_or_else(|_| json!({})),
    );
    let verify_request = json!({
        "bundle_path": fixture.root,
        "policy_path": policy_path,
        "registry_path": registry_path,
        "receipt_mode": "emit_signed",
        "run_id": run_id,
        "receipt_signer": {
            "verifier_node_id": fixture.receipt_signer.verifier_node_id,
            "verifier_key_id": fixture.receipt_signer.verifier_key_id,
            "signature_algorithm": fixture.receipt_signer.signature_algorithm,
            "private_key": fixture.receipt_signer.private_key,
            "verified_at_utc": fixture.receipt_signer.verified_at_utc,
        },
    });
    write_json(out_dir.join("proofd_verify_request.json"), &verify_request);

    let root_parity_before = fs::read(evidence_root.join("parity_report.json"))
        .map_err(|error| format!("failed to snapshot root parity artifact: {error}"))?;
    let run_parity_before = fs::read(run_dir.join("parity_report.json"))
        .map_err(|error| format!("failed to snapshot run parity artifact: {error}"))?;
    let (verify_status, verify_response) =
        route_json_with_body("POST", "/verify/bundle", &verify_request, evidence_root)?;
    write_json(
        out_dir.join("proofd_verify_response.json"),
        &verify_response,
    );

    let verify_ok = verify_status == 200
        && verify_response
            .get("status")
            .and_then(Value::as_str)
            .is_some_and(|status| status == "ok")
        && verify_response
            .get("run_id")
            .and_then(Value::as_str)
            .is_some_and(|value| value == run_id)
        && verify_response
            .get("receipt_emitted")
            .and_then(Value::as_bool)
            .is_some_and(|value| value)
        && verify_response
            .get("receipt_path")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "receipts/verification_receipt.json");
    if !verify_ok {
        violations.push("verify_endpoint_contract_mismatch".to_string());
    } else {
        verification_execution_active = true;
        explicit_policy_binding_active = true;
        explicit_registry_binding_active = true;
        receipt_emission_active = true;
    }
    endpoint_checks.push(json!({
        "endpoint": "/verify/bundle",
        "scope": "execution",
        "method": "POST",
        "status": pass_fail(verify_ok),
    }));

    let run_manifest_path = run_dir.join("proofd_run_manifest.json");
    let receipt_path = run_dir.join("receipts/verification_receipt.json");
    let first_run_manifest_bytes = fs::read(&run_manifest_path)
        .map_err(|error| format!("failed to read first run manifest: {error}"))?;
    let first_receipt_bytes = fs::read(&receipt_path)
        .map_err(|error| format!("failed to read first receipt artifact: {error}"))?;
    let run_artifacts_after_first = list_json_artifacts(&run_dir)?;

    let (verify_repeat_status, verify_repeat_response) =
        route_json_with_body("POST", "/verify/bundle", &verify_request, evidence_root)?;
    let deterministic_repeated_execution_ok =
        verify_repeat_status == verify_status && verify_repeat_response == verify_response;
    if !deterministic_repeated_execution_ok {
        violations.push("repeated_execution_determinism_failed".to_string());
    }

    let second_run_manifest_bytes = fs::read(&run_manifest_path)
        .map_err(|error| format!("failed to read second run manifest: {error}"))?;
    let second_receipt_bytes = fs::read(&receipt_path)
        .map_err(|error| format!("failed to read second receipt artifact: {error}"))?;
    let repeated_receipt_bytes_equal = first_receipt_bytes == second_receipt_bytes;
    if !repeated_receipt_bytes_equal {
        violations.push("repeated_execution_receipt_bytes_drift".to_string());
    }
    let repeated_run_manifest_equal = first_run_manifest_bytes == second_run_manifest_bytes;
    if !repeated_run_manifest_equal {
        violations.push("repeated_execution_run_manifest_drift".to_string());
    }
    let run_artifacts_after_second = list_json_artifacts(&run_dir)?;
    let run_artifact_merge_detected = run_artifacts_after_first != run_artifacts_after_second;
    if run_artifact_merge_detected {
        violations.push("run_artifact_merge_detected".to_string());
    }
    let root_parity_after = fs::read(evidence_root.join("parity_report.json"))
        .map_err(|error| format!("failed to resnapshot root parity artifact: {error}"))?;
    let run_parity_after = fs::read(run_dir.join("parity_report.json"))
        .map_err(|error| format!("failed to resnapshot run parity artifact: {error}"))?;
    let diagnostics_artifacts_unchanged =
        root_parity_before == root_parity_after && run_parity_before == run_parity_after;
    if !diagnostics_artifacts_unchanged {
        violations.push("diagnostics_passthrough_drift".to_string());
    }

    let run_manifest = read_json_file(&run_manifest_path)?;
    write_json(out_dir.join("proofd_run_manifest.json"), &run_manifest);
    let run_manifest_ok = run_manifest
        .get("receipt_mode")
        .and_then(Value::as_str)
        .is_some_and(|value| value == "emit_signed")
        && run_manifest
            .get("receipt_emitted")
            .and_then(Value::as_bool)
            .is_some_and(|value| value);
    if !run_manifest_ok {
        violations.push("run_manifest_receipt_mode_mismatch".to_string());
    }

    let receipt_json = read_json_file(&receipt_path)?;
    let receipt = serde_json::from_value::<VerificationReceipt>(receipt_json.clone())
        .map_err(|error| format!("failed to decode receipt artifact: {error}"))?;
    let verdict_subject = serde_json::from_value::<VerdictSubject>(
        verify_response
            .get("verdict_subject")
            .cloned()
            .ok_or_else(|| "verify response missing verdict_subject".to_string())?,
    )
    .map_err(|error| format!("failed to decode verdict_subject from response: {error}"))?;
    let signed_receipt_findings =
        verify_signed_receipt(&receipt, &verdict_subject, &fixture.receipt_verifier_key)
            .map_err(|error| format!("signed receipt verification failed: {error}"))?;
    let signed_receipt_verified = !has_error_findings(&signed_receipt_findings);
    if !signed_receipt_verified {
        violations.push("signed_receipt_verification_failed".to_string());
    }
    let distributed_receipt = verify_signed_receipt_with_authority(
        &receipt,
        &verdict_subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .map_err(|error| format!("authority-aware signed receipt verification failed: {error}"))?;
    let receipt_authority_verified = !has_error_findings(&distributed_receipt.findings);
    if !receipt_authority_verified {
        violations.push("receipt_authority_verification_failed".to_string());
    }
    let receipt_boundary_preserved = [
        "bundle_id",
        "trust_overlay_hash",
        "policy_hash",
        "registry_snapshot_hash",
    ]
    .iter()
    .all(|field| {
        receipt_json.get(field).and_then(Value::as_str)
            == verify_response
                .get("verdict_subject")
                .and_then(|value| value.get(*field))
                .and_then(Value::as_str)
    });
    if !receipt_boundary_preserved {
        violations.push("receipt_boundary_preserved_failed".to_string());
    }
    let request_bound_timestamp_preserved =
        receipt_json.get("verified_at_utc").and_then(Value::as_str)
            == verify_request
                .get("receipt_signer")
                .and_then(|value| value.get("verified_at_utc"))
                .and_then(Value::as_str);
    if !request_bound_timestamp_preserved {
        violations.push("request_bound_timestamp_not_preserved".to_string());
    }

    let closure_complete = verify_ok
        && deterministic_repeated_read_ok
        && deterministic_repeated_execution_ok
        && repeated_receipt_bytes_equal
        && repeated_run_manifest_equal
        && diagnostics_artifacts_unchanged
        && !run_artifact_merge_detected
        && signed_receipt_verified
        && receipt_authority_verified
        && request_bound_timestamp_preserved
        && receipt_boundary_preserved
        && run_manifest_ok
        && violations.is_empty();

    let endpoint_contract = json!({
        "status": pass_fail(closure_complete),
        "mode": "phase12_proofd_service_gate_execution_slice",
        "run_id": run_id,
        "endpoint_count": endpoint_checks.len(),
        "endpoint_checks": endpoint_checks,
        "verify_request_path": "proofd_verify_request.json",
        "verify_response_path": "proofd_verify_response.json",
    });
    let service_report = json!({
        "status": pass_fail(closure_complete),
        "gate": "proofd-service",
        "mode": "phase12_proofd_service_gate_execution_slice",
        "service_mode": "verification_execution_and_read_only_diagnostics",
        "receipt_mode": "emit_signed",
        "run_count": 1,
        "run_id": run_id,
        "root_passthrough_ok": root_passthrough_ok,
        "run_scoped_passthrough_ok": run_scoped_passthrough_ok,
        "deterministic_repeated_read_ok": deterministic_repeated_read_ok,
        "deterministic_repeated_execution_ok": deterministic_repeated_execution_ok,
        "verification_execution_active": verification_execution_active,
        "explicit_policy_binding_active": explicit_policy_binding_active,
        "explicit_registry_binding_active": explicit_registry_binding_active,
        "receipt_emission_active": receipt_emission_active,
        "signed_receipt_execution_active": receipt_emission_active,
        "signed_receipt_verified": signed_receipt_verified,
        "receipt_authority_binding_verified": receipt_authority_verified,
        "request_bound_timestamp_preserved": request_bound_timestamp_preserved,
        "repeated_receipt_bytes_equal": repeated_receipt_bytes_equal,
        "repeated_run_manifest_equal": repeated_run_manifest_equal,
        "diagnostics_artifacts_unchanged": diagnostics_artifacts_unchanged,
        "run_artifact_merge_detected": run_artifact_merge_detected,
        "closure_complete": closure_complete,
        "endpoint_contract_path": "proofd_endpoint_contract.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    let receipt_report = json!({
        "status": pass_fail(closure_complete),
        "gate": "proofd-service",
        "mode": "phase12_proofd_receipt_execution_slice",
        "receipt_mode": "emit_signed",
        "receipt_boundary_preserved": receipt_boundary_preserved,
        "receipt_emission_active": receipt_emission_active,
        "signed_receipt_verified": signed_receipt_verified,
        "signed_receipt_findings_count": signed_receipt_findings.len(),
        "receipt_authority_verified": receipt_authority_verified,
        "receipt_authority_findings_count": distributed_receipt.findings.len(),
        "receipt_authority_chain_id": distributed_receipt.authority_resolution.authority_chain_id,
        "request_bound_timestamp_preserved": request_bound_timestamp_preserved,
        "receipt_endpoint_exposed": false,
        "proofd_recomputes_receipts": false,
        "proofd_reinterprets_receipts": false,
        "closure_complete": closure_complete,
        "receipt_path": "receipts/verification_receipt.json",
        "reason": if closure_complete {
            "closure_ready_final_hardening_green"
        } else {
            "final_hardening_assertions_failed"
        },
    });
    let receipt_verification_report = json!({
        "status": pass_fail(
            signed_receipt_verified
                && receipt_authority_verified
                && request_bound_timestamp_preserved
                && receipt_boundary_preserved,
        ),
        "gate": "proofd-service",
        "mode": "phase12_proofd_receipt_final_hardening",
        "signed_receipt_verified": signed_receipt_verified,
        "signed_receipt_findings_count": signed_receipt_findings.len(),
        "receipt_authority_verified": receipt_authority_verified,
        "receipt_authority_findings_count": distributed_receipt.findings.len(),
        "receipt_authority_chain_id": distributed_receipt.authority_resolution.authority_chain_id,
        "request_bound_timestamp_preserved": request_bound_timestamp_preserved,
        "receipt_boundary_preserved": receipt_boundary_preserved,
        "receipt_path": "receipts/verification_receipt.json",
    });
    let repeated_execution_report = json!({
        "status": pass_fail(
            deterministic_repeated_execution_ok
                && repeated_receipt_bytes_equal
                && repeated_run_manifest_equal
                && diagnostics_artifacts_unchanged
                && !run_artifact_merge_detected,
        ),
        "gate": "proofd-service",
        "mode": "phase12_proofd_repeated_execution_final_hardening",
        "repeated_response_equal": deterministic_repeated_execution_ok,
        "repeated_receipt_bytes_equal": repeated_receipt_bytes_equal,
        "repeated_run_manifest_equal": repeated_run_manifest_equal,
        "diagnostics_artifacts_unchanged": diagnostics_artifacts_unchanged,
        "run_artifact_merge_detected": run_artifact_merge_detected,
        "run_artifact_count_after_first": run_artifacts_after_first.len(),
        "run_artifact_count_after_second": run_artifacts_after_second.len(),
    });
    let report = json!({
        "gate": "proofd-service",
        "mode": "phase12_proofd_service_gate_execution_slice",
        "verdict": if closure_complete { "PASS" } else { "FAIL" },
        "violations": violations,
        "violations_count": violations.len(),
    });

    write_json(
        out_dir.join("proofd_endpoint_contract.json"),
        &endpoint_contract,
    );
    write_json(out_dir.join("proofd_service_report.json"), &service_report);
    write_json(out_dir.join("proofd_receipt_report.json"), &receipt_report);
    write_json(
        out_dir.join("proofd_receipt_verification_report.json"),
        &receipt_verification_report,
    );
    write_json(
        out_dir.join("proofd_repeated_execution_report.json"),
        &repeated_execution_report,
    );
    write_json(out_dir.join("report.json"), &report);
    write_violations(out_dir.join("violations.txt"), &violations);

    Ok(if closure_complete { 0 } else { 2 })
}

fn build_observability_boundary_artifacts(
    evidence_root: &Path,
    run_id: &str,
    out_dir: &Path,
) -> Result<i32, String> {
    let run_dir = evidence_root.join(run_id);
    if !run_dir.is_dir() {
        return Err(format!("missing run directory {}", run_dir.display()));
    }

    let run_endpoint_files = vec![
        (
            format!("/diagnostics/runs/{run_id}/incidents"),
            "parity_determinism_incidents.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/parity"),
            "parity_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/drift"),
            "parity_drift_attribution_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/convergence"),
            "parity_convergence_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/failure-matrix"),
            "failure_matrix.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/authority-topology"),
            "parity_authority_drift_topology.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/authority-suppression"),
            "parity_authority_suppression_report.json",
        ),
        (
            format!("/diagnostics/runs/{run_id}/graph"),
            "parity_incident_graph.json",
        ),
    ];

    let mut violations = Vec::new();
    let mut endpoint_checks = Vec::new();
    let mut payload_hits = Vec::new();
    let mut payload_scan_targets = 0usize;
    let mut artifact_backed_ok = true;

    for (endpoint, filename) in OBSERVABILITY_ROOT_ENDPOINTS {
        let expected = read_json_file(&evidence_root.join(filename))?;
        let (status_code, payload) = route_json(endpoint, evidence_root)?;
        let passed = status_code == 200 && payload == expected;
        if !passed {
            violations.push(format!("artifact_passthrough_mismatch:{endpoint}"));
            artifact_backed_ok = false;
        } else {
            payload_scan_targets += 1;
            payload_hits.extend(scan_forbidden_observability_fields(endpoint, &payload));
        }
        endpoint_checks.push(json!({
            "endpoint": endpoint,
            "artifact": filename,
            "scope": "root",
            "status": pass_fail(passed),
        }));
    }

    for (endpoint, filename) in &run_endpoint_files {
        let expected = read_json_file(&run_dir.join(filename))?;
        let (status_code, payload) = route_json(endpoint, evidence_root)?;
        let passed = status_code == 200 && payload == expected;
        if !passed {
            violations.push(format!("artifact_passthrough_mismatch:{endpoint}"));
            artifact_backed_ok = false;
        } else {
            payload_scan_targets += 1;
            payload_hits.extend(scan_forbidden_observability_fields(endpoint, &payload));
        }
        endpoint_checks.push(json!({
            "endpoint": endpoint,
            "artifact": filename,
            "scope": "run",
            "status": pass_fail(passed),
        }));
    }

    let incident_report = read_json_file(&evidence_root.join("parity_determinism_incidents.json"))?;
    let filter_value = incident_report
        .get("incidents")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("severity"))
        .and_then(Value::as_str)
        .unwrap_or("pure_determinism_failure");
    let filter_target = format!("/diagnostics/incidents?severity={filter_value}");
    let (filtered_status, filtered_body) = route_json(&filter_target, evidence_root)?;
    let allowed_incident_filter_ok = filtered_status == 200
        && filtered_body
            .get("filtered")
            .and_then(Value::as_bool)
            .is_some_and(|value| value)
        && filtered_body
            .get("incidents")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().all(|item| {
                    item.get("severity").and_then(Value::as_str) == Some(filter_value)
                })
            });
    if !allowed_incident_filter_ok {
        violations.push("allowed_incident_filter_contract_mismatch".to_string());
    } else {
        payload_scan_targets += 1;
        payload_hits.extend(scan_forbidden_observability_fields(
            &filter_target,
            &filtered_body,
        ));
    }
    endpoint_checks.push(json!({
        "endpoint": filter_target,
        "scope": "root",
        "status": pass_fail(allowed_incident_filter_ok),
        "rule": "allow-listed incident filters remain read-only and artifact-backed",
    }));

    let mut negative_cases = Vec::new();
    let mut read_only_namespace_ok = true;
    let mut unsupported_query_fail_closed_ok = true;

    let (post_graph_status, post_graph_body) =
        route_json_with_body("POST", "/diagnostics/graph", &json!({}), evidence_root)?;
    let post_graph_ok = post_graph_status == 405
        && post_graph_body
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "method_not_allowed");
    if !post_graph_ok {
        violations.push("observability_namespace_mutation_allowed:/diagnostics/graph".to_string());
        read_only_namespace_ok = false;
    }
    negative_cases.push(json!({
        "case_id": "P13-NEG-01",
        "target": "/diagnostics/graph",
        "method": "POST",
        "expected_status_code": 405,
        "observed_status_code": post_graph_status,
        "status": pass_fail(post_graph_ok),
        "rule": "observability namespace must not mutate or trigger execution",
    }));

    let (post_topology_status, post_topology_body) = route_json_with_body(
        "POST",
        "/diagnostics/authority-topology",
        &json!({}),
        evidence_root,
    )?;
    let post_topology_ok = post_topology_status == 405
        && post_topology_body
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "method_not_allowed");
    if !post_topology_ok {
        violations.push(
            "authority_observability_mutation_allowed:/diagnostics/authority-topology".to_string(),
        );
        read_only_namespace_ok = false;
    }
    negative_cases.push(json!({
        "case_id": "P13-NEG-02",
        "target": "/diagnostics/authority-topology",
        "method": "POST",
        "expected_status_code": 405,
        "observed_status_code": post_topology_status,
        "status": pass_fail(post_topology_ok),
        "rule": "authority observability must not become authority control",
    }));

    let (graph_query_status, graph_query_body) =
        route_json("/diagnostics/graph?select_winner=true", evidence_root)?;
    let graph_query_ok = graph_query_status == 400
        && graph_query_body
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "unsupported_query_parameter");
    if !graph_query_ok {
        violations.push("truth_selection_query_not_fail_closed:/diagnostics/graph".to_string());
        unsupported_query_fail_closed_ok = false;
    }
    negative_cases.push(json!({
        "case_id": "P13-NEG-03",
        "target": "/diagnostics/graph?select_winner=true",
        "method": "GET",
        "expected_status_code": 400,
        "observed_status_code": graph_query_status,
        "status": pass_fail(graph_query_ok),
        "rule": "query parameters must not smuggle truth election semantics",
    }));

    let (convergence_query_status, convergence_query_body) =
        route_json("/diagnostics/convergence?commit=true", evidence_root)?;
    let convergence_query_ok = convergence_query_status == 400
        && convergence_query_body
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "unsupported_query_parameter");
    if !convergence_query_ok {
        violations.push("commit_query_not_fail_closed:/diagnostics/convergence".to_string());
        unsupported_query_fail_closed_ok = false;
    }
    negative_cases.push(json!({
        "case_id": "P13-NEG-04",
        "target": "/diagnostics/convergence?commit=true",
        "method": "GET",
        "expected_status_code": 400,
        "observed_status_code": convergence_query_status,
        "status": pass_fail(convergence_query_ok),
        "rule": "convergence query must not imply cluster-state commit",
    }));

    let payload_non_authoritative_hits = payload_hits
        .iter()
        .filter(|hit| {
            hit.get("case_id")
                .and_then(Value::as_str)
                .is_some_and(|value| value == "P13-NEG-13")
        })
        .cloned()
        .collect::<Vec<_>>();
    let payload_control_plane_hits = payload_hits
        .iter()
        .filter(|hit| {
            hit.get("case_id")
                .and_then(Value::as_str)
                .is_some_and(|value| value == "P13-NEG-14")
        })
        .cloned()
        .collect::<Vec<_>>();
    let payload_non_authoritative_ok = payload_non_authoritative_hits.is_empty();
    let payload_control_plane_free_ok = payload_control_plane_hits.is_empty();
    if !payload_non_authoritative_ok {
        violations.push("forbidden_truth_or_authority_field_exposed".to_string());
    }
    if !payload_control_plane_free_ok {
        violations.push("forbidden_control_plane_field_exposed".to_string());
    }

    negative_cases.push(json!({
        "case_id": "P13-NEG-13",
        "status": pass_fail(payload_non_authoritative_ok),
        "payload_scan_target_count": payload_scan_targets,
        "forbidden_field_hits": payload_non_authoritative_hits,
        "rule": "payloads must not encode hidden consensus or arbitration outputs",
    }));
    negative_cases.push(json!({
        "case_id": "P13-NEG-14",
        "status": pass_fail(payload_control_plane_free_ok),
        "payload_scan_target_count": payload_scan_targets,
        "forbidden_field_hits": payload_control_plane_hits,
        "rule": "observability payloads must not embed control-plane affordances",
    }));

    let boundary_preserved = artifact_backed_ok
        && read_only_namespace_ok
        && unsupported_query_fail_closed_ok
        && allowed_incident_filter_ok
        && payload_non_authoritative_ok
        && payload_control_plane_free_ok
        && violations.is_empty();

    write_json(
        out_dir.join("proofd_observability_boundary_report.json"),
        &json!({
            "status": pass_fail(boundary_preserved),
            "gate": "proofd-observability-boundary",
            "mode": "phase13_proofd_observability_boundary",
            "artifact_backed_ok": artifact_backed_ok,
            "read_only_namespace_ok": read_only_namespace_ok,
            "unsupported_query_fail_closed_ok": unsupported_query_fail_closed_ok,
            "allowed_incident_filter_ok": allowed_incident_filter_ok,
            "payload_non_authoritative_ok": payload_non_authoritative_ok,
            "payload_control_plane_free_ok": payload_control_plane_free_ok,
            "forbidden_fields": FORBIDDEN_OBSERVABILITY_FIELDS,
            "endpoint_count": endpoint_checks.len(),
            "endpoint_checks": endpoint_checks,
            "payload_scan_target_count": payload_scan_targets,
            "payload_field_hits": payload_hits,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    );
    write_json(
        out_dir.join("proofd_observability_negative_matrix.json"),
        &json!({
            "status": pass_fail(boundary_preserved),
            "gate": "proofd-observability-boundary",
            "case_count": negative_cases.len(),
            "cases": negative_cases,
        }),
    );
    write_json(
        out_dir.join("report.json"),
        &json!({
            "gate": "proofd-observability-boundary",
            "mode": "phase13_proofd_observability_boundary",
            "verdict": if boundary_preserved { "PASS" } else { "FAIL" },
            "violations": violations,
            "violations_count": violations.len(),
        }),
    );
    write_violations(out_dir.join("violations.txt"), &violations);

    Ok(if boundary_preserved { 0 } else { 2 })
}

fn route_json(target: &str, evidence_root: &Path) -> Result<(u16, Value), String> {
    let response = route_request("GET", target, evidence_root);
    let body = serde_json::from_slice::<Value>(&response.body)
        .map_err(|error| format!("invalid json body for {target}: {error}"))?;
    Ok((response.status_code, body))
}

fn scan_forbidden_observability_fields(endpoint: &str, value: &Value) -> Vec<Value> {
    let mut hits = Vec::new();
    scan_forbidden_observability_fields_inner(endpoint, "$", value, &mut hits);
    hits
}

fn scan_forbidden_observability_fields_inner(
    endpoint: &str,
    path: &str,
    value: &Value,
    hits: &mut Vec<Value>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let normalized = normalize_field_key(key);
                if let Some(case_id) = observability_case_for_field(&normalized) {
                    hits.push(json!({
                        "case_id": case_id,
                        "endpoint": endpoint,
                        "field": key,
                        "normalized_field": normalized,
                        "json_path": format!("{path}.{key}"),
                    }));
                }
                scan_forbidden_observability_fields_inner(
                    endpoint,
                    &format!("{path}.{key}"),
                    child,
                    hits,
                );
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                scan_forbidden_observability_fields_inner(
                    endpoint,
                    &format!("{path}[{index}]"),
                    item,
                    hits,
                );
            }
        }
        _ => {}
    }
}

fn normalize_field_key(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn observability_case_for_field(field: &str) -> Option<&'static str> {
    match field {
        "selectedtruth"
        | "winningverdict"
        | "committedcluster"
        | "acceptedauthority"
        | "acceptauthority"
        | "resolvetruth"
        | "selectwinner"
        | "elect" => Some("P13-NEG-13"),
        "retry"
        | "override"
        | "promote"
        | "commit"
        | "forceaccept"
        | "recommendedaction"
        | "recommendedactions"
        | "mitigation"
        | "routinghint"
        | "nodepriority"
        | "verificationweight"
        | "executionoverride"
        | "quarantine"
        | "autoquarantine"
        | "autorecovery"
        | "suppressnode"
        | "triggerreplayadmission"
        | "commitclusterstate" => Some("P13-NEG-14"),
        _ => None,
    }
}

fn route_json_with_body(
    method: &str,
    target: &str,
    body: &Value,
    evidence_root: &Path,
) -> Result<(u16, Value), String> {
    let request_bytes = serde_json::to_vec(body)
        .map_err(|error| format!("failed to serialize request body for {target}: {error}"))?;
    let response = route_request_with_body(
        method,
        target,
        Some(request_bytes.as_slice()),
        evidence_root,
    );
    let response_body = serde_json::from_slice::<Value>(&response.body)
        .map_err(|error| format!("invalid json body for {target}: {error}"))?;
    Ok((response.status_code, response_body))
}

fn list_json_artifacts(run_dir: &Path) -> Result<Vec<String>, String> {
    let mut artifacts = fs::read_dir(run_dir)
        .map_err(|error| format!("failed to read {}: {error}", run_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect::<Vec<_>>();
    artifacts.sort();
    Ok(artifacts)
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn write_json(path: PathBuf, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).unwrap_or_else(|_| b"{}".to_vec()),
    )
    .expect("write json artifact");
}

fn write_violations(path: PathBuf, violations: &[String]) {
    let body = if violations.is_empty() {
        String::new()
    } else {
        violations
            .iter()
            .map(|violation| format!("{violation}\n"))
            .collect::<String>()
    };
    fs::write(path, body).expect("write violations");
}

fn pass_fail(condition: bool) -> &'static str {
    if condition {
        "PASS"
    } else {
        "FAIL"
    }
}

fn has_error_findings<T>(findings: &[T]) -> bool
where
    T: FindingSeverityView,
{
    findings
        .iter()
        .any(|finding| finding.finding_severity() == FindingSeverity::Error)
}

trait FindingSeverityView {
    fn finding_severity(&self) -> FindingSeverity;
}

impl FindingSeverityView for proof_verifier::types::VerificationFinding {
    fn finding_severity(&self) -> FindingSeverity {
        self.severity.clone()
    }
}
