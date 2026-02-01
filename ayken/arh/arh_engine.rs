// Constitutional Module: ARH Engine
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! ARH Generation Engine & Orchestration (advisory-only, non-enforcing).

use crate::arh::assisted_fix_engine::AssistedFixEngine;
use crate::arh::confidence_calculator::{
    AutomationEligibility, ConfidenceCalculator, ComplexityBudget,
};
use crate::arh::context_analyzer::{BoundaryFlag, ContextAnalyzer, ContextSummary, UsagePattern};
use crate::arh::design_hint_engine::DesignHintEngine;
use crate::arh::fix_mapping::{FixMapping, HintType};
use crate::arh::hint_orchestrator::HintOrchestrator;
use crate::arh::hint_prioritizer::{HintPrioritizer, RankedHint};
use crate::arh::pattern_matcher::{PatternLibrary, PatternMatcher, PatternToken};
use crate::arh::semantic_analysis::{SemanticAnalyzer, SemanticAssessment, SemanticSafety};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LatencyProfile {
    VsCode,
    Ci,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArreGuidance {
    Avoid,
    Neutral,
    Prefer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArreNote {
    pub guidance: ArreGuidance,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViolationInput {
    pub violation_id: String,
    pub rule_id: String,
    pub is_kernel: bool,
    pub tokens: Vec<PatternToken>,
    pub context_scope_depth: u32,
    pub usage_pattern: UsagePattern,
    pub boundary_flags: Vec<BoundaryFlag>,
    pub context_certainty: u8,
    pub edge_case_density: u8,
    pub semantic_flags: SemanticFlags,
    pub assisted_fix_payload: Option<AssistedFixPayload>,
    pub arre_note: Option<ArreNote>,
    pub latency_profile: LatencyProfile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticFlags {
    pub side_effect_risk: bool,
    pub state_mutation_risk: bool,
    pub determinism_break_risk: bool,
    pub lifetime_or_ownership_risk: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistedFixPayload {
    pub before_code: String,
    pub after_code: String,
    pub before_signature: String,
    pub after_signature: String,
    pub impacted_call_sites: Vec<crate::arh::signature_analysis::CallSiteImpact>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuppressedHint {
    pub hint_type: HintType,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArhOutput {
    pub violation_id: String,
    pub selected_hint_types: Vec<HintType>,
    pub ranked_hints: Vec<RankedHint>,
    pub combined_risk_score: u8,
    pub automation_eligibility: AutomationEligibility,
    pub suppressed_hints: Vec<SuppressedHint>,
    pub arre_precedence_notes: String,
    pub latency_profile: LatencyProfile,
}

pub struct ArhEngine {
    matcher: PatternMatcher,
    context_analyzer: ContextAnalyzer,
    semantic_analyzer: SemanticAnalyzer,
    confidence: ConfidenceCalculator,
    fix_mapping: FixMapping,
    orchestrator: HintOrchestrator,
    prioritizer: HintPrioritizer,
    pattern_library: PatternLibrary,
}

impl ArhEngine {
    pub fn new() -> Self {
        Self {
            matcher: PatternMatcher,
            context_analyzer: ContextAnalyzer,
            semantic_analyzer: SemanticAnalyzer,
            confidence: ConfidenceCalculator,
            fix_mapping: FixMapping::default(),
            orchestrator: HintOrchestrator::new(AssistedFixEngine::new(), DesignHintEngine),
            prioritizer: HintPrioritizer,
            pattern_library: PatternLibrary { version: "v1".to_string() },
        }
    }

    /// Orchestrate ARH outputs for a violation (advisory-only).
    pub fn generate(&self, input: ViolationInput) -> ArhOutput {
        let pattern_result = self.matcher.match_patterns(&input.tokens, &self.pattern_library);
        let context_summary = self.context_analyzer.analyze(
            input.context_scope_depth,
            input.usage_pattern,
            input.boundary_flags.clone(),
        );

        let semantic_assessment = self.semantic_analyzer.assess(
            input.semantic_flags.side_effect_risk,
            input.semantic_flags.state_mutation_risk,
            input.semantic_flags.determinism_break_risk,
            input.semantic_flags.lifetime_or_ownership_risk,
        );

        let confidence = self.confidence.compute(
            pattern_result.total_complexity_cost,
            input.context_certainty,
            semantic_assessment.safety,
            input.edge_case_density,
        );

        let budget = complexity_budget(input.latency_profile, pattern_result.total_complexity_cost);
        let automation_eligibility = self.confidence.automation_eligibility(&context_summary.boundary_flags, budget);

        let mapping = self.fix_mapping.for_violation(&input.rule_id, input.is_kernel);

        let arre_notes = arre_notes(input.arre_note.as_ref());
        let orchestration = self.orchestrator.orchestrate(
            &input,
            &mapping,
            &pattern_result,
            &context_summary,
            &semantic_assessment,
            &confidence,
            automation_eligibility,
        );

        let ranked = self.prioritizer.prioritize(
            orchestration.generated_hints,
            input.is_kernel,
        );

        let combined_risk_score = combined_risk(&ranked, &semantic_assessment, &context_summary);

        ArhOutput {
            violation_id: input.violation_id,
            selected_hint_types: orchestration.selected_types,
            ranked_hints: ranked,
            combined_risk_score,
            automation_eligibility,
            suppressed_hints: orchestration.suppressed_hints,
            arre_precedence_notes: arre_notes,
            latency_profile: input.latency_profile,
        }
    }
}

fn complexity_budget(profile: LatencyProfile, current_cost: u32) -> ComplexityBudget {
    let max_cost = match profile {
        LatencyProfile::VsCode => 6,
        LatencyProfile::Ci => 12,
    };
    ComplexityBudget {
        max_cost,
        current_cost,
    }
}

fn combined_risk(
    ranked: &[RankedHint],
    semantic: &SemanticAssessment,
    context: &ContextSummary,
) -> u8 {
    let mut risk = match semantic.safety {
        SemanticSafety::Safe => 20,
        SemanticSafety::Risky => 60,
        SemanticSafety::Blocked => 90,
    };
    if !context.boundary_flags.is_empty() {
        risk = risk.max(75);
    }
    if !ranked.is_empty() && ranked[0].risk_score > risk {
        risk = ranked[0].risk_score;
    }
    risk
}

fn arre_notes(note: Option<&ArreNote>) -> String {
    match note {
        Some(n) => format!("ARRE: {:?} — {}", n.guidance, n.note),
        None => "ARRE: none".to_string(),
    }
}
