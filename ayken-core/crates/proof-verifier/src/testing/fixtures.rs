use crate::authority::snapshot::compute_verifier_trust_registry_snapshot_hash;
use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use crate::portable_core::identity::recompute_bundle_id;
use crate::registry::snapshot::compute_registry_snapshot_hash;
use crate::types::DetachedSignature;
use crate::types::{
    ChecksumsFile, Manifest, ProducerDeclaration, ReceiptSignerConfig, ReceiptVerifierKey,
    RegistryEntry, RegistryPublicKey, RegistrySnapshot, SignatureEnvelope, SignatureRequirement,
    TrustPolicy, VerifierAuthorityNode, VerifierAuthorityState, VerifierDelegationEdge,
    VerifierTrustRegistryPublicKey, VerifierTrustRegistrySnapshot,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct FixtureBundle {
    pub root: PathBuf,
    pub policy: TrustPolicy,
    pub registry: RegistrySnapshot,
    pub verifier_registry: VerifierTrustRegistrySnapshot,
    pub receipt_signer: ReceiptSignerConfig,
    pub receipt_verifier_key: ReceiptVerifierKey,
    pub authority_requested_verifier_id: String,
    pub authority_requested_scope: Vec<String>,
}

pub fn create_fixture_bundle() -> FixtureBundle {
    let root = unique_fixture_root();
    let evidence_dir = root.join("evidence");
    let traces_dir = root.join("traces");
    let reports_dir = root.join("reports");
    let meta_dir = root.join("meta");
    let producer_dir = root.join("producer");
    let signatures_dir = root.join("signatures");

    fs::create_dir_all(&evidence_dir).unwrap();
    fs::create_dir_all(&traces_dir).unwrap();
    fs::create_dir_all(&reports_dir).unwrap();
    fs::create_dir_all(&meta_dir).unwrap();
    fs::create_dir_all(&producer_dir).unwrap();
    fs::create_dir_all(&signatures_dir).unwrap();

    write_text(
        &evidence_dir.join("abdf_snapshot_hash.txt"),
        &("a".repeat(64) + "\n"),
    );
    write_text(
        &evidence_dir.join("bcib_plan_hash.txt"),
        &("b".repeat(64) + "\n"),
    );
    write_text(
        &evidence_dir.join("decision_ledger.jsonl"),
        "{\"event_seq\":1,\"ltick\":1}\n",
    );
    write_text(
        &evidence_dir.join("eti_transcript.jsonl"),
        "{\"event_seq\":1,\"ltick\":1,\"event_type\":\"AY_EVT_SYSCALL_ENTER\"}\n",
    );
    write_bytes(&evidence_dir.join("kernel.elf"), b"KERNEL");

    write_text(
        &traces_dir.join("execution_trace.jsonl"),
        "{\"event_seq\":1,\"ltick\":1,\"event_type\":\"AY_EVT_SYSCALL_ENTER\"}\n",
    );
    write_text(
        &traces_dir.join("replay_trace.jsonl"),
        "{\"event_seq\":1,\"ltick\":1,\"event_type\":\"AY_EVT_SYSCALL_ENTER\"}\n",
    );
    write_text(&meta_dir.join("run.json"), "{\"run_id\":\"fixture-run\"}\n");

    let execution_trace_hash = sha256_hex(
        &fs::read(traces_dir.join("execution_trace.jsonl")).expect("execution trace should exist"),
    );
    let replay_trace_hash = sha256_hex(
        &fs::read(traces_dir.join("replay_trace.jsonl")).expect("replay trace should exist"),
    );
    write_text(
        &evidence_dir.join("execution_trace_hash.txt"),
        &(execution_trace_hash.clone() + "\n"),
    );
    write_text(
        &evidence_dir.join("replay_trace_hash.txt"),
        &(replay_trace_hash.clone() + "\n"),
    );

    let replay_result_hash = sha256_hex(
        format!(
            "{}|{}|{}",
            "a".repeat(64),
            "b".repeat(64),
            execution_trace_hash
        )
        .as_bytes(),
    );
    let final_state_hash = sha256_hex(b"fixture-final-state");
    write_json(
        &reports_dir.join("replay_report.json"),
        &json!({
            "status": "PASS",
            "replay_execution_trace_hash": replay_trace_hash,
            "replay_result_hash": replay_result_hash,
            "final_state_hash": final_state_hash,
            "replay_event_count": 1u64,
            "violations_count": 0u64
        }),
    );
    write_json(
        &reports_dir.join("proof_verify.json"),
        &json!({"status":"PASS"}),
    );
    write_json(&reports_dir.join("report.json"), &json!({"verdict":"PASS"}));
    write_json(
        &reports_dir.join("summary.json"),
        &json!({"verdict":"PASS"}),
    );
    write_json(
        &reports_dir.join("proof_manifest.json"),
        &build_fixture_proof_manifest(
            &evidence_dir,
            &meta_dir.join("run.json"),
            &reports_dir.join("replay_report.json"),
            &traces_dir.join("execution_trace.jsonl"),
        ),
    );

    let producer = ProducerDeclaration {
        metadata_version: 1,
        producer_id: "ayken-ci".to_string(),
        producer_pubkey_id: "ed25519-key-2026-03-a".to_string(),
        producer_registry_ref: "trust://registry/ayken-ci".to_string(),
        producer_key_epoch: "2026-03".to_string(),
        build_id: Some("build-fe9031d7".to_string()),
    };

    let required_files = vec![
        "evidence/abdf_snapshot_hash.txt".to_string(),
        "evidence/bcib_plan_hash.txt".to_string(),
        "evidence/decision_ledger.jsonl".to_string(),
        "evidence/eti_transcript.jsonl".to_string(),
        "evidence/execution_trace_hash.txt".to_string(),
        "evidence/kernel.elf".to_string(),
        "evidence/replay_trace_hash.txt".to_string(),
        "traces/execution_trace.jsonl".to_string(),
        "traces/replay_trace.jsonl".to_string(),
        "reports/proof_manifest.json".to_string(),
        "reports/proof_verify.json".to_string(),
        "reports/replay_report.json".to_string(),
        "reports/report.json".to_string(),
        "reports/summary.json".to_string(),
        "meta/run.json".to_string(),
    ];
    let checksums = ChecksumsFile {
        algorithm: "sha256".to_string(),
        bundle_version: 2,
        files: checksum_map(&root, &required_files),
    };
    let mut manifest = Manifest {
        bundle_id: String::new(),
        bundle_version: 2,
        checksums_file: "checksums.json".to_string(),
        compatibility_mode: Some("phase11-portable-core".to_string()),
        mode: Some("portable_proof_bundle_v2".to_string()),
        required_files,
    };
    manifest.bundle_id = recompute_bundle_id(&manifest, &checksums).unwrap();

    let signing_key = SigningKey::from_bytes(&fixture_secret_key_bytes());
    let verifying_key = signing_key.verifying_key();
    let detached_signature = signing_key.sign(manifest.bundle_id.as_bytes());
    let signature_envelope = SignatureEnvelope {
        envelope_version: 1,
        bundle_id: manifest.bundle_id.clone(),
        bundle_id_algorithm: "sha256".to_string(),
        signatures: vec![DetachedSignature {
            signer_id: "ayken-ci".to_string(),
            producer_pubkey_id: "ed25519-key-2026-03-a".to_string(),
            signature_algorithm: "ed25519".to_string(),
            signature: format!("base64:{}", STANDARD.encode(detached_signature.to_bytes())),
            signed_at_utc: "2026-03-07T10:33:00Z".to_string(),
        }],
    };

    write_json(&root.join("checksums.json"), &checksums);
    write_json(&root.join("manifest.json"), &manifest);
    write_json(&producer_dir.join("producer.json"), &producer);
    write_json(
        &signatures_dir.join("signature-envelope.json"),
        &signature_envelope,
    );

    let policy = TrustPolicy {
        policy_version: 1,
        policy_hash: None,
        quorum_policy_ref: Some("policy://quorum/at-least-1-of-n".to_string()),
        trusted_producers: vec!["ayken-ci".to_string()],
        trusted_pubkey_ids: vec!["ed25519-key-2026-03-a".to_string()],
        required_signatures: Some(SignatureRequirement {
            kind: "at_least".to_string(),
            count: 1,
        }),
        revoked_pubkey_ids: Vec::new(),
    };

    let mut producers = BTreeMap::new();
    producers.insert(
        "ayken-ci".to_string(),
        RegistryEntry {
            active_pubkey_ids: vec!["ed25519-key-2026-03-a".to_string()],
            revoked_pubkey_ids: Vec::new(),
            superseded_pubkey_ids: Vec::new(),
            public_keys: BTreeMap::from([(
                "ed25519-key-2026-03-a".to_string(),
                RegistryPublicKey {
                    algorithm: "ed25519".to_string(),
                    public_key: format!("base64:{}", STANDARD.encode(verifying_key.as_bytes())),
                },
            )]),
        },
    );
    let mut registry = RegistrySnapshot {
        registry_format_version: 1,
        registry_version: 1,
        registry_snapshot_hash: String::new(),
        producers,
    };
    registry.registry_snapshot_hash = compute_registry_snapshot_hash(&registry).unwrap();

    let receipt_signing_key = SigningKey::from_bytes(&fixture_receipt_secret_key_bytes());
    let receipt_verifying_key = receipt_signing_key.verifying_key();
    let receipt_signer = ReceiptSignerConfig {
        verifier_node_id: "node-b".to_string(),
        verifier_key_id: "receipt-ed25519-key-2026-03-a".to_string(),
        signature_algorithm: "ed25519".to_string(),
        private_key: format!("base64:{}", STANDARD.encode(receipt_signing_key.to_bytes())),
        verified_at_utc: "2026-03-08T12:00:00Z".to_string(),
    };
    let receipt_verifier_key = ReceiptVerifierKey {
        verifier_node_id: "node-b".to_string(),
        verifier_key_id: "receipt-ed25519-key-2026-03-a".to_string(),
        signature_algorithm: "ed25519".to_string(),
        public_key: format!(
            "base64:{}",
            STANDARD.encode(receipt_verifying_key.as_bytes())
        ),
    };
    let root_verifier_signing_key =
        SigningKey::from_bytes(&fixture_root_receipt_secret_key_bytes());
    let root_verifier_verifying_key = root_verifier_signing_key.verifying_key();
    let mut verifier_registry = VerifierTrustRegistrySnapshot {
        registry_format_version: 1,
        verifier_registry_snapshot_hash: String::new(),
        verifier_registry_parent_hash: "genesis".to_string(),
        verifier_registry_epoch: 1,
        registry_scope: "verifier-trust/main".to_string(),
        root_verifier_ids: vec!["root-verifier-a".to_string()],
        verifiers: BTreeMap::from([
            (
                "root-verifier-a".to_string(),
                VerifierAuthorityNode {
                    verifier_id: "root-verifier-a".to_string(),
                    verifier_pubkey_id: "root-verifier-ed25519-key-2026-03-a".to_string(),
                    authority_scope: vec![
                        "context-distributor".to_string(),
                        "distributed-receipt-issuer".to_string(),
                        "parity-reporter".to_string(),
                    ],
                    authority_state: VerifierAuthorityState::Current,
                },
            ),
            (
                "node-b".to_string(),
                VerifierAuthorityNode {
                    verifier_id: "node-b".to_string(),
                    verifier_pubkey_id: "receipt-ed25519-key-2026-03-a".to_string(),
                    authority_scope: vec!["distributed-receipt-issuer".to_string()],
                    authority_state: VerifierAuthorityState::Current,
                },
            ),
        ]),
        public_keys: BTreeMap::from([
            (
                "root-verifier-ed25519-key-2026-03-a".to_string(),
                VerifierTrustRegistryPublicKey {
                    algorithm: "ed25519".to_string(),
                    public_key: format!(
                        "base64:{}",
                        STANDARD.encode(root_verifier_verifying_key.as_bytes())
                    ),
                },
            ),
            (
                "receipt-ed25519-key-2026-03-a".to_string(),
                VerifierTrustRegistryPublicKey {
                    algorithm: "ed25519".to_string(),
                    public_key: format!(
                        "base64:{}",
                        STANDARD.encode(receipt_verifying_key.as_bytes())
                    ),
                },
            ),
        ]),
        delegation_edges: vec![VerifierDelegationEdge {
            parent_verifier_id: "root-verifier-a".to_string(),
            delegate_verifier_id: "node-b".to_string(),
            delegated_scope: vec!["distributed-receipt-issuer".to_string()],
        }],
    };
    verifier_registry.verifier_registry_snapshot_hash =
        compute_verifier_trust_registry_snapshot_hash(&verifier_registry).unwrap();

    FixtureBundle {
        root,
        policy,
        registry,
        verifier_registry,
        receipt_signer,
        receipt_verifier_key,
        authority_requested_verifier_id: "node-b".to_string(),
        authority_requested_scope: vec!["distributed-receipt-issuer".to_string()],
    }
}

fn checksum_map(root: &Path, required_files: &[String]) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for relative_path in required_files {
        let digest = sha256_hex(&fs::read(root.join(relative_path)).unwrap());
        files.insert(relative_path.clone(), digest);
    }
    files
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) {
    let bytes = canonicalize_json(value).unwrap();
    fs::write(path, bytes).unwrap();
}

fn write_text(path: &Path, value: &str) {
    fs::write(path, value.as_bytes()).unwrap();
}

fn write_bytes(path: &Path, value: &[u8]) {
    fs::write(path, value).unwrap();
}

fn unique_fixture_root() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "proof-verifier-fixture-{}-{}-{}",
        std::process::id(),
        nanos,
        counter
    ));
    path
}

fn fixture_secret_key_bytes() -> [u8; 32] {
    [7u8; 32]
}

fn fixture_receipt_secret_key_bytes() -> [u8; 32] {
    [11u8; 32]
}

fn fixture_root_receipt_secret_key_bytes() -> [u8; 32] {
    [13u8; 32]
}

fn build_fixture_proof_manifest(
    evidence_dir: &Path,
    run_json_path: &Path,
    replay_report_path: &Path,
    execution_trace_path: &Path,
) -> serde_json::Value {
    let abdf_snapshot_hash =
        first_hash_token(&fs::read_to_string(evidence_dir.join("abdf_snapshot_hash.txt")).unwrap());
    let bcib_plan_hash =
        first_hash_token(&fs::read_to_string(evidence_dir.join("bcib_plan_hash.txt")).unwrap());
    let execution_trace_hash = first_hash_token(
        &fs::read_to_string(evidence_dir.join("execution_trace_hash.txt")).unwrap(),
    );
    let ledger_root_hash =
        sha256_hex(&fs::read(evidence_dir.join("decision_ledger.jsonl")).unwrap());
    let transcript_root_hash =
        sha256_hex(&fs::read(evidence_dir.join("eti_transcript.jsonl")).unwrap());
    let kernel_image_hash = sha256_hex(&fs::read(evidence_dir.join("kernel.elf")).unwrap());
    let config_hash = sha256_hex(&fs::read(run_json_path).unwrap());
    let replay_report: serde_json::Value =
        serde_json::from_slice(&fs::read(replay_report_path).unwrap()).unwrap();
    let event_count = count_nonempty_lines(&fs::read(execution_trace_path).unwrap()) as u64;
    let violation_count = replay_report["violations_count"].as_u64().unwrap_or(0);

    let mut manifest = json!({
        "manifest_version": 1u32,
        "mode": "bootstrap_kpl_proof_manifest",
        "signature_mode": "bootstrap-none",
        "signer_sig": "",
        "hash_algorithm": "sha256",
        "kernel_image_hash": kernel_image_hash,
        "config_hash": config_hash,
        "ledger_root_hash": ledger_root_hash,
        "transcript_root_hash": transcript_root_hash,
        "abdf_snapshot_hash": abdf_snapshot_hash,
        "bcib_plan_hash": bcib_plan_hash,
        "execution_trace_hash": execution_trace_hash,
        "replay_result_hash": replay_report["replay_result_hash"].as_str().unwrap_or_default(),
        "final_state_hash": replay_report["final_state_hash"].as_str().unwrap_or_default(),
        "event_count": event_count,
        "violation_count": violation_count
    });
    let proof_hash = recompute_fixture_proof_hash(&manifest);
    manifest["proof_hash"] = serde_json::Value::String(proof_hash);
    manifest
}

fn recompute_fixture_proof_hash(proof_manifest: &serde_json::Value) -> String {
    let mut value = proof_manifest.clone();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("proof_hash");
    }
    let bytes = canonicalize_json_value(&value).unwrap();
    sha256_hex(&bytes)
}

fn first_hash_token(raw: &str) -> String {
    raw.lines()
        .find_map(|line| {
            let token = line.split_whitespace().next()?.trim().to_ascii_lowercase();
            if token.is_empty() {
                None
            } else {
                Some(token)
            }
        })
        .unwrap_or_default()
}

fn count_nonempty_lines(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
}
