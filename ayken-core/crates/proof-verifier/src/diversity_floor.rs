use crate::diversity_ledger::{load_diversity_ledger_entries, VerificationDiversityLedgerEntry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const NANOS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Debug, Clone)]
pub struct DiversityFloorGateConfig {
    pub ledger_path: PathBuf,
    pub policy_path: PathBuf,
    pub output_dir: PathBuf,
    pub window_runs_override: Option<usize>,
    pub window_seconds_override: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    Pass,
    Fail,
}

impl GateVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DiversityPolicy {
    pub policy_version: u32,
    #[serde(default)]
    pub window_runs: Option<usize>,
    #[serde(default)]
    pub window_seconds: Option<u64>,
    pub min_unique_verifiers: usize,
    pub min_unique_verification_nodes: usize,
    pub min_unique_authority_chains: usize,
    pub min_unique_lineages: usize,
    pub max_dominance_ratio: f64,
    pub min_lineage_entropy: f64,
}

#[derive(Debug)]
pub struct DiversityFloorGateOutcome {
    pub verdict: GateVerdict,
    pub violations: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct DistributionEntry {
    pub id: String,
    pub count: usize,
    pub share: f64,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct DiversityMetrics {
    pub selected_entry_count: usize,
    pub unique_verifier_count: usize,
    pub unique_verification_node_count: usize,
    pub unique_authority_chain_count: usize,
    pub unique_lineage_count: usize,
    pub unique_execution_cluster_count: usize,
    pub dominance_ratio: f64,
    pub verifier_dominance_ratio: f64,
    pub verification_node_dominance_ratio: f64,
    pub authority_chain_dominance_ratio: f64,
    pub lineage_dominance_ratio: f64,
    pub execution_cluster_dominance_ratio: Option<f64>,
    pub verifier_entropy: f64,
    pub verification_node_entropy: f64,
    pub authority_chain_entropy: f64,
    pub lineage_entropy: f64,
    pub execution_cluster_entropy: Option<f64>,
}

#[derive(Debug, Clone)]
struct WindowSelection {
    selected_entries: Vec<VerificationDiversityLedgerEntry>,
    selected_entry_count: usize,
    total_entry_count: usize,
    post_time_filter_entry_count: usize,
    post_run_limit_entry_count: usize,
    reference_timestamp_unix_ns: Option<u64>,
    applied_window_runs: Option<usize>,
    applied_window_seconds: Option<u64>,
    empty_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
struct OptionalDistributionSummary {
    distribution: Vec<DistributionEntry>,
    present_entry_count: usize,
    missing_entry_count: usize,
}

pub fn run_diversity_floor_gate(
    config: &DiversityFloorGateConfig,
) -> Result<DiversityFloorGateOutcome, String> {
    let entries = match load_ledger_entries(&config.ledger_path) {
        Ok(entries) => entries,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_ledger:{}",
                config.ledger_path.display()
            )];
            write_loading_failure_outputs(config, &violations, &error, "ledger_load")?;
            return Ok(DiversityFloorGateOutcome {
                verdict: GateVerdict::Fail,
                violations,
            });
        }
    };
    let policy = match load_policy(&config.policy_path) {
        Ok(policy) => policy,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_policy:{}",
                config.policy_path.display()
            )];
            write_loading_failure_outputs(config, &violations, &error, "policy_load")?;
            return Ok(DiversityFloorGateOutcome {
                verdict: GateVerdict::Fail,
                violations,
            });
        }
    };

    let selection = slice_window(
        entries,
        config.window_runs_override.or(policy.window_runs),
        config.window_seconds_override.or(policy.window_seconds),
    );
    let lineage_distribution = build_distribution(&selection.selected_entries, |entry| {
        entry.lineage_id.clone()
    });
    let verifier_distribution = build_distribution(&selection.selected_entries, |entry| {
        entry.verifier_id.clone()
    });
    let authority_distribution = build_distribution(&selection.selected_entries, |entry| {
        entry.authority_chain_id.clone()
    });
    let node_distribution = build_distribution(&selection.selected_entries, |entry| {
        entry.verification_node_id.clone()
    });
    let cluster_distribution = build_optional_distribution(&selection.selected_entries, |entry| {
        entry.execution_cluster_id.clone()
    });
    let metrics = compute_metrics(
        &selection.selected_entries,
        &verifier_distribution,
        &node_distribution,
        &authority_distribution,
        &lineage_distribution,
        &cluster_distribution,
    );
    let violations = evaluate_policy(&metrics, &policy, &selection);

    write_outputs(
        config,
        &config.output_dir,
        &selection,
        &policy,
        &metrics,
        &verifier_distribution,
        &node_distribution,
        &authority_distribution,
        &lineage_distribution,
        &cluster_distribution,
        &violations,
    )?;

    Ok(DiversityFloorGateOutcome {
        verdict: if violations.is_empty() {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        },
        violations,
    })
}

fn load_ledger_entries(path: &Path) -> Result<Vec<VerificationDiversityLedgerEntry>, String> {
    load_diversity_ledger_entries(path)
}

fn load_policy(path: &Path) -> Result<DiversityPolicy, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read policy at {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse policy at {}: {error}", path.display()))
}

fn slice_window(
    entries: Vec<VerificationDiversityLedgerEntry>,
    window_runs: Option<usize>,
    window_seconds: Option<u64>,
) -> WindowSelection {
    let total_entry_count = entries.len();
    let reference_timestamp_unix_ns = entries.last().map(|entry| entry.timestamp_unix_ns);
    let mut selected_entries = entries;

    if let (Some(reference), Some(seconds)) = (reference_timestamp_unix_ns, window_seconds) {
        let min_timestamp = reference.saturating_sub(seconds.saturating_mul(NANOS_PER_SECOND));
        selected_entries.retain(|entry| entry.timestamp_unix_ns >= min_timestamp);
    }
    let post_time_filter_entry_count = selected_entries.len();

    if let Some(limit) = window_runs {
        if selected_entries.len() > limit {
            let start = selected_entries.len() - limit;
            selected_entries = selected_entries.split_off(start);
        }
    }
    let post_run_limit_entry_count = selected_entries.len();
    let empty_reason = if total_entry_count == 0 {
        Some("empty_ledger")
    } else if post_time_filter_entry_count == 0 {
        Some("empty_window_after_time_filter")
    } else if post_run_limit_entry_count == 0 {
        Some("empty_window_after_run_limit")
    } else {
        None
    };

    WindowSelection {
        selected_entry_count: selected_entries.len(),
        selected_entries,
        total_entry_count,
        post_time_filter_entry_count,
        post_run_limit_entry_count,
        reference_timestamp_unix_ns,
        applied_window_runs: window_runs,
        applied_window_seconds: window_seconds,
        empty_reason,
    }
}

fn build_distribution<F>(
    entries: &[VerificationDiversityLedgerEntry],
    key_fn: F,
) -> Vec<DistributionEntry>
where
    F: Fn(&VerificationDiversityLedgerEntry) -> String,
{
    let mut counts = BTreeMap::<String, usize>::new();
    for entry in entries {
        *counts.entry(key_fn(entry)).or_insert(0) += 1;
    }

    let total = entries.len() as f64;
    let mut distribution: Vec<DistributionEntry> = counts
        .into_iter()
        .map(|(id, count)| DistributionEntry {
            id,
            count,
            share: if total == 0.0 {
                0.0
            } else {
                count as f64 / total
            },
        })
        .collect();
    distribution.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.id.cmp(&right.id))
    });
    distribution
}

fn build_optional_distribution<F>(
    entries: &[VerificationDiversityLedgerEntry],
    key_fn: F,
) -> OptionalDistributionSummary
where
    F: Fn(&VerificationDiversityLedgerEntry) -> Option<String>,
{
    let mut counts = BTreeMap::<String, usize>::new();
    let mut present_entry_count = 0usize;
    let mut missing_entry_count = 0usize;
    for entry in entries {
        match key_fn(entry) {
            Some(value) if !value.is_empty() => {
                present_entry_count += 1;
                *counts.entry(value).or_insert(0) += 1;
            }
            _ => {
                missing_entry_count += 1;
            }
        }
    }

    let total = present_entry_count as f64;
    let mut distribution: Vec<DistributionEntry> = counts
        .into_iter()
        .map(|(id, count)| DistributionEntry {
            id,
            count,
            share: if total == 0.0 {
                0.0
            } else {
                count as f64 / total
            },
        })
        .collect();
    distribution.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.id.cmp(&right.id))
    });

    OptionalDistributionSummary {
        distribution,
        present_entry_count,
        missing_entry_count,
    }
}

fn compute_metrics(
    entries: &[VerificationDiversityLedgerEntry],
    verifier_distribution: &[DistributionEntry],
    node_distribution: &[DistributionEntry],
    authority_distribution: &[DistributionEntry],
    lineage_distribution: &[DistributionEntry],
    cluster_distribution: &OptionalDistributionSummary,
) -> DiversityMetrics {
    let verifier_dominance_ratio = distribution_dominance(verifier_distribution);
    let verification_node_dominance_ratio = distribution_dominance(node_distribution);
    let authority_chain_dominance_ratio = distribution_dominance(authority_distribution);
    let lineage_dominance_ratio = distribution_dominance(lineage_distribution);
    let execution_cluster_dominance_ratio = if cluster_distribution.present_entry_count == 0 {
        None
    } else {
        Some(distribution_dominance(&cluster_distribution.distribution))
    };
    DiversityMetrics {
        selected_entry_count: entries.len(),
        unique_verifier_count: verifier_distribution.len(),
        unique_verification_node_count: node_distribution.len(),
        unique_authority_chain_count: authority_distribution.len(),
        unique_lineage_count: lineage_distribution.len(),
        unique_execution_cluster_count: cluster_distribution.distribution.len(),
        dominance_ratio: verifier_dominance_ratio,
        verifier_dominance_ratio,
        verification_node_dominance_ratio,
        authority_chain_dominance_ratio,
        lineage_dominance_ratio,
        execution_cluster_dominance_ratio,
        verifier_entropy: compute_shannon_entropy(verifier_distribution),
        verification_node_entropy: compute_shannon_entropy(node_distribution),
        authority_chain_entropy: compute_shannon_entropy(authority_distribution),
        lineage_entropy: compute_shannon_entropy(lineage_distribution),
        execution_cluster_entropy: if cluster_distribution.present_entry_count == 0 {
            None
        } else {
            Some(compute_shannon_entropy(&cluster_distribution.distribution))
        },
    }
}

fn compute_shannon_entropy(distribution: &[DistributionEntry]) -> f64 {
    distribution
        .iter()
        .filter(|entry| entry.share > 0.0)
        .map(|entry| -entry.share * entry.share.log2())
        .sum()
}

fn distribution_dominance(distribution: &[DistributionEntry]) -> f64 {
    distribution.first().map(|entry| entry.share).unwrap_or(0.0)
}

fn evaluate_policy(
    metrics: &DiversityMetrics,
    policy: &DiversityPolicy,
    selection: &WindowSelection,
) -> Vec<String> {
    let mut violations = Vec::new();
    if selection.selected_entry_count == 0 {
        let reason = selection.empty_reason.unwrap_or("empty_window");
        violations.push(format!("diversity_floor_violation:{reason}"));
        return violations;
    }
    if metrics.unique_verifier_count < policy.min_unique_verifiers {
        violations.push(format!(
            "diversity_floor_violation:unique_verifier_count:actual={}:min={}",
            metrics.unique_verifier_count, policy.min_unique_verifiers
        ));
    }
    if metrics.unique_verification_node_count < policy.min_unique_verification_nodes {
        violations.push(format!(
            "diversity_floor_violation:unique_verification_node_count:actual={}:min={}",
            metrics.unique_verification_node_count, policy.min_unique_verification_nodes
        ));
    }
    if metrics.unique_authority_chain_count < policy.min_unique_authority_chains {
        violations.push(format!(
            "diversity_floor_violation:unique_authority_chain_count:actual={}:min={}",
            metrics.unique_authority_chain_count, policy.min_unique_authority_chains
        ));
    }
    if metrics.unique_lineage_count < policy.min_unique_lineages {
        violations.push(format!(
            "diversity_floor_violation:unique_lineage_count:actual={}:min={}",
            metrics.unique_lineage_count, policy.min_unique_lineages
        ));
    }
    if metrics.dominance_ratio > policy.max_dominance_ratio {
        violations.push(format!(
            "diversity_floor_violation:dominance_ratio:actual={:.6}:max={:.6}",
            metrics.dominance_ratio, policy.max_dominance_ratio
        ));
    }
    if metrics.lineage_entropy < policy.min_lineage_entropy {
        violations.push(format!(
            "diversity_floor_violation:lineage_entropy:actual={:.6}:min={:.6}",
            metrics.lineage_entropy, policy.min_lineage_entropy
        ));
    }
    violations
}

fn write_outputs(
    config: &DiversityFloorGateConfig,
    output_dir: &Path,
    selection: &WindowSelection,
    policy: &DiversityPolicy,
    metrics: &DiversityMetrics,
    verifier_distribution: &[DistributionEntry],
    node_distribution: &[DistributionEntry],
    authority_distribution: &[DistributionEntry],
    lineage_distribution: &[DistributionEntry],
    cluster_distribution: &OptionalDistributionSummary,
    violations: &[String],
) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "failed to create output dir {}: {error}",
            output_dir.display()
        )
    })?;

    let verdict = if violations.is_empty() {
        GateVerdict::Pass
    } else {
        GateVerdict::Fail
    };

    write_json(
        &output_dir.join("vdl_window.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "window_model": "dual_window",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "total_entry_count": selection.total_entry_count,
            "post_time_filter_entry_count": selection.post_time_filter_entry_count,
            "post_run_limit_entry_count": selection.post_run_limit_entry_count,
            "selected_entry_count": selection.selected_entry_count,
            "applied_window_runs": selection.applied_window_runs,
            "applied_window_seconds": selection.applied_window_seconds,
            "reference_timestamp_unix_ns": selection.reference_timestamp_unix_ns,
            "empty_reason": selection.empty_reason,
            "entries": selection.selected_entries,
        }),
    )?;
    write_json(
        &output_dir.join("diversity_metrics.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "window_model": "dual_window",
            "selected_entry_count": metrics.selected_entry_count,
            "unique_verifier_count": metrics.unique_verifier_count,
            "unique_verification_node_count": metrics.unique_verification_node_count,
            "unique_authority_chain_count": metrics.unique_authority_chain_count,
            "unique_lineage_count": metrics.unique_lineage_count,
            "unique_execution_cluster_count": metrics.unique_execution_cluster_count,
            "dominance_ratio": metrics.dominance_ratio,
            "verifier_dominance_ratio": metrics.verifier_dominance_ratio,
            "verification_node_dominance_ratio": metrics.verification_node_dominance_ratio,
            "authority_chain_dominance_ratio": metrics.authority_chain_dominance_ratio,
            "lineage_dominance_ratio": metrics.lineage_dominance_ratio,
            "execution_cluster_dominance_ratio": metrics.execution_cluster_dominance_ratio,
            "verifier_entropy": metrics.verifier_entropy,
            "verification_node_entropy": metrics.verification_node_entropy,
            "authority_chain_entropy": metrics.authority_chain_entropy,
            "lineage_entropy": metrics.lineage_entropy,
            "execution_cluster_entropy": metrics.execution_cluster_entropy,
        }),
    )?;
    write_json(
        &output_dir.join("lineage_distribution.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "distribution": lineage_distribution,
        }),
    )?;
    write_json(
        &output_dir.join("cluster_distribution.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "distribution": cluster_distribution.distribution,
            "unique_execution_cluster_count": metrics.unique_execution_cluster_count,
            "present_entry_count": cluster_distribution.present_entry_count,
            "missing_entry_count": cluster_distribution.missing_entry_count,
        }),
    )?;
    write_json(
        &output_dir.join("dominance_analysis.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "verifier_distribution": verifier_distribution,
            "verification_node_distribution": node_distribution,
            "authority_chain_distribution": authority_distribution,
            "lineage_distribution": lineage_distribution,
            "execution_cluster_distribution": cluster_distribution.distribution,
            "dominant_verifier_id": verifier_distribution.first().map(|entry| entry.id.clone()),
            "dominant_verifier_share": metrics.verifier_dominance_ratio,
            "dominant_verification_node_id": node_distribution.first().map(|entry| entry.id.clone()),
            "dominant_verification_node_share": metrics.verification_node_dominance_ratio,
            "dominant_authority_chain_id": authority_distribution.first().map(|entry| entry.id.clone()),
            "dominant_authority_chain_share": metrics.authority_chain_dominance_ratio,
            "dominant_lineage_id": lineage_distribution.first().map(|entry| entry.id.clone()),
            "dominant_lineage_share": metrics.lineage_dominance_ratio,
            "dominant_execution_cluster_id": cluster_distribution.distribution.first().map(|entry| entry.id.clone()),
            "dominant_execution_cluster_share": metrics.execution_cluster_dominance_ratio,
        }),
    )?;
    write_json(
        &output_dir.join("entropy_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "verifier_entropy": metrics.verifier_entropy,
            "verification_node_entropy": metrics.verification_node_entropy,
            "authority_chain_entropy": metrics.authority_chain_entropy,
            "lineage_entropy": metrics.lineage_entropy,
            "execution_cluster_entropy": metrics.execution_cluster_entropy,
            "minimum_required_lineage_entropy": policy.min_lineage_entropy,
        }),
    )?;
    write_json(
        &output_dir.join("verification_diversity_floor_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "mode": "phase13_verification_diversity_floor_gate",
            "risk_class": "verification-gravity-drift",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "window_model": "dual_window",
            "applied_window_runs": selection.applied_window_runs,
            "applied_window_seconds": selection.applied_window_seconds,
            "reference_timestamp_unix_ns": selection.reference_timestamp_unix_ns,
            "empty_reason": selection.empty_reason,
            "policy": policy,
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &output_dir.join("report.json"),
        &serde_json::json!({
            "gate": "verification-diversity-floor",
            "mode": "phase13_verification_diversity_floor_gate",
            "verdict": verdict.as_str(),
            "detail_report_path": "verification_diversity_floor_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_loading_failure_outputs(
    config: &DiversityFloorGateConfig,
    violations: &[String],
    load_error: &str,
    load_failure_stage: &str,
) -> Result<(), String> {
    let output_dir = &config.output_dir;
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "failed to create output dir {}: {error}",
            output_dir.display()
        )
    })?;

    write_json(
        &output_dir.join("verification_diversity_floor_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_verification_diversity_floor_gate",
            "risk_class": "verification-gravity-drift",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "window_model": "dual_window",
            "applied_window_runs": config.window_runs_override,
            "applied_window_seconds": config.window_seconds_override,
            "reference_timestamp_unix_ns": serde_json::Value::Null,
            "empty_reason": "load_failure",
            "load_failure_stage": load_failure_stage,
            "load_error": load_error,
            "policy": serde_json::Value::Null,
            "metrics": {
                "selected_entry_count": 0,
                "unique_verifier_count": 0,
                "unique_verification_node_count": 0,
                "unique_authority_chain_count": 0,
                "unique_lineage_count": 0,
                "unique_execution_cluster_count": 0,
                "dominance_ratio": 0.0,
                "verifier_dominance_ratio": 0.0,
                "verification_node_dominance_ratio": 0.0,
                "authority_chain_dominance_ratio": 0.0,
                "lineage_dominance_ratio": 0.0,
                "execution_cluster_dominance_ratio": serde_json::Value::Null,
                "verifier_entropy": 0.0,
                "verification_node_entropy": 0.0,
                "authority_chain_entropy": 0.0,
                "lineage_entropy": 0.0,
                "execution_cluster_entropy": serde_json::Value::Null
            },
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &output_dir.join("report.json"),
        &serde_json::json!({
            "gate": "verification-diversity-floor",
            "mode": "phase13_verification_diversity_floor_gate",
            "verdict": "FAIL",
            "detail_report_path": "verification_diversity_floor_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &output_dir.join("vdl_window.json"),
        &serde_json::json!({
            "status": "FAIL",
            "window_model": "dual_window",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "entries": [],
            "selected_entry_count": 0,
            "total_entry_count": 0,
            "post_time_filter_entry_count": 0,
            "post_run_limit_entry_count": 0,
            "applied_window_runs": config.window_runs_override,
            "applied_window_seconds": config.window_seconds_override,
            "reference_timestamp_unix_ns": serde_json::Value::Null,
            "empty_reason": "load_failure"
        }),
    )?;
    write_json(
        &output_dir.join("diversity_metrics.json"),
        &serde_json::json!({
            "status": "FAIL",
            "selected_entry_count": 0,
            "unique_verifier_count": 0,
            "unique_verification_node_count": 0,
            "unique_authority_chain_count": 0,
            "unique_lineage_count": 0,
            "unique_execution_cluster_count": 0,
            "dominance_ratio": 0.0,
            "verifier_dominance_ratio": 0.0,
            "verification_node_dominance_ratio": 0.0,
            "authority_chain_dominance_ratio": 0.0,
            "lineage_dominance_ratio": 0.0,
            "execution_cluster_dominance_ratio": serde_json::Value::Null,
            "verifier_entropy": 0.0,
            "verification_node_entropy": 0.0,
            "authority_chain_entropy": 0.0,
            "lineage_entropy": 0.0,
            "execution_cluster_entropy": serde_json::Value::Null
        }),
    )?;
    write_json(
        &output_dir.join("lineage_distribution.json"),
        &serde_json::json!({
            "status": "FAIL",
            "distribution": []
        }),
    )?;
    write_json(
        &output_dir.join("cluster_distribution.json"),
        &serde_json::json!({
            "status": "FAIL",
            "distribution": [],
            "unique_execution_cluster_count": 0,
            "present_entry_count": 0,
            "missing_entry_count": 0
        }),
    )?;
    write_json(
        &output_dir.join("dominance_analysis.json"),
        &serde_json::json!({
            "status": "FAIL",
            "verifier_distribution": [],
            "verification_node_distribution": [],
            "authority_chain_distribution": [],
            "lineage_distribution": [],
            "execution_cluster_distribution": []
        }),
    )?;
    write_json(
        &output_dir.join("entropy_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "verifier_entropy": 0.0,
            "verification_node_entropy": 0.0,
            "authority_chain_entropy": 0.0,
            "lineage_entropy": 0.0,
            "execution_cluster_entropy": serde_json::Value::Null
        }),
    )?;
    write_violations(&output_dir.join("violations.txt"), violations)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(
        timestamp_unix_ns: u64,
        verifier_id: &str,
        node_id: &str,
        authority_chain_id: &str,
        lineage_id: &str,
    ) -> VerificationDiversityLedgerEntry {
        VerificationDiversityLedgerEntry {
            ledger_version: 1,
            entry_id: format!("entry-{timestamp_unix_ns}-{verifier_id}"),
            run_id: format!("run-{timestamp_unix_ns}"),
            timestamp_unix_ns,
            subject_bundle_id: "bundle-a".to_string(),
            verification_context_id: "context-a".to_string(),
            verification_node_id: node_id.to_string(),
            verifier_id: verifier_id.to_string(),
            authority_chain_id: authority_chain_id.to_string(),
            lineage_id: lineage_id.to_string(),
            execution_cluster_id: None,
            verdict: "PASS".to_string(),
            receipt_hash: format!("receipt-{timestamp_unix_ns}"),
        }
    }

    #[test]
    fn slice_window_applies_time_then_run_limit() {
        let entries = vec![
            sample_entry(1 * NANOS_PER_SECOND, "v1", "n1", "a1", "l1"),
            sample_entry(2 * NANOS_PER_SECOND, "v2", "n2", "a2", "l2"),
            sample_entry(3 * NANOS_PER_SECOND, "v3", "n3", "a3", "l3"),
            sample_entry(4 * NANOS_PER_SECOND, "v4", "n4", "a4", "l4"),
        ];

        let selection = slice_window(entries, Some(2), Some(2));

        assert_eq!(selection.total_entry_count, 4);
        assert_eq!(selection.selected_entry_count, 2);
        assert_eq!(
            selection
                .selected_entries
                .iter()
                .map(|entry| entry.verifier_id.as_str())
                .collect::<Vec<_>>(),
            vec!["v3", "v4"]
        );
    }

    #[test]
    fn compute_metrics_detects_dominance_and_entropy() {
        let entries = vec![
            sample_entry(1, "v1", "n1", "a1", "l1"),
            sample_entry(2, "v1", "n1", "a1", "l1"),
            sample_entry(3, "v2", "n2", "a2", "l2"),
            sample_entry(4, "v3", "n3", "a3", "l3"),
        ];

        let verifier_distribution = build_distribution(&entries, |entry| entry.verifier_id.clone());
        let node_distribution =
            build_distribution(&entries, |entry| entry.verification_node_id.clone());
        let authority_distribution =
            build_distribution(&entries, |entry| entry.authority_chain_id.clone());
        let lineage_distribution = build_distribution(&entries, |entry| entry.lineage_id.clone());
        let cluster_distribution =
            build_optional_distribution(&entries, |entry| entry.execution_cluster_id.clone());
        let metrics = compute_metrics(
            &entries,
            &verifier_distribution,
            &node_distribution,
            &authority_distribution,
            &lineage_distribution,
            &cluster_distribution,
        );

        assert_eq!(metrics.unique_verifier_count, 3);
        assert_eq!(metrics.unique_verification_node_count, 3);
        assert_eq!(metrics.unique_authority_chain_count, 3);
        assert_eq!(metrics.unique_lineage_count, 3);
        assert_eq!(metrics.unique_execution_cluster_count, 0);
        assert!((metrics.dominance_ratio - 0.5).abs() < 0.000_001);
        assert!((metrics.verifier_dominance_ratio - 0.5).abs() < 0.000_001);
        assert!((metrics.authority_chain_dominance_ratio - 0.5).abs() < 0.000_001);
        assert!(metrics.lineage_entropy > 1.4);
    }

    #[test]
    fn evaluate_policy_collects_floor_violations() {
        let metrics = DiversityMetrics {
            selected_entry_count: 4,
            unique_verifier_count: 2,
            unique_verification_node_count: 2,
            unique_authority_chain_count: 1,
            unique_lineage_count: 1,
            unique_execution_cluster_count: 0,
            dominance_ratio: 0.75,
            verifier_dominance_ratio: 0.75,
            verification_node_dominance_ratio: 0.75,
            authority_chain_dominance_ratio: 1.0,
            lineage_dominance_ratio: 1.0,
            execution_cluster_dominance_ratio: None,
            verifier_entropy: 0.81,
            verification_node_entropy: 0.81,
            authority_chain_entropy: 0.0,
            lineage_entropy: 0.2,
            execution_cluster_entropy: None,
        };
        let policy = DiversityPolicy {
            policy_version: 1,
            window_runs: Some(10),
            window_seconds: Some(60),
            min_unique_verifiers: 3,
            min_unique_verification_nodes: 3,
            min_unique_authority_chains: 2,
            min_unique_lineages: 2,
            max_dominance_ratio: 0.4,
            min_lineage_entropy: 1.2,
        };

        let selection = WindowSelection {
            selected_entries: Vec::new(),
            selected_entry_count: metrics.selected_entry_count,
            total_entry_count: metrics.selected_entry_count,
            post_time_filter_entry_count: metrics.selected_entry_count,
            post_run_limit_entry_count: metrics.selected_entry_count,
            reference_timestamp_unix_ns: Some(4),
            applied_window_runs: Some(10),
            applied_window_seconds: Some(60),
            empty_reason: None,
        };
        let violations = evaluate_policy(&metrics, &policy, &selection);

        assert_eq!(violations.len(), 6);
        assert!(violations
            .iter()
            .any(|item| item.contains("unique_verifier_count")));
        assert!(violations
            .iter()
            .any(|item| item.contains("dominance_ratio")));
        assert!(violations
            .iter()
            .any(|item| item.contains("lineage_entropy")));
    }

    #[test]
    fn evaluate_policy_reports_empty_window_reason() {
        let metrics = DiversityMetrics {
            selected_entry_count: 0,
            unique_verifier_count: 0,
            unique_verification_node_count: 0,
            unique_authority_chain_count: 0,
            unique_lineage_count: 0,
            unique_execution_cluster_count: 0,
            dominance_ratio: 0.0,
            verifier_dominance_ratio: 0.0,
            verification_node_dominance_ratio: 0.0,
            authority_chain_dominance_ratio: 0.0,
            lineage_dominance_ratio: 0.0,
            execution_cluster_dominance_ratio: None,
            verifier_entropy: 0.0,
            verification_node_entropy: 0.0,
            authority_chain_entropy: 0.0,
            lineage_entropy: 0.0,
            execution_cluster_entropy: None,
        };
        let policy = DiversityPolicy {
            policy_version: 1,
            window_runs: Some(0),
            window_seconds: Some(60),
            min_unique_verifiers: 3,
            min_unique_verification_nodes: 3,
            min_unique_authority_chains: 2,
            min_unique_lineages: 2,
            max_dominance_ratio: 0.4,
            min_lineage_entropy: 1.2,
        };
        let selection = WindowSelection {
            selected_entries: Vec::new(),
            selected_entry_count: 0,
            total_entry_count: 4,
            post_time_filter_entry_count: 4,
            post_run_limit_entry_count: 0,
            reference_timestamp_unix_ns: Some(4),
            applied_window_runs: Some(0),
            applied_window_seconds: Some(60),
            empty_reason: Some("empty_window_after_run_limit"),
        };

        let violations = evaluate_policy(&metrics, &policy, &selection);

        assert_eq!(
            violations,
            vec!["diversity_floor_violation:empty_window_after_run_limit".to_string()]
        );
    }
}
