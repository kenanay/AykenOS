//! Specification Reports for D4 Constitutional Framework
//!
//! This module defines comprehensive reporting for constitutional violations,
//! contract analysis, and system findings that occur during framework specification.
//! 
//! **PURE B-MODE PRINCIPLE:** This module ONLY reports findings, NEVER blocks execution.
//! All APIs return reports with violation lists instead of Result<()> for specification issues.

use crate::types::{ComponentId, RuleId, Severity, DeterministicClock, LogicalTimestamp};
use crate::bmode::templates::TemplateType;
use serde::{Deserialize, Serialize};

/// B-MODE Contract Violation Report (replaces ContractValidationFailure error)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractViolationReport {
    pub findings: Vec<ContractFinding>,
    pub violations: Vec<ContractViolation>,
    pub recommendations: Vec<ContractRecommendation>,
    pub analysis_timestamp: LogicalTimestamp,
    pub component_context: ComponentId,
}

/// Contract analysis findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractFinding {
    InvalidStructure { component: String, reason: String },
    MissingTemplate { template_type: TemplateType },
    RuleBreach { rule: String, context: String },
    IncompleteSpecification { specification_type: String, missing_elements: Vec<String> },
    DocumentationGap { element: String, severity: Severity },
}

/// Contract specification violations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractViolation {
    pub violation_id: String,
    pub component: ComponentId,
    pub violation_type: ContractViolationType,
    pub description: String,
    pub formal_specification: String,
    pub context: std::collections::BTreeMap<String, String>,
}

/// Types of contract violations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractViolationType {
    MissingStructDefinition,
    MissingTraitDefinition,
    InvariantViolation,
    PropertyTestFailure,
    PerformanceTargetUndefined,
    IncompleteImplementation,
}

/// Contract improvement recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractRecommendation {
    pub recommendation_id: String,
    pub priority: RecommendationPriority,
    pub description: String,
    pub remediation_steps: Vec<String>,
    pub estimated_effort: String,
}

/// Recommendation priority levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// B-MODE Gate Decision Report (replaces GateValidationError)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateDecisionReport {
    pub readiness_analysis: GateReadinessAnalysis,
    pub transition_findings: Vec<TransitionFinding>,
    pub permission_analysis: PermissionAnalysis,
    pub fingerprint_analysis: FingerprintAnalysis,
    pub decision_timestamp: LogicalTimestamp,
    pub overall_readiness_score: f64,
}

/// Gate readiness analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateReadinessAnalysis {
    pub gate_phase: GatePhase,
    pub readiness_status: ReadinessStatus,
    pub completion_percentage: f64,
    pub blocking_issues: Vec<String>,
    pub advisory_notes: Vec<String>,
}

/// Gate phases
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GatePhase {
    Initialization,
    Validation,
    Approval,
    Implementation,
}

/// Readiness status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReadinessStatus {
    NotReady,
    PartiallyReady,
    Ready,
    Approved,
}

/// Transition analysis findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionFinding {
    pub finding_type: TransitionFindingType,
    pub from_state: String,
    pub to_state: String,
    pub analysis_result: String,
    pub recommendations: Vec<String>,
}

/// Types of transition findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransitionFindingType {
    TransitionAllowed,
    TransitionBlocked,
    TransitionConditional,
    TransitionRequiresApproval,
}

/// Permission analysis results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionAnalysis {
    pub required_permissions: Vec<Permission>,
    pub available_permissions: Vec<Permission>,
    pub permission_gaps: Vec<PermissionGap>,
    pub authorization_recommendations: Vec<String>,
}

/// Permission definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    pub permission_id: String,
    pub permission_type: PermissionType,
    pub scope: String,
    pub authority_level: AuthorityLevel,
}

/// Permission types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionType {
    Read,
    Write,
    Execute,
    Modify,
    Approve,
    Override,
}

/// Authority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthorityLevel {
    Component,
    System,
    Constitutional,
    Administrative,
}

/// Permission gaps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionGap {
    pub required_permission: Permission,
    pub gap_severity: Severity,
    pub resolution_path: Vec<String>,
}

/// Build fingerprint analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintAnalysis {
    pub current_fingerprint: String,
    pub expected_fingerprint: Option<String>,
    pub fingerprint_validity: FingerprintValidity,
    pub variation_analysis: Vec<FingerprintVariation>,
    pub integrity_status: IntegrityStatus,
}

/// Fingerprint validity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FingerprintValidity {
    Valid,
    Invalid,
    Suspicious,
    RequiresVerification,
}

/// Fingerprint variations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerprintVariation {
    pub variation_type: String,
    pub impact_assessment: String,
    pub legitimacy_analysis: String,
}

/// Integrity status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegrityStatus {
    Verified,
    Compromised,
    Unknown,
    PendingVerification,
}

/// Detailed information about constitutional rule violations (for reporting, not blocking)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalViolation {
    /// Unique identifier of the violated rule
    pub rule_id: RuleId,
    /// Component that attempted the violation
    pub violating_component: ComponentId,
    /// Operation that was attempted
    pub attempted_operation: String,
    /// Severity of the violation
    pub violation_severity: Severity,
    /// Detailed description of the violation
    pub description: String,
    /// Timestamp when the violation occurred
    pub timestamp: LogicalTimestamp,
    /// Additional context information
    pub context: std::collections::BTreeMap<String, String>,
}

impl std::fmt::Display for ConstitutionalViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Constitutional violation by {:?}: {} (severity: {:?})",
            self.violating_component, self.description, self.violation_severity
        )
    }
}

impl ConstitutionalViolation {
    /// Create a new constitutional violation
    pub fn new(
        rule_id: RuleId,
        violating_component: ComponentId,
        attempted_operation: String,
        violation_severity: Severity,
        description: String,
    ) -> Self {
        Self {
            rule_id,
            violating_component,
            attempted_operation,
            violation_severity,
            description,
            timestamp: DeterministicClock::new().now(),
            context: std::collections::BTreeMap::new(),
        }
    }

    /// Add context information to the violation
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }

    /// Check if this is a critical violation
    pub fn is_critical(&self) -> bool {
        self.violation_severity == Severity::Critical
    }
}

/// B-MODE Specification Report (replaces Result<()> for specification violations)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationReport {
    pub violations: Vec<SpecificationViolation>,
    pub findings: Vec<SpecificationFinding>,
    pub recommendations: Vec<SpecificationRecommendation>,
    pub compliance_score: f64,
    pub report_timestamp: LogicalTimestamp,
}

/// Specification violations (reported, not thrown)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationViolation {
    pub violation_type: ViolationType,
    pub component: ComponentId,
    pub rule_id: Option<String>,
    pub description: String,
    pub remediation_hint: String,
}

/// Specification analysis findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationFinding {
    pub finding_type: FindingType,
    pub component: ComponentId,
    pub description: String,
    pub severity: Severity,
    pub location: crate::bmode::validation_location::ValidationLocation,
}

/// Specification recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationRecommendation {
    pub recommendation_type: RecommendationType,
    pub component: ComponentId,
    pub description: String,
    pub priority: Priority,
}

/// Violation types for specification violations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationType {
    SpecificationIncomplete,
    SpecificationInconsistent,
    SpecificationViolation,
    StructuralIntegrityViolation,
    SemanticConsistencyViolation,
    RuntimeSemanticsMixing,
    UnauthorizedOperation,
    DeterminismViolation,
    IncompleteSpecification,
    ConstitutionalViolation,
    InvalidConfiguration,
    TemplateViolation,
    GateTransitionViolation,
    ComplianceViolation,
}

/// Finding types for specification findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FindingType {
    SpecificationCompliance,
    StructuralCompliance,
    SemanticCompliance,
    PerformanceCompliance,
    SecurityCompliance,
    PatternValidityNotProven,
    BoundaryViolation,
    PolicyDeclaration,
}

/// Recommendation types for specification recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationType {
    StructuralImprovement,
    SemanticImprovement,
    PerformanceImprovement,
    SecurityImprovement,
    DocumentationImprovement,
    BoundaryEnforcement,
}

/// Priority levels for recommendations (alias for RecommendationPriority)
pub type Priority = RecommendationPriority;

/// Analysis summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_violations: usize,
    pub critical_violations: usize,
    pub compliance_score: f64,
    pub analysis_completeness: f64,
    pub next_steps: Vec<String>,
}

impl SpecificationReport {
    /// Create a new empty specification report
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            findings: Vec::new(),
            recommendations: Vec::new(),
            compliance_score: 1.0,
            report_timestamp: DeterministicClock::new().now(),
        }
    }

    /// Add a violation to the report
    pub fn add_violation(&mut self, violation: SpecificationViolation) {
        self.violations.push(violation);
        self.update_compliance_score();
    }

    /// Add a finding to the report
    pub fn add_finding(&mut self, finding: SpecificationFinding) {
        self.findings.push(finding);
    }

    /// Add a recommendation to the report
    pub fn add_recommendation(&mut self, recommendation: SpecificationRecommendation) {
        self.recommendations.push(recommendation);
    }

    /// Check if the specification is compliant (no violations)
    pub fn is_compliant(&self) -> bool {
        self.violations.is_empty()
    }

    /// Update compliance score based on violations with severity weighting
    fn update_compliance_score(&mut self) {
        if self.violations.is_empty() {
            self.compliance_score = 1.0;
        } else {
            // Calculate severity-weighted compliance score
            let total_severity_weight: f64 = self.violations.iter()
                .map(|v| match v.violation_type {
                    ViolationType::ConstitutionalViolation => 1.0,
                    ViolationType::SpecificationIncomplete => 0.8,
                    ViolationType::UnauthorizedOperation => 1.0,
                    ViolationType::InvalidConfiguration => 0.6,
                    ViolationType::TemplateViolation => 0.7,
                    ViolationType::GateTransitionViolation => 0.9,
                    ViolationType::ComplianceViolation => 0.8,
                    ViolationType::SpecificationInconsistent => 0.7,
                    ViolationType::SpecificationViolation => 0.8,
                    ViolationType::StructuralIntegrityViolation => 0.9,
                    ViolationType::SemanticConsistencyViolation => 0.8,
                    ViolationType::RuntimeSemanticsMixing => 1.0,
                    ViolationType::DeterminismViolation => 1.0,
                    ViolationType::IncompleteSpecification => 0.6,
                })
                .sum();
            
            // Base compliance starts at 1.0, reduced by severity-weighted violations
            // Each violation reduces compliance proportionally to its severity
            let max_possible_weight = self.violations.len() as f64; // If all were constitutional violations
            let violation_impact = total_severity_weight / (max_possible_weight + 1.0); // +1 to prevent division issues
            
            self.compliance_score = (1.0 - violation_impact).max(0.0);
        }
    }

    /// Merge another report into this one
    pub fn merge(&mut self, other: SpecificationReport) {
        for violation in other.violations {
            self.add_violation(violation);
        }
        for finding in other.findings {
            self.add_finding(finding);
        }
        for recommendation in other.recommendations {
            self.add_recommendation(recommendation);
        }
    }
}

impl Default for SpecificationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractViolationReport {
    /// Create a new contract violation report
    pub fn new(component: ComponentId) -> Self {
        Self {
            findings: Vec::new(),
            violations: Vec::new(),
            recommendations: Vec::new(),
            analysis_timestamp: DeterministicClock::new().now(),
            component_context: component,
        }
    }

    /// Add a contract finding
    pub fn add_finding(&mut self, finding: ContractFinding) {
        self.findings.push(finding);
    }

    /// Add a contract violation
    pub fn add_violation(&mut self, violation: ContractViolation) {
        self.violations.push(violation);
    }

    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: ContractRecommendation) {
        self.recommendations.push(recommendation);
    }

    /// Check if there are any critical violations
    pub fn has_critical_violations(&self) -> bool {
        self.violations.iter().any(|v| match v.violation_type {
            ContractViolationType::MissingStructDefinition => true,
            ContractViolationType::MissingTraitDefinition => true,
            ContractViolationType::InvariantViolation => true,
            _ => false,
        })
    }
}

impl GateDecisionReport {
    /// Create a new gate decision report
    pub fn new() -> Self {
        Self {
            readiness_analysis: GateReadinessAnalysis {
                gate_phase: GatePhase::Initialization,
                readiness_status: ReadinessStatus::NotReady,
                completion_percentage: 0.0,
                blocking_issues: Vec::new(),
                advisory_notes: Vec::new(),
            },
            transition_findings: Vec::new(),
            permission_analysis: PermissionAnalysis {
                required_permissions: Vec::new(),
                available_permissions: Vec::new(),
                permission_gaps: Vec::new(),
                authorization_recommendations: Vec::new(),
            },
            fingerprint_analysis: FingerprintAnalysis {
                current_fingerprint: String::new(),
                expected_fingerprint: None,
                fingerprint_validity: FingerprintValidity::RequiresVerification,
                variation_analysis: Vec::new(),
                integrity_status: IntegrityStatus::PendingVerification,
            },
            decision_timestamp: DeterministicClock::new().now(),
            overall_readiness_score: 0.0,
        }
    }

    /// Check if gate transition is ready
    pub fn is_ready_for_transition(&self) -> bool {
        matches!(self.readiness_analysis.readiness_status, ReadinessStatus::Ready | ReadinessStatus::Approved)
            && self.readiness_analysis.blocking_issues.is_empty()
            && self.overall_readiness_score >= 0.8
    }
}

/// Property test failure information for debugging (now reported, not thrown)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyTestFailureInfo {
    pub property_name: String,
    pub test_case_id: String,
    pub seed: u64,
    pub shrunk_input: String,
    pub failure_reason: String,
    pub ir_fingerprint: String,
    pub failure_scenario_id: String,
    pub stack_trace: Option<String>,
    pub reproduction_command: String,
}

impl PropertyTestFailureInfo {
    /// Generate a reproduction command for the failed test
    pub fn generate_reproduction_command(&self) -> String {
        format!(
            "cargo test {} -- --seed {} --case-id {} --ir-fingerprint {} --scenario-id {}",
            self.property_name,
            self.seed,
            self.test_case_id,
            self.ir_fingerprint,
            self.failure_scenario_id
        )
    }

    /// Convert to specification violation for reporting
    pub fn to_specification_violation(&self, component: ComponentId) -> SpecificationViolation {
        SpecificationViolation {
            violation_type: ViolationType::SpecificationViolation,
            component,
            rule_id: Some(self.property_name.clone()),
            description: format!("Property test failed: {} - {}", self.property_name, self.failure_reason),
            remediation_hint: format!("Fix the property test failure. Reproduction command: {}", self.generate_reproduction_command()),
        }
    }
}

/// Helper trait for adding context to reports (B-MODE compliant)
pub trait ReportContext<T> {
    fn with_context(self, context: &str) -> SpecificationReport;
}

impl<T, E> ReportContext<T> for std::result::Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context(self, context: &str) -> SpecificationReport {
        match self {
            Ok(_) => SpecificationReport::new(),
            Err(e) => {
                let mut report = SpecificationReport::new();
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationViolation,
                    component: ComponentId::ConstitutionalRuleEngine, // Default component
                    rule_id: Some("CONTEXT_HANDLING".to_string()),
                    description: format!("{}: {}", context, e),
                    remediation_hint: "Check the context and resolve the underlying issue".to_string(),
                });
                report
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, RuleId, Severity};

    #[test]
    fn test_constitutional_violation_creation() {
        let rule_id = RuleId::from_content(b"test_constitutional_violation");
        let violation = ConstitutionalViolation::new(
            rule_id.clone(),
            ComponentId::JITCompiler,
            "register_rewrite".to_string(),
            Severity::Critical,
            "JIT attempted to rewrite register allocation".to_string(),
        );

        assert_eq!(violation.rule_id, rule_id);
        assert_eq!(violation.violating_component, ComponentId::JITCompiler);
        assert!(violation.is_critical());
    }

    #[test]
    fn test_violation_with_context() {
        let rule_id = RuleId::from_content(b"test_violation_with_context");
        let violation = ConstitutionalViolation::new(
            rule_id,
            ComponentId::JITCompiler,
            "register_rewrite".to_string(),
            Severity::Error,
            "Test violation".to_string(),
        )
        .with_context("test_key".to_string(), "test_value".to_string());

        assert_eq!(violation.context.get("test_key"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_specification_report_creation() {
        let report = SpecificationReport::new();
        assert!(report.violations.is_empty());
        assert!(report.is_compliant());
        assert_eq!(report.compliance_score, 1.0);
    }

    #[test]
    fn test_specification_report_with_violations() {
        let mut report = SpecificationReport::new();
        
        let violation = SpecificationViolation {
            violation_type: ViolationType::SpecificationIncomplete,
            component: ComponentId::D4RegisterAllocator,
            rule_id: Some("TEST_RULE".to_string()),
            description: "Test violation description".to_string(),
            remediation_hint: "Fix the test violation".to_string(),
        };
        
        report.add_violation(violation);
        assert_eq!(report.violations.len(), 1);
        assert!(report.compliance_score < 1.0);
    }

    #[test]
    fn test_contract_violation_report_creation() {
        let mut report = ContractViolationReport::new(ComponentId::D4RegisterAllocator);
        assert_eq!(report.component_context, ComponentId::D4RegisterAllocator);
        assert!(report.findings.is_empty());
        assert!(!report.has_critical_violations());
    }

    #[test]
    fn test_contract_violation_report_with_critical_violation() {
        let mut report = ContractViolationReport::new(ComponentId::D4RegisterAllocator);
        
        let violation = ContractViolation {
            violation_id: "MISSING_STRUCT".to_string(),
            component: ComponentId::D4RegisterAllocator,
            violation_type: ContractViolationType::MissingStructDefinition,
            description: "Missing required struct definition".to_string(),
            formal_specification: "struct RegisterAllocator must be defined".to_string(),
            context: std::collections::BTreeMap::new(),
        };
        
        report.add_violation(violation);
        assert!(report.has_critical_violations());
    }

    #[test]
    fn test_gate_decision_report_creation() {
        let report = GateDecisionReport::new();
        assert_eq!(report.readiness_analysis.readiness_status, ReadinessStatus::NotReady);
        assert!(!report.is_ready_for_transition());
        assert_eq!(report.overall_readiness_score, 0.0);
    }

    #[test]
    fn test_property_test_failure_to_specification_violation() {
        let failure = PropertyTestFailureInfo {
            property_name: "test_allocation_uniqueness".to_string(),
            test_case_id: "case_123".to_string(),
            seed: 42,
            shrunk_input: "input_data".to_string(),
            failure_reason: "Duplicate allocation detected".to_string(),
            ir_fingerprint: "fingerprint_abc".to_string(),
            failure_scenario_id: "scenario_xyz".to_string(),
            stack_trace: None,
            reproduction_command: String::new(),
        };

        let violation = failure.to_specification_violation(ComponentId::D4RegisterAllocator);
        assert_eq!(violation.component, ComponentId::D4RegisterAllocator);
        assert!(violation.description.contains("test_allocation_uniqueness"));
    }

    #[test]
    fn test_property_test_failure_reproduction_command() {
        let failure = PropertyTestFailureInfo {
            property_name: "test_allocation_uniqueness".to_string(),
            test_case_id: "case_123".to_string(),
            seed: 42,
            shrunk_input: "input_data".to_string(),
            failure_reason: "Duplicate allocation detected".to_string(),
            ir_fingerprint: "fingerprint_abc".to_string(),
            failure_scenario_id: "scenario_xyz".to_string(),
            stack_trace: None,
            reproduction_command: String::new(),
        };

        let command = failure.generate_reproduction_command();
        assert!(command.contains("--seed 42"));
        assert!(command.contains("--case-id case_123"));
        assert!(command.contains("--ir-fingerprint fingerprint_abc"));
        assert!(command.contains("--scenario-id scenario_xyz"));
    }

    #[test]
    fn test_report_serialization() {
        let report = SpecificationReport::new();
        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: SpecificationReport = serde_json::from_str(&serialized).unwrap();
        assert_eq!(report.violations.len(), deserialized.violations.len());
        assert_eq!(report.compliance_score, deserialized.compliance_score);
    }
}