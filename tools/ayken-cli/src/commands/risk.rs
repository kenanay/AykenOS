use crate::cli::RiskArgs;
use crate::commands::status::{gather_authority_status, AuthorityStatus, LineageConfidence};
use crate::core::{error::AykenError, output};
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct AdvisoryRiskSummary {
    authority: &'static str,
    lineage_confidence: Option<LineageConfidence>,
    lineage_tainted: bool,
    ancestor_distance: Option<usize>,
    risk_level: &'static str,
    note: &'static str,
}

pub fn run(_args: RiskArgs, json: bool) -> Result<(), AykenError> {
    let status = gather_authority_status();
    let summary = compute_risk(&status);

    if json {
        output::print_json(&summary)
    } else {
        println!("ayken risk");
        println!("  authority            : {}", summary.authority);
        println!(
            "  lineage_confidence   : {}",
            summary
                .lineage_confidence
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!("  lineage_tainted      : {}", summary.lineage_tainted);
        println!(
            "  ancestor_distance    : {}",
            summary
                .ancestor_distance
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!("  risk_level           : {}", summary.risk_level);
        println!("  note: {}", summary.note);
        io::stdout().flush()?;
        Ok(())
    }
}

fn compute_risk(status: &AuthorityStatus) -> AdvisoryRiskSummary {
    let risk_level = match status.effective_authority {
        "closure" => "none",
        "verified_head" => "low",
        "none" => {
            if status.lineage_tainted {
                "high"
            } else {
                match status.ancestor_distance {
                    Some(distance) if distance <= 3 => "medium",
                    Some(distance) if distance <= 10 => "medium-high",
                    Some(_) => "high",
                    None => "unknown",
                }
            }
        }
        _ => "unknown",
    };

    AdvisoryRiskSummary {
        authority: status.effective_authority,
        lineage_confidence: status.lineage_confidence,
        lineage_tainted: status.lineage_tainted,
        ancestor_distance: status.ancestor_distance,
        risk_level,
        note: "Advisory only. Does not affect authority.",
    }
}
