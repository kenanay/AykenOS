use crate::core::error::AykenError;
use serde::de::DeserializeOwned;
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
