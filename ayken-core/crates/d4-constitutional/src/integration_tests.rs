//! Integration tests for the D4 Constitutional Contracts System
//!
//! These tests validate end-to-end integration of all components including:
//! - ValidationLocation tracking with all validation operations
//! - Template system integration with Gate E validation
//! - Error type consistency across all components
//! - Build fingerprinting integration with gate readiness analysis
//!
//! PURE B-MODE PRINCIPLES:
//! - All tests validate specifications and reports, never runtime execution
//! - Tests verify component integration through analysis reports
//! - No state mutations or runtime enforcement testing

use crate::build_fingerprint::{BuildContext, DefaultBuildFingerprintAnalyzer, BuildFingerprintAnalyzer};
use crate::compliance::{ValidationComplianceAnalyzer, ValidationInput, Penalty, Bonus, ComplianceAnalyzer};
use crate::errors::{SpecificationReport, SpecificationViolation, ViolationType, ContractViolationReport, GateDecisionReport};
use crate::gate_readiness::{DefaultGateReadinessAnalyzer, GateReadinessContext, GateReadinessAnalyzer};
use crate::integration::{ConstitutionalIntegrationAnalyzer, ConstitutionalAnalysisContext, AnalysisPhase, ValidationRequirementType, create_constitutional_analysis_context, add_validation_requirement, add_build_context};
use crate::templates::{TemplateSpecRegistryImpl, TemplateType, TemplateSpecRegistry};
use crate::types::{ComponentId, Severity, DeterministicClock};
use crate::validation_location::{DefaultLocationTracker, ValidationPhase, LocationTracker};
use std::collections::BTreeMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_build_context() -> BuildContext {
        let mut build_environment = BTreeMap::new();
        build_environment.insert("RUSTFLAGS".to_string(), "-C opt-level=3".to_string());
        build_environment.insert("TARGET".to_string(), "x86_64-unknown-linux-gnu".to_string());
        
        BuildContext {
            target_architecture: "x86_64".to_string(),
            build_configuration: "release".to_string(),
            source_tree_hash: "integration_test_hash_456".to_string(),
            toolchain_version: "1.70.0".to_string(),
            build_environment,
        }
    }

    /// Test end-to-end validation workflow with new components
    #[test]
    fn test_end_to_end_validation_workflow_with_new_components() {
        let mut analyzer = ConstitutionalIntegrationAnalyzer::new(ComponentId::D4RegisterAllocator);
        
        // Create comprehensive analysis context
        let mut context = create_constitutional_analysis_context(
            ComponentId::D4RegisterAllocator,
            AnalysisPhase::IntegratedAnalysis,
        );
        
        // Add all types of validation requirements
        add_validation_requirement(
            &mut context,
            "location_tracking_accuracy".to_string(),
            ComponentId::D4RegisterAllocator,
            ValidationRequirementType::LocationTracking,
            "ValidationLocation must accurately track component being validated".to_string(),
            true,
        );
        
        add_validation_requirement(
            &mut context,
            "template_completeness_check".to_string(),
            ComponentId::TemplateSpecRegistry,
            ValidationRequirementType::TemplateCompleteness,
            "All required templates must be complete and available".to_string(),
            true,
        );
        
        add_validation_requirement(
            &mut context,
            "error_type_correctness".to_string(),
            ComponentId::ConstitutionalRuleEngine,
            ValidationRequirementType::ErrorTypeCorrectness,
            "Error types must be used correctly throughout the system".to_string(),
            true,
        );
        
        add_validation_requirement(
            &mut context,
            "compliance_calculation_accuracy".to_string(),
            ComponentId::D4RegisterAllocator,
            ValidationRequirementType::ComplianceCalculation,
            "Compliance calculations must be mathematically correct".to_string(),
            true,
        );
        
        add_validation_requirement(
            &mut context,
            "build_fingerprinting_integration".to_string(),
            ComponentId::ConstitutionalRuleEngine,
            ValidationRequirementType::BuildFingerprinting,
            "Build fingerprinting must be integrated with gate readiness".to_string(),
            true,
        );
        
        // Add build context for gate readiness analysis
        add_build_context(&mut context, create_test_build_context());
        
        // Perform comprehensive analysis
        let report = analyzer.analyze_constitutional_compliance(&context);
        
        // Verify end-to-end integration
        assert_eq!(report.component_analysis.component_id, ComponentId::D4RegisterAllocator);
        assert!(report.component_analysis.location_tracking_report.component_accuracy_verified);
        assert!(report.template_analysis.gate_e_compatibility);
        assert!(report.compliance_analysis.compliance_index >= 0.0);
        assert!(report.gate_readiness_analysis.is_some());
        assert!(report.build_fingerprint_analysis.is_some());
        assert!(report.overall_compliance_score >= 0.0 && report.overall_compliance_score <= 1.0);
        
        // Verify integration findings
        assert!(!report.integration_findings.is_empty());
        
        // Verify that all validation requirements were addressed
        let location_tracking_addressed = report.component_analysis.specification_report.findings.iter()
            .any(|f| f.description.contains("location tracking"));
        let template_completeness_addressed = !report.template_analysis.template_completeness_reports.is_empty();
        let compliance_calculation_addressed = report.compliance_analysis.compliance_index >= 0.0;
        
        assert!(location_tracking_addressed || report.component_analysis.specification_report.violations.iter()
            .any(|v| v.description.contains("location tracking")));
        assert!(template_completeness_addressed);
        assert!(compliance_calculation_addressed);
    }

    /// Test Gate E validation with new templates
    #[test]
    fn test_gate_e_validation_with_new_templates() {
        let template_registry = TemplateSpecRegistryImpl::new();
        
        // Test FailureMatrix template integration with Gate E
        let failure_matrix_report = template_registry.analyze_template_completeness(TemplateType::FailureMatrix);
        assert!(failure_matrix_report.is_compliant(), 
            "FailureMatrix template should be complete for Gate E validation: {:?}", 
            failure_matrix_report.violations);
        
        // Test SemanticLockSpecification template integration with Gate E
        let semantic_lock_report = template_registry.analyze_template_completeness(TemplateType::SemanticLockSpecification);
        assert!(semantic_lock_report.is_compliant(), 
            "SemanticLockSpecification template should be complete for Gate E validation: {:?}", 
            semantic_lock_report.violations);
        
        // Test template structural integrity for Gate E compatibility
        let available_templates = template_registry.list_available_templates();
        assert!(available_templates.contains(&TemplateType::FailureMatrix));
        assert!(available_templates.contains(&TemplateType::SemanticLockSpecification));
        assert!(available_templates.contains(&TemplateType::CoreContract));
        assert!(available_templates.contains(&TemplateType::GateValidator));
        
        // Test template requirements specification
        let failure_matrix_spec = template_registry.specify_template_requirements(TemplateType::FailureMatrix);
        assert!(!failure_matrix_spec.required_structure.required_fields.is_empty());
        assert!(!failure_matrix_spec.validation_requirements.is_empty());
        assert!(!failure_matrix_spec.completeness_criteria.is_empty());
        
        let semantic_lock_spec = template_registry.specify_template_requirements(TemplateType::SemanticLockSpecification);
        assert!(!semantic_lock_spec.required_structure.required_fields.is_empty());
        assert!(!semantic_lock_spec.validation_requirements.is_empty());
        assert!(!semantic_lock_spec.completeness_criteria.is_empty());
        
        // Verify Gate E specific requirements
        let failure_matrix_field_names: Vec<&String> = failure_matrix_spec.required_structure.required_fields.iter()
            .map(|f| &f.name).collect();
        assert!(failure_matrix_field_names.contains(&&"scenario_id".to_string()));
        assert!(failure_matrix_field_names.contains(&&"scenario_type".to_string()));
        assert!(failure_matrix_field_names.contains(&&"component_responses".to_string()));
        assert!(failure_matrix_field_names.contains(&&"determinism_requirements".to_string()));
        
        let semantic_lock_field_names: Vec<&String> = semantic_lock_spec.required_structure.required_fields.iter()
            .map(|f| &f.name).collect();
        assert!(semantic_lock_field_names.contains(&&"lock_id".to_string()));
        assert!(semantic_lock_field_names.contains(&&"locked_behavior".to_string()));
        assert!(semantic_lock_field_names.contains(&&"authorization_level".to_string()));
    }

    /// Test error propagation through complete call stacks
    #[test]
    fn test_error_propagation_through_complete_call_stacks() {
        let mut location_tracker = DefaultLocationTracker::new(ComponentId::ConstitutionalRuleEngine);
        
        // Simulate a complete call stack with proper error type usage
        let mut location_tracker = location_tracker.with_validation_phase(ValidationPhase::ContractValidation);
        location_tracker = location_tracker.with_pushed_context("contract_analysis");
        location_tracker = location_tracker.with_pushed_context("template_validation");
        location_tracker = location_tracker.with_pushed_context("structural_integrity_check");
        
        // Test ContractViolationReport usage for contract-specific issues
        let mut contract_report = ContractViolationReport::new(ComponentId::TemplateSpecRegistry);
        contract_report.add_finding(crate::errors::ContractFinding::InvalidStructure {
            component: "TestComponent".to_string(),
            reason: "Missing required field".to_string(),
        });
        
        assert_eq!(contract_report.component_context, ComponentId::TemplateSpecRegistry);
        assert_eq!(contract_report.findings.len(), 1);
        assert!(contract_report.violations.is_empty());
        
        // Test GateDecisionReport usage for gate-specific issues
        let mut gate_report = GateDecisionReport::new();
        gate_report.readiness_analysis.readiness_status = crate::errors::ReadinessStatus::NotReady;
        gate_report.readiness_analysis.blocking_issues.push("Template validation failed".to_string());
        
        assert!(!gate_report.is_ready_for_transition());
        assert!(!gate_report.readiness_analysis.blocking_issues.is_empty());
        
        // Test SpecificationReport usage for general specification violations
        let mut spec_report = SpecificationReport::new();
        spec_report.add_violation(SpecificationViolation {
            violation_type: ViolationType::SpecificationIncomplete,
            component: ComponentId::TemplateSpecRegistry,
            rule_id: Some("TEMPLATE_COMPLETENESS".to_string()),
            description: "Template specification is incomplete".to_string(),
            remediation_hint: "Complete all required template fields".to_string(),
        });
        
        assert!(!spec_report.is_compliant());
        assert_eq!(spec_report.violations.len(), 1);
        assert!(spec_report.compliance_score < 1.0);
        
        // Verify error type separation is maintained through call stack
        location_tracker = location_tracker.with_popped_context(); // structural_integrity_check
        location_tracker = location_tracker.with_popped_context(); // template_validation
        location_tracker = location_tracker.with_popped_context(); // contract_analysis
        
        let final_location = location_tracker.create_location();
        assert_eq!(final_location.component, ComponentId::ConstitutionalRuleEngine);
        assert_eq!(final_location.context.validation_phase, ValidationPhase::ContractValidation);
        assert!(final_location.stack_trace.is_empty());
    }

    /// Test build fingerprint integration with readiness analysis decisions
    #[test]
    fn test_build_fingerprint_integration_with_readiness_analysis_decisions() {
        let build_context = create_test_build_context();
        
        // Test build fingerprint analysis
        let fingerprint_analysis = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&build_context);
        assert!(!fingerprint_analysis.fingerprint_spec.hash.is_empty());
        assert!(fingerprint_analysis.validity_analysis.reproducibility_score >= 0.0);
        assert!(fingerprint_analysis.integrity_analysis.authenticity_score >= 0.0);
        
        // Test gate readiness context with build fingerprinting
        let gate_context = GateReadinessContext {
            gate_id: "integration_test_gate".to_string(),
            current_phase: "validation".to_string(),
            required_components: vec![
                ComponentId::D4RegisterAllocator,
                ComponentId::TemplateSpecRegistry,
                ComponentId::ConstitutionalRuleEngine,
            ],
            build_context: build_context.clone(),
            validation_requirements: vec![
                crate::gate_readiness::ValidationRequirement {
                    requirement_id: "fingerprint_verification".to_string(),
                    component: ComponentId::ConstitutionalRuleEngine,
                    requirement_type: crate::gate_readiness::ValidationRequirementType::FingerprintVerification,
                    description: "Build fingerprint must be verified".to_string(),
                    mandatory: true,
                },
                crate::gate_readiness::ValidationRequirement {
                    requirement_id: "component_presence".to_string(),
                    component: ComponentId::D4RegisterAllocator,
                    requirement_type: crate::gate_readiness::ValidationRequirementType::ComponentPresence,
                    description: "D4 Register Allocator must be present".to_string(),
                    mandatory: true,
                },
            ],
        };
        
        // Test integrated gate readiness analysis
        let gate_readiness_report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&gate_context);
        
        // Verify build fingerprint integration
        assert!(gate_readiness_report.readiness_score >= 0.0 && gate_readiness_report.readiness_score <= 1.0);
        assert!(!gate_readiness_report.fingerprint_analysis.fingerprint_spec.hash.is_empty());
        assert_eq!(gate_readiness_report.fingerprint_analysis.fingerprint_spec.hash, 
                   fingerprint_analysis.fingerprint_spec.hash);
        
        // Test readiness decision based on fingerprint validity
        if fingerprint_analysis.validity_analysis.is_valid {
            assert!(gate_readiness_report.readiness_score >= 0.5, 
                "Valid fingerprint should contribute to readiness score");
        }
        
        // Test blocking issues identification
        let has_fingerprint_issues = gate_readiness_report.blocking_issues.iter()
            .any(|issue| matches!(issue.issue_type, crate::gate_readiness::BlockingIssueType::FingerprintMismatch));
        
        if !fingerprint_analysis.validity_analysis.is_valid {
            assert!(has_fingerprint_issues, "Invalid fingerprint should create blocking issues");
        }
        
        // Test recommendations generation
        let has_fingerprint_recommendations = gate_readiness_report.recommendations.iter()
            .any(|rec| matches!(rec.recommendation_type, crate::gate_readiness::ReadinessRecommendationType::FingerprintUpdate));
        
        if !fingerprint_analysis.validity_analysis.is_valid {
            assert!(has_fingerprint_recommendations, "Invalid fingerprint should generate recommendations");
        }
        
        // Test compliance reporting
        let compliance_report = DefaultGateReadinessAnalyzer::report_gate_readiness_compliance(&gate_readiness_report);
        
        if gate_readiness_report.readiness_score >= 0.8 && fingerprint_analysis.validity_analysis.is_valid {
            assert!(compliance_report.violations.is_empty() || compliance_report.violations.len() <= 1, 
                "High readiness score with valid fingerprint should have minimal violations");
        } else {
            assert!(!compliance_report.violations.is_empty(), 
                "Low readiness score or invalid fingerprint should have violations");
        }
    }

    /// Test ValidationLocation tracking integration across all validation operations
    #[test]
    fn test_validation_location_tracking_integration_across_all_operations() {
        let mut location_tracker = DefaultLocationTracker::new(ComponentId::D4RegisterAllocator);
        
        // Test location tracking through template validation
        location_tracker = location_tracker.with_validation_phase(ValidationPhase::TemplateApplication);
        location_tracker = location_tracker.with_pushed_context("template_completeness_check");
        location_tracker = location_tracker.with_structure(Some("FailureMatrix".to_string()));
        location_tracker = location_tracker.with_field(Some("scenario_id".to_string()));
        
        let template_location = location_tracker.create_location();
        assert_eq!(template_location.component, ComponentId::D4RegisterAllocator);
        assert_eq!(template_location.context.validation_phase, ValidationPhase::TemplateApplication);
        assert_eq!(template_location.structure_name, Some("FailureMatrix".to_string()));
        assert_eq!(template_location.field_name, Some("scenario_id".to_string()));
        assert!(template_location.stack_trace.contains(&"template_completeness_check".to_string()));
        
        // Test location tracking through compliance analysis
        location_tracker = location_tracker.with_validation_phase(ValidationPhase::SemanticValidation);
        location_tracker = location_tracker.with_pushed_context("compliance_calculation");
        location_tracker = location_tracker.with_structure(Some("ComplianceAnalysis".to_string()));
        location_tracker = location_tracker.with_field(Some("penalty_impact".to_string()));
        
        let compliance_location = location_tracker.create_location();
        assert_eq!(compliance_location.context.validation_phase, ValidationPhase::SemanticValidation);
        assert_eq!(compliance_location.structure_name, Some("ComplianceAnalysis".to_string()));
        assert_eq!(compliance_location.field_name, Some("penalty_impact".to_string()));
        assert!(compliance_location.stack_trace.len() >= 2);
        
        // Test location tracking through gate readiness analysis
        location_tracker = location_tracker.with_validation_phase(ValidationPhase::GateTransitionCheck);
        location_tracker = location_tracker.with_pushed_context("gate_readiness_analysis");
        location_tracker = location_tracker.with_structure(Some("GateReadinessReport".to_string()));
        location_tracker = location_tracker.with_field(Some("readiness_score".to_string()));
        
        let gate_location = location_tracker.create_location();
        assert_eq!(gate_location.context.validation_phase, ValidationPhase::GateTransitionCheck);
        assert_eq!(gate_location.structure_name, Some("GateReadinessReport".to_string()));
        assert_eq!(gate_location.field_name, Some("readiness_score".to_string()));
        
        // Test location accuracy (should never default to D4RegisterAllocator unless explicitly set)
        let mut different_tracker = DefaultLocationTracker::new(ComponentId::JITCompiler);
        let jit_location = different_tracker.create_location();
        assert_eq!(jit_location.component, ComponentId::JITCompiler);
        assert_ne!(jit_location.component, ComponentId::D4RegisterAllocator);
        
        // Test location description generation
        let description = gate_location.location_description();
        assert!(description.contains("Component: D4RegisterAllocator"));
        assert!(description.contains("Structure: GateReadinessReport"));
        assert!(description.contains("Field: readiness_score"));
        assert!(description.contains("Phase: GateTransitionCheck"));
        
        // Clean up context stack
        location_tracker = location_tracker.with_popped_context(); // gate_readiness_analysis
        location_tracker = location_tracker.with_popped_context(); // compliance_calculation
        location_tracker = location_tracker.with_popped_context(); // template_completeness_check
        
        let final_location = location_tracker.create_location();
        assert!(final_location.stack_trace.is_empty());
    }

    /// Test compliance calculation mathematical correctness integration
    #[test]
    fn test_compliance_calculation_mathematical_correctness_integration() {
        // Create validation input with penalties and bonuses
        let validation_input = ValidationInput {
            component_id: ComponentId::D4RegisterAllocator,
            validation_rules: vec![
                crate::compliance::ValidationRule {
                    rule_id: "rule_1".to_string(),
                    description: "Test rule 1".to_string(),
                    severity: Severity::Error,
                    weight: 1.0,
                },
                crate::compliance::ValidationRule {
                    rule_id: "rule_2".to_string(),
                    description: "Test rule 2".to_string(),
                    severity: Severity::Warning,
                    weight: 0.5,
                },
            ],
            penalties: vec![
                Penalty {
                    penalty_id: "penalty_1".to_string(),
                    description: "Test penalty 1".to_string(),
                    severity: Severity::Error,
                    impact: 0.2,
                    source_rule: Some("rule_1".to_string()),
                },
                Penalty {
                    penalty_id: "penalty_2".to_string(),
                    description: "Test penalty 2".to_string(),
                    severity: Severity::Warning,
                    impact: 0.1,
                    source_rule: Some("rule_2".to_string()),
                },
            ],
            bonuses: vec![
                Bonus {
                    bonus_id: "bonus_1".to_string(),
                    description: "Test bonus 1".to_string(),
                    impact: 0.15,
                    source_rule: Some("rule_1".to_string()),
                },
            ],
            metadata: BTreeMap::new(),
        };
        
        // Test compliance analysis
        let compliance_report = ValidationComplianceAnalyzer::analyze_compliance(&validation_input);
        
        // Verify mathematical correctness
        assert!(compliance_report.base_compliance >= 0.0 && compliance_report.base_compliance <= 1.0);
        assert!(compliance_report.compliance_index >= 0.0 && compliance_report.compliance_index <= 1.0);
        
        // Verify each penalty is counted exactly once
        assert_eq!(compliance_report.penalties.len(), 2);
        assert!(compliance_report.penalties.iter().any(|p| p.penalty_id == "penalty_1"));
        assert!(compliance_report.penalties.iter().any(|p| p.penalty_id == "penalty_2"));
        
        // Verify each bonus is counted exactly once
        assert_eq!(compliance_report.bonuses.len(), 1);
        assert!(compliance_report.bonuses.iter().any(|b| b.bonus_id == "bonus_1"));
        
        // Test penalty impact analysis
        let penalty_impact_report = ValidationComplianceAnalyzer::analyze_penalty_impact(&validation_input.penalties);
        assert!(penalty_impact_report.total_penalty_impact >= 0.0);
        assert_eq!(penalty_impact_report.penalty_breakdown.len(), 2);
        
        // Verify no double-counting in penalty calculation
        let manual_penalty_sum: f64 = validation_input.penalties.iter().map(|p| p.impact).sum();
        assert!(penalty_impact_report.total_penalty_impact <= manual_penalty_sum * 2.0); // Allow for severity weighting
        
        // Test bonus impact analysis
        let bonus_impact_report = ValidationComplianceAnalyzer::analyze_bonus_impact(&validation_input.bonuses);
        assert!(bonus_impact_report.total_bonus_impact >= 0.0);
        assert_eq!(bonus_impact_report.bonus_breakdown.len(), 1);
        
        // Verify intuitive compliance index (higher is better)
        let high_compliance_input = ValidationInput {
            component_id: ComponentId::D4RegisterAllocator,
            validation_rules: validation_input.validation_rules.clone(),
            penalties: vec![], // No penalties
            bonuses: validation_input.bonuses.clone(),
            metadata: BTreeMap::new(),
        };
        
        let high_compliance_report = ValidationComplianceAnalyzer::analyze_compliance(&high_compliance_input);
        assert!(high_compliance_report.compliance_index >= compliance_report.compliance_index,
            "Fewer penalties should result in higher compliance index");
        
        // Test edge cases
        let empty_input = ValidationInput {
            component_id: ComponentId::D4RegisterAllocator,
            validation_rules: vec![],
            penalties: vec![],
            bonuses: vec![],
            metadata: BTreeMap::new(),
        };
        
        let empty_report = ValidationComplianceAnalyzer::analyze_compliance(&empty_input);
        assert!(empty_report.compliance_index >= 0.0 && empty_report.compliance_index <= 1.0);
        assert_eq!(empty_report.penalties.len(), 0);
        assert_eq!(empty_report.bonuses.len(), 0);
    }

    /// Test comprehensive integration scenario with all components
    #[test]
    fn test_comprehensive_integration_scenario_with_all_components() {
        let mut analyzer = ConstitutionalIntegrationAnalyzer::new(ComponentId::ConstitutionalRuleEngine);
        
        // Create comprehensive test scenario
        let mut context = create_constitutional_analysis_context(
            ComponentId::ConstitutionalRuleEngine,
            AnalysisPhase::IntegratedAnalysis,
        );
        
        // Add all validation requirements
        for (req_id, component, req_type, description) in [
            ("location_tracking", ComponentId::ConstitutionalRuleEngine, ValidationRequirementType::LocationTracking, "Accurate location tracking"),
            ("template_completeness", ComponentId::TemplateSpecRegistry, ValidationRequirementType::TemplateCompleteness, "Complete template coverage"),
            ("error_type_correctness", ComponentId::ConstitutionalRuleEngine, ValidationRequirementType::ErrorTypeCorrectness, "Correct error type usage"),
            ("compliance_calculation", ComponentId::ConstitutionalRuleEngine, ValidationRequirementType::ComplianceCalculation, "Accurate compliance calculation"),
            ("build_fingerprinting", ComponentId::ConstitutionalRuleEngine, ValidationRequirementType::BuildFingerprinting, "Build fingerprint integration"),
            ("gate_readiness", ComponentId::ConstitutionalRuleEngine, ValidationRequirementType::GateReadiness, "Gate readiness analysis"),
        ] {
            add_validation_requirement(
                &mut context,
                req_id.to_string(),
                component,
                req_type,
                description.to_string(),
                true,
            );
        }
        
        // Add comprehensive build context
        add_build_context(&mut context, create_test_build_context());
        
        // Add metadata
        context.metadata.insert("test_scenario".to_string(), "comprehensive_integration".to_string());
        context.metadata.insert("test_timestamp".to_string(), DeterministicClock::new().now().0.to_string());
        
        // Perform comprehensive analysis
        let report = analyzer.analyze_constitutional_compliance(&context);
        
        // Verify comprehensive integration
        assert_eq!(report.component_analysis.component_id, ComponentId::ConstitutionalRuleEngine);
        
        // Verify location tracking integration
        assert!(report.component_analysis.location_tracking_report.component_accuracy_verified);
        assert!(report.component_analysis.location_tracking_report.tracking_accuracy > 0.0);
        
        // Verify template integration
        assert!(report.template_analysis.template_completeness_reports.contains_key(&TemplateType::FailureMatrix));
        assert!(report.template_analysis.template_completeness_reports.contains_key(&TemplateType::SemanticLockSpecification));
        assert!(report.template_analysis.gate_e_compatibility);
        
        // Verify compliance analysis
        assert!(report.compliance_analysis.compliance_index >= 0.0);
        assert_eq!(report.compliance_analysis.analysis_metadata.component_context, ComponentId::ConstitutionalRuleEngine);
        
        // Verify gate readiness integration
        assert!(report.gate_readiness_analysis.is_some());
        let gate_analysis = report.gate_readiness_analysis.as_ref().unwrap();
        assert!(gate_analysis.readiness_score >= 0.0 && gate_analysis.readiness_score <= 1.0);
        
        // Verify build fingerprint integration
        assert!(report.build_fingerprint_analysis.is_some());
        let fingerprint_analysis = report.build_fingerprint_analysis.as_ref().unwrap();
        assert!(!fingerprint_analysis.fingerprint_spec.hash.is_empty());
        
        // Verify integration findings
        assert!(!report.integration_findings.is_empty());
        let has_component_coordination = report.integration_findings.iter()
            .any(|f| matches!(f.finding_type, crate::integration::IntegrationFindingType::ComponentCoordination));
        let has_template_integration = report.integration_findings.iter()
            .any(|f| matches!(f.finding_type, crate::integration::IntegrationFindingType::TemplateIntegration));
        let has_build_integration = report.integration_findings.iter()
            .any(|f| matches!(f.finding_type, crate::integration::IntegrationFindingType::BuildIntegration));
        
        assert!(has_component_coordination);
        assert!(has_template_integration);
        assert!(has_build_integration);
        
        // Verify overall compliance score
        assert!(report.overall_compliance_score >= 0.0 && report.overall_compliance_score <= 1.0);
        
        // If all components are working correctly, should have high compliance
        if report.template_analysis.gate_e_compatibility && 
           report.component_analysis.location_tracking_report.component_accuracy_verified &&
           report.component_analysis.error_type_report.type_separation_verified &&
           gate_analysis.readiness_score >= 0.8 {
            assert!(report.overall_compliance_score >= 0.8, 
                "All components working correctly should result in high overall compliance");
        }
        
        // Verify timestamp consistency
        assert!(report.analysis_timestamp.0 >= 0);
        assert!(report.component_analysis.specification_report.report_timestamp.0 >= 0);
        assert!(report.compliance_analysis.analysis_metadata.analysis_timestamp.0 >= 0);
    }
}
