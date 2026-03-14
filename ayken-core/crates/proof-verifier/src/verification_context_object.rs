use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::{canonicalize_json, canonicalize_json_value};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationContextObject {
    pub context_version: u32,
    pub verification_context_id: String,
    pub policy_hash: String,
    pub registry_snapshot_hash: String,
    pub verifier_contract_version: String,
    pub context_rules_hash: String,
    #[serde(default)]
    pub context_epoch: Option<u64>,
    #[serde(default)]
    pub historical_cutoff_utc: Option<String>,
    #[serde(default)]
    pub policy_snapshot_ref: Option<String>,
    #[serde(default)]
    pub registry_snapshot_ref: Option<String>,
    #[serde(default)]
    pub time_semantics_mode: Option<String>,
}

pub fn load_verification_context_object(path: &Path) -> Result<VerificationContextObject, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read verification context object at {}: {error}",
            path.display()
        )
    })?;
    let context: VerificationContextObject = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse verification context object at {}: {error}",
            path.display()
        )
    })?;
    validate_verification_context_object(&context)?;
    Ok(context)
}

pub fn write_verification_context_object(
    path: &Path,
    context: &VerificationContextObject,
) -> Result<(), String> {
    validate_verification_context_object(context)?;
    let bytes = canonicalize_json(context).map_err(|error| {
        format!(
            "failed to canonicalize verification context object for {}: {error}",
            path.display()
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create verification context parent {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to write verification context object {}: {error}",
            path.display()
        )
    })
}

pub fn compute_verification_context_id(
    context: &VerificationContextObject,
) -> Result<String, String> {
    let mut context_value = serde_json::to_value(context).map_err(|error| {
        format!("failed to serialize verification context object for hashing: {error}")
    })?;
    if let Value::Object(map) = &mut context_value {
        map.remove("verification_context_id");
    }
    let bytes = canonicalize_json_value(&context_value).map_err(|error| {
        format!("failed to canonicalize verification context object for hashing: {error}")
    })?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

pub fn validate_verification_context_object(
    context: &VerificationContextObject,
) -> Result<(), String> {
    if context.context_version != 1 {
        return Err(format!(
            "unsupported context_version {} for verification context object",
            context.context_version
        ));
    }
    if !is_prefixed_sha256(&context.verification_context_id) {
        return Err(
            "verification_context_id must use sha256:<64-lowercase-hex> format".to_string(),
        );
    }
    for (label, value) in [
        ("policy_hash", context.policy_hash.as_str()),
        (
            "registry_snapshot_hash",
            context.registry_snapshot_hash.as_str(),
        ),
        ("context_rules_hash", context.context_rules_hash.as_str()),
    ] {
        if !is_lower_hex_digest(value) {
            return Err(format!(
                "{label} must be a 64-character lowercase SHA-256 hex digest"
            ));
        }
    }
    if context.verifier_contract_version.trim().is_empty() {
        return Err("verifier_contract_version must not be empty".to_string());
    }
    let recomputed = compute_verification_context_id(context)?;
    if context.verification_context_id != recomputed {
        return Err(
            "verification_context_id does not match canonical recomputed context identity"
                .to_string(),
        );
    }
    Ok(())
}

fn is_prefixed_sha256(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(is_lower_hex_digest)
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::{
        compute_verification_context_id, validate_verification_context_object,
        VerificationContextObject,
    };

    fn sample_context() -> VerificationContextObject {
        let mut context = VerificationContextObject {
            context_version: 1,
            verification_context_id: String::new(),
            policy_hash: "a".repeat(64),
            registry_snapshot_hash: "b".repeat(64),
            verifier_contract_version: "phase12-context-v1".to_string(),
            context_rules_hash: "c".repeat(64),
            context_epoch: Some(1),
            historical_cutoff_utc: None,
            policy_snapshot_ref: None,
            registry_snapshot_ref: None,
            time_semantics_mode: None,
        };
        context.verification_context_id =
            compute_verification_context_id(&context).expect("compute context id");
        context
    }

    #[test]
    fn verification_context_validation_accepts_canonical_object() {
        let context = sample_context();
        validate_verification_context_object(&context).expect("context should validate");
    }

    #[test]
    fn verification_context_validation_rejects_hash_drift() {
        let mut context = sample_context();
        context.policy_hash = "d".repeat(64);
        let error =
            validate_verification_context_object(&context).expect_err("context drift must fail");
        assert!(error.contains("verification_context_id"));
    }
}
