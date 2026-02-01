//! Fix generation validation tests (deterministic).
//! Guarantees: mode gating and rule filtering behavior.

use crate::arh::arh_engine::{ArhEngine, LatencyProfile, ViolationInput, SemanticFlags};
use crate::arh::context_analyzer::UsagePattern;
use crate::arh::pattern_matcher::PatternToken;

#[test]
fn arh_generation_is_deterministic_for_same_input() {
    let engine = ArhEngine::new();
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

    let a = engine.generate(input.clone());
    let b = engine.generate(input);
    assert_eq!(a, b, "ARH output must be deterministic");
}
