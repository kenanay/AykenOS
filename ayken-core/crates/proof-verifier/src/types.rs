use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct VerifyRequest<'a> {
    pub bundle_path: &'a Path,
    pub policy: &'a TrustPolicy,
    pub registry_snapshot: &'a RegistrySnapshot,
    pub receipt_mode: ReceiptMode,
    pub receipt_signer: Option<&'a ReceiptSignerConfig>,
    pub audit_mode: AuditMode,
    pub audit_ledger_path: Option<&'a Path>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptMode {
    None,
    EmitUnsigned,
    EmitSigned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditMode {
    None,
    Append,
}

#[derive(Debug, Clone)]
pub struct LoadedBundle {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub checksums_path: PathBuf,
    pub evidence_dir: PathBuf,
    pub traces_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub meta_run_path: PathBuf,
    pub producer_path: PathBuf,
    pub signature_envelope_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub bundle_id: String,
    pub bundle_version: u32,
    pub checksums_file: String,
    #[serde(default)]
    pub compatibility_mode: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub required_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumsFile {
    pub algorithm: String,
    pub bundle_version: u32,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerDeclaration {
    pub metadata_version: u32,
    pub producer_id: String,
    pub producer_pubkey_id: String,
    pub producer_registry_ref: String,
    pub producer_key_epoch: String,
    #[serde(default)]
    pub build_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachedSignature {
    pub signer_id: String,
    pub producer_pubkey_id: String,
    pub signature_algorithm: String,
    pub signature: String,
    pub signed_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEnvelope {
    pub envelope_version: u32,
    pub bundle_id: String,
    pub bundle_id_algorithm: String,
    #[serde(default)]
    pub signatures: Vec<DetachedSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPublicKey {
    pub algorithm: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    #[serde(default)]
    pub active_pubkey_ids: Vec<String>,
    #[serde(default)]
    pub revoked_pubkey_ids: Vec<String>,
    #[serde(default)]
    pub superseded_pubkey_ids: Vec<String>,
    #[serde(default)]
    pub public_keys: BTreeMap<String, RegistryPublicKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub registry_format_version: u32,
    pub registry_version: u32,
    pub registry_snapshot_hash: String,
    pub producers: BTreeMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierAuthorityState {
    Current,
    HistoricalOnly,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierTrustRegistryPublicKey {
    pub algorithm: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierAuthorityNode {
    pub verifier_id: String,
    pub verifier_pubkey_id: String,
    #[serde(default)]
    pub authority_scope: Vec<String>,
    pub authority_state: VerifierAuthorityState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierDelegationEdge {
    pub parent_verifier_id: String,
    pub delegate_verifier_id: String,
    #[serde(default)]
    pub delegated_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierTrustRegistrySnapshot {
    pub registry_format_version: u32,
    pub verifier_registry_snapshot_hash: String,
    pub verifier_registry_parent_hash: String,
    pub verifier_registry_epoch: u32,
    pub registry_scope: String,
    #[serde(default)]
    pub root_verifier_ids: Vec<String>,
    #[serde(default)]
    pub verifiers: BTreeMap<String, VerifierAuthorityNode>,
    #[serde(default)]
    pub public_keys: BTreeMap<String, VerifierTrustRegistryPublicKey>,
    #[serde(default)]
    pub delegation_edges: Vec<VerifierDelegationEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifierAuthorityResolutionClass {
    AuthorityResolvedRoot,
    AuthorityResolvedDelegated,
    AuthorityHistoricalOnly,
    AuthorityGraphAmbiguous,
    AuthorityGraphCycle,
    AuthorityGraphDepthExceeded,
    AuthorityScopeWidening,
    AuthorityNoValidChain,
}

#[derive(Debug, Clone)]
pub struct VerifierAuthorityResolution {
    pub result_class: VerifierAuthorityResolutionClass,
    pub requested_verifier_id: String,
    pub requested_authority_scope: Vec<String>,
    pub authority_chain: Vec<String>,
    pub authority_chain_id: Option<String>,
    pub effective_authority_scope: Vec<String>,
    pub verifier_registry_snapshot_hash: String,
    pub findings: Vec<VerificationFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRequirement {
    #[serde(rename = "type")]
    pub kind: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPolicy {
    pub policy_version: u32,
    #[serde(default)]
    pub policy_hash: Option<String>,
    #[serde(default)]
    pub quorum_policy_ref: Option<String>,
    #[serde(default)]
    pub trusted_producers: Vec<String>,
    #[serde(default)]
    pub trusted_pubkey_ids: Vec<String>,
    #[serde(default)]
    pub required_signatures: Option<SignatureRequirement>,
    #[serde(default)]
    pub revoked_pubkey_ids: Vec<String>,
}

impl TrustPolicy {
    pub fn required_signature_count(&self) -> usize {
        self.required_signatures
            .as_ref()
            .map(|value| value.count as usize)
            .unwrap_or(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyStatus {
    Active,
    Revoked,
    Superseded,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ResolvedSigner {
    pub signer_id: String,
    pub producer_pubkey_id: String,
    pub status: KeyStatus,
    pub public_key: Option<RegistryPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct VerificationFinding {
    pub code: String,
    pub message: String,
    pub severity: FindingSeverity,
    pub deterministic: bool,
}

impl VerificationFinding {
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: FindingSeverity::Info,
            deterministic: true,
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: FindingSeverity::Warning,
            deterministic: true,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: FindingSeverity::Error,
            deterministic: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortableCoreState {
    pub manifest: Manifest,
    pub checksums: ChecksumsFile,
    pub bundle_id: String,
}

#[derive(Debug, Clone)]
pub struct OverlayState {
    pub producer: ProducerDeclaration,
    pub signature_envelope: SignatureEnvelope,
    pub trust_overlay_hash: String,
    pub findings: Vec<VerificationFinding>,
}

#[derive(Debug, Clone)]
pub struct RegistryResolution {
    pub registry_snapshot_hash: String,
    pub resolved_signers: Vec<ResolvedSigner>,
    pub findings: Vec<VerificationFinding>,
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub policy_hash: String,
    pub verdict: VerificationVerdict,
    pub findings: Vec<VerificationFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictSubject {
    pub bundle_id: String,
    pub trust_overlay_hash: String,
    pub policy_hash: String,
    pub registry_snapshot_hash: String,
}

#[derive(Debug, Clone)]
pub struct ReceiptSignerConfig {
    pub verifier_node_id: String,
    pub verifier_key_id: String,
    pub signature_algorithm: String,
    pub private_key: String,
    pub verified_at_utc: String,
}

#[derive(Debug, Clone)]
pub struct ReceiptVerifierKey {
    pub verifier_node_id: String,
    pub verifier_key_id: String,
    pub signature_algorithm: String,
    pub public_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationVerdict {
    Trusted,
    Untrusted,
    Invalid,
    RejectedByPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReceiptPayload {
    pub receipt_version: u32,
    pub bundle_id: String,
    pub trust_overlay_hash: String,
    pub policy_hash: String,
    pub registry_snapshot_hash: String,
    pub verifier_node_id: String,
    #[serde(default)]
    pub verifier_key_id: Option<String>,
    pub verdict: VerificationVerdict,
    pub verified_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReceipt {
    #[serde(flatten)]
    pub payload: VerificationReceiptPayload,
    pub verifier_signature_algorithm: Option<String>,
    pub verifier_signature: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DistributedReceiptVerification {
    pub authority_resolution: VerifierAuthorityResolution,
    pub findings: Vec<VerificationFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAuditEvent {
    pub event_version: u32,
    pub event_type: String,
    pub event_id: String,
    pub event_time_utc: String,
    pub verifier_node_id: String,
    #[serde(default)]
    pub verifier_key_id: Option<String>,
    pub bundle_id: String,
    pub trust_overlay_hash: String,
    pub policy_hash: String,
    pub registry_snapshot_hash: String,
    pub verdict: VerificationVerdict,
    pub receipt_hash: String,
    #[serde(default)]
    pub previous_event_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationOutcome {
    pub verdict: VerificationVerdict,
    pub subject: VerdictSubject,
    pub findings: Vec<VerificationFinding>,
    pub receipt: Option<VerificationReceipt>,
    pub audit_event: Option<VerificationAuditEvent>,
}
