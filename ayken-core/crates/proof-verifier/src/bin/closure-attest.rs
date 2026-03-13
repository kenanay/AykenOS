use proof_verifier::canonical::jcs::canonicalize_json_bytes;
use proof_verifier::crypto::{sign_ed25519_bytes, verify_ed25519_bytes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::ffi::OsString;
use std::fs;
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
    let command = match parse_cli(args)? {
        Some(command) => command,
        None => return Ok(()),
    };

    match command {
        ParsedCommand::SignJson {
            payload_path,
            output_path,
            attestor_node_id,
            attestor_key_id,
            private_key,
            attested_at_utc,
        } => run_sign_json(
            &payload_path,
            &output_path,
            &attestor_node_id,
            &attestor_key_id,
            &private_key,
            &attested_at_utc,
        ),
        ParsedCommand::VerifyJson {
            payload_path,
            attestation_path,
            public_key,
        } => run_verify_json(&payload_path, &attestation_path, &public_key),
    }
}

enum ParsedCommand {
    SignJson {
        payload_path: PathBuf,
        output_path: PathBuf,
        attestor_node_id: String,
        attestor_key_id: String,
        private_key: String,
        attested_at_utc: String,
    },
    VerifyJson {
        payload_path: PathBuf,
        attestation_path: PathBuf,
        public_key: String,
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
        .ok_or_else(|| "missing command (expected `sign-json`)".to_string())?;
    match command.to_string_lossy().as_ref() {
        "sign-json" => parse_sign_json_command(args.collect()).map(Some),
        "verify-json" => parse_verify_json_command(args.collect()).map(Some),
        other => Err(format!("unknown command: {other}")),
    }
}

fn parse_sign_json_command(args: Vec<OsString>) -> Result<ParsedCommand, String> {
    let mut args = args.into_iter();
    let mut payload_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut attestor_node_id: Option<String> = None;
    let mut attestor_key_id: Option<String> = None;
    let mut private_key: Option<String> = None;
    let mut attested_at_utc: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--payload" => {
                payload_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--payload`".to_string())?,
                ));
            }
            "--output" => {
                output_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--output`".to_string())?,
                ));
            }
            "--attestor-node-id" => {
                attestor_node_id = Some(
                    args.next()
                        .ok_or_else(|| "missing value for `--attestor-node-id`".to_string())?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--attestor-key-id" => {
                attestor_key_id = Some(
                    args.next()
                        .ok_or_else(|| "missing value for `--attestor-key-id`".to_string())?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--private-key" => {
                private_key = Some(
                    args.next()
                        .ok_or_else(|| "missing value for `--private-key`".to_string())?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            "--attested-at-utc" => {
                attested_at_utc = Some(
                    args.next()
                        .ok_or_else(|| "missing value for `--attested-at-utc`".to_string())?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            other => return Err(format!("unknown argument for `sign-json`: {other}")),
        }
    }

    Ok(ParsedCommand::SignJson {
        payload_path: payload_path.ok_or_else(|| "missing required `--payload`".to_string())?,
        output_path: output_path.ok_or_else(|| "missing required `--output`".to_string())?,
        attestor_node_id: attestor_node_id
            .ok_or_else(|| "missing required `--attestor-node-id`".to_string())?,
        attestor_key_id: attestor_key_id
            .ok_or_else(|| "missing required `--attestor-key-id`".to_string())?,
        private_key: private_key.ok_or_else(|| "missing required `--private-key`".to_string())?,
        attested_at_utc: attested_at_utc
            .ok_or_else(|| "missing required `--attested-at-utc`".to_string())?,
    })
}

fn parse_verify_json_command(args: Vec<OsString>) -> Result<ParsedCommand, String> {
    let mut args = args.into_iter();
    let mut payload_path: Option<PathBuf> = None;
    let mut attestation_path: Option<PathBuf> = None;
    let mut public_key: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--payload" => {
                payload_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--payload`".to_string())?,
                ));
            }
            "--attestation" => {
                attestation_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for `--attestation`".to_string())?,
                ));
            }
            "--public-key" => {
                public_key = Some(
                    args.next()
                        .ok_or_else(|| "missing value for `--public-key`".to_string())?
                        .to_string_lossy()
                        .to_string(),
                );
            }
            other => return Err(format!("unknown argument for `verify-json`: {other}")),
        }
    }

    Ok(ParsedCommand::VerifyJson {
        payload_path: payload_path.ok_or_else(|| "missing required `--payload`".to_string())?,
        attestation_path: attestation_path
            .ok_or_else(|| "missing required `--attestation`".to_string())?,
        public_key: public_key.ok_or_else(|| "missing required `--public-key`".to_string())?,
    })
}

fn contains_help_flag(args: &[OsString]) -> bool {
    (args.len() == 1 && args[0].to_string_lossy().as_ref() == "help")
        || args
            .iter()
            .any(|arg| matches!(arg.to_string_lossy().as_ref(), "-h" | "--help"))
}

fn run_sign_json(
    payload_path: &PathBuf,
    output_path: &PathBuf,
    attestor_node_id: &str,
    attestor_key_id: &str,
    private_key: &str,
    attested_at_utc: &str,
) -> Result<(), String> {
    let payload_bytes = fs::read(payload_path)
        .map_err(|error| format!("failed to read payload at {}: {error}", payload_path.display()))?;
    let canonical_payload = canonicalize_json_bytes(&payload_bytes)
        .map_err(|error| format!("failed to canonicalize payload: {error}"))?;
    let signature = sign_ed25519_bytes(private_key, &canonical_payload)
        .map_err(|error| format!("failed to sign payload: {error}"))?;

    let attestation = ClosureManifestAttestation {
        attestation_version: 1,
        artifact_kind: "phase12_closure_manifest".to_string(),
        payload_sha256: sha256_hex(&canonical_payload),
        attestor_node_id: attestor_node_id.to_string(),
        attestor_key_id: attestor_key_id.to_string(),
        signature_algorithm: "ed25519".to_string(),
        attested_at_utc: attested_at_utc.to_string(),
        signature,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create output dir {}: {error}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(&attestation)
        .map_err(|error| format!("failed to serialize attestation: {error}"))?;
    fs::write(output_path, bytes)
        .map_err(|error| format!("failed to write attestation {}: {error}", output_path.display()))?;
    Ok(())
}

fn run_verify_json(
    payload_path: &PathBuf,
    attestation_path: &PathBuf,
    public_key: &str,
) -> Result<(), String> {
    let payload_bytes = fs::read(payload_path)
        .map_err(|error| format!("failed to read payload at {}: {error}", payload_path.display()))?;
    let canonical_payload = canonicalize_json_bytes(&payload_bytes)
        .map_err(|error| format!("failed to canonicalize payload: {error}"))?;
    let attestation_bytes = fs::read(attestation_path).map_err(|error| {
        format!(
            "failed to read attestation at {}: {error}",
            attestation_path.display()
        )
    })?;
    let attestation: ClosureManifestAttestation =
        serde_json::from_slice(&attestation_bytes).map_err(|error| {
            format!(
                "failed to deserialize attestation {}: {error}",
                attestation_path.display()
            )
        })?;

    if attestation.attestation_version != 1 {
        return Err(format!(
            "unsupported attestation_version: {}",
            attestation.attestation_version
        ));
    }
    if attestation.artifact_kind != "phase12_closure_manifest" {
        return Err(format!(
            "unsupported artifact_kind: {}",
            attestation.artifact_kind
        ));
    }
    if !attestation
        .signature_algorithm
        .eq_ignore_ascii_case("ed25519")
    {
        return Err(format!(
            "unsupported signature_algorithm: {}",
            attestation.signature_algorithm
        ));
    }

    let payload_sha256 = sha256_hex(&canonical_payload);
    if attestation.payload_sha256 != payload_sha256 {
        return Err(format!(
            "payload_sha256 mismatch: attestation={}, computed={payload_sha256}",
            attestation.payload_sha256
        ));
    }

    verify_ed25519_bytes(
        public_key,
        &attestation.signature,
        &canonical_payload,
        "PV9901",
        "closure manifest detached signature verification failed",
    )
    .map_err(|finding| format!("{}: {}", finding.code, finding.message))?;
    println!("OK: detached attestation verified");
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn print_usage() {
    println!(
        "\
Usage:
  closure-attest sign-json --payload <payload.json> --output <attestation.json> --attestor-node-id <id> --attestor-key-id <id> --private-key <base64:...> --attested-at-utc <timestamp>
  closure-attest verify-json --payload <payload.json> --attestation <attestation.json> --public-key <base64:...>

Commands:
  sign-json   Canonicalize JSON payload and emit detached Ed25519 attestation
  verify-json Verify detached Ed25519 attestation against canonical JSON payload

Options:
  --payload <path>            Path to JSON payload to canonicalize and sign
  --output <path>             Output path for detached attestation JSON
  --attestation <path>        Detached attestation JSON to verify
  --public-key <base64>       Ed25519 public key material for verification
  --attestor-node-id <id>     Logical attestor node identifier
  --attestor-key-id <id>      Attestor key identifier
  --private-key <base64>      Ed25519 signing key material (base64:...)
  --attested-at-utc <ts>      Attestation timestamp
  -h, --help                  Show this help
"
    );
}

#[derive(Deserialize, Serialize)]
struct ClosureManifestAttestation {
    attestation_version: u32,
    artifact_kind: String,
    payload_sha256: String,
    attestor_node_id: String,
    attestor_key_id: String,
    signature_algorithm: String,
    attested_at_utc: String,
    signature: String,
}
