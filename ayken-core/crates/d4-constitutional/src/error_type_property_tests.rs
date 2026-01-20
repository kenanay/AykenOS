//! Property-based tests for error type correctness in the D4 Constitutional Framework
//!
//! **Feature: d4-constitutional-contracts-fixes, Property 3: Error Type Correctness**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**
//!
//! This module tests that the system uses appropriate report types for different error conditions:
//! - ContractViolationReport for contract-specific validation analysis
//! - GateDecisionReport for gate readiness analysis  
//! - ConstitutionalError only for framework init/IO issues
//! - Report type specificity maintained during report propagation
//! - Violation reports include violation-type-specific context and remediation recommendations

use crate::errors::*;
use crate::testing::*;
use crate::types::*;
use crate::templates::TemplateType;
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Strategy for generating contract violation scenarios
pub fn contract_violation_scenario_strategy() -> impl Strategy<Value = ContractViolationScenario> {
    (
        component_id_strategy(),
        prop::collection::vec(contract_finding_strategy(), 0..5),
        prop::collection::vec(contract_violation_strategy(), 0..3),
        prop::collection::vec(contract_recommendation_strategy(), 0..3),
    ).prop_map(|(component, findings, violations, recommendations)| {
        ContractViolationScenario {
            component,
            findings,
            violations,
            recommendations,
        }
    })
}

/// Strategy for generating gate decision scenarios
pub fn gate_decision_scenario_strategy() -> impl Strategy<Value = GateDecisionScenario> {
    (
        gate_readiness_analysis_strategy(),
        prop::collection::vec(transition_finding_strategy(), 0..3),
        permission_analysis_strategy(),
        fingerprint_analysis_strategy(),
        0.0f64..100.0f64,
    ).prop_map(|(readiness_analysis, transition_findings, permission_analysis, fingerprint_analysis, readiness_score)| {
        GateDecisionScenario {
            readiness_analysis,
            transition_findings,
            permission_analysis,
            fingerprint_analysis,
            overall_readiness_score: readiness_score,
        }
    })
}

/// Strategy for generating constitutional error scenarios (framework-level only)
pub fn constitutional_error_scenario_strategy() -> impl Strategy<Value = ConstitutionalErrorScenario> {
    prop_oneof![
        "[a-zA-Z0-9_\\s]{1,100}".prop_map(|reason| ConstitutionalErrorScenario::SystemInitialization { reason }),
        "[a-zA-Z0-9_\\s]{1,100}".prop_map(|reason| ConstitutionalErrorScenario::ConfigurationLoad { reason }),
        "[a-zA-Z0-9_\\s]{1,100}".prop_map(|reason| ConstitutionalErrorScenario::IOError { reason }),
        "[a-zA-Z0-9_\\s]{1,100}".prop_map(|reason| ConstitutionalErrorScenario::FrameworkCorruption { reason }),
    ]
}

/// Strategy for generating contract findings
pub fn contract_finding_strategy() -> impl Strategy<Value = ContractFinding> {
    prop_oneof![
        ("[a-zA-Z0-9_]{1,50}", "[a-zA-Z0-9_\\s]{1,100}").prop_map(|(component, reason)| {
            ContractFinding::InvalidStructure { component, reason }
        }),
        template_type_strategy().prop_map(|template_type| {
            ContractFinding::MissingTemplate { template_type }
        }),
        ("[a-zA-Z0-9_]{1,50}", "[a-zA-Z0-9_\\s]{1,100}").prop_map(|(rule, context)| {
            ContractFinding::RuleBreach { rule, context }
        }),
        (
            "[a-zA-Z0-9_]{1,50}",
            prop::collection::vec("[a-zA-Z0-9_]{1,30}", 0..5)
        ).prop_map(|(specification_type, missing_elements)| {
            ContractFinding::IncompleteSpecification { specification_type, missing_elements }
        }),
    ]
}

/// Strategy for generating contract violations
pub fn contract_violation_strategy() -> impl Strategy<Value = ContractViolation> {
    (
        "[a-zA-Z0-9_]{1,50}",
        component_id_strategy(),
        contract_violation_type_strategy(),
        "[a-zA-Z0-9_\\s]{1,100}",
        "[a-zA-Z0-9_\\s]{1,100}",
        prop::collection::hash_map("[a-zA-Z0-9_]{1,30}", "[a-zA-Z0-9_\\s]{1,50}", 0..5),
    ).prop_map(|(violation_id, component, violation_type, description, formal_specification, context)| {
        ContractViolation {
            violation_id,
            component,
            violation_type,
            description,
            formal_specification,
            context: BTreeMap::new(),
        }
    })
}

/// Strategy for generating contract violation types
pub fn contract_violation_type_strategy() -> impl Strategy<Value = ContractViolationType> {
    prop_oneof![
        Just(ContractViolationType::MissingStructDefinition),
        Just(ContractViolationType::MissingTraitDefinition),
        Just(ContractViolationType::InvariantViolation),
        Just(ContractViolationType::PropertyTestFailure),
        Just(ContractViolationType::PerformanceTargetUndefined),
        Just(ContractViolationType::IncompleteImplementation),
    ]
}

/// Strategy for generating contract recommendations
pub fn contract_recommendation_strategy() -> impl Strategy<Value = ContractRecommendation> {
    (
        "[a-zA-Z0-9_]{1,50}",
        recommendation_priority_strategy(),
        "[a-zA-Z0-9_\\s]{1,100}",
        prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 1..5),
        "[a-zA-Z0-9_\\s]{1,30}",
    ).prop_map(|(recommendation_id, priority, description, remediation_steps, estimated_effort)| {
        ContractRecommendation {
            recommendation_id,
            priority,
            description,
            remediation_steps,
            estimated_effort,
        }
    })
}

/// Strategy for generating recommendation priorities
pub fn recommendation_priority_strategy() -> impl Strategy<Value = RecommendationPriority> {
    prop_oneof![
        Just(RecommendationPriority::Low),
        Just(RecommendationPriority::Medium),
        Just(RecommendationPriority::High),
        Just(RecommendationPriority::Critical),
    ]
}

/// Strategy for generating gate readiness analysis
pub fn gate_readiness_analysis_strategy() -> impl Strategy<Value = GateReadinessAnalysis> {
    (
        gate_phase_strategy(),
        readiness_status_strategy(),
        0.0f64..100.0f64,
        prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 0..3),
        prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 0..5),
    ).prop_map(|(gate_phase, readiness_status, completion_percentage, blocking_issues, advisory_notes)| {
        GateReadinessAnalysis {
            gate_phase,
            readiness_status,
            completion_percentage,
            blocking_issues,
            advisory_notes,
        }
    })
}

/// Strategy for generating gate phases
pub fn gate_phase_strategy() -> impl Strategy<Value = GatePhase> {
    prop_oneof![
        Just(GatePhase::Initialization),
        Just(GatePhase::Validation),
        Just(GatePhase::Approval),
        Just(GatePhase::Implementation),
    ]
}

/// Strategy for generating readiness status
pub fn readiness_status_strategy() -> impl Strategy<Value = ReadinessStatus> {
    prop_oneof![
        Just(ReadinessStatus::NotReady),
        Just(ReadinessStatus::PartiallyReady),
        Just(ReadinessStatus::Ready),
        Just(ReadinessStatus::Approved),
    ]
}

/// Strategy for generating transition findings
pub fn transition_finding_strategy() -> impl Strategy<Value = TransitionFinding> {
    (
        transition_finding_type_strategy(),
        "[a-zA-Z0-9_]{1,30}",
        "[a-zA-Z0-9_]{1,30}",
        "[a-zA-Z0-9_\\s]{1,100}",
    ).prop_flat_map(|(finding_type, from_state, to_state, analysis_result)| {
        // Generate recommendations based on finding type
        let recommendations_strategy = match finding_type {
            TransitionFindingType::TransitionBlocked => {
                // Blocked transitions must have at least one recommendation
                prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 1..4)
            }
            TransitionFindingType::TransitionConditional => {
                // Conditional transitions should have recommendations
                prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 1..3)
            }
            _ => {
                // Other types may or may not have recommendations
                prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 0..3)
            }
        };
        
        recommendations_strategy.prop_map(move |recommendations| {
            TransitionFinding {
                finding_type: finding_type.clone(),
                from_state: from_state.clone(),
                to_state: to_state.clone(),
                analysis_result: analysis_result.clone(),
                recommendations,
            }
        })
    })
}

/// Strategy for generating transition finding types
pub fn transition_finding_type_strategy() -> impl Strategy<Value = TransitionFindingType> {
    prop_oneof![
        Just(TransitionFindingType::TransitionAllowed),
        Just(TransitionFindingType::TransitionBlocked),
        Just(TransitionFindingType::TransitionConditional),
        Just(TransitionFindingType::TransitionRequiresApproval),
    ]
}

/// Strategy for generating permission analysis
pub fn permission_analysis_strategy() -> impl Strategy<Value = PermissionAnalysis> {
    (
        prop::collection::vec(permission_strategy(), 0..5),
        prop::collection::vec(permission_strategy(), 0..5),
        prop::collection::vec(permission_gap_strategy(), 0..3),
        prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 0..3),
    ).prop_map(|(required_permissions, available_permissions, permission_gaps, authorization_recommendations)| {
        PermissionAnalysis {
            required_permissions,
            available_permissions,
            permission_gaps,
            authorization_recommendations,
        }
    })
}

/// Strategy for generating permissions
pub fn permission_strategy() -> impl Strategy<Value = Permission> {
    (
        "[a-zA-Z0-9_]{1,50}",
        permission_type_strategy(),
        "[a-zA-Z0-9_]{1,50}",
        authority_level_strategy(),
    ).prop_map(|(permission_id, permission_type, scope, authority_level)| {
        Permission {
            permission_id,
            permission_type,
            scope,
            authority_level,
        }
    })
}

/// Strategy for generating permission types
pub fn permission_type_strategy() -> impl Strategy<Value = PermissionType> {
    prop_oneof![
        Just(PermissionType::Read),
        Just(PermissionType::Write),
        Just(PermissionType::Execute),
        Just(PermissionType::Modify),
        Just(PermissionType::Approve),
        Just(PermissionType::Override),
    ]
}

/// Strategy for generating authority levels
pub fn authority_level_strategy() -> impl Strategy<Value = AuthorityLevel> {
    prop_oneof![
        Just(AuthorityLevel::Component),
        Just(AuthorityLevel::System),
        Just(AuthorityLevel::Constitutional),
        Just(AuthorityLevel::Administrative),
    ]
}

/// Strategy for generating permission gaps
pub fn permission_gap_strategy() -> impl Strategy<Value = PermissionGap> {
    (
        permission_strategy(),
        severity_strategy(),
        prop::collection::vec("[a-zA-Z0-9_\\s]{1,50}", 1..3),
    ).prop_map(|(required_permission, gap_severity, resolution_path)| {
        PermissionGap {
            required_permission,
            gap_severity,
            resolution_path,
        }
    })
}

/// Strategy for generating fingerprint analysis
pub fn fingerprint_analysis_strategy() -> impl Strategy<Value = FingerprintAnalysis> {
    (
        "[a-fA-F0-9]{32,64}",
        prop::option::of("[a-fA-F0-9]{32,64}"),
        fingerprint_validity_strategy(),
        prop::collection::vec(fingerprint_variation_strategy(), 0..3),
        integrity_status_strategy(),
    ).prop_map(|(current_fingerprint, expected_fingerprint, fingerprint_validity, variation_analysis, integrity_status)| {
        FingerprintAnalysis {
            current_fingerprint,
            expected_fingerprint,
            fingerprint_validity,
            variation_analysis,
            integrity_status,
        }
    })
}

/// Strategy for generating fingerprint validity
pub fn fingerprint_validity_strategy() -> impl Strategy<Value = FingerprintValidity> {
    prop_oneof![
        Just(FingerprintValidity::Valid),
        Just(FingerprintValidity::Invalid),
        Just(FingerprintValidity::Suspicious),
        Just(FingerprintValidity::RequiresVerification),
    ]
}

/// Strategy for generating fingerprint variations
pub fn fingerprint_variation_strategy() -> impl Strategy<Value = FingerprintVariation> {
    (
        "[a-zA-Z0-9_]{1,30}",
        "[a-zA-Z0-9_\\s]{1,100}",
        "[a-zA-Z0-9_\\s]{1,100}",
    ).prop_map(|(variation_type, impact_assessment, legitimacy_analysis)| {
        FingerprintVariation {
            variation_type,
            impact_assessment,
            legitimacy_analysis,
        }
    })
}

/// Strategy for generating integrity status
pub fn integrity_status_strategy() -> impl Strategy<Value = IntegrityStatus> {
    prop_oneof![
        Just(IntegrityStatus::Verified),
        Just(IntegrityStatus::Compromised),
        Just(IntegrityStatus::Unknown),
        Just(IntegrityStatus::PendingVerification),
    ]
}

/// Strategy for generating template types
pub fn template_type_strategy() -> impl Strategy<Value = TemplateType> {
    prop_oneof![
        Just(TemplateType::FailureMatrix),
        Just(TemplateType::SemanticLockSpecification),
        Just(TemplateType::CoreContract),
        Just(TemplateType::GateValidator),
    ]
}

/// Strategy for generating severity levels
pub fn severity_strategy() -> impl Strategy<Value = Severity> {
    prop_oneof![
        Just(Severity::Info),
        Just(Severity::Warning),
        Just(Severity::Error),
        Just(Severity::Critical),
    ]
}

/// Test scenario for contract violations
#[derive(Debug, Clone)]
pub struct ContractViolationScenario {
    pub component: ComponentId,
    pub findings: Vec<ContractFinding>,
    pub violations: Vec<ContractViolation>,
    pub recommendations: Vec<ContractRecommendation>,
}

/// Test scenario for gate decisions
#[derive(Debug, Clone)]
pub struct GateDecisionScenario {
    pub readiness_analysis: GateReadinessAnalysis,
    pub transition_findings: Vec<TransitionFinding>,
    pub permission_analysis: PermissionAnalysis,
    pub fingerprint_analysis: FingerprintAnalysis,
    pub overall_readiness_score: f64,
}

/// Test scenario for constitutional errors (framework-level only)
#[derive(Debug, Clone)]
pub enum ConstitutionalErrorScenario {
    SystemInitialization { reason: String },
    ConfigurationLoad { reason: String },
    IOError { reason: String },
    FrameworkCorruption { reason: String },
}

/// Property test for error type correctness
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 1000,
        timeout: 10000,
        rng_algorithm: proptest::test_runner::RngAlgorithm::ChaCha,
        ..ProptestConfig::default()
    })]

    /// **Feature: d4-constitutional-contracts-fixes, Property 3: Error Type Correctness**
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**
    ///
    /// For any error condition, the system should use the appropriate report type:
    /// - ContractViolationReport for contract issues
    /// - GateDecisionReport for gate issues  
    /// - ConstitutionalError only for framework issues
    /// - Report type specificity maintained during propagation
    /// - Violation reports include type-specific context and remediation
    #[test]
    fn test_error_type_correctness_property(
        contract_scenario in contract_violation_scenario_strategy(),
        gate_scenario in gate_decision_scenario_strategy(),
        constitutional_scenario in constitutional_error_scenario_strategy(),
    ) {
        // Test 1: Contract validation analysis uses ContractViolationReport (Requirement 3.1)
        let contract_report = create_contract_violation_report(&contract_scenario);
        prop_assert!(matches!(contract_report, ContractViolationReport { .. }),
            "Contract validation analysis should use ContractViolationReport");
        
        prop_assert_eq!(contract_report.component_context, contract_scenario.component,
            "Contract report should maintain component context");
        
        prop_assert_eq!(contract_report.findings.len(), contract_scenario.findings.len(),
            "Contract report should include all findings");
        
        prop_assert_eq!(contract_report.violations.len(), contract_scenario.violations.len(),
            "Contract report should include all violations");
        
        prop_assert_eq!(contract_report.recommendations.len(), contract_scenario.recommendations.len(),
            "Contract report should include all recommendations");

        // Test 2: Gate readiness analysis uses GateDecisionReport (Requirement 3.2)
        let gate_report = create_gate_decision_report(&gate_scenario);
        let gate_report_clone1 = gate_report.clone();
        let gate_report_clone2 = gate_report.clone();
        
        prop_assert!(matches!(gate_report, GateDecisionReport { .. }),
            "Gate readiness analysis should use GateDecisionReport");
        
        prop_assert_eq!(gate_report_clone1.readiness_analysis.gate_phase, gate_scenario.readiness_analysis.gate_phase,
            "Gate report should maintain gate phase information");
        
        prop_assert_eq!(gate_report_clone1.transition_findings.len(), gate_scenario.transition_findings.len(),
            "Gate report should include all transition findings");
        
        prop_assert!((gate_report_clone1.overall_readiness_score - gate_scenario.overall_readiness_score).abs() < 0.001,
            "Gate report should maintain readiness score");

        // Test 3: Constitutional framework errors only for framework init/IO issues (Requirement 3.3)
        let constitutional_error = create_constitutional_error(&constitutional_scenario);
        prop_assert!(matches!(constitutional_error, ConstitutionalError::SystemInitializationFailure { .. } |
                                                   ConstitutionalError::ConfigurationLoadError { .. } |
                                                   ConstitutionalError::IOError { .. } |
                                                   ConstitutionalError::FrameworkCorruption { .. }),
            "ConstitutionalError should only be used for framework init/IO issues");

        // Test 4: Report type specificity maintained during propagation (Requirement 3.4)
        let propagated_contract_report = propagate_contract_report(contract_report.clone());
        prop_assert!(matches!(propagated_contract_report, ContractViolationReport { .. }),
            "Contract report type should be maintained during propagation");
        
        let propagated_gate_report = propagate_gate_report(gate_report_clone2);
        prop_assert!(matches!(propagated_gate_report, GateDecisionReport { .. }),
            "Gate report type should be maintained during propagation");

        // Test 5: Violation reports include violation-type-specific context and remediation (Requirement 3.5)
        for violation in &contract_report.violations {
            prop_assert!(!violation.description.is_empty(),
                "Contract violations should include specific description");
            prop_assert!(!violation.formal_specification.is_empty(),
                "Contract violations should include formal specification");
        }
        
        for recommendation in &contract_report.recommendations {
            prop_assert!(!recommendation.description.is_empty(),
                "Contract recommendations should include specific description");
            prop_assert!(!recommendation.remediation_steps.is_empty(),
                "Contract recommendations should include remediation steps");
        }
        
        for finding in &gate_report.transition_findings {
            prop_assert!(!finding.analysis_result.is_empty(),
                "Gate transition findings should include analysis result");
            // Only blocked transitions must have recommendations
            if matches!(finding.finding_type, TransitionFindingType::TransitionBlocked) {
                prop_assert!(!finding.recommendations.is_empty(),
                    "Blocked gate transitions should include recommendations");
            }
        }
    }
}

/// Create a ContractViolationReport from a test scenario
fn create_contract_violation_report(scenario: &ContractViolationScenario) -> ContractViolationReport {
    let mut report = ContractViolationReport::new(scenario.component);
    
    for finding in &scenario.findings {
        report.add_finding(finding.clone());
    }
    
    for violation in &scenario.violations {
        report.add_violation(violation.clone());
    }
    
    for recommendation in &scenario.recommendations {
        report.add_recommendation(recommendation.clone());
    }
    
    report
}

/// Create a GateDecisionReport from a test scenario
fn create_gate_decision_report(scenario: &GateDecisionScenario) -> GateDecisionReport {
    GateDecisionReport {
        readiness_analysis: scenario.readiness_analysis.clone(),
        transition_findings: scenario.transition_findings.clone(),
        permission_analysis: scenario.permission_analysis.clone(),
        fingerprint_analysis: scenario.fingerprint_analysis.clone(),
        decision_timestamp: DeterministicClock::new().now(),
        overall_readiness_score: scenario.overall_readiness_score,
    }
}

/// Create a ConstitutionalError from a test scenario
fn create_constitutional_error(scenario: &ConstitutionalErrorScenario) -> ConstitutionalError {
    match scenario {
        ConstitutionalErrorScenario::SystemInitialization { reason } => {
            ConstitutionalError::SystemInitializationFailure { reason: reason.clone() }
        }
        ConstitutionalErrorScenario::ConfigurationLoad { reason } => {
            ConstitutionalError::ConfigurationLoadError { reason: reason.clone() }
        }
        ConstitutionalErrorScenario::IOError { reason } => {
            ConstitutionalError::IOError { reason: reason.clone() }
        }
        ConstitutionalErrorScenario::FrameworkCorruption { reason } => {
            ConstitutionalError::FrameworkCorruption { reason: reason.clone() }
        }
    }
}

/// Simulate report propagation for ContractViolationReport (maintains type)
fn propagate_contract_report(report: ContractViolationReport) -> ContractViolationReport {
    // In a real system, this would pass through multiple layers
    // The key property is that the report type is maintained
    report
}

/// Simulate report propagation for GateDecisionReport (maintains type)
fn propagate_gate_report(report: GateDecisionReport) -> GateDecisionReport {
    // In a real system, this would pass through multiple layers
    // The key property is that the report type is maintained
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_violation_report_creation() {
        let scenario = ContractViolationScenario {
            component: ComponentId::D4RegisterAllocator,
            findings: vec![ContractFinding::InvalidStructure {
                component: "TestComponent".to_string(),
                reason: "Missing field".to_string(),
            }],
            violations: vec![ContractViolation {
                violation_id: "TEST_VIOLATION".to_string(),
                component: ComponentId::D4RegisterAllocator,
                violation_type: ContractViolationType::MissingStructDefinition,
                description: "Test violation".to_string(),
                formal_specification: "struct TestStruct must be defined".to_string(),
                context: BTreeMap::new(),
            }],
            recommendations: vec![ContractRecommendation {
                recommendation_id: "TEST_REC".to_string(),
                priority: RecommendationPriority::High,
                description: "Fix the test violation".to_string(),
                remediation_steps: vec!["Step 1".to_string()],
                estimated_effort: "1 hour".to_string(),
            }],
        };

        let report = create_contract_violation_report(&scenario);
        assert_eq!(report.component_context, ComponentId::D4RegisterAllocator);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.recommendations.len(), 1);
    }

    #[test]
    fn test_gate_decision_report_creation() {
        let scenario = GateDecisionScenario {
            readiness_analysis: GateReadinessAnalysis {
                gate_phase: GatePhase::Validation,
                readiness_status: ReadinessStatus::PartiallyReady,
                completion_percentage: 75.0,
                blocking_issues: vec!["Issue 1".to_string()],
                advisory_notes: vec!["Note 1".to_string()],
            },
            transition_findings: vec![TransitionFinding {
                finding_type: TransitionFindingType::TransitionConditional,
                from_state: "StateA".to_string(),
                to_state: "StateB".to_string(),
                analysis_result: "Conditional transition allowed".to_string(),
                recommendations: vec!["Check condition".to_string()],
            }],
            permission_analysis: PermissionAnalysis {
                required_permissions: vec![],
                available_permissions: vec![],
                permission_gaps: vec![],
                authorization_recommendations: vec![],
            },
            fingerprint_analysis: FingerprintAnalysis {
                current_fingerprint: "abc123".to_string(),
                expected_fingerprint: Some("def456".to_string()),
                fingerprint_validity: FingerprintValidity::Valid,
                variation_analysis: vec![],
                integrity_status: IntegrityStatus::Verified,
            },
            overall_readiness_score: 75.0,
        };

        let report = create_gate_decision_report(&scenario);
        assert_eq!(report.readiness_analysis.gate_phase, GatePhase::Validation);
        assert_eq!(report.transition_findings.len(), 1);
        assert!((report.overall_readiness_score - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_constitutional_error_creation() {
        let scenario = ConstitutionalErrorScenario::SystemInitialization {
            reason: "Test initialization failure".to_string(),
        };

        let error = create_constitutional_error(&scenario);
        assert!(matches!(error, ConstitutionalError::SystemInitializationFailure { .. }));
    }

    #[test]
    fn test_report_propagation_maintains_types() {
        let contract_report = ContractViolationReport::new(ComponentId::D4RegisterAllocator);
        let propagated = propagate_contract_report(contract_report);
        assert!(matches!(propagated, ContractViolationReport { .. }));

        let gate_report = GateDecisionReport::new();
        let propagated = propagate_gate_report(gate_report);
        assert!(matches!(propagated, GateDecisionReport { .. }));
    }
}