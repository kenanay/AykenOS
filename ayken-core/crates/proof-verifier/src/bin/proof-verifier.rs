use proof_verifier::types::{
    AuditMode, FindingSeverity, ReceiptMode, VerificationFinding, VerificationOutcome,
    VerificationVerdict, VerifyRequest,
};
use proof_verifier::{verify_bundle, RegistrySnapshot, TrustPolicy};
use serde::Serialize;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ERROR: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let command = match parse_cli(args)? {
        Some(command) => command,
        None => return Ok(()),
    };

    match command {
        ParsedCommand::VerifyBundle {
            bundle_path,
            policy_path,
            registry_path,
            json,
        } => run_verify_bundle(&bundle_path, &policy_path, &registry_path, json),
    }
}

enum ParsedCommand {
    VerifyBundle {
        bundle_path: PathBuf,
        policy_path: PathBuf,
        registry_path: PathBuf,
        json: bool,
    },
}

fn parse_cli(args: Vec<OsString>) -> Result<Option<ParsedCommand>, String> {
    if args.is_empty() || contains_help_flag(&args) {
        print_usage();
        return Ok(None);
    }

    let mut args = args.into_iter();
    let command = args
        .next()
        .ok_or_else(|| "missing command (expected `verify`)".to_string())?;
    match command.to_string_lossy().as_ref() {
        "verify" => parse_verify_command(args.collect()),
        other => Err(format!("unknown command: {other}")),
    }
}

fn parse_verify_command(args: Vec<OsString>) -> Result<Option<ParsedCommand>, String> {
    let mut args = args.into_iter();
    let target = args
        .next()
        .ok_or_else(|| "missing verify target (expected `bundle`)".to_string())?;
    match target.to_string_lossy().as_ref() {
        "bundle" => parse_verify_bundle_command(args.collect()).map(Some),
        other => Err(format!("unknown verify target: {other}")),
    }
}

fn parse_verify_bundle_command(args: Vec<OsString>) -> Result<ParsedCommand, String> {
    let mut args = args.into_iter();
    let bundle_path = args
        .next()
        .ok_or_else(|| "missing bundle path for `verify bundle`".to_string())?;

    let mut policy_path: Option<PathBuf> = None;
    let mut registry_path: Option<PathBuf> = None;
    let mut json = false;

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--policy" => {
                if policy_path.is_some() {
                    return Err("duplicate `--policy` flag".to_string());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for `--policy`".to_string())?;
                policy_path = Some(PathBuf::from(value));
            }
            "--registry" => {
                if registry_path.is_some() {
                    return Err("duplicate `--registry` flag".to_string());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for `--registry`".to_string())?;
                registry_path = Some(PathBuf::from(value));
            }
            "--json" => {
                json = true;
            }
            other => return Err(format!("unknown argument for `verify bundle`: {other}")),
        }
    }

    let policy_path = policy_path.ok_or_else(|| "missing required `--policy`".to_string())?;
    let registry_path = registry_path.ok_or_else(|| "missing required `--registry`".to_string())?;

    Ok(ParsedCommand::VerifyBundle {
        bundle_path: PathBuf::from(bundle_path),
        policy_path,
        registry_path,
        json,
    })
}

fn contains_help_flag(args: &[OsString]) -> bool {
    (args.len() == 1 && args[0].to_string_lossy().as_ref() == "help")
        || args
            .iter()
            .any(|arg| matches!(arg.to_string_lossy().as_ref(), "-h" | "--help"))
}

fn run_verify_bundle(
    bundle_path: &Path,
    policy_path: &Path,
    registry_path: &Path,
    json: bool,
) -> Result<(), String> {
    let policy = load_json_file::<TrustPolicy>(policy_path, "policy")?;
    let registry = load_json_file::<RegistrySnapshot>(registry_path, "registry snapshot")?;

    let request = VerifyRequest {
        bundle_path,
        policy: &policy,
        registry_snapshot: &registry,
        receipt_mode: ReceiptMode::None,
        receipt_signer: None,
        audit_mode: AuditMode::None,
        audit_ledger_path: None,
    };
    let outcome =
        verify_bundle(&request).map_err(|error| format!("runtime verification failed: {error}"))?;

    if json {
        let payload = CliVerificationOutput::from_outcome(&outcome);
        let bytes = serde_json::to_vec_pretty(&payload)
            .map_err(|error| format!("failed to serialize CLI JSON output: {error}"))?;
        println!("{}", String::from_utf8_lossy(&bytes));
    } else {
        print_human_readable(&outcome);
    }

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

fn print_human_readable(outcome: &VerificationOutcome) {
    println!("Verdict: {}", verdict_label(&outcome.verdict));
    println!("Bundle ID: {}", outcome.subject.bundle_id);
    println!("Trust Overlay Hash: {}", outcome.subject.trust_overlay_hash);
    println!("Policy Hash: {}", outcome.subject.policy_hash);
    println!(
        "Registry Snapshot Hash: {}",
        outcome.subject.registry_snapshot_hash
    );
    println!("Findings: {}", outcome.findings.len());

    for finding in &outcome.findings {
        println!(
            "Finding [{}] {}: {}",
            severity_label(&finding.severity),
            finding.code,
            finding.message
        );
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

fn severity_label(severity: &FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Info => "INFO",
        FindingSeverity::Warning => "WARNING",
        FindingSeverity::Error => "ERROR",
    }
}

fn print_usage() {
    println!(
        "\
Usage:
  proof-verifier verify bundle <bundle_path> --policy <policy.json> --registry <registry.json> [--json]

Commands:
  verify bundle    Verify a proof bundle with external policy and registry inputs

Options:
  --policy <path>    Path to trust policy JSON
  --registry <path>  Path to producer registry snapshot JSON
  --json             Emit machine-readable JSON output
  -h, --help         Show this help
"
    );
}

#[derive(Serialize)]
struct CliVerificationOutput {
    verdict: String,
    bundle_id: String,
    trust_overlay_hash: String,
    policy_hash: String,
    registry_snapshot_hash: String,
    findings_count: usize,
    findings: Vec<CliFindingOutput>,
}

impl CliVerificationOutput {
    fn from_outcome(outcome: &VerificationOutcome) -> Self {
        Self {
            verdict: verdict_label(&outcome.verdict).to_string(),
            bundle_id: outcome.subject.bundle_id.clone(),
            trust_overlay_hash: outcome.subject.trust_overlay_hash.clone(),
            policy_hash: outcome.subject.policy_hash.clone(),
            registry_snapshot_hash: outcome.subject.registry_snapshot_hash.clone(),
            findings_count: outcome.findings.len(),
            findings: outcome
                .findings
                .iter()
                .map(CliFindingOutput::from_finding)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct CliFindingOutput {
    code: String,
    message: String,
    severity: String,
    deterministic: bool,
}

impl CliFindingOutput {
    fn from_finding(finding: &VerificationFinding) -> Self {
        Self {
            code: finding.code.clone(),
            message: finding.message.clone(),
            severity: severity_label(&finding.severity).to_string(),
            deterministic: finding.deterministic,
        }
    }
}
