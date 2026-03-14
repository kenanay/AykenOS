use crate::audit::schema::compute_receipt_hash;
use crate::authority::snapshot::validate_verifier_trust_registry_snapshot;
use crate::canonical::jcs::canonicalize_json;
use crate::receipt::verify::verify_signed_receipt_with_authority;
use crate::trust_reuse_runtime_surface::{
    compute_trust_reuse_runtime_event_id, write_trust_reuse_runtime_surface, TrustReuseOutcome,
    TrustReuseRuntimeEvent, TrustReuseRuntimeSurfaceReport,
};
use crate::types::{
    FindingSeverity, ReceiptVerifierKey, VerdictSubject, VerificationFinding, VerificationReceipt,
    VerifierAuthorityResolutionClass, VerifierTrustRegistrySnapshot,
};
use crate::verification_context_object::{
    load_verification_context_object, VerificationContextObject,
};
use crate::verifier_attestation::{
    compute_verifier_attestation_ref, load_verifier_attestation, verify_verifier_attestation,
    VerifierAttestation,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TrustReuseRuntimeEvaluatorConfig {
    pub receipt_path: PathBuf,
    pub verifier_key_path: PathBuf,
    pub expected_subject_path: PathBuf,
    pub verification_context_path: PathBuf,
    pub verifier_attestation_path: PathBuf,
    pub verifier_registry_path: PathBuf,
    pub output_path: PathBuf,
    pub output_dir: PathBuf,
    pub run_id: String,
    pub timestamp_unix_ns: u64,
    pub source_run_id: Option<String>,
    pub execution_cluster_id: Option<String>,
    pub lineage_id: Option<String>,
    pub reuse_group_id: Option<String>,
    pub surface_local_path_id: Option<String>,
    pub trust_reuse_source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrustReuseRuntimeEvaluatorOutcome {
    pub trust_reuse_outcome: TrustReuseOutcome,
    pub event_count: usize,
}

#[derive(Debug, Serialize)]
struct EvaluationReport {
    status: &'static str,
    trust_reuse_outcome: TrustReuseOutcome,
    run_id: String,
    timestamp_unix_ns: u64,
    receipt_path: String,
    verifier_key_path: String,
    expected_subject_path: String,
    verification_context_path: String,
    verifier_attestation_path: String,
    verifier_registry_path: String,
    output_path: String,
    surface_ref: String,
    receipt_ref: String,
    verification_context_ref: String,
    verifier_attestation_ref: String,
    authority_chain_id: String,
    authority_resolution_class: String,
    finding_count: usize,
    error_finding_count: usize,
    findings: Vec<EvaluationFinding>,
}

#[derive(Debug, Serialize)]
struct EvaluationFinding {
    code: String,
    severity: String,
    message: String,
}

pub fn run_trust_reuse_runtime_evaluator(
    config: &TrustReuseRuntimeEvaluatorConfig,
) -> Result<TrustReuseRuntimeEvaluatorOutcome, String> {
    if config.run_id.trim().is_empty() {
        return Err("run_id must not be empty for trust reuse runtime evaluator".to_string());
    }
    if config.timestamp_unix_ns == 0 {
        return Err(
            "timestamp_unix_ns must be non-zero for trust reuse runtime evaluator".to_string(),
        );
    }

    let receipt =
        load_json_file::<VerificationReceipt>(&config.receipt_path, "verification receipt")?;
    let verifier_key =
        load_json_file::<ReceiptVerifierKey>(&config.verifier_key_path, "receipt verifier key")?;
    let expected_subject = load_json_file::<VerdictSubject>(
        &config.expected_subject_path,
        "expected verdict subject",
    )?;
    let verification_context = load_verification_context_object(&config.verification_context_path)?;
    let verifier_attestation = load_verifier_attestation(&config.verifier_attestation_path)?;
    let verifier_registry = load_json_file::<VerifierTrustRegistrySnapshot>(
        &config.verifier_registry_path,
        "verifier trust registry snapshot",
    )?;

    let mut findings = Vec::new();
    findings.extend(validate_context_binding(
        &verification_context,
        &expected_subject,
        &receipt,
    ));
    findings.extend(validate_attestation_binding(
        &verifier_attestation,
        &verification_context,
        &receipt,
        &verifier_key,
    ));
    findings.extend(verify_verifier_attestation(
        &verifier_attestation,
        &verifier_registry,
    )?);

    let registry_validation = validate_verifier_trust_registry_snapshot(&verifier_registry)
        .map_err(|error| format!("failed to validate verifier trust registry: {error}"))?;
    findings.extend(registry_validation.findings);

    let distributed = verify_signed_receipt_with_authority(
        &receipt,
        &expected_subject,
        &verifier_key,
        &verifier_registry,
    )
    .map_err(|error| format!("failed to evaluate trust reuse receipt with authority: {error}"))?;
    findings.extend(distributed.findings.clone());

    let authority_chain_id = distributed
        .authority_resolution
        .authority_chain_id
        .clone()
        .ok_or_else(|| {
            "trust reuse runtime evaluator could not materialize authority_chain_id".to_string()
        })?;
    let trust_reuse_outcome =
        classify_trust_reuse_outcome(&distributed.authority_resolution.result_class, &findings);

    let receipt_ref = format!(
        "cas:sha256:{}",
        compute_receipt_hash(&receipt).map_err(|error| format!(
            "failed to compute receipt hash for trust reuse runtime evaluator: {error}"
        ))?
    );
    let verification_context_ref = format!("cas:{}", verification_context.verification_context_id);
    let verifier_attestation_ref = compute_verifier_attestation_ref(&verifier_attestation)?;
    let event = build_runtime_event(
        config,
        &expected_subject,
        &verification_context,
        &verifier_attestation,
        &verifier_registry,
        &receipt,
        &authority_chain_id,
        &receipt_ref,
        &verification_context_ref,
        &verifier_attestation_ref,
        trust_reuse_outcome.clone(),
    )?;
    let report = TrustReuseRuntimeSurfaceReport {
        surface_version: 1,
        flow_surface: "trust_reuse_runtime".to_string(),
        status: "PASS".to_string(),
        run_id: config.run_id.clone(),
        source_kind: "local_runtime_evidence".to_string(),
        event_count: 1,
        accepted_event_count: usize::from(trust_reuse_outcome == TrustReuseOutcome::Accepted),
        historical_only_event_count: usize::from(
            trust_reuse_outcome == TrustReuseOutcome::HistoricalOnly,
        ),
        rejected_event_count: usize::from(trust_reuse_outcome == TrustReuseOutcome::Rejected),
        events: vec![event],
    };
    write_trust_reuse_runtime_surface(&config.output_path, &report)?;
    write_evaluation_artifacts(
        config,
        &EvaluationReport {
            status: "PASS",
            trust_reuse_outcome: trust_reuse_outcome.clone(),
            run_id: config.run_id.clone(),
            timestamp_unix_ns: config.timestamp_unix_ns,
            receipt_path: config.receipt_path.display().to_string(),
            verifier_key_path: config.verifier_key_path.display().to_string(),
            expected_subject_path: config.expected_subject_path.display().to_string(),
            verification_context_path: config.verification_context_path.display().to_string(),
            verifier_attestation_path: config.verifier_attestation_path.display().to_string(),
            verifier_registry_path: config.verifier_registry_path.display().to_string(),
            output_path: config.output_path.display().to_string(),
            surface_ref: format!("cas:sha256:{}", sha256_hex_from_file(&config.output_path)?),
            receipt_ref,
            verification_context_ref,
            verifier_attestation_ref,
            authority_chain_id,
            authority_resolution_class: format!(
                "{:?}",
                distributed.authority_resolution.result_class
            ),
            finding_count: findings.len(),
            error_finding_count: findings
                .iter()
                .filter(|finding| finding.severity == FindingSeverity::Error)
                .count(),
            findings: findings
                .iter()
                .map(EvaluationFinding::from_finding)
                .collect(),
        },
        &findings,
    )?;

    Ok(TrustReuseRuntimeEvaluatorOutcome {
        trust_reuse_outcome,
        event_count: 1,
    })
}

fn build_runtime_event(
    config: &TrustReuseRuntimeEvaluatorConfig,
    expected_subject: &VerdictSubject,
    verification_context: &VerificationContextObject,
    verifier_attestation: &VerifierAttestation,
    verifier_registry: &VerifierTrustRegistrySnapshot,
    receipt: &VerificationReceipt,
    authority_chain_id: &str,
    receipt_ref: &str,
    verification_context_ref: &str,
    verifier_attestation_ref: &str,
    trust_reuse_outcome: TrustReuseOutcome,
) -> Result<TrustReuseRuntimeEvent, String> {
    let mut event = TrustReuseRuntimeEvent {
        event_schema_version: 1,
        event_id: String::new(),
        run_id: config.run_id.clone(),
        timestamp_unix_ns: config.timestamp_unix_ns,
        subject_bundle_id: expected_subject.bundle_id.clone(),
        verification_context_id: verification_context.verification_context_id.clone(),
        authority_chain_id: authority_chain_id.to_string(),
        trust_reuse_outcome,
        terminal: true,
        reused: true,
        receipt_ref: receipt_ref.to_string(),
        verification_context_ref: verification_context_ref.to_string(),
        verifier_attestation_ref: verifier_attestation_ref.to_string(),
        verifier_registry_snapshot_hash: strip_sha256_prefix(
            &verifier_registry.verifier_registry_snapshot_hash,
        )?
        .to_string(),
        verification_node_id: Some(receipt.payload.verifier_node_id.clone()),
        verifier_id: Some(verifier_attestation.verifier_id.clone()),
        lineage_id: config.lineage_id.clone(),
        execution_cluster_id: config.execution_cluster_id.clone(),
        source_run_id: Some(
            config
                .source_run_id
                .clone()
                .unwrap_or_else(|| config.run_id.clone()),
        ),
        reuse_group_id: config.reuse_group_id.clone(),
        surface_local_path_id: Some(
            config
                .surface_local_path_id
                .clone()
                .unwrap_or_else(|| "reports/trust_reuse_runtime_surface.json".to_string()),
        ),
        trust_reuse_source: Some(
            config
                .trust_reuse_source
                .clone()
                .unwrap_or_else(|| "native-runtime-trust-reuse-evaluator".to_string()),
        ),
    };
    event.event_id = compute_trust_reuse_runtime_event_id(&event)?;
    Ok(event)
}

fn validate_context_binding(
    verification_context: &VerificationContextObject,
    expected_subject: &VerdictSubject,
    receipt: &VerificationReceipt,
) -> Vec<VerificationFinding> {
    let mut findings = Vec::new();

    if verification_context.policy_hash != expected_subject.policy_hash {
        findings.push(VerificationFinding::error(
            "PV1201",
            "verification context policy_hash does not match expected verdict subject",
        ));
    }
    if verification_context.registry_snapshot_hash != expected_subject.registry_snapshot_hash {
        findings.push(VerificationFinding::error(
            "PV1202",
            "verification context registry_snapshot_hash does not match expected verdict subject",
        ));
    }
    if verification_context.policy_hash != receipt.payload.policy_hash {
        findings.push(VerificationFinding::error(
            "PV1203",
            "verification context policy_hash does not match receipt payload",
        ));
    }
    if verification_context.registry_snapshot_hash != receipt.payload.registry_snapshot_hash {
        findings.push(VerificationFinding::error(
            "PV1204",
            "verification context registry_snapshot_hash does not match receipt payload",
        ));
    }

    findings
}

fn validate_attestation_binding(
    verifier_attestation: &VerifierAttestation,
    verification_context: &VerificationContextObject,
    receipt: &VerificationReceipt,
    verifier_key: &ReceiptVerifierKey,
) -> Vec<VerificationFinding> {
    let mut findings = Vec::new();

    if verifier_attestation.verifier_id != receipt.payload.verifier_node_id {
        findings.push(VerificationFinding::error(
            "PV1205",
            "verifier attestation verifier_id does not match receipt payload verifier_node_id",
        ));
    }
    if receipt.payload.verifier_key_id.as_deref()
        != Some(verifier_attestation.verifier_pubkey_id.as_str())
    {
        findings.push(VerificationFinding::error(
            "PV1206",
            "verifier attestation verifier_pubkey_id does not match receipt payload verifier_key_id",
        ));
    }
    if verifier_attestation.verifier_pubkey_id != verifier_key.verifier_key_id {
        findings.push(VerificationFinding::error(
            "PV1207",
            "verifier attestation verifier_pubkey_id does not match receipt verifier key identity",
        ));
    }
    if verifier_attestation.verifier_contract_version
        != verification_context.verifier_contract_version
    {
        findings.push(VerificationFinding::error(
            "PV1208",
            "verifier attestation verifier_contract_version does not match verification context contract version",
        ));
    }

    findings
}

fn classify_trust_reuse_outcome(
    result_class: &VerifierAuthorityResolutionClass,
    findings: &[VerificationFinding],
) -> TrustReuseOutcome {
    let error_codes: Vec<&str> = findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Error)
        .map(|finding| finding.code.as_str())
        .collect();

    if matches!(
        result_class,
        VerifierAuthorityResolutionClass::AuthorityHistoricalOnly
    ) && !error_codes.is_empty()
        && error_codes.iter().all(|code| *code == "PV0711")
    {
        return TrustReuseOutcome::HistoricalOnly;
    }

    if error_codes.is_empty()
        && matches!(
            result_class,
            VerifierAuthorityResolutionClass::AuthorityResolvedRoot
                | VerifierAuthorityResolutionClass::AuthorityResolvedDelegated
        )
    {
        return TrustReuseOutcome::Accepted;
    }

    TrustReuseOutcome::Rejected
}

fn write_evaluation_artifacts(
    config: &TrustReuseRuntimeEvaluatorConfig,
    report: &EvaluationReport,
    findings: &[VerificationFinding],
) -> Result<(), String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create trust reuse runtime evaluator output dir {}: {error}",
            config.output_dir.display()
        )
    })?;
    let report_path = config
        .output_dir
        .join("trust_reuse_runtime_surface_evaluate_report.json");
    let report_bytes = canonicalize_json(report).map_err(|error| {
        format!("failed to canonicalize trust reuse runtime evaluation report: {error}")
    })?;
    fs::write(&report_path, report_bytes).map_err(|error| {
        format!(
            "failed to write trust reuse runtime evaluation report {}: {error}",
            report_path.display()
        )
    })?;
    let violations_path = config.output_dir.join("violations.txt");
    let mut lines = Vec::new();
    for finding in findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Error)
    {
        lines.push(format!("{}: {}", finding.code, finding.message));
    }
    fs::write(&violations_path, lines.join("\n")).map_err(|error| {
        format!(
            "failed to write trust reuse runtime evaluation violations {}: {error}",
            violations_path.display()
        )
    })?;
    Ok(())
}

fn load_json_file<T>(path: &Path, label: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read {label} at {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {label} at {}: {error}", path.display()))
}

fn strip_sha256_prefix(value: &str) -> Result<&str, String> {
    value
        .strip_prefix("sha256:")
        .ok_or_else(|| format!("expected sha256:<hex> value, found {value}"))
}

fn sha256_hex_from_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read file hash input at {}: {error}",
            path.display()
        )
    })?;
    Ok(crate::canonical::digest::sha256_hex(&bytes))
}

impl EvaluationFinding {
    fn from_finding(finding: &VerificationFinding) -> Self {
        Self {
            code: finding.code.clone(),
            severity: match finding.severity {
                FindingSeverity::Info => "info",
                FindingSeverity::Warning => "warning",
                FindingSeverity::Error => "error",
            }
            .to_string(),
            message: finding.message.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{run_trust_reuse_runtime_evaluator, TrustReuseRuntimeEvaluatorConfig};
    use crate::authority::snapshot::compute_verifier_trust_registry_snapshot_hash;
    use crate::canonical::jcs::canonicalize_json;
    use crate::crypto::ed25519::sign_ed25519_bytes;
    use crate::testing::fixtures::create_fixture_bundle;
    use crate::types::{
        AuditMode, ReceiptMode, VerdictSubject, VerificationReceipt, VerifierAuthorityState,
        VerifyRequest,
    };
    use crate::verification_context_object::{
        compute_verification_context_id, write_verification_context_object,
        VerificationContextObject,
    };
    use crate::verifier_attestation::{write_verifier_attestation, VerifierAttestation};
    use crate::verify_bundle;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("trust-reuse-runtime-evaluator-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn build_context(subject: &VerdictSubject) -> VerificationContextObject {
        let mut context = VerificationContextObject {
            context_version: 1,
            verification_context_id: String::new(),
            policy_hash: subject.policy_hash.clone(),
            registry_snapshot_hash: subject.registry_snapshot_hash.clone(),
            verifier_contract_version: "phase12-context-v1".to_string(),
            context_rules_hash: "c".repeat(64),
            context_epoch: Some(1),
            historical_cutoff_utc: None,
            policy_snapshot_ref: Some(format!("cas:sha256:{}", "d".repeat(64))),
            registry_snapshot_ref: Some(format!("cas:sha256:{}", "e".repeat(64))),
            time_semantics_mode: Some("historical-aware".to_string()),
        };
        context.verification_context_id =
            compute_verification_context_id(&context).expect("compute context id");
        context
    }

    fn build_attestation(fixture: &crate::testing::fixtures::FixtureBundle) -> VerifierAttestation {
        let mut attestation = VerifierAttestation {
            attestation_version: 1,
            verifier_id: fixture.receipt_signer.verifier_node_id.clone(),
            verifier_pubkey_id: fixture.receipt_signer.verifier_key_id.clone(),
            verifier_registry_ref: fixture.verifier_registry.registry_scope.clone(),
            verifier_key_epoch: u64::from(fixture.verifier_registry.verifier_registry_epoch),
            verifier_contract_version: "phase12-context-v1".to_string(),
            attestation_signature_algorithm: "ed25519".to_string(),
            attestation_signature: String::new(),
            attested_at_utc: Some("2026-03-14T00:00:00Z".to_string()),
        };
        let mut payload = serde_json::to_value(&attestation).expect("serialize attestation");
        payload
            .as_object_mut()
            .expect("attestation object")
            .remove("attestation_signature");
        let payload_bytes = crate::canonical::jcs::canonicalize_json_value(&payload)
            .expect("canonicalize attestation payload");
        attestation.attestation_signature =
            sign_ed25519_bytes(&fixture.receipt_signer.private_key, &payload_bytes)
                .expect("sign attestation");
        attestation
    }

    fn write_runtime_artifacts(
        fixture: &crate::testing::fixtures::FixtureBundle,
        subject: &VerdictSubject,
        receipt: &VerificationReceipt,
        dir: &std::path::Path,
        verifier_registry: &crate::VerifierTrustRegistrySnapshot,
        context_mutator: impl FnOnce(&mut VerificationContextObject),
    ) -> TrustReuseRuntimeEvaluatorConfig {
        let receipt_path = dir.join("verification_receipt.json");
        let verifier_key_path = dir.join("receipt_verifier_key.json");
        let expected_subject_path = dir.join("expected_subject.json");
        let verification_context_path = dir.join("verification_context_object.json");
        let verifier_attestation_path = dir.join("verifier_attestation.json");
        let verifier_registry_path = dir.join("verifier_registry.json");
        let output_dir = dir.join("reports");
        let output_path = output_dir.join("trust_reuse_runtime_surface.json");
        fs::create_dir_all(&output_dir).expect("create output dir");

        fs::write(
            &receipt_path,
            canonicalize_json(receipt).expect("serialize receipt"),
        )
        .expect("write receipt");
        fs::write(
            &verifier_key_path,
            canonicalize_json(&fixture.receipt_verifier_key).expect("serialize verifier key"),
        )
        .expect("write verifier key");
        fs::write(
            &expected_subject_path,
            canonicalize_json(subject).expect("serialize subject"),
        )
        .expect("write subject");
        let mut context = build_context(subject);
        context_mutator(&mut context);
        context.verification_context_id =
            compute_verification_context_id(&context).expect("recompute context id");
        write_verification_context_object(&verification_context_path, &context)
            .expect("write verification context");
        let attestation = build_attestation(fixture);
        write_verifier_attestation(&verifier_attestation_path, &attestation)
            .expect("write attestation");
        fs::write(
            &verifier_registry_path,
            canonicalize_json(verifier_registry).expect("serialize verifier registry"),
        )
        .expect("write verifier registry");

        TrustReuseRuntimeEvaluatorConfig {
            receipt_path,
            verifier_key_path,
            expected_subject_path,
            verification_context_path,
            verifier_attestation_path,
            verifier_registry_path,
            output_path,
            output_dir,
            run_id: "trust-reuse-run-a".to_string(),
            timestamp_unix_ns: 1_710_000_123_000_000_000,
            source_run_id: Some("source-run-a".to_string()),
            execution_cluster_id: Some("cluster-a".to_string()),
            lineage_id: Some("lineage-a".to_string()),
            reuse_group_id: Some("reuse-group-a".to_string()),
            surface_local_path_id: Some("reports/trust_reuse_runtime_surface.json".to_string()),
            trust_reuse_source: Some("runtime-evaluator".to_string()),
        }
    }

    #[test]
    fn evaluator_emits_accepted_runtime_surface() {
        let fixture = create_fixture_bundle();
        let request = VerifyRequest {
            bundle_path: &fixture.root,
            policy: &fixture.policy,
            registry_snapshot: &fixture.registry,
            receipt_mode: ReceiptMode::EmitSigned,
            receipt_signer: Some(&fixture.receipt_signer),
            audit_mode: AuditMode::None,
            audit_ledger_path: None,
        };
        let outcome = verify_bundle(&request).expect("verify bundle");
        let receipt = outcome.receipt.as_ref().expect("signed receipt").clone();
        let dir = temp_dir();
        let config = write_runtime_artifacts(
            &fixture,
            &outcome.subject,
            &receipt,
            &dir,
            &fixture.verifier_registry,
            |_| {},
        );

        let result = run_trust_reuse_runtime_evaluator(&config).expect("run evaluator");
        assert_eq!(
            result.trust_reuse_outcome,
            crate::trust_reuse_runtime_surface::TrustReuseOutcome::Accepted
        );
        let surface: crate::trust_reuse_runtime_surface::TrustReuseRuntimeSurfaceReport =
            serde_json::from_slice(&fs::read(&config.output_path).expect("read surface output"))
                .expect("parse surface output");
        assert_eq!(surface.accepted_event_count, 1);
        assert_eq!(
            surface.events[0].trust_reuse_outcome,
            crate::trust_reuse_runtime_surface::TrustReuseOutcome::Accepted
        );
    }

    #[test]
    fn evaluator_emits_historical_only_runtime_surface() {
        let fixture = create_fixture_bundle();
        let request = VerifyRequest {
            bundle_path: &fixture.root,
            policy: &fixture.policy,
            registry_snapshot: &fixture.registry,
            receipt_mode: ReceiptMode::EmitSigned,
            receipt_signer: Some(&fixture.receipt_signer),
            audit_mode: AuditMode::None,
            audit_ledger_path: None,
        };
        let outcome = verify_bundle(&request).expect("verify bundle");
        let receipt = outcome.receipt.as_ref().expect("signed receipt").clone();
        let mut historical_registry = fixture.verifier_registry.clone();
        historical_registry
            .verifiers
            .get_mut("node-b")
            .expect("node-b registry entry")
            .authority_state = VerifierAuthorityState::HistoricalOnly;
        historical_registry.verifier_registry_snapshot_hash =
            compute_verifier_trust_registry_snapshot_hash(&historical_registry)
                .expect("recompute historical registry hash");
        let dir = temp_dir();
        let config = write_runtime_artifacts(
            &fixture,
            &outcome.subject,
            &receipt,
            &dir,
            &historical_registry,
            |_| {},
        );

        let result = run_trust_reuse_runtime_evaluator(&config).expect("run evaluator");
        assert_eq!(
            result.trust_reuse_outcome,
            crate::trust_reuse_runtime_surface::TrustReuseOutcome::HistoricalOnly
        );
    }

    #[test]
    fn evaluator_emits_rejected_runtime_surface_for_context_drift() {
        let fixture = create_fixture_bundle();
        let request = VerifyRequest {
            bundle_path: &fixture.root,
            policy: &fixture.policy,
            registry_snapshot: &fixture.registry,
            receipt_mode: ReceiptMode::EmitSigned,
            receipt_signer: Some(&fixture.receipt_signer),
            audit_mode: AuditMode::None,
            audit_ledger_path: None,
        };
        let outcome = verify_bundle(&request).expect("verify bundle");
        let receipt = outcome.receipt.as_ref().expect("signed receipt").clone();
        let dir = temp_dir();
        let config = write_runtime_artifacts(
            &fixture,
            &outcome.subject,
            &receipt,
            &dir,
            &fixture.verifier_registry,
            |context| {
                context.policy_hash = "f".repeat(64);
            },
        );

        let result = run_trust_reuse_runtime_evaluator(&config).expect("run evaluator");
        assert_eq!(
            result.trust_reuse_outcome,
            crate::trust_reuse_runtime_surface::TrustReuseOutcome::Rejected
        );
    }
}
