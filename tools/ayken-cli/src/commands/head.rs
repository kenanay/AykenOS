use crate::cli::{HeadArgs, HeadTarget};
use crate::core::{
    authority::{fail_closed_policy, load_json_file, sha256_hex_json},
    error::AykenError,
    git::read_git_head_sha,
    output,
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERIFIED_HEADS_BASE: &str = "reports/verified_heads";
const VERIFIED_HEAD_SCHEMA: &str = "ayken-verified-head/1.1";

#[derive(Deserialize)]
struct VerifiedHeadRecord {
    schema: String,
    head_sha: String,
    verification: VerifiedHeadVerification,
    integrity: VerifiedHeadIntegrity,
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
    run_url: String,
}

#[derive(Deserialize)]
struct VerifiedHeadIntegrity {
    mode: String,
    binding_sha256: String,
}

#[derive(Serialize)]
struct VerifiedHeadBinding<'a> {
    head_sha: &'a str,
    workflow: &'a str,
    run_id: &'a str,
    result: &'a str,
    authority: &'a str,
    completed_utc: &'a str,
    run_url: &'a str,
}

#[derive(Serialize)]
pub(crate) struct HeadStatus {
    pub(crate) base_path: &'static str,
    pub(crate) record_path: String,
    pub(crate) record_exists: bool,
    pub(crate) git_head_sha: Option<String>,
    pub(crate) recorded_head_sha: Option<String>,
    pub(crate) schema_valid: bool,
    pub(crate) head_sha_match: bool,
    pub(crate) binding_integrity_mode: Option<String>,
    pub(crate) binding_sha256: Option<String>,
    pub(crate) binding_integrity_valid: bool,
    pub(crate) ci_freeze_workflow: Option<String>,
    pub(crate) ci_freeze_run_id: Option<String>,
    pub(crate) ci_freeze_pass: bool,
    pub(crate) ci_freeze_authority: Option<String>,
    pub(crate) ci_freeze_completed_utc: Option<String>,
    pub(crate) ci_freeze_run_url: Option<String>,
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
            .join("<FULL_SHA>.json")
            .display()
            .to_string(),
        record_exists: false,
        git_head_sha: None,
        recorded_head_sha: None,
        schema_valid: false,
        head_sha_match: false,
        binding_integrity_mode: None,
        binding_sha256: None,
        binding_integrity_valid: false,
        ci_freeze_workflow: None,
        ci_freeze_run_id: None,
        ci_freeze_pass: false,
        ci_freeze_authority: None,
        ci_freeze_completed_utc: None,
        ci_freeze_run_url: None,
        head_verified: false,
        evaluation_error: None,
        note: "Verified head authority requires a CI-backed record under reports/verified_heads/<FULL_SHA>.json with ci-freeze PASS and a valid binding hash for the exact current SHA. A verified head is not an official closure.",
    };

    let git_head_sha = match read_git_head_sha() {
        Ok(sha) => sha,
        Err(error) => return evaluation_failure(status, error),
    };
    let record_path = Path::new(VERIFIED_HEADS_BASE).join(format!("{git_head_sha}.json"));

    status.record_path = record_path.display().to_string();
    status.record_exists = record_path.exists();
    status.git_head_sha = Some(git_head_sha.clone());

    let record = match load_verified_head_record(&record_path) {
        Ok(record) => record,
        Err(error) => return evaluation_failure(status, error),
    };

    status.recorded_head_sha = Some(record.head_sha.clone());
    status.schema_valid = record.schema == VERIFIED_HEAD_SCHEMA;
    status.head_sha_match = record.head_sha == git_head_sha;
    status.binding_integrity_mode = Some(record.integrity.mode.clone());
    status.binding_sha256 = Some(record.integrity.binding_sha256.clone());
    status.ci_freeze_workflow = Some(record.verification.ci_freeze.workflow.clone());
    status.ci_freeze_run_id = Some(record.verification.ci_freeze.run_id.clone());
    status.ci_freeze_pass = record.verification.ci_freeze.result == "PASS";
    status.ci_freeze_authority = Some(record.verification.ci_freeze.authority.clone());
    status.ci_freeze_completed_utc = Some(record.verification.ci_freeze.completed_utc.clone());
    status.ci_freeze_run_url = Some(record.verification.ci_freeze.run_url.clone());
    status.binding_integrity_valid = match binding_sha256(&record) {
        Ok(binding_sha256) => binding_sha256 == record.integrity.binding_sha256,
        Err(error) => return evaluation_failure(status, error),
    };
    status.head_verified = status.schema_valid
        && status.head_sha_match
        && status.binding_integrity_mode.as_deref() == Some("sha256")
        && status.binding_integrity_valid
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

fn binding_sha256(record: &VerifiedHeadRecord) -> Result<String, AykenError> {
    let binding = VerifiedHeadBinding {
        head_sha: &record.head_sha,
        workflow: &record.verification.ci_freeze.workflow,
        run_id: &record.verification.ci_freeze.run_id,
        result: &record.verification.ci_freeze.result,
        authority: &record.verification.ci_freeze.authority,
        completed_utc: &record.verification.ci_freeze.completed_utc,
        run_url: &record.verification.ci_freeze.run_url,
    };
    sha256_hex_json(&binding)
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
            "  recorded_head_sha    : {}",
            status.recorded_head_sha.as_deref().unwrap_or("n/a")
        );
        println!("  schema_valid         : {}", status.schema_valid);
        println!("  head_sha_match       : {}", status.head_sha_match);
        println!(
            "  integrity_mode       : {}",
            status.binding_integrity_mode.as_deref().unwrap_or("n/a")
        );
        println!(
            "  binding_sha256       : {}",
            status.binding_sha256.as_deref().unwrap_or("n/a")
        );
        println!(
            "  binding_integrity_ok : {}",
            status.binding_integrity_valid
        );
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
        println!(
            "  run_url              : {}",
            status.ci_freeze_run_url.as_deref().unwrap_or("n/a")
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
