use crate::authority_sinkhole_companion_flow::{
    compute_companion_flow_event_id, sort_companion_flow_events, validate_companion_flow_event,
    validate_surface, write_companion_flow_report, AuthoritySinkholeCompanionFlowEvent,
    AuthoritySinkholeCompanionFlowReport,
};
use crate::diversity_floor::GateVerdict;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AuthoritySinkholeCompanionProducerConfig {
    pub replay_source_path: PathBuf,
    pub trust_source_path: PathBuf,
    pub replay_output_path: PathBuf,
    pub trust_output_path: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug)]
pub struct AuthoritySinkholeCompanionProducerOutcome {
    pub verdict: GateVerdict,
    pub violations: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct CompanionSourceDocument {
    pub source_version: u32,
    pub flow_surface: String,
    #[serde(default = "default_source_status")]
    pub status: String,
    pub run_id: String,
    #[serde(default = "default_window_model")]
    pub window_model: String,
    pub events: Vec<CompanionSourceEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct CompanionSourceEvent {
    #[serde(default)]
    pub run_id: Option<String>,
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

#[derive(Debug, Serialize)]
struct ProducerMetrics {
    discovered_surface_count: usize,
    materialized_surface_count: usize,
    missing_surfaces: Vec<String>,
    surfaces: Vec<SurfaceMetrics>,
}

#[derive(Debug, Serialize)]
struct SurfaceMetrics {
    flow_surface: String,
    source_status: String,
    report_status: String,
    source_path: String,
    output_path: String,
    source_event_count: usize,
    materialized_event_count: usize,
    duplicate_skipped_count: usize,
}

pub fn run_authority_sinkhole_companion_producer(
    config: &AuthoritySinkholeCompanionProducerConfig,
) -> Result<AuthoritySinkholeCompanionProducerOutcome, String> {
    let mut violations = Vec::new();
    let mut surface_metrics = Vec::new();
    let mut discovered_surface_count = 0usize;
    let mut materialized_surface_count = 0usize;
    let mut missing_surfaces = Vec::new();

    for (flow_surface, source_path, output_path) in [
        (
            "replay_boundary",
            &config.replay_source_path,
            &config.replay_output_path,
        ),
        (
            "trust_reuse",
            &config.trust_source_path,
            &config.trust_output_path,
        ),
    ] {
        if !source_path.exists() {
            missing_surfaces.push(flow_surface.to_string());
            continue;
        }

        discovered_surface_count += 1;
        match materialize_surface(flow_surface, source_path, output_path) {
            Ok(metrics) => {
                materialized_surface_count += 1;
                surface_metrics.push(metrics);
            }
            Err(error) => {
                violations.push(format!("{flow_surface}_materialization_failure:{error}"))
            }
        }
    }

    if discovered_surface_count == 0 {
        violations.push("missing_companion_flow_sources".to_string());
        write_loading_failure_outputs(
            config,
            &violations,
            "no_companion_flow_sources_discovered",
            "source_discovery",
            &missing_surfaces,
        )?;
        return Ok(AuthoritySinkholeCompanionProducerOutcome {
            verdict: GateVerdict::Fail,
            violations,
        });
    }

    let metrics = ProducerMetrics {
        discovered_surface_count,
        materialized_surface_count,
        missing_surfaces,
        surfaces: surface_metrics,
    };

    if !violations.is_empty() {
        write_failure_outputs(config, &metrics, &violations)?;
        return Ok(AuthoritySinkholeCompanionProducerOutcome {
            verdict: GateVerdict::Fail,
            violations,
        });
    }

    write_outputs(config, &metrics, &violations)?;
    Ok(AuthoritySinkholeCompanionProducerOutcome {
        verdict: GateVerdict::Pass,
        violations,
    })
}

fn materialize_surface(
    expected_surface: &str,
    source_path: &Path,
    output_path: &Path,
) -> Result<SurfaceMetrics, String> {
    let source = load_source_document(source_path)?;
    validate_source_document(&source, expected_surface)?;
    if source.status == "NO_REUSABLE_EVENTS" {
        let report = AuthoritySinkholeCompanionFlowReport {
            report_version: 1,
            flow_surface: source.flow_surface.clone(),
            status: "NOT_EVALUATED".to_string(),
            run_id: source.run_id.clone(),
            window_model: source.window_model.clone(),
            event_count: 0,
            terminal_event_count: 0,
            reused_event_count: 0,
            events: Vec::new(),
        };
        write_companion_flow_report(output_path, &report)?;
        return Ok(SurfaceMetrics {
            flow_surface: expected_surface.to_string(),
            source_status: source.status,
            report_status: report.status,
            source_path: source_path.display().to_string(),
            output_path: output_path.display().to_string(),
            source_event_count: 0,
            materialized_event_count: 0,
            duplicate_skipped_count: 0,
        });
    }
    let mut events = Vec::new();
    let mut seen_event_ids = BTreeSet::new();
    let mut duplicate_skipped_count = 0usize;

    for source_event in &source.events {
        let mut event = AuthoritySinkholeCompanionFlowEvent {
            event_schema_version: 1,
            event_id: String::new(),
            flow_surface: source.flow_surface.clone(),
            run_id: source_event
                .run_id
                .clone()
                .unwrap_or_else(|| source.run_id.clone()),
            timestamp_unix_ns: source_event.timestamp_unix_ns,
            subject_bundle_id: source_event.subject_bundle_id.clone(),
            verification_context_id: source_event.verification_context_id.clone(),
            authority_chain_id: source_event.authority_chain_id.clone(),
            terminal: source_event.terminal,
            reused: source_event.reused,
            verification_node_id: source_event.verification_node_id.clone(),
            verifier_id: source_event.verifier_id.clone(),
            lineage_id: source_event.lineage_id.clone(),
            execution_cluster_id: source_event.execution_cluster_id.clone(),
            source_run_id: source_event.source_run_id.clone(),
            replay_contract_id: source_event.replay_contract_id.clone(),
            trust_reuse_source: source_event.trust_reuse_source.clone(),
            reuse_group_id: source_event.reuse_group_id.clone(),
            surface_local_path_id: source_event.surface_local_path_id.clone(),
        };
        event.event_id = compute_companion_flow_event_id(&event)?;
        validate_companion_flow_event(&event)?;
        if !seen_event_ids.insert(event.event_id.clone()) {
            duplicate_skipped_count += 1;
            continue;
        }
        events.push(event);
    }

    sort_companion_flow_events(&mut events);
    let report = AuthoritySinkholeCompanionFlowReport {
        report_version: 1,
        flow_surface: source.flow_surface.clone(),
        status: "PASS".to_string(),
        run_id: source.run_id.clone(),
        window_model: source.window_model.clone(),
        event_count: events.len(),
        terminal_event_count: events.iter().filter(|event| event.terminal).count(),
        reused_event_count: events.iter().filter(|event| event.reused).count(),
        events,
    };
    write_companion_flow_report(output_path, &report)?;
    Ok(SurfaceMetrics {
        flow_surface: expected_surface.to_string(),
        source_status: source.status,
        report_status: report.status,
        source_path: source_path.display().to_string(),
        output_path: output_path.display().to_string(),
        source_event_count: source.events.len(),
        materialized_event_count: report.event_count,
        duplicate_skipped_count,
    })
}

fn default_source_status() -> String {
    "PASS".to_string()
}

fn default_window_model() -> String {
    "append_only_event_stream".to_string()
}

fn load_source_document(path: &Path) -> Result<CompanionSourceDocument, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read source document at {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse source document at {}: {error}",
            path.display()
        )
    })
}

fn validate_source_document(
    source: &CompanionSourceDocument,
    expected_surface: &str,
) -> Result<(), String> {
    if source.source_version != 1 {
        return Err(format!(
            "unsupported_source_version:{}",
            source.source_version
        ));
    }
    validate_surface(&source.flow_surface)?;
    if source.flow_surface != expected_surface {
        return Err(format!(
            "source_flow_surface_mismatch:{}:{}",
            source.flow_surface, expected_surface
        ));
    }
    if source.run_id.trim().is_empty() {
        return Err("run_id_must_not_be_empty".to_string());
    }
    if source.status.trim().is_empty() {
        return Err("status_must_not_be_empty".to_string());
    }
    if source.window_model.trim().is_empty() {
        return Err("window_model_must_not_be_empty".to_string());
    }
    if source.status == "NO_REUSABLE_EVENTS" {
        if expected_surface != "trust_reuse" {
            return Err("status_not_supported_for_surface:NO_REUSABLE_EVENTS".to_string());
        }
        if !source.events.is_empty() {
            return Err("no_reusable_events_source_must_be_empty".to_string());
        }
        return Ok(());
    }
    if source.events.is_empty() {
        return Err("events_must_not_be_empty".to_string());
    }
    for (index, event) in source.events.iter().enumerate() {
        for (label, value) in [
            ("subject_bundle_id", event.subject_bundle_id.as_str()),
            (
                "verification_context_id",
                event.verification_context_id.as_str(),
            ),
            ("authority_chain_id", event.authority_chain_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("event_field_must_not_be_empty:{}:{}", index, label));
            }
        }
        if event.timestamp_unix_ns == 0 {
            return Err(format!("event_timestamp_unix_ns_must_be_non_zero:{index}"));
        }
        if event
            .run_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(format!("event_run_id_must_not_be_empty:{index}"));
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
                    "event_optional_field_must_not_be_empty:{index}:{label}"
                ));
            }
        }
    }
    Ok(())
}

fn write_outputs(
    config: &AuthoritySinkholeCompanionProducerConfig,
    metrics: &ProducerMetrics,
    violations: &[String],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create output dir {}: {error}",
            config.output_dir.display()
        )
    })?;
    for (source, output_name) in [
        (
            &config.replay_output_path,
            "replay_boundary_flow_report.json",
        ),
        (&config.trust_output_path, "trust_reuse_flow_report.json"),
    ] {
        if source.exists() {
            fs::copy(source, config.output_dir.join(output_name)).map_err(|error| {
                format!(
                    "failed to copy companion flow report {} to output dir: {error}",
                    source.display()
                )
            })?;
        }
    }
    write_json(
        &config
            .output_dir
            .join("authority_sinkhole_companion_flow_materialization_report.json"),
        &serde_json::json!({
            "status": "PASS",
            "mode": "phase13_authority_sinkhole_companion_flow_producer",
            "replay_source_path": config.replay_source_path.display().to_string(),
            "trust_source_path": config.trust_source_path.display().to_string(),
            "replay_output_path": config.replay_output_path.display().to_string(),
            "trust_output_path": config.trust_output_path.display().to_string(),
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "artifact": "authority-sinkhole-companion-flow-producer",
            "mode": "phase13_authority_sinkhole_companion_flow_producer",
            "verdict": "PASS",
            "detail_report_path": "authority_sinkhole_companion_flow_materialization_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_failure_outputs(
    config: &AuthoritySinkholeCompanionProducerConfig,
    metrics: &ProducerMetrics,
    violations: &[String],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create output dir {}: {error}",
            config.output_dir.display()
        )
    })?;
    write_json(
        &config
            .output_dir
            .join("authority_sinkhole_companion_flow_materialization_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_authority_sinkhole_companion_flow_producer",
            "replay_source_path": config.replay_source_path.display().to_string(),
            "trust_source_path": config.trust_source_path.display().to_string(),
            "replay_output_path": config.replay_output_path.display().to_string(),
            "trust_output_path": config.trust_output_path.display().to_string(),
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "artifact": "authority-sinkhole-companion-flow-producer",
            "mode": "phase13_authority_sinkhole_companion_flow_producer",
            "verdict": "FAIL",
            "detail_report_path": "authority_sinkhole_companion_flow_materialization_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_loading_failure_outputs(
    config: &AuthoritySinkholeCompanionProducerConfig,
    violations: &[String],
    load_error: &str,
    load_failure_stage: &str,
    missing_surfaces: &[String],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create output dir {}: {error}",
            config.output_dir.display()
        )
    })?;
    write_json(
        &config
            .output_dir
            .join("authority_sinkhole_companion_flow_materialization_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_authority_sinkhole_companion_flow_producer",
            "replay_source_path": config.replay_source_path.display().to_string(),
            "trust_source_path": config.trust_source_path.display().to_string(),
            "replay_output_path": config.replay_output_path.display().to_string(),
            "trust_output_path": config.trust_output_path.display().to_string(),
            "load_failure_stage": load_failure_stage,
            "load_error": load_error,
            "metrics": {
                "discovered_surface_count": 0,
                "materialized_surface_count": 0,
                "missing_surfaces": missing_surfaces,
                "surfaces": []
            },
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "artifact": "authority-sinkhole-companion-flow-producer",
            "mode": "phase13_authority_sinkhole_companion_flow_producer",
            "verdict": "FAIL",
            "detail_report_path": "authority_sinkhole_companion_flow_materialization_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize JSON for {}: {error}", path.display()))?;
    fs::write(path, bytes)
        .map_err(|error| format!("failed to write JSON {}: {error}", path.display()))
}

fn write_violations(path: &Path, violations: &[String]) -> Result<(), String> {
    let contents = if violations.is_empty() {
        String::new()
    } else {
        format!("{}\n", violations.join("\n"))
    };
    fs::write(path, contents)
        .map_err(|error| format!("failed to write violations {}: {error}", path.display()))
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
    use super::*;

    fn sample_source_document(flow_surface: &str) -> CompanionSourceDocument {
        CompanionSourceDocument {
            source_version: 1,
            flow_surface: flow_surface.to_string(),
            status: default_source_status(),
            run_id: "run-1".to_string(),
            window_model: default_window_model(),
            events: vec![
                CompanionSourceEvent {
                    run_id: None,
                    timestamp_unix_ns: 10,
                    subject_bundle_id: "bundle-a".to_string(),
                    verification_context_id: "context-a".to_string(),
                    authority_chain_id: "chain-a".to_string(),
                    terminal: true,
                    reused: true,
                    verification_node_id: Some("node-a".to_string()),
                    verifier_id: Some("verifier-a".to_string()),
                    lineage_id: Some("lineage-a".to_string()),
                    execution_cluster_id: Some("cluster-a".to_string()),
                    source_run_id: None,
                    replay_contract_id: None,
                    trust_reuse_source: None,
                    reuse_group_id: None,
                    surface_local_path_id: None,
                },
                CompanionSourceEvent {
                    run_id: None,
                    timestamp_unix_ns: 20,
                    subject_bundle_id: "bundle-b".to_string(),
                    verification_context_id: "context-a".to_string(),
                    authority_chain_id: "chain-b".to_string(),
                    terminal: true,
                    reused: false,
                    verification_node_id: None,
                    verifier_id: None,
                    lineage_id: None,
                    execution_cluster_id: None,
                    source_run_id: None,
                    replay_contract_id: None,
                    trust_reuse_source: None,
                    reuse_group_id: None,
                    surface_local_path_id: None,
                },
            ],
        }
    }

    #[test]
    fn source_document_validation_rejects_surface_mismatch() {
        let source = sample_source_document("trust_reuse");
        let error = validate_source_document(&source, "replay_boundary").unwrap_err();
        assert_eq!(
            error,
            "source_flow_surface_mismatch:trust_reuse:replay_boundary"
        );
    }

    #[test]
    fn materialized_surface_deduplicates_identical_events() {
        let tmp = std::env::temp_dir().join(format!(
            "authority_sinkhole_companion_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("temp dir should exist");
        let source_path = tmp.join("replay.json");
        let output_path = tmp.join("replay_boundary_flow_report.json");
        let mut source = sample_source_document("replay_boundary");
        source.events.push(source.events[0].clone());
        fs::write(
            &source_path,
            serde_json::to_vec_pretty(&source).expect("source should serialize"),
        )
        .expect("source should write");

        let metrics = materialize_surface("replay_boundary", &source_path, &output_path)
            .expect("materialize");

        assert_eq!(metrics.materialized_event_count, 2);
        assert_eq!(metrics.duplicate_skipped_count, 1);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn trust_reuse_source_without_reusable_events_materializes_not_evaluated_report() {
        let tmp = std::env::temp_dir().join(format!(
            "authority_sinkhole_companion_no_reuse_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("temp dir should exist");
        let source_path = tmp.join("trust.json");
        let output_path = tmp.join("trust_reuse_flow_report.json");
        let mut source = sample_source_document("trust_reuse");
        source.status = "NO_REUSABLE_EVENTS".to_string();
        source.events.clear();
        fs::write(
            &source_path,
            serde_json::to_vec_pretty(&source).expect("source should serialize"),
        )
        .expect("source should write");

        let metrics =
            materialize_surface("trust_reuse", &source_path, &output_path).expect("materialize");
        let report: AuthoritySinkholeCompanionFlowReport =
            serde_json::from_slice(&fs::read(&output_path).expect("read output"))
                .expect("parse report");

        assert_eq!(metrics.source_status, "NO_REUSABLE_EVENTS");
        assert_eq!(metrics.report_status, "NOT_EVALUATED");
        assert_eq!(report.status, "NOT_EVALUATED");
        assert_eq!(report.event_count, 0);
        let _ = fs::remove_dir_all(&tmp);
    }
}
