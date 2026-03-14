use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::types::{ChecksumsFile, Manifest};
use serde_json::Value;

pub fn recompute_bundle_id(
    manifest: &Manifest,
    checksums: &ChecksumsFile,
) -> Result<String, VerifierRuntimeError> {
    let mut manifest_value = serde_json::to_value(manifest)
        .map_err(|error| VerifierRuntimeError::json("serialize manifest", error))?;
    if let Value::Object(map) = &mut manifest_value {
        map.remove("bundle_id");
    }

    let manifest_bytes = canonicalize_json_value(&manifest_value)?;
    let checksums_value = serde_json::to_value(checksums)
        .map_err(|error| VerifierRuntimeError::json("serialize checksums", error))?;
    let checksum_bytes = canonicalize_json_value(&checksums_value)?;

    let mut material = Vec::new();
    material.extend_from_slice(&manifest_bytes);
    material.extend_from_slice(&checksum_bytes);
    Ok(sha256_hex(&material))
}
