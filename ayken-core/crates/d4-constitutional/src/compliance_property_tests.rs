//! Property-based tests for validation compliance mathematical correctness
//!
//! **Feature: d4-constitutional-contracts-fixes, Property 4: Validation Compliance Mathematical Correctness**
//! **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

use crate::compliance::{
    ComplianceAnalyzer, ValidationComplianceAnalyzer, ValidationInput, ValidationRule, 
    Penalty, Bonus
};
use crate::types::{ComponentId, Severity};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::collections::BTreeMap;

/// Strategy for generating validation rules
fn validation_rule_strategy() -> impl Strategy<Value = ValidationRule> {
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
        ValidationRule {
            rule_id,
            description,
            severity,
            weight,
        }
    })
}

/// Strategy for generating penalties
fn penalty_strategy() -> impl Strategy<Value = Penalty> {
    (
        "P[0-9]{1,3}",
        ".*",
        prop_oneof![
            Just(Severity::Info),
            Just(Severity::Warning),
            Just(Severity::Error),
            Just(Severity::Critical),
        ],
        0.0f64..1.0f64, // Impact should be normalized
        prop::option::of("[A-Z][0-9]{1,3}"),
    ).prop_map(|(penalty_id, description, severity, impact, source_rule)| {
        Penalty {
            penalty_id,
            description,
            severity,
            impact,
            source_rule,
        }
    })
}

/// Strategy for generating bonuses
fn bonus_strategy() -> impl Strategy<Value = Bonus> {
    (
        "B[0-9]{1,3}",
        ".*",
        0.0f64..0.5f64, // Bonuses should be smaller than penalties
        prop::option::of("[A-Z][0-9]{1,3}"),
    ).prop_map(|(bonus_id, description, impact, source_rule)| {
        Bonus {
            bonus_id,
            description,
            impact,
            source_rule,
        }
    })
}

/// Strategy for generating component IDs
fn component_id_strategy() -> impl Strategy<Value = ComponentId> {
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

/// Strategy for generating validation inputs
fn validation_input_strategy() -> impl Strategy<Value = ValidationInput> {
    (
        component_id_strategy(),
        prop::collection::vec(validation_rule_strategy(), 0..10),
        prop::collection::vec(penalty_strategy(), 0..5),
        prop::collection::vec(bonus_strategy(), 0..3),
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Property 4: Validation Compliance Mathematical Correctness**
    /// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**
    ///
    /// For any validation operation with penalties and bonuses, the compliance calculation should:
    /// - Count each penalty exactly once (Requirement 4.1)
    /// - Aggregate without double-counting (Requirement 4.2) 
    /// - Produce intuitive results where higher compliance indices indicate better validation (Requirement 4.3)
    /// - Handle edge cases gracefully (Requirement 4.4)
    /// - Ensure consistent compliance methodology (Requirement 4.5)
    #[test]
    fn property_validation_compliance_mathematical_correctness(
        validation_input in validation_input_strategy()
    ) {
        let report = ValidationComplianceAnalyzer::analyze_compliance(&validation_input);
        
        // Requirement 4.1: Count each penalty exactly once
        prop_assert_eq!(report.penalties.len(), validation_input.penalties.len(),
            "Each penalty should be counted exactly once");
        
        // Verify penalty IDs are preserved (no double-counting)
        let input_penalty_ids: std::collections::BTreeSet<_> = 
            validation_input.penalties.iter().map(|p| &p.penalty_id).collect();
        let report_penalty_ids: std::collections::BTreeSet<_> = 
            report.penalties.iter().map(|p| &p.penalty_id).collect();
        prop_assert_eq!(input_penalty_ids, report_penalty_ids,
            "Penalty IDs should be preserved without duplication");
        
        // Requirement 4.1: Count each bonus exactly once
        prop_assert_eq!(report.bonuses.len(), validation_input.bonuses.len(),
            "Each bonus should be counted exactly once");
        
        // Verify bonus IDs are preserved (no double-counting)
        let input_bonus_ids: std::collections::BTreeSet<_> = 
            validation_input.bonuses.iter().map(|b| &b.bonus_id).collect();
        let report_bonus_ids: std::collections::BTreeSet<_> = 
            report.bonuses.iter().map(|b| &b.bonus_id).collect();
        prop_assert_eq!(input_bonus_ids, report_bonus_ids,
            "Bonus IDs should be preserved without duplication");
        
        // Requirement 4.2: Aggregate without double-counting
        // Verify penalty impact analysis doesn't double-count
        let penalty_report = ValidationComplianceAnalyzer::analyze_penalty_impact(&validation_input.penalties);
        let manual_penalty_sum: f64 = validation_input.penalties.iter()
            .map(|p| {
                let severity_weight = match p.severity {
                    Severity::Critical => 1.0,
                    Severity::Error => 0.8,
                    Severity::Warning => 0.5,
                    Severity::Info => 0.2,
                };
                (p.impact * severity_weight * 1_000_000.0).round() / 1_000_000.0
            })
            .sum();
        
        prop_assert!((penalty_report.total_penalty_impact - manual_penalty_sum).abs() < 0.000001,
            "Penalty aggregation should match manual calculation without double-counting: {} vs {}",
            penalty_report.total_penalty_impact, manual_penalty_sum);
        
        // Verify bonus impact analysis doesn't double-count
        let bonus_report = ValidationComplianceAnalyzer::analyze_bonus_impact(&validation_input.bonuses);
        let manual_bonus_sum: f64 = validation_input.bonuses.iter()
            .map(|b| (b.impact * 1_000_000.0).round() / 1_000_000.0)
            .sum();
        
        prop_assert!((bonus_report.total_bonus_impact - manual_bonus_sum).abs() < 0.000001,
            "Bonus aggregation should match manual calculation without double-counting: {} vs {}",
            bonus_report.total_bonus_impact, manual_bonus_sum);
        
        // Requirement 4.3: Produce intuitive results where higher compliance indices indicate better validation
        prop_assert!(report.compliance_index >= 0.0 && report.compliance_index <= 1.0,
            "Compliance index should be in valid range [0.0, 1.0]: {}", report.compliance_index);
        
        prop_assert!(report.base_compliance >= 0.0 && report.base_compliance <= 1.0,
            "Base compliance should be in valid range [0.0, 1.0]: {}", report.base_compliance);
        
        // Test intuitive behavior: more penalties should generally reduce compliance
        if !validation_input.penalties.is_empty() {
            let no_penalty_input = ValidationInput {
                penalties: Vec::new(),
                ..validation_input.clone()
            };
            let no_penalty_report = ValidationComplianceAnalyzer::analyze_compliance(&no_penalty_input);
            
            prop_assert!(report.compliance_index <= no_penalty_report.compliance_index,
                "Adding penalties should not increase compliance: {} vs {}",
                report.compliance_index, no_penalty_report.compliance_index);
        }
        
        // Test intuitive behavior: more bonuses should generally increase compliance
        if !validation_input.bonuses.is_empty() {
            let no_bonus_input = ValidationInput {
                bonuses: Vec::new(),
                ..validation_input.clone()
            };
            let no_bonus_report = ValidationComplianceAnalyzer::analyze_compliance(&no_bonus_input);
            
            prop_assert!(report.compliance_index >= no_bonus_report.compliance_index,
                "Adding bonuses should not decrease compliance: {} vs {}",
                report.compliance_index, no_bonus_report.compliance_index);
        }
        
        // Requirement 4.4: Handle edge cases gracefully
        // Test that extreme values don't cause mathematical errors
        prop_assert!(report.compliance_index.is_finite(),
            "Compliance index should be finite: {}", report.compliance_index);
        
        prop_assert!(report.base_compliance.is_finite(),
            "Base compliance should be finite: {}", report.base_compliance);
        
        prop_assert!(!report.compliance_index.is_nan(),
            "Compliance index should not be NaN");
        
        prop_assert!(!report.base_compliance.is_nan(),
            "Base compliance should not be NaN");
        
        // Requirement 4.5: Ensure consistent compliance methodology
        // Test deterministic behavior: same input should produce same output
        let report2 = ValidationComplianceAnalyzer::analyze_compliance(&validation_input);
        
        prop_assert_eq!(report.compliance_index, report2.compliance_index,
            "Compliance calculation should be deterministic");
        
        prop_assert_eq!(report.base_compliance, report2.base_compliance,
            "Base compliance calculation should be deterministic");
        
        // Test that analysis metadata is properly populated
        prop_assert_eq!(report.analysis_metadata.component_context, validation_input.component_id,
            "Analysis metadata should reflect input component");
        
        prop_assert_eq!(report.analysis_metadata.total_rules_analyzed, validation_input.validation_rules.len(),
            "Analysis metadata should reflect total rules analyzed");
        
        prop_assert_eq!(report.analysis_metadata.calculation_method, "mathematical_compliance_v1",
            "Analysis metadata should specify calculation method");
        
        // Test float normalization consistency (6 decimal places)
        let compliance_str = format!("{:.6}", report.compliance_index);
        let parsed_compliance: f64 = compliance_str.parse().unwrap();
        prop_assert_eq!(report.compliance_index, parsed_compliance,
            "Compliance index should be normalized to 6 decimal places");
        
        let base_str = format!("{:.6}", report.base_compliance);
        let parsed_base: f64 = base_str.parse().unwrap();
        prop_assert_eq!(report.base_compliance, parsed_base,
            "Base compliance should be normalized to 6 decimal places");
    }
    
    /// Test edge case: empty validation input
    #[test]
    fn property_empty_validation_input_edge_case(
        component_id in component_id_strategy()
    ) {
        let empty_input = ValidationInput {
            component_id,
            validation_rules: Vec::new(),
            penalties: Vec::new(),
            bonuses: Vec::new(),
            metadata: BTreeMap::new(),
        };
        
        let report = ValidationComplianceAnalyzer::analyze_compliance(&empty_input);
        
        // Requirement 4.4: Handle edge cases gracefully
        prop_assert_eq!(report.base_compliance, 1.0,
            "Empty validation should have perfect base compliance");
        
        prop_assert_eq!(report.compliance_index, 1.0,
            "Empty validation should have perfect compliance index");
        
        prop_assert!(report.penalties.is_empty(),
            "Empty input should result in empty penalties");
        
        prop_assert!(report.bonuses.is_empty(),
            "Empty input should result in empty bonuses");
    }
    
    /// Test edge case: extreme penalty values
    #[test]
    fn property_extreme_penalty_values_edge_case(
        component_id in component_id_strategy(),
        extreme_impact in 1.0f64..10.0f64
    ) {
        let extreme_penalty = Penalty {
            penalty_id: "EXTREME".to_string(),
            description: "Extreme penalty test".to_string(),
            severity: Severity::Critical,
            impact: extreme_impact,
            source_rule: None,
        };
        
        let input = ValidationInput {
            component_id,
            validation_rules: Vec::new(),
            penalties: vec![extreme_penalty],
            bonuses: Vec::new(),
            metadata: BTreeMap::new(),
        };
        
        let report = ValidationComplianceAnalyzer::analyze_compliance(&input);
        
        // Requirement 4.4: Handle edge cases gracefully
        prop_assert!(report.compliance_index >= 0.0,
            "Extreme penalties should not result in negative compliance");
        
        prop_assert!(report.compliance_index.is_finite(),
            "Extreme penalties should not result in infinite compliance");
        
        prop_assert!(!report.compliance_index.is_nan(),
            "Extreme penalties should not result in NaN compliance");
    }
    
    /// Test edge case: extreme bonus values
    #[test]
    fn property_extreme_bonus_values_edge_case(
        component_id in component_id_strategy(),
        extreme_impact in 1.0f64..10.0f64
    ) {
        let extreme_bonus = Bonus {
            bonus_id: "EXTREME".to_string(),
            description: "Extreme bonus test".to_string(),
            impact: extreme_impact,
            source_rule: None,
        };
        
        let input = ValidationInput {
            component_id,
            validation_rules: Vec::new(),
            penalties: Vec::new(),
            bonuses: vec![extreme_bonus],
            metadata: BTreeMap::new(),
        };
        
        let report = ValidationComplianceAnalyzer::analyze_compliance(&input);
        
        // Requirement 4.4: Handle edge cases gracefully
        prop_assert!(report.compliance_index <= 1.0,
            "Extreme bonuses should not result in compliance > 1.0");
        
        prop_assert!(report.compliance_index.is_finite(),
            "Extreme bonuses should not result in infinite compliance");
        
        prop_assert!(!report.compliance_index.is_nan(),
            "Extreme bonuses should not result in NaN compliance");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_property_test_strategies() {
        // Test that our strategies generate valid data
        let mut runner = proptest::test_runner::TestRunner::default();
        
        // Test validation rule strategy
        let rule = validation_rule_strategy().new_tree(&mut runner).unwrap().current();
        assert!(!rule.rule_id.is_empty());
        assert!(rule.weight >= 0.0);
        
        // Test penalty strategy
        let penalty = penalty_strategy().new_tree(&mut runner).unwrap().current();
        assert!(!penalty.penalty_id.is_empty());
        assert!(penalty.impact >= 0.0 && penalty.impact <= 1.0);
        
        // Test bonus strategy
        let bonus = bonus_strategy().new_tree(&mut runner).unwrap().current();
        assert!(!bonus.bonus_id.is_empty());
        assert!(bonus.impact >= 0.0 && bonus.impact <= 0.5);
        
        // Test validation input strategy
        let input = validation_input_strategy().new_tree(&mut runner).unwrap().current();
        assert!(input.validation_rules.len() <= 10);
        assert!(input.penalties.len() <= 5);
        assert!(input.bonuses.len() <= 3);
    }
}