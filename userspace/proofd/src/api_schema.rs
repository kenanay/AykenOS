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

const ROOT_GRAPH_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "graph_origin",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "authority_classification",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "aggregation_mode",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "partition_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "partitions",
        kind: SchemaValueKind::Array,
    },
];

const GRAPH_OVERLAY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "graph_origin",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "authority_classification",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "aggregation_mode",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "partition_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "agreement_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "conflict_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "island_count",
        kind: SchemaValueKind::Number,
    },
    SchemaField {
        name: "agreements",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "conflicts",
        kind: SchemaValueKind::Array,
    },
    SchemaField {
        name: "islands",
        kind: SchemaValueKind::Array,
    },
];

const SUMMARY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "summary_origin",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "authority_classification",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "display_mode",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "epistemic_boundary",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "snapshot",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "overlay",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "incidents",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "explanation",
        kind: SchemaValueKind::Array,
    },
];

const RUN_SCOPED_SUMMARY_REQUIRED_FIELDS: &[SchemaField] = &[
    SchemaField {
        name: "summary_origin",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "authority_classification",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "display_mode",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "epistemic_boundary",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "run_id",
        kind: SchemaValueKind::String,
    },
    SchemaField {
        name: "snapshot",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "incidents",
        kind: SchemaValueKind::Object,
    },
    SchemaField {
        name: "explanation",
        kind: SchemaValueKind::Array,
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
        endpoint_id: DiagnosticsEndpointId::Graph,
        root_kind: SchemaValueKind::Object,
        required_fields: ROOT_GRAPH_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::GraphOverlay,
        root_kind: SchemaValueKind::Object,
        required_fields: GRAPH_OVERLAY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::Summary,
        root_kind: SchemaValueKind::Object,
        required_fields: SUMMARY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunSummary,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_SUMMARY_REQUIRED_FIELDS,
        optional_fields: EMPTY_FIELDS,
    },
    EndpointSchema {
        endpoint_id: DiagnosticsEndpointId::RunScopedSummary,
        root_kind: SchemaValueKind::Object,
        required_fields: RUN_SCOPED_SUMMARY_REQUIRED_FIELDS,
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
        | DiagnosticsEndpointId::Graph
        | DiagnosticsEndpointId::GraphOverlay
        | DiagnosticsEndpointId::Summary
        | DiagnosticsEndpointId::RunSummary
        | DiagnosticsEndpointId::RunScopedSummary
        | DiagnosticsEndpointId::RunArtifactsIndex
        | DiagnosticsEndpointId::RunFederation
        | DiagnosticsEndpointId::RunContext
        | DiagnosticsEndpointId::RunRegistry
        | DiagnosticsEndpointId::RunBoundary
        | DiagnosticsEndpointId::RunGraph => SchemaCoverage::Full,
        DiagnosticsEndpointId::Parity
        | DiagnosticsEndpointId::AuthoritySuppression
        | DiagnosticsEndpointId::AuthorityTopology
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
        DiagnosticsEndpointId::Graph => validate_root_graph_contract_v1(value),
        DiagnosticsEndpointId::GraphOverlay => validate_root_graph_overlay_contract_v1(value),
        DiagnosticsEndpointId::Summary => validate_observability_summary_contract_v1(value),
        DiagnosticsEndpointId::RunScopedSummary => {
            validate_run_scoped_observability_summary_contract_v1(value)
        }
        _ => Ok(()),
    }
}

pub fn validate_observability_summary_contract_v1(
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let Some(root) = value.as_object() else {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: SchemaValueKind::Object,
        });
    };

    validate_summary_epistemic_boundary(root)?;
    let snapshot = require_object(root, "snapshot")?;
    require_number_field(snapshot, "partition_count")?;
    require_number_field(snapshot, "total_nodes")?;
    require_number_field(snapshot, "total_incidents")?;

    let overlay = require_object(root, "overlay")?;
    require_number_field(overlay, "agreements")?;
    require_number_field(overlay, "conflicts")?;
    require_number_field(overlay, "islands")?;

    let incidents = require_object(root, "incidents")?;
    validate_numeric_object_values(incidents, "incidents")?;
    validate_explanation_array(root, "explanation")?;

    Ok(())
}

pub fn validate_run_scoped_observability_summary_contract_v1(
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let Some(root) = value.as_object() else {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: SchemaValueKind::Object,
        });
    };

    validate_summary_epistemic_boundary(root)?;
    require_non_empty_string(root, "run_id")?;

    let snapshot = require_object(root, "snapshot")?;
    require_number_field(snapshot, "node_count")?;
    require_number_field(snapshot, "incident_count")?;

    let incidents = require_object(root, "incidents")?;
    validate_numeric_object_values(incidents, "incidents")?;
    validate_explanation_array(root, "explanation")?;

    Ok(())
}

pub fn validate_phase14_graph_contract_v1(value: &Value) -> Result<(), SchemaValidationError> {
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
    validate_sorted_unique_string_array(source_runs, "source_runs", true)?;

    let graph = require_object(root, "graph")?;
    validate_phase14_graph_payload_v1(graph)?;

    Ok(())
}

pub fn validate_root_graph_contract_v1(value: &Value) -> Result<(), SchemaValidationError> {
    let Some(root) = value.as_object() else {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: SchemaValueKind::Object,
        });
    };

    require_exact_string(root, "graph_origin", "derived")?;
    require_exact_string(root, "authority_classification", "non_authoritative")?;
    require_exact_string(root, "aggregation_mode", "overlay_only")?;

    let partitions = require_array(root, "partitions")?;
    require_exact_count(root, "partition_count", partitions.len())?;
    validate_graph_partition_entries(partitions)?;

    Ok(())
}

pub fn validate_root_graph_overlay_contract_v1(value: &Value) -> Result<(), SchemaValidationError> {
    let Some(root) = value.as_object() else {
        return Err(SchemaValidationError::RootKindMismatch {
            expected: SchemaValueKind::Object,
        });
    };

    require_exact_string(root, "graph_origin", "derived")?;
    require_exact_string(root, "authority_classification", "non_authoritative")?;
    require_exact_string(root, "aggregation_mode", "overlay_only")?;

    let agreements = require_array(root, "agreements")?;
    let conflicts = require_array(root, "conflicts")?;
    let islands = require_array(root, "islands")?;

    require_exact_count(root, "agreement_count", agreements.len())?;
    require_exact_count(root, "conflict_count", conflicts.len())?;
    require_exact_count(root, "island_count", islands.len())?;

    validate_graph_overlay_agreements(agreements)?;
    validate_graph_overlay_conflicts(conflicts)?;
    validate_graph_overlay_islands(islands)?;

    Ok(())
}

fn validate_graph_partition_entries(partitions: &[Value]) -> Result<(), SchemaValidationError> {
    let mut partition_ids = BTreeSet::new();
    for partition in partitions {
        let Some(partition) = partition.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "partitions[]",
                expected: SchemaValueKind::Object,
            });
        };
        let partition_id = require_non_empty_string(partition, "partition_id")?;
        if !partition_ids.insert(partition_id.to_string()) {
            return Err(SchemaValidationError::InvalidFieldValue {
                field: "partition_id",
            });
        }
        let partition_key = require_object(partition, "partition_key")?;
        require_exact_string(partition_key, "graph_version", "v1")?;
        require_non_empty_string(partition_key, "authority")?;
        require_non_empty_string(partition_key, "env_hash")?;
        require_non_empty_string(partition_key, "artifact_set_hash")?;

        let run_ids = require_array(partition, "run_ids")?;
        validate_sorted_unique_string_array(run_ids, "run_ids", true)?;
        require_exact_count(partition, "run_count", run_ids.len())?;

        let source_runs = require_array(partition, "source_runs")?;
        validate_sorted_unique_string_array(source_runs, "source_runs", true)?;

        let graph = require_object(partition, "graph")?;
        validate_phase14_graph_payload_v1(graph)?;
    }

    Ok(())
}

fn validate_phase14_graph_payload_v1(
    graph: &serde_json::Map<String, Value>,
) -> Result<(), SchemaValidationError> {
    let nodes = require_array(graph, "nodes")?;
    let edges = require_array(graph, "edges")?;
    let incidents = require_array(graph, "incidents")?;

    require_exact_count(graph, "node_count", nodes.len())?;
    require_exact_count(graph, "edge_count", edges.len())?;
    require_exact_count(graph, "incident_count", incidents.len())?;

    let node_ids = validate_graph_nodes(nodes)?;
    let incident_ids = validate_graph_incidents(incidents, &node_ids)?;
    validate_graph_edges(edges, &node_ids, &incident_ids)?;

    Ok(())
}

fn validate_graph_overlay_agreements(agreements: &[Value]) -> Result<(), SchemaValidationError> {
    for agreement in agreements {
        let Some(agreement) = agreement.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "agreements[]",
                expected: SchemaValueKind::Object,
            });
        };
        require_non_empty_string(agreement, "node_fingerprint")?;
        let verdict = require_non_empty_string(agreement, "verdict")?;
        if !is_valid_graph_verdict_label(verdict) {
            return Err(SchemaValidationError::InvalidFieldValue { field: "verdict" });
        }
        let partitions = require_array(agreement, "partitions")?;
        validate_sorted_unique_string_array(partitions, "partitions", true)?;
        require_exact_count(agreement, "partition_count", partitions.len())?;
    }
    Ok(())
}

fn validate_graph_overlay_conflicts(conflicts: &[Value]) -> Result<(), SchemaValidationError> {
    for conflict in conflicts {
        let Some(conflict) = conflict.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "conflicts[]",
                expected: SchemaValueKind::Object,
            });
        };
        require_non_empty_string(conflict, "node_fingerprint")?;
        let partitions = require_array(conflict, "partitions")?;
        validate_sorted_unique_string_array(partitions, "partitions", true)?;
        require_exact_count(conflict, "partition_count", partitions.len())?;

        let observed_verdicts = require_array(conflict, "observed_verdicts")?;
        validate_sorted_unique_string_array(observed_verdicts, "observed_verdicts", true)?;
        if observed_verdicts.len() < 2 {
            return Err(SchemaValidationError::InvalidFieldValue {
                field: "observed_verdicts",
            });
        }
        for verdict in observed_verdicts {
            let verdict = verdict
                .as_str()
                .ok_or(SchemaValidationError::FieldTypeMismatch {
                    field: "observed_verdicts[]",
                    expected: SchemaValueKind::String,
                })?;
            if !is_valid_graph_verdict_label(verdict) {
                return Err(SchemaValidationError::InvalidFieldValue {
                    field: "observed_verdicts",
                });
            }
        }
    }
    Ok(())
}

fn validate_graph_overlay_islands(islands: &[Value]) -> Result<(), SchemaValidationError> {
    let mut partition_ids = BTreeSet::new();
    for island in islands {
        let Some(island) = island.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "islands[]",
                expected: SchemaValueKind::Object,
            });
        };
        let partition_id = require_non_empty_string(island, "partition_id")?;
        if !partition_ids.insert(partition_id.to_string()) {
            return Err(SchemaValidationError::InvalidFieldValue {
                field: "partition_id",
            });
        }
        require_number_field(island, "run_count")?;
        require_number_field(island, "node_count")?;
        require_number_field(island, "edge_count")?;
        require_number_field(island, "incident_count")?;
    }
    Ok(())
}

fn require_exact_string(
    map: &serde_json::Map<String, Value>,
    field: &'static str,
    expected_value: &'static str,
) -> Result<(), SchemaValidationError> {
    let value = require_non_empty_string(map, field)?;
    if value != expected_value {
        return Err(SchemaValidationError::InvalidFieldValue { field });
    }
    Ok(())
}

fn require_exact_bool(
    map: &serde_json::Map<String, Value>,
    field: &'static str,
    expected_value: bool,
) -> Result<(), SchemaValidationError> {
    let value = map
        .get(field)
        .ok_or(SchemaValidationError::MissingRequiredField { field })?;
    let Some(value) = value.as_bool() else {
        return Err(SchemaValidationError::FieldTypeMismatch {
            field,
            expected: SchemaValueKind::Boolean,
        });
    };
    if value != expected_value {
        return Err(SchemaValidationError::InvalidFieldValue { field });
    }
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
    value
        .as_object()
        .ok_or(SchemaValidationError::FieldTypeMismatch {
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
    value
        .as_array()
        .ok_or(SchemaValidationError::FieldTypeMismatch {
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

fn require_number_field(
    map: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<u64, SchemaValidationError> {
    let value = map
        .get(field)
        .ok_or(SchemaValidationError::MissingRequiredField { field })?;
    value
        .as_u64()
        .ok_or(SchemaValidationError::FieldTypeMismatch {
            field,
            expected: SchemaValueKind::Number,
        })
}

fn validate_summary_epistemic_boundary(
    root: &serde_json::Map<String, Value>,
) -> Result<(), SchemaValidationError> {
    require_exact_string(root, "summary_origin", "derived")?;
    require_exact_string(root, "authority_classification", "non_authoritative")?;
    require_exact_string(root, "display_mode", "human_readable")?;

    let boundary = require_object(root, "epistemic_boundary")?;
    require_exact_bool(boundary, "produces_truth", false)?;
    require_exact_bool(boundary, "produces_decision", false)?;
    require_exact_bool(boundary, "produces_ranking", false)?;
    Ok(())
}

fn validate_numeric_object_values(
    map: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<(), SchemaValidationError> {
    for value in map.values() {
        if value.as_u64().is_none() {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field,
                expected: SchemaValueKind::Number,
            });
        }
    }
    Ok(())
}

fn validate_explanation_array(
    map: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<(), SchemaValidationError> {
    let values = require_array(map, field)?;
    if values.is_empty() {
        return Err(SchemaValidationError::InvalidFieldValue { field });
    }
    let mut previous: Option<&str> = None;
    for value in values {
        let Some(value) = value.as_str() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field,
                expected: SchemaValueKind::String,
            });
        };
        if value.trim().is_empty() {
            return Err(SchemaValidationError::InvalidFieldValue { field });
        }
        if let Some(previous_value) = previous {
            if value <= previous_value {
                return Err(SchemaValidationError::InvalidFieldValue { field });
            }
        }
        previous = Some(value);
    }
    Ok(())
}

fn validate_sorted_unique_string_array(
    values: &[Value],
    field: &'static str,
    require_non_empty: bool,
) -> Result<(), SchemaValidationError> {
    if require_non_empty && values.is_empty() {
        return Err(SchemaValidationError::InvalidFieldValue { field });
    }
    let mut previous: Option<&str> = None;
    let mut seen = BTreeSet::new();
    for value in values {
        let Some(value) = value.as_str() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field,
                expected: SchemaValueKind::String,
            });
        };
        if value.trim().is_empty() {
            return Err(SchemaValidationError::InvalidFieldValue { field });
        }
        if let Some(previous_value) = previous {
            if value <= previous_value {
                return Err(SchemaValidationError::InvalidFieldValue { field });
            }
        }
        if !seen.insert(value) {
            return Err(SchemaValidationError::InvalidFieldValue { field });
        }
        previous = Some(value);
    }
    Ok(())
}

fn is_valid_graph_verdict_label(verdict: &str) -> bool {
    matches!(
        verdict,
        "TRUSTED" | "UNTRUSTED" | "INVALID" | "REJECTED_BY_POLICY"
    )
}

fn validate_graph_incidents(
    incidents: &[Value],
    node_ids: &BTreeSet<String>,
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
        require_non_empty_string(incident, "surface_key")?;
        let severity = require_non_empty_string(incident, "severity")?;
        if !matches!(
            severity,
            "pure_determinism_failure"
                | "authority_drift"
                | "context_drift"
                | "subject_drift"
                | "mixed"
        ) {
            return Err(SchemaValidationError::InvalidFieldValue { field: "severity" });
        }
        let incident_nodes = require_array(incident, "nodes")?;
        require_exact_count(incident, "node_count", incident_nodes.len())?;
        if incident_nodes.is_empty() {
            return Err(SchemaValidationError::InvalidFieldValue { field: "nodes" });
        }
        let mut seen_node_ids = BTreeSet::new();
        for incident_node in incident_nodes {
            let Some(incident_node_id) = incident_node.as_str() else {
                return Err(SchemaValidationError::FieldTypeMismatch {
                    field: "nodes[]",
                    expected: SchemaValueKind::String,
                });
            };
            if incident_node_id.trim().is_empty() {
                return Err(SchemaValidationError::InvalidFieldValue { field: "nodes" });
            }
            if !seen_node_ids.insert(incident_node_id) {
                return Err(SchemaValidationError::InvalidFieldValue { field: "nodes" });
            }
            if !node_ids.contains(incident_node_id) {
                return Err(SchemaValidationError::InvalidFieldValue { field: "nodes" });
            }
        }
    }
    Ok(incident_ids)
}

fn validate_graph_nodes(nodes: &[Value]) -> Result<BTreeSet<String>, SchemaValidationError> {
    let mut node_ids = BTreeSet::new();
    for node in nodes {
        let Some(node) = node.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "nodes[]",
                expected: SchemaValueKind::Object,
            });
        };
        let node_id = require_non_empty_string(node, "id")?;
        if !node_ids.insert(node_id.to_string()) {
            return Err(SchemaValidationError::InvalidFieldValue { field: "id" });
        }
        require_non_empty_string(node, "node_fingerprint")?;
        require_non_empty_string(node, "surface_key")?;
        require_non_empty_string(node, "outcome_key")?;
        let verdict = require_non_empty_string(node, "verdict")?;
        if !is_valid_graph_verdict_label(verdict) {
            return Err(SchemaValidationError::InvalidFieldValue { field: "verdict" });
        }
    }
    Ok(node_ids)
}

fn validate_graph_edges(
    edges: &[Value],
    node_ids: &BTreeSet<String>,
    incident_ids: &BTreeSet<String>,
) -> Result<(), SchemaValidationError> {
    for edge in edges {
        let Some(edge) = edge.as_object() else {
            return Err(SchemaValidationError::FieldTypeMismatch {
                field: "edges[]",
                expected: SchemaValueKind::Object,
            });
        };
        let from = require_non_empty_string(edge, "from")?;
        let to = require_non_empty_string(edge, "to")?;
        if !node_ids.contains(from) {
            return Err(SchemaValidationError::InvalidFieldValue { field: "from" });
        }
        if !node_ids.contains(to) {
            return Err(SchemaValidationError::InvalidFieldValue { field: "to" });
        }
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
                return Err(SchemaValidationError::InvalidFieldValue { field: "edge_type" });
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
    fn coverage_marks_root_graph_surfaces_as_full() {
        assert_eq!(
            schema_coverage_for_endpoint_id(DiagnosticsEndpointId::Graph),
            SchemaCoverage::Full
        );
        assert_eq!(
            schema_coverage_for_endpoint_id(DiagnosticsEndpointId::GraphOverlay),
            SchemaCoverage::Full
        );
    }

    #[test]
    fn coverage_marks_summary_surfaces_as_full() {
        assert_eq!(
            schema_coverage_for_endpoint_id(DiagnosticsEndpointId::Summary),
            SchemaCoverage::Full
        );
        assert_eq!(
            schema_coverage_for_endpoint_id(DiagnosticsEndpointId::RunScopedSummary),
            SchemaCoverage::Full
        );
    }

    #[test]
    fn root_summary_schema_rejects_missing_overlay() {
        let error = validate_response_schema_for_path(
            "/diagnostics/summary",
            &json!({
                "summary_origin": "derived",
                "authority_classification": "non_authoritative",
                "display_mode": "human_readable",
                "epistemic_boundary": {
                    "produces_truth": false,
                    "produces_decision": false,
                    "produces_ranking": false
                },
                "snapshot": {
                    "partition_count": 1,
                    "total_nodes": 2,
                    "total_incidents": 1
                },
                "incidents": {
                    "pure_determinism_failure": 1
                },
                "explanation": [
                    "A",
                    "B"
                ]
            }),
        )
        .expect_err("missing overlay must fail");
        assert_eq!(
            error,
            SchemaValidationError::MissingRequiredField { field: "overlay" }
        );
    }

    #[test]
    fn run_scoped_summary_schema_rejects_authoritative_boundary() {
        let error = validate_response_schema_for_path(
            "/diagnostics/runs/run-a/summary",
            &json!({
                "summary_origin": "derived",
                "authority_classification": "authoritative",
                "display_mode": "human_readable",
                "epistemic_boundary": {
                    "produces_truth": false,
                    "produces_decision": false,
                    "produces_ranking": false
                },
                "run_id": "run-a",
                "snapshot": {
                    "node_count": 1,
                    "incident_count": 0
                },
                "incidents": {},
                "explanation": [
                    "A",
                    "B"
                ]
            }),
        )
        .expect_err("authoritative classification must fail");
        assert_eq!(
            error,
            SchemaValidationError::InvalidFieldValue {
                field: "authority_classification"
            }
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

    #[test]
    fn run_graph_schema_rejects_nodes_missing_required_contract_fields() {
        let error = validate_response_schema_for_path(
            "/diagnostics/runs/run-a/graph",
            &json!({
                "graph_version": "v1",
                "authority": "proof-verifier-cross-node-parity",
                "env_hash": "sha256:env",
                "status": "PASS",
                "provenance": {
                    "artifact_set_hash": "sha256:set",
                    "source_runs": ["run-a"]
                },
                "graph": {
                    "node_count": 1,
                    "edge_count": 0,
                    "incident_count": 0,
                    "nodes": [{
                        "id": "node-a",
                        "node_fingerprint": "sha256:fingerprint-a",
                        "surface_key": "sha256:surface",
                        "verdict": "TRUSTED"
                    }],
                    "edges": [],
                    "incidents": []
                }
            }),
        )
        .expect_err("node objects missing outcome_key must fail");
        assert_eq!(
            error,
            SchemaValidationError::MissingRequiredField {
                field: "outcome_key"
            }
        );
    }

    #[test]
    fn run_graph_schema_rejects_incidents_missing_required_contract_fields() {
        let error = validate_response_schema_for_path(
            "/diagnostics/runs/run-a/graph",
            &json!({
                "graph_version": "v1",
                "authority": "proof-verifier-cross-node-parity",
                "env_hash": "sha256:env",
                "status": "PASS",
                "provenance": {
                    "artifact_set_hash": "sha256:set",
                    "source_runs": ["run-a"]
                },
                "graph": {
                    "node_count": 1,
                    "edge_count": 0,
                    "incident_count": 1,
                    "nodes": [{
                        "id": "node-a",
                        "node_fingerprint": "sha256:fingerprint-a",
                        "surface_key": "sha256:surface",
                        "outcome_key": "sha256:outcome-a",
                        "verdict": "TRUSTED"
                    }],
                    "edges": [],
                    "incidents": [{
                        "incident_id": "sha256:incident-a",
                        "surface_key": "sha256:surface",
                        "nodes": ["node-a"],
                        "node_count": 1
                    }]
                }
            }),
        )
        .expect_err("incident objects missing severity must fail");
        assert_eq!(
            error,
            SchemaValidationError::MissingRequiredField { field: "severity" }
        );
    }

    #[test]
    fn root_graph_schema_rejects_missing_partition_key() {
        let error = validate_response_schema_for_path(
            "/diagnostics/graph",
            &json!({
                "graph_origin": "derived",
                "authority_classification": "non_authoritative",
                "aggregation_mode": "overlay_only",
                "partition_count": 1,
                "partitions": [{
                    "partition_id": "sha256:partition-a",
                    "run_count": 1,
                    "run_ids": ["run-a"],
                    "source_runs": ["phase12-cross-node-parity"],
                    "graph": {
                        "node_count": 0,
                        "edge_count": 0,
                        "incident_count": 0,
                        "nodes": [],
                        "edges": [],
                        "incidents": []
                    }
                }]
            }),
        )
        .expect_err("missing partition_key must fail");
        assert_eq!(
            error,
            SchemaValidationError::MissingRequiredField {
                field: "partition_key"
            }
        );
    }

    #[test]
    fn graph_overlay_schema_rejects_invalid_aggregation_mode() {
        let error = validate_response_schema_for_path(
            "/diagnostics/graph/overlay",
            &json!({
                "graph_origin": "derived",
                "authority_classification": "non_authoritative",
                "aggregation_mode": "majority",
                "partition_count": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0,
                "agreements": [],
                "conflicts": [],
                "islands": []
            }),
        )
        .expect_err("non overlay aggregation mode must fail");
        assert_eq!(
            error,
            SchemaValidationError::InvalidFieldValue {
                field: "aggregation_mode"
            }
        );
    }
}
