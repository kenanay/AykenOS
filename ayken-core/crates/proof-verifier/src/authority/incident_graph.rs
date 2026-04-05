use crate::authority::determinism_incident::{DeterminismIncident, DeterminismIncidentReport};
use crate::authority::parity::NodeParityOutcome;
use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    pub node_fingerprint: String,
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

pub const PHASE14_GRAPH_VERSION: &str = "v1";
pub const PARITY_INCIDENT_GRAPH_ARTIFACT: &str = "parity_incident_graph.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase14GraphProvenance {
    pub artifact_set_hash: String,
    pub source_runs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase14IncidentGraphEnvelope {
    pub graph_version: String,
    pub authority: String,
    pub env_hash: String,
    pub status: String,
    pub provenance: Phase14GraphProvenance,
    pub graph: IncidentGraph,
}

pub fn build_incident_graph(
    node_outcomes: &[NodeParityOutcome],
    incident_report: &DeterminismIncidentReport,
) -> Result<IncidentGraph, VerifierRuntimeError> {
    let mut nodes = node_outcomes
        .iter()
        .map(|node| {
            Ok(IncidentGraphNode {
                id: node.node_id.clone(),
                node_fingerprint: derive_node_fingerprint(node)?,
                surface_key: node.surface_key().to_string(),
                outcome_key: node.outcome_key().to_string(),
                verdict: verdict_label(&node.verdict).to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut edges = Vec::new();
    let mut seen_edges = BTreeSet::new();

    for incident in &incident_report.incidents {
        push_partition_edges(incident, &mut edges, &mut seen_edges)?;
        push_incident_edges(incident, &mut edges, &mut seen_edges)?;
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

    Ok(IncidentGraph {
        node_count: nodes.len(),
        edge_count: edges.len(),
        incident_count: incidents.len(),
        nodes,
        edges,
        incidents,
    })
}

pub fn derive_node_fingerprint(
    node: &NodeParityOutcome,
) -> Result<String, VerifierRuntimeError> {
    compute_canonical_value_hash(&json!({
        "subject_hash": node.subject_hash(),
        "context_hash": node.context_hash(),
        "authority_hash": node.authority_hash(),
        "node_id": node.node_id,
        "verifier_registry_snapshot_hash": node.verifier_registry_snapshot_hash(),
    }))
}

pub fn compute_graph_artifact_set_hash(
    artifacts: &[(&str, &Value)],
) -> Result<String, VerifierRuntimeError> {
    if artifacts.is_empty() {
        return Err(VerifierRuntimeError::config(
            "graph artifact set must not be empty",
        ));
    }

    let mut canonical_artifacts = artifacts
        .iter()
        .map(|(name, value)| {
            if name.is_empty() {
                return Err(VerifierRuntimeError::config(
                    "graph artifact name must not be empty",
                ));
            }
            if *name == PARITY_INCIDENT_GRAPH_ARTIFACT {
                return Err(VerifierRuntimeError::config(
                    "graph artifact set must exclude parity_incident_graph.json to avoid self-reference",
                ));
            }
            Ok(((*name).to_string(), compute_canonical_value_hash(value)?))
        })
        .collect::<Result<Vec<_>, _>>()?;
    canonical_artifacts.sort_by(|left, right| left.0.cmp(&right.0));
    if canonical_artifacts
        .windows(2)
        .any(|window| window[0].0 == window[1].0)
    {
        return Err(VerifierRuntimeError::config(
            "graph artifact set contains duplicate artifact names",
        ));
    }

    compute_canonical_value_hash(&json!(
        canonical_artifacts
            .iter()
            .map(|(name, content_hash)| {
                json!({
                    "artifact_name": name,
                    "content_hash": content_hash,
                })
            })
            .collect::<Vec<_>>()
    ))
}

pub fn build_phase14_incident_graph_envelope(
    graph: IncidentGraph,
    authority: &str,
    env_hash: &str,
    status: &str,
    artifact_set_hash: &str,
    source_runs: &[String],
) -> Result<Phase14IncidentGraphEnvelope, VerifierRuntimeError> {
    if authority.trim().is_empty() {
        return Err(VerifierRuntimeError::config(
            "graph authority must not be empty",
        ));
    }
    if env_hash.trim().is_empty() {
        return Err(VerifierRuntimeError::config(
            "graph env_hash must not be empty",
        ));
    }
    if status.trim().is_empty() {
        return Err(VerifierRuntimeError::config(
            "graph status must not be empty",
        ));
    }
    if artifact_set_hash.trim().is_empty() {
        return Err(VerifierRuntimeError::config(
            "graph artifact_set_hash must not be empty",
        ));
    }

    Ok(Phase14IncidentGraphEnvelope {
        graph_version: PHASE14_GRAPH_VERSION.to_string(),
        authority: authority.to_string(),
        env_hash: env_hash.to_string(),
        status: status.to_string(),
        provenance: Phase14GraphProvenance {
            artifact_set_hash: artifact_set_hash.to_string(),
            source_runs: normalize_source_runs(source_runs)?,
        },
        graph,
    })
}

fn push_partition_edges(
    incident: &DeterminismIncident,
    edges: &mut Vec<IncidentGraphEdge>,
    seen_edges: &mut BTreeSet<String>,
) -> Result<(), VerifierRuntimeError> {
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
            )?;
        }
    }
    Ok(())
}

fn push_incident_edges(
    incident: &DeterminismIncident,
    edges: &mut Vec<IncidentGraphEdge>,
    seen_edges: &mut BTreeSet<String>,
) -> Result<(), VerifierRuntimeError> {
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
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn push_edge(
    edges: &mut Vec<IncidentGraphEdge>,
    seen_edges: &mut BTreeSet<String>,
    left: &str,
    right: &str,
    edge_type: IncidentGraphEdgeType,
    incident_id: Option<String>,
    surface_key: Option<String>,
) -> Result<(), VerifierRuntimeError> {
    if edge_type == IncidentGraphEdgeType::Incident
        && incident_id
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(VerifierRuntimeError::config(
            "incident graph incident edges require a non-empty incident_id",
        ));
    }
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
        return Ok(());
    }
    edges.push(IncidentGraphEdge {
        from,
        to,
        edge_type,
        incident_id,
        surface_key,
    });
    Ok(())
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

fn normalize_source_runs(source_runs: &[String]) -> Result<Vec<String>, VerifierRuntimeError> {
    if source_runs.is_empty() {
        return Err(VerifierRuntimeError::config(
            "graph source_runs must not be empty",
        ));
    }
    let mut normalized = source_runs
        .iter()
        .map(|value| {
            if value.trim().is_empty() {
                Err(VerifierRuntimeError::config(
                    "graph source_runs entries must not be empty",
                ))
            } else {
                Ok(value.to_string())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn compute_canonical_value_hash(value: &Value) -> Result<String, VerifierRuntimeError> {
    let bytes = canonicalize_json_value(value)?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

#[cfg(test)]
mod tests {
    use super::{
        build_phase14_incident_graph_envelope, compute_graph_artifact_set_hash,
        derive_node_fingerprint, push_edge, IncidentGraph, IncidentGraphEdge, IncidentGraphEdgeType,
        IncidentGraphIncidentView, IncidentGraphNode, PARITY_INCIDENT_GRAPH_ARTIFACT,
        PHASE14_GRAPH_VERSION,
    };
    use crate::authority::parity::{
        build_node_parity_outcome, NodeParityOutcome, ParityArtifactForm, ParityEvidenceState,
    };
    use crate::types::{
        VerdictSubject, VerificationFinding, VerificationVerdict, VerifierAuthorityResolution,
        VerifierAuthorityResolutionClass,
    };
    use serde_json::json;
    use std::collections::BTreeSet;

    fn sample_authority() -> VerifierAuthorityResolution {
        VerifierAuthorityResolution {
            result_class: VerifierAuthorityResolutionClass::AuthorityResolvedRoot,
            requested_verifier_id: "node-a".to_string(),
            requested_authority_scope: vec!["distributed_receipt_acceptance".to_string()],
            authority_chain: vec!["root-a".to_string(), "node-a".to_string()],
            authority_chain_id: Some("sha256:chain-a".to_string()),
            effective_authority_scope: vec!["distributed_receipt_acceptance".to_string()],
            verifier_registry_snapshot_hash: "sha256:registry-snapshot".to_string(),
            findings: Vec::<VerificationFinding>::new(),
        }
    }

    fn sample_node(node_id: &str) -> NodeParityOutcome {
        build_node_parity_outcome(
            node_id,
            &format!("alias-{node_id}"),
            &VerdictSubject {
                bundle_id: "bundle-a".to_string(),
                trust_overlay_hash: "overlay-a".to_string(),
                policy_hash: "policy-a".to_string(),
                registry_snapshot_hash: "registry-a".to_string(),
            },
            "ctx-a",
            "1.0.0",
            &sample_authority(),
            &VerificationVerdict::Trusted,
            ParityArtifactForm::SignedReceipt,
            ParityEvidenceState::Sufficient,
        )
        .expect("sample node must build")
    }

    fn sample_graph() -> IncidentGraph {
        IncidentGraph {
            node_count: 1,
            edge_count: 0,
            incident_count: 0,
            nodes: vec![IncidentGraphNode {
                id: "node-a".to_string(),
                node_fingerprint: "sha256:fingerprint-a".to_string(),
                surface_key: "sha256:surface".to_string(),
                outcome_key: "sha256:outcome".to_string(),
                verdict: "TRUSTED".to_string(),
            }],
            edges: Vec::<IncidentGraphEdge>::new(),
            incidents: Vec::<IncidentGraphIncidentView>::new(),
        }
    }

    #[test]
    fn node_fingerprint_is_deterministic_and_node_id_sensitive() {
        let node_a = sample_node("node-a");
        let node_a_again = sample_node("node-a");
        let node_b = sample_node("node-b");

        let fingerprint_a = derive_node_fingerprint(&node_a).expect("fingerprint a");
        let fingerprint_a_again = derive_node_fingerprint(&node_a_again).expect("fingerprint a 2");
        let fingerprint_b = derive_node_fingerprint(&node_b).expect("fingerprint b");

        assert_eq!(fingerprint_a, fingerprint_a_again);
        assert_ne!(fingerprint_a, fingerprint_b);
    }

    #[test]
    fn graph_artifact_set_hash_is_order_independent_and_rejects_self_reference() {
        let artifact_a = json!({"status": "PASS", "row_count": 2});
        let artifact_b = json!({"status": "PASS", "node_count": 2});

        let left = compute_graph_artifact_set_hash(&[
            ("parity_report.json", &artifact_a),
            ("parity_determinism_incidents.json", &artifact_b),
        ])
        .expect("left hash");
        let right = compute_graph_artifact_set_hash(&[
            ("parity_determinism_incidents.json", &artifact_b),
            ("parity_report.json", &artifact_a),
        ])
        .expect("right hash");

        assert_eq!(left, right);
        assert!(compute_graph_artifact_set_hash(&[(PARITY_INCIDENT_GRAPH_ARTIFACT, &artifact_a)])
            .is_err());
    }

    #[test]
    fn envelope_normalizes_source_runs_and_keeps_v1() {
        let envelope = build_phase14_incident_graph_envelope(
            sample_graph(),
            "phase12-cross-node-parity",
            "sha256:env",
            "PASS",
            "sha256:artifact-set",
            &[
                "run-b".to_string(),
                "run-a".to_string(),
                "run-b".to_string(),
            ],
        )
        .expect("build envelope");

        assert_eq!(envelope.graph_version, PHASE14_GRAPH_VERSION);
        assert_eq!(
            envelope.provenance.source_runs,
            vec!["run-a".to_string(), "run-b".to_string()]
        );
    }

    #[test]
    fn incident_edges_require_incident_id() {
        let mut edges = Vec::new();
        let mut seen = BTreeSet::new();
        let error = push_edge(
            &mut edges,
            &mut seen,
            "node-a",
            "node-b",
            IncidentGraphEdgeType::Incident,
            None,
            Some("sha256:surface".to_string()),
        )
        .expect_err("incident edge without incident_id must fail");
        assert!(error
            .to_string()
            .contains("incident edges require a non-empty incident_id"));
    }
}
