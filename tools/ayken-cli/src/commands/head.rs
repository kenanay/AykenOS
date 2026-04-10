use crate::cli::{HeadArgs, HeadTarget};
use crate::core::{
    authority::{fail_closed_policy, load_optional_json_file, sha256_hex_json},
    error::AykenError,
    git::{is_git_worktree_dirty, list_first_parent_ancestors, read_git_head_sha},
    output,
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERIFIED_HEADS_BASE: &str = "reports/verified_heads";
const VERIFIED_HEAD_SCHEMA: &str = "ayken-verified-head/1.1";
const LINEAGE_MAX_DEPTH: usize = 32;
const LINEAGE_TRAVERSAL_MODE: &str = "first-parent";

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

#[derive(Serialize)]
pub(crate) struct HeadLineageStatus {
    pub(crate) base_path: &'static str,
    pub(crate) git_head_sha: Option<String>,
    pub(crate) traversal_mode: &'static str,
    pub(crate) max_depth: usize,
    pub(crate) dirty_worktree: bool,
    pub(crate) lineage_tainted: bool,
    pub(crate) exact_head_verified: bool,
    pub(crate) lineage_resolved: bool,
    pub(crate) nearest_verified_ancestor: Option<String>,
    pub(crate) ancestor_distance: Option<usize>,
    pub(crate) nearest_record_path: Option<String>,
    pub(crate) nearest_verified_run_id: Option<String>,
    pub(crate) nearest_verified_authority: Option<String>,
    pub(crate) inspected_ancestors: usize,
    pub(crate) advisory_diagnostics: Vec<String>,
    pub(crate) evaluation_error: Option<String>,
    pub(crate) note: &'static str,
}

struct VerifiedHeadInspection {
    record_path: String,
    record_exists: bool,
    recorded_head_sha: Option<String>,
    schema_valid: bool,
    head_sha_match: bool,
    binding_integrity_mode: Option<String>,
    binding_sha256: Option<String>,
    binding_integrity_valid: bool,
    ci_freeze_workflow: Option<String>,
    ci_freeze_run_id: Option<String>,
    ci_freeze_pass: bool,
    ci_freeze_authority: Option<String>,
    ci_freeze_completed_utc: Option<String>,
    ci_freeze_run_url: Option<String>,
    head_verified: bool,
    evaluation_error: Option<String>,
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
        HeadTarget::Lineage => {
            let status = evaluate_head_lineage();
            emit_lineage_status(&status, json)
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
    status.git_head_sha = Some(git_head_sha.clone());

    let inspection = inspect_verified_head_sha(&git_head_sha);
    apply_inspection_to_head_status(&mut status, &inspection);
    if !status.record_exists && status.evaluation_error.is_none() {
        status.evaluation_error = Some(
            fail_closed_policy(format!(
                "verified head record {} missing",
                status.record_path
            ))
            .to_string(),
        );
    }

    let verify_error = (!status.head_verified)
        .then(|| fail_closed_policy("verified head authority not confirmed"));

    HeadEvaluation {
        status,
        verify_error,
    }
}

pub(crate) fn evaluate_head_lineage() -> HeadLineageStatus {
    let mut status = HeadLineageStatus {
        base_path: VERIFIED_HEADS_BASE,
        git_head_sha: None,
        traversal_mode: LINEAGE_TRAVERSAL_MODE,
        max_depth: LINEAGE_MAX_DEPTH,
        dirty_worktree: false,
        lineage_tainted: false,
        exact_head_verified: false,
        lineage_resolved: false,
        nearest_verified_ancestor: None,
        ancestor_distance: None,
        nearest_record_path: None,
        nearest_verified_run_id: None,
        nearest_verified_authority: None,
        inspected_ancestors: 0,
        advisory_diagnostics: Vec::new(),
        evaluation_error: None,
        note: "Authority lineage is advisory only. It may locate the nearest verified ancestor, but it MUST NOT change effective_authority or produce head_verified=true for the current SHA.",
    };

    let git_head_sha = match read_git_head_sha() {
        Ok(sha) => sha,
        Err(error) => {
            status.evaluation_error = Some(error.to_string());
            return status;
        }
    };
    status.git_head_sha = Some(git_head_sha.clone());

    match is_git_worktree_dirty() {
        Ok(dirty) => {
            status.dirty_worktree = dirty;
            status.lineage_tainted = dirty;
            if dirty {
                status
                    .advisory_diagnostics
                    .push("dirty worktree taints lineage diagnostics".to_string());
            }
        }
        Err(error) => {
            status.evaluation_error = Some(error.to_string());
            return status;
        }
    }

    let exact = inspect_verified_head_sha(&git_head_sha);
    status.exact_head_verified = exact.head_verified;
    if exact.head_verified {
        status.advisory_diagnostics.push(
            "current SHA already has exact verified-head authority; lineage traversal skipped"
                .to_string(),
        );
        return status;
    }
    if let Some(error) = exact.evaluation_error {
        status.lineage_tainted = true;
        status.advisory_diagnostics.push(format!(
            "ignored invalid exact-SHA verified-head record: {error}"
        ));
    }

    let ancestors = match list_first_parent_ancestors(&git_head_sha, LINEAGE_MAX_DEPTH) {
        Ok(ancestors) => ancestors,
        Err(error) => {
            status.evaluation_error = Some(error.to_string());
            return status;
        }
    };

    for (index, ancestor_sha) in ancestors.iter().enumerate() {
        status.inspected_ancestors = index + 1;
        let inspection = inspect_verified_head_sha(ancestor_sha);

        if inspection.head_verified {
            status.lineage_resolved = true;
            status.nearest_verified_ancestor = Some(ancestor_sha.clone());
            status.ancestor_distance = Some(index + 1);
            status.nearest_record_path = Some(inspection.record_path);
            status.nearest_verified_run_id = inspection.ci_freeze_run_id;
            status.nearest_verified_authority = inspection.ci_freeze_authority;
            break;
        }

        if let Some(error) = inspection.evaluation_error {
            status.lineage_tainted = true;
            status.advisory_diagnostics.push(format!(
                "ignored invalid verified-head record for {ancestor_sha}: {error}"
            ));
        }
    }

    if !status.lineage_resolved {
        status.advisory_diagnostics.push(format!(
            "no verified ancestor found within {} first-parent commits",
            LINEAGE_MAX_DEPTH
        ));
    }

    status
}

fn load_verified_head_record(path: &PathBuf) -> Result<Option<VerifiedHeadRecord>, AykenError> {
    load_optional_json_file(path, &format!("verified head record {}", path.display()))
}

fn verified_head_record_path(sha: &str) -> PathBuf {
    Path::new(VERIFIED_HEADS_BASE).join(format!("{sha}.json"))
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

fn inspect_verified_head_sha(expected_sha: &str) -> VerifiedHeadInspection {
    let record_path = verified_head_record_path(expected_sha);
    let mut inspection = VerifiedHeadInspection {
        record_path: record_path.display().to_string(),
        record_exists: false,
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
    };

    let record = match load_verified_head_record(&record_path) {
        Ok(Some(record)) => {
            inspection.record_exists = true;
            record
        }
        Ok(None) => return inspection,
        Err(error) => {
            inspection.record_exists = record_path.exists();
            inspection.evaluation_error = Some(error.to_string());
            return inspection;
        }
    };

    inspection.recorded_head_sha = Some(record.head_sha.clone());
    inspection.schema_valid = record.schema == VERIFIED_HEAD_SCHEMA;
    inspection.head_sha_match = record.head_sha == expected_sha;
    inspection.binding_integrity_mode = Some(record.integrity.mode.clone());
    inspection.binding_sha256 = Some(record.integrity.binding_sha256.clone());
    inspection.ci_freeze_workflow = Some(record.verification.ci_freeze.workflow.clone());
    inspection.ci_freeze_run_id = Some(record.verification.ci_freeze.run_id.clone());
    inspection.ci_freeze_pass = record.verification.ci_freeze.result == "PASS";
    inspection.ci_freeze_authority = Some(record.verification.ci_freeze.authority.clone());
    inspection.ci_freeze_completed_utc = Some(record.verification.ci_freeze.completed_utc.clone());
    inspection.ci_freeze_run_url = Some(record.verification.ci_freeze.run_url.clone());
    inspection.binding_integrity_valid = match binding_sha256(&record) {
        Ok(binding_sha256) => binding_sha256 == record.integrity.binding_sha256,
        Err(error) => {
            inspection.evaluation_error = Some(error.to_string());
            return inspection;
        }
    };
    inspection.head_verified = inspection.schema_valid
        && inspection.head_sha_match
        && inspection.binding_integrity_mode.as_deref() == Some("sha256")
        && inspection.binding_integrity_valid
        && inspection.ci_freeze_workflow.as_deref() == Some("ci-freeze")
        && inspection.ci_freeze_pass;

    inspection
}

fn apply_inspection_to_head_status(status: &mut HeadStatus, inspection: &VerifiedHeadInspection) {
    status.record_path = inspection.record_path.clone();
    status.record_exists = inspection.record_exists;
    status.recorded_head_sha = inspection.recorded_head_sha.clone();
    status.schema_valid = inspection.schema_valid;
    status.head_sha_match = inspection.head_sha_match;
    status.binding_integrity_mode = inspection.binding_integrity_mode.clone();
    status.binding_sha256 = inspection.binding_sha256.clone();
    status.binding_integrity_valid = inspection.binding_integrity_valid;
    status.ci_freeze_workflow = inspection.ci_freeze_workflow.clone();
    status.ci_freeze_run_id = inspection.ci_freeze_run_id.clone();
    status.ci_freeze_pass = inspection.ci_freeze_pass;
    status.ci_freeze_authority = inspection.ci_freeze_authority.clone();
    status.ci_freeze_completed_utc = inspection.ci_freeze_completed_utc.clone();
    status.ci_freeze_run_url = inspection.ci_freeze_run_url.clone();
    status.head_verified = inspection.head_verified;
    status.evaluation_error = inspection.evaluation_error.clone();
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

fn emit_lineage_status(status: &HeadLineageStatus, json: bool) -> Result<(), AykenError> {
    if json {
        output::print_json(status)
    } else {
        println!("ayken head lineage");
        println!("  base_path                 : {}", status.base_path);
        println!(
            "  git_head_sha              : {}",
            status.git_head_sha.as_deref().unwrap_or("n/a")
        );
        println!("  traversal_mode            : {}", status.traversal_mode);
        println!("  max_depth                 : {}", status.max_depth);
        println!("  dirty_worktree            : {}", status.dirty_worktree);
        println!("  lineage_tainted           : {}", status.lineage_tainted);
        println!(
            "  exact_head_verified       : {}",
            status.exact_head_verified
        );
        println!("  lineage_resolved          : {}", status.lineage_resolved);
        println!(
            "  nearest_verified_ancestor : {}",
            status.nearest_verified_ancestor.as_deref().unwrap_or("n/a")
        );
        println!(
            "  ancestor_distance         : {}",
            status
                .ancestor_distance
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  nearest_record_path       : {}",
            status.nearest_record_path.as_deref().unwrap_or("n/a")
        );
        println!(
            "  nearest_verified_run_id   : {}",
            status.nearest_verified_run_id.as_deref().unwrap_or("n/a")
        );
        println!(
            "  nearest_verified_authority: {}",
            status
                .nearest_verified_authority
                .as_deref()
                .unwrap_or("n/a")
        );
        println!(
            "  inspected_ancestors       : {}",
            status.inspected_ancestors
        );
        if let Some(error) = &status.evaluation_error {
            println!("  evaluation_error          : {error}");
        }
        for diagnostic in &status.advisory_diagnostics {
            println!("  advisory_diagnostic       : {diagnostic}");
        }
        println!("  note: {}", status.note);
        io::stdout().flush()?;
        Ok(())
    }
}
