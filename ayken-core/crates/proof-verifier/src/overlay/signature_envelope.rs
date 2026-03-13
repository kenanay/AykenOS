use crate::errors::VerifierRuntimeError;
use crate::types::SignatureEnvelope;
use std::fs;
use std::path::Path;

pub fn load_signature_envelope(path: &Path) -> Result<SignatureEnvelope, VerifierRuntimeError> {
    let bytes = fs::read(path)
        .map_err(|error| VerifierRuntimeError::io("read signature-envelope.json", error))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| VerifierRuntimeError::json("parse signature-envelope.json", error))
}
