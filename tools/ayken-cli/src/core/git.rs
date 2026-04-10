use crate::core::error::AykenError;
use std::process::Command;

pub fn read_git_head_sha() -> Result<String, AykenError> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AykenError::Process(format!(
            "`git rev-parse HEAD` exited with status {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    let head_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if head_sha.is_empty() {
        return Err(AykenError::Process(
            "`git rev-parse HEAD` returned an empty SHA".to_string(),
        ));
    }

    Ok(head_sha)
}

pub fn short_sha(head_sha: &str) -> Result<String, AykenError> {
    if head_sha.len() < 8 {
        return Err(AykenError::Process(format!(
            "git HEAD SHA too short for verified head lookup: {head_sha}"
        )));
    }

    Ok(head_sha[..8].to_string())
}
