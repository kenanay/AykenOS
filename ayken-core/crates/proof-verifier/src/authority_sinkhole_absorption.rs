use crate::authority_sinkhole_companion_flow::{
    load_companion_flow_report, AuthoritySinkholeCompanionFlowEvent,
    AuthoritySinkholeCompanionFlowReport,
};
use crate::diversity_floor::{DistributionEntry, GateVerdict};
use crate::diversity_ledger::{load_diversity_ledger_entries, VerificationDiversityLedgerEntry};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const NANOS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Debug, Clone)]
pub struct AuthoritySinkholeAbsorptionGateConfig {
    pub ledger_path: PathBuf,
    pub policy_path: PathBuf,
    pub replay_boundary_flow_path: PathBuf,
    pub trust_reuse_flow_path: PathBuf,
    pub output_dir: PathBuf,
    pub window_runs_override: Option<usize>,
    pub window_seconds_override: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AuthoritySinkholePolicy {
    pub policy_version: u32,
    #[serde(default)]
    pub window_runs: Option<usize>,
    #[serde(default)]
    pub window_seconds: Option<u64>,
    pub min_repeated_subject_groups: usize,
    pub max_authority_basin_share: f64,
    pub max_authority_basin_reuse_ratio: f64,
    pub max_authority_basin_repeat_capture_rate: f64,
    pub min_alternate_path_decay_ratio: f64,
    pub series_window_runs: usize,
    pub series_window_count: usize,
    pub max_basin_dominance_slope: f64,
    #[serde(default)]
    pub max_replay_boundary_basin_capture_ratio: Option<f64>,
    #[serde(default)]
    pub max_replay_boundary_repeat_capture_rate: Option<f64>,
    #[serde(default)]
    pub max_trust_reuse_basin_capture_ratio: Option<f64>,
    #[serde(default)]
    pub max_trust_reuse_repeat_capture_rate: Option<f64>,
    #[serde(default)]
    pub max_cross_surface_basin_alignment_ratio: Option<f64>,
    #[serde(default)]
    pub min_cross_surface_alternate_path_decay_ratio: Option<f64>,
    #[serde(default)]
    pub max_basin_alignment_slope: Option<f64>,
}

#[derive(Debug)]
pub struct AuthoritySinkholeAbsorptionGateOutcome {
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
struct SubjectKey {
    subject_bundle_id: String,
    verification_context_id: String,
}

#[derive(Debug, Clone)]
struct SubjectGroup {
    key: SubjectKey,
    entries: Vec<VerificationDiversityLedgerEntry>,
}

impl SubjectGroup {
    fn event_count(&self) -> usize {
        self.entries.len()
    }

    fn unique_authority_chain_count(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.authority_chain_id.clone())
            .collect::<BTreeSet<_>>()
            .len()
    }

    fn terminal_authority_chain_id(&self) -> Option<&str> {
        self.entries
            .last()
            .map(|entry| entry.authority_chain_id.as_str())
    }

    fn authority_chain_sequence(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| entry.authority_chain_id.clone())
            .collect()
    }

    fn reference_basin_event_count(&self, reference_basin_id: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.authority_chain_id == reference_basin_id)
            .count()
    }

    fn reference_basin_terminal(&self, reference_basin_id: &str) -> bool {
        self.terminal_authority_chain_id() == Some(reference_basin_id)
    }

    fn reference_basin_repeat_capture(&self, reference_basin_id: &str) -> bool {
        self.reference_basin_terminal(reference_basin_id)
            && self.reference_basin_event_count(reference_basin_id) >= 2
    }

    fn retains_alternate_terminal_path(&self, reference_basin_id: &str) -> bool {
        self.unique_authority_chain_count() > 1
            && !self.reference_basin_terminal(reference_basin_id)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct AuthorityChainFlowRecord {
    authority_chain_id: String,
    entry_count: usize,
    entry_share: f64,
    terminal_subject_count: usize,
    terminal_subject_share: f64,
    repeated_terminal_subject_count: usize,
    repeated_terminal_subject_share: f64,
    repeat_capture_subject_count: usize,
    repeat_capture_subject_share: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct SubjectFlowRecord {
    subject_bundle_id: String,
    verification_context_id: String,
    event_count: usize,
    unique_authority_chain_count: usize,
    authority_chain_sequence: Vec<String>,
    terminal_authority_chain_id: Option<String>,
    reference_basin_terminal: bool,
    reference_basin_event_count: usize,
    reference_basin_repeat_capture: bool,
    retains_alternate_terminal_path: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct BasinWindowRecord {
    window_index: usize,
    selected_entry_count: usize,
    unique_authority_chain_count: usize,
    start_timestamp_unix_ns: Option<u64>,
    end_timestamp_unix_ns: Option<u64>,
    dominant_authority_chain_id: Option<String>,
    dominant_authority_chain_share: f64,
    reference_basin_authority_chain_id: Option<String>,
    reference_basin_share: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct CrossSurfaceWindowRecord {
    window_index: usize,
    start_timestamp_unix_ns: Option<u64>,
    end_timestamp_unix_ns: Option<u64>,
    verification_terminal_subject_count: usize,
    replay_boundary_terminal_subject_count: usize,
    trust_reuse_terminal_subject_count: usize,
    cross_surface_comparable_subject_count: usize,
    cross_surface_basin_alignment_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct AuthoritySinkholeMetrics {
    selected_entry_count: usize,
    unique_authority_chain_count: usize,
    subject_group_count: usize,
    repeated_subject_group_count: usize,
    alternate_path_subject_group_count: usize,
    reference_basin_authority_chain_id: Option<String>,
    authority_basin_share: f64,
    authority_basin_reuse_ratio: Option<f64>,
    authority_basin_repeat_capture_rate: Option<f64>,
    alternate_path_decay_ratio: Option<f64>,
    basin_dominance_slope: Option<f64>,
    evaluated_series_window_count: usize,
    reference_basin_first_window_share: Option<f64>,
    reference_basin_last_window_share: Option<f64>,
    monotonic_non_decreasing: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct CrossSurfaceAlignmentMetrics {
    replay_boundary_report_present: bool,
    trust_reuse_report_present: bool,
    stage2_evaluated: bool,
    verification_terminal_subject_count: usize,
    replay_boundary_event_count: usize,
    replay_boundary_terminal_event_count: usize,
    replay_boundary_reused_terminal_event_count: usize,
    trust_reuse_event_count: usize,
    trust_reuse_terminal_event_count: usize,
    trust_reuse_reused_terminal_event_count: usize,
    replay_boundary_basin_capture_ratio: Option<f64>,
    replay_boundary_repeat_capture_rate: Option<f64>,
    trust_reuse_basin_capture_ratio: Option<f64>,
    trust_reuse_repeat_capture_rate: Option<f64>,
    verification_replay_comparable_subject_count: usize,
    verification_trust_comparable_subject_count: usize,
    replay_trust_comparable_subject_count: usize,
    cross_surface_comparable_subject_count: usize,
    cross_surface_alternate_candidate_count: usize,
    verification_replay_alignment_ratio: Option<f64>,
    verification_trust_alignment_ratio: Option<f64>,
    replay_trust_alignment_ratio: Option<f64>,
    cross_surface_basin_alignment_ratio: Option<f64>,
    cross_surface_alternate_path_decay_ratio: Option<f64>,
    basin_alignment_slope: Option<f64>,
    evaluated_alignment_window_count: usize,
    alignment_first_window_ratio: Option<f64>,
    alignment_last_window_ratio: Option<f64>,
}

pub fn run_authority_sinkhole_absorption_gate(
    config: &AuthoritySinkholeAbsorptionGateConfig,
) -> Result<AuthoritySinkholeAbsorptionGateOutcome, String> {
    let entries = match load_ledger_entries(&config.ledger_path) {
        Ok(entries) => entries,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_ledger:{}",
                config.ledger_path.display()
            )];
            write_loading_failure_outputs(config, &violations, &error, "ledger_load")?;
            return Ok(AuthoritySinkholeAbsorptionGateOutcome {
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
            return Ok(AuthoritySinkholeAbsorptionGateOutcome {
                verdict: GateVerdict::Fail,
                violations,
            });
        }
    };
    let replay_boundary_report = match load_optional_companion_report(
        &config.replay_boundary_flow_path,
        "replay_boundary",
    ) {
        Ok(report) => report,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_replay_boundary_flow:{}",
                config.replay_boundary_flow_path.display()
            )];
            write_loading_failure_outputs(
                config,
                &violations,
                &error,
                "replay_boundary_flow_load",
            )?;
            return Ok(AuthoritySinkholeAbsorptionGateOutcome {
                verdict: GateVerdict::Fail,
                violations,
            });
        }
    };
    let trust_reuse_report =
        match load_optional_companion_report(&config.trust_reuse_flow_path, "trust_reuse") {
            Ok(report) => report,
            Err(error) => {
                let violations = vec![format!(
                    "missing_or_invalid_trust_reuse_flow:{}",
                    config.trust_reuse_flow_path.display()
                )];
                write_loading_failure_outputs(
                    config,
                    &violations,
                    &error,
                    "trust_reuse_flow_load",
                )?;
                return Ok(AuthoritySinkholeAbsorptionGateOutcome {
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
    let authority_distribution =
        build_distribution(&selection.entries, |entry| entry.authority_chain_id.clone());
    let reference_basin_id = authority_distribution.first().map(|entry| entry.id.clone());
    let subject_groups = build_subject_groups(&selection.entries);
    let subject_flow_records =
        build_subject_flow_records(&subject_groups, reference_basin_id.as_deref());
    let authority_flow_records =
        build_authority_chain_flow_records(&authority_distribution, &subject_groups);
    let basin_window_series = build_basin_window_series(
        &selection.entries,
        reference_basin_id.as_deref(),
        policy.series_window_runs,
        policy.series_window_count,
    );
    let metrics = build_metrics(
        selection.selected_entry_count,
        &authority_distribution,
        &subject_groups,
        &basin_window_series,
        reference_basin_id.as_deref(),
    );
    let cross_surface_window_series = build_cross_surface_window_series(
        &selection.entries,
        &basin_window_series,
        replay_boundary_report.as_ref(),
        trust_reuse_report.as_ref(),
        reference_basin_id.as_deref(),
    );
    let cross_surface_metrics = build_cross_surface_metrics(
        &subject_groups,
        replay_boundary_report.as_ref(),
        trust_reuse_report.as_ref(),
        &cross_surface_window_series,
        reference_basin_id.as_deref(),
    );
    let violations = evaluate_policy(
        &selection,
        &metrics,
        &cross_surface_metrics,
        replay_boundary_report.as_ref(),
        trust_reuse_report.as_ref(),
        &policy,
    );

    write_outputs(
        config,
        &selection,
        &policy,
        &metrics,
        &cross_surface_metrics,
        &authority_distribution,
        &authority_flow_records,
        &subject_flow_records,
        &basin_window_series,
        &cross_surface_window_series,
        &violations,
    )?;

    Ok(AuthoritySinkholeAbsorptionGateOutcome {
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

fn load_policy(path: &Path) -> Result<AuthoritySinkholePolicy, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read policy at {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse policy at {}: {error}", path.display()))
}

fn load_optional_companion_report(
    path: &Path,
    expected_surface: &str,
) -> Result<Option<AuthoritySinkholeCompanionFlowReport>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let report = load_companion_flow_report(path)?;
    if report.flow_surface != expected_surface {
        return Err(format!(
            "companion_flow_surface_mismatch:{}:{}",
            report.flow_surface, expected_surface
        ));
    }
    if report.status == "NOT_EVALUATED" {
        return Ok(None);
    }
    if report.status != "PASS" {
        return Err(format!(
            "companion_flow_status_not_pass:{}:{}",
            expected_surface, report.status
        ));
    }
    Ok(Some(report))
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

fn build_subject_groups(entries: &[VerificationDiversityLedgerEntry]) -> Vec<SubjectGroup> {
    let mut grouped = BTreeMap::<SubjectKey, Vec<VerificationDiversityLedgerEntry>>::new();
    for entry in entries {
        grouped
            .entry(SubjectKey {
                subject_bundle_id: entry.subject_bundle_id.clone(),
                verification_context_id: entry.verification_context_id.clone(),
            })
            .or_default()
            .push(entry.clone());
    }

    grouped
        .into_iter()
        .map(|(key, mut group_entries)| {
            group_entries.sort_by(|left, right| {
                left.timestamp_unix_ns
                    .cmp(&right.timestamp_unix_ns)
                    .then_with(|| left.entry_id.cmp(&right.entry_id))
            });
            SubjectGroup {
                key,
                entries: group_entries,
            }
        })
        .collect()
}

fn build_subject_flow_records(
    subject_groups: &[SubjectGroup],
    reference_basin_id: Option<&str>,
) -> Vec<SubjectFlowRecord> {
    let mut records: Vec<SubjectFlowRecord> = subject_groups
        .iter()
        .map(|group| {
            let reference_basin_event_count = reference_basin_id
                .map(|value| group.reference_basin_event_count(value))
                .unwrap_or(0);
            let reference_basin_terminal = reference_basin_id
                .map(|value| group.reference_basin_terminal(value))
                .unwrap_or(false);
            let reference_basin_repeat_capture = reference_basin_id
                .map(|value| group.reference_basin_repeat_capture(value))
                .unwrap_or(false);
            let retains_alternate_terminal_path = reference_basin_id
                .map(|value| group.retains_alternate_terminal_path(value))
                .unwrap_or(false);
            SubjectFlowRecord {
                subject_bundle_id: group.key.subject_bundle_id.clone(),
                verification_context_id: group.key.verification_context_id.clone(),
                event_count: group.event_count(),
                unique_authority_chain_count: group.unique_authority_chain_count(),
                authority_chain_sequence: group.authority_chain_sequence(),
                terminal_authority_chain_id: group
                    .terminal_authority_chain_id()
                    .map(str::to_string),
                reference_basin_terminal,
                reference_basin_event_count,
                reference_basin_repeat_capture,
                retains_alternate_terminal_path,
            }
        })
        .collect();
    records.sort_by(|left, right| {
        right
            .event_count
            .cmp(&left.event_count)
            .then_with(|| left.subject_bundle_id.cmp(&right.subject_bundle_id))
            .then_with(|| {
                left.verification_context_id
                    .cmp(&right.verification_context_id)
            })
    });
    records
}

fn build_authority_chain_flow_records(
    authority_distribution: &[DistributionEntry],
    subject_groups: &[SubjectGroup],
) -> Vec<AuthorityChainFlowRecord> {
    let total_subject_groups = subject_groups.len() as f64;
    let repeated_subject_groups: Vec<&SubjectGroup> = subject_groups
        .iter()
        .filter(|group| group.event_count() >= 2)
        .collect();
    let total_repeated_subject_groups = repeated_subject_groups.len() as f64;

    authority_distribution
        .iter()
        .map(|distribution_entry| {
            let authority_chain_id = distribution_entry.id.clone();
            let terminal_subject_count = subject_groups
                .iter()
                .filter(|group| {
                    group.terminal_authority_chain_id() == Some(authority_chain_id.as_str())
                })
                .count();
            let repeated_terminal_subject_count = repeated_subject_groups
                .iter()
                .filter(|group| {
                    group.terminal_authority_chain_id() == Some(authority_chain_id.as_str())
                })
                .count();
            let repeat_capture_subject_count = repeated_subject_groups
                .iter()
                .filter(|group| group.reference_basin_repeat_capture(authority_chain_id.as_str()))
                .count();
            AuthorityChainFlowRecord {
                authority_chain_id,
                entry_count: distribution_entry.count,
                entry_share: distribution_entry.share,
                terminal_subject_count,
                terminal_subject_share: if total_subject_groups == 0.0 {
                    0.0
                } else {
                    terminal_subject_count as f64 / total_subject_groups
                },
                repeated_terminal_subject_count,
                repeated_terminal_subject_share: if total_repeated_subject_groups == 0.0 {
                    0.0
                } else {
                    repeated_terminal_subject_count as f64 / total_repeated_subject_groups
                },
                repeat_capture_subject_count,
                repeat_capture_subject_share: if total_repeated_subject_groups == 0.0 {
                    0.0
                } else {
                    repeat_capture_subject_count as f64 / total_repeated_subject_groups
                },
            }
        })
        .collect()
}

fn build_basin_window_series(
    entries: &[VerificationDiversityLedgerEntry],
    reference_basin_id: Option<&str>,
    series_window_runs: usize,
    series_window_count: usize,
) -> Vec<BasinWindowRecord> {
    if series_window_runs == 0 || series_window_count == 0 || entries.is_empty() {
        return Vec::new();
    }

    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by(|left, right| {
        left.timestamp_unix_ns
            .cmp(&right.timestamp_unix_ns)
            .then_with(|| left.entry_id.cmp(&right.entry_id))
    });

    let mut windows = Vec::<Vec<VerificationDiversityLedgerEntry>>::new();
    let mut cursor = sorted_entries.len();
    while cursor > 0 && windows.len() < series_window_count {
        let start = cursor.saturating_sub(series_window_runs);
        windows.push(sorted_entries[start..cursor].to_vec());
        cursor = start;
    }
    windows.reverse();

    windows
        .into_iter()
        .enumerate()
        .map(|(window_index, window_entries)| {
            let authority_distribution =
                build_distribution(&window_entries, |entry| entry.authority_chain_id.clone());
            let dominant = authority_distribution.first();
            let reference_basin_share = reference_basin_id.map_or(0.0, |reference_basin_id| {
                authority_distribution
                    .iter()
                    .find(|entry| entry.id == reference_basin_id)
                    .map(|entry| entry.share)
                    .unwrap_or(0.0)
            });
            BasinWindowRecord {
                window_index: window_index + 1,
                selected_entry_count: window_entries.len(),
                unique_authority_chain_count: authority_distribution.len(),
                start_timestamp_unix_ns: window_entries
                    .first()
                    .map(|entry| entry.timestamp_unix_ns),
                end_timestamp_unix_ns: window_entries.last().map(|entry| entry.timestamp_unix_ns),
                dominant_authority_chain_id: dominant.map(|entry| entry.id.clone()),
                dominant_authority_chain_share: dominant.map(|entry| entry.share).unwrap_or(0.0),
                reference_basin_authority_chain_id: reference_basin_id.map(str::to_string),
                reference_basin_share,
            }
        })
        .collect()
}

fn build_metrics(
    selected_entry_count: usize,
    authority_distribution: &[DistributionEntry],
    subject_groups: &[SubjectGroup],
    basin_window_series: &[BasinWindowRecord],
    reference_basin_id: Option<&str>,
) -> AuthoritySinkholeMetrics {
    let repeated_subject_groups: Vec<&SubjectGroup> = subject_groups
        .iter()
        .filter(|group| group.event_count() >= 2)
        .collect();
    let alternate_path_subject_groups: Vec<&SubjectGroup> = repeated_subject_groups
        .iter()
        .copied()
        .filter(|group| group.unique_authority_chain_count() > 1)
        .collect();

    let authority_basin_share = authority_distribution
        .first()
        .map(|entry| entry.share)
        .unwrap_or(0.0);
    let authority_basin_reuse_ratio = reference_basin_id.and_then(|reference_basin_id| {
        if repeated_subject_groups.is_empty() {
            None
        } else {
            Some(
                repeated_subject_groups
                    .iter()
                    .filter(|group| group.reference_basin_terminal(reference_basin_id))
                    .count() as f64
                    / repeated_subject_groups.len() as f64,
            )
        }
    });
    let authority_basin_repeat_capture_rate = reference_basin_id.and_then(|reference_basin_id| {
        if repeated_subject_groups.is_empty() {
            None
        } else {
            Some(
                repeated_subject_groups
                    .iter()
                    .filter(|group| group.reference_basin_repeat_capture(reference_basin_id))
                    .count() as f64
                    / repeated_subject_groups.len() as f64,
            )
        }
    });
    let alternate_path_decay_ratio = reference_basin_id.and_then(|reference_basin_id| {
        if alternate_path_subject_groups.is_empty() {
            None
        } else {
            Some(
                alternate_path_subject_groups
                    .iter()
                    .filter(|group| group.retains_alternate_terminal_path(reference_basin_id))
                    .count() as f64
                    / alternate_path_subject_groups.len() as f64,
            )
        }
    });

    let basin_dominance_slope = compute_reference_basin_slope(basin_window_series);
    let reference_basin_first_window_share = basin_window_series
        .first()
        .map(|record| record.reference_basin_share);
    let reference_basin_last_window_share = basin_window_series
        .last()
        .map(|record| record.reference_basin_share);
    let monotonic_non_decreasing = basin_window_series
        .windows(2)
        .all(|pair| pair[0].reference_basin_share <= pair[1].reference_basin_share);

    AuthoritySinkholeMetrics {
        selected_entry_count,
        unique_authority_chain_count: authority_distribution.len(),
        subject_group_count: subject_groups.len(),
        repeated_subject_group_count: repeated_subject_groups.len(),
        alternate_path_subject_group_count: alternate_path_subject_groups.len(),
        reference_basin_authority_chain_id: reference_basin_id.map(str::to_string),
        authority_basin_share,
        authority_basin_reuse_ratio,
        authority_basin_repeat_capture_rate,
        alternate_path_decay_ratio,
        basin_dominance_slope,
        evaluated_series_window_count: basin_window_series.len(),
        reference_basin_first_window_share,
        reference_basin_last_window_share,
        monotonic_non_decreasing,
    }
}

fn compute_reference_basin_slope(basin_window_series: &[BasinWindowRecord]) -> Option<f64> {
    if basin_window_series.len() < 2 {
        return None;
    }
    let first = basin_window_series.first()?.reference_basin_share;
    let last = basin_window_series.last()?.reference_basin_share;
    Some((last - first) / (basin_window_series.len() - 1) as f64)
}

fn build_verification_terminal_map(
    subject_groups: &[SubjectGroup],
) -> BTreeMap<SubjectKey, String> {
    let mut map = BTreeMap::new();
    for group in subject_groups {
        if let Some(terminal_authority_chain_id) = group.terminal_authority_chain_id() {
            map.insert(group.key.clone(), terminal_authority_chain_id.to_string());
        }
    }
    map
}

fn build_companion_terminal_map<'a, I>(
    events: I,
) -> BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>
where
    I: IntoIterator<Item = &'a AuthoritySinkholeCompanionFlowEvent>,
{
    let mut terminal_events: Vec<AuthoritySinkholeCompanionFlowEvent> = events
        .into_iter()
        .filter(|event| event.terminal)
        .cloned()
        .collect();
    terminal_events.sort_by(|left, right| {
        left.timestamp_unix_ns
            .cmp(&right.timestamp_unix_ns)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });

    let mut map = BTreeMap::new();
    for event in terminal_events {
        map.insert(
            SubjectKey {
                subject_bundle_id: event.subject_bundle_id.clone(),
                verification_context_id: event.verification_context_id.clone(),
            },
            event,
        );
    }
    map
}

fn build_cross_surface_window_series(
    entries: &[VerificationDiversityLedgerEntry],
    basin_window_series: &[BasinWindowRecord],
    replay_boundary_report: Option<&AuthoritySinkholeCompanionFlowReport>,
    trust_reuse_report: Option<&AuthoritySinkholeCompanionFlowReport>,
    reference_basin_id: Option<&str>,
) -> Vec<CrossSurfaceWindowRecord> {
    let mut windows = Vec::new();
    for window in basin_window_series {
        let (Some(start), Some(end)) =
            (window.start_timestamp_unix_ns, window.end_timestamp_unix_ns)
        else {
            windows.push(CrossSurfaceWindowRecord {
                window_index: window.window_index,
                start_timestamp_unix_ns: window.start_timestamp_unix_ns,
                end_timestamp_unix_ns: window.end_timestamp_unix_ns,
                verification_terminal_subject_count: 0,
                replay_boundary_terminal_subject_count: 0,
                trust_reuse_terminal_subject_count: 0,
                cross_surface_comparable_subject_count: 0,
                cross_surface_basin_alignment_ratio: None,
            });
            continue;
        };

        let window_entries: Vec<VerificationDiversityLedgerEntry> = entries
            .iter()
            .filter(|entry| entry.timestamp_unix_ns >= start && entry.timestamp_unix_ns <= end)
            .cloned()
            .collect();
        let verification_subject_groups = build_subject_groups(&window_entries);
        let verification_terminal_map =
            build_verification_terminal_map(&verification_subject_groups);
        let replay_terminal_map =
            replay_boundary_report.map_or_else(BTreeMap::new, |report| {
                build_companion_terminal_map(report.events.iter().filter(|event| {
                    event.timestamp_unix_ns >= start && event.timestamp_unix_ns <= end
                }))
            });
        let trust_terminal_map =
            trust_reuse_report.map_or_else(BTreeMap::new, |report| {
                build_companion_terminal_map(report.events.iter().filter(|event| {
                    event.timestamp_unix_ns >= start && event.timestamp_unix_ns <= end
                }))
            });
        let cross_surface_comparable_subject_count = cross_surface_subject_keys(
            &verification_terminal_map,
            &replay_terminal_map,
            &trust_terminal_map,
        )
        .len();
        let cross_surface_basin_alignment_ratio =
            reference_basin_id.and_then(|reference_basin_id| {
                cross_surface_alignment_ratio(
                    &verification_terminal_map,
                    &replay_terminal_map,
                    &trust_terminal_map,
                    reference_basin_id,
                )
            });
        windows.push(CrossSurfaceWindowRecord {
            window_index: window.window_index,
            start_timestamp_unix_ns: Some(start),
            end_timestamp_unix_ns: Some(end),
            verification_terminal_subject_count: verification_terminal_map.len(),
            replay_boundary_terminal_subject_count: replay_terminal_map.len(),
            trust_reuse_terminal_subject_count: trust_terminal_map.len(),
            cross_surface_comparable_subject_count,
            cross_surface_basin_alignment_ratio,
        });
    }
    windows
}

fn build_cross_surface_metrics(
    subject_groups: &[SubjectGroup],
    replay_boundary_report: Option<&AuthoritySinkholeCompanionFlowReport>,
    trust_reuse_report: Option<&AuthoritySinkholeCompanionFlowReport>,
    cross_surface_window_series: &[CrossSurfaceWindowRecord],
    reference_basin_id: Option<&str>,
) -> CrossSurfaceAlignmentMetrics {
    let verification_terminal_map = build_verification_terminal_map(subject_groups);
    let replay_terminal_map = replay_boundary_report.map_or_else(BTreeMap::new, |report| {
        build_companion_terminal_map(report.events.iter())
    });
    let trust_terminal_map = trust_reuse_report.map_or_else(BTreeMap::new, |report| {
        build_companion_terminal_map(report.events.iter())
    });
    let replay_reused_terminal_event_count = replay_terminal_map
        .values()
        .filter(|event| event.reused)
        .count();
    let trust_reuse_reused_terminal_event_count = trust_terminal_map
        .values()
        .filter(|event| event.reused)
        .count();
    let verification_replay_subjects =
        pair_subject_keys(&verification_terminal_map, &replay_terminal_map);
    let verification_trust_subjects =
        pair_subject_keys(&verification_terminal_map, &trust_terminal_map);
    let replay_trust_subjects = pair_subject_keys(&replay_terminal_map, &trust_terminal_map);
    let cross_surface_subjects = cross_surface_subject_keys(
        &verification_terminal_map,
        &replay_terminal_map,
        &trust_terminal_map,
    );
    let cross_surface_alternate_candidate_count = cross_surface_subjects
        .iter()
        .filter(|key| {
            distinct_basin_count([
                verification_terminal_map.get(*key).map(String::as_str),
                replay_terminal_map
                    .get(*key)
                    .map(|event| event.authority_chain_id.as_str()),
                trust_terminal_map
                    .get(*key)
                    .map(|event| event.authority_chain_id.as_str()),
            ]) > 1
        })
        .count();
    let alignment_window_values: Vec<f64> = cross_surface_window_series
        .iter()
        .filter_map(|window| window.cross_surface_basin_alignment_ratio)
        .collect();

    CrossSurfaceAlignmentMetrics {
        replay_boundary_report_present: replay_boundary_report.is_some(),
        trust_reuse_report_present: trust_reuse_report.is_some(),
        stage2_evaluated: replay_boundary_report.is_some()
            && trust_reuse_report.is_some()
            && reference_basin_id.is_some(),
        verification_terminal_subject_count: verification_terminal_map.len(),
        replay_boundary_event_count: replay_boundary_report
            .map(|report| report.event_count)
            .unwrap_or(0),
        replay_boundary_terminal_event_count: replay_terminal_map.len(),
        replay_boundary_reused_terminal_event_count: replay_reused_terminal_event_count,
        trust_reuse_event_count: trust_reuse_report
            .map(|report| report.event_count)
            .unwrap_or(0),
        trust_reuse_terminal_event_count: trust_terminal_map.len(),
        trust_reuse_reused_terminal_event_count,
        replay_boundary_basin_capture_ratio: reference_basin_id.and_then(|reference_basin_id| {
            surface_capture_ratio(&replay_terminal_map, reference_basin_id)
        }),
        replay_boundary_repeat_capture_rate: reference_basin_id.and_then(|reference_basin_id| {
            surface_repeat_capture_rate(&replay_terminal_map, reference_basin_id)
        }),
        trust_reuse_basin_capture_ratio: reference_basin_id.and_then(|reference_basin_id| {
            surface_capture_ratio(&trust_terminal_map, reference_basin_id)
        }),
        trust_reuse_repeat_capture_rate: reference_basin_id.and_then(|reference_basin_id| {
            surface_repeat_capture_rate(&trust_terminal_map, reference_basin_id)
        }),
        verification_replay_comparable_subject_count: verification_replay_subjects.len(),
        verification_trust_comparable_subject_count: verification_trust_subjects.len(),
        replay_trust_comparable_subject_count: replay_trust_subjects.len(),
        cross_surface_comparable_subject_count: cross_surface_subjects.len(),
        cross_surface_alternate_candidate_count,
        verification_replay_alignment_ratio: reference_basin_id.and_then(|reference_basin_id| {
            pair_alignment_ratio(
                &verification_terminal_map,
                &replay_terminal_map,
                reference_basin_id,
            )
        }),
        verification_trust_alignment_ratio: reference_basin_id.and_then(|reference_basin_id| {
            pair_alignment_ratio(
                &verification_terminal_map,
                &trust_terminal_map,
                reference_basin_id,
            )
        }),
        replay_trust_alignment_ratio: reference_basin_id.and_then(|reference_basin_id| {
            pair_alignment_ratio(
                &replay_terminal_map,
                &trust_terminal_map,
                reference_basin_id,
            )
        }),
        cross_surface_basin_alignment_ratio: reference_basin_id.and_then(|reference_basin_id| {
            cross_surface_alignment_ratio(
                &verification_terminal_map,
                &replay_terminal_map,
                &trust_terminal_map,
                reference_basin_id,
            )
        }),
        cross_surface_alternate_path_decay_ratio: reference_basin_id.and_then(
            |reference_basin_id| {
                cross_surface_alternate_path_decay_ratio(
                    &verification_terminal_map,
                    &replay_terminal_map,
                    &trust_terminal_map,
                    reference_basin_id,
                )
            },
        ),
        basin_alignment_slope: if alignment_window_values.len() < 2 {
            None
        } else {
            let first = alignment_window_values[0];
            let last = alignment_window_values[alignment_window_values.len() - 1];
            Some((last - first) / (alignment_window_values.len() - 1) as f64)
        },
        evaluated_alignment_window_count: alignment_window_values.len(),
        alignment_first_window_ratio: alignment_window_values.first().copied(),
        alignment_last_window_ratio: alignment_window_values.last().copied(),
    }
}

fn surface_capture_ratio(
    map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    reference_basin_id: &str,
) -> Option<f64> {
    ratio(
        map.values()
            .filter(|event| event.authority_chain_id == reference_basin_id)
            .count(),
        map.len(),
    )
}

fn surface_repeat_capture_rate(
    map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    reference_basin_id: &str,
) -> Option<f64> {
    let reused_total = map.values().filter(|event| event.reused).count();
    ratio(
        map.values()
            .filter(|event| event.reused && event.authority_chain_id == reference_basin_id)
            .count(),
        reused_total,
    )
}

fn pair_subject_keys<L, R>(
    left: &BTreeMap<SubjectKey, L>,
    right: &BTreeMap<SubjectKey, R>,
) -> BTreeSet<SubjectKey> {
    left.keys()
        .filter(|key| right.contains_key(*key))
        .cloned()
        .collect()
}

fn cross_surface_subject_keys(
    verification_terminal_map: &BTreeMap<SubjectKey, String>,
    replay_terminal_map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    trust_terminal_map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
) -> BTreeSet<SubjectKey> {
    verification_terminal_map
        .keys()
        .filter(|key| {
            replay_terminal_map.contains_key(*key) && trust_terminal_map.contains_key(*key)
        })
        .cloned()
        .collect()
}

fn pair_alignment_ratio<L, R>(
    left: &BTreeMap<SubjectKey, L>,
    right: &BTreeMap<SubjectKey, R>,
    reference_basin_id: &str,
) -> Option<f64>
where
    L: AsAuthorityChainId,
    R: AsAuthorityChainId,
{
    let comparable = pair_subject_keys(left, right);
    ratio(
        comparable
            .iter()
            .filter(|key| {
                left.get(*key)
                    .is_some_and(|value| value.authority_chain_id() == reference_basin_id)
                    && right
                        .get(*key)
                        .is_some_and(|value| value.authority_chain_id() == reference_basin_id)
            })
            .count(),
        comparable.len(),
    )
}

fn cross_surface_alignment_ratio(
    verification_terminal_map: &BTreeMap<SubjectKey, String>,
    replay_terminal_map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    trust_terminal_map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    reference_basin_id: &str,
) -> Option<f64> {
    let comparable = cross_surface_subject_keys(
        verification_terminal_map,
        replay_terminal_map,
        trust_terminal_map,
    );
    ratio(
        comparable
            .iter()
            .filter(|key| {
                verification_terminal_map
                    .get(*key)
                    .is_some_and(|value| value == reference_basin_id)
                    && replay_terminal_map
                        .get(*key)
                        .is_some_and(|value| value.authority_chain_id == reference_basin_id)
                    && trust_terminal_map
                        .get(*key)
                        .is_some_and(|value| value.authority_chain_id == reference_basin_id)
            })
            .count(),
        comparable.len(),
    )
}

fn cross_surface_alternate_path_decay_ratio(
    verification_terminal_map: &BTreeMap<SubjectKey, String>,
    replay_terminal_map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    trust_terminal_map: &BTreeMap<SubjectKey, AuthoritySinkholeCompanionFlowEvent>,
    reference_basin_id: &str,
) -> Option<f64> {
    let comparable = cross_surface_subject_keys(
        verification_terminal_map,
        replay_terminal_map,
        trust_terminal_map,
    );
    let alternate_candidates: Vec<SubjectKey> = comparable
        .into_iter()
        .filter(|key| {
            distinct_basin_count([
                verification_terminal_map.get(key).map(String::as_str),
                replay_terminal_map
                    .get(key)
                    .map(|event| event.authority_chain_id.as_str()),
                trust_terminal_map
                    .get(key)
                    .map(|event| event.authority_chain_id.as_str()),
            ]) > 1
        })
        .collect();
    ratio(
        alternate_candidates
            .iter()
            .filter(|key| {
                verification_terminal_map
                    .get(*key)
                    .is_some_and(|value| value != reference_basin_id)
                    || replay_terminal_map
                        .get(*key)
                        .is_some_and(|event| event.authority_chain_id != reference_basin_id)
                    || trust_terminal_map
                        .get(*key)
                        .is_some_and(|event| event.authority_chain_id != reference_basin_id)
            })
            .count(),
        alternate_candidates.len(),
    )
}

fn distinct_basin_count<'a, I>(values: I) -> usize
where
    I: IntoIterator<Item = Option<&'a str>>,
{
    values.into_iter().flatten().collect::<BTreeSet<_>>().len()
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

trait AsAuthorityChainId {
    fn authority_chain_id(&self) -> &str;
}

impl AsAuthorityChainId for String {
    fn authority_chain_id(&self) -> &str {
        self
    }
}

impl AsAuthorityChainId for AuthoritySinkholeCompanionFlowEvent {
    fn authority_chain_id(&self) -> &str {
        &self.authority_chain_id
    }
}

fn evaluate_policy(
    selection: &WindowSelection,
    metrics: &AuthoritySinkholeMetrics,
    cross_surface_metrics: &CrossSurfaceAlignmentMetrics,
    replay_boundary_report: Option<&AuthoritySinkholeCompanionFlowReport>,
    trust_reuse_report: Option<&AuthoritySinkholeCompanionFlowReport>,
    policy: &AuthoritySinkholePolicy,
) -> Vec<String> {
    let mut violations = Vec::new();
    if selection.selected_entry_count == 0 {
        let reason = selection.empty_reason.unwrap_or("empty_window");
        violations.push(format!("authority_sinkhole_violation:{reason}"));
        return violations;
    }

    if metrics.repeated_subject_group_count < policy.min_repeated_subject_groups {
        violations.push(format!(
            "authority_sinkhole_violation:insufficient_repeated_subject_groups:actual={}:min={}",
            metrics.repeated_subject_group_count, policy.min_repeated_subject_groups
        ));
    }
    if metrics.authority_basin_share > policy.max_authority_basin_share {
        violations.push(format!(
            "authority_sinkhole_violation:authority_basin_share:{}:actual={:.6}:max={:.6}",
            metrics
                .reference_basin_authority_chain_id
                .as_deref()
                .unwrap_or("unknown"),
            metrics.authority_basin_share,
            policy.max_authority_basin_share
        ));
    }
    if let Some(value) = metrics.authority_basin_reuse_ratio {
        if value > policy.max_authority_basin_reuse_ratio {
            violations.push(format!(
                "authority_sinkhole_violation:authority_basin_reuse_ratio:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                policy.max_authority_basin_reuse_ratio
            ));
        }
    }
    if let Some(value) = metrics.authority_basin_repeat_capture_rate {
        if value > policy.max_authority_basin_repeat_capture_rate {
            violations.push(format!(
                "authority_sinkhole_violation:authority_basin_repeat_capture_rate:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                policy.max_authority_basin_repeat_capture_rate
            ));
        }
    }
    if let Some(value) = metrics.alternate_path_decay_ratio {
        if value < policy.min_alternate_path_decay_ratio {
            violations.push(format!(
                "authority_sinkhole_violation:alternate_path_decay_ratio:{}:actual={:.6}:min={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                policy.min_alternate_path_decay_ratio
            ));
        }
    }
    if let Some(value) = metrics.basin_dominance_slope {
        if value > policy.max_basin_dominance_slope {
            violations.push(format!(
                "authority_sinkhole_violation:basin_dominance_slope:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                policy.max_basin_dominance_slope
            ));
        }
    }

    let stage2_requested = policy.max_replay_boundary_basin_capture_ratio.is_some()
        || policy.max_replay_boundary_repeat_capture_rate.is_some()
        || policy.max_trust_reuse_basin_capture_ratio.is_some()
        || policy.max_trust_reuse_repeat_capture_rate.is_some()
        || policy.max_cross_surface_basin_alignment_ratio.is_some()
        || policy
            .min_cross_surface_alternate_path_decay_ratio
            .is_some()
        || policy.max_basin_alignment_slope.is_some();

    if stage2_requested {
        if replay_boundary_report.is_none() {
            violations.push(
                "authority_sinkhole_violation:missing_stage2_companion_flow:replay_boundary"
                    .to_string(),
            );
        }
        if trust_reuse_report.is_none() {
            violations.push(
                "authority_sinkhole_violation:missing_stage2_companion_flow:trust_reuse"
                    .to_string(),
            );
        }
    }

    if let Some(max_value) = policy.max_replay_boundary_basin_capture_ratio {
        match cross_surface_metrics.replay_boundary_basin_capture_ratio {
            Some(value) if value > max_value => violations.push(format!(
                "authority_sinkhole_violation:replay_boundary_basin_capture_ratio:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                max_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:replay_boundary_basin_capture_ratio"
                    .to_string(),
            ),
            _ => {}
        }
    }
    if let Some(max_value) = policy.max_replay_boundary_repeat_capture_rate {
        match cross_surface_metrics.replay_boundary_repeat_capture_rate {
            Some(value) if value > max_value => violations.push(format!(
                "authority_sinkhole_violation:replay_boundary_repeat_capture_rate:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                max_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:replay_boundary_repeat_capture_rate"
                    .to_string(),
            ),
            _ => {}
        }
    }
    if let Some(max_value) = policy.max_trust_reuse_basin_capture_ratio {
        match cross_surface_metrics.trust_reuse_basin_capture_ratio {
            Some(value) if value > max_value => violations.push(format!(
                "authority_sinkhole_violation:trust_reuse_basin_capture_ratio:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                max_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:trust_reuse_basin_capture_ratio"
                    .to_string(),
            ),
            _ => {}
        }
    }
    if let Some(max_value) = policy.max_trust_reuse_repeat_capture_rate {
        match cross_surface_metrics.trust_reuse_repeat_capture_rate {
            Some(value) if value > max_value => violations.push(format!(
                "authority_sinkhole_violation:trust_reuse_repeat_capture_rate:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                max_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:trust_reuse_repeat_capture_rate"
                    .to_string(),
            ),
            _ => {}
        }
    }
    if let Some(max_value) = policy.max_cross_surface_basin_alignment_ratio {
        match cross_surface_metrics.cross_surface_basin_alignment_ratio {
            Some(value) if value > max_value => violations.push(format!(
                "authority_sinkhole_violation:cross_surface_basin_alignment_ratio:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                max_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:cross_surface_basin_alignment_ratio"
                    .to_string(),
            ),
            _ => {}
        }
    }
    if let Some(min_value) = policy.min_cross_surface_alternate_path_decay_ratio {
        match cross_surface_metrics.cross_surface_alternate_path_decay_ratio {
            Some(value) if value < min_value => violations.push(format!(
                "authority_sinkhole_violation:cross_surface_alternate_path_decay_ratio:{}:actual={:.6}:min={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                min_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:cross_surface_alternate_path_decay_ratio"
                    .to_string(),
            ),
            _ => {}
        }
    }
    if let Some(max_value) = policy.max_basin_alignment_slope {
        match cross_surface_metrics.basin_alignment_slope {
            Some(value) if value > max_value => violations.push(format!(
                "authority_sinkhole_violation:basin_alignment_slope:{}:actual={:.6}:max={:.6}",
                metrics
                    .reference_basin_authority_chain_id
                    .as_deref()
                    .unwrap_or("unknown"),
                value,
                max_value
            )),
            None if stage2_requested => violations.push(
                "authority_sinkhole_violation:stage2_metric_not_evaluable:basin_alignment_slope"
                    .to_string(),
            ),
            _ => {}
        }
    }

    violations
}

fn write_outputs(
    config: &AuthoritySinkholeAbsorptionGateConfig,
    selection: &WindowSelection,
    policy: &AuthoritySinkholePolicy,
    metrics: &AuthoritySinkholeMetrics,
    cross_surface_metrics: &CrossSurfaceAlignmentMetrics,
    authority_distribution: &[DistributionEntry],
    authority_flow_records: &[AuthorityChainFlowRecord],
    subject_flow_records: &[SubjectFlowRecord],
    basin_window_series: &[BasinWindowRecord],
    cross_surface_window_series: &[CrossSurfaceWindowRecord],
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
    let stage2_violations: Vec<&String> = violations
        .iter()
        .filter(|violation| is_stage2_violation(violation))
        .collect();
    let cross_surface_status = if !cross_surface_metrics.stage2_evaluated {
        "NOT_EVALUATED"
    } else if stage2_violations.is_empty() {
        "PASS"
    } else {
        "FAIL"
    };

    write_json(
        &config.output_dir.join("vdl_window.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "window_model": "dual_window",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "replay_boundary_flow_path": config.replay_boundary_flow_path.display().to_string(),
            "trust_reuse_flow_path": config.trust_reuse_flow_path.display().to_string(),
            "total_entry_count": selection.total_entry_count,
            "post_time_filter_entry_count": selection.post_time_filter_entry_count,
            "post_run_limit_entry_count": selection.post_run_limit_entry_count,
            "selected_entry_count": selection.selected_entry_count,
            "applied_window_runs": selection.applied_window_runs,
            "applied_window_seconds": selection.applied_window_seconds,
            "reference_timestamp_unix_ns": selection.reference_timestamp_unix_ns,
            "empty_reason": selection.empty_reason,
            "entries": selection.entries,
        }),
    )?;
    write_json(
        &config.output_dir.join("dominance_analysis.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "authority_chain_distribution": authority_distribution,
            "reference_basin_authority_chain_id": metrics.reference_basin_authority_chain_id,
            "authority_basin_share": metrics.authority_basin_share,
            "subject_group_count": metrics.subject_group_count,
            "repeated_subject_group_count": metrics.repeated_subject_group_count,
            "alternate_path_subject_group_count": metrics.alternate_path_subject_group_count,
        }),
    )?;
    write_json(
        &config.output_dir.join("authority_chain_flow_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "reference_basin_authority_chain_id": metrics.reference_basin_authority_chain_id,
            "authority_chain_flow": authority_flow_records,
            "subject_flow": subject_flow_records,
        }),
    )?;
    write_json(
        &config.output_dir.join("basin_window_series.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "series_window_runs": policy.series_window_runs,
            "series_window_count": policy.series_window_count,
            "reference_basin_authority_chain_id": metrics.reference_basin_authority_chain_id,
            "windows": basin_window_series,
        }),
    )?;
    write_json(
        &config.output_dir.join("basin_absorption_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "risk_class": "authority-sinkhole-drift",
            "reference_basin_authority_chain_id": metrics.reference_basin_authority_chain_id,
            "metrics": metrics,
            "policy_thresholds": {
                "min_repeated_subject_groups": policy.min_repeated_subject_groups,
                "max_authority_basin_share": policy.max_authority_basin_share,
                "max_authority_basin_reuse_ratio": policy.max_authority_basin_reuse_ratio,
                "max_authority_basin_repeat_capture_rate": policy.max_authority_basin_repeat_capture_rate,
                "min_alternate_path_decay_ratio": policy.min_alternate_path_decay_ratio,
                "max_basin_dominance_slope": policy.max_basin_dominance_slope,
            },
            "cross_surface_policy_thresholds": {
                "max_replay_boundary_basin_capture_ratio": policy.max_replay_boundary_basin_capture_ratio,
                "max_replay_boundary_repeat_capture_rate": policy.max_replay_boundary_repeat_capture_rate,
                "max_trust_reuse_basin_capture_ratio": policy.max_trust_reuse_basin_capture_ratio,
                "max_trust_reuse_repeat_capture_rate": policy.max_trust_reuse_repeat_capture_rate,
                "max_cross_surface_basin_alignment_ratio": policy.max_cross_surface_basin_alignment_ratio,
                "min_cross_surface_alternate_path_decay_ratio": policy.min_cross_surface_alternate_path_decay_ratio,
                "max_basin_alignment_slope": policy.max_basin_alignment_slope,
            },
            "series_window_runs": policy.series_window_runs,
            "series_window_count": policy.series_window_count,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config
            .output_dir
            .join("cross_surface_basin_alignment_report.json"),
        &serde_json::json!({
            "status": cross_surface_status,
            "risk_class": "authority-sinkhole-drift",
            "reference_basin_authority_chain_id": metrics.reference_basin_authority_chain_id,
            "replay_boundary_flow_path": config.replay_boundary_flow_path.display().to_string(),
            "trust_reuse_flow_path": config.trust_reuse_flow_path.display().to_string(),
            "metrics": cross_surface_metrics,
            "policy_thresholds": {
                "max_replay_boundary_basin_capture_ratio": policy.max_replay_boundary_basin_capture_ratio,
                "max_replay_boundary_repeat_capture_rate": policy.max_replay_boundary_repeat_capture_rate,
                "max_trust_reuse_basin_capture_ratio": policy.max_trust_reuse_basin_capture_ratio,
                "max_trust_reuse_repeat_capture_rate": policy.max_trust_reuse_repeat_capture_rate,
                "max_cross_surface_basin_alignment_ratio": policy.max_cross_surface_basin_alignment_ratio,
                "min_cross_surface_alternate_path_decay_ratio": policy.min_cross_surface_alternate_path_decay_ratio,
                "max_basin_alignment_slope": policy.max_basin_alignment_slope,
            },
            "windows": cross_surface_window_series,
            "violations": stage2_violations,
            "violations_count": stage2_violations.len(),
        }),
    )?;
    write_json(
        &config
            .output_dir
            .join("authority_sinkhole_absorption_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "mode": "phase13_authority_sinkhole_absorption_gate",
            "risk_class": "authority-sinkhole-drift",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "replay_boundary_flow_path": config.replay_boundary_flow_path.display().to_string(),
            "trust_reuse_flow_path": config.trust_reuse_flow_path.display().to_string(),
            "window_model": "dual_window",
            "applied_window_runs": selection.applied_window_runs,
            "applied_window_seconds": selection.applied_window_seconds,
            "reference_timestamp_unix_ns": selection.reference_timestamp_unix_ns,
            "empty_reason": selection.empty_reason,
            "policy": policy,
            "metrics": metrics,
            "cross_surface_metrics": cross_surface_metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "gate": "authority-sinkhole-absorption",
            "mode": "phase13_authority_sinkhole_absorption_gate",
            "verdict": verdict.as_str(),
            "detail_report_path": "authority_sinkhole_absorption_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_loading_failure_outputs(
    config: &AuthoritySinkholeAbsorptionGateConfig,
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

    let empty_stage1_metrics = empty_stage1_metrics_json();
    let empty_cross_surface_metrics = empty_cross_surface_metrics_json();
    write_json(
        &config
            .output_dir
            .join("authority_sinkhole_absorption_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_authority_sinkhole_absorption_gate",
            "risk_class": "authority-sinkhole-drift",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "replay_boundary_flow_path": config.replay_boundary_flow_path.display().to_string(),
            "trust_reuse_flow_path": config.trust_reuse_flow_path.display().to_string(),
            "window_model": "dual_window",
            "applied_window_runs": config.window_runs_override,
            "applied_window_seconds": config.window_seconds_override,
            "reference_timestamp_unix_ns": serde_json::Value::Null,
            "empty_reason": "load_failure",
            "load_failure_stage": load_failure_stage,
            "load_error": load_error,
            "policy": serde_json::Value::Null,
            "metrics": empty_stage1_metrics,
            "cross_surface_metrics": empty_cross_surface_metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "gate": "authority-sinkhole-absorption",
            "mode": "phase13_authority_sinkhole_absorption_gate",
            "verdict": "FAIL",
            "detail_report_path": "authority_sinkhole_absorption_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("vdl_window.json"),
        &serde_json::json!({
            "status": "FAIL",
            "window_model": "dual_window",
            "ledger_path": config.ledger_path.display().to_string(),
            "policy_path": config.policy_path.display().to_string(),
            "replay_boundary_flow_path": config.replay_boundary_flow_path.display().to_string(),
            "trust_reuse_flow_path": config.trust_reuse_flow_path.display().to_string(),
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
        &config.output_dir.join("dominance_analysis.json"),
        &serde_json::json!({
            "status": "FAIL",
            "authority_chain_distribution": [],
            "reference_basin_authority_chain_id": serde_json::Value::Null,
            "authority_basin_share": 0.0,
            "subject_group_count": 0,
            "repeated_subject_group_count": 0,
            "alternate_path_subject_group_count": 0
        }),
    )?;
    write_json(
        &config.output_dir.join("authority_chain_flow_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "reference_basin_authority_chain_id": serde_json::Value::Null,
            "authority_chain_flow": [],
            "subject_flow": []
        }),
    )?;
    write_json(
        &config.output_dir.join("basin_window_series.json"),
        &serde_json::json!({
            "status": "FAIL",
            "series_window_runs": 0,
            "series_window_count": 0,
            "reference_basin_authority_chain_id": serde_json::Value::Null,
            "windows": []
        }),
    )?;
    write_json(
        &config
            .output_dir
            .join("cross_surface_basin_alignment_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "risk_class": "authority-sinkhole-drift",
            "reference_basin_authority_chain_id": serde_json::Value::Null,
            "replay_boundary_flow_path": config.replay_boundary_flow_path.display().to_string(),
            "trust_reuse_flow_path": config.trust_reuse_flow_path.display().to_string(),
            "metrics": empty_cross_surface_metrics,
            "policy_thresholds": serde_json::Value::Null,
            "windows": [],
            "violations": violations,
            "violations_count": violations.len()
        }),
    )?;
    write_json(
        &config.output_dir.join("basin_absorption_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "risk_class": "authority-sinkhole-drift",
            "reference_basin_authority_chain_id": serde_json::Value::Null,
            "metrics": empty_stage1_metrics,
            "policy_thresholds": serde_json::Value::Null,
            "violations": violations,
            "violations_count": violations.len()
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn is_stage2_violation(violation: &str) -> bool {
    violation.contains("missing_stage2_companion_flow")
        || violation.contains("stage2_metric_not_evaluable")
        || violation.contains("replay_boundary_basin_capture_ratio")
        || violation.contains("replay_boundary_repeat_capture_rate")
        || violation.contains("trust_reuse_basin_capture_ratio")
        || violation.contains("trust_reuse_repeat_capture_rate")
        || violation.contains("cross_surface_basin_alignment_ratio")
        || violation.contains("cross_surface_alternate_path_decay_ratio")
        || violation.contains("basin_alignment_slope")
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize JSON for {}: {error}", path.display()))?;
    fs::write(path, bytes)
        .map_err(|error| format!("failed to write JSON {}: {error}", path.display()))
}

fn empty_stage1_metrics_json() -> serde_json::Value {
    serde_json::json!({
        "selected_entry_count": 0,
        "unique_authority_chain_count": 0,
        "subject_group_count": 0,
        "repeated_subject_group_count": 0,
        "alternate_path_subject_group_count": 0,
        "reference_basin_authority_chain_id": serde_json::Value::Null,
        "authority_basin_share": 0.0,
        "authority_basin_reuse_ratio": serde_json::Value::Null,
        "authority_basin_repeat_capture_rate": serde_json::Value::Null,
        "alternate_path_decay_ratio": serde_json::Value::Null,
        "basin_dominance_slope": serde_json::Value::Null,
        "evaluated_series_window_count": 0,
        "reference_basin_first_window_share": serde_json::Value::Null,
        "reference_basin_last_window_share": serde_json::Value::Null,
        "monotonic_non_decreasing": false
    })
}

fn empty_cross_surface_metrics_json() -> serde_json::Value {
    serde_json::json!({
        "replay_boundary_report_present": false,
        "trust_reuse_report_present": false,
        "stage2_evaluated": false,
        "verification_terminal_subject_count": 0,
        "replay_boundary_event_count": 0,
        "replay_boundary_terminal_event_count": 0,
        "replay_boundary_reused_terminal_event_count": 0,
        "trust_reuse_event_count": 0,
        "trust_reuse_terminal_event_count": 0,
        "trust_reuse_reused_terminal_event_count": 0,
        "replay_boundary_basin_capture_ratio": serde_json::Value::Null,
        "replay_boundary_repeat_capture_rate": serde_json::Value::Null,
        "trust_reuse_basin_capture_ratio": serde_json::Value::Null,
        "trust_reuse_repeat_capture_rate": serde_json::Value::Null,
        "verification_replay_comparable_subject_count": 0,
        "verification_trust_comparable_subject_count": 0,
        "replay_trust_comparable_subject_count": 0,
        "cross_surface_comparable_subject_count": 0,
        "cross_surface_alternate_candidate_count": 0,
        "verification_replay_alignment_ratio": serde_json::Value::Null,
        "verification_trust_alignment_ratio": serde_json::Value::Null,
        "replay_trust_alignment_ratio": serde_json::Value::Null,
        "cross_surface_basin_alignment_ratio": serde_json::Value::Null,
        "cross_surface_alternate_path_decay_ratio": serde_json::Value::Null,
        "basin_alignment_slope": serde_json::Value::Null,
        "evaluated_alignment_window_count": 0,
        "alignment_first_window_ratio": serde_json::Value::Null,
        "alignment_last_window_ratio": serde_json::Value::Null
    })
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
        authority_chain_id: &str,
    ) -> VerificationDiversityLedgerEntry {
        VerificationDiversityLedgerEntry {
            ledger_version: 1,
            entry_id: format!("entry-{timestamp_unix_ns}-{verifier_id}"),
            run_id: format!("run-{timestamp_unix_ns}"),
            timestamp_unix_ns,
            subject_bundle_id: subject_bundle_id.to_string(),
            verification_context_id: "context-a".to_string(),
            verification_node_id: format!("node-{verifier_id}"),
            verifier_id: verifier_id.to_string(),
            authority_chain_id: authority_chain_id.to_string(),
            lineage_id: format!("lineage-{verifier_id}"),
            execution_cluster_id: None,
            verdict: "PASS".to_string(),
            receipt_hash: format!("receipt-{timestamp_unix_ns}"),
        }
    }

    #[test]
    fn subject_metrics_detect_reference_basin_capture() {
        let entries = vec![
            sample_entry(1, "bundle-a", "verifier-a", "chain-a"),
            sample_entry(2, "bundle-a", "verifier-b", "chain-a"),
            sample_entry(3, "bundle-b", "verifier-c", "chain-b"),
            sample_entry(4, "bundle-b", "verifier-a", "chain-a"),
            sample_entry(5, "bundle-c", "verifier-d", "chain-a"),
        ];
        let authority_distribution =
            build_distribution(&entries, |entry| entry.authority_chain_id.clone());
        let subject_groups = build_subject_groups(&entries);
        let series = build_basin_window_series(&entries, Some("chain-a"), 2, 3);
        let metrics = build_metrics(
            entries.len(),
            &authority_distribution,
            &subject_groups,
            &series,
            Some("chain-a"),
        );

        assert_eq!(metrics.repeated_subject_group_count, 2);
        assert_eq!(metrics.alternate_path_subject_group_count, 1);
        assert!((metrics.authority_basin_share - 0.8).abs() < 0.000_001);
        assert_eq!(metrics.authority_basin_reuse_ratio, Some(1.0));
        assert_eq!(metrics.authority_basin_repeat_capture_rate, Some(0.5));
        assert_eq!(metrics.alternate_path_decay_ratio, Some(0.0));
    }

    #[test]
    fn basin_window_series_reports_positive_reference_slope() {
        let entries = vec![
            sample_entry(1, "bundle-a", "verifier-a", "chain-a"),
            sample_entry(2, "bundle-b", "verifier-b", "chain-b"),
            sample_entry(3, "bundle-c", "verifier-a", "chain-a"),
            sample_entry(4, "bundle-d", "verifier-a", "chain-a"),
            sample_entry(5, "bundle-e", "verifier-a", "chain-a"),
            sample_entry(6, "bundle-f", "verifier-c", "chain-c"),
        ];
        let series = build_basin_window_series(&entries, Some("chain-a"), 2, 3);

        assert_eq!(series.len(), 3);
        assert!((series[0].reference_basin_share - 0.5).abs() < 0.000_001);
        assert!((series[1].reference_basin_share - 1.0).abs() < 0.000_001);
        assert!((series[2].reference_basin_share - 0.5).abs() < 0.000_001);
        assert_eq!(compute_reference_basin_slope(&series), Some(0.0));
    }

    #[test]
    fn slice_window_applies_time_then_run_limit() {
        let entries = vec![
            sample_entry(1 * NANOS_PER_SECOND, "bundle-a", "verifier-a", "chain-a"),
            sample_entry(2 * NANOS_PER_SECOND, "bundle-b", "verifier-b", "chain-b"),
            sample_entry(3 * NANOS_PER_SECOND, "bundle-c", "verifier-c", "chain-c"),
            sample_entry(4 * NANOS_PER_SECOND, "bundle-d", "verifier-d", "chain-d"),
        ];
        let selection = slice_window(entries, Some(2), Some(2));

        assert_eq!(selection.total_entry_count, 4);
        assert_eq!(selection.post_time_filter_entry_count, 3);
        assert_eq!(selection.post_run_limit_entry_count, 2);
        assert_eq!(selection.selected_entry_count, 2);
        assert_eq!(
            selection
                .entries
                .iter()
                .map(|entry| entry.subject_bundle_id.as_str())
                .collect::<Vec<_>>(),
            vec!["bundle-c", "bundle-d"]
        );
    }
}
