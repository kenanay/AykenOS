// Constitutional Module: HintOrchestrator
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Orchestrate hint generation across ARH subsystems.

use crate::arh::assisted_fix_engine::{AssistedFixDecision, AssistedFixRequest, ImpactScope, ViolationType as AssistedViolationType};
use crate::arh::confidence_calculator::{AutomationEligibility, ConfidenceScore};
use crate::arh::context_analyzer::{ContextSummary};
use crate::arh::design_hint_engine::{DesignHintEngine, DesignHintRequest, DesignHintOutput, ViolationType as DesignViolationType};
use crate::arh::fix_mapping::{FixMappingResult, HintType};
use crate::arh::pattern_matcher::PatternMatchResult;
use crate::arh::semantic_analysis::{SemanticAssessment, SemanticSafety};
use crate::arh::arh_engine::{SuppressedHint, ViolationInput, AssistedFixPayload};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HintOutput {
    AssistedFix(AssistedFixDecision),
    DesignHint(DesignHintOutput),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedHint {
    pub hint_type: HintType,
    pub output: HintOutput,
    pub confidence: ConfidenceScore,
    pub risk_score: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrchestrationResult {
    pub selected_types: Vec<HintType>,
    pub generated_hints: Vec<GeneratedHint>,
    pub suppressed_hints: Vec<SuppressedHint>,
}

pub struct HintOrchestrator {
    assisted_fix_engine: crate::arh::assisted_fix_engine::AssistedFixEngine,
    design_hint_engine: DesignHintEngine,
}

impl HintOrchestrator {
    pub fn new(
        assisted_fix_engine: crate::arh::assisted_fix_engine::AssistedFixEngine,
        design_hint_engine: DesignHintEngine,
    ) -> Self {
        Self { assisted_fix_engine, design_hint_engine }
    }

    pub fn orchestrate(
        &self,
        input: &ViolationInput,
        mapping: &FixMappingResult,
        _pattern: &PatternMatchResult,
        context: &ContextSummary,
        semantic: &SemanticAssessment,
        confidence: &ConfidenceScore,
        automation_eligibility: AutomationEligibility,
    ) -> OrchestrationResult {
        let mut selected_types = Vec::new();
        let mut generated = Vec::new();
        let mut suppressed = Vec::new();

        for hint_type in &mapping.allowed {
            match hint_type {
                HintType::AssistedFix => {
                    if let Some(payload) = input.assisted_fix_payload.clone() {
                        let decision = self.build_assisted_fix(input, payload);
                        let risk_score = risk_from_semantic(semantic);
                        generated.push(GeneratedHint {
                            hint_type: *hint_type,
                            output: HintOutput::AssistedFix(decision),
                            confidence: confidence.clone(),
                            risk_score,
                        });
                        selected_types.push(*hint_type);
                    } else {
                        suppressed.push(SuppressedHint {
                            hint_type: *hint_type,
                            reason: "Missing assisted-fix payload".to_string(),
                        });
                    }
                }
                HintType::DesignHint => {
                    let output = self.build_design_hint(input);
                    let risk_score = risk_from_semantic(semantic).max(if !context.boundary_flags.is_empty() { 75 } else { 0 });
                    generated.push(GeneratedHint {
                        hint_type: *hint_type,
                        output: HintOutput::DesignHint(output),
                        confidence: confidence.clone(),
                        risk_score,
                    });
                    selected_types.push(*hint_type);
                }
            }
        }

        for hint_type in &mapping.forbidden {
            suppressed.push(SuppressedHint {
                hint_type: *hint_type,
                reason: "Forbidden by canonical fix mapping".to_string(),
            });
        }

        if automation_eligibility == AutomationEligibility::No {
            for hint in &mut generated {
                let reason = SuppressedHint {
                    hint_type: hint.hint_type,
                    reason: "Automation disabled by boundary or complexity budget".to_string(),
                };
                suppressed.push(reason);
            }
        }

        OrchestrationResult {
            selected_types,
            generated_hints: generated,
            suppressed_hints: suppressed,
        }
    }

    fn build_assisted_fix(&self, input: &ViolationInput, payload: AssistedFixPayload) -> AssistedFixDecision {
        let violation = map_assisted_violation(&input.rule_id);
        let request = AssistedFixRequest {
            is_kernel: input.is_kernel,
            impact_scope: ImpactScope::ProcessOnly,
            violation,
            before_code: payload.before_code,
            after_code: payload.after_code,
            before_signature: payload.before_signature,
            after_signature: payload.after_signature,
            impacted_call_sites: payload.impacted_call_sites,
        };
        self.assisted_fix_engine.analyze(request)
    }

    fn build_design_hint(&self, input: &ViolationInput) -> DesignHintOutput {
        let violation = map_design_violation(&input.rule_id);
        let request = DesignHintRequest {
            violation,
            is_kernel: input.is_kernel,
        };
        self.design_hint_engine.generate(request)
    }
}

fn map_assisted_violation(rule_id: &str) -> AssistedViolationType {
    match rule_id {
        "TIME.INSTANT" => AssistedViolationType::Time,
        "ERROR.UNWRAP" => AssistedViolationType::Error,
        _ => AssistedViolationType::Error,
    }
}

fn map_design_violation(rule_id: &str) -> DesignViolationType {
    match rule_id {
        "ALLOC.GLOBAL" => DesignViolationType::AllocGlobal,
        "DETERMINISM.RNG" => DesignViolationType::DeterminismRng,
        _ => DesignViolationType::AllocGlobal,
    }
}

fn risk_from_semantic(semantic: &SemanticAssessment) -> u8 {
    match semantic.safety {
        SemanticSafety::Safe => 20,
        SemanticSafety::Risky => 60,
        SemanticSafety::Blocked => 90,
    }
}
