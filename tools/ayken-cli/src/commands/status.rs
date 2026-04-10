use crate::cli::StatusArgs;
use crate::commands::{closure, head};
use crate::core::{error::AykenError, output};
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct AuthorityStatus {
    git_head_sha: Option<String>,
    closure_authority_confirmed: bool,
    closure_reference: &'static str,
    closure_run_id: Option<String>,
    closure_evaluation_error: Option<String>,
    head_verified: bool,
    effective_authority: &'static str,
    verified_head_reference: String,
    verified_head_run_id: Option<String>,
    verified_head_authority: Option<String>,
    head_evaluation_error: Option<String>,
    lineage_resolved: bool,
    lineage_tainted: bool,
    nearest_verified_ancestor: Option<String>,
    ancestor_distance: Option<usize>,
    lineage_evaluation_error: Option<String>,
    note: &'static str,
}

pub fn run(_args: StatusArgs, json: bool) -> Result<(), AykenError> {
    let closure = closure::evaluate_closure_status();
    let head = head::evaluate_head_status();
    let lineage = head::evaluate_head_lineage();
    let effective_authority = if closure.status.authority_confirmed {
        "closure"
    } else if head.status.head_verified {
        "verified_head"
    } else {
        "none"
    };

    let status = AuthorityStatus {
        git_head_sha: head
            .status
            .git_head_sha
            .clone()
            .or_else(|| closure.status.git_head_sha.clone()),
        closure_authority_confirmed: closure.status.authority_confirmed,
        closure_reference: "reports/phase15_official_closure/closure_index.json",
        closure_run_id: closure.status.remote_ci_run_id.clone(),
        closure_evaluation_error: closure.status.evaluation_error.clone(),
        head_verified: head.status.head_verified,
        effective_authority,
        verified_head_reference: head.status.record_path.clone(),
        verified_head_run_id: head.status.ci_freeze_run_id.clone(),
        verified_head_authority: head.status.ci_freeze_authority.clone(),
        head_evaluation_error: head.status.evaluation_error.clone(),
        lineage_resolved: lineage.lineage_resolved,
        lineage_tainted: lineage.lineage_tainted,
        nearest_verified_ancestor: lineage.nearest_verified_ancestor,
        ancestor_distance: lineage.ancestor_distance,
        lineage_evaluation_error: lineage.evaluation_error,
        note: "Official closure authority and verified development head authority are separate. Authority lineage is advisory only and does not change effective_authority.",
    };

    if json {
        output::print_json(&status)
    } else {
        println!("ayken status");
        println!(
            "  git_head_sha                 : {}",
            status.git_head_sha.as_deref().unwrap_or("n/a")
        );
        println!(
            "  closure_authority_confirmed  : {}",
            status.closure_authority_confirmed
        );
        println!(
            "  effective_authority          : {}",
            status.effective_authority
        );
        println!(
            "  closure_reference            : {}",
            status.closure_reference
        );
        println!(
            "  closure_run_id               : {}",
            status.closure_run_id.as_deref().unwrap_or("n/a")
        );
        if let Some(error) = &status.closure_evaluation_error {
            println!("  closure_evaluation_error     : {error}");
        }
        println!("  head_verified                : {}", status.head_verified);
        println!(
            "  verified_head_reference      : {}",
            status.verified_head_reference
        );
        println!(
            "  verified_head_run_id         : {}",
            status.verified_head_run_id.as_deref().unwrap_or("n/a")
        );
        println!(
            "  verified_head_authority      : {}",
            status.verified_head_authority.as_deref().unwrap_or("n/a")
        );
        if let Some(error) = &status.head_evaluation_error {
            println!("  head_evaluation_error        : {error}");
        }
        println!(
            "  lineage_resolved             : {}",
            status.lineage_resolved
        );
        println!(
            "  lineage_tainted              : {}",
            status.lineage_tainted
        );
        println!(
            "  nearest_verified_ancestor    : {}",
            status.nearest_verified_ancestor.as_deref().unwrap_or("n/a")
        );
        println!(
            "  ancestor_distance            : {}",
            status
                .ancestor_distance
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        if let Some(error) = &status.lineage_evaluation_error {
            println!("  lineage_evaluation_error     : {error}");
        }
        println!("  note: {}", status.note);
        io::stdout().flush()?;
        Ok(())
    }
}
