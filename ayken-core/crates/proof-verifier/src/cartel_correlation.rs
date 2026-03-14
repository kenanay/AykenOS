use crate::diversity_floor::GateVerdict;
use crate::diversity_ledger::{load_diversity_ledger_entries, VerificationDiversityLedgerEntry};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const NANOS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Debug, Clone)]
pub struct CartelCorrelationGateConfig {
    pub ledger_path: PathBuf,
    pub policy_path: PathBuf,
    pub output_dir: PathBuf,
    pub window_runs_override: Option<usize>,
    pub window_seconds_override: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CartelCorrelationPolicy {
    pub policy_version: u32,
    #[serde(default)]
    pub window_runs: Option<usize>,
    #[serde(default)]
    pub window_seconds: Option<u64>,
    pub min_shared_events: usize,
    pub pairwise_correlation_threshold: f64,
    pub lineage_conditioned_correlation_threshold: f64,
    pub authority_chain_conditioned_correlation_threshold: f64,
    pub max_execution_cluster_overlap_ratio: f64,
    pub stability_window_runs: usize,
    pub stability_window_count: usize,
    pub stability_min_high_windows: usize,
    #[serde(default)]
    pub stability_correlation_threshold: Option<f64>,
}

#[derive(Debug)]
pub struct CartelCorrelationGateOutcome {
    pub verdict: GateVerdict,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone)]
struct WindowSelection {
    entries: Vec<VerificationDiversityLedgerEntry>,
    total_entry_count: usize,
    post_time_filter_entry_count: usize,
    post_run_limit_entry_count: usize,
    selected_entry_count: usize,
    reference_timestamp_unix_ns: Option<u64>,
    applied_window_runs: Option<usize>,
    applied_window_seconds: Option<u64>,
    empty_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EventKey {
    subject_bundle_id: String,
    verification_context_id: String,
}

#[derive(Debug, Clone)]
struct VerifierEvent {
    verdict: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct VerifierMetadata {
    verifier_id: String,
    entry_count: usize,
    lineage_id: String,
    authority_chain_id: String,
    execution_cluster_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct PairwiseCorrelationRecord {
    verifier_a: String,
    verifier_b: String,
    shared_event_count: usize,
    agreement_count: usize,
    pairwise_verdict_correlation: f64,
    lineage_id: Option<String>,
    authority_chain_id: Option<String>,
    execution_cluster_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct StabilityRecord {
    verifier_a: String,
    verifier_b: String,
    high_window_count: usize,
    evaluated_window_count: usize,
    max_window_correlation: f64,
    min_window_correlation: f64,
    sustained_high_correlation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ClusterOverlapRecord {
    execution_cluster_id: String,
    verifier_count: usize,
    share: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct CartelCorrelationMetrics {
    selected_entry_count: usize,
    unique_verifier_count: usize,
    pairwise_pair_count: usize,
    max_pairwise_correlation: f64,
    suspicious_pairwise_pair_count: usize,
    suspicious_lineage_pair_count: usize,
    suspicious_authority_pair_count: usize,
    max_execution_cluster_overlap_ratio: Option<f64>,
    suspicious_execution_cluster_overlap: bool,
    suspicious_stability_pair_count: usize,
}

pub fn run_cartel_correlation_gate(
    config: &CartelCorrelationGateConfig,
) -> Result<CartelCorrelationGateOutcome, String> {
    let entries = match load_ledger_entries(&config.ledger_path) {
        Ok(entries) => entries,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_ledger:{}",
                config.ledger_path.display()
            )];
            write_loading_failure_outputs(config, &violations, &error, "ledger_load")?;
            return Ok(CartelCorrelationGateOutcome {
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
            return Ok(CartelCorrelationGateOutcome {
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
    let metadata = derive_verifier_metadata(&selection.entries);
    let pairwise_records = build_pairwise_correlation_records(&selection.entries, &metadata);
    let lineage_records = filter_same_lineage(&pairwise_records);
    let authority_records = filter_same_authority_chain(&pairwise_records);
    let cluster_overlap = compute_execution_cluster_overlap(&metadata);
    let stability_records = compute_stability_records(&selection.entries, &metadata, &policy);

    let metrics = build_metrics(
        selection.selected_entry_count,
        metadata.len(),
        &pairwise_records,
        &lineage_records,
        &authority_records,
        &cluster_overlap,
        &stability_records,
        &policy,
    );
    let violations = evaluate_policy(
        &selection,
        &pairwise_records,
        &lineage_records,
        &authority_records,
        &cluster_overlap,
        &stability_records,
        &policy,
    );

    write_outputs(
        config,
        &selection,
        &policy,
        &metrics,
        &pairwise_records,
        &lineage_records,
        &authority_records,
        &cluster_overlap,
        &stability_records,
        &violations,
    )?;

    Ok(CartelCorrelationGateOutcome {
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

fn load_policy(path: &Path) -> Result<CartelCorrelationPolicy, String> {
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
        entries: selected_entries,
        total_entry_count,
        post_time_filter_entry_count,
        post_run_limit_entry_count,
        reference_timestamp_unix_ns,
        applied_window_runs: window_runs,
        applied_window_seconds: window_seconds,
        empty_reason,
    }
}

fn derive_verifier_metadata(
    entries: &[VerificationDiversityLedgerEntry],
) -> BTreeMap<String, VerifierMetadata> {
    let mut grouped = BTreeMap::<String, Vec<&VerificationDiversityLedgerEntry>>::new();
    for entry in entries {
        grouped
            .entry(entry.verifier_id.clone())
            .or_default()
            .push(entry);
    }

    grouped
        .into_iter()
        .map(|(verifier_id, verifier_entries)| {
            let lineage_id = dominant_required_value(&verifier_entries, |entry| &entry.lineage_id);
            let authority_chain_id =
                dominant_required_value(&verifier_entries, |entry| &entry.authority_chain_id);
            let execution_cluster_id = dominant_optional_value(&verifier_entries, |entry| {
                entry.execution_cluster_id.as_deref()
            });
            (
                verifier_id.clone(),
                VerifierMetadata {
                    verifier_id,
                    entry_count: verifier_entries.len(),
                    lineage_id,
                    authority_chain_id,
                    execution_cluster_id,
                },
            )
        })
        .collect()
}

fn dominant_required_value<'a, F>(
    entries: &'a [&'a VerificationDiversityLedgerEntry],
    value_fn: F,
) -> String
where
    F: Fn(&VerificationDiversityLedgerEntry) -> &str,
{
    let mut counts = BTreeMap::<String, usize>::new();
    for entry in entries {
        *counts.entry(value_fn(entry).to_string()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(value, _)| value)
        .unwrap_or_default()
}

fn dominant_optional_value<'a, F>(
    entries: &'a [&'a VerificationDiversityLedgerEntry],
    value_fn: F,
) -> Option<String>
where
    F: Fn(&VerificationDiversityLedgerEntry) -> Option<&str>,
{
    let mut counts = BTreeMap::<String, usize>::new();
    for entry in entries {
        if let Some(value) = value_fn(entry) {
            if !value.is_empty() {
                *counts.entry(value.to_string()).or_insert(0) += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(value, _)| value)
}

fn build_pairwise_correlation_records(
    entries: &[VerificationDiversityLedgerEntry],
    metadata: &BTreeMap<String, VerifierMetadata>,
) -> Vec<PairwiseCorrelationRecord> {
    let event_maps = build_verifier_event_maps(entries);
    let verifier_ids: Vec<String> = metadata.keys().cloned().collect();
    let mut records = Vec::new();

    for left_index in 0..verifier_ids.len() {
        for right_index in (left_index + 1)..verifier_ids.len() {
            let verifier_a = &verifier_ids[left_index];
            let verifier_b = &verifier_ids[right_index];
            let left_events = match event_maps.get(verifier_a) {
                Some(value) => value,
                None => continue,
            };
            let right_events = match event_maps.get(verifier_b) {
                Some(value) => value,
                None => continue,
            };
            let shared_keys: Vec<&EventKey> = left_events
                .keys()
                .filter(|key| right_events.contains_key(*key))
                .collect();
            if shared_keys.is_empty() {
                continue;
            }
            let mut agreement_count = 0usize;
            for key in &shared_keys {
                if left_events.get(*key).map(|value| value.verdict.as_str())
                    == right_events.get(*key).map(|value| value.verdict.as_str())
                {
                    agreement_count += 1;
                }
            }
            let shared_event_count = shared_keys.len();
            let pairwise_verdict_correlation = agreement_count as f64 / shared_event_count as f64;
            let meta_a = metadata.get(verifier_a).expect("verifier metadata present");
            let meta_b = metadata.get(verifier_b).expect("verifier metadata present");
            records.push(PairwiseCorrelationRecord {
                verifier_a: verifier_a.clone(),
                verifier_b: verifier_b.clone(),
                shared_event_count,
                agreement_count,
                pairwise_verdict_correlation,
                lineage_id: if meta_a.lineage_id == meta_b.lineage_id {
                    Some(meta_a.lineage_id.clone())
                } else {
                    None
                },
                authority_chain_id: if meta_a.authority_chain_id == meta_b.authority_chain_id {
                    Some(meta_a.authority_chain_id.clone())
                } else {
                    None
                },
                execution_cluster_id: match (
                    meta_a.execution_cluster_id.as_ref(),
                    meta_b.execution_cluster_id.as_ref(),
                ) {
                    (Some(left), Some(right)) if left == right => Some(left.clone()),
                    _ => None,
                },
            });
        }
    }

    records.sort_by(|left, right| {
        right
            .pairwise_verdict_correlation
            .total_cmp(&left.pairwise_verdict_correlation)
            .then_with(|| left.verifier_a.cmp(&right.verifier_a))
            .then_with(|| left.verifier_b.cmp(&right.verifier_b))
    });
    records
}

fn build_verifier_event_maps(
    entries: &[VerificationDiversityLedgerEntry],
) -> BTreeMap<String, BTreeMap<EventKey, VerifierEvent>> {
    let mut maps = BTreeMap::<String, BTreeMap<EventKey, VerifierEvent>>::new();
    for entry in entries {
        let key = EventKey {
            subject_bundle_id: entry.subject_bundle_id.clone(),
            verification_context_id: entry.verification_context_id.clone(),
        };
        maps.entry(entry.verifier_id.clone()).or_default().insert(
            key,
            VerifierEvent {
                verdict: entry.verdict.clone(),
            },
        );
    }
    maps
}

fn filter_same_lineage(records: &[PairwiseCorrelationRecord]) -> Vec<PairwiseCorrelationRecord> {
    records
        .iter()
        .filter(|record| record.lineage_id.is_some())
        .cloned()
        .collect()
}

fn filter_same_authority_chain(
    records: &[PairwiseCorrelationRecord],
) -> Vec<PairwiseCorrelationRecord> {
    records
        .iter()
        .filter(|record| record.authority_chain_id.is_some())
        .cloned()
        .collect()
}

fn compute_execution_cluster_overlap(
    metadata: &BTreeMap<String, VerifierMetadata>,
) -> Vec<ClusterOverlapRecord> {
    let mut cluster_to_verifiers = BTreeMap::<String, BTreeSet<String>>::new();
    for verifier in metadata.values() {
        if let Some(cluster_id) = &verifier.execution_cluster_id {
            cluster_to_verifiers
                .entry(cluster_id.clone())
                .or_default()
                .insert(verifier.verifier_id.clone());
        }
    }
    let total_verifiers = cluster_to_verifiers
        .values()
        .fold(BTreeSet::<String>::new(), |mut acc, set| {
            acc.extend(set.iter().cloned());
            acc
        })
        .len();
    let total = total_verifiers as f64;
    let mut records: Vec<ClusterOverlapRecord> = cluster_to_verifiers
        .into_iter()
        .map(|(execution_cluster_id, verifiers)| ClusterOverlapRecord {
            execution_cluster_id,
            verifier_count: verifiers.len(),
            share: if total == 0.0 {
                0.0
            } else {
                verifiers.len() as f64 / total
            },
        })
        .collect();
    records.sort_by(|left, right| {
        right
            .share
            .total_cmp(&left.share)
            .then_with(|| left.execution_cluster_id.cmp(&right.execution_cluster_id))
    });
    records
}

fn compute_stability_records(
    entries: &[VerificationDiversityLedgerEntry],
    metadata: &BTreeMap<String, VerifierMetadata>,
    policy: &CartelCorrelationPolicy,
) -> Vec<StabilityRecord> {
    if policy.stability_window_runs == 0 || policy.stability_window_count == 0 {
        return Vec::new();
    }
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by_key(|entry| entry.timestamp_unix_ns);
    let mut windows = Vec::<Vec<VerificationDiversityLedgerEntry>>::new();
    let mut cursor = sorted_entries.len();
    while cursor > 0 && windows.len() < policy.stability_window_count {
        let start = cursor.saturating_sub(policy.stability_window_runs);
        windows.push(sorted_entries[start..cursor].to_vec());
        cursor = start;
    }
    windows.reverse();
    if windows.is_empty() {
        return Vec::new();
    }

    let mut aggregated = BTreeMap::<(String, String), Vec<f64>>::new();
    for window in &windows {
        let records = build_pairwise_correlation_records(window, metadata);
        for record in records {
            if record.shared_event_count >= policy.min_shared_events {
                aggregated
                    .entry((record.verifier_a.clone(), record.verifier_b.clone()))
                    .or_default()
                    .push(record.pairwise_verdict_correlation);
            }
        }
    }

    let threshold = policy
        .stability_correlation_threshold
        .unwrap_or(policy.pairwise_correlation_threshold);
    let mut records = Vec::new();
    for ((verifier_a, verifier_b), correlations) in aggregated {
        if correlations.is_empty() {
            continue;
        }
        let high_window_count = correlations
            .iter()
            .filter(|value| **value >= threshold)
            .count();
        let max_window_correlation = correlations
            .iter()
            .fold(0.0f64, |acc, value| acc.max(*value));
        let min_window_correlation = correlations
            .iter()
            .fold(1.0f64, |acc, value| acc.min(*value));
        records.push(StabilityRecord {
            verifier_a,
            verifier_b,
            high_window_count,
            evaluated_window_count: correlations.len(),
            max_window_correlation,
            min_window_correlation,
            sustained_high_correlation: high_window_count >= policy.stability_min_high_windows,
        });
    }
    records.sort_by(|left, right| {
        right
            .high_window_count
            .cmp(&left.high_window_count)
            .then_with(|| {
                right
                    .max_window_correlation
                    .total_cmp(&left.max_window_correlation)
            })
            .then_with(|| left.verifier_a.cmp(&right.verifier_a))
            .then_with(|| left.verifier_b.cmp(&right.verifier_b))
    });
    records
}

fn build_metrics(
    selected_entry_count: usize,
    unique_verifier_count: usize,
    pairwise_records: &[PairwiseCorrelationRecord],
    lineage_records: &[PairwiseCorrelationRecord],
    authority_records: &[PairwiseCorrelationRecord],
    cluster_overlap: &[ClusterOverlapRecord],
    stability_records: &[StabilityRecord],
    policy: &CartelCorrelationPolicy,
) -> CartelCorrelationMetrics {
    let suspicious_pairwise_pair_count = pairwise_records
        .iter()
        .filter(|record| {
            record.shared_event_count >= policy.min_shared_events
                && record.pairwise_verdict_correlation >= policy.pairwise_correlation_threshold
        })
        .count();
    let suspicious_lineage_pair_count = lineage_records
        .iter()
        .filter(|record| {
            record.shared_event_count >= policy.min_shared_events
                && record.pairwise_verdict_correlation
                    >= policy.lineage_conditioned_correlation_threshold
        })
        .count();
    let suspicious_authority_pair_count = authority_records
        .iter()
        .filter(|record| {
            record.shared_event_count >= policy.min_shared_events
                && record.pairwise_verdict_correlation
                    >= policy.authority_chain_conditioned_correlation_threshold
        })
        .count();
    let suspicious_stability_pair_count = stability_records
        .iter()
        .filter(|record| record.sustained_high_correlation)
        .count();
    let max_execution_cluster_overlap_ratio = cluster_overlap.first().map(|record| record.share);

    CartelCorrelationMetrics {
        selected_entry_count,
        unique_verifier_count,
        pairwise_pair_count: pairwise_records.len(),
        max_pairwise_correlation: pairwise_records
            .first()
            .map(|record| record.pairwise_verdict_correlation)
            .unwrap_or(0.0),
        suspicious_pairwise_pair_count,
        suspicious_lineage_pair_count,
        suspicious_authority_pair_count,
        max_execution_cluster_overlap_ratio,
        suspicious_execution_cluster_overlap: max_execution_cluster_overlap_ratio
            .map(|value| value > policy.max_execution_cluster_overlap_ratio)
            .unwrap_or(false),
        suspicious_stability_pair_count,
    }
}

fn evaluate_policy(
    selection: &WindowSelection,
    pairwise_records: &[PairwiseCorrelationRecord],
    lineage_records: &[PairwiseCorrelationRecord],
    authority_records: &[PairwiseCorrelationRecord],
    cluster_overlap: &[ClusterOverlapRecord],
    stability_records: &[StabilityRecord],
    policy: &CartelCorrelationPolicy,
) -> Vec<String> {
    let mut violations = Vec::new();
    if selection.selected_entry_count == 0 {
        let reason = selection.empty_reason.unwrap_or("empty_window");
        violations.push(format!("cartel_correlation_violation:{reason}"));
        return violations;
    }

    for record in pairwise_records {
        if record.shared_event_count >= policy.min_shared_events
            && record.pairwise_verdict_correlation >= policy.pairwise_correlation_threshold
        {
            violations.push(format!(
                "cartel_correlation_violation:pairwise:{}:{}:actual={:.6}:threshold={:.6}:shared_events={}",
                record.verifier_a,
                record.verifier_b,
                record.pairwise_verdict_correlation,
                policy.pairwise_correlation_threshold,
                record.shared_event_count
            ));
        }
    }
    for record in lineage_records {
        if record.shared_event_count >= policy.min_shared_events
            && record.pairwise_verdict_correlation
                >= policy.lineage_conditioned_correlation_threshold
        {
            violations.push(format!(
                "cartel_correlation_violation:lineage:{}:{}:{}:actual={:.6}:threshold={:.6}:shared_events={}",
                record.lineage_id.as_deref().unwrap_or("unknown"),
                record.verifier_a,
                record.verifier_b,
                record.pairwise_verdict_correlation,
                policy.lineage_conditioned_correlation_threshold,
                record.shared_event_count
            ));
        }
    }
    for record in authority_records {
        if record.shared_event_count >= policy.min_shared_events
            && record.pairwise_verdict_correlation
                >= policy.authority_chain_conditioned_correlation_threshold
        {
            violations.push(format!(
                "cartel_correlation_violation:authority_chain:{}:{}:{}:actual={:.6}:threshold={:.6}:shared_events={}",
                record.authority_chain_id.as_deref().unwrap_or("unknown"),
                record.verifier_a,
                record.verifier_b,
                record.pairwise_verdict_correlation,
                policy.authority_chain_conditioned_correlation_threshold,
                record.shared_event_count
            ));
        }
    }
    if let Some(record) = cluster_overlap.first() {
        if record.share > policy.max_execution_cluster_overlap_ratio {
            violations.push(format!(
                "cartel_correlation_violation:execution_cluster_overlap:{}:actual={:.6}:max={:.6}:verifier_count={}",
                record.execution_cluster_id,
                record.share,
                policy.max_execution_cluster_overlap_ratio,
                record.verifier_count
            ));
        }
    }
    for record in stability_records {
        if record.sustained_high_correlation {
            violations.push(format!(
                "cartel_correlation_violation:correlation_stability:{}:{}:high_windows={}:min_required={}:max_window_correlation={:.6}",
                record.verifier_a,
                record.verifier_b,
                record.high_window_count,
                policy.stability_min_high_windows,
                record.max_window_correlation
            ));
        }
    }
    violations
}

fn write_outputs(
    config: &CartelCorrelationGateConfig,
    selection: &WindowSelection,
    policy: &CartelCorrelationPolicy,
    metrics: &CartelCorrelationMetrics,
    pairwise_records: &[PairwiseCorrelationRecord],
    lineage_records: &[PairwiseCorrelationRecord],
    authority_records: &[PairwiseCorrelationRecord],
    cluster_overlap: &[ClusterOverlapRecord],
    stability_records: &[StabilityRecord],
    violations: &[String],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create output dir {}: {error}",
            config.output_dir.display()
        )
    })?;
    let verdict = if violations.is_empty() {
        GateVerdict::Pass
    } else {
        GateVerdict::Fail
    };

    write_json(
        &config.output_dir.join("cartel_correlation_metrics.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "window_model": "dual_window",
            "window_counts": {
                "total_entry_count": selection.total_entry_count,
                "post_time_filter_entry_count": selection.post_time_filter_entry_count,
                "post_run_limit_entry_count": selection.post_run_limit_entry_count,
                "selected_entry_count": selection.selected_entry_count,
            },
            "metrics": metrics,
        }),
    )?;
    write_json(
        &config.output_dir.join("pairwise_correlation_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "threshold": policy.pairwise_correlation_threshold,
            "min_shared_events": policy.min_shared_events,
            "pairs": pairwise_records,
        }),
    )?;
    write_json(
        &config.output_dir.join("lineage_correlation_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "threshold": policy.lineage_conditioned_correlation_threshold,
            "min_shared_events": policy.min_shared_events,
            "pairs": lineage_records,
        }),
    )?;
    write_json(
        &config
            .output_dir
            .join("authority_chain_correlation_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "threshold": policy.authority_chain_conditioned_correlation_threshold,
            "min_shared_events": policy.min_shared_events,
            "pairs": authority_records,
        }),
    )?;
    write_json(
        &config.output_dir.join("cluster_overlap_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "max_execution_cluster_overlap_ratio": policy.max_execution_cluster_overlap_ratio,
            "clusters": cluster_overlap,
        }),
    )?;
    write_json(
        &config.output_dir.join("correlation_stability_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "stability_window_runs": policy.stability_window_runs,
            "stability_window_count": policy.stability_window_count,
            "stability_min_high_windows": policy.stability_min_high_windows,
            "stability_correlation_threshold": policy
                .stability_correlation_threshold
                .unwrap_or(policy.pairwise_correlation_threshold),
            "pairs": stability_records,
        }),
    )?;
    write_json(
        &config
            .output_dir
            .join("verifier_cartel_correlation_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "mode": "phase13_verifier_cartel_correlation_gate",
            "risk_class": "cartel-formation-drift",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "window_model": "dual_window",
            "applied_window_runs": selection.applied_window_runs,
            "applied_window_seconds": selection.applied_window_seconds,
            "reference_timestamp_unix_ns": selection.reference_timestamp_unix_ns,
            "empty_reason": selection.empty_reason,
            "window_counts": {
                "total_entry_count": selection.total_entry_count,
                "post_time_filter_entry_count": selection.post_time_filter_entry_count,
                "post_run_limit_entry_count": selection.post_run_limit_entry_count,
                "selected_entry_count": selection.selected_entry_count,
            },
            "policy": policy,
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "gate": "verifier-cartel-correlation",
            "mode": "phase13_verifier_cartel_correlation_gate",
            "verdict": verdict.as_str(),
            "detail_report_path": "verifier_cartel_correlation_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_loading_failure_outputs(
    config: &CartelCorrelationGateConfig,
    violations: &[String],
    load_error: &str,
    load_failure_stage: &str,
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
            .join("verifier_cartel_correlation_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_verifier_cartel_correlation_gate",
            "risk_class": "cartel-formation-drift",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "window_model": "dual_window",
            "applied_window_runs": config.window_runs_override,
            "applied_window_seconds": config.window_seconds_override,
            "reference_timestamp_unix_ns": serde_json::Value::Null,
            "empty_reason": "load_failure",
            "window_counts": {
                "total_entry_count": 0,
                "post_time_filter_entry_count": 0,
                "post_run_limit_entry_count": 0,
                "selected_entry_count": 0,
            },
            "load_failure_stage": load_failure_stage,
            "load_error": load_error,
            "policy": serde_json::Value::Null,
            "metrics": {
                "selected_entry_count": 0,
                "unique_verifier_count": 0,
                "pairwise_pair_count": 0,
                "max_pairwise_correlation": 0.0,
                "suspicious_pairwise_pair_count": 0,
                "suspicious_lineage_pair_count": 0,
                "suspicious_authority_pair_count": 0,
                "max_execution_cluster_overlap_ratio": serde_json::Value::Null,
                "suspicious_execution_cluster_overlap": false,
                "suspicious_stability_pair_count": 0
            },
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "gate": "verifier-cartel-correlation",
            "mode": "phase13_verifier_cartel_correlation_gate",
            "verdict": "FAIL",
            "detail_report_path": "verifier_cartel_correlation_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("cartel_correlation_metrics.json"),
        &serde_json::json!({
            "status": "FAIL",
            "window_model": "dual_window",
            "window_counts": {
                "total_entry_count": 0,
                "post_time_filter_entry_count": 0,
                "post_run_limit_entry_count": 0,
                "selected_entry_count": 0,
            },
            "metrics": {
                "selected_entry_count": 0,
                "unique_verifier_count": 0,
                "pairwise_pair_count": 0,
                "max_pairwise_correlation": 0.0,
                "suspicious_pairwise_pair_count": 0,
                "suspicious_lineage_pair_count": 0,
                "suspicious_authority_pair_count": 0,
                "max_execution_cluster_overlap_ratio": serde_json::Value::Null,
                "suspicious_execution_cluster_overlap": false,
                "suspicious_stability_pair_count": 0
            },
        }),
    )?;
    write_json(
        &config.output_dir.join("pairwise_correlation_report.json"),
        &serde_json::json!({"status": "FAIL", "pairs": []}),
    )?;
    write_json(
        &config.output_dir.join("lineage_correlation_report.json"),
        &serde_json::json!({"status": "FAIL", "pairs": []}),
    )?;
    write_json(
        &config
            .output_dir
            .join("authority_chain_correlation_report.json"),
        &serde_json::json!({"status": "FAIL", "pairs": []}),
    )?;
    write_json(
        &config.output_dir.join("cluster_overlap_report.json"),
        &serde_json::json!({"status": "FAIL", "clusters": []}),
    )?;
    write_json(
        &config.output_dir.join("correlation_stability_report.json"),
        &serde_json::json!({"status": "FAIL", "pairs": []}),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(
        timestamp_unix_ns: u64,
        subject_bundle_id: &str,
        verifier_id: &str,
        lineage_id: &str,
        authority_chain_id: &str,
        execution_cluster_id: Option<&str>,
        verdict: &str,
    ) -> VerificationDiversityLedgerEntry {
        VerificationDiversityLedgerEntry {
            ledger_version: 1,
            entry_id: format!("entry-{timestamp_unix_ns}-{verifier_id}"),
            run_id: format!("run-{timestamp_unix_ns}-{verifier_id}"),
            timestamp_unix_ns,
            subject_bundle_id: subject_bundle_id.to_string(),
            verification_context_id: "context-a".to_string(),
            verification_node_id: format!("node-{verifier_id}"),
            verifier_id: verifier_id.to_string(),
            authority_chain_id: authority_chain_id.to_string(),
            lineage_id: lineage_id.to_string(),
            execution_cluster_id: execution_cluster_id.map(ToString::to_string),
            verdict: verdict.to_string(),
            receipt_hash: format!("receipt-{timestamp_unix_ns}-{verifier_id}"),
        }
    }

    #[test]
    fn pairwise_records_capture_shared_event_agreement() {
        let entries = vec![
            sample_entry(
                1,
                "bundle-1",
                "verifier-a",
                "lineage-a",
                "chain-a",
                None,
                "PASS",
            ),
            sample_entry(
                2,
                "bundle-1",
                "verifier-b",
                "lineage-a",
                "chain-a",
                None,
                "PASS",
            ),
            sample_entry(
                3,
                "bundle-2",
                "verifier-a",
                "lineage-a",
                "chain-a",
                None,
                "FAIL",
            ),
            sample_entry(
                4,
                "bundle-2",
                "verifier-b",
                "lineage-a",
                "chain-a",
                None,
                "FAIL",
            ),
            sample_entry(
                5,
                "bundle-3",
                "verifier-a",
                "lineage-a",
                "chain-a",
                None,
                "PASS",
            ),
            sample_entry(
                6,
                "bundle-3",
                "verifier-b",
                "lineage-a",
                "chain-a",
                None,
                "FAIL",
            ),
        ];
        let metadata = derive_verifier_metadata(&entries);

        let records = build_pairwise_correlation_records(&entries, &metadata);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].shared_event_count, 3);
        assert_eq!(records[0].agreement_count, 2);
        assert!((records[0].pairwise_verdict_correlation - (2.0 / 3.0)).abs() < 0.000_001);
        assert_eq!(records[0].lineage_id.as_deref(), Some("lineage-a"));
        assert_eq!(records[0].authority_chain_id.as_deref(), Some("chain-a"));
    }

    #[test]
    fn cluster_overlap_uses_unique_verifier_share() {
        let entries = vec![
            sample_entry(
                1,
                "bundle-1",
                "verifier-a",
                "lineage-a",
                "chain-a",
                Some("cluster-1"),
                "PASS",
            ),
            sample_entry(
                2,
                "bundle-1",
                "verifier-b",
                "lineage-b",
                "chain-b",
                Some("cluster-1"),
                "PASS",
            ),
            sample_entry(
                3,
                "bundle-1",
                "verifier-c",
                "lineage-c",
                "chain-c",
                Some("cluster-2"),
                "PASS",
            ),
        ];
        let metadata = derive_verifier_metadata(&entries);

        let overlap = compute_execution_cluster_overlap(&metadata);

        assert_eq!(overlap.len(), 2);
        assert_eq!(overlap[0].execution_cluster_id, "cluster-1");
        assert!((overlap[0].share - (2.0 / 3.0)).abs() < 0.000_001);
    }

    #[test]
    fn stability_records_detect_sustained_high_correlation() {
        let mut entries = Vec::new();
        for offset in 0..6u64 {
            let bundle = format!("bundle-{offset}");
            entries.push(sample_entry(
                offset * 10 + 1,
                &bundle,
                "verifier-a",
                "lineage-a",
                "chain-a",
                None,
                if offset % 2 == 0 { "PASS" } else { "FAIL" },
            ));
            entries.push(sample_entry(
                offset * 10 + 2,
                &bundle,
                "verifier-b",
                "lineage-b",
                "chain-b",
                None,
                if offset % 2 == 0 { "PASS" } else { "FAIL" },
            ));
        }
        let metadata = derive_verifier_metadata(&entries);
        let policy = CartelCorrelationPolicy {
            policy_version: 1,
            window_runs: Some(20),
            window_seconds: Some(60),
            min_shared_events: 2,
            pairwise_correlation_threshold: 0.98,
            lineage_conditioned_correlation_threshold: 0.98,
            authority_chain_conditioned_correlation_threshold: 0.98,
            max_execution_cluster_overlap_ratio: 0.8,
            stability_window_runs: 4,
            stability_window_count: 3,
            stability_min_high_windows: 3,
            stability_correlation_threshold: Some(0.98),
        };

        let records = compute_stability_records(&entries, &metadata, &policy);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].high_window_count, 3);
        assert!(records[0].sustained_high_correlation);
    }
}
