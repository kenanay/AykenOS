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
pub const PLATFORM_VALIDATION_RECEIPT_CONTRACT_ID: &str =
    "ayken.phase18.platform_abi_validation.receipt.v1";
pub const FROZEN_SYSCALL_RANGE: &str = "1000-1011";
pub const FROZEN_SYSCALL_COUNT: u16 = 12;
pub const FROZEN_ABI_VERSION: &str = "0x00010001";

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
    pub stage_result_digests: Vec<String>,
    pub status: PlatformValidationStatus,
    #[serde(default)]
    pub declares_authority_grant: bool,
    #[serde(default)]
    pub unknown_stage_observed: bool,
    #[serde(default)]
    pub stale_digest: bool,
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
    SubjectMismatch,
    MissingPlatformValidation,
    PlatformValidationFailed,
    ValidationAuthorityDenied,
    ValidationStaleDigest,
    UnknownValidationStage,
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

    let manifest_ref = bundle
        .manifest_ref
        .as_ref()
        .expect("manifest_ref checked above");
    if manifest_ref.stale {
        return denied_before_input(DenialReason::StaleManifestDigest);
    }

    if let Some(subject) = &manifest_ref.subject {
        if subject != &bundle.subject {
            return denied_before_input(DenialReason::SubjectMismatch);
        }
    }

    if let Some(package_ref) = &bundle.package_ref {
        if package_ref.stale {
            return denied_before_input(DenialReason::StaleManifestDigest);
        }
        if let Some(subject) = &package_ref.subject {
            if subject != &bundle.subject {
                return denied_before_input(DenialReason::SubjectMismatch);
            }
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

    if validation.contract_id != PLATFORM_VALIDATION_RECEIPT_CONTRACT_ID
        || validation.subject != bundle.subject
    {
        return denied_after_input(
            input_bound_transcript,
            Some(input_bundle_digest),
            DenialReason::SubjectMismatch,
        );
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

    let seed = ValidationRecordSeed {
        schema_id: VALIDATION_INTEGRATION_RECORD_SCHEMA_ID,
        input_bundle_digest,
        validation_receipt_digest: &validation.receipt_digest,
        stage_count: validation.stage_result_digests.len(),
        stage_result_digests: &validation.stage_result_digests,
        decision_status: "recordable",
    };

    Ok(ValidationIntegrationRecord {
        schema_id: VALIDATION_INTEGRATION_RECORD_SCHEMA_ID.to_string(),
        input_bundle_digest: input_bundle_digest.to_string(),
        validation_receipt_digest: validation.receipt_digest.clone(),
        stage_count: validation.stage_result_digests.len(),
        stage_result_digests: validation.stage_result_digests.clone(),
        decision_status: "recordable".to_string(),
        digest: canonical_hash_prefixed(&seed)?,
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

    #[test]
    fn positive_flow_emits_inert_deterministic_records() {
        let bundle = valid_bundle();
        let validation = valid_validation(&bundle.subject);

        let first = run_harness(&bundle, Some(&validation)).expect("first run");
        let second = run_harness(&bundle, Some(&validation)).expect("second run");

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
        let first = run_harness(&first_bundle, Some(&first_validation)).expect("first");

        let mut second_bundle = valid_bundle();
        second_bundle.subject.version = "1.0.1".to_string();
        second_bundle.subject.digest = "sha256:subject-digest-v101".to_string();
        second_bundle.manifest_ref = Some(digest_ref_with_subject(
            "docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md",
            "ayken.phase18.module_manifest.schema.v1",
            &second_bundle.subject,
        ));
        second_bundle.package_ref = Some(digest_ref_with_subject(
            "docs/specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md",
            "ayken.phase18.package_metadata.schema.v1",
            &second_bundle.subject,
        ));
        second_bundle.workspace_admission_request.declaration_ref = Some(digest_ref_with_subject(
            "docs/specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md",
            "ayken.phase18.workspace_lifecycle.specification.v1",
            &second_bundle.subject,
        ));
        let second_validation = valid_validation(&second_bundle.subject);
        let second = run_harness(&second_bundle, Some(&second_validation)).expect("second");

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

    fn assert_denied(
        bundle: &StaticInputBundle,
        validation: Option<&PlatformValidationEvidence>,
        reason: DenialReason,
        input_digest_present: Option<bool>,
    ) {
        let outcome = run_harness(bundle, validation).expect("harness run");
        assert_eq!(outcome.status, HarnessStatus::Denied);
        assert!(outcome.validation_integration_record.is_none());
        assert!(outcome.workspace_admission_record.is_none());
        assert!(outcome.runtime_receipt.is_none());
        assert_eq!(
            outcome.denial_record.as_ref().expect("denial").reason,
            reason
        );
        if let Some(expected) = input_digest_present {
            assert_eq!(outcome.input_bundle_digest.is_some(), expected);
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
            digest: "sha256:subject-digest-v100".to_string(),
        };

        StaticInputBundle {
            schema_id: INPUT_BUNDLE_SCHEMA_ID.to_string(),
            bundle_id: "phase19-static-bundle-alpha".to_string(),
            bundle_version: "1".to_string(),
            subject: subject.clone(),
            manifest_ref: Some(digest_ref_with_subject(
                "docs/specs/phase18-platform-constitution/MODULE_MANIFEST_SCHEMA.md",
                "ayken.phase18.module_manifest.schema.v1",
                &subject,
            )),
            package_ref: Some(digest_ref_with_subject(
                "docs/specs/phase18-platform-constitution/PACKAGE_METADATA_SCHEMA.md",
                "ayken.phase18.package_metadata.schema.v1",
                &subject,
            )),
            platform_validation_policy_ref: Some(digest_ref(
                "docs/specs/phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md",
                "ayken.phase18.platform_abi_validation_gate.v1",
            )),
            workspace_admission_request: WorkspaceAdmissionRequest {
                profile: "inert-record-only".to_string(),
                declaration_ref: Some(digest_ref_with_subject(
                    "docs/specs/phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md",
                    "ayken.phase18.workspace_lifecycle.specification.v1",
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
            evidence_refs: vec![digest_ref(
                "docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md",
                "ayken.phase19.runtime.evidence_matrix.v1",
            )],
            shape: StaticBundleShape::default(),
            authority_claims: AuthorityClaims::default(),
        }
    }

    fn valid_validation(subject: &Subject) -> PlatformValidationEvidence {
        PlatformValidationEvidence {
            contract_id: PLATFORM_VALIDATION_RECEIPT_CONTRACT_ID.to_string(),
            schema_version: "1".to_string(),
            subject: subject.clone(),
            receipt_digest: "sha256:platform-validation-receipt-alpha".to_string(),
            stage_result_digests: vec![
                "sha256:manifest-stage".to_string(),
                "sha256:workspace-stage".to_string(),
                "sha256:receipt-boundary-stage".to_string(),
            ],
            status: PlatformValidationStatus::Pass,
            declares_authority_grant: false,
            unknown_stage_observed: false,
            stale_digest: false,
        }
    }

    fn digest_ref(path: &str, contract_id: &str) -> DigestReference {
        DigestReference {
            path_or_uri: path.to_string(),
            digest_algorithm: "sha256".to_string(),
            digest_value: format!("sha256:{}", path.replace('/', "_")),
            contract_id: contract_id.to_string(),
            schema_version: "1".to_string(),
            subject: None,
            stale: false,
        }
    }

    fn digest_ref_with_subject(
        path: &str,
        contract_id: &str,
        subject: &Subject,
    ) -> DigestReference {
        DigestReference {
            subject: Some(subject.clone()),
            ..digest_ref(path, contract_id)
        }
    }
}
