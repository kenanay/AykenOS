use crate::authority::determinism_incident::{DeterminismIncident, DeterminismIncidentReport};
use crate::authority::parity::NodeParityOutcome;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentGraphEdgeType {
    SameOutcome,
    Incident,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentGraphNode {
    pub id: String,
    pub surface_key: String,
    pub outcome_key: String,
    pub verdict: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentGraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: IncidentGraphEdgeType,
    #[serde(default)]
    pub incident_id: Option<String>,
    #[serde(default)]
    pub surface_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentGraphIncidentView {
    pub incident_id: String,
    pub surface_key: String,
    pub severity: String,
    pub nodes: Vec<String>,
    pub node_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentGraph {
    pub node_count: usize,
    pub edge_count: usize,
    pub incident_count: usize,
    pub nodes: Vec<IncidentGraphNode>,
    pub edges: Vec<IncidentGraphEdge>,
    pub incidents: Vec<IncidentGraphIncidentView>,
}

pub fn build_incident_graph(
    node_outcomes: &[NodeParityOutcome],
    incident_report: &DeterminismIncidentReport,
) -> IncidentGraph {
    let mut nodes = node_outcomes
        .iter()
        .map(|node| IncidentGraphNode {
            id: node.node_id.clone(),
            surface_key: node.surface_key().to_string(),
            outcome_key: node.outcome_key().to_string(),
            verdict: verdict_label(&node.verdict).to_string(),
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut edges = Vec::new();
    let mut seen_edges = BTreeSet::new();

    for incident in &incident_report.incidents {
        push_partition_edges(incident, &mut edges, &mut seen_edges);
        push_incident_edges(incident, &mut edges, &mut seen_edges);
    }

    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| format!("{:?}", left.edge_type).cmp(&format!("{:?}", right.edge_type)))
            .then_with(|| left.incident_id.cmp(&right.incident_id))
    });

    let mut incidents = incident_report
        .incidents
        .iter()
        .map(|incident| IncidentGraphIncidentView {
            incident_id: incident.incident_id.clone(),
            surface_key: incident.surface_key.clone(),
            severity: severity_label(&incident.severity).to_string(),
            nodes: incident.nodes.clone(),
            node_count: incident.node_count,
        })
        .collect::<Vec<_>>();
    incidents.sort_by(|left, right| left.incident_id.cmp(&right.incident_id));

    IncidentGraph {
        node_count: nodes.len(),
        edge_count: edges.len(),
        incident_count: incidents.len(),
        nodes,
        edges,
        incidents,
    }
}

fn push_partition_edges(
    incident: &DeterminismIncident,
    edges: &mut Vec<IncidentGraphEdge>,
    seen_edges: &mut BTreeSet<String>,
) {
    for partition in &incident.outcome_partitions {
        for pair in pairwise_node_ids(&partition.node_ids) {
            push_edge(
                edges,
                seen_edges,
                pair.0,
                pair.1,
                IncidentGraphEdgeType::SameOutcome,
                Some(incident.incident_id.clone()),
                Some(incident.surface_key.clone()),
            );
        }
    }
}

fn push_incident_edges(
    incident: &DeterminismIncident,
    edges: &mut Vec<IncidentGraphEdge>,
    seen_edges: &mut BTreeSet<String>,
) {
    for left_index in 0..incident.outcome_partitions.len() {
        for right_index in (left_index + 1)..incident.outcome_partitions.len() {
            let left = &incident.outcome_partitions[left_index];
            let right = &incident.outcome_partitions[right_index];
            for left_node in &left.node_ids {
                for right_node in &right.node_ids {
                    push_edge(
                        edges,
                        seen_edges,
                        left_node,
                        right_node,
                        IncidentGraphEdgeType::Incident,
                        Some(incident.incident_id.clone()),
                        Some(incident.surface_key.clone()),
                    );
                }
            }
        }
    }
}

fn push_edge(
    edges: &mut Vec<IncidentGraphEdge>,
    seen_edges: &mut BTreeSet<String>,
    left: &str,
    right: &str,
    edge_type: IncidentGraphEdgeType,
    incident_id: Option<String>,
    surface_key: Option<String>,
) {
    let (from, to) = if left <= right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    };
    let edge_key = format!(
        "{}|{}|{:?}|{}",
        from,
        to,
        edge_type,
        incident_id.as_deref().unwrap_or("")
    );
    if !seen_edges.insert(edge_key) {
        return;
    }
    edges.push(IncidentGraphEdge {
        from,
        to,
        edge_type,
        incident_id,
        surface_key,
    });
}

fn pairwise_node_ids(node_ids: &[String]) -> Vec<(&str, &str)> {
    let mut pairs = Vec::new();
    for left_index in 0..node_ids.len() {
        for right_index in (left_index + 1)..node_ids.len() {
            pairs.push((
                node_ids[left_index].as_str(),
                node_ids[right_index].as_str(),
            ));
        }
    }
    pairs
}

fn verdict_label(verdict: &crate::types::VerificationVerdict) -> &'static str {
    match verdict {
        crate::types::VerificationVerdict::Trusted => "TRUSTED",
        crate::types::VerificationVerdict::Untrusted => "UNTRUSTED",
        crate::types::VerificationVerdict::Invalid => "INVALID",
        crate::types::VerificationVerdict::RejectedByPolicy => "REJECTED_BY_POLICY",
    }
}

fn severity_label(
    severity: &crate::authority::determinism_incident::DeterminismIncidentSeverity,
) -> &'static str {
    match severity {
        crate::authority::determinism_incident::DeterminismIncidentSeverity::PureDeterminismFailure => {
            "pure_determinism_failure"
        }
        crate::authority::determinism_incident::DeterminismIncidentSeverity::AuthorityDrift => {
            "authority_drift"
        }
        crate::authority::determinism_incident::DeterminismIncidentSeverity::ContextDrift => {
            "context_drift"
        }
        crate::authority::determinism_incident::DeterminismIncidentSeverity::SubjectDrift => {
            "subject_drift"
        }
        crate::authority::determinism_incident::DeterminismIncidentSeverity::Mixed => "mixed",
    }
}
