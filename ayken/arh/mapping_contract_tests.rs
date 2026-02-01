//! Mapping contract tests for AHTS → ARH AssistedFix boundary.
//! Guarantees: mapping invariants and justification preservation.

use crate::arh::ahts_mapping::{
    map_quickfix_to_refactor_hint, AhtsQuickFix, AutomationLevel, QuickFixImpactScope,
    TrustBoundary,
};
use crate::arh::assisted_fix_engine::ImpactScope;
use crate::arh::fix_mapping::HintType;
use crate::ci::assisted_fix_boundary_validator::AssistedFixBoundaryValidator;

#[test]
fn quickfix_maps_to_assistedfix_userspace_assisted() {
    let quickfix = AhtsQuickFix {
        id: "q1".to_string(),
        impact_scope: QuickFixImpactScope::ProcessOnly,
        constitutional_justification: "Justification text".to_string(),
    };
    let mapped = map_quickfix_to_refactor_hint(&quickfix);
    assert_eq!(mapped.hint_type, HintType::AssistedFix);
    assert_eq!(mapped.automation_level, AutomationLevel::Assisted);
    assert_eq!(mapped.trust_boundary, TrustBoundary::Userspace);
}

#[test]
fn impact_scope_mapping_is_lossless() {
    let cases = vec![
        (QuickFixImpactScope::ProcessOnly, ImpactScope::ProcessOnly),
        (QuickFixImpactScope::ApiPotential, ImpactScope::ApiPotential),
        (QuickFixImpactScope::Architecture, ImpactScope::Architecture),
        (QuickFixImpactScope::CrossModule, ImpactScope::CrossModule),
    ];
    for (input, expected) in cases {
        let quickfix = AhtsQuickFix {
            id: "q2".to_string(),
            impact_scope: input,
            constitutional_justification: "Justification text".to_string(),
        };
        let mapped = map_quickfix_to_refactor_hint(&quickfix);
        assert_eq!(mapped.impact_scope, expected);
    }
}

#[test]
fn justification_is_preserved_verbatim() {
    let quickfix = AhtsQuickFix {
        id: "q3".to_string(),
        impact_scope: QuickFixImpactScope::ApiPotential,
        constitutional_justification: "VERBATIM".to_string(),
    };
    let mapped = map_quickfix_to_refactor_hint(&quickfix);
    assert_eq!(mapped.constitutional_justification, "VERBATIM");
}

#[test]
fn validator_fails_on_boundary_violation() {
    let quickfix = AhtsQuickFix {
        id: "q4".to_string(),
        impact_scope: QuickFixImpactScope::CrossModule,
        constitutional_justification: "Justification".to_string(),
    };
    let mut mapped = map_quickfix_to_refactor_hint(&quickfix);
    mapped.trust_boundary = TrustBoundary::Kernel;

    let result = AssistedFixBoundaryValidator::validate_mapping(&quickfix, &mapped);
    assert!(result.is_err(), "boundary violation must be detected");
}
