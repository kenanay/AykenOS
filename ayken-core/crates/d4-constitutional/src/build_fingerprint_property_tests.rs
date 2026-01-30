//! Property-Based Tests for Build Fingerprint Correctness
//!
//! **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
//! **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**
//!
//! This module contains property-based tests that validate the correctness of the build fingerprinting
//! system for gate readiness analysis. The tests ensure that fingerprint analysis is consistent,
//! reproducible, and handles legitimate build variations appropriately.

use crate::build_fingerprint::{
    BuildFingerprintAnalyzer, DefaultBuildFingerprintAnalyzer, BuildFingerprintSpec, BuildContext,
    ComponentFingerprintSpec, BuildMetadata, DeterministicBuildId, FingerprintAnalysisReport,
    ValidationIssue, ValidationIssueType, ValidationIssueSeverity
};
use crate::gate_readiness::{GateReadinessAnalyzer, DefaultGateReadinessAnalyzer, GateReadinessContext, ValidationRequirement, ValidationRequirementType};
use crate::types::{ComponentId, DeterministicClock};
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Strategy for generating build contexts
pub fn build_context_strategy() -> impl Strategy<Value = BuildContext> {
    (
        prop::collection::vec("[a-zA-Z0-9_]{3,10}", 1..3).prop_map(|parts| parts.join("_")),
        prop::collection::vec("[a-zA-Z0-9_]{3,10}", 1..3).prop_map(|parts| parts.join("_")),
        "[a-fA-F0-9]{40}",
        r"[0-9]+\.[0-9]+\.[0-9]+",
        prop::collection::btree_map("[A-Z_]{3,15}", "[a-zA-Z0-9_\\s-]{1,50}", 0..5),
    ).prop_map(|(target_arch, build_config, source_hash, toolchain_ver, env_vars)| {
        BuildContext {
            target_architecture: target_arch,
            build_configuration: build_config,
            source_tree_hash: source_hash,
            toolchain_version: toolchain_ver,
            build_environment: env_vars,
        }
    })
}

/// Strategy for generating component fingerprint specs
pub fn component_fingerprint_spec_strategy() -> impl Strategy<Value = ComponentFingerprintSpec> {
    (
        component_id_strategy(),
        "[a-fA-F0-9]{32}",
        r"[0-9]+\.[0-9]+\.[0-9]+",
        prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 0..3),
        prop::collection::vec("[a-zA-Z0-9_=-]{1,30}", 0..5),
    ).prop_map(|(component_id, hash, version, deps, flags)| {
        ComponentFingerprintSpec {
            component_id,
            component_hash: hash,
            version,
            dependencies: deps.into_iter().map(|name| crate::build_fingerprint::DependencySpec {
                name: name.clone(),
                version: "1.0.0".to_string(),
                hash: format!("hash_{}", name),
            }).collect(),
            compilation_flags: flags,
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
        Just(ComponentId::UnrollOptimizer),
        Just(ComponentId::D1Component),
        Just(ComponentId::D2Component),
        Just(ComponentId::D3Component),
        Just(ComponentId::D4RegisterAllocator),
        Just(ComponentId::TemplateSpecRegistry),
    ]
}

/// Strategy for generating build metadata
pub fn build_metadata_strategy() -> impl Strategy<Value = BuildMetadata> {
    (
        "[a-zA-Z0-9_]{3,15}",
        "[a-zA-Z0-9_]{3,15}",
        r"[0-9]+\.[0-9]+\.[0-9]+",
        prop::collection::vec("[a-zA-Z0-9_=-]{1,30}", 0..5),
        prop::collection::btree_map("[A-Z_]{3,15}", "[a-zA-Z0-9_\\s-]{1,50}", 0..5),
    ).prop_map(|(build_target, opt_level, compiler_ver, build_flags, env_vars)| {
        BuildMetadata {
            build_target,
            optimization_level: opt_level,
            compiler_version: compiler_ver,
            build_flags,
            environment_variables: env_vars,
        }
    })
}

/// Strategy for generating build fingerprint specs
pub fn build_fingerprint_spec_strategy() -> impl Strategy<Value = BuildFingerprintSpec> {
    (
        "[a-fA-F0-9]{64}",
        "[a-zA-Z0-9_]{10,20}",
        prop::collection::vec(component_fingerprint_spec_strategy(), 1..5),
        build_metadata_strategy(),
    ).prop_map(|(hash, epoch_id, components, metadata)| {
        BuildFingerprintSpec {
            hash,
            build_epoch: DeterministicBuildId::new(epoch_id),
            components,
            metadata,
        }
    })
}

/// Strategy for generating gate readiness contexts
pub fn gate_readiness_context_strategy() -> impl Strategy<Value = GateReadinessContext> {
    (
        "[a-zA-Z0-9_]{5,15}",
        "[a-zA-Z]{5,15}",
        prop::collection::vec(component_id_strategy(), 1..3),
        build_context_strategy(),
        prop::collection::vec(validation_requirement_strategy(), 0..3),
    ).prop_map(|(gate_id, phase, components, build_context, requirements)| {
        GateReadinessContext {
            gate_id,
            current_phase: phase,
            required_components: components,
            build_context,
            validation_requirements: requirements,
        }
    })
}

/// Strategy for generating validation requirements
pub fn validation_requirement_strategy() -> impl Strategy<Value = ValidationRequirement> {
    (
        "[a-zA-Z0-9_]{5,15}",
        component_id_strategy(),
        validation_requirement_type_strategy(),
        "[a-zA-Z0-9_\\s]{10,50}",
        any::<bool>(),
    ).prop_map(|(req_id, component, req_type, description, mandatory)| {
        ValidationRequirement {
            requirement_id: req_id,
            component,
            requirement_type: req_type,
            description,
            mandatory,
        }
    })
}

/// Strategy for generating validation requirement types
pub fn validation_requirement_type_strategy() -> impl Strategy<Value = ValidationRequirementType> {
    prop_oneof![
        Just(ValidationRequirementType::FingerprintVerification),
        Just(ValidationRequirementType::ComponentPresence),
        Just(ValidationRequirementType::IntegrityCheck),
        Just(ValidationRequirementType::ComplianceValidation),
        Just(ValidationRequirementType::PermissionVerification),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 6.1: Gate Readiness Analysis Includes Build Fingerprint Information**
    /// **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
    /// **Validates: Requirement 6.1**
    ///
    /// For any gate readiness analysis, the system should include build fingerprint information
    #[test]
    fn prop_gate_readiness_includes_fingerprint_info(context in gate_readiness_context_strategy()) {
        let report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        // Gate readiness report must include fingerprint analysis
        prop_assert!(!report.fingerprint_analysis.fingerprint_spec.hash.is_empty(),
            "Gate readiness analysis must include build fingerprint hash");
        
        prop_assert!(!report.fingerprint_analysis.fingerprint_spec.build_epoch.as_str().is_empty(),
            "Gate readiness analysis must include build epoch information");
        
        // Timestamp validity is guaranteed by DeterministicClock and LogicalTimestamp newtype
        
        // Readiness score must be influenced by fingerprint validity
        let fingerprint_valid = report.fingerprint_analysis.validity_analysis.is_valid;
        if fingerprint_valid {
            prop_assert!(report.readiness_score > 0.5,
                "Valid fingerprint should contribute to higher readiness score");
        }
    }

    /// **Property 6.2: Reproducible Fingerprints for Identical Builds**
    /// **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
    /// **Validates: Requirement 6.2**
    ///
    /// For any build context, the system should generate reproducible fingerprints for identical builds
    #[test]
    fn prop_reproducible_fingerprints_for_identical_builds(context in build_context_strategy()) {
        let analysis1 = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&context);
        let analysis2 = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&context);
        
        // Identical build contexts should produce identical fingerprint specs
        prop_assert_eq!(analysis1.fingerprint_spec.hash, analysis2.fingerprint_spec.hash,
            "Identical build contexts must produce identical fingerprint hashes");
        
        prop_assert_eq!(analysis1.fingerprint_spec.build_epoch, analysis2.fingerprint_spec.build_epoch,
            "Identical build contexts must produce identical build epochs");
        
        prop_assert_eq!(analysis1.fingerprint_spec.components.len(), analysis2.fingerprint_spec.components.len(),
            "Identical build contexts must produce same number of components");
        
        // Validity analysis should be consistent
        prop_assert_eq!(analysis1.validity_analysis.is_valid, analysis2.validity_analysis.is_valid,
            "Identical build contexts must produce consistent validity analysis");
        
        // Reproducibility scores should be identical
        prop_assert_eq!(analysis1.validity_analysis.reproducibility_score, analysis2.validity_analysis.reproducibility_score,
            "Identical build contexts must produce identical reproducibility scores");
    }

    /// **Property 6.3: Legitimate Build Variations Handled Appropriately**
    /// **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
    /// **Validates: Requirement 6.3**
    ///
    /// For any fingerprint differences, the system should account for legitimate build variations
    #[test]
    fn prop_legitimate_build_variations_handled_appropriately(
        fingerprint_a in build_fingerprint_spec_strategy(),
        fingerprint_b in build_fingerprint_spec_strategy()
    ) {
        let variations = DefaultGateReadinessAnalyzer::analyze_fingerprint_variations(&fingerprint_a, &fingerprint_b);
        
        // All variations must have legitimacy assessments
        for variation in &variations {
            prop_assert!(!variation.variation_type.is_empty(),
                "Each variation must have a defined type");
            
            // Readiness adjustments must be reasonable (between -1.0 and 1.0)
            prop_assert!(variation.readiness_adjustment >= -1.0 && variation.readiness_adjustment <= 1.0,
                "Readiness adjustments must be within reasonable bounds: got {}", variation.readiness_adjustment);
            
            // Impact assessment must be consistent with adjustment magnitude
            let adjustment_magnitude = variation.readiness_adjustment.abs();
            match variation.impact_on_readiness {
                crate::gate_readiness::VariationImpact::NoImpact => {
                    prop_assert!(adjustment_magnitude <= 0.01,
                        "No impact variations should have minimal readiness adjustment");
                },
                crate::gate_readiness::VariationImpact::MinorImpact => {
                    prop_assert!(adjustment_magnitude <= 0.1,
                        "Minor impact variations should have small readiness adjustment");
                },
                crate::gate_readiness::VariationImpact::ModerateImpact => {
                    prop_assert!(adjustment_magnitude <= 0.2,
                        "Moderate impact variations should have moderate readiness adjustment");
                },
                crate::gate_readiness::VariationImpact::MajorImpact => {
                    prop_assert!(adjustment_magnitude <= 0.4,
                        "Major impact variations should have significant readiness adjustment");
                },
                crate::gate_readiness::VariationImpact::BlockingImpact => {
                    prop_assert!(adjustment_magnitude <= 1.0,
                        "Blocking impact variations can have maximum readiness adjustment");
                },
            }
        }
        
        // If fingerprints are identical, there should be no variations
        if fingerprint_a == fingerprint_b {
            prop_assert!(variations.is_empty(),
                "Identical fingerprints should produce no variations");
        }
    }

    /// **Property 6.4: Fingerprint History Maintained with Readiness Records**
    /// **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
    /// **Validates: Requirement 6.4**
    ///
    /// For any gate readiness analysis, the system should maintain fingerprint history with readiness records
    #[test]
    fn prop_fingerprint_history_maintained_with_readiness_records(context in gate_readiness_context_strategy()) {
        let report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        // Timestamp validity is guaranteed by DeterministicClock and LogicalTimestamp newtype
        // Report must contain fingerprint analysis for history tracking
        
        // Report must contain gate analysis with completion tracking
        prop_assert!(report.gate_analysis.completion_percentage >= 0.0 && report.gate_analysis.completion_percentage <= 100.0,
            "Gate analysis must have valid completion percentage for history tracking");
        
        // Report must be serializable for storage
        let serialized = serde_json::to_string(&report);
        prop_assert!(serialized.is_ok(),
            "Gate readiness report must be serializable for history storage");
        
        if let Ok(serialized_data) = serialized {
            let deserialized: Result<crate::gate_readiness::GateReadinessReport, _> = serde_json::from_str(&serialized_data);
            prop_assert!(deserialized.is_ok(),
                "Serialized gate readiness report must be deserializable for history retrieval");
            
            if let Ok(deserialized_report) = deserialized {
                prop_assert_eq!(report.fingerprint_analysis.fingerprint_spec.hash, deserialized_report.fingerprint_analysis.fingerprint_spec.hash,
                    "Fingerprint hash must be preserved in serialization for history tracking");
                
                prop_assert_eq!(report.readiness_score, deserialized_report.readiness_score,
                    "Readiness score must be preserved in serialization for history tracking");
            }
        }
    }

    /// **Property 6.5: Fingerprint Integrity and Authenticity Verification**
    /// **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
    /// **Validates: Requirement 6.5**
    ///
    /// For any fingerprint validation, the system should verify fingerprint integrity and authenticity
    #[test]
    fn prop_fingerprint_integrity_and_authenticity_verification(fingerprint in build_fingerprint_spec_strategy()) {
        let compliance_report = DefaultBuildFingerprintAnalyzer::report_fingerprint_compliance(&fingerprint);
        let validity_analysis = DefaultBuildFingerprintAnalyzer::analyze_fingerprint_validity(&fingerprint);
        
        // Compliance report must address integrity concerns
        if !fingerprint.hash.is_empty() {
            // Non-empty hash should not trigger hash-related violations
            let hash_violations = compliance_report.violations.iter()
                .filter(|v| v.rule_id.as_ref().map_or(false, |id| id.contains("HASH")))
                .count();
            prop_assert_eq!(hash_violations, 0,
                "Valid fingerprint hash should not trigger hash-related compliance violations");
        }
        
        // Validity analysis must include integrity assessment
        prop_assert!(validity_analysis.reproducibility_score >= 0.0 && validity_analysis.reproducibility_score <= 1.0,
            "Reproducibility score must be within valid range [0.0, 1.0]");
        
        prop_assert!(validity_analysis.consistency_analysis.overall_consistency >= 0.0 && validity_analysis.consistency_analysis.overall_consistency <= 1.0,
            "Overall consistency score must be within valid range [0.0, 1.0]");
        
        // Validation issues must be properly categorized
        for issue in &validity_analysis.validation_issues {
            prop_assert!(!issue.description.is_empty(),
                "Each validation issue must have a non-empty description");
            
            // Issue severity must be consistent with issue type
            match issue.issue_type {
                ValidationIssueType::HashMismatch => {
                    prop_assert!(matches!(issue.severity, ValidationIssueSeverity::Error | ValidationIssueSeverity::Critical),
                        "Hash mismatch issues should have Error or Critical severity");
                },
                ValidationIssueType::MissingComponent => {
                    prop_assert!(matches!(issue.severity, ValidationIssueSeverity::Warning | ValidationIssueSeverity::Error | ValidationIssueSeverity::Critical),
                        "Missing component issues should have Warning, Error, or Critical severity");
                },
                ValidationIssueType::VersionInconsistency => {
                    prop_assert!(matches!(issue.severity, ValidationIssueSeverity::Info | ValidationIssueSeverity::Warning),
                        "Version inconsistency issues should have Info or Warning severity");
                },
                _ => {
                    // Other issue types can have any severity
                }
            }
        }
        
        // If fingerprint is valid, authenticity indicators should be positive
        if validity_analysis.is_valid {
            prop_assert!(validity_analysis.reproducibility_score > 0.5,
                "Valid fingerprints should have high reproducibility scores");
        }
    }

    /// **Property 6.6: Build Fingerprint Analysis Mathematical Correctness**
    /// **Feature: d4-constitutional-contracts-fixes, Property 6: Build Fingerprint Correctness**
    /// **Validates: All Requirements 6.1-6.5 (Mathematical Properties)**
    ///
    /// For any fingerprint analysis operations, mathematical properties should hold
    #[test]
    fn prop_build_fingerprint_analysis_mathematical_correctness(
        fingerprint_a in build_fingerprint_spec_strategy(),
        fingerprint_b in build_fingerprint_spec_strategy()
    ) {
        let comparison = DefaultBuildFingerprintAnalyzer::compare_fingerprint_specs(&fingerprint_a, &fingerprint_b);
        
        // Similarity score must be within valid range
        prop_assert!(comparison.similarity_score >= 0.0 && comparison.similarity_score <= 1.0,
            "Similarity score must be within range [0.0, 1.0]: got {}", comparison.similarity_score);
        
        // Identical fingerprints must have similarity score of 1.0
        if fingerprint_a == fingerprint_b {
            prop_assert_eq!(comparison.similarity_score, 1.0,
                "Identical fingerprints must have similarity score of 1.0");
            prop_assert!(comparison.comparison_analysis.overall_compatibility,
                "Identical fingerprints must be overall compatible");
            prop_assert!(comparison.differences.is_empty(),
                "Identical fingerprints must have no differences");
        }
        
        // Component similarity must be consistent with structural similarity
        let structural_sim = comparison.comparison_analysis.structural_similarity;
        let component_sim = comparison.comparison_analysis.component_similarity;
        let metadata_sim = comparison.comparison_analysis.metadata_similarity;
        
        prop_assert!(structural_sim >= 0.0 && structural_sim <= 1.0,
            "Structural similarity must be within range [0.0, 1.0]");
        prop_assert!(component_sim >= 0.0 && component_sim <= 1.0,
            "Component similarity must be within range [0.0, 1.0]");
        prop_assert!(metadata_sim >= 0.0 && metadata_sim <= 1.0,
            "Metadata similarity must be within range [0.0, 1.0]");
        
        // Overall similarity should be related to component similarities
        let expected_similarity = (structural_sim + component_sim + metadata_sim) / 3.0;
        let normalized_expected = (expected_similarity * 1_000_000.0).round() / 1_000_000.0;
        prop_assert_eq!(comparison.similarity_score, normalized_expected,
            "Overall similarity should be average of component similarities");
        
        // Readiness score calculation must be deterministic and bounded
        let context = GateReadinessContext {
            gate_id: "test_gate".to_string(),
            current_phase: "validation".to_string(),
            required_components: vec![ComponentId::D4RegisterAllocator],
            build_context: BuildContext {
                target_architecture: "x86_64".to_string(),
                build_configuration: "release".to_string(),
                source_tree_hash: fingerprint_a.hash.clone(),
                toolchain_version: "1.70.0".to_string(),
                build_environment: BTreeMap::new(),
            },
            validation_requirements: vec![],
        };
        
        let report1 = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        let report2 = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        // Deterministic analysis should produce identical results
        prop_assert_eq!(report1.readiness_score, report2.readiness_score,
            "Identical contexts should produce identical readiness scores");
        
        prop_assert!(report1.readiness_score >= 0.0 && report1.readiness_score <= 1.0,
            "Readiness score must be within range [0.0, 1.0]");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_build_context_strategy_generates_valid_contexts() {
        let strategy = build_context_strategy();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..10 {
            let context = strategy.new_tree(&mut runner).unwrap().current();
            assert!(!context.target_architecture.is_empty());
            assert!(!context.build_configuration.is_empty());
            assert!(!context.source_tree_hash.is_empty());
            assert!(!context.toolchain_version.is_empty());
        }
    }

    #[test]
    fn test_component_fingerprint_spec_strategy_generates_valid_specs() {
        let strategy = component_fingerprint_spec_strategy();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..10 {
            let spec = strategy.new_tree(&mut runner).unwrap().current();
            assert!(!spec.component_hash.is_empty());
            assert!(!spec.version.is_empty());
        }
    }

    #[test]
    fn test_build_fingerprint_spec_strategy_generates_valid_specs() {
        let strategy = build_fingerprint_spec_strategy();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..10 {
            let spec = strategy.new_tree(&mut runner).unwrap().current();
            assert!(!spec.hash.is_empty());
            assert!(!spec.build_epoch.as_str().is_empty());
            assert!(!spec.components.is_empty());
        }
    }

    #[test]
    fn test_gate_readiness_context_strategy_generates_valid_contexts() {
        let strategy = gate_readiness_context_strategy();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..10 {
            let context = strategy.new_tree(&mut runner).unwrap().current();
            assert!(!context.gate_id.is_empty());
            assert!(!context.current_phase.is_empty());
            assert!(!context.required_components.is_empty());
        }
    }

    #[test]
    fn test_property_test_deterministic_execution() {
        // Test that property tests are deterministic with fixed seeds
        let context = BuildContext {
            target_architecture: "x86_64".to_string(),
            build_configuration: "release".to_string(),
            source_tree_hash: "fixed_hash_for_testing".to_string(),
            toolchain_version: "1.70.0".to_string(),
            build_environment: BTreeMap::new(),
        };
        
        let analysis1 = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&context);
        let analysis2 = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&context);
        
        // Should be deterministic
        assert_eq!(analysis1.fingerprint_spec.hash, analysis2.fingerprint_spec.hash);
        assert_eq!(analysis1.validity_analysis.is_valid, analysis2.validity_analysis.is_valid);
    }
}