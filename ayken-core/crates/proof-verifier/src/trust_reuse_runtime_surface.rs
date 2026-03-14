use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustReuseOutcome {
    Accepted,
    HistoricalOnly,
    Rejected,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct TrustReuseRuntimeEvent {
    pub event_schema_version: u32,
    pub event_id: String,
    pub run_id: String,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct TrustReuseRuntimeSurfaceReport {
    pub surface_version: u32,
    pub flow_surface: String,
    pub status: String,
    pub run_id: String,
    pub source_kind: String,
    pub event_count: usize,
    pub accepted_event_count: usize,
    pub historical_only_event_count: usize,
    pub rejected_event_count: usize,
    pub events: Vec<TrustReuseRuntimeEvent>,
}

pub fn load_trust_reuse_runtime_surface(
    path: &Path,
) -> Result<TrustReuseRuntimeSurfaceReport, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read trust reuse runtime surface at {}: {error}",
            path.display()
        )
    })?;
    let mut report: TrustReuseRuntimeSurfaceReport =
        serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "failed to parse trust reuse runtime surface at {}: {error}",
                path.display()
            )
        })?;
    sort_trust_reuse_runtime_events(&mut report.events);
    validate_trust_reuse_runtime_surface(&report)?;
    Ok(report)
}

pub fn write_trust_reuse_runtime_surface(
    path: &Path,
    report: &TrustReuseRuntimeSurfaceReport,
) -> Result<(), String> {
    validate_trust_reuse_runtime_surface(report)?;
    let mut sorted_report = report.clone();
    sort_trust_reuse_runtime_events(&mut sorted_report.events);
    let bytes = canonicalize_json(&sorted_report).map_err(|error| {
        format!(
            "failed to canonicalize trust reuse runtime surface for {}: {error}",
            path.display()
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create trust reuse runtime surface parent {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to write trust reuse runtime surface {}: {error}",
            path.display()
        )
    })
}

pub fn sort_trust_reuse_runtime_events(events: &mut [TrustReuseRuntimeEvent]) {
    events.sort_by(|left, right| {
        left.timestamp_unix_ns
            .cmp(&right.timestamp_unix_ns)
            .then_with(|| left.subject_bundle_id.cmp(&right.subject_bundle_id))
            .then_with(|| {
                left.verification_context_id
                    .cmp(&right.verification_context_id)
            })
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
}

pub fn compute_trust_reuse_runtime_event_id(
    event: &TrustReuseRuntimeEvent,
) -> Result<String, String> {
    let mut value = serde_json::to_value(event).map_err(|error| {
        format!("failed to serialize trust reuse runtime event for hashing: {error}")
    })?;
    if let Value::Object(map) = &mut value {
        map.remove("event_id");
    }
    let bytes = canonicalize_json_value(&value).map_err(|error| {
        format!("failed to canonicalize trust reuse runtime event for hashing: {error}")
    })?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

pub fn validate_trust_reuse_runtime_surface(
    report: &TrustReuseRuntimeSurfaceReport,
) -> Result<(), String> {
    if report.surface_version != 1 {
        return Err(format!(
            "unsupported surface_version {} for trust reuse runtime surface",
            report.surface_version
        ));
    }
    if report.flow_surface != "trust_reuse_runtime" {
        return Err(format!(
            "unsupported flow_surface {} for trust reuse runtime surface",
            report.flow_surface
        ));
    }
    if report.status.trim().is_empty() {
        return Err("status must not be empty".to_string());
    }
    if report.run_id.trim().is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    if report.source_kind != "local_runtime_evidence" {
        return Err(format!(
            "unsupported source_kind {} for trust reuse runtime surface",
            report.source_kind
        ));
    }
    let mut sorted_events = report.events.clone();
    sort_trust_reuse_runtime_events(&mut sorted_events);
    if sorted_events != report.events {
        return Err(
            "events are not in canonical order for trust reuse runtime surface".to_string(),
        );
    }
    let accepted_event_count = report
        .events
        .iter()
        .filter(|event| event.trust_reuse_outcome == TrustReuseOutcome::Accepted)
        .count();
    let historical_only_event_count = report
        .events
        .iter()
        .filter(|event| event.trust_reuse_outcome == TrustReuseOutcome::HistoricalOnly)
        .count();
    let rejected_event_count = report
        .events
        .iter()
        .filter(|event| event.trust_reuse_outcome == TrustReuseOutcome::Rejected)
        .count();
    if report.event_count != report.events.len() {
        return Err("event_count does not match event vector length".to_string());
    }
    if report.accepted_event_count != accepted_event_count {
        return Err("accepted_event_count does not match accepted events".to_string());
    }
    if report.historical_only_event_count != historical_only_event_count {
        return Err(
            "historical_only_event_count does not match historical-only events".to_string(),
        );
    }
    if report.rejected_event_count != rejected_event_count {
        return Err("rejected_event_count does not match rejected events".to_string());
    }
    for event in &report.events {
        validate_trust_reuse_runtime_event(event)?;
    }
    Ok(())
}

pub fn validate_trust_reuse_runtime_event(event: &TrustReuseRuntimeEvent) -> Result<(), String> {
    if event.event_schema_version != 1 {
        return Err(format!(
            "unsupported event_schema_version {} for trust reuse runtime event {}",
            event.event_schema_version, event.event_id
        ));
    }
    if event.timestamp_unix_ns == 0 {
        return Err(format!(
            "timestamp_unix_ns must be non-zero for trust reuse runtime event {}",
            event.event_id
        ));
    }
    for (label, value) in [
        ("event_id", event.event_id.as_str()),
        ("run_id", event.run_id.as_str()),
        ("subject_bundle_id", event.subject_bundle_id.as_str()),
        (
            "verification_context_id",
            event.verification_context_id.as_str(),
        ),
        ("authority_chain_id", event.authority_chain_id.as_str()),
        ("receipt_ref", event.receipt_ref.as_str()),
        (
            "verification_context_ref",
            event.verification_context_ref.as_str(),
        ),
        (
            "verifier_attestation_ref",
            event.verifier_attestation_ref.as_str(),
        ),
        (
            "verifier_registry_snapshot_hash",
            event.verifier_registry_snapshot_hash.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "{label} must not be empty for trust reuse runtime event {}",
                event.event_id
            ));
        }
    }
    if !is_lower_hex_digest(&event.verifier_registry_snapshot_hash) {
        return Err(format!(
            "verifier_registry_snapshot_hash must be a 64-character lowercase SHA-256 hex digest for trust reuse runtime event {}",
            event.event_id
        ));
    }
    for (label, value) in [
        (
            "verification_node_id",
            event.verification_node_id.as_deref(),
        ),
        ("verifier_id", event.verifier_id.as_deref()),
        ("lineage_id", event.lineage_id.as_deref()),
        (
            "execution_cluster_id",
            event.execution_cluster_id.as_deref(),
        ),
        ("source_run_id", event.source_run_id.as_deref()),
        ("reuse_group_id", event.reuse_group_id.as_deref()),
        (
            "surface_local_path_id",
            event.surface_local_path_id.as_deref(),
        ),
        ("trust_reuse_source", event.trust_reuse_source.as_deref()),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(format!(
                "{label} must not be empty for trust reuse runtime event {}",
                event.event_id
            ));
        }
    }
    let expected_event_id = compute_trust_reuse_runtime_event_id(event)?;
    if event.event_id != expected_event_id {
        return Err(format!(
            "event_id does not match canonical content-addressed identity for trust reuse runtime event {}",
            event.event_id
        ));
    }
    Ok(())
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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
    use super::{
        compute_trust_reuse_runtime_event_id, validate_trust_reuse_runtime_surface,
        write_trust_reuse_runtime_surface, TrustReuseOutcome, TrustReuseRuntimeEvent,
        TrustReuseRuntimeSurfaceReport,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("trust-reuse-runtime-surface-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn sample_event() -> TrustReuseRuntimeEvent {
        let mut event = TrustReuseRuntimeEvent {
            event_schema_version: 1,
            event_id: String::new(),
            run_id: "run-a".to_string(),
            timestamp_unix_ns: 1_710_000_000_000_000_000,
            subject_bundle_id: "bundle-a".to_string(),
            verification_context_id: "sha256:context-a".to_string(),
            authority_chain_id: "sha256:authority-a".to_string(),
            trust_reuse_outcome: TrustReuseOutcome::Accepted,
            terminal: true,
            reused: true,
            receipt_ref: "receipts/verification_receipt.json".to_string(),
            verification_context_ref: "cas:sha256:context-object-a".to_string(),
            verifier_attestation_ref: "cas:sha256:attestation-a".to_string(),
            verifier_registry_snapshot_hash: "a".repeat(64),
            verification_node_id: Some("node-a".to_string()),
            verifier_id: Some("verifier-a".to_string()),
            lineage_id: Some("lineage-a".to_string()),
            execution_cluster_id: Some("cluster-a".to_string()),
            source_run_id: Some("source-run-a".to_string()),
            reuse_group_id: Some("reuse-group-a".to_string()),
            surface_local_path_id: Some("trust-path-a".to_string()),
            trust_reuse_source: Some("runtime-attested-trust-reuse".to_string()),
        };
        event.event_id =
            compute_trust_reuse_runtime_event_id(&event).expect("compute trust reuse event id");
        event
    }

    #[test]
    fn trust_reuse_runtime_surface_validates_and_writes() {
        let dir = temp_dir();
        let path = dir.join("trust_reuse_runtime_surface.json");
        let report = TrustReuseRuntimeSurfaceReport {
            surface_version: 1,
            flow_surface: "trust_reuse_runtime".to_string(),
            status: "PASS".to_string(),
            run_id: "run-a".to_string(),
            source_kind: "local_runtime_evidence".to_string(),
            event_count: 1,
            accepted_event_count: 1,
            historical_only_event_count: 0,
            rejected_event_count: 0,
            events: vec![sample_event()],
        };
        validate_trust_reuse_runtime_surface(&report).expect("surface should validate");
        write_trust_reuse_runtime_surface(&path, &report).expect("surface should write");
        assert!(path.is_file());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn trust_reuse_runtime_surface_rejects_missing_attestation_ref() {
        let mut event = sample_event();
        event.verifier_attestation_ref.clear();
        event.event_id =
            compute_trust_reuse_runtime_event_id(&event).expect("recompute trust reuse event id");
        let report = TrustReuseRuntimeSurfaceReport {
            surface_version: 1,
            flow_surface: "trust_reuse_runtime".to_string(),
            status: "PASS".to_string(),
            run_id: "run-a".to_string(),
            source_kind: "local_runtime_evidence".to_string(),
            event_count: 1,
            accepted_event_count: 1,
            historical_only_event_count: 0,
            rejected_event_count: 0,
            events: vec![event],
        };
        let error = validate_trust_reuse_runtime_surface(&report)
            .expect_err("missing attestation ref must fail");
        assert!(error.contains("verifier_attestation_ref"));
    }
}
