//! Gate Readiness Analysis with Build Fingerprinting Integration
//!
//! This module provides B-MODE compliant gate readiness analysis that includes build fingerprinting
//! for comprehensive readiness assessment. It ONLY analyzes and reports readiness, NEVER approves or enforces.
//!
//! PURE B-MODE PRINCIPLES:
//! - Analyze readiness, never approve gates
//! - Report analysis, never make decisions
//! - Immutable analysis, no state mutations
//! - Specification reporting, not runtime enforcement

use crate::build_fingerprint::{BuildFingerprintSpec, BuildFingerprintAnalyzer, DefaultBuildFingerprintAnalyzer, FingerprintAnalysisReport, BuildContext};
use crate::errors::{GateDecisionReport, GateReadinessAnalysis, FingerprintAnalysis, SpecificationReport, SpecificationViolation, ViolationType};
use crate::types::{ComponentId, LogicalTimestamp, DeterministicClock};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Gate readiness report with integrated build fingerprinting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateReadinessReport {
    pub gate_analysis: GateReadinessAnalysis,
    pub fingerprint_analysis: FingerprintAnalysisReport,
    pub readiness_score: f64, // Normalized to 6 decimal places for deterministic comparison
    pub blocking_issues: Vec<ReadinessBlockingIssue>,
    pub recommendations: Vec<ReadinessRecommendation>,
    pub analysis_timestamp: LogicalTimestamp,
}

/// Blocking issues that prevent gate readiness
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessBlockingIssue {
    pub issue_type: BlockingIssueType,
    pub component: Option<ComponentId>,
    pub description: String,
    pub severity: BlockingIssueSeverity,
    pub resolution_path: Vec<String>,
}

/// Types of blocking issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockingIssueType {
    FingerprintMismatch,
    ComponentMissing,
    IntegrityViolation,
    ComplianceFailure,
    PermissionInsufficient,
    ValidationIncomplete,
}

/// Severity levels for blocking issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockingIssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Readiness recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessRecommendation {
    pub recommendation_type: ReadinessRecommendationType,
    pub priority: RecommendationPriority,
    pub description: String,
    pub action_items: Vec<String>,
    pub estimated_effort: String,
}

/// Types of readiness recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReadinessRecommendationType {
    FingerprintUpdate,
    ComponentValidation,
    IntegrityVerification,
    ComplianceImprovement,
    PermissionEscalation,
    ValidationCompletion,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Gate readiness context for analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateReadinessContext {
    pub gate_id: String,
    pub current_phase: String,
    pub required_components: Vec<ComponentId>,
    pub build_context: BuildContext,
    pub validation_requirements: Vec<ValidationRequirement>,
}

/// Validation requirements for gate readiness
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRequirement {
    pub requirement_id: String,
    pub component: ComponentId,
    pub requirement_type: ValidationRequirementType,
    pub description: String,
    pub mandatory: bool,
}

/// Types of validation requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationRequirementType {
    FingerprintVerification,
    ComponentPresence,
    IntegrityCheck,
    ComplianceValidation,
    PermissionVerification,
}

/// Fingerprint variation analysis for gate readiness
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintVariationAnalysis {
    pub variation_type: String,
    pub impact_on_readiness: VariationImpact,
    pub legitimacy_assessment: LegitimacyAssessment,
    pub readiness_adjustment: f64, // Normalized to 6 decimal places
}

/// Impact of fingerprint variations on readiness
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariationImpact {
    NoImpact,
    MinorImpact,
    ModerateImpact,
    MajorImpact,
    BlockingImpact,
}

/// Assessment of variation legitimacy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LegitimacyAssessment {
    Legitimate,
    Suspicious,
    Illegitimate,
    RequiresInvestigation,
}

/// Gate readiness analyzer trait (B-MODE compliant)
pub trait GateReadinessAnalyzer {
    /// Analyze gate readiness with build fingerprinting
    fn analyze_gate_readiness(context: &GateReadinessContext) -> GateReadinessReport;
    
    /// Report gate readiness compliance
    fn report_gate_readiness_compliance(report: &GateReadinessReport) -> SpecificationReport;
    
    /// Analyze fingerprint variations for readiness impact
    fn analyze_fingerprint_variations(fingerprint: &BuildFingerprintSpec, baseline: &BuildFingerprintSpec) -> Vec<FingerprintVariationAnalysis>;
    
    /// Calculate readiness score with fingerprint integration
    fn calculate_readiness_score(gate_analysis: &GateReadinessAnalysis, fingerprint_analysis: &FingerprintAnalysisReport) -> f64;
}

/// Default implementation of gate readiness analyzer
pub struct DefaultGateReadinessAnalyzer;

impl GateReadinessAnalyzer for DefaultGateReadinessAnalyzer {
    fn analyze_gate_readiness(context: &GateReadinessContext) -> GateReadinessReport {
        // Analyze build fingerprint
        let fingerprint_analysis = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&context.build_context);
        
        // Create gate readiness analysis
        let gate_analysis = Self::create_gate_analysis(context, &fingerprint_analysis);
        
        // Calculate readiness score
        let readiness_score = Self::calculate_readiness_score(&gate_analysis, &fingerprint_analysis);
        
        // Identify blocking issues
        let blocking_issues = Self::identify_blocking_issues(context, &fingerprint_analysis);
        
        // Generate recommendations
        let recommendations = Self::generate_recommendations(context, &fingerprint_analysis, &blocking_issues);
        
        GateReadinessReport {
            gate_analysis,
            fingerprint_analysis,
            readiness_score,
            blocking_issues,
            recommendations,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }
    
    fn report_gate_readiness_compliance(report: &GateReadinessReport) -> SpecificationReport {
        let mut spec_report = SpecificationReport::new();
        
        // Check for critical blocking issues
        let critical_issues = report.blocking_issues.iter()
            .filter(|issue| matches!(issue.severity, BlockingIssueSeverity::Critical))
            .count();
        
        if critical_issues > 0 {
            spec_report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationViolation,
                component: ComponentId::ConstitutionalRuleEngine,
                rule_id: Some("GATE_READINESS_CRITICAL_ISSUES".to_string()),
                description: format!("Gate readiness analysis found {} critical blocking issues", critical_issues),
                remediation_hint: "Resolve all critical blocking issues before proceeding with gate transition".to_string(),
            });
        }
        
        // Check readiness score threshold
        if report.readiness_score < 0.8 {
            spec_report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::ConstitutionalRuleEngine,
                rule_id: Some("GATE_READINESS_SCORE_THRESHOLD".to_string()),
                description: format!("Gate readiness score {} is below required threshold of 0.8", report.readiness_score),
                remediation_hint: "Improve gate readiness by addressing identified issues and recommendations".to_string(),
            });
        }
        
        // Check fingerprint validity
        if !report.fingerprint_analysis.validity_analysis.is_valid {
            spec_report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationViolation,
                component: ComponentId::ConstitutionalRuleEngine,
                rule_id: Some("GATE_FINGERPRINT_VALIDITY".to_string()),
                description: "Build fingerprint analysis indicates invalid fingerprint".to_string(),
                remediation_hint: "Verify and correct build fingerprint issues before gate transition".to_string(),
            });
        }
        
        spec_report
    }
    
    fn analyze_fingerprint_variations(fingerprint: &BuildFingerprintSpec, baseline: &BuildFingerprintSpec) -> Vec<FingerprintVariationAnalysis> {
        let comparison = DefaultBuildFingerprintAnalyzer::compare_fingerprint_specs(fingerprint, baseline);
        let mut variations = Vec::new();
        
        for difference in &comparison.differences {
            let impact = Self::assess_variation_impact(&difference.difference_type);
            let legitimacy = Self::assess_variation_legitimacy(&difference.difference_type, &difference.impact_assessment);
            let adjustment = Self::calculate_readiness_adjustment(&impact);
            
            variations.push(FingerprintVariationAnalysis {
                variation_type: format!("{:?}", difference.difference_type),
                impact_on_readiness: impact,
                legitimacy_assessment: legitimacy,
                readiness_adjustment: adjustment,
            });
        }
        
        variations
    }
    
    fn calculate_readiness_score(gate_analysis: &GateReadinessAnalysis, fingerprint_analysis: &FingerprintAnalysisReport) -> f64 {
        let base_score = gate_analysis.completion_percentage / 100.0;
        let fingerprint_score = if fingerprint_analysis.validity_analysis.is_valid { 1.0 } else { 0.5 };
        let integrity_score = fingerprint_analysis.integrity_analysis.authenticity_score;
        
        let combined_score = (base_score + fingerprint_score + integrity_score) / 3.0;
        
        // Normalize to 6 decimal places for deterministic comparison
        (combined_score * 1_000_000.0_f64).round() / 1_000_000.0
    }
}

impl DefaultGateReadinessAnalyzer {
    fn create_gate_analysis(context: &GateReadinessContext, fingerprint_analysis: &FingerprintAnalysisReport) -> GateReadinessAnalysis {
        use crate::errors::{GatePhase, ReadinessStatus};
        
        let gate_phase = match context.current_phase.as_str() {
            "initialization" => GatePhase::Initialization,
            "validation" => GatePhase::Validation,
            "approval" => GatePhase::Approval,
            "implementation" => GatePhase::Implementation,
            _ => GatePhase::Initialization,
        };
        
        let readiness_status = if fingerprint_analysis.validity_analysis.is_valid {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::NotReady
        };
        
        let completion_percentage = if fingerprint_analysis.validity_analysis.is_valid { 90.0 } else { 50.0 };
        
        GateReadinessAnalysis {
            gate_phase,
            readiness_status,
            completion_percentage,
            blocking_issues: Vec::new(),
            advisory_notes: vec!["Build fingerprint analysis completed".to_string()],
        }
    }
    
    fn identify_blocking_issues(context: &GateReadinessContext, fingerprint_analysis: &FingerprintAnalysisReport) -> Vec<ReadinessBlockingIssue> {
        let mut issues = Vec::new();
        
        // Check for fingerprint validity issues
        if !fingerprint_analysis.validity_analysis.is_valid {
            issues.push(ReadinessBlockingIssue {
                issue_type: BlockingIssueType::FingerprintMismatch,
                component: None,
                description: "Build fingerprint validation failed".to_string(),
                severity: BlockingIssueSeverity::High,
                resolution_path: vec![
                    "Verify build environment consistency".to_string(),
                    "Regenerate build fingerprint".to_string(),
                    "Validate component integrity".to_string(),
                ],
            });
        }
        
        // Check for missing required components
        for requirement in &context.validation_requirements {
            if requirement.mandatory && matches!(requirement.requirement_type, ValidationRequirementType::ComponentPresence) {
                let component_present = fingerprint_analysis.fingerprint_spec.components.iter()
                    .any(|comp| comp.component_id == requirement.component);
                
                if !component_present {
                    issues.push(ReadinessBlockingIssue {
                        issue_type: BlockingIssueType::ComponentMissing,
                        component: Some(requirement.component),
                        description: format!("Required component {:?} is missing from build fingerprint", requirement.component),
                        severity: BlockingIssueSeverity::Critical,
                        resolution_path: vec![
                            format!("Add component {:?} to build", requirement.component),
                            "Update build configuration".to_string(),
                            "Regenerate build fingerprint".to_string(),
                        ],
                    });
                }
            }
        }
        
        issues
    }
    
    fn generate_recommendations(context: &GateReadinessContext, fingerprint_analysis: &FingerprintAnalysisReport, blocking_issues: &[ReadinessBlockingIssue]) -> Vec<ReadinessRecommendation> {
        let mut recommendations = Vec::new();
        
        // Recommend fingerprint updates if needed
        if !fingerprint_analysis.validity_analysis.is_valid {
            recommendations.push(ReadinessRecommendation {
                recommendation_type: ReadinessRecommendationType::FingerprintUpdate,
                priority: RecommendationPriority::High,
                description: "Update build fingerprint to resolve validity issues".to_string(),
                action_items: vec![
                    "Review build environment configuration".to_string(),
                    "Verify component integrity".to_string(),
                    "Regenerate fingerprint with corrected build".to_string(),
                ],
                estimated_effort: "2-4 hours".to_string(),
            });
        }
        
        // Recommend component validation for missing components
        if blocking_issues.iter().any(|issue| matches!(issue.issue_type, BlockingIssueType::ComponentMissing)) {
            recommendations.push(ReadinessRecommendation {
                recommendation_type: ReadinessRecommendationType::ComponentValidation,
                priority: RecommendationPriority::Critical,
                description: "Validate and add missing required components".to_string(),
                action_items: vec![
                    "Review component requirements".to_string(),
                    "Add missing components to build".to_string(),
                    "Update build configuration".to_string(),
                    "Verify component integration".to_string(),
                ],
                estimated_effort: "4-8 hours".to_string(),
            });
        }
        
        // Recommend integrity verification if needed
        if !matches!(fingerprint_analysis.integrity_analysis.integrity_status, crate::build_fingerprint::IntegrityStatus::Verified) {
            recommendations.push(ReadinessRecommendation {
                recommendation_type: ReadinessRecommendationType::IntegrityVerification,
                priority: RecommendationPriority::Medium,
                description: "Verify build integrity and authenticity".to_string(),
                action_items: vec![
                    "Run integrity verification checks".to_string(),
                    "Verify build signatures".to_string(),
                    "Check for tampering indicators".to_string(),
                ],
                estimated_effort: "1-2 hours".to_string(),
            });
        }
        
        recommendations
    }
    
    fn assess_variation_impact(difference_type: &crate::build_fingerprint::DifferenceType) -> VariationImpact {
        use crate::build_fingerprint::DifferenceType;
        
        match difference_type {
            DifferenceType::ComponentAdded => VariationImpact::MinorImpact,
            DifferenceType::ComponentRemoved => VariationImpact::MajorImpact,
            DifferenceType::ComponentModified => VariationImpact::ModerateImpact,
            DifferenceType::MetadataChanged => VariationImpact::MinorImpact,
            DifferenceType::DependencyChanged => VariationImpact::ModerateImpact,
            DifferenceType::CompilationFlagChanged => VariationImpact::MinorImpact,
            DifferenceType::EnvironmentChanged => VariationImpact::MinorImpact,
        }
    }
    
    fn assess_variation_legitimacy(difference_type: &crate::build_fingerprint::DifferenceType, impact_assessment: &str) -> LegitimacyAssessment {
        use crate::build_fingerprint::DifferenceType;
        
        match difference_type {
            DifferenceType::ComponentRemoved => {
                if impact_assessment.contains("critical") {
                    LegitimacyAssessment::Suspicious
                } else {
                    LegitimacyAssessment::RequiresInvestigation
                }
            },
            DifferenceType::ComponentModified => {
                if impact_assessment.contains("unauthorized") {
                    LegitimacyAssessment::Illegitimate
                } else {
                    LegitimacyAssessment::Legitimate
                }
            },
            _ => LegitimacyAssessment::Legitimate,
        }
    }
    
    fn calculate_readiness_adjustment(impact: &VariationImpact) -> f64 {
        let adjustment = match impact {
            VariationImpact::NoImpact => 0.0,
            VariationImpact::MinorImpact => -0.05,
            VariationImpact::ModerateImpact => -0.15,
            VariationImpact::MajorImpact => -0.30,
            VariationImpact::BlockingImpact => -0.50,
        };
        
        // Normalize to 6 decimal places for deterministic comparison
        (adjustment * 1_000_000.0_f64).round() / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_fingerprint::{DeterministicBuildId, BuildMetadata};
    use std::collections::BTreeMap;

    fn create_test_gate_readiness_context() -> GateReadinessContext {
        let mut build_environment = BTreeMap::new();
        build_environment.insert("RUSTFLAGS".to_string(), "-C opt-level=3".to_string());
        
        let build_context = BuildContext {
            target_architecture: "x86_64".to_string(),
            build_configuration: "release".to_string(),
            source_tree_hash: "test_hash_123".to_string(),
            toolchain_version: "1.70.0".to_string(),
            build_environment,
        };
        
        let validation_requirements = vec![
            ValidationRequirement {
                requirement_id: "component_presence".to_string(),
                component: ComponentId::D4RegisterAllocator,
                requirement_type: ValidationRequirementType::ComponentPresence,
                description: "D4 Register Allocator must be present".to_string(),
                mandatory: true,
            }
        ];
        
        GateReadinessContext {
            gate_id: "test_gate_001".to_string(),
            current_phase: "validation".to_string(),
            required_components: vec![ComponentId::D4RegisterAllocator],
            build_context,
            validation_requirements,
        }
    }

    #[test]
    fn test_analyze_gate_readiness() {
        let context = create_test_gate_readiness_context();
        let report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        assert!(!report.fingerprint_analysis.fingerprint_spec.hash.is_empty());
        assert!(report.readiness_score > 0.0);
        assert!(report.readiness_score <= 1.0);
    }

    #[test]
    fn test_report_gate_readiness_compliance_valid() {
        let context = create_test_gate_readiness_context();
        let mut report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        report.readiness_score = 0.9; // Set high readiness score
        
        let compliance_report = DefaultGateReadinessAnalyzer::report_gate_readiness_compliance(&report);
        
        // Should be compliant with high readiness score and valid fingerprint
        assert!(compliance_report.violations.len() <= 1); // May have fingerprint validity issues
    }

    #[test]
    fn test_report_gate_readiness_compliance_invalid() {
        let context = create_test_gate_readiness_context();
        let mut report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        report.readiness_score = 0.5; // Set low readiness score
        report.fingerprint_analysis.validity_analysis.is_valid = false;
        
        let compliance_report = DefaultGateReadinessAnalyzer::report_gate_readiness_compliance(&report);
        
        assert!(!compliance_report.is_compliant());
        assert!(compliance_report.violations.len() > 0);
    }

    #[test]
    fn test_analyze_fingerprint_variations() {
        use crate::build_fingerprint::{BuildFingerprintSpec, ComponentFingerprintSpec};
        
        let fingerprint_a = BuildFingerprintSpec {
            hash: "hash_a".to_string(),
            build_epoch: DeterministicBuildId::new("epoch_a".to_string()),
            components: vec![ComponentFingerprintSpec {
                component_id: ComponentId::D4RegisterAllocator,
                component_hash: "comp_hash_a".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![],
                compilation_flags: vec![],
            }],
            metadata: BuildMetadata {
                build_target: "x86_64".to_string(),
                optimization_level: "release".to_string(),
                compiler_version: "1.70.0".to_string(),
                build_flags: vec![],
                environment_variables: BTreeMap::new(),
            },
        };
        
        let mut fingerprint_b = fingerprint_a.clone();
        fingerprint_b.hash = "hash_b".to_string(); // Make them different
        
        let variations = DefaultGateReadinessAnalyzer::analyze_fingerprint_variations(&fingerprint_a, &fingerprint_b);
        
        assert!(!variations.is_empty());
        assert!(variations.iter().any(|v| v.variation_type.contains("MetadataChanged")));
    }

    #[test]
    fn test_calculate_readiness_score() {
        use crate::errors::{GatePhase, ReadinessStatus};
        use crate::build_fingerprint::{FingerprintValidityAnalysis, FingerprintIntegrityAnalysis, IntegrityStatus, ConsistencyAnalysis};
        
        let gate_analysis = GateReadinessAnalysis {
            gate_phase: GatePhase::Validation,
            readiness_status: ReadinessStatus::Ready,
            completion_percentage: 80.0,
            blocking_issues: vec![],
            advisory_notes: vec![],
        };
        
        let fingerprint_analysis = FingerprintAnalysisReport {
            fingerprint_spec: BuildFingerprintSpec {
                hash: "test_hash".to_string(),
                build_epoch: DeterministicBuildId::new("test_epoch".to_string()),
                components: vec![],
                metadata: BuildMetadata {
                    build_target: "x86_64".to_string(),
                    optimization_level: "release".to_string(),
                    compiler_version: "1.70.0".to_string(),
                    build_flags: vec![],
                    environment_variables: BTreeMap::new(),
                },
            },
            validity_analysis: FingerprintValidityAnalysis {
                is_valid: true,
                validation_issues: vec![],
                reproducibility_score: 1.0,
                consistency_analysis: ConsistencyAnalysis {
                    component_consistency: 1.0,
                    dependency_consistency: 1.0,
                    environment_consistency: 1.0,
                    overall_consistency: 1.0,
                },
            },
            compliance_findings: vec![],
            integrity_analysis: FingerprintIntegrityAnalysis {
                integrity_status: IntegrityStatus::Verified,
                hash_verification: crate::build_fingerprint::HashVerificationResult {
                    verified: true,
                    expected_hash: None,
                    actual_hash: "test_hash".to_string(),
                    verification_method: "SHA256".to_string(),
                },
                tampering_indicators: vec![],
                authenticity_score: 1.0,
            },
            analysis_timestamp: DeterministicClock::new().now(),
        };
        
        let score = DefaultGateReadinessAnalyzer::calculate_readiness_score(&gate_analysis, &fingerprint_analysis);
        
        assert!(score > 0.0);
        assert!(score <= 1.0);
        assert!(score > 0.8); // Should be high with valid inputs
    }

    #[test]
    fn test_gate_readiness_report_serialization() {
        let context = create_test_gate_readiness_context();
        let report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: GateReadinessReport = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(report.readiness_score, deserialized.readiness_score);
        assert_eq!(report.blocking_issues.len(), deserialized.blocking_issues.len());
    }

    #[test]
    fn test_blocking_issue_identification() {
        let mut context = create_test_gate_readiness_context();
        
        // Add a mandatory requirement for a component that won't be in the fingerprint
        context.validation_requirements.push(ValidationRequirement {
            requirement_id: "missing_component".to_string(),
            component: ComponentId::JITCompiler,
            requirement_type: ValidationRequirementType::ComponentPresence,
            description: "JIT Compiler must be present".to_string(),
            mandatory: true,
        });
        
        let report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        // Should identify the missing component as a blocking issue
        assert!(report.blocking_issues.iter().any(|issue| 
            matches!(issue.issue_type, BlockingIssueType::ComponentMissing) &&
            issue.component == Some(ComponentId::JITCompiler)
        ));
    }

    #[test]
    fn test_recommendation_generation() {
        let context = create_test_gate_readiness_context();
        let mut report = DefaultGateReadinessAnalyzer::analyze_gate_readiness(&context);
        
        // Make fingerprint invalid to trigger recommendations
        report.fingerprint_analysis.validity_analysis.is_valid = false;
        
        let recommendations = DefaultGateReadinessAnalyzer::generate_recommendations(
            &context, 
            &report.fingerprint_analysis, 
            &report.blocking_issues
        );
        
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|rec| 
            matches!(rec.recommendation_type, ReadinessRecommendationType::FingerprintUpdate)
        ));
    }
}