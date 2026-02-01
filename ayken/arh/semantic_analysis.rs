// Constitutional Module: SemanticAnalysis
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Semantic analysis for transformation safety (advisory-only).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticSafety {
    Safe,
    Risky,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticAssessment {
    pub safety: SemanticSafety,
    pub side_effect_risk: bool,
    pub state_mutation_risk: bool,
    pub determinism_break_risk: bool,
    pub lifetime_or_ownership_risk: bool,
}

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    /// Perform a deterministic semantic assessment based on provided flags.
    pub fn assess(
        &self,
        side_effect_risk: bool,
        state_mutation_risk: bool,
        determinism_break_risk: bool,
        lifetime_or_ownership_risk: bool,
    ) -> SemanticAssessment {
        let safety = if determinism_break_risk {
            SemanticSafety::Blocked
        } else if side_effect_risk || state_mutation_risk || lifetime_or_ownership_risk {
            SemanticSafety::Risky
        } else {
            SemanticSafety::Safe
        };

        SemanticAssessment {
            safety,
            side_effect_risk,
            state_mutation_risk,
            determinism_break_risk,
            lifetime_or_ownership_risk,
        }
    }

    /// Advisory-only summary suitable for higher-level engines.
    pub fn summary(&self, assessment: &SemanticAssessment) -> String {
        match assessment.safety {
            SemanticSafety::Safe => "Semantic safety: Safe".to_string(),
            SemanticSafety::Risky => "Semantic safety: Risky".to_string(),
            SemanticSafety::Blocked => "Semantic safety: Blocked".to_string(),
        }
    }
}
