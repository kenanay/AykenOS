use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::types::{
    VerdictSubject, VerificationVerdict, VerifierAuthorityResolution,
    VerifierAuthorityResolutionClass,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParityArtifactForm {
    SignedReceipt,
    LocalVerificationOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParityEvidenceState {
    Sufficient,
    Insufficient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeParityOutcome {
    pub node_id: String,
    pub pairwise_alias: String,
    pub verdict: VerificationVerdict,
    subject_hash: String,
    context_hash: String,
    authority_hash: String,
    artifact_form: ParityArtifactForm,
    evidence_state: ParityEvidenceState,
    verifier_contract_version: String,
    authority_result_class: String,
    subject: VerdictSubject,
    verification_context_id: String,
    verifier_registry_snapshot_hash: String,
    effective_authority_scope: Vec<String>,
    authority_chain_id: Option<String>,
    surface_key: String,
    outcome_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeParityOutcomeView {
    pub node_id: String,
    pub pairwise_alias: String,
    pub verdict: VerificationVerdict,
    pub subject_hash: String,
    pub context_hash: String,
    pub authority_hash: String,
    pub artifact_form: ParityArtifactForm,
    pub evidence_state: ParityEvidenceState,
    pub verifier_contract_version: String,
    pub authority_result_class: String,
    pub subject: VerdictSubject,
    pub verification_context_id: String,
    pub verifier_registry_snapshot_hash: String,
    pub effective_authority_scope: Vec<String>,
    pub authority_chain_id: Option<String>,
    pub surface_key: String,
    pub outcome_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityParityComparison {
    pub result_class_equal: bool,
    pub verifier_registry_snapshot_hash_equal: bool,
    pub effective_authority_scope_equal: bool,
    pub authority_chain_equal: bool,
    pub authority_chain_id_equal: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossNodeParityStatus {
    ParityMatch,
    ParitySubjectMismatch,
    ParityContextMismatch,
    ParityVerifierMismatch,
    ParityVerdictMismatch,
    ParityHistoricalOnly,
    ParityInsufficientEvidence,
}

#[derive(Debug, Clone)]
pub struct CrossNodeParityInput<'a> {
    pub node_id: &'a str,
    pub subject: &'a VerdictSubject,
    pub verification_context_id: &'a str,
    pub authority_resolution: &'a VerifierAuthorityResolution,
    pub local_verdict: &'a VerificationVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossNodeParityRecord {
    pub node_a: String,
    pub node_b: String,
    pub parity_status: CrossNodeParityStatus,
    pub bundle_id_equal: bool,
    pub trust_overlay_hash_equal: bool,
    pub policy_hash_equal: bool,
    pub registry_snapshot_hash_equal: bool,
    pub verification_context_id_equal: bool,
    pub trusted_verifier_semantics_equal: bool,
    pub result_class_equal: bool,
    pub effective_authority_scope_equal: bool,
    pub authority_chain_equal: bool,
    pub authority_chain_id_equal: Option<bool>,
    pub local_verdict_equal: bool,
}

pub fn build_node_parity_outcome(
    node_id: &str,
    pairwise_alias: &str,
    subject: &VerdictSubject,
    verification_context_id: &str,
    verifier_contract_version: &str,
    authority_resolution: &VerifierAuthorityResolution,
    local_verdict: &VerificationVerdict,
    artifact_form: ParityArtifactForm,
    evidence_state: ParityEvidenceState,
) -> Result<NodeParityOutcome, VerifierRuntimeError> {
    let subject_hash = compute_subject_hash(subject)?;
    let context_hash = verification_context_id.to_string();
    let authority_hash = compute_authority_hash(authority_resolution)?;
    let surface_key = compute_surface_key(&subject_hash, &context_hash, &authority_hash)?;
    let outcome_key =
        compute_outcome_key(&subject_hash, &context_hash, &authority_hash, local_verdict)?;

    Ok(NodeParityOutcome {
        node_id: node_id.to_string(),
        pairwise_alias: pairwise_alias.to_string(),
        subject_hash,
        context_hash,
        authority_hash,
        verdict: local_verdict.clone(),
        artifact_form,
        evidence_state,
        verifier_contract_version: verifier_contract_version.to_string(),
        authority_result_class: authority_resolution_label(authority_resolution).to_string(),
        subject: subject.clone(),
        verification_context_id: verification_context_id.to_string(),
        verifier_registry_snapshot_hash: authority_resolution
            .verifier_registry_snapshot_hash
            .clone(),
        effective_authority_scope: authority_resolution.effective_authority_scope.clone(),
        authority_chain_id: authority_resolution.authority_chain_id.clone(),
        surface_key,
        outcome_key,
    })
}

impl NodeParityOutcome {
    pub fn subject_hash(&self) -> &str {
        &self.subject_hash
    }

    pub fn context_hash(&self) -> &str {
        &self.context_hash
    }

    pub fn authority_hash(&self) -> &str {
        &self.authority_hash
    }

    pub fn artifact_form(&self) -> &ParityArtifactForm {
        &self.artifact_form
    }

    pub fn evidence_state(&self) -> &ParityEvidenceState {
        &self.evidence_state
    }

    pub fn verifier_contract_version(&self) -> &str {
        &self.verifier_contract_version
    }

    pub fn authority_result_class(&self) -> &str {
        &self.authority_result_class
    }

    pub fn is_historical_only(&self) -> bool {
        self.authority_result_class == "AUTHORITY_HISTORICAL_ONLY"
    }

    pub fn subject(&self) -> &VerdictSubject {
        &self.subject
    }

    pub fn verification_context_id(&self) -> &str {
        &self.verification_context_id
    }

    pub fn verifier_registry_snapshot_hash(&self) -> &str {
        &self.verifier_registry_snapshot_hash
    }

    pub fn effective_authority_scope(&self) -> &[String] {
        &self.effective_authority_scope
    }

    pub fn authority_chain_id(&self) -> Option<&str> {
        self.authority_chain_id.as_deref()
    }

    pub fn surface_key(&self) -> &str {
        &self.surface_key
    }

    pub fn outcome_key(&self) -> &str {
        &self.outcome_key
    }
}

impl From<&NodeParityOutcome> for NodeParityOutcomeView {
    fn from(value: &NodeParityOutcome) -> Self {
        Self {
            node_id: value.node_id.clone(),
            pairwise_alias: value.pairwise_alias.clone(),
            verdict: value.verdict.clone(),
            subject_hash: value.subject_hash.clone(),
            context_hash: value.context_hash.clone(),
            authority_hash: value.authority_hash.clone(),
            artifact_form: value.artifact_form.clone(),
            evidence_state: value.evidence_state.clone(),
            verifier_contract_version: value.verifier_contract_version.clone(),
            authority_result_class: value.authority_result_class.clone(),
            subject: value.subject.clone(),
            verification_context_id: value.verification_context_id.clone(),
            verifier_registry_snapshot_hash: value.verifier_registry_snapshot_hash.clone(),
            effective_authority_scope: value.effective_authority_scope.clone(),
            authority_chain_id: value.authority_chain_id.clone(),
            surface_key: value.surface_key.clone(),
            outcome_key: value.outcome_key.clone(),
        }
    }
}

pub fn compare_authority_resolution(
    left: &VerifierAuthorityResolution,
    right: &VerifierAuthorityResolution,
) -> AuthorityParityComparison {
    AuthorityParityComparison {
        result_class_equal: left.result_class == right.result_class,
        verifier_registry_snapshot_hash_equal: left.verifier_registry_snapshot_hash
            == right.verifier_registry_snapshot_hash,
        effective_authority_scope_equal: left.effective_authority_scope
            == right.effective_authority_scope,
        authority_chain_equal: left.authority_chain == right.authority_chain,
        authority_chain_id_equal: match (
            left.authority_chain_id.as_deref(),
            right.authority_chain_id.as_deref(),
        ) {
            (Some(left), Some(right)) => Some(left == right),
            _ => None,
        },
    }
}

pub fn authority_resolution_label(resolution: &VerifierAuthorityResolution) -> &'static str {
    match resolution.result_class {
        VerifierAuthorityResolutionClass::AuthorityResolvedRoot => "AUTHORITY_RESOLVED_ROOT",
        VerifierAuthorityResolutionClass::AuthorityResolvedDelegated => {
            "AUTHORITY_RESOLVED_DELEGATED"
        }
        VerifierAuthorityResolutionClass::AuthorityHistoricalOnly => "AUTHORITY_HISTORICAL_ONLY",
        VerifierAuthorityResolutionClass::AuthorityGraphAmbiguous => "AUTHORITY_GRAPH_AMBIGUOUS",
        VerifierAuthorityResolutionClass::AuthorityGraphCycle => "AUTHORITY_GRAPH_CYCLE",
        VerifierAuthorityResolutionClass::AuthorityGraphDepthExceeded => {
            "AUTHORITY_GRAPH_DEPTH_EXCEEDED"
        }
        VerifierAuthorityResolutionClass::AuthorityScopeWidening => "AUTHORITY_SCOPE_WIDENING",
        VerifierAuthorityResolutionClass::AuthorityNoValidChain => "AUTHORITY_NO_VALID_CHAIN",
    }
}

pub fn compare_cross_node_parity(
    left: CrossNodeParityInput<'_>,
    right: CrossNodeParityInput<'_>,
) -> CrossNodeParityRecord {
    let authority =
        compare_authority_resolution(left.authority_resolution, right.authority_resolution);
    let bundle_id_equal = left.subject.bundle_id == right.subject.bundle_id;
    let trust_overlay_hash_equal =
        left.subject.trust_overlay_hash == right.subject.trust_overlay_hash;
    let policy_hash_equal = left.subject.policy_hash == right.subject.policy_hash;
    let registry_snapshot_hash_equal =
        left.subject.registry_snapshot_hash == right.subject.registry_snapshot_hash;
    let verification_context_id_equal =
        left.verification_context_id == right.verification_context_id;
    let local_verdict_equal = left.local_verdict == right.local_verdict;
    let trusted_verifier_semantics_equal = authority.result_class_equal
        && authority.verifier_registry_snapshot_hash_equal
        && authority.effective_authority_scope_equal
        && authority.authority_chain_equal
        && authority.authority_chain_id_equal == Some(true);

    let parity_status = if left.node_id.trim().is_empty()
        || right.node_id.trim().is_empty()
        || left.verification_context_id.trim().is_empty()
        || right.verification_context_id.trim().is_empty()
    {
        CrossNodeParityStatus::ParityInsufficientEvidence
    } else if !bundle_id_equal
        || !trust_overlay_hash_equal
        || !policy_hash_equal
        || !registry_snapshot_hash_equal
    {
        CrossNodeParityStatus::ParitySubjectMismatch
    } else if !verification_context_id_equal {
        CrossNodeParityStatus::ParityContextMismatch
    } else if !trusted_verifier_semantics_equal {
        CrossNodeParityStatus::ParityVerifierMismatch
    } else if !local_verdict_equal {
        CrossNodeParityStatus::ParityVerdictMismatch
    } else if matches!(
        left.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityHistoricalOnly
    ) || matches!(
        right.authority_resolution.result_class,
        VerifierAuthorityResolutionClass::AuthorityHistoricalOnly
    ) {
        CrossNodeParityStatus::ParityHistoricalOnly
    } else {
        CrossNodeParityStatus::ParityMatch
    };

    CrossNodeParityRecord {
        node_a: left.node_id.to_string(),
        node_b: right.node_id.to_string(),
        parity_status,
        bundle_id_equal,
        trust_overlay_hash_equal,
        policy_hash_equal,
        registry_snapshot_hash_equal,
        verification_context_id_equal,
        trusted_verifier_semantics_equal,
        result_class_equal: authority.result_class_equal,
        effective_authority_scope_equal: authority.effective_authority_scope_equal,
        authority_chain_equal: authority.authority_chain_equal,
        authority_chain_id_equal: authority.authority_chain_id_equal,
        local_verdict_equal,
    }
}

fn compute_subject_hash(subject: &VerdictSubject) -> Result<String, VerifierRuntimeError> {
    compute_canonical_value_hash(&json!({
        "bundle_id": subject.bundle_id,
        "trust_overlay_hash": subject.trust_overlay_hash,
        "policy_hash": subject.policy_hash,
        "registry_snapshot_hash": subject.registry_snapshot_hash,
    }))
}

fn compute_authority_hash(
    resolution: &VerifierAuthorityResolution,
) -> Result<String, VerifierRuntimeError> {
    compute_canonical_value_hash(&json!({
        "result_class": authority_resolution_label(resolution),
        "verifier_registry_snapshot_hash": resolution.verifier_registry_snapshot_hash,
        "effective_authority_scope": resolution.effective_authority_scope,
        "authority_chain_id": resolution.authority_chain_id,
    }))
}

fn compute_surface_key(
    subject_hash: &str,
    context_hash: &str,
    authority_hash: &str,
) -> Result<String, VerifierRuntimeError> {
    compute_canonical_value_hash(&json!({
        "subject_hash": subject_hash,
        "context_hash": context_hash,
        "authority_hash": authority_hash,
    }))
}

fn compute_outcome_key(
    subject_hash: &str,
    context_hash: &str,
    authority_hash: &str,
    verdict: &VerificationVerdict,
) -> Result<String, VerifierRuntimeError> {
    compute_canonical_value_hash(&json!({
        "subject_hash": subject_hash,
        "context_hash": context_hash,
        "authority_hash": authority_hash,
        "verdict": verdict,
    }))
}

fn compute_canonical_value_hash(value: &serde_json::Value) -> Result<String, VerifierRuntimeError> {
    let bytes = canonicalize_json_value(value)?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}
