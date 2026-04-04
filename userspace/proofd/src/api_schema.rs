use serde_json::{json, Value};

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
];

pub fn schema_for_endpoint_id(
    endpoint_id: DiagnosticsEndpointId,
) -> Option<&'static EndpointSchema> {
    PUBLIC_ENDPOINT_SCHEMAS
        .iter()
        .find(|schema| schema.endpoint_id == endpoint_id)
}

pub fn schema_for_path(path: &str) -> Option<&'static EndpointSchema> {
    public_endpoint_contract_for_path(path).and_then(|contract| schema_for_endpoint_id(contract.id))
}

pub fn validate_response_schema_for_path(
    path: &str,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let Some(schema) = schema_for_path(path) else {
        return Ok(());
    };
    validate_response_schema(schema, value)
}

pub fn public_schema_declarations() -> Vec<Value> {
    ROOT_DIAGNOSTICS_ENDPOINTS
        .iter()
        .chain(RUN_SCOPED_DIAGNOSTICS_ENDPOINTS.iter())
        .filter_map(|contract| {
            schema_for_endpoint_id(contract.id).map(|schema| {
                json!({
                    "path_template": contract.path_template,
                    "root_kind": schema_value_kind_name(schema.root_kind),
                    "required_fields": schema
                        .required_fields
                        .iter()
                        .map(|field| json!({
                            "name": field.name,
                            "kind": schema_value_kind_name(field.kind),
                        }))
                        .collect::<Vec<_>>(),
                    "optional_fields": schema
                        .optional_fields
                        .iter()
                        .map(|field| json!({
                            "name": field.name,
                            "kind": schema_value_kind_name(field.kind),
                        }))
                        .collect::<Vec<_>>(),
                })
            })
        })
        .collect()
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

fn schema_value_kind_name(kind: SchemaValueKind) -> &'static str {
    match kind {
        SchemaValueKind::Object => "object",
        SchemaValueKind::Array => "array",
        SchemaValueKind::String => "string",
        SchemaValueKind::Number => "number",
        SchemaValueKind::Boolean => "boolean",
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
        schema_for_path, validate_response_schema_for_path, SchemaValidationError, SchemaValueKind,
    };
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
}
