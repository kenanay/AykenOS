use crate::cli::ClosureArgs;
use crate::core::{error::AykenError, output};
use serde::{Deserialize, Serialize};
use std::fs;
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
    note: &'static str,
}

pub fn run(args: ClosureArgs, json: bool) -> Result<(), AykenError> {
    if args.target != "status" {
        return Err(AykenError::Policy(format!(
            "unsupported closure sub-command: `{}`. Valid: status",
            args.target
        )));
    }

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
        note: "Official closure requires OFFICIAL_CLOSURE_CONFIRMED + integrity_verified + remote GitHub Actions ci-freeze PASS + HEAD SHA match",
    };

    if !index {
        emit_status(&status, json)?;
        return Err(AykenError::Policy(
            "closure_index.json missing (fail-closed)".to_string(),
        ));
    }

    let closure_index_path = base.join("closure_index.json");
    let closure_index_text = fs::read_to_string(&closure_index_path).map_err(|err| {
        AykenError::Policy(format!(
            "failed to read closure_index.json (fail-closed): {err}"
        ))
    })?;
    let closure_index: ClosureIndex = serde_json::from_str(&closure_index_text).map_err(|err| {
        AykenError::Policy(format!(
            "failed to parse closure_index.json (fail-closed): {err}"
        ))
    })?;
    let git_head_sha = read_git_head_sha()?;

    status.official_closure_state =
        closure_index.closure_state == "OFFICIAL_CLOSURE_CONFIRMED";
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

    emit_status(&status, json)?;
    if !status.authority_confirmed {
        Err(AykenError::Policy(
            "closure authority not confirmed (fail-closed)".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn emit_status(status: &ClosureStatus, json: bool) -> Result<(), AykenError> {
    if json {
        output::print_json(status)
    } else {
        println!("ayken closure status");
        println!("  base_path            : {}", status.base_path);
        println!("  evidence_index.json  : {}", status.evidence_index_exists);
        println!("  closure_manifest.json: {}", status.closure_manifest_exists);
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
        println!("  note: {}", status.note);
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
