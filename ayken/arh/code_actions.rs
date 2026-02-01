// Constitutional Module: CodeActions
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only and UI/UX oriented.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! VS Code Code Action mapping (advisory-only, non-enforcing).

use crate::arh::arh_engine::ArhOutput;
use crate::arh::confidence_calculator::AutomationEligibility;
use crate::arh::fix_mapping::HintType;
use crate::arh::hint_prioritizer::RankedHint;
use crate::arh::hint_orchestrator::HintOutput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RiskLabel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeAction {
    pub title: String,
    pub preferred: bool,
    pub hint_type: HintType,
    pub risk_label: RiskLabel,
    pub confidence: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeActionList {
    pub actions: Vec<CodeAction>,
}

pub struct CodeActionBuilder {
    actions: Vec<CodeAction>,
}

impl CodeActionBuilder {
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    pub fn from_arh_output(&mut self, output: &ArhOutput) {
        for hint in &output.ranked_hints {
            self.actions.push(build_action(hint));
        }
    }

    pub fn build(self) -> CodeActionList {
        CodeActionList { actions: self.actions }
    }
}

fn build_action(hint: &RankedHint) -> CodeAction {
    let risk_label = if hint.risk_score >= 75 {
        RiskLabel::High
    } else if hint.risk_score >= 40 {
        RiskLabel::Medium
    } else {
        RiskLabel::Low
    };

    let base = match (&hint.hint_type, &hint.output) {
        (HintType::AssistedFix, HintOutput::AssistedFix(_)) => {
            if hint.risk_score >= 60 {
                "Refactor with Preview"
            } else {
                "Quick Fix (Preview)"
            }
        }
        (HintType::DesignHint, HintOutput::DesignHint(_)) => "View Architectural Guidance",
        _ => "View Guidance",
    };

    let title = format!(
        "{} (Confidence: {}%, Risk: {:?})",
        base, hint.confidence_score, risk_label
    );

    CodeAction {
        title,
        preferred: hint.automation_eligibility == AutomationEligibility::Yes,
        hint_type: hint.hint_type,
        risk_label,
        confidence: hint.confidence_score,
    }
}
