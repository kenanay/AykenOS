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

pub fn list_first_parent_ancestors(
    head_sha: &str,
    max_depth: usize,
) -> Result<Vec<String>, AykenError> {
    let max_count = (max_depth + 1).to_string();
    let output = run_git_output(&[
        "rev-list",
        "--first-parent",
        "--max-count",
        &max_count,
        head_sha,
    ])?;
    let mut commits = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if !commits.is_empty() {
        commits.remove(0);
    }
    Ok(commits)
}

pub fn is_git_worktree_dirty() -> Result<bool, AykenError> {
    let output = run_git_output(&["status", "--porcelain"])?;
    Ok(!output.trim().is_empty())
}

fn is_full_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn run_git_output(args: &[&str]) -> Result<String, AykenError> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AykenError::Process(format!(
            "`git {}` exited with status {}: {}",
            args.join(" "),
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
