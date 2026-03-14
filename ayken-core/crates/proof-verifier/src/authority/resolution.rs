use crate::authority::snapshot::validate_verifier_trust_registry_snapshot;
use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::types::{
    VerificationFinding, VerifierAuthorityResolution, VerifierAuthorityResolutionClass,
    VerifierAuthorityState, VerifierDelegationEdge, VerifierTrustRegistrySnapshot,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

// Phase-12 depth is counted as explicit delegation hops from an explicit root.
// A root has depth 0, its direct delegate has depth 1, and so on.
const MAX_DELEGATION_DEPTH: usize = 8;

pub fn resolve_verifier_authority(
    snapshot: &VerifierTrustRegistrySnapshot,
    requested_verifier_id: &str,
    requested_authority_scope: &[String],
) -> Result<VerifierAuthorityResolution, VerifierRuntimeError> {
    let mut findings = Vec::new();
    let validation = validate_verifier_trust_registry_snapshot(snapshot)?;
    findings.extend(validation.findings);

    let requested_scope = canonical_scope(requested_authority_scope);
    if requested_scope.is_empty() {
        findings.push(VerificationFinding::error(
            "PV0913",
            "requested verifier authority scope must not be empty",
        ));
        return Ok(build_resolution(
            VerifierAuthorityResolutionClass::AuthorityNoValidChain,
            requested_verifier_id,
            &requested_scope,
            Vec::new(),
            None,
            Vec::new(),
            findings,
            snapshot,
        ));
    }

    if !snapshot.verifiers.contains_key(requested_verifier_id) {
        findings.push(VerificationFinding::error(
            "PV0910",
            format!("requested verifier authority node {requested_verifier_id} is missing"),
        ));
        return Ok(build_resolution(
            VerifierAuthorityResolutionClass::AuthorityNoValidChain,
            requested_verifier_id,
            &requested_scope,
            Vec::new(),
            None,
            Vec::new(),
            findings,
            snapshot,
        ));
    }

    if let Some(class) = validate_graph(snapshot, &requested_scope, &mut findings)? {
        return Ok(build_resolution(
            class,
            requested_verifier_id,
            &requested_scope,
            Vec::new(),
            None,
            Vec::new(),
            findings,
            snapshot,
        ));
    }

    let enumeration =
        enumerate_candidate_chains(snapshot, requested_verifier_id, &requested_scope)?;
    if enumeration.depth_exceeded {
        findings.push(VerificationFinding::error(
            "PV0911",
            format!(
                "verifier authority resolution exceeded max delegation depth {} for requested verifier {}",
                MAX_DELEGATION_DEPTH, requested_verifier_id
            ),
        ));
        return Ok(build_resolution(
            VerifierAuthorityResolutionClass::AuthorityGraphDepthExceeded,
            requested_verifier_id,
            &requested_scope,
            Vec::new(),
            None,
            Vec::new(),
            findings,
            snapshot,
        ));
    }

    let chains = enumeration.chains;
    let current_chains: Vec<CandidateChain> = chains
        .iter()
        .filter(|chain| !chain.historical_only)
        .cloned()
        .collect();
    let historical_chains: Vec<CandidateChain> = chains
        .iter()
        .filter(|chain| chain.historical_only)
        .cloned()
        .collect();

    if current_chains.len() > 1 {
        findings.push(VerificationFinding::error(
            "PV0909",
            "multiple surviving current authority parent chains remain after filtering",
        ));
        return Ok(build_resolution(
            VerifierAuthorityResolutionClass::AuthorityGraphAmbiguous,
            requested_verifier_id,
            &requested_scope,
            Vec::new(),
            None,
            Vec::new(),
            findings,
            snapshot,
        ));
    }
    if current_chains.is_empty() && historical_chains.len() > 1 {
        findings.push(VerificationFinding::error(
            "PV0909",
            "multiple surviving historical authority parent chains remain after filtering",
        ));
        return Ok(build_resolution(
            VerifierAuthorityResolutionClass::AuthorityGraphAmbiguous,
            requested_verifier_id,
            &requested_scope,
            Vec::new(),
            None,
            Vec::new(),
            findings,
            snapshot,
        ));
    }

    if let Some(chain) = current_chains.first() {
        return Ok(build_resolution(
            if chain.chain.len() == 1 {
                VerifierAuthorityResolutionClass::AuthorityResolvedRoot
            } else {
                VerifierAuthorityResolutionClass::AuthorityResolvedDelegated
            },
            requested_verifier_id,
            &requested_scope,
            chain.chain.clone(),
            Some(chain.authority_chain_id.clone()),
            chain.effective_authority_scope.clone(),
            findings,
            snapshot,
        ));
    }
    if let Some(chain) = historical_chains.first() {
        return Ok(build_resolution(
            VerifierAuthorityResolutionClass::AuthorityHistoricalOnly,
            requested_verifier_id,
            &requested_scope,
            chain.chain.clone(),
            Some(chain.authority_chain_id.clone()),
            chain.effective_authority_scope.clone(),
            findings,
            snapshot,
        ));
    }

    findings.push(VerificationFinding::error(
        "PV0910",
        "no valid authority chain remains after validation and filtering",
    ));
    Ok(build_resolution(
        VerifierAuthorityResolutionClass::AuthorityNoValidChain,
        requested_verifier_id,
        &requested_scope,
        Vec::new(),
        None,
        Vec::new(),
        findings,
        snapshot,
    ))
}

#[derive(Debug, Clone)]
struct CandidateChain {
    chain: Vec<String>,
    authority_chain_id: String,
    effective_authority_scope: Vec<String>,
    historical_only: bool,
}

#[derive(Debug, Clone)]
struct EnumerationResult {
    chains: Vec<CandidateChain>,
    depth_exceeded: bool,
}

fn validate_graph(
    snapshot: &VerifierTrustRegistrySnapshot,
    requested_scope: &[String],
    findings: &mut Vec<VerificationFinding>,
) -> Result<Option<VerifierAuthorityResolutionClass>, VerifierRuntimeError> {
    for edge in &snapshot.delegation_edges {
        if edge.parent_verifier_id == edge.delegate_verifier_id {
            findings.push(VerificationFinding::error(
                "PV0906",
                format!(
                    "verifier authority edge {} -> {} is self-delegation",
                    edge.parent_verifier_id, edge.delegate_verifier_id
                ),
            ));
            return Ok(Some(VerifierAuthorityResolutionClass::AuthorityGraphCycle));
        }
        let Some(parent) = snapshot.verifiers.get(&edge.parent_verifier_id) else {
            continue;
        };
        let Some(delegate) = snapshot.verifiers.get(&edge.delegate_verifier_id) else {
            continue;
        };
        if !scope_is_subset(&edge.delegated_scope, &parent.authority_scope)
            || !scope_is_subset(&edge.delegated_scope, &delegate.authority_scope)
        {
            findings.push(VerificationFinding::error(
                "PV0908",
                format!(
                    "delegated authority scope for {} -> {} widens beyond declared parent or delegate scope",
                    edge.parent_verifier_id, edge.delegate_verifier_id
                ),
            ));
            return Ok(Some(
                VerifierAuthorityResolutionClass::AuthorityScopeWidening,
            ));
        }
        if !scope_is_subset(requested_scope, &delegate.authority_scope)
            && edge.delegate_verifier_id == delegate.verifier_id
        {
            continue;
        }
    }

    if detect_cycle(snapshot)? {
        findings.push(VerificationFinding::error(
            "PV0907",
            "verifier authority graph contains a direct or indirect cycle",
        ));
        return Ok(Some(VerifierAuthorityResolutionClass::AuthorityGraphCycle));
    }

    Ok(None)
}

fn enumerate_candidate_chains(
    snapshot: &VerifierTrustRegistrySnapshot,
    requested_verifier_id: &str,
    requested_scope: &[String],
) -> Result<EnumerationResult, VerifierRuntimeError> {
    let mut edges_by_parent: BTreeMap<&str, Vec<&VerifierDelegationEdge>> = BTreeMap::new();
    for edge in &snapshot.delegation_edges {
        edges_by_parent
            .entry(edge.parent_verifier_id.as_str())
            .or_default()
            .push(edge);
    }

    let mut unique = BTreeMap::<Vec<String>, CandidateChain>::new();
    let mut depth_exceeded = false;
    for root_verifier_id in &snapshot.root_verifier_ids {
        let mut path = Vec::new();
        walk_candidate_chains(
            snapshot,
            &edges_by_parent,
            root_verifier_id,
            requested_verifier_id,
            requested_scope,
            &mut path,
            requested_scope.to_vec(),
            false,
            &mut unique,
            &mut depth_exceeded,
        )?;
    }
    Ok(EnumerationResult {
        chains: unique.into_values().collect(),
        depth_exceeded,
    })
}

fn walk_candidate_chains(
    snapshot: &VerifierTrustRegistrySnapshot,
    edges_by_parent: &BTreeMap<&str, Vec<&VerifierDelegationEdge>>,
    current_verifier_id: &str,
    requested_verifier_id: &str,
    requested_scope: &[String],
    path: &mut Vec<String>,
    effective_scope: Vec<String>,
    historical_only: bool,
    unique: &mut BTreeMap<Vec<String>, CandidateChain>,
    depth_exceeded: &mut bool,
) -> Result<(), VerifierRuntimeError> {
    // `path` contains the ancestor chain before `current_verifier_id` is pushed.
    // Therefore `path.len()` equals the explicit hop depth of `current_verifier_id`.
    if path.len() > MAX_DELEGATION_DEPTH {
        *depth_exceeded = true;
        return Ok(());
    }
    let Some(node) = snapshot.verifiers.get(current_verifier_id) else {
        return Ok(());
    };
    if node.authority_state == VerifierAuthorityState::Revoked {
        return Ok(());
    }
    let effective_scope = intersect_scopes(&effective_scope, &node.authority_scope);
    if !scope_is_subset(requested_scope, &effective_scope) {
        return Ok(());
    }

    path.push(current_verifier_id.to_string());
    let historical_only =
        historical_only || node.authority_state == VerifierAuthorityState::HistoricalOnly;
    if current_verifier_id == requested_verifier_id {
        let authority_chain_id = compute_authority_chain_id(
            &path[..],
            &effective_scope,
            &snapshot.verifier_registry_snapshot_hash,
        )?;
        unique.insert(
            path.clone(),
            CandidateChain {
                chain: path.clone(),
                authority_chain_id,
                effective_authority_scope: effective_scope.clone(),
                historical_only,
            },
        );
        path.pop();
        return Ok(());
    }

    if let Some(edges) = edges_by_parent.get(current_verifier_id) {
        for edge in edges {
            let edge_effective_scope = intersect_scopes(&effective_scope, &edge.delegated_scope);
            if !scope_is_subset(requested_scope, &edge_effective_scope) {
                continue;
            }
            walk_candidate_chains(
                snapshot,
                edges_by_parent,
                &edge.delegate_verifier_id,
                requested_verifier_id,
                requested_scope,
                path,
                edge_effective_scope,
                historical_only,
                unique,
                depth_exceeded,
            )?;
        }
    }
    path.pop();
    Ok(())
}

fn compute_authority_chain_id(
    chain: &[String],
    effective_scope: &[String],
    verifier_registry_snapshot_hash: &str,
) -> Result<String, VerifierRuntimeError> {
    let representation = json!({
        "authority_chain": chain,
        "effective_authority_scope": canonical_scope(effective_scope),
        "verifier_registry_snapshot_hash": verifier_registry_snapshot_hash,
    });
    let bytes = canonicalize_json_value(&representation)?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

fn detect_cycle(snapshot: &VerifierTrustRegistrySnapshot) -> Result<bool, VerifierRuntimeError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Done,
    }

    fn dfs(
        current: &str,
        snapshot: &VerifierTrustRegistrySnapshot,
        states: &mut BTreeMap<String, VisitState>,
    ) -> bool {
        match states.get(current) {
            Some(VisitState::Visiting) => return true,
            Some(VisitState::Done) => return false,
            None => {}
        }
        states.insert(current.to_string(), VisitState::Visiting);
        for edge in snapshot
            .delegation_edges
            .iter()
            .filter(|edge| edge.parent_verifier_id == current)
        {
            if dfs(&edge.delegate_verifier_id, snapshot, states) {
                return true;
            }
        }
        states.insert(current.to_string(), VisitState::Done);
        false
    }

    let mut states = BTreeMap::new();
    for verifier_id in snapshot.verifiers.keys() {
        if dfs(verifier_id, snapshot, &mut states) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn scope_is_subset(candidate: &[String], allowed: &[String]) -> bool {
    let candidate: BTreeSet<&str> = candidate.iter().map(String::as_str).collect();
    let allowed: BTreeSet<&str> = allowed.iter().map(String::as_str).collect();
    candidate.is_subset(&allowed)
}

fn intersect_scopes(left: &[String], right: &[String]) -> Vec<String> {
    let left: BTreeSet<&str> = left.iter().map(String::as_str).collect();
    let right: BTreeSet<&str> = right.iter().map(String::as_str).collect();
    left.intersection(&right)
        .map(|value| (*value).to_string())
        .collect()
}

fn canonical_scope(scope: &[String]) -> Vec<String> {
    scope
        .iter()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn build_resolution(
    result_class: VerifierAuthorityResolutionClass,
    requested_verifier_id: &str,
    requested_scope: &[String],
    authority_chain: Vec<String>,
    authority_chain_id: Option<String>,
    effective_authority_scope: Vec<String>,
    findings: Vec<VerificationFinding>,
    snapshot: &VerifierTrustRegistrySnapshot,
) -> VerifierAuthorityResolution {
    VerifierAuthorityResolution {
        result_class,
        requested_verifier_id: requested_verifier_id.to_string(),
        requested_authority_scope: requested_scope.to_vec(),
        effective_authority_scope,
        authority_chain,
        authority_chain_id,
        verifier_registry_snapshot_hash: snapshot.verifier_registry_snapshot_hash.clone(),
        findings,
    }
}
