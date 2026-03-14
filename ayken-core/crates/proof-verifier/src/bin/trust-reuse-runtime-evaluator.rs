use proof_verifier::trust_reuse_runtime_evaluator::{
    run_trust_reuse_runtime_evaluator, TrustReuseRuntimeEvaluatorConfig,
};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
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
    let Some(config) = parse_cli(args)? else {
        return Ok(());
    };
    run_trust_reuse_runtime_evaluator(&config)?;
    Ok(())
}

fn parse_cli(args: Vec<OsString>) -> Result<Option<TrustReuseRuntimeEvaluatorConfig>, String> {
    if args.is_empty() || contains_help_flag(&args) {
        print_usage();
        return Ok(None);
    }

    let mut receipt_path: Option<PathBuf> = None;
    let mut verifier_key_path: Option<PathBuf> = None;
    let mut expected_subject_path: Option<PathBuf> = None;
    let mut verification_context_path: Option<PathBuf> = None;
    let mut verifier_attestation_path: Option<PathBuf> = None;
    let mut verifier_registry_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut run_id: Option<String> = None;
    let mut timestamp_unix_ns: Option<u64> = None;
    let mut source_run_id: Option<String> = None;
    let mut execution_cluster_id: Option<String> = None;
    let mut lineage_id: Option<String> = None;
    let mut reuse_group_id: Option<String> = None;
    let mut surface_local_path_id: Option<String> = None;
    let mut trust_reuse_source: Option<String> = None;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--receipt" => {
                receipt_path = Some(PathBuf::from(next_value(&mut args, "--receipt")?));
            }
            "--verifier-key" => {
                verifier_key_path = Some(PathBuf::from(next_value(&mut args, "--verifier-key")?));
            }
            "--expected-subject" => {
                expected_subject_path =
                    Some(PathBuf::from(next_value(&mut args, "--expected-subject")?));
            }
            "--verification-context" => {
                verification_context_path = Some(PathBuf::from(next_value(
                    &mut args,
                    "--verification-context",
                )?));
            }
            "--verifier-attestation" => {
                verifier_attestation_path = Some(PathBuf::from(next_value(
                    &mut args,
                    "--verifier-attestation",
                )?));
            }
            "--verifier-registry" => {
                verifier_registry_path =
                    Some(PathBuf::from(next_value(&mut args, "--verifier-registry")?));
            }
            "--output" => {
                output_path = Some(PathBuf::from(next_value(&mut args, "--output")?));
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(next_value(&mut args, "--output-dir")?));
            }
            "--run-id" => {
                run_id = Some(
                    next_value(&mut args, "--run-id")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--timestamp-unix-ns" => {
                timestamp_unix_ns = Some(
                    next_value(&mut args, "--timestamp-unix-ns")?
                        .to_string_lossy()
                        .parse::<u64>()
                        .map_err(|error| {
                            format!("invalid value for `--timestamp-unix-ns`: {error}")
                        })?,
                );
            }
            "--source-run-id" => {
                source_run_id = Some(
                    next_value(&mut args, "--source-run-id")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--execution-cluster-id" => {
                execution_cluster_id = Some(
                    next_value(&mut args, "--execution-cluster-id")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--lineage-id" => {
                lineage_id = Some(
                    next_value(&mut args, "--lineage-id")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--reuse-group-id" => {
                reuse_group_id = Some(
                    next_value(&mut args, "--reuse-group-id")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--surface-local-path-id" => {
                surface_local_path_id = Some(
                    next_value(&mut args, "--surface-local-path-id")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--trust-reuse-source" => {
                trust_reuse_source = Some(
                    next_value(&mut args, "--trust-reuse-source")?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            other => {
                return Err(format!(
                    "unknown argument for `trust-reuse-runtime-evaluator`: {other}"
                ));
            }
        }
    }

    Ok(Some(TrustReuseRuntimeEvaluatorConfig {
        receipt_path: receipt_path.ok_or_else(|| "missing required `--receipt`".to_string())?,
        verifier_key_path: verifier_key_path
            .ok_or_else(|| "missing required `--verifier-key`".to_string())?,
        expected_subject_path: expected_subject_path
            .ok_or_else(|| "missing required `--expected-subject`".to_string())?,
        verification_context_path: verification_context_path
            .ok_or_else(|| "missing required `--verification-context`".to_string())?,
        verifier_attestation_path: verifier_attestation_path
            .ok_or_else(|| "missing required `--verifier-attestation`".to_string())?,
        verifier_registry_path: verifier_registry_path
            .ok_or_else(|| "missing required `--verifier-registry`".to_string())?,
        output_path: output_path.ok_or_else(|| "missing required `--output`".to_string())?,
        output_dir: output_dir.ok_or_else(|| "missing required `--output-dir`".to_string())?,
        run_id: run_id.ok_or_else(|| "missing required `--run-id`".to_string())?,
        timestamp_unix_ns: timestamp_unix_ns
            .ok_or_else(|| "missing required `--timestamp-unix-ns`".to_string())?,
        source_run_id,
        execution_cluster_id,
        lineage_id,
        reuse_group_id,
        surface_local_path_id,
        trust_reuse_source,
    }))
}

fn next_value(args: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<OsString, String> {
    args.next()
        .ok_or_else(|| format!("missing value for `{flag}`"))
}

fn contains_help_flag(args: &[OsString]) -> bool {
    (args.len() == 1 && args[0].to_string_lossy().as_ref() == "help")
        || args
            .iter()
            .any(|arg| matches!(arg.to_string_lossy().as_ref(), "-h" | "--help"))
}

fn print_usage() {
    println!(
        "\
Usage:
  trust-reuse-runtime-evaluator \\
    --receipt <receipt.json> \\
    --verifier-key <receipt_verifier_key.json> \\
    --expected-subject <verdict_subject.json> \\
    --verification-context <verification_context_object.json> \\
    --verifier-attestation <verifier_attestation.json> \\
    --verifier-registry <verifier_registry.json> \\
    --run-id <run_id> \\
    --timestamp-unix-ns <unix_ns> \\
    --output <trust_reuse_runtime_surface.json> \\
    --output-dir <artifact_dir> \\
    [--source-run-id <run_id>] \\
    [--execution-cluster-id <id>] \\
    [--lineage-id <id>] \\
    [--reuse-group-id <id>] \\
    [--surface-local-path-id <id>] \\
    [--trust-reuse-source <source>]

Emits:
  trust_reuse_runtime_surface.json
  trust_reuse_runtime_surface_evaluate_report.json
  violations.txt
"
    );
}
