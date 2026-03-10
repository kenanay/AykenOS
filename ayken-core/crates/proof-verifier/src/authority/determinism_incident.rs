use crate::canonical::digest::sha256_hex;
use crate::authority::parity::{NodeParityOutcome, ParityEvidenceState};
use crate::types::VerificationVerdict;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use crate::authority::parity::{build_node_parity_outcome, ParityArtifactForm};
#[cfg(test)]
use crate::types::{
    VerdictSubject, VerifierAuthorityResolution, VerifierAuthorityResolutionClass,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterminismIncidentClass {
    DeterminismFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterminismIncidentSeverity {
    PureDeterminismFailure,
    AuthorityDrift,
    ContextDrift,
    SubjectDrift,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterminismSuppressionReason {
    HistoricalOnly,
    InsufficientEvidence,
    SubjectDrift,
    ContextDrift,
    AuthorityDrift,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismOutcomePartition {
    pub outcome_key: String,
    pub node_ids: Vec<String>,
    pub node_count: usize,
    pub verdicts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismIncident {
    pub incident_id: String,
    pub surface_key: String,
    pub nodes: Vec<String>,
    pub outcome_keys: Vec<String>,
    pub node_count: usize,
    pub outcome_partition_count: usize,
    pub subject_equal: bool,
    pub context_equal: bool,
    pub authority_equal: bool,
    pub drift_class: DeterminismIncidentClass,
    pub severity: DeterminismIncidentSeverity,
    pub outcome_partitions: Vec<DeterminismOutcomePartition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppressedDeterminismIncident {
    pub surface_key: String,
    pub nodes: Vec<String>,
    pub outcome_keys: Vec<String>,
    pub node_count: usize,
    pub outcome_partition_count: usize,
    pub subject_equal: bool,
    pub context_equal: bool,
    pub authority_equal: bool,
    pub historical_only_present: bool,
    pub insufficient_evidence_present: bool,
    pub suppression_reason: DeterminismSuppressionReason,
    pub outcome_partitions: Vec<DeterminismOutcomePartition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismIncidentReport {
    pub node_count: usize,
    pub surface_partition_count: usize,
    pub determinism_incident_count: usize,
    pub severity_counts: BTreeMap<String, usize>,
    pub suppressed_incident_count: usize,
    pub suppression_reason_counts: BTreeMap<String, usize>,
    pub incidents: Vec<DeterminismIncident>,
    pub suppressed_incidents: Vec<SuppressedDeterminismIncident>,
}

pub fn analyze_determinism_incidents(
    node_outcomes: &[NodeParityOutcome],
) -> DeterminismIncidentReport {
    let mut surfaces: BTreeMap<String, Vec<&NodeParityOutcome>> = BTreeMap::new();
    for node in node_outcomes {
        surfaces
            .entry(node.surface_key().to_string())
            .or_default()
            .push(node);
    }

    let surface_partition_count = surfaces.len();
    let mut incidents = Vec::new();
    let mut severity_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut suppressed_incidents = Vec::new();
    let mut suppression_reason_counts: BTreeMap<String, usize> = BTreeMap::new();

    for (surface_key, mut nodes) in surfaces.into_iter() {
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        let outcome_partitions = build_outcome_partitions(&nodes);
        if outcome_partitions.len() <= 1 {
            continue;
        }

        let nodes_list = sorted_node_ids(&nodes);
        let outcome_keys = outcome_partitions
            .iter()
            .map(|partition| partition.outcome_key.clone())
            .collect();
        let subject_equal = unique_count(&nodes, |node| node.subject_hash()) == 1;
        let context_equal = unique_count(&nodes, |node| node.context_hash()) == 1;
        let authority_equal = unique_count(&nodes, |node| node.authority_hash()) == 1;
        let historical_only_present = nodes.iter().any(|node| node.is_historical_only());
        let insufficient_evidence_present = nodes
            .iter()
            .any(|node| node.evidence_state() == &ParityEvidenceState::Insufficient);
        let severity = derive_severity(subject_equal, context_equal, authority_equal);
        if let Some(reason) = classify_suppression_reason(
            subject_equal,
            context_equal,
            authority_equal,
            historical_only_present,
            insufficient_evidence_present,
        ) {
            *suppression_reason_counts
                .entry(suppression_reason_label(&reason).to_string())
                .or_insert(0) += 1;
            suppressed_incidents.push(SuppressedDeterminismIncident {
                surface_key,
                nodes: nodes_list.clone(),
                outcome_keys,
                node_count: nodes_list.len(),
                outcome_partition_count: outcome_partitions.len(),
                subject_equal,
                context_equal,
                authority_equal,
                historical_only_present,
                insufficient_evidence_present,
                suppression_reason: reason,
                outcome_partitions,
            });
            continue;
        }
        *severity_counts
            .entry(severity_label(&severity).to_string())
            .or_insert(0) += 1;

        incidents.push(DeterminismIncident {
            incident_id: compute_incident_id(&surface_key, &outcome_partitions),
            surface_key,
            nodes: nodes_list.clone(),
            outcome_keys,
            node_count: nodes_list.len(),
            outcome_partition_count: outcome_partitions.len(),
            subject_equal,
            context_equal,
            authority_equal,
            drift_class: DeterminismIncidentClass::DeterminismFailure,
            severity,
            outcome_partitions,
        });
    }

    incidents.sort_by(|left, right| {
        right
            .node_count
            .cmp(&left.node_count)
            .then_with(|| left.incident_id.cmp(&right.incident_id))
    });
    suppressed_incidents.sort_by(|left, right| {
        right
            .node_count
            .cmp(&left.node_count)
            .then_with(|| left.surface_key.cmp(&right.surface_key))
    });

    DeterminismIncidentReport {
        node_count: node_outcomes.len(),
        surface_partition_count,
        determinism_incident_count: incidents.len(),
        severity_counts,
        suppressed_incident_count: suppressed_incidents.len(),
        suppression_reason_counts,
        incidents,
        suppressed_incidents,
    }
}

fn derive_severity(
    subject_equal: bool,
    context_equal: bool,
    authority_equal: bool,
) -> DeterminismIncidentSeverity {
    match (subject_equal, context_equal, authority_equal) {
        (true, true, true) => DeterminismIncidentSeverity::PureDeterminismFailure,
        (true, true, false) => DeterminismIncidentSeverity::AuthorityDrift,
        (true, false, true) => DeterminismIncidentSeverity::ContextDrift,
        (false, true, true) => DeterminismIncidentSeverity::SubjectDrift,
        _ => DeterminismIncidentSeverity::Mixed,
    }
}

fn classify_suppression_reason(
    subject_equal: bool,
    context_equal: bool,
    authority_equal: bool,
    historical_only_present: bool,
    insufficient_evidence_present: bool,
) -> Option<DeterminismSuppressionReason> {
    if insufficient_evidence_present {
        return Some(DeterminismSuppressionReason::InsufficientEvidence);
    }
    if historical_only_present {
        return Some(DeterminismSuppressionReason::HistoricalOnly);
    }
    match (subject_equal, context_equal, authority_equal) {
        (true, true, true) => None,
        (false, true, true) => Some(DeterminismSuppressionReason::SubjectDrift),
        (true, false, true) => Some(DeterminismSuppressionReason::ContextDrift),
        (true, true, false) => Some(DeterminismSuppressionReason::AuthorityDrift),
        _ => Some(DeterminismSuppressionReason::Mixed),
    }
}

fn build_outcome_partitions(nodes: &[&NodeParityOutcome]) -> Vec<DeterminismOutcomePartition> {
    let mut partitions: BTreeMap<String, Vec<&NodeParityOutcome>> = BTreeMap::new();
    for node in nodes {
        partitions
            .entry(node.outcome_key().to_string())
            .or_default()
            .push(*node);
    }

    let mut values = Vec::new();
    for (outcome_key, mut partition_nodes) in partitions {
        partition_nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        let node_ids = sorted_node_ids(&partition_nodes);
        let verdicts = partition_nodes
            .iter()
            .map(|node| verdict_label(&node.verdict).to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        values.push(DeterminismOutcomePartition {
            outcome_key,
            node_count: node_ids.len(),
            node_ids,
            verdicts,
        });
    }

    values.sort_by(|left, right| {
        right
            .node_count
            .cmp(&left.node_count)
            .then_with(|| left.outcome_key.cmp(&right.outcome_key))
    });
    values
}

fn compute_incident_id(
    surface_key: &str,
    outcome_partitions: &[DeterminismOutcomePartition],
) -> String {
    let mut parts = outcome_partitions
        .iter()
        .map(|partition| format!("{}:{}", partition.outcome_key, partition.node_count))
        .collect::<Vec<_>>();
    parts.sort();
    let material = format!("{surface_key}|{}", parts.join("|"));
    format!("sha256:{}", sha256_hex(material.as_bytes()))
}

fn unique_count<F>(nodes: &[&NodeParityOutcome], key_fn: F) -> usize
where
    F: Fn(&NodeParityOutcome) -> &str,
{
    nodes.iter()
        .map(|node| key_fn(node).to_string())
        .collect::<BTreeSet<_>>()
        .len()
}

fn sorted_node_ids(nodes: &[&NodeParityOutcome]) -> Vec<String> {
    let mut ids = nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn verdict_label(verdict: &VerificationVerdict) -> &'static str {
    match verdict {
        VerificationVerdict::Trusted => "TRUSTED",
        VerificationVerdict::Untrusted => "UNTRUSTED",
        VerificationVerdict::Invalid => "INVALID",
        VerificationVerdict::RejectedByPolicy => "REJECTED_BY_POLICY",
    }
}

fn severity_label(severity: &DeterminismIncidentSeverity) -> &'static str {
    match severity {
        DeterminismIncidentSeverity::PureDeterminismFailure => "pure_determinism_failure",
        DeterminismIncidentSeverity::AuthorityDrift => "authority_drift",
        DeterminismIncidentSeverity::ContextDrift => "context_drift",
        DeterminismIncidentSeverity::SubjectDrift => "subject_drift",
        DeterminismIncidentSeverity::Mixed => "mixed",
    }
}

fn suppression_reason_label(reason: &DeterminismSuppressionReason) -> &'static str {
    match reason {
        DeterminismSuppressionReason::HistoricalOnly => "historical_only",
        DeterminismSuppressionReason::InsufficientEvidence => "insufficient_evidence",
        DeterminismSuppressionReason::SubjectDrift => "subject_drift",
        DeterminismSuppressionReason::ContextDrift => "context_drift",
        DeterminismSuppressionReason::AuthorityDrift => "authority_drift",
        DeterminismSuppressionReason::Mixed => "mixed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_subject() -> VerdictSubject {
        VerdictSubject {
            bundle_id: "bundle-1".to_string(),
            trust_overlay_hash: "overlay-1".to_string(),
            policy_hash: "policy-1".to_string(),
            registry_snapshot_hash: "registry-1".to_string(),
        }
    }

    fn sample_authority(result_class: VerifierAuthorityResolutionClass) -> VerifierAuthorityResolution {
        VerifierAuthorityResolution {
            result_class,
            requested_verifier_id: "verifier-a".to_string(),
            requested_authority_scope: vec!["distributed_receipt_acceptance".to_string()],
            authority_chain: vec!["root-a".to_string()],
            authority_chain_id: Some("chain-a".to_string()),
            effective_authority_scope: vec!["distributed_receipt_acceptance".to_string()],
            verifier_registry_snapshot_hash: "verifier-registry-1".to_string(),
            findings: Vec::new(),
        }
    }

    fn sample_node(
        node_id: &str,
        verdict: VerificationVerdict,
        authority: &VerifierAuthorityResolution,
        evidence_state: ParityEvidenceState,
    ) -> NodeParityOutcome {
        build_node_parity_outcome(
            node_id,
            node_id,
            &sample_subject(),
            "context-1",
            "contract-v1",
            authority,
            &verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            evidence_state,
        )
        .expect("build node parity outcome")
    }

    #[test]
    fn emits_pure_determinism_incident_for_current_sufficient_surface() {
        let authority = sample_authority(VerifierAuthorityResolutionClass::AuthorityResolvedRoot);
        let nodes = vec![
            sample_node(
                "node-a",
                VerificationVerdict::Trusted,
                &authority,
                ParityEvidenceState::Sufficient,
            ),
            sample_node(
                "node-b",
                VerificationVerdict::RejectedByPolicy,
                &authority,
                ParityEvidenceState::Sufficient,
            ),
        ];

        let report = analyze_determinism_incidents(&nodes);
        assert_eq!(report.determinism_incident_count, 1);
        assert_eq!(report.suppressed_incident_count, 0);
        assert_eq!(
            report.severity_counts.get("pure_determinism_failure"),
            Some(&1usize)
        );
    }

    #[test]
    fn suppresses_false_incident_when_evidence_is_insufficient() {
        let authority = sample_authority(VerifierAuthorityResolutionClass::AuthorityResolvedRoot);
        let nodes = vec![
            sample_node(
                "node-a",
                VerificationVerdict::Trusted,
                &authority,
                ParityEvidenceState::Sufficient,
            ),
            sample_node(
                "node-b",
                VerificationVerdict::RejectedByPolicy,
                &authority,
                ParityEvidenceState::Insufficient,
            ),
        ];

        let report = analyze_determinism_incidents(&nodes);
        assert_eq!(report.determinism_incident_count, 0);
        assert_eq!(report.suppressed_incident_count, 1);
        assert_eq!(
            report.suppression_reason_counts.get("insufficient_evidence"),
            Some(&1usize)
        );
    }

    #[test]
    fn suppresses_false_incident_when_authority_is_historical_only() {
        let authority = sample_authority(VerifierAuthorityResolutionClass::AuthorityHistoricalOnly);
        let nodes = vec![
            sample_node(
                "node-a",
                VerificationVerdict::Trusted,
                &authority,
                ParityEvidenceState::Sufficient,
            ),
            sample_node(
                "node-b",
                VerificationVerdict::RejectedByPolicy,
                &authority,
                ParityEvidenceState::Sufficient,
            ),
        ];

        let report = analyze_determinism_incidents(&nodes);
        assert_eq!(report.determinism_incident_count, 0);
        assert_eq!(report.suppressed_incident_count, 1);
        assert_eq!(
            report.suppression_reason_counts.get("historical_only"),
            Some(&1usize)
        );
    }
}
