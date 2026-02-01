//! End-to-end workflow tests (ARRE + ARH).
//! Guarantees: ARRE avoid suppresses ARH hints.

use crate::arh::arre_arh_integration::{ArreEngine, ArreRecommendationResult, IntegratedRefactorSystem};
use crate::arh::arh_engine::{ViolationInput, LatencyProfile, SemanticFlags};
use crate::arh::refactor_guidance::{ArreRecommendation, ArchitecturalPriority};
use crate::arh::context_analyzer::UsagePattern;
use crate::arh::pattern_matcher::PatternToken;

struct AvoidArre;

impl ArreEngine for AvoidArre {
    fn analyze(&self, _violation: &ViolationInput) -> ArreRecommendationResult {
        ArreRecommendationResult {
            architectural_intent: "avoid".to_string(),
            recommendation: ArreRecommendation::Avoid,
            priority: ArchitecturalPriority::High,
            long_term_risk: "high".to_string(),
        }
    }
}

#[test]
fn arre_avoid_suppresses_arh_hints() {
    let system = IntegratedRefactorSystem::new(AvoidArre);
    let input = ViolationInput {
        violation_id: "v1".to_string(),
        rule_id: "TIME.INSTANT".to_string(),
        is_kernel: false,
        tokens: vec![PatternToken::RuleTag("TIME.INSTANT".to_string())],
        context_scope_depth: 0,
        usage_pattern: UsagePattern::InitPath,
        boundary_flags: vec![],
        context_certainty: 100,
        edge_case_density: 0,
        semantic_flags: SemanticFlags {
            side_effect_risk: false,
            state_mutation_risk: false,
            determinism_break_risk: false,
            lifetime_or_ownership_risk: false,
        },
        assisted_fix_payload: None,
        arre_note: None,
        latency_profile: LatencyProfile::VsCode,
    };

    let guidance = system.integrate(input);
    assert!(guidance.tactical_hints.is_empty(), "ARRE avoid must suppress ARH hints");
}
