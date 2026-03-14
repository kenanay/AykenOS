use proof_verifier::trust_reuse_runtime_surface_emitter::{
    run_trust_reuse_runtime_surface_emitter, TrustReuseRuntimeSurfaceEmitterConfig,
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
    let config = match parse_cli(args)? {
        Some(config) => config,
        None => return Ok(()),
    };
    run_trust_reuse_runtime_surface_emitter(&config)?;
    Ok(())
}

fn parse_cli(args: Vec<OsString>) -> Result<Option<TrustReuseRuntimeSurfaceEmitterConfig>, String> {
    if args.is_empty() || contains_help_flag(&args) {
        print_usage();
        return Ok(None);
    }

    let mut args = args.into_iter();
    let mut input_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--input" => {
                input_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--input`".to_string())?,
                ));
            }
            "--output" => {
                output_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--output`".to_string())?,
                ));
            }
            "--output-dir" => {
                output_dir =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for `--output-dir`".to_string()
                    })?));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(Some(TrustReuseRuntimeSurfaceEmitterConfig {
        input_path: input_path.ok_or_else(|| "missing required `--input`".to_string())?,
        output_path: output_path.ok_or_else(|| "missing required `--output`".to_string())?,
        output_dir: output_dir.ok_or_else(|| "missing required `--output-dir`".to_string())?,
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
  trust-reuse-runtime-surface-emitter --input <path> --output <path> --output-dir <dir>

Purpose:
  Materialize canonical native trust-reuse runtime evidence for future Stage-2 sinkhole companion production.
"
    );
}
