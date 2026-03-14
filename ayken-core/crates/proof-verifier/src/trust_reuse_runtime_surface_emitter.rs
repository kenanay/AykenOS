use crate::canonical::jcs::canonicalize_json;
use crate::trust_reuse_runtime_surface::{
    compute_trust_reuse_runtime_event_id, validate_trust_reuse_runtime_surface,
    write_trust_reuse_runtime_surface, TrustReuseOutcome, TrustReuseRuntimeEvent,
    TrustReuseRuntimeSurfaceReport,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TrustReuseRuntimeSurfaceEmitterConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug)]
pub struct TrustReuseRuntimeSurfaceEmitterOutcome {
    pub event_count: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct TrustReuseRuntimeSurfaceInputDocument {
    pub surface_version: u32,
    pub run_id: String,
    pub source_kind: String,
    pub events: Vec<TrustReuseRuntimeSurfaceInputEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct TrustReuseRuntimeSurfaceInputEvent {
    pub run_id: Option<String>,
    #[serde(deserialize_with = "deserialize_u64_like")]
    pub timestamp_unix_ns: u64,
    pub subject_bundle_id: String,
    pub verification_context_id: String,
    pub authority_chain_id: String,
    pub trust_reuse_outcome: TrustReuseOutcome,
    pub terminal: bool,
    pub reused: bool,
    pub receipt_ref: String,
    pub verification_context_ref: String,
    pub verifier_attestation_ref: String,
    pub verifier_registry_snapshot_hash: String,
    #[serde(default)]
    pub verification_node_id: Option<String>,
    #[serde(default)]
    pub verifier_id: Option<String>,
    #[serde(default)]
    pub lineage_id: Option<String>,
    #[serde(default)]
    pub execution_cluster_id: Option<String>,
    #[serde(default)]
    pub source_run_id: Option<String>,
    #[serde(default)]
    pub reuse_group_id: Option<String>,
    #[serde(default)]
    pub surface_local_path_id: Option<String>,
    #[serde(default)]
    pub trust_reuse_source: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmitReport {
    status: &'static str,
    input_path: String,
    output_path: String,
    event_count: usize,
    accepted_event_count: usize,
    historical_only_event_count: usize,
    rejected_event_count: usize,
}

pub fn run_trust_reuse_runtime_surface_emitter(
    config: &TrustReuseRuntimeSurfaceEmitterConfig,
) -> Result<TrustReuseRuntimeSurfaceEmitterOutcome, String> {
    let input = load_input_document(&config.input_path)?;
    let mut events = Vec::with_capacity(input.events.len());
    for input_event in &input.events {
        let mut event = TrustReuseRuntimeEvent {
            event_schema_version: 1,
            event_id: String::new(),
            run_id: input_event
                .run_id
                .clone()
                .unwrap_or_else(|| input.run_id.clone()),
            timestamp_unix_ns: input_event.timestamp_unix_ns,
            subject_bundle_id: input_event.subject_bundle_id.clone(),
            verification_context_id: input_event.verification_context_id.clone(),
            authority_chain_id: input_event.authority_chain_id.clone(),
            trust_reuse_outcome: input_event.trust_reuse_outcome.clone(),
            terminal: input_event.terminal,
            reused: input_event.reused,
            receipt_ref: input_event.receipt_ref.clone(),
            verification_context_ref: input_event.verification_context_ref.clone(),
            verifier_attestation_ref: input_event.verifier_attestation_ref.clone(),
            verifier_registry_snapshot_hash: input_event.verifier_registry_snapshot_hash.clone(),
            verification_node_id: input_event.verification_node_id.clone(),
            verifier_id: input_event.verifier_id.clone(),
            lineage_id: input_event.lineage_id.clone(),
            execution_cluster_id: input_event.execution_cluster_id.clone(),
            source_run_id: input_event.source_run_id.clone(),
            reuse_group_id: input_event.reuse_group_id.clone(),
            surface_local_path_id: input_event.surface_local_path_id.clone(),
            trust_reuse_source: input_event.trust_reuse_source.clone(),
        };
        event.event_id = compute_trust_reuse_runtime_event_id(&event)?;
        events.push(event);
    }

    let accepted_event_count = events
        .iter()
        .filter(|event| event.trust_reuse_outcome == TrustReuseOutcome::Accepted)
        .count();
    let historical_only_event_count = events
        .iter()
        .filter(|event| event.trust_reuse_outcome == TrustReuseOutcome::HistoricalOnly)
        .count();
    let rejected_event_count = events
        .iter()
        .filter(|event| event.trust_reuse_outcome == TrustReuseOutcome::Rejected)
        .count();
    let report = TrustReuseRuntimeSurfaceReport {
        surface_version: input.surface_version,
        flow_surface: "trust_reuse_runtime".to_string(),
        status: "PASS".to_string(),
        run_id: input.run_id,
        source_kind: input.source_kind,
        event_count: events.len(),
        accepted_event_count,
        historical_only_event_count,
        rejected_event_count,
        events,
    };
    validate_trust_reuse_runtime_surface(&report)?;
    write_trust_reuse_runtime_surface(&config.output_path, &report)?;
    write_emit_artifacts(
        config,
        &EmitReport {
            status: "PASS",
            input_path: config.input_path.display().to_string(),
            output_path: config.output_path.display().to_string(),
            event_count: report.event_count,
            accepted_event_count: report.accepted_event_count,
            historical_only_event_count: report.historical_only_event_count,
            rejected_event_count: report.rejected_event_count,
        },
    )?;
    Ok(TrustReuseRuntimeSurfaceEmitterOutcome {
        event_count: report.event_count,
    })
}

fn load_input_document(path: &PathBuf) -> Result<TrustReuseRuntimeSurfaceInputDocument, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read trust reuse runtime surface input at {}: {error}",
            path.display()
        )
    })?;
    let document: TrustReuseRuntimeSurfaceInputDocument =
        serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "failed to parse trust reuse runtime surface input at {}: {error}",
                path.display()
            )
        })?;
    if document.surface_version != 1 {
        return Err(format!(
            "unsupported surface_version {} in trust reuse runtime surface input",
            document.surface_version
        ));
    }
    if document.run_id.trim().is_empty() {
        return Err("run_id must not be empty in trust reuse runtime surface input".to_string());
    }
    if document.source_kind != "local_runtime_evidence" {
        return Err(format!(
            "unsupported source_kind {} in trust reuse runtime surface input",
            document.source_kind
        ));
    }
    Ok(document)
}

fn write_emit_artifacts(
    config: &TrustReuseRuntimeSurfaceEmitterConfig,
    report: &EmitReport,
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create trust reuse runtime emitter output dir {}: {error}",
            config.output_dir.display()
        )
    })?;
    let report_path = config
        .output_dir
        .join("trust_reuse_runtime_surface_emit_report.json");
    let report_bytes = canonicalize_json(report).map_err(|error| {
        format!("failed to canonicalize trust reuse runtime emit report: {error}")
    })?;
    fs::write(&report_path, report_bytes).map_err(|error| {
        format!(
            "failed to write trust reuse runtime emit report {}: {error}",
            report_path.display()
        )
    })?;
    let violations_path = config.output_dir.join("violations.txt");
    fs::write(&violations_path, []).map_err(|error| {
        format!(
            "failed to write trust reuse runtime emitter violations {}: {error}",
            violations_path.display()
        )
    })?;
    Ok(())
}

fn deserialize_u64_like<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64Like {
        Int(u64),
        String(String),
    }

    match U64Like::deserialize(deserializer)? {
        U64Like::Int(value) => Ok(value),
        U64Like::String(value) => value.parse::<u64>().map_err(serde::de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use super::{run_trust_reuse_runtime_surface_emitter, TrustReuseRuntimeSurfaceEmitterConfig};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("trust-reuse-runtime-emitter-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn emitter_materializes_native_trust_reuse_surface() {
        let dir = temp_dir();
        let input_path = dir.join("trust_reuse_runtime_surface_input.json");
        let output_path = dir.join("trust_reuse_runtime_surface.json");
        let output_dir = dir.join("reports");
        fs::write(
            &input_path,
            serde_json::to_vec_pretty(&json!({
                "surface_version": 1,
                "run_id": "run-a",
                "source_kind": "local_runtime_evidence",
                "events": [{
                    "timestamp_unix_ns": 1710000000000000000u64,
                    "subject_bundle_id": "bundle-a",
                    "verification_context_id": "sha256:context-a",
                    "authority_chain_id": "sha256:authority-a",
                    "trust_reuse_outcome": "accepted",
                    "terminal": true,
                    "reused": true,
                    "receipt_ref": "receipts/verification_receipt.json",
                    "verification_context_ref": "cas:sha256:context-object-a",
                    "verifier_attestation_ref": "cas:sha256:attestation-a",
                    "verifier_registry_snapshot_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                }]
            }))
            .expect("serialize input"),
        )
        .expect("write input");

        let outcome =
            run_trust_reuse_runtime_surface_emitter(&TrustReuseRuntimeSurfaceEmitterConfig {
                input_path,
                output_path: output_path.clone(),
                output_dir: output_dir.clone(),
            })
            .expect("emitter should pass");
        assert_eq!(outcome.event_count, 1);
        assert!(output_path.is_file());
        assert!(output_dir
            .join("trust_reuse_runtime_surface_emit_report.json")
            .is_file());
        let _ = fs::remove_dir_all(dir);
    }
}
