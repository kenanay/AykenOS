use crate::audit::ledger::load_audit_events;
use crate::diversity_floor::GateVerdict;
use crate::diversity_ledger::{
    compute_diversity_ledger_entry_id, load_diversity_ledger_entries,
    validate_diversity_ledger_entry, write_diversity_ledger_entries,
    VerificationDiversityLedgerEntry,
};
use crate::types::{VerificationAuditEvent, VerificationVerdict};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct VerificationDiversityLedgerProducerConfig {
    pub audit_ledger_path: PathBuf,
    pub binding_path: PathBuf,
    pub ledger_path: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct VerificationDiversityLedgerProducerManifest {
    pub binding_version: u32,
    pub run_id: String,
    #[serde(default = "default_context_id_source")]
    pub verification_context_id_source: String,
    pub node_bindings: Vec<VerificationNodeBinding>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct VerificationNodeBinding {
    pub verification_node_id: String,
    #[serde(default)]
    pub verifier_key_id: Option<String>,
    pub verifier_id: String,
    pub authority_chain_id: String,
    pub lineage_id: String,
    #[serde(default)]
    pub execution_cluster_id: Option<String>,
}

#[derive(Debug)]
pub struct VerificationDiversityLedgerProducerOutcome {
    pub verdict: GateVerdict,
    pub violations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProducerMetrics {
    source_event_count: usize,
    candidate_entry_count: usize,
    appended_entry_count: usize,
    duplicate_skipped_count: usize,
    existing_entry_count: usize,
    final_entry_count: usize,
}

pub fn run_diversity_ledger_producer(
    config: &VerificationDiversityLedgerProducerConfig,
) -> Result<VerificationDiversityLedgerProducerOutcome, String> {
    let audit_events = match load_audit_events(&config.audit_ledger_path) {
        Ok(events) => events,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_audit_ledger:{}",
                config.audit_ledger_path.display()
            )];
            write_loading_failure_outputs(config, &violations, &error.to_string(), "audit_ledger_load")?;
            return Ok(VerificationDiversityLedgerProducerOutcome {
                verdict: GateVerdict::Fail,
                violations,
            });
        }
    };
    let manifest = match load_manifest(&config.binding_path) {
        Ok(value) => value,
        Err(error) => {
            let violations = vec![format!(
                "missing_or_invalid_binding_manifest:{}",
                config.binding_path.display()
            )];
            write_loading_failure_outputs(config, &violations, &error, "binding_manifest_load")?;
            return Ok(VerificationDiversityLedgerProducerOutcome {
                verdict: GateVerdict::Fail,
                violations,
            });
        }
    };

    let existing_entries = if config.ledger_path.exists() {
        match load_diversity_ledger_entries(&config.ledger_path) {
            Ok(entries) => entries,
            Err(error) => {
                let violations = vec![format!(
                    "missing_or_invalid_target_ledger:{}",
                    config.ledger_path.display()
                )];
                write_loading_failure_outputs(config, &violations, &error, "target_ledger_load")?;
                return Ok(VerificationDiversityLedgerProducerOutcome {
                    verdict: GateVerdict::Fail,
                    violations,
                });
            }
        }
    } else {
        Vec::new()
    };

    let mut violations = validate_manifest(&manifest);
    let bindings = build_binding_map(&manifest, &mut violations);
    let existing_entry_count = existing_entries.len();

    let mut existing_by_id = BTreeMap::<String, VerificationDiversityLedgerEntry>::new();
    for entry in &existing_entries {
        if let Err(error) = validate_diversity_ledger_entry(entry) {
            violations.push(format!("invalid_existing_ledger_entry:{}:{}", entry.entry_id, error));
        } else {
            existing_by_id.insert(entry.entry_id.clone(), entry.clone());
        }
    }

    let mut candidate_entries = Vec::new();
    for event in &audit_events {
        match build_entry(event, &manifest, &bindings) {
            Ok(entry) => candidate_entries.push(entry),
            Err(error) => violations.push(format!("entry_derivation_failure:{}:{error}", event.event_id)),
        }
    }

    let mut final_entries = existing_entries.clone();
    let mut seen_candidate_ids = BTreeSet::<String>::new();
    let mut duplicate_skipped_count = 0usize;
    let mut appended_entry_count = 0usize;

    for candidate in &candidate_entries {
        if !seen_candidate_ids.insert(candidate.entry_id.clone()) {
            duplicate_skipped_count += 1;
            continue;
        }
        if let Some(existing) = existing_by_id.get(&candidate.entry_id) {
            if existing != candidate {
                violations.push(format!(
                    "entry_id_conflict:{}:existing_target_ledger_differs_from_candidate",
                    candidate.entry_id
                ));
            } else {
                duplicate_skipped_count += 1;
            }
            continue;
        }
        final_entries.push(candidate.clone());
        existing_by_id.insert(candidate.entry_id.clone(), candidate.clone());
        appended_entry_count += 1;
    }

    let metrics = ProducerMetrics {
        source_event_count: audit_events.len(),
        candidate_entry_count: candidate_entries.len(),
        appended_entry_count,
        duplicate_skipped_count,
        existing_entry_count,
        final_entry_count: final_entries.len(),
    };

    if !violations.is_empty() {
        write_failure_outputs(config, &manifest, &metrics, &violations)?;
        return Ok(VerificationDiversityLedgerProducerOutcome {
            verdict: GateVerdict::Fail,
            violations,
        });
    }

    write_diversity_ledger_entries(&config.ledger_path, &final_entries)?;
    write_outputs(config, &manifest, &metrics, &final_entries, &violations)?;

    Ok(VerificationDiversityLedgerProducerOutcome {
        verdict: GateVerdict::Pass,
        violations,
    })
}

fn default_context_id_source() -> String {
    "policy_hash".to_string()
}

fn load_manifest(path: &Path) -> Result<VerificationDiversityLedgerProducerManifest, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read manifest at {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse manifest at {}: {error}", path.display()))
}

fn validate_manifest(manifest: &VerificationDiversityLedgerProducerManifest) -> Vec<String> {
    let mut violations = Vec::new();
    if manifest.binding_version != 1 {
        violations.push(format!(
            "unsupported_binding_version:{}",
            manifest.binding_version
        ));
    }
    if manifest.run_id.trim().is_empty() {
        violations.push("run_id_must_not_be_empty".to_string());
    }
    if manifest.verification_context_id_source != "policy_hash" {
        violations.push(format!(
            "unsupported_verification_context_id_source:{}",
            manifest.verification_context_id_source
        ));
    }
    if manifest.node_bindings.is_empty() {
        violations.push("node_bindings_must_not_be_empty".to_string());
    }
    violations
}

fn build_binding_map(
    manifest: &VerificationDiversityLedgerProducerManifest,
    violations: &mut Vec<String>,
) -> BTreeMap<String, VerificationNodeBinding> {
    let mut bindings = BTreeMap::new();
    for binding in &manifest.node_bindings {
        for (label, value) in [
            ("verification_node_id", binding.verification_node_id.as_str()),
            ("verifier_id", binding.verifier_id.as_str()),
            ("authority_chain_id", binding.authority_chain_id.as_str()),
            ("lineage_id", binding.lineage_id.as_str()),
        ] {
            if value.trim().is_empty() {
                violations.push(format!(
                    "binding_field_must_not_be_empty:{}:{}",
                    binding.verification_node_id, label
                ));
            }
        }
        if bindings
            .insert(binding.verification_node_id.clone(), binding.clone())
            .is_some()
        {
            violations.push(format!(
                "duplicate_verification_node_binding:{}",
                binding.verification_node_id
            ));
        }
    }
    bindings
}

fn build_entry(
    event: &VerificationAuditEvent,
    manifest: &VerificationDiversityLedgerProducerManifest,
    bindings: &BTreeMap<String, VerificationNodeBinding>,
) -> Result<VerificationDiversityLedgerEntry, String> {
    if event.event_type != "verification" {
        return Err(format!("unsupported_event_type:{}", event.event_type));
    }
    let binding = bindings
        .get(&event.verifier_node_id)
        .ok_or_else(|| format!("missing_binding_for_verification_node_id:{}", event.verifier_node_id))?;
    if let Some(expected_key_id) = binding.verifier_key_id.as_deref() {
        let actual = event.verifier_key_id.as_deref().unwrap_or("");
        if actual != expected_key_id {
            return Err(format!(
                "verifier_key_id_mismatch:expected={expected_key_id}:actual={actual}"
            ));
        }
    }
    if event.bundle_id.trim().is_empty() || event.policy_hash.trim().is_empty() {
        return Err("bundle_id_and_policy_hash_must_not_be_empty".to_string());
    }
    if event.receipt_hash.trim().is_empty() {
        return Err("receipt_hash_must_not_be_empty".to_string());
    }

    let mut entry = VerificationDiversityLedgerEntry {
        ledger_version: 1,
        entry_id: String::new(),
        run_id: manifest.run_id.clone(),
        timestamp_unix_ns: parse_event_time_to_unix_ns(&event.event_time_utc)?,
        subject_bundle_id: event.bundle_id.clone(),
        verification_context_id: event.policy_hash.clone(),
        verification_node_id: event.verifier_node_id.clone(),
        verifier_id: binding.verifier_id.clone(),
        authority_chain_id: binding.authority_chain_id.clone(),
        lineage_id: binding.lineage_id.clone(),
        execution_cluster_id: binding.execution_cluster_id.clone(),
        verdict: normalize_verdict(&event.verdict).to_string(),
        receipt_hash: event.receipt_hash.clone(),
    };
    entry.entry_id = compute_diversity_ledger_entry_id(&entry)?;
    validate_diversity_ledger_entry(&entry)?;
    Ok(entry)
}

fn normalize_verdict(verdict: &VerificationVerdict) -> &'static str {
    match verdict {
        VerificationVerdict::Trusted => "PASS",
        VerificationVerdict::Untrusted
        | VerificationVerdict::Invalid
        | VerificationVerdict::RejectedByPolicy => "FAIL",
    }
}

fn parse_event_time_to_unix_ns(value: &str) -> Result<u64, String> {
    let (datetime, fraction) = value
        .strip_suffix('Z')
        .ok_or_else(|| format!("unsupported_timestamp_format:{value}"))?
        .split_once('.')
        .map_or((value.strip_suffix('Z').unwrap_or(value), ""), |(base, frac)| (base, frac));
    let parts: Vec<&str> = datetime.split('T').collect();
    if parts.len() != 2 {
        return Err(format!("unsupported_timestamp_format:{value}"));
    }
    let date: Vec<u32> = parts[0]
        .split('-')
        .map(|item| item.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("invalid_timestamp_date:{value}:{error}"))?;
    let time: Vec<u32> = parts[1]
        .split(':')
        .map(|item| item.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("invalid_timestamp_time:{value}:{error}"))?;
    if date.len() != 3 || time.len() != 3 {
        return Err(format!("unsupported_timestamp_format:{value}"));
    }
    let days = days_from_civil(date[0] as i64, date[1], date[2])?;
    let seconds = days
        .checked_mul(86_400)
        .and_then(|base| base.checked_add((time[0] as i64) * 3_600 + (time[1] as i64) * 60 + time[2] as i64))
        .ok_or_else(|| format!("timestamp_overflow:{value}"))?;
    if seconds < 0 {
        return Err(format!("timestamp_before_unix_epoch:{value}"));
    }
    let nanos = parse_fractional_nanos(fraction)?;
    Ok((seconds as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add(nanos))
}

fn parse_fractional_nanos(fraction: &str) -> Result<u64, String> {
    if fraction.is_empty() {
        return Ok(0);
    }
    if fraction.len() > 9 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid_fractional_timestamp_component:{fraction}"));
    }
    let mut value = fraction.to_string();
    while value.len() < 9 {
        value.push('0');
    }
    value
        .parse::<u64>()
        .map_err(|error| format!("invalid_fractional_timestamp_component:{fraction}:{error}"))
}

fn days_from_civil(year: i64, month: u32, day: u32) -> Result<i64, String> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(format!("invalid_calendar_date:{year:04}-{month:02}-{day:02}"));
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year / 400
    } else {
        (adjusted_year - 399) / 400
    };
    let year_of_era = adjusted_year - era * 400;
    let month = month as i64;
    let day_of_year =
        (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Ok(era * 146_097 + day_of_era - 719_468)
}

fn write_outputs(
    config: &VerificationDiversityLedgerProducerConfig,
    manifest: &VerificationDiversityLedgerProducerManifest,
    metrics: &ProducerMetrics,
    final_entries: &[VerificationDiversityLedgerEntry],
    violations: &[String],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir)
        .map_err(|error| format!("failed to create output dir {}: {error}", config.output_dir.display()))?;
    let verdict = if violations.is_empty() {
        GateVerdict::Pass
    } else {
        GateVerdict::Fail
    };
    let output_ledger_path = config.output_dir.join("verification_diversity_ledger.json");
    write_diversity_ledger_entries(&output_ledger_path, final_entries)?;
    write_json(
        &config.output_dir.join("verification_diversity_ledger_append_report.json"),
        &serde_json::json!({
            "status": verdict.as_str(),
            "mode": "phase13_verification_diversity_ledger_producer",
            "audit_ledger_path": config.audit_ledger_path.display().to_string(),
            "binding_path": config.binding_path.display().to_string(),
            "ledger_path": config.ledger_path.display().to_string(),
            "output_ledger_path": output_ledger_path.display().to_string(),
            "run_id": manifest.run_id,
            "verification_context_id_source": manifest.verification_context_id_source,
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "artifact": "verification-diversity-ledger-producer",
            "mode": "phase13_verification_diversity_ledger_producer",
            "verdict": verdict.as_str(),
            "detail_report_path": "verification_diversity_ledger_append_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_failure_outputs(
    config: &VerificationDiversityLedgerProducerConfig,
    manifest: &VerificationDiversityLedgerProducerManifest,
    metrics: &ProducerMetrics,
    violations: &[String],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir)
        .map_err(|error| format!("failed to create output dir {}: {error}", config.output_dir.display()))?;
    write_json(
        &config.output_dir.join("verification_diversity_ledger_append_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_verification_diversity_ledger_producer",
            "audit_ledger_path": config.audit_ledger_path.display().to_string(),
            "binding_path": config.binding_path.display().to_string(),
            "ledger_path": config.ledger_path.display().to_string(),
            "output_ledger_path": config.output_dir.join("verification_diversity_ledger.json").display().to_string(),
            "run_id": manifest.run_id,
            "verification_context_id_source": manifest.verification_context_id_source,
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "artifact": "verification-diversity-ledger-producer",
            "mode": "phase13_verification_diversity_ledger_producer",
            "verdict": "FAIL",
            "detail_report_path": "verification_diversity_ledger_append_report.json",
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_violations(&config.output_dir.join("violations.txt"), violations)?;
    Ok(())
}

fn write_loading_failure_outputs(
    config: &VerificationDiversityLedgerProducerConfig,
    violations: &[String],
    load_error: &str,
    load_failure_stage: &str,
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir)
        .map_err(|error| format!("failed to create output dir {}: {error}", config.output_dir.display()))?;
    let metrics = ProducerMetrics {
        source_event_count: 0,
        candidate_entry_count: 0,
        appended_entry_count: 0,
        duplicate_skipped_count: 0,
        existing_entry_count: 0,
        final_entry_count: 0,
    };
    write_json(
        &config.output_dir.join("verification_diversity_ledger_append_report.json"),
        &serde_json::json!({
            "status": "FAIL",
            "mode": "phase13_verification_diversity_ledger_producer",
            "audit_ledger_path": config.audit_ledger_path.display().to_string(),
            "binding_path": config.binding_path.display().to_string(),
            "ledger_path": config.ledger_path.display().to_string(),
            "output_ledger_path": config.output_dir.join("verification_diversity_ledger.json").display().to_string(),
            "load_failure_stage": load_failure_stage,
            "load_error": load_error,
            "metrics": metrics,
            "violations": violations,
            "violations_count": violations.len(),
        }),
    )?;
    write_json(
        &config.output_dir.join("report.json"),
        &serde_json::json!({
            "artifact": "verification-diversity-ledger-producer",
            "mode": "phase13_verification_diversity_ledger_producer",
            "verdict": "FAIL",
            "detail_report_path": "verification_diversity_ledger_append_report.json",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(
        event_id: &str,
        bundle_id: &str,
        verifier_node_id: &str,
        verdict: VerificationVerdict,
        receipt_hash: &str,
        event_time_utc: &str,
    ) -> VerificationAuditEvent {
        VerificationAuditEvent {
            event_version: 1,
            event_type: "verification".to_string(),
            event_id: event_id.to_string(),
            event_time_utc: event_time_utc.to_string(),
            verifier_node_id: verifier_node_id.to_string(),
            verifier_key_id: Some("key-a".to_string()),
            bundle_id: bundle_id.to_string(),
            trust_overlay_hash: "overlay-a".to_string(),
            policy_hash: "policy-a".to_string(),
            registry_snapshot_hash: "registry-a".to_string(),
            verdict,
            receipt_hash: receipt_hash.to_string(),
            previous_event_hash: None,
        }
    }

    fn sample_manifest() -> VerificationDiversityLedgerProducerManifest {
        VerificationDiversityLedgerProducerManifest {
            binding_version: 1,
            run_id: "run-1".to_string(),
            verification_context_id_source: "policy_hash".to_string(),
            node_bindings: vec![VerificationNodeBinding {
                verification_node_id: "node-a".to_string(),
                verifier_key_id: Some("key-a".to_string()),
                verifier_id: "verifier-a".to_string(),
                authority_chain_id: "chain-a".to_string(),
                lineage_id: "lineage-a".to_string(),
                execution_cluster_id: Some("cluster-a".to_string()),
            }],
        }
    }

    #[test]
    fn build_entry_generates_content_addressed_identity() {
        let manifest = sample_manifest();
        let bindings = build_binding_map(&manifest, &mut Vec::new());
        let event = sample_event(
            "audit-1",
            "bundle-a",
            "node-a",
            VerificationVerdict::Trusted,
            &"a".repeat(64),
            "2026-03-14T12:00:00Z",
        );

        let entry = build_entry(&event, &manifest, &bindings).expect("entry should build");

        assert_eq!(entry.run_id, "run-1");
        assert_eq!(entry.verification_context_id, "policy-a");
        assert_eq!(entry.verdict, "PASS");
        assert_eq!(
            entry.entry_id,
            compute_diversity_ledger_entry_id(&entry).expect("entry id should recompute")
        );
    }

    #[test]
    fn timestamp_parser_supports_fractional_seconds() {
        let parsed = parse_event_time_to_unix_ns("2026-03-14T12:00:00.123456789Z")
            .expect("timestamp should parse");
        let base = parse_event_time_to_unix_ns("2026-03-14T12:00:00Z")
            .expect("base timestamp should parse");
        assert_eq!(parsed - base, 123_456_789);
    }

    #[test]
    fn normalize_verdict_maps_non_trusted_to_fail() {
        assert_eq!(normalize_verdict(&VerificationVerdict::Trusted), "PASS");
        assert_eq!(normalize_verdict(&VerificationVerdict::Untrusted), "FAIL");
        assert_eq!(normalize_verdict(&VerificationVerdict::Invalid), "FAIL");
        assert_eq!(
            normalize_verdict(&VerificationVerdict::RejectedByPolicy),
            "FAIL"
        );
    }
}
