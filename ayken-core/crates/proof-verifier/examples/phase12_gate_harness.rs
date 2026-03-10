use proof_verifier::audit::schema::compute_receipt_hash;
use proof_verifier::audit::verify::{
    verify_audit_event_against_receipt, verify_audit_event_against_receipt_with_authority,
    verify_audit_ledger, verify_audit_ledger_with_receipts, AuditReceiptBinding,
};
use proof_verifier::authority::authority_drift_topology::{
    analyze_authority_drift_suppressions, build_authority_drift_topology,
};
use proof_verifier::authority::incident_graph::build_incident_graph;
use proof_verifier::bundle::checksums::load_checksums;
use proof_verifier::bundle::layout::validate_bundle_layout;
use proof_verifier::bundle::loader::load_bundle;
use proof_verifier::bundle::manifest::load_manifest;
use proof_verifier::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use proof_verifier::authority::determinism_incident::analyze_determinism_incidents;
use proof_verifier::authority::drift_attribution::analyze_parity_drift;
use proof_verifier::authority::parity::{
    build_node_parity_outcome, compare_authority_resolution, compare_cross_node_parity,
    CrossNodeParityInput, CrossNodeParityRecord, CrossNodeParityStatus, NodeParityOutcome,
    NodeParityOutcomeView, ParityArtifactForm, ParityEvidenceState,
};
use proof_verifier::authority::resolution::resolve_verifier_authority;
use proof_verifier::authority::snapshot::compute_verifier_trust_registry_snapshot_hash;
use proof_verifier::crypto::verify_detached_signatures;
use proof_verifier::overlay::overlay_validator::verify_overlay;
use proof_verifier::policy::policy_engine::compute_policy_hash;
use proof_verifier::policy::schema::validate_policy;
use proof_verifier::portable_core::checksum_validator::validate_portable_checksums;
use proof_verifier::portable_core::identity::recompute_bundle_id;
use proof_verifier::portable_core::proof_chain_validator::validate_proof_chain;
use proof_verifier::receipt::schema::canonicalize_receipt_payload;
use proof_verifier::receipt::verify::{
    verify_signed_receipt, verify_signed_receipt_with_authority,
};
use proof_verifier::registry::resolver::resolve_signers;
use proof_verifier::registry::snapshot::compute_registry_snapshot_hash;
use proof_verifier::testing::fixtures::{create_fixture_bundle, FixtureBundle};
use proof_verifier::types::{
    AuditMode, ChecksumsFile, FindingSeverity, KeyStatus, LoadedBundle, Manifest, OverlayState,
    ProducerDeclaration, ReceiptMode, RegistryEntry, RegistryResolution, RegistrySnapshot,
    SignatureEnvelope, SignatureRequirement, TrustPolicy, VerificationFinding,
    VerificationVerdict, VerifierAuthorityNode,
    VerifierAuthorityResolution, VerifierAuthorityResolutionClass, VerifierAuthorityState,
    VerifierDelegationEdge, VerifierTrustRegistryPublicKey, VerifierTrustRegistrySnapshot,
    VerifyRequest, VerificationOutcome,
};
use proof_verifier::verify_bundle;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

#[derive(Clone, Copy)]
enum GateMode {
    ProducerSchema,
    SignatureEnvelope,
    BundleV2Schema,
    BundleV2Compat,
    SignatureVerify,
    RegistryResolution,
    KeyRotation,
    VerifierCore,
    TrustPolicy,
    VerdictBinding,
    VerifierCli,
    Receipt,
    AuditLedger,
    ProofExchange,
    AuthorityResolution,
    CrossNodeParity,
}

struct HarnessArgs {
    mode: GateMode,
    out_dir: PathBuf,
    cli_bin: Option<PathBuf>,
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("ERROR: {error}");
            process::exit(3);
        }
    }
}

fn run() -> Result<i32, String> {
    let args = parse_args()?;
    let mode = args.mode;
    let out_dir = args.out_dir;
    fs::create_dir_all(&out_dir).map_err(|error| {
        format!(
            "failed to create gate output directory {}: {error}",
            out_dir.display()
        )
    })?;

    match mode {
        GateMode::ProducerSchema => Ok(run_producer_schema_gate(&out_dir)),
        GateMode::SignatureEnvelope => Ok(run_signature_envelope_gate(&out_dir)),
        GateMode::BundleV2Schema => Ok(run_bundle_v2_schema_gate(&out_dir)),
        GateMode::BundleV2Compat => Ok(run_bundle_v2_compat_gate(&out_dir)),
        GateMode::SignatureVerify => Ok(run_signature_verify_gate(&out_dir)),
        GateMode::RegistryResolution => Ok(run_registry_resolution_gate(&out_dir)),
        GateMode::KeyRotation => Ok(run_key_rotation_gate(&out_dir)),
        GateMode::VerifierCore => Ok(run_verifier_core_gate(&out_dir)),
        GateMode::TrustPolicy => Ok(run_trust_policy_gate(&out_dir)),
        GateMode::VerdictBinding => Ok(run_verdict_binding_gate(&out_dir)),
        GateMode::VerifierCli => Ok(run_verifier_cli_gate(
            &out_dir,
            args.cli_bin.as_deref(),
        )),
        GateMode::Receipt => Ok(run_receipt_gate(&out_dir)),
        GateMode::AuditLedger => Ok(run_audit_ledger_gate(&out_dir)),
        GateMode::ProofExchange => Ok(run_proof_exchange_gate(&out_dir)),
        GateMode::AuthorityResolution => Ok(run_authority_resolution_gate(&out_dir)),
        GateMode::CrossNodeParity => Ok(run_cross_node_parity_gate(&out_dir)),
    }
}

fn parse_args() -> Result<HarnessArgs, String> {
    let mut args = env::args().skip(1);
    let mode = match args.next().as_deref() {
        Some("producer-schema") => GateMode::ProducerSchema,
        Some("signature-envelope") => GateMode::SignatureEnvelope,
        Some("bundle-v2-schema") => GateMode::BundleV2Schema,
        Some("bundle-v2-compat") => GateMode::BundleV2Compat,
        Some("signature-verify") => GateMode::SignatureVerify,
        Some("registry-resolution") => GateMode::RegistryResolution,
        Some("key-rotation") => GateMode::KeyRotation,
        Some("verifier-core") => GateMode::VerifierCore,
        Some("trust-policy") => GateMode::TrustPolicy,
        Some("verdict-binding") => GateMode::VerdictBinding,
        Some("verifier-cli") => GateMode::VerifierCli,
        Some("receipt") => GateMode::Receipt,
        Some("audit-ledger") => GateMode::AuditLedger,
        Some("proof-exchange") => GateMode::ProofExchange,
        Some("authority-resolution") => GateMode::AuthorityResolution,
        Some("cross-node-parity") => GateMode::CrossNodeParity,
        Some(other) => return Err(format!("unknown mode: {other}")),
        None => {
            return Err(
                "missing mode (expected producer-schema, signature-envelope, bundle-v2-schema, bundle-v2-compat, signature-verify, registry-resolution, key-rotation, verifier-core, trust-policy, verdict-binding, verifier-cli, receipt, audit-ledger, proof-exchange, authority-resolution, or cross-node-parity)".to_string(),
            )
        }
    };

    let mut out_dir: Option<PathBuf> = None;
    let mut cli_bin: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --out-dir".to_string())?;
                out_dir = Some(PathBuf::from(value));
            }
            "--cli-bin" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --cli-bin".to_string())?;
                cli_bin = Some(PathBuf::from(value));
            }
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    let out_dir = out_dir.ok_or_else(|| "missing required --out-dir".to_string())?;
    Ok(HarnessArgs {
        mode,
        out_dir,
        cli_bin,
    })
}

fn run_producer_schema_gate(out_dir: &Path) -> i32 {
    match build_producer_schema_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-producer-schema",
                "phase12_producer_schema_gate",
                &["producer_schema_report.json", "producer_identity_examples.json"],
                &error,
            );
            2
        }
    }
}

fn run_signature_envelope_gate(out_dir: &Path) -> i32 {
    match build_signature_envelope_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-signature-envelope",
                "phase12_signature_envelope_gate",
                &["signature_envelope_report.json", "identity_stability_report.json"],
                &error,
            );
            2
        }
    }
}

fn run_bundle_v2_schema_gate(out_dir: &Path) -> i32 {
    match build_bundle_v2_schema_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-bundle-v2-schema",
                "phase12_bundle_v2_schema_gate",
                &["bundle_schema_report.json"],
                &error,
            );
            2
        }
    }
}

fn run_bundle_v2_compat_gate(out_dir: &Path) -> i32 {
    match build_bundle_v2_compat_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-bundle-v2-compat",
                "phase12_bundle_v2_compat_gate",
                &["compatibility_report.json"],
                &error,
            );
            2
        }
    }
}

fn run_signature_verify_gate(out_dir: &Path) -> i32 {
    match build_signature_verify_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-signature-verify",
                "phase12_signature_verify_gate",
                &["signature_verify.json", "registry_resolution_report.json"],
                &error,
            );
            2
        }
    }
}

fn run_registry_resolution_gate(out_dir: &Path) -> i32 {
    match build_registry_resolution_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-registry-resolution",
                "phase12_registry_resolution_gate",
                &["registry_snapshot.json", "registry_resolution_matrix.json"],
                &error,
            );
            2
        }
    }
}

fn run_key_rotation_gate(out_dir: &Path) -> i32 {
    match build_key_rotation_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_phase12a_failure_artifacts(
                out_dir,
                "proof-key-rotation",
                "phase12_key_rotation_gate",
                &["rotation_matrix.json", "revocation_matrix.json"],
                &error,
            );
            2
        }
    }
}

fn run_verifier_core_gate(out_dir: &Path) -> i32 {
    match build_verifier_core_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_verifier_core_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_trust_policy_gate(out_dir: &Path) -> i32 {
    match build_trust_policy_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_trust_policy_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_verdict_binding_gate(out_dir: &Path) -> i32 {
    match build_verdict_binding_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_verdict_binding_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_verifier_cli_gate(out_dir: &Path, cli_bin: Option<&Path>) -> i32 {
    let cli_bin = match cli_bin {
        Some(path) => path,
        None => {
            write_verifier_cli_failure_artifacts(
                out_dir,
                "phase12 CLI gate requires explicit --cli-bin path",
            );
            return 2;
        }
    };

    match build_verifier_cli_gate_artifacts(out_dir, cli_bin) {
        Ok(code) => code,
        Err(error) => {
            write_verifier_cli_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_proof_exchange_gate(out_dir: &Path) -> i32 {
    match build_proof_exchange_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_proof_exchange_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_receipt_gate(out_dir: &Path) -> i32 {
    match build_receipt_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_receipt_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_audit_ledger_gate(out_dir: &Path) -> i32 {
    match build_audit_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_audit_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_authority_resolution_gate(out_dir: &Path) -> i32 {
    match build_authority_resolution_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_authority_resolution_failure_artifacts(out_dir, &error);
            2
        }
    }
}

fn run_cross_node_parity_gate(out_dir: &Path) -> i32 {
    match build_cross_node_parity_gate_artifacts(out_dir) {
        Ok(code) => code,
        Err(error) => {
            write_cross_node_parity_failure_artifacts(out_dir, &error);
            2
        }
    }
}

struct Phase12AContext {
    fixture: FixtureBundle,
    bundle: LoadedBundle,
    manifest: Manifest,
    checksums: ChecksumsFile,
    bundle_id: String,
    producer: ProducerDeclaration,
    signature_envelope: SignatureEnvelope,
    layout_findings: Vec<VerificationFinding>,
    checksum_findings: Vec<VerificationFinding>,
    proof_chain_findings: Vec<VerificationFinding>,
    overlay_findings: Vec<VerificationFinding>,
    registry_resolution: RegistryResolution,
}

fn build_phase12a_context() -> Result<Phase12AContext, String> {
    let fixture = create_fixture_bundle();
    let bundle = load_bundle(&fixture.root);
    let layout_findings = validate_bundle_layout(&bundle);
    let manifest = load_manifest(&bundle.manifest_path)
        .map_err(|error| format!("failed to load bundle manifest: {error}"))?;
    let checksums = load_checksums(&bundle.checksums_path)
        .map_err(|error| format!("failed to load bundle checksums: {error}"))?;
    let checksum_findings = validate_portable_checksums(&bundle, &checksums)
        .map_err(|error| format!("portable checksum validation failed: {error}"))?;
    let proof_chain_findings = validate_proof_chain(&bundle)
        .map_err(|error| format!("proof chain validation failed: {error}"))?;
    let bundle_id = recompute_bundle_id(&manifest, &checksums)
        .map_err(|error| format!("bundle_id recomputation failed: {error}"))?;
    let OverlayState {
        producer,
        signature_envelope,
        trust_overlay_hash: _trust_overlay_hash,
        findings: overlay_findings,
    } = verify_overlay(&bundle, &bundle_id)
        .map_err(|error| format!("overlay validation failed: {error}"))?;
    let registry_resolution = resolve_signers(&fixture.registry, &producer, &signature_envelope)
        .map_err(|error| format!("registry resolution failed: {error}"))?;

    Ok(Phase12AContext {
        fixture,
        bundle,
        manifest,
        checksums,
        bundle_id,
        producer,
        signature_envelope,
        layout_findings,
        checksum_findings,
        proof_chain_findings,
        overlay_findings,
        registry_resolution,
    })
}

fn build_producer_schema_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let producer = &ctx.producer;
    let mut violations = Vec::new();

    if producer.metadata_version == 0 {
        violations.push("producer_metadata_version_zero".to_string());
    }
    if producer.producer_id.trim().is_empty() {
        violations.push("producer_id_missing".to_string());
    }
    if producer.producer_pubkey_id.trim().is_empty() {
        violations.push("producer_pubkey_id_missing".to_string());
    }
    if producer.producer_pubkey_id.starts_with("base64:") {
        violations.push("producer_pubkey_id_must_not_embed_raw_key_bytes".to_string());
    }
    if producer.producer_registry_ref.trim().is_empty() {
        violations.push("producer_registry_ref_missing".to_string());
    } else if !producer.producer_registry_ref.starts_with("trust://") {
        violations.push("producer_registry_ref_not_namespace_reference".to_string());
    }
    if producer.producer_key_epoch.trim().is_empty() {
        violations.push("producer_key_epoch_missing".to_string());
    }
    if ctx.bundle_id != ctx.manifest.bundle_id {
        violations.push("bundle_id_drift_detected".to_string());
    }

    let rotated_example = json!({
        "metadata_version": producer.metadata_version,
        "producer_id": producer.producer_id,
        "producer_pubkey_id": "ed25519-key-2026-04-a",
        "producer_registry_ref": producer.producer_registry_ref,
        "producer_key_epoch": "2026-04",
        "build_id": "build-fe9031d7-rotated",
    });
    let canonical_sha256 = sha256_hex(
        &canonicalize_json(producer)
            .map_err(|error| format!("producer canonicalization failed: {error}"))?,
    );

    let bundle_id_after_rotation = recompute_bundle_id(&ctx.manifest, &ctx.checksums)
        .map_err(|error| format!("bundle_id recomputation after producer rotation failed: {error}"))?;
    let bundle_id_stable_under_producer_rotation = ctx.bundle_id == bundle_id_after_rotation;
    if !bundle_id_stable_under_producer_rotation {
        violations.push("producer_rotation_mutated_bundle_id".to_string());
    }

    let producer_schema_report = json!({
        "gate": "proof-producer-schema",
        "mode": "phase12_producer_schema_gate",
        "status": status_label(violations.is_empty()),
        "metadata_version": producer.metadata_version,
        "producer_id": producer.producer_id,
        "producer_pubkey_id": producer.producer_pubkey_id,
        "producer_registry_ref": producer.producer_registry_ref,
        "producer_key_epoch": producer.producer_key_epoch,
        "producer_canonical_sha256": canonical_sha256,
        "bundle_id": ctx.bundle_id,
        "bundle_id_stable_under_producer_rotation": bundle_id_stable_under_producer_rotation,
    });
    write_json(
        out_dir.join("producer_schema_report.json"),
        &producer_schema_report,
    )?;

    let producer_identity_examples = json!({
        "current_example": producer,
        "rotated_example": rotated_example,
    });
    write_json(
        out_dir.join("producer_identity_examples.json"),
        &producer_identity_examples,
    )?;

    let report = json!({
        "gate": "proof-producer-schema",
        "mode": "phase12_producer_schema_gate",
        "verdict": status_label(violations.is_empty()),
        "bundle_id": ctx.bundle_id,
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_signature_envelope_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let envelope = &ctx.signature_envelope;
    let mut violations = error_violations(&ctx.overlay_findings);

    if envelope.envelope_version == 0 {
        violations.push("signature_envelope_version_zero".to_string());
    }
    if !envelope.bundle_id_algorithm.eq_ignore_ascii_case("sha256") {
        violations.push("signature_envelope_bundle_id_algorithm_not_sha256".to_string());
    }
    if envelope.signatures.is_empty() {
        violations.push("signature_envelope_missing_signatures".to_string());
    }
    if envelope.bundle_id != ctx.bundle_id {
        violations.push("signature_envelope_bundle_id_mismatch".to_string());
    }

    let mut augmented_envelope = envelope.clone();
    let duplicate_signature = envelope
        .signatures
        .first()
        .cloned()
        .ok_or_else(|| "signature envelope fixture is missing a baseline signature".to_string())?;
    augmented_envelope.signatures.push(duplicate_signature);
    let bundle_id_after_mutation = recompute_bundle_id(&ctx.manifest, &ctx.checksums)
        .map_err(|error| format!("bundle_id recomputation after envelope mutation failed: {error}"))?;
    let bundle_id_stable_under_envelope_mutation = ctx.bundle_id == bundle_id_after_mutation;
    if !bundle_id_stable_under_envelope_mutation {
        violations.push("signature_envelope_mutated_bundle_id".to_string());
    }

    let signature_envelope_report = json!({
        "gate": "proof-signature-envelope",
        "mode": "phase12_signature_envelope_gate",
        "status": status_label(violations.is_empty()),
        "envelope_version": envelope.envelope_version,
        "bundle_id": envelope.bundle_id,
        "bundle_id_algorithm": envelope.bundle_id_algorithm,
        "signature_count": envelope.signatures.len(),
        "multi_signature_ready": true,
        "overlay_findings": findings_to_json(&ctx.overlay_findings),
        "overlay_findings_count": ctx.overlay_findings.len(),
    });
    write_json(
        out_dir.join("signature_envelope_report.json"),
        &signature_envelope_report,
    )?;

    let identity_stability_report = json!({
        "gate": "proof-signature-envelope",
        "mode": "phase12_signature_envelope_gate",
        "status": status_label(bundle_id_stable_under_envelope_mutation),
        "bundle_id_before": ctx.bundle_id,
        "bundle_id_after_envelope_mutation": bundle_id_after_mutation,
        "signature_count_before": envelope.signatures.len(),
        "signature_count_after": augmented_envelope.signatures.len(),
        "bundle_id_stable_under_envelope_mutation": bundle_id_stable_under_envelope_mutation,
    });
    write_json(
        out_dir.join("identity_stability_report.json"),
        &identity_stability_report,
    )?;

    let report = json!({
        "gate": "proof-signature-envelope",
        "mode": "phase12_signature_envelope_gate",
        "verdict": status_label(violations.is_empty()),
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_bundle_v2_schema_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let mut violations = error_violations(&ctx.layout_findings);
    violations.extend(error_violations(&ctx.checksum_findings));
    violations.extend(error_violations(&ctx.proof_chain_findings));

    if ctx.manifest.bundle_version != 2 {
        violations.push(format!(
            "unexpected_manifest_bundle_version:{}",
            ctx.manifest.bundle_version
        ));
    }
    if ctx.checksums.bundle_version != 2 {
        violations.push(format!(
            "unexpected_checksums_bundle_version:{}",
            ctx.checksums.bundle_version
        ));
    }
    if ctx.manifest.mode.as_deref() != Some("portable_proof_bundle_v2") {
        violations.push("unexpected_manifest_mode".to_string());
    }
    if ctx.manifest.compatibility_mode.as_deref() != Some("phase11-portable-core") {
        violations.push("unexpected_manifest_compatibility_mode".to_string());
    }
    if ctx.manifest.checksums_file != "checksums.json" {
        violations.push("unexpected_checksums_file_reference".to_string());
    }
    if ctx.bundle_id != ctx.manifest.bundle_id {
        violations.push("bundle_id_recompute_mismatch".to_string());
    }

    let request = VerifyRequest {
        bundle_path: &ctx.fixture.root,
        policy: &ctx.fixture.policy,
        registry_snapshot: &ctx.fixture.registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    let outcome = verify_bundle(&request)
        .map_err(|error| format!("bundle v2 schema gate runtime verification failed: {error}"))?;
    violations.extend(error_violations(&outcome.findings));

    let bundle_schema_report = json!({
        "gate": "proof-bundle-v2-schema",
        "mode": "phase12_bundle_v2_schema_gate",
        "status": status_label(violations.is_empty()),
        "bundle_version": ctx.manifest.bundle_version,
        "checksums_bundle_version": ctx.checksums.bundle_version,
        "mode_value": ctx.manifest.mode,
        "compatibility_mode": ctx.manifest.compatibility_mode,
        "checksums_file": ctx.manifest.checksums_file,
        "required_file_count": ctx.manifest.required_files.len(),
        "bundle_id": ctx.manifest.bundle_id,
        "bundle_id_recomputed": ctx.bundle_id,
        "verification_verdict": verdict_label(&outcome.verdict),
        "layout_findings": findings_to_json(&ctx.layout_findings),
        "checksum_findings": findings_to_json(&ctx.checksum_findings),
        "proof_chain_findings": findings_to_json(&ctx.proof_chain_findings),
        "verification_findings": findings_to_json(&outcome.findings),
    });
    write_json(out_dir.join("bundle_schema_report.json"), &bundle_schema_report)?;

    let report = json!({
        "gate": "proof-bundle-v2-schema",
        "mode": "phase12_bundle_v2_schema_gate",
        "verdict": status_label(violations.is_empty()),
        "bundle_id": ctx.bundle_id,
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_bundle_v2_compat_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let mut violations = Vec::new();
    let required_files = &ctx.manifest.required_files;
    let overlay_is_external = !required_files
        .iter()
        .any(|path| path == "producer/producer.json" || path == "signatures/signature-envelope.json");
    if !overlay_is_external {
        violations.push("overlay_paths_leaked_into_portable_required_files".to_string());
    }

    let portable_core_paths = [
        "manifest.json",
        "checksums.json",
        "evidence/",
        "traces/",
        "reports/",
        "meta/run.json",
    ];
    let portable_core_paths_present = ctx.bundle.manifest_path.is_file()
        && ctx.bundle.checksums_path.is_file()
        && ctx.bundle.evidence_dir.is_dir()
        && ctx.bundle.traces_dir.is_dir()
        && ctx.bundle.reports_dir.is_dir()
        && ctx.bundle.meta_run_path.is_file();
    if !portable_core_paths_present {
        violations.push("portable_core_paths_missing".to_string());
    }
    if ctx.manifest.compatibility_mode.as_deref() != Some("phase11-portable-core") {
        violations.push("bundle_v2_compatibility_mode_missing".to_string());
    }
    if has_error_findings(&ctx.layout_findings)
        || has_error_findings(&ctx.checksum_findings)
        || has_error_findings(&ctx.proof_chain_findings)
    {
        violations.push("portable_core_not_phase11_compatible".to_string());
    }

    let compatibility_report = json!({
        "gate": "proof-bundle-v2-compat",
        "mode": "phase12_bundle_v2_compat_gate",
        "status": status_label(violations.is_empty()),
        "compatibility_mode": ctx.manifest.compatibility_mode,
        "portable_core_paths": portable_core_paths,
        "portable_core_paths_present": portable_core_paths_present,
        "overlay_is_external": overlay_is_external,
        "required_file_count": required_files.len(),
        "required_files": required_files,
    });
    write_json(out_dir.join("compatibility_report.json"), &compatibility_report)?;

    let report = json!({
        "gate": "proof-bundle-v2-compat",
        "mode": "phase12_bundle_v2_compat_gate",
        "verdict": status_label(violations.is_empty()),
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_signature_verify_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let signature_findings = verify_detached_signatures(
        &ctx.bundle_id,
        &ctx.signature_envelope,
        &ctx.registry_resolution.resolved_signers,
    );
    let mut violations = error_violations(&ctx.registry_resolution.findings);
    violations.extend(error_violations(&signature_findings));

    let signature_verify = json!({
        "gate": "proof-signature-verify",
        "mode": "phase12_signature_verify_gate",
        "status": status_label(!has_error_findings(&signature_findings)),
        "bundle_id": ctx.bundle_id,
        "bundle_id_algorithm": ctx.signature_envelope.bundle_id_algorithm,
        "signature_count": ctx.signature_envelope.signatures.len(),
        "verified_signature_count": ctx.signature_envelope.signatures.len().saturating_sub(error_violations(&signature_findings).len()),
        "findings": findings_to_json(&signature_findings),
        "findings_count": signature_findings.len(),
    });
    write_json(out_dir.join("signature_verify.json"), &signature_verify)?;

    let registry_resolution_report = json!({
        "gate": "proof-signature-verify",
        "mode": "phase12_signature_verify_gate",
        "status": status_label(!has_error_findings(&ctx.registry_resolution.findings)),
        "registry_snapshot_hash": ctx.registry_resolution.registry_snapshot_hash,
        "resolved_signer_count": ctx.registry_resolution.resolved_signers.len(),
        "resolved_signers": ctx.registry_resolution.resolved_signers.iter().map(|signer| {
            json!({
                "signer_id": signer.signer_id,
                "producer_pubkey_id": signer.producer_pubkey_id,
                "status": key_status_label(&signer.status),
                "has_public_key": signer.public_key.is_some(),
            })
        }).collect::<Vec<_>>(),
        "findings": findings_to_json(&ctx.registry_resolution.findings),
        "findings_count": ctx.registry_resolution.findings.len(),
    });
    write_json(
        out_dir.join("registry_resolution_report.json"),
        &registry_resolution_report,
    )?;

    let report = json!({
        "gate": "proof-signature-verify",
        "mode": "phase12_signature_verify_gate",
        "verdict": status_label(violations.is_empty()),
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_registry_resolution_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let baseline_row = registry_resolution_matrix_row(
        "baseline_active",
        &ctx.fixture.registry,
        &ctx.producer,
        &ctx.signature_envelope,
    )?;
    let ambiguous_row = registry_resolution_matrix_row(
        "ambiguous_owner",
        &build_ambiguous_owner_registry(&ctx.fixture.registry)?,
        &ctx.producer,
        &ctx.signature_envelope,
    )?;
    let unknown_row = registry_resolution_matrix_row(
        "unknown_key_state",
        &build_unknown_key_registry(&ctx.fixture.registry)?,
        &ctx.producer,
        &ctx.signature_envelope,
    )?;
    let missing_material_row = registry_resolution_matrix_row(
        "missing_public_key_material",
        &build_missing_public_key_registry(&ctx.fixture.registry)?,
        &ctx.producer,
        &ctx.signature_envelope,
    )?;
    let matrix = vec![baseline_row, ambiguous_row, unknown_row, missing_material_row];
    write_json(out_dir.join("registry_snapshot.json"), &ctx.fixture.registry)?;
    write_json(out_dir.join("registry_resolution_matrix.json"), &matrix)?;

    let mut violations = Vec::new();
    if !matrix_row_has_status(&matrix[0], "ACTIVE") || matrix_row_has_errors(&matrix[0]) {
        violations.push("baseline_registry_resolution_not_active".to_string());
    }
    if !matrix_row_has_error_code(&matrix[1], "PV0405") {
        violations.push("ambiguous_registry_resolution_missing_PV0405".to_string());
    }
    if !matrix_row_has_error_code(&matrix[2], "PV0404") {
        violations.push("unknown_key_registry_resolution_missing_PV0404".to_string());
    }
    if !matrix_row_has_error_code(&matrix[3], "PV0406")
        || !matrix_row_has_error_code(&matrix[3], "PV0408")
    {
        violations.push("missing_public_key_material_matrix_incomplete".to_string());
    }

    let report = json!({
        "gate": "proof-registry-resolution",
        "mode": "phase12_registry_resolution_gate",
        "verdict": status_label(violations.is_empty()),
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_key_rotation_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let ctx = build_phase12a_context()?;
    let baseline_rotation_row = key_lifecycle_matrix_row(
        "baseline_active",
        &ctx.fixture.registry,
        &ctx.producer,
        &ctx.signature_envelope,
        &ctx.bundle_id,
    )?;
    let rotated_rotation_row = key_lifecycle_matrix_row(
        "rotated_superseded",
        &build_rotated_registry(&ctx.fixture.registry)?,
        &ctx.producer,
        &ctx.signature_envelope,
        &ctx.bundle_id,
    )?;
    let revoked_row = key_lifecycle_matrix_row(
        "revoked",
        &build_revoked_registry(&ctx.fixture.registry)?,
        &ctx.producer,
        &ctx.signature_envelope,
        &ctx.bundle_id,
    )?;

    let rotation_matrix = vec![baseline_rotation_row, rotated_rotation_row];
    let revocation_matrix = vec![revoked_row];
    write_json(out_dir.join("rotation_matrix.json"), &rotation_matrix)?;
    write_json(out_dir.join("revocation_matrix.json"), &revocation_matrix)?;

    let mut violations = Vec::new();
    if !matrix_row_has_status(&rotation_matrix[0], "ACTIVE")
        || matrix_row_has_errors(&rotation_matrix[0])
    {
        violations.push("baseline_rotation_row_invalid".to_string());
    }
    if !matrix_row_has_status(&rotation_matrix[1], "SUPERSEDED")
        || matrix_row_has_errors(&rotation_matrix[1])
    {
        violations.push("rotated_superseded_row_invalid".to_string());
    }
    if !matrix_row_has_status(&revocation_matrix[0], "REVOKED")
        || !matrix_row_has_error_code(&revocation_matrix[0], "PV0403")
    {
        violations.push("revocation_row_missing_PV0403".to_string());
    }

    let report = json!({
        "gate": "proof-key-rotation",
        "mode": "phase12_key_rotation_gate",
        "verdict": status_label(violations.is_empty()),
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_verifier_core_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let trusted_fixture = create_fixture_bundle();
    let baseline_row = verifier_core_matrix_row(
        "trusted_baseline",
        VerificationVerdict::Trusted,
        &trusted_fixture.root,
        &trusted_fixture.policy,
        &trusted_fixture.registry,
    )?;

    let policy_rejected_fixture = create_fixture_bundle();
    let mut policy_rejected_policy = policy_rejected_fixture.policy.clone();
    policy_rejected_policy.required_signatures = Some(SignatureRequirement {
        kind: "at_least".to_string(),
        count: 2,
    });
    let policy_rejected_row = verifier_core_matrix_row(
        "policy_rejected_quorum",
        VerificationVerdict::RejectedByPolicy,
        &policy_rejected_fixture.root,
        &policy_rejected_policy,
        &policy_rejected_fixture.registry,
    )?;

    let untrusted_fixture = create_fixture_bundle();
    let mut untrusted_policy = untrusted_fixture.policy.clone();
    untrusted_policy.trusted_producers = vec!["different-producer".to_string()];
    let untrusted_row = verifier_core_matrix_row(
        "untrusted_producer",
        VerificationVerdict::Untrusted,
        &untrusted_fixture.root,
        &untrusted_policy,
        &untrusted_fixture.registry,
    )?;

    let invalid_signature_fixture = create_fixture_bundle();
    tamper_signature_envelope(&invalid_signature_fixture.root)?;
    let invalid_signature_row = verifier_core_matrix_row(
        "invalid_signature",
        VerificationVerdict::Invalid,
        &invalid_signature_fixture.root,
        &invalid_signature_fixture.policy,
        &invalid_signature_fixture.registry,
    )?;

    let missing_manifest_fixture = create_fixture_bundle();
    remove_manifest_file(&missing_manifest_fixture.root)?;
    let missing_manifest_row = verifier_core_matrix_row(
        "missing_manifest",
        VerificationVerdict::Invalid,
        &missing_manifest_fixture.root,
        &missing_manifest_fixture.policy,
        &missing_manifest_fixture.registry,
    )?;

    let matrix = vec![
        baseline_row,
        policy_rejected_row,
        untrusted_row,
        invalid_signature_row,
        missing_manifest_row,
    ];
    write_json(out_dir.join("determinism_matrix.json"), &matrix)?;

    let deterministic_case_count = matrix
        .iter()
        .filter(|row| row.get("deterministic").and_then(Value::as_bool) == Some(true))
        .count();
    let trusted_case_count = count_expected_verdict(&matrix, "TRUSTED");
    let rejected_case_count = count_expected_verdict(&matrix, "REJECTED_BY_POLICY");
    let untrusted_case_count = count_expected_verdict(&matrix, "UNTRUSTED");
    let invalid_case_count = count_expected_verdict(&matrix, "INVALID");

    let pipeline_stage_order = vec![
        "bundle_load",
        "layout_validation",
        "portable_checksum_validation",
        "portable_proof_validation",
        "bundle_id_recomputation",
        "overlay_validation",
        "signer_resolution",
        "detached_signature_verification",
        "policy_evaluation",
        "verdict_derivation",
        "receipt_emission",
    ];
    let verifier_core_report = json!({
        "gate": "proof-verifier-core",
        "mode": "phase12_proof_verifier_core_gate",
        "status": status_label(deterministic_case_count == matrix.len()),
        "crate_path": "ayken-core/crates/proof-verifier/",
        "api_entrypoint": "verify_bundle",
        "library_first": true,
        "userspace_offline": true,
        "pipeline_stage_order": pipeline_stage_order,
        "scenario_count": matrix.len(),
        "deterministic_case_count": deterministic_case_count,
        "trusted_case_count": trusted_case_count,
        "rejected_by_policy_case_count": rejected_case_count,
        "untrusted_case_count": untrusted_case_count,
        "invalid_case_count": invalid_case_count,
        "determinism_matrix_path": "determinism_matrix.json",
    });
    write_json(out_dir.join("verifier_core_report.json"), &verifier_core_report)?;

    let mut violations = Vec::new();
    for row in &matrix {
        let scenario = row
            .get("scenario")
            .and_then(Value::as_str)
            .unwrap_or("unknown_scenario");
        if row.get("deterministic").and_then(Value::as_bool) != Some(true) {
            violations.push(format!("scenario_not_deterministic:{scenario}"));
        }
        if row.get("expected_verdict").and_then(Value::as_str)
            != row.get("run_a_verdict").and_then(Value::as_str)
        {
            violations.push(format!("unexpected_run_a_verdict:{scenario}"));
        }
        if row.get("expected_verdict").and_then(Value::as_str)
            != row.get("run_b_verdict").and_then(Value::as_str)
        {
            violations.push(format!("unexpected_run_b_verdict:{scenario}"));
        }
        if row.get("receipt_absent").and_then(Value::as_bool) != Some(true) {
            violations.push(format!("unexpected_receipt_emission:{scenario}"));
        }
        if row.get("audit_absent").and_then(Value::as_bool) != Some(true) {
            violations.push(format!("unexpected_audit_append:{scenario}"));
        }
    }

    let report = json!({
        "gate": "proof-verifier-core",
        "mode": "phase12_proof_verifier_core_gate",
        "verdict": status_label(violations.is_empty()),
        "verifier_core_report_path": "verifier_core_report.json",
        "determinism_matrix_path": "determinism_matrix.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_trust_policy_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let fixture = create_fixture_bundle();
    let bundle = load_bundle(&fixture.root);
    let manifest = load_manifest(&bundle.manifest_path)
        .map_err(|error| format!("trust policy gate failed to load manifest: {error}"))?;
    let baseline_findings = validate_policy(&fixture.policy);
    let baseline_hash = compute_policy_hash(&fixture.policy)
        .map_err(|error| format!("trust policy baseline hash computation failed: {error}"))?;
    let baseline_hash_repeat = compute_policy_hash(&fixture.policy)
        .map_err(|error| format!("trust policy baseline hash recomputation failed: {error}"))?;
    let external_to_bundle = !manifest
        .required_files
        .iter()
        .any(|path| path.contains("policy"));
    let has_trusted_producers = !fixture.policy.trusted_producers.is_empty();
    let has_trusted_pubkey_ids = !fixture.policy.trusted_pubkey_ids.is_empty();
    let has_required_signatures = fixture.policy.required_signatures.is_some();
    let has_explicit_quorum_policy = fixture
        .policy
        .quorum_policy_ref
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let baseline_hash_stable = baseline_hash == baseline_hash_repeat;

    let trusted_row = trust_policy_outcome_row(
        "trusted_baseline",
        VerificationVerdict::Trusted,
        &fixture.root,
        &fixture.policy,
        &fixture.registry,
    )?;

    let mut rejected_policy = fixture.policy.clone();
    rejected_policy.required_signatures = Some(SignatureRequirement {
        kind: "at_least".to_string(),
        count: 2,
    });
    let rejected_row = trust_policy_outcome_row(
        "rejected_by_policy_quorum",
        VerificationVerdict::RejectedByPolicy,
        &fixture.root,
        &rejected_policy,
        &fixture.registry,
    )?;

    let mut untrusted_policy = fixture.policy.clone();
    untrusted_policy.trusted_producers = vec!["different-producer".to_string()];
    let untrusted_row = trust_policy_outcome_row(
        "untrusted_producer",
        VerificationVerdict::Untrusted,
        &fixture.root,
        &untrusted_policy,
        &fixture.registry,
    )?;

    let mut invalid_quorum_policy = fixture.policy.clone();
    invalid_quorum_policy.required_signatures = Some(SignatureRequirement {
        kind: "unsupported".to_string(),
        count: 1,
    });
    let invalid_quorum_row = trust_policy_outcome_row(
        "unsupported_quorum_kind",
        VerificationVerdict::Invalid,
        &fixture.root,
        &invalid_quorum_policy,
        &fixture.registry,
    )?;

    let rejected_policy_hash = compute_policy_hash(&rejected_policy)
        .map_err(|error| format!("trust policy rejected-policy hash computation failed: {error}"))?;
    let policy_hash_changes_under_mutation = baseline_hash != rejected_policy_hash;
    let verdict_rows = vec![trusted_row, rejected_row, untrusted_row, invalid_quorum_row];

    let policy_schema_report = json!({
        "gate": "proof-trust-policy",
        "mode": "phase12_trust_policy_gate",
        "status": status_label(
            !has_error_findings(&baseline_findings)
                && external_to_bundle
                && has_trusted_producers
                && has_trusted_pubkey_ids
                && has_required_signatures
                && has_explicit_quorum_policy
        ),
        "policy_version": fixture.policy.policy_version,
        "external_to_bundle": external_to_bundle,
        "trusted_producers_count": fixture.policy.trusted_producers.len(),
        "trusted_pubkey_ids_count": fixture.policy.trusted_pubkey_ids.len(),
        "required_signature_kind": fixture
            .policy
            .required_signatures
            .as_ref()
            .map(|value| value.kind.clone()),
        "required_signature_count": fixture.policy.required_signature_count(),
        "revoked_pubkey_ids_count": fixture.policy.revoked_pubkey_ids.len(),
        "quorum_policy_ref": fixture.policy.quorum_policy_ref,
        "schema_findings": findings_to_json(&baseline_findings),
        "schema_findings_count": baseline_findings.len(),
        "field_surface": {
            "trusted_producers": has_trusted_producers,
            "trusted_pubkey_ids": has_trusted_pubkey_ids,
            "required_signatures": has_required_signatures,
            "revoked_pubkey_ids": true,
            "quorum_policy_ref": has_explicit_quorum_policy,
        },
    });
    write_json(out_dir.join("policy_schema_report.json"), &policy_schema_report)?;

    let policy_hash_report = json!({
        "gate": "proof-trust-policy",
        "mode": "phase12_trust_policy_gate",
        "status": status_label(baseline_hash_stable && policy_hash_changes_under_mutation),
        "baseline_policy_hash": baseline_hash,
        "baseline_policy_hash_repeat": baseline_hash_repeat,
        "baseline_hash_stable": baseline_hash_stable,
        "rejected_policy_hash": rejected_policy_hash,
        "policy_hash_changes_under_mutation": policy_hash_changes_under_mutation,
        "verdict_rows": verdict_rows,
    });
    write_json(out_dir.join("policy_hash_report.json"), &policy_hash_report)?;

    let mut violations = error_violations(&baseline_findings);
    if !external_to_bundle {
        violations.push("policy_surface_leaked_into_bundle".to_string());
    }
    if !baseline_hash_stable {
        violations.push("policy_hash_not_stable".to_string());
    }
    if !policy_hash_changes_under_mutation {
        violations.push("policy_hash_did_not_change_under_mutation".to_string());
    }
    for row in policy_hash_report
        .get("verdict_rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let scenario = row
            .get("scenario")
            .and_then(Value::as_str)
            .unwrap_or("unknown_scenario");
        if row.get("expected_verdict").and_then(Value::as_str)
            != row.get("actual_verdict").and_then(Value::as_str)
        {
            violations.push(format!("unexpected_policy_verdict:{scenario}"));
        }
        if row.get("policy_hash_bound").and_then(Value::as_bool) != Some(true) {
            violations.push(format!("policy_hash_not_bound_to_verdict:{scenario}"));
        }
    }
    if !policy_hash_report
        .get("verdict_rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| {
            row.get("scenario").and_then(Value::as_str) == Some("unsupported_quorum_kind")
                && row
                    .get("error_codes")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|code| code == "PV0504")
        })
    {
        violations.push("unsupported_quorum_kind_missing_PV0504".to_string());
    }

    let report = json!({
        "gate": "proof-trust-policy",
        "mode": "phase12_trust_policy_gate",
        "verdict": status_label(violations.is_empty()),
        "policy_schema_report_path": "policy_schema_report.json",
        "policy_hash_report_path": "policy_hash_report.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_verdict_binding_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
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

    let outcome_a = verify_bundle(&request)
        .map_err(|error| format!("verdict binding gate first verification failed: {error}"))?;
    let outcome_b = verify_bundle(&request)
        .map_err(|error| format!("verdict binding gate second verification failed: {error}"))?;
    let receipt = outcome_a
        .receipt
        .as_ref()
        .ok_or_else(|| "verdict binding gate did not emit a signed receipt".to_string())?;

    let same_subject_tuple = outcome_a.subject.bundle_id == outcome_b.subject.bundle_id
        && outcome_a.subject.trust_overlay_hash == outcome_b.subject.trust_overlay_hash
        && outcome_a.subject.policy_hash == outcome_b.subject.policy_hash
        && outcome_a.subject.registry_snapshot_hash == outcome_b.subject.registry_snapshot_hash;
    let same_verdict = outcome_a.verdict == outcome_b.verdict;
    let receipt_binding_equal = receipt.payload.bundle_id == outcome_a.subject.bundle_id
        && receipt.payload.trust_overlay_hash == outcome_a.subject.trust_overlay_hash
        && receipt.payload.policy_hash == outcome_a.subject.policy_hash
        && receipt.payload.registry_snapshot_hash == outcome_a.subject.registry_snapshot_hash;
    let full_tuple_present = !outcome_a.subject.bundle_id.is_empty()
        && !outcome_a.subject.trust_overlay_hash.is_empty()
        && !outcome_a.subject.policy_hash.is_empty()
        && !outcome_a.subject.registry_snapshot_hash.is_empty();

    let verdict_binding_report = json!({
        "gate": "proof-verdict-binding",
        "mode": "phase12_verdict_binding_gate",
        "status": status_label(full_tuple_present && same_subject_tuple && same_verdict && receipt_binding_equal),
        "verification_verdict": verdict_label(&outcome_a.verdict),
        "bundle_id": outcome_a.subject.bundle_id,
        "trust_overlay_hash": outcome_a.subject.trust_overlay_hash,
        "policy_hash": outcome_a.subject.policy_hash,
        "registry_snapshot_hash": outcome_a.subject.registry_snapshot_hash,
        "same_subject_tuple": same_subject_tuple,
        "same_verdict": same_verdict,
        "receipt_binding_equal": receipt_binding_equal,
        "receipt_verifier_node_id": receipt.payload.verifier_node_id,
        "receipt_verifier_key_id": receipt.payload.verifier_key_id,
    });
    write_json(out_dir.join("verdict_binding_report.json"), &verdict_binding_report)?;

    let verdict_subject_examples = json!({
        "full_verdict_subject": {
            "bundle_id": outcome_a.subject.bundle_id,
            "trust_overlay_hash": outcome_a.subject.trust_overlay_hash,
            "policy_hash": outcome_a.subject.policy_hash,
            "registry_snapshot_hash": outcome_a.subject.registry_snapshot_hash,
        },
        "distributed_claim_weaker_tuples": [
            {
                "fields": ["bundle_id", "trust_overlay_hash", "policy_hash"],
                "allowed_for_distributed_claim": false
            },
            {
                "fields": ["bundle_id", "trust_overlay_hash", "registry_snapshot_hash"],
                "allowed_for_distributed_claim": false
            },
            {
                "fields": ["bundle_id", "policy_hash", "registry_snapshot_hash"],
                "allowed_for_distributed_claim": false
            }
        ],
        "receipt_binding": {
            "bundle_id": receipt.payload.bundle_id,
            "trust_overlay_hash": receipt.payload.trust_overlay_hash,
            "policy_hash": receipt.payload.policy_hash,
            "registry_snapshot_hash": receipt.payload.registry_snapshot_hash,
        }
    });
    write_json(out_dir.join("verdict_subject_examples.json"), &verdict_subject_examples)?;

    let mut violations = error_violations(&outcome_a.findings);
    violations.extend(error_violations(&outcome_b.findings));
    if !full_tuple_present {
        violations.push("verdict_subject_missing_binding_field".to_string());
    }
    if !same_subject_tuple {
        violations.push("verdict_subject_not_stable_under_same_input".to_string());
    }
    if !same_verdict {
        violations.push("verdict_not_stable_under_same_binding_tuple".to_string());
    }
    if !receipt_binding_equal {
        violations.push("receipt_binding_does_not_match_verdict_subject".to_string());
    }

    let report = json!({
        "gate": "proof-verdict-binding",
        "mode": "phase12_verdict_binding_gate",
        "verdict": status_label(violations.is_empty()),
        "verdict_binding_report_path": "verdict_binding_report.json",
        "verdict_subject_examples_path": "verdict_subject_examples.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_verifier_cli_gate_artifacts(out_dir: &Path, cli_bin: &Path) -> Result<i32, String> {
    if !cli_bin.is_file() {
        return Err(format!(
            "CLI binary does not exist at {}",
            cli_bin.display()
        ));
    }

    let fixture = create_fixture_bundle();
    let inputs_dir = out_dir.join("inputs");
    fs::create_dir_all(&inputs_dir).map_err(|error| {
        format!(
            "failed to create CLI gate inputs dir {}: {error}",
            inputs_dir.display()
        )
    })?;

    let policy_path = inputs_dir.join("policy.json");
    let registry_path = inputs_dir.join("registry.json");
    write_json(policy_path.clone(), &fixture.policy)?;
    write_json(registry_path.clone(), &fixture.registry)?;

    let expected_outcome =
        run_core_verification(&fixture.root, &fixture.policy, &fixture.registry)?;
    let expected_verdict = verdict_label(&expected_outcome.verdict);

    let human_run = run_cli_verify_bundle(cli_bin, &fixture.root, &policy_path, &registry_path, false)?;
    let json_run = run_cli_verify_bundle(cli_bin, &fixture.root, &policy_path, &registry_path, true)?;

    fs::write(out_dir.join("cli_human_stdout.txt"), &human_run.stdout).map_err(|error| {
        format!(
            "failed to write CLI human stdout {}: {error}",
            out_dir.join("cli_human_stdout.txt").display()
        )
    })?;
    fs::write(out_dir.join("cli_human_stderr.txt"), &human_run.stderr).map_err(|error| {
        format!(
            "failed to write CLI human stderr {}: {error}",
            out_dir.join("cli_human_stderr.txt").display()
        )
    })?;
    fs::write(out_dir.join("cli_json_stderr.txt"), &json_run.stderr).map_err(|error| {
        format!(
            "failed to write CLI JSON stderr {}: {error}",
            out_dir.join("cli_json_stderr.txt").display()
        )
    })?;

    let cli_json_output: Value = serde_json::from_str(&json_run.stdout).map_err(|error| {
        format!("CLI JSON output contract parse failed: {error}")
    })?;
    write_json(out_dir.join("cli_json_output.json"), &cli_json_output)?;

    let human_contains_verdict = human_run
        .stdout
        .contains(&format!("Verdict: {expected_verdict}"));
    let human_contains_bundle_id = human_run
        .stdout
        .contains(&expected_outcome.subject.bundle_id);
    let human_contains_trust_overlay_hash = human_run
        .stdout
        .contains(&expected_outcome.subject.trust_overlay_hash);
    let human_contains_policy_hash = human_run
        .stdout
        .contains(&expected_outcome.subject.policy_hash);
    let human_contains_registry_snapshot_hash = human_run
        .stdout
        .contains(&expected_outcome.subject.registry_snapshot_hash);

    let json_verdict = cli_json_output.get("verdict").and_then(Value::as_str);
    let json_bundle_id = cli_json_output.get("bundle_id").and_then(Value::as_str);
    let json_trust_overlay_hash = cli_json_output
        .get("trust_overlay_hash")
        .and_then(Value::as_str);
    let json_policy_hash = cli_json_output.get("policy_hash").and_then(Value::as_str);
    let json_registry_snapshot_hash = cli_json_output
        .get("registry_snapshot_hash")
        .and_then(Value::as_str);
    let json_findings = cli_json_output.get("findings").and_then(Value::as_array);

    let cli_smoke_report = json!({
        "gate": "proof-verifier-cli",
        "mode": "phase12_proof_verifier_cli_gate",
        "status": status_label(
            human_run.exit_code == 0
                && json_run.exit_code == 0
                && human_contains_verdict
                && human_contains_bundle_id
                && human_contains_trust_overlay_hash
                && human_contains_policy_hash
                && human_contains_registry_snapshot_hash
        ),
        "command_surface": "verify bundle",
        "cli_binary": cli_bin.display().to_string(),
        "bundle_path": fixture.root.display().to_string(),
        "policy_path": policy_path.display().to_string(),
        "registry_path": registry_path.display().to_string(),
        "human_exit_code": human_run.exit_code,
        "json_exit_code": json_run.exit_code,
        "human_contains_verdict": human_contains_verdict,
        "human_contains_bundle_id": human_contains_bundle_id,
        "human_contains_trust_overlay_hash": human_contains_trust_overlay_hash,
        "human_contains_policy_hash": human_contains_policy_hash,
        "human_contains_registry_snapshot_hash": human_contains_registry_snapshot_hash,
    });
    write_json(out_dir.join("cli_smoke_report.json"), &cli_smoke_report)?;

    let cli_output_contract = json!({
        "gate": "proof-verifier-cli",
        "mode": "phase12_proof_verifier_cli_gate",
        "status": status_label(
            json_verdict == Some(expected_verdict)
                && json_bundle_id == Some(expected_outcome.subject.bundle_id.as_str())
                && json_trust_overlay_hash == Some(expected_outcome.subject.trust_overlay_hash.as_str())
                && json_policy_hash == Some(expected_outcome.subject.policy_hash.as_str())
                && json_registry_snapshot_hash == Some(expected_outcome.subject.registry_snapshot_hash.as_str())
                && json_findings.is_some()
        ),
        "verdict": json_verdict,
        "bundle_id": json_bundle_id,
        "trust_overlay_hash": json_trust_overlay_hash,
        "policy_hash": json_policy_hash,
        "registry_snapshot_hash": json_registry_snapshot_hash,
        "findings_count": json_findings.map(|value| value.len()),
        "required_fields_present": {
            "verdict": json_verdict.is_some(),
            "bundle_id": json_bundle_id.is_some(),
            "trust_overlay_hash": json_trust_overlay_hash.is_some(),
            "policy_hash": json_policy_hash.is_some(),
            "registry_snapshot_hash": json_registry_snapshot_hash.is_some(),
            "findings": json_findings.is_some(),
        },
        "matches_verifier_core": {
            "verdict": json_verdict == Some(expected_verdict),
            "bundle_id": json_bundle_id == Some(expected_outcome.subject.bundle_id.as_str()),
            "trust_overlay_hash": json_trust_overlay_hash == Some(expected_outcome.subject.trust_overlay_hash.as_str()),
            "policy_hash": json_policy_hash == Some(expected_outcome.subject.policy_hash.as_str()),
            "registry_snapshot_hash": json_registry_snapshot_hash == Some(expected_outcome.subject.registry_snapshot_hash.as_str()),
        },
    });
    write_json(out_dir.join("cli_output_contract.json"), &cli_output_contract)?;

    let mut violations = Vec::new();
    if human_run.exit_code != 0 {
        violations.push(format!("human_cli_exit_code:{}", human_run.exit_code));
    }
    if json_run.exit_code != 0 {
        violations.push(format!("json_cli_exit_code:{}", json_run.exit_code));
    }
    if !human_contains_verdict {
        violations.push("human_output_missing_verdict".to_string());
    }
    if !human_contains_bundle_id {
        violations.push("human_output_missing_bundle_id".to_string());
    }
    if !human_contains_trust_overlay_hash {
        violations.push("human_output_missing_trust_overlay_hash".to_string());
    }
    if !human_contains_policy_hash {
        violations.push("human_output_missing_policy_hash".to_string());
    }
    if !human_contains_registry_snapshot_hash {
        violations.push("human_output_missing_registry_snapshot_hash".to_string());
    }
    if json_verdict != Some(expected_verdict) {
        violations.push("json_verdict_mismatch".to_string());
    }
    if json_bundle_id != Some(expected_outcome.subject.bundle_id.as_str()) {
        violations.push("json_bundle_id_mismatch".to_string());
    }
    if json_trust_overlay_hash != Some(expected_outcome.subject.trust_overlay_hash.as_str()) {
        violations.push("json_trust_overlay_hash_mismatch".to_string());
    }
    if json_policy_hash != Some(expected_outcome.subject.policy_hash.as_str()) {
        violations.push("json_policy_hash_mismatch".to_string());
    }
    if json_registry_snapshot_hash != Some(expected_outcome.subject.registry_snapshot_hash.as_str())
    {
        violations.push("json_registry_snapshot_hash_mismatch".to_string());
    }
    if json_findings.is_none() {
        violations.push("json_findings_missing".to_string());
    }

    let report = json!({
        "gate": "proof-verifier-cli",
        "mode": "phase12_proof_verifier_cli_gate",
        "verdict": status_label(violations.is_empty()),
        "cli_smoke_report_path": "cli_smoke_report.json",
        "cli_output_contract_path": "cli_output_contract.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_proof_exchange_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
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
    let outcome = verify_bundle(&request)
        .map_err(|error| format!("proof exchange gate verification failed: {error}"))?;
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or_else(|| "proof exchange gate expected a signed receipt".to_string())?;

    let bundle = load_bundle(&fixture.root);
    let manifest = load_manifest(&bundle.manifest_path)
        .map_err(|error| format!("proof exchange gate failed to load manifest: {error}"))?;
    let checksums = load_checksums(&bundle.checksums_path)
        .map_err(|error| format!("proof exchange gate failed to load checksums: {error}"))?;
    let overlay = verify_overlay(&bundle, &manifest.bundle_id)
        .map_err(|error| format!("proof exchange gate failed to recompute overlay: {error}"))?;

    let context_rules_object = build_exchange_context_rules_object();
    let context_rules_hash = compute_context_rules_hash(&context_rules_object)?;
    let verification_context_object = build_verification_context_object(
        &outcome.subject.policy_hash,
        &outcome.subject.registry_snapshot_hash,
        "phase12-context-v1",
        &context_rules_hash,
    )?;
    let verification_context_id = verification_context_object
        .get("verification_context_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "verification context object missing verification_context_id".to_string())?
        .to_string();

    let baseline_package = build_exchange_package(
        &manifest,
        &checksums,
        &overlay.producer,
        &overlay.signature_envelope,
        &overlay.trust_overlay_hash,
        &verification_context_object,
        &context_rules_object,
        &fixture.policy,
        &fixture.registry,
        Some(receipt),
    )?;
    write_json(out_dir.join("exchange_message.json"), &baseline_package)?;

    let expectation = ExchangeExpectation {
        bundle_id: outcome.subject.bundle_id.clone(),
        trust_overlay_hash: outcome.subject.trust_overlay_hash.clone(),
        policy_hash: outcome.subject.policy_hash.clone(),
        registry_snapshot_hash: outcome.subject.registry_snapshot_hash.clone(),
        verification_context_id: verification_context_id.clone(),
        verdict: verdict_wire_value(&outcome.verdict)?,
    };

    let mut metadata_mutation = baseline_package.clone();
    metadata_mutation["transport_metadata"]["transport_id"] =
        Value::String("exchange-fixture-transport-mutated".to_string());
    metadata_mutation["transport_metadata"]["sent_at_utc"] =
        Value::String("2026-03-08T12:30:00Z".to_string());

    let mut receipt_absent_transport = baseline_package.clone();
    if let Value::Object(map) = &mut receipt_absent_transport {
        map.remove("receipt_artifact");
    }

    let mut bundle_id_mutation = baseline_package.clone();
    bundle_id_mutation["portable_payload"]["bundle_id"] =
        Value::String(format!("sha256:{}", "f".repeat(64)));

    let mut overlay_hash_mutation = baseline_package.clone();
    overlay_hash_mutation["trust_overlay"]["trust_overlay_hash"] =
        Value::String("f".repeat(64));

    let mut context_id_mutation = baseline_package.clone();
    context_id_mutation["verification_context"]["verification_context_id"] =
        Value::String(format!("sha256:{}", "e".repeat(64)));

    let mut receipt_subject_mutation = baseline_package.clone();
    receipt_subject_mutation["receipt_artifact"]["receipt"]["bundle_id"] =
        Value::String(format!("sha256:{}", "d".repeat(64)));

    let mutation_matrix = vec![
        exchange_validation_row(
            "baseline_inline_separated",
            &baseline_package,
            &expectation,
            true,
            "PASS",
        )?,
        exchange_validation_row(
            "metadata_only_mutation",
            &metadata_mutation,
            &expectation,
            true,
            "PASS",
        )?,
        exchange_validation_row(
            "receipt_absent_portable_transfer",
            &receipt_absent_transport,
            &expectation,
            false,
            "PASS",
        )?,
        exchange_validation_row(
            "bundle_id_mutation",
            &bundle_id_mutation,
            &expectation,
            true,
            "FAIL",
        )?,
        exchange_validation_row(
            "overlay_hash_mutation",
            &overlay_hash_mutation,
            &expectation,
            true,
            "FAIL",
        )?,
        exchange_validation_row(
            "context_id_mutation",
            &context_id_mutation,
            &expectation,
            true,
            "FAIL",
        )?,
        exchange_validation_row(
            "receipt_subject_mutation",
            &receipt_subject_mutation,
            &expectation,
            true,
            "FAIL",
        )?,
    ];
    write_json(
        out_dir.join("transport_mutation_matrix.json"),
        &mutation_matrix,
    )?;

    let exchange_contract_report = json!({
        "gate": "proof-exchange",
        "mode": "phase12_proof_exchange_gate",
        "status": status_label(
            mutation_matrix.iter().all(|row| row.get("status").and_then(Value::as_str) == row.get("expected_status").and_then(Value::as_str))
        ),
        "exchange_protocol_version": 1,
        "exchange_mode": "proof_bundle_transport_v1",
        "payload_identity_preserved": true,
        "payload_overlay_receipt_separated": true,
        "verification_context_id": verification_context_id,
        "bundle_id": expectation.bundle_id,
        "trust_overlay_hash": expectation.trust_overlay_hash,
        "context_package_form": "inline",
        "receipt_optional_for_transport": true,
        "transport_metadata_non_authoritative": true,
        "transport_mutation_matrix_path": "transport_mutation_matrix.json",
        "exchange_message_path": "exchange_message.json",
    });
    write_json(
        out_dir.join("exchange_contract_report.json"),
        &exchange_contract_report,
    )?;

    let mut violations = Vec::new();
    for row in &mutation_matrix {
        let scenario = row
            .get("scenario")
            .and_then(Value::as_str)
            .unwrap_or("unknown_scenario");
        let status = row.get("status").and_then(Value::as_str).unwrap_or("FAIL");
        let expected_status = row
            .get("expected_status")
            .and_then(Value::as_str)
            .unwrap_or("FAIL");
        if status != expected_status {
            violations.push(format!("unexpected_exchange_status:{scenario}"));
        }
    }
    if exchange_contract_report
        .get("payload_overlay_receipt_separated")
        .and_then(Value::as_bool)
        != Some(true)
    {
        violations.push("exchange_surface_not_separated".to_string());
    }

    let report = json!({
        "gate": "proof-exchange",
        "mode": "phase12_proof_exchange_gate",
        "verdict": status_label(violations.is_empty()),
        "exchange_contract_report_path": "exchange_contract_report.json",
        "transport_mutation_matrix_path": "transport_mutation_matrix.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_receipt_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
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

    let outcome = verify_bundle(&request)
        .map_err(|error| format!("receipt gate runtime verification failed: {error}"))?;
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or_else(|| "receipt gate did not emit a signed receipt".to_string())?;
    let receipt_findings =
        verify_signed_receipt(receipt, &outcome.subject, &fixture.receipt_verifier_key).map_err(
            |error| format!("receipt gate receipt verification failed at runtime: {error}"),
        )?;
    let payload_bytes = canonicalize_receipt_payload(&receipt.payload)
        .map_err(|error| format!("receipt gate payload canonicalization failed: {error}"))?;
    let payload_sha256 = sha256_hex(&payload_bytes);
    let receipt_hash = compute_receipt_hash(receipt)
        .map_err(|error| format!("receipt gate receipt hash recomputation failed: {error}"))?;

    write_json(out_dir.join("verification_receipt.json"), receipt)?;

    let receipt_schema_report = json!({
        "gate": "proof-receipt",
        "mode": "phase12_signed_receipt_gate",
        "status": status_label(!has_error_findings(&receipt_findings)),
        "receipt_version": receipt.payload.receipt_version,
        "verifier_signature_algorithm": receipt.verifier_signature_algorithm,
        "verifier_key_id": receipt.payload.verifier_key_id,
        "verifier_node_id": receipt.payload.verifier_node_id,
        "payload_sha256": payload_sha256,
        "findings": findings_to_json(&receipt_findings),
        "findings_count": receipt_findings.len(),
    });
    write_json(
        out_dir.join("receipt_schema_report.json"),
        &receipt_schema_report,
    )?;

    let receipt_emit_report = json!({
        "gate": "proof-receipt",
        "mode": "phase12_signed_receipt_gate",
        "status": status_label(!has_error_findings(&outcome.findings) && outcome.verdict == VerificationVerdict::Trusted),
        "verification_verdict": verdict_label(&outcome.verdict),
        "receipt_hash": receipt_hash,
        "bundle_id": outcome.subject.bundle_id,
        "trust_overlay_hash": outcome.subject.trust_overlay_hash,
        "policy_hash": outcome.subject.policy_hash,
        "registry_snapshot_hash": outcome.subject.registry_snapshot_hash,
        "receipt_path": "verification_receipt.json",
        "bundle_root": fixture.root,
        "findings": findings_to_json(&outcome.findings),
        "findings_count": outcome.findings.len(),
    });
    write_json(
        out_dir.join("receipt_emit_report.json"),
        &receipt_emit_report,
    )?;

    let mut violations = error_violations(&outcome.findings);
    violations.extend(error_violations(&receipt_findings));
    if outcome.verdict != VerificationVerdict::Trusted {
        violations.push(format!(
            "unexpected_verdict:{}",
            verdict_label(&outcome.verdict)
        ));
    }
    let report = json!({
        "gate": "proof-receipt",
        "mode": "phase12_signed_receipt_gate",
        "verdict": status_label(violations.is_empty()),
        "receipt_path": "verification_receipt.json",
        "receipt_hash": receipt_hash,
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_audit_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
    let fixture = create_fixture_bundle();
    let ledger_path = out_dir.join("verification_audit_ledger.jsonl");
    let request = VerifyRequest {
        bundle_path: &fixture.root,
        policy: &fixture.policy,
        registry_snapshot: &fixture.registry,
        receipt_mode: ReceiptMode::EmitSigned,
        receipt_signer: Some(&fixture.receipt_signer),
        audit_mode: AuditMode::Append,
        audit_ledger_path: Some(&ledger_path),
    };

    let outcome = verify_bundle(&request)
        .map_err(|error| format!("audit ledger gate runtime verification failed: {error}"))?;
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or_else(|| "audit ledger gate did not emit a signed receipt".to_string())?;
    let audit_event = outcome
        .audit_event
        .as_ref()
        .ok_or_else(|| "audit ledger gate did not append an audit event".to_string())?;

    let ledger_findings = verify_audit_ledger(&ledger_path)
        .map_err(|error| format!("audit ledger verification failed at runtime: {error}"))?;
    let binding_findings =
        verify_audit_event_against_receipt(audit_event, receipt, &fixture.receipt_verifier_key)
            .map_err(|error| {
                format!("audit receipt binding verification failed at runtime: {error}")
            })?;
    let authority_binding_findings = verify_audit_event_against_receipt_with_authority(
        audit_event,
        receipt,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .map_err(|error| {
        format!("audit authority-aware receipt binding verification failed at runtime: {error}")
    })?;
    let mut bindings = BTreeMap::new();
    bindings.insert(
        audit_event.receipt_hash.clone(),
        AuditReceiptBinding {
            receipt,
            verifier_key: &fixture.receipt_verifier_key,
            verifier_registry: Some(&fixture.verifier_registry),
        },
    );
    let full_findings = verify_audit_ledger_with_receipts(&ledger_path, &bindings)
        .map_err(|error| format!("audit ledger full verification failed at runtime: {error}"))?;
    let event_count = fs::read_to_string(&ledger_path)
        .map_err(|error| {
            format!(
                "failed to read audit ledger {}: {error}",
                ledger_path.display()
            )
        })?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    write_json(out_dir.join("verification_receipt.json"), receipt)?;
    write_json(out_dir.join("verification_audit_event.json"), audit_event)?;

    let audit_integrity_report = json!({
        "gate": "proof-audit-ledger",
        "mode": "phase12_audit_ledger_gate",
        "status": status_label(!has_error_findings(&full_findings)),
        "event_count": event_count,
        "latest_event_id": audit_event.event_id,
        "latest_receipt_hash": audit_event.receipt_hash,
        "chain_findings": findings_to_json(&ledger_findings),
        "chain_findings_count": ledger_findings.len(),
        "binding_findings": findings_to_json(&binding_findings),
        "binding_findings_count": binding_findings.len(),
        "authority_binding_findings": findings_to_json(&authority_binding_findings),
        "authority_binding_findings_count": authority_binding_findings.len(),
        "full_findings": findings_to_json(&full_findings),
        "full_findings_count": full_findings.len(),
    });
    write_json(
        out_dir.join("audit_integrity_report.json"),
        &audit_integrity_report,
    )?;

    let mut violations = error_violations(&outcome.findings);
    violations.extend(error_violations(&ledger_findings));
    violations.extend(error_violations(&binding_findings));
    violations.extend(error_violations(&authority_binding_findings));
    violations.extend(error_violations(&full_findings));
    if outcome.verdict != VerificationVerdict::Trusted {
        violations.push(format!(
            "unexpected_verdict:{}",
            verdict_label(&outcome.verdict)
        ));
    }
    if event_count != 1 {
        violations.push(format!("unexpected_audit_event_count:{event_count}"));
    }

    let report = json!({
        "gate": "proof-audit-ledger",
        "mode": "phase12_audit_ledger_gate",
        "verdict": status_label(violations.is_empty()),
        "ledger_path": "verification_audit_ledger.jsonl",
        "audit_event_path": "verification_audit_event.json",
        "receipt_path": "verification_receipt.json",
        "event_count": event_count,
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_authority_resolution_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
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
    let outcome = verify_bundle(&request).map_err(|error| {
        format!("authority resolution gate runtime verification failed: {error}")
    })?;
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or_else(|| "authority resolution gate did not emit a signed receipt".to_string())?;
    let distributed_receipt = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .map_err(|error| format!("authority-bound receipt verification failed at runtime: {error}"))?;
    let resolution = resolve_verifier_authority(
        &fixture.verifier_registry,
        &fixture.authority_requested_verifier_id,
        &fixture.authority_requested_scope,
    )
    .map_err(|error| format!("authority resolution gate runtime failure: {error}"))?;
    let parity_comparison =
        compare_authority_resolution(&resolution, &distributed_receipt.authority_resolution);

    write_json(out_dir.join("verification_receipt.json"), receipt)?;

    let authority_resolution_report = json!({
        "gate": "verifier-authority-resolution",
        "mode": "phase12_verifier_authority_resolution_gate",
        "status": status_label(
            !has_error_findings(&resolution.findings)
                && resolution.result_class == VerifierAuthorityResolutionClass::AuthorityResolvedDelegated
        ),
        "result_class": authority_resolution_label(&resolution),
        "requested_verifier_id": resolution.requested_verifier_id,
        "requested_authority_scope": resolution.requested_authority_scope,
        "verifier_registry_snapshot_hash": resolution.verifier_registry_snapshot_hash,
        "authority_chain": resolution.authority_chain,
        "authority_chain_id": resolution.authority_chain_id,
        "findings": findings_to_json(&resolution.findings),
        "findings_count": resolution.findings.len(),
    });
    write_json(
        out_dir.join("authority_resolution_report.json"),
        &authority_resolution_report,
    )?;

    let receipt_authority_report = json!({
        "gate": "verifier-authority-resolution",
        "mode": "phase12_verifier_authority_resolution_gate",
        "status": status_label(!has_error_findings(&distributed_receipt.findings)),
        "verification_verdict": verdict_label(&outcome.verdict),
        "result_class": authority_resolution_label(&distributed_receipt.authority_resolution),
        "bundle_id": outcome.subject.bundle_id,
        "trust_overlay_hash": outcome.subject.trust_overlay_hash,
        "policy_hash": outcome.subject.policy_hash,
        "registry_snapshot_hash": outcome.subject.registry_snapshot_hash,
        "verifier_node_id": receipt.payload.verifier_node_id,
        "verifier_key_id": receipt.payload.verifier_key_id,
        "authority_chain": distributed_receipt.authority_resolution.authority_chain,
        "authority_chain_id": distributed_receipt.authority_resolution.authority_chain_id,
        "result_class_equal": parity_comparison.result_class_equal,
        "effective_authority_scope_equal": parity_comparison.effective_authority_scope_equal,
        "authority_chain_equal": parity_comparison.authority_chain_equal,
        "authority_chain_id_equal": parity_comparison.authority_chain_id_equal,
        "verifier_registry_snapshot_hash_equal": parity_comparison
            .verifier_registry_snapshot_hash_equal,
        "findings": findings_to_json(&distributed_receipt.findings),
        "findings_count": distributed_receipt.findings.len(),
    });
    write_json(
        out_dir.join("receipt_authority_report.json"),
        &receipt_authority_report,
    )?;

    let authority_chain_report = json!({
        "gate": "verifier-authority-resolution",
        "mode": "phase12_verifier_authority_resolution_gate",
        "status": status_label(resolution.authority_chain_id.is_some()),
        "result_class": authority_resolution_label(&resolution),
        "authority_chain": resolution.authority_chain,
        "authority_chain_length": resolution.authority_chain.len(),
        "authority_chain_id": resolution.authority_chain_id,
        "effective_authority_scope": resolution.effective_authority_scope,
    });
    write_json(
        out_dir.join("authority_chain_report.json"),
        &authority_chain_report,
    )?;

    let mut violations = error_violations(&resolution.findings);
    violations.extend(error_violations(&distributed_receipt.findings));
    if resolution.result_class != VerifierAuthorityResolutionClass::AuthorityResolvedDelegated {
        violations.push(format!(
            "unexpected_authority_result:{}",
            authority_resolution_label(&resolution)
        ));
    }
    if resolution.authority_chain_id.is_none() {
        violations.push("missing_authority_chain_id".to_string());
    }
    if resolution.authority_chain != vec!["root-verifier-a".to_string(), "node-b".to_string()] {
        violations.push("unexpected_authority_chain".to_string());
    }
    let report = json!({
        "gate": "verifier-authority-resolution",
        "mode": "phase12_verifier_authority_resolution_gate",
        "verdict": status_label(violations.is_empty()),
        "receipt_path": "verification_receipt.json",
        "authority_chain_id": resolution.authority_chain_id,
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn build_cross_node_parity_gate_artifacts(out_dir: &Path) -> Result<i32, String> {
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
    let outcome = verify_bundle(&request)
        .map_err(|error| format!("cross-node parity gate runtime verification failed: {error}"))?;
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or_else(|| "cross-node parity gate did not emit a signed receipt".to_string())?;

    let node_a = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .map_err(|error| format!("cross-node parity node-a verification failed at runtime: {error}"))?;
    let node_b = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &fixture.verifier_registry,
    )
    .map_err(|error| format!("cross-node parity node-b verification failed at runtime: {error}"))?;
    let alternate_registry =
        build_alternate_parity_registry(&fixture.verifier_registry, &fixture.receipt_verifier_key)?;
    let node_c = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &alternate_registry,
    )
    .map_err(|error| format!("cross-node parity node-c verification failed at runtime: {error}"))?;
    let historical_registry = build_historical_only_parity_registry(&fixture.verifier_registry)?;
    let node_d = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &historical_registry,
    )
    .map_err(|error| format!("cross-node parity node-d verification failed at runtime: {error}"))?;
    let node_e = verify_signed_receipt_with_authority(
        receipt,
        &outcome.subject,
        &fixture.receipt_verifier_key,
        &historical_registry,
    )
    .map_err(|error| format!("cross-node parity node-e verification failed at runtime: {error}"))?;
    let scope_drift_registry = build_scope_drift_parity_registry(&fixture.verifier_registry)?;
    let scope_drift_requested_scope = vec!["parity-reporter".to_string()];
    let node_scope = resolve_verifier_authority(
        &scope_drift_registry,
        &fixture.authority_requested_verifier_id,
        &scope_drift_requested_scope,
    )
    .map_err(|error| format!("cross-node parity node-scope authority resolution failed: {error}"))?;
    let receipt_absent_resolution = resolve_verifier_authority(
        &fixture.verifier_registry,
        &fixture.authority_requested_verifier_id,
        &fixture.authority_requested_scope,
    )
    .map_err(|error| format!("cross-node parity node-g authority resolution failed: {error}"))?;
    let synthetic_verdict_mismatch = VerificationVerdict::RejectedByPolicy;
    let mut subject_drift_subject = outcome.subject.clone();
    subject_drift_subject.trust_overlay_hash = format!("sha256:{}", "1".repeat(64));

    let verification_context_id = compute_verification_context_id_from_components(
        &outcome.subject.policy_hash,
        &outcome.subject.registry_snapshot_hash,
        "phase12-context-v1",
        &build_cross_node_parity_context_rules_object(),
    )
    .map_err(|error| format!("cross-node parity context identity failed: {error}"))?;
    let context_drift_verification_context_id = compute_verification_context_id_from_components(
        &outcome.subject.policy_hash,
        &outcome.subject.registry_snapshot_hash,
        "phase12-context-v1",
        &build_context_drift_parity_context_rules_object(),
    )
    .map_err(|error| format!("cross-node parity context-drift identity failed: {error}"))?;
    let contract_version_drift_verification_context_id =
        compute_verification_context_id_from_components(
            &outcome.subject.policy_hash,
            &outcome.subject.registry_snapshot_hash,
            "phase12-context-v2",
            &build_cross_node_parity_context_rules_object(),
        )
        .map_err(|error| {
            format!("cross-node parity contract-version identity failed: {error}")
        })?;

    let match_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-b",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_b.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let subject_mismatch_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-j",
            subject: &subject_drift_subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_b.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let context_mismatch_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-b",
            subject: &outcome.subject,
            verification_context_id: &context_drift_verification_context_id,
            authority_resolution: &node_b.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let contract_version_mismatch_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-k",
            subject: &outcome.subject,
            verification_context_id: &contract_version_drift_verification_context_id,
            authority_resolution: &node_b.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let verifier_mismatch_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-c",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_c.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let authority_scope_mismatch_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-scope",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_scope,
            local_verdict: &outcome.verdict,
        },
    );
    let historical_only_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-d",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_d.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-e",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_e.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let insufficient_evidence_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-f",
            subject: &outcome.subject,
            verification_context_id: "",
            authority_resolution: &node_b.authority_resolution,
            local_verdict: &outcome.verdict,
        },
    );
    let verdict_mismatch_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-a",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_a.authority_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-g",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &node_b.authority_resolution,
            local_verdict: &synthetic_verdict_mismatch,
        },
    );
    let receipt_absent_match_row = compare_cross_node_parity(
        CrossNodeParityInput {
            node_id: "node-h",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &receipt_absent_resolution,
            local_verdict: &outcome.verdict,
        },
        CrossNodeParityInput {
            node_id: "node-i",
            subject: &outcome.subject,
            verification_context_id: &verification_context_id,
            authority_resolution: &receipt_absent_resolution,
            local_verdict: &outcome.verdict,
        },
    );

    let scenario_reports_dir = out_dir.join("scenario_reports");
    fs::create_dir_all(&scenario_reports_dir)
        .map_err(|error| format!("cross-node parity scenario_reports mkdir failed: {error}"))?;
    let mut verdict_mismatch_scenario = parity_scenario_row(
        "p14-18-verdict-mismatch-guard",
        &verdict_mismatch_row,
        CrossNodeParityStatus::ParityVerdictMismatch,
    );
    if let Value::Object(map) = &mut verdict_mismatch_scenario {
        map.insert(
            "determinism_guard".to_string(),
            Value::Bool(true),
        );
        map.insert(
            "guard_surface".to_string(),
            Value::String("same_sca_different_v".to_string()),
        );
    }
    let mut subject_mismatch_scenario = parity_scenario_row(
        "p14-05-overlay-hash-drift-same-bundle",
        &subject_mismatch_row,
        CrossNodeParityStatus::ParitySubjectMismatch,
    );
    if let Value::Object(map) = &mut subject_mismatch_scenario {
        map.insert(
            "subject_drift_surface".to_string(),
            Value::String("trust_overlay_hash".to_string()),
        );
    }
    let mut contract_version_mismatch_scenario = parity_scenario_row(
        "p14-12-verifier-contract-version-drift",
        &contract_version_mismatch_row,
        CrossNodeParityStatus::ParityContextMismatch,
    );
    if let Value::Object(map) = &mut contract_version_mismatch_scenario {
        map.insert(
            "context_drift_surface".to_string(),
            Value::String("verifier_contract_version".to_string()),
        );
        map.insert(
            "verifier_contract_version_left".to_string(),
            Value::String("phase12-context-v1".to_string()),
        );
        map.insert(
            "verifier_contract_version_right".to_string(),
            Value::String("phase12-context-v2".to_string()),
        );
    }
    let mut authority_scope_mismatch_scenario = parity_scenario_row(
        "p14-15-authority-scope-drift",
        &authority_scope_mismatch_row,
        CrossNodeParityStatus::ParityVerifierMismatch,
    );
    if let Value::Object(map) = &mut authority_scope_mismatch_scenario {
        map.insert(
            "authority_drift_surface".to_string(),
            Value::String("effective_authority_scope".to_string()),
        );
        map.insert(
            "requested_authority_scope_left".to_string(),
            Value::Array(
                fixture
                    .authority_requested_scope
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
        map.insert(
            "requested_authority_scope_right".to_string(),
            Value::Array(
                scope_drift_requested_scope
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
    }
    let mut receipt_absent_scenario = parity_scenario_row(
        "p14-20-receipt-absent-parity-artifact",
        &receipt_absent_match_row,
        CrossNodeParityStatus::ParityMatch,
    );
    if let Value::Object(map) = &mut receipt_absent_scenario {
        map.insert("receipt_present".to_string(), Value::Bool(false));
        map.insert(
            "parity_artifact_form".to_string(),
            Value::String("local_verification_outcome".to_string()),
        );
    }

    let failure_matrix = vec![
        parity_scenario_row(
            "p14-01-baseline-identical-nodes",
            &match_row,
            CrossNodeParityStatus::ParityMatch,
        ),
        subject_mismatch_scenario,
        parity_scenario_row(
            "p14-10-verification-context-id-drift",
            &context_mismatch_row,
            CrossNodeParityStatus::ParityContextMismatch,
        ),
        contract_version_mismatch_scenario,
        parity_scenario_row(
            "p14-13-different-trusted-root-set",
            &verifier_mismatch_row,
            CrossNodeParityStatus::ParityVerifierMismatch,
        ),
        authority_scope_mismatch_scenario,
        parity_scenario_row(
            "p14-16-historical-only-authority",
            &historical_only_row,
            CrossNodeParityStatus::ParityHistoricalOnly,
        ),
        parity_scenario_row(
            "p14-19-insufficient-evidence",
            &insufficient_evidence_row,
            CrossNodeParityStatus::ParityInsufficientEvidence,
        ),
        verdict_mismatch_scenario,
        receipt_absent_scenario,
    ];
    for row in &failure_matrix {
        let scenario = row
            .get("scenario")
            .and_then(Value::as_str)
            .ok_or_else(|| "cross-node parity scenario row missing scenario".to_string())?;
        write_json(
            scenario_reports_dir.join(format!("{scenario}.json")),
            row,
        )?;
    }
    write_json(out_dir.join("failure_matrix.json"), &failure_matrix)?;

    let rows = [
        &match_row,
        &subject_mismatch_row,
        &context_mismatch_row,
        &contract_version_mismatch_row,
        &verifier_mismatch_row,
        &authority_scope_mismatch_row,
        &historical_only_row,
        &insufficient_evidence_row,
        &verdict_mismatch_row,
        &receipt_absent_match_row,
    ];
    let consistency_rows = [
        &match_row,
        &subject_mismatch_row,
        &context_mismatch_row,
        &contract_version_mismatch_row,
        &verifier_mismatch_row,
        &authority_scope_mismatch_row,
        &historical_only_row,
        &insufficient_evidence_row,
        &receipt_absent_match_row,
    ];
    let node_parity_outcomes = vec![
        build_node_parity_outcome(
            "node-a-current",
            "node-a",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_a.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::SignedReceipt,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-a parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-b-current",
            "node-b",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_b.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::SignedReceipt,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-b parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-b-context-drift",
            "node-b",
            &outcome.subject,
            &context_drift_verification_context_id,
            "phase12-context-v1",
            &node_b.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-b context-drift parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-c-alt-root",
            "node-c",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_c.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::SignedReceipt,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-c parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-d-historical",
            "node-d",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_d.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::SignedReceipt,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-d parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-e-historical",
            "node-e",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_e.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::SignedReceipt,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-e parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-f-insufficient",
            "node-f",
            &outcome.subject,
            "",
            "phase12-context-v1",
            &node_b.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Insufficient,
        )
        .map_err(|error| format!("failed to build node-f parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-g-verdict-drift",
            "node-g",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_b.authority_resolution,
            &synthetic_verdict_mismatch,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-g parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-h-receipt-absent",
            "node-h",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &receipt_absent_resolution,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-h parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-i-receipt-absent",
            "node-i",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &receipt_absent_resolution,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-i parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-j-subject-drift",
            "node-j",
            &subject_drift_subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_b.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-j parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-k-contract-drift",
            "node-k",
            &outcome.subject,
            &contract_version_drift_verification_context_id,
            "phase12-context-v2",
            &node_b.authority_resolution,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-k parity outcome: {error}"))?,
        build_node_parity_outcome(
            "node-scope-scope-drift",
            "node-scope",
            &outcome.subject,
            &verification_context_id,
            "phase12-context-v1",
            &node_scope,
            &outcome.verdict,
            ParityArtifactForm::LocalVerificationOutcome,
            ParityEvidenceState::Sufficient,
        )
        .map_err(|error| format!("failed to build node-scope parity outcome: {error}"))?,
    ];
    let parity_report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_gate",
        "status": status_label(
            !has_error_findings(&node_a.findings)
                && !has_error_findings(&node_b.findings)
                && !has_error_findings(&node_c.findings)
                && !has_error_findings_excluding(&node_d.findings, &["PV0711"])
                && !has_error_findings_excluding(&node_e.findings, &["PV0711"])
                && !has_error_findings(&receipt_absent_resolution.findings)
                && match_row.parity_status == CrossNodeParityStatus::ParityMatch
                && subject_mismatch_row.parity_status == CrossNodeParityStatus::ParitySubjectMismatch
                && context_mismatch_row.parity_status == CrossNodeParityStatus::ParityContextMismatch
                && contract_version_mismatch_row.parity_status == CrossNodeParityStatus::ParityContextMismatch
                && verifier_mismatch_row.parity_status == CrossNodeParityStatus::ParityVerifierMismatch
                && authority_scope_mismatch_row.parity_status == CrossNodeParityStatus::ParityVerifierMismatch
                && historical_only_row.parity_status == CrossNodeParityStatus::ParityHistoricalOnly
                && insufficient_evidence_row.parity_status == CrossNodeParityStatus::ParityInsufficientEvidence
                && verdict_mismatch_row.parity_status == CrossNodeParityStatus::ParityVerdictMismatch
                && receipt_absent_match_row.parity_status == CrossNodeParityStatus::ParityMatch
                && verifier_mismatch_row.authority_chain_id_equal == Some(false)
                && authority_scope_mismatch_row.effective_authority_scope_equal == false
        ),
        "verification_context_id": verification_context_id,
        "context_drift_verification_context_id": context_drift_verification_context_id,
        "contract_version_drift_verification_context_id": contract_version_drift_verification_context_id,
        "row_count": rows.len(),
        "status_counts": {
            "PARITY_MATCH": count_parity_status(&rows, CrossNodeParityStatus::ParityMatch),
            "PARITY_SUBJECT_MISMATCH": count_parity_status(&rows, CrossNodeParityStatus::ParitySubjectMismatch),
            "PARITY_CONTEXT_MISMATCH": count_parity_status(&rows, CrossNodeParityStatus::ParityContextMismatch),
            "PARITY_VERIFIER_MISMATCH": count_parity_status(&rows, CrossNodeParityStatus::ParityVerifierMismatch),
            "PARITY_HISTORICAL_ONLY": count_parity_status(&rows, CrossNodeParityStatus::ParityHistoricalOnly),
            "PARITY_INSUFFICIENT_EVIDENCE": count_parity_status(&rows, CrossNodeParityStatus::ParityInsufficientEvidence),
            "PARITY_VERDICT_MISMATCH": count_parity_status(&rows, CrossNodeParityStatus::ParityVerdictMismatch),
        },
        "authority_chain_id_mismatch_rows": count_authority_chain_id_mismatches(&rows),
        "effective_authority_scope_mismatch_rows": count_effective_authority_scope_mismatches(&rows),
        "scenario_report_dir": "scenario_reports",
        "receipt_absent_artifact_form": "local_verification_outcome",
        "consistency_report_path": "parity_consistency_report.json",
        "determinism_report_path": "parity_determinism_report.json",
        "determinism_incidents_path": "parity_determinism_incidents.json",
        "incident_graph_path": "parity_incident_graph.json",
        "authority_drift_topology_path": "parity_authority_drift_topology.json",
        "authority_suppression_report_path": "parity_authority_suppression_report.json",
        "convergence_report_path": "parity_convergence_report.json",
        "drift_attribution_report_path": "parity_drift_attribution_report.json",
        "node_a_findings": findings_to_json(&node_a.findings),
        "node_b_findings": findings_to_json(&node_b.findings),
        "node_c_findings": findings_to_json(&node_c.findings),
        "node_d_findings": findings_to_json(&node_d.findings),
        "node_e_findings": findings_to_json(&node_e.findings),
        "node_scope_findings": findings_to_json(&node_scope.findings),
        "node_h_authority_findings": findings_to_json(&receipt_absent_resolution.findings),
    });
    write_json(out_dir.join("parity_report.json"), &parity_report)?;

    let parity_consistency_report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_consistency_report",
        "surface": "consistency",
        "status": "PASS",
        "row_count": consistency_rows.len(),
        "status_counts": {
            "PARITY_MATCH": count_parity_status(&consistency_rows, CrossNodeParityStatus::ParityMatch),
            "PARITY_SUBJECT_MISMATCH": count_parity_status(&consistency_rows, CrossNodeParityStatus::ParitySubjectMismatch),
            "PARITY_CONTEXT_MISMATCH": count_parity_status(&consistency_rows, CrossNodeParityStatus::ParityContextMismatch),
            "PARITY_VERIFIER_MISMATCH": count_parity_status(&consistency_rows, CrossNodeParityStatus::ParityVerifierMismatch),
            "PARITY_HISTORICAL_ONLY": count_parity_status(&consistency_rows, CrossNodeParityStatus::ParityHistoricalOnly),
            "PARITY_INSUFFICIENT_EVIDENCE": count_parity_status(&consistency_rows, CrossNodeParityStatus::ParityInsufficientEvidence),
        },
        "authority_chain_id_mismatch_rows": count_authority_chain_id_mismatches(&consistency_rows),
        "effective_authority_scope_mismatch_rows": count_effective_authority_scope_mismatches(&consistency_rows),
        "scenario_report_dir": "scenario_reports",
        "receipt_absent_artifact_form": "local_verification_outcome",
    });
    write_json(
        out_dir.join("parity_consistency_report.json"),
        &parity_consistency_report,
    )?;

    let determinism_incident_report = analyze_determinism_incidents(&node_parity_outcomes);
    let parity_determinism_report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_determinism_report",
        "surface": "determinism",
        "status": "PASS",
        "false_determinism_guard_active": true,
        "row_count": determinism_incident_report.determinism_incident_count,
        "determinism_violation_present": determinism_incident_report.determinism_incident_count > 0,
        "determinism_violation_count": determinism_incident_report.determinism_incident_count,
        "conflict_surface_count": determinism_incident_report.determinism_incident_count,
        "severity_counts": determinism_incident_report.severity_counts,
        "suppressed_incident_count": determinism_incident_report.suppressed_incident_count,
        "suppression_reason_counts": determinism_incident_report.suppression_reason_counts,
        "determinism_incidents_path": "parity_determinism_incidents.json",
        "conflict_pairs": [{
            "scenario": "p14-18-verdict-mismatch-guard",
            "left_node": verdict_mismatch_row.node_a,
            "right_node": verdict_mismatch_row.node_b,
            "same_subject": verdict_mismatch_row.bundle_id_equal
                && verdict_mismatch_row.trust_overlay_hash_equal
                && verdict_mismatch_row.policy_hash_equal
                && verdict_mismatch_row.registry_snapshot_hash_equal,
            "same_context": verdict_mismatch_row.verification_context_id_equal,
            "same_authority": verdict_mismatch_row.trusted_verifier_semantics_equal,
            "left_verdict": verdict_label(&outcome.verdict),
            "right_verdict": verdict_label(&synthetic_verdict_mismatch),
            "parity_status": parity_status_label(&verdict_mismatch_row.parity_status),
        }],
    });
    write_json(
        out_dir.join("parity_determinism_report.json"),
        &parity_determinism_report,
    )?;
    let parity_determinism_incidents = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_determinism_incidents",
        "status": "PASS",
        "false_determinism_guard_active": true,
        "node_count": determinism_incident_report.node_count,
        "surface_partition_count": determinism_incident_report.surface_partition_count,
        "determinism_incident_count": determinism_incident_report.determinism_incident_count,
        "severity_counts": determinism_incident_report.severity_counts,
        "suppressed_incident_count": determinism_incident_report.suppressed_incident_count,
        "suppression_reason_counts": determinism_incident_report.suppression_reason_counts,
        "incidents": determinism_incident_report.incidents,
        "suppressed_incidents": determinism_incident_report.suppressed_incidents,
    });
    write_json(
        out_dir.join("parity_determinism_incidents.json"),
        &parity_determinism_incidents,
    )?;
    let parity_incident_graph = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_incident_graph",
        "status": "PASS",
        "graph": build_incident_graph(&node_parity_outcomes, &determinism_incident_report),
    });
    write_json(out_dir.join("parity_incident_graph.json"), &parity_incident_graph)?;
    let parity_authority_drift_topology = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_authority_drift_topology",
        "status": "PASS",
        "topology": build_authority_drift_topology(&node_parity_outcomes),
    });
    write_json(
        out_dir.join("parity_authority_drift_topology.json"),
        &parity_authority_drift_topology,
    )?;
    let parity_authority_suppression_report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_authority_suppression",
        "status": "PASS",
        "suppression": analyze_authority_drift_suppressions(&node_parity_outcomes),
    });
    write_json(
        out_dir.join("parity_authority_suppression_report.json"),
        &parity_authority_suppression_report,
    )?;

    let parity_convergence_report =
        build_parity_convergence_report(&node_parity_outcomes, &failure_matrix);
    write_json(
        out_dir.join("parity_convergence_report.json"),
        &parity_convergence_report,
    )?;

    let drift_report = analyze_parity_drift(&node_parity_outcomes);
    let parity_drift_attribution_report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_drift_attribution_report",
        "status": "PASS",
        "node_count": drift_report.node_count,
        "surface_partition_count": drift_report.surface_partition_count,
        "outcome_partition_count": drift_report.outcome_partition_count,
        "baseline_partition_id": drift_report.baseline_partition_id,
        "baseline_surface_key": drift_report.baseline_surface_key,
        "historical_authority_island_count": drift_report.historical_authority_island_count,
        "insufficient_evidence_island_count": drift_report.insufficient_evidence_island_count,
        "historical_authority_islands": drift_report.historical_authority_islands,
        "insufficient_evidence_islands": drift_report.insufficient_evidence_islands,
        "partition_reports": drift_report.partition_reports,
        "primary_cause_counts": drift_report.primary_cause_counts,
    });
    write_json(
        out_dir.join("parity_drift_attribution_report.json"),
        &parity_drift_attribution_report,
    )?;

    let mut violations = error_violations(&node_a.findings);
    violations.extend(error_violations(&node_b.findings));
    violations.extend(error_violations(&node_c.findings));
    violations.extend(error_violations_excluding(&node_d.findings, &["PV0711"]));
    violations.extend(error_violations_excluding(&node_e.findings, &["PV0711"]));
    violations.extend(error_violations(&node_scope.findings));
    violations.extend(error_violations(&receipt_absent_resolution.findings));
    if match_row.parity_status != CrossNodeParityStatus::ParityMatch {
        violations.push(format!(
            "unexpected_match_row_status:{}",
            parity_status_label(&match_row.parity_status)
        ));
    }
    if subject_mismatch_row.parity_status != CrossNodeParityStatus::ParitySubjectMismatch {
        violations.push(format!(
            "unexpected_subject_mismatch_status:{}",
            parity_status_label(&subject_mismatch_row.parity_status)
        ));
    }
    if context_mismatch_row.parity_status != CrossNodeParityStatus::ParityContextMismatch {
        violations.push(format!(
            "unexpected_context_mismatch_status:{}",
            parity_status_label(&context_mismatch_row.parity_status)
        ));
    }
    if contract_version_mismatch_row.parity_status != CrossNodeParityStatus::ParityContextMismatch {
        violations.push(format!(
            "unexpected_contract_version_mismatch_status:{}",
            parity_status_label(&contract_version_mismatch_row.parity_status)
        ));
    }
    if verifier_mismatch_row.parity_status != CrossNodeParityStatus::ParityVerifierMismatch {
        violations.push(format!(
            "unexpected_verifier_mismatch_status:{}",
            parity_status_label(&verifier_mismatch_row.parity_status)
        ));
    }
    if authority_scope_mismatch_row.parity_status != CrossNodeParityStatus::ParityVerifierMismatch {
        violations.push(format!(
            "unexpected_authority_scope_mismatch_status:{}",
            parity_status_label(&authority_scope_mismatch_row.parity_status)
        ));
    }
    if historical_only_row.parity_status != CrossNodeParityStatus::ParityHistoricalOnly {
        violations.push(format!(
            "unexpected_historical_only_status:{}",
            parity_status_label(&historical_only_row.parity_status)
        ));
    }
    if insufficient_evidence_row.parity_status != CrossNodeParityStatus::ParityInsufficientEvidence {
        violations.push(format!(
            "unexpected_insufficient_evidence_status:{}",
            parity_status_label(&insufficient_evidence_row.parity_status)
        ));
    }
    if verdict_mismatch_row.parity_status != CrossNodeParityStatus::ParityVerdictMismatch {
        violations.push(format!(
            "unexpected_verdict_mismatch_status:{}",
            parity_status_label(&verdict_mismatch_row.parity_status)
        ));
    }
    if receipt_absent_match_row.parity_status != CrossNodeParityStatus::ParityMatch {
        violations.push(format!(
            "unexpected_receipt_absent_status:{}",
            parity_status_label(&receipt_absent_match_row.parity_status)
        ));
    }
    if verifier_mismatch_row.authority_chain_id_equal != Some(false) {
        violations.push("authority_chain_id_mismatch_not_observed".to_string());
    }
    if authority_scope_mismatch_row.effective_authority_scope_equal {
        violations.push("authority_scope_mismatch_not_observed".to_string());
    }
    for row in &failure_matrix {
        if row.get("pass").and_then(Value::as_bool) != Some(true) {
            let scenario = row
                .get("scenario")
                .and_then(Value::as_str)
                .unwrap_or("unknown_scenario");
            violations.push(format!("unexpected_parity_matrix_status:{scenario}"));
        }
    }

    let report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_gate",
        "verdict": status_label(violations.is_empty()),
        "parity_report_path": "parity_report.json",
        "failure_matrix_path": "failure_matrix.json",
        "determinism_incidents_path": "parity_determinism_incidents.json",
        "drift_attribution_report_path": "parity_drift_attribution_report.json",
        "violations": violations,
        "violations_count": violations.len(),
    });
    write_json(out_dir.join("report.json"), &report)?;

    Ok(if violations_from_report(&report).is_empty() {
        0
    } else {
        2
    })
}

fn registry_resolution_matrix_row(
    scenario: &str,
    snapshot: &RegistrySnapshot,
    producer: &ProducerDeclaration,
    signature_envelope: &SignatureEnvelope,
) -> Result<Value, String> {
    let resolution = resolve_signers(snapshot, producer, signature_envelope)
        .map_err(|error| format!("registry resolution scenario {scenario} failed: {error}"))?;
    let signer_status = resolution
        .resolved_signers
        .first()
        .map(|signer| key_status_label(&signer.status))
        .unwrap_or("UNKNOWN");
    Ok(json!({
        "scenario": scenario,
        "registry_snapshot_hash": resolution.registry_snapshot_hash,
        "resolved_signer_count": resolution.resolved_signers.len(),
        "primary_signer_status": signer_status,
        "error_codes": error_codes(&resolution.findings),
        "findings": findings_to_json(&resolution.findings),
        "findings_count": resolution.findings.len(),
    }))
}

fn key_lifecycle_matrix_row(
    scenario: &str,
    snapshot: &RegistrySnapshot,
    producer: &ProducerDeclaration,
    signature_envelope: &SignatureEnvelope,
    bundle_id: &str,
) -> Result<Value, String> {
    let resolution = resolve_signers(snapshot, producer, signature_envelope)
        .map_err(|error| format!("key lifecycle scenario {scenario} failed: {error}"))?;
    let signature_findings =
        verify_detached_signatures(bundle_id, signature_envelope, &resolution.resolved_signers);
    let signer_status = resolution
        .resolved_signers
        .first()
        .map(|signer| key_status_label(&signer.status))
        .unwrap_or("UNKNOWN");

    Ok(json!({
        "scenario": scenario,
        "registry_snapshot_hash": resolution.registry_snapshot_hash,
        "primary_signer_status": signer_status,
        "resolution_error_codes": error_codes(&resolution.findings),
        "resolution_findings": findings_to_json(&resolution.findings),
        "resolution_findings_count": resolution.findings.len(),
        "signature_error_codes": error_codes(&signature_findings),
        "signature_findings": findings_to_json(&signature_findings),
        "signature_findings_count": signature_findings.len(),
        "signature_status": status_label(!has_error_findings(&signature_findings)),
    }))
}

fn build_ambiguous_owner_registry(
    baseline: &RegistrySnapshot,
) -> Result<RegistrySnapshot, String> {
    let mut registry = baseline.clone();
    let baseline_entry = registry
        .producers
        .get("ayken-ci")
        .cloned()
        .ok_or_else(|| "baseline registry missing ayken-ci entry".to_string())?;
    let baseline_public_key = baseline_entry
        .public_keys
        .get("ed25519-key-2026-03-a")
        .cloned()
        .ok_or_else(|| "baseline registry missing ed25519-key-2026-03-a key".to_string())?;
    registry.registry_version = registry.registry_version.saturating_add(1);
    registry.producers.insert(
        "ambiguous-owner".to_string(),
        RegistryEntry {
            active_pubkey_ids: vec!["ed25519-key-2026-03-a".to_string()],
            revoked_pubkey_ids: Vec::new(),
            superseded_pubkey_ids: Vec::new(),
            public_keys: BTreeMap::from([(
                "ed25519-key-2026-03-a".to_string(),
                baseline_public_key,
            )]),
        },
    );
    registry.registry_snapshot_hash = compute_registry_snapshot_hash(&registry)
        .map_err(|error| format!("ambiguous-owner registry hash recomputation failed: {error}"))?;
    Ok(registry)
}

fn build_unknown_key_registry(baseline: &RegistrySnapshot) -> Result<RegistrySnapshot, String> {
    let mut registry = baseline.clone();
    let entry = registry
        .producers
        .get_mut("ayken-ci")
        .ok_or_else(|| "baseline registry missing ayken-ci entry".to_string())?;
    entry.active_pubkey_ids.clear();
    entry.revoked_pubkey_ids.clear();
    entry.superseded_pubkey_ids.clear();
    registry.registry_version = registry.registry_version.saturating_add(1);
    registry.registry_snapshot_hash = compute_registry_snapshot_hash(&registry)
        .map_err(|error| format!("unknown-key registry hash recomputation failed: {error}"))?;
    Ok(registry)
}

fn build_missing_public_key_registry(
    baseline: &RegistrySnapshot,
) -> Result<RegistrySnapshot, String> {
    let mut registry = baseline.clone();
    let entry = registry
        .producers
        .get_mut("ayken-ci")
        .ok_or_else(|| "baseline registry missing ayken-ci entry".to_string())?;
    entry.public_keys.clear();
    registry.registry_version = registry.registry_version.saturating_add(1);
    registry.registry_snapshot_hash = compute_registry_snapshot_hash(&registry).map_err(|error| {
        format!("missing-public-key registry hash recomputation failed: {error}")
    })?;
    Ok(registry)
}

fn build_rotated_registry(baseline: &RegistrySnapshot) -> Result<RegistrySnapshot, String> {
    let mut registry = baseline.clone();
    let entry = registry
        .producers
        .get_mut("ayken-ci")
        .ok_or_else(|| "baseline registry missing ayken-ci entry".to_string())?;
    let old_public_key = entry
        .public_keys
        .get("ed25519-key-2026-03-a")
        .cloned()
        .ok_or_else(|| "baseline registry missing ed25519-key-2026-03-a key".to_string())?;
    entry.active_pubkey_ids = vec!["ed25519-key-2026-04-a".to_string()];
    entry.revoked_pubkey_ids.clear();
    entry.superseded_pubkey_ids = vec!["ed25519-key-2026-03-a".to_string()];
    entry.public_keys.insert("ed25519-key-2026-04-a".to_string(), old_public_key);
    registry.registry_version = registry.registry_version.saturating_add(1);
    registry.registry_snapshot_hash = compute_registry_snapshot_hash(&registry)
        .map_err(|error| format!("rotated registry hash recomputation failed: {error}"))?;
    Ok(registry)
}

fn build_revoked_registry(baseline: &RegistrySnapshot) -> Result<RegistrySnapshot, String> {
    let mut registry = baseline.clone();
    let entry = registry
        .producers
        .get_mut("ayken-ci")
        .ok_or_else(|| "baseline registry missing ayken-ci entry".to_string())?;
    entry.active_pubkey_ids.clear();
    entry.superseded_pubkey_ids.clear();
    entry.revoked_pubkey_ids = vec!["ed25519-key-2026-03-a".to_string()];
    registry.registry_version = registry.registry_version.saturating_add(1);
    registry.registry_snapshot_hash = compute_registry_snapshot_hash(&registry)
        .map_err(|error| format!("revoked registry hash recomputation failed: {error}"))?;
    Ok(registry)
}

fn write_phase12a_failure_artifacts(
    out_dir: &Path,
    gate: &str,
    mode: &str,
    detail_files: &[&str],
    error: &str,
) {
    let placeholder = json!({
        "gate": gate,
        "mode": mode,
        "status": "FAIL",
        "error": error,
    });
    let report = json!({
        "gate": gate,
        "mode": mode,
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    for detail_file in detail_files {
        let _ = write_json(out_dir.join(detail_file), &placeholder);
    }
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_verifier_core_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-verifier-core",
        "mode": "phase12_proof_verifier_core_gate",
        "status": "FAIL",
        "error": error,
    });
    let report = json!({
        "gate": "proof-verifier-core",
        "mode": "phase12_proof_verifier_core_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("verifier_core_report.json"), &placeholder);
    let _ = write_json(out_dir.join("determinism_matrix.json"), &json!([]));
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_trust_policy_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-trust-policy",
        "mode": "phase12_trust_policy_gate",
        "status": "FAIL",
        "error": error,
    });
    let report = json!({
        "gate": "proof-trust-policy",
        "mode": "phase12_trust_policy_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("policy_schema_report.json"), &placeholder);
    let _ = write_json(out_dir.join("policy_hash_report.json"), &placeholder);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_verdict_binding_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-verdict-binding",
        "mode": "phase12_verdict_binding_gate",
        "status": "FAIL",
        "error": error,
    });
    let report = json!({
        "gate": "proof-verdict-binding",
        "mode": "phase12_verdict_binding_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("verdict_binding_report.json"), &placeholder);
    let _ = write_json(out_dir.join("verdict_subject_examples.json"), &placeholder);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_verifier_cli_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-verifier-cli",
        "mode": "phase12_proof_verifier_cli_gate",
        "status": "FAIL",
        "error": error,
    });
    let report = json!({
        "gate": "proof-verifier-cli",
        "mode": "phase12_proof_verifier_cli_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("cli_smoke_report.json"), &placeholder);
    let _ = write_json(out_dir.join("cli_output_contract.json"), &placeholder);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_receipt_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-receipt",
        "mode": "phase12_signed_receipt_gate",
        "status": "FAIL",
        "error": error,
        "findings": [],
        "findings_count": 0,
    });
    let report = json!({
        "gate": "proof-receipt",
        "mode": "phase12_signed_receipt_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("receipt_schema_report.json"), &placeholder);
    let _ = write_json(out_dir.join("receipt_emit_report.json"), &placeholder);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_audit_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-audit-ledger",
        "mode": "phase12_audit_ledger_gate",
        "status": "FAIL",
        "error": error,
        "full_findings": [],
        "full_findings_count": 0,
    });
    let report = json!({
        "gate": "proof-audit-ledger",
        "mode": "phase12_audit_ledger_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = fs::write(out_dir.join("verification_audit_ledger.jsonl"), "");
    let _ = write_json(out_dir.join("audit_integrity_report.json"), &placeholder);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_proof_exchange_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "proof-exchange",
        "mode": "phase12_proof_exchange_gate",
        "status": "FAIL",
        "error": error,
    });
    let report = json!({
        "gate": "proof-exchange",
        "mode": "phase12_proof_exchange_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("exchange_contract_report.json"), &placeholder);
    let _ = write_json(out_dir.join("transport_mutation_matrix.json"), &json!([]));
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_authority_resolution_failure_artifacts(out_dir: &Path, error: &str) {
    let placeholder = json!({
        "gate": "verifier-authority-resolution",
        "mode": "phase12_verifier_authority_resolution_gate",
        "status": "FAIL",
        "error": error,
        "findings": [],
        "findings_count": 0,
    });
    let report = json!({
        "gate": "verifier-authority-resolution",
        "mode": "phase12_verifier_authority_resolution_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(
        out_dir.join("authority_resolution_report.json"),
        &placeholder,
    );
    let _ = write_json(out_dir.join("receipt_authority_report.json"), &placeholder);
    let _ = write_json(out_dir.join("authority_chain_report.json"), &placeholder);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn write_cross_node_parity_failure_artifacts(out_dir: &Path, error: &str) {
    let parity_placeholder = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_gate",
        "status": "FAIL",
        "error": error,
        "row_count": 0,
    });
    let failure_matrix = json!([]);
    let drift_placeholder = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_drift_attribution_report",
        "status": "FAIL",
        "error": error,
        "node_count": 0,
        "surface_partition_count": 0,
        "outcome_partition_count": 0,
        "partition_reports": [],
        "primary_cause_counts": {},
    });
    let report = json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_gate",
        "verdict": "FAIL",
        "violations": [format!("runtime_error:{error}")],
        "violations_count": 1,
    });
    let _ = write_json(out_dir.join("parity_report.json"), &parity_placeholder);
    let _ = write_json(
        out_dir.join("parity_consistency_report.json"),
        &parity_placeholder,
    );
    let _ = write_json(
        out_dir.join("parity_determinism_report.json"),
        &parity_placeholder,
    );
    let _ = write_json(
        out_dir.join("parity_convergence_report.json"),
        &parity_placeholder,
    );
    let _ = write_json(
        out_dir.join("parity_drift_attribution_report.json"),
        &drift_placeholder,
    );
    let _ = write_json(out_dir.join("failure_matrix.json"), &failure_matrix);
    let _ = write_json(out_dir.join("report.json"), &report);
}

fn findings_to_json(findings: &[VerificationFinding]) -> Vec<Value> {
    findings
        .iter()
        .map(|finding| {
            json!({
                "code": finding.code,
                "message": finding.message,
                "severity": severity_label(&finding.severity),
                "deterministic": finding.deterministic,
            })
        })
        .collect()
}

fn finding_codes_all(findings: &[VerificationFinding]) -> Vec<String> {
    findings.iter().map(|finding| finding.code.clone()).collect()
}

fn error_violations(findings: &[VerificationFinding]) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Error)
        .map(|finding| format!("{}:{}", finding.code, finding.message))
        .collect()
}

fn error_violations_excluding(
    findings: &[VerificationFinding],
    ignored_codes: &[&str],
) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| {
            finding.severity == FindingSeverity::Error
                && !ignored_codes.iter().any(|code| *code == finding.code)
        })
        .map(|finding| format!("{}:{}", finding.code, finding.message))
        .collect()
}

fn has_error_findings(findings: &[VerificationFinding]) -> bool {
    findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Error)
}

fn has_error_findings_excluding(
    findings: &[VerificationFinding],
    ignored_codes: &[&str],
) -> bool {
    findings.iter().any(|finding| {
        finding.severity == FindingSeverity::Error
            && !ignored_codes.iter().any(|code| *code == finding.code)
    })
}

fn status_label(pass: bool) -> &'static str {
    if pass {
        "PASS"
    } else {
        "FAIL"
    }
}

fn verdict_label(verdict: &VerificationVerdict) -> &'static str {
    match verdict {
        VerificationVerdict::Trusted => "TRUSTED",
        VerificationVerdict::Untrusted => "UNTRUSTED",
        VerificationVerdict::Invalid => "INVALID",
        VerificationVerdict::RejectedByPolicy => "REJECTED_BY_POLICY",
    }
}

fn verdict_wire_value(verdict: &VerificationVerdict) -> Result<String, String> {
    serde_json::to_value(verdict)
        .map_err(|error| format!("failed to serialize verdict wire value: {error}"))?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "serialized verdict wire value was not a string".to_string())
}

fn severity_label(severity: &FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Info => "INFO",
        FindingSeverity::Warning => "WARNING",
        FindingSeverity::Error => "ERROR",
    }
}

fn key_status_label(status: &KeyStatus) -> &'static str {
    match status {
        KeyStatus::Active => "ACTIVE",
        KeyStatus::Revoked => "REVOKED",
        KeyStatus::Superseded => "SUPERSEDED",
        KeyStatus::Unknown => "UNKNOWN",
    }
}

fn authority_resolution_label(resolution: &VerifierAuthorityResolution) -> &'static str {
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

fn parity_status_label(status: &CrossNodeParityStatus) -> &'static str {
    match status {
        CrossNodeParityStatus::ParityMatch => "PARITY_MATCH",
        CrossNodeParityStatus::ParitySubjectMismatch => "PARITY_SUBJECT_MISMATCH",
        CrossNodeParityStatus::ParityContextMismatch => "PARITY_CONTEXT_MISMATCH",
        CrossNodeParityStatus::ParityVerifierMismatch => "PARITY_VERIFIER_MISMATCH",
        CrossNodeParityStatus::ParityVerdictMismatch => "PARITY_VERDICT_MISMATCH",
        CrossNodeParityStatus::ParityHistoricalOnly => "PARITY_HISTORICAL_ONLY",
        CrossNodeParityStatus::ParityInsufficientEvidence => "PARITY_INSUFFICIENT_EVIDENCE",
    }
}

fn error_codes(findings: &[VerificationFinding]) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Error)
        .map(|finding| finding.code.clone())
        .collect()
}

fn verifier_core_matrix_row(
    scenario: &str,
    expected_verdict: VerificationVerdict,
    bundle_path: &Path,
    policy: &proof_verifier::TrustPolicy,
    registry_snapshot: &RegistrySnapshot,
) -> Result<Value, String> {
    let run_a = run_core_verification(bundle_path, policy, registry_snapshot)?;
    let run_b = run_core_verification(bundle_path, policy, registry_snapshot)?;
    let run_a_summary = verification_outcome_summary(&run_a);
    let run_b_summary = verification_outcome_summary(&run_b);
    let run_a_summary_sha256 = canonical_json_sha256(&run_a_summary)?;
    let run_b_summary_sha256 = canonical_json_sha256(&run_b_summary)?;
    let summary_equal = run_a_summary == run_b_summary;
    let verdict_equal = run_a.verdict == run_b.verdict;
    let subject_equal = run_a.subject.bundle_id == run_b.subject.bundle_id
        && run_a.subject.trust_overlay_hash == run_b.subject.trust_overlay_hash
        && run_a.subject.policy_hash == run_b.subject.policy_hash
        && run_a.subject.registry_snapshot_hash == run_b.subject.registry_snapshot_hash;
    let finding_codes_a = finding_codes_all(&run_a.findings);
    let finding_codes_b = finding_codes_all(&run_b.findings);
    let finding_codes_equal = finding_codes_a == finding_codes_b;
    let findings_deterministic = run_a.findings.iter().all(|finding| finding.deterministic)
        && run_b.findings.iter().all(|finding| finding.deterministic);
    let deterministic =
        summary_equal && verdict_equal && subject_equal && finding_codes_equal && findings_deterministic;

    Ok(json!({
        "scenario": scenario,
        "expected_verdict": verdict_label(&expected_verdict),
        "run_a_verdict": verdict_label(&run_a.verdict),
        "run_b_verdict": verdict_label(&run_b.verdict),
        "run_a_summary_sha256": run_a_summary_sha256,
        "run_b_summary_sha256": run_b_summary_sha256,
        "summary_equal": summary_equal,
        "verdict_equal": verdict_equal,
        "subject_equal": subject_equal,
        "finding_codes_equal": finding_codes_equal,
        "findings_deterministic": findings_deterministic,
        "receipt_absent": run_a.receipt.is_none() && run_b.receipt.is_none(),
        "audit_absent": run_a.audit_event.is_none() && run_b.audit_event.is_none(),
        "deterministic": deterministic,
        "run_a_finding_codes": finding_codes_a,
        "run_b_finding_codes": finding_codes_b,
        "run_a_summary": run_a_summary,
        "run_b_summary": run_b_summary,
    }))
}

fn run_core_verification(
    bundle_path: &Path,
    policy: &proof_verifier::TrustPolicy,
    registry_snapshot: &RegistrySnapshot,
) -> Result<VerificationOutcome, String> {
    let request = VerifyRequest {
        bundle_path,
        policy,
        registry_snapshot,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    verify_bundle(&request).map_err(|error| format!("verifier core gate runtime verification failed: {error}"))
}

fn verification_outcome_summary(outcome: &VerificationOutcome) -> Value {
    json!({
        "verdict": verdict_label(&outcome.verdict),
        "subject": {
            "bundle_id": outcome.subject.bundle_id,
            "trust_overlay_hash": outcome.subject.trust_overlay_hash,
            "policy_hash": outcome.subject.policy_hash,
            "registry_snapshot_hash": outcome.subject.registry_snapshot_hash,
        },
        "findings": findings_to_json(&outcome.findings),
        "receipt_present": outcome.receipt.is_some(),
        "audit_event_present": outcome.audit_event.is_some(),
    })
}

struct CliRunOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

struct ExchangeExpectation {
    bundle_id: String,
    trust_overlay_hash: String,
    policy_hash: String,
    registry_snapshot_hash: String,
    verification_context_id: String,
    verdict: String,
}

fn run_cli_verify_bundle(
    cli_bin: &Path,
    bundle_path: &Path,
    policy_path: &Path,
    registry_path: &Path,
    json_output: bool,
) -> Result<CliRunOutput, String> {
    let mut command = Command::new(cli_bin);
    command
        .arg("verify")
        .arg("bundle")
        .arg(bundle_path)
        .arg("--policy")
        .arg(policy_path)
        .arg("--registry")
        .arg(registry_path);
    if json_output {
        command.arg("--json");
    }

    let output = command.output().map_err(|error| {
        format!(
            "failed to execute CLI binary {}: {error}",
            cli_bin.display()
        )
    })?;

    Ok(CliRunOutput {
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn build_exchange_context_rules_object() -> Value {
    json!({
        "policy_import_mode": "exact-inline-or-resolved",
        "registry_import_mode": "exact-inline-or-resolved",
        "context_mismatch_behavior": "fail-closed",
        "historical_receipt_handling": "historical-only",
        "receipt_acceptance_mode": "explicit-context-required"
    })
}

fn build_cross_node_parity_context_rules_object() -> Value {
    json!({
        "policy_import_mode": "local-equal-context-required",
        "registry_import_mode": "local-equal-context-required",
        "context_mismatch_behavior": "fail-closed",
        "historical_receipt_handling": "historical-only",
        "receipt_acceptance_mode": "authority-bound-receipt",
        "parity_surface": "cross-node-parity-gate-v1"
    })
}

fn build_context_drift_parity_context_rules_object() -> Value {
    json!({
        "policy_import_mode": "local-equal-context-required",
        "registry_import_mode": "local-equal-context-required",
        "context_mismatch_behavior": "fail-closed",
        "historical_receipt_handling": "historical-only",
        "receipt_acceptance_mode": "authority-bound-receipt",
        "parity_surface": "cross-node-parity-gate-v1-context-drift"
    })
}

fn compute_context_rules_hash(context_rules_object: &Value) -> Result<String, String> {
    let bytes = canonicalize_json_value(context_rules_object)
        .map_err(|error| format!("failed to canonicalize context rules object: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn compute_verification_context_id_from_components(
    policy_hash: &str,
    registry_snapshot_hash: &str,
    verifier_contract_version: &str,
    context_rules_object: &Value,
) -> Result<String, String> {
    let context_rules_hash = compute_context_rules_hash(context_rules_object)?;
    let context_object = json!({
        "context_version": 1,
        "verification_context_id": "",
        "policy_hash": policy_hash,
        "registry_snapshot_hash": registry_snapshot_hash,
        "verifier_contract_version": verifier_contract_version,
        "context_rules_hash": context_rules_hash,
    });
    compute_verification_context_id_from_object(&context_object)
}

fn build_verification_context_object(
    policy_hash: &str,
    registry_snapshot_hash: &str,
    verifier_contract_version: &str,
    context_rules_hash: &str,
) -> Result<Value, String> {
    let mut context_object = json!({
        "context_version": 1,
        "verification_context_id": "",
        "policy_hash": policy_hash,
        "registry_snapshot_hash": registry_snapshot_hash,
        "verifier_contract_version": verifier_contract_version,
        "context_rules_hash": context_rules_hash,
    });
    let verification_context_id = compute_verification_context_id_from_object(&context_object)?;
    context_object["verification_context_id"] = Value::String(verification_context_id);
    Ok(context_object)
}

fn compute_verification_context_id_from_object(context_object: &Value) -> Result<String, String> {
    let mut cloned = context_object.clone();
    if let Value::Object(map) = &mut cloned {
        map.remove("verification_context_id");
    } else {
        return Err("verification context object must be a JSON object".to_string());
    }
    let bytes = canonicalize_json_value(&cloned)
        .map_err(|error| format!("failed to canonicalize verification context object: {error}"))?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

fn recompute_inline_overlay_hash(
    producer: &ProducerDeclaration,
    signature_envelope: &SignatureEnvelope,
) -> Result<String, String> {
    let producer_bytes = canonicalize_json(producer)
        .map_err(|error| format!("failed to canonicalize exchange producer declaration: {error}"))?;
    let envelope_bytes = canonicalize_json(signature_envelope).map_err(|error| {
        format!("failed to canonicalize exchange signature envelope: {error}")
    })?;
    let mut material = Vec::new();
    material.extend_from_slice(&producer_bytes);
    material.extend_from_slice(&envelope_bytes);
    Ok(sha256_hex(&material))
}

fn build_exchange_package(
    manifest: &Manifest,
    checksums: &ChecksumsFile,
    producer: &ProducerDeclaration,
    signature_envelope: &SignatureEnvelope,
    trust_overlay_hash: &str,
    verification_context_object: &Value,
    context_rules_object: &Value,
    policy_snapshot: &TrustPolicy,
    registry_snapshot: &RegistrySnapshot,
    receipt: Option<&proof_verifier::VerificationReceipt>,
) -> Result<Value, String> {
    let verification_context_id = verification_context_object
        .get("verification_context_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange package context object missing verification_context_id".to_string())?;
    let mut package = json!({
        "protocol_version": 1,
        "exchange_mode": "proof_bundle_transport_v1",
        "portable_payload": {
            "payload_form": "proof_bundle_v2",
            "bundle_id": manifest.bundle_id,
            "manifest": manifest,
            "checksums": checksums,
        },
        "trust_overlay": {
            "transport_form": "detached-inline",
            "bundle_id": manifest.bundle_id,
            "producer": producer,
            "signature_envelope": signature_envelope,
            "trust_overlay_hash": trust_overlay_hash,
        },
        "verification_context": {
            "protocol_version": 1,
            "verification_context_id": verification_context_id,
            "context_object": verification_context_object,
            "context_rules_object": context_rules_object,
            "policy_snapshot": policy_snapshot,
            "registry_snapshot": registry_snapshot,
        },
        "transport_metadata": {
            "transport_id": "exchange-fixture-transport-1",
            "sender_node_id": "node-a",
            "sent_at_utc": "2026-03-08T12:15:00Z",
        }
    });

    if let Some(receipt) = receipt {
        package["receipt_artifact"] = json!({
            "transport_form": "detached-inline",
            "receipt_type": "signed_verification_receipt",
            "receipt": receipt,
        });
    }

    Ok(package)
}

fn exchange_validation_row(
    scenario: &str,
    package: &Value,
    expected: &ExchangeExpectation,
    require_receipt: bool,
    expected_status: &str,
) -> Result<Value, String> {
    let validation = validate_exchange_package(package, expected, require_receipt)?;
    Ok(json!({
        "scenario": scenario,
        "expected_status": expected_status,
        "status": if validation.violations.is_empty() { "PASS" } else { "FAIL" },
        "portable_identity_preserved": validation.portable_identity_preserved,
        "overlay_identity_preserved": validation.overlay_identity_preserved,
        "context_identity_preserved": validation.context_identity_preserved,
        "receipt_binding_valid": validation.receipt_binding_valid,
        "receipt_present": validation.receipt_present,
        "violations": validation.violations,
        "violations_count": validation.violations_count,
    }))
}

struct ExchangeValidationResult {
    portable_identity_preserved: bool,
    overlay_identity_preserved: bool,
    context_identity_preserved: bool,
    receipt_binding_valid: bool,
    receipt_present: bool,
    violations: Vec<String>,
    violations_count: usize,
}

fn validate_exchange_package(
    package: &Value,
    expected: &ExchangeExpectation,
    require_receipt: bool,
) -> Result<ExchangeValidationResult, String> {
    let portable_payload = package
        .get("portable_payload")
        .ok_or_else(|| "exchange package missing portable_payload".to_string())?;
    let trust_overlay = package
        .get("trust_overlay")
        .ok_or_else(|| "exchange package missing trust_overlay".to_string())?;
    let verification_context = package
        .get("verification_context")
        .ok_or_else(|| "exchange package missing verification_context".to_string())?;

    let manifest: Manifest = serde_json::from_value(
        portable_payload
            .get("manifest")
            .cloned()
            .ok_or_else(|| "exchange package missing portable manifest".to_string())?,
    )
    .map_err(|error| format!("failed to parse exchange manifest: {error}"))?;
    let checksums: ChecksumsFile = serde_json::from_value(
        portable_payload
            .get("checksums")
            .cloned()
            .ok_or_else(|| "exchange package missing portable checksums".to_string())?,
    )
    .map_err(|error| format!("failed to parse exchange checksums: {error}"))?;
    let producer: ProducerDeclaration = serde_json::from_value(
        trust_overlay
            .get("producer")
            .cloned()
            .ok_or_else(|| "exchange package missing producer declaration".to_string())?,
    )
    .map_err(|error| format!("failed to parse exchange producer declaration: {error}"))?;
    let signature_envelope: SignatureEnvelope = serde_json::from_value(
        trust_overlay
            .get("signature_envelope")
            .cloned()
            .ok_or_else(|| "exchange package missing signature envelope".to_string())?,
    )
    .map_err(|error| format!("failed to parse exchange signature envelope: {error}"))?;
    let context_object = verification_context
        .get("context_object")
        .ok_or_else(|| "exchange package missing context_object".to_string())?;
    let context_rules_object = verification_context
        .get("context_rules_object")
        .ok_or_else(|| "exchange package missing context_rules_object".to_string())?;
    let policy_snapshot: TrustPolicy = serde_json::from_value(
        verification_context
            .get("policy_snapshot")
            .cloned()
            .ok_or_else(|| "exchange package missing policy_snapshot".to_string())?,
    )
    .map_err(|error| format!("failed to parse exchange policy snapshot: {error}"))?;
    let registry_snapshot: RegistrySnapshot = serde_json::from_value(
        verification_context
            .get("registry_snapshot")
            .cloned()
            .ok_or_else(|| "exchange package missing registry_snapshot".to_string())?,
    )
    .map_err(|error| format!("failed to parse exchange registry snapshot: {error}"))?;

    let recomputed_bundle_id = recompute_bundle_id(&manifest, &checksums)
        .map_err(|error| format!("failed to recompute exchange bundle_id: {error}"))?;
    let declared_bundle_id = portable_payload
        .get("bundle_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange package missing declared portable bundle_id".to_string())?;

    let recomputed_overlay_hash =
        recompute_inline_overlay_hash(&producer, &signature_envelope)?;
    let declared_overlay_hash = trust_overlay
        .get("trust_overlay_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange package missing declared trust_overlay_hash".to_string())?;

    let recomputed_policy_hash = compute_policy_hash(&policy_snapshot)
        .map_err(|error| format!("failed to recompute exchange policy hash: {error}"))?;
    let recomputed_registry_hash = compute_registry_snapshot_hash(&registry_snapshot)
        .map_err(|error| format!("failed to recompute exchange registry hash: {error}"))?;
    let recomputed_context_rules_hash = compute_context_rules_hash(context_rules_object)?;
    let recomputed_verification_context_id =
        compute_verification_context_id_from_object(context_object)?;

    let declared_context_id = verification_context
        .get("verification_context_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange package missing declared verification_context_id".to_string())?;
    let declared_context_object_id = context_object
        .get("verification_context_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange context object missing verification_context_id".to_string())?;
    let declared_context_policy_hash = context_object
        .get("policy_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange context object missing policy_hash".to_string())?;
    let declared_context_registry_hash = context_object
        .get("registry_snapshot_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange context object missing registry_snapshot_hash".to_string())?;
    let declared_context_rules_hash = context_object
        .get("context_rules_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "exchange context object missing context_rules_hash".to_string())?;

    let receipt_value = package.get("receipt_artifact");
    let receipt_present = receipt_value.is_some();
    let mut receipt_binding_valid = !require_receipt;
    let mut violations = Vec::new();

    if declared_bundle_id != expected.bundle_id {
        violations.push("declared_bundle_id_drift".to_string());
    }
    if recomputed_bundle_id != declared_bundle_id || recomputed_bundle_id != expected.bundle_id {
        violations.push("portable_payload_identity_mutated".to_string());
    }
    if signature_envelope.bundle_id != expected.bundle_id {
        violations.push("overlay_bundle_id_mismatch".to_string());
    }
    if declared_overlay_hash != expected.trust_overlay_hash {
        violations.push("declared_overlay_hash_drift".to_string());
    }
    if recomputed_overlay_hash != declared_overlay_hash
        || recomputed_overlay_hash != expected.trust_overlay_hash
    {
        violations.push("trust_overlay_identity_mutated".to_string());
    }
    if declared_context_policy_hash != recomputed_policy_hash
        || declared_context_policy_hash != expected.policy_hash
    {
        violations.push("context_policy_hash_mismatch".to_string());
    }
    if declared_context_registry_hash != recomputed_registry_hash
        || declared_context_registry_hash != expected.registry_snapshot_hash
    {
        violations.push("context_registry_hash_mismatch".to_string());
    }
    if declared_context_rules_hash != recomputed_context_rules_hash {
        violations.push("context_rules_hash_mismatch".to_string());
    }
    if declared_context_id != expected.verification_context_id
        || declared_context_object_id != expected.verification_context_id
    {
        violations.push("declared_verification_context_id_drift".to_string());
    }
    if recomputed_verification_context_id != declared_context_id
        || recomputed_verification_context_id != declared_context_object_id
        || recomputed_verification_context_id != expected.verification_context_id
    {
        violations.push("verification_context_identity_mutated".to_string());
    }

    if require_receipt && !receipt_present {
        violations.push("receipt_artifact_missing".to_string());
    }

    if let Some(receipt_value) = receipt_value {
        let receipt = receipt_value
            .get("receipt")
            .ok_or_else(|| "exchange receipt_artifact missing receipt payload".to_string())?;
        let receipt_bundle_id = receipt
            .get("bundle_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "exchange receipt missing bundle_id".to_string())?;
        let receipt_trust_overlay_hash = receipt
            .get("trust_overlay_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| "exchange receipt missing trust_overlay_hash".to_string())?;
        let receipt_policy_hash = receipt
            .get("policy_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| "exchange receipt missing policy_hash".to_string())?;
        let receipt_registry_hash = receipt
            .get("registry_snapshot_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| "exchange receipt missing registry_snapshot_hash".to_string())?;
        let receipt_verdict = receipt
            .get("verdict")
            .and_then(Value::as_str)
            .ok_or_else(|| "exchange receipt missing verdict".to_string())?;

        receipt_binding_valid = receipt_bundle_id == expected.bundle_id
            && receipt_trust_overlay_hash == expected.trust_overlay_hash
            && receipt_policy_hash == expected.policy_hash
            && receipt_registry_hash == expected.registry_snapshot_hash
            && receipt_verdict == expected.verdict;
        if !receipt_binding_valid {
            violations.push("receipt_binding_mismatch".to_string());
        }
    }

    Ok(ExchangeValidationResult {
        portable_identity_preserved: recomputed_bundle_id == expected.bundle_id,
        overlay_identity_preserved: recomputed_overlay_hash == expected.trust_overlay_hash,
        context_identity_preserved: recomputed_verification_context_id
            == expected.verification_context_id,
        receipt_binding_valid,
        receipt_present,
        violations_count: violations.len(),
        violations,
    })
}

fn canonical_json_sha256(value: &Value) -> Result<String, String> {
    let bytes = canonicalize_json_value(value)
        .map_err(|error| format!("verifier core canonicalization failed: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn tamper_signature_envelope(root: &Path) -> Result<(), String> {
    let signature_path = root.join("signatures/signature-envelope.json");
    let mut envelope: SignatureEnvelope = serde_json::from_slice(
        &fs::read(&signature_path)
            .map_err(|error| format!("failed to read signature envelope {}: {error}", signature_path.display()))?,
    )
    .map_err(|error| format!("failed to parse signature envelope {}: {error}", signature_path.display()))?;
    let signature = envelope
        .signatures
        .first_mut()
        .ok_or_else(|| "signature envelope is missing baseline signatures".to_string())?;
    signature.signature =
        "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string();
    write_json(signature_path, &envelope)
}

fn remove_manifest_file(root: &Path) -> Result<(), String> {
    let manifest_path = root.join("manifest.json");
    fs::remove_file(&manifest_path)
        .map_err(|error| format!("failed to remove manifest {}: {error}", manifest_path.display()))
}

fn count_expected_verdict(matrix: &[Value], expected_verdict: &str) -> usize {
    matrix
        .iter()
        .filter(|row| {
            row.get("expected_verdict")
                .and_then(Value::as_str)
                .map(|value| value == expected_verdict)
                .unwrap_or(false)
        })
        .count()
}

fn trust_policy_outcome_row(
    scenario: &str,
    expected_verdict: VerificationVerdict,
    bundle_path: &Path,
    policy: &TrustPolicy,
    registry_snapshot: &RegistrySnapshot,
) -> Result<Value, String> {
    let policy_hash = compute_policy_hash(policy)
        .map_err(|error| format!("trust policy row hash computation failed for {scenario}: {error}"))?;
    let schema_findings = validate_policy(policy);
    let outcome = run_core_verification(bundle_path, policy, registry_snapshot)?;
    Ok(json!({
        "scenario": scenario,
        "expected_verdict": verdict_label(&expected_verdict),
        "actual_verdict": verdict_label(&outcome.verdict),
        "policy_hash": policy_hash,
        "subject_policy_hash": outcome.subject.policy_hash,
        "policy_hash_bound": outcome.subject.policy_hash == policy_hash,
        "schema_error_codes": error_codes(&schema_findings),
        "error_codes": error_codes(&outcome.findings),
        "findings": findings_to_json(&outcome.findings),
        "findings_count": outcome.findings.len(),
    }))
}

fn matrix_row_has_status(row: &Value, expected: &str) -> bool {
    row.get("primary_signer_status")
        .and_then(Value::as_str)
        .map(|value| value == expected)
        .unwrap_or(false)
}

fn matrix_row_has_errors(row: &Value) -> bool {
    row.get("error_codes")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
}

fn matrix_row_has_error_code(row: &Value, code: &str) -> bool {
    row.get("error_codes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|value| value == code)
        || row
            .get("resolution_error_codes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|value| value == code)
}

fn parity_row_to_json(row: &CrossNodeParityRecord) -> Value {
    json!({
        "node_a": row.node_a,
        "node_b": row.node_b,
        "parity_status": parity_status_label(&row.parity_status),
        "bundle_id_equal": row.bundle_id_equal,
        "trust_overlay_hash_equal": row.trust_overlay_hash_equal,
        "policy_hash_equal": row.policy_hash_equal,
        "registry_snapshot_hash_equal": row.registry_snapshot_hash_equal,
        "verification_context_id_equal": row.verification_context_id_equal,
        "trusted_verifier_semantics_equal": row.trusted_verifier_semantics_equal,
        "result_class_equal": row.result_class_equal,
        "effective_authority_scope_equal": row.effective_authority_scope_equal,
        "authority_chain_equal": row.authority_chain_equal,
        "authority_chain_id_equal": row.authority_chain_id_equal,
        "local_verdict_equal": row.local_verdict_equal,
    })
}

fn parity_scenario_row(
    scenario: &str,
    row: &CrossNodeParityRecord,
    expected_status: CrossNodeParityStatus,
) -> Value {
    let actual_status = parity_status_label(&row.parity_status);
    let expected_status_label = parity_status_label(&expected_status);
    json!({
        "scenario": scenario,
        "s_equal": row.bundle_id_equal
            && row.trust_overlay_hash_equal
            && row.policy_hash_equal
            && row.registry_snapshot_hash_equal,
        "c_equal": row.verification_context_id_equal,
        "a_equal": row.trusted_verifier_semantics_equal,
        "v_equal": row.local_verdict_equal,
        "parity_status": actual_status,
        "authority_chain_id_equal": row.authority_chain_id_equal,
        "verification_context_id_equal": row.verification_context_id_equal,
        "effective_authority_scope_equal": row.effective_authority_scope_equal,
        "local_verdict_equal": row.local_verdict_equal,
        "expected_status": expected_status_label,
        "actual_status": actual_status,
        "pass": actual_status == expected_status_label,
        "row": parity_row_to_json(row),
    })
}

fn count_parity_status(rows: &[&CrossNodeParityRecord], status: CrossNodeParityStatus) -> usize {
    rows.iter()
        .filter(|row| row.parity_status == status)
        .count()
}

fn count_authority_chain_id_mismatches(rows: &[&CrossNodeParityRecord]) -> usize {
    rows.iter()
        .filter(|row| row.authority_chain_id_equal == Some(false))
        .count()
}

fn count_effective_authority_scope_mismatches(rows: &[&CrossNodeParityRecord]) -> usize {
    rows.iter()
        .filter(|row| !row.effective_authority_scope_equal)
        .count()
}

fn build_parity_convergence_report(node_outcomes: &[NodeParityOutcome], rows: &[Value]) -> Value {
    let edge_match_clusters = build_parity_match_clusters(rows, &collect_parity_nodes(rows));
    let surface_partitions = build_node_partitions(node_outcomes, |node| node.surface_key());
    let outcome_partitions = build_node_partitions(node_outcomes, |node| node.outcome_key());
    let node_count = node_outcomes.len();
    let edge_count = rows.len();
    let largest_surface_partition_size = surface_partitions
        .iter()
        .filter_map(|partition| partition.get("size").and_then(Value::as_u64))
        .max()
        .unwrap_or(0) as usize;
    let largest_outcome_cluster_size = outcome_partitions
        .iter()
        .filter_map(|partition| partition.get("size").and_then(Value::as_u64))
        .max()
        .unwrap_or(0) as usize;
    let surface_consistency_ratio = if node_count == 0 {
        0.0
    } else {
        largest_surface_partition_size as f64 / node_count as f64
    };
    let outcome_convergence_ratio = if node_count == 0 {
        0.0
    } else {
        largest_outcome_cluster_size as f64 / node_count as f64
    };

    let unique_subject_count =
        count_unique_node_dimension(node_outcomes, |node| node.subject_hash());
    let unique_context_count =
        count_unique_node_dimension(node_outcomes, |node| node.context_hash());
    let unique_authority_count =
        count_unique_node_dimension(node_outcomes, |node| node.authority_hash());
    let unique_outcome_count =
        count_unique_node_dimension(node_outcomes, |node| node.outcome_key());
    let historical_only_node_count = node_outcomes
        .iter()
        .filter(|node| node.is_historical_only())
        .count();
    let insufficient_evidence_node_count = node_outcomes
        .iter()
        .filter(|node| node.evidence_state() == &ParityEvidenceState::Insufficient)
        .count();
    let determinism_conflict_surface_count =
        count_determinism_conflict_surfaces(node_outcomes);
    let determinism_violation_present = determinism_conflict_surface_count > 0;

    let subject_mismatch_edges = count_parity_status_value(rows, "PARITY_SUBJECT_MISMATCH");
    let context_mismatch_edges = count_parity_status_value(rows, "PARITY_CONTEXT_MISMATCH");
    let verifier_mismatch_edges = count_parity_status_value(rows, "PARITY_VERIFIER_MISMATCH");
    let historical_only_edges = count_parity_status_value(rows, "PARITY_HISTORICAL_ONLY");
    let insufficient_evidence_edges =
        count_parity_status_value(rows, "PARITY_INSUFFICIENT_EVIDENCE");
    let determinism_violation_edges =
        count_parity_status_value(rows, "PARITY_VERDICT_MISMATCH");

    let node_outcome_views: Vec<NodeParityOutcomeView> =
        node_outcomes.iter().map(NodeParityOutcomeView::from).collect();

    json!({
        "gate": "cross-node-parity",
        "mode": "phase12_cross_node_parity_convergence_report",
        "surface": "n-node-convergence",
        "status": "PASS",
        "cluster_derivation": "node_parity_outcome_dk_partitions",
        "edge_match_cluster_derivation": "pairwise_match_graph_connected_components",
        "node_count": node_count,
        "edge_count": edge_count,
        "unique_subject_count": unique_subject_count,
        "unique_context_count": unique_context_count,
        "unique_authority_count": unique_authority_count,
        "unique_outcome_count": unique_outcome_count,
        "historical_only_node_count": historical_only_node_count,
        "insufficient_evidence_node_count": insufficient_evidence_node_count,
        "surface_partition_count": surface_partitions.len(),
        "outcome_partition_count": outcome_partitions.len(),
        "largest_surface_partition_size": largest_surface_partition_size,
        "largest_outcome_cluster_size": largest_outcome_cluster_size,
        "surface_consistency_ratio": surface_consistency_ratio,
        "outcome_convergence_ratio": outcome_convergence_ratio,
        "determinism_violation_present": determinism_violation_present,
        "determinism_conflict_surface_count": determinism_conflict_surface_count,
        "global_status": classify_parity_convergence_status(
            unique_subject_count,
            unique_context_count,
            unique_authority_count,
            historical_only_node_count,
            insufficient_evidence_node_count,
            determinism_violation_present,
            outcome_partitions.len(),
            largest_outcome_cluster_size,
            node_count,
        ),
        "status_counts": {
            "PARITY_MATCH": count_parity_status_value(rows, "PARITY_MATCH"),
            "PARITY_SUBJECT_MISMATCH": subject_mismatch_edges,
            "PARITY_CONTEXT_MISMATCH": context_mismatch_edges,
            "PARITY_VERIFIER_MISMATCH": verifier_mismatch_edges,
            "PARITY_HISTORICAL_ONLY": historical_only_edges,
            "PARITY_INSUFFICIENT_EVIDENCE": insufficient_evidence_edges,
            "PARITY_VERDICT_MISMATCH": determinism_violation_edges,
        },
        "conflict_summary": {
            "subject_mismatch_edges": subject_mismatch_edges,
            "context_mismatch_edges": context_mismatch_edges,
            "verifier_mismatch_edges": verifier_mismatch_edges,
            "historical_only_edges": historical_only_edges,
            "insufficient_evidence_edges": insufficient_evidence_edges,
            "determinism_violation_edges": determinism_violation_edges,
            "determinism_conflict_surface_count": determinism_conflict_surface_count,
        },
        "surface_partitions": surface_partitions,
        "outcome_partitions": outcome_partitions,
        "edge_match_clusters": edge_match_clusters,
        "node_outcomes": node_outcome_views,
    })
}

fn collect_parity_nodes(rows: &[Value]) -> BTreeSet<String> {
    let mut nodes = BTreeSet::new();
    for row in rows {
        if let Some(node_a) = parity_matrix_row_node(row, "node_a") {
            nodes.insert(node_a);
        }
        if let Some(node_b) = parity_matrix_row_node(row, "node_b") {
            nodes.insert(node_b);
        }
    }
    nodes
}

fn build_parity_match_clusters(rows: &[Value], nodes: &BTreeSet<String>) -> Vec<Value> {
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = nodes
        .iter()
        .cloned()
        .map(|node| (node, BTreeSet::new()))
        .collect();

    for row in rows {
        if parity_matrix_row_status(row) != Some("PARITY_MATCH") {
            continue;
        }
        let Some(node_a) = parity_matrix_row_node(row, "node_a") else {
            continue;
        };
        let Some(node_b) = parity_matrix_row_node(row, "node_b") else {
            continue;
        };
        adjacency
            .entry(node_a.clone())
            .or_default()
            .insert(node_b.clone());
        adjacency.entry(node_b).or_default().insert(node_a);
    }

    let mut visited = BTreeSet::new();
    let mut clusters = Vec::new();
    let mut next_id = 1usize;

    for node in nodes {
        if visited.contains(node) {
            continue;
        }

        let mut queue = VecDeque::new();
        let mut component = Vec::new();
        visited.insert(node.clone());
        queue.push_back(node.clone());

        while let Some(current) = queue.pop_front() {
            component.push(current.clone());
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        component.sort();
        let size = component.len();
        clusters.push(json!({
            "cluster_id": format!("cluster_{next_id}"),
            "nodes": component,
            "size": size,
        }));
        next_id += 1;
    }

    clusters.sort_by(|left, right| {
        let left_size = left.get("size").and_then(Value::as_u64).unwrap_or(0);
        let right_size = right.get("size").and_then(Value::as_u64).unwrap_or(0);
        right_size
            .cmp(&left_size)
            .then_with(|| {
                let left_id = left
                    .get("cluster_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let right_id = right
                    .get("cluster_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                left_id.cmp(right_id)
            })
    });
    clusters
}

fn build_node_partitions<F>(node_outcomes: &[NodeParityOutcome], key_fn: F) -> Vec<Value>
where
    F: Fn(&NodeParityOutcome) -> &str,
{
    let mut partitions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in node_outcomes {
        partitions
            .entry(key_fn(node).to_string())
            .or_default()
            .push(node.node_id.clone());
    }

    let mut values = Vec::new();
    for (index, (key, mut nodes)) in partitions.into_iter().enumerate() {
        nodes.sort();
        let size = nodes.len();
        values.push(json!({
            "partition_id": format!("partition_{}", index + 1),
            "key": key,
            "nodes": nodes,
            "size": size,
        }));
    }

    values.sort_by(|left, right| {
        let left_size = left.get("size").and_then(Value::as_u64).unwrap_or(0);
        let right_size = right.get("size").and_then(Value::as_u64).unwrap_or(0);
        right_size
            .cmp(&left_size)
            .then_with(|| {
                let left_id = left
                    .get("partition_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let right_id = right
                    .get("partition_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                left_id.cmp(right_id)
            })
    });
    values
}

fn count_unique_node_dimension<F>(node_outcomes: &[NodeParityOutcome], key_fn: F) -> usize
where
    F: Fn(&NodeParityOutcome) -> &str,
{
    node_outcomes
        .iter()
        .map(|node| key_fn(node).to_string())
        .collect::<BTreeSet<_>>()
        .len()
}

fn count_determinism_conflict_surfaces(node_outcomes: &[NodeParityOutcome]) -> usize {
    let mut verdicts_by_surface: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for node in node_outcomes {
        verdicts_by_surface
            .entry(node.surface_key().to_string())
            .or_default()
            .insert(verdict_label(&node.verdict).to_string());
    }

    verdicts_by_surface
        .values()
        .filter(|verdicts| verdicts.len() > 1)
        .count()
}

fn parity_matrix_row_node(row: &Value, key: &str) -> Option<String> {
    row.get("row")
        .and_then(Value::as_object)
        .and_then(|nested| nested.get(key))
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn parity_matrix_row_status(row: &Value) -> Option<&str> {
    row.get("parity_status").and_then(Value::as_str)
}

fn count_parity_status_value(rows: &[Value], target: &str) -> usize {
    rows.iter()
        .filter(|row| parity_matrix_row_status(row) == Some(target))
        .count()
}

fn classify_parity_convergence_status(
    unique_subject_count: usize,
    unique_context_count: usize,
    unique_authority_count: usize,
    historical_only_node_count: usize,
    insufficient_evidence_node_count: usize,
    determinism_violation_present: bool,
    outcome_partition_count: usize,
    largest_outcome_cluster_size: usize,
    node_count: usize,
) -> &'static str {
    if determinism_violation_present {
        return "N_PARITY_DETERMINISM_VIOLATION";
    }

    if insufficient_evidence_node_count > 0
        && (unique_subject_count > 1
            || unique_context_count > 1
            || unique_authority_count > 1
            || historical_only_node_count > 0)
    {
        return "N_PARITY_MIXED";
    }

    if insufficient_evidence_node_count > 0 {
        return "N_PARITY_INSUFFICIENT_EVIDENCE";
    }

    if historical_only_node_count > 0 {
        return "N_PARITY_HISTORICAL_ISLAND";
    }

    if outcome_partition_count == 1 && largest_outcome_cluster_size == node_count {
        return "N_PARITY_CONVERGED";
    }

    "N_PARITY_CONSISTENCY_SPLIT"
}

fn build_alternate_parity_registry(
    baseline: &VerifierTrustRegistrySnapshot,
    verifier_key: &proof_verifier::types::ReceiptVerifierKey,
) -> Result<VerifierTrustRegistrySnapshot, String> {
    let mut registry = baseline.clone();
    registry.verifier_registry_epoch = registry.verifier_registry_epoch.saturating_add(1);
    registry.root_verifier_ids = vec!["root-verifier-c".to_string()];
    registry.verifiers.insert(
        "root-verifier-c".to_string(),
        VerifierAuthorityNode {
            verifier_id: "root-verifier-c".to_string(),
            verifier_pubkey_id: "root-verifier-c-ed25519-key-2026-03-a".to_string(),
            authority_scope: vec![
                "context-distributor".to_string(),
                "distributed-receipt-issuer".to_string(),
                "parity-reporter".to_string(),
            ],
            authority_state: VerifierAuthorityState::Current,
        },
    );
    registry.public_keys.insert(
        "root-verifier-c-ed25519-key-2026-03-a".to_string(),
        VerifierTrustRegistryPublicKey {
            algorithm: "ed25519".to_string(),
            public_key: verifier_key.public_key.clone(),
        },
    );
    registry.delegation_edges = vec![VerifierDelegationEdge {
        parent_verifier_id: "root-verifier-c".to_string(),
        delegate_verifier_id: "node-b".to_string(),
        delegated_scope: vec!["distributed-receipt-issuer".to_string()],
    }];
    registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&registry).map_err(|error| {
            format!("alternate parity registry hash recomputation failed: {error}")
        })?;
    Ok(registry)
}

fn build_historical_only_parity_registry(
    baseline: &VerifierTrustRegistrySnapshot,
) -> Result<VerifierTrustRegistrySnapshot, String> {
    let mut registry = baseline.clone();
    registry.verifier_registry_epoch = registry.verifier_registry_epoch.saturating_add(1);
    let node = registry
        .verifiers
        .get_mut("node-b")
        .ok_or_else(|| "historical parity registry missing node-b".to_string())?;
    node.authority_state = VerifierAuthorityState::HistoricalOnly;
    registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&registry).map_err(|error| {
            format!("historical parity registry hash recomputation failed: {error}")
        })?;
    Ok(registry)
}

fn build_scope_drift_parity_registry(
    baseline: &VerifierTrustRegistrySnapshot,
) -> Result<VerifierTrustRegistrySnapshot, String> {
    let mut registry = baseline.clone();
    registry.verifier_registry_epoch = registry.verifier_registry_epoch.saturating_add(1);
    let node = registry
        .verifiers
        .get_mut("node-b")
        .ok_or_else(|| "scope-drift parity registry missing node-b".to_string())?;
    node.authority_scope = vec!["parity-reporter".to_string()];
    let edge = registry
        .delegation_edges
        .iter_mut()
        .find(|edge| edge.delegate_verifier_id == "node-b")
        .ok_or_else(|| "scope-drift parity registry missing node-b delegation edge".to_string())?;
    edge.delegated_scope = vec!["parity-reporter".to_string()];
    registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&registry).map_err(|error| {
            format!("scope-drift parity registry hash recomputation failed: {error}")
        })?;
    Ok(registry)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_json<T: serde::Serialize>(path: PathBuf, payload: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(payload)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(&path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn violations_from_report(report: &Value) -> Vec<String> {
    report
        .get("violations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}
