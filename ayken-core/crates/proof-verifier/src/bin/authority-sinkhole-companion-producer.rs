use proof_verifier::authority_sinkhole_companion_producer::{
    run_authority_sinkhole_companion_producer, AuthoritySinkholeCompanionProducerConfig,
};
use proof_verifier::diversity_floor::GateVerdict;
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
    let outcome = run_authority_sinkhole_companion_producer(&command)?;
    Ok(outcome.verdict)
}

fn parse_cli(
    args: Vec<OsString>,
) -> Result<Option<AuthoritySinkholeCompanionProducerConfig>, String> {
    if args.is_empty() || contains_help_flag(&args) {
        print_usage();
        return Ok(None);
    }

    let mut args = args.into_iter();
    let mut artifact_root: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut replay_source_path: Option<PathBuf> = None;
    let mut trust_source_path: Option<PathBuf> = None;
    let mut replay_output_path: Option<PathBuf> = None;
    let mut trust_output_path: Option<PathBuf> = None;

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
            "--replay-source" => {
                replay_source_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--replay-source`".to_string()
                    })?));
            }
            "--trust-source" => {
                trust_source_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--trust-source`".to_string()
                    })?));
            }
            "--replay-output" => {
                replay_output_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--replay-output`".to_string()
                    })?));
            }
            "--trust-output" => {
                trust_output_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--trust-output`".to_string()
                    })?));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let artifact_root =
        artifact_root.ok_or_else(|| "missing required `--artifact-root`".to_string())?;
    let output_dir = output_dir.ok_or_else(|| "missing required `--output-dir`".to_string())?;

    Ok(Some(AuthoritySinkholeCompanionProducerConfig {
        replay_source_path: replay_source_path
            .unwrap_or_else(|| artifact_root.join("replay_boundary_flow_source.json")),
        trust_source_path: trust_source_path
            .unwrap_or_else(|| artifact_root.join("trust_reuse_flow_source.json")),
        replay_output_path: replay_output_path
            .unwrap_or_else(|| artifact_root.join("replay_boundary_flow_report.json")),
        trust_output_path: trust_output_path
            .unwrap_or_else(|| artifact_root.join("trust_reuse_flow_report.json")),
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
  authority-sinkhole-companion-producer --artifact-root <dir> --output-dir <dir> [--replay-source <path>] [--trust-source <path>] [--replay-output <path>] [--trust-output <path>]

Purpose:
  Materialize canonical Stage-2 replay-boundary and trust-reuse companion flow reports for authority sinkhole analysis.

Defaults:
  --replay-source defaults to <artifact-root>/replay_boundary_flow_source.json
  --trust-source defaults to <artifact-root>/trust_reuse_flow_source.json
  --replay-output defaults to <artifact-root>/replay_boundary_flow_report.json
  --trust-output defaults to <artifact-root>/trust_reuse_flow_report.json
"
    );
}
