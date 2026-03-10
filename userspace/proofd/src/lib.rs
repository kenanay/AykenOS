use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const RUN_LEVEL_ARTIFACTS: &[&str] = &[
    "report.json",
    "parity_report.json",
    "parity_authority_suppression_report.json",
    "parity_authority_drift_topology.json",
    "parity_incident_graph.json",
    "parity_consistency_report.json",
    "parity_determinism_report.json",
    "parity_determinism_incidents.json",
    "parity_drift_attribution_report.json",
    "parity_convergence_report.json",
    "failure_matrix.json",
];

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
    if method != "GET" {
        return json_response(405, json!({ "error": "method_not_allowed" }));
    }

    let target = parse_target(raw_target);
    match target.path.as_str() {
        "/healthz" => json_response(
            200,
            json!({
                "status": "ok",
                "service": "proofd",
                "mode": "read_only_diagnostics",
            }),
        ),
        "/diagnostics/incidents" => match load_incident_report(evidence_dir, target.query.as_deref())
        {
            Ok(value) => json_response(200, value),
            Err(error) => error_response(error),
        },
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
        "/diagnostics/failure-matrix" => serve_json_file(evidence_dir.join("failure_matrix.json")),
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

fn list_runs(evidence_dir: &Path) -> Result<Value, ServiceError> {
    let entries = fs::read_dir(evidence_dir).map_err(|_| ServiceError::NotFound("evidence_dir_not_found"))?;
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

        let artifacts = list_run_artifacts(&path)?;
        if artifacts.is_empty() {
            continue;
        }

        runs.push(json!({
            "run_id": run_id,
            "artifacts": artifacts,
        }));
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
    if parts.len() < 4 {
        return json_response(404, json!({ "error": "invalid_run_path" }));
    }

    let run_id = parts[2];
    if !is_safe_path_segment(run_id) {
        return json_response(404, json!({ "error": "invalid_run_id" }));
    }

    let run_dir = evidence_dir.join(run_id);
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
        "graph" if parts.len() == 4 => {
            serve_json_file(run_dir.join("parity_incident_graph.json"))
        }
        _ => json_response(404, json!({ "error": "not_found" })),
    };
    response
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
    let filters = parse_query(raw_query);
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
            let current = acc
                .get(severity)
                .and_then(Value::as_u64)
                .unwrap_or(0);
            acc.insert(severity.to_string(), json!(current + 1));
        }
        acc
    });

    if let Some(object) = report.as_object_mut() {
        object.insert(
            "determinism_incident_count".to_string(),
            json!(filtered.len()),
        );
        object.insert("severity_counts".to_string(), Value::Object(severity_counts));
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
            .map(|nodes| nodes.iter().any(|item| item.as_str() == Some(value.as_str())))
            .unwrap_or(false),
        _ => true,
    })
}

fn parse_query(raw_query: Option<&str>) -> Vec<(String, String)> {
    raw_query
        .unwrap_or("")
        .split('&')
        .filter(|part| !part.is_empty())
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
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

fn serve_json_file(path: PathBuf) -> DiagnosticsResponse {
    match read_json_file(&path) {
        Ok(value) => json_response(200, value),
        Err(error) => error_response(error),
    }
}

fn read_json_file(path: &Path) -> Result<Value, ServiceError> {
    let text = fs::read_to_string(path).map_err(|_| ServiceError::NotFound("artifact_not_found"))?;
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
        ServiceError::NotFound(code) => json_response(404, json!({ "error": code })),
        ServiceError::MalformedArtifact(code) => json_response(500, json!({ "error": code })),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceError {
    NotFound(&'static str),
    MalformedArtifact(&'static str),
}

#[cfg(test)]
mod tests {
    use super::{route_request, DiagnosticsResponse};
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
        assert_eq!(body.get("determinism_incident_count").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            body.get("severity_counts")
                .and_then(|v| v.get("pure_determinism_failure"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(body.get("incidents").and_then(|v| v.as_array()).map(|v| v.len()), Some(1));
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
        assert_eq!(body.get("incident_id").and_then(|v| v.as_str()), Some("sha256:abc"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parity_endpoint_serves_raw_artifact() {
        let dir = temp_dir();
        write_artifact(&dir, "parity_report.json", r#"{"status":"PASS","row_count":10}"#);

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
        write_artifact(&run_a, "parity_determinism_incidents.json", r#"{"incidents":[]}"#);
        write_artifact(&run_b, "parity_report.json", r#"{"status":"PASS"}"#);
        write_artifact(&scenario_reports, "row-1.json", r#"{"scenario":"ignored"}"#);

        let response = route_request("GET", "/diagnostics/runs", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("run_count").and_then(|v| v.as_u64()), Some(2));
        let runs = body.get("runs").and_then(|v| v.as_array()).expect("runs array");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].get("run_id").and_then(|v| v.as_str()), Some("run-a"));
        assert_eq!(runs[1].get("run_id").and_then(|v| v.as_str()), Some("run-b"));
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
    fn run_scoped_authority_topology_endpoint_serves_selected_run_artifact() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_authority_drift_topology.json",
            r#"{"topology":{"node_count":3,"drifted_node_count":1,"dominant_authority_chain_id":"chain-a"}}"#,
        );

        let response =
            route_request("GET", "/diagnostics/runs/run-20260310-1/authority-topology", &dir);
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
        assert_eq!(body.get("error").and_then(|v| v.as_str()), Some("invalid_run_id"));
        let _ = fs::remove_dir_all(&dir);
    }
}
