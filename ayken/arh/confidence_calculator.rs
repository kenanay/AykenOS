// Constitutional Module: ConfidenceCalculator
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Confidence calculation (advisory-only).

use crate::arh::context_analyzer::BoundaryFlag;
use crate::arh::semantic_analysis::SemanticSafety;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidenceScore {
    pub score: u8,
    pub lower_bound: u8,
    pub upper_bound: u8,
    pub human_review_recommended: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationEligibility {
    Yes,
    No,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComplexityBudget {
    pub max_cost: u32,
    pub current_cost: u32,
}

pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// Compute a deterministic confidence score from analysis inputs.
    pub fn compute(
        &self,
        pattern_complexity_cost: u32,
        context_certainty: u8,
        semantic_safety: SemanticSafety,
        edge_case_density: u8,
    ) -> ConfidenceScore {
        let mut score = 100u32;
        score = score.saturating_sub(pattern_complexity_cost * 10);
        score = score.saturating_sub((100u32.saturating_sub(context_certainty as u32)) / 2);
        score = score.saturating_sub(edge_case_density as u32 * 5);

        if semantic_safety == SemanticSafety::Blocked {
            score = score.min(30);
        } else if semantic_safety == SemanticSafety::Risky {
            score = score.min(60);
        }

        let final_score = score.clamp(0, 100) as u8;
        let lower_bound = final_score.saturating_sub(10);
        let upper_bound = (final_score + 10).min(100);
        let human_review_recommended = final_score < 85;

        ConfidenceScore {
            score: final_score,
            lower_bound,
            upper_bound,
            human_review_recommended,
        }
    }

    /// Determine automation eligibility based on boundary flags and complexity budget.
    pub fn automation_eligibility(
        &self,
        boundary_flags: &[BoundaryFlag],
        budget: ComplexityBudget,
    ) -> AutomationEligibility {
        if budget.current_cost > budget.max_cost {
            return AutomationEligibility::No;
        }
        if boundary_flags.iter().any(is_boundary_lock) {
            return AutomationEligibility::No;
        }
        AutomationEligibility::Yes
    }
}

fn is_boundary_lock(flag: &BoundaryFlag) -> bool {
    matches!(
        flag,
        BoundaryFlag::UserspaceToKernel
            | BoundaryFlag::KernelToUserspace
            | BoundaryFlag::SharedAbiOrFfi
            | BoundaryFlag::ModuleBoundaryViolation
    )
}
