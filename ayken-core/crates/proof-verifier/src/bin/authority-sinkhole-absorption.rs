use proof_verifier::authority_sinkhole_absorption::{
    run_authority_sinkhole_absorption_gate, AuthoritySinkholeAbsorptionGateConfig,
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
    let outcome = run_authority_sinkhole_absorption_gate(&command)?;
    Ok(outcome.verdict)
}

fn parse_cli(args: Vec<OsString>) -> Result<Option<AuthoritySinkholeAbsorptionGateConfig>, String> {
    if args.is_empty() || contains_help_flag(&args) {
        print_usage();
        return Ok(None);
    }

    let mut args = args.into_iter();
    let mut artifact_root: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut ledger_path: Option<PathBuf> = None;
    let mut policy_path: Option<PathBuf> = None;
    let mut replay_boundary_flow_path: Option<PathBuf> = None;
    let mut trust_reuse_flow_path: Option<PathBuf> = None;
    let mut window_runs_override: Option<usize> = None;
    let mut window_seconds_override: Option<u64> = None;

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
            "--ledger" => {
                ledger_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--ledger`".to_string())?,
                ));
            }
            "--policy" => {
                policy_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--policy`".to_string())?,
                ));
            }
            "--replay-flow" => {
                replay_boundary_flow_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--replay-flow`".to_string()
                    })?));
            }
            "--trust-reuse-flow" => {
                trust_reuse_flow_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--trust-reuse-flow`".to_string()
                    })?));
            }
            "--window-runs" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for `--window-runs`".to_string())?;
                window_runs_override = Some(
                    value
                        .to_string_lossy()
                        .parse::<usize>()
                        .map_err(|error| format!("invalid `--window-runs` value: {error}"))?,
                );
            }
            "--window-seconds" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for `--window-seconds`".to_string())?;
                window_seconds_override = Some(
                    value
                        .to_string_lossy()
                        .parse::<u64>()
                        .map_err(|error| format!("invalid `--window-seconds` value: {error}"))?,
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let artifact_root =
        artifact_root.ok_or_else(|| "missing required `--artifact-root`".to_string())?;
    let output_dir = output_dir.ok_or_else(|| "missing required `--output-dir`".to_string())?;
    let ledger_path =
        ledger_path.unwrap_or_else(|| artifact_root.join("verification_diversity_ledger.json"));
    let policy_path =
        policy_path.unwrap_or_else(|| artifact_root.join("authority_sinkhole_policy.json"));
    let replay_boundary_flow_path = replay_boundary_flow_path
        .unwrap_or_else(|| artifact_root.join("replay_boundary_flow_report.json"));
    let trust_reuse_flow_path =
        trust_reuse_flow_path.unwrap_or_else(|| artifact_root.join("trust_reuse_flow_report.json"));

    Ok(Some(AuthoritySinkholeAbsorptionGateConfig {
        ledger_path,
        policy_path,
        replay_boundary_flow_path,
        trust_reuse_flow_path,
        output_dir,
        window_runs_override,
        window_seconds_override,
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
  authority-sinkhole-absorption --artifact-root <dir> --output-dir <dir> [--ledger <path>] [--policy <path>] [--replay-flow <path>] [--trust-reuse-flow <path>] [--window-runs <n>] [--window-seconds <seconds>]

Purpose:
  Evaluate Verification Diversity Ledger evidence against authority sinkhole absorption policy.

Defaults:
  --ledger defaults to <artifact-root>/verification_diversity_ledger.json
  --policy defaults to <artifact-root>/authority_sinkhole_policy.json
  --replay-flow defaults to <artifact-root>/replay_boundary_flow_report.json
  --trust-reuse-flow defaults to <artifact-root>/trust_reuse_flow_report.json
"
    );
}
