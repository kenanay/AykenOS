use crate::errors::VerifierRuntimeError;
use crate::types::Manifest;
use std::fs;
use std::path::Path;

pub fn load_manifest(path: &Path) -> Result<Manifest, VerifierRuntimeError> {
    let bytes =
        fs::read(path).map_err(|error| VerifierRuntimeError::io("read manifest.json", error))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| VerifierRuntimeError::json("parse manifest.json", error))
}
