use serde_json::{json, Value};
use std::collections::BTreeSet;

use crate::api_contract::{
    public_endpoint_contract_for_path, DiagnosticsEndpointId, ROOT_DIAGNOSTICS_ENDPOINTS,
    RUN_SCOPED_DIAGNOSTICS_ENDPOINTS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaValueKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCoverage {
    None,
    RootOnly,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseMode {
    Computed,
    ArtifactFiltered,
    ArtifactJsonPassthrough,
    ArtifactFilePassthrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaField {
    pub name: &'static str,
    pub kind: SchemaValueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointSchema {
    pub endpoint_id: DiagnosticsEndpointId,
    pub root_kind: SchemaValueKind,
    pub required_fields: &'static [SchemaField],
    pub optional_fields: &'static [SchemaField],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaValidationError {
    RootKindMismatch {
        expected: SchemaValueKind,
    },
    MissingRequiredField {
        field: &'static str,
    },
    FieldTypeMismatch {
        field: &'static str,
        expected: SchemaValueKind,
    },
    InvalidFieldValue {
        field: &'static str,
    },
}

impl SchemaValidationError {
    pub fn reason_code(self) -> &'static str {
        "diagnostics_schema_contract_violation"
    }
}

const EMPTY_FIELDS: &[SchemaField] = &[];

const VERSION_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "api_version",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "service",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "contract",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "invariants",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "endpoints",
        kind: SchemaValueKind::Array,
    },
];

const RUNS_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "runs",
        kind: SchemaValueKind::Array,
    },
];

const FEDERATION_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "verifier_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "verifiers",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "runs",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "fingerprints",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "total_ledger_entries",
        kind: SchemaValueKind::Number,
    },
];

const CONTEXT_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "context_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "contexts",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "runs",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "context_drift",
        kind: SchemaValueKind::Object,
    },
];

const TRUST_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "producer_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "producers",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "runs",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "registry_version_distribution",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "registry_hash_consistency",
        kind: SchemaValueKind::Object,
    },
];

const PARITY_CONTEXT_RELATION_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "pair_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "context_relation_summary",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "pairs",
        kind: SchemaValueKind::Array,
    },
];

const INCIDENTS_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "determinism_incident_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "incidents",
        kind: SchemaValueKind::Array,
    },
];

const INCIDENTS_OPTIONAL_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "gate",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "mode",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "status",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "node_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "surface_partition_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "severity_counts",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "false_determinism_guard_active",
        kind: SchemaValueKind::Boolean,
    },
    SchemaField {
        name: "suppressed_incident_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "suppressed_incidents",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "suppression_reason_counts",
        kind: SchemaValueKind::Object,
    },
];

const INCIDENT_BY_ID_REQUIRED_FIELDS: &[SchemaField] = &[SchemaField {
    name: "incident_id",
    kind: SchemaValueKind::String,
}];

const INCIDENT_BY_ID_OPTIONAL_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "surface_key",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "severity",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "nodes",
        kind: SchemaValueKind::Array,
    },
];

const FINGERPRINT_BOUNDARY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "request_fingerprint",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "run_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "run_ids",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "context_ids",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "context_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "verdict_consistency",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "context_hash_consistency",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "registry_hash_consistency",
        kind: SchemaValueKind::Object,
    },
];

const REPLICATED_BOUNDARY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "boundary_status",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "invariants",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "disallowed_routes",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "diagnostics_routes_allowed",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "phase",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "note",
        kind: SchemaValueKind::String,
    },
];

const RUN_SUMMARY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "artifacts",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "artifact_paths",
        kind: SchemaValueKind::Array,
    },
];

const RUN_ARTIFACTS_INDEX_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "artifact_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "artifacts",
        kind: SchemaValueKind::Array,
    },
];

const RUN_FEDERATION_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "verifier_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "observed_verifiers",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "authority_chain_distribution",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "execution_cluster_distribution",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "missing_execution_cluster_entry_count",
        kind: SchemaValueKind::Number,
    },
];

const RUN_CONTEXT_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "source_artifact_paths",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "declared_context",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "material_binding_status",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "observed_context_id_sources",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "observed_context_ref_sources",
        kind: SchemaValueKind::Array,
    },
];

const RUN_REGISTRY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "source_artifact_path",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "declared_registry_snapshot_hash",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "declared_registry_entry_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "context_binding_status",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "observed_registry_hash_sources",
        kind: SchemaValueKind::Array,
    },
];

const RUN_BOUNDARY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "request_fingerprint",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "peer_run_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "peer_run_ids",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "verdict_consistency",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "context_hash_consistency",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "registry_hash_consistency",
        kind: SchemaValueKind::Object,
    },
];

const RUN_GRAPH_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "graph_version",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "authority",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "env_hash",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "status",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "provenance",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "graph",
        kind: SchemaValueKind::Object,
    },
];

const PUBLIC_ENDPOINT_SCHEMAS: &[EndpointSchema] = &[
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Version,
        root_kind: SchemaValueKind::Object,
        required_fields: VERSION_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Runs,
        root_kind: SchemaValueKind::Object,
        required_fields: RUNS_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Federation,
        root_kind: SchemaValueKind::Object,
        required_fields: FEDERATION_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Context,
        root_kind: SchemaValueKind::Object,
        required_fields: CONTEXT_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Trust,
        root_kind: SchemaValueKind::Object,
        required_fields: TRUST_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::ParityContextRelation,
        root_kind: SchemaValueKind::Object,
        required_fields: PARITY_CONTEXT_RELATION_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Incidents,
        root_kind: SchemaValueKind::Object,
        required_fields: INCIDENTS_REQUIRED_FIELDS,
        optional_fields: INCIDENTS_OPTIONAL_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::IncidentById,
        root_kind: SchemaValueKind::Object,
        required_fields: INCIDENT_BY_ID_REQUIRED_FIELDS,
        optional_fields: INCIDENT_BY_ID_OPTIONAL_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::FingerprintBoundary,
        root_kind: SchemaValueKind::Object,
        required_fields: FINGERPRINT_BOUNDARY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::ReplicatedBoundary,
        root_kind: SchemaValueKind::Object,
        required_fields: REPLICATED_BOUNDARY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunSummary,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_SUMMARY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunArtifactsIndex,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_ARTIFACTS_INDEX_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunFederation,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_FEDERATION_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunContext,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_CONTEXT_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunRegistry,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_REGISTRY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunBoundary,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_BOUNDARY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunGraph,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_GRAPH_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
];

pub fn schema_for_endpoint_id(
    endpoint_id: DiagnosticsEndpointId,
) -> Option<&'static EndpointSchema> {
    PUBLIC_ENDPOINT_SCHEMAS
        .iter()
        .find(|schema| schema.endpoint_id == endpoint_id)
}

pub fn schema_coverage_for_endpoint_id(endpoint_id: DiagnosticsEndpointId) -> SchemaCoverage {
    match endpoint_id {
        DiagnosticsEndpointId::Version
        | DiagnosticsEndpointId::Runs
        | DiagnosticsEndpointId::Federation
        | DiagnosticsEndpointId::Context
        | DiagnosticsEndpointId::Trust
        | DiagnosticsEndpointId::ParityContextRelation
        | DiagnosticsEndpointId::Incidents
        | DiagnosticsEndpointId::IncidentById
        | DiagnosticsEndpointId::FingerprintBoundary
        | DiagnosticsEndpointId::ReplicatedBoundary
        | DiagnosticsEndpointId::RunSummary
        | DiagnosticsEndpointId::RunArtifactsIndex
        | DiagnosticsEndpointId::RunFederation
        | DiagnosticsEndpointId::RunContext
        | DiagnosticsEndpointId::RunRegistry
        | DiagnosticsEndpointId::RunBoundary
        | DiagnosticsEndpointId::RunGraph => SchemaCoverage::Full,
        DiagnosticsEndpointId::Parity
        | DiagnosticsEndpointId::AuthoritySuppression
        | DiagnosticsEndpointId::AuthorityTopology
        | DiagnosticsEndpointId::Graph
        | DiagnosticsEndpointId::Drift
        | DiagnosticsEndpointId::Convergence
        | DiagnosticsEndpointId::FailureMatrix
        | DiagnosticsEndpointId::RunArtifactFile
        | DiagnosticsEndpointId::RunIncidents
        | DiagnosticsEndpointId::RunParity
        | DiagnosticsEndpointId::RunAuthoritySuppression
        | DiagnosticsEndpointId::RunAuthorityTopology
        | DiagnosticsEndpointId::RunDrift
        | DiagnosticsEndpointId::RunConvergence
        | DiagnosticsEndpointId::RunFailureMatrix => SchemaCoverage::None,
    }
}

pub fn response_mode_for_endpoint_id(endpoint_id: DiagnosticsEndpointId) -> ResponseMode {
    match endpoint_id {
        DiagnosticsEndpointId::Parity
        | DiagnosticsEndpointId::AuthoritySuppression
        | DiagnosticsEndpointId::AuthorityTopology
        | DiagnosticsEndpointId::Graph
        | DiagnosticsEndpointId::Drift
        | DiagnosticsEndpointId::Convergence
        | DiagnosticsEndpointId::FailureMatrix
        | DiagnosticsEndpointId::RunIncidents
        | DiagnosticsEndpointId::RunParity
        | DiagnosticsEndpointId::RunAuthoritySuppression
        | DiagnosticsEndpointId::RunAuthorityTopology
        | DiagnosticsEndpointId::RunDrift
        | DiagnosticsEndpointId::RunConvergence
        | DiagnosticsEndpointId::RunFailureMatrix => ResponseMode::ArtifactJsonPassthrough,
        DiagnosticsEndpointId::RunGraph => ResponseMode::ArtifactFiltered,
        DiagnosticsEndpointId::RunArtifactFile => ResponseMode::ArtifactFilePassthrough,
        DiagnosticsEndpointId::Incidents => ResponseMode::ArtifactFiltered,
        _ => ResponseMode::Computed,
    }
}

pub fn schema_for_path(path: &str) -> Option<&'static EndpointSchema> {
    public_endpoint_contract_for_path(path).and_then(|contract| schema_for_endpoint_id(contract.id))
}

pub fn schema_coverage_for_path(path: &str) -> Option<SchemaCoverage> {
    public_endpoint_contract_for_path(path)
        .map(|contract| schema_coverage_for_endpoint_id(contract.id))
}

pub fn validate_response_schema_for_path(
    path: &str,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let Some(contract) = public_endpoint_contract_for_path(path) else {
        return Ok(());
    };
    let coverage = schema_coverage_for_endpoint_id(contract.id);
    match coverage {
        SchemaCoverage::None => Ok(()),
        SchemaCoverage::RootOnly => {
            let schema = schema_for_endpoint_id(contract.id)
                .expect("schema coverage root_only requires schema declaration");
            validate_root_kind_only(schema, value)
        }
        SchemaCoverage::Full => {
            let schema = schema_for_endpoint_id(contract.id)
                .expect("schema coverage full requires schema declaration");
            validate_response_schema(schema, value)?;
            validate_endpoint_specific_contract(contract.id, value)
        }
    }
}

pub fn public_schema_declarations() -> Vec<Value> {
    ROOT_DIAGNOSTICS_ENDPOINTS
        .iter()
        .chain(RUN_SCOPED_DIAGNOSTICS_ENDPOINTS.iter())
        .map(|contract| {
            let coverage = schema_coverage_for_endpoint_id(contract.id);
            let schema = schema_for_endpoint_id(contract.id);
            let response_mode = response_mode_for_endpoint_id(contract.id);
            json!({
                "path_template": contract.path_template,
                "scope": match contract.scope {
                    crate::api_contract::EndpointScope::Root => "root",
                    crate::api_contract::EndpointScope::RunScoped => "run",
                },
                "artifact_backed": contract.artifact_file.is_some(),
                "response_mode": response_mode_name(response_mode),
                "coverage": schema_coverage_name(coverage),
                "schema_present": schema.is_some(),
                "schema_enforcement_active": coverage != SchemaCoverage::None,
                "root_kind": schema.map(|schema| schema_value_kind_name(schema.root_kind)),
                "required_fields": schema
                    .map(|schema| {
                        schema
                            .required_fields
                            .iter()
                            .map(|field| json!({
                                "name": field.name,
                                "kind": schema_value_kind_name(field.kind),
                            }))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                "optional_fields": schema
                    .map(|schema| {
                        schema
                            .optional_fields
                            .iter()
                            .map(|field| json!({
                                "name": field.name,
                                "kind": schema_value_kind_name(field.kind),
                            }))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect()
}

fn validate_root_kind_only(
    schema: &EndpointSchema,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    if !schema.root_kind.matches(value) {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: schema.root_kind,
        });
    }
    Ok(())
}

fn validate_response_schema(
    schema: &EndpointSchema,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    if !schema.root_kind.matches(value) {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: schema.root_kind,
        });
    }

    let Some(map) = value.as_object() else {
        return Ok(());
    };

    for field in schema.required_fields {
        let Some(field_value) = map.get(field.name) else {
            return Err(SchemaValidationError::MissingRequiredField { field: field.name });
        };
        if !field.kind.matches(field_value) {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: field.name,
                expected: field.kind,
            });
        }
    }

    for field in schema.optional_fields {
        if let Some(field_value) = map.get(field.name) {
            if !field.kind.matches(field_value) {
                return Err(SchemaValidationError::FieldTypeMismatch {
                    field: field.name,
                    expected: field.kind,
                });
            }
        }
    }

    Ok(())
}

fn validate_endpoint_specific_contract(
    endpoint_id: DiagnosticsEndpointId,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    match endpoint_id {
        DiagnosticsEndpointId::RunGraph => validate_phase14_graph_contract_v1(value),
        _ => Ok(()),
    }
}

pub fn validate_phase14_graph_contract_v1(
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let Some(root) = value.as_object() else {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: SchemaValueKind::Object,
        });
    };

    let graph_version = require_non_empty_string(root, "graph_version")?;
    if graph_version != "v1" {
        return Err(SchemaValidationError::InvalidFieldValue {
            field: "graph_version",
        });
    }
    require_non_empty_string(root, "authority")?;
    require_non_empty_string(root, "env_hash")?;

    let provenance = require_object(root, "provenance")?;
    require_non_empty_string(provenance, "artifact_set_hash")?;
    let source_runs = require_array(provenance, "source_runs")?;
    validate_source_runs(source_runs)?;

    let graph = require_object(root, "graph")?;
    let nodes = require_array(graph, "nodes")?;
    let edges = require_array(graph, "edges")?;
    let incidents = require_array(graph, "incidents")?;

    require_exact_count(graph, "node_count", nodes.len())?;
    require_exact_count(graph, "edge_count", edges.len())?;
    require_exact_count(graph, "incident_count", incidents.len())?;

    let incident_ids = collect_incident_ids(incidents)?;
    validate_graph_nodes(nodes)?;
    validate_graph_edges(edges, &incident_ids)?;

    Ok(())
}

fn require_non_empty_string<'a>(
    map: &'a serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<&'a str, SchemaValidationError> {
    let value = map
        .get(field)
        .ok_or(SchemaValidationError::MissingRequiredField { field })?;
    let Some(value) = value.as_str() else {
        return Err(SchemaValidationError::FieldTypeMismatch {
            field,
            expected: SchemaValueKind::String,
        });
    };
    if value.trim().is_empty() {
        return Err(SchemaValidationError::InvalidFieldValue { field });
    }
    Ok(value)
}

fn require_object<'a>(
    map: &'a serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<&'a serde_json::Map<String, Value>, SchemaValidationError> {
    let value = map
        .get(field)
        .ok_or(SchemaValidationError::MissingRequiredField { field })?;
    value.as_object().ok_or(SchemaValidationError::FieldTypeMismatch {
        field,
        expected: SchemaValueKind::Object,
    })
}

fn require_array<'a>(
    map: &'a serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<&'a Vec<Value>, SchemaValidationError> {
    let value = map
        .get(field)
        .ok_or(SchemaValidationError::MissingRequiredField { field })?;
    value.as_array().ok_or(SchemaValidationError::FieldTypeMismatch {
        field,
        expected: SchemaValueKind::Array,
    })
}

fn require_exact_count(
    map: &serde_json::Map<String, Value>,
    field: &'static str,
    expected_len: usize,
) -> Result<(), SchemaValidationError> {
    let value = map
        .get(field)
        .ok_or(SchemaValidationError::MissingRequiredField { field })?;
    let Some(value) = value.as_u64() else {
        return Err(SchemaValidationError::FieldTypeMismatch {
            field,
            expected: SchemaValueKind::Number,
        });
    };
    if value as usize != expected_len {
        return Err(SchemaValidationError::InvalidFieldValue { field });
    }
    Ok(())
}

fn validate_source_runs(source_runs: &[Value]) -> Result<(), SchemaValidationError> {
    if source_runs.is_empty() {
        return Err(SchemaValidationError::InvalidFieldValue {
            field: "source_runs",
        });
    }
    let mut previous: Option<&str> = None;
    let mut seen = BTreeSet::new();
    for value in source_runs {
        let Some(value) = value.as_str() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "source_runs[]",
                expected: SchemaValueKind::String,
            });
        };
        if value.trim().is_empty() {
            return Err(SchemaValidationError::InvalidFieldValue {
                field: "source_runs",
            });
        }
        if let Some(previous_value) = previous {
            if value <= previous_value {
                return Err(SchemaValidationError::InvalidFieldValue {
                    field: "source_runs",
                });
            }
        }
        if !seen.insert(value) {
            return Err(SchemaValidationError::InvalidFieldValue {
                field: "source_runs",
            });
        }
        previous = Some(value);
    }
    Ok(())
}

fn collect_incident_ids(
    incidents: &[Value],
) -> Result<BTreeSet<String>, SchemaValidationError> {
    let mut incident_ids = BTreeSet::new();
    for incident in incidents {
        let Some(incident) = incident.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "incidents[]",
                expected: SchemaValueKind::Object,
            });
        };
        let incident_id = require_non_empty_string(incident, "incident_id")?;
        if !incident_ids.insert(incident_id.to_string()) {
            return Err(SchemaValidationError::InvalidFieldValue {
                field: "incident_id",
            });
        }
    }
    Ok(incident_ids)
}

fn validate_graph_nodes(nodes: &[Value]) -> Result<(), SchemaValidationError> {
    for node in nodes {
        let Some(node) = node.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "nodes[]",
                expected: SchemaValueKind::Object,
            });
        };
        require_non_empty_string(node, "id")?;
        require_non_empty_string(node, "node_fingerprint")?;
    }
    Ok(())
}

fn validate_graph_edges(
    edges: &[Value],
    incident_ids: &BTreeSet<String>,
) -> Result<(), SchemaValidationError> {
    for edge in edges {
        let Some(edge) = edge.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "edges[]",
                expected: SchemaValueKind::Object,
            });
        };
        require_non_empty_string(edge, "from")?;
        require_non_empty_string(edge, "to")?;
        let edge_type = require_non_empty_string(edge, "edge_type")?;
        match edge_type {
            "same_outcome" => {}
            "incident" => {
                let incident_id = require_non_empty_string(edge, "incident_id")?;
                if !incident_ids.contains(incident_id) {
                    return Err(SchemaValidationError::InvalidFieldValue {
                        field: "incident_id",
                    });
                }
            }
            _ => {
                return Err(SchemaValidationError::InvalidFieldValue {
                    field: "edge_type",
                });
            }
        }
    }
    Ok(())
}

fn schema_value_kind_name(kind: SchemaValueKind) -> &'static str {
    match kind {
        SchemaValueKind::Object => "object",
        SchemaValueKind::Array => "array",
        SchemaValueKind::String => "string",
        SchemaValueKind::Number => "number",
        SchemaValueKind::Boolean => "boolean",
    }
}

fn schema_coverage_name(coverage: SchemaCoverage) -> &'static str {
    match coverage {
        SchemaCoverage::None => "none",
        SchemaCoverage::RootOnly => "root_only",
        SchemaCoverage::Full => "full",
    }
}

fn response_mode_name(mode: ResponseMode) -> &'static str {
    match mode {
        ResponseMode::Computed => "computed",
        ResponseMode::ArtifactFiltered => "artifact_filtered",
        ResponseMode::ArtifactJsonPassthrough => "artifact_json_passthrough",
        ResponseMode::ArtifactFilePassthrough => "artifact_file_passthrough",
    }
}

impl SchemaValueKind {
    fn matches(self, value: &Value) -> bool {
        match self {
            SchemaValueKind::Object => value.is_object(),
            SchemaValueKind::Array => value.is_array(),
            SchemaValueKind::String => value.is_string(),
            SchemaValueKind::Number => value.is_number(),
            SchemaValueKind::Boolean => value.is_boolean(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        schema_coverage_for_endpoint_id, schema_for_path, validate_response_schema_for_path,
        SchemaCoverage, SchemaValidationError, SchemaValueKind,
    };
    use crate::api_contract::DiagnosticsEndpointId;
    use serde_json::json;

    #[test]
    fn version_schema_rejects_missing_required_field() {
        let error = validate_response_schema_for_path(
            "/diagnostics/version",
            &json!({
                "api_version": 1,
                "service": "proofd",
                "contract": "read-only diagnostics surface",
                "invariants": [],
            }),
        )
        .expect_err("missing endpoints must fail");
        assert_eq!(
            error,
            SchemaValidationError::MissingRequiredField { field: "endpoints" }
        );
    }

    #[test]
    fn version_schema_allows_unknown_fields() {
        validate_response_schema_for_path(
            "/diagnostics/version",
            &json!({
                "api_version": 1,
                "service": "proofd",
                "contract": "read-only diagnostics surface",
                "invariants": [],
                "endpoints": [],
                "extra": "allowed"
            }),
        )
        .expect("unknown fields must remain forward-compatible");
    }

    #[test]
    fn schema_lookup_covers_run_boundary() {
        let schema = schema_for_path("/diagnostics/runs/run-a/boundary")
            .expect("run boundary schema must exist");
        assert_eq!(schema.root_kind, SchemaValueKind::Object);
        assert!(schema
            .required_fields
            .iter()
            .any(|field| field.name == "request_fingerprint"));
    }

    #[test]
    fn coverage_marks_parity_passthrough_as_none() {
        assert_eq!(
            schema_coverage_for_endpoint_id(DiagnosticsEndpointId::Parity),
            SchemaCoverage::None
        );
    }

    #[test]
    fn coverage_marks_run_graph_as_full() {
        assert_eq!(
            schema_coverage_for_endpoint_id(DiagnosticsEndpointId::RunGraph),
            SchemaCoverage::Full
        );
    }

    #[test]
    fn run_graph_schema_rejects_missing_graph_version() {
        let error = validate_response_schema_for_path(
            "/diagnostics/runs/run-a/graph",
            &json!({
                "authority": "proof-verifier-cross-node-parity",
                "env_hash": "sha256:env",
                "status": "PASS",
                "provenance": {
                    "artifact_set_hash": "sha256:set",
                    "source_runs": ["run-a"]
                },
                "graph": {
                    "node_count": 0,
                    "edge_count": 0,
                    "incident_count": 0,
                    "nodes": [],
                    "edges": [],
                    "incidents": []
                }
            }),
        )
        .expect_err("missing graph_version must fail");
        assert_eq!(
            error,
            SchemaValidationError::MissingRequiredField {
                field: "graph_version"
            }
        );
    }

    #[test]
    fn run_graph_schema_rejects_unsorted_source_runs() {
        let error = validate_response_schema_for_path(
            "/diagnostics/runs/run-a/graph",
            &json!({
                "graph_version": "v1",
                "authority": "proof-verifier-cross-node-parity",
                "env_hash": "sha256:env",
                "status": "PASS",
                "provenance": {
                    "artifact_set_hash": "sha256:set",
                    "source_runs": ["run-b", "run-a"]
                },
                "graph": {
                    "node_count": 0,
                    "edge_count": 0,
                    "incident_count": 0,
                    "nodes": [],
                    "edges": [],
                    "incidents": []
                }
            }),
        )
        .expect_err("unsorted source_runs must fail");
        assert_eq!(
            error,
            SchemaValidationError::InvalidFieldValue {
                field: "source_runs"
            }
        );
    }
}
