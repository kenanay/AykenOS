use crate::errors::VerifierRuntimeError;
use crate::types::ChecksumsFile;
use std::fs;
use std::path::Path;

pub fn load_checksums(path: &Path) -> Result<ChecksumsFile, VerifierRuntimeError> {
    let bytes =
        fs::read(path).map_err(|error| VerifierRuntimeError::io("read checksums.json", error))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| VerifierRuntimeError::json("parse checksums.json", error))
}
