use crate::errors::VerifierRuntimeError;
use crate::types::ProducerDeclaration;
use std::fs;
use std::path::Path;

pub fn load_producer(path: &Path) -> Result<ProducerDeclaration, VerifierRuntimeError> {
    let bytes =
        fs::read(path).map_err(|error| VerifierRuntimeError::io("read producer.json", error))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| VerifierRuntimeError::json("parse producer.json", error))
}
