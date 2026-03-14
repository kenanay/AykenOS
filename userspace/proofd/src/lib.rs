use proof_verifier::diversity_ledger_producer::{
    parse_event_time_to_unix_ns, run_diversity_ledger_producer,
    VerificationDiversityLedgerProducerConfig, VerificationDiversityLedgerProducerManifest,
    VerificationNodeBinding,
};
use proof_verifier::trust_reuse_runtime_surface::{
    load_trust_reuse_runtime_surface as load_native_trust_reuse_runtime_surface, TrustReuseOutcome,
    TrustReuseRuntimeEvent, TrustReuseRuntimeSurfaceReport,
};
use proof_verifier::types::{AuditMode, ReceiptMode, ReceiptSignerConfig, VerifyRequest};
use proof_verifier::{verify_bundle, RegistrySnapshot, TrustPolicy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const RUN_LEVEL_ARTIFACTS: &[&str] = &[
    "report.json",
    "parity_report.json",
    "proofd_run_manifest.json",
    "verification_audit_ledger.jsonl",
    "verification_diversity_ledger_binding.json",
    "verification_diversity_ledger.json",
    "verification_diversity_ledger_append_report.json",
    "replay_boundary_flow_source.json",
    "trust_reuse_flow_source.json",
    "parity_authority_suppression_report.json",
    "parity_authority_drift_topology.json",
    "parity_incident_graph.json",
    "parity_consistency_report.json",
    "parity_determinism_report.json",
    "parity_determinism_incidents.json",
    "parity_drift_attribution_report.json",
    "parity_convergence_report.json",
    "parity_closure_audit_report.json",
    "failure_matrix.json",
];

const ALLOWED_INCIDENT_FILTERS: &[&str] = &["severity", "surface_key", "node_id"];
const VERIFICATION_AUDIT_LEDGER_FILE: &str = "verification_audit_ledger.jsonl";
const VERIFICATION_DIVERSITY_BINDING_FILE: &str = "verification_diversity_ledger_binding.json";
const VERIFICATION_DIVERSITY_LEDGER_FILE: &str = "verification_diversity_ledger.json";
const VERIFICATION_DIVERSITY_APPEND_REPORT_FILE: &str =
    "verification_diversity_ledger_append_report.json";
const REPLAY_BOUNDARY_FLOW_SOURCE_FILE: &str = "replay_boundary_flow_source.json";
const REPLAY_REPORT_FILE: &str = "replay_report.json";
const TRUST_REUSE_FLOW_SOURCE_FILE: &str = "trust_reuse_flow_source.json";
const TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH: &str = "reports/trust_reuse_runtime_surface.json";
const PROOFD_RUN_MANIFEST_FILE: &str = "proofd_run_manifest.json";
const MAX_VERIFY_BUNDLE_BODY_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestTarget {
    pub path: String,
    pub query: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub content_type: &'static str,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum VerifyBundleReceiptMode {
    None,
    EmitUnsigned,
    EmitSigned,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VerifyBundleReceiptSigner {
    verifier_node_id: String,
    verifier_key_id: String,
    signature_algorithm: String,
    private_key: String,
    verified_at_utc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VerifyBundleDiversityBinding {
    verifier_id: String,
    authority_chain_id: String,
    lineage_id: String,
    #[serde(default)]
    execution_cluster_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VerifyBundleReplayBoundaryBinding {
    replay_contract_id: String,
    #[serde(default)]
    source_run_id: Option<String>,
    #[serde(default)]
    reuse_group_id: Option<String>,
    #[serde(default)]
    surface_local_path_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VerifyBundleTrustReuseBinding {
    trust_reuse_source: String,
    #[serde(default)]
    source_run_id: Option<String>,
    #[serde(default)]
    reuse_group_id: Option<String>,
    #[serde(default)]
    surface_local_path_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VerifyBundleRequestBody {
    bundle_path: String,
    policy_path: String,
    registry_path: String,
    #[serde(default)]
    receipt_mode: Option<VerifyBundleReceiptMode>,
    run_id: String,
    #[serde(default)]
    receipt_signer: Option<VerifyBundleReceiptSigner>,
    #[serde(default)]
    diversity_binding: Option<VerifyBundleDiversityBinding>,
    #[serde(default)]
    replay_boundary_binding: Option<VerifyBundleReplayBoundaryBinding>,
    #[serde(default)]
    trust_reuse_binding: Option<VerifyBundleTrustReuseBinding>,
}

#[derive(Debug, Clone, Serialize)]
struct VerifyBundleResponseBody {
    status: &'static str,
    run_id: String,
    verdict: &'static str,
    verdict_subject: Value,
    receipt_emitted: bool,
    receipt_path: Option<String>,
    behavioral_observability_emitted: bool,
    audit_ledger_path: Option<String>,
    verification_diversity_ledger_binding_path: Option<String>,
    verification_diversity_ledger_path: Option<String>,
    replay_boundary_flow_source_path: Option<String>,
    replay_boundary_flow_source_origin: Option<String>,
    trust_reuse_flow_source_path: Option<String>,
    trust_reuse_flow_source_origin: Option<String>,
    findings_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct AuthoritySinkholeCompanionSourceDocument {
    source_version: u32,
    flow_surface: String,
    status: String,
    run_id: String,
    window_model: String,
    events: Vec<AuthoritySinkholeCompanionSourceEvent>,
}

#[derive(Debug, Clone, Serialize)]
struct AuthoritySinkholeCompanionSourceEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    timestamp_unix_ns: u64,
    subject_bundle_id: String,
    verification_context_id: String,
    authority_chain_id: String,
    terminal: bool,
    reused: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verifier_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lineage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_cluster_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replay_contract_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trust_reuse_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reuse_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    surface_local_path_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProofBundleReplayReport {
    status: String,
    replay_execution_trace_hash: String,
    replay_result_hash: String,
    final_state_hash: String,
    replay_event_count: u64,
    violations_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ProofBundleMetaRun {
    run_id: String,
}

#[derive(Debug, Clone)]
struct ReplayRuntimeSurface {
    source_run_id: String,
    replay_report: ProofBundleReplayReport,
}

pub fn parse_target(raw: &str) -> RequestTarget {
    match raw.split_once('?') {
        Some((path, query)) => RequestTarget {
            path: path.to_string(),
            query: Some(query.to_string()),
        },
        None => RequestTarget {
            path: raw.to_string(),
            query: None,
        },
    }
}

fn replay_report_relative_path() -> PathBuf {
    Path::new("reports").join(REPLAY_REPORT_FILE)
}

pub fn route_request(method: &str, raw_target: &str, evidence_dir: &Path) -> DiagnosticsResponse {
    route_request_with_body(method, raw_target, None, evidence_dir)
}

pub fn route_request_with_body(
    method: &str,
    raw_target: &str,
    raw_body: Option<&[u8]>,
    evidence_dir: &Path,
) -> DiagnosticsResponse {
    let target = parse_target(raw_target);
    match method {
        "GET" => {
            if let Err(error) = validate_get_query(&target) {
                return error_response(error);
            }

            match target.path.as_str() {
                "/healthz" => json_response(
                    200,
                    json!({
                        "status": "ok",
                        "service": "proofd",
                        "mode": "verification_execution_and_read_only_diagnostics",
                    }),
                ),
                "/diagnostics/incidents" => {
                    match load_incident_report(evidence_dir, target.query.as_deref()) {
                        Ok(value) => json_response(200, value),
                        Err(error) => error_response(error),
                    }
                }
                "/diagnostics/parity" => serve_json_file(evidence_dir.join("parity_report.json")),
                "/diagnostics/authority-suppression" => {
                    serve_json_file(evidence_dir.join("parity_authority_suppression_report.json"))
                }
                "/diagnostics/authority-topology" => {
                    serve_json_file(evidence_dir.join("parity_authority_drift_topology.json"))
                }
                "/diagnostics/graph" => {
                    serve_json_file(evidence_dir.join("parity_incident_graph.json"))
                }
                "/diagnostics/drift" => {
                    serve_json_file(evidence_dir.join("parity_drift_attribution_report.json"))
                }
                "/diagnostics/convergence" => {
                    serve_json_file(evidence_dir.join("parity_convergence_report.json"))
                }
                "/diagnostics/failure-matrix" => {
                    serve_json_file(evidence_dir.join("failure_matrix.json"))
                }
                "/diagnostics/runs" => match list_runs(evidence_dir) {
                    Ok(value) => json_response(200, value),
                    Err(error) => error_response(error),
                },
                _ if target.path.starts_with("/diagnostics/incidents/") => {
                    let incident_id = target
                        .path
                        .trim_start_matches("/diagnostics/incidents/")
                        .to_string();
                    match load_single_incident(evidence_dir, &incident_id) {
                        Ok(value) => json_response(200, value),
                        Err(error) => error_response(error),
                    }
                }
                _ if target.path.starts_with("/diagnostics/runs/") => {
                    handle_run_endpoint(&target.path, evidence_dir)
                }
                _ => json_response(404, json!({ "error": "not_found" })),
            }
        }
        "POST" => match target.path.as_str() {
            "/verify/bundle" => handle_verify_bundle(raw_body.unwrap_or_default(), evidence_dir),
            _ if is_observability_path(&target.path) => {
                json_response(405, json!({ "error": "method_not_allowed" }))
            }
            _ => json_response(404, json!({ "error": "not_found" })),
        },
        "PUT" | "PATCH" | "DELETE" if is_observability_path(&target.path) => {
            json_response(405, json!({ "error": "method_not_allowed" }))
        }
        _ => json_response(405, json!({ "error": "method_not_allowed" })),
    }
}

fn validate_get_query(target: &RequestTarget) -> Result<(), ServiceError> {
    if target.query.is_none() {
        return Ok(());
    }

    if target.path == "/diagnostics/incidents" {
        let _ = parse_query(target.query.as_deref(), ALLOWED_INCIDENT_FILTERS)?;
        return Ok(());
    }

    Err(ServiceError::BadRequest("unsupported_query_parameter"))
}

fn list_runs(evidence_dir: &Path) -> Result<Value, ServiceError> {
    let entries =
        fs::read_dir(evidence_dir).map_err(|_| ServiceError::NotFound("evidence_dir_not_found"))?;
    let mut runs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|_| ServiceError::MalformedArtifact("dir_read_error"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let run_id = entry.file_name().to_string_lossy().to_string();
        if !is_safe_path_segment(&run_id) {
            continue;
        }

        let summary = build_run_summary(&run_id, &path)?;
        let has_artifacts = summary
            .get("artifacts")
            .and_then(Value::as_array)
            .map(|artifacts| !artifacts.is_empty())
            .unwrap_or(false);
        if !has_artifacts {
            continue;
        }

        runs.push(summary);
    }

    runs.sort_by(|left, right| {
        left.get("run_id")
            .and_then(Value::as_str)
            .cmp(&right.get("run_id").and_then(Value::as_str))
    });

    Ok(json!({
        "run_count": runs.len(),
        "runs": runs,
    }))
}

fn handle_run_endpoint(path: &str, evidence_dir: &Path) -> DiagnosticsResponse {
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 3 {
        return json_response(404, json!({ "error": "invalid_run_path" }));
    }

    let run_id = parts[2];
    if !is_safe_path_segment(run_id) {
        return json_response(404, json!({ "error": "invalid_run_id" }));
    }

    let run_dir = evidence_dir.join(run_id);
    if parts.len() == 3 {
        return match build_run_summary(run_id, &run_dir) {
            Ok(summary) => json_response(200, summary),
            Err(error) => error_response(error),
        };
    }

    let response = match parts[3] {
        "incidents" if parts.len() == 4 => {
            serve_json_file(run_dir.join("parity_determinism_incidents.json"))
        }
        "parity" if parts.len() == 4 => serve_json_file(run_dir.join("parity_report.json")),
        "authority-suppression" if parts.len() == 4 => {
            serve_json_file(run_dir.join("parity_authority_suppression_report.json"))
        }
        "authority-topology" if parts.len() == 4 => {
            serve_json_file(run_dir.join("parity_authority_drift_topology.json"))
        }
        "graph" if parts.len() == 4 => serve_json_file(run_dir.join("parity_incident_graph.json")),
        "drift" if parts.len() == 4 => {
            serve_json_file(run_dir.join("parity_drift_attribution_report.json"))
        }
        "convergence" if parts.len() == 4 => {
            serve_json_file(run_dir.join("parity_convergence_report.json"))
        }
        "failure-matrix" if parts.len() == 4 => {
            serve_json_file(run_dir.join("failure_matrix.json"))
        }
        _ => json_response(404, json!({ "error": "not_found" })),
    };
    response
}

fn build_run_summary(run_id: &str, run_dir: &Path) -> Result<Value, ServiceError> {
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    let artifacts = list_run_artifacts(run_dir)?;
    Ok(json!({
        "run_id": run_id,
        "artifacts": artifacts,
    }))
}

fn handle_verify_bundle(raw_body: &[u8], evidence_dir: &Path) -> DiagnosticsResponse {
    match verify_bundle_request(raw_body, evidence_dir) {
        Ok(value) => json_response(200, value),
        Err(error) => error_response(error),
    }
}

fn verify_bundle_request(raw_body: &[u8], evidence_dir: &Path) -> Result<Value, ServiceError> {
    let request = parse_verify_bundle_request(raw_body)?;
    validate_verify_bundle_request(&request)?;

    let bundle_path = PathBuf::from(&request.bundle_path);
    let policy_path = PathBuf::from(&request.policy_path);
    let registry_path = PathBuf::from(&request.registry_path);
    let request_fingerprint = compute_verify_bundle_request_fingerprint(&request)?;
    let policy = load_json_from_path::<TrustPolicy>(&policy_path, "invalid_policy_json")?;
    let registry =
        load_json_from_path::<RegistrySnapshot>(&registry_path, "invalid_registry_json")?;
    let receipt_mode = map_receipt_mode(request.receipt_mode.as_ref());
    let receipt_signer = request
        .receipt_signer
        .as_ref()
        .map(map_receipt_signer_config);
    let run_dir = evidence_dir.join(&request.run_id);
    fs::create_dir_all(&run_dir).map_err(|_| ServiceError::Runtime("run_dir_create_failed"))?;
    let rerun_same_request = verify_existing_run_fingerprint(&run_dir, &request_fingerprint)?;
    let audit_ledger_path = run_dir.join(VERIFICATION_AUDIT_LEDGER_FILE);
    let diversity_manifest = request
        .diversity_binding
        .as_ref()
        .map(|binding| build_diversity_binding_manifest(&request, binding))
        .transpose()?;
    let declared_audit_mode = if diversity_manifest.is_some() {
        AuditMode::Append
    } else {
        AuditMode::None
    };
    let effective_audit_mode = if diversity_manifest.is_some() && !rerun_same_request {
        AuditMode::Append
    } else {
        AuditMode::None
    };
    let audit_ledger_relative_path = diversity_manifest
        .as_ref()
        .map(|_| VERIFICATION_AUDIT_LEDGER_FILE.to_string());
    let diversity_binding_relative_path = diversity_manifest
        .as_ref()
        .map(|_| VERIFICATION_DIVERSITY_BINDING_FILE.to_string());
    let diversity_ledger_relative_path = diversity_manifest
        .as_ref()
        .map(|_| VERIFICATION_DIVERSITY_LEDGER_FILE.to_string());
    let diversity_append_report_relative_path = diversity_manifest
        .as_ref()
        .map(|_| VERIFICATION_DIVERSITY_APPEND_REPORT_FILE.to_string());
    let replay_boundary_source_relative_path = diversity_manifest
        .as_ref()
        .map(|_| REPLAY_BOUNDARY_FLOW_SOURCE_FILE.to_string());
    let native_trust_reuse_surface_present =
        trust_reuse_runtime_surface_path(&bundle_path).is_file();
    let mut trust_reuse_source_relative_path: Option<String> = None;
    let mut replay_boundary_source_origin: Option<String> = None;
    let mut trust_reuse_source_origin: Option<String> = None;

    if receipt_mode == ReceiptMode::EmitSigned && receipt_signer.is_none() {
        return Err(ServiceError::BadRequest("receipt_signer_missing"));
    }

    let verify_request = VerifyRequest {
        bundle_path: &bundle_path,
        policy: &policy,
        registry_snapshot: &registry,
        receipt_mode: receipt_mode.clone(),
        receipt_signer: receipt_signer.as_ref(),
        audit_mode: effective_audit_mode,
        audit_ledger_path: diversity_manifest
            .as_ref()
            .map(|_| audit_ledger_path.as_path()),
    };
    let outcome = verify_bundle(&verify_request)
        .map_err(|_| ServiceError::Runtime("verifier_runtime_failure"))?;

    let receipt_relative_path = if let Some(receipt) = &outcome.receipt {
        let receipts_dir = run_dir.join("receipts");
        fs::create_dir_all(&receipts_dir)
            .map_err(|_| ServiceError::Runtime("receipt_dir_create_failed"))?;
        let receipt_path = receipts_dir.join("verification_receipt.json");
        write_json_file_if_absent_or_same(
            &receipt_path,
            receipt,
            "receipt_write_failed",
            "receipt_bytes_conflict",
        )?;
        Some("receipts/verification_receipt.json".to_string())
    } else {
        None
    };

    let behavioral_observability_emitted = if let Some(manifest) = &diversity_manifest {
        let binding_path = run_dir.join(VERIFICATION_DIVERSITY_BINDING_FILE);
        write_json_file_if_absent_or_same(
            &binding_path,
            manifest,
            "diversity_binding_write_failed",
            "diversity_binding_bytes_conflict",
        )?;

        if rerun_same_request {
            for required_path in [
                run_dir.join(VERIFICATION_AUDIT_LEDGER_FILE),
                run_dir.join(VERIFICATION_DIVERSITY_BINDING_FILE),
                run_dir.join(VERIFICATION_DIVERSITY_LEDGER_FILE),
                run_dir.join(VERIFICATION_DIVERSITY_APPEND_REPORT_FILE),
                run_dir.join(REPLAY_BOUNDARY_FLOW_SOURCE_FILE),
                run_dir.join(TRUST_REUSE_FLOW_SOURCE_FILE),
            ] {
                if required_path.ends_with(TRUST_REUSE_FLOW_SOURCE_FILE)
                    && !native_trust_reuse_surface_present
                    && request.trust_reuse_binding.is_none()
                {
                    continue;
                }
                if !required_path.is_file() {
                    return Err(ServiceError::Runtime(
                        "existing_behavioral_artifact_missing",
                    ));
                }
            }
        } else {
            let producer_outcome =
                run_diversity_ledger_producer(&VerificationDiversityLedgerProducerConfig {
                    audit_ledger_path: audit_ledger_path.clone(),
                    binding_path,
                    ledger_path: run_dir.join(VERIFICATION_DIVERSITY_LEDGER_FILE),
                    output_dir: run_dir.clone(),
                })
                .map_err(|_| ServiceError::Runtime("diversity_ledger_producer_runtime_failure"))?;
            if producer_outcome.verdict.as_str() != "PASS" {
                return Err(ServiceError::Runtime("diversity_ledger_producer_failed"));
            }
        }
        let node_binding = manifest.node_bindings.first().ok_or(ServiceError::Runtime(
            "diversity_binding_manifest_missing_node",
        ))?;
        let (replay_boundary_origin, replay_boundary_document) =
            match build_runtime_replay_boundary_flow_source_document(
                &bundle_path,
                &request,
                request.replay_boundary_binding.as_ref(),
                node_binding,
                &outcome,
            ) {
                Ok(document) => (Some("runtime_bundle_replay".to_string()), Some(document)),
                Err(ServiceError::BadRequest("replay_boundary_runtime_surface_invalid"))
                    if request.replay_boundary_binding.is_some() =>
                {
                    let document = build_request_bound_replay_boundary_flow_source_document(
                        &request,
                        request
                            .replay_boundary_binding
                            .as_ref()
                            .ok_or(ServiceError::Runtime("replay_boundary_binding_missing"))?,
                        node_binding,
                        &outcome,
                    )?;
                    (Some("request_binding".to_string()), Some(document))
                }
                Err(error) => return Err(error),
            };
        replay_boundary_source_origin = replay_boundary_origin;
        if let Some(document) = replay_boundary_document {
            write_json_file_if_absent_or_same(
                &run_dir.join(REPLAY_BOUNDARY_FLOW_SOURCE_FILE),
                &document,
                "replay_boundary_flow_source_write_failed",
                "replay_boundary_flow_source_bytes_conflict",
            )?;
        }
        if let Some(document) = build_runtime_trust_reuse_flow_source_document(
            &bundle_path,
            &request,
            &outcome,
            request.trust_reuse_binding.as_ref(),
        )? {
            write_json_file_if_absent_or_same(
                &run_dir.join(TRUST_REUSE_FLOW_SOURCE_FILE),
                &document,
                "trust_reuse_flow_source_write_failed",
                "trust_reuse_flow_source_bytes_conflict",
            )?;
            trust_reuse_source_relative_path = Some(TRUST_REUSE_FLOW_SOURCE_FILE.to_string());
            trust_reuse_source_origin = Some("runtime_bundle_trust_reuse".to_string());
        } else if let Some(binding) = request.trust_reuse_binding.as_ref() {
            let document = build_request_bound_trust_reuse_flow_source_document(
                &request,
                binding,
                node_binding,
                &outcome,
            )?;
            write_json_file_if_absent_or_same(
                &run_dir.join(TRUST_REUSE_FLOW_SOURCE_FILE),
                &document,
                "trust_reuse_flow_source_write_failed",
                "trust_reuse_flow_source_bytes_conflict",
            )?;
            trust_reuse_source_relative_path = Some(TRUST_REUSE_FLOW_SOURCE_FILE.to_string());
            trust_reuse_source_origin = Some("request_binding".to_string());
        }
        true
    } else {
        false
    };

    let run_manifest = json!({
        "run_id": request.run_id,
        "service_mode": "verification_execution_and_read_only_diagnostics",
        "bundle_path": request.bundle_path,
        "policy_path": request.policy_path,
        "registry_path": request.registry_path,
        "receipt_mode": receipt_mode_label(&receipt_mode),
        "receipt_emitted": receipt_relative_path.is_some(),
        "receipt_path": receipt_relative_path,
        "behavioral_observability_emitted": behavioral_observability_emitted,
        "audit_mode": audit_mode_label(declared_audit_mode),
        "audit_ledger_path": audit_ledger_relative_path,
        "verification_diversity_ledger_binding_path": diversity_binding_relative_path,
        "verification_diversity_ledger_path": diversity_ledger_relative_path,
        "verification_diversity_ledger_append_report_path": diversity_append_report_relative_path,
        "replay_boundary_flow_source_path": replay_boundary_source_relative_path,
        "replay_boundary_flow_source_origin": replay_boundary_source_origin,
        "trust_reuse_flow_source_path": trust_reuse_source_relative_path,
        "trust_reuse_flow_source_origin": trust_reuse_source_origin,
        "request_fingerprint": request_fingerprint,
        "verdict": verdict_label(&outcome.verdict),
        "verdict_subject": outcome.subject,
        "findings_count": outcome.findings.len(),
    });
    write_json_value_if_absent_or_same(
        &run_dir.join(PROOFD_RUN_MANIFEST_FILE),
        &run_manifest,
        "run_manifest_write_failed",
        "run_manifest_bytes_conflict",
    )?;

    let response = VerifyBundleResponseBody {
        status: "ok",
        run_id: request.run_id,
        verdict: verdict_label(&outcome.verdict),
        verdict_subject: serde_json::to_value(&outcome.subject).unwrap_or_else(|_| json!({})),
        receipt_emitted: outcome.receipt.is_some(),
        receipt_path: run_manifest
            .get("receipt_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        behavioral_observability_emitted,
        audit_ledger_path: run_manifest
            .get("audit_ledger_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        verification_diversity_ledger_binding_path: run_manifest
            .get("verification_diversity_ledger_binding_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        verification_diversity_ledger_path: run_manifest
            .get("verification_diversity_ledger_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        replay_boundary_flow_source_path: run_manifest
            .get("replay_boundary_flow_source_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        replay_boundary_flow_source_origin: run_manifest
            .get("replay_boundary_flow_source_origin")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        trust_reuse_flow_source_path: run_manifest
            .get("trust_reuse_flow_source_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        trust_reuse_flow_source_origin: run_manifest
            .get("trust_reuse_flow_source_origin")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        findings_count: outcome.findings.len(),
    };

    serde_json::to_value(response).map_err(|_| ServiceError::Runtime("response_serialize_failed"))
}

fn load_single_incident(evidence_dir: &Path, incident_id: &str) -> Result<Value, ServiceError> {
    let report = read_json_file(&evidence_dir.join("parity_determinism_incidents.json"))?;
    let incidents = report
        .get("incidents")
        .and_then(Value::as_array)
        .ok_or(ServiceError::MalformedArtifact("missing incidents array"))?;
    let incident = incidents
        .iter()
        .find(|item| item.get("incident_id").and_then(Value::as_str) == Some(incident_id))
        .cloned()
        .ok_or(ServiceError::NotFound("incident_not_found"))?;
    Ok(incident)
}

fn load_incident_report(
    evidence_dir: &Path,
    raw_query: Option<&str>,
) -> Result<Value, ServiceError> {
    let mut report = read_json_file(&evidence_dir.join("parity_determinism_incidents.json"))?;
    let filters = parse_query(raw_query, ALLOWED_INCIDENT_FILTERS)?;
    if filters.is_empty() {
        return Ok(report);
    }

    let incidents = report
        .get("incidents")
        .and_then(Value::as_array)
        .ok_or(ServiceError::MalformedArtifact("missing incidents array"))?;

    let filtered = incidents
        .iter()
        .filter(|incident| incident_matches_filters(incident, &filters))
        .cloned()
        .collect::<Vec<_>>();

    let severity_counts = filtered.iter().fold(Map::new(), |mut acc, incident| {
        if let Some(severity) = incident.get("severity").and_then(Value::as_str) {
            let current = acc.get(severity).and_then(Value::as_u64).unwrap_or(0);
            acc.insert(severity.to_string(), json!(current + 1));
        }
        acc
    });

    if let Some(object) = report.as_object_mut() {
        object.insert(
            "determinism_incident_count".to_string(),
            json!(filtered.len()),
        );
        object.insert(
            "severity_counts".to_string(),
            Value::Object(severity_counts),
        );
        object.insert("incidents".to_string(), Value::Array(filtered));
        object.insert("filtered".to_string(), json!(true));
        object.insert("filters".to_string(), json!(filters));
    }

    Ok(report)
}

fn incident_matches_filters(incident: &Value, filters: &[(String, String)]) -> bool {
    filters.iter().all(|(key, value)| match key.as_str() {
        "severity" => incident.get("severity").and_then(Value::as_str) == Some(value.as_str()),
        "surface_key" => {
            incident.get("surface_key").and_then(Value::as_str) == Some(value.as_str())
        }
        "node_id" => incident
            .get("nodes")
            .and_then(Value::as_array)
            .map(|nodes| {
                nodes
                    .iter()
                    .any(|item| item.as_str() == Some(value.as_str()))
            })
            .unwrap_or(false),
        _ => true,
    })
}

fn parse_query(
    raw_query: Option<&str>,
    allowed_keys: &[&str],
) -> Result<Vec<(String, String)>, ServiceError> {
    let mut filters = Vec::new();
    for part in raw_query
        .unwrap_or("")
        .split('&')
        .filter(|part| !part.is_empty())
    {
        let (key, value) = part
            .split_once('=')
            .ok_or(ServiceError::BadRequest("invalid_query_parameter"))?;
        if value.is_empty() {
            return Err(ServiceError::BadRequest("invalid_query_parameter"));
        }
        if !allowed_keys.iter().any(|allowed| *allowed == key) {
            return Err(ServiceError::BadRequest("unsupported_query_parameter"));
        }
        filters.push((key.to_string(), value.to_string()));
    }
    Ok(filters)
}

fn parse_verify_bundle_request(raw_body: &[u8]) -> Result<VerifyBundleRequestBody, ServiceError> {
    if raw_body.is_empty() {
        return Err(ServiceError::BadRequest("missing_request_body"));
    }
    if raw_body.len() > MAX_VERIFY_BUNDLE_BODY_BYTES {
        return Err(ServiceError::BadRequest("request_body_too_large"));
    }

    serde_json::from_slice(raw_body).map_err(|_| ServiceError::BadRequest("invalid_request_body"))
}

fn validate_verify_bundle_request(request: &VerifyBundleRequestBody) -> Result<(), ServiceError> {
    if request.run_id.is_empty() || !is_safe_path_segment(&request.run_id) {
        return Err(ServiceError::BadRequest("invalid_run_id"));
    }

    if request.diversity_binding.is_some()
        && !matches!(
            request.receipt_mode,
            Some(VerifyBundleReceiptMode::EmitSigned)
        )
    {
        return Err(ServiceError::BadRequest(
            "diversity_binding_requires_emit_signed",
        ));
    }

    if request.replay_boundary_binding.is_some() && request.diversity_binding.is_none() {
        return Err(ServiceError::BadRequest(
            "replay_boundary_binding_requires_diversity_binding",
        ));
    }

    if request.trust_reuse_binding.is_some() && request.diversity_binding.is_none() {
        return Err(ServiceError::BadRequest(
            "trust_reuse_binding_requires_diversity_binding",
        ));
    }

    if matches!(
        request.receipt_mode,
        Some(VerifyBundleReceiptMode::EmitSigned)
    ) && request.receipt_signer.is_none()
    {
        return Err(ServiceError::BadRequest("receipt_signer_missing"));
    }

    for (label, value) in [
        ("bundle_path", request.bundle_path.as_str()),
        ("policy_path", request.policy_path.as_str()),
        ("registry_path", request.registry_path.as_str()),
    ] {
        if value.is_empty() {
            return Err(ServiceError::BadRequest(match label {
                "bundle_path" => "bundle_path_missing",
                "policy_path" => "policy_path_missing",
                "registry_path" => "registry_path_missing",
                _ => "request_path_missing",
            }));
        }
        if !Path::new(value).is_absolute() {
            return Err(ServiceError::BadRequest(match label {
                "bundle_path" => "bundle_path_not_absolute",
                "policy_path" => "policy_path_not_absolute",
                "registry_path" => "registry_path_not_absolute",
                _ => "request_path_not_absolute",
            }));
        }
    }

    if let Some(binding) = request.diversity_binding.as_ref() {
        for (label, value) in [
            ("verifier_id", binding.verifier_id.as_str()),
            ("authority_chain_id", binding.authority_chain_id.as_str()),
            ("lineage_id", binding.lineage_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(ServiceError::BadRequest(match label {
                    "verifier_id" => "diversity_binding_verifier_id_missing",
                    "authority_chain_id" => "diversity_binding_authority_chain_id_missing",
                    "lineage_id" => "diversity_binding_lineage_id_missing",
                    _ => "diversity_binding_field_missing",
                }));
            }
        }
        if binding
            .execution_cluster_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ServiceError::BadRequest(
                "diversity_binding_execution_cluster_id_invalid",
            ));
        }
    }

    if let Some(binding) = request.replay_boundary_binding.as_ref() {
        if binding.replay_contract_id.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "replay_boundary_binding_replay_contract_id_missing",
            ));
        }
        for (label, value) in [
            ("source_run_id", binding.source_run_id.as_deref()),
            ("reuse_group_id", binding.reuse_group_id.as_deref()),
            (
                "surface_local_path_id",
                binding.surface_local_path_id.as_deref(),
            ),
        ] {
            if value.is_some_and(|value| value.trim().is_empty()) {
                return Err(ServiceError::BadRequest(match label {
                    "source_run_id" => "replay_boundary_binding_source_run_id_invalid",
                    "reuse_group_id" => "replay_boundary_binding_reuse_group_id_invalid",
                    "surface_local_path_id" => {
                        "replay_boundary_binding_surface_local_path_id_invalid"
                    }
                    _ => "replay_boundary_binding_field_invalid",
                }));
            }
        }
    }

    if let Some(binding) = request.trust_reuse_binding.as_ref() {
        if binding.trust_reuse_source.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "trust_reuse_binding_trust_reuse_source_missing",
            ));
        }
        for (label, value) in [
            ("source_run_id", binding.source_run_id.as_deref()),
            ("reuse_group_id", binding.reuse_group_id.as_deref()),
            (
                "surface_local_path_id",
                binding.surface_local_path_id.as_deref(),
            ),
        ] {
            if value.is_some_and(|value| value.trim().is_empty()) {
                return Err(ServiceError::BadRequest(match label {
                    "source_run_id" => "trust_reuse_binding_source_run_id_invalid",
                    "reuse_group_id" => "trust_reuse_binding_reuse_group_id_invalid",
                    "surface_local_path_id" => "trust_reuse_binding_surface_local_path_id_invalid",
                    _ => "trust_reuse_binding_field_invalid",
                }));
            }
        }
    }

    Ok(())
}

fn load_json_from_path<T>(path: &Path, error_code: &'static str) -> Result<T, ServiceError>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = fs::read(path).map_err(|_| ServiceError::BadRequest(error_code))?;
    serde_json::from_slice(&bytes).map_err(|_| ServiceError::BadRequest(error_code))
}

fn compute_verify_bundle_request_fingerprint(
    request: &VerifyBundleRequestBody,
) -> Result<String, ServiceError> {
    let bytes = serde_json::to_vec(request)
        .map_err(|_| ServiceError::Runtime("request_fingerprint_serialize_failed"))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("sha256:{}", encode_lower_hex(&hasher.finalize())))
}

fn verify_existing_run_fingerprint(
    run_dir: &Path,
    request_fingerprint: &str,
) -> Result<bool, ServiceError> {
    let manifest_path = run_dir.join(PROOFD_RUN_MANIFEST_FILE);
    if !manifest_path.exists() {
        return Ok(false);
    }
    let manifest = read_json_file(&manifest_path)
        .map_err(|_| ServiceError::Runtime("existing_run_manifest_invalid"))?;
    let existing_fingerprint = manifest
        .get("request_fingerprint")
        .and_then(Value::as_str)
        .ok_or(ServiceError::Runtime(
            "existing_run_manifest_missing_request_fingerprint",
        ))?;
    if existing_fingerprint != request_fingerprint {
        return Err(ServiceError::BadRequest(
            "run_id_request_fingerprint_mismatch",
        ));
    }
    Ok(true)
}

fn map_receipt_mode(mode: Option<&VerifyBundleReceiptMode>) -> ReceiptMode {
    match mode.unwrap_or(&VerifyBundleReceiptMode::None) {
        VerifyBundleReceiptMode::None => ReceiptMode::None,
        VerifyBundleReceiptMode::EmitUnsigned => ReceiptMode::EmitUnsigned,
        VerifyBundleReceiptMode::EmitSigned => ReceiptMode::EmitSigned,
    }
}

fn map_receipt_signer_config(signer: &VerifyBundleReceiptSigner) -> ReceiptSignerConfig {
    ReceiptSignerConfig {
        verifier_node_id: signer.verifier_node_id.clone(),
        verifier_key_id: signer.verifier_key_id.clone(),
        signature_algorithm: signer.signature_algorithm.clone(),
        private_key: signer.private_key.clone(),
        verified_at_utc: signer.verified_at_utc.clone(),
    }
}

fn build_diversity_binding_manifest(
    request: &VerifyBundleRequestBody,
    binding: &VerifyBundleDiversityBinding,
) -> Result<VerificationDiversityLedgerProducerManifest, ServiceError> {
    let signer = request
        .receipt_signer
        .as_ref()
        .ok_or(ServiceError::BadRequest("receipt_signer_missing"))?;
    Ok(VerificationDiversityLedgerProducerManifest {
        binding_version: 1,
        run_id: request.run_id.clone(),
        verification_context_id_source: "policy_hash".to_string(),
        node_bindings: vec![VerificationNodeBinding {
            verification_node_id: signer.verifier_node_id.clone(),
            verifier_key_id: Some(signer.verifier_key_id.clone()),
            verifier_id: binding.verifier_id.clone(),
            authority_chain_id: binding.authority_chain_id.clone(),
            lineage_id: binding.lineage_id.clone(),
            execution_cluster_id: binding.execution_cluster_id.clone(),
        }],
    })
}

fn build_request_bound_replay_boundary_flow_source_document(
    request: &VerifyBundleRequestBody,
    binding: &VerifyBundleReplayBoundaryBinding,
    node_binding: &VerificationNodeBinding,
    outcome: &proof_verifier::types::VerificationOutcome,
) -> Result<AuthoritySinkholeCompanionSourceDocument, ServiceError> {
    Ok(AuthoritySinkholeCompanionSourceDocument {
        source_version: 1,
        flow_surface: "replay_boundary".to_string(),
        status: "PASS".to_string(),
        run_id: request.run_id.clone(),
        window_model: default_companion_window_model(),
        events: vec![build_companion_source_event(
            outcome,
            node_binding,
            binding.source_run_id.clone(),
            Some(binding.replay_contract_id.clone()),
            None,
            binding.reuse_group_id.clone(),
            binding.surface_local_path_id.clone(),
        )?],
    })
}

fn build_runtime_replay_boundary_flow_source_document(
    bundle_path: &Path,
    request: &VerifyBundleRequestBody,
    binding: Option<&VerifyBundleReplayBoundaryBinding>,
    node_binding: &VerificationNodeBinding,
    outcome: &proof_verifier::types::VerificationOutcome,
) -> Result<AuthoritySinkholeCompanionSourceDocument, ServiceError> {
    let runtime_surface = load_replay_runtime_surface(bundle_path)?;
    if let Some(binding) = binding {
        if let Some(source_run_id) = binding.source_run_id.as_deref() {
            if source_run_id != runtime_surface.source_run_id {
                return Err(ServiceError::BadRequest(
                    "replay_boundary_binding_source_run_id_mismatch",
                ));
            }
        }
    }

    let replay_report = &runtime_surface.replay_report;
    let _ = (
        replay_report.status.as_str(),
        replay_report.replay_execution_trace_hash.as_str(),
        replay_report.replay_result_hash.as_str(),
        replay_report.final_state_hash.as_str(),
        replay_report.violations_count,
    );

    Ok(AuthoritySinkholeCompanionSourceDocument {
        source_version: 1,
        flow_surface: "replay_boundary".to_string(),
        status: "PASS".to_string(),
        run_id: request.run_id.clone(),
        window_model: default_companion_window_model(),
        events: vec![build_companion_source_event(
            outcome,
            node_binding,
            Some(runtime_surface.source_run_id),
            binding.map(|value| value.replay_contract_id.clone()),
            None,
            binding.and_then(|value| value.reuse_group_id.clone()),
            Some(
                binding
                    .and_then(|value| value.surface_local_path_id.clone())
                    .unwrap_or_else(|| replay_report_relative_path().display().to_string()),
            ),
        )?],
    })
}

fn build_request_bound_trust_reuse_flow_source_document(
    request: &VerifyBundleRequestBody,
    binding: &VerifyBundleTrustReuseBinding,
    node_binding: &VerificationNodeBinding,
    outcome: &proof_verifier::types::VerificationOutcome,
) -> Result<AuthoritySinkholeCompanionSourceDocument, ServiceError> {
    Ok(AuthoritySinkholeCompanionSourceDocument {
        source_version: 1,
        flow_surface: "trust_reuse".to_string(),
        status: "PASS".to_string(),
        run_id: request.run_id.clone(),
        window_model: default_companion_window_model(),
        events: vec![build_companion_source_event(
            outcome,
            node_binding,
            binding.source_run_id.clone(),
            None,
            Some(binding.trust_reuse_source.clone()),
            binding.reuse_group_id.clone(),
            binding.surface_local_path_id.clone(),
        )?],
    })
}

fn build_runtime_trust_reuse_flow_source_document(
    bundle_path: &Path,
    request: &VerifyBundleRequestBody,
    outcome: &proof_verifier::types::VerificationOutcome,
    binding: Option<&VerifyBundleTrustReuseBinding>,
) -> Result<Option<AuthoritySinkholeCompanionSourceDocument>, ServiceError> {
    let runtime_surface_path = trust_reuse_runtime_surface_path(bundle_path);
    if !runtime_surface_path.is_file() {
        return Ok(None);
    }

    let runtime_surface = load_native_trust_reuse_runtime_surface(&runtime_surface_path)
        .map_err(|_| ServiceError::Runtime("trust_reuse_runtime_surface_invalid"))?;
    let source_run_id = resolve_trust_reuse_runtime_source_run_id(&runtime_surface)?;
    if let Some(binding) = binding {
        if let Some(expected_source_run_id) = binding.source_run_id.as_deref() {
            if expected_source_run_id != source_run_id {
                return Err(ServiceError::BadRequest(
                    "trust_reuse_binding_source_run_id_mismatch",
                ));
            }
        }
    }

    let events = runtime_surface
        .events
        .iter()
        .filter(|event| event.trust_reuse_outcome != TrustReuseOutcome::Rejected)
        .map(|event| {
            build_trust_reuse_runtime_companion_source_event(
                event,
                &runtime_surface,
                outcome,
                binding,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(AuthoritySinkholeCompanionSourceDocument {
        source_version: 1,
        flow_surface: "trust_reuse".to_string(),
        status: if events.is_empty() {
            "NO_REUSABLE_EVENTS".to_string()
        } else {
            "PASS".to_string()
        },
        run_id: request.run_id.clone(),
        window_model: default_companion_window_model(),
        events,
    }))
}

fn build_companion_source_event(
    outcome: &proof_verifier::types::VerificationOutcome,
    node_binding: &VerificationNodeBinding,
    source_run_id: Option<String>,
    replay_contract_id: Option<String>,
    trust_reuse_source: Option<String>,
    reuse_group_id: Option<String>,
    surface_local_path_id: Option<String>,
) -> Result<AuthoritySinkholeCompanionSourceEvent, ServiceError> {
    let receipt = outcome.receipt.as_ref().ok_or(ServiceError::Runtime(
        "signed_receipt_missing_for_companion_source",
    ))?;
    let timestamp_unix_ns = parse_event_time_to_unix_ns(&receipt.payload.verified_at_utc)
        .map_err(|_| ServiceError::Runtime("companion_source_timestamp_invalid"))?;
    Ok(AuthoritySinkholeCompanionSourceEvent {
        run_id: None,
        timestamp_unix_ns,
        subject_bundle_id: outcome.subject.bundle_id.clone(),
        verification_context_id: outcome.subject.policy_hash.clone(),
        authority_chain_id: node_binding.authority_chain_id.clone(),
        terminal: true,
        reused: true,
        verification_node_id: Some(node_binding.verification_node_id.clone()),
        verifier_id: Some(node_binding.verifier_id.clone()),
        lineage_id: Some(node_binding.lineage_id.clone()),
        execution_cluster_id: node_binding.execution_cluster_id.clone(),
        source_run_id,
        replay_contract_id,
        trust_reuse_source,
        reuse_group_id,
        surface_local_path_id,
    })
}

fn default_companion_window_model() -> String {
    "append_only_event_stream".to_string()
}

fn trust_reuse_runtime_surface_path(bundle_path: &Path) -> PathBuf {
    bundle_path.join(TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH)
}

fn resolve_trust_reuse_runtime_source_run_id(
    report: &TrustReuseRuntimeSurfaceReport,
) -> Result<&str, ServiceError> {
    let mut resolved_source_run_id: Option<&str> = None;
    for event in &report.events {
        let candidate = event
            .source_run_id
            .as_deref()
            .unwrap_or(event.run_id.as_str());
        match resolved_source_run_id {
            Some(current) if current != candidate => {
                return Err(ServiceError::Runtime("trust_reuse_runtime_surface_invalid"));
            }
            Some(_) => {}
            None => resolved_source_run_id = Some(candidate),
        }
    }
    resolved_source_run_id.ok_or(ServiceError::Runtime("trust_reuse_runtime_surface_invalid"))
}

fn build_trust_reuse_runtime_companion_source_event(
    event: &TrustReuseRuntimeEvent,
    report: &TrustReuseRuntimeSurfaceReport,
    outcome: &proof_verifier::types::VerificationOutcome,
    binding: Option<&VerifyBundleTrustReuseBinding>,
) -> Result<AuthoritySinkholeCompanionSourceEvent, ServiceError> {
    if !event.terminal || !event.reused {
        return Err(ServiceError::Runtime("trust_reuse_runtime_surface_invalid"));
    }
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or(ServiceError::Runtime(
            "signed_receipt_missing_for_companion_source",
        ))?;
    let timestamp_unix_ns = parse_event_time_to_unix_ns(&receipt.payload.verified_at_utc)
        .map_err(|_| ServiceError::Runtime("companion_source_timestamp_invalid"))?;
    Ok(AuthoritySinkholeCompanionSourceEvent {
        run_id: None,
        timestamp_unix_ns,
        subject_bundle_id: outcome.subject.bundle_id.clone(),
        verification_context_id: outcome.subject.policy_hash.clone(),
        authority_chain_id: event.authority_chain_id.clone(),
        terminal: event.terminal,
        reused: event.reused,
        verification_node_id: event.verification_node_id.clone(),
        verifier_id: event.verifier_id.clone(),
        lineage_id: event.lineage_id.clone(),
        execution_cluster_id: event.execution_cluster_id.clone(),
        source_run_id: event
            .source_run_id
            .clone()
            .or_else(|| Some(report.run_id.clone())),
        replay_contract_id: None,
        trust_reuse_source: event
            .trust_reuse_source
            .clone()
            .or_else(|| binding.map(|value| value.trust_reuse_source.clone()))
            .or_else(|| Some("native-runtime-trust-reuse".to_string())),
        reuse_group_id: event
            .reuse_group_id
            .clone()
            .or_else(|| binding.and_then(|value| value.reuse_group_id.clone())),
        surface_local_path_id: event
            .surface_local_path_id
            .clone()
            .or_else(|| Some(TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH.to_string())),
    })
}

fn load_replay_runtime_surface(bundle_path: &Path) -> Result<ReplayRuntimeSurface, ServiceError> {
    let replay_report_path = bundle_path.join(replay_report_relative_path());
    let meta_run_path = bundle_path.join("meta/run.json");
    let replay_report = load_json_from_path::<ProofBundleReplayReport>(
        &replay_report_path,
        "replay_boundary_runtime_surface_invalid",
    )?;
    let meta_run = load_json_from_path::<ProofBundleMetaRun>(
        &meta_run_path,
        "replay_boundary_runtime_surface_invalid",
    )?;
    if meta_run.run_id.trim().is_empty()
        || replay_report.status.trim().is_empty()
        || replay_report.replay_execution_trace_hash.trim().is_empty()
        || replay_report.replay_result_hash.trim().is_empty()
        || replay_report.final_state_hash.trim().is_empty()
        || replay_report.replay_event_count == 0
    {
        return Err(ServiceError::Runtime(
            "replay_boundary_runtime_surface_invalid",
        ));
    }
    Ok(ReplayRuntimeSurface {
        source_run_id: meta_run.run_id,
        replay_report,
    })
}

fn receipt_mode_label(mode: &ReceiptMode) -> &'static str {
    match mode {
        ReceiptMode::None => "none",
        ReceiptMode::EmitUnsigned => "emit_unsigned",
        ReceiptMode::EmitSigned => "emit_signed",
    }
}

fn audit_mode_label(mode: AuditMode) -> &'static str {
    match mode {
        AuditMode::None => "none",
        AuditMode::Append => "append",
    }
}

fn write_json_file_if_absent_or_same<T>(
    path: &Path,
    value: &T,
    write_error: &'static str,
    conflict_error: &'static str,
) -> Result<(), ServiceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| ServiceError::Runtime(write_error))?;
    write_bytes_if_absent_or_same(path, &bytes, write_error, conflict_error)
}

fn write_json_value_if_absent_or_same(
    path: &Path,
    value: &Value,
    write_error: &'static str,
    conflict_error: &'static str,
) -> Result<(), ServiceError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| ServiceError::Runtime(write_error))?;
    write_bytes_if_absent_or_same(path, &bytes, write_error, conflict_error)
}

fn write_bytes_if_absent_or_same(
    path: &Path,
    bytes: &[u8],
    write_error: &'static str,
    conflict_error: &'static str,
) -> Result<(), ServiceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| ServiceError::Runtime(write_error))?;
    }
    match fs::read(path) {
        Ok(existing) if existing == bytes => Ok(()),
        Ok(_) => Err(ServiceError::Runtime(conflict_error)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, bytes).map_err(|_| ServiceError::Runtime(write_error))
        }
        Err(_) => Err(ServiceError::Runtime(write_error)),
    }
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(b"0123456789abcdef"[(byte >> 4) as usize]));
        output.push(char::from(b"0123456789abcdef"[(byte & 0x0f) as usize]));
    }
    output
}

fn verdict_label(verdict: &proof_verifier::Verdict) -> &'static str {
    match verdict {
        proof_verifier::Verdict::Trusted => "TRUSTED",
        proof_verifier::Verdict::Untrusted => "UNTRUSTED",
        proof_verifier::Verdict::Invalid => "INVALID",
        proof_verifier::Verdict::RejectedByPolicy => "REJECTED_BY_POLICY",
    }
}

fn list_run_artifacts(run_dir: &Path) -> Result<Vec<String>, ServiceError> {
    let entries = fs::read_dir(run_dir).map_err(|_| ServiceError::NotFound("run_dir_not_found"))?;
    let mut artifacts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| ServiceError::MalformedArtifact("dir_read_error"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if RUN_LEVEL_ARTIFACTS.contains(&name.as_str()) {
            artifacts.push(name);
        }
    }
    artifacts.sort();
    Ok(artifacts)
}

fn is_safe_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && !segment.contains('/')
        && !segment.contains('\\')
}

fn is_observability_path(path: &str) -> bool {
    path == "/diagnostics" || path.starts_with("/diagnostics/")
}

fn serve_json_file(path: PathBuf) -> DiagnosticsResponse {
    match read_json_file(&path) {
        Ok(value) => json_response(200, value),
        Err(error) => error_response(error),
    }
}

fn read_json_file(path: &Path) -> Result<Value, ServiceError> {
    let text =
        fs::read_to_string(path).map_err(|_| ServiceError::NotFound("artifact_not_found"))?;
    serde_json::from_str(&text).map_err(|_| ServiceError::MalformedArtifact("invalid_json"))
}

fn json_response(status_code: u16, value: Value) -> DiagnosticsResponse {
    DiagnosticsResponse {
        status_code,
        body: serde_json::to_vec_pretty(&value).unwrap_or_else(|_| b"{}".to_vec()),
        content_type: "application/json; charset=utf-8",
    }
}

fn error_response(error: ServiceError) -> DiagnosticsResponse {
    match error {
        ServiceError::BadRequest(code) => json_response(400, json!({ "error": code })),
        ServiceError::NotFound(code) => json_response(404, json!({ "error": code })),
        ServiceError::MalformedArtifact(code) => json_response(500, json!({ "error": code })),
        ServiceError::Runtime(code) => json_response(500, json!({ "error": code })),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceError {
    BadRequest(&'static str),
    NotFound(&'static str),
    MalformedArtifact(&'static str),
    Runtime(&'static str),
}

#[cfg(test)]
mod tests {
    use super::{
        route_request, route_request_with_body, DiagnosticsResponse, MAX_VERIFY_BUNDLE_BODY_BYTES,
    };
    use proof_verifier::testing::fixtures::create_fixture_bundle;
    use proof_verifier::trust_reuse_runtime_surface::{
        compute_trust_reuse_runtime_event_id, TrustReuseOutcome, TrustReuseRuntimeEvent,
        TrustReuseRuntimeSurfaceReport,
    };
    use serde::Serialize;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("proofd-test-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn write_artifact(dir: &PathBuf, name: &str, body: &str) {
        fs::write(dir.join(name), body).expect("write artifact");
    }

    fn write_json<T>(path: &std::path::Path, value: &T)
    where
        T: Serialize,
    {
        fs::write(
            path,
            serde_json::to_vec_pretty(value).expect("serialize json"),
        )
        .expect("write json");
    }

    fn body_json(response: DiagnosticsResponse) -> serde_json::Value {
        serde_json::from_slice(&response.body).expect("valid json body")
    }

    #[test]
    fn incidents_endpoint_filters_by_severity() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_determinism_incidents.json",
            r#"{
              "node_count": 5,
              "surface_partition_count": 1,
              "determinism_incident_count": 2,
              "severity_counts": {
                "pure_determinism_failure": 1,
                "authority_drift": 1
              },
              "incidents": [
                {"incident_id":"sha256:a","surface_key":"s1","severity":"pure_determinism_failure","nodes":["n1","n2"]},
                {"incident_id":"sha256:b","surface_key":"s2","severity":"authority_drift","nodes":["n3"]}
              ]
            }"#,
        );

        let response = route_request(
            "GET",
            "/diagnostics/incidents?severity=pure_determinism_failure",
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("determinism_incident_count")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            body.get("severity_counts")
                .and_then(|v| v.get("pure_determinism_failure"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            body.get("incidents")
                .and_then(|v| v.as_array())
                .map(|v| v.len()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn incidents_endpoint_rejects_unknown_query_parameter() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_determinism_incidents.json",
            r#"{"incidents":[]}"#,
        );

        let response = route_request("GET", "/diagnostics/incidents?select_winner=true", &dir);
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn single_incident_endpoint_returns_matching_object() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_determinism_incidents.json",
            r#"{
              "incidents": [
                {"incident_id":"sha256:abc","surface_key":"s1","severity":"pure_determinism_failure","nodes":["n1","n2"]}
              ]
            }"#,
        );

        let response = route_request("GET", "/diagnostics/incidents/sha256:abc", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("incident_id").and_then(|v| v.as_str()),
            Some("sha256:abc")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parity_endpoint_serves_raw_artifact() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_report.json",
            r#"{"status":"PASS","row_count":10}"#,
        );

        let response = route_request("GET", "/diagnostics/parity", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("PASS"));
        assert_eq!(body.get("row_count").and_then(|v| v.as_u64()), Some(10));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn graph_endpoint_serves_raw_artifact() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_incident_graph.json",
            r#"{"status":"PASS","graph":{"node_count":2,"edge_count":1,"incident_count":1}}"#,
        );

        let response = route_request("GET", "/diagnostics/graph", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("PASS"));
        assert_eq!(
            body.get("graph")
                .and_then(|v| v.get("incident_count"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn graph_endpoint_rejects_truth_selection_query() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_incident_graph.json",
            r#"{"status":"PASS","graph":{"node_count":2,"edge_count":1,"incident_count":1}}"#,
        );

        let response = route_request("GET", "/diagnostics/graph?select_winner=true", &dir);
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn authority_topology_endpoint_serves_raw_artifact() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_authority_drift_topology.json",
            r#"{"status":"PASS","topology":{"node_count":3,"authority_cluster_count":2,"dominant_authority_chain_id":"chain-a"}}"#,
        );

        let response = route_request("GET", "/diagnostics/authority-topology", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("PASS"));
        assert_eq!(
            body.get("topology")
                .and_then(|v| v.get("authority_cluster_count"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn authority_suppression_endpoint_serves_raw_artifact() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_authority_suppression_report.json",
            r#"{"status":"PASS","suppression":{"suppressed_drift_count":1,"rule_counts":{"historical_shadow":1}}}"#,
        );

        let response = route_request("GET", "/diagnostics/authority-suppression", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("PASS"));
        assert_eq!(
            body.get("suppression")
                .and_then(|v| v.get("suppressed_drift_count"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn runs_endpoint_lists_only_directories_with_known_artifacts() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b");
        let scenario_reports = dir.join("scenario_reports");
        fs::create_dir_all(&run_a).expect("create run a");
        fs::create_dir_all(&run_b).expect("create run b");
        fs::create_dir_all(&scenario_reports).expect("create scenario reports");

        write_artifact(&run_a, "parity_report.json", r#"{"status":"PASS"}"#);
        write_artifact(
            &run_a,
            "parity_determinism_incidents.json",
            r#"{"incidents":[]}"#,
        );
        write_artifact(&run_b, "parity_report.json", r#"{"status":"PASS"}"#);
        write_artifact(&scenario_reports, "row-1.json", r#"{"scenario":"ignored"}"#);

        let response = route_request("GET", "/diagnostics/runs", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("run_count").and_then(|v| v.as_u64()), Some(2));
        let runs = body
            .get("runs")
            .and_then(|v| v.as_array())
            .expect("runs array");
        assert_eq!(runs.len(), 2);
        assert_eq!(
            runs[0].get("run_id").and_then(|v| v.as_str()),
            Some("run-a")
        );
        assert_eq!(
            runs[1].get("run_id").and_then(|v| v.as_str()),
            Some("run-b")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_incidents_endpoint_serves_selected_run_artifact() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_determinism_incidents.json",
            r#"{"determinism_incident_count":1,"incidents":[{"incident_id":"sha256:r1"}]}"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/incidents", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("incidents")
                .and_then(|v| v.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("incident_id"))
                .and_then(|v| v.as_str()),
            Some("sha256:r1")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_summary_endpoint_serves_selected_run_metadata() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(&run_dir, "parity_report.json", r#"{"status":"PASS"}"#);
        write_artifact(
            &run_dir,
            "parity_determinism_incidents.json",
            r#"{"determinism_incident_count":1,"incidents":[{"incident_id":"sha256:r1"}]}"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("run_id").and_then(|v| v.as_str()),
            Some("run-20260310-1")
        );
        assert_eq!(
            body.get("artifacts")
                .and_then(|v| v.as_array())
                .map(|items| items.len()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_graph_endpoint_serves_selected_run_artifact() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_incident_graph.json",
            r#"{"graph":{"node_count":3,"edge_count":2,"incident_count":1}}"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/graph", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("graph")
                .and_then(|v| v.get("edge_count"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_drift_and_convergence_endpoints_serve_selected_artifacts() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_drift_attribution_report.json",
            r#"{"status":"PASS","node_count":3}"#,
        );
        write_artifact(
            &run_dir,
            "parity_convergence_report.json",
            r#"{"status":"PASS","node_count":3,"surface_partition_count":2}"#,
        );

        let drift = route_request("GET", "/diagnostics/runs/run-20260310-1/drift", &dir);
        assert_eq!(drift.status_code, 200);
        let drift_body = body_json(drift);
        assert_eq!(
            drift_body.get("node_count").and_then(|v| v.as_u64()),
            Some(3)
        );

        let convergence =
            route_request("GET", "/diagnostics/runs/run-20260310-1/convergence", &dir);
        assert_eq!(convergence.status_code, 200);
        let convergence_body = body_json(convergence);
        assert_eq!(
            convergence_body
                .get("surface_partition_count")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn convergence_endpoint_rejects_commit_query() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_convergence_report.json",
            r#"{"status":"PASS","surface_partition_count":2}"#,
        );

        let response = route_request("GET", "/diagnostics/convergence?commit=true", &dir);
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_authority_topology_endpoint_serves_selected_run_artifact() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_authority_drift_topology.json",
            r#"{"topology":{"node_count":3,"drifted_node_count":1,"dominant_authority_chain_id":"chain-a"}}"#,
        );

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-20260310-1/authority-topology",
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("topology")
                .and_then(|v| v.get("drifted_node_count"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_authority_suppression_endpoint_serves_selected_run_artifact() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_authority_suppression_report.json",
            r#"{"suppression":{"suppressed_drift_count":1,"rule_counts":{"historical_shadow":1}}}"#,
        );

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-20260310-1/authority-suppression",
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("suppression")
                .and_then(|v| v.get("suppressed_drift_count"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_parity_endpoint_rejects_invalid_run_id() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/runs/../parity", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("invalid_run_id")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_executes_verifier_core_and_emits_receipt() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-execution-r1",
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(
            body.get("run_id").and_then(|v| v.as_str()),
            Some("run-proofd-execution-r1")
        );
        assert_eq!(
            body.get("verdict").and_then(|v| v.as_str()),
            Some("TRUSTED")
        );
        assert_eq!(
            body.get("receipt_emitted").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("receipt_path").and_then(|v| v.as_str()),
            Some("receipts/verification_receipt.json")
        );

        let run_dir = dir.join("run-proofd-execution-r1");
        assert!(run_dir.join("proofd_run_manifest.json").is_file());
        assert!(run_dir.join("receipts/verification_receipt.json").is_file());

        let run_summary = body_json(route_request(
            "GET",
            "/diagnostics/runs/run-proofd-execution-r1",
            &dir,
        ));
        assert!(run_summary
            .get("artifacts")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("proofd_run_manifest.json"))));

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diagnostics_namespace_rejects_post_methods() {
        let dir = temp_dir();
        let response = route_request_with_body("POST", "/diagnostics/graph", Some(br#"{}"#), &dir);
        assert_eq!(response.status_code, 405);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("method_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn authority_observability_endpoint_rejects_post_methods() {
        let dir = temp_dir();
        let response = route_request_with_body(
            "POST",
            "/diagnostics/authority-topology",
            Some(br#"{}"#),
            &dir,
        );
        assert_eq!(response.status_code, 405);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("method_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_emits_signed_receipt_when_signer_present() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            },
            "replay_boundary_binding": {
                "replay_contract_id": "replay-contract-proofd-local-a",
                "source_run_id": "fixture-run",
                "reuse_group_id": "reuse-group-proofd-a",
                "surface_local_path_id": "replay-path-proofd-a"
            },
            "trust_reuse_binding": {
                "trust_reuse_source": "trust-overlay-cache",
                "source_run_id": "source-run-proofd-bootstrap-a",
                "reuse_group_id": "reuse-group-proofd-a",
                "surface_local_path_id": "trust-path-proofd-a"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(
            body.get("run_id").and_then(|v| v.as_str()),
            Some("run-proofd-execution-r2")
        );
        assert_eq!(
            body.get("verdict").and_then(|v| v.as_str()),
            Some("TRUSTED")
        );
        assert_eq!(
            body.get("receipt_emitted").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("behavioral_observability_emitted")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("audit_ledger_path").and_then(|v| v.as_str()),
            Some("verification_audit_ledger.jsonl")
        );
        assert_eq!(
            body.get("verification_diversity_ledger_binding_path")
                .and_then(|v| v.as_str()),
            Some("verification_diversity_ledger_binding.json")
        );
        assert_eq!(
            body.get("verification_diversity_ledger_path")
                .and_then(|v| v.as_str()),
            Some("verification_diversity_ledger.json")
        );
        assert_eq!(
            body.get("replay_boundary_flow_source_path")
                .and_then(|v| v.as_str()),
            Some("replay_boundary_flow_source.json")
        );
        assert_eq!(
            body.get("replay_boundary_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_replay")
        );
        assert_eq!(
            body.get("trust_reuse_flow_source_path")
                .and_then(|v| v.as_str()),
            Some("trust_reuse_flow_source.json")
        );
        assert_eq!(
            body.get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_trust_reuse")
        );

        let receipt = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2")
                    .join("receipts/verification_receipt.json"),
            )
            .expect("read receipt"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            receipt
                .get("verifier_signature_algorithm")
                .and_then(|v| v.as_str()),
            Some("ed25519")
        );
        assert!(receipt
            .get("verifier_signature")
            .and_then(|v| v.as_str())
            .is_some_and(|value| !value.is_empty()));
        assert_eq!(
            receipt.get("verifier_key_id").and_then(|v| v.as_str()),
            Some("receipt-ed25519-key-2026-03-a")
        );

        let run_manifest = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2")
                    .join("proofd_run_manifest.json"),
            )
            .expect("read run manifest"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            run_manifest.get("receipt_mode").and_then(|v| v.as_str()),
            Some("emit_signed")
        );
        assert_eq!(
            run_manifest
                .get("behavioral_observability_emitted")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            run_manifest.get("audit_mode").and_then(|v| v.as_str()),
            Some("append")
        );
        assert_eq!(
            run_manifest
                .get("audit_ledger_path")
                .and_then(|v| v.as_str()),
            Some("verification_audit_ledger.jsonl")
        );
        assert_eq!(
            run_manifest
                .get("verification_diversity_ledger_binding_path")
                .and_then(|v| v.as_str()),
            Some("verification_diversity_ledger_binding.json")
        );
        assert_eq!(
            run_manifest
                .get("verification_diversity_ledger_path")
                .and_then(|v| v.as_str()),
            Some("verification_diversity_ledger.json")
        );
        assert_eq!(
            run_manifest
                .get("replay_boundary_flow_source_path")
                .and_then(|v| v.as_str()),
            Some("replay_boundary_flow_source.json")
        );
        assert_eq!(
            run_manifest
                .get("replay_boundary_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_replay")
        );
        assert_eq!(
            run_manifest
                .get("trust_reuse_flow_source_path")
                .and_then(|v| v.as_str()),
            Some("trust_reuse_flow_source.json")
        );
        assert_eq!(
            run_manifest
                .get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_trust_reuse")
        );
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("verification_audit_ledger.jsonl")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("verification_diversity_ledger_binding.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("verification_diversity_ledger.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("verification_diversity_ledger_append_report.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("replay_boundary_flow_source.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("trust_reuse_flow_source.json")
            .is_file());
        let replay_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2")
                    .join("replay_boundary_flow_source.json"),
            )
            .expect("read replay source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            replay_source.get("flow_surface").and_then(|v| v.as_str()),
            Some("replay_boundary")
        );
        assert_eq!(
            replay_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("source_run_id"))
                .and_then(|v| v.as_str()),
            Some("fixture-run")
        );
        let trust_reuse_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2")
                    .join("trust_reuse_flow_source.json"),
            )
            .expect("read trust reuse source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            trust_reuse_source
                .get("flow_surface")
                .and_then(|v| v.as_str()),
            Some("trust_reuse")
        );
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("source_run_id"))
                .and_then(|v| v.as_str()),
            Some("source-run-proofd-bootstrap-a")
        );

        let run_summary = body_json(route_request(
            "GET",
            "/diagnostics/runs/run-proofd-execution-r2",
            &dir,
        ));
        assert!(run_summary
            .get("artifacts")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("verification_audit_ledger.jsonl"))));
        assert!(run_summary
            .get("artifacts")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("verification_diversity_ledger.json"))));
        assert!(run_summary
            .get("artifacts")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("replay_boundary_flow_source.json"))));
        assert!(run_summary
            .get("artifacts")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("trust_reuse_flow_source.json"))));

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_requires_receipt_signer_for_emit_signed() {
        let dir = temp_dir();
        let request_body = json!({
            "bundle_path": "/abs/bundle",
            "policy_path": "/abs/policy.json",
            "registry_path": "/abs/registry.json",
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2",
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("receipt_signer_missing")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_diversity_binding_without_emit_signed() {
        let dir = temp_dir();
        let request_body = json!({
            "bundle_path": "/abs/bundle",
            "policy_path": "/abs/policy.json",
            "registry_path": "/abs/registry.json",
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-execution-r3",
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("diversity_binding_requires_emit_signed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_replay_boundary_binding_without_diversity_binding() {
        let dir = temp_dir();
        let request_body = json!({
            "bundle_path": "/abs/bundle",
            "policy_path": "/abs/policy.json",
            "registry_path": "/abs/registry.json",
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r3b",
            "receipt_signer": {
                "verifier_node_id": "node-b",
                "verifier_key_id": "key-b",
                "signature_algorithm": "ed25519",
                "private_key": "base64:abc",
                "verified_at_utc": "2026-03-08T12:00:00Z"
            },
            "replay_boundary_binding": {
                "replay_contract_id": "replay-contract-proofd-local-a"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("replay_boundary_binding_requires_diversity_binding")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_blank_trust_reuse_source() {
        let dir = temp_dir();
        let request_body = json!({
            "bundle_path": "/abs/bundle",
            "policy_path": "/abs/policy.json",
            "registry_path": "/abs/registry.json",
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r3c",
            "receipt_signer": {
                "verifier_node_id": "node-b",
                "verifier_key_id": "key-b",
                "signature_algorithm": "ed25519",
                "private_key": "base64:abc",
                "verified_at_utc": "2026-03-08T12:00:00Z"
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b"
            },
            "trust_reuse_binding": {
                "trust_reuse_source": "   "
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("trust_reuse_binding_trust_reuse_source_missing")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_auto_emits_runtime_replay_and_trust_reuse_sources() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2b",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("replay_boundary_flow_source_path")
                .and_then(|v| v.as_str()),
            Some("replay_boundary_flow_source.json")
        );
        assert_eq!(
            body.get("replay_boundary_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_replay")
        );
        assert_eq!(
            body.get("trust_reuse_flow_source_path")
                .and_then(|v| v.as_str()),
            Some("trust_reuse_flow_source.json")
        );
        assert_eq!(
            body.get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_trust_reuse")
        );

        let replay_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2b")
                    .join("replay_boundary_flow_source.json"),
            )
            .expect("read replay source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            replay_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("source_run_id"))
                .and_then(|v| v.as_str()),
            Some("fixture-run")
        );
        let trust_reuse_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2b")
                    .join("trust_reuse_flow_source.json"),
            )
            .expect("read trust reuse source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("source_run_id"))
                .and_then(|v| v.as_str()),
            Some("source-run-proofd-bootstrap-a")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_prefers_native_trust_reuse_over_request_binding() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2d",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            },
            "trust_reuse_binding": {
                "trust_reuse_source": "trust-overlay-cache",
                "source_run_id": "source-run-proofd-bootstrap-a",
                "reuse_group_id": "reuse-group-proofd-a",
                "surface_local_path_id": "trust-path-proofd-a"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_trust_reuse")
        );

        let trust_reuse_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2d")
                    .join("trust_reuse_flow_source.json"),
            )
            .expect("read trust reuse source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("trust_reuse_source"))
                .and_then(|v| v.as_str()),
            Some("native-runtime-trust-reuse")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_falls_back_to_request_bound_trust_reuse_when_native_surface_missing()
    {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);
        fs::remove_file(
            fixture
                .root
                .join("reports/trust_reuse_runtime_surface.json"),
        )
        .expect("remove native trust reuse surface");

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2e",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            },
            "trust_reuse_binding": {
                "trust_reuse_source": "trust-overlay-cache",
                "source_run_id": "source-run-proofd-bootstrap-a",
                "reuse_group_id": "reuse-group-proofd-a",
                "surface_local_path_id": "trust-path-proofd-a"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("request_binding")
        );

        let trust_reuse_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2e")
                    .join("trust_reuse_flow_source.json"),
            )
            .expect("read trust reuse source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("trust_reuse_source"))
                .and_then(|v| v.as_str()),
            Some("trust-overlay-cache")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_trust_reuse_binding_source_run_id_mismatch() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2f",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            },
            "trust_reuse_binding": {
                "trust_reuse_source": "trust-overlay-cache",
                "source_run_id": "mismatched-source-run"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("trust_reuse_binding_source_run_id_mismatch")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_keeps_native_trust_reuse_when_all_events_are_rejected() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let mut event = TrustReuseRuntimeEvent {
            event_schema_version: 1,
            event_id: String::new(),
            run_id: "fixture-run".to_string(),
            timestamp_unix_ns: 1_710_000_000_000_000_000,
            subject_bundle_id: "fixture-bundle-subject".to_string(),
            verification_context_id: "proofd-policy-hash-a".to_string(),
            authority_chain_id: "sha256:proofd-authority-chain-node-b".to_string(),
            trust_reuse_outcome: TrustReuseOutcome::Rejected,
            terminal: true,
            reused: true,
            receipt_ref: "receipts/verification_receipt.json".to_string(),
            verification_context_ref: "cas:sha256:fixture-verification-context".to_string(),
            verifier_attestation_ref: "cas:sha256:fixture-verifier-attestation".to_string(),
            verifier_registry_snapshot_hash: "a".repeat(64),
            verification_node_id: Some("node-b".to_string()),
            verifier_id: Some("verifier-node-b".to_string()),
            lineage_id: Some("lineage-receipt-node-b".to_string()),
            execution_cluster_id: Some("cluster-local-a".to_string()),
            source_run_id: Some("source-run-proofd-bootstrap-a".to_string()),
            reuse_group_id: Some("reuse-group-proofd-a".to_string()),
            surface_local_path_id: Some("reports/trust_reuse_runtime_surface.json".to_string()),
            trust_reuse_source: Some("native-runtime-trust-reuse".to_string()),
        };
        event.event_id = compute_trust_reuse_runtime_event_id(&event).expect("compute event id");
        write_json(
            &fixture
                .root
                .join("reports/trust_reuse_runtime_surface.json"),
            &TrustReuseRuntimeSurfaceReport {
                surface_version: 1,
                flow_surface: "trust_reuse_runtime".to_string(),
                status: "PASS".to_string(),
                run_id: "fixture-run".to_string(),
                source_kind: "local_runtime_evidence".to_string(),
                event_count: 1,
                accepted_event_count: 0,
                historical_only_event_count: 0,
                rejected_event_count: 1,
                events: vec![event],
            },
        );

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2g",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            },
            "trust_reuse_binding": {
                "trust_reuse_source": "trust-overlay-cache",
                "source_run_id": "source-run-proofd-bootstrap-a",
                "reuse_group_id": "reuse-group-proofd-a",
                "surface_local_path_id": "trust-path-proofd-a"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_bundle_trust_reuse")
        );

        let trust_reuse_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2g")
                    .join("trust_reuse_flow_source.json"),
            )
            .expect("read trust reuse source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            trust_reuse_source.get("status").and_then(|v| v.as_str()),
            Some("NO_REUSABLE_EVENTS")
        );
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .map(|events| events.len()),
            Some(0)
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_replay_boundary_binding_source_run_id_mismatch() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2c",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b"
            },
            "replay_boundary_binding": {
                "replay_contract_id": "replay-contract-proofd-local-a",
                "source_run_id": "mismatched-source-run"
            }
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("replay_boundary_binding_source_run_id_mismatch")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_oversized_request_body() {
        let dir = temp_dir();
        let oversized = vec![b'a'; MAX_VERIFY_BUNDLE_BODY_BYTES + 1];
        let response =
            route_request_with_body("POST", "/verify/bundle", Some(oversized.as_slice()), &dir);
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("request_body_too_large")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_run_id_reuse_with_different_request_fingerprint() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let first_request = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r4",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            }
        });
        let first_bytes = serde_json::to_vec(&first_request).expect("serialize first request");
        let first_response =
            route_request_with_body("POST", "/verify/bundle", Some(first_bytes.as_slice()), &dir);
        assert_eq!(first_response.status_code, 200);

        let second_request = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r4",
            "receipt_signer": {
                "verifier_node_id": fixture.receipt_signer.verifier_node_id,
                "verifier_key_id": fixture.receipt_signer.verifier_key_id,
                "signature_algorithm": fixture.receipt_signer.signature_algorithm,
                "private_key": fixture.receipt_signer.private_key,
                "verified_at_utc": fixture.receipt_signer.verified_at_utc,
            },
            "diversity_binding": {
                "verifier_id": "verifier-node-b-variant",
                "authority_chain_id": "sha256:proofd-authority-chain-node-b",
                "lineage_id": "lineage-receipt-node-b",
                "execution_cluster_id": "cluster-local-a",
            }
        });
        let second_bytes = serde_json::to_vec(&second_request).expect("serialize second request");
        let second_response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(second_bytes.as_slice()),
            &dir,
        );
        assert_eq!(second_response.status_code, 400);
        let body = body_json(second_response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("run_id_request_fingerprint_mismatch")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_rejects_relative_policy_path() {
        let dir = temp_dir();
        let request_body = json!({
            "bundle_path": "/abs/bundle",
            "policy_path": "relative-policy.json",
            "registry_path": "/abs/registry.json",
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-execution-r1",
        });
        let request_bytes = serde_json::to_vec(&request_body).expect("serialize request");
        let response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("policy_path_not_absolute")
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
