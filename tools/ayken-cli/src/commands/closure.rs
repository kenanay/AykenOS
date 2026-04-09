use crate::cli::{ClosureArgs, ClosureTarget};
use crate::core::{error::AykenError, output};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[derive(Deserialize)]
struct ClosureIndex {
    closure_state: String,
    integrity_verified: bool,
    remote_ci_confirmation: RemoteCiConfirmation,
}

#[derive(Deserialize)]
struct RemoteCiConfirmation {
    run_id: String,
    head_sha: String,
    result: String,
}

#[derive(Serialize)]
struct ClosureStatus {
    base_path: &'static str,
    evidence_index_exists: bool,
    closure_manifest_exists: bool,
    closure_index_exists: bool,
    local_closure_ready: bool,
    official_closure_state: bool,
    integrity_verified: bool,
    remote_ci_run_id: Option<String>,
    remote_ci_pass: bool,
    git_head_sha: Option<String>,
    indexed_head_sha: Option<String>,
    head_sha_match: bool,
    authority_confirmed: bool,
    remote_authority_required: bool,
    evaluation_error: Option<String>,
    note: &'static str,
}

struct ClosureEvaluation {
    status: ClosureStatus,
    verify_error: Option<AykenError>,
}

pub fn run(args: ClosureArgs, json: bool) -> Result<(), AykenError> {
    let evaluation = evaluate_closure_status();
    match args.target {
        ClosureTarget::Status => emit_status("status", &evaluation.status, json),
        ClosureTarget::Verify => {
            emit_status("verify", &evaluation.status, json)?;
            match evaluation.verify_error {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }
}

fn evaluate_closure_status() -> ClosureEvaluation {
    let base = Path::new("reports/phase15_official_closure");
    let evidence = base.join("evidence_index.json").exists();
    let manifest = base.join("closure_manifest.json").exists();
    let index = base.join("closure_index.json").exists();
    let local_closure_ready = evidence && manifest && index;

    let mut status = ClosureStatus {
        base_path: "reports/phase15_official_closure",
        evidence_index_exists: evidence,
        closure_manifest_exists: manifest,
        closure_index_exists: index,
        local_closure_ready,
        official_closure_state: false,
        integrity_verified: false,
        remote_ci_run_id: None,
        remote_ci_pass: false,
        git_head_sha: None,
        indexed_head_sha: None,
        head_sha_match: false,
        authority_confirmed: false,
        remote_authority_required: true,
        evaluation_error: None,
        note: "Official closure requires OFFICIAL_CLOSURE_CONFIRMED + integrity_verified + remote GitHub Actions ci-freeze PASS + HEAD SHA match",
    };

    let closure_index = match load_closure_index(base) {
        Ok(index) => index,
        Err(error) => return evaluation_failure(status, error),
    };
    let git_head_sha = match read_git_head_sha() {
        Ok(sha) => sha,
        Err(error) => return evaluation_failure(status, error),
    };

    status.official_closure_state = closure_index.closure_state == "OFFICIAL_CLOSURE_CONFIRMED";
    status.integrity_verified = closure_index.integrity_verified;
    status.remote_ci_run_id = Some(closure_index.remote_ci_confirmation.run_id.clone());
    status.remote_ci_pass = closure_index.remote_ci_confirmation.result == "PASS";
    status.git_head_sha = Some(git_head_sha.clone());
    status.indexed_head_sha = Some(closure_index.remote_ci_confirmation.head_sha.clone());
    status.head_sha_match = git_head_sha == closure_index.remote_ci_confirmation.head_sha;
    status.authority_confirmed = status.local_closure_ready
        && status.official_closure_state
        && status.integrity_verified
        && status.remote_ci_pass
        && status.head_sha_match;

    let verify_error = (!status.authority_confirmed)
        .then(|| AykenError::Policy("closure authority not confirmed (fail-closed)".to_string()));

    ClosureEvaluation {
        status,
        verify_error,
    }
}

fn load_closure_index(base: &Path) -> Result<ClosureIndex, AykenError> {
    let closure_index_path = base.join("closure_index.json");
    if !closure_index_path.exists() {
        return Err(fail_closed_policy("closure_index.json missing"));
    }

    let closure_index_text = fs::read_to_string(&closure_index_path)
        .map_err(|err| fail_closed_policy(format!("failed to read closure_index.json: {err}")))?;
    serde_json::from_str(&closure_index_text)
        .map_err(|err| fail_closed_policy(format!("failed to parse closure_index.json: {err}")))
}

fn fail_closed_policy(message: impl Into<String>) -> AykenError {
    AykenError::Policy(format!("{} (fail-closed)", message.into()))
}

fn evaluation_failure(mut status: ClosureStatus, error: AykenError) -> ClosureEvaluation {
    status.evaluation_error = Some(error.to_string());
    ClosureEvaluation {
        status,
        verify_error: Some(error),
    }
}

fn emit_status(command: &str, status: &ClosureStatus, json: bool) -> Result<(), AykenError> {
    if json {
        output::print_json(status)
    } else {
        println!("ayken closure {command}");
        println!("  base_path            : {}", status.base_path);
        println!("  evidence_index.json  : {}", status.evidence_index_exists);
        println!(
            "  closure_manifest.json: {}",
            status.closure_manifest_exists
        );
        println!("  closure_index.json   : {}", status.closure_index_exists);
        println!("  local_closure_ready  : {}", status.local_closure_ready);
        println!("  official_closure     : {}", status.official_closure_state);
        println!("  integrity_verified   : {}", status.integrity_verified);
        println!(
            "  remote_ci_run_id     : {}",
            status.remote_ci_run_id.as_deref().unwrap_or("n/a")
        );
        println!("  remote_ci_pass       : {}", status.remote_ci_pass);
        println!(
            "  git_head_sha         : {}",
            status.git_head_sha.as_deref().unwrap_or("n/a")
        );
        println!(
            "  indexed_head_sha     : {}",
            status.indexed_head_sha.as_deref().unwrap_or("n/a")
        );
        println!("  head_sha_match       : {}", status.head_sha_match);
        println!("  authority_confirmed  : {}", status.authority_confirmed);
        println!("  remote_authority     : required");
        if let Some(error) = &status.evaluation_error {
            println!("  evaluation_error     : {error}");
        }
        println!("  note: {}", status.note);
        io::stdout().flush()?;
        Ok(())
    }
}

fn read_git_head_sha() -> Result<String, AykenError> {
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
