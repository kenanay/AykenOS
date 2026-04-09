use crate::cli::ClosureArgs;
use crate::core::{error::AykenError, output};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct ClosureStatus {
    base_path: &'static str,
    evidence_index_exists: bool,
    closure_manifest_exists: bool,
    closure_index_exists: bool,
    local_closure_ready: bool,
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

    let status = ClosureStatus {
        base_path: "reports/phase15_official_closure",
        evidence_index_exists: evidence,
        closure_manifest_exists: manifest,
        closure_index_exists: index,
        local_closure_ready: evidence && manifest && index,
        remote_authority_required: true,
        note: "Official closure requires remote GitHub Actions ci-freeze PASS + HEAD SHA",
    };

    if json {
        output::print_json(&status)
    } else {
        println!("ayken closure status");
        println!("  base_path            : {}", status.base_path);
        println!("  evidence_index.json  : {}", status.evidence_index_exists);
        println!("  closure_manifest.json: {}", status.closure_manifest_exists);
        println!("  closure_index.json   : {}", status.closure_index_exists);
        println!("  local_closure_ready  : {}", status.local_closure_ready);
        println!("  remote_authority     : required");
        println!("  note: {}", status.note);
        Ok(())
    }
}
