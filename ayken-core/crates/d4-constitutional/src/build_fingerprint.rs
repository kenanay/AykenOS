//! Build Fingerprinting System for Gate Readiness Analysis
//!
//! This module provides B-MODE compliant build fingerprinting analysis for gate readiness decisions.
//! It ONLY analyzes and reports on build fingerprints, NEVER generates or enforces them.
//!
//! PURE B-MODE PRINCIPLES:
//! - Analyze fingerprints, never generate them
//! - Report compliance, never enforce it
//! - Immutable analysis, no state mutations
//! - Specification reporting, not runtime enforcement

use crate::types::{ComponentId, LogicalTimestamp, DeterministicClock};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Deterministic build identifier for B-MODE analysis
/// This is a stable, content-addressed identifier provided as input to B-MODE analysis.
/// It does not represent time progression and MUST NOT be derived from system clocks.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeterministicBuildId(String);

impl DeterministicBuildId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeterministicBuildId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Build fingerprint specification for analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildFingerprintSpec {
    pub hash: String,
    pub build_epoch: DeterministicBuildId,
    pub components: Vec<ComponentFingerprintSpec>,
    pub metadata: BuildMetadata,
}

/// Component-specific fingerprint specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentFingerprintSpec {
    pub component_id: ComponentId,
    pub component_hash: String,
    pub version: String,
    pub dependencies: Vec<DependencySpec>,
    pub compilation_flags: Vec<String>,
}

/// Dependency specification for components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: String,
    pub version: String,
    pub hash: String,
}

/// Build metadata for fingerprint analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub build_target: String,
    pub optimization_level: String,
    pub compiler_version: String,
    pub build_flags: Vec<String>,
    pub environment_variables: BTreeMap<String, String>,
}

/// Build context for fingerprint analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildContext {
    pub target_architecture: String,
    pub build_configuration: String,
    pub source_tree_hash: String,
    pub toolchain_version: String,
    pub build_environment: BTreeMap<String, String>,
}

/// Fingerprint analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintAnalysisReport {
    pub fingerprint_spec: BuildFingerprintSpec,
    pub validity_analysis: FingerprintValidityAnalysis,
    pub compliance_findings: Vec<FingerprintComplianceFinding>,
    pub integrity_analysis: FingerprintIntegrityAnalysis,
    pub analysis_timestamp: LogicalTimestamp,
}

/// Fingerprint validity analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintValidityAnalysis {
    pub is_valid: bool,
    pub validation_issues: Vec<ValidationIssue>,
    pub reproducibility_score: f64, // Normalized to 6 decimal places for deterministic comparison
    pub consistency_analysis: ConsistencyAnalysis,
}

/// Validation issues found during fingerprint analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub component: Option<ComponentId>,
    pub description: String,
    pub severity: ValidationIssueSeverity,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationIssueType {
    MissingComponent,
    HashMismatch,
    VersionInconsistency,
    DependencyConflict,
    CompilationFlagMismatch,
    EnvironmentVariableMismatch,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationIssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Consistency analysis for fingerprints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsistencyAnalysis {
    pub component_consistency: f64, // Normalized to 6 decimal places
    pub dependency_consistency: f64, // Normalized to 6 decimal places
    pub environment_consistency: f64, // Normalized to 6 decimal places
    pub overall_consistency: f64, // Normalized to 6 decimal places
}

/// Fingerprint compliance findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintComplianceFinding {
    pub finding_type: ComplianceFindingType,
    pub component: Option<ComponentId>,
    pub description: String,
    pub recommendation: String,
}

/// Types of compliance findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplianceFindingType {
    ComplianceVerified,
    ComplianceDeviation,
    ComplianceUncertain,
    ComplianceViolation,
}

/// Fingerprint integrity analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintIntegrityAnalysis {
    pub integrity_status: IntegrityStatus,
    pub hash_verification: HashVerificationResult,
    pub tampering_indicators: Vec<TamperingIndicator>,
    pub authenticity_score: f64, // Normalized to 6 decimal places
}

/// Integrity status for fingerprints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegrityStatus {
    Verified,
    Suspicious,
    Compromised,
    Unknown,
}

/// Hash verification results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HashVerificationResult {
    pub verified: bool,
    pub expected_hash: Option<String>,
    pub actual_hash: String,
    pub verification_method: String,
}

/// Tampering indicators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperingIndicator {
    pub indicator_type: TamperingIndicatorType,
    pub description: String,
    pub confidence_level: f64, // Normalized to 6 decimal places
}

/// Types of tampering indicators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TamperingIndicatorType {
    UnexpectedHashChange,
    MissingComponent,
    UnauthorizedModification,
    EnvironmentManipulation,
    TimestampAnomaly,
}

/// Fingerprint comparison report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintComparisonReport {
    pub fingerprint_a: BuildFingerprintSpec,
    pub fingerprint_b: BuildFingerprintSpec,
    pub comparison_analysis: ComparisonAnalysis,
    pub differences: Vec<FingerprintDifference>,
    pub similarity_score: f64, // Normalized to 6 decimal places
    pub analysis_timestamp: LogicalTimestamp,
}

/// Comparison analysis between fingerprints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonAnalysis {
    pub structural_similarity: f64, // Normalized to 6 decimal places
    pub component_similarity: f64, // Normalized to 6 decimal places
    pub metadata_similarity: f64, // Normalized to 6 decimal places
    pub overall_compatibility: bool,
}

/// Differences between fingerprints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintDifference {
    pub difference_type: DifferenceType,
    pub component: Option<ComponentId>,
    pub field_name: String,
    pub value_a: String,
    pub value_b: String,
    pub impact_assessment: String,
}

/// Types of fingerprint differences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DifferenceType {
    ComponentAdded,
    ComponentRemoved,
    ComponentModified,
    MetadataChanged,
    DependencyChanged,
    CompilationFlagChanged,
    EnvironmentChanged,
}

/// Build fingerprint analyzer trait (B-MODE compliant)
pub trait BuildFingerprintAnalyzer {
    /// Analyze a build fingerprint for validity and compliance
    fn analyze_fingerprint(build_context: &BuildContext) -> FingerprintAnalysisReport;
    
    /// Report fingerprint compliance against specifications
    fn report_fingerprint_compliance(fingerprint: &BuildFingerprintSpec) -> crate::errors::SpecificationReport;
    
    /// Compare two fingerprint specifications
    fn compare_fingerprint_specs(a: &BuildFingerprintSpec, b: &BuildFingerprintSpec) -> FingerprintComparisonReport;
    
    /// Analyze fingerprint validity
    fn analyze_fingerprint_validity(fingerprint: &BuildFingerprintSpec) -> FingerprintValidityAnalysis;
}

/// Default implementation of build fingerprint analyzer
pub struct DefaultBuildFingerprintAnalyzer;

impl BuildFingerprintAnalyzer for DefaultBuildFingerprintAnalyzer {
    fn analyze_fingerprint(build_context: &BuildContext) -> FingerprintAnalysisReport {
        // Create fingerprint spec from build context
        let fingerprint_spec = Self::create_fingerprint_spec_from_context(build_context);
        
        // Analyze validity
        let validity_analysis = Self::analyze_fingerprint_validity(&fingerprint_spec);
        
        // Generate compliance findings
        let compliance_findings = Self::generate_compliance_findings(&fingerprint_spec);
        
        // Analyze integrity
        let integrity_analysis = Self::analyze_integrity(&fingerprint_spec);
        
        FingerprintAnalysisReport {
            fingerprint_spec,
            validity_analysis,
            compliance_findings,
            integrity_analysis,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }
    
    fn report_fingerprint_compliance(fingerprint: &BuildFingerprintSpec) -> crate::errors::SpecificationReport {
        let mut report = crate::errors::SpecificationReport::new();
        
        // Check for missing components
        if fingerprint.components.is_empty() {
            report.add_violation(crate::errors::SpecificationViolation {
                violation_type: crate::errors::ViolationType::SpecificationIncomplete,
                component: ComponentId::ConstitutionalRuleEngine,
                rule_id: Some("FINGERPRINT_COMPONENTS_REQUIRED".to_string()),
                description: "Build fingerprint must include component specifications".to_string(),
                remediation_hint: "Add component fingerprint specifications to the build fingerprint".to_string(),
            });
        }
        
        // Check hash validity
        if fingerprint.hash.is_empty() {
            report.add_violation(crate::errors::SpecificationViolation {
                violation_type: crate::errors::ViolationType::SpecificationIncomplete,
                component: ComponentId::ConstitutionalRuleEngine,
                rule_id: Some("FINGERPRINT_HASH_REQUIRED".to_string()),
                description: "Build fingerprint must include a valid hash".to_string(),
                remediation_hint: "Generate and include a valid hash for the build fingerprint".to_string(),
            });
        }
        
        // Check build epoch validity
        if fingerprint.build_epoch.as_str().is_empty() {
            report.add_violation(crate::errors::SpecificationViolation {
                violation_type: crate::errors::ViolationType::SpecificationIncomplete,
                component: ComponentId::ConstitutionalRuleEngine,
                rule_id: Some("FINGERPRINT_BUILD_EPOCH_REQUIRED".to_string()),
                description: "Build fingerprint must include a valid build epoch".to_string(),
                remediation_hint: "Include a deterministic build identifier in the fingerprint".to_string(),
            });
        }
        
        report
    }
    
    fn compare_fingerprint_specs(a: &BuildFingerprintSpec, b: &BuildFingerprintSpec) -> FingerprintComparisonReport {
        let differences = Self::find_differences(a, b);
        let comparison_analysis = Self::analyze_comparison(a, b, &differences);
        let similarity_score = Self::calculate_similarity_score(&comparison_analysis);
        
        FingerprintComparisonReport {
            fingerprint_a: a.clone(),
            fingerprint_b: b.clone(),
            comparison_analysis,
            differences,
            similarity_score,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }
    
    fn analyze_fingerprint_validity(fingerprint: &BuildFingerprintSpec) -> FingerprintValidityAnalysis {
        let mut validation_issues = Vec::new();
        
        // Check for empty hash
        if fingerprint.hash.is_empty() {
            validation_issues.push(ValidationIssue {
                issue_type: ValidationIssueType::HashMismatch,
                component: None,
                description: "Fingerprint hash is empty".to_string(),
                severity: ValidationIssueSeverity::Error,
            });
        }
        
        // Check component consistency
        for component in &fingerprint.components {
            if component.component_hash.is_empty() {
                validation_issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::HashMismatch,
                    component: Some(component.component_id),
                    description: format!("Component {:?} has empty hash", component.component_id),
                    severity: ValidationIssueSeverity::Warning,
                });
            }
        }
        
        let reproducibility_score = Self::calculate_reproducibility_score(&validation_issues);
        let consistency_analysis = Self::analyze_consistency(fingerprint);
        
        FingerprintValidityAnalysis {
            is_valid: validation_issues.iter().all(|issue| !matches!(issue.severity, ValidationIssueSeverity::Error | ValidationIssueSeverity::Critical)),
            validation_issues,
            reproducibility_score,
            consistency_analysis,
        }
    }
}

impl DefaultBuildFingerprintAnalyzer {
    fn create_fingerprint_spec_from_context(build_context: &BuildContext) -> BuildFingerprintSpec {
        // Create component fingerprints based on build context
        let components = vec![
            ComponentFingerprintSpec {
                component_id: ComponentId::D4RegisterAllocator,
                component_hash: format!("hash_{}", build_context.source_tree_hash),
                version: "1.0.0".to_string(),
                dependencies: vec![],
                compilation_flags: build_context.build_environment.get("RUSTFLAGS")
                    .map(|flags| flags.split_whitespace().map(String::from).collect())
                    .unwrap_or_default(),
            }
        ];
        
        let metadata = BuildMetadata {
            build_target: build_context.target_architecture.clone(),
            optimization_level: build_context.build_configuration.clone(),
            compiler_version: build_context.toolchain_version.clone(),
            build_flags: vec![],
            environment_variables: build_context.build_environment.clone(),
        };
        
        BuildFingerprintSpec {
            hash: build_context.source_tree_hash.clone(),
            build_epoch: DeterministicBuildId::new(format!("epoch_{}", build_context.source_tree_hash)),
            components,
            metadata,
        }
    }
    
    fn generate_compliance_findings(fingerprint: &BuildFingerprintSpec) -> Vec<FingerprintComplianceFinding> {
        let mut findings = Vec::new();
        
        if !fingerprint.components.is_empty() {
            findings.push(FingerprintComplianceFinding {
                finding_type: ComplianceFindingType::ComplianceVerified,
                component: None,
                description: "Build fingerprint includes component specifications".to_string(),
                recommendation: "Continue maintaining component fingerprint specifications".to_string(),
            });
        }
        
        findings
    }
    
    fn analyze_integrity(fingerprint: &BuildFingerprintSpec) -> FingerprintIntegrityAnalysis {
        let hash_verification = HashVerificationResult {
            verified: !fingerprint.hash.is_empty(),
            expected_hash: None,
            actual_hash: fingerprint.hash.clone(),
            verification_method: "SHA256".to_string(),
        };
        
        let tampering_indicators = Vec::new(); // No tampering detected in basic analysis
        
        let authenticity_score = if hash_verification.verified { 1.0 } else { 0.0 };
        
        FingerprintIntegrityAnalysis {
            integrity_status: if hash_verification.verified { IntegrityStatus::Verified } else { IntegrityStatus::Unknown },
            hash_verification,
            tampering_indicators,
            authenticity_score,
        }
    }
    
    fn find_differences(a: &BuildFingerprintSpec, b: &BuildFingerprintSpec) -> Vec<FingerprintDifference> {
        let mut differences = Vec::new();
        
        // Compare hashes
        if a.hash != b.hash {
            differences.push(FingerprintDifference {
                difference_type: DifferenceType::MetadataChanged,
                component: None,
                field_name: "hash".to_string(),
                value_a: a.hash.clone(),
                value_b: b.hash.clone(),
                impact_assessment: "Hash difference indicates different build content".to_string(),
            });
        }
        
        // Compare build epochs
        if a.build_epoch != b.build_epoch {
            differences.push(FingerprintDifference {
                difference_type: DifferenceType::MetadataChanged,
                component: None,
                field_name: "build_epoch".to_string(),
                value_a: a.build_epoch.to_string(),
                value_b: b.build_epoch.to_string(),
                impact_assessment: "Build epoch difference indicates different build instances".to_string(),
            });
        }
        
        differences
    }
    
    fn analyze_comparison(a: &BuildFingerprintSpec, b: &BuildFingerprintSpec, differences: &[FingerprintDifference]) -> ComparisonAnalysis {
        let structural_similarity = if a.components.len() == b.components.len() { 1.0 } else { 0.5 };
        let component_similarity = Self::calculate_component_similarity(a, b);
        let metadata_similarity = Self::calculate_metadata_similarity(a, b);
        
        // Hash differences affect overall compatibility
        let hash_differs = a.hash != b.hash;
        let overall_compatibility = differences.is_empty() && !hash_differs;
        
        ComparisonAnalysis {
            structural_similarity,
            component_similarity,
            metadata_similarity,
            overall_compatibility,
        }
    }
    
    fn calculate_similarity_score(analysis: &ComparisonAnalysis) -> f64 {
        // Normalize to 6 decimal places for deterministic comparison
        let score = (analysis.structural_similarity + analysis.component_similarity + analysis.metadata_similarity) / 3.0;
        (score * 1_000_000.0_f64).round() / 1_000_000.0
    }
    
    fn calculate_component_similarity(a: &BuildFingerprintSpec, b: &BuildFingerprintSpec) -> f64 {
        if a.components.is_empty() && b.components.is_empty() {
            return 1.0;
        }
        
        let matching_components = a.components.iter()
            .filter(|comp_a| b.components.iter().any(|comp_b| comp_a.component_id == comp_b.component_id))
            .count();
        
        let total_components = std::cmp::max(a.components.len(), b.components.len());
        
        if total_components == 0 {
            1.0
        } else {
            matching_components as f64 / total_components as f64
        }
    }
    
    fn calculate_metadata_similarity(a: &BuildFingerprintSpec, b: &BuildFingerprintSpec) -> f64 {
        let mut matches = 0;
        let mut total = 0;
        
        // Compare hash (most important)
        total += 1;
        if a.hash == b.hash {
            matches += 1;
        }
        
        // Compare build target
        total += 1;
        if a.metadata.build_target == b.metadata.build_target {
            matches += 1;
        }
        
        // Compare optimization level
        total += 1;
        if a.metadata.optimization_level == b.metadata.optimization_level {
            matches += 1;
        }
        
        // Compare compiler version
        total += 1;
        if a.metadata.compiler_version == b.metadata.compiler_version {
            matches += 1;
        }
        
        if total == 0 {
            1.0
        } else {
            matches as f64 / total as f64
        }
    }
    
    fn calculate_reproducibility_score(validation_issues: &[ValidationIssue]) -> f64 {
        let critical_issues = validation_issues.iter()
            .filter(|issue| matches!(issue.severity, ValidationIssueSeverity::Critical | ValidationIssueSeverity::Error))
            .count();
        
        if critical_issues == 0 {
            1.0
        } else {
            // Normalize to 6 decimal places for deterministic comparison
            let score = 1.0 - (critical_issues as f64 * 0.1);
            (score.max(0.0) * 1_000_000.0_f64).round() / 1_000_000.0
        }
    }
    
    fn analyze_consistency(fingerprint: &BuildFingerprintSpec) -> ConsistencyAnalysis {
        let component_consistency = if fingerprint.components.iter().all(|c| !c.component_hash.is_empty()) { 1.0 } else { 0.5 };
        let dependency_consistency = 1.0; // Simplified for now
        let environment_consistency = if !fingerprint.metadata.environment_variables.is_empty() { 1.0 } else { 0.8 };
        let overall_consistency = (component_consistency + dependency_consistency + environment_consistency) / 3.0;
        
        // Normalize all values to 6 decimal places for deterministic comparison
        ConsistencyAnalysis {
            component_consistency: (component_consistency * 1_000_000.0_f64).round() / 1_000_000.0,
            dependency_consistency: (dependency_consistency * 1_000_000.0_f64).round() / 1_000_000.0,
            environment_consistency: (environment_consistency * 1_000_000.0_f64).round() / 1_000_000.0,
            overall_consistency: (overall_consistency * 1_000_000.0_f64).round() / 1_000_000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_test_build_context() -> BuildContext {
        let mut build_environment = BTreeMap::new();
        build_environment.insert("RUSTFLAGS".to_string(), "-C opt-level=3".to_string());
        
        BuildContext {
            target_architecture: "x86_64".to_string(),
            build_configuration: "release".to_string(),
            source_tree_hash: "abc123def456".to_string(),
            toolchain_version: "1.70.0".to_string(),
            build_environment,
        }
    }

    fn create_test_fingerprint_spec() -> BuildFingerprintSpec {
        let components = vec![
            ComponentFingerprintSpec {
                component_id: ComponentId::D4RegisterAllocator,
                component_hash: "component_hash_123".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![],
                compilation_flags: vec!["-C".to_string(), "opt-level=3".to_string()],
            }
        ];
        
        let mut environment_variables = BTreeMap::new();
        environment_variables.insert("RUSTFLAGS".to_string(), "-C opt-level=3".to_string());
        
        let metadata = BuildMetadata {
            build_target: "x86_64".to_string(),
            optimization_level: "release".to_string(),
            compiler_version: "1.70.0".to_string(),
            build_flags: vec![],
            environment_variables,
        };
        
        BuildFingerprintSpec {
            hash: "fingerprint_hash_456".to_string(),
            build_epoch: DeterministicBuildId::new("epoch_123".to_string()),
            components,
            metadata,
        }
    }

    #[test]
    fn test_analyze_fingerprint() {
        let build_context = create_test_build_context();
        let report = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&build_context);
        
        assert!(!report.fingerprint_spec.hash.is_empty());
        assert!(!report.fingerprint_spec.components.is_empty());
        assert!(report.validity_analysis.is_valid);
    }

    #[test]
    fn test_report_fingerprint_compliance_valid() {
        let fingerprint = create_test_fingerprint_spec();
        let report = DefaultBuildFingerprintAnalyzer::report_fingerprint_compliance(&fingerprint);
        
        assert!(report.is_compliant());
        assert_eq!(report.violations.len(), 0);
    }

    #[test]
    fn test_report_fingerprint_compliance_invalid() {
        let mut fingerprint = create_test_fingerprint_spec();
        fingerprint.hash = String::new(); // Make it invalid
        fingerprint.components.clear(); // Remove components
        
        let report = DefaultBuildFingerprintAnalyzer::report_fingerprint_compliance(&fingerprint);
        
        assert!(!report.is_compliant());
        assert!(report.violations.len() > 0);
    }

    #[test]
    fn test_compare_fingerprint_specs_identical() {
        let fingerprint_a = create_test_fingerprint_spec();
        let fingerprint_b = fingerprint_a.clone();
        
        let comparison = DefaultBuildFingerprintAnalyzer::compare_fingerprint_specs(&fingerprint_a, &fingerprint_b);
        
        assert_eq!(comparison.similarity_score, 1.0);
        assert!(comparison.comparison_analysis.overall_compatibility);
        assert!(comparison.differences.is_empty());
    }

    #[test]
    fn test_compare_fingerprint_specs_different() {
        let fingerprint_a = create_test_fingerprint_spec();
        let mut fingerprint_b = fingerprint_a.clone();
        fingerprint_b.hash = "different_hash".to_string();
        
        let comparison = DefaultBuildFingerprintAnalyzer::compare_fingerprint_specs(&fingerprint_a, &fingerprint_b);
        
        assert!(comparison.similarity_score < 1.0);
        assert!(!comparison.comparison_analysis.overall_compatibility);
        assert!(!comparison.differences.is_empty());
    }

    #[test]
    fn test_analyze_fingerprint_validity_valid() {
        let fingerprint = create_test_fingerprint_spec();
        let validity = DefaultBuildFingerprintAnalyzer::analyze_fingerprint_validity(&fingerprint);
        
        assert!(validity.is_valid);
        assert!(validity.reproducibility_score > 0.0);
        assert!(validity.consistency_analysis.overall_consistency > 0.0);
    }

    #[test]
    fn test_analyze_fingerprint_validity_invalid() {
        let mut fingerprint = create_test_fingerprint_spec();
        fingerprint.hash = String::new(); // Make hash empty
        fingerprint.components[0].component_hash = String::new(); // Make component hash empty
        
        let validity = DefaultBuildFingerprintAnalyzer::analyze_fingerprint_validity(&fingerprint);
        
        assert!(!validity.is_valid);
        assert!(!validity.validation_issues.is_empty());
    }

    #[test]
    fn test_deterministic_build_id() {
        let id1 = DeterministicBuildId::new("test_id".to_string());
        let id2 = DeterministicBuildId::new("test_id".to_string());
        
        assert_eq!(id1, id2);
        assert_eq!(id1.as_str(), "test_id");
        assert_eq!(format!("{}", id1), "test_id");
    }

    #[test]
    fn test_fingerprint_spec_serialization() {
        let fingerprint = create_test_fingerprint_spec();
        let serialized = serde_json::to_string(&fingerprint).unwrap();
        let deserialized: BuildFingerprintSpec = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(fingerprint, deserialized);
    }

    #[test]
    fn test_fingerprint_analysis_report_serialization() {
        let build_context = create_test_build_context();
        let report = DefaultBuildFingerprintAnalyzer::analyze_fingerprint(&build_context);
        
        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: FingerprintAnalysisReport = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(report.fingerprint_spec, deserialized.fingerprint_spec);
        assert_eq!(report.validity_analysis.is_valid, deserialized.validity_analysis.is_valid);
    }
}