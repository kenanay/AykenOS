// Constitutional Module: ARRE-ARH Integration
// This module MUST NOT mutate code or apply fixes.
// It integrates strategic ARRE guidance with tactical ARH hints.

//! ARRE-ARH integration and consistency validation.

use crate::arh::arh_engine::{ArhEngine, ArhOutput, ViolationInput};
use crate::arh::refactor_guidance::{
    ArreRecommendation, ArchitecturalPriority, ConsistencyStatus, ImplementationPath, RefactorGuidance,
    tactical_hints_from_arh,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArreRecommendationResult {
    pub architectural_intent: String,
    pub recommendation: ArreRecommendation,
    pub priority: ArchitecturalPriority,
    pub long_term_risk: String,
}

pub trait ArreEngine {
    fn analyze(&self, violation: &ViolationInput) -> ArreRecommendationResult;
}

pub struct IntegratedRefactorSystem<E: ArreEngine> {
    arre_engine: E,
    arh_engine: ArhEngine,
}

impl<E: ArreEngine> IntegratedRefactorSystem<E> {
    pub fn new(arre_engine: E) -> Self {
        Self {
            arre_engine,
            arh_engine: ArhEngine::new(),
        }
    }

    pub fn integrate(&self, violation: ViolationInput) -> RefactorGuidance {
        let arre = self.arre_engine.analyze(&violation);
        let arh = self.arh_engine.generate(violation.clone());
        let consistency = validate_alignment(&arre, &arh);
        synthesize_guidance(&arre, &arh, consistency)
    }
}

pub fn validate_alignment(arre: &ArreRecommendationResult, arh: &ArhOutput) -> ConsistencyStatus {
    match arre.recommendation {
        ArreRecommendation::Avoid => {
            if !arh.ranked_hints.is_empty() {
                ConsistencyStatus::Conflict("ARRE advises avoid; ARH hints suppressed".to_string())
            } else {
                ConsistencyStatus::Aligned
            }
        }
        ArreRecommendation::Redesign => {
            if arh.ranked_hints.iter().any(|h| h.risk_score < 30) {
                ConsistencyStatus::Warning("ARRE recommends redesign; low-risk hints may mislead".to_string())
            } else {
                ConsistencyStatus::Aligned
            }
        }
        ArreRecommendation::Refactor | ArreRecommendation::Monitor => ConsistencyStatus::Aligned,
    }
}

pub fn synthesize_guidance(
    arre: &ArreRecommendationResult,
    arh: &ArhOutput,
    consistency: ConsistencyStatus,
) -> RefactorGuidance {
    let tactical_hints = match &consistency {
        ConsistencyStatus::Conflict(_) => Vec::new(),
        _ => tactical_hints_from_arh(arh),
    };
    let implementation_paths = vec![ImplementationPath {
        description: "Translate architectural intent into concrete steps".to_string(),
        steps: vec!["Review ARRE guidance".to_string(), "Select ARH hint".to_string()],
        risk_level: "Medium".to_string(),
        reversible: true,
    }];

    let user_decision_required = matches!(
        consistency,
        ConsistencyStatus::Warning(_) | ConsistencyStatus::Conflict(_)
    );

    RefactorGuidance {
        violation_id: arh.violation_id.clone(),
        architectural_intent: arre.architectural_intent.clone(),
        arre_recommendation: arre.recommendation.clone(),
        architectural_priority: arre.priority.clone(),
        long_term_risk: arre.long_term_risk.clone(),
        tactical_hints,
        implementation_paths,
        consistency_status: consistency,
        user_decision_required,
    }
}
