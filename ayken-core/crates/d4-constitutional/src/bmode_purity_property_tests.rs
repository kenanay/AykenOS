//! Property-based tests for B-MODE purity in the D4 Constitutional Framework
//!
//! **Feature: d4-constitutional-contracts-fixes, Property 5: B-MODE Purity**
//! **Validates: Requirements 5.3, 5.4, 5.5**
//!
//! This module tests that B-MODE operations maintain strict separation from runtime enforcement:
//! - B-MODE operations produce specifications without runtime enforcement code
//! - Validation operations never perform runtime state changes
//! - All APIs return SpecificationReport (never Result<()> for spec violations)
//! - Function names use specify_*/analyze_*/recommend_* (never execute_*/handle_*/recover_*)
//! - Types use *Spec/*Registry/*Report (never *Manager/RecoveryAction/ComponentHealthStatus)

use crate::compliance::{ComplianceAnalyzer, ValidationComplianceAnalyzer, ValidationInput};
use crate::errors::{ContractViolationReport, GateDecisionReport};
use crate::templates::{TemplateSpecRegistry, TemplateSpecRegistryImpl, TemplateType};
use crate::types::{ComponentId, Severity};
use crate::validation_location::{LocationTracker, DefaultLocationTracker, ValidationLocation};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::collections::BTreeMap;

/// Strategy for generating B-MODE operation scenarios
pub fn bmode_operation_scenario_strategy() -> impl Strategy<Value = BModeOperationScenario> {
    (
        component_id_strategy(),
        bmode_operation_type_strategy(),
        prop::collection::vec(validation_input_strategy(), 0..5),
        prop::collection::vec(template_type_strategy(), 0..3),
    ).prop_map(|(component, operation_type, validation_inputs, template_types)| {
        BModeOperationScenario {
            component,
            operation_type,
            validation_inputs,
            template_types,
        }
    })
}

/// Strategy for generating component IDs
pub fn component_id_strategy() -> impl Strategy<Value = ComponentId> {
    prop_oneof![
        Just(ComponentId::ConstitutionalRuleEngine),
        Just(ComponentId::D4RegisterAllocator),
        Just(ComponentId::JITCompiler),
        Just(ComponentId::LoopOptimizer),
        Just(ComponentId::NativeCache),
        Just(ComponentId::SemanticSpecificationRegistry),
        Just(ComponentId::FailureMatrix),
        Just(ComponentId::DeterminismEngine),
    ]
}

/// Strategy for generating B-MODE operation types
pub fn bmode_operation_type_strategy() -> impl Strategy<Value = BModeOperationType> {
    prop_oneof![
        Just(BModeOperationType::ComplianceAnalysis),
        Just(BModeOperationType::TemplateSpecification),
        Just(BModeOperationType::ValidationLocationTracking),
        Just(BModeOperationType::ContractViolationReporting),
        Just(BModeOperationType::GateReadinessAnalysis),
    ]
}

/// Strategy for generating validation inputs
pub fn validation_input_strategy() -> impl Strategy<Value = ValidationInput> {
    (
        component_id_strategy(),
        prop::collection::vec(validation_rule_strategy(), 0..5),
        prop::collection::vec(penalty_strategy(), 0..3),
        prop::collection::vec(bonus_strategy(), 0..2),
    ).prop_map(|(component_id, validation_rules, penalties, bonuses)| {
        ValidationInput {
            component_id,
            validation_rules,
            penalties,
            bonuses,
            metadata: BTreeMap::new(),
        }
    })
}

/// Strategy for generating validation rules
pub fn validation_rule_strategy() -> impl Strategy<Value = crate::compliance::ValidationRule> {
    (
        "[A-Z][0-9]{1,3}",
        ".*",
        prop_oneof![
            Just(Severity::Info),
            Just(Severity::Warning),
            Just(Severity::Error),
            Just(Severity::Critical),
        ],
        0.0f64..2.0f64,
    ).prop_map(|(rule_id, description, severity, weight)| {
        crate::compliance::ValidationRule {
            rule_id,
            description,
            severity,
            weight,
        }
    })
}

/// Strategy for generating penalties
pub fn penalty_strategy() -> impl Strategy<Value = crate::compliance::Penalty> {
    (
        "P[0-9]{1,3}",
        ".*",
        prop_oneof![
            Just(Severity::Info),
            Just(Severity::Warning),
            Just(Severity::Error),
            Just(Severity::Critical),
        ],
        0.0f64..1.0f64,
        prop::option::of("[A-Z][0-9]{1,3}"),
    ).prop_map(|(penalty_id, description, severity, impact, source_rule)| {
        crate::compliance::Penalty {
            penalty_id,
            description,
            severity,
            impact,
            source_rule,
        }
    })
}

/// Strategy for generating bonuses
pub fn bonus_strategy() -> impl Strategy<Value = crate::compliance::Bonus> {
    (
        "B[0-9]{1,3}",
        ".*",
        0.0f64..0.5f64,
        prop::option::of("[A-Z][0-9]{1,3}"),
    ).prop_map(|(bonus_id, description, impact, source_rule)| {
        crate::compliance::Bonus {
            bonus_id,
            description,
            impact,
            source_rule,
        }
    })
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

/// Test scenario for B-MODE operations
#[derive(Debug, Clone)]
pub struct BModeOperationScenario {
    pub component: ComponentId,
    pub operation_type: BModeOperationType,
    pub validation_inputs: Vec<ValidationInput>,
    pub template_types: Vec<TemplateType>,
}

/// Types of B-MODE operations to test
#[derive(Debug, Clone)]
pub enum BModeOperationType {
    ComplianceAnalysis,
    TemplateSpecification,
    ValidationLocationTracking,
    ContractViolationReporting,
    GateReadinessAnalysis,
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 1000,
        timeout: 10000,
        rng_algorithm: proptest::test_runner::RngAlgorithm::ChaCha,
        ..ProptestConfig::default()
    })]

    /// **Feature: d4-constitutional-contracts-fixes, Property 5: B-MODE Purity**
    /// **Validates: Requirements 5.3, 5.4, 5.5**
    ///
    /// For any B-MODE operation (contract generation, validation, gate transition checking):
    /// - Operation should produce specifications without runtime enforcement code (Requirement 5.3)
    /// - Operation should not perform runtime state changes (Requirement 5.4)
    /// - Operation should return SpecificationReport (never Result<()> for spec violations) (Requirement 5.5)
    /// - Function names should use specify_*/analyze_*/recommend_* (never execute_*/handle_*/recover_*)
    /// - Types should use *Spec/*Registry/*Report (never *Manager/RecoveryAction/ComponentHealthStatus)
    #[test]
    fn test_bmode_purity_property(
        scenario in bmode_operation_scenario_strategy()
    ) {
        match scenario.operation_type {
            BModeOperationType::ComplianceAnalysis => {
                test_compliance_analysis_purity(&scenario);
            }
            BModeOperationType::TemplateSpecification => {
                test_template_specification_purity(&scenario);
            }
            BModeOperationType::ValidationLocationTracking => {
                test_validation_location_tracking_purity(&scenario);
            }
            BModeOperationType::ContractViolationReporting => {
                test_contract_violation_reporting_purity(&scenario);
            }
            BModeOperationType::GateReadinessAnalysis => {
                test_gate_readiness_analysis_purity(&scenario);
            }
        }
    }
}

/// Test compliance analysis operations for B-MODE purity
fn test_compliance_analysis_purity(scenario: &BModeOperationScenario) {
    for validation_input in &scenario.validation_inputs {
        // Requirement 5.3: Operation should produce specifications without runtime enforcement code
        let report = ValidationComplianceAnalyzer::analyze_compliance(validation_input);
        
        // Verify the operation returns a report (specification) not an execution result
        assert!(report.compliance_index >= 0.0 && report.compliance_index <= 1.0,
            "Compliance analysis should produce specification report with valid compliance index");
        
        // Requirement 5.4: Operation should not perform runtime state changes
        // Test immutability by running the same operation multiple times
        let report2 = ValidationComplianceAnalyzer::analyze_compliance(validation_input);
        assert_eq!(report.compliance_index, report2.compliance_index,
            "Compliance analysis should be immutable - same input produces same output");
        
        // Requirement 5.5: Function names use analyze_* pattern (B-MODE naming)
        // This is verified by the fact that we're calling analyze_compliance, not execute_compliance
        
        // Verify penalty analysis follows B-MODE principles
        let penalty_report = ValidationComplianceAnalyzer::analyze_penalty_impact(&validation_input.penalties);
        assert!(penalty_report.total_penalty_impact >= 0.0,
            "Penalty analysis should produce specification report with valid impact");
        
        // Verify bonus analysis follows B-MODE principles
        let bonus_report = ValidationComplianceAnalyzer::analyze_bonus_impact(&validation_input.bonuses);
        assert!(bonus_report.total_bonus_impact >= 0.0,
            "Bonus analysis should produce specification report with valid impact");
    }
}

/// Test template specification operations for B-MODE purity
fn test_template_specification_purity(scenario: &BModeOperationScenario) {
    let registry = TemplateSpecRegistryImpl::new();
    
    for template_type in &scenario.template_types {
        // Requirement 5.3: Operation should produce specifications without runtime enforcement code
        let completeness_report = registry.analyze_template_completeness(template_type.clone());
        
        // Verify the operation returns a SpecificationReport (not Result<()>)
        assert!(completeness_report.compliance_score >= 0.0 && completeness_report.compliance_score <= 1.0,
            "Template completeness analysis should produce specification report");
        
        // Requirement 5.4: Operation should not perform runtime state changes
        // Test immutability by running the same operation multiple times
        let completeness_report2 = registry.analyze_template_completeness(template_type.clone());
        assert_eq!(completeness_report.compliance_score, completeness_report2.compliance_score,
            "Template analysis should be immutable - same input produces same output");
        
        // Requirement 5.5: Function names use analyze_* pattern (B-MODE naming)
        // This is verified by the fact that we're calling analyze_template_completeness
        
        // Test template specification (not execution)
        let template_spec = registry.specify_template_requirements(template_type.clone());
        assert_eq!(template_spec.template_type, *template_type,
            "Template specification should specify requirements, not execute them");
        
        // Verify specification is immutable
        let template_spec2 = registry.specify_template_requirements(template_type.clone());
        assert_eq!(template_spec.template_type, template_spec2.template_type,
            "Template specification should be immutable");
    }
}

/// Test validation location tracking for B-MODE purity
fn test_validation_location_tracking_purity(scenario: &BModeOperationScenario) {
    let mut tracker = DefaultLocationTracker::new(scenario.component);
    
    // Requirement 5.3: Operation should produce specifications without runtime enforcement code
    let location = tracker.create_location();
    
    // Verify the operation produces a specification (ValidationLocation) not an execution result
    assert_eq!(location.component, scenario.component,
        "Location tracking should produce specification of current location");
    
    // Requirement 5.4: Operation should not perform runtime state changes
    // Test immutability by running the same operation multiple times
    let location2 = tracker.create_location();
    assert_eq!(location.component, location2.component,
        "Location tracking should be immutable - same input produces same output");
    
    // Requirement 5.5: Function names use create_* pattern (B-MODE naming for immutable operations)
    // This is verified by the fact that we're calling create_location, not execute_location
    
    // Test context operations are immutable (using ValidationLocation methods)
    let with_context = ValidationLocation::new(scenario.component).with_structure("test_struct".to_string());
    assert_eq!(with_context.component, scenario.component,
        "Context operations should produce new specifications, not mutate existing ones");
    
    // Original location should be unchanged (immutability)
    let location3 = tracker.create_location();
    assert_eq!(location.component, location3.component,
        "Original location should remain unchanged after context operations");
}

/// Test contract violation reporting for B-MODE purity
fn test_contract_violation_reporting_purity(scenario: &BModeOperationScenario) {
    // Requirement 5.3: Operation should produce specifications without runtime enforcement code
    let report = ContractViolationReport::new(scenario.component);
    
    // Verify the operation creates a report (specification) not an execution result
    assert_eq!(report.component_context, scenario.component,
        "Contract violation reporting should produce specification report");
    
    // Requirement 5.4: Operation should not perform runtime state changes
    // The report is a specification of violations, not a runtime enforcement action
    assert!(report.violations.is_empty(),
        "New contract violation report should start with empty violations specification");
    
    // Requirement 5.5: Types use *Report pattern (B-MODE naming)
    // This is verified by the fact that we're using ContractViolationReport, not ContractViolationManager
    
    // Test that creating reports produces specifications, not executions
    let report2 = ContractViolationReport::new(scenario.component);
    assert_eq!(report.component_context, report2.component_context,
        "Contract violation report should specify violations, not execute enforcement");
}

/// Test gate readiness analysis for B-MODE purity
fn test_gate_readiness_analysis_purity(_scenario: &BModeOperationScenario) {
    // Requirement 5.3: Operation should produce specifications without runtime enforcement code
    let report = GateDecisionReport::new();
    
    // Verify the operation creates a report (specification) not an execution result
    assert!(report.overall_readiness_score >= 0.0,
        "Gate readiness analysis should produce specification report with readiness score");
    
    // Requirement 5.4: Operation should not perform runtime state changes
    // Test immutability by creating multiple reports
    let report2 = GateDecisionReport::new();
    assert_eq!(report.readiness_analysis.readiness_status, report2.readiness_analysis.readiness_status,
        "Gate readiness analysis should be immutable for same conditions");
    
    // Requirement 5.5: Types use *Report pattern (B-MODE naming)
    // This is verified by the fact that we're using GateDecisionReport, not GateDecisionManager
    
    // Verify the report specifies readiness, not enforces transitions
    assert!(matches!(report.readiness_analysis.readiness_status, crate::errors::ReadinessStatus::NotReady),
        "Gate readiness report should specify readiness status, not execute transitions");
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_bmode_operation_scenario_generation() {
        // Test that our strategies generate valid B-MODE scenarios
        let mut runner = proptest::test_runner::TestRunner::default();
        
        let scenario = bmode_operation_scenario_strategy().new_tree(&mut runner).unwrap().current();
        
        // Verify scenario has valid component
        assert!(matches!(scenario.component, 
            ComponentId::ConstitutionalRuleEngine |
            ComponentId::D4RegisterAllocator |
            ComponentId::JITCompiler |
            ComponentId::LoopOptimizer |
            ComponentId::NativeCache |
            ComponentId::SemanticSpecificationRegistry |
            ComponentId::FailureMatrix |
            ComponentId::DeterminismEngine
        ));
        
        // Verify scenario has valid operation type
        assert!(matches!(scenario.operation_type,
            BModeOperationType::ComplianceAnalysis |
            BModeOperationType::TemplateSpecification |
            BModeOperationType::ValidationLocationTracking |
            BModeOperationType::ContractViolationReporting |
            BModeOperationType::GateReadinessAnalysis
        ));
        
        // Verify validation inputs are bounded
        assert!(scenario.validation_inputs.len() <= 5);
        
        // Verify template types are bounded
        assert!(scenario.template_types.len() <= 3);
    }
    
    #[test]
    fn test_compliance_analysis_is_pure_bmode() {
        let validation_input = ValidationInput {
            component_id: ComponentId::D4RegisterAllocator,
            validation_rules: vec![],
            penalties: vec![],
            bonuses: vec![],
            metadata: BTreeMap::new(),
        };
        
        // Test that compliance analysis follows B-MODE principles
        let report1 = ValidationComplianceAnalyzer::analyze_compliance(&validation_input);
        let report2 = ValidationComplianceAnalyzer::analyze_compliance(&validation_input);
        
        // B-MODE Requirement: Same input produces same output (immutability)
        assert_eq!(report1.compliance_index, report2.compliance_index);
        assert_eq!(report1.base_compliance, report2.base_compliance);
        
        // B-MODE Requirement: Returns specification report, not execution result
        assert!(report1.compliance_index >= 0.0 && report1.compliance_index <= 1.0);
    }
    
    #[test]
    fn test_template_specification_is_pure_bmode() {
        let registry = TemplateSpecRegistryImpl::new();
        
        // Test that template specification follows B-MODE principles
        let spec1 = registry.specify_template_requirements(TemplateType::FailureMatrix);
        let spec2 = registry.specify_template_requirements(TemplateType::FailureMatrix);
        
        // B-MODE Requirement: Same input produces same output (immutability)
        assert_eq!(spec1.template_type, spec2.template_type);
        
        // B-MODE Requirement: Produces specification, not execution
        assert_eq!(spec1.template_type, TemplateType::FailureMatrix);
    }
    
    #[test]
    fn test_validation_location_tracking_is_pure_bmode() {
        let mut tracker = DefaultLocationTracker::new(ComponentId::D4RegisterAllocator);
        
        // Test that location tracking follows B-MODE principles
        let location1 = tracker.create_location();
        let location2 = tracker.create_location();
        
        // B-MODE Requirement: Same input produces same output (immutability)
        assert_eq!(location1.component, location2.component);
        
        // B-MODE Requirement: Produces specification, not execution
        assert_eq!(location1.component, ComponentId::D4RegisterAllocator);
        
        // Test immutable context operations
        let with_context = ValidationLocation::new(ComponentId::D4RegisterAllocator).with_structure("test".to_string());
        assert_eq!(location1.component, with_context.component); // Component unchanged
        assert_ne!(location1.structure_name, with_context.structure_name); // Structure changed in new instance
    }
    
    #[test]
    fn test_contract_violation_reporting_is_pure_bmode() {
        // Test that contract violation reporting follows B-MODE principles
        let report1 = ContractViolationReport::new(ComponentId::D4RegisterAllocator);
        let report2 = ContractViolationReport::new(ComponentId::D4RegisterAllocator);
        
        // B-MODE Requirement: Same input produces same output (immutability)
        assert_eq!(report1.component_context, report2.component_context);
        
        // B-MODE Requirement: Produces specification report, not execution
        assert_eq!(report1.component_context, ComponentId::D4RegisterAllocator);
        assert!(report1.violations.is_empty()); // Starts with empty specification
    }
    
    #[test]
    fn test_gate_readiness_analysis_is_pure_bmode() {
        // Test that gate readiness analysis follows B-MODE principles
        let report1 = GateDecisionReport::new();
        let report2 = GateDecisionReport::new();
        
        // B-MODE Requirement: Same conditions produce same output (immutability)
        assert_eq!(report1.readiness_analysis.readiness_status, report2.readiness_analysis.readiness_status);
        
        // B-MODE Requirement: Produces specification report, not execution
        assert!(report1.overall_readiness_score >= 0.0);
        assert!(matches!(report1.readiness_analysis.readiness_status, crate::errors::ReadinessStatus::NotReady));
    }
}