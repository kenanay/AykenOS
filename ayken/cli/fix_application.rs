// Constitutional Module: Fix Application
// This module MUST NOT silently mutate code.
// Any applied changes require explicit mode permission and rollback support.

use crate::arh::arh_engine::ArhOutput;
use crate::arh::fix_mapping::HintType;
use crate::arh::hint_prioritizer::RankedHint;
use crate::arh::hint_orchestrator::HintOutput;
use crate::cli::fix_modes::{automation_allowed, FixMode, FixModeConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixApplicationSummary {
    pub applied: usize,
    pub skipped: usize,
    pub rejected: usize,
    pub failed: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixOutcome {
    pub summary: FixApplicationSummary,
    pub messages: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixEdit {
    pub file: String,
    pub range: String,
    pub summary: String,
    pub reversibility_level: String,
}

pub trait FixApplier {
    fn apply(&self, edits: &[FixEdit]) -> Result<(), String>;
    fn rollback(&self) -> Result<(), String>;
}

pub trait ApprovalProvider {
    fn approve(&self, hint: &RankedHint, preview: Option<&crate::arh::preview_generator::PreviewOutput>) -> bool;
}

pub fn apply_fixes(
    mode: FixMode,
    output: &ArhOutput,
    is_kernel: bool,
    applier: &dyn FixApplier,
    approvals: &dyn ApprovalProvider,
) -> FixOutcome {
    let mut summary = FixApplicationSummary {
        applied: 0,
        skipped: 0,
        rejected: 0,
        failed: 0,
    };
    let mut messages = Vec::new();
    let config = FixModeConfig::safe_defaults();

    if is_kernel {
        summary.skipped = output.ranked_hints.len();
        messages.push("Kernel files: automatic fixes are disabled".to_string());
        return FixOutcome { summary, messages };
    }

    for hint in &output.ranked_hints {
        if hint.hint_type == HintType::DesignHint {
            summary.skipped += 1;
            continue;
        }

        match mode {
            FixMode::Report => {
                summary.skipped += 1;
            }
            FixMode::Preview => {
                let preview = match &hint.output {
                    HintOutput::AssistedFix(decision) => decision.preview.as_ref(),
                    _ => None,
                };
                if approvals.approve(hint, preview) {
                    summary.applied += 1;
                } else {
                    summary.rejected += 1;
                }
            }
            FixMode::Safe => {
                let allowed = automation_allowed(
                    hint.automation_eligibility,
                    hint.confidence_score,
                    hint.risk_score,
                    &config,
                );
                if !allowed {
                    summary.skipped += 1;
                    continue;
                }

                // Apply via injected applier to avoid direct mutation here.
                let edits = vec![FixEdit {
                    file: "<detected-file>".to_string(),
                    range: "<start..end>".to_string(),
                    summary: "Safe autofix edit (advisory model)".to_string(),
                    reversibility_level: "Rollback supported".to_string(),
                }];
                match applier.apply(&edits) {
                    Ok(_) => {
                        summary.applied += 1;
                    }
                    Err(err) => {
                        summary.failed += 1;
                        let _ = applier.rollback();
                        messages.push(format!("Fix failed: {}", err));
                    }
                }
            }
        }
    }

    FixOutcome { summary, messages }
}
