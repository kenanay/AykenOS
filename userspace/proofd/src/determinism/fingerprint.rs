use crate::{encode_lower_hex, ServiceError};
use proof_verifier::types::VerificationFinding;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::canonical_json::{canonicalize, canonicalize_value};

fn sha256_prefixed(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", encode_lower_hex(&hasher.finalize()))
}

pub(crate) fn canonical_hash_prefixed<T>(
    value: &T,
    error_code: &'static str,
) -> Result<String, ServiceError>
where
    T: Serialize,
{
    let bytes = canonicalize(value).map_err(|_| ServiceError::Runtime(error_code))?;
    Ok(sha256_prefixed(&bytes))
}

pub(crate) fn canonical_hash_value_prefixed(
    value: &Value,
    error_code: &'static str,
) -> Result<String, ServiceError> {
    let bytes = canonicalize_value(value).map_err(|_| ServiceError::Runtime(error_code))?;
    Ok(sha256_prefixed(&bytes))
}

pub(crate) fn canonical_hash_findings_prefixed(
    findings: &[VerificationFinding],
    error_code: &'static str,
) -> Result<String, ServiceError> {
    let value = Value::Array(
        findings
            .iter()
            .map(|finding| {
                json!({
                    "code": finding.code,
                    "message": finding.message,
                    "severity": format!("{:?}", finding.severity),
                    "deterministic": finding.deterministic,
                })
            })
            .collect(),
    );
    canonical_hash_value_prefixed(&value, error_code)
}
