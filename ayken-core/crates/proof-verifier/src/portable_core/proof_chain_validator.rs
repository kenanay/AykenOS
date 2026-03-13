use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::types::{LoadedBundle, VerificationFinding};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn validate_proof_chain(
    bundle: &LoadedBundle,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let mut findings = Vec::new();

    let proof_manifest_path = bundle.reports_dir.join("proof_manifest.json");
    let proof_verify_path = bundle.reports_dir.join("proof_verify.json");
    let replay_report_path = bundle.reports_dir.join("replay_report.json");
    let report_path = bundle.reports_dir.join("report.json");
    let summary_path = bundle.reports_dir.join("summary.json");

    let proof_manifest = load_json_struct::<ProofManifest>(
        &proof_manifest_path,
        "reports/proof_manifest.json",
        "PV0204",
        &mut findings,
    )?;
    let proof_verify = load_json_struct::<ProofVerifyStatus>(
        &proof_verify_path,
        "reports/proof_verify.json",
        "PV0205",
        &mut findings,
    )?;
    let replay_report = load_json_struct::<ReplayReport>(
        &replay_report_path,
        "reports/replay_report.json",
        "PV0207",
        &mut findings,
    )?;
    let report = load_json_struct::<VerdictReport>(
        &report_path,
        "reports/report.json",
        "PV0208",
        &mut findings,
    )?;
    let summary = load_json_struct::<VerdictReport>(
        &summary_path,
        "reports/summary.json",
        "PV0209",
        &mut findings,
    )?;

    let Some(proof_manifest) = proof_manifest else {
        return Ok(findings);
    };

    if proof_manifest.manifest_version != 1 {
        findings.push(VerificationFinding::error(
            "PV0210",
            "reports/proof_manifest.json uses unsupported manifest_version",
        ));
    }
    if proof_manifest.mode.trim().is_empty() {
        findings.push(VerificationFinding::error(
            "PV0211",
            "reports/proof_manifest.json is missing mode",
        ));
    } else if proof_manifest.mode != "bootstrap_kpl_proof_manifest" {
        findings.push(VerificationFinding::error(
            "PV0245",
            "reports/proof_manifest.json mode is not bootstrap_kpl_proof_manifest",
        ));
    }
    if proof_manifest.signature_mode.trim().is_empty() {
        findings.push(VerificationFinding::error(
            "PV0212",
            "reports/proof_manifest.json is missing signature_mode",
        ));
    } else if proof_manifest.signature_mode != "bootstrap-none" {
        findings.push(VerificationFinding::error(
            "PV0246",
            "reports/proof_manifest.json signature_mode is not bootstrap-none",
        ));
    }
    if proof_manifest.signature_mode == "bootstrap-none" && !proof_manifest.signer_sig.is_empty() {
        findings.push(VerificationFinding::error(
            "PV0247",
            "reports/proof_manifest.json signer_sig must be empty when signature_mode is bootstrap-none",
        ));
    }
    if proof_manifest.hash_algorithm != "sha256" {
        findings.push(VerificationFinding::error(
            "PV0213",
            "reports/proof_manifest.json uses unsupported hash_algorithm",
        ));
    }
    validate_manifest_hash_fields(&proof_manifest, &mut findings);

    let proof_hash_recomputed = recompute_proof_hash(&proof_manifest)?;
    if proof_hash_recomputed != proof_manifest.proof_hash {
        findings.push(VerificationFinding::error(
            "PV0214",
            "reports/proof_manifest.json proof_hash does not match recomputed manifest hash",
        ));
    }

    let Some(abdf_snapshot_hash) = load_hash_artifact(
        &bundle.evidence_dir.join("abdf_snapshot_hash.txt"),
        "evidence/abdf_snapshot_hash.txt",
        "PV0215",
        "PV0216",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(bcib_plan_hash) = load_hash_artifact(
        &bundle.evidence_dir.join("bcib_plan_hash.txt"),
        "evidence/bcib_plan_hash.txt",
        "PV0217",
        "PV0218",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(execution_trace_hash_from_evidence) = load_hash_artifact(
        &bundle.evidence_dir.join("execution_trace_hash.txt"),
        "evidence/execution_trace_hash.txt",
        "PV0219",
        "PV0220",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(replay_trace_hash_from_evidence) = load_hash_artifact(
        &bundle.evidence_dir.join("replay_trace_hash.txt"),
        "evidence/replay_trace_hash.txt",
        "PV0248",
        "PV0249",
        &mut findings,
    ) else {
        return Ok(findings);
    };

    let Some(decision_ledger_bytes) = read_required_bytes(
        &bundle.evidence_dir.join("decision_ledger.jsonl"),
        "evidence/decision_ledger.jsonl",
        "PV0221",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(eti_transcript_bytes) = read_required_bytes(
        &bundle.evidence_dir.join("eti_transcript.jsonl"),
        "evidence/eti_transcript.jsonl",
        "PV0222",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(kernel_image_bytes) = read_required_bytes(
        &bundle.evidence_dir.join("kernel.elf"),
        "evidence/kernel.elf",
        "PV0223",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(config_bytes) = read_required_bytes(
        &bundle.meta_run_path,
        "meta/run.json",
        "PV0224",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(execution_trace_bytes) = read_required_bytes(
        &bundle.traces_dir.join("execution_trace.jsonl"),
        "traces/execution_trace.jsonl",
        "PV0225",
        &mut findings,
    ) else {
        return Ok(findings);
    };
    let Some(replay_trace_bytes) = read_required_bytes(
        &bundle.traces_dir.join("replay_trace.jsonl"),
        "traces/replay_trace.jsonl",
        "PV0226",
        &mut findings,
    ) else {
        return Ok(findings);
    };

    let ledger_root_hash = sha256_hex(&decision_ledger_bytes);
    let transcript_root_hash = sha256_hex(&eti_transcript_bytes);
    let kernel_image_hash = sha256_hex(&kernel_image_bytes);
    let config_hash = sha256_hex(&config_bytes);
    let execution_trace_hash = sha256_hex(&execution_trace_bytes);
    let replay_trace_hash = sha256_hex(&replay_trace_bytes);

    compare_hash_binding(
        &mut findings,
        "PV0227",
        "proof_manifest abdf_snapshot_hash does not match evidence hash artifact",
        &proof_manifest.abdf_snapshot_hash,
        &abdf_snapshot_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0228",
        "proof_manifest bcib_plan_hash does not match evidence hash artifact",
        &proof_manifest.bcib_plan_hash,
        &bcib_plan_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0229",
        "execution_trace hash artifact does not match recomputed execution trace hash",
        &execution_trace_hash_from_evidence,
        &execution_trace_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0230",
        "proof_manifest execution_trace_hash does not match recomputed execution trace hash",
        &proof_manifest.execution_trace_hash,
        &execution_trace_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0231",
        "proof_manifest ledger_root_hash does not match recomputed decision ledger hash",
        &proof_manifest.ledger_root_hash,
        &ledger_root_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0232",
        "proof_manifest transcript_root_hash does not match recomputed ETI transcript hash",
        &proof_manifest.transcript_root_hash,
        &transcript_root_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0233",
        "proof_manifest kernel_image_hash does not match bundled kernel image hash",
        &proof_manifest.kernel_image_hash,
        &kernel_image_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0234",
        "proof_manifest config_hash does not match bundled meta/run.json hash",
        &proof_manifest.config_hash,
        &config_hash,
    );
    compare_hash_binding(
        &mut findings,
        "PV0250",
        "replay_trace hash artifact does not match recomputed replay trace hash",
        &replay_trace_hash_from_evidence,
        &replay_trace_hash,
    );

    if let Some(proof_verify) = proof_verify {
        if proof_verify.status != "PASS" {
            findings.push(VerificationFinding::error(
                "PV0235",
                "reports/proof_verify.json status is not PASS",
            ));
        }
    }

    if let Some(replay_report) = replay_report {
        if replay_report.status != "PASS" {
            findings.push(VerificationFinding::error(
                "PV0236",
                "reports/replay_report.json status is not PASS",
            ));
        }
        compare_hash_binding(
            &mut findings,
            "PV0237",
            "proof_manifest replay_result_hash does not match replay_report binding",
            &proof_manifest.replay_result_hash,
            &replay_report.replay_result_hash,
        );
        compare_hash_binding(
            &mut findings,
            "PV0238",
            "proof_manifest final_state_hash does not match replay_report binding",
            &proof_manifest.final_state_hash,
            &replay_report.final_state_hash,
        );
        compare_hash_binding(
            &mut findings,
            "PV0251",
            "reports/replay_report.json replay_execution_trace_hash does not match recomputed replay trace hash",
            &replay_report.replay_execution_trace_hash,
            &replay_trace_hash,
        );
        if proof_manifest.event_count != replay_report.replay_event_count {
            findings.push(VerificationFinding::error(
                "PV0239",
                "proof_manifest event_count does not match replay_report replay_event_count",
            ));
        }
        if proof_manifest.violation_count != replay_report.violations_count {
            findings.push(VerificationFinding::error(
                "PV0240",
                "proof_manifest violation_count does not match replay_report violations_count",
            ));
        }

        let execution_trace_count = count_nonempty_lines(&execution_trace_bytes);
        let replay_trace_count = count_nonempty_lines(&replay_trace_bytes);
        if replay_report.replay_event_count != execution_trace_count
            || replay_report.replay_event_count != replay_trace_count
        {
            findings.push(VerificationFinding::error(
                "PV0241",
                "replay_report replay_event_count does not match bundled trace counts",
            ));
        }
    }

    if let Some(report) = report {
        if report.verdict != "PASS" {
            findings.push(VerificationFinding::error(
                "PV0242",
                "reports/report.json verdict is not PASS",
            ));
        }
        if let Some(summary) = &summary {
            if report.verdict != summary.verdict {
                findings.push(VerificationFinding::error(
                    "PV0243",
                    "reports/report.json and reports/summary.json verdicts diverge",
                ));
            }
        }
    }

    if let Some(summary) = summary {
        if summary.verdict != "PASS" {
            findings.push(VerificationFinding::error(
                "PV0244",
                "reports/summary.json verdict is not PASS",
            ));
        }
    }

    Ok(findings)
}

#[derive(Debug, Deserialize, Serialize)]
struct ProofManifest {
    manifest_version: u32,
    mode: String,
    signature_mode: String,
    signer_sig: String,
    hash_algorithm: String,
    kernel_image_hash: String,
    config_hash: String,
    ledger_root_hash: String,
    transcript_root_hash: String,
    abdf_snapshot_hash: String,
    bcib_plan_hash: String,
    execution_trace_hash: String,
    replay_result_hash: String,
    final_state_hash: String,
    event_count: u64,
    violation_count: u64,
    proof_hash: String,
}

#[derive(Debug, Deserialize)]
struct ProofVerifyStatus {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ReplayReport {
    status: String,
    replay_execution_trace_hash: String,
    replay_result_hash: String,
    final_state_hash: String,
    replay_event_count: u64,
    violations_count: u64,
}

#[derive(Debug, Deserialize)]
struct VerdictReport {
    verdict: String,
}

fn load_json_struct<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &str,
    error_code: &str,
    findings: &mut Vec<VerificationFinding>,
) -> Result<Option<T>, VerifierRuntimeError> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes =
        fs::read(path).map_err(|error| VerifierRuntimeError::io(format!("read {label}"), error))?;
    match serde_json::from_slice(&bytes) {
        Ok(value) => Ok(Some(value)),
        Err(_) => {
            findings.push(VerificationFinding::error(
                error_code,
                format!("{label} is malformed or missing required fields"),
            ));
            Ok(None)
        }
    }
}

fn load_hash_artifact(
    path: &Path,
    label: &str,
    missing_code: &str,
    invalid_code: &str,
    findings: &mut Vec<VerificationFinding>,
) -> Option<String> {
    let bytes = read_required_bytes(path, label, missing_code, findings)?;
    let value = normalize_hash_text(&String::from_utf8_lossy(&bytes));
    if !is_sha256_hex(&value) {
        findings.push(VerificationFinding::error(
            invalid_code,
            format!("{label} does not contain a valid SHA-256 hex digest"),
        ));
        return None;
    }
    Some(value)
}

fn read_required_bytes(
    path: &Path,
    label: &str,
    missing_code: &str,
    findings: &mut Vec<VerificationFinding>,
) -> Option<Vec<u8>> {
    match fs::read(path) {
        Ok(bytes) if !bytes.is_empty() => Some(bytes),
        _ => {
            findings.push(VerificationFinding::error(
                missing_code,
                format!("{label} is missing or empty"),
            ));
            None
        }
    }
}

fn normalize_hash_text(raw: &str) -> String {
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

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn compare_hash_binding(
    findings: &mut Vec<VerificationFinding>,
    error_code: &str,
    message: &str,
    expected: &str,
    actual: &str,
) {
    if !expected.eq_ignore_ascii_case(actual) {
        findings.push(VerificationFinding::error(error_code, message));
    }
}

fn validate_manifest_hash_fields(
    proof_manifest: &ProofManifest,
    findings: &mut Vec<VerificationFinding>,
) {
    for (field_name, value) in [
        (
            "kernel_image_hash",
            proof_manifest.kernel_image_hash.as_str(),
        ),
        ("config_hash", proof_manifest.config_hash.as_str()),
        ("ledger_root_hash", proof_manifest.ledger_root_hash.as_str()),
        (
            "transcript_root_hash",
            proof_manifest.transcript_root_hash.as_str(),
        ),
        (
            "abdf_snapshot_hash",
            proof_manifest.abdf_snapshot_hash.as_str(),
        ),
        ("bcib_plan_hash", proof_manifest.bcib_plan_hash.as_str()),
        (
            "execution_trace_hash",
            proof_manifest.execution_trace_hash.as_str(),
        ),
        (
            "replay_result_hash",
            proof_manifest.replay_result_hash.as_str(),
        ),
        ("final_state_hash", proof_manifest.final_state_hash.as_str()),
        ("proof_hash", proof_manifest.proof_hash.as_str()),
    ] {
        if !is_sha256_hex(value) {
            findings.push(VerificationFinding::error(
                "PV0252",
                format!(
                    "reports/proof_manifest.json {field_name} must be a 64-character SHA-256 hex digest"
                ),
            ));
        }
    }
}

fn count_nonempty_lines(bytes: &[u8]) -> u64 {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count() as u64
}

fn recompute_proof_hash(proof_manifest: &ProofManifest) -> Result<String, VerifierRuntimeError> {
    let mut proof_manifest_value = serde_json::to_value(proof_manifest).map_err(|error| {
        VerifierRuntimeError::json("serialize reports/proof_manifest.json", error)
    })?;
    if let Value::Object(map) = &mut proof_manifest_value {
        map.remove("proof_hash");
    }
    let bytes = canonicalize_json_value(&proof_manifest_value)?;
    Ok(sha256_hex(&bytes))
}
