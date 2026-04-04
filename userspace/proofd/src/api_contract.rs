#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointScope {
    Root,
    RunScoped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticsEndpointId {
    Version,
    Runs,
    Federation,
    Context,
    Trust,
    Parity,
    ParityContextRelation,
    Incidents,
    IncidentById,
    FingerprintBoundary,
    ReplicatedBoundary,
    AuthoritySuppression,
    AuthorityTopology,
    Graph,
    Drift,
    Convergence,
    FailureMatrix,
    RunSummary,
    RunArtifactsIndex,
    RunArtifactFile,
    RunFederation,
    RunContext,
    RunRegistry,
    RunBoundary,
    RunIncidents,
    RunParity,
    RunAuthoritySuppression,
    RunAuthorityTopology,
    RunGraph,
    RunDrift,
    RunConvergence,
    RunFailureMatrix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticsEndpointContract {
    pub id: DiagnosticsEndpointId,
    pub path_template: &'static str,
    pub methods: &'static [&'static str],
    pub allowed_query_keys: &'static [&'static str],
    pub artifact_file: Option<&'static str>,
    pub scope: EndpointScope,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticsPathParams {
    pub run_id: Option<String>,
    pub incident_id: Option<String>,
    pub fp: Option<String>,
    pub artifact_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDiagnosticsEndpoint {
    pub contract: &'static DiagnosticsEndpointContract,
    pub params: DiagnosticsPathParams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForbiddenObservabilityField {
    pub normalized_field: &'static str,
    pub case_id: &'static str,
}

const GET_ONLY: &[&str] = &["GET"];
const NO_QUERY_KEYS: &[&str] = &[];

pub const API_VERSION: u16 = 1;
pub const ALLOWED_INCIDENT_FILTERS: &[&str] = &["severity", "surface_key", "node_id"];

pub const PHASE13_FORBIDDEN_FIELDS: &[&str] = &[
    "preferred_verifier",
    "winning_verifier",
    "trust_rank",
    "verifier_score",
    "trust_score",
    "reliability_index",
    "weighted_authority",
    "correctness_rate",
    "agreement_ratio",
    "node_success_ratio",
    "verifier_reputation",
    "recommended_action",
    "routing_hint",
    "execution_override",
    "retry",
    "override",
    "promote",
    "commit",
    "mitigation",
    "node_priority",
    "verification_weight",
];

pub const ROOT_DIAGNOSTICS_ENDPOINTS: &[DiagnosticsEndpointContract] = &[
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Version,
        path_template: "/diagnostics/version",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Runs,
        path_template: "/diagnostics/runs",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Federation,
        path_template: "/diagnostics/federation",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Context,
        path_template: "/diagnostics/context",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Trust,
        path_template: "/diagnostics/trust",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Parity,
        path_template: "/diagnostics/parity",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_report.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::ParityContextRelation,
        path_template: "/diagnostics/parity/context-relation",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Incidents,
        path_template: "/diagnostics/incidents",
        methods: GET_ONLY,
        allowed_query_keys: ALLOWED_INCIDENT_FILTERS,
        artifact_file: Some("parity_determinism_incidents.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::IncidentById,
        path_template: "/diagnostics/incidents/{incident_id}",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::FingerprintBoundary,
        path_template: "/diagnostics/fingerprints/{fp}",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::ReplicatedBoundary,
        path_template: "/diagnostics/replicated-boundary",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::AuthoritySuppression,
        path_template: "/diagnostics/authority-suppression",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_authority_suppression_report.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::AuthorityTopology,
        path_template: "/diagnostics/authority-topology",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_authority_drift_topology.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Graph,
        path_template: "/diagnostics/graph",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_incident_graph.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Drift,
        path_template: "/diagnostics/drift",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_drift_attribution_report.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::Convergence,
        path_template: "/diagnostics/convergence",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_convergence_report.json"),
        scope: EndpointScope::Root,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::FailureMatrix,
        path_template: "/diagnostics/failure-matrix",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("failure_matrix.json"),
        scope: EndpointScope::Root,
    },
];

pub const RUN_SCOPED_DIAGNOSTICS_ENDPOINTS: &[DiagnosticsEndpointContract] = &[
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunSummary,
        path_template: "/diagnostics/runs/{run_id}",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunArtifactsIndex,
        path_template: "/diagnostics/runs/{run_id}/artifacts",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunArtifactFile,
        path_template: "/diagnostics/runs/{run_id}/artifacts/{artifact_path}",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunFederation,
        path_template: "/diagnostics/runs/{run_id}/federation",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunContext,
        path_template: "/diagnostics/runs/{run_id}/context",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunRegistry,
        path_template: "/diagnostics/runs/{run_id}/registry",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunBoundary,
        path_template: "/diagnostics/runs/{run_id}/boundary",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: None,
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunIncidents,
        path_template: "/diagnostics/runs/{run_id}/incidents",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_determinism_incidents.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunParity,
        path_template: "/diagnostics/runs/{run_id}/parity",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_report.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunAuthoritySuppression,
        path_template: "/diagnostics/runs/{run_id}/authority-suppression",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_authority_suppression_report.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunAuthorityTopology,
        path_template: "/diagnostics/runs/{run_id}/authority-topology",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_authority_drift_topology.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunGraph,
        path_template: "/diagnostics/runs/{run_id}/graph",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_incident_graph.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunDrift,
        path_template: "/diagnostics/runs/{run_id}/drift",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_drift_attribution_report.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunConvergence,
        path_template: "/diagnostics/runs/{run_id}/convergence",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("parity_convergence_report.json"),
        scope: EndpointScope::RunScoped,
    },
    DiagnosticsEndpointContract {
        id: DiagnosticsEndpointId::RunFailureMatrix,
        path_template: "/diagnostics/runs/{run_id}/failure-matrix",
        methods: GET_ONLY,
        allowed_query_keys: NO_QUERY_KEYS,
        artifact_file: Some("failure_matrix.json"),
        scope: EndpointScope::RunScoped,
    },
];

pub const FORBIDDEN_OBSERVABILITY_FIELDS: &[ForbiddenObservabilityField] = &[
    ForbiddenObservabilityField {
        normalized_field: "selectedtruth",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "winningverdict",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "committedcluster",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "acceptedauthority",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "acceptauthority",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "resolvetruth",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "selectwinner",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "elect",
        case_id: "P13-NEG-13",
    },
    ForbiddenObservabilityField {
        normalized_field: "retry",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "override",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "promote",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "commit",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "forceaccept",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "recommendedaction",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "recommendedactions",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "mitigation",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "routinghint",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "nodepriority",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "verificationweight",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "executionoverride",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "quarantine",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "autoquarantine",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "autorecovery",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "suppressnode",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "triggerreplayadmission",
        case_id: "P13-NEG-14",
    },
    ForbiddenObservabilityField {
        normalized_field: "commitclusterstate",
        case_id: "P13-NEG-14",
    },
];

pub fn public_endpoint_declarations() -> Vec<String> {
    let mut declarations = vec!["GET /healthz".to_string()];
    declarations.extend(
        ROOT_DIAGNOSTICS_ENDPOINTS
            .iter()
            .chain(RUN_SCOPED_DIAGNOSTICS_ENDPOINTS.iter())
            .map(|endpoint| format!("GET {}", endpoint.path_template)),
    );
    declarations
}

pub fn root_passthrough_endpoints() -> Vec<&'static DiagnosticsEndpointContract> {
    ROOT_DIAGNOSTICS_ENDPOINTS
        .iter()
        .filter(|endpoint| endpoint.artifact_file.is_some())
        .collect()
}

pub fn run_scoped_passthrough_endpoints() -> Vec<&'static DiagnosticsEndpointContract> {
    RUN_SCOPED_DIAGNOSTICS_ENDPOINTS
        .iter()
        .filter(|endpoint| endpoint.artifact_file.is_some())
        .collect()
}

pub fn materialize_path_template(path_template: &str, run_id: &str) -> String {
    path_template.replace("{run_id}", run_id)
}

pub fn observability_case_for_field(field: &str) -> Option<&'static str> {
    FORBIDDEN_OBSERVABILITY_FIELDS
        .iter()
        .find(|entry| entry.normalized_field == field)
        .map(|entry| entry.case_id)
}

pub fn forbidden_observability_field_tokens() -> Vec<&'static str> {
    FORBIDDEN_OBSERVABILITY_FIELDS
        .iter()
        .map(|entry| entry.normalized_field)
        .collect()
}

pub fn normalize_field_key(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

pub fn scan_forbidden_observability_fields(endpoint: &str, value: &Value) -> Vec<Value> {
    let mut hits = Vec::new();
    scan_forbidden_observability_fields_inner(endpoint, "$", value, &mut hits);
    hits
}

pub fn allowed_query_keys_for_path(path: &str) -> Option<&'static [&'static str]> {
    if path == "/healthz" {
        return Some(NO_QUERY_KEYS);
    }

    if let Some(resolved) = resolve_public_endpoint(path) {
        return Some(resolved.contract.allowed_query_keys);
    }
    None
}

pub fn root_endpoint_contract_for_path(path: &str) -> Option<&'static DiagnosticsEndpointContract> {
    resolve_root_endpoint(path).map(|resolved| resolved.contract)
}

pub fn run_scoped_endpoint_contract_for_path(
    path: &str,
) -> Option<&'static DiagnosticsEndpointContract> {
    resolve_run_scoped_endpoint(path).map(|resolved| resolved.contract)
}

pub fn public_endpoint_contract_for_path(
    path: &str,
) -> Option<&'static DiagnosticsEndpointContract> {
    resolve_public_endpoint(path).map(|resolved| resolved.contract)
}

pub fn resolve_root_endpoint(path: &str) -> Option<ResolvedDiagnosticsEndpoint> {
    ROOT_DIAGNOSTICS_ENDPOINTS
        .iter()
        .find_map(|contract| resolve_endpoint_contract(path, contract))
}

pub fn resolve_run_scoped_endpoint(path: &str) -> Option<ResolvedDiagnosticsEndpoint> {
    RUN_SCOPED_DIAGNOSTICS_ENDPOINTS
        .iter()
        .find_map(|contract| resolve_endpoint_contract(path, contract))
}

pub fn resolve_public_endpoint(path: &str) -> Option<ResolvedDiagnosticsEndpoint> {
    resolve_root_endpoint(path).or_else(|| resolve_run_scoped_endpoint(path))
}

pub fn root_passthrough_contract_for_path(
    path: &str,
) -> Option<&'static DiagnosticsEndpointContract> {
    root_endpoint_contract_for_path(path).filter(|endpoint| {
        endpoint.artifact_file.is_some() && endpoint.allowed_query_keys.is_empty()
    })
}

pub fn run_scoped_passthrough_contract_for_path(
    path: &str,
) -> Option<&'static DiagnosticsEndpointContract> {
    run_scoped_endpoint_contract_for_path(path).filter(|endpoint| endpoint.artifact_file.is_some())
}

fn resolve_endpoint_contract(
    path: &str,
    contract: &'static DiagnosticsEndpointContract,
) -> Option<ResolvedDiagnosticsEndpoint> {
    resolve_path_params(path, contract.path_template)
        .map(|params| ResolvedDiagnosticsEndpoint { contract, params })
}

fn resolve_path_params(path: &str, path_template: &str) -> Option<DiagnosticsPathParams> {
    let path_segments = split_segments(path);
    let template_segments = split_segments(path_template);

    let mut params = DiagnosticsPathParams::default();
    let mut path_index = 0usize;
    let mut template_index = 0usize;

    while template_index < template_segments.len() {
        let template_segment = template_segments[template_index];

        if template_segment == "{artifact_path}" {
            if path_index >= path_segments.len() {
                return None;
            }
            params.artifact_path = Some(path_segments[path_index..].join("/"));
            return (template_index + 1 == template_segments.len()).then_some(params);
        }

        if path_index >= path_segments.len() {
            return None;
        }

        let path_segment = path_segments[path_index];
        let is_placeholder = template_segment.starts_with('{') && template_segment.ends_with('}');
        if is_placeholder {
            if !capture_path_param(&mut params, template_segment, path_segment) {
                return None;
            }
        } else if template_segment != path_segment {
            return None;
        }

        template_index += 1;
        path_index += 1;
    }

    (path_index == path_segments.len()).then_some(params)
}

fn capture_path_param(params: &mut DiagnosticsPathParams, key: &str, value: &str) -> bool {
    match key {
        "{run_id}" => params.run_id = Some(value.to_string()),
        "{incident_id}" => params.incident_id = Some(value.to_string()),
        "{fp}" => params.fp = Some(value.to_string()),
        _ => return false,
    }
    true
}

fn split_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{resolve_public_endpoint, DiagnosticsEndpointId};

    #[test]
    fn resolver_captures_root_diagnostics_params() {
        let incident = resolve_public_endpoint("/diagnostics/incidents/sha256:incident")
            .expect("incident endpoint should resolve");
        assert_eq!(incident.contract.id, DiagnosticsEndpointId::IncidentById);
        assert_eq!(
            incident.params.incident_id.as_deref(),
            Some("sha256:incident")
        );

        let fingerprint = resolve_public_endpoint(
            "/diagnostics/fingerprints/sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("fingerprint endpoint should resolve");
        assert_eq!(
            fingerprint.contract.id,
            DiagnosticsEndpointId::FingerprintBoundary
        );
        assert_eq!(
            fingerprint.params.fp.as_deref(),
            Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
    }

    #[test]
    fn resolver_captures_run_scoped_artifact_path() {
        let resolved = resolve_public_endpoint(
            "/diagnostics/runs/run-20260310-1/artifacts/receipts/verification_receipt.json",
        )
        .expect("artifact endpoint should resolve");
        assert_eq!(resolved.contract.id, DiagnosticsEndpointId::RunArtifactFile);
        assert_eq!(resolved.params.run_id.as_deref(), Some("run-20260310-1"));
        assert_eq!(
            resolved.params.artifact_path.as_deref(),
            Some("receipts/verification_receipt.json")
        );
    }
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
use serde_json::{json, Value};
