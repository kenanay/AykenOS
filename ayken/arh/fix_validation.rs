// Constitutional Module: Fix Validation
// This module MUST NOT mutate code or auto-apply fixes.
// It classifies fixes into enforcement signals.

//! Fix validation and classification for CI enforcement.

use crate::arh::confidence_calculator::AutomationEligibility;
use crate::arh::hint_prioritizer::RankedHint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixSeverity {
    SecurityCritical,
    PerformanceRisk,
    Correctness,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixValidation {
    pub enforceable: bool,
    pub is_safe_autofix: bool,
    pub severity: FixSeverity,
    pub reason: String,
    pub suggested_command: String,
}

pub fn classify_fix(rule_id: &str, hint: &RankedHint, is_kernel: bool) -> FixValidation {
    if hint.hint_type == crate::arh::fix_mapping::HintType::DesignHint {
        return FixValidation {
            enforceable: false,
            is_safe_autofix: false,
            severity: FixSeverity::Correctness,
            reason: "DesignHint is advisory-only and never enforceable".to_string(),
            suggested_command: format!(
                "Run:\n  ayken fix --report --rule {}",
                rule_id
            ),
        };
    }

    let is_safe_autofix = hint.confidence_score >= 95;

    // Default severity assignment based on rule family (simplified, deterministic).
    let severity = if rule_id.contains("SECURITY") {
        FixSeverity::SecurityCritical
    } else if rule_id.contains("PERF") {
        FixSeverity::PerformanceRisk
    } else {
        FixSeverity::Correctness
    };

    let enforceable = !is_kernel
        && hint.automation_eligibility == AutomationEligibility::Yes
        && hint.confidence_score >= confidence_threshold(hint, is_safe_autofix)
        && hint.risk_score < risk_threshold(hint);

    let suggested_command = format!(
        "Run:\n  ayken fix --safe --rule {}",
        rule_id
    );

    FixValidation {
        enforceable,
        is_safe_autofix,
        severity,
        reason: format!(
            "confidence={} risk={} eligibility={:?}",
            hint.confidence_score, hint.risk_score, hint.automation_eligibility
        ),
        suggested_command,
    }
}

fn confidence_threshold(hint: &RankedHint, is_safe_autofix: bool) -> u8 {
    match hint.hint_type {
        crate::arh::fix_mapping::HintType::AssistedFix => {
            if is_safe_autofix {
                95
            } else {
                85
            }
        }
        crate::arh::fix_mapping::HintType::DesignHint => u8::MAX, // unreachable; handled earlier
    }
}

fn risk_threshold(hint: &RankedHint) -> u8 {
    match hint.hint_type {
        crate::arh::fix_mapping::HintType::AssistedFix => 60,
        crate::arh::fix_mapping::HintType::DesignHint => 0,
    }
}
