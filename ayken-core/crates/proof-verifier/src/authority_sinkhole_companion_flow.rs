use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AuthoritySinkholeCompanionFlowEvent {
    pub event_schema_version: u32,
    pub event_id: String,
    pub flow_surface: String,
    pub run_id: String,
    #[serde(deserialize_with = "deserialize_u64_like")]
    pub timestamp_unix_ns: u64,
    pub subject_bundle_id: String,
    pub verification_context_id: String,
    pub authority_chain_id: String,
    pub terminal: bool,
    pub reused: bool,
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
    pub replay_contract_id: Option<String>,
    #[serde(default)]
    pub trust_reuse_source: Option<String>,
    #[serde(default)]
    pub reuse_group_id: Option<String>,
    #[serde(default)]
    pub surface_local_path_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AuthoritySinkholeCompanionFlowReport {
    pub report_version: u32,
    pub flow_surface: String,
    pub status: String,
    pub run_id: String,
    pub window_model: String,
    pub event_count: usize,
    pub terminal_event_count: usize,
    pub reused_event_count: usize,
    pub events: Vec<AuthoritySinkholeCompanionFlowEvent>,
}

pub fn load_companion_flow_report(
    path: &Path,
) -> Result<AuthoritySinkholeCompanionFlowReport, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read companion flow report at {}: {error}",
            path.display()
        )
    })?;
    let mut report: AuthoritySinkholeCompanionFlowReport =
        serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "failed to parse companion flow report at {}: {error}",
                path.display()
            )
        })?;
    sort_companion_flow_events(&mut report.events);
    validate_companion_flow_report(&report)?;
    Ok(report)
}

pub fn write_companion_flow_report(
    path: &Path,
    report: &AuthoritySinkholeCompanionFlowReport,
) -> Result<(), String> {
    validate_companion_flow_report(report)?;
    let mut sorted_report = report.clone();
    sort_companion_flow_events(&mut sorted_report.events);
    let bytes = canonicalize_json(&sorted_report).map_err(|error| {
        format!(
            "failed to canonicalize companion flow report for {}: {error}",
            path.display()
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create companion flow parent {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to write companion flow report {}: {error}",
            path.display()
        )
    })
}

pub fn sort_companion_flow_events(events: &mut [AuthoritySinkholeCompanionFlowEvent]) {
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

pub fn compute_companion_flow_event_id(
    event: &AuthoritySinkholeCompanionFlowEvent,
) -> Result<String, String> {
    let mut value = serde_json::to_value(event)
        .map_err(|error| format!("failed to serialize companion event for hashing: {error}"))?;
    if let Value::Object(map) = &mut value {
        map.remove("event_id");
    }
    let bytes = canonicalize_json_value(&value)
        .map_err(|error| format!("failed to canonicalize companion event for hashing: {error}"))?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

pub fn validate_companion_flow_report(
    report: &AuthoritySinkholeCompanionFlowReport,
) -> Result<(), String> {
    if report.report_version != 1 {
        return Err(format!(
            "unsupported report_version {} for flow surface {}",
            report.report_version, report.flow_surface
        ));
    }
    validate_surface(&report.flow_surface)?;
    if report.run_id.trim().is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    if report.status.trim().is_empty() {
        return Err("status must not be empty".to_string());
    }
    if report.window_model.trim().is_empty() {
        return Err("window_model must not be empty".to_string());
    }
    let mut sorted_events = report.events.clone();
    sort_companion_flow_events(&mut sorted_events);
    if sorted_events != report.events {
        return Err(format!(
            "events are not in canonical order for flow surface {}",
            report.flow_surface
        ));
    }
    let terminal_event_count = report.events.iter().filter(|event| event.terminal).count();
    let reused_event_count = report.events.iter().filter(|event| event.reused).count();
    if report.event_count != report.events.len() {
        return Err(format!(
            "event_count does not match event vector length for flow surface {}",
            report.flow_surface
        ));
    }
    if report.terminal_event_count != terminal_event_count {
        return Err(format!(
            "terminal_event_count does not match terminal events for flow surface {}",
            report.flow_surface
        ));
    }
    if report.reused_event_count != reused_event_count {
        return Err(format!(
            "reused_event_count does not match reused events for flow surface {}",
            report.flow_surface
        ));
    }
    for event in &report.events {
        validate_companion_flow_event(event)?;
        if event.flow_surface != report.flow_surface {
            return Err(format!(
                "event {} flow_surface {} does not match report flow surface {}",
                event.event_id, event.flow_surface, report.flow_surface
            ));
        }
    }
    Ok(())
}

pub fn validate_companion_flow_event(
    event: &AuthoritySinkholeCompanionFlowEvent,
) -> Result<(), String> {
    if event.event_schema_version != 1 {
        return Err(format!(
            "unsupported event_schema_version {} for event {}",
            event.event_schema_version, event.event_id
        ));
    }
    validate_surface(&event.flow_surface)?;
    for (label, value) in [
        ("event_id", event.event_id.as_str()),
        ("run_id", event.run_id.as_str()),
        ("subject_bundle_id", event.subject_bundle_id.as_str()),
        (
            "verification_context_id",
            event.verification_context_id.as_str(),
        ),
        ("authority_chain_id", event.authority_chain_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "{label} must not be empty for event {}",
                event.event_id
            ));
        }
    }
    if event.timestamp_unix_ns == 0 {
        return Err(format!(
            "timestamp_unix_ns must be non-zero for event {}",
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
        ("replay_contract_id", event.replay_contract_id.as_deref()),
        ("trust_reuse_source", event.trust_reuse_source.as_deref()),
        ("reuse_group_id", event.reuse_group_id.as_deref()),
        (
            "surface_local_path_id",
            event.surface_local_path_id.as_deref(),
        ),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(format!(
                "{label} must not be empty for event {}",
                event.event_id
            ));
        }
    }
    let expected_event_id = compute_companion_flow_event_id(event)?;
    if event.event_id != expected_event_id {
        return Err(format!(
            "event_id does not match canonical content-addressed identity for event {}",
            event.event_id
        ));
    }
    Ok(())
}

pub fn validate_surface(flow_surface: &str) -> Result<(), String> {
    if matches!(flow_surface, "replay_boundary" | "trust_reuse") {
        Ok(())
    } else {
        Err(format!("unsupported flow_surface:{flow_surface}"))
    }
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
