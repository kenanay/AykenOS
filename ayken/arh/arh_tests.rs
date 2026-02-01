//! ARH core unit tests (deterministic, fail-closed).
//! Guarantees: hint classification invariants, kernel restrictions, confidence thresholds.

use crate::arh::fix_mapping::HintType;
use crate::arh::fix_validation::classify_fix;
use crate::arh::hint_prioritizer::RankedHint;
use crate::arh::confidence_calculator::AutomationEligibility;
use crate::arh::hint_orchestrator::HintOutput;

#[test]
fn design_hint_is_never_enforceable() {
    let hint = RankedHint {
        hint_type: HintType::DesignHint,
        output: HintOutput::DesignHint(crate::arh::design_hint_engine::DesignHintOutput {
            violation: crate::arh::design_hint_engine::ViolationType::AllocGlobal,
            architectural_intent: "intent".to_string(),
            design_options: vec![],
            trade_offs: crate::arh::design_hint_engine::TradeOffMatrix { rows: vec![] },
            roadmap: crate::arh::implementation_roadmaps::ImplementationRoadmap { steps: vec![] },
            misapplication_risks: vec![],
            educational_notes: crate::arh::educational_content::EducationalContent { notes: vec![] },
            related_arre_patterns: vec![],
            kernel_risk_notice: None,
            sections: vec![],
        }),
        confidence_score: 99,
        risk_score: 0,
        automation_eligibility: AutomationEligibility::Yes,
    };

    let validation = classify_fix("ALLOC.GLOBAL", &hint, false);
    assert!(!validation.enforceable, "DesignHint must never be enforceable");
}

#[test]
fn design_hint_is_never_enforceable_in_kernel() {
    let hint = RankedHint {
        hint_type: HintType::DesignHint,
        output: HintOutput::DesignHint(crate::arh::design_hint_engine::DesignHintOutput {
            violation: crate::arh::design_hint_engine::ViolationType::AllocGlobal,
            architectural_intent: "intent".to_string(),
            design_options: vec![],
            trade_offs: crate::arh::design_hint_engine::TradeOffMatrix { rows: vec![] },
            roadmap: crate::arh::implementation_roadmaps::ImplementationRoadmap { steps: vec![] },
            misapplication_risks: vec![],
            educational_notes: crate::arh::educational_content::EducationalContent { notes: vec![] },
            related_arre_patterns: vec![],
            kernel_risk_notice: None,
            sections: vec![],
        }),
        confidence_score: 99,
        risk_score: 0,
        automation_eligibility: AutomationEligibility::Yes,
    };

    let validation = classify_fix("ALLOC.GLOBAL", &hint, true);
    assert!(!validation.enforceable, "DesignHint must never be enforceable in kernel");
}

#[test]
fn safe_autofix_threshold_is_95() {
    let hint = RankedHint {
        hint_type: HintType::AssistedFix,
        output: HintOutput::AssistedFix(crate::arh::assisted_fix_engine::AssistedFixDecision {
            disposition: crate::arh::assisted_fix_engine::AssistedFixDisposition::AdvisoryOnly,
            requires_opt_in: false,
            preview: None,
            approval_request: None,
            approval_result: None,
            automation_boundary: crate::arh::assisted_fix_engine::AutomationBoundary {
                min_percent: 60,
                max_percent: 90,
                level: crate::arh::assisted_fix_engine::AutomationLevel::AssistedOnly,
            },
            recommendation: crate::arh::assisted_fix_engine::Recommendation {
                title: "r".to_string(),
                details: vec![],
            },
        }),
        confidence_score: 94,
        risk_score: 10,
        automation_eligibility: AutomationEligibility::Yes,
    };

    let validation = classify_fix("TIME.INSTANT", &hint, false);
    assert!(!validation.is_safe_autofix, "<95 confidence must not be safe autofix");
}

#[test]
fn safe_autofix_threshold_accepts_95() {
    let hint = RankedHint {
        hint_type: HintType::AssistedFix,
        output: HintOutput::AssistedFix(crate::arh::assisted_fix_engine::AssistedFixDecision {
            disposition: crate::arh::assisted_fix_engine::AssistedFixDisposition::AdvisoryOnly,
            requires_opt_in: false,
            preview: None,
            approval_request: None,
            approval_result: None,
            automation_boundary: crate::arh::assisted_fix_engine::AutomationBoundary {
                min_percent: 60,
                max_percent: 90,
                level: crate::arh::assisted_fix_engine::AutomationLevel::AssistedOnly,
            },
            recommendation: crate::arh::assisted_fix_engine::Recommendation {
                title: "r".to_string(),
                details: vec![],
            },
        }),
        confidence_score: 95,
        risk_score: 10,
        automation_eligibility: AutomationEligibility::Yes,
    };

    let validation = classify_fix("TIME.INSTANT", &hint, false);
    assert!(validation.is_safe_autofix, "95 confidence must be safe autofix");
}
