use proof_verifier::types::{AuditMode, ReceiptMode, ReceiptSignerConfig, VerifyRequest};
use proof_verifier::{verify_bundle, RegistrySnapshot, TrustPolicy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const RUN_LEVEL_ARTIFACTS: &[&str] = &[
    "report.json",
    "parity_report.json",
    "proofd_run_manifest.json",
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum VerifyBundleReceiptMode {
    None,
    EmitUnsigned,
    EmitSigned,
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyBundleReceiptSigner {
    verifier_node_id: String,
    verifier_key_id: String,
    signature_algorithm: String,
    private_key: String,
    verified_at_utc: String,
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyBundleRequestBody {
    bundle_path: String,
    policy_path: String,
    registry_path: String,
    #[serde(default)]
    receipt_mode: Option<VerifyBundleReceiptMode>,
    run_id: String,
    #[serde(default)]
    receipt_signer: Option<VerifyBundleReceiptSigner>,
}

#[derive(Debug, Clone, Serialize)]
struct VerifyBundleResponseBody {
    status: &'static str,
    run_id: String,
    verdict: &'static str,
    verdict_subject: Value,
    receipt_emitted: bool,
    receipt_path: Option<String>,
    findings_count: usize,
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
    let policy = load_json_from_path::<TrustPolicy>(&policy_path, "invalid_policy_json")?;
    let registry =
        load_json_from_path::<RegistrySnapshot>(&registry_path, "invalid_registry_json")?;
    let receipt_mode = map_receipt_mode(request.receipt_mode.as_ref());
    let receipt_signer = request
        .receipt_signer
        .as_ref()
        .map(map_receipt_signer_config);

    if receipt_mode == ReceiptMode::EmitSigned && receipt_signer.is_none() {
        return Err(ServiceError::BadRequest("receipt_signer_missing"));
    }

    let verify_request = VerifyRequest {
        bundle_path: &bundle_path,
        policy: &policy,
        registry_snapshot: &registry,
        receipt_mode: receipt_mode.clone(),
        receipt_signer: receipt_signer.as_ref(),
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    let outcome = verify_bundle(&verify_request)
        .map_err(|_| ServiceError::Runtime("verifier_runtime_failure"))?;

    let run_dir = evidence_dir.join(&request.run_id);
    fs::create_dir_all(&run_dir).map_err(|_| ServiceError::Runtime("run_dir_create_failed"))?;

    let receipt_relative_path = if let Some(receipt) = &outcome.receipt {
        let receipts_dir = run_dir.join("receipts");
        fs::create_dir_all(&receipts_dir)
            .map_err(|_| ServiceError::Runtime("receipt_dir_create_failed"))?;
        let receipt_path = receipts_dir.join("verification_receipt.json");
        write_json_file(&receipt_path, receipt)
            .map_err(|_| ServiceError::Runtime("receipt_write_failed"))?;
        Some("receipts/verification_receipt.json".to_string())
    } else {
        None
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
        "verdict": verdict_label(&outcome.verdict),
        "verdict_subject": outcome.subject,
        "findings_count": outcome.findings.len(),
    });
    write_json_value(&run_dir.join("proofd_run_manifest.json"), &run_manifest)
        .map_err(|_| ServiceError::Runtime("run_manifest_write_failed"))?;

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
    for part in raw_query.unwrap_or("").split('&').filter(|part| !part.is_empty()) {
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

    serde_json::from_slice(raw_body).map_err(|_| ServiceError::BadRequest("invalid_request_body"))
}

fn validate_verify_bundle_request(request: &VerifyBundleRequestBody) -> Result<(), ServiceError> {
    if request.run_id.is_empty() || !is_safe_path_segment(&request.run_id) {
        return Err(ServiceError::BadRequest("invalid_run_id"));
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

    Ok(())
}

fn load_json_from_path<T>(path: &Path, error_code: &'static str) -> Result<T, ServiceError>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = fs::read(path).map_err(|_| ServiceError::BadRequest(error_code))?;
    serde_json::from_slice(&bytes).map_err(|_| ServiceError::BadRequest(error_code))
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

fn receipt_mode_label(mode: &ReceiptMode) -> &'static str {
    match mode {
        ReceiptMode::None => "none",
        ReceiptMode::EmitUnsigned => "emit_unsigned",
        ReceiptMode::EmitSigned => "emit_signed",
    }
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

fn write_json_file<T>(path: &Path, value: &T) -> Result<(), serde_json::Error>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).map_err(serde_json::Error::io)
}

fn write_json_value(path: &Path, value: &Value) -> Result<(), serde_json::Error> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).map_err(serde_json::Error::io)
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
    use super::{route_request, route_request_with_body, DiagnosticsResponse};
    use proof_verifier::testing::fixtures::create_fixture_bundle;
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
        let response = route_request_with_body(
            "POST",
            "/diagnostics/graph",
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
