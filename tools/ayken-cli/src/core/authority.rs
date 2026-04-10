use crate::core::error::AykenError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn fail_closed_policy(message: impl Into<String>) -> AykenError {
    AykenError::Policy(format!("{} (fail-closed)", message.into()))
}

pub fn load_json_file<T>(path: &Path, label: &str) -> Result<T, AykenError>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Err(fail_closed_policy(format!("{label} missing")));
    }

    let text = fs::read_to_string(path)
        .map_err(|err| fail_closed_policy(format!("failed to read {label}: {err}")))?;
    serde_json::from_str(&text)
        .map_err(|err| fail_closed_policy(format!("failed to parse {label}: {err}")))
}

pub fn sha256_hex_json<T>(value: &T) -> Result<String, AykenError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex_bytes(&bytes))
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}
