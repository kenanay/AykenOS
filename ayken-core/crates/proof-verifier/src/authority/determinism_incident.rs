use crate::canonical::digest::sha256_hex;
use crate::authority::parity::NodeParityOutcome;
use crate::types::VerificationVerdict;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterminismIncidentClass {
    DeterminismFailure,
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
    pub outcome_partitions: Vec<DeterminismOutcomePartition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismIncidentReport {
    pub node_count: usize,
    pub surface_partition_count: usize,
    pub determinism_incident_count: usize,
    pub incidents: Vec<DeterminismIncident>,
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

        incidents.push(DeterminismIncident {
            incident_id: compute_incident_id(&surface_key, &outcome_partitions),
            surface_key,
            nodes: nodes_list.clone(),
            outcome_keys,
            node_count: nodes_list.len(),
            outcome_partition_count: outcome_partitions.len(),
            subject_equal: unique_count(&nodes, |node| node.subject_hash()) == 1,
            context_equal: unique_count(&nodes, |node| node.context_hash()) == 1,
            authority_equal: unique_count(&nodes, |node| node.authority_hash()) == 1,
            drift_class: DeterminismIncidentClass::DeterminismFailure,
            outcome_partitions,
        });
    }

    incidents.sort_by(|left, right| {
        right
            .node_count
            .cmp(&left.node_count)
            .then_with(|| left.incident_id.cmp(&right.incident_id))
    });

    DeterminismIncidentReport {
        node_count: node_outcomes.len(),
        surface_partition_count,
        determinism_incident_count: incidents.len(),
        incidents,
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
