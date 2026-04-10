use crate::core::error::AykenError;
use std::env;
use std::process::Command;

pub fn read_git_head_sha() -> Result<String, AykenError> {
    for key in ["GITHUB_SHA", "CI_COMMIT_SHA"] {
        if let Ok(value) = env::var(key) {
            let sha = value.trim();
            if is_full_hex_sha(sha) {
                return Ok(sha.to_string());
            }
        }
    }

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

fn is_full_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
