use crate::authority::parity::NodeParityOutcome;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityClusterKind {
    Current,
    CurrentDrift,
    HistoricalOnly,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityCluster {
    pub authority_cluster_key: String,
    pub authority_chain_id: String,
    pub effective_authority_scope: Vec<String>,
    pub node_ids: Vec<String>,
    pub node_count: usize,
    pub kind: AuthorityClusterKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityDriftTopology {
    pub node_count: usize,
    pub authority_cluster_count: usize,
    #[serde(default)]
    pub dominant_authority_chain_id: Option<String>,
    #[serde(default)]
    pub dominant_authority_cluster_key: Option<String>,
    pub drifted_node_count: usize,
    pub historical_only_node_count: usize,
    pub unresolved_node_count: usize,
    pub clusters: Vec<AuthorityCluster>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoritySuppressionRule {
    ScopeAlias,
    HistoricalShadow,
    RegistrySkew,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppressedAuthorityDrift {
    pub rule: AuthoritySuppressionRule,
    #[serde(default)]
    pub authority_chain_id: Option<String>,
    pub node_ids: Vec<String>,
    pub node_count: usize,
    #[serde(default)]
    pub raw_effective_authority_scopes: Vec<Vec<String>>,
    #[serde(default)]
    pub verifier_registry_snapshot_hashes: Vec<String>,
    #[serde(default)]
    pub suppressed_against_cluster_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritySuppressionReport {
    pub node_count: usize,
    pub suppression_guard_active: bool,
    pub suppressed_drift_count: usize,
    pub rule_counts: BTreeMap<String, usize>,
    pub suppressed_drifts: Vec<SuppressedAuthorityDrift>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthorityClusterIdentity {
    authority_chain_id: String,
    effective_authority_scope: Vec<String>,
}

#[derive(Debug, Clone)]
struct CurrentAuthorityGroup<'a> {
    cluster_key: String,
    identity: AuthorityClusterIdentity,
    nodes: Vec<&'a NodeParityOutcome>,
}

pub fn build_authority_drift_topology(
    node_outcomes: &[NodeParityOutcome],
) -> AuthorityDriftTopology {
    let mut grouped: BTreeMap<String, Vec<&NodeParityOutcome>> = BTreeMap::new();

    for node in node_outcomes {
        grouped
            .entry(authority_cluster_key(node))
            .or_default()
            .push(node);
    }

    let dominant_authority_cluster_key = grouped
        .iter()
        .filter(|(key, _)| !is_historical_cluster_key(key) && !is_unresolved_cluster_key(key))
        .max_by(|(left_key, left_nodes), (right_key, right_nodes)| {
            left_nodes
                .len()
                .cmp(&right_nodes.len())
                .then_with(|| right_key.cmp(left_key))
        })
        .map(|(key, _)| key.clone());

    let dominant_authority_chain_id = dominant_authority_cluster_key
        .as_deref()
        .and_then(parse_cluster_identity)
        .map(|identity| identity.authority_chain_id);

    let mut clusters = Vec::new();
    let mut drifted_node_count = 0usize;
    let mut historical_only_node_count = 0usize;
    let mut unresolved_node_count = 0usize;

    for (cluster_key, mut nodes) in grouped {
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        let node_ids = nodes
            .iter()
            .map(|node| node.node_id.clone())
            .collect::<Vec<_>>();
        let node_count = node_ids.len();

        let (authority_chain_id, effective_authority_scope, kind) =
            if is_historical_cluster_key(&cluster_key) {
                historical_only_node_count += node_count;
                (
                    "historical-only".to_string(),
                    Vec::new(),
                    AuthorityClusterKind::HistoricalOnly,
                )
            } else if is_unresolved_cluster_key(&cluster_key) {
                drifted_node_count += node_count;
                unresolved_node_count += node_count;
                (
                    "unresolved-authority".to_string(),
                    Vec::new(),
                    AuthorityClusterKind::Unresolved,
                )
            } else {
                let identity = parse_cluster_identity(&cluster_key)
                    .expect("current authority cluster keys must parse");
                if Some(cluster_key.clone()) == dominant_authority_cluster_key {
                    (
                        identity.authority_chain_id,
                        identity.effective_authority_scope,
                        AuthorityClusterKind::Current,
                    )
                } else {
                    drifted_node_count += node_count;
                    (
                        identity.authority_chain_id,
                        identity.effective_authority_scope,
                        AuthorityClusterKind::CurrentDrift,
                    )
                }
            };

        clusters.push(AuthorityCluster {
            authority_cluster_key: cluster_key,
            authority_chain_id,
            effective_authority_scope,
            node_ids,
            node_count,
            kind,
        });
    }

    clusters.sort_by(|left, right| {
        right
            .node_count
            .cmp(&left.node_count)
            .then_with(|| left.authority_cluster_key.cmp(&right.authority_cluster_key))
    });

    AuthorityDriftTopology {
        node_count: node_outcomes.len(),
        authority_cluster_count: clusters.len(),
        dominant_authority_chain_id,
        dominant_authority_cluster_key,
        drifted_node_count,
        historical_only_node_count,
        unresolved_node_count,
        clusters,
    }
}

pub fn analyze_authority_drift_suppressions(
    node_outcomes: &[NodeParityOutcome],
) -> AuthoritySuppressionReport {
    let current_groups = build_current_authority_groups(node_outcomes);
    let dominant_authority_cluster_key = current_groups
        .iter()
        .max_by(|left, right| {
            left.nodes
                .len()
                .cmp(&right.nodes.len())
                .then_with(|| right.cluster_key.cmp(&left.cluster_key))
        })
        .map(|group| group.cluster_key.clone());

    let mut suppressed_drifts = Vec::new();
    suppressed_drifts.extend(build_scope_alias_suppressions(&current_groups));
    suppressed_drifts.extend(build_registry_skew_suppressions(&current_groups));
    suppressed_drifts.extend(build_historical_shadow_suppressions(
        node_outcomes,
        &current_groups,
        dominant_authority_cluster_key.as_deref(),
    ));

    suppressed_drifts.sort_by(|left, right| {
        suppression_rule_label(&left.rule)
            .cmp(suppression_rule_label(&right.rule))
            .then_with(|| left.node_count.cmp(&right.node_count).reverse())
            .then_with(|| left.node_ids.cmp(&right.node_ids))
    });

    let mut rule_counts = BTreeMap::new();
    for suppressed in &suppressed_drifts {
        let key = suppression_rule_label(&suppressed.rule).to_string();
        *rule_counts.entry(key).or_insert(0) += 1;
    }

    AuthoritySuppressionReport {
        node_count: node_outcomes.len(),
        suppression_guard_active: true,
        suppressed_drift_count: suppressed_drifts.len(),
        rule_counts,
        suppressed_drifts,
    }
}

fn build_current_authority_groups<'a>(
    node_outcomes: &'a [NodeParityOutcome],
) -> Vec<CurrentAuthorityGroup<'a>> {
    let mut grouped: BTreeMap<String, Vec<&NodeParityOutcome>> = BTreeMap::new();
    for node in node_outcomes {
        if node.is_historical_only() || node.authority_chain_id().is_none() {
            continue;
        }
        grouped
            .entry(authority_cluster_key(node))
            .or_default()
            .push(node);
    }

    let mut groups = Vec::new();
    for (cluster_key, mut nodes) in grouped {
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        let identity = parse_cluster_identity(&cluster_key)
            .expect("current authority cluster keys must parse");
        groups.push(CurrentAuthorityGroup {
            cluster_key,
            identity,
            nodes,
        });
    }

    groups.sort_by(|left, right| left.cluster_key.cmp(&right.cluster_key));
    groups
}

fn build_scope_alias_suppressions(
    current_groups: &[CurrentAuthorityGroup<'_>],
) -> Vec<SuppressedAuthorityDrift> {
    let mut suppressions = Vec::new();

    for group in current_groups {
        let raw_scope_sets = unique_scope_sets(&group.nodes);
        if raw_scope_sets.len() <= 1 {
            continue;
        }

        suppressions.push(SuppressedAuthorityDrift {
            rule: AuthoritySuppressionRule::ScopeAlias,
            authority_chain_id: Some(group.identity.authority_chain_id.clone()),
            node_ids: group
                .nodes
                .iter()
                .map(|node| node.node_id.clone())
                .collect(),
            node_count: group.nodes.len(),
            raw_effective_authority_scopes: raw_scope_sets,
            verifier_registry_snapshot_hashes: unique_registry_snapshot_hashes(&group.nodes),
            suppressed_against_cluster_key: Some(group.cluster_key.clone()),
        });
    }

    suppressions
}

fn build_registry_skew_suppressions(
    current_groups: &[CurrentAuthorityGroup<'_>],
) -> Vec<SuppressedAuthorityDrift> {
    let mut suppressions = Vec::new();

    for group in current_groups {
        let registry_hashes = unique_registry_snapshot_hashes(&group.nodes);
        if registry_hashes.len() <= 1 {
            continue;
        }

        suppressions.push(SuppressedAuthorityDrift {
            rule: AuthoritySuppressionRule::RegistrySkew,
            authority_chain_id: Some(group.identity.authority_chain_id.clone()),
            node_ids: group
                .nodes
                .iter()
                .map(|node| node.node_id.clone())
                .collect(),
            node_count: group.nodes.len(),
            raw_effective_authority_scopes: unique_scope_sets(&group.nodes),
            verifier_registry_snapshot_hashes: registry_hashes,
            suppressed_against_cluster_key: Some(group.cluster_key.clone()),
        });
    }

    suppressions
}

fn build_historical_shadow_suppressions(
    node_outcomes: &[NodeParityOutcome],
    current_groups: &[CurrentAuthorityGroup<'_>],
    dominant_authority_cluster_key: Option<&str>,
) -> Vec<SuppressedAuthorityDrift> {
    let mut current_by_chain: BTreeMap<String, String> = BTreeMap::new();
    for group in current_groups {
        current_by_chain
            .entry(group.identity.authority_chain_id.clone())
            .or_insert_with(|| group.cluster_key.clone());
    }

    let mut historical_by_chain: BTreeMap<String, Vec<&NodeParityOutcome>> = BTreeMap::new();
    for node in node_outcomes {
        if !node.is_historical_only() {
            continue;
        }
        let Some(authority_chain_id) = node.authority_chain_id() else {
            continue;
        };
        if !current_by_chain.contains_key(authority_chain_id) {
            continue;
        }
        historical_by_chain
            .entry(authority_chain_id.to_string())
            .or_default()
            .push(node);
    }

    let mut suppressions = Vec::new();
    for (authority_chain_id, mut nodes) in historical_by_chain {
        nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
        let node_ids = nodes
            .iter()
            .map(|node| node.node_id.clone())
            .collect::<Vec<_>>();
        let suppressed_against_cluster_key = dominant_authority_cluster_key
            .filter(|current| {
                current_by_chain
                    .get(&authority_chain_id)
                    .map(|value| value.as_str())
                    == Some(*current)
            })
            .map(ToString::to_string)
            .or_else(|| current_by_chain.get(&authority_chain_id).cloned());
        suppressions.push(SuppressedAuthorityDrift {
            rule: AuthoritySuppressionRule::HistoricalShadow,
            authority_chain_id: Some(authority_chain_id),
            node_count: node_ids.len(),
            node_ids,
            raw_effective_authority_scopes: vec![Vec::new()],
            verifier_registry_snapshot_hashes: unique_registry_snapshot_hashes(&nodes),
            suppressed_against_cluster_key,
        });
    }

    suppressions
}

fn authority_cluster_key(node: &NodeParityOutcome) -> String {
    if node.is_historical_only() {
        return "historical-only".to_string();
    }

    if let Some(authority_chain_id) = node.authority_chain_id() {
        return format!(
            "chain:{}|scope:{}",
            authority_chain_id,
            normalize_scope(node.effective_authority_scope())
        );
    }

    "unresolved-authority".to_string()
}

fn normalize_scope(scope: &[String]) -> String {
    if scope.is_empty() {
        return "<none>".to_string();
    }

    let mut sorted = scope
        .iter()
        .map(|item| canonicalize_scope_token(item))
        .collect::<Vec<_>>();
    sorted.sort();
    sorted.dedup();
    sorted.join(",")
}

fn canonicalize_scope_token(token: &str) -> String {
    let canonical = token.trim().to_ascii_lowercase().replace('_', "-");
    match canonical.as_str() {
        "*" | "root" | "global" | "all" => "global".to_string(),
        _ => canonical,
    }
}

fn unique_scope_sets(nodes: &[&NodeParityOutcome]) -> Vec<Vec<String>> {
    let mut unique = BTreeSet::new();
    for node in nodes {
        let mut raw_scope = node.effective_authority_scope().to_vec();
        raw_scope.sort();
        unique.insert(raw_scope);
    }
    unique.into_iter().collect()
}

fn unique_registry_snapshot_hashes(nodes: &[&NodeParityOutcome]) -> Vec<String> {
    let mut unique = nodes
        .iter()
        .map(|node| node.verifier_registry_snapshot_hash().to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    unique.sort();
    unique
}

fn is_historical_cluster_key(key: &str) -> bool {
    key == "historical-only"
}

fn is_unresolved_cluster_key(key: &str) -> bool {
    key == "unresolved-authority"
}

fn parse_cluster_identity(key: &str) -> Option<AuthorityClusterIdentity> {
    let rest = key.strip_prefix("chain:")?;
    let (authority_chain_id, scope) = rest.split_once("|scope:")?;
    let effective_authority_scope = if scope == "<none>" {
        Vec::new()
    } else {
        scope
            .split(',')
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    };
    Some(AuthorityClusterIdentity {
        authority_chain_id: authority_chain_id.to_string(),
        effective_authority_scope,
    })
}

fn suppression_rule_label(rule: &AuthoritySuppressionRule) -> &'static str {
    match rule {
        AuthoritySuppressionRule::ScopeAlias => "scope_alias",
        AuthoritySuppressionRule::HistoricalShadow => "historical_shadow",
        AuthoritySuppressionRule::RegistrySkew => "registry_skew",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::parity::{
        build_node_parity_outcome, ParityArtifactForm, ParityEvidenceState,
    };
    use crate::types::{
        VerdictSubject, VerificationVerdict, VerifierAuthorityResolution,
        VerifierAuthorityResolutionClass,
    };

    fn sample_subject() -> VerdictSubject {
        VerdictSubject {
            bundle_id: "bundle-1".to_string(),
            trust_overlay_hash: "overlay-1".to_string(),
            policy_hash: "policy-1".to_string(),
            registry_snapshot_hash: "registry-1".to_string(),
        }
    }

    fn sample_authority(
        result_class: VerifierAuthorityResolutionClass,
        chain_id: Option<&str>,
        scope: &[&str],
        verifier_registry_snapshot_hash: &str,
    ) -> VerifierAuthorityResolution {
        VerifierAuthorityResolution {
            result_class,
            requested_verifier_id: "verifier-a".to_string(),
            requested_authority_scope: scope.iter().map(|item| item.to_string()).collect(),
            authority_chain: chain_id
                .map(|value| vec!["root-a".to_string(), value.to_string()])
                .unwrap_or_default(),
            authority_chain_id: chain_id.map(ToString::to_string),
            effective_authority_scope: scope.iter().map(|item| item.to_string()).collect(),
            verifier_registry_snapshot_hash: verifier_registry_snapshot_hash.to_string(),
            findings: Vec::new(),
        }
    }

    fn sample_node(node_id: &str, authority: &VerifierAuthorityResolution) -> NodeParityOutcome {
        build_node_parity_outcome(
            node_id,
            node_id,
            &sample_subject(),
            "context-1",
            "contract-v1",
            authority,
            &VerificationVerdict::Trusted,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .expect("build node parity outcome")
    }

    #[test]
    fn groups_current_drift_historical_and_unresolved_clusters() {
        let current = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityResolvedDelegated,
            Some("chain-a"),
            &["distributed_receipt_acceptance"],
            "registry-1",
        );
        let current_scope_drift = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityResolvedDelegated,
            Some("chain-a"),
            &["parity-reporter"],
            "registry-1",
        );
        let alt_current = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityResolvedDelegated,
            Some("chain-b"),
            &["distributed_receipt_acceptance"],
            "registry-1",
        );
        let historical = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityHistoricalOnly,
            Some("chain-a"),
            &["distributed_receipt_acceptance"],
            "registry-1",
        );
        let unresolved = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityNoValidChain,
            None,
            &["distributed_receipt_acceptance"],
            "registry-1",
        );

        let nodes = vec![
            sample_node("node-a", &current),
            sample_node("node-b", &current),
            sample_node("node-c", &current_scope_drift),
            sample_node("node-d", &alt_current),
            sample_node("node-e", &historical),
            sample_node("node-f", &unresolved),
        ];

        let topology = build_authority_drift_topology(&nodes);
        assert_eq!(topology.node_count, 6);
        assert_eq!(topology.authority_cluster_count, 5);
        assert_eq!(
            topology.dominant_authority_chain_id.as_deref(),
            Some("chain-a")
        );
        assert_eq!(topology.drifted_node_count, 3);
        assert_eq!(topology.historical_only_node_count, 1);
        assert_eq!(topology.unresolved_node_count, 1);
        assert_eq!(topology.clusters[0].kind, AuthorityClusterKind::Current);
        assert_eq!(topology.clusters[0].node_count, 2);
    }

    #[test]
    fn suppresses_scope_alias_registry_skew_and_historical_shadow() {
        let current = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityResolvedDelegated,
            Some("chain-a"),
            &["global"],
            "registry-1",
        );
        let scope_alias = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityResolvedDelegated,
            Some("chain-a"),
            &["*"],
            "registry-2",
        );
        let historical = sample_authority(
            VerifierAuthorityResolutionClass::AuthorityHistoricalOnly,
            Some("chain-a"),
            &["global"],
            "registry-3",
        );

        let nodes = vec![
            sample_node("node-a", &current),
            sample_node("node-b", &scope_alias),
            sample_node("node-c", &historical),
        ];

        let report = analyze_authority_drift_suppressions(&nodes);
        assert_eq!(report.node_count, 3);
        assert!(report.suppression_guard_active);
        assert_eq!(report.suppressed_drift_count, 3);
        assert_eq!(report.rule_counts.get("scope_alias"), Some(&1));
        assert_eq!(report.rule_counts.get("registry_skew"), Some(&1));
        assert_eq!(report.rule_counts.get("historical_shadow"), Some(&1));
    }
}
