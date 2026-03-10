use crate::canonical::digest::sha256_hex;
use crate::errors::VerifierRuntimeError;
use crate::types::{ChecksumsFile, LoadedBundle, VerificationFinding};
use std::fs;

pub fn validate_portable_checksums(
    bundle: &LoadedBundle,
    checksums: &ChecksumsFile,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let mut findings = Vec::new();

    if checksums.algorithm != "sha256" {
        findings.push(VerificationFinding::error(
            "PV0200",
            "checksums.json uses unsupported digest algorithm",
        ));
    }

    for (relative_path, expected_digest) in &checksums.files {
        let full_path = bundle.root.join(relative_path);
        if !full_path.exists() {
            findings.push(VerificationFinding::error(
                "PV0201",
                format!("checksummed file missing from bundle: {relative_path}"),
            ));
            continue;
        }

        let bytes = fs::read(&full_path).map_err(|error| {
            VerifierRuntimeError::io(format!("read checksummed file {relative_path}"), error)
        })?;
        let actual_digest = sha256_hex(&bytes);
        if actual_digest != *expected_digest {
            findings.push(VerificationFinding::error(
                "PV0202",
                format!("checksum mismatch for {relative_path}"),
            ));
        }
    }

    Ok(findings)
}
