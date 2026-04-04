pub mod api_contract;
pub mod api_schema;
pub mod determinism;
pub mod internal;

use proof_verifier::canonical::jcs::canonicalize_json_value;
use proof_verifier::diversity_ledger::{
    load_diversity_ledger_entries, VerificationDiversityLedgerEntry,
};
use proof_verifier::diversity_ledger_producer::{
    parse_event_time_to_unix_ns, run_diversity_ledger_producer,
    VerificationDiversityLedgerProducerConfig, VerificationDiversityLedgerProducerManifest,
    VerificationNodeBinding,
};
use proof_verifier::policy::policy_engine::compute_policy_hash;
use proof_verifier::registry::snapshot::compute_registry_snapshot_hash;
use proof_verifier::trust_reuse_runtime_evaluator::{
    run_trust_reuse_runtime_evaluator, TrustReuseRuntimeEvaluatorConfig,
};
use proof_verifier::trust_reuse_runtime_surface::{
    load_trust_reuse_runtime_surface as load_native_trust_reuse_runtime_surface, TrustReuseOutcome,
    TrustReuseRuntimeEvent, TrustReuseRuntimeSurfaceReport,
};
use proof_verifier::types::{AuditMode, ReceiptMode, ReceiptSignerConfig, VerifyRequest};
use proof_verifier::verification_context_object::{
    load_verification_context_object, VerificationContextObject,
};
use proof_verifier::{verify_bundle, RegistrySnapshot, TrustPolicy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api_contract::{
    allowed_query_keys_for_path, resolve_public_endpoint, scan_forbidden_observability_fields,
    DiagnosticsEndpointId, ResolvedDiagnosticsEndpoint, ALLOWED_INCIDENT_FILTERS,
};
use crate::api_schema::validate_response_schema_for_path;
use crate::determinism::artifacts::{
    write_canonical_json_file_if_absent_or_same, write_canonical_json_value_if_absent_or_same,
};
use crate::determinism::contract::{
    build_verification_context_material, build_verification_determinism_contract,
};
use crate::determinism::fingerprint::canonical_hash_value_prefixed;
use crate::internal::replay_route::handle_internal_replay;

const RUN_LEVEL_ARTIFACTS: &[&str] = &[
    "report.json",
    "parity_report.json",
    "proofd_run_manifest.json",
    "verification_determinism_contract.json",
    "verification_determinism_replay_report.json",
    "verification_determinism_incident.json",
    "verification_audit_ledger.jsonl",
    "verification_diversity_ledger_binding.json",
    "verification_diversity_ledger.json",
    "verification_diversity_ledger_append_report.json",
    "replay_boundary_flow_source.json",
    "trust_reuse_runtime_surface.json",
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

const VERIFICATION_AUDIT_LEDGER_FILE: &str = "verification_audit_ledger.jsonl";
const VERIFICATION_DIVERSITY_BINDING_FILE: &str = "verification_diversity_ledger_binding.json";
const VERIFICATION_DIVERSITY_LEDGER_FILE: &str = "verification_diversity_ledger.json";
const VERIFICATION_DIVERSITY_APPEND_REPORT_FILE: &str =
    "verification_diversity_ledger_append_report.json";
const REPORTS_DIR: &str = "reports";
const REPLAY_BOUNDARY_FLOW_SOURCE_FILE: &str = "replay_boundary_flow_source.json";
const REPLAY_REPORT_FILE: &str = "replay_report.json";
const TRUST_REUSE_FLOW_SOURCE_FILE: &str = "trust_reuse_flow_source.json";
const TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH: &str = "reports/trust_reuse_runtime_surface.json";
const CONTEXT_POLICY_SNAPSHOT_RELATIVE_PATH: &str = "context/policy_snapshot.json";
const CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH: &str = "context/registry_snapshot.json";
const CONTEXT_RULES_RELATIVE_PATH: &str = "context/context_rules.json";
const VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH: &str = "context/verification_context_object.json";
const VERIFICATION_CONTEXT_VERIFIER_CONTRACT_VERSION: &str = "phase12-context-v1";
const RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE: &str = "trust_reuse_runtime_surface.json";
const TRUST_REUSE_RUNTIME_EVALUATOR_DIR: &str = "trust_reuse_runtime_evaluator";
const TRUST_REUSE_RUNTIME_EXPECTED_SUBJECT_FILE: &str = "expected_verdict_subject.json";
const PROOFD_RUN_MANIFEST_FILE: &str = "proofd_run_manifest.json";
const RECEIPT_RELATIVE_PATH: &str = "receipts/verification_receipt.json";
const VERIFICATION_DETERMINISM_CONTRACT_FILE: &str = "verification_determinism_contract.json";
const VERIFICATION_DETERMINISM_REPLAY_REPORT_FILE: &str =
    "verification_determinism_replay_report.json";
const VERIFICATION_DETERMINISM_INCIDENT_FILE: &str = "verification_determinism_incident.json";
pub use crate::api_contract::{API_VERSION, PHASE13_FORBIDDEN_FIELDS};
const NESTED_RUN_LEVEL_ARTIFACTS: &[&str] = &[
    RECEIPT_RELATIVE_PATH,
    CONTEXT_POLICY_SNAPSHOT_RELATIVE_PATH,
    CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH,
    CONTEXT_RULES_RELATIVE_PATH,
    VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH,
    TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH,
];
const MAX_VERIFY_BUNDLE_BODY_BYTES: usize = 64 * 1024;
static GENERATED_RUN_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
static TEST_TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
fn unique_test_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let counter = TEST_TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}-{counter:016x}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

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
struct VerifyBundleTrustReuseRuntimeBinding {
    verification_context_path: String,
    verifier_attestation_path: String,
    verifier_registry_path: String,
    verifier_key_path: String,
    #[serde(default)]
    source_run_id: Option<String>,
    #[serde(default)]
    reuse_group_id: Option<String>,
    #[serde(default)]
    surface_local_path_id: Option<String>,
    #[serde(default)]
    trust_reuse_source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VerifyBundleRequestBody {
    bundle_path: String,
    policy_path: String,
    registry_path: String,
    #[serde(default)]
    receipt_mode: Option<VerifyBundleReceiptMode>,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    receipt_signer: Option<VerifyBundleReceiptSigner>,
    #[serde(default)]
    diversity_binding: Option<VerifyBundleDiversityBinding>,
    #[serde(default)]
    replay_boundary_binding: Option<VerifyBundleReplayBoundaryBinding>,
    #[serde(default)]
    trust_reuse_binding: Option<VerifyBundleTrustReuseBinding>,
    #[serde(default)]
    trust_reuse_runtime_binding: Option<VerifyBundleTrustReuseRuntimeBinding>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReplayDeterminismRequestBody {
    verify_request: VerifyBundleRequestBody,
    #[serde(default)]
    source_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct VerifyBundleResponseBody {
    status: &'static str,
    run_id: String,
    verdict: &'static str,
    verdict_subject: Value,
    receipt_emitted: bool,
    receipt_path: Option<String>,
    request_fingerprint: String,
    behavioral_observability_emitted: bool,
    audit_ledger_path: Option<String>,
    verification_diversity_ledger_binding_path: Option<String>,
    verification_diversity_ledger_path: Option<String>,
    replay_boundary_flow_source_path: Option<String>,
    replay_boundary_flow_source_origin: Option<String>,
    trust_reuse_runtime_surface_path: Option<String>,
    trust_reuse_runtime_surface_origin: Option<String>,
    trust_reuse_flow_source_path: Option<String>,
    trust_reuse_flow_source_origin: Option<String>,
    verification_determinism_contract_path: String,
    verification_determinism_artifact_hash: String,
    findings_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RunArtifactDescriptor {
    path: String,
    content_type: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct FederationDistributionEntry {
    id: String,
    entry_count: usize,
}

// Task 6: Spec-compliant projection structs for federation diagnostics API surface
#[derive(Debug, Clone, Serialize)]
struct FederationDiagnosticsProjection {
    run_id: String,
    verifier_count: usize,
    observed_verifiers: Vec<SpecFederationVerifierEntry>,
    authority_chain_distribution: Vec<SpecAuthorityChainEntry>,
    execution_cluster_distribution: Vec<SpecExecutionClusterEntry>,
    missing_execution_cluster_entry_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SpecFederationVerifierEntry {
    verifier_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lineage_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SpecAuthorityChainEntry {
    authority_chain_id: String,
    entry_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SpecExecutionClusterEntry {
    cluster_id: String,
    entry_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct FederationObservedEntry {
    entry_id: String,
    verification_node_id: String,
    verifier_id: String,
    authority_chain_id: String,
    lineage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_cluster_id: Option<String>,
    receipt_hash: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct FederationDiagnosticsResponseBody {
    run_id: String,
    source_artifact_path: &'static str,
    entry_count: usize,
    unique_verification_node_count: usize,
    unique_verifier_count: usize,
    unique_authority_chain_count: usize,
    unique_lineage_count: usize,
    unique_execution_cluster_count: usize,
    missing_execution_cluster_entry_count: usize,
    verification_node_distribution: Vec<FederationDistributionEntry>,
    verifier_distribution: Vec<FederationDistributionEntry>,
    authority_chain_distribution: Vec<FederationDistributionEntry>,
    lineage_distribution: Vec<FederationDistributionEntry>,
    execution_cluster_distribution: Vec<FederationDistributionEntry>,
    observed_entries: Vec<FederationObservedEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct ContextArtifactPaths {
    context_object: &'static str,
    context_rules: &'static str,
    policy_snapshot: &'static str,
    registry_snapshot: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diversity_binding: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diversity_ledger: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replay_boundary_flow_source: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trust_reuse_flow_source: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trust_reuse_runtime_surface: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
struct ContextMaterialBindingStatus {
    policy_hash_matches_declared_context: bool,
    registry_snapshot_hash_matches_declared_context: bool,
    context_rules_hash_matches_declared_context: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_policy_hash_matches_declared_context: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_registry_snapshot_hash_matches_declared_context: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    legacy_verification_context_id_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ContextObservationSource {
    source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_artifact_path: Option<&'static str>,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ContextDiagnosticsResponseBody {
    run_id: String,
    source_artifact_paths: ContextArtifactPaths,
    declared_context: VerificationContextObject,
    material_binding_status: ContextMaterialBindingStatus,
    observed_context_id_sources: Vec<ContextObservationSource>,
    observed_context_ref_sources: Vec<ContextObservationSource>,
}

#[derive(Debug, Clone, Serialize)]
struct RegistryDiagnosticsResponseBody {
    run_id: String,
    source_artifact_path: &'static str,
    declared_registry_snapshot_hash: String,
    declared_registry_entry_count: usize,
    context_binding_status: RegistryContextBindingStatus,
    observed_registry_hash_sources: Vec<RegistryObservationSource>,
}

#[derive(Debug, Clone, Serialize)]
struct RegistryContextBindingStatus {
    registry_snapshot_hash_matches_declared_context: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct RegistryObservationSource {
    source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_artifact_path: Option<&'static str>,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryDiagnosticsResponseBody {
    run_id: String,
    request_fingerprint: String,
    peer_run_count: usize,
    peer_run_ids: Vec<String>,
    verdict_consistency: VerdictConsistency,
    context_hash_consistency: ContextHashConsistency,
    registry_hash_consistency: RegistryHashConsistency,
}

#[derive(Debug, Clone, Serialize)]
struct VerdictConsistency {
    all_verdicts_match: bool,
    observed_verdicts: Vec<RunVerdictEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct RunVerdictEntry {
    run_id: String,
    verdict: String,
}

#[derive(Debug, Clone, Serialize)]
struct ContextHashConsistency {
    all_context_hashes_match: Option<bool>,
    observed_context_hashes: Vec<RunHashEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct RegistryHashConsistency {
    all_registry_hashes_match: Option<bool>,
    observed_registry_hashes: Vec<RunHashEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct RunHashEntry {
    run_id: String,
    hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuthoritySinkholeCompanionSourceDocument {
    source_version: u32,
    flow_surface: String,
    status: String,
    run_id: String,
    window_model: String,
    events: Vec<AuthoritySinkholeCompanionSourceEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    Path::new(REPORTS_DIR).join(REPLAY_REPORT_FILE)
}

fn replay_report_surface_path_id() -> String {
    format!("{REPORTS_DIR}/{REPLAY_REPORT_FILE}")
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

            if target.path == "/healthz" {
                json_response(
                    200,
                    json!({
                        "status": "ok",
                        "service": "proofd",
                        "mode": "verification_execution_and_read_only_diagnostics",
                    }),
                )
            } else if let Some(resolved) = resolve_public_endpoint(&target.path) {
                handle_diagnostics_endpoint(resolved, &target, evidence_dir)
            } else {
                json_response(404, json!({ "error": "not_found" }))
            }
        }
        "POST" => match target.path.as_str() {
            "/verify/bundle" => handle_verify_bundle(raw_body.unwrap_or_default(), evidence_dir),
            "/internal/replay" => {
                handle_internal_replay(raw_body.unwrap_or_default(), evidence_dir)
            }
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

    if let Some(allowed_keys) = allowed_query_keys_for_path(&target.path) {
        let _ = parse_query(target.query.as_deref(), allowed_keys)?;
        return Ok(());
    }

    Err(ServiceError::BadRequest("unsupported_query_parameter"))
}

fn handle_diagnostics_endpoint(
    resolved: ResolvedDiagnosticsEndpoint,
    target: &RequestTarget,
    evidence_dir: &Path,
) -> DiagnosticsResponse {
    let contract = resolved.contract;
    match contract.id {
        DiagnosticsEndpointId::Version => observability_json_response(
            &target.path,
            200,
            json!({
                "api_version": API_VERSION,
                "service": "proofd",
                "contract": "read-only diagnostics surface",
                "invariants": [
                    "service != authority",
                    "diagnostics != decision",
                    "parity != consensus",
                    "trust does not affect verdict"
                ],
                "endpoints": crate::api_contract::public_endpoint_declarations()
            }),
        ),
        DiagnosticsEndpointId::Runs => match list_runs(evidence_dir) {
            Ok(value) => observability_json_response(&target.path, 200, value),
            Err(error) => error_response(error),
        },
        DiagnosticsEndpointId::Federation => {
            match build_global_federation_diagnostics(evidence_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::Context => match build_global_context_diagnostics(evidence_dir) {
            Ok(value) => observability_json_response(&target.path, 200, value),
            Err(error) => error_response(error),
        },
        DiagnosticsEndpointId::Trust => match build_global_trust_diagnostics(evidence_dir) {
            Ok(value) => observability_json_response(&target.path, 200, value),
            Err(error) => error_response(error),
        },
        DiagnosticsEndpointId::ParityContextRelation => {
            match build_parity_context_relation(evidence_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::Incidents => {
            match load_incident_report(evidence_dir, target.query.as_deref()) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::IncidentById => {
            let Some(incident_id) = resolved.params.incident_id.as_deref() else {
                return json_response(404, json!({ "error": "not_found" }));
            };
            match load_single_incident(evidence_dir, &incident_id) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::FingerprintBoundary => {
            let Some(fp) = resolved.params.fp.as_deref() else {
                return json_response(404, json!({ "error": "not_found" }));
            };
            match build_fingerprint_boundary_diagnostics(fp, evidence_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::ReplicatedBoundary => {
            match build_replicated_boundary_status(evidence_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunSummary => {
            let (run_id, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            match build_run_summary(run_id, &run_dir) {
                Ok(summary) => observability_json_response(&target.path, 200, summary),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunArtifactsIndex => {
            let (run_id, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            match build_run_artifact_index(run_id, &run_dir) {
                Ok(index) => observability_json_response(&target.path, 200, index),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunArtifactFile => {
            let (_, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            let Some(artifact_path) = resolved.params.artifact_path.as_deref() else {
                return json_response(404, json!({ "error": "not_found" }));
            };
            let artifact_path = match parse_run_artifact_path(artifact_path) {
                Ok(path) => path,
                Err(error) => return error_response(error),
            };
            match resolve_run_artifact_path(&run_dir, artifact_path) {
                Ok(path) => serve_artifact_file(path, artifact_content_type(artifact_path)),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunFederation => {
            let (run_id, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            match build_run_federation_diagnostics(run_id, &run_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunContext => {
            let (run_id, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            match build_run_context_diagnostics(run_id, &run_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunRegistry => {
            let (run_id, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            match build_run_registry_diagnostics(run_id, &run_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        DiagnosticsEndpointId::RunBoundary => {
            let (run_id, run_dir) = match resolve_run_scope(&resolved, evidence_dir) {
                Ok(parts) => parts,
                Err(response) => return response,
            };
            match build_run_boundary_diagnostics(run_id, &run_dir, evidence_dir) {
                Ok(value) => observability_json_response(&target.path, 200, value),
                Err(error) => error_response(error),
            }
        }
        _ if contract.artifact_file.is_some() => {
            let base_dir = if resolved.params.run_id.is_some() {
                match resolve_run_scope(&resolved, evidence_dir) {
                    Ok((_, run_dir)) => run_dir,
                    Err(response) => return response,
                }
            } else {
                evidence_dir.to_path_buf()
            };
            let artifact = contract
                .artifact_file
                .expect("passthrough artifact missing");
            serve_observability_json_file(&target.path, base_dir.join(artifact))
        }
        _ => json_response(404, json!({ "error": "not_found" })),
    }
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
            .get("artifact_paths")
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

fn resolve_run_scope<'a>(
    resolved: &'a ResolvedDiagnosticsEndpoint,
    evidence_dir: &Path,
) -> Result<(&'a str, PathBuf), DiagnosticsResponse> {
    let Some(run_id) = resolved.params.run_id.as_deref() else {
        return Err(json_response(404, json!({ "error": "not_found" })));
    };
    if !is_safe_path_segment(run_id) {
        return Err(json_response(404, json!({ "error": "invalid_run_id" })));
    }
    Ok((run_id, evidence_dir.join(run_id)))
}

fn build_run_summary(run_id: &str, run_dir: &Path) -> Result<Value, ServiceError> {
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    let artifacts = list_run_artifacts(run_dir)?;
    let artifact_paths = list_run_artifact_paths(run_dir)?;
    Ok(json!({
        "run_id": run_id,
        "artifacts": artifacts,
        "artifact_paths": artifact_paths,
    }))
}

fn build_run_artifact_index(run_id: &str, run_dir: &Path) -> Result<Value, ServiceError> {
    // Task 5.1: propagate run_dir_not_found before listing descriptors
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }
    let artifacts = list_run_artifact_descriptors(run_dir)?;
    Ok(json!({
        "run_id": run_id,
        "artifact_count": artifacts.len(),
        "artifacts": artifacts,
    }))
}

fn build_run_federation_diagnostics(run_id: &str, run_dir: &Path) -> Result<Value, ServiceError> {
    // Task 6.1: run_dir_not_found check before artifact check
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    let ledger_path = run_dir.join(VERIFICATION_DIVERSITY_LEDGER_FILE);
    let entries = load_diversity_ledger_entries(&ledger_path).map_err(|_| {
        if ledger_path.is_file() {
            ServiceError::MalformedArtifact("invalid_federation_artifact")
        } else {
            ServiceError::NotFound("artifact_not_found")
        }
    })?;

    // Build internal rich model
    let verifier_distribution =
        build_federation_distribution(&entries, |entry| entry.verifier_id.clone());
    let authority_chain_distribution =
        build_federation_distribution(&entries, |entry| entry.authority_chain_id.clone());
    let (execution_cluster_distribution_internal, missing_execution_cluster_entry_count) =
        build_optional_federation_distribution(&entries, |entry| {
            entry.execution_cluster_id.clone()
        });

    // Task 6.7: observed_verifiers sorted by verifier_id (BTreeMap already gives lex order,
    // but we deduplicate by verifier_id keeping first lineage_id seen)
    let mut seen_verifiers: std::collections::BTreeMap<String, Option<String>> =
        std::collections::BTreeMap::new();
    for entry in &entries {
        seen_verifiers
            .entry(entry.verifier_id.clone())
            .or_insert_with(|| entry.lineage_id.clone().into());
    }
    let observed_verifiers: Vec<SpecFederationVerifierEntry> = seen_verifiers
        .into_iter()
        .map(|(verifier_id, lineage_id)| SpecFederationVerifierEntry {
            verifier_id,
            lineage_id,
        })
        .collect();

    // Task 6.4: SpecAuthorityChainEntry — authority_chain_id + entry_count
    let authority_chain_dist: Vec<SpecAuthorityChainEntry> = authority_chain_distribution
        .iter()
        .map(|e| SpecAuthorityChainEntry {
            authority_chain_id: e.id.clone(),
            entry_count: e.entry_count,
        })
        .collect();

    // Task 6.5: SpecExecutionClusterEntry — cluster_id + entry_count
    let execution_cluster_dist: Vec<SpecExecutionClusterEntry> =
        execution_cluster_distribution_internal
            .iter()
            .map(|e| SpecExecutionClusterEntry {
                cluster_id: e.id.clone(),
                entry_count: e.entry_count,
            })
            .collect();

    // Task 6.6: serialize the projection, not the internal body
    let projection = FederationDiagnosticsProjection {
        run_id: run_id.to_string(),
        verifier_count: verifier_distribution.len(),
        observed_verifiers,
        authority_chain_distribution: authority_chain_dist,
        execution_cluster_distribution: execution_cluster_dist,
        missing_execution_cluster_entry_count,
    };

    serde_json::to_value(&projection)
        .map_err(|_| ServiceError::Runtime("response_serialize_failed"))
}

/// Global federation diagnostics: aggregate verifier topology across all runs.
/// Scans all run directories, loads diversity ledgers, and produces a system-level
/// view of verifier distribution. Read-only, deterministic, non-authoritative.
fn build_global_federation_diagnostics(evidence_dir: &Path) -> Result<Value, ServiceError> {
    let entries_iter =
        fs::read_dir(evidence_dir).map_err(|_| ServiceError::NotFound("evidence_dir_not_found"))?;

    // Aggregate entries across all runs (fail-open per run)
    let mut all_entries: Vec<VerificationDiversityLedgerEntry> = Vec::new();
    let mut run_count = 0usize;
    let mut runs_with_ledger = 0usize;

    let mut run_dirs: Vec<PathBuf> = entries_iter
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    run_dirs.sort();

    for run_dir in &run_dirs {
        if !is_safe_path_segment(&run_dir.file_name().unwrap_or_default().to_string_lossy()) {
            continue;
        }
        run_count += 1;
        let ledger_path = run_dir.join(VERIFICATION_DIVERSITY_LEDGER_FILE);
        if let Ok(entries) = load_diversity_ledger_entries(&ledger_path) {
            runs_with_ledger += 1;
            all_entries.extend(entries);
        }
    }

    // verifier_id → { run_ids (unique), entry_count }
    let mut verifier_runs: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    let mut verifier_entry_count: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    // fingerprint_id → unique verifier_ids
    let mut fp_verifiers: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();

    for entry in &all_entries {
        verifier_runs
            .entry(entry.verifier_id.clone())
            .or_default()
            .insert(entry.run_id.clone());
        *verifier_entry_count
            .entry(entry.verifier_id.clone())
            .or_default() += 1;
        fp_verifiers
            .entry(entry.verification_context_id.clone())
            .or_default()
            .insert(entry.verifier_id.clone());
    }

    // verifier_id → set of context_ids observed across all entries
    let mut verifier_contexts: std::collections::BTreeMap<
        String,
        std::collections::BTreeSet<String>,
    > = std::collections::BTreeMap::new();
    for entry in &all_entries {
        verifier_contexts
            .entry(entry.verifier_id.clone())
            .or_default()
            .insert(entry.verification_context_id.clone());
    }

    // Build verifier summary (sorted by verifier_id — BTreeMap guarantees this)
    let verifiers: Vec<Value> = verifier_runs
        .iter()
        .map(|(verifier_id, run_ids)| {
            let contexts: Vec<&str> = verifier_contexts
                .get(verifier_id)
                .map(|s| {
                    let mut v: Vec<&str> = s.iter().map(String::as_str).collect();
                    v.sort();
                    v
                })
                .unwrap_or_default();
            json!({
                "verifier_id": verifier_id,
                "run_count": run_ids.len(),
                "entry_count": verifier_entry_count.get(verifier_id).copied().unwrap_or(0),
                "context_ids": contexts,
            })
        })
        .collect();

    // Unique fingerprint count (by verification_context_id as proxy)
    let fingerprint_count = fp_verifiers.len();

    Ok(json!({
        "verifier_count": verifiers.len(),
        "verifiers": verifiers,
        "runs": {
            "total": run_count,
            "with_ledger": runs_with_ledger,
        },
        "fingerprints": {
            "total": fingerprint_count,
        },
        "total_ledger_entries": all_entries.len(),
    }))
}

/// Global context diagnostics: aggregate verification_context_id distribution
/// across all runs. Shows which contexts are active, how many runs share each
/// context, and whether context IDs are consistent within fingerprint groups.
/// Read-only, deterministic, non-authoritative.
fn build_global_context_diagnostics(evidence_dir: &Path) -> Result<Value, ServiceError> {
    let entries_iter =
        fs::read_dir(evidence_dir).map_err(|_| ServiceError::NotFound("evidence_dir_not_found"))?;

    let mut run_dirs: Vec<PathBuf> = entries_iter
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    run_dirs.sort();

    // context_id → set of run_ids that produced it
    let mut context_runs: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    // fingerprint → set of context_ids observed
    let mut fp_contexts: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();

    let mut run_count = 0usize;
    let mut runs_with_context = 0usize;

    for run_dir in &run_dirs {
        let run_id_os = run_dir.file_name().unwrap_or_default().to_string_lossy();
        if !is_safe_path_segment(&run_id_os) {
            continue;
        }
        let run_id = run_id_os.to_string();
        run_count += 1;

        // Load context object (fail-open) — read context_id directly from JSON
        // to avoid strict validation requirements in test fixtures
        let ctx_path = run_dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH);
        if !ctx_path.is_file() {
            continue;
        }
        let Ok(ctx_json) = load_required_run_json_artifact::<Value>(&ctx_path, "skip") else {
            continue;
        };
        let Some(context_id) = ctx_json
            .get("verification_context_id")
            .and_then(Value::as_str)
        else {
            continue;
        };
        runs_with_context += 1;
        context_runs
            .entry(context_id.to_string())
            .or_default()
            .insert(run_id.clone());

        // Associate fingerprint → context (via manifest, fail-open)
        let manifest_path = run_dir.join(PROOFD_RUN_MANIFEST_FILE);
        if let Ok(manifest) = load_required_run_json_artifact::<Value>(&manifest_path, "skip") {
            if let Some(fp) = manifest.get("request_fingerprint").and_then(Value::as_str) {
                fp_contexts
                    .entry(fp.to_string())
                    .or_default()
                    .insert(context_id.to_string());
            }
        }
    }

    // Build context summary (sorted by context_id — BTreeMap guarantees this)
    let contexts: Vec<Value> = context_runs
        .iter()
        .map(|(context_id, run_ids)| {
            json!({
                "context_id": context_id,
                "run_count": run_ids.len(),
            })
        })
        .collect();

    // Fingerprints with multiple distinct contexts (potential context drift)
    let fingerprints_with_context_drift: Vec<Value> = fp_contexts
        .iter()
        .filter(|(_, ctx_ids)| ctx_ids.len() > 1)
        .map(|(fp, ctx_ids)| {
            let mut sorted: Vec<&str> = ctx_ids.iter().map(String::as_str).collect();
            sorted.sort();
            json!({
                "request_fingerprint": fp,
                "distinct_context_count": ctx_ids.len(),
                "context_ids": sorted,
            })
        })
        .collect();

    Ok(json!({
        "context_count": contexts.len(),
        "contexts": contexts,
        "runs": {
            "total": run_count,
            "with_context": runs_with_context,
        },
        "context_drift": {
            "fingerprints_with_multiple_contexts": fingerprints_with_context_drift.len(),
            "fingerprints": fingerprints_with_context_drift,
        },
    }))
}

/// Global trust diagnostics: aggregate producer registry topology across all runs.
/// Reads context/registry_snapshot.json from each run (fail-open per run).
/// Shows which producers appear, their entry counts, and registry version distribution.
/// Read-only, deterministic, non-authoritative — trust does not affect verdict.
fn build_global_trust_diagnostics(evidence_dir: &Path) -> Result<Value, ServiceError> {
    let entries_iter =
        fs::read_dir(evidence_dir).map_err(|_| ServiceError::NotFound("evidence_dir_not_found"))?;

    let mut run_dirs: Vec<PathBuf> = entries_iter
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    run_dirs.sort();

    // producer_id → set of run_ids where this producer appears
    let mut producer_runs: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    // registry_version → count of runs
    let mut version_counts: std::collections::BTreeMap<u32, usize> =
        std::collections::BTreeMap::new();
    // registry_snapshot_hash → set of run_ids (for hash consistency)
    let mut hash_runs: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();

    let mut run_count = 0usize;
    let mut runs_with_registry = 0usize;

    for run_dir in &run_dirs {
        let run_id_os = run_dir.file_name().unwrap_or_default().to_string_lossy();
        if !is_safe_path_segment(&run_id_os) {
            continue;
        }
        let run_id = run_id_os.to_string();
        run_count += 1;

        let reg_path = run_dir.join(CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH);
        if !reg_path.is_file() {
            continue;
        }
        let Ok(registry) = load_required_run_json_artifact::<RegistrySnapshot>(&reg_path, "skip")
        else {
            continue;
        };
        runs_with_registry += 1;

        *version_counts.entry(registry.registry_version).or_default() += 1;

        hash_runs
            .entry(registry.registry_snapshot_hash.clone())
            .or_default()
            .insert(run_id.clone());

        for producer_id in registry.producers.keys() {
            producer_runs
                .entry(producer_id.clone())
                .or_default()
                .insert(run_id.clone());
        }
    }

    // Build producer summary (sorted by producer_id — BTreeMap guarantees this)
    let producers: Vec<Value> = producer_runs
        .iter()
        .map(|(producer_id, run_ids)| {
            json!({
                "producer_id": producer_id,
                "run_count": run_ids.len(),
            })
        })
        .collect();

    // Registry version distribution
    let version_distribution: Vec<Value> = version_counts
        .iter()
        .map(|(version, count)| json!({ "registry_version": version, "run_count": count }))
        .collect();

    // Hash consistency: multiple distinct hashes = registry drift
    let distinct_hash_count = hash_runs.len();
    let registry_hash_consistent = distinct_hash_count <= 1;

    Ok(json!({
        "producer_count": producers.len(),
        "producers": producers,
        "runs": {
            "total": run_count,
            "with_registry": runs_with_registry,
        },
        "registry_version_distribution": version_distribution,
        "registry_hash_consistency": {
            "consistent": registry_hash_consistent,
            "distinct_hash_count": distinct_hash_count,
        },
    }))
}

/// Replicated verification boundary status: reports the current boundary state
/// between proofd diagnostics surface and disallowed Phase-13 routes.
/// Invariant: verified proof != replay admission.
/// Read-only, deterministic, non-authoritative.
fn build_replicated_boundary_status(_evidence_dir: &Path) -> Result<Value, ServiceError> {
    // Boundary invariants from PHASE13_ARCHITECTURE_MAP.md §4.5
    // These are static architectural facts, not runtime-computed values.
    Ok(json!({
        "boundary_status": "HOLD",
        "invariants": [
            "verified proof != replay admission",
            "replicated verification remains a Phase-13 bridge concern",
            "proofd = verification and diagnostics service",
            "automatic replay admission is outside Phase-13 scope"
        ],
        "disallowed_routes": ["/replay", "/consensus", "/cluster"],
        "diagnostics_routes_allowed": [
            "/diagnostics/runs",
            "/diagnostics/federation",
            "/diagnostics/context",
            "/diagnostics/trust",
            "/diagnostics/parity",
            "/diagnostics/incidents",
            "/diagnostics/fingerprints/{fp}",
            "/diagnostics/parity/context-relation",
            "/diagnostics/replicated-boundary"
        ],
        "phase": "phase-13",
        "note": "proofd is a verification and diagnostics service. It does not perform replay execution, consensus arbitration, or cluster coordination.",
    }))
}
/// whether the two runs share the same context, have different contexts, or
/// context information is unavailable. Diagnostic only — no authority.
fn build_parity_context_relation(evidence_dir: &Path) -> Result<Value, ServiceError> {
    let parity_path = evidence_dir.join("parity_report.json");
    let parity = load_required_run_json_artifact::<Value>(&parity_path, "invalid_parity_report")?;

    // Build run_id → context_id map (fail-open per run)
    let mut run_context: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();

    if let Ok(entries) = fs::read_dir(evidence_dir) {
        for entry in entries.flatten() {
            let run_dir = entry.path();
            if !run_dir.is_dir() {
                continue;
            }
            let run_id = entry.file_name().to_string_lossy().to_string();
            if !is_safe_path_segment(&run_id) {
                continue;
            }
            let ctx_path = run_dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH);
            if !ctx_path.is_file() {
                continue;
            }
            if let Ok(ctx_json) = load_required_run_json_artifact::<Value>(&ctx_path, "skip") {
                if let Some(ctx_id) = ctx_json
                    .get("verification_context_id")
                    .and_then(Value::as_str)
                {
                    run_context.insert(run_id, ctx_id.to_string());
                }
            }
        }
    }

    // Annotate pairs from parity report
    let pairs = parity
        .get("pairs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let annotated: Vec<Value> = pairs
        .iter()
        .map(|pair| {
            let run_a = pair.get("run_a").and_then(Value::as_str).unwrap_or("");
            let run_b = pair.get("run_b").and_then(Value::as_str).unwrap_or("");
            let ctx_a = run_context.get(run_a);
            let ctx_b = run_context.get(run_b);

            let context_relation = match (ctx_a, ctx_b) {
                (Some(a), Some(b)) if a == b => "same",
                (Some(_), Some(_)) => "different",
                _ => "unknown",
            };

            let mut annotated_pair = pair.clone();
            if let Value::Object(ref mut map) = annotated_pair {
                map.insert(
                    "context_relation".to_string(),
                    Value::String(context_relation.to_string()),
                );
                if let Some(ctx_id) = ctx_a {
                    map.insert(
                        "context_id_run_a".to_string(),
                        Value::String(ctx_id.clone()),
                    );
                }
                if let Some(ctx_id) = ctx_b {
                    map.insert(
                        "context_id_run_b".to_string(),
                        Value::String(ctx_id.clone()),
                    );
                }
            }
            annotated_pair
        })
        .collect();

    let same_count = annotated
        .iter()
        .filter(|p| p.get("context_relation").and_then(Value::as_str) == Some("same"))
        .count();
    let different_count = annotated
        .iter()
        .filter(|p| p.get("context_relation").and_then(Value::as_str) == Some("different"))
        .count();
    let unknown_count = annotated
        .iter()
        .filter(|p| p.get("context_relation").and_then(Value::as_str) == Some("unknown"))
        .count();

    Ok(json!({
        "pair_count": annotated.len(),
        "context_relation_summary": {
            "same": same_count,
            "different": different_count,
            "unknown": unknown_count,
        },
        "pairs": annotated,
    }))
}

fn build_run_context_diagnostics(run_id: &str, run_dir: &Path) -> Result<Value, ServiceError> {
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    let context_object_path = run_dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH);
    if !context_object_path.is_file() {
        return Err(ServiceError::NotFound("artifact_not_found"));
    }

    let declared_context = load_verification_context_object(&context_object_path)
        .map_err(|_| ServiceError::MalformedArtifact("invalid_verification_context_object"))?;
    let policy = load_required_run_json_artifact::<TrustPolicy>(
        &run_dir.join(CONTEXT_POLICY_SNAPSHOT_RELATIVE_PATH),
        "invalid_context_policy_snapshot",
    )?;
    let registry = load_required_run_json_artifact::<RegistrySnapshot>(
        &run_dir.join(CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH),
        "invalid_context_registry_snapshot",
    )?;
    let context_rules = read_required_run_json_artifact(
        &run_dir.join(CONTEXT_RULES_RELATIVE_PATH),
        "invalid_context_rules_object",
    )?;
    let recomputed_policy_hash = compute_policy_hash(&policy)
        .map_err(|_| ServiceError::MalformedArtifact("invalid_context_policy_snapshot"))?;
    let recomputed_registry_snapshot_hash = compute_registry_snapshot_hash(&registry)
        .map_err(|_| ServiceError::MalformedArtifact("invalid_context_registry_snapshot"))?;
    let recomputed_context_rules_hash = compute_context_rules_hash(&context_rules)
        .map_err(|_| ServiceError::MalformedArtifact("invalid_context_rules_object"))?;

    let receipt = load_optional_run_json_artifact::<proof_verifier::VerificationReceipt>(
        &run_dir.join(RECEIPT_RELATIVE_PATH),
        "invalid_receipt_artifact",
    )?;
    let diversity_binding =
        load_optional_run_json_artifact::<VerificationDiversityLedgerProducerManifest>(
            &run_dir.join(VERIFICATION_DIVERSITY_BINDING_FILE),
            "invalid_diversity_binding_manifest",
        )?;
    let diversity_entries = load_optional_run_diversity_ledger_entries(
        &run_dir.join(VERIFICATION_DIVERSITY_LEDGER_FILE),
    )?;
    let replay_boundary_flow_source =
        load_optional_run_json_artifact::<AuthoritySinkholeCompanionSourceDocument>(
            &run_dir.join(REPLAY_BOUNDARY_FLOW_SOURCE_FILE),
            "invalid_context_flow_source",
        )?;
    let trust_reuse_flow_source =
        load_optional_run_json_artifact::<AuthoritySinkholeCompanionSourceDocument>(
            &run_dir.join(TRUST_REUSE_FLOW_SOURCE_FILE),
            "invalid_context_flow_source",
        )?;
    let trust_reuse_runtime_surface_path = run_dir.join(TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH);
    let trust_reuse_runtime_surface = if trust_reuse_runtime_surface_path.is_file() {
        Some(
            load_native_trust_reuse_runtime_surface(&trust_reuse_runtime_surface_path).map_err(
                |_| ServiceError::MalformedArtifact("invalid_trust_reuse_runtime_surface"),
            )?,
        )
    } else {
        None
    };

    let mut observed_context_id_sources = vec![ContextObservationSource {
        source: "declared_context_object",
        source_artifact_path: Some(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH),
        values: vec![declared_context.verification_context_id.clone()],
    }];
    if let Some(receipt) = receipt.as_ref() {
        observed_context_id_sources.push(ContextObservationSource {
            source: "receipt_policy_hash",
            source_artifact_path: Some(RECEIPT_RELATIVE_PATH),
            values: vec![receipt.payload.policy_hash.clone()],
        });
    }
    if !diversity_entries.is_empty() {
        observed_context_id_sources.push(ContextObservationSource {
            source: "verification_diversity_ledger",
            source_artifact_path: Some(VERIFICATION_DIVERSITY_LEDGER_FILE),
            values: unique_sorted_strings(
                diversity_entries
                    .iter()
                    .map(|entry| entry.verification_context_id.clone()),
            ),
        });
    }
    if let Some(document) = replay_boundary_flow_source.as_ref() {
        observed_context_id_sources.push(ContextObservationSource {
            source: "replay_boundary_flow_source",
            source_artifact_path: Some(REPLAY_BOUNDARY_FLOW_SOURCE_FILE),
            values: unique_sorted_strings(
                document
                    .events
                    .iter()
                    .map(|event| event.verification_context_id.clone()),
            ),
        });
    }
    if let Some(document) = trust_reuse_flow_source.as_ref() {
        observed_context_id_sources.push(ContextObservationSource {
            source: "trust_reuse_flow_source",
            source_artifact_path: Some(TRUST_REUSE_FLOW_SOURCE_FILE),
            values: unique_sorted_strings(
                document
                    .events
                    .iter()
                    .map(|event| event.verification_context_id.clone()),
            ),
        });
    }
    if let Some(report) = trust_reuse_runtime_surface.as_ref() {
        observed_context_id_sources.push(ContextObservationSource {
            source: "trust_reuse_runtime_surface",
            source_artifact_path: Some(TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH),
            values: unique_sorted_strings(
                report
                    .events
                    .iter()
                    .map(|event| event.verification_context_id.clone()),
            ),
        });
    }
    observed_context_id_sources.retain(|source| !source.values.is_empty());

    let mut observed_context_ref_sources = Vec::new();
    if let Some(report) = trust_reuse_runtime_surface.as_ref() {
        observed_context_ref_sources.push(ContextObservationSource {
            source: "trust_reuse_runtime_surface",
            source_artifact_path: Some(TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH),
            values: unique_sorted_strings(
                report
                    .events
                    .iter()
                    .map(|event| event.verification_context_ref.clone()),
            ),
        });
    }
    observed_context_ref_sources.retain(|source| !source.values.is_empty());

    let source_artifact_paths = ContextArtifactPaths {
        context_object: VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH,
        context_rules: CONTEXT_RULES_RELATIVE_PATH,
        policy_snapshot: CONTEXT_POLICY_SNAPSHOT_RELATIVE_PATH,
        registry_snapshot: CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH,
        receipt: receipt.as_ref().map(|_| RECEIPT_RELATIVE_PATH),
        diversity_binding: diversity_binding
            .as_ref()
            .map(|_| VERIFICATION_DIVERSITY_BINDING_FILE),
        diversity_ledger: (!diversity_entries.is_empty())
            .then_some(VERIFICATION_DIVERSITY_LEDGER_FILE),
        replay_boundary_flow_source: replay_boundary_flow_source
            .as_ref()
            .map(|_| REPLAY_BOUNDARY_FLOW_SOURCE_FILE),
        trust_reuse_flow_source: trust_reuse_flow_source
            .as_ref()
            .map(|_| TRUST_REUSE_FLOW_SOURCE_FILE),
        trust_reuse_runtime_surface: trust_reuse_runtime_surface
            .as_ref()
            .map(|_| TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH),
    };
    let material_binding_status = ContextMaterialBindingStatus {
        policy_hash_matches_declared_context: recomputed_policy_hash
            == declared_context.policy_hash,
        registry_snapshot_hash_matches_declared_context: recomputed_registry_snapshot_hash
            == declared_context.registry_snapshot_hash,
        context_rules_hash_matches_declared_context: recomputed_context_rules_hash
            == declared_context.context_rules_hash,
        receipt_policy_hash_matches_declared_context: receipt
            .as_ref()
            .map(|receipt| receipt.payload.policy_hash == declared_context.policy_hash),
        receipt_registry_snapshot_hash_matches_declared_context: receipt.as_ref().map(|receipt| {
            receipt.payload.registry_snapshot_hash == declared_context.registry_snapshot_hash
        }),
        legacy_verification_context_id_source: diversity_binding
            .as_ref()
            .map(|binding| binding.verification_context_id_source.clone()),
    };

    serde_json::to_value(ContextDiagnosticsResponseBody {
        run_id: run_id.to_string(),
        source_artifact_paths,
        declared_context,
        material_binding_status,
        observed_context_id_sources,
        observed_context_ref_sources,
    })
    .map_err(|_| ServiceError::Runtime("response_serialize_failed"))
}

fn build_run_registry_diagnostics(run_id: &str, run_dir: &Path) -> Result<Value, ServiceError> {
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    let registry = load_required_run_json_artifact::<RegistrySnapshot>(
        &run_dir.join(CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH),
        "invalid_context_registry_snapshot",
    )?;
    let declared_registry_snapshot_hash = compute_registry_snapshot_hash(&registry)
        .map_err(|_| ServiceError::MalformedArtifact("invalid_context_registry_snapshot"))?;
    let declared_registry_entry_count = registry.producers.len();

    let context_object = load_optional_run_json_artifact::<VerificationContextObject>(
        &run_dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH),
        "invalid_verification_context_object",
    )?;
    let receipt = load_optional_run_json_artifact::<proof_verifier::VerificationReceipt>(
        &run_dir.join(RECEIPT_RELATIVE_PATH),
        "invalid_receipt_artifact",
    )?;

    let context_binding_status = RegistryContextBindingStatus {
        registry_snapshot_hash_matches_declared_context: context_object
            .as_ref()
            .map(|ctx| ctx.registry_snapshot_hash == declared_registry_snapshot_hash),
    };

    let mut observed_registry_hash_sources = Vec::new();

    if let Some(ctx) = context_object.as_ref() {
        let values = unique_sorted_strings(std::iter::once(ctx.registry_snapshot_hash.clone()));
        if !values.is_empty() {
            observed_registry_hash_sources.push(RegistryObservationSource {
                source: "verification_context_object",
                source_artifact_path: Some(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH),
                values,
            });
        }
    }

    if let Some(r) = receipt.as_ref() {
        let values =
            unique_sorted_strings(std::iter::once(r.payload.registry_snapshot_hash.clone()));
        if !values.is_empty() {
            observed_registry_hash_sources.push(RegistryObservationSource {
                source: "receipt",
                source_artifact_path: Some(RECEIPT_RELATIVE_PATH),
                values,
            });
        }
    }

    serde_json::to_value(RegistryDiagnosticsResponseBody {
        run_id: run_id.to_string(),
        source_artifact_path: CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH,
        declared_registry_snapshot_hash,
        declared_registry_entry_count,
        context_binding_status,
        observed_registry_hash_sources,
    })
    .map_err(|_| ServiceError::Runtime("response_serialize_failed"))
}

fn build_run_boundary_diagnostics(
    run_id: &str,
    run_dir: &Path,
    evidence_dir: &Path,
) -> Result<Value, ServiceError> {
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    // Load primary run manifest (fail-closed)
    let manifest = load_required_run_json_artifact::<Value>(
        &run_dir.join(PROOFD_RUN_MANIFEST_FILE),
        "invalid_run_manifest",
    )?;
    let request_fingerprint = manifest
        .get("request_fingerprint")
        .and_then(Value::as_str)
        .ok_or(ServiceError::MalformedArtifact("invalid_run_manifest"))?
        .to_string();

    // Discover peer runs (fail-open for each sibling)
    let mut peer_run_ids: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(evidence_dir) {
        for entry in entries.flatten() {
            let candidate_path = entry.path();
            if !candidate_path.is_dir() {
                continue;
            }
            let candidate_id = entry.file_name().to_string_lossy().to_string();
            if candidate_id == run_id || !is_safe_path_segment(&candidate_id) {
                continue;
            }
            // Silently skip on any error
            let Ok(peer_manifest) = load_required_run_json_artifact::<Value>(
                &candidate_path.join(PROOFD_RUN_MANIFEST_FILE),
                "invalid_run_manifest",
            ) else {
                continue;
            };
            let Some(peer_fp) = peer_manifest
                .get("request_fingerprint")
                .and_then(Value::as_str)
            else {
                continue;
            };
            if peer_fp == request_fingerprint {
                peer_run_ids.push(candidate_id);
            }
        }
    }
    peer_run_ids.sort();

    // Build all_run_ids = primary + peers, sorted by run_id
    let mut all_run_ids: Vec<(String, PathBuf)> =
        std::iter::once((run_id.to_string(), run_dir.to_path_buf()))
            .chain(
                peer_run_ids
                    .iter()
                    .map(|id| (id.clone(), evidence_dir.join(id))),
            )
            .collect();
    all_run_ids.sort_by(|a, b| a.0.cmp(&b.0));

    let projection = collect_boundary_projection_for_runs(&request_fingerprint, &all_run_ids)?;

    serde_json::to_value(json!({
        "run_id": run_id,
        "request_fingerprint": request_fingerprint,
        "peer_run_count": peer_run_ids.len(),
        "peer_run_ids": peer_run_ids,
        "verdict_consistency": projection.verdict_consistency,
        "context_hash_consistency": projection.context_hash_consistency,
        "registry_hash_consistency": projection.registry_hash_consistency,
    }))
    .map_err(|_| ServiceError::Runtime("response_serialize_failed"))
}

/// Fingerprint-based boundary diagnostics: discover all runs matching `request_fingerprint`
/// and compute cross-run consistency without a designated primary run.
/// All runs are treated as peers — no authority assignment.
fn build_fingerprint_boundary_diagnostics(
    request_fingerprint: &str,
    evidence_dir: &Path,
) -> Result<Value, ServiceError> {
    // Validate fingerprint format: must be sha256:<64 hex chars>
    if !is_valid_fingerprint_format(request_fingerprint) {
        return Err(ServiceError::BadRequest("invalid_fingerprint_format"));
    }

    // Discover all runs matching this fingerprint (fail-open per entry)
    let mut matched: Vec<(String, PathBuf)> = Vec::new();
    let entries =
        fs::read_dir(evidence_dir).map_err(|_| ServiceError::NotFound("evidence_dir_not_found"))?;

    for entry in entries.flatten() {
        let candidate_path = entry.path();
        if !candidate_path.is_dir() {
            continue;
        }
        let candidate_id = entry.file_name().to_string_lossy().to_string();
        if !is_safe_path_segment(&candidate_id) {
            continue;
        }
        let Ok(manifest) = load_required_run_json_artifact::<Value>(
            &candidate_path.join(PROOFD_RUN_MANIFEST_FILE),
            "invalid_run_manifest",
        ) else {
            continue;
        };
        let Some(fp) = manifest.get("request_fingerprint").and_then(Value::as_str) else {
            continue;
        };
        if fp == request_fingerprint {
            matched.push((candidate_id, candidate_path));
        }
    }

    if matched.is_empty() {
        return Err(ServiceError::NotFound("fingerprint_not_found"));
    }

    matched.sort_by(|a, b| a.0.cmp(&b.0));
    let run_ids: Vec<String> = matched.iter().map(|(id, _)| id.clone()).collect();

    // Collect context_ids for this fingerprint (fail-open per run)
    let mut context_ids: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (_, run_dir) in &matched {
        let ctx_path = run_dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH);
        if !ctx_path.is_file() {
            continue;
        }
        if let Ok(ctx_json) = load_required_run_json_artifact::<Value>(&ctx_path, "skip") {
            if let Some(ctx_id) = ctx_json
                .get("verification_context_id")
                .and_then(Value::as_str)
            {
                context_ids.insert(ctx_id.to_string());
            }
        }
    }
    let context_ids_sorted: Vec<&str> = context_ids.iter().map(String::as_str).collect();

    let projection = collect_boundary_projection_for_runs(request_fingerprint, &matched)?;

    serde_json::to_value(json!({
        "request_fingerprint": request_fingerprint,
        "run_count": run_ids.len(),
        "run_ids": run_ids,
        "context_ids": context_ids_sorted,
        "context_count": context_ids.len(),
        "verdict_consistency": projection.verdict_consistency,
        "context_hash_consistency": projection.context_hash_consistency,
        "registry_hash_consistency": projection.registry_hash_consistency,
    }))
    .map_err(|_| ServiceError::Runtime("response_serialize_failed"))
}

fn handle_verify_bundle(raw_body: &[u8], evidence_dir: &Path) -> DiagnosticsResponse {
    match verify_bundle_request(raw_body, evidence_dir) {
        Ok(value) => json_response(200, value),
        Err(error) => error_response(error),
    }
}

fn verify_bundle_request(raw_body: &[u8], evidence_dir: &Path) -> Result<Value, ServiceError> {
    let mut request = parse_verify_bundle_request(raw_body)?;
    validate_verify_bundle_request(&request)?;

    let request_fingerprint = compute_verify_bundle_request_fingerprint(&request)?;
    if request.run_id.is_none() {
        request.run_id = Some(generate_run_id()?);
    }
    let run_id = resolved_request_run_id(&request)?.to_string();
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
    let run_dir = evidence_dir.join(&run_id);
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
    let bundle_native_trust_reuse_surface_present =
        trust_reuse_runtime_surface_path(&bundle_path).is_file();
    let mut trust_reuse_runtime_surface_relative_path: Option<String> = None;
    let mut trust_reuse_runtime_surface_origin: Option<String> = None;
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

    write_verification_context_package(&run_dir, &policy, &registry, &outcome)?;
    let determinism_contract =
        build_verification_determinism_contract(&registry, &outcome, &request_fingerprint)?;
    write_canonical_json_file_if_absent_or_same(
        &run_dir.join(VERIFICATION_DETERMINISM_CONTRACT_FILE),
        &determinism_contract,
        "determinism_contract_write_failed",
        "determinism_contract_bytes_conflict",
    )?;

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
                run_dir.join(RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE),
                run_dir.join(TRUST_REUSE_FLOW_SOURCE_FILE),
            ] {
                if required_path.ends_with(RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE)
                    && request.trust_reuse_runtime_binding.is_none()
                {
                    continue;
                }
                if required_path.ends_with(TRUST_REUSE_FLOW_SOURCE_FILE)
                    && !bundle_native_trust_reuse_surface_present
                    && request.trust_reuse_runtime_binding.is_none()
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
        if let Some(runtime_origin) =
            materialize_native_trust_reuse_runtime_surface(&run_dir, &request, &outcome)?
        {
            trust_reuse_runtime_surface_relative_path =
                Some(RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE.to_string());
            trust_reuse_runtime_surface_origin = Some(runtime_origin.to_string());
        }
        if let Some((document, origin)) = build_runtime_trust_reuse_flow_source_document(
            &run_dir,
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
            if let Some((runtime_surface_source_path, runtime_surface_origin, relative_path)) =
                resolve_native_trust_reuse_runtime_surface(&run_dir, &bundle_path)
            {
                copy_file_if_absent_or_same(
                    &runtime_surface_source_path,
                    &run_dir.join(relative_path),
                    "trust_reuse_runtime_surface_copy_failed",
                    "trust_reuse_runtime_surface_bytes_conflict",
                )?;
                if trust_reuse_runtime_surface_relative_path.is_none() {
                    trust_reuse_runtime_surface_relative_path = Some(relative_path.to_string());
                }
                if trust_reuse_runtime_surface_origin.is_none() {
                    trust_reuse_runtime_surface_origin = Some(runtime_surface_origin.to_string());
                }
            }
            trust_reuse_source_relative_path = Some(TRUST_REUSE_FLOW_SOURCE_FILE.to_string());
            trust_reuse_source_origin = Some(origin);
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
        "run_id": run_id.clone(),
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
        "trust_reuse_runtime_surface_path": trust_reuse_runtime_surface_relative_path,
        "trust_reuse_runtime_surface_origin": trust_reuse_runtime_surface_origin,
        "trust_reuse_flow_source_path": trust_reuse_source_relative_path,
        "trust_reuse_flow_source_origin": trust_reuse_source_origin,
        "verification_determinism_contract_path": VERIFICATION_DETERMINISM_CONTRACT_FILE,
        "verification_determinism_artifact_hash": determinism_contract.artifact_hash,
        "request_fingerprint": request_fingerprint,
        "verdict": verdict_label(&outcome.verdict),
        "verdict_subject": outcome.subject,
        "findings_count": outcome.findings.len(),
    });
    persist_run_manifest(
        &run_dir.join(PROOFD_RUN_MANIFEST_FILE),
        &run_manifest,
        &request_fingerprint,
        rerun_same_request,
    )?;

    let response = VerifyBundleResponseBody {
        status: "ok",
        run_id,
        verdict: verdict_label(&outcome.verdict),
        verdict_subject: serde_json::to_value(&outcome.subject).unwrap_or_else(|_| json!({})),
        receipt_emitted: outcome.receipt.is_some(),
        receipt_path: run_manifest
            .get("receipt_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        request_fingerprint,
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
        trust_reuse_runtime_surface_path: run_manifest
            .get("trust_reuse_runtime_surface_path")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        trust_reuse_runtime_surface_origin: run_manifest
            .get("trust_reuse_runtime_surface_origin")
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
        verification_determinism_contract_path: run_manifest
            .get("verification_determinism_contract_path")
            .and_then(Value::as_str)
            .unwrap_or(VERIFICATION_DETERMINISM_CONTRACT_FILE)
            .to_string(),
        verification_determinism_artifact_hash: run_manifest
            .get("verification_determinism_artifact_hash")
            .and_then(Value::as_str)
            .unwrap_or(&determinism_contract.artifact_hash)
            .to_string(),
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

fn parse_replay_determinism_request(
    raw_body: &[u8],
) -> Result<ReplayDeterminismRequestBody, ServiceError> {
    if raw_body.is_empty() {
        return Err(ServiceError::BadRequest("missing_request_body"));
    }
    if raw_body.len() > MAX_VERIFY_BUNDLE_BODY_BYTES {
        return Err(ServiceError::BadRequest("request_body_too_large"));
    }

    serde_json::from_slice(raw_body).map_err(|_| ServiceError::BadRequest("invalid_request_body"))
}

fn validate_verify_bundle_request(request: &VerifyBundleRequestBody) -> Result<(), ServiceError> {
    if let Some(run_id) = request.run_id.as_deref() {
        if run_id.is_empty() || !is_safe_path_segment(run_id) {
            return Err(ServiceError::BadRequest("invalid_run_id"));
        }
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

    if request.trust_reuse_runtime_binding.is_some() && request.diversity_binding.is_none() {
        return Err(ServiceError::BadRequest(
            "trust_reuse_runtime_binding_requires_diversity_binding",
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

    if let Some(binding) = request.trust_reuse_runtime_binding.as_ref() {
        for (label, value) in [
            (
                "verification_context_path",
                binding.verification_context_path.as_str(),
            ),
            (
                "verifier_attestation_path",
                binding.verifier_attestation_path.as_str(),
            ),
            (
                "verifier_registry_path",
                binding.verifier_registry_path.as_str(),
            ),
            ("verifier_key_path", binding.verifier_key_path.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(ServiceError::BadRequest(match label {
                    "verification_context_path" => {
                        "trust_reuse_runtime_binding_verification_context_path_missing"
                    }
                    "verifier_attestation_path" => {
                        "trust_reuse_runtime_binding_verifier_attestation_path_missing"
                    }
                    "verifier_registry_path" => {
                        "trust_reuse_runtime_binding_verifier_registry_path_missing"
                    }
                    "verifier_key_path" => "trust_reuse_runtime_binding_verifier_key_path_missing",
                    _ => "trust_reuse_runtime_binding_path_missing",
                }));
            }
            if !Path::new(value).is_absolute() {
                return Err(ServiceError::BadRequest(match label {
                    "verification_context_path" => {
                        "trust_reuse_runtime_binding_verification_context_path_not_absolute"
                    }
                    "verifier_attestation_path" => {
                        "trust_reuse_runtime_binding_verifier_attestation_path_not_absolute"
                    }
                    "verifier_registry_path" => {
                        "trust_reuse_runtime_binding_verifier_registry_path_not_absolute"
                    }
                    "verifier_key_path" => {
                        "trust_reuse_runtime_binding_verifier_key_path_not_absolute"
                    }
                    _ => "trust_reuse_runtime_binding_path_not_absolute",
                }));
            }
        }

        for (label, value) in [
            ("source_run_id", binding.source_run_id.as_deref()),
            ("reuse_group_id", binding.reuse_group_id.as_deref()),
            (
                "surface_local_path_id",
                binding.surface_local_path_id.as_deref(),
            ),
            ("trust_reuse_source", binding.trust_reuse_source.as_deref()),
        ] {
            if value.is_some_and(|value| value.trim().is_empty()) {
                return Err(ServiceError::BadRequest(match label {
                    "source_run_id" => "trust_reuse_runtime_binding_source_run_id_invalid",
                    "reuse_group_id" => "trust_reuse_runtime_binding_reuse_group_id_invalid",
                    "surface_local_path_id" => {
                        "trust_reuse_runtime_binding_surface_local_path_id_invalid"
                    }
                    "trust_reuse_source" => {
                        "trust_reuse_runtime_binding_trust_reuse_source_invalid"
                    }
                    _ => "trust_reuse_runtime_binding_field_invalid",
                }));
            }
        }
    }

    Ok(())
}

fn validate_replay_determinism_request(
    request: &ReplayDeterminismRequestBody,
) -> Result<(), ServiceError> {
    validate_verify_bundle_request(&request.verify_request)?;
    if let Some(source_run_id) = request.source_run_id.as_deref() {
        if source_run_id.is_empty() || !is_safe_path_segment(source_run_id) {
            return Err(ServiceError::BadRequest("invalid_run_id"));
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
    let mut value = serde_json::to_value(request)
        .map_err(|_| ServiceError::Runtime("request_fingerprint_serialize_failed"))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("run_id");
    }
    canonical_hash_value_prefixed(&value, "request_fingerprint_serialize_failed")
}

fn generate_run_id() -> Result<String, ServiceError> {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ServiceError::Runtime("run_id_generation_failed"))?
        .as_nanos();
    let counter = GENERATED_RUN_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(format!("run-{timestamp_nanos:032x}-{counter:016x}"))
}

fn resolved_request_run_id(request: &VerifyBundleRequestBody) -> Result<&str, ServiceError> {
    request
        .run_id
        .as_deref()
        .ok_or(ServiceError::Runtime("run_id_not_resolved"))
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
        return Err(ServiceError::Conflict("run_id_fingerprint_conflict"));
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
        run_id: resolved_request_run_id(request)?.to_string(),
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
        run_id: resolved_request_run_id(request)?.to_string(),
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
        run_id: resolved_request_run_id(request)?.to_string(),
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
                    .unwrap_or_else(replay_report_surface_path_id),
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
        run_id: resolved_request_run_id(request)?.to_string(),
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
    run_dir: &Path,
    bundle_path: &Path,
    request: &VerifyBundleRequestBody,
    outcome: &proof_verifier::types::VerificationOutcome,
    binding: Option<&VerifyBundleTrustReuseBinding>,
) -> Result<Option<(AuthoritySinkholeCompanionSourceDocument, String)>, ServiceError> {
    let Some((runtime_surface_path, origin, default_surface_local_path)) =
        resolve_native_trust_reuse_runtime_surface(run_dir, bundle_path)
    else {
        return Ok(None);
    };

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
                default_surface_local_path,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some((
        AuthoritySinkholeCompanionSourceDocument {
            source_version: 1,
            flow_surface: "trust_reuse".to_string(),
            status: if events.is_empty() {
                "NO_REUSABLE_EVENTS".to_string()
            } else {
                "PASS".to_string()
            },
            run_id: resolved_request_run_id(request)?.to_string(),
            window_model: default_companion_window_model(),
            events,
        },
        origin.to_string(),
    )))
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

fn run_trust_reuse_runtime_surface_path(run_dir: &Path) -> PathBuf {
    run_dir.join(RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE)
}

fn resolve_native_trust_reuse_runtime_surface(
    run_dir: &Path,
    bundle_path: &Path,
) -> Option<(PathBuf, &'static str, &'static str)> {
    let run_local_path = run_trust_reuse_runtime_surface_path(run_dir);
    if run_local_path.is_file() {
        return Some((
            run_local_path,
            "runtime_proofd_trust_reuse",
            RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE,
        ));
    }

    let bundle_runtime_path = trust_reuse_runtime_surface_path(bundle_path);
    if bundle_runtime_path.is_file() {
        return Some((
            bundle_runtime_path,
            "runtime_bundle_trust_reuse",
            TRUST_REUSE_RUNTIME_SURFACE_RELATIVE_PATH,
        ));
    }

    None
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
    default_surface_local_path: &str,
) -> Result<AuthoritySinkholeCompanionSourceEvent, ServiceError> {
    if !event.terminal || !event.reused {
        return Err(ServiceError::Runtime("trust_reuse_runtime_surface_invalid"));
    }
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
            .or_else(|| Some(default_surface_local_path.to_string())),
    })
}

fn materialize_native_trust_reuse_runtime_surface(
    run_dir: &Path,
    request: &VerifyBundleRequestBody,
    outcome: &proof_verifier::types::VerificationOutcome,
) -> Result<Option<&'static str>, ServiceError> {
    let Some(binding) = request.trust_reuse_runtime_binding.as_ref() else {
        return Ok(None);
    };

    let receipt = outcome.receipt.as_ref().ok_or(ServiceError::Runtime(
        "signed_receipt_missing_for_trust_reuse_runtime_surface",
    ))?;
    let timestamp_unix_ns = parse_event_time_to_unix_ns(&receipt.payload.verified_at_utc)
        .map_err(|_| ServiceError::Runtime("trust_reuse_runtime_surface_timestamp_invalid"))?;
    let evaluator_dir = run_dir.join(TRUST_REUSE_RUNTIME_EVALUATOR_DIR);
    let expected_subject_path = evaluator_dir.join(TRUST_REUSE_RUNTIME_EXPECTED_SUBJECT_FILE);
    write_json_file_if_absent_or_same(
        &expected_subject_path,
        &outcome.subject,
        "trust_reuse_runtime_expected_subject_write_failed",
        "trust_reuse_runtime_expected_subject_bytes_conflict",
    )?;
    let output_path = run_trust_reuse_runtime_surface_path(run_dir);
    let config = TrustReuseRuntimeEvaluatorConfig {
        receipt_path: run_dir.join("receipts").join("verification_receipt.json"),
        verifier_key_path: PathBuf::from(&binding.verifier_key_path),
        expected_subject_path,
        verification_context_path: PathBuf::from(&binding.verification_context_path),
        verifier_attestation_path: PathBuf::from(&binding.verifier_attestation_path),
        verifier_registry_path: PathBuf::from(&binding.verifier_registry_path),
        output_path,
        output_dir: evaluator_dir,
        run_id: resolved_request_run_id(request)?.to_string(),
        timestamp_unix_ns,
        source_run_id: binding.source_run_id.clone(),
        execution_cluster_id: request
            .diversity_binding
            .as_ref()
            .and_then(|value| value.execution_cluster_id.clone()),
        lineage_id: request
            .diversity_binding
            .as_ref()
            .map(|value| value.lineage_id.clone()),
        reuse_group_id: binding.reuse_group_id.clone(),
        surface_local_path_id: Some(
            binding
                .surface_local_path_id
                .clone()
                .unwrap_or_else(|| RUN_LEVEL_TRUST_REUSE_RUNTIME_SURFACE_FILE.to_string()),
        ),
        trust_reuse_source: binding.trust_reuse_source.clone(),
    };
    run_trust_reuse_runtime_evaluator(&config)
        .map_err(|_| ServiceError::Runtime("trust_reuse_runtime_evaluator_runtime_failure"))?;
    Ok(Some("runtime_proofd_trust_reuse"))
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

fn persist_run_manifest(
    path: &Path,
    value: &Value,
    request_fingerprint: &str,
    rerun_same_request: bool,
) -> Result<(), ServiceError> {
    if rerun_same_request {
        return write_json_value_if_absent_or_same(
            path,
            value,
            "run_manifest_write_failed",
            "run_manifest_bytes_conflict",
        );
    }
    create_run_manifest_atomically(
        path,
        value,
        request_fingerprint,
        "run_manifest_write_failed",
    )
}

fn create_run_manifest_atomically(
    path: &Path,
    value: &Value,
    request_fingerprint: &str,
    write_error: &'static str,
) -> Result<(), ServiceError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| ServiceError::Runtime(write_error))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| ServiceError::Runtime(write_error))?;
    }

    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => file
            .write_all(&bytes)
            .map_err(|_| ServiceError::Runtime(write_error)),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            if let Ok(manifest) = read_json_file(path) {
                let _same_fingerprint = manifest
                    .get("request_fingerprint")
                    .and_then(Value::as_str)
                    .is_some_and(|existing_fingerprint| {
                        existing_fingerprint == request_fingerprint
                    });
            }
            Err(ServiceError::Conflict("run_id_fingerprint_conflict"))
        }
        Err(_) => Err(ServiceError::Runtime(write_error)),
    }
}

fn copy_file_if_absent_or_same(
    source: &Path,
    target: &Path,
    write_error: &'static str,
    conflict_error: &'static str,
) -> Result<(), ServiceError> {
    let bytes = fs::read(source).map_err(|_| ServiceError::Runtime(write_error))?;
    write_bytes_if_absent_or_same(target, &bytes, write_error, conflict_error)
}

fn build_default_context_rules_object() -> Value {
    json!({
        "rules_version": 1,
        "policy_import_mode": "external-only",
        "registry_import_mode": "external-only",
        "context_mismatch_mode": "fail-closed",
        "historical_receipt_mode": "historical-only",
        "receipt_acceptance_mode": "context-bound-only",
    })
}

fn compute_context_rules_hash(context_rules: &Value) -> Result<String, ServiceError> {
    let bytes = canonicalize_json_value(context_rules)
        .map_err(|_| ServiceError::Runtime("context_rules_hash_compute_failed"))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(encode_lower_hex(&hasher.finalize()))
}

fn write_verification_context_package(
    run_dir: &Path,
    policy: &TrustPolicy,
    registry: &RegistrySnapshot,
    outcome: &proof_verifier::types::VerificationOutcome,
) -> Result<(), ServiceError> {
    let (context_rules, context) = build_verification_context_material(outcome)?;

    write_canonical_json_file_if_absent_or_same(
        &run_dir.join(CONTEXT_POLICY_SNAPSHOT_RELATIVE_PATH),
        policy,
        "context_policy_snapshot_write_failed",
        "context_policy_snapshot_bytes_conflict",
    )?;
    write_canonical_json_file_if_absent_or_same(
        &run_dir.join(CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH),
        registry,
        "context_registry_snapshot_write_failed",
        "context_registry_snapshot_bytes_conflict",
    )?;
    write_canonical_json_value_if_absent_or_same(
        &run_dir.join(CONTEXT_RULES_RELATIVE_PATH),
        &context_rules,
        "context_rules_write_failed",
        "context_rules_bytes_conflict",
    )?;
    write_canonical_json_file_if_absent_or_same(
        &run_dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH),
        &context,
        "verification_context_object_write_failed",
        "verification_context_object_bytes_conflict",
    )
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

fn load_required_run_json_artifact<T>(
    path: &Path,
    invalid_error: &'static str,
) -> Result<T, ServiceError>
where
    T: serde::de::DeserializeOwned,
{
    if !path.is_file() {
        return Err(ServiceError::NotFound("artifact_not_found"));
    }
    let bytes = fs::read(path).map_err(|_| ServiceError::MalformedArtifact(invalid_error))?;
    serde_json::from_slice(&bytes).map_err(|_| ServiceError::MalformedArtifact(invalid_error))
}

fn load_optional_run_json_artifact<T>(
    path: &Path,
    invalid_error: &'static str,
) -> Result<Option<T>, ServiceError>
where
    T: serde::de::DeserializeOwned,
{
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|_| ServiceError::MalformedArtifact(invalid_error))?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|_| ServiceError::MalformedArtifact(invalid_error))?;
    Ok(Some(value))
}

fn read_required_run_json_artifact(
    path: &Path,
    invalid_error: &'static str,
) -> Result<Value, ServiceError> {
    if !path.is_file() {
        return Err(ServiceError::NotFound("artifact_not_found"));
    }
    let bytes = fs::read(path).map_err(|_| ServiceError::MalformedArtifact(invalid_error))?;
    serde_json::from_slice(&bytes).map_err(|_| ServiceError::MalformedArtifact(invalid_error))
}

fn load_optional_run_diversity_ledger_entries(
    path: &Path,
) -> Result<Vec<VerificationDiversityLedgerEntry>, ServiceError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    load_diversity_ledger_entries(path)
        .map_err(|_| ServiceError::MalformedArtifact("invalid_federation_ledger"))
}

fn unique_sorted_strings<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
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

fn list_run_artifact_paths(run_dir: &Path) -> Result<Vec<String>, ServiceError> {
    if !run_dir.is_dir() {
        return Err(ServiceError::NotFound("run_dir_not_found"));
    }

    let mut artifact_paths = RUN_LEVEL_ARTIFACTS
        .iter()
        .chain(NESTED_RUN_LEVEL_ARTIFACTS.iter())
        .filter_map(|relative_path| {
            run_dir
                .join(relative_path)
                .is_file()
                .then_some((*relative_path).to_string())
        })
        .collect::<Vec<_>>();
    artifact_paths.sort();
    artifact_paths.dedup();
    Ok(artifact_paths)
}

fn list_run_artifact_descriptors(
    run_dir: &Path,
) -> Result<Vec<RunArtifactDescriptor>, ServiceError> {
    Ok(list_run_artifact_paths(run_dir)?
        .into_iter()
        .map(|path| RunArtifactDescriptor {
            content_type: artifact_content_type(&path),
            path,
        })
        .collect())
}

fn parse_run_artifact_path<'a>(artifact_path: &'a str) -> Result<&'a str, ServiceError> {
    if artifact_path.is_empty() {
        return Err(ServiceError::Forbidden("artifact_path_not_allowed"));
    }
    // Task 4.3: ".." and "." segments must be rejected with 403 (path traversal guard)
    if artifact_path
        .split('/')
        .any(|segment| !is_safe_path_segment(segment))
    {
        return Err(ServiceError::Forbidden("artifact_path_not_allowed"));
    }
    Ok(artifact_path)
}

fn resolve_run_artifact_path(run_dir: &Path, artifact_path: &str) -> Result<PathBuf, ServiceError> {
    // Task 4.1: Two-phase check — Allowed_Artifact_Set (403) then disk existence (404)
    let allowed: std::collections::HashSet<&str> = RUN_LEVEL_ARTIFACTS
        .iter()
        .chain(NESTED_RUN_LEVEL_ARTIFACTS.iter())
        .copied()
        .collect();
    if !allowed.contains(artifact_path) {
        return Err(ServiceError::Forbidden("artifact_path_not_allowed"));
    }
    let full_path = run_dir.join(artifact_path);
    if !full_path.is_file() {
        return Err(ServiceError::NotFound("artifact_not_found"));
    }
    Ok(full_path)
}

fn build_federation_distribution<F>(
    entries: &[VerificationDiversityLedgerEntry],
    key_fn: F,
) -> Vec<FederationDistributionEntry>
where
    F: Fn(&VerificationDiversityLedgerEntry) -> String,
{
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for entry in entries {
        let id = key_fn(entry);
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(id, entry_count)| FederationDistributionEntry { id, entry_count })
        .collect()
}

fn build_optional_federation_distribution<F>(
    entries: &[VerificationDiversityLedgerEntry],
    key_fn: F,
) -> (Vec<FederationDistributionEntry>, usize)
where
    F: Fn(&VerificationDiversityLedgerEntry) -> Option<String>,
{
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    let mut missing_count = 0usize;
    for entry in entries {
        match key_fn(entry) {
            Some(id) => *counts.entry(id).or_insert(0) += 1,
            None => missing_count += 1,
        }
    }
    let distribution = counts
        .into_iter()
        .map(|(id, entry_count)| FederationDistributionEntry { id, entry_count })
        .collect::<Vec<_>>();
    (distribution, missing_count)
}

fn is_safe_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && !segment.contains('/')
        && !segment.contains('\\')
}

/// Validates fingerprint format: must be `sha256:` followed by exactly 64 lowercase hex chars.
fn is_valid_fingerprint_format(fp: &str) -> bool {
    fp.strip_prefix("sha256:")
        .map(|hex| hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or(false)
}

/// Shared boundary projection: given a sorted list of (run_id, run_dir) pairs,
/// compute verdict/context/registry consistency observations.
/// Discovery is the caller's responsibility — this function only projects.
struct BoundaryProjection {
    verdict_consistency: Value,
    context_hash_consistency: Value,
    registry_hash_consistency: Value,
}

fn collect_boundary_projection_for_runs(
    _request_fingerprint: &str,
    runs: &[(String, PathBuf)],
) -> Result<BoundaryProjection, ServiceError> {
    // observed_verdicts (fail-open per run)
    let mut observed_verdicts: Vec<RunVerdictEntry> = Vec::new();
    for (rid, rdir) in runs {
        let Ok(manifest) = load_required_run_json_artifact::<Value>(
            &rdir.join(PROOFD_RUN_MANIFEST_FILE),
            "invalid_run_manifest",
        ) else {
            continue;
        };
        let Some(verdict) = manifest.get("verdict").and_then(Value::as_str) else {
            continue;
        };
        observed_verdicts.push(RunVerdictEntry {
            run_id: rid.clone(),
            verdict: verdict.to_string(),
        });
    }

    // observed_context_hashes (fail-open per run)
    let mut observed_context_hashes: Vec<RunHashEntry> = Vec::new();
    for (rid, rdir) in runs {
        let ctx_path = rdir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH);
        if !ctx_path.is_file() {
            continue;
        }
        let Ok(ctx) = load_required_run_json_artifact::<Value>(&ctx_path, "skip") else {
            continue;
        };
        let Some(hash) = ctx.get("verification_context_id").and_then(Value::as_str) else {
            continue;
        };
        observed_context_hashes.push(RunHashEntry {
            run_id: rid.clone(),
            hash: hash.to_string(),
        });
    }

    // observed_registry_hashes (recompute — never trust self-declared field)
    let mut observed_registry_hashes: Vec<RunHashEntry> = Vec::new();
    for (rid, rdir) in runs {
        let reg_path = rdir.join(CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH);
        if !reg_path.is_file() {
            continue;
        }
        let Ok(snapshot) = load_required_run_json_artifact::<RegistrySnapshot>(&reg_path, "skip")
        else {
            continue;
        };
        let Ok(hash) = compute_registry_snapshot_hash(&snapshot) else {
            continue;
        };
        observed_registry_hashes.push(RunHashEntry {
            run_id: rid.clone(),
            hash,
        });
    }

    let all_verdicts_match = observed_verdicts
        .windows(2)
        .all(|w| w[0].verdict == w[1].verdict);

    let all_context_hashes_match = if observed_context_hashes.is_empty() {
        None
    } else {
        Some(
            observed_context_hashes
                .windows(2)
                .all(|w| w[0].hash == w[1].hash),
        )
    };

    let all_registry_hashes_match = if observed_registry_hashes.is_empty() {
        None
    } else {
        Some(
            observed_registry_hashes
                .windows(2)
                .all(|w| w[0].hash == w[1].hash),
        )
    };

    Ok(BoundaryProjection {
        verdict_consistency: json!({
            "all_verdicts_match": all_verdicts_match,
            "observed_verdicts": observed_verdicts,
        }),
        context_hash_consistency: json!({
            "all_context_hashes_match": all_context_hashes_match,
            "observed_context_hashes": observed_context_hashes,
        }),
        registry_hash_consistency: json!({
            "all_registry_hashes_match": all_registry_hashes_match,
            "observed_registry_hashes": observed_registry_hashes,
        }),
    })
}

fn is_observability_path(path: &str) -> bool {
    path == "/diagnostics" || path.starts_with("/diagnostics/")
}

fn artifact_content_type(path: &str) -> &'static str {
    if path.ends_with(".jsonl") {
        "application/x-ndjson; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

fn serve_artifact_file(path: PathBuf, content_type: &'static str) -> DiagnosticsResponse {
    match fs::read(path) {
        Ok(body) => DiagnosticsResponse {
            status_code: 200,
            body,
            content_type,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            error_response(ServiceError::NotFound("artifact_not_found"))
        }
        Err(_) => error_response(ServiceError::Runtime("artifact_read_failed")),
    }
}

fn serve_observability_json_file(endpoint: &str, path: PathBuf) -> DiagnosticsResponse {
    match read_json_file(&path) {
        Ok(value) => observability_json_response(endpoint, 200, value),
        Err(error) => error_response(error),
    }
}

fn observability_json_response(
    endpoint: &str,
    status_code: u16,
    value: Value,
) -> DiagnosticsResponse {
    let hits = scan_forbidden_observability_fields(endpoint, &value);
    if !hits.is_empty() {
        return error_response(ServiceError::Runtime(
            "forbidden_observability_field_exposed",
        ));
    }
    if let Err(error) = validate_response_schema_for_path(endpoint, &value) {
        return error_response(ServiceError::Runtime(error.reason_code()));
    }
    json_response(status_code, value)
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
        ServiceError::Conflict(code) => json_response(409, json!({ "error": code })),
        ServiceError::Forbidden(code) => json_response(403, json!({ "error": code })),
        ServiceError::NotFound(code) => json_response(404, json!({ "error": code })),
        ServiceError::MalformedArtifact(code) => json_response(500, json!({ "error": code })),
        ServiceError::Runtime(code) => json_response(500, json!({ "error": code })),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceError {
    BadRequest(&'static str),
    Conflict(&'static str),
    Forbidden(&'static str),
    NotFound(&'static str),
    MalformedArtifact(&'static str),
    Runtime(&'static str),
}

#[cfg(test)]
mod tests {
    use super::{
        compute_verify_bundle_request_fingerprint, create_run_manifest_atomically,
        observability_json_response, parse_verify_bundle_request, route_request,
        route_request_with_body, DiagnosticsResponse, ServiceError, MAX_VERIFY_BUNDLE_BODY_BYTES,
    };
    use proof_verifier::canonical::jcs::canonicalize_json_value;
    use proof_verifier::crypto::ed25519::sign_ed25519_bytes;
    use proof_verifier::testing::fixtures::create_fixture_bundle;
    use proof_verifier::trust_reuse_runtime_surface::{
        compute_trust_reuse_runtime_event_id, TrustReuseOutcome, TrustReuseRuntimeEvent,
        TrustReuseRuntimeSurfaceReport,
    };
    use proof_verifier::types::{
        AuditMode, ReceiptMode, VerifierTrustRegistrySnapshot, VerifyRequest,
    };
    use proof_verifier::verification_context_object::{
        compute_verification_context_id, write_verification_context_object,
        VerificationContextObject,
    };
    use proof_verifier::verifier_attestation::{write_verifier_attestation, VerifierAttestation};
    use proof_verifier::verify_bundle;
    use serde::Serialize;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-test")
    }

    fn write_artifact(dir: &PathBuf, name: &str, body: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create artifact parent");
        }
        fs::write(path, body).expect("write artifact");
    }

    fn write_diversity_ledger(run_dir: &PathBuf, run_id: &str, verifier_id: &str) {
        let entry = serde_json::json!({
            "entries": [{
                "ledger_version": 1,
                "entry_id": format!("entry-{run_id}"),
                "run_id": run_id,
                "timestamp_unix_ns": 1000000000u64,
                "subject_bundle_id": "bundle-test",
                "verification_context_id": format!("ctx-{run_id}"),
                "verification_node_id": "node-test",
                "verifier_id": verifier_id,
                "authority_chain_id": "chain-test",
                "lineage_id": "lineage-test",
                "execution_cluster_id": null,
                "verdict": "PASS",
                "receipt_hash": "sha256:aabbcc"
            }]
        });
        write_artifact(
            run_dir,
            "verification_diversity_ledger.json",
            &entry.to_string(),
        );
    }

    fn write_context_object_simple(run_dir: &PathBuf, context_id: &str) {
        // Minimal verification_context_object.json for context propagation tests
        let obj = serde_json::json!({
            "context_version": 1,
            "verification_context_id": context_id,
            "policy_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "registry_snapshot_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "context_rules_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "verifier_contract_version": "phase12-context-v1"
        });
        write_artifact(
            run_dir,
            "context/verification_context_object.json",
            &obj.to_string(),
        );
    }

    fn write_manifest_simple(run_dir: &PathBuf, fingerprint: &str, verdict: &str) {
        write_artifact(
            run_dir,
            "proofd_run_manifest.json",
            &serde_json::json!({
                "run_id": run_dir.file_name().unwrap_or_default().to_string_lossy(),
                "request_fingerprint": fingerprint,
                "verdict": verdict
            })
            .to_string(),
        );
    }

    fn write_json<T>(path: &std::path::Path, value: &T)
    where
        T: Serialize,
    {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create json parent");
        }
        fs::write(
            path,
            serde_json::to_vec_pretty(value).expect("serialize json"),
        )
        .expect("write json");
    }

    fn build_verification_context(
        subject: &proof_verifier::VerdictSubject,
    ) -> VerificationContextObject {
        let mut context = VerificationContextObject {
            context_version: 1,
            verification_context_id: String::new(),
            policy_hash: subject.policy_hash.clone(),
            registry_snapshot_hash: subject.registry_snapshot_hash.clone(),
            verifier_contract_version: "phase12-context-v1".to_string(),
            context_rules_hash: "c".repeat(64),
            context_epoch: Some(1),
            historical_cutoff_utc: None,
            policy_snapshot_ref: Some(format!("cas:sha256:{}", "d".repeat(64))),
            registry_snapshot_ref: Some(format!("cas:sha256:{}", "e".repeat(64))),
            time_semantics_mode: Some("historical-aware".to_string()),
        };
        context.verification_context_id =
            compute_verification_context_id(&context).expect("compute verification context id");
        context
    }

    fn build_verifier_attestation(
        fixture: &proof_verifier::testing::fixtures::FixtureBundle,
    ) -> VerifierAttestation {
        let mut attestation = VerifierAttestation {
            attestation_version: 1,
            verifier_id: fixture.receipt_signer.verifier_node_id.clone(),
            verifier_pubkey_id: fixture.receipt_signer.verifier_key_id.clone(),
            verifier_registry_ref: fixture.verifier_registry.registry_scope.clone(),
            verifier_key_epoch: u64::from(fixture.verifier_registry.verifier_registry_epoch),
            verifier_contract_version: "phase12-context-v1".to_string(),
            attestation_signature_algorithm: "ed25519".to_string(),
            attestation_signature: String::new(),
            attested_at_utc: Some("2026-03-14T00:00:00Z".to_string()),
        };
        let mut payload = serde_json::to_value(&attestation).expect("serialize attestation");
        payload
            .as_object_mut()
            .expect("attestation object")
            .remove("attestation_signature");
        let payload_bytes =
            canonicalize_json_value(&payload).expect("canonicalize attestation payload");
        attestation.attestation_signature =
            sign_ed25519_bytes(&fixture.receipt_signer.private_key, &payload_bytes)
                .expect("sign attestation");
        attestation
    }

    fn write_trust_reuse_runtime_inputs(
        fixture: &proof_verifier::testing::fixtures::FixtureBundle,
        subject: &proof_verifier::VerdictSubject,
        dir: &std::path::Path,
        verifier_registry: &VerifierTrustRegistrySnapshot,
    ) -> (PathBuf, PathBuf, PathBuf) {
        let verification_context_path = dir.join("verification_context_object.json");
        let verifier_attestation_path = dir.join("verifier_attestation.json");
        let verifier_registry_path = dir.join("verifier_registry.json");
        let context = build_verification_context(subject);
        write_verification_context_object(&verification_context_path, &context)
            .expect("write verification context");
        let attestation = build_verifier_attestation(fixture);
        write_verifier_attestation(&verifier_attestation_path, &attestation)
            .expect("write verifier attestation");
        write_json(&verifier_registry_path, verifier_registry);
        (
            verification_context_path,
            verifier_attestation_path,
            verifier_registry_path,
        )
    }

    fn body_json(response: DiagnosticsResponse) -> serde_json::Value {
        serde_json::from_slice(&response.body).expect("valid json body")
    }

    #[test]
    fn diagnostics_version_endpoint_returns_phase14_contract() {
        let dir = temp_dir();

        let response = route_request("GET", "/diagnostics/version", &dir);
        assert_eq!(response.status_code, 200);

        let body = body_json(response);
        assert_eq!(body.get("api_version").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(body.get("service").and_then(|v| v.as_str()), Some("proofd"));
        assert!(body.get("phase").is_none());
        assert_eq!(
            body.get("contract").and_then(|v| v.as_str()),
            Some("read-only diagnostics surface")
        );
        assert!(body
            .get("invariants")
            .and_then(|v| v.as_array())
            .is_some_and(|items| items
                .iter()
                .any(|item| item.as_str() == Some("service != authority"))));
        assert!(body
            .get("endpoints")
            .and_then(|v| v.as_array())
            .is_some_and(|items| {
                let actual = items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .collect::<Vec<_>>();
                let expected = super::api_contract::public_endpoint_declarations();
                actual.len() == expected.len()
                    && actual
                        .iter()
                        .zip(expected.iter())
                        .all(|(left, right)| *left == right.as_str())
            }));
        assert!(body
            .get("endpoints")
            .and_then(|v| v.as_array())
            .is_some_and(|items| items
                .iter()
                .all(|item| item.as_str().is_some_and(|endpoint| {
                    endpoint == "GET /healthz" || endpoint.starts_with("GET /diagnostics/")
                }))));
        assert!(body
            .get("endpoints")
            .and_then(|v| v.as_array())
            .is_some_and(|items| items.iter().all(|item| item
                .as_str()
                .is_some_and(|endpoint| !endpoint.contains("/verify")))));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diagnostics_version_schema_violation_fails_closed() {
        let response = observability_json_response(
            "/diagnostics/version",
            200,
            json!({
                "api_version": 1,
                "service": "proofd",
                "contract": "read-only diagnostics surface",
                "invariants": [],
            }),
        );
        assert_eq!(response.status_code, 500);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|value| value.as_str()),
            Some("diagnostics_schema_contract_violation")
        );
    }

    #[test]
    fn diagnostics_version_schema_allows_unknown_fields() {
        let response = observability_json_response(
            "/diagnostics/version",
            200,
            json!({
                "api_version": 1,
                "service": "proofd",
                "contract": "read-only diagnostics surface",
                "invariants": [],
                "endpoints": [],
                "extra_field": {"forward_compatible": true},
            }),
        );
        assert_eq!(response.status_code, 200);
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
    fn parity_endpoint_fail_closes_when_artifact_exposes_forbidden_field() {
        let dir = temp_dir();
        write_artifact(
            &dir,
            "parity_report.json",
            r#"{"status":"PASS","routing_hint":"forbidden"}"#,
        );

        let response = route_request("GET", "/diagnostics/parity", &dir);
        assert_eq!(response.status_code, 500);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("forbidden_observability_field_exposed")
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
    fn run_summary_endpoint_includes_nested_artifact_paths() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "proofd_run_manifest.json",
            r#"{"run_id":"run-20260310-1"}"#,
        );
        write_artifact(
            &run_dir,
            "receipts/verification_receipt.json",
            r#"{"status":"signed"}"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert!(body
            .get("artifact_paths")
            .and_then(|v| v.as_array())
            .is_some_and(|paths| paths
                .iter()
                .any(|item| item.as_str() == Some("receipts/verification_receipt.json"))));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_artifacts_endpoint_lists_canonical_paths_with_content_types() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "proofd_run_manifest.json",
            r#"{"run_id":"run-20260310-1"}"#,
        );
        write_artifact(
            &run_dir,
            "receipts/verification_receipt.json",
            r#"{"status":"signed"}"#,
        );
        write_artifact(
            &run_dir,
            "verification_audit_ledger.jsonl",
            "{\"event_id\":\"1\"}\n",
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/artifacts", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("run_id").and_then(|v| v.as_str()),
            Some("run-20260310-1")
        );
        assert_eq!(body.get("artifact_count").and_then(|v| v.as_u64()), Some(3));
        let artifacts = body
            .get("artifacts")
            .and_then(|v| v.as_array())
            .expect("artifacts array");
        assert!(artifacts.iter().any(|artifact| {
            artifact.get("path").and_then(|v| v.as_str()) == Some("proofd_run_manifest.json")
                && artifact.get("content_type").and_then(|v| v.as_str())
                    == Some("application/json; charset=utf-8")
        }));
        assert!(artifacts.iter().any(|artifact| {
            artifact.get("path").and_then(|v| v.as_str())
                == Some("receipts/verification_receipt.json")
                && artifact.get("content_type").and_then(|v| v.as_str())
                    == Some("application/json; charset=utf-8")
        }));
        assert!(artifacts.iter().any(|artifact| {
            artifact.get("path").and_then(|v| v.as_str()) == Some("verification_audit_ledger.jsonl")
                && artifact.get("content_type").and_then(|v| v.as_str())
                    == Some("application/x-ndjson; charset=utf-8")
        }));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_artifact_endpoint_serves_selected_json_and_jsonl_artifacts() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "receipts/verification_receipt.json",
            r#"{"status":"signed","verifier_key_id":"k1"}"#,
        );
        write_artifact(
            &run_dir,
            "verification_audit_ledger.jsonl",
            "{\"event_id\":\"1\"}\n{\"event_id\":\"2\"}\n",
        );

        let receipt = route_request(
            "GET",
            "/diagnostics/runs/run-20260310-1/artifacts/receipts/verification_receipt.json",
            &dir,
        );
        assert_eq!(receipt.status_code, 200);
        assert_eq!(receipt.content_type, "application/json; charset=utf-8");
        let receipt_body = body_json(receipt);
        assert_eq!(
            receipt_body.get("verifier_key_id").and_then(|v| v.as_str()),
            Some("k1")
        );

        let ledger = route_request(
            "GET",
            "/diagnostics/runs/run-20260310-1/artifacts/verification_audit_ledger.jsonl",
            &dir,
        );
        assert_eq!(ledger.status_code, 200);
        assert_eq!(ledger.content_type, "application/x-ndjson; charset=utf-8");
        let ledger_body = String::from_utf8(ledger.body).expect("utf8 ledger");
        assert_eq!(ledger_body, "{\"event_id\":\"1\"}\n{\"event_id\":\"2\"}\n");

        let _ = fs::remove_dir_all(&dir);
    }

    // Task 4.4: updated to expect 403 artifact_path_not_allowed for traversal segments
    #[test]
    fn run_artifact_endpoint_rejects_invalid_relative_path() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "proofd_run_manifest.json",
            r#"{"run_id":"run-20260310-1"}"#,
        );

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-20260310-1/artifacts/../proofd_run_manifest.json",
            &dir,
        );
        assert_eq!(response.status_code, 403);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_path_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // Task 6.9: updated to use new projection field names (verifier_count, observed_verifiers,
    // authority_chain_distribution with authority_chain_id, execution_cluster_distribution with cluster_id)
    #[test]
    fn run_scoped_federation_endpoint_summarizes_diversity_ledger() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "verification_diversity_ledger.json",
            r#"{
              "entries": [
                {
                  "ledger_version": 1,
                  "entry_id": "entry-a",
                  "run_id": "run-20260310-1",
                  "timestamp_unix_ns": 10,
                  "subject_bundle_id": "bundle-a",
                  "verification_context_id": "ctx-a",
                  "verification_node_id": "node-a",
                  "verifier_id": "verifier-a",
                  "authority_chain_id": "chain-a",
                  "lineage_id": "lineage-a",
                  "execution_cluster_id": "cluster-a",
                  "verdict": "PASS",
                  "receipt_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                },
                {
                  "ledger_version": 1,
                  "entry_id": "entry-b",
                  "run_id": "run-20260310-1",
                  "timestamp_unix_ns": 20,
                  "subject_bundle_id": "bundle-b",
                  "verification_context_id": "ctx-b",
                  "verification_node_id": "node-b",
                  "verifier_id": "verifier-b",
                  "authority_chain_id": "chain-a",
                  "lineage_id": "lineage-b",
                  "execution_cluster_id": "cluster-a",
                  "verdict": "PASS",
                  "receipt_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                },
                {
                  "ledger_version": 1,
                  "entry_id": "entry-c",
                  "run_id": "run-20260310-1",
                  "timestamp_unix_ns": 30,
                  "subject_bundle_id": "bundle-c",
                  "verification_context_id": "ctx-c",
                  "verification_node_id": "node-c",
                  "verifier_id": "verifier-a",
                  "authority_chain_id": "chain-b",
                  "lineage_id": "lineage-a",
                  "verdict": "FAIL",
                  "receipt_hash": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                }
              ]
            }"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        // Projection fields (Requirement 4)
        assert_eq!(
            body.get("run_id").and_then(|v| v.as_str()),
            Some("run-20260310-1")
        );
        assert_eq!(body.get("verifier_count").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(
            body.get("missing_execution_cluster_entry_count")
                .and_then(|v| v.as_u64()),
            Some(1)
        );

        // observed_verifiers sorted by verifier_id
        let observed_verifiers = body
            .get("observed_verifiers")
            .and_then(|v| v.as_array())
            .expect("observed_verifiers array");
        assert_eq!(observed_verifiers.len(), 2);
        assert_eq!(
            observed_verifiers[0]
                .get("verifier_id")
                .and_then(|v| v.as_str()),
            Some("verifier-a")
        );
        assert_eq!(
            observed_verifiers[1]
                .get("verifier_id")
                .and_then(|v| v.as_str()),
            Some("verifier-b")
        );

        // authority_chain_distribution uses authority_chain_id key
        assert!(body
            .get("authority_chain_distribution")
            .and_then(|v| v.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("authority_chain_id").and_then(|v| v.as_str()) == Some("chain-a")
                    && item.get("entry_count").and_then(|v| v.as_u64()) == Some(2)
            })));

        // execution_cluster_distribution uses cluster_id key
        assert!(body
            .get("execution_cluster_distribution")
            .and_then(|v| v.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("cluster_id").and_then(|v| v.as_str()) == Some("cluster-a")
                    && item.get("entry_count").and_then(|v| v.as_u64()) == Some(2)
            })));

        // Forbidden fields must not appear (Requirement 5.1)
        for forbidden in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !body.as_object().is_some_and(|m| m.contains_key(*forbidden)),
                "Forbidden field '{}' found in federation response",
                forbidden
            );
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_federation_endpoint_requires_diversity_ledger() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/federation", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_not_found")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_context_endpoint_summarizes_packaged_context_and_observed_bindings() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let policy = proof_verifier::TrustPolicy {
            policy_version: 1,
            policy_hash: None,
            quorum_policy_ref: Some("policy://quorum/at-least-1-of-n".to_string()),
            trusted_producers: vec!["ayken-ci".to_string()],
            trusted_pubkey_ids: vec!["ed25519-key-a".to_string()],
            required_signatures: Some(proof_verifier::SignatureRequirement {
                kind: "at_least".to_string(),
                count: 1,
            }),
            revoked_pubkey_ids: Vec::new(),
        };
        let policy_hash = proof_verifier::policy::policy_engine::compute_policy_hash(&policy)
            .expect("policy hash");
        let mut registry = proof_verifier::RegistrySnapshot {
            registry_format_version: 1,
            registry_version: 1,
            registry_snapshot_hash: String::new(),
            producers: std::collections::BTreeMap::from([(
                "ayken-ci".to_string(),
                proof_verifier::RegistryEntry {
                    active_pubkey_ids: vec!["ed25519-key-a".to_string()],
                    revoked_pubkey_ids: Vec::new(),
                    superseded_pubkey_ids: Vec::new(),
                    public_keys: std::collections::BTreeMap::from([(
                        "ed25519-key-a".to_string(),
                        proof_verifier::RegistryPublicKey {
                            algorithm: "ed25519".to_string(),
                            public_key: "11".repeat(32),
                        },
                    )]),
                },
            )]),
        };
        registry.registry_snapshot_hash =
            proof_verifier::registry::snapshot::compute_registry_snapshot_hash(&registry)
                .expect("registry hash");

        let context_rules = super::build_default_context_rules_object();
        let context_rules_hash =
            super::compute_context_rules_hash(&context_rules).expect("context rules hash");
        let mut context = proof_verifier::verification_context_object::VerificationContextObject {
            context_version: 1,
            verification_context_id: String::new(),
            policy_hash: policy_hash.clone(),
            registry_snapshot_hash: registry.registry_snapshot_hash.clone(),
            verifier_contract_version: "phase12-context-v1".to_string(),
            context_rules_hash,
            context_epoch: None,
            historical_cutoff_utc: None,
            policy_snapshot_ref: None,
            registry_snapshot_ref: None,
            time_semantics_mode: None,
        };
        context.verification_context_id =
            proof_verifier::verification_context_object::compute_verification_context_id(&context)
                .expect("context id");
        let expected_context_ref = format!("cas:{}", context.verification_context_id);

        write_json(&run_dir.join("context/policy_snapshot.json"), &policy);
        write_json(&run_dir.join("context/registry_snapshot.json"), &registry);
        write_json(&run_dir.join("context/context_rules.json"), &context_rules);
        write_json(
            &run_dir.join("context/verification_context_object.json"),
            &context,
        );
        write_artifact(
            &run_dir,
            "receipts/verification_receipt.json",
            &format!(
                r#"{{
                  "receipt_version": 1,
                  "bundle_id": "bundle-a",
                  "trust_overlay_hash": "sha256:overlay-a",
                  "policy_hash": "{policy_hash}",
                  "registry_snapshot_hash": "{registry_hash}",
                  "verifier_node_id": "node-a",
                  "verifier_key_id": "key-a",
                  "verdict": "Trusted",
                  "verified_at_utc": "2026-03-15T12:00:00Z",
                  "verifier_signature_algorithm": "ed25519",
                  "verifier_signature": "abcd"
                }}"#,
                policy_hash = policy_hash,
                registry_hash = registry.registry_snapshot_hash
            ),
        );
        write_artifact(
            &run_dir,
            "verification_diversity_ledger_binding.json",
            r#"{
              "binding_version": 1,
              "run_id": "run-20260310-1",
              "verification_context_id_source": "policy_hash",
              "node_bindings": [
                {
                  "verification_node_id": "node-a",
                  "verifier_key_id": "key-a",
                  "verifier_id": "verifier-a",
                  "authority_chain_id": "chain-a",
                  "lineage_id": "lineage-a"
                }
              ]
            }"#,
        );
        write_artifact(
            &run_dir,
            "verification_diversity_ledger.json",
            &format!(
                r#"{{
                  "entries": [
                    {{
                      "ledger_version": 1,
                      "entry_id": "entry-a",
                      "run_id": "run-20260310-1",
                      "timestamp_unix_ns": 10,
                      "subject_bundle_id": "bundle-a",
                      "verification_context_id": "{policy_hash}",
                      "verification_node_id": "node-a",
                      "verifier_id": "verifier-a",
                      "authority_chain_id": "chain-a",
                      "lineage_id": "lineage-a",
                      "verdict": "PASS",
                      "receipt_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    }}
                  ]
                }}"#,
                policy_hash = policy_hash
            ),
        );
        write_artifact(
            &run_dir,
            "replay_boundary_flow_source.json",
            &format!(
                r#"{{
                  "source_version": 1,
                  "flow_surface": "replay_boundary",
                  "status": "PASS",
                  "run_id": "run-20260310-1",
                  "window_model": "append_only_event_stream",
                  "events": [
                    {{
                      "timestamp_unix_ns": 10,
                      "subject_bundle_id": "bundle-a",
                      "verification_context_id": "{policy_hash}",
                      "authority_chain_id": "chain-a",
                      "terminal": true,
                      "reused": true
                    }}
                  ]
                }}"#,
                policy_hash = policy_hash
            ),
        );
        write_artifact(
            &run_dir,
            "trust_reuse_flow_source.json",
            &format!(
                r#"{{
                  "source_version": 1,
                  "flow_surface": "trust_reuse",
                  "status": "PASS",
                  "run_id": "run-20260310-1",
                  "window_model": "append_only_event_stream",
                  "events": [
                    {{
                      "timestamp_unix_ns": 20,
                      "subject_bundle_id": "bundle-a",
                      "verification_context_id": "{policy_hash}",
                      "authority_chain_id": "chain-a",
                      "terminal": true,
                      "reused": true
                    }}
                  ]
                }}"#,
                policy_hash = policy_hash
            ),
        );

        let mut runtime_event = TrustReuseRuntimeEvent {
            event_schema_version: 1,
            event_id: String::new(),
            run_id: "runtime-run-a".to_string(),
            timestamp_unix_ns: 30,
            subject_bundle_id: "bundle-a".to_string(),
            verification_context_id: policy_hash.clone(),
            authority_chain_id: "chain-a".to_string(),
            trust_reuse_outcome: TrustReuseOutcome::Accepted,
            terminal: true,
            reused: true,
            receipt_ref: "receipts/verification_receipt.json".to_string(),
            verification_context_ref: expected_context_ref.clone(),
            verifier_attestation_ref: "cas:sha256:verifier-attestation-a".to_string(),
            verifier_registry_snapshot_hash: "a".repeat(64),
            verification_node_id: Some("node-a".to_string()),
            verifier_id: Some("verifier-a".to_string()),
            lineage_id: Some("lineage-a".to_string()),
            execution_cluster_id: None,
            source_run_id: Some("source-run-a".to_string()),
            reuse_group_id: None,
            surface_local_path_id: Some("reports/trust_reuse_runtime_surface.json".to_string()),
            trust_reuse_source: Some("native-runtime-trust-reuse".to_string()),
        };
        runtime_event.event_id =
            compute_trust_reuse_runtime_event_id(&runtime_event).expect("runtime event id");
        write_json(
            &run_dir.join("reports/trust_reuse_runtime_surface.json"),
            &TrustReuseRuntimeSurfaceReport {
                surface_version: 1,
                flow_surface: "trust_reuse_runtime".to_string(),
                status: "PASS".to_string(),
                run_id: "runtime-run-a".to_string(),
                source_kind: "local_runtime_evidence".to_string(),
                event_count: 1,
                accepted_event_count: 1,
                historical_only_event_count: 0,
                rejected_event_count: 0,
                events: vec![runtime_event],
            },
        );

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/context", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("declared_context")
                .and_then(|value| value.get("verifier_contract_version"))
                .and_then(|value| value.as_str()),
            Some("phase12-context-v1")
        );
        assert_eq!(
            body.get("material_binding_status")
                .and_then(|value| value.get("policy_hash_matches_declared_context"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("material_binding_status")
                .and_then(|value| value.get("legacy_verification_context_id_source"))
                .and_then(|value| value.as_str()),
            Some("policy_hash")
        );
        assert!(body
            .get("observed_context_id_sources")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("source").and_then(|value| value.as_str())
                    == Some("verification_diversity_ledger")
                    && item
                        .get("values")
                        .and_then(|value| value.as_array())
                        .is_some_and(|values| {
                            values
                                .iter()
                                .any(|value| value.as_str() == Some(policy_hash.as_str()))
                        })
            })));
        assert!(body
            .get("observed_context_ref_sources")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("source").and_then(|value| value.as_str())
                    == Some("trust_reuse_runtime_surface")
                    && item
                        .get("values")
                        .and_then(|value| value.as_array())
                        .is_some_and(|values| {
                            values
                                .iter()
                                .any(|value| value.as_str() == Some(expected_context_ref.as_str()))
                        })
            })));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_scoped_context_endpoint_requires_context_package() {
        let dir = temp_dir();
        let run_dir = dir.join("run-20260310-1");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let response = route_request("GET", "/diagnostics/runs/run-20260310-1/context", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_not_found")
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
    fn run_scoped_parity_endpoint_fail_closes_when_artifact_exposes_forbidden_field() {
        let dir = temp_dir();
        let run_dir = dir.join("run-safe");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "parity_report.json",
            r#"{"status":"PASS","recommended_action":"forbidden"}"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-safe/parity", &dir);
        assert_eq!(response.status_code, 500);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("forbidden_observability_field_exposed")
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
        assert!(body
            .get("request_fingerprint")
            .and_then(|v| v.as_str())
            .is_some_and(|value| value.starts_with("sha256:")));
        assert_eq!(
            body.get("verification_determinism_contract_path")
                .and_then(|v| v.as_str()),
            Some("verification_determinism_contract.json")
        );
        assert!(body
            .get("verification_determinism_artifact_hash")
            .and_then(|v| v.as_str())
            .is_some_and(|value| value.starts_with("sha256:")));

        let run_dir = dir.join("run-proofd-execution-r1");
        assert!(run_dir.join("proofd_run_manifest.json").is_file());
        assert!(run_dir.join("receipts/verification_receipt.json").is_file());
        assert!(run_dir
            .join("verification_determinism_contract.json")
            .is_file());

        let determinism_contract = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(run_dir.join("verification_determinism_contract.json"))
                .expect("read determinism contract"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            determinism_contract
                .get("artifact_hash")
                .and_then(|v| v.as_str()),
            body.get("verification_determinism_artifact_hash")
                .and_then(|v| v.as_str())
        );

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
    fn verify_bundle_request_fingerprint_is_canonical_and_ignores_run_id() {
        let request_a = parse_verify_bundle_request(
            br#"{
                "bundle_path":"/abs/bundle",
                "policy_path":"/abs/policy.json",
                "registry_path":"/abs/registry.json",
                "receipt_mode":"emit_unsigned",
                "run_id":"run-a"
            }"#,
        )
        .expect("parse request a");
        let request_b = parse_verify_bundle_request(
            br#"{
                "run_id":"run-b",
                "registry_path":"/abs/registry.json",
                "receipt_mode":"emit_unsigned",
                "policy_path":"/abs/policy.json",
                "bundle_path":"/abs/bundle"
            }"#,
        )
        .expect("parse request b");

        let fingerprint_a =
            compute_verify_bundle_request_fingerprint(&request_a).expect("fingerprint a");
        let fingerprint_b =
            compute_verify_bundle_request_fingerprint(&request_b).expect("fingerprint b");

        assert_eq!(fingerprint_a, fingerprint_b);
    }

    #[test]
    fn internal_replay_endpoint_confirms_determinism_against_existing_run() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let verify_request = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-replay-r1",
        });
        let verify_request_bytes = serde_json::to_vec(&verify_request).expect("serialize request");
        let verify_response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(verify_request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(verify_response.status_code, 200);

        let replay_request = json!({
            "source_run_id": "run-proofd-replay-r1",
            "verify_request": verify_request,
        });
        let replay_request_bytes =
            serde_json::to_vec(&replay_request).expect("serialize replay request");
        let replay_response = route_request_with_body(
            "POST",
            "/internal/replay",
            Some(replay_request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(replay_response.status_code, 200);
        let body = body_json(replay_response);
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(
            body.get("matches_original").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert!(dir
            .join("run-proofd-replay-r1")
            .join("verification_determinism_replay_report.json")
            .is_file());

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_keeps_determinism_hash_stable_across_run_ids() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let request_a = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-cross-node-a",
        });
        let request_b = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-cross-node-b",
        });
        let request_a_bytes = serde_json::to_vec(&request_a).expect("serialize request a");
        let request_b_bytes = serde_json::to_vec(&request_b).expect("serialize request b");
        let response_a = body_json(route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_a_bytes.as_slice()),
            &dir,
        ));
        let response_b = body_json(route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(request_b_bytes.as_slice()),
            &dir,
        ));

        assert_eq!(
            response_a
                .get("request_fingerprint")
                .and_then(|v| v.as_str()),
            response_b
                .get("request_fingerprint")
                .and_then(|v| v.as_str())
        );
        assert_eq!(
            response_a
                .get("verification_determinism_artifact_hash")
                .and_then(|v| v.as_str()),
            response_b
                .get("verification_determinism_artifact_hash")
                .and_then(|v| v.as_str())
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn internal_replay_endpoint_emits_determinism_incident_on_hash_mismatch() {
        let dir = temp_dir();
        let fixture = create_fixture_bundle();
        let policy_path = fixture.root.join("proofd-policy.json");
        let registry_path = fixture.root.join("proofd-registry.json");
        write_json(&policy_path, &fixture.policy);
        write_json(&registry_path, &fixture.registry);

        let verify_request = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_unsigned",
            "run_id": "run-proofd-replay-r2",
        });
        let verify_request_bytes = serde_json::to_vec(&verify_request).expect("serialize request");
        let verify_response = route_request_with_body(
            "POST",
            "/verify/bundle",
            Some(verify_request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(verify_response.status_code, 200);

        write_artifact(
            &dir.join("run-proofd-replay-r2"),
            "verification_determinism_contract.json",
            r#"{
              "contract": {
                "contract_version": 1,
                "request_fingerprint": "sha256:tampered",
                "verdict": "TRUSTED",
                "subject_hash": "sha256:tampered",
                "context_hash": "sha256:tampered",
                "authority_hash": "sha256:tampered",
                "findings_hash": "sha256:tampered",
                "receipt_payload_hash": null
              },
              "artifact_hash": "sha256:tampered"
            }"#,
        );

        let replay_request = json!({
            "source_run_id": "run-proofd-replay-r2",
            "verify_request": verify_request,
        });
        let replay_request_bytes =
            serde_json::to_vec(&replay_request).expect("serialize replay request");
        let replay_response = route_request_with_body(
            "POST",
            "/internal/replay",
            Some(replay_request_bytes.as_slice()),
            &dir,
        );
        assert_eq!(replay_response.status_code, 409);
        let body = body_json(replay_response);
        assert_eq!(
            body.get("status").and_then(|v| v.as_str()),
            Some("DETERMINISM_VIOLATION")
        );
        assert_eq!(
            body.get("incident")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str()),
            Some("determinism_incident")
        );
        assert!(dir
            .join("run-proofd-replay-r2")
            .join("verification_determinism_incident.json")
            .is_file());

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
        assert!(body
            .get("request_fingerprint")
            .and_then(|v| v.as_str())
            .is_some_and(|value| value.starts_with("sha256:")));
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
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("context/policy_snapshot.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("context/registry_snapshot.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("context/context_rules.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("context/verification_context_object.json")
            .is_file());
        assert!(dir
            .join("run-proofd-execution-r2")
            .join("reports/trust_reuse_runtime_surface.json")
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
        assert!(run_summary
            .get("artifact_paths")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("context/verification_context_object.json"))));
        assert!(run_summary
            .get("artifact_paths")
            .and_then(|v| v.as_array())
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|item| item.as_str() == Some("reports/trust_reuse_runtime_surface.json"))));

        let federation = body_json(route_request(
            "GET",
            "/diagnostics/runs/run-proofd-execution-r2/federation",
            &dir,
        ));
        // Task 6.9: use projection field names
        assert_eq!(
            federation.get("verifier_count").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            federation
                .get("authority_chain_distribution")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(1)
        );
        assert!(federation
            .get("observed_verifiers")
            .and_then(|v| v.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("lineage_id").and_then(|v| v.as_str()) == Some("lineage-receipt-node-b")
            })));

        let context = body_json(route_request(
            "GET",
            "/diagnostics/runs/run-proofd-execution-r2/context",
            &dir,
        ));
        let declared_context_id = context
            .get("declared_context")
            .and_then(|value| value.get("verification_context_id"))
            .and_then(|value| value.as_str())
            .expect("declared context id");
        assert!(declared_context_id.starts_with("sha256:"));
        assert_eq!(
            context
                .get("material_binding_status")
                .and_then(|value| value.get("policy_hash_matches_declared_context"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            context
                .get("material_binding_status")
                .and_then(|value| value.get("legacy_verification_context_id_source"))
                .and_then(|value| value.as_str()),
            Some("policy_hash")
        );
        assert!(context
            .get("observed_context_id_sources")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("source").and_then(|value| value.as_str())
                    == Some("verification_diversity_ledger")
                    && item
                        .get("values")
                        .and_then(|value| value.as_array())
                        .is_some_and(|values| {
                            values.iter().any(|value| {
                                value.as_str()
                                    == body
                                        .get("verdict_subject")
                                        .and_then(|value| value.get("policy_hash"))
                                        .and_then(|value| value.as_str())
                            })
                        })
            })));
        assert!(context
            .get("observed_context_ref_sources")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("source").and_then(|value| value.as_str())
                    == Some("trust_reuse_runtime_surface")
                    && item
                        .get("values")
                        .and_then(|value| value.as_array())
                        .is_some_and(|values| {
                            values.iter().any(|value| {
                                value
                                    .as_str()
                                    .is_some_and(|value| value.starts_with("cas:sha256:"))
                            })
                        })
            })));

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
    fn verify_bundle_endpoint_auto_materializes_native_trust_reuse_surface_from_runtime_binding() {
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
        .expect("remove bundle native trust reuse surface");

        let verify_request = VerifyRequest {
            bundle_path: &fixture.root,
            policy: &fixture.policy,
            registry_snapshot: &fixture.registry,
            receipt_mode: ReceiptMode::EmitSigned,
            receipt_signer: Some(&fixture.receipt_signer),
            audit_mode: AuditMode::None,
            audit_ledger_path: None,
        };
        let outcome = verify_bundle(&verify_request).expect("verify fixture bundle");
        let runtime_input_dir = dir.join("trust-reuse-runtime-inputs");
        fs::create_dir_all(&runtime_input_dir).expect("create trust reuse runtime input dir");
        let (verification_context_path, verifier_attestation_path, verifier_registry_path) =
            write_trust_reuse_runtime_inputs(
                &fixture,
                &outcome.subject,
                &runtime_input_dir,
                &fixture.verifier_registry,
            );
        let verifier_key_path = runtime_input_dir.join("receipt_verifier_key.json");
        write_json(&verifier_key_path, &fixture.receipt_verifier_key);

        let request_body = json!({
            "bundle_path": fixture.root,
            "policy_path": policy_path,
            "registry_path": registry_path,
            "receipt_mode": "emit_signed",
            "run_id": "run-proofd-execution-r2h",
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
            "trust_reuse_runtime_binding": {
                "verification_context_path": verification_context_path,
                "verifier_attestation_path": verifier_attestation_path,
                "verifier_registry_path": verifier_registry_path,
                "verifier_key_path": verifier_key_path,
                "source_run_id": "source-run-proofd-bootstrap-b",
                "reuse_group_id": "reuse-group-proofd-b",
                "surface_local_path_id": "trust-reuse-runtime-surface-proofd-b",
                "trust_reuse_source": "proofd-runtime-evaluator"
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
            body.get("trust_reuse_runtime_surface_path")
                .and_then(|v| v.as_str()),
            Some("trust_reuse_runtime_surface.json")
        );
        assert_eq!(
            body.get("trust_reuse_runtime_surface_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_proofd_trust_reuse")
        );
        assert_eq!(
            body.get("trust_reuse_flow_source_origin")
                .and_then(|v| v.as_str()),
            Some("runtime_proofd_trust_reuse")
        );

        let native_surface = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2h")
                    .join("trust_reuse_runtime_surface.json"),
            )
            .expect("read run-local native trust reuse surface"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            native_surface
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("surface_local_path_id"))
                .and_then(|v| v.as_str()),
            Some("trust-reuse-runtime-surface-proofd-b")
        );
        assert_eq!(
            native_surface
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("source_run_id"))
                .and_then(|v| v.as_str()),
            Some("source-run-proofd-bootstrap-b")
        );

        let trust_reuse_source = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(
                dir.join("run-proofd-execution-r2h")
                    .join("trust_reuse_flow_source.json"),
            )
            .expect("read trust reuse flow source"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("surface_local_path_id"))
                .and_then(|v| v.as_str()),
            Some("trust-reuse-runtime-surface-proofd-b")
        );
        assert_eq!(
            trust_reuse_source
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|events| events.first())
                .and_then(|event| event.get("trust_reuse_source"))
                .and_then(|v| v.as_str()),
            Some("proofd-runtime-evaluator")
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
        assert_eq!(second_response.status_code, 409);
        let body = body_json(second_response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("run_id_fingerprint_conflict")
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_bundle_endpoint_generates_run_id_when_missing() {
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
        let run_id = body
            .get("run_id")
            .and_then(|v| v.as_str())
            .expect("generated run id")
            .to_string();
        assert!(!run_id.is_empty());
        assert!(run_id.len() <= 128);
        assert!(run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));

        let request_fingerprint = body
            .get("request_fingerprint")
            .and_then(|v| v.as_str())
            .expect("request fingerprint");
        assert!(request_fingerprint.starts_with("sha256:"));

        let run_manifest = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(dir.join(&run_id).join("proofd_run_manifest.json"))
                .expect("read run manifest"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            run_manifest.get("run_id").and_then(|v| v.as_str()),
            Some(run_id.as_str())
        );
        assert_eq!(
            run_manifest
                .get("request_fingerprint")
                .and_then(|v| v.as_str()),
            Some(request_fingerprint)
        );

        let _ = fs::remove_dir_all(&fixture.root);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_manifest_creation_allows_only_one_writer() {
        let dir = temp_dir();
        let manifest_path = dir.join("proofd_run_manifest.json");
        let manifest = json!({
            "run_id": "run-proofd-atomic-manifest",
            "request_fingerprint": "sha256:test-fingerprint",
            "verdict": "TRUSTED"
        });
        let barrier = Arc::new(Barrier::new(3));

        let spawn_writer = |barrier: Arc<Barrier>| {
            let manifest_path = manifest_path.clone();
            let manifest = manifest.clone();
            thread::spawn(move || {
                barrier.wait();
                create_run_manifest_atomically(
                    &manifest_path,
                    &manifest,
                    "sha256:test-fingerprint",
                    "run_manifest_write_failed",
                )
            })
        };

        let handle_a = spawn_writer(barrier.clone());
        let handle_b = spawn_writer(barrier.clone());
        barrier.wait();

        let result_a = handle_a.join().expect("writer a");
        let result_b = handle_b.join().expect("writer b");

        let success_count = usize::from(result_a.is_ok()) + usize::from(result_b.is_ok());
        assert_eq!(success_count, 1);
        assert!(matches!(
            (&result_a, &result_b),
            (
                Err(ServiceError::Conflict("run_id_fingerprint_conflict")),
                Ok(())
            ) | (
                Ok(()),
                Err(ServiceError::Conflict("run_id_fingerprint_conflict"))
            )
        ));

        let manifest_body = body_json(DiagnosticsResponse {
            status_code: 200,
            body: fs::read(&manifest_path).expect("read manifest"),
            content_type: "application/json; charset=utf-8",
        });
        assert_eq!(
            manifest_body
                .get("request_fingerprint")
                .and_then(|v| v.as_str()),
            Some("sha256:test-fingerprint")
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

    // ── registry diagnostics unit tests ──────────────────────────────────────

    fn make_registry(producers: &[&str]) -> proof_verifier::RegistrySnapshot {
        let mut snapshot = proof_verifier::RegistrySnapshot {
            registry_format_version: 1,
            registry_version: 1,
            registry_snapshot_hash: String::new(),
            producers: producers
                .iter()
                .map(|id| {
                    (
                        id.to_string(),
                        proof_verifier::RegistryEntry {
                            active_pubkey_ids: vec!["key-a".to_string()],
                            revoked_pubkey_ids: Vec::new(),
                            superseded_pubkey_ids: Vec::new(),
                            public_keys: std::collections::BTreeMap::from([(
                                "key-a".to_string(),
                                proof_verifier::RegistryPublicKey {
                                    algorithm: "ed25519".to_string(),
                                    public_key: "11".repeat(32),
                                },
                            )]),
                        },
                    )
                })
                .collect(),
        };
        snapshot.registry_snapshot_hash =
            proof_verifier::registry::snapshot::compute_registry_snapshot_hash(&snapshot)
                .expect("registry hash");
        snapshot
    }

    fn write_registry_snapshot(run_dir: &PathBuf, registry: &proof_verifier::RegistrySnapshot) {
        let path = run_dir.join("context/registry_snapshot.json");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create context dir");
        }
        let bytes = proof_verifier::canonical::jcs::canonicalize_json(registry)
            .expect("canonicalize registry");
        fs::write(path, bytes).expect("write registry snapshot");
    }

    fn write_context_object(
        run_dir: &PathBuf,
        registry_snapshot_hash: &str,
    ) -> proof_verifier::verification_context_object::VerificationContextObject {
        let context_rules = super::build_default_context_rules_object();
        let context_rules_hash =
            super::compute_context_rules_hash(&context_rules).expect("context rules hash");
        let mut ctx = proof_verifier::verification_context_object::VerificationContextObject {
            context_version: 1,
            verification_context_id: String::new(),
            policy_hash: "sha256:policy-placeholder".to_string(),
            registry_snapshot_hash: registry_snapshot_hash.to_string(),
            verifier_contract_version: "phase12-context-v1".to_string(),
            context_rules_hash,
            context_epoch: None,
            historical_cutoff_utc: None,
            policy_snapshot_ref: None,
            registry_snapshot_ref: None,
            time_semantics_mode: None,
        };
        ctx.verification_context_id =
            proof_verifier::verification_context_object::compute_verification_context_id(&ctx)
                .expect("context id");
        write_json(
            &run_dir.join("context/verification_context_object.json"),
            &ctx,
        );
        ctx
    }

    fn write_receipt_with_registry_hash(run_dir: &PathBuf, registry_snapshot_hash: &str) {
        let receipt_json = format!(
            r#"{{
              "receipt_version": 1,
              "bundle_id": "bundle-reg-test",
              "trust_overlay_hash": "sha256:overlay-reg",
              "policy_hash": "sha256:policy-placeholder",
              "registry_snapshot_hash": "{registry_snapshot_hash}",
              "verifier_node_id": "node-reg",
              "verifier_key_id": "key-reg",
              "verdict": "Trusted",
              "verified_at_utc": "2026-03-15T12:00:00Z",
              "verifier_signature_algorithm": "ed25519",
              "verifier_signature": "abcd"
            }}"#
        );
        write_artifact(run_dir, "receipts/verification_receipt.json", &receipt_json);
    }

    #[test]
    fn registry_endpoint_happy_path_all_artifacts_present_hashes_match() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-1");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let registry = make_registry(&["producer-a", "producer-b"]);
        let hash = registry.registry_snapshot_hash.clone();
        write_registry_snapshot(&run_dir, &registry);
        write_context_object(&run_dir, &hash);
        write_receipt_with_registry_hash(&run_dir, &hash);

        let response = route_request("GET", "/diagnostics/runs/run-reg-1/registry", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        assert_eq!(
            body.get("run_id").and_then(|v| v.as_str()),
            Some("run-reg-1")
        );
        assert_eq!(
            body.get("source_artifact_path").and_then(|v| v.as_str()),
            Some("context/registry_snapshot.json")
        );
        assert_eq!(
            body.get("declared_registry_snapshot_hash")
                .and_then(|v| v.as_str()),
            Some(hash.as_str())
        );
        assert_eq!(
            body.get("declared_registry_entry_count")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            body.get("context_binding_status")
                .and_then(|v| v.get("registry_snapshot_hash_matches_declared_context"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        let sources = body
            .get("observed_registry_hash_sources")
            .and_then(|v| v.as_array())
            .expect("sources array");
        assert!(sources.iter().any(
            |s| s.get("source").and_then(|v| v.as_str()) == Some("verification_context_object")
        ));
        assert!(sources
            .iter()
            .any(|s| s.get("source").and_then(|v| v.as_str()) == Some("receipt")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_happy_path_registry_only_null_binding_empty_sources() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-2");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let registry = make_registry(&["producer-a"]);
        write_registry_snapshot(&run_dir, &registry);
        // no context object, no receipt

        let response = route_request("GET", "/diagnostics/runs/run-reg-2/registry", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        assert!(
            body.get("context_binding_status")
                .and_then(|v| v.get("registry_snapshot_hash_matches_declared_context"))
                .is_some_and(|v| v.is_null()),
            "expected null binding status"
        );
        assert_eq!(
            body.get("observed_registry_hash_sources")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_hash_mismatch_returns_false_binding_status() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-3");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let registry = make_registry(&["producer-a"]);
        write_registry_snapshot(&run_dir, &registry);
        write_context_object(&run_dir, "sha256:completely-different-hash");

        let response = route_request("GET", "/diagnostics/runs/run-reg-3/registry", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        assert_eq!(
            body.get("context_binding_status")
                .and_then(|v| v.get("registry_snapshot_hash_matches_declared_context"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_missing_run_dir_returns_404_run_dir_not_found() {
        let dir = temp_dir();

        let response = route_request("GET", "/diagnostics/runs/run-reg-missing/registry", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("run_dir_not_found")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_missing_registry_snapshot_returns_404_artifact_not_found() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-4");
        fs::create_dir_all(&run_dir).expect("create run dir");
        // no context/registry_snapshot.json

        let response = route_request("GET", "/diagnostics/runs/run-reg-4/registry", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_not_found")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_malformed_registry_snapshot_returns_500() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-5");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "context/registry_snapshot.json",
            "not valid json {{",
        );

        let response = route_request("GET", "/diagnostics/runs/run-reg-5/registry", &dir);
        assert_eq!(response.status_code, 500);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("invalid_context_registry_snapshot")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_entry_count_matches_producers_len() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-6");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let registry = make_registry(&["p1", "p2", "p3", "p4", "p5"]);
        write_registry_snapshot(&run_dir, &registry);

        let response = route_request("GET", "/diagnostics/runs/run-reg-6/registry", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("declared_registry_entry_count")
                .and_then(|v| v.as_u64()),
            Some(5)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_rejects_query_string() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-7");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-reg-7/registry?select_winner=true",
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_endpoint_rejects_post_method() {
        let dir = temp_dir();
        let run_dir = dir.join("run-reg-8");
        fs::create_dir_all(&run_dir).expect("create run dir");

        let response =
            route_request_with_body("POST", "/diagnostics/runs/run-reg-8/registry", None, &dir);
        assert_eq!(response.status_code, 405);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("method_not_allowed")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 2.3: VerifyBundleResponseBody forbidden fields serialize guard ──

    #[test]
    fn verify_bundle_response_contains_no_forbidden_fields() {
        // Validates: Requirement 1.11, 8.6
        // Construct a minimal VerifyBundleResponseBody and check its serialized form
        let response_body = super::VerifyBundleResponseBody {
            status: "ok",
            run_id: "run-test".to_string(),
            verdict: "TRUSTED",
            verdict_subject: serde_json::json!({}),
            receipt_emitted: false,
            receipt_path: None,
            request_fingerprint: "sha256:abc".to_string(),
            behavioral_observability_emitted: false,
            audit_ledger_path: None,
            verification_diversity_ledger_binding_path: None,
            verification_diversity_ledger_path: None,
            replay_boundary_flow_source_path: None,
            replay_boundary_flow_source_origin: None,
            trust_reuse_runtime_surface_path: None,
            trust_reuse_runtime_surface_origin: None,
            trust_reuse_flow_source_path: None,
            trust_reuse_flow_source_origin: None,
            verification_determinism_contract_path: "verification_determinism_contract.json"
                .to_string(),
            verification_determinism_artifact_hash: "sha256:abc".to_string(),
            findings_count: 0,
        };
        let serialized = serde_json::to_value(&response_body).expect("serialize response body");
        let obj = serialized.as_object().expect("response body is object");
        for forbidden in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !obj.contains_key(*forbidden),
                "Forbidden field '{}' found in VerifyBundleResponseBody",
                forbidden
            );
        }
    }

    // ── Task 4.5: Allowed artifact in set but missing on disk → 404 ──────────

    #[test]
    fn run_artifact_endpoint_returns_404_for_allowed_but_missing_artifact() {
        // Validates: Requirement 3.6
        let dir = temp_dir();
        let run_dir = dir.join("run-artifact-404");
        fs::create_dir_all(&run_dir).expect("create run dir");
        // Do NOT write proofd_run_manifest.json — it's in Allowed_Artifact_Set but absent

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-artifact-404/artifacts/proofd_run_manifest.json",
            &dir,
        );
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_not_found")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 4.4 extra: path outside Allowed_Artifact_Set → 403 ─────────────

    #[test]
    fn run_artifact_endpoint_returns_403_for_path_outside_allowed_set() {
        // Validates: Requirement 3.7
        let dir = temp_dir();
        let run_dir = dir.join("run-artifact-403");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(&run_dir, "secret.json", r#"{"secret":"data"}"#);

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-artifact-403/artifacts/secret.json",
            &dir,
        );
        assert_eq!(response.status_code, 403);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_path_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 5.2: run_dir_not_found for missing run dir on artifacts index ───

    #[test]
    fn run_artifacts_index_returns_404_run_dir_not_found_when_dir_missing() {
        // Validates: Requirement 3.2
        let dir = temp_dir();
        // Do NOT create the run directory

        let response = route_request("GET", "/diagnostics/runs/run-nonexistent/artifacts", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("run_dir_not_found")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 6.8 / 10.2: FederationDiagnosticsProjection forbidden fields guard

    #[test]
    fn federation_projection_contains_no_forbidden_fields() {
        // Validates: Requirement 5.1, 8.6
        let dir = temp_dir();
        let run_dir = dir.join("run-fed-guard");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "verification_diversity_ledger.json",
            r#"{"entries":[{
                "ledger_version":1,"entry_id":"e1","run_id":"run-fed-guard",
                "timestamp_unix_ns":1,"subject_bundle_id":"b1",
                "verification_context_id":"ctx1","verification_node_id":"n1",
                "verifier_id":"v1","authority_chain_id":"chain1","lineage_id":"lin1",
                "verdict":"PASS",
                "receipt_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }]}"#,
        );

        let response = route_request("GET", "/diagnostics/runs/run-fed-guard/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let obj = body.as_object().expect("federation response is object");
        for forbidden in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !obj.contains_key(*forbidden),
                "Forbidden field '{}' found in FederationDiagnosticsProjection response",
                forbidden
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 6.1: federation run_dir_not_found ────────────────────────────────

    #[test]
    fn federation_endpoint_returns_run_dir_not_found_when_dir_missing() {
        // Validates: Requirement 4.3
        let dir = temp_dir();
        // Do NOT create the run directory

        let response = route_request("GET", "/diagnostics/runs/run-fed-missing/federation", &dir);
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("run_dir_not_found")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 8.3: concurrent manifest creation — only one writer succeeds ────

    #[test]
    fn concurrent_manifest_creation_only_one_writer_succeeds() {
        // Validates: Requirement 1.12
        // Already covered by run_manifest_creation_allows_only_one_writer but
        // this variant uses a different fingerprint to confirm conflict detection
        use std::sync::{Arc, Barrier};
        use std::thread;

        let dir = temp_dir();
        let manifest_path = dir.join("proofd_run_manifest.json");
        let fingerprint = "sha256:concurrent-test-fingerprint";
        let manifest = serde_json::json!({
            "run_id": "run-concurrent",
            "request_fingerprint": fingerprint,
            "verdict": "TRUSTED"
        });
        let barrier = Arc::new(Barrier::new(4));

        let spawn_writer = |barrier: Arc<Barrier>| {
            let manifest_path = manifest_path.clone();
            let manifest = manifest.clone();
            thread::spawn(move || {
                barrier.wait();
                super::create_run_manifest_atomically(
                    &manifest_path,
                    &manifest,
                    fingerprint,
                    "run_manifest_write_failed",
                )
            })
        };

        let handles: Vec<_> = (0..3).map(|_| spawn_writer(barrier.clone())).collect();
        barrier.wait();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("thread"))
            .collect();
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let conflict_count = results
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    Err(ServiceError::Conflict("run_id_fingerprint_conflict"))
                )
            })
            .count();

        assert_eq!(success_count, 1, "exactly one writer must succeed");
        assert_eq!(
            conflict_count, 2,
            "remaining writers must get conflict error"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Task 9.2: projection isolation — internal field additions don't leak ─

    #[test]
    fn federation_projection_only_contains_spec_fields() {
        // Validates: Requirement 4, 5; Design §4 Spec Projection Layer
        // Verifies that the serialized response contains ONLY the projection fields
        let dir = temp_dir();
        let run_dir = dir.join("run-proj-isolation");
        fs::create_dir_all(&run_dir).expect("create run dir");
        write_artifact(
            &run_dir,
            "verification_diversity_ledger.json",
            r#"{"entries":[{
                "ledger_version":1,"entry_id":"e1","run_id":"run-proj-isolation",
                "timestamp_unix_ns":1,"subject_bundle_id":"b1",
                "verification_context_id":"ctx1","verification_node_id":"n1",
                "verifier_id":"v1","authority_chain_id":"chain1","lineage_id":"lin1",
                "execution_cluster_id":"cluster1",
                "verdict":"PASS",
                "receipt_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }]}"#,
        );

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-proj-isolation/federation",
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let obj = body.as_object().expect("federation response is object");

        // Only these spec-defined top-level keys must be present
        const ALLOWED_KEYS: &[&str] = &[
            "run_id",
            "verifier_count",
            "observed_verifiers",
            "authority_chain_distribution",
            "execution_cluster_distribution",
            "missing_execution_cluster_entry_count",
        ];
        // Internal-only fields must NOT appear
        const INTERNAL_ONLY_KEYS: &[&str] = &[
            "source_artifact_path",
            "entry_count",
            "unique_verification_node_count",
            "unique_verifier_count",
            "unique_authority_chain_count",
            "unique_lineage_count",
            "unique_execution_cluster_count",
            "verification_node_distribution",
            "verifier_distribution",
            "lineage_distribution",
            "observed_entries",
        ];
        for key in INTERNAL_ONLY_KEYS {
            assert!(
                !obj.contains_key(*key),
                "Internal-only field '{}' must not appear in projection response",
                key
            );
        }
        for key in ALLOWED_KEYS {
            assert!(
                obj.contains_key(*key),
                "Required projection field '{}' is missing from response",
                key
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/federation — global view ────────────────────────────────
    #[test]
    fn global_federation_endpoint_aggregates_all_runs() {
        let dir = temp_dir();
        // Two runs with diversity ledgers
        for (run_id, verifier_id) in [("run-a", "verifier-1"), ("run-b", "verifier-2")] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_diversity_ledger(&run_dir, run_id, verifier_id);
        }
        // One run without ledger (fail-open)
        let run_dir_c = dir.join("run-c");
        fs::create_dir_all(&run_dir_c).unwrap();

        let response = route_request("GET", "/diagnostics/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        assert_eq!(body.get("verifier_count").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("total"))
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("with_ledger"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/federation — empty evidence dir ──────────────────────────
    #[test]
    fn global_federation_endpoint_empty_dir_returns_zero_counts() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("verifier_count").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("total"))
                .and_then(|v| v.as_u64()),
            Some(0)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/federation — verifiers sorted ───────────────────────────
    #[test]
    fn global_federation_endpoint_verifiers_sorted() {
        let dir = temp_dir();
        for (run_id, verifier_id) in [
            ("run-a", "verifier-z"),
            ("run-b", "verifier-a"),
            ("run-c", "verifier-m"),
        ] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_diversity_ledger(&run_dir, run_id, verifier_id);
        }

        let response = route_request("GET", "/diagnostics/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let verifier_ids: Vec<&str> = body
            .get("verifiers")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.get("verifier_id").and_then(|v| v.as_str()))
            .collect();
        let mut sorted = verifier_ids.clone();
        sorted.sort();
        assert_eq!(verifier_ids, sorted);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/federation — POST → 405 ─────────────────────────────────
    #[test]
    fn global_federation_endpoint_post_method_not_allowed() {
        let dir = temp_dir();
        let response = route_request("POST", "/diagnostics/federation", &dir);
        assert_eq!(response.status_code, 405);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/federation — no forbidden fields ─────────────────────────
    #[test]
    fn global_federation_endpoint_no_forbidden_fields() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_diversity_ledger(&run_dir, "run-a", "verifier-1");

        let response = route_request("GET", "/diagnostics/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body_str = String::from_utf8_lossy(&response.body);
        for field in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !body_str.contains(field),
                "forbidden field '{field}' found in global federation response"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/context — global context topology ────────────────────────
    #[test]
    fn global_context_endpoint_aggregates_all_runs() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"a".repeat(64);
        for (run_id, ctx_id) in [("run-a", "ctx-1"), ("run-b", "ctx-2")] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_context_object_simple(&run_dir, ctx_id);
            write_artifact(
                &run_dir,
                "proofd_run_manifest.json",
                &serde_json::json!({"run_id": run_id, "request_fingerprint": fp, "verdict": "PASS"}).to_string(),
            );
        }
        // One run without context (fail-open)
        let run_dir_c = dir.join("run-c");
        fs::create_dir_all(&run_dir_c).unwrap();

        let response = route_request("GET", "/diagnostics/context", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        assert_eq!(body.get("context_count").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("total"))
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("with_context"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/context — empty dir ─────────────────────────────────────
    #[test]
    fn global_context_endpoint_empty_dir_returns_zero_counts() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/context", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("context_count").and_then(|v| v.as_u64()), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/context — contexts sorted ────────────────────────────────
    #[test]
    fn global_context_endpoint_contexts_sorted() {
        let dir = temp_dir();
        for (run_id, ctx_id) in [("run-a", "ctx-z"), ("run-b", "ctx-a"), ("run-c", "ctx-m")] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_context_object_simple(&run_dir, ctx_id);
        }

        let response = route_request("GET", "/diagnostics/context", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let ctx_ids: Vec<&str> = body
            .get("contexts")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.get("context_id").and_then(|v| v.as_str()))
            .collect();
        let mut sorted = ctx_ids.clone();
        sorted.sort();
        assert_eq!(ctx_ids, sorted);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/context — context drift detection ────────────────────────
    #[test]
    fn global_context_endpoint_detects_context_drift() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"b".repeat(64);
        // Same fingerprint, two different contexts → drift
        for (run_id, ctx_id) in [("run-a", "ctx-1"), ("run-b", "ctx-2")] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_context_object_simple(&run_dir, ctx_id);
            write_artifact(
                &run_dir,
                "proofd_run_manifest.json",
                &serde_json::json!({"run_id": run_id, "request_fingerprint": fp, "verdict": "PASS"}).to_string(),
            );
        }

        let response = route_request("GET", "/diagnostics/context", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("context_drift")
                .and_then(|v| v.get("fingerprints_with_multiple_contexts"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/context — POST → 405 ────────────────────────────────────
    #[test]
    fn global_context_endpoint_post_method_not_allowed() {
        let dir = temp_dir();
        let response = route_request("POST", "/diagnostics/context", &dir);
        assert_eq!(response.status_code, 405);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/context — no forbidden fields ────────────────────────────
    #[test]
    fn global_context_endpoint_no_forbidden_fields() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_context_object_simple(&run_dir, "ctx-test");

        let response = route_request("GET", "/diagnostics/context", &dir);
        assert_eq!(response.status_code, 200);
        let body_str = String::from_utf8_lossy(&response.body);
        for field in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !body_str.contains(field),
                "forbidden field '{field}' found in global context response"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — context_ids in response ─────────────
    #[test]
    fn fingerprints_endpoint_includes_context_ids() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"a".repeat(64);
        for (run_id, ctx_id) in [("run-a", "ctx-1"), ("run-b", "ctx-2")] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest_simple(&run_dir, &fp, "PASS");
            write_context_object_simple(&run_dir, ctx_id);
        }

        let response = route_request("GET", &format!("/diagnostics/fingerprints/{fp}"), &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let ctx_ids: Vec<&str> = body
            .get("context_ids")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(ctx_ids.len(), 2);
        assert_eq!(body.get("context_count").and_then(|v| v.as_u64()), Some(2));
        // sorted
        let mut sorted = ctx_ids.clone();
        sorted.sort();
        assert_eq!(ctx_ids, sorted);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/federation — context_ids in verifier entries ────────────
    #[test]
    fn global_federation_endpoint_includes_context_ids_per_verifier() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_diversity_ledger(&run_dir, "run-a", "verifier-1");

        let response = route_request("GET", "/diagnostics/federation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let verifiers = body.get("verifiers").and_then(|v| v.as_array()).unwrap();
        assert!(!verifiers.is_empty());
        // Each verifier entry must have context_ids field
        for v in verifiers {
            assert!(
                v.get("context_ids").is_some(),
                "verifier entry missing context_ids field"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/parity/context-relation — annotates pairs ───────────────
    #[test]
    fn parity_context_relation_annotates_same_context() {
        let dir = temp_dir();
        // Two runs with same context
        for run_id in ["run-a", "run-b"] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_context_object_simple(&run_dir, "ctx-shared");
        }
        // Write a parity report with one pair
        write_artifact(
            &dir,
            "parity_report.json",
            r#"{"pairs":[{"run_a":"run-a","run_b":"run-b","status":"PARITY_MATCH"}]}"#,
        );

        let response = route_request("GET", "/diagnostics/parity/context-relation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let pairs = body.get("pairs").and_then(|v| v.as_array()).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(
            pairs[0].get("context_relation").and_then(|v| v.as_str()),
            Some("same")
        );
        assert_eq!(
            body.get("context_relation_summary")
                .and_then(|v| v.get("same"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/parity/context-relation — different context ─────────────
    #[test]
    fn parity_context_relation_annotates_different_context() {
        let dir = temp_dir();
        let run_dir_a = dir.join("run-a");
        fs::create_dir_all(&run_dir_a).unwrap();
        write_context_object_simple(&run_dir_a, "ctx-1");
        let run_dir_b = dir.join("run-b");
        fs::create_dir_all(&run_dir_b).unwrap();
        write_context_object_simple(&run_dir_b, "ctx-2");
        write_artifact(
            &dir,
            "parity_report.json",
            r#"{"pairs":[{"run_a":"run-a","run_b":"run-b","status":"PARITY_VERDICT_MISMATCH"}]}"#,
        );

        let response = route_request("GET", "/diagnostics/parity/context-relation", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let pairs = body.get("pairs").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            pairs[0].get("context_relation").and_then(|v| v.as_str()),
            Some("different")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/parity/context-relation — 404 when no parity report ─────
    #[test]
    fn parity_context_relation_404_when_no_parity_report() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/parity/context-relation", &dir);
        assert_eq!(response.status_code, 404);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/trust — global producer registry topology ───────────────
    #[test]
    fn global_trust_endpoint_aggregates_producers() {
        let dir = temp_dir();
        // Write two runs with registry snapshots
        for run_id in ["run-a", "run-b"] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_artifact(
                &run_dir,
                "context/registry_snapshot.json",
                r#"{"registry_format_version":1,"registry_version":3,"registry_snapshot_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","producers":{"producer-1":{"active_pubkey_ids":[],"revoked_pubkey_ids":[],"superseded_pubkey_ids":[],"public_keys":{}}}}"#,
            );
        }
        // One run without registry (fail-open)
        let run_dir_c = dir.join("run-c");
        fs::create_dir_all(&run_dir_c).unwrap();

        let response = route_request("GET", "/diagnostics/trust", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        assert_eq!(body.get("producer_count").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("total"))
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            body.get("runs")
                .and_then(|v| v.get("with_registry"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/trust — empty dir ───────────────────────────────────────
    #[test]
    fn global_trust_endpoint_empty_dir_returns_zero_counts() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/trust", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(body.get("producer_count").and_then(|v| v.as_u64()), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/trust — hash consistency ────────────────────────────────
    #[test]
    fn global_trust_endpoint_detects_registry_hash_inconsistency() {
        let dir = temp_dir();
        for (run_id, hash) in [("run-a", "a".repeat(64)), ("run-b", "b".repeat(64))] {
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_artifact(
                &run_dir,
                "context/registry_snapshot.json",
                &format!(
                    r#"{{"registry_format_version":1,"registry_version":1,"registry_snapshot_hash":"{hash}","producers":{{}}}}"#
                ),
            );
        }

        let response = route_request("GET", "/diagnostics/trust", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("registry_hash_consistency")
                .and_then(|v| v.get("consistent"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            body.get("registry_hash_consistency")
                .and_then(|v| v.get("distinct_hash_count"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/trust — POST → 405 ──────────────────────────────────────
    #[test]
    fn global_trust_endpoint_post_method_not_allowed() {
        let dir = temp_dir();
        let response = route_request("POST", "/diagnostics/trust", &dir);
        assert_eq!(response.status_code, 405);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/trust — no forbidden fields ──────────────────────────────
    #[test]
    fn global_trust_endpoint_no_forbidden_fields() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_artifact(
            &run_dir,
            "context/registry_snapshot.json",
            r#"{"registry_format_version":1,"registry_version":1,"registry_snapshot_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","producers":{}}"#,
        );

        let response = route_request("GET", "/diagnostics/trust", &dir);
        assert_eq!(response.status_code, 200);
        let body_str = String::from_utf8_lossy(&response.body);
        for field in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !body_str.contains(field),
                "forbidden field '{field}' found in global trust response"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/replicated-boundary — boundary status ───────────────────
    #[test]
    fn replicated_boundary_endpoint_returns_hold_status() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/replicated-boundary", &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("boundary_status").and_then(|v| v.as_str()),
            Some("HOLD")
        );
        // Invariants must be present
        let invariants = body.get("invariants").and_then(|v| v.as_array()).unwrap();
        assert!(!invariants.is_empty());
        // Disallowed routes must not include /diagnostics paths
        let disallowed: Vec<&str> = body
            .get("disallowed_routes")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        for route in &disallowed {
            assert!(
                !route.starts_with("/diagnostics"),
                "disallowed route '{route}' must not be a diagnostics path"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/replicated-boundary — POST → 405 ────────────────────────
    #[test]
    fn replicated_boundary_endpoint_post_method_not_allowed() {
        let dir = temp_dir();
        let response = route_request("POST", "/diagnostics/replicated-boundary", &dir);
        assert_eq!(response.status_code, 405);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/replicated-boundary — no forbidden fields ───────────────
    #[test]
    fn replicated_boundary_endpoint_no_forbidden_fields() {
        let dir = temp_dir();
        let response = route_request("GET", "/diagnostics/replicated-boundary", &dir);
        assert_eq!(response.status_code, 200);
        let body_str = String::from_utf8_lossy(&response.body);
        for field in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !body_str.contains(field),
                "forbidden field '{field}' found in replicated-boundary response"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod proptest_registry {
    //! Property-based tests for Phase 13 trust registry propagation.
    //! Feature: phase13-trust-registry-propagation

    use super::{
        build_default_context_rules_object, compute_context_rules_hash,
        write_canonical_json_file_if_absent_or_same, CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH,
        RECEIPT_RELATIVE_PATH, VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH,
    };
    use proof_verifier::canonical::jcs::canonicalize_json;
    use proof_verifier::registry::snapshot::compute_registry_snapshot_hash;
    use proof_verifier::verification_context_object::{
        compute_verification_context_id, VerificationContextObject,
    };
    use proof_verifier::{RegistryEntry, RegistryPublicKey, RegistrySnapshot};
    use proptest::prelude::*;
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-pbt")
    }

    /// Build a `RegistrySnapshot` from a list of producer ids.
    fn make_snapshot(producer_ids: &[String]) -> RegistrySnapshot {
        let mut snapshot = RegistrySnapshot {
            registry_format_version: 1,
            registry_version: 1,
            registry_snapshot_hash: String::new(),
            producers: producer_ids
                .iter()
                .map(|id| {
                    (
                        id.clone(),
                        RegistryEntry {
                            active_pubkey_ids: vec!["key-a".to_string()],
                            revoked_pubkey_ids: Vec::new(),
                            superseded_pubkey_ids: Vec::new(),
                            public_keys: BTreeMap::from([(
                                "key-a".to_string(),
                                RegistryPublicKey {
                                    algorithm: "ed25519".to_string(),
                                    public_key: "11".repeat(32),
                                },
                            )]),
                        },
                    )
                })
                .collect(),
        };
        snapshot.registry_snapshot_hash =
            compute_registry_snapshot_hash(&snapshot).expect("registry hash");
        snapshot
    }

    fn write_snapshot_canonical(dir: &PathBuf, snapshot: &RegistrySnapshot) {
        let path = dir.join(CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH);
        fs::create_dir_all(path.parent().unwrap()).expect("create context dir");
        let bytes = canonicalize_json(snapshot).expect("canonicalize");
        fs::write(&path, bytes).expect("write snapshot");
    }

    fn write_context_object_with_hash(dir: &PathBuf, registry_snapshot_hash: &str) {
        let context_rules = build_default_context_rules_object();
        let context_rules_hash =
            compute_context_rules_hash(&context_rules).expect("context rules hash");
        let mut ctx = VerificationContextObject {
            context_version: 1,
            verification_context_id: String::new(),
            policy_hash: "sha256:policy-placeholder".to_string(),
            registry_snapshot_hash: registry_snapshot_hash.to_string(),
            verifier_contract_version: "phase12-context-v1".to_string(),
            context_rules_hash,
            context_epoch: None,
            historical_cutoff_utc: None,
            policy_snapshot_ref: None,
            registry_snapshot_ref: None,
            time_semantics_mode: None,
        };
        ctx.verification_context_id = compute_verification_context_id(&ctx).expect("context id");
        let path = dir.join(VERIFICATION_CONTEXT_OBJECT_RELATIVE_PATH);
        fs::create_dir_all(path.parent().unwrap()).expect("create context dir");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&ctx).expect("serialize ctx"),
        )
        .expect("write ctx");
    }

    fn write_receipt_with_hash(dir: &PathBuf, registry_snapshot_hash: &str) {
        let receipt = serde_json::json!({
            "receipt_version": 1,
            "bundle_id": "bundle-pbt",
            "trust_overlay_hash": "sha256:overlay-pbt",
            "policy_hash": "sha256:policy-placeholder",
            "registry_snapshot_hash": registry_snapshot_hash,
            "verifier_node_id": "node-pbt",
            "verifier_key_id": "key-pbt",
            "verdict": "Trusted",
            "verified_at_utc": "2026-03-15T12:00:00Z",
            "verifier_signature_algorithm": "ed25519",
            "verifier_signature": "abcd"
        });
        let path = dir.join(RECEIPT_RELATIVE_PATH);
        fs::create_dir_all(path.parent().unwrap()).expect("create receipts dir");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&receipt).expect("serialize receipt"),
        )
        .expect("write receipt");
    }

    fn call_registry_endpoint(evidence_dir: &PathBuf, run_id: &str) -> Value {
        let response = super::route_request(
            "GET",
            &format!("/diagnostics/runs/{run_id}/registry"),
            evidence_dir,
        );
        serde_json::from_slice(&response.body).expect("valid json body")
    }

    fn collect_run_files(dir: &PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();
        collect_files_recursive(dir, &mut files);
        files.sort();
        files
    }

    fn collect_files_recursive(dir: &PathBuf, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    out.push(path);
                } else if path.is_dir() {
                    collect_files_recursive(&path, out);
                }
            }
        }
    }

    // ── strategy helpers ──────────────────────────────────────────────────────

    /// Generate 0–8 unique producer ids (safe ASCII identifiers).
    fn producer_ids_strategy() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[a-z][a-z0-9-]{0,15}", 0usize..=8usize).prop_map(|mut ids| {
            ids.sort();
            ids.dedup();
            ids
        })
    }

    // ── Property 1: Registry artifact write idempotence ──────────────────────
    // Validates: Requirements 1.2

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 1: Registry artifact write idempotence**
        /// Validates: Requirements 1.2
        #[test]
        fn prop1_registry_artifact_write_idempotence(
            producer_ids in producer_ids_strategy()
        ) {
            let dir = temp_dir();
            let snapshot = make_snapshot(&producer_ids);
            let path = dir.join("context/registry_snapshot.json");
            fs::create_dir_all(path.parent().unwrap()).expect("create dir");

            // First write
            let result1 = write_canonical_json_file_if_absent_or_same(
                &path,
                &snapshot,
                "write_failed",
                "conflict",
            );
            prop_assert!(result1.is_ok(), "first write failed: {:?}", result1);

            let bytes_after_first = fs::read(&path).expect("read after first write");

            // Second write — must succeed and produce identical bytes
            let result2 = write_canonical_json_file_if_absent_or_same(
                &path,
                &snapshot,
                "write_failed",
                "conflict",
            );
            prop_assert!(result2.is_ok(), "second write failed: {:?}", result2);

            let bytes_after_second = fs::read(&path).expect("read after second write");
            prop_assert_eq!(bytes_after_first, bytes_after_second,
                "file bytes changed between identical writes");

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 2: Registry diagnostics hash consistency ────────────────────
    // Validates: Requirements 3.3, 5.3, 5.4

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 2: Registry diagnostics hash consistency**
        /// Validates: Requirements 3.3, 5.3, 5.4
        #[test]
        fn prop2_registry_diagnostics_hash_consistency(
            producer_ids in producer_ids_strategy()
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p2";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let expected_hash = compute_registry_snapshot_hash(&snapshot)
                .expect("compute hash");
            write_snapshot_canonical(&run_dir, &snapshot);

            let body = call_registry_endpoint(&dir, run_id);
            prop_assert_eq!(
                body.get("declared_registry_snapshot_hash")
                    .and_then(|v| v.as_str()),
                Some(expected_hash.as_str()),
                "declared hash must equal compute_registry_snapshot_hash"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 3: Entry count matches producers map ─────────────────────────
    // Validates: Requirements 3.4

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 6: Entry count matches producers map**
        /// Validates: Requirements 3.4
        #[test]
        fn prop3_entry_count_matches_producers_len(
            producer_ids in producer_ids_strategy()
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p3";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let expected_count = snapshot.producers.len();
            write_snapshot_canonical(&run_dir, &snapshot);

            let body = call_registry_endpoint(&dir, run_id);
            prop_assert_eq!(
                body.get("declared_registry_entry_count")
                    .and_then(|v| v.as_u64()),
                Some(expected_count as u64),
                "declared_registry_entry_count must equal producers.len()"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 4: Context binding status correctness ────────────────────────
    // Validates: Requirements 3.5, 3.6

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 3: Context binding status correctness**
        /// Validates: Requirements 3.5, 3.6
        #[test]
        fn prop4_context_binding_status_correctness(
            producer_ids in producer_ids_strategy(),
            use_correct_hash in any::<bool>(),
            include_context_object in any::<bool>(),
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p4";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let real_hash = compute_registry_snapshot_hash(&snapshot).expect("hash");
            write_snapshot_canonical(&run_dir, &snapshot);

            if include_context_object {
                let ctx_hash = if use_correct_hash {
                    real_hash.clone()
                } else {
                    "sha256:deliberately-wrong-hash".to_string()
                };
                write_context_object_with_hash(&run_dir, &ctx_hash);

                let body = call_registry_endpoint(&dir, run_id);
                let matches = body
                    .get("context_binding_status")
                    .and_then(|v| v.get("registry_snapshot_hash_matches_declared_context"))
                    .and_then(|v| v.as_bool());
                prop_assert_eq!(
                    matches,
                    Some(use_correct_hash),
                    "binding status must reflect actual hash equality"
                );
            } else {
                // no context object → must be null
                let body = call_registry_endpoint(&dir, run_id);
                let field = body
                    .get("context_binding_status")
                    .and_then(|v| v.get("registry_snapshot_hash_matches_declared_context"));
                prop_assert!(
                    field.is_some_and(|v| v.is_null()),
                    "absent context object must yield null binding status"
                );
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 5: Source observation completeness ───────────────────────────
    // Validates: Requirements 4.1, 4.2

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 5: Source observation completeness**
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop5_source_observation_completeness(
            producer_ids in producer_ids_strategy(),
            include_context_object in any::<bool>(),
            include_receipt in any::<bool>(),
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p5";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let hash = compute_registry_snapshot_hash(&snapshot).expect("hash");
            write_snapshot_canonical(&run_dir, &snapshot);

            if include_context_object {
                write_context_object_with_hash(&run_dir, &hash);
            }
            if include_receipt {
                write_receipt_with_hash(&run_dir, &hash);
            }

            let body = call_registry_endpoint(&dir, run_id);
            let sources = body
                .get("observed_registry_hash_sources")
                .and_then(|v| v.as_array())
                .expect("observed_registry_hash_sources array");

            let has_ctx_source = sources.iter().any(|s| {
                s.get("source").and_then(|v| v.as_str()) == Some("verification_context_object")
            });
            let has_receipt_source = sources.iter().any(|s| {
                s.get("source").and_then(|v| v.as_str()) == Some("receipt")
            });

            prop_assert_eq!(
                has_ctx_source, include_context_object,
                "context_object source presence must match artifact presence"
            );
            prop_assert_eq!(
                has_receipt_source, include_receipt,
                "receipt source presence must match artifact presence"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 6: Observed sources values are unique and sorted ─────────────
    // Validates: Requirements 3.7

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 4: Observed sources values are unique and sorted**
        /// Validates: Requirements 3.7
        #[test]
        fn prop6_observed_sources_values_unique_and_sorted(
            producer_ids in producer_ids_strategy(),
            include_context_object in any::<bool>(),
            include_receipt in any::<bool>(),
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p6";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let hash = compute_registry_snapshot_hash(&snapshot).expect("hash");
            write_snapshot_canonical(&run_dir, &snapshot);

            if include_context_object {
                write_context_object_with_hash(&run_dir, &hash);
            }
            if include_receipt {
                write_receipt_with_hash(&run_dir, &hash);
            }

            let body = call_registry_endpoint(&dir, run_id);
            let sources = body
                .get("observed_registry_hash_sources")
                .and_then(|v| v.as_array())
                .expect("observed_registry_hash_sources array");

            for source in sources {
                let values: Vec<&str> = source
                    .get("values")
                    .and_then(|v| v.as_array())
                    .expect("values array")
                    .iter()
                    .map(|v| v.as_str().expect("string value"))
                    .collect();

                // no duplicates
                let mut deduped = values.clone();
                deduped.dedup();
                prop_assert_eq!(values.len(), deduped.len(), "values must have no duplicates");

                // lexicographically sorted
                let mut sorted = values.clone();
                sorted.sort();
                prop_assert_eq!(values, sorted, "values must be in lexicographic order");
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 7: Empty sources are omitted ─────────────────────────────────
    // Validates: Requirements 3.8, 4.3

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 5 (edge): Empty sources are omitted**
        /// Validates: Requirements 3.8, 4.3
        #[test]
        fn prop7_empty_sources_are_omitted(
            producer_ids in producer_ids_strategy(),
            include_context_object in any::<bool>(),
            include_receipt in any::<bool>(),
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p7";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let hash = compute_registry_snapshot_hash(&snapshot).expect("hash");
            write_snapshot_canonical(&run_dir, &snapshot);

            if include_context_object {
                write_context_object_with_hash(&run_dir, &hash);
            }
            if include_receipt {
                write_receipt_with_hash(&run_dir, &hash);
            }

            let body = call_registry_endpoint(&dir, run_id);
            let sources = body
                .get("observed_registry_hash_sources")
                .and_then(|v| v.as_array())
                .expect("observed_registry_hash_sources array");

            for source in sources {
                let values = source
                    .get("values")
                    .and_then(|v| v.as_array())
                    .expect("values array");
                prop_assert!(
                    !values.is_empty(),
                    "no source entry with empty values array should appear"
                );
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 8: Endpoint is read-only ─────────────────────────────────────
    // Validates: Requirements 5.1, 5.2

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 7: Endpoint is read-only**
        /// Validates: Requirements 5.1, 5.2
        #[test]
        fn prop8_endpoint_is_read_only(
            producer_ids in producer_ids_strategy(),
            call_count in 1usize..=3usize,
        ) {
            let dir = temp_dir();
            let run_id = "run-pbt-p8";
            let run_dir = dir.join(run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            let snapshot = make_snapshot(&producer_ids);
            let hash = compute_registry_snapshot_hash(&snapshot).expect("hash");
            write_snapshot_canonical(&run_dir, &snapshot);
            write_context_object_with_hash(&run_dir, &hash);
            write_receipt_with_hash(&run_dir, &hash);

            let files_before = collect_run_files(&run_dir);

            for _ in 0..call_count {
                let _ = call_registry_endpoint(&dir, run_id);
            }

            let files_after = collect_run_files(&run_dir);
            prop_assert_eq!(
                files_before, files_after,
                "GET registry endpoint must not modify any files on disk"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase-13 Replicated Verification Boundary — Unit Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests_boundary {
    use super::{route_request, DiagnosticsResponse};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-boundary-test")
    }

    fn write_manifest(run_dir: &PathBuf, fingerprint: &str, verdict: &str) {
        let path = run_dir.join("proofd_run_manifest.json");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&json!({
                "run_id": run_dir.file_name().unwrap().to_string_lossy(),
                "request_fingerprint": fingerprint,
                "verdict": verdict,
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_context_object(run_dir: &PathBuf, ctx_id: &str) {
        let ctx_dir = run_dir.join("context");
        fs::create_dir_all(&ctx_dir).unwrap();
        fs::write(
            ctx_dir.join("verification_context_object.json"),
            serde_json::to_vec_pretty(&json!({ "verification_context_id": ctx_id })).unwrap(),
        )
        .unwrap();
    }

    fn write_registry_snapshot(run_dir: &PathBuf, version: u32) {
        let ctx_dir = run_dir.join("context");
        fs::create_dir_all(&ctx_dir).unwrap();
        fs::write(
            ctx_dir.join("registry_snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "registry_format_version": 1,
                "registry_version": version,
                "registry_snapshot_hash": "",
                "producers": {}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn body_json(response: DiagnosticsResponse) -> serde_json::Value {
        serde_json::from_slice(&response.body).expect("valid json body")
    }

    fn call_boundary(evidence_dir: &PathBuf, run_id: &str) -> DiagnosticsResponse {
        route_request(
            "GET",
            &format!("/diagnostics/runs/{run_id}/boundary"),
            evidence_dir,
        )
    }

    // ── Happy path: single run, no peers ─────────────────────────────────────
    #[test]
    fn boundary_single_run_no_peers() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, "sha256:fp1", "TRUSTED");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(body.get("run_id").and_then(|v| v.as_str()), Some("run-a"));
        assert_eq!(
            body.get("request_fingerprint").and_then(|v| v.as_str()),
            Some("sha256:fp1")
        );
        assert_eq!(body.get("peer_run_count").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(
            body.get("peer_run_ids")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        let verdicts = body
            .get("verdict_consistency")
            .and_then(|v| v.get("observed_verdicts"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(verdicts.len(), 1);
        assert_eq!(
            verdicts[0].get("verdict").and_then(|v| v.as_str()),
            Some("TRUSTED")
        );
        assert_eq!(
            body.get("verdict_consistency")
                .and_then(|v| v.get("all_verdicts_match"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Happy path: primary + 2 peers, same fingerprint ──────────────────────
    #[test]
    fn boundary_primary_plus_two_peers() {
        let dir = temp_dir();
        for id in ["run-a", "run-b", "run-c"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, "sha256:same-fp", "TRUSTED");
        }

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(body.get("peer_run_count").and_then(|v| v.as_u64()), Some(2));
        let peer_ids = body.get("peer_run_ids").and_then(|v| v.as_array()).unwrap();
        assert_eq!(peer_ids.len(), 2);
        let verdicts = body
            .get("verdict_consistency")
            .and_then(|v| v.get("observed_verdicts"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(verdicts.len(), 3);
        assert_eq!(
            body.get("verdict_consistency")
                .and_then(|v| v.get("all_verdicts_match"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Sibling with different fingerprint is not a peer ─────────────────────
    #[test]
    fn boundary_different_fingerprint_not_peer() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b");
        fs::create_dir_all(&run_a).unwrap();
        fs::create_dir_all(&run_b).unwrap();
        write_manifest(&run_a, "sha256:fp-A", "TRUSTED");
        write_manifest(&run_b, "sha256:fp-B", "TRUSTED");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(body.get("peer_run_count").and_then(|v| v.as_u64()), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Verdict mismatch → all_verdicts_match = false ─────────────────────────
    #[test]
    fn boundary_verdict_mismatch() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b");
        fs::create_dir_all(&run_a).unwrap();
        fs::create_dir_all(&run_b).unwrap();
        write_manifest(&run_a, "sha256:fp1", "TRUSTED");
        write_manifest(&run_b, "sha256:fp1", "UNTRUSTED");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(
            body.get("verdict_consistency")
                .and_then(|v| v.get("all_verdicts_match"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Context hash consistency: all match ───────────────────────────────────
    #[test]
    fn boundary_context_hash_all_match() {
        let dir = temp_dir();
        for id in ["run-a", "run-b"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, "sha256:fp1", "TRUSTED");
            write_context_object(&run_dir, "ctx-hash-xyz");
        }

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(
            body.get("context_hash_consistency")
                .and_then(|v| v.get("all_context_hashes_match"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("context_hash_consistency")
                .and_then(|v| v.get("observed_context_hashes"))
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Context hash mismatch ─────────────────────────────────────────────────
    #[test]
    fn boundary_context_hash_mismatch() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b");
        fs::create_dir_all(&run_a).unwrap();
        fs::create_dir_all(&run_b).unwrap();
        write_manifest(&run_a, "sha256:fp1", "TRUSTED");
        write_manifest(&run_b, "sha256:fp1", "TRUSTED");
        write_context_object(&run_a, "ctx-hash-AAA");
        write_context_object(&run_b, "ctx-hash-BBB");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(
            body.get("context_hash_consistency")
                .and_then(|v| v.get("all_context_hashes_match"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── No context objects → null ─────────────────────────────────────────────
    #[test]
    fn boundary_no_context_objects_null() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, "sha256:fp1", "TRUSTED");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert!(body
            .get("context_hash_consistency")
            .and_then(|v| v.get("all_context_hashes_match"))
            .is_some_and(|v| v.is_null()));
        assert_eq!(
            body.get("context_hash_consistency")
                .and_then(|v| v.get("observed_context_hashes"))
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Registry hash consistency: all match ──────────────────────────────────
    #[test]
    fn boundary_registry_hash_all_match() {
        let dir = temp_dir();
        for id in ["run-a", "run-b"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, "sha256:fp1", "TRUSTED");
            write_registry_snapshot(&run_dir, 1); // same version → same hash
        }

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(
            body.get("registry_hash_consistency")
                .and_then(|v| v.get("all_registry_hashes_match"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("registry_hash_consistency")
                .and_then(|v| v.get("observed_registry_hashes"))
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(2)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Registry hash mismatch ────────────────────────────────────────────────
    #[test]
    fn boundary_registry_hash_mismatch() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b");
        fs::create_dir_all(&run_a).unwrap();
        fs::create_dir_all(&run_b).unwrap();
        write_manifest(&run_a, "sha256:fp1", "TRUSTED");
        write_manifest(&run_b, "sha256:fp1", "TRUSTED");
        write_registry_snapshot(&run_a, 1);
        write_registry_snapshot(&run_b, 2); // different version → different hash
        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(
            body.get("registry_hash_consistency")
                .and_then(|v| v.get("all_registry_hashes_match"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── No registry snapshots → null ──────────────────────────────────────────
    #[test]
    fn boundary_no_registry_snapshots_null() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, "sha256:fp1", "TRUSTED");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert!(body
            .get("registry_hash_consistency")
            .and_then(|v| v.get("all_registry_hashes_match"))
            .is_some_and(|v| v.is_null()));
        assert_eq!(
            body.get("registry_hash_consistency")
                .and_then(|v| v.get("observed_registry_hashes"))
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Sibling with missing manifest is silently skipped ─────────────────────
    #[test]
    fn boundary_sibling_missing_manifest_skipped() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b"); // no manifest
        fs::create_dir_all(&run_a).unwrap();
        fs::create_dir_all(&run_b).unwrap();
        write_manifest(&run_a, "sha256:fp1", "TRUSTED");

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(body.get("peer_run_count").and_then(|v| v.as_u64()), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Sibling with malformed manifest is silently skipped ───────────────────
    #[test]
    fn boundary_sibling_malformed_manifest_skipped() {
        let dir = temp_dir();
        let run_a = dir.join("run-a");
        let run_b = dir.join("run-b");
        fs::create_dir_all(&run_a).unwrap();
        fs::create_dir_all(&run_b).unwrap();
        write_manifest(&run_a, "sha256:fp1", "TRUSTED");
        fs::write(run_b.join("proofd_run_manifest.json"), b"not-json").unwrap();

        let body = body_json(call_boundary(&dir, "run-a"));
        assert_eq!(body.get("peer_run_count").and_then(|v| v.as_u64()), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Missing run directory → 404 run_dir_not_found ────────────────────────
    #[test]
    fn boundary_missing_run_dir_404() {
        let dir = temp_dir();
        let response = call_boundary(&dir, "run-nonexistent");
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("run_dir_not_found")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Missing manifest → 404 artifact_not_found ────────────────────────────
    #[test]
    fn boundary_missing_manifest_404() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        // no manifest written

        let response = call_boundary(&dir, "run-a");
        assert_eq!(response.status_code, 404);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("artifact_not_found")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Malformed manifest → 500 invalid_run_manifest ────────────────────────
    #[test]
    fn boundary_malformed_manifest_500() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(run_dir.join("proofd_run_manifest.json"), b"not-json").unwrap();

        let response = call_boundary(&dir, "run-a");
        assert_eq!(response.status_code, 500);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("invalid_run_manifest")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Manifest missing request_fingerprint → 500 invalid_run_manifest ───────
    #[test]
    fn boundary_manifest_missing_fingerprint_500() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(
            run_dir.join("proofd_run_manifest.json"),
            serde_json::to_vec_pretty(&json!({ "verdict": "TRUSTED" })).unwrap(),
        )
        .unwrap();

        let response = call_boundary(&dir, "run-a");
        assert_eq!(response.status_code, 500);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("invalid_run_manifest")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Query string → 400 unsupported_query_parameter ───────────────────────
    #[test]
    fn boundary_query_string_rejected() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, "sha256:fp1", "TRUSTED");

        let response = route_request(
            "GET",
            "/diagnostics/runs/run-a/boundary?select_winner=true",
            &dir,
        );
        assert_eq!(response.status_code, 400);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── POST → 405 method_not_allowed ─────────────────────────────────────────
    #[test]
    fn boundary_post_method_not_allowed() {
        let dir = temp_dir();
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, "sha256:fp1", "TRUSTED");

        let response = route_request("POST", "/diagnostics/runs/run-a/boundary", &dir);
        assert_eq!(response.status_code, 405);
        let body = body_json(response);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("method_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Observation arrays sorted by run_id ───────────────────────────────────
    #[test]
    fn boundary_observation_arrays_sorted_by_run_id() {
        let dir = temp_dir();
        // Write in reverse order to ensure sort is applied
        for id in ["run-c", "run-a", "run-b"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, "sha256:fp1", "TRUSTED");
            write_context_object(&run_dir, &format!("ctx-{id}"));
            write_registry_snapshot(&run_dir, 1);
        }

        let body = body_json(call_boundary(&dir, "run-a"));

        let verdict_ids: Vec<&str> = body
            .get("verdict_consistency")
            .and_then(|v| v.get("observed_verdicts"))
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("run_id").and_then(|v| v.as_str()))
            .collect();
        assert_eq!(verdict_ids, vec!["run-a", "run-b", "run-c"]);

        let ctx_ids: Vec<&str> = body
            .get("context_hash_consistency")
            .and_then(|v| v.get("observed_context_hashes"))
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("run_id").and_then(|v| v.as_str()))
            .collect();
        assert_eq!(ctx_ids, vec!["run-a", "run-b", "run-c"]);

        let reg_ids: Vec<&str> = body
            .get("registry_hash_consistency")
            .and_then(|v| v.get("observed_registry_hashes"))
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("run_id").and_then(|v| v.as_str()))
            .collect();
        assert_eq!(reg_ids, vec!["run-a", "run-b", "run-c"]);

        let _ = fs::remove_dir_all(&dir);
    }

    // ── peer_run_ids is sorted ────────────────────────────────────────────────
    #[test]
    fn boundary_peer_run_ids_sorted() {
        let dir = temp_dir();
        for id in ["run-z", "run-a", "run-m", "run-b"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, "sha256:fp1", "TRUSTED");
        }

        let body = body_json(call_boundary(&dir, "run-a"));
        let peer_ids: Vec<&str> = body
            .get("peer_run_ids")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        let mut sorted = peer_ids.clone();
        sorted.sort();
        assert_eq!(peer_ids, sorted);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — basic discovery ─────────────────────
    #[test]
    fn fingerprints_endpoint_returns_matching_runs() {
        let dir = temp_dir();
        let fp_match = "sha256:".to_string() + &"a".repeat(64);
        let fp_other = "sha256:".to_string() + &"b".repeat(64);
        for id in ["run-a", "run-b"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, &fp_match, "PASS");
        }
        let other_dir = dir.join("run-c");
        fs::create_dir_all(&other_dir).unwrap();
        write_manifest(&other_dir, &fp_other, "PASS");

        let response = route_request(
            "GET",
            &format!("/diagnostics/fingerprints/{fp_match}"),
            &dir,
        );
        assert_eq!(response.status_code, 200);
        let body = body_json(response);

        let run_ids: Vec<&str> = body
            .get("run_ids")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(run_ids, vec!["run-a", "run-b"]);
        assert_eq!(body.get("run_count").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(
            body.get("request_fingerprint").and_then(|v| v.as_str()),
            Some(fp_match.as_str())
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — 404 when no match ───────────────────
    #[test]
    fn fingerprints_endpoint_404_when_no_match() {
        let dir = temp_dir();
        let fp_other = "sha256:".to_string() + &"b".repeat(64);
        let fp_missing = "sha256:".to_string() + &"c".repeat(64);
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, &fp_other, "PASS");

        let response = route_request(
            "GET",
            &format!("/diagnostics/fingerprints/{fp_missing}"),
            &dir,
        );
        assert_eq!(response.status_code, 404);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — run_ids sorted ─────────────────────
    #[test]
    fn fingerprints_endpoint_run_ids_sorted() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"1".repeat(64);
        for id in ["run-z", "run-a", "run-m"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, &fp, "PASS");
        }

        let response = route_request("GET", &format!("/diagnostics/fingerprints/{fp}"), &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        let run_ids: Vec<&str> = body
            .get("run_ids")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        let mut sorted = run_ids.clone();
        sorted.sort();
        assert_eq!(run_ids, sorted);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — POST → 405 ──────────────────────────
    #[test]
    fn fingerprints_endpoint_post_method_not_allowed() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"1".repeat(64);
        let response = route_request("POST", &format!("/diagnostics/fingerprints/{fp}"), &dir);
        assert_eq!(response.status_code, 405);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — verdict consistency ─────────────────
    #[test]
    fn fingerprints_endpoint_verdict_consistency_true_when_all_match() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"2".repeat(64);
        for id in ["run-a", "run-b", "run-c"] {
            let run_dir = dir.join(id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, &fp, "PASS");
        }

        let response = route_request("GET", &format!("/diagnostics/fingerprints/{fp}"), &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("verdict_consistency")
                .and_then(|v| v.get("all_verdicts_match"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — verdict inconsistency ───────────────
    #[test]
    fn fingerprints_endpoint_verdict_consistency_false_when_mismatch() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"3".repeat(64);
        let run_dir_a = dir.join("run-a");
        fs::create_dir_all(&run_dir_a).unwrap();
        write_manifest(&run_dir_a, &fp, "PASS");
        let run_dir_b = dir.join("run-b");
        fs::create_dir_all(&run_dir_b).unwrap();
        write_manifest(&run_dir_b, &fp, "FAIL");

        let response = route_request("GET", &format!("/diagnostics/fingerprints/{fp}"), &dir);
        assert_eq!(response.status_code, 200);
        let body = body_json(response);
        assert_eq!(
            body.get("verdict_consistency")
                .and_then(|v| v.get("all_verdicts_match"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — invalid format → 400 ───────────────
    #[test]
    fn fingerprints_endpoint_invalid_format_400() {
        let dir = temp_dir();
        // Not sha256: prefix
        let response = route_request("GET", "/diagnostics/fingerprints/fp-abc123", &dir);
        assert_eq!(response.status_code, 400);
        // sha256: but wrong length
        let response2 = route_request("GET", "/diagnostics/fingerprints/sha256:abc123", &dir);
        assert_eq!(response2.status_code, 400);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── /diagnostics/fingerprints/{fp} — no forbidden fields in response ─────
    #[test]
    fn fingerprints_endpoint_no_forbidden_fields_in_response() {
        let dir = temp_dir();
        let fp = "sha256:".to_string() + &"4".repeat(64);
        let run_dir = dir.join("run-a");
        fs::create_dir_all(&run_dir).unwrap();
        write_manifest(&run_dir, &fp, "PASS");

        let response = route_request("GET", &format!("/diagnostics/fingerprints/{fp}"), &dir);
        assert_eq!(response.status_code, 200);
        let body_str = String::from_utf8_lossy(&response.body);
        for field in super::PHASE13_FORBIDDEN_FIELDS {
            assert!(
                !body_str.contains(field),
                "forbidden field '{field}' found in fingerprints response"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }
} // end mod tests_boundary

// ─────────────────────────────────────────────────────────────────────────────
// Phase-13 Replicated Verification Boundary — Property-Based Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod proptest_boundary {
    use super::{route_request, DiagnosticsResponse};
    use proptest::prelude::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-boundary-pbt")
    }

    fn write_manifest(run_dir: &PathBuf, fingerprint: &str, verdict: &str) {
        fs::write(
            run_dir.join("proofd_run_manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": run_dir.file_name().unwrap().to_string_lossy(),
                "request_fingerprint": fingerprint,
                "verdict": verdict,
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_context_object(run_dir: &PathBuf, ctx_id: &str) {
        let ctx_dir = run_dir.join("context");
        fs::create_dir_all(&ctx_dir).unwrap();
        fs::write(
            ctx_dir.join("verification_context_object.json"),
            serde_json::to_vec_pretty(&json!({ "verification_context_id": ctx_id })).unwrap(),
        )
        .unwrap();
    }

    fn write_registry_snapshot(run_dir: &PathBuf, version: u32) {
        let ctx_dir = run_dir.join("context");
        fs::create_dir_all(&ctx_dir).unwrap();
        fs::write(
            ctx_dir.join("registry_snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "registry_format_version": 1,
                "registry_version": version,
                "registry_snapshot_hash": "",
                "producers": {}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn collect_files(dir: &PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();
        fn walk(dir: &PathBuf, out: &mut Vec<PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        out.push(path);
                    } else if path.is_dir() {
                        walk(&path, out);
                    }
                }
            }
        }
        walk(dir, &mut files);
        files.sort();
        files
    }

    fn body_json(response: DiagnosticsResponse) -> serde_json::Value {
        serde_json::from_slice(&response.body).expect("valid json body")
    }

    fn call_boundary(evidence_dir: &PathBuf, run_id: &str) -> DiagnosticsResponse {
        route_request(
            "GET",
            &format!("/diagnostics/runs/{run_id}/boundary"),
            evidence_dir,
        )
    }

    fn safe_run_id(suffix: &str) -> String {
        format!("run-pbt-{suffix}")
    }

    // ── Property 1: Response structure completeness ───────────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 1: Response structure completeness**
        /// Validates: Requirements 1.1, 3.1, 4.1, 4.2, 4.3
        #[test]
        fn prop1_response_structure_completeness(
            fp_suffix in "[a-f0-9]{8}",
            verdict in prop::sample::select(vec!["TRUSTED", "UNTRUSTED", "INVALID"]),
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p1");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).unwrap();
            let fingerprint = format!("sha256:{fp_suffix}");
            write_manifest(&run_dir, &fingerprint, verdict);

            let body = body_json(call_boundary(&dir, &run_id));
            prop_assert_eq!(body.get("run_id").and_then(|v| v.as_str()), Some(run_id.as_str()));
            prop_assert_eq!(
                body.get("request_fingerprint").and_then(|v| v.as_str()),
                Some(fingerprint.as_str())
            );
            prop_assert!(body.get("peer_run_count").and_then(|v| v.as_u64()).is_some());
            prop_assert!(body.get("peer_run_ids").and_then(|v| v.as_array()).is_some());
            prop_assert!(body.get("verdict_consistency").is_some());
            prop_assert!(body.get("context_hash_consistency").is_some());
            prop_assert!(body.get("registry_hash_consistency").is_some());

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 2: Peer discovery accuracy ──────────────────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Property 2: Peer discovery accuracy**
        /// Validates: Requirements 2.1, 2.4, 4.3, 4.4
        #[test]
        fn prop2_peer_discovery_accuracy(
            n_peers in 0usize..=4usize,
            m_other in 0usize..=3usize,
        ) {
            let dir = temp_dir();
            let primary_id = safe_run_id("p2-primary");
            let shared_fp = "sha256:shared-fingerprint";

            // Primary + n_peers share the same fingerprint
            let primary_dir = dir.join(&primary_id);
            fs::create_dir_all(&primary_dir).unwrap();
            write_manifest(&primary_dir, shared_fp, "TRUSTED");

            for i in 0..n_peers {
                let peer_id = format!("run-pbt-p2-peer-{i:02}");
                let peer_dir = dir.join(&peer_id);
                fs::create_dir_all(&peer_dir).unwrap();
                write_manifest(&peer_dir, shared_fp, "TRUSTED");
            }

            // m_other runs with a different fingerprint
            for i in 0..m_other {
                let other_id = format!("run-pbt-p2-other-{i:02}");
                let other_dir = dir.join(&other_id);
                fs::create_dir_all(&other_dir).unwrap();
                write_manifest(&other_dir, "sha256:different-fp", "TRUSTED");
            }

            let body = body_json(call_boundary(&dir, &primary_id));
            prop_assert_eq!(
                body.get("peer_run_count").and_then(|v| v.as_u64()),
                Some(n_peers as u64),
                "peer_run_count must equal n_peers"
            );
            let observed_len = body
                .get("verdict_consistency")
                .and_then(|v| v.get("observed_verdicts"))
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            prop_assert_eq!(
                observed_len,
                n_peers + 1,
                "observed_verdicts must include primary + all peers"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 3: Verdict consistency semantics ─────────────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 3: Verdict consistency semantics**
        /// Validates: Requirements 4.4, 5.1
        #[test]
        fn prop3_verdict_consistency_semantics(
            verdicts in prop::collection::vec(
                prop::sample::select(vec!["TRUSTED", "UNTRUSTED", "INVALID"]),
                1..=5,
            ),
        ) {
            let dir = temp_dir();
            let fp = "sha256:fp-prop3";
            let primary_id = safe_run_id("p3-primary");

            let primary_dir = dir.join(&primary_id);
            fs::create_dir_all(&primary_dir).unwrap();
            write_manifest(&primary_dir, fp, verdicts[0]);

            for (i, v) in verdicts[1..].iter().enumerate() {
                let peer_id = format!("run-pbt-p3-peer-{i:02}");
                let peer_dir = dir.join(&peer_id);
                fs::create_dir_all(&peer_dir).unwrap();
                write_manifest(&peer_dir, fp, v);
            }

            let body = body_json(call_boundary(&dir, &primary_id));
            let all_match = body
                .get("verdict_consistency")
                .and_then(|v| v.get("all_verdicts_match"))
                .and_then(|v| v.as_bool())
                .unwrap();
            let expected_all_match = verdicts.windows(2).all(|w| w[0] == w[1]);
            prop_assert_eq!(
                all_match, expected_all_match,
                "all_verdicts_match must reflect actual verdict equality"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 4: Context hash consistency semantics ───────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 4: Context hash consistency semantics**
        /// Validates: Requirements 4.5, 4.7, 5.2, 5.5
        #[test]
        fn prop4_context_hash_consistency_semantics(
            // Each element: (has_context_object, ctx_id_suffix)
            runs in prop::collection::vec(
                (any::<bool>(), "[a-f0-9]{4}"),
                1..=5usize,
            ),
        ) {
            let dir = temp_dir();
            let fp = "sha256:fp-prop4";
            let primary_id = safe_run_id("p4-primary");

            // Write primary (always has manifest; context object conditional)
            let primary_dir = dir.join(&primary_id);
            fs::create_dir_all(&primary_dir).unwrap();
            let (primary_has_ctx, primary_ctx_suffix) = &runs[0];
            let primary_ctx_id = format!("ctx-{primary_ctx_suffix}");
            write_manifest(&primary_dir, fp, "TRUSTED");
            if *primary_has_ctx {
                write_context_object(&primary_dir, &primary_ctx_id);
            }

            // Write peers
            let mut expected_ctx_entries: Vec<(String, String)> = Vec::new();
            if *primary_has_ctx {
                expected_ctx_entries.push((primary_id.clone(), primary_ctx_id.clone()));
            }
            for (i, (has_ctx, ctx_suffix)) in runs[1..].iter().enumerate() {
                let peer_id = format!("run-pbt-p4-peer-{i:02}");
                let peer_dir = dir.join(&peer_id);
                fs::create_dir_all(&peer_dir).unwrap();
                let ctx_id = format!("ctx-{ctx_suffix}");
                write_manifest(&peer_dir, fp, "TRUSTED");
                if *has_ctx {
                    write_context_object(&peer_dir, &ctx_id);
                    expected_ctx_entries.push((peer_id, ctx_id));
                }
            }
            // Sort by run_id to match expected response order
            expected_ctx_entries.sort_by(|a, b| a.0.cmp(&b.0));

            let body = body_json(call_boundary(&dir, &primary_id));
            let observed = body
                .get("context_hash_consistency")
                .and_then(|v| v.get("observed_context_hashes"))
                .and_then(|v| v.as_array())
                .unwrap();

            // Exactly one entry per run that has the artifact
            prop_assert_eq!(
                observed.len(),
                expected_ctx_entries.len(),
                "observed_context_hashes length must equal runs with context objects"
            );

            // Each entry's hash equals the verification_context_id
            for (entry, (exp_run_id, exp_hash)) in observed.iter().zip(expected_ctx_entries.iter()) {
                prop_assert_eq!(
                    entry.get("run_id").and_then(|v| v.as_str()),
                    Some(exp_run_id.as_str()),
                    "run_id must match"
                );
                prop_assert_eq!(
                    entry.get("hash").and_then(|v| v.as_str()),
                    Some(exp_hash.as_str()),
                    "hash must equal verification_context_id"
                );
            }

            // all_context_hashes_match semantics
            let all_match_val = body
                .get("context_hash_consistency")
                .and_then(|v| v.get("all_context_hashes_match"))
                .unwrap();
            if expected_ctx_entries.is_empty() {
                prop_assert!(all_match_val.is_null(), "must be null when no context objects");
            } else {
                let all_same = expected_ctx_entries.windows(2).all(|w| w[0].1 == w[1].1);
                prop_assert_eq!(
                    all_match_val.as_bool(),
                    Some(all_same),
                    "all_context_hashes_match must reflect actual hash equality"
                );
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 5: Registry hash consistency semantics ───────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 5: Registry hash consistency semantics**
        /// Validates: Requirements 4.6, 4.8, 5.3, 5.4
        #[test]
        fn prop5_registry_hash_consistency_semantics(
            // Each element: (has_registry_snapshot, registry_version)
            runs in prop::collection::vec(
                (any::<bool>(), 1u32..=8u32),
                1..=5usize,
            ),
        ) {
            use proof_verifier::registry::snapshot::compute_registry_snapshot_hash;
            use proof_verifier::RegistrySnapshot;

            let dir = temp_dir();
            let fp = "sha256:fp-prop5";
            let primary_id = safe_run_id("p5-primary");

            // Write primary
            let primary_dir = dir.join(&primary_id);
            fs::create_dir_all(&primary_dir).unwrap();
            let (primary_has_reg, primary_version) = &runs[0];
            write_manifest(&primary_dir, fp, "TRUSTED");
            if *primary_has_reg {
                write_registry_snapshot(&primary_dir, *primary_version);
            }

            // Write peers
            for (i, (has_reg, version)) in runs[1..].iter().enumerate() {
                let peer_id = format!("run-pbt-p5-peer-{i:02}");
                let peer_dir = dir.join(&peer_id);
                fs::create_dir_all(&peer_dir).unwrap();
                write_manifest(&peer_dir, fp, "TRUSTED");
                if *has_reg {
                    write_registry_snapshot(&peer_dir, *version);
                }
            }

            let body = body_json(call_boundary(&dir, &primary_id));
            let observed = body
                .get("registry_hash_consistency")
                .and_then(|v| v.get("observed_registry_hashes"))
                .and_then(|v| v.as_array())
                .unwrap();

            // Build expected entries by recomputing hashes ourselves
            let mut all_run_dirs: Vec<(String, PathBuf, bool, u32)> = Vec::new();
            all_run_dirs.push((primary_id.clone(), primary_dir.clone(), *primary_has_reg, *primary_version));
            for (i, (has_reg, version)) in runs[1..].iter().enumerate() {
                let peer_id = format!("run-pbt-p5-peer-{i:02}");
                let peer_dir = dir.join(&peer_id);
                all_run_dirs.push((peer_id, peer_dir, *has_reg, *version));
            }
            all_run_dirs.sort_by(|a, b| a.0.cmp(&b.0));

            let mut expected_reg_entries: Vec<(String, String)> = Vec::new();
            for (run_id, run_dir, has_reg, _version) in &all_run_dirs {
                if !has_reg { continue; }
                let reg_path = run_dir.join("context/registry_snapshot.json");
                if !reg_path.is_file() { continue; }
                let bytes = fs::read(&reg_path).unwrap();
                let snapshot: RegistrySnapshot = serde_json::from_slice(&bytes).unwrap();
                let hash = compute_registry_snapshot_hash(&snapshot).unwrap();
                expected_reg_entries.push((run_id.clone(), hash));
            }

            prop_assert_eq!(
                observed.len(),
                expected_reg_entries.len(),
                "observed_registry_hashes length must equal runs with registry snapshots"
            );

            for (entry, (exp_run_id, exp_hash)) in observed.iter().zip(expected_reg_entries.iter()) {
                prop_assert_eq!(
                    entry.get("run_id").and_then(|v| v.as_str()),
                    Some(exp_run_id.as_str()),
                    "run_id must match"
                );
                prop_assert_eq!(
                    entry.get("hash").and_then(|v| v.as_str()),
                    Some(exp_hash.as_str()),
                    "hash must equal recomputed registry snapshot hash"
                );
            }

            // all_registry_hashes_match semantics
            let all_match_val = body
                .get("registry_hash_consistency")
                .and_then(|v| v.get("all_registry_hashes_match"))
                .unwrap();
            if expected_reg_entries.is_empty() {
                prop_assert!(all_match_val.is_null(), "must be null when no registry snapshots");
            } else {
                let all_same = expected_reg_entries.windows(2).all(|w| w[0].1 == w[1].1);
                prop_assert_eq!(
                    all_match_val.as_bool(),
                    Some(all_same),
                    "all_registry_hashes_match must reflect actual hash equality"
                );
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 6: Observation arrays sorted by run_id ──────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 6: Observation arrays are sorted by run_id**
        /// Validates: Requirements 5.6
        #[test]
        fn prop6_observation_arrays_sorted_by_run_id(
            n_runs in 1usize..=6usize,
        ) {
            let dir = temp_dir();
            let fp = "sha256:fp-prop6";
            let primary_id = safe_run_id("p6-primary");

            let primary_dir = dir.join(&primary_id);
            fs::create_dir_all(&primary_dir).unwrap();
            write_manifest(&primary_dir, fp, "TRUSTED");
            write_context_object(&primary_dir, "ctx-primary");
            write_registry_snapshot(&primary_dir, 1);

            for i in 0..n_runs.saturating_sub(1) {
                let peer_id = format!("run-pbt-p6-peer-{i:04}");
                let peer_dir = dir.join(&peer_id);
                fs::create_dir_all(&peer_dir).unwrap();
                write_manifest(&peer_dir, fp, "TRUSTED");
                write_context_object(&peer_dir, &format!("ctx-peer-{i}"));
                write_registry_snapshot(&peer_dir, 1);
            }

            let body = body_json(call_boundary(&dir, &primary_id));

            let verdict_ids: Vec<String> = body
                .get("verdict_consistency")
                .and_then(|v| v.get("observed_verdicts"))
                .and_then(|v| v.as_array())
                .unwrap()
                .iter()
                .filter_map(|e| e.get("run_id").and_then(|v| v.as_str()).map(String::from))
                .collect();
            let mut sorted_v = verdict_ids.clone();
            sorted_v.sort();
            prop_assert_eq!(&verdict_ids, &sorted_v, "observed_verdicts must be sorted by run_id");

            let ctx_ids: Vec<String> = body
                .get("context_hash_consistency")
                .and_then(|v| v.get("observed_context_hashes"))
                .and_then(|v| v.as_array())
                .unwrap()
                .iter()
                .filter_map(|e| e.get("run_id").and_then(|v| v.as_str()).map(String::from))
                .collect();
            let mut sorted_c = ctx_ids.clone();
            sorted_c.sort();
            prop_assert_eq!(&ctx_ids, &sorted_c, "observed_context_hashes must be sorted by run_id");

            let reg_ids: Vec<String> = body
                .get("registry_hash_consistency")
                .and_then(|v| v.get("observed_registry_hashes"))
                .and_then(|v| v.as_array())
                .unwrap()
                .iter()
                .filter_map(|e| e.get("run_id").and_then(|v| v.as_str()).map(String::from))
                .collect();
            let mut sorted_r = reg_ids.clone();
            sorted_r.sort();
            prop_assert_eq!(&reg_ids, &sorted_r, "observed_registry_hashes must be sorted by run_id");

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 7: Endpoint is read-only ─────────────────────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 7: Endpoint is read-only**
        /// Validates: Requirements 3.4
        #[test]
        fn prop7_endpoint_is_read_only(
            call_count in 1usize..=3usize,
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p7");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).unwrap();
            write_manifest(&run_dir, "sha256:fp-p7", "TRUSTED");
            write_context_object(&run_dir, "ctx-p7");
            write_registry_snapshot(&run_dir, 1);

            let files_before = collect_files(&run_dir);
            for _ in 0..call_count {
                let _ = call_boundary(&dir, &run_id);
            }
            let files_after = collect_files(&run_dir);

            prop_assert_eq!(
                files_before, files_after,
                "GET boundary endpoint must not modify any files on disk"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase-13 Kill-Switch Gates — unit tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests_kill_switch_gates {
    use super::{route_request, DiagnosticsResponse};
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    // ── shared helpers ────────────────────────────────────────────────────────

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-ks")
    }

    fn body_json(r: DiagnosticsResponse) -> Value {
        serde_json::from_slice(&r.body).expect("valid json body")
    }

    pub(super) fn json_contains_key(value: &Value, key: &str) -> bool {
        match value {
            Value::Object(map) => {
                map.contains_key(key) || map.values().any(|v| json_contains_key(v, key))
            }
            Value::Array(arr) => arr.iter().any(|v| json_contains_key(v, key)),
            _ => false,
        }
    }

    pub(super) fn response_contains_forbidden_field(body: &Value, fields: &[&str]) -> bool {
        fields.iter().any(|f| json_contains_key(body, f))
    }

    fn write_json(dir: &PathBuf, name: &str, value: &Value) {
        fs::write(dir.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    // ── Gate 1: ci-gate-proofd-observability-boundary ────────────────────────

    #[test]
    fn gate1_post_diagnostics_graph_returns_405() {
        let dir = temp_dir();
        let r = route_request("POST", "/diagnostics/graph", &dir);
        assert_eq!(r.status_code, 405);
        let body = body_json(r);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("method_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate1_post_diagnostics_authority_topology_returns_405() {
        let dir = temp_dir();
        let r = route_request("POST", "/diagnostics/authority-topology", &dir);
        assert_eq!(r.status_code, 405);
        let body = body_json(r);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("method_not_allowed")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate1_get_diagnostics_graph_with_query_returns_400() {
        let dir = temp_dir();
        let r = route_request("GET", "/diagnostics/graph?select_winner=true", &dir);
        assert_eq!(r.status_code, 400);
        let body = body_json(r);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate1_get_diagnostics_convergence_with_query_returns_400() {
        let dir = temp_dir();
        let r = route_request("GET", "/diagnostics/convergence?commit=true", &dir);
        assert_eq!(r.status_code, 400);
        let body = body_json(r);
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("unsupported_query_parameter")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate1_no_dominant_authority_chain_id_in_convergence_response() {
        let dir = temp_dir();
        let artifact = serde_json::json!({ "status": "ok", "cluster_count": 3 });
        write_json(&dir, "parity_convergence_report.json", &artifact);
        let r = route_request("GET", "/diagnostics/convergence", &dir);
        assert_eq!(r.status_code, 200);
        let body = body_json(r);
        assert!(
            !json_contains_key(&body, "dominant_authority_chain_id"),
            "dominant_authority_chain_id must not appear in convergence response"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate1_no_verification_weight_in_convergence_response() {
        let dir = temp_dir();
        let artifact = serde_json::json!({ "status": "ok", "cluster_count": 3 });
        write_json(&dir, "parity_convergence_report.json", &artifact);
        let r = route_request("GET", "/diagnostics/convergence", &dir);
        assert_eq!(r.status_code, 200);
        let body = body_json(r);
        assert!(
            !json_contains_key(&body, "verification_weight"),
            "verification_weight must not appear in convergence response"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Gate 2: ci-gate-observability-routing-separation (source scan) ────────

    #[test]
    fn gate2_routing_functions_do_not_contain_forbidden_observability_fields() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let source_path = std::path::Path::new(manifest_dir).join("src/lib.rs");
        let source =
            fs::read_to_string(&source_path).expect("failed to read lib.rs for source scan");

        // Only scan production code — stop at the first #[cfg(test)] block.
        // All test modules appear after production code in this file.
        let production_source: String = source
            .lines()
            .take_while(|line| !line.trim_start().starts_with("#[cfg(test)]"))
            .collect::<Vec<_>>()
            .join("\n");

        const FORBIDDEN: &[&str] = &[
            "dominant_authority_chain_id",
            "largest_outcome_cluster_size",
            "outcome_convergence_ratio",
            "global_status",
            "historical_authority_islands",
            "insufficient_evidence_islands",
        ];

        for field in FORBIDDEN {
            assert!(
                !production_source.contains(field),
                "Gate 2 FAIL: forbidden field '{}' found in production routing code (routing separation violation)",
                field
            );
        }
    }

    // ── Gate 3: ci-gate-convergence-non-election-boundary ────────────────────

    #[test]
    fn gate3_winning_cluster_absent_from_convergence_response() {
        // proofd must not inject winning_cluster into convergence responses.
        // Artifact contains no such field; assert it is absent from the response.
        let dir = temp_dir();
        let artifact = serde_json::json!({ "status": "ok", "cluster_count": 3 });
        write_json(&dir, "parity_convergence_report.json", &artifact);
        let r = route_request("GET", "/diagnostics/convergence", &dir);
        assert_eq!(r.status_code, 200);
        let body = body_json(r);
        assert!(
            !json_contains_key(&body, "winning_cluster"),
            "winning_cluster must not appear in convergence response (P13-NEG-07, P13-NEG-08)"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate3_selected_partition_absent_from_drift_response() {
        // proofd must not inject selected_partition into drift responses.
        let dir = temp_dir();
        let artifact = serde_json::json!({ "status": "ok", "partition_count": 2 });
        write_json(&dir, "parity_drift_attribution_report.json", &artifact);
        let r = route_request("GET", "/diagnostics/drift", &dir);
        assert_eq!(r.status_code, 200);
        let body = body_json(r);
        assert!(
            !json_contains_key(&body, "selected_partition"),
            "selected_partition must not appear in drift response (P13-NEG-09, P13-NEG-10)"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ── Gate 4: ci-gate-verifier-reputation-prohibition ──────────────────────

    #[test]
    fn gate4_verifier_score_absent_from_parity_response() {
        // proofd must not inject verifier_score into parity responses.
        let dir = temp_dir();
        let artifact = serde_json::json!({ "status": "ok", "incident_count": 0 });
        write_json(&dir, "parity_report.json", &artifact);
        let r = route_request("GET", "/diagnostics/parity", &dir);
        assert_eq!(r.status_code, 200);
        let body = body_json(r);
        assert!(
            !json_contains_key(&body, "verifier_score"),
            "verifier_score must not appear in parity response (P13-NEG-15)"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate4_trust_score_absent_from_convergence_response() {
        // proofd must not inject trust_score into convergence responses.
        let dir = temp_dir();
        let artifact = serde_json::json!({ "status": "ok", "cluster_count": 1 });
        write_json(&dir, "parity_convergence_report.json", &artifact);
        let r = route_request("GET", "/diagnostics/convergence", &dir);
        assert_eq!(r.status_code, 200);
        let body = body_json(r);
        assert!(
            !json_contains_key(&body, "trust_score"),
            "trust_score must not appear in convergence response (P13-NEG-16)"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase-13 Kill-Switch Gates — property-based tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod proptest_kill_switch_gates {
    use super::tests_kill_switch_gates::{json_contains_key, response_contains_forbidden_field};
    use super::{route_request, DiagnosticsResponse};
    use proptest::prelude::*;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-ks-pbt")
    }

    fn body_json(r: DiagnosticsResponse) -> Value {
        serde_json::from_slice(&r.body).expect("valid json body")
    }

    fn write_json(dir: &PathBuf, name: &str, value: &Value) {
        fs::write(dir.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    // Safe JSON key strategy (no forbidden fields)
    fn safe_key_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z][a-z0-9_]{0,15}").unwrap()
    }

    fn safe_val_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z0-9]{1,20}").unwrap()
    }

    fn safe_artifact_strategy() -> impl Strategy<Value = Value> {
        prop::collection::vec((safe_key_strategy(), safe_val_strategy()), 0..8).prop_map(|pairs| {
            let mut map = serde_json::Map::new();
            for (k, v) in pairs {
                // Exclude any key that matches a forbidden field
                const ALL_FORBIDDEN: &[&str] = &[
                    "dominant_authority_chain_id",
                    "largest_outcome_cluster_size",
                    "outcome_convergence_ratio",
                    "global_status",
                    "historical_authority_islands",
                    "insufficient_evidence_islands",
                    "retry",
                    "override",
                    "promote",
                    "commit",
                    "recommended_action",
                    "mitigation",
                    "routing_hint",
                    "node_priority",
                    "verification_weight",
                    "execution_override",
                    "winning_cluster",
                    "selected_partition",
                    "preferred_cluster",
                    "cluster_policy_input",
                    "partition_replay_admission",
                    "execution_route",
                    "committed_cluster",
                    "verifier_score",
                    "trust_score",
                    "reliability_index",
                    "weighted_authority",
                    "correctness_rate",
                    "agreement_ratio",
                    "node_success_ratio",
                    "verifier_reputation",
                ];
                if !ALL_FORBIDDEN.contains(&k.as_str()) {
                    map.insert(k, Value::String(v));
                }
            }
            Value::Object(map)
        })
    }

    // ── Property 1: POST observability paths always return 405 ───────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 1: POST observability paths always return 405
        /// Validates: Requirements 1.1
        #[test]
        fn prop1_post_observability_paths_always_405(
            suffix in prop::string::string_regex("[a-z0-9-]{1,20}").unwrap(),
        ) {
            let dir = temp_dir();
            let path = format!("/diagnostics/{suffix}");
            let r = route_request("POST", &path, &dir);
            prop_assert_eq!(r.status_code, 405, "POST to {} must return 405", path);
            let body = body_json(r);
            prop_assert_eq!(
                body.get("error").and_then(|v| v.as_str()),
                Some("method_not_allowed")
            );
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 2: Unsupported query always returns 400 ─────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 2: unsupported query parameter always returns 400
        /// Validates: Requirements 1.2, 1.5
        #[test]
        fn prop2_unsupported_query_always_400(
            suffix in prop::string::string_regex("[a-z]{2,15}").unwrap(),
            qkey in prop::string::string_regex("[a-z]{2,10}").unwrap(),
            qval in prop::string::string_regex("[a-z0-9]{1,10}").unwrap(),
        ) {
            // Exclude /diagnostics/incidents which allows query params
            let path = format!("/diagnostics/{suffix}?{qkey}={qval}");
            if path.starts_with("/diagnostics/incidents?") {
                return Ok(());
            }
            let dir = temp_dir();
            let r = route_request("GET", &path, &dir);
            prop_assert_eq!(r.status_code, 400, "GET {} must return 400", path);
            let body = body_json(r);
            prop_assert_eq!(
                body.get("error").and_then(|v| v.as_str()),
                Some("unsupported_query_parameter")
            );
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 3: No forbidden fields in observability responses ────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 3: no forbidden fields in observability responses
        /// Validates: Requirements 1.3, 1.4
        #[test]
        fn prop3_no_forbidden_fields_in_observability_responses(
            artifact in safe_artifact_strategy(),
        ) {
            const FORBIDDEN: &[&str] = &[
                "dominant_authority_chain_id", "largest_outcome_cluster_size",
                "outcome_convergence_ratio", "global_status",
                "historical_authority_islands", "insufficient_evidence_islands",
                "retry", "override", "promote", "commit", "recommended_action",
                "mitigation", "routing_hint", "node_priority",
                "verification_weight", "execution_override",
            ];

            let dir = temp_dir();
            write_json(&dir, "parity_convergence_report.json", &artifact);

            let r = route_request("GET", "/diagnostics/convergence", &dir);
            prop_assert_eq!(r.status_code, 200);
            let body = body_json(r);

            for field in FORBIDDEN {
                prop_assert!(
                    !json_contains_key(&body, field),
                    "forbidden field '{}' found in convergence response",
                    field
                );
            }
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 4: No forbidden election fields in convergence responses ─────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 4: no forbidden election fields in convergence artifact responses
        /// Validates: Requirements 3.1, 3.2
        #[test]
        fn prop4_no_forbidden_election_fields_in_convergence_responses(
            artifact in safe_artifact_strategy(),
        ) {
            const FORBIDDEN: &[&str] = &[
                "winning_cluster", "selected_partition", "preferred_cluster",
                "cluster_policy_input", "partition_replay_admission",
                "verification_weight", "execution_route", "committed_cluster",
            ];

            let dir = temp_dir();
            write_json(&dir, "parity_convergence_report.json", &artifact);
            write_json(&dir, "parity_drift_attribution_report.json", &artifact);

            for (endpoint, file) in &[
                ("/diagnostics/convergence", "parity_convergence_report.json"),
                ("/diagnostics/drift", "parity_drift_attribution_report.json"),
            ] {
                let r = route_request("GET", endpoint, &dir);
                prop_assert_eq!(r.status_code, 200, "endpoint {} must return 200", endpoint);
                let body = body_json(r);
                prop_assert!(
                    !response_contains_forbidden_field(&body, FORBIDDEN),
                    "forbidden election field found in {} response (artifact: {})",
                    endpoint, file
                );
            }
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 5: No forbidden reputation fields in parity responses ────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 5: no forbidden reputation fields in parity artifact responses
        /// Validates: Requirements 4.1, 4.4
        #[test]
        fn prop5_no_forbidden_reputation_fields_in_parity_responses(
            artifact in safe_artifact_strategy(),
        ) {
            const FORBIDDEN: &[&str] = &[
                "verifier_score", "trust_score", "reliability_index",
                "weighted_authority", "correctness_rate", "agreement_ratio",
                "node_success_ratio", "verifier_reputation",
            ];

            let dir = temp_dir();
            write_json(&dir, "parity_report.json", &artifact);
            write_json(&dir, "parity_convergence_report.json", &artifact);
            write_json(&dir, "parity_drift_attribution_report.json", &artifact);
            write_json(&dir, "parity_authority_suppression_report.json", &artifact);
            write_json(&dir, "parity_authority_drift_topology.json", &artifact);
            write_json(&dir, "parity_incident_graph.json", &artifact);

            for endpoint in &[
                "/diagnostics/parity",
                "/diagnostics/convergence",
                "/diagnostics/drift",
                "/diagnostics/authority-suppression",
                "/diagnostics/authority-topology",
                "/diagnostics/graph",
            ] {
                let r = route_request("GET", endpoint, &dir);
                prop_assert_eq!(r.status_code, 200, "endpoint {} must return 200", endpoint);
                let body = body_json(r);
                prop_assert!(
                    !response_contains_forbidden_field(&body, FORBIDDEN),
                    "forbidden reputation field found in {} response",
                    endpoint
                );
            }
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 6: Artifact passthrough integrity ───────────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 6: artifact passthrough integrity
        /// proofd must not modify, interpret, aggregate, vote, or rank artifact content.
        /// The response body must deserialize to the same JSON value as the written artifact.
        /// Validates: Requirements 1.3, 1.4, 3.1, 4.1
        #[test]
        fn prop6_artifact_passthrough_integrity(
            artifact in safe_artifact_strategy(),
        ) {
            let artifact_reparsed: Value =
                serde_json::from_str(&serde_json::to_string_pretty(&artifact).unwrap()).unwrap();

            let dir = temp_dir();
            write_json(&dir, "parity_convergence_report.json", &artifact);

            let r = route_request("GET", "/diagnostics/convergence", &dir);
            prop_assert_eq!(r.status_code, 200);

            let response_value: Value = serde_json::from_slice(&r.body)
                .expect("response must be valid JSON");

            prop_assert_eq!(
                &response_value, &artifact_reparsed,
                "proofd must not modify artifact content: response differs from written artifact"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── Property 7: Diagnostics read-only surface ─────────────────────────────
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: phase13-kill-switch-gates
        /// Property 7: diagnostics read-only surface
        /// HTTP methods other than GET must be rejected on all diagnostics paths.
        /// proofd diagnostics cannot mutate state.
        /// Validates: Requirements 1.1
        #[test]
        fn prop7_diagnostics_read_only_surface(
            suffix in prop::string::string_regex("[a-z0-9-]{1,20}").unwrap(),
            method in prop::sample::select(vec!["POST", "PUT", "PATCH", "DELETE"]),
        ) {
            let dir = temp_dir();
            let path = format!("/diagnostics/{suffix}");
            let r = route_request(method, &path, &dir);
            prop_assert_eq!(
                r.status_code, 405,
                "{} to {} must return 405 (diagnostics surface is read-only)",
                method, path
            );
            let body = body_json(r);
            prop_assert_eq!(
                body.get("error").and_then(|v| v.as_str()),
                Some("method_not_allowed")
            );
            let _ = fs::remove_dir_all(&dir);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase-13 Service-Backed Verification Expansion — Property-Based Tests
// Tasks 7.1–7.8
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod proptest_phase13 {
    use super::{
        route_request, route_request_with_body, DiagnosticsResponse, PHASE13_FORBIDDEN_FIELDS,
    };
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        super::unique_test_temp_dir("proofd-p13-pbt")
    }

    fn body_json(r: &DiagnosticsResponse) -> Value {
        serde_json::from_slice(&r.body).expect("valid json body")
    }

    fn write_artifact(dir: &PathBuf, name: &str, body: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, body).expect("write artifact");
    }

    fn collect_files(dir: &PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();
        collect_recursive(dir, &mut files);
        files.sort();
        files
    }

    fn collect_recursive(dir: &PathBuf, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    out.push(path);
                } else if path.is_dir() {
                    collect_recursive(&path, out);
                }
            }
        }
    }

    fn minimal_ledger_json(verifier_id: &str, authority_chain_id: &str) -> String {
        format!(
            r#"{{"entries":[{{
                "ledger_version":1,"entry_id":"e1","run_id":"run-pbt",
                "timestamp_unix_ns":1,"subject_bundle_id":"b1",
                "verification_context_id":"ctx1","verification_node_id":"n1",
                "verifier_id":"{verifier_id}","authority_chain_id":"{authority_chain_id}",
                "lineage_id":"lin1","verdict":"PASS",
                "receipt_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }}]}}"#
        )
    }

    fn safe_id_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z][a-z0-9-]{0,15}").unwrap()
    }

    fn safe_run_id(prefix: &str) -> String {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        format!("run-p13-{prefix}-{unique:x}")
    }

    // ── P1: Run_Id Fingerprint Conflict Protection ────────────────────────────
    // Validates: Requirement 1.9, 1.12
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **P1 — Run_Id Fingerprint Conflict Protection**
        /// Validates: Requirement 1.9
        /// ∀ run_id, req1, req2: fingerprint(req1) ≠ fingerprint(req2) ∧ same run_id → HTTP 409
        #[test]
        fn p1_run_id_fingerprint_conflict_protection(
            verifier_a in safe_id_strategy(),
            verifier_b in safe_id_strategy(),
        ) {
            // Only test when verifiers differ (different fingerprints)
            prop_assume!(verifier_a != verifier_b);

            use proof_verifier::testing::fixtures::create_fixture_bundle;

            let dir = temp_dir();
            let fixture = create_fixture_bundle();
            let policy_path = fixture.root.join("proofd-policy.json");
            let registry_path = fixture.root.join("proofd-registry.json");
            fs::write(&policy_path, serde_json::to_vec_pretty(&fixture.policy).unwrap()).unwrap();
            fs::write(&registry_path, serde_json::to_vec_pretty(&fixture.registry).unwrap()).unwrap();

            let run_id = safe_run_id("p1");

            let req1 = json!({
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
                "diversity_binding": {
                    "verifier_id": verifier_a,
                    "authority_chain_id": "sha256:chain-a",
                    "lineage_id": "lineage-a",
                }
            });
            let req2 = json!({
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
                "diversity_binding": {
                    "verifier_id": verifier_b,
                    "authority_chain_id": "sha256:chain-a",
                    "lineage_id": "lineage-a",
                }
            });

            let r1 = route_request_with_body(
                "POST", "/verify/bundle",
                Some(serde_json::to_vec(&req1).unwrap().as_slice()),
                &dir,
            );
            prop_assert_eq!(r1.status_code, 200, "first request must succeed");

            let r2 = route_request_with_body(
                "POST", "/verify/bundle",
                Some(serde_json::to_vec(&req2).unwrap().as_slice()),
                &dir,
            );
            prop_assert_eq!(r2.status_code, 409, "second request with different fingerprint must return 409");
            let body2 = body_json(&r2);
            prop_assert_eq!(
                body2.get("error").and_then(|v| v.as_str()),
                Some("run_id_fingerprint_conflict")
            );

            let _ = fs::remove_dir_all(&fixture.root);
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── P4: Artifact Discovery Read-Only Invariant ────────────────────────────
    // Validates: Requirement 7.1, 3.9
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P4 — Artifact Discovery Read-Only Invariant**
        /// Validates: Requirement 7.1, 3.9
        /// GET /diagnostics/runs/{run_id}/artifacts must not write any files
        #[test]
        fn p4_artifact_discovery_read_only(
            call_count in 1usize..=3usize,
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p4");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");
            write_artifact(&run_dir, "proofd_run_manifest.json", r#"{"run_id":"test"}"#);
            write_artifact(&run_dir, "report.json", r#"{"status":"PASS"}"#);

            let files_before = collect_files(&run_dir);

            for _ in 0..call_count {
                let path = format!("/diagnostics/runs/{run_id}/artifacts");
                let _ = route_request("GET", &path, &dir);
            }

            let files_after = collect_files(&run_dir);
            prop_assert_eq!(
                files_before, files_after,
                "GET artifacts endpoint must not modify any files on disk"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── P5: Federation Forbidden Field Invariant ──────────────────────────────
    // Validates: Requirement 5.1, 8.6
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P5 — Federation Forbidden Field Invariant**
        /// Validates: Requirement 5.1, 8.6
        /// ∀ run_id, ledger: response ∩ Phase13_Forbidden_Fields = ∅
        #[test]
        fn p5_federation_forbidden_field_invariant(
            verifier_id in safe_id_strategy(),
            authority_chain_id in safe_id_strategy(),
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p5");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");
            write_artifact(
                &run_dir,
                "verification_diversity_ledger.json",
                &minimal_ledger_json(&verifier_id, &authority_chain_id),
            );

            let path = format!("/diagnostics/runs/{run_id}/federation");
            let r = route_request("GET", &path, &dir);
            prop_assert_eq!(r.status_code, 200, "federation endpoint must return 200");

            let body = body_json(&r);
            let obj = body.as_object().expect("response is object");
            for forbidden in PHASE13_FORBIDDEN_FIELDS {
                prop_assert!(
                    !obj.contains_key(*forbidden),
                    "Forbidden field '{}' found in federation response",
                    forbidden
                );
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── P6: Federation Sorting Invariant ─────────────────────────────────────
    // Validates: Requirement 5.2, 5.3, 5.4
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P6 — Federation Sorting Invariant**
        /// Validates: Requirement 5.2, 5.3, 5.4
        /// observed_verifiers, authority_chain_distribution, execution_cluster_distribution
        /// must all be in lexicographic order
        #[test]
        fn p6_federation_sorting_invariant(
            entries in prop::collection::vec(
                (safe_id_strategy(), safe_id_strategy(), prop::option::of(safe_id_strategy())),
                1..=6usize,
            ),
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p6");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            // Build ledger JSON from entries
            let entry_jsons: Vec<String> = entries.iter().enumerate().map(|(i, (vid, chain, cluster))| {
                let cluster_field = match cluster {
                    Some(c) => format!(r#","execution_cluster_id":"{c}""#),
                    None => String::new(),
                };
                format!(
                    r#"{{"ledger_version":1,"entry_id":"e{i}","run_id":"{run_id}",
                    "timestamp_unix_ns":{i},"subject_bundle_id":"b{i}",
                    "verification_context_id":"ctx{i}","verification_node_id":"n{i}",
                    "verifier_id":"{vid}","authority_chain_id":"{chain}",
                    "lineage_id":"lin{i}","verdict":"PASS",
                    "receipt_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"{cluster_field}}}"#
                )
            }).collect();
            let ledger = format!(r#"{{"entries":[{}]}}"#, entry_jsons.join(","));
            write_artifact(&run_dir, "verification_diversity_ledger.json", &ledger);

            let path = format!("/diagnostics/runs/{run_id}/federation");
            let r = route_request("GET", &path, &dir);
            prop_assert_eq!(r.status_code, 200);

            let body = body_json(&r);

            // observed_verifiers sorted by verifier_id
            let verifiers: Vec<String> = body
                .get("observed_verifiers")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|e| e.get("verifier_id").and_then(|v| v.as_str()).map(String::from))
                .collect();
            let mut sorted_v = verifiers.clone();
            sorted_v.sort();
            prop_assert_eq!(&verifiers, &sorted_v, "observed_verifiers must be sorted by verifier_id");

            // authority_chain_distribution sorted by authority_chain_id
            let chains: Vec<String> = body
                .get("authority_chain_distribution")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|e| e.get("authority_chain_id").and_then(|v| v.as_str()).map(String::from))
                .collect();
            let mut sorted_c = chains.clone();
            sorted_c.sort();
            prop_assert_eq!(&chains, &sorted_c, "authority_chain_distribution must be sorted by authority_chain_id");

            // execution_cluster_distribution sorted by cluster_id
            let clusters: Vec<String> = body
                .get("execution_cluster_distribution")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|e| e.get("cluster_id").and_then(|v| v.as_str()).map(String::from))
                .collect();
            let mut sorted_cl = clusters.clone();
            sorted_cl.sort();
            prop_assert_eq!(&clusters, &sorted_cl, "execution_cluster_distribution must be sorted by cluster_id");

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── P7: Artifact Fetch Passthrough Invariant ──────────────────────────────
    // Validates: Requirement 3.5, 3.8, 7.6
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P7 — Artifact Fetch Passthrough Invariant**
        /// Validates: Requirement 3.5, 3.8, 7.6
        /// ∀ artifact_path ∈ Allowed_Artifact_Set: response_body = disk_bytes
        #[test]
        fn p7_artifact_fetch_passthrough(
            content in prop::string::string_regex(r#"\{"[a-z]{1,8}":"[a-z0-9]{1,16}"\}"#).unwrap(),
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p7");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            // Use proofd_run_manifest.json — always in Allowed_Artifact_Set
            write_artifact(&run_dir, "proofd_run_manifest.json", &content);

            let path = format!("/diagnostics/runs/{run_id}/artifacts/proofd_run_manifest.json");
            let r = route_request("GET", &path, &dir);
            prop_assert_eq!(r.status_code, 200, "artifact fetch must return 200");

            let disk_bytes = fs::read(run_dir.join("proofd_run_manifest.json"))
                .expect("read artifact from disk");
            prop_assert_eq!(
                r.body, disk_bytes,
                "response body must equal disk bytes verbatim"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── P8: Method Not Allowed Invariant ─────────────────────────────────────
    // Validates: Requirement 7.4, 3.4, 4.5
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P8 — Method Not Allowed Invariant**
        /// Validates: Requirement 7.4, 3.4, 4.5
        /// ∀ diagnostics_path: POST → HTTP 405
        #[test]
        fn p8_method_not_allowed_invariant(
            run_id in safe_id_strategy(),
            sub in prop::sample::select(vec!["artifacts", "federation", "context", "registry", "boundary"]),
        ) {
            let dir = temp_dir();
            let path = format!("/diagnostics/runs/{run_id}/{sub}");
            let r = route_request_with_body("POST", &path, Some(b"{}"), &dir);
            prop_assert_eq!(
                r.status_code, 405,
                "POST to {} must return 405",
                path
            );
            let body = body_json(&r);
            prop_assert_eq!(
                body.get("error").and_then(|v| v.as_str()),
                Some("method_not_allowed")
            );
            let _ = fs::remove_dir_all(&dir);
        }
    }

    // ── P9: Artifact Path Normalization Invariant ─────────────────────────────
    // Validates: Requirement 3.11, 3.7
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P9 — Artifact Path Normalization Invariant**
        /// Validates: Requirement 3.11, 3.7
        /// Paths with ".." or "." segments → HTTP 403
        /// Paths outside Allowed_Artifact_Set → HTTP 403
        #[test]
        fn p9_artifact_path_normalization_traversal(
            prefix in prop::string::string_regex("[a-z]{1,8}").unwrap(),
            suffix in prop::string::string_regex("[a-z]{1,8}\\.json").unwrap(),
        ) {
            let dir = temp_dir();
            let run_id = safe_run_id("p9t");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");

            // Path with ".." traversal segment
            let traversal_path = format!(
                "/diagnostics/runs/{run_id}/artifacts/{prefix}/../{suffix}"
            );
            let r = route_request("GET", &traversal_path, &dir);
            prop_assert_eq!(
                r.status_code, 403,
                "path with '..' segment must return 403, got {} for {}",
                r.status_code, traversal_path
            );
            let body = body_json(&r);
            prop_assert_eq!(
                body.get("error").and_then(|v| v.as_str()),
                Some("artifact_path_not_allowed")
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **P9b — Artifact Path Outside Allowed Set → 403**
        /// Validates: Requirement 3.7
        #[test]
        fn p9b_artifact_path_outside_allowed_set(
            name in prop::string::string_regex("[a-z]{4,12}_[a-z]{4,8}\\.json").unwrap(),
        ) {
            // Only test names that are NOT in the allowed set
            let allowed: std::collections::HashSet<&str> = super::RUN_LEVEL_ARTIFACTS
                .iter()
                .chain(super::NESTED_RUN_LEVEL_ARTIFACTS.iter())
                .copied()
                .collect();
            prop_assume!(!allowed.contains(name.as_str()));

            let dir = temp_dir();
            let run_id = safe_run_id("p9b");
            let run_dir = dir.join(&run_id);
            fs::create_dir_all(&run_dir).expect("create run dir");
            // Write the file so it exists on disk — must still be rejected
            write_artifact(&run_dir, &name, r#"{"data":"secret"}"#);

            let path = format!("/diagnostics/runs/{run_id}/artifacts/{name}");
            let r = route_request("GET", &path, &dir);
            prop_assert_eq!(
                r.status_code, 403,
                "path outside Allowed_Artifact_Set must return 403 even if file exists on disk"
            );
            let body = body_json(&r);
            prop_assert_eq!(
                body.get("error").and_then(|v| v.as_str()),
                Some("artifact_path_not_allowed")
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }
}
