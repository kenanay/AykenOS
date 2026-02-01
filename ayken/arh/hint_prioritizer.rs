// Constitutional Module: HintPrioritizer
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Prioritize and limit hints deterministically.

use crate::arh::fix_mapping::HintType;
use crate::arh::hint_orchestrator::{GeneratedHint, HintOutput};
use crate::arh::confidence_calculator::AutomationEligibility;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankedHint {
    pub hint_type: HintType,
    pub output: HintOutput,
    pub confidence_score: u8,
    pub risk_score: u8,
    pub automation_eligibility: AutomationEligibility,
}

pub struct HintPrioritizer;

impl HintPrioritizer {
    /// Prioritize hints with deterministic ordering and enforce limits.
    pub fn prioritize(&self, mut hints: Vec<GeneratedHint>, is_kernel: bool) -> Vec<RankedHint> {
        hints.sort_by(|a, b| {
            b.risk_score
                .cmp(&a.risk_score)
                .then_with(|| b.confidence.score.cmp(&a.confidence.score))
                .then_with(|| hint_type_rank(&a.hint_type).cmp(&hint_type_rank(&b.hint_type)))
        });

        let mut ranked = Vec::new();
        let mut seen_types = Vec::new();
        let max_hints = if is_kernel { 1 } else { 3 };

        for hint in hints {
            if ranked.len() >= max_hints {
                break;
            }
            if seen_types.contains(&hint.hint_type) {
                continue;
            }
            seen_types.push(hint.hint_type);
            ranked.push(RankedHint {
                hint_type: hint.hint_type,
                output: hint.output,
                confidence_score: hint.confidence.score,
                risk_score: hint.risk_score,
                automation_eligibility: if hint.confidence.human_review_recommended {
                    AutomationEligibility::No
                } else {
                    AutomationEligibility::Yes
                },
            });
        }

        ranked
    }
}

fn hint_type_rank(hint_type: &HintType) -> u8 {
    match hint_type {
        HintType::AssistedFix => 0,
        HintType::DesignHint => 1,
    }
}
