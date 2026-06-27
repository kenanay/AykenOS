//! Bounded Phase-19 admission/receipt harness.
//!
//! This crate implements the first permitted userspace code surface from
//! `PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md`: deterministic record
//! emission for a static test-owned input bundle. It does not parse general
//! manifests, install packages, load modules, create workspaces, issue
//! capabilities, assign trust, execute code, call syscalls, or widen kernel
//! ABI.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const INPUT_BUNDLE_SCHEMA_ID: &str = "ayken.phase19.runtime.input_bundle.v1";
pub const VALIDATION_INTEGRATION_RECORD_SCHEMA_ID: &str =
    "ayken.phase19.platform_validation.integration.record.v1";
pub const WORKSPACE_ADMISSION_SCHEMA_ID: &str = "ayken.phase19.workspace_admission.runtime.v1";
pub const RUNTIME_RECEIPT_SCHEMA_ID: &str = "ayken.phase19.runtime.receipt.v1";
pub const PLATFORM_VALIDATION_RECEIPT_CONTRACT_ID: &str = "ayken.platform.abi.validation.gate.v1";
pub const PLATFORM_VALIDATION_RECEIPT_SCHEMA_VERSION: &str = "1";
pub const MODULE_MANIFEST_CONTRACT_ID: &str = "ayken.platform.module.manifest.v1";
pub const PACKAGE_METADATA_CONTRACT_ID: &str = "ayken.platform.package.metadata.v1";
pub const PLATFORM_VALIDATION_POLICY_CONTRACT_ID: &str = "ayken.platform.abi.validation.gate.v1";
pub const WORKSPACE_DECLARATION_CONTRACT_ID: &str = "ayken.platform.workspace.lifecycle.v1";
pub const RUNTIME_EVIDENCE_MATRIX_CONTRACT_ID: &str = "ayken.phase19.runtime.evidence_matrix.v1";
pub const REFERENCE_ENVELOPE_SCHEMA_VERSION: &str = "1";
pub const REFERENCE_DIGEST_ALGORITHM: &str = "sha256";
pub const FROZEN_SYSCALL_RANGE: &str = "1000-1011";
pub const FROZEN_SYSCALL_COUNT: u16 = 12;
pub const FROZEN_ABI_VERSION: &str = "0x00010001";

const VALIDATION_STAGE_ORDER: [(&str, u8); 10] = [
    ("kernel_freeze_guard", 0),
    ("manifest_validation", 1),
    ("package_metadata_validation", 2),
    ("package_manifest_binding", 3),
    ("trust_classification_validation", 4),
    ("capability_contract_validation", 5),
    ("workspace_lifecycle_validation", 6),
    ("plugin_boundary_validation", 7),
    ("cross_contract_separation", 8),
    ("validation_receipt_emission", 9),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subject {
    pub kind: String,
    pub id: String,
    pub version: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DigestReference {
    pub path_or_uri: String,
    pub digest_algorithm: String,
    pub digest_value: String,
    pub contract_id: String,
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject>,
    #[serde(default)]
    pub stale: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceClass {
    ModuleManifest,
    PackageMetadata,
    PlatformValidationPolicy,
    WorkspaceDeclaration,
    RuntimeEvidenceMatrix,
}

impl ReferenceClass {
    fn contract_id(self) -> &'static str {
        match self {
            Self::ModuleManifest => MODULE_MANIFEST_CONTRACT_ID,
            Self::PackageMetadata => PACKAGE_METADATA_CONTRACT_ID,
            Self::PlatformValidationPolicy => PLATFORM_VALIDATION_POLICY_CONTRACT_ID,
            Self::WorkspaceDeclaration => WORKSPACE_DECLARATION_CONTRACT_ID,
            Self::RuntimeEvidenceMatrix => RUNTIME_EVIDENCE_MATRIX_CONTRACT_ID,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestOwnedReferenceContent {
    pub path_or_uri: String,
    pub reference_class: ReferenceClass,
    pub subject: Subject,
    pub content_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaticInputBundle {
    pub schema_id: String,
    pub bundle_id: String,
    pub bundle_version: String,
    pub subject: Subject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_ref: Option<DigestReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_ref: Option<DigestReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_validation_policy_ref: Option<DigestReference>,
    pub workspace_admission_request: WorkspaceAdmissionRequest,
    pub expected_receipt_profile: ReceiptProfile,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<DigestReference>,
    #[serde(default)]
    pub shape: StaticBundleShape,
    #[serde(default)]
    pub authority_claims: AuthorityClaims,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StaticBundleShape {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_top_level_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duplicate_top_level_keys: Vec<String>,
    #[serde(default)]
    pub kernel_abi_expansion_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceAdmissionRequest {
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_ref: Option<DigestReference>,
    #[serde(default)]
    pub requests_real_mount: bool,
    #[serde(default)]
    pub claims_workspace_handle: bool,
    #[serde(default)]
    pub requests_capability_issuance: bool,
    #[serde(default)]
    pub requests_trust_assignment: bool,
    #[serde(default)]
    pub requests_package_install_or_execution: bool,
    #[serde(default)]
    pub requests_plugin_loading: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptProfile {
    pub profile_id: String,
    pub profile_version: String,
    #[serde(default)]
    pub declares_token_authority: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AuthorityClaims {
    #[serde(default)]
    pub trust_as_capability: bool,
    #[serde(default)]
    pub plugin_as_loading: bool,
    #[serde(default)]
    pub semantic_cli_output_as_authority: bool,
    #[serde(default)]
    pub ai_output_as_authority: bool,
    #[serde(default)]
    pub evidence_as_control_input: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformValidationEvidence {
    pub contract_id: String,
    pub schema_version: String,
    pub subject: Subject,
    pub receipt_digest: String,
    pub stage_results: Vec<ValidationStageReference>,
    pub status: PlatformValidationStatus,
    #[serde(default)]
    pub declares_authority_grant: bool,
    #[serde(default)]
    pub unknown_stage_observed: bool,
    #[serde(default)]
    pub stale_digest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationStageReference {
    pub stage_id: String,
    pub stage_index: u8,
    pub digest_algorithm: String,
    pub digest_value: String,
    pub content_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformValidationStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessOutcome {
    pub status: HarnessStatus,
    pub transcript: Vec<LifecycleState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_bundle_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_integration_record: Option<ValidationIntegrationRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_admission_record: Option<WorkspaceAdmissionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_receipt: Option<RuntimeReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denial_record: Option<DenialRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessStatus {
    AdmittedRecorded,
    Denied,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationIntegrationRecord {
    pub schema_id: String,
    pub input_bundle_digest: String,
    pub validation_receipt_digest: String,
    pub stage_count: usize,
    pub stage_result_digests: Vec<String>,
    pub decision_status: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceAdmissionRecord {
    pub schema_id: String,
    pub admission_id: String,
    pub subject: Subject,
    pub input_bundle_digest: String,
    pub validation_result_digest: String,
    pub workspace_profile: String,
    pub admission_status: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeReceipt {
    pub schema_id: String,
    pub receipt_id: String,
    pub subject: Subject,
    pub input_bundle_digest: String,
    pub lifecycle_digest: String,
    pub validation_result_digest: String,
    pub admission_record_digest: String,
    pub receipt_status: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DenialRecord {
    pub reason: DenialReason,
    pub transcript_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_bundle_digest: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialReason {
    InputSchemaDenied,
    UnknownInputField,
    DuplicateInputKey,
    MissingManifestReference,
    StaleManifestDigest,
    MissingValidationPolicyReference,
    MissingWorkspaceDeclaration,
    StaleWorkspaceDeclaration,
    WorkspaceDeclarationSubjectMismatch,
    SubjectMismatch,
    MissingPlatformValidation,
    PlatformValidationFailed,
    ValidationAuthorityDenied,
    UnknownValidationSchemaVersion,
    ValidationStaleDigest,
    UnknownValidationStage,
    UnknownReferenceContract,
    UnknownReferenceSchemaVersion,
    MissingReferenceSubject,
    ReferenceSubjectMismatch,
    UnsupportedReferenceDigestAlgorithm,
    MalformedReferenceDigest,
    MissingReferenceContent,
    DuplicateReferenceContent,
    UnexpectedReferenceContent,
    ReferenceDigestMismatch,
    ValidationContractMismatch,
    ValidationStageCountMismatch,
    UnknownValidationStageId,
    ValidationStageIndexMismatch,
    ValidationStageOrderMismatch,
    ValidationStageDigestMismatch,
    RealMountDenied,
    WorkspaceHandleDenied,
    CapabilityIssuanceDenied,
    TrustAssignmentDenied,
    PackageInstallExecutionDenied,
    ReceiptTokenDenied,
    TrustCapabilityDenied,
    PluginLoadingDenied,
    SemanticCliAuthorityDenied,
    AiAuthorityDenied,
    EvidenceControlInputDenied,
    KernelAbiExpansionDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleState {
    Uninitialized,
    InputBound,
    Validating,
    ValidationRejected,
    ValidatedRecordable,
    AdmissionRecorded,
    ReceiptEmitted,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessError {
    CanonicalizationFailed,
}

pub fn run_harness(
    bundle: &StaticInputBundle,
    validation: Option<&PlatformValidationEvidence>,
    reference_contents: &[TestOwnedReferenceContent],
) -> Result<HarnessOutcome, HarnessError> {
    if bundle.schema_id != INPUT_BUNDLE_SCHEMA_ID {
        return denied_before_input(DenialReason::InputSchemaDenied);
    }

    if !bundle.shape.unknown_top_level_fields.is_empty() {
        return denied_before_input(DenialReason::UnknownInputField);
    }

    if !bundle.shape.duplicate_top_level_keys.is_empty() {
        return denied_before_input(DenialReason::DuplicateInputKey);
    }

    if bundle.shape.kernel_abi_expansion_requested {
        return denied_before_input(DenialReason::KernelAbiExpansionDenied);
    }

    if bundle.manifest_ref.is_none() {
        return denied_before_input(DenialReason::MissingManifestReference);
    }

    if bundle.platform_validation_policy_ref.is_none() {
        return denied_before_input(DenialReason::MissingValidationPolicyReference);
    }

    if bundle.workspace_admission_request.declaration_ref.is_none() {
        return denied_before_input(DenialReason::MissingWorkspaceDeclaration);
    }

    if let Some(reason) = validate_reference_integrity(bundle, reference_contents) {
        return denied_before_input(reason);
    }

    let workspace_declaration_ref = bundle
        .workspace_admission_request
        .declaration_ref
        .as_ref()
        .expect("workspace declaration_ref checked above");
    if workspace_declaration_ref.stale {
        return denied_before_input(DenialReason::StaleWorkspaceDeclaration);
    }

    let manifest_ref = bundle
        .manifest_ref
        .as_ref()
        .expect("manifest_ref checked above");
    if manifest_ref.stale {
        return denied_before_input(DenialReason::StaleManifestDigest);
    }

    if let Some(package_ref) = &bundle.package_ref {
        if package_ref.stale {
            return denied_before_input(DenialReason::StaleManifestDigest);
        }
    }

    if let Some(policy_ref) = &bundle.platform_validation_policy_ref {
        if policy_ref.stale {
            return denied_before_input(DenialReason::StaleManifestDigest);
        }
    }

    let input_bundle_digest = canonical_hash_prefixed(bundle)?;
    let input_bound_transcript = vec![LifecycleState::Uninitialized, LifecycleState::InputBound];

    let Some(validation) = validation else {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::MissingPlatformValidation,
        );
    };

    if validation.contract_id != PLATFORM_VALIDATION_RECEIPT_CONTRACT_ID {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::ValidationContractMismatch,
        );
    }

    if validation.schema_version != PLATFORM_VALIDATION_RECEIPT_SCHEMA_VERSION {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::UnknownValidationSchemaVersion,
        );
    }

    if validation.subject != bundle.subject {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::SubjectMismatch,
        );
    }

    if let Some(reason) = validate_stage_integrity(validation) {
        return denied_after_input(input_bound_transcript, Some(input_bundle_digest), reason);
    }

    if validation.stale_digest {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::ValidationStaleDigest,
        );
    }

    if validation.unknown_stage_observed {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::UnknownValidationStage,
        );
    }

    if validation.declares_authority_grant {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::ValidationAuthorityDenied,
        );
    }

    let validating_transcript = vec![
        LifecycleState::Uninitialized,
        LifecycleState::InputBound,
        LifecycleState::Validating,
    ];

    if validation.status == PlatformValidationStatus::Fail {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidationRejected),
            Some(input_bundle_digest),
            DenialReason::PlatformValidationFailed,
        );
    }

    let validation_record = build_validation_record(&input_bundle_digest, validation)?;
    let validation_record_digest = validation_record.digest.clone();

    if bundle.workspace_admission_request.requests_real_mount {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::RealMountDenied,
        );
    }

    if bundle.workspace_admission_request.claims_workspace_handle {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::WorkspaceHandleDenied,
        );
    }

    if bundle
        .workspace_admission_request
        .requests_capability_issuance
    {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::CapabilityIssuanceDenied,
        );
    }

    if bundle.workspace_admission_request.requests_trust_assignment {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::TrustAssignmentDenied,
        );
    }

    if bundle
        .workspace_admission_request
        .requests_package_install_or_execution
    {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::PackageInstallExecutionDenied,
        );
    }

    if bundle.workspace_admission_request.requests_plugin_loading
        || bundle.authority_claims.plugin_as_loading
    {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::PluginLoadingDenied,
        );
    }

    if bundle.expected_receipt_profile.declares_token_authority {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::ReceiptTokenDenied,
        );
    }

    if bundle.authority_claims.trust_as_capability {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::TrustCapabilityDenied,
        );
    }

    if bundle.authority_claims.semantic_cli_output_as_authority {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::SemanticCliAuthorityDenied,
        );
    }

    if bundle.authority_claims.ai_output_as_authority {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::AiAuthorityDenied,
        );
    }

    if bundle.authority_claims.evidence_as_control_input {
        return denied_after_input(
            append_state(validating_transcript, LifecycleState::ValidatedRecordable),
            Some(input_bundle_digest),
            DenialReason::EvidenceControlInputDenied,
        );
    }

    let admission_record =
        build_admission_record(bundle, &input_bundle_digest, &validation_record_digest)?;
    let transcript = vec![
        LifecycleState::Uninitialized,
        LifecycleState::InputBound,
        LifecycleState::Validating,
        LifecycleState::ValidatedRecordable,
        LifecycleState::AdmissionRecorded,
        LifecycleState::ReceiptEmitted,
    ];
    let lifecycle_digest = canonical_hash_prefixed(&transcript)?;
    let receipt = build_receipt(
        bundle,
        &input_bundle_digest,
        &lifecycle_digest,
        &validation_record_digest,
        &admission_record.digest,
    )?;

    Ok(HarnessOutcome {
        status: HarnessStatus::AdmittedRecorded,
        transcript,
        input_bundle_digest: Some(input_bundle_digest),
        validation_integration_record: Some(validation_record),
        workspace_admission_record: Some(admission_record),
        runtime_receipt: Some(receipt),
        denial_record: None,
    })
}

fn validate_reference_integrity(
    bundle: &StaticInputBundle,
    contents: &[TestOwnedReferenceContent],
) -> Option<DenialReason> {
    let references = declared_references(bundle);

    for (reference, class) in &references {
        if reference.contract_id != class.contract_id() {
            return Some(DenialReason::UnknownReferenceContract);
        }
    }

    for content in contents {
        for (_, class) in references
            .iter()
            .filter(|(reference, _)| reference.path_or_uri == content.path_or_uri)
        {
            if content.reference_class != *class {
                return Some(DenialReason::UnknownReferenceContract);
            }
        }
    }

    for (reference, _) in &references {
        if reference.schema_version != REFERENCE_ENVELOPE_SCHEMA_VERSION {
            return Some(DenialReason::UnknownReferenceSchemaVersion);
        }
    }

    for (reference, _) in &references {
        let Some(subject) = &reference.subject else {
            return Some(DenialReason::MissingReferenceSubject);
        };
        if subject != &bundle.subject {
            return Some(DenialReason::ReferenceSubjectMismatch);
        }
    }

    for content in contents {
        if content.subject != bundle.subject {
            return Some(DenialReason::ReferenceSubjectMismatch);
        }
    }

    for (reference, _) in &references {
        if reference.digest_algorithm != REFERENCE_DIGEST_ALGORITHM {
            return Some(DenialReason::UnsupportedReferenceDigestAlgorithm);
        }
    }

    for (reference, _) in &references {
        if !is_well_formed_sha256(&reference.digest_value) {
            return Some(DenialReason::MalformedReferenceDigest);
        }
    }

    for (reference, _) in &references {
        if !contents
            .iter()
            .any(|content| content.path_or_uri == reference.path_or_uri)
        {
            return Some(DenialReason::MissingReferenceContent);
        }
    }

    for (reference, _) in &references {
        let declaration_count = references
            .iter()
            .filter(|(candidate, _)| candidate.path_or_uri == reference.path_or_uri)
            .count();
        let content_count = contents
            .iter()
            .filter(|content| content.path_or_uri == reference.path_or_uri)
            .count();
        if declaration_count > 1 || content_count > 1 {
            return Some(DenialReason::DuplicateReferenceContent);
        }
    }

    for (index, content) in contents.iter().enumerate() {
        if contents[..index]
            .iter()
            .any(|candidate| candidate.path_or_uri == content.path_or_uri)
        {
            return Some(DenialReason::DuplicateReferenceContent);
        }
    }

    for content in contents {
        if !references
            .iter()
            .any(|(reference, _)| reference.path_or_uri == content.path_or_uri)
        {
            return Some(DenialReason::UnexpectedReferenceContent);
        }
    }

    for (reference, _) in references {
        let content = contents
            .iter()
            .find(|content| content.path_or_uri == reference.path_or_uri)
            .expect("reference content cardinality checked above");
        if hash_bytes_prefixed(&content.content_bytes) != reference.digest_value {
            return Some(DenialReason::ReferenceDigestMismatch);
        }
    }

    None
}

fn declared_references(bundle: &StaticInputBundle) -> Vec<(&DigestReference, ReferenceClass)> {
    let mut references = Vec::with_capacity(4 + bundle.evidence_refs.len());

    if let Some(reference) = &bundle.manifest_ref {
        references.push((reference, ReferenceClass::ModuleManifest));
    }
    if let Some(reference) = &bundle.package_ref {
        references.push((reference, ReferenceClass::PackageMetadata));
    }
    if let Some(reference) = &bundle.platform_validation_policy_ref {
        references.push((reference, ReferenceClass::PlatformValidationPolicy));
    }
    if let Some(reference) = &bundle.workspace_admission_request.declaration_ref {
        references.push((reference, ReferenceClass::WorkspaceDeclaration));
    }
    for reference in &bundle.evidence_refs {
        references.push((reference, ReferenceClass::RuntimeEvidenceMatrix));
    }

    references
}

fn validate_stage_integrity(validation: &PlatformValidationEvidence) -> Option<DenialReason> {
    if validation.stage_results.len() != VALIDATION_STAGE_ORDER.len() {
        return Some(DenialReason::ValidationStageCountMismatch);
    }

    for stage in &validation.stage_results {
        if canonical_stage_index(&stage.stage_id).is_none() {
            return Some(DenialReason::UnknownValidationStageId);
        }
    }

    for stage in &validation.stage_results {
        if canonical_stage_index(&stage.stage_id) != Some(stage.stage_index) {
            return Some(DenialReason::ValidationStageIndexMismatch);
        }
    }

    for (position, stage) in validation.stage_results.iter().enumerate() {
        if stage.stage_index as usize != position {
            return Some(DenialReason::ValidationStageOrderMismatch);
        }
    }

    for stage in &validation.stage_results {
        if stage.digest_algorithm != REFERENCE_DIGEST_ALGORITHM
            || !is_well_formed_sha256(&stage.digest_value)
            || hash_bytes_prefixed(&stage.content_bytes) != stage.digest_value
        {
            return Some(DenialReason::ValidationStageDigestMismatch);
        }
    }

    None
}

fn canonical_stage_index(stage_id: &str) -> Option<u8> {
    VALIDATION_STAGE_ORDER
        .iter()
        .find_map(|(candidate, index)| (*candidate == stage_id).then_some(*index))
}

fn is_well_formed_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hash_bytes_prefixed(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", encode_lower_hex(&hasher.finalize()))
}

fn build_validation_record(
    input_bundle_digest: &str,
    validation: &PlatformValidationEvidence,
) -> Result<ValidationIntegrationRecord, HarnessError> {
    #[derive(Serialize)]
    struct ValidationRecordSeed<'a> {
        schema_id: &'a str,
        input_bundle_digest: &'a str,
        validation_receipt_digest: &'a str,
        stage_count: usize,
        stage_result_digests: &'a [String],
        decision_status: &'a str,
    }

    let stage_result_digests: Vec<String> = validation
        .stage_results
        .iter()
        .map(|stage| stage.digest_value.clone())
        .collect();

    let seed = ValidationRecordSeed {
        schema_id: VALIDATION_INTEGRATION_RECORD_SCHEMA_ID,
        input_bundle_digest,
        validation_receipt_digest: &validation.receipt_digest,
        stage_count: stage_result_digests.len(),
        stage_result_digests: &stage_result_digests,
        decision_status: "recordable",
    };
    let digest = canonical_hash_prefixed(&seed)?;

    Ok(ValidationIntegrationRecord {
        schema_id: VALIDATION_INTEGRATION_RECORD_SCHEMA_ID.to_string(),
        input_bundle_digest: input_bundle_digest.to_string(),
        validation_receipt_digest: validation.receipt_digest.clone(),
        stage_count: stage_result_digests.len(),
        stage_result_digests,
        decision_status: "recordable".to_string(),
        digest,
    })
}

fn build_admission_record(
    bundle: &StaticInputBundle,
    input_bundle_digest: &str,
    validation_result_digest: &str,
) -> Result<WorkspaceAdmissionRecord, HarnessError> {
    #[derive(Serialize)]
    struct AdmissionSeed<'a> {
        schema_id: &'a str,
        subject: &'a Subject,
        input_bundle_digest: &'a str,
        validation_result_digest: &'a str,
        workspace_profile: &'a str,
        admission_status: &'a str,
    }

    let seed = AdmissionSeed {
        schema_id: WORKSPACE_ADMISSION_SCHEMA_ID,
        subject: &bundle.subject,
        input_bundle_digest,
        validation_result_digest,
        workspace_profile: &bundle.workspace_admission_request.profile,
        admission_status: "admitted_record",
    };
    let digest = canonical_hash_prefixed(&seed)?;

    Ok(WorkspaceAdmissionRecord {
        schema_id: WORKSPACE_ADMISSION_SCHEMA_ID.to_string(),
        admission_id: format!("phase19-admission-record:{digest}"),
        subject: bundle.subject.clone(),
        input_bundle_digest: input_bundle_digest.to_string(),
        validation_result_digest: validation_result_digest.to_string(),
        workspace_profile: bundle.workspace_admission_request.profile.clone(),
        admission_status: "admitted_record".to_string(),
        digest,
    })
}

fn build_receipt(
    bundle: &StaticInputBundle,
    input_bundle_digest: &str,
    lifecycle_digest: &str,
    validation_result_digest: &str,
    admission_record_digest: &str,
) -> Result<RuntimeReceipt, HarnessError> {
    #[derive(Serialize)]
    struct ReceiptSeed<'a> {
        schema_id: &'a str,
        subject: &'a Subject,
        input_bundle_digest: &'a str,
        lifecycle_digest: &'a str,
        validation_result_digest: &'a str,
        admission_record_digest: &'a str,
        receipt_status: &'a str,
    }

    let seed = ReceiptSeed {
        schema_id: RUNTIME_RECEIPT_SCHEMA_ID,
        subject: &bundle.subject,
        input_bundle_digest,
        lifecycle_digest,
        validation_result_digest,
        admission_record_digest,
        receipt_status: "admitted_recorded",
    };
    let digest = canonical_hash_prefixed(&seed)?;

    Ok(RuntimeReceipt {
        schema_id: RUNTIME_RECEIPT_SCHEMA_ID.to_string(),
        receipt_id: format!("phase19-runtime-receipt:{digest}"),
        subject: bundle.subject.clone(),
        input_bundle_digest: input_bundle_digest.to_string(),
        lifecycle_digest: lifecycle_digest.to_string(),
        validation_result_digest: validation_result_digest.to_string(),
        admission_record_digest: admission_record_digest.to_string(),
        receipt_status: "admitted_recorded".to_string(),
        digest,
    })
}

fn denied_before_input(reason: DenialReason) -> Result<HarnessOutcome, HarnessError> {
    denied_after_input(
        vec![LifecycleState::Uninitialized, LifecycleState::Aborted],
        None,
        reason,
    )
}

fn denied_after_input(
    transcript: Vec<LifecycleState>,
    input_bundle_digest: Option<String>,
    reason: DenialReason,
) -> Result<HarnessOutcome, HarnessError> {
    let terminal_transcript = ensure_terminal_denial_state(transcript);
    let transcript_digest = canonical_hash_prefixed(&terminal_transcript)?;
    Ok(HarnessOutcome {
        status: HarnessStatus::Denied,
        transcript: terminal_transcript,
        input_bundle_digest: input_bundle_digest.clone(),
        validation_integration_record: None,
        workspace_admission_record: None,
        runtime_receipt: None,
        denial_record: Some(DenialRecord {
            reason,
            transcript_digest,
            input_bundle_digest,
        }),
    })
}

fn ensure_terminal_denial_state(mut transcript: Vec<LifecycleState>) -> Vec<LifecycleState> {
    match transcript.last() {
        Some(LifecycleState::ValidationRejected) | Some(LifecycleState::Aborted) => transcript,
        _ => {
            transcript.push(LifecycleState::Aborted);
            transcript
        }
    }
}

fn append_state(mut transcript: Vec<LifecycleState>, state: LifecycleState) -> Vec<LifecycleState> {
    transcript.push(state);
    transcript
}

fn canonical_hash_prefixed<T>(value: &T) -> Result<String, HarnessError>
where
    T: Serialize,
{
    let canonical = canonical_bytes(value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical);
    Ok(format!("sha256:{}", encode_lower_hex(&hasher.finalize())))
}

fn canonical_bytes<T>(value: &T) -> Result<Vec<u8>, HarnessError>
where
    T: Serialize,
{
    let value = serde_json::to_value(value).map_err(|_| HarnessError::CanonicalizationFailed)?;
    let mut out = String::new();
    write_canonical_value(&value, &mut out)?;
    Ok(out.into_bytes())
}

fn write_canonical_value(value: &Value, out: &mut String) -> Result<(), HarnessError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&number.to_string()),
        Value::String(text) => {
            let encoded =
                serde_json::to_string(text).map_err(|_| HarnessError::CanonicalizationFailed)?;
            out.push_str(&encoded);
        }
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(object) => {
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                let encoded =
                    serde_json::to_string(key).map_err(|_| HarnessError::CanonicalizationFailed)?;
                out.push_str(&encoded);
                out.push(':');
                let value = object
                    .get(*key)
                    .ok_or(HarnessError::CanonicalizationFailed)?;
                write_canonical_value(value, out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST_PATH: &str =
        "docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md";
    const PACKAGE_PATH: &str =
        "docs/specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md";
    const VALIDATION_POLICY_PATH: &str =
        "docs/specs/phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md";
    const WORKSPACE_PATH: &str =
        "docs/specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md";
    const EVIDENCE_MATRIX_PATH: &str =
        "docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md";

    const MANIFEST_BYTES: &[u8] = b"phase19-test-owned-module-manifest-v1";
    const PACKAGE_BYTES: &[u8] = b"phase19-test-owned-package-metadata-v1";
    const VALIDATION_POLICY_BYTES: &[u8] = b"phase19-test-owned-validation-policy-v1";
    const WORKSPACE_BYTES: &[u8] = b"phase19-test-owned-workspace-declaration-v1";
    const EVIDENCE_MATRIX_BYTES: &[u8] = b"phase19-test-owned-evidence-matrix-v1";

    #[test]
    fn positive_flow_emits_inert_deterministic_records() {
        let bundle = valid_bundle();
        let validation = valid_validation(&bundle.subject);
        let contents = valid_reference_contents(&bundle.subject);

        let first = run_harness(&bundle, Some(&validation), &contents).expect("first run");
        let second = run_harness(&bundle, Some(&validation), &contents).expect("second run");

        assert_eq!(first, second);
        assert_eq!(first.status, HarnessStatus::AdmittedRecorded);
        assert_eq!(
            first.transcript,
            vec![
                LifecycleState::Uninitialized,
                LifecycleState::InputBound,
                LifecycleState::Validating,
                LifecycleState::ValidatedRecordable,
                LifecycleState::AdmissionRecorded,
                LifecycleState::ReceiptEmitted,
            ]
        );
        assert!(first.validation_integration_record.is_some());
        assert!(first.workspace_admission_record.is_some());
        assert!(first.runtime_receipt.is_some());
        assert!(first.denial_record.is_none());

        let receipt = first.runtime_receipt.as_ref().expect("receipt");
        assert_eq!(receipt.receipt_status, "admitted_recorded");
        assert!(receipt.digest.starts_with("sha256:"));
        assert!(receipt.receipt_id.starts_with("phase19-runtime-receipt:"));
        let validation_record = first
            .validation_integration_record
            .as_ref()
            .expect("validation integration record");
        assert_eq!(validation_record.stage_count, VALIDATION_STAGE_ORDER.len());
        assert_eq!(validation_record.stage_result_digests.len(), 10);
    }

    #[test]
    fn canonical_reference_map_and_stage_order_are_exact() {
        assert_eq!(
            ReferenceClass::ModuleManifest.contract_id(),
            "ayken.platform.module.manifest.v1"
        );
        assert_eq!(
            ReferenceClass::PackageMetadata.contract_id(),
            "ayken.platform.package.metadata.v1"
        );
        assert_eq!(
            ReferenceClass::PlatformValidationPolicy.contract_id(),
            "ayken.platform.abi.validation.gate.v1"
        );
        assert_eq!(
            ReferenceClass::WorkspaceDeclaration.contract_id(),
            "ayken.platform.workspace.lifecycle.v1"
        );
        assert_eq!(
            ReferenceClass::RuntimeEvidenceMatrix.contract_id(),
            "ayken.phase19.runtime.evidence_matrix.v1"
        );

        let validation = valid_validation(&valid_bundle().subject);
        let actual: Vec<(&str, u8)> = validation
            .stage_results
            .iter()
            .map(|stage| (stage.stage_id.as_str(), stage.stage_index))
            .collect();
        assert_eq!(actual, VALIDATION_STAGE_ORDER);
    }

    #[test]
    fn unknown_field_and_duplicate_key_deny_before_input_binding() {
        let validation_subject = valid_bundle().subject;

        let mut bad_schema = valid_bundle();
        bad_schema.schema_id = "ayken.phase19.runtime.input_bundle.v2".to_string();
        assert_denied(
            &bad_schema,
            Some(&valid_validation(&validation_subject)),
            DenialReason::InputSchemaDenied,
            None,
        );

        let mut unknown = valid_bundle();
        unknown
            .shape
            .unknown_top_level_fields
            .push("loader_request".to_string());
        assert_denied(
            &unknown,
            Some(&valid_validation(&validation_subject)),
            DenialReason::UnknownInputField,
            None,
        );

        let mut duplicate = valid_bundle();
        duplicate
            .shape
            .duplicate_top_level_keys
            .push("subject".to_string());
        assert_denied(
            &duplicate,
            Some(&valid_validation(&validation_subject)),
            DenialReason::DuplicateInputKey,
            None,
        );
    }

    #[test]
    fn validation_and_subject_mismatch_fail_closed() {
        let bundle = valid_bundle();

        assert_denied(
            &bundle,
            None,
            DenialReason::MissingPlatformValidation,
            Some(true),
        );

        let mut failed = valid_validation(&bundle.subject);
        failed.status = PlatformValidationStatus::Fail;
        assert_denied(
            &bundle,
            Some(&failed),
            DenialReason::PlatformValidationFailed,
            Some(true),
        );

        let mut mismatched = valid_validation(&bundle.subject);
        mismatched.subject.digest = "sha256:subject-mismatch".to_string();
        assert_denied(
            &bundle,
            Some(&mismatched),
            DenialReason::SubjectMismatch,
            Some(true),
        );

        let mut unknown_schema = valid_validation(&bundle.subject);
        unknown_schema.schema_version = "2".to_string();
        assert_denied(
            &bundle,
            Some(&unknown_schema),
            DenialReason::UnknownValidationSchemaVersion,
            Some(true),
        );

        let mut stale_validation = valid_validation(&bundle.subject);
        stale_validation.stale_digest = true;
        assert_denied(
            &bundle,
            Some(&stale_validation),
            DenialReason::ValidationStaleDigest,
            Some(true),
        );

        let mut unknown_stage = valid_validation(&bundle.subject);
        unknown_stage.unknown_stage_observed = true;
        assert_denied(
            &bundle,
            Some(&unknown_stage),
            DenialReason::UnknownValidationStage,
            Some(true),
        );
    }

    #[test]
    fn workspace_declaration_binding_fail_closed() {
        let base = valid_bundle();
        let validation = valid_validation(&base.subject);

        let stale = with_bundle(&base, |bundle| {
            bundle
                .workspace_admission_request
                .declaration_ref
                .as_mut()
                .expect("workspace declaration")
                .stale = true;
        });
        assert_denied(
            &stale,
            Some(&validation),
            DenialReason::StaleWorkspaceDeclaration,
            None,
        );

        let mismatched = with_bundle(&base, |bundle| {
            bundle
                .workspace_admission_request
                .declaration_ref
                .as_mut()
                .expect("workspace declaration")
                .subject
                .as_mut()
                .expect("workspace declaration subject")
                .digest = "sha256:workspace-subject-mismatch".to_string();
        });
        assert_denied(
            &mismatched,
            Some(&validation),
            DenialReason::ReferenceSubjectMismatch,
            None,
        );
    }

    #[test]
    fn authority_expansion_requests_fail_closed() {
        let base = valid_bundle();
        let validation = valid_validation(&base.subject);

        let cases: Vec<(StaticInputBundle, DenialReason)> = vec![
            (
                with_bundle(&base, |b| {
                    b.workspace_admission_request.requests_real_mount = true
                }),
                DenialReason::RealMountDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.workspace_admission_request.claims_workspace_handle = true
                }),
                DenialReason::WorkspaceHandleDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.workspace_admission_request.requests_capability_issuance = true
                }),
                DenialReason::CapabilityIssuanceDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.workspace_admission_request.requests_trust_assignment = true
                }),
                DenialReason::TrustAssignmentDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.workspace_admission_request
                        .requests_package_install_or_execution = true
                }),
                DenialReason::PackageInstallExecutionDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.workspace_admission_request.requests_plugin_loading = true
                }),
                DenialReason::PluginLoadingDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.expected_receipt_profile.declares_token_authority = true
                }),
                DenialReason::ReceiptTokenDenied,
            ),
            (
                with_bundle(&base, |b| b.authority_claims.trust_as_capability = true),
                DenialReason::TrustCapabilityDenied,
            ),
            (
                with_bundle(&base, |b| b.authority_claims.plugin_as_loading = true),
                DenialReason::PluginLoadingDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.authority_claims.semantic_cli_output_as_authority = true
                }),
                DenialReason::SemanticCliAuthorityDenied,
            ),
            (
                with_bundle(&base, |b| b.authority_claims.ai_output_as_authority = true),
                DenialReason::AiAuthorityDenied,
            ),
            (
                with_bundle(&base, |b| {
                    b.authority_claims.evidence_as_control_input = true
                }),
                DenialReason::EvidenceControlInputDenied,
            ),
        ];

        for (bundle, reason) in cases {
            assert_denied(&bundle, Some(&validation), reason, Some(true));
        }
    }

    #[test]
    fn kernel_abi_expansion_request_denies_before_input_binding() {
        let mut bundle = valid_bundle();
        bundle.shape.kernel_abi_expansion_requested = true;
        let validation = valid_validation(&bundle.subject);

        assert_denied(
            &bundle,
            Some(&validation),
            DenialReason::KernelAbiExpansionDenied,
            None,
        );
        assert_eq!(FROZEN_SYSCALL_RANGE, "1000-1011");
        assert_eq!(FROZEN_SYSCALL_COUNT, 12);
        assert_eq!(FROZEN_ABI_VERSION, "0x00010001");
    }

    #[test]
    fn changed_static_bundle_changes_success_digest() {
        let first_bundle = valid_bundle();
        let first_validation = valid_validation(&first_bundle.subject);
        let first_contents = valid_reference_contents(&first_bundle.subject);
        let first =
            run_harness(&first_bundle, Some(&first_validation), &first_contents).expect("first");

        let second_subject = Subject {
            version: "1.0.1".to_string(),
            digest: hash_bytes_prefixed(b"phase19-subject-v1.0.1"),
            ..first_bundle.subject.clone()
        };
        let second_bundle = bundle_for_subject(second_subject);
        let second_validation = valid_validation(&second_bundle.subject);
        let second_contents = valid_reference_contents(&second_bundle.subject);
        let second = run_harness(&second_bundle, Some(&second_validation), &second_contents)
            .expect("second");

        assert_ne!(first.input_bundle_digest, second.input_bundle_digest);
        assert_ne!(
            first
                .runtime_receipt
                .as_ref()
                .expect("first receipt")
                .digest,
            second
                .runtime_receipt
                .as_ref()
                .expect("second receipt")
                .digest
        );
    }

    #[test]
    fn reference_contract_and_version_fail_closed() {
        let base = valid_bundle();
        let validation = valid_validation(&base.subject);

        let unknown_contract = with_bundle(&base, |bundle| {
            bundle.manifest_ref.as_mut().expect("manifest").contract_id =
                "ayken.platform.module.manifest.v2".to_string();
        });
        assert_denied(
            &unknown_contract,
            Some(&validation),
            DenialReason::UnknownReferenceContract,
            None,
        );

        let unknown_version = with_bundle(&base, |bundle| {
            bundle
                .manifest_ref
                .as_mut()
                .expect("manifest")
                .schema_version = "2".to_string();
        });
        assert_denied(
            &unknown_version,
            Some(&validation),
            DenialReason::UnknownReferenceSchemaVersion,
            None,
        );

        let mut contents = valid_reference_contents(&base.subject);
        contents[0].reference_class = ReferenceClass::PackageMetadata;
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &contents,
            DenialReason::UnknownReferenceContract,
            None,
        );
    }

    #[test]
    fn reference_subject_binding_fail_closed() {
        let base = valid_bundle();
        let validation = valid_validation(&base.subject);

        let missing_subject = with_bundle(&base, |bundle| {
            bundle.manifest_ref.as_mut().expect("manifest").subject = None;
        });
        assert_denied(
            &missing_subject,
            Some(&validation),
            DenialReason::MissingReferenceSubject,
            None,
        );

        let mismatched_subject = with_bundle(&base, |bundle| {
            bundle
                .manifest_ref
                .as_mut()
                .expect("manifest")
                .subject
                .as_mut()
                .expect("manifest subject")
                .digest = hash_bytes_prefixed(b"different-subject");
        });
        assert_denied(
            &mismatched_subject,
            Some(&validation),
            DenialReason::ReferenceSubjectMismatch,
            None,
        );

        let mut contents = valid_reference_contents(&base.subject);
        contents[0].subject.digest = hash_bytes_prefixed(b"different-content-subject");
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &contents,
            DenialReason::ReferenceSubjectMismatch,
            None,
        );
    }

    #[test]
    fn reference_digest_shape_and_content_fail_closed() {
        let base = valid_bundle();
        let validation = valid_validation(&base.subject);

        let unsupported_algorithm = with_bundle(&base, |bundle| {
            bundle
                .manifest_ref
                .as_mut()
                .expect("manifest")
                .digest_algorithm = "sha512".to_string();
        });
        assert_denied(
            &unsupported_algorithm,
            Some(&validation),
            DenialReason::UnsupportedReferenceDigestAlgorithm,
            None,
        );

        let malformed = with_bundle(&base, |bundle| {
            bundle.manifest_ref.as_mut().expect("manifest").digest_value =
                "sha256:NOT-LOWER-HEX".to_string();
        });
        assert_denied(
            &malformed,
            Some(&validation),
            DenialReason::MalformedReferenceDigest,
            None,
        );

        let mut contents = valid_reference_contents(&base.subject);
        contents[0].content_bytes.push(b'!');
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &contents,
            DenialReason::ReferenceDigestMismatch,
            None,
        );
    }

    #[test]
    fn reference_content_cardinality_fail_closed() {
        let base = valid_bundle();
        let validation = valid_validation(&base.subject);

        let mut missing = valid_reference_contents(&base.subject);
        missing.remove(0);
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &missing,
            DenialReason::MissingReferenceContent,
            None,
        );

        let mut duplicate = valid_reference_contents(&base.subject);
        duplicate.push(duplicate[0].clone());
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &duplicate,
            DenialReason::DuplicateReferenceContent,
            None,
        );

        let mut duplicate_unexpected = valid_reference_contents(&base.subject);
        let unexpected = TestOwnedReferenceContent {
            path_or_uri: "test-owned://duplicate-undeclared-reference".to_string(),
            reference_class: ReferenceClass::ModuleManifest,
            subject: base.subject.clone(),
            content_bytes: b"duplicate-undeclared-content".to_vec(),
        };
        duplicate_unexpected.push(unexpected.clone());
        duplicate_unexpected.push(unexpected);
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &duplicate_unexpected,
            DenialReason::DuplicateReferenceContent,
            None,
        );

        let mut unexpected = valid_reference_contents(&base.subject);
        unexpected.push(TestOwnedReferenceContent {
            path_or_uri: "test-owned://undeclared-reference".to_string(),
            reference_class: ReferenceClass::ModuleManifest,
            subject: base.subject.clone(),
            content_bytes: b"undeclared-content".to_vec(),
        });
        assert_denied_with_contents(
            &base,
            Some(&validation),
            &unexpected,
            DenialReason::UnexpectedReferenceContent,
            None,
        );
    }

    #[test]
    fn validation_contract_and_stage_structure_fail_closed() {
        let bundle = valid_bundle();

        let mut contract = valid_validation(&bundle.subject);
        contract.contract_id = "ayken.platform.abi.validation.gate.v2".to_string();
        assert_denied(
            &bundle,
            Some(&contract),
            DenialReason::ValidationContractMismatch,
            Some(true),
        );

        let mut count = valid_validation(&bundle.subject);
        count.stage_results.pop();
        assert_denied(
            &bundle,
            Some(&count),
            DenialReason::ValidationStageCountMismatch,
            Some(true),
        );

        let mut unknown_id = valid_validation(&bundle.subject);
        unknown_id.stage_results[3].stage_id = "unknown_stage".to_string();
        assert_denied(
            &bundle,
            Some(&unknown_id),
            DenialReason::UnknownValidationStageId,
            Some(true),
        );

        let mut index = valid_validation(&bundle.subject);
        index.stage_results[3].stage_index = 4;
        assert_denied(
            &bundle,
            Some(&index),
            DenialReason::ValidationStageIndexMismatch,
            Some(true),
        );

        let mut order = valid_validation(&bundle.subject);
        order.stage_results.swap(2, 3);
        assert_denied(
            &bundle,
            Some(&order),
            DenialReason::ValidationStageOrderMismatch,
            Some(true),
        );
    }

    #[test]
    fn validation_stage_digest_fail_closed() {
        let bundle = valid_bundle();

        let mut algorithm = valid_validation(&bundle.subject);
        algorithm.stage_results[0].digest_algorithm = "sha512".to_string();
        assert_denied(
            &bundle,
            Some(&algorithm),
            DenialReason::ValidationStageDigestMismatch,
            Some(true),
        );

        let mut malformed = valid_validation(&bundle.subject);
        malformed.stage_results[0].digest_value = "sha256:bad".to_string();
        assert_denied(
            &bundle,
            Some(&malformed),
            DenialReason::ValidationStageDigestMismatch,
            Some(true),
        );

        let mut recomputation = valid_validation(&bundle.subject);
        recomputation.stage_results[0].content_bytes.push(b'!');
        assert_denied(
            &bundle,
            Some(&recomputation),
            DenialReason::ValidationStageDigestMismatch,
            Some(true),
        );
    }

    #[test]
    fn reference_integrity_denial_precedence_is_stable() {
        let base = valid_bundle();
        let mut validation = valid_validation(&base.subject);

        let contract_first = with_bundle(&base, |bundle| {
            let manifest = bundle.manifest_ref.as_mut().expect("manifest");
            manifest.contract_id = "unknown-contract".to_string();
            manifest.schema_version = "2".to_string();
            manifest.subject = None;
            manifest.digest_algorithm = "sha512".to_string();
            manifest.digest_value = "malformed".to_string();
        });
        assert_denied(
            &contract_first,
            Some(&validation),
            DenialReason::UnknownReferenceContract,
            None,
        );

        let version_first = with_bundle(&base, |bundle| {
            let manifest = bundle.manifest_ref.as_mut().expect("manifest");
            manifest.schema_version = "2".to_string();
            manifest.subject = None;
            manifest.digest_algorithm = "sha512".to_string();
        });
        assert_denied(
            &version_first,
            Some(&validation),
            DenialReason::UnknownReferenceSchemaVersion,
            None,
        );

        let classification_first = with_bundle(&base, |bundle| {
            let manifest = bundle.manifest_ref.as_mut().expect("manifest");
            manifest.schema_version = "2".to_string();
            manifest.digest_algorithm = "sha512".to_string();
        });
        let mut classification_contents = valid_reference_contents(&base.subject);
        classification_contents[0].reference_class = ReferenceClass::PackageMetadata;
        classification_contents[0].subject.digest = hash_bytes_prefixed(b"different-subject");
        classification_contents.remove(1);
        assert_denied_with_contents(
            &classification_first,
            Some(&validation),
            &classification_contents,
            DenialReason::UnknownReferenceContract,
            None,
        );

        let subject_first = with_bundle(&base, |bundle| {
            bundle
                .manifest_ref
                .as_mut()
                .expect("manifest")
                .digest_algorithm = "sha512".to_string();
        });
        let mut subject_contents = valid_reference_contents(&base.subject);
        subject_contents[0].subject.digest = hash_bytes_prefixed(b"different-subject");
        subject_contents.remove(1);
        assert_denied_with_contents(
            &subject_first,
            Some(&validation),
            &subject_contents,
            DenialReason::ReferenceSubjectMismatch,
            None,
        );

        validation.stage_results[0].stage_id = "unknown_stage".to_string();
        validation.stage_results[0].stage_index = 9;
        validation.stage_results[1].content_bytes.push(b'!');
        validation.stale_digest = true;
        assert_denied(
            &base,
            Some(&validation),
            DenialReason::UnknownValidationStageId,
            Some(true),
        );
    }

    #[test]
    fn reference_integrity_denial_is_deterministic() {
        let bundle = valid_bundle();
        let validation = valid_validation(&bundle.subject);
        let mut contents = valid_reference_contents(&bundle.subject);
        contents[0].content_bytes.push(b'!');

        let first = run_harness(&bundle, Some(&validation), &contents).expect("first denial");
        let second = run_harness(&bundle, Some(&validation), &contents).expect("second denial");

        assert_eq!(first, second);
        assert_eq!(first.status, HarnessStatus::Denied);
        assert_eq!(
            first.denial_record.as_ref().expect("denial").reason,
            DenialReason::ReferenceDigestMismatch
        );
    }

    fn assert_denied(
        bundle: &StaticInputBundle,
        validation: Option<&PlatformValidationEvidence>,
        reason: DenialReason,
        input_digest_present: Option<bool>,
    ) {
        let contents = valid_reference_contents(&bundle.subject);
        assert_denied_with_contents(bundle, validation, &contents, reason, input_digest_present);
    }

    fn assert_denied_with_contents(
        bundle: &StaticInputBundle,
        validation: Option<&PlatformValidationEvidence>,
        contents: &[TestOwnedReferenceContent],
        reason: DenialReason,
        input_digest_present: Option<bool>,
    ) {
        let first = run_harness(bundle, validation, contents).expect("first harness run");
        let second = run_harness(bundle, validation, contents).expect("second harness run");

        assert_eq!(first, second, "denial fixture must be deterministic");
        assert_eq!(first.status, HarnessStatus::Denied);
        assert!(first.validation_integration_record.is_none());
        assert!(first.workspace_admission_record.is_none());
        assert!(first.runtime_receipt.is_none());
        assert_eq!(first.denial_record.as_ref().expect("denial").reason, reason);
        if let Some(expected) = input_digest_present {
            assert_eq!(first.input_bundle_digest.is_some(), expected);
        }
    }

    fn with_bundle(
        base: &StaticInputBundle,
        edit: impl FnOnce(&mut StaticInputBundle),
    ) -> StaticInputBundle {
        let mut bundle = base.clone();
        edit(&mut bundle);
        bundle
    }

    fn valid_bundle() -> StaticInputBundle {
        let subject = Subject {
            kind: "phase19_static_test_bundle".to_string(),
            id: "bundle-alpha".to_string(),
            version: "1.0.0".to_string(),
            digest: hash_bytes_prefixed(b"phase19-subject-v1.0.0"),
        };

        bundle_for_subject(subject)
    }

    fn bundle_for_subject(subject: Subject) -> StaticInputBundle {
        StaticInputBundle {
            schema_id: INPUT_BUNDLE_SCHEMA_ID.to_string(),
            bundle_id: "phase19-static-bundle-alpha".to_string(),
            bundle_version: "1".to_string(),
            subject: subject.clone(),
            manifest_ref: Some(digest_ref_with_subject(
                MANIFEST_PATH,
                MODULE_MANIFEST_CONTRACT_ID,
                MANIFEST_BYTES,
                &subject,
            )),
            package_ref: Some(digest_ref_with_subject(
                PACKAGE_PATH,
                PACKAGE_METADATA_CONTRACT_ID,
                PACKAGE_BYTES,
                &subject,
            )),
            platform_validation_policy_ref: Some(digest_ref_with_subject(
                VALIDATION_POLICY_PATH,
                PLATFORM_VALIDATION_POLICY_CONTRACT_ID,
                VALIDATION_POLICY_BYTES,
                &subject,
            )),
            workspace_admission_request: WorkspaceAdmissionRequest {
                profile: "inert-record-only".to_string(),
                declaration_ref: Some(digest_ref_with_subject(
                    WORKSPACE_PATH,
                    WORKSPACE_DECLARATION_CONTRACT_ID,
                    WORKSPACE_BYTES,
                    &subject,
                )),
                requests_real_mount: false,
                claims_workspace_handle: false,
                requests_capability_issuance: false,
                requests_trust_assignment: false,
                requests_package_install_or_execution: false,
                requests_plugin_loading: false,
            },
            expected_receipt_profile: ReceiptProfile {
                profile_id: "deterministic-inert-receipt".to_string(),
                profile_version: "1".to_string(),
                declares_token_authority: false,
            },
            evidence_refs: vec![digest_ref_with_subject(
                EVIDENCE_MATRIX_PATH,
                RUNTIME_EVIDENCE_MATRIX_CONTRACT_ID,
                EVIDENCE_MATRIX_BYTES,
                &subject,
            )],
            shape: StaticBundleShape::default(),
            authority_claims: AuthorityClaims::default(),
        }
    }

    fn valid_validation(subject: &Subject) -> PlatformValidationEvidence {
        let stage_results = VALIDATION_STAGE_ORDER
            .iter()
            .map(|(stage_id, stage_index)| {
                let content_bytes = format!("{stage_index}:{stage_id}:pass").into_bytes();
                ValidationStageReference {
                    stage_id: (*stage_id).to_string(),
                    stage_index: *stage_index,
                    digest_algorithm: REFERENCE_DIGEST_ALGORITHM.to_string(),
                    digest_value: hash_bytes_prefixed(&content_bytes),
                    content_bytes,
                }
            })
            .collect();

        PlatformValidationEvidence {
            contract_id: PLATFORM_VALIDATION_RECEIPT_CONTRACT_ID.to_string(),
            schema_version: PLATFORM_VALIDATION_RECEIPT_SCHEMA_VERSION.to_string(),
            subject: subject.clone(),
            receipt_digest: hash_bytes_prefixed(b"platform-validation-receipt-alpha"),
            stage_results,
            status: PlatformValidationStatus::Pass,
            declares_authority_grant: false,
            unknown_stage_observed: false,
            stale_digest: false,
        }
    }

    fn valid_reference_contents(subject: &Subject) -> Vec<TestOwnedReferenceContent> {
        vec![
            test_owned_content(
                MANIFEST_PATH,
                ReferenceClass::ModuleManifest,
                subject,
                MANIFEST_BYTES,
            ),
            test_owned_content(
                PACKAGE_PATH,
                ReferenceClass::PackageMetadata,
                subject,
                PACKAGE_BYTES,
            ),
            test_owned_content(
                VALIDATION_POLICY_PATH,
                ReferenceClass::PlatformValidationPolicy,
                subject,
                VALIDATION_POLICY_BYTES,
            ),
            test_owned_content(
                WORKSPACE_PATH,
                ReferenceClass::WorkspaceDeclaration,
                subject,
                WORKSPACE_BYTES,
            ),
            test_owned_content(
                EVIDENCE_MATRIX_PATH,
                ReferenceClass::RuntimeEvidenceMatrix,
                subject,
                EVIDENCE_MATRIX_BYTES,
            ),
        ]
    }

    fn test_owned_content(
        path: &str,
        reference_class: ReferenceClass,
        subject: &Subject,
        bytes: &[u8],
    ) -> TestOwnedReferenceContent {
        TestOwnedReferenceContent {
            path_or_uri: path.to_string(),
            reference_class,
            subject: subject.clone(),
            content_bytes: bytes.to_vec(),
        }
    }

    fn digest_ref_with_subject(
        path: &str,
        contract_id: &str,
        bytes: &[u8],
        subject: &Subject,
    ) -> DigestReference {
        DigestReference {
            path_or_uri: path.to_string(),
            digest_algorithm: REFERENCE_DIGEST_ALGORITHM.to_string(),
            digest_value: hash_bytes_prefixed(bytes),
            contract_id: contract_id.to_string(),
            schema_version: REFERENCE_ENVELOPE_SCHEMA_VERSION.to_string(),
            subject: Some(subject.clone()),
            stale: false,
        }
    }
}
