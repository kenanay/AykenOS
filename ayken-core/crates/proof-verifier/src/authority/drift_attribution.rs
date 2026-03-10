use crate::authority::parity::{NodeParityOutcome, ParityEvidenceState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftCause {
    NoDrift,
    SubjectDrift,
    ContextDrift,
    AuthorityDrift,
    AuthorityScopeDrift,
    AuthorityChainDrift,
    AuthorityHistoricalOnly,
    InsufficientEvidence,
    VerdictDrift,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftPartitionReport {
    pub partition_id: String,
    pub surface_key: String,
    pub node_ids: Vec<String>,
    pub outcome_partition_count: usize,
    pub subject_equal: bool,
    pub context_equal: bool,
    pub authority_equal: bool,
    pub verdict_split: bool,
    pub historical_only_present: bool,
    pub insufficient_evidence_present: bool,
    pub primary_cause: DriftCause,
    pub secondary_causes: Vec<DriftCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftReport {
    pub node_count: usize,
    pub surface_partition_count: usize,
    pub outcome_partition_count: usize,
    #[serde(default)]
    pub baseline_partition_id: Option<String>,
    #[serde(default)]
    pub baseline_surface_key: Option<String>,
    pub historical_authority_island_count: usize,
    pub insufficient_evidence_island_count: usize,
    pub historical_authority_islands: Vec<DriftIslandReport>,
    pub insufficient_evidence_islands: Vec<DriftIslandReport>,
    pub partition_reports: Vec<DriftPartitionReport>,
    pub primary_cause_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftIslandReport {
    pub partition_id: String,
    pub surface_key: String,
    pub node_ids: Vec<String>,
    pub node_count: usize,
    pub island_type: DriftCause,
}

struct SurfacePartition<'a> {
    surface_key: String,
    nodes: Vec<&'a NodeParityOutcome>,
}

pub fn analyze_parity_drift(node_outcomes: &[NodeParityOutcome]) -> DriftReport {
    let partitions = partition_by_surface(node_outcomes);
    let baseline = partitions.first();
    let baseline_partition_id = baseline.map(|_| "partition_1".to_string());
    let baseline_surface_key = baseline.map(|partition| partition.surface_key.clone());
    let outcome_partition_count = unique_outcome_partition_count(node_outcomes);

    let mut partition_reports = Vec::new();
    let mut primary_cause_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut historical_authority_islands = Vec::new();
    let mut insufficient_evidence_islands = Vec::new();

    for (index, partition) in partitions.iter().enumerate() {
        let report = analyze_surface_partition(index + 1, partition, baseline);
        *primary_cause_counts
            .entry(drift_cause_label(&report.primary_cause).to_string())
            .or_insert(0) += 1;
        if report.historical_only_present {
            historical_authority_islands.push(DriftIslandReport::from_partition(
                &report,
                DriftCause::AuthorityHistoricalOnly,
            ));
        }
        if report.insufficient_evidence_present {
            insufficient_evidence_islands.push(DriftIslandReport::from_partition(
                &report,
                DriftCause::InsufficientEvidence,
            ));
        }
        partition_reports.push(report);
    }

    DriftReport {
        node_count: node_outcomes.len(),
        surface_partition_count: partition_reports.len(),
        outcome_partition_count,
        baseline_partition_id,
        baseline_surface_key,
        historical_authority_island_count: historical_authority_islands.len(),
        insufficient_evidence_island_count: insufficient_evidence_islands.len(),
        historical_authority_islands,
        insufficient_evidence_islands,
        partition_reports,
        primary_cause_counts,
    }
}

impl DriftIslandReport {
    fn from_partition(partition: &DriftPartitionReport, island_type: DriftCause) -> Self {
        Self {
            partition_id: partition.partition_id.clone(),
            surface_key: partition.surface_key.clone(),
            node_ids: partition.node_ids.clone(),
            node_count: partition.node_ids.len(),
            island_type,
        }
    }
}

fn partition_by_surface<'a>(node_outcomes: &'a [NodeParityOutcome]) -> Vec<SurfacePartition<'a>> {
    let mut grouped: BTreeMap<String, Vec<&'a NodeParityOutcome>> = BTreeMap::new();
    for node in node_outcomes {
        grouped
            .entry(node.surface_key().to_string())
            .or_default()
            .push(node);
    }

    let mut partitions: Vec<SurfacePartition<'a>> = grouped
        .into_iter()
        .map(|(surface_key, mut nodes)| {
            nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
            SurfacePartition { surface_key, nodes }
        })
        .collect();

    partitions.sort_by(|left, right| {
        right
            .nodes
            .len()
            .cmp(&left.nodes.len())
            .then_with(|| left.surface_key.cmp(&right.surface_key))
    });
    partitions
}

fn analyze_surface_partition(
    partition_index: usize,
    partition: &SurfacePartition<'_>,
    baseline: Option<&SurfacePartition<'_>>,
) -> DriftPartitionReport {
    let node_ids = sorted_node_ids(&partition.nodes);
    let outcome_partition_count = unique_count(&partition.nodes, |node| node.outcome_key());
    let verdict_split = outcome_partition_count > 1;
    let historical_only_present = partition.nodes.iter().any(|node| node.is_historical_only());
    let insufficient_evidence_present = partition
        .nodes
        .iter()
        .any(|node| node.evidence_state() == &ParityEvidenceState::Insufficient);

    let (subject_equal, context_equal, authority_equal) = if let Some(baseline_partition) = baseline
    {
        let reference = baseline_partition
            .nodes
            .first()
            .copied()
            .expect("baseline partition must have at least one node");
        let current = partition
            .nodes
            .first()
            .copied()
            .expect("surface partition must have at least one node");
        (
            current.subject_hash() == reference.subject_hash(),
            current.context_hash() == reference.context_hash(),
            current.authority_hash() == reference.authority_hash(),
        )
    } else {
        (true, true, true)
    };

    let (primary_cause, secondary_causes) = classify_partition_causes(
        partition,
        baseline,
        subject_equal,
        context_equal,
        authority_equal,
        verdict_split,
        historical_only_present,
        insufficient_evidence_present,
    );

    DriftPartitionReport {
        partition_id: format!("partition_{partition_index}"),
        surface_key: partition.surface_key.clone(),
        node_ids,
        outcome_partition_count,
        subject_equal,
        context_equal,
        authority_equal,
        verdict_split,
        historical_only_present,
        insufficient_evidence_present,
        primary_cause,
        secondary_causes,
    }
}

fn classify_partition_causes(
    partition: &SurfacePartition<'_>,
    baseline: Option<&SurfacePartition<'_>>,
    subject_equal: bool,
    context_equal: bool,
    authority_equal: bool,
    verdict_split: bool,
    historical_only_present: bool,
    insufficient_evidence_present: bool,
) -> (DriftCause, Vec<DriftCause>) {
    let mut causes = Vec::new();

    if !subject_equal {
        causes.push(DriftCause::SubjectDrift);
    }
    if !context_equal {
        causes.push(DriftCause::ContextDrift);
    }
    if !authority_equal {
        causes.push(classify_authority_drift(partition, baseline));
    }
    if historical_only_present {
        causes.push(DriftCause::AuthorityHistoricalOnly);
    }
    if insufficient_evidence_present {
        causes.push(DriftCause::InsufficientEvidence);
    }
    if verdict_split {
        causes.push(DriftCause::VerdictDrift);
    }

    if causes.is_empty() {
        return (DriftCause::NoDrift, Vec::new());
    }

    if causes.len() == 1 {
        return (causes[0].clone(), Vec::new());
    }

    let prioritized = [
        DriftCause::InsufficientEvidence,
        DriftCause::AuthorityHistoricalOnly,
        DriftCause::SubjectDrift,
        DriftCause::ContextDrift,
        DriftCause::AuthorityScopeDrift,
        DriftCause::AuthorityChainDrift,
        DriftCause::AuthorityDrift,
        DriftCause::VerdictDrift,
    ];

    for candidate in prioritized {
        if let Some(position) = causes.iter().position(|cause| *cause == candidate) {
            let primary = causes.remove(position);
            return (primary, causes);
        }
    }

    (DriftCause::Mixed, causes)
}

fn classify_authority_drift(
    partition: &SurfacePartition<'_>,
    baseline: Option<&SurfacePartition<'_>>,
) -> DriftCause {
    let Some(baseline_partition) = baseline else {
        return DriftCause::AuthorityDrift;
    };
    let reference = baseline_partition
        .nodes
        .first()
        .copied()
        .expect("baseline partition must have at least one node");
    let current = partition
        .nodes
        .first()
        .copied()
        .expect("surface partition must have at least one node");

    if current.effective_authority_scope() != reference.effective_authority_scope() {
        return DriftCause::AuthorityScopeDrift;
    }

    if current.authority_chain_id() != reference.authority_chain_id() {
        return DriftCause::AuthorityChainDrift;
    }

    DriftCause::AuthorityDrift
}

fn sorted_node_ids(nodes: &[&NodeParityOutcome]) -> Vec<String> {
    let mut ids: Vec<String> = nodes.iter().map(|node| node.node_id.clone()).collect();
    ids.sort();
    ids
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

fn unique_outcome_partition_count(node_outcomes: &[NodeParityOutcome]) -> usize {
    node_outcomes
        .iter()
        .map(|node| node.outcome_key().to_string())
        .collect::<BTreeSet<_>>()
        .len()
}

fn drift_cause_label(cause: &DriftCause) -> &'static str {
    match cause {
        DriftCause::NoDrift => "no_drift",
        DriftCause::SubjectDrift => "subject_drift",
        DriftCause::ContextDrift => "context_drift",
        DriftCause::AuthorityDrift => "authority_drift",
        DriftCause::AuthorityScopeDrift => "authority_scope_drift",
        DriftCause::AuthorityChainDrift => "authority_chain_drift",
        DriftCause::AuthorityHistoricalOnly => "authority_historical_only",
        DriftCause::InsufficientEvidence => "insufficient_evidence",
        DriftCause::VerdictDrift => "verdict_drift",
        DriftCause::Mixed => "mixed",
    }
}
