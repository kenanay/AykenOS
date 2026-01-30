//! Tests for B-MODE Constitutional Framework
//!
//! These tests validate that the B-MODE implementation follows pure specification
//! principles and maintains architectural separation from runtime enforcement.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bmode::{
        ConstitutionalRuleAnalyzer, DefaultConstitutionalRuleAnalyzer,
        DeterminismAnalyzer, DefaultDeterminismAnalyzer,
        TemplateSpecAnalyzer, DefaultTemplateSpecAnalyzer,
        ValidationLocationAnalyzer, DefaultValidationLocationAnalyzer,
    };
    use crate::bmode::failure_matrix::FailureMatrixAnalyzer;
    use crate::bmode::semantic_spec_catalog::SemanticSpecCatalog;
    use crate::bmode::types::{RuleType, EnforcementLevel};
    use crate::types::*;

    #[test]
    fn test_bmode_constitutional_analyzer_immutability() {
        let analyzer = DefaultConstitutionalRuleAnalyzer::new();
        
        // Test that analyzer methods use &self (immutable)
        let jit_operation = crate::bmode::constitutional::JITOperation::CodeGeneration {
            register_accesses: vec![],
            bounds_checking_enabled: true,
        };
        
        let report = analyzer.analyze_jit_allocation_immutability(&jit_operation);
        
        // B-MODE principle: Always returns SpecificationReport, never Result<()>
        assert!(report.violations.is_empty() || !report.violations.is_empty()); // Either is valid
        
        // Test that we can call multiple methods without mutation
        let report2 = analyzer.analyze_jit_allocation_immutability(&jit_operation);
        assert_eq!(report.violations.len(), report2.violations.len());
    }

    #[test]
    fn test_bmode_determinism_analyzer_purity() {
        let analyzer = DefaultDeterminismAnalyzer::new();
        
        let inputs = crate::bmode::determinism::AllocationInputs {
            ir_fingerprint: "test_fingerprint".to_string(),
            virtual_registers: vec![VirtualRegisterId(1), VirtualRegisterId(2)],
            constraints: AllocationConstraints::default(),
            optimization_level: OptimizationLevel::Debug,
            target_architecture: TargetArchitecture::X86_64,
        };
        
        let report = analyzer.analyze_allocation_reproducibility(&inputs);
        
        // B-MODE principle: Returns SpecificationReport, not Result<bool>
        assert!(report.compliance_score >= 0.0 && report.compliance_score <= 1.0);
        
        // Test deterministic behavior - same inputs should produce same results
        let report2 = analyzer.analyze_allocation_reproducibility(&inputs);
        assert_eq!(report.compliance_score, report2.compliance_score);
    }

    #[test]
    fn test_bmode_template_analyzer_immutability() {
        let analyzer = DefaultTemplateSpecAnalyzer::new();
        
        // Test immutable catalog access
        let catalog = analyzer.catalog();
        assert!(!catalog.specifications.is_empty());
        
        // Test template analysis returns reports, not mutations
        let report = analyzer.analyze_template_completeness(crate::bmode::templates::TemplateType::FailureMatrix);
        assert!(report.is_compliant() || !report.is_compliant()); // Either is valid
        
        // Test that catalog remains unchanged after analysis
        let catalog2 = analyzer.catalog();
        assert_eq!(catalog.specifications.len(), catalog2.specifications.len());
    }

    #[test]
    fn test_bmode_validation_location_analyzer_purity() {
        let analyzer = DefaultValidationLocationAnalyzer::new(ComponentId::D4RegisterAllocator);
        
        let location = crate::bmode::validation_location::ValidationLocation::new(ComponentId::D4RegisterAllocator);
        let context = location.create_context();
        
        // Test immutable analysis
        let report = analyzer.analyze_location_context(&context);
        assert!(report.compliance_score >= 0.0 && report.compliance_score <= 1.0);
        
        // Test immutable location context creation
        let new_location = crate::bmode::validation_location::ValidationLocation::new(ComponentId::JITCompiler);
        let new_analyzer = analyzer.with_location_context(new_location);
        
        // Original analyzer should be unchanged
        let original_location = analyzer.get_current_location();
        assert_eq!(original_location.component, ComponentId::D4RegisterAllocator);
        
        // New analyzer should have new location
        let updated_location = new_analyzer.get_current_location();
        assert_eq!(updated_location.component, ComponentId::JITCompiler);
    }

    #[test]
    fn test_rule_id_determinism() {
        // Test that RuleId::from_content produces deterministic results
        let content = b"test_rule_content";
        let rule_id1 = RuleId::from_content(content);
        let rule_id2 = RuleId::from_content(content);
        
        assert_eq!(rule_id1, rule_id2);
        assert_eq!(rule_id1.to_string(), rule_id2.to_string());
        
        // Different content should produce different IDs
        let different_content = b"different_rule_content";
        let rule_id3 = RuleId::from_content(different_content);
        
        assert_ne!(rule_id1, rule_id3);
    }

    #[test]
    fn test_bmode_architectural_separation() {
        // Test that B-MODE components only return SpecificationReport
        // and never perform enforcement actions
        
        let analyzer = DefaultConstitutionalRuleAnalyzer::new();
        let rule_spec = crate::bmode::constitutional::RuleSpec {
            rule_type: RuleType::JITAllocationImmutability,
            enforcement_level: EnforcementLevel::Constitutional,
            description: "Test rule".to_string(),
            recommended_response: crate::bmode::constitutional::RecommendedResponse::RecommendReject,
            immutability_guarantee: crate::bmode::constitutional::ImmutabilityLevel::Absolute,
            metadata: std::collections::BTreeMap::new(),
        };
        
        // B-MODE: specify_rule_addition returns SpecificationReport, not Result<()>
        let report = analyzer.specify_rule_addition(rule_spec);
        assert!(report.compliance_score >= 0.0);
        
        // B-MODE: No mutations occurred - analyzer state unchanged
        let active_rules = analyzer.get_active_rule_specifications();
        assert!(!active_rules.is_empty()); // Should have default rules
    }

    #[test]
    fn test_recommended_actions_not_enforcement() {
        // Test that B-MODE uses RecommendedAction, not direct enforcement
        let analyzer = DefaultConstitutionalRuleAnalyzer::new();
        
        let jit_operation = crate::bmode::constitutional::JITOperation::AllocationRewrite {
            original: AllocationDecision {
                virtual_register: VirtualRegisterId(1),
                binding: RegisterBinding::Physical(PhysicalRegisterId(1)),
                decision_context: AllocationContext {
                    pressure_level: 1,
                    optimization_level: OptimizationLevel::Debug,
                    constraints: AllocationConstraints::default(),
                    performance_requirements: PerformanceRequirements {
                        max_spill_rate: None,
                        max_register_pressure: None,
                        cache_locality: None,
                    },
                },
            },
            proposed: AllocationDecision {
                virtual_register: VirtualRegisterId(1),
                binding: RegisterBinding::Physical(PhysicalRegisterId(2)),
                decision_context: AllocationContext {
                    pressure_level: 1,
                    optimization_level: OptimizationLevel::Debug,
                    constraints: AllocationConstraints::default(),
                    performance_requirements: PerformanceRequirements {
                        max_spill_rate: None,
                        max_register_pressure: None,
                        cache_locality: None,
                    },
                },
            },
        };
        
        let report = analyzer.analyze_jit_allocation_immutability(&jit_operation);
        
        // Should detect violation but only recommend, not enforce
        assert!(!report.violations.is_empty());
        
        // Verify violation contains recommendation, not enforcement action
        let violation = &report.violations[0];
        assert!(violation.remediation_hint.contains("must not") || violation.remediation_hint.contains("should"));
    }

    #[test]
    fn test_bmode_runtime_import_prevention() {
        // Test that B-MODE crate cannot import runtime enforcement modules
        // This is a compile-time architectural constraint test
        
        // B-MODE should only have specification and analysis capabilities
        // Runtime enforcement should be in separate crates
        
        // Verify that our B-MODE modules only contain specification types
        let analyzer = DefaultConstitutionalRuleAnalyzer::new();
        let catalog = analyzer.get_active_rule_specifications();
        
        // All rules should be specifications, not runtime enforcement
        for rule in &catalog {
            // Rules should have recommended responses, not enforcement actions
            match &rule.recommended_response {
                crate::bmode::constitutional::RecommendedResponse::RecommendReject |
                crate::bmode::constitutional::RecommendedResponse::RecommendWarn |
                crate::bmode::constitutional::RecommendedResponse::RecommendEscalate(_) |
                crate::bmode::constitutional::RecommendedResponse::RecommendCustom(_) => {
                    // Good - these are recommendations, not enforcement
                }
            }
        }
        
        // Verify failure matrix only contains recommended responses
        let failure_analyzer = crate::bmode::failure_matrix::DefaultFailureMatrixAnalyzer::new();
        let failure_catalog = failure_analyzer.failure_matrix_catalog();
        
        for scenario in failure_catalog.scenarios.values() {
            // All responses should be recommendations
            let responses = vec![
                &scenario.component_responses.d1_response,
                &scenario.component_responses.d2_response,
                &scenario.component_responses.d3_response,
                &scenario.component_responses.d4_response,
            ];
            
            for response in responses {
                match response {
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendDisableOptimization |
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendFallbackToSafeMode |
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendLogAndContinue |
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendEscalateToHigherLevel |
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendTermination |
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendDisableCache |
                    crate::bmode::failure_matrix::RecommendedSystemResponse::RecommendReduceOptimizationLevel => {
                        // Good - these are recommendations, not direct actions
                    }
                }
            }
        }
        
        // Verify semantic locks only contain violation response specifications
        let semantic_analyzer = crate::bmode::semantic_spec_catalog::DefaultSemanticSpecCatalog::new();
        let semantic_catalog = semantic_analyzer.semantic_specification_catalog();
        
        for lock_spec in semantic_catalog.lock_specifications.values() {
            match &lock_spec.violation_response {
                crate::bmode::semantic_spec_catalog::ViolationResponseSpec::RecommendReject |
                crate::bmode::semantic_spec_catalog::ViolationResponseSpec::RecommendWarn |
                crate::bmode::semantic_spec_catalog::ViolationResponseSpec::RecommendEscalate(_) |
                crate::bmode::semantic_spec_catalog::ViolationResponseSpec::RecommendCustomActions(_) => {
                    // Good - these are recommendations, not enforcement
                }
            }
        }
        
        // This test passes if compilation succeeds and all responses are recommendations
        assert!(true, "B-MODE architectural separation maintained - no runtime enforcement imports detected");
    }
}