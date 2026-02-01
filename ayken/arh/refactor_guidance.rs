// Constitutional Module: RefactorGuidance
// This module MUST NOT mutate code or apply fixes.
// It defines canonical guidance structures only.

//! Unified refactor guidance structure (ARRE + ARH).

use crate::arh::arh_engine::ArhOutput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefactorGuidance {
    pub violation_id: String,
    // Strategic (ARRE)
    pub architectural_intent: String,
    pub arre_recommendation: ArreRecommendation,
    pub architectural_priority: ArchitecturalPriority,
    pub long_term_risk: String,
    // Tactical (ARH)
    pub tactical_hints: Vec<TacticalHint>,
    pub implementation_paths: Vec<ImplementationPath>,
    // Consistency
    pub consistency_status: ConsistencyStatus,
    pub user_decision_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchitecturalPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArreRecommendation {
    Avoid,
    Redesign,
    Refactor,
    Monitor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsistencyStatus {
    Aligned,
    Warning(String),
    Conflict(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TacticalHint {
    pub summary: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplementationPath {
    pub description: String,
    pub steps: Vec<String>,
    pub risk_level: String,
    pub reversible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnifiedWorkflowResult {
    pub guidance: RefactorGuidance,
    pub next_actions: Vec<String>,
    pub enforcement_level: EnforcementLevel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnforcementLevel {
    Advisory,
    Warn,
    Block,
}

pub fn tactical_hints_from_arh(arh: &ArhOutput) -> Vec<TacticalHint> {
    arh.ranked_hints
        .iter()
        .map(|hint| TacticalHint {
            summary: format!("Hint {:?} (confidence {}%)", hint.hint_type, hint.confidence_score),
            source: "ARH".to_string(),
        })
        .collect()
}
