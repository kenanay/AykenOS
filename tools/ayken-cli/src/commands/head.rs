use crate::cli::{HeadArgs, HeadTarget};
use crate::core::{
    authority::{fail_closed_policy, load_json_file},
    error::AykenError,
    git::{read_git_head_sha, short_sha},
    output,
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERIFIED_HEADS_BASE: &str = "reports/verified_heads";
const VERIFIED_HEAD_SCHEMA: &str = "ayken-verified-head/1.0";

#[derive(Deserialize)]
struct VerifiedHeadRecord {
    schema: String,
    head_sha: String,
    verification: VerifiedHeadVerification,
}

#[derive(Deserialize)]
struct VerifiedHeadVerification {
    ci_freeze: VerifiedHeadCiFreeze,
}

#[derive(Deserialize)]
struct VerifiedHeadCiFreeze {
    workflow: String,
    run_id: String,
    result: String,
    authority: String,
    completed_utc: String,
}

#[derive(Serialize)]
pub(crate) struct HeadStatus {
    pub(crate) base_path: &'static str,
    pub(crate) record_path: String,
    pub(crate) record_exists: bool,
    pub(crate) git_head_sha: Option<String>,
    pub(crate) head_short_sha: Option<String>,
    pub(crate) recorded_head_sha: Option<String>,
    pub(crate) schema_valid: bool,
    pub(crate) head_sha_match: bool,
    pub(crate) ci_freeze_workflow: Option<String>,
    pub(crate) ci_freeze_run_id: Option<String>,
    pub(crate) ci_freeze_pass: bool,
    pub(crate) ci_freeze_authority: Option<String>,
    pub(crate) ci_freeze_completed_utc: Option<String>,
    pub(crate) head_verified: bool,
    pub(crate) evaluation_error: Option<String>,
    pub(crate) note: &'static str,
}

pub(crate) struct HeadEvaluation {
    pub(crate) status: HeadStatus,
    pub(crate) verify_error: Option<AykenError>,
}

pub fn run(args: HeadArgs, json: bool) -> Result<(), AykenError> {
    match args.target {
        HeadTarget::Verify => {
            let evaluation = evaluate_head_status();
            emit_status("verify", &evaluation.status, json)?;
            match evaluation.verify_error {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }
}

pub(crate) fn evaluate_head_status() -> HeadEvaluation {
    let mut status = HeadStatus {
        base_path: VERIFIED_HEADS_BASE,
        record_path: Path::new(VERIFIED_HEADS_BASE)
            .join("<HEAD>.json")
            .display()
            .to_string(),
        record_exists: false,
        git_head_sha: None,
        head_short_sha: None,
        recorded_head_sha: None,
        schema_valid: false,
        head_sha_match: false,
        ci_freeze_workflow: None,
        ci_freeze_run_id: None,
        ci_freeze_pass: false,
        ci_freeze_authority: None,
        ci_freeze_completed_utc: None,
        head_verified: false,
        evaluation_error: None,
        note: "Verified head authority requires a CI-backed record under reports/verified_heads/ with ci-freeze PASS for the current HEAD. A verified head is not an official closure.",
    };

    let git_head_sha = match read_git_head_sha() {
        Ok(sha) => sha,
        Err(error) => return evaluation_failure(status, error),
    };
    let head_short_sha = match short_sha(&git_head_sha) {
        Ok(sha) => sha,
        Err(error) => return evaluation_failure(status, error),
    };
    let record_path = Path::new(VERIFIED_HEADS_BASE).join(format!("{head_short_sha}.json"));

    status.record_path = record_path.display().to_string();
    status.record_exists = record_path.exists();
    status.git_head_sha = Some(git_head_sha.clone());
    status.head_short_sha = Some(head_short_sha);

    let record = match load_verified_head_record(&record_path) {
        Ok(record) => record,
        Err(error) => return evaluation_failure(status, error),
    };

    status.recorded_head_sha = Some(record.head_sha.clone());
    status.schema_valid = record.schema == VERIFIED_HEAD_SCHEMA;
    status.head_sha_match = record.head_sha == git_head_sha;
    status.ci_freeze_workflow = Some(record.verification.ci_freeze.workflow.clone());
    status.ci_freeze_run_id = Some(record.verification.ci_freeze.run_id.clone());
    status.ci_freeze_pass = record.verification.ci_freeze.result == "PASS";
    status.ci_freeze_authority = Some(record.verification.ci_freeze.authority.clone());
    status.ci_freeze_completed_utc = Some(record.verification.ci_freeze.completed_utc.clone());
    status.head_verified = status.schema_valid
        && status.head_sha_match
        && status.ci_freeze_workflow.as_deref() == Some("ci-freeze")
        && status.ci_freeze_pass;

    let verify_error = (!status.head_verified)
        .then(|| fail_closed_policy("verified head authority not confirmed"));

    HeadEvaluation {
        status,
        verify_error,
    }
}

fn load_verified_head_record(path: &PathBuf) -> Result<VerifiedHeadRecord, AykenError> {
    load_json_file(path, &format!("verified head record {}", path.display()))
}

fn evaluation_failure(mut status: HeadStatus, error: AykenError) -> HeadEvaluation {
    status.evaluation_error = Some(error.to_string());
    HeadEvaluation {
        status,
        verify_error: Some(error),
    }
}

fn emit_status(command: &str, status: &HeadStatus, json: bool) -> Result<(), AykenError> {
    if json {
        output::print_json(status)
    } else {
        println!("ayken head {command}");
        println!("  base_path            : {}", status.base_path);
        println!("  record_path          : {}", status.record_path);
        println!("  record_exists        : {}", status.record_exists);
        println!(
            "  git_head_sha         : {}",
            status.git_head_sha.as_deref().unwrap_or("n/a")
        );
        println!(
            "  head_short_sha       : {}",
            status.head_short_sha.as_deref().unwrap_or("n/a")
        );
        println!(
            "  recorded_head_sha    : {}",
            status.recorded_head_sha.as_deref().unwrap_or("n/a")
        );
        println!("  schema_valid         : {}", status.schema_valid);
        println!("  head_sha_match       : {}", status.head_sha_match);
        println!(
            "  ci_freeze_workflow   : {}",
            status.ci_freeze_workflow.as_deref().unwrap_or("n/a")
        );
        println!(
            "  ci_freeze_run_id     : {}",
            status.ci_freeze_run_id.as_deref().unwrap_or("n/a")
        );
        println!("  ci_freeze_pass       : {}", status.ci_freeze_pass);
        println!(
            "  ci_freeze_authority  : {}",
            status.ci_freeze_authority.as_deref().unwrap_or("n/a")
        );
        println!(
            "  completed_utc        : {}",
            status.ci_freeze_completed_utc.as_deref().unwrap_or("n/a")
        );
        println!("  head_verified        : {}", status.head_verified);
        if let Some(error) = &status.evaluation_error {
            println!("  evaluation_error     : {error}");
        }
        println!("  note: {}", status.note);
        io::stdout().flush()?;
        Ok(())
    }
}
