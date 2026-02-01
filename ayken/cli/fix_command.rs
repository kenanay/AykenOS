// Constitutional Module: Fix Command
// This module MUST NOT silently mutate code.
// All fix applications are gated by mode and approvals.

use crate::cli::fix_application::{apply_fixes, ApprovalProvider, FixApplier, FixOutcome};
use crate::cli::fix_modes::{validate_mode_flags, FixMode};
use crate::cli::fix_reporting::{generate_report, report_to_text, FixReport};
use crate::arh::arh_engine::{ArhEngine, ViolationInput};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixCommandArgs {
    pub safe: bool,
    pub preview: bool,
    pub report: bool,
    pub rule_filter: Option<String>,
    pub file_filter: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixCommandResult {
    pub mode: FixMode,
    pub outcome: Option<FixOutcome>,
    pub report: Option<FixReport>,
}

pub struct FixCommand {
    engine: ArhEngine,
}

impl FixCommand {
    pub fn new() -> Self {
        Self { engine: ArhEngine::new() }
    }

    pub fn run(
        &self,
        args: FixCommandArgs,
        inputs: Vec<(ViolationInput, bool, String)>,
        applier: &dyn FixApplier,
        approvals: &dyn ApprovalProvider,
    ) -> Result<FixCommandResult, String> {
        let mode = validate_mode_flags(args.safe, args.preview, args.report)?;

        let mut outputs = Vec::new();
        for (input, is_kernel, rule_id) in inputs {
            if let Some(filter) = &args.rule_filter {
                if &rule_id != filter {
                    continue;
                }
            }
            if let Some(filter) = &args.file_filter {
                if !input.violation_id.contains(filter) {
                    continue;
                }
            }

            let output = self.engine.generate(input);
            outputs.push((rule_id, output, is_kernel));
        }

        match mode {
            FixMode::Report => {
                let report = generate_report(&outputs);
                Ok(FixCommandResult { mode, outcome: None, report: Some(report) })
            }
            FixMode::Safe | FixMode::Preview => {
                let mut summary_outcome: Option<FixOutcome> = None;
                for (_, output, is_kernel) in &outputs {
                    let outcome = apply_fixes(mode, output, *is_kernel, applier, approvals);
                    summary_outcome = Some(match summary_outcome {
                        None => outcome,
                        Some(existing) => merge_outcomes(existing, outcome),
                    });
                }
                Ok(FixCommandResult { mode, outcome: summary_outcome, report: None })
            }
        }
    }

    pub fn help_text(&self) -> String {
        [
            "ayken fix --safe",
            "ayken fix --preview --rule ALLOC.GLOBAL",
            "ayken fix --report --file src/kernel/mm.rs",
        ].join("\n")
    }
}

pub fn format_report(report: &FixReport) -> String {
    report_to_text(report)
}

fn merge_outcomes(a: FixOutcome, b: FixOutcome) -> FixOutcome {
    let summary = crate::cli::fix_application::FixApplicationSummary {
        applied: a.summary.applied + b.summary.applied,
        skipped: a.summary.skipped + b.summary.skipped,
        rejected: a.summary.rejected + b.summary.rejected,
        failed: a.summary.failed + b.summary.failed,
    };
    let mut messages = a.messages;
    messages.extend(b.messages);
    FixOutcome { summary, messages }
}
