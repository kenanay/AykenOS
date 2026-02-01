// Constitutional Module: QuickFixes
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only and preview-based.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Quick fix gating (safe autofix only, preview required).

use crate::arh::confidence_calculator::AutomationEligibility;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickFixAction {
    pub preview_required: bool,
    pub confidence: u8,
    pub risk: u8,
    pub eligible: bool,
}

pub fn allow_quick_fix(
    automation: AutomationEligibility,
    confidence: u8,
    risk: u8,
    is_kernel: bool,
) -> QuickFixAction {
    let eligible = automation == AutomationEligibility::Yes && confidence >= 90 && risk < 30 && !is_kernel;

    QuickFixAction {
        preview_required: true,
        confidence,
        risk,
        eligible,
    }
}
