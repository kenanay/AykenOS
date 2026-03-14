use proof_verifier::diversity_floor::GateVerdict;
use proof_verifier::diversity_ledger_producer::{
    run_diversity_ledger_producer, VerificationDiversityLedgerProducerConfig,
};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(verdict) => match verdict {
            GateVerdict::Pass => ExitCode::SUCCESS,
            GateVerdict::Fail => ExitCode::from(2),
        },
        Err(error) => {
            eprintln!("ERROR: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<GateVerdict, String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let command = match parse_cli(args)? {
        Some(command) => command,
        None => return Ok(GateVerdict::Pass),
    };
    let outcome = run_diversity_ledger_producer(&command)?;
    Ok(outcome.verdict)
}

fn parse_cli(
    args: Vec<OsString>,
) -> Result<Option<VerificationDiversityLedgerProducerConfig>, String> {
    if args.is_empty() || contains_help_flag(&args) {
        print_usage();
        return Ok(None);
    }

    let mut args = args.into_iter();
    let mut artifact_root: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut audit_ledger_path: Option<PathBuf> = None;
    let mut binding_path: Option<PathBuf> = None;
    let mut ledger_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--artifact-root" => {
                artifact_root =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--artifact-root`".to_string()
                    })?));
            }
            "--output-dir" => {
                output_dir =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--output-dir`".to_string()
                    })?));
            }
            "--audit-ledger" => {
                audit_ledger_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--audit-ledger`".to_string()
                    })?));
            }
            "--binding" => {
                binding_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--binding`".to_string()
                    })?));
            }
            "--ledger" => {
                ledger_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--ledger`".to_string())?,
                ));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let artifact_root =
        artifact_root.ok_or_else(|| "missing required `--artifact-root`".to_string())?;
    let output_dir = output_dir.ok_or_else(|| "missing required `--output-dir`".to_string())?;
    let audit_ledger_path =
        audit_ledger_path.unwrap_or_else(|| artifact_root.join("verification_audit_ledger.jsonl"));
    let binding_path = binding_path
        .unwrap_or_else(|| artifact_root.join("verification_diversity_ledger_binding.json"));
    let ledger_path =
        ledger_path.unwrap_or_else(|| artifact_root.join("verification_diversity_ledger.json"));

    Ok(Some(VerificationDiversityLedgerProducerConfig {
        audit_ledger_path,
        binding_path,
        ledger_path,
        output_dir,
    }))
}

fn contains_help_flag(args: &[OsString]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.to_string_lossy().as_ref(), "help" | "-h" | "--help"))
}

fn print_usage() {
    println!(
        "\
Usage:
  verification-diversity-ledger-producer --artifact-root <dir> --output-dir <dir> [--audit-ledger <path>] [--binding <path>] [--ledger <path>]

Purpose:
  Append canonical Verification Diversity Ledger entries from verifier audit evidence and node bindings.

Defaults:
  --audit-ledger defaults to <artifact-root>/verification_audit_ledger.jsonl
  --binding defaults to <artifact-root>/verification_diversity_ledger_binding.json
  --ledger defaults to <artifact-root>/verification_diversity_ledger.json
"
    );
}
