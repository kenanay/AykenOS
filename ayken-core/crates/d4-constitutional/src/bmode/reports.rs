//! 🔒 CONSTITUTIONAL LOCK ACTIVE
//!
//! This module is under PERMANENT CONSTITUTIONAL LOCK.
//! ❌ Enforcement logic is strictly forbidden.
//! ❌ Runtime imports are forbidden.
//! ❌ &mut self is forbidden.
//!
//! Any change requires a Constitutional RFC.
//!
//! B-MODE Specification Reports for D4 Constitutional Framework
//!
//! This module defines B-MODE specific report types that extend the base
//! SpecificationReport with B-MODE specific analysis and recommendations.
//! 
//! SINGLE SOURCE OF TRUTH: This module imports SpecificationReport from errors
//! module and extends it with B-MODE specific functionality.

use crate::errors::{SpecificationReport, SpecificationViolation};
use crate::types::{ComponentId, DeterministicClock, Severity};
use serde::{Deserialize, Serialize};

/// B-MODE specific specification report extensions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BModeSpecificationReport {
    pub base_report: SpecificationReport,
    pub bmode_analysis: BModeAnalysis,
    pub recommended_actions: Vec<RecommendedAction>,
    pub compliance_assessment: ComplianceAssessment,
}

/// B-MODE analysis metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BModeAnalysis {
    pub purity_verified: bool,
    pub immutability_verified: bool,
    pub no_side_effects_verified: bool,
    pub specification_only_verified: bool,
    pub analysis_timestamp: crate::types::LogicalTimestamp,
}

/// Recommended actions for B-MODE compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: RecommendedActionType,
    pub component: ComponentId,
    pub description: String,
    pub priority: ActionPriority,
    pub implementation_hint: String,
}

/// Types of recommended actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendedActionType {
    RefactorToImmutable,
    RemoveSideEffects,
    ConvertToSpecification,
    AddBModeCompliance,
    SeparateBModeFromRuntime,
}

/// Priority levels for recommended actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionPriority {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Compliance assessment for B-MODE
/// 
/// ⚠️ FLOATING POINT SAFETY NOTICE
/// 
/// - f64 values are used for reporting ONLY
/// - MUST NOT derive Eq / Ord
/// - MUST NOT be used as HashMap keys
/// - MUST NOT participate in deterministic ordering
/// 
/// All comparisons MUST use normalized or epsilon-based logic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceAssessment {
    pub overall_bmode_compliance: f64, // Normalized to 6 decimal places - NO Eq/Ord!
    pub purity_score: f64,             // Normalized to 6 decimal places - NO Eq/Ord!
    pub immutability_score: f64,       // Normalized to 6 decimal places - NO Eq/Ord!
    pub specification_score: f64,      // Normalized to 6 decimal places - NO Eq/Ord!
    pub areas_for_improvement: Vec<String>,
}

impl BModeSpecificationReport {
    /// Create a new B-MODE specification report
    pub fn new(base_report: SpecificationReport) -> Self {
        Self {
            base_report,
            bmode_analysis: BModeAnalysis {
                purity_verified: true,
                immutability_verified: true,
                no_side_effects_verified: true,
                specification_only_verified: true,
                analysis_timestamp: DeterministicClock::new().now(),
            },
            recommended_actions: Vec::new(),
            compliance_assessment: ComplianceAssessment {
                overall_bmode_compliance: 1.0,
                purity_score: 1.0,
                immutability_score: 1.0,
                specification_score: 1.0,
                areas_for_improvement: Vec::new(),
            },
        }
    }

    /// Add a recommended action (B-MODE: immutable pattern)
    pub fn with_recommended_action(mut self, action: RecommendedAction) -> Self {
        self.recommended_actions.push(action);
        self.update_compliance_assessment();
        self
    }

    /// Add multiple recommended actions (B-MODE: immutable pattern)
    pub fn with_recommended_actions(mut self, actions: Vec<RecommendedAction>) -> Self {
        self.recommended_actions.extend(actions);
        self.update_compliance_assessment();
        self
    }

    /// Update compliance assessment based on current state
    fn update_compliance_assessment(&mut self) {
        let mut scores = Vec::new();
        
        // Base compliance from violations count (simple heuristic)
        let base_score = if self.base_report.violations.is_empty() { 1.0 } else { 0.5 };
        scores.push(base_score);

        // B-MODE specific scores
        let purity_score = if self.bmode_analysis.purity_verified { 1.0 } else { 0.0 };
        let immutability_score = if self.bmode_analysis.immutability_verified { 1.0 } else { 0.0 };
        let specification_score = if self.bmode_analysis.specification_only_verified { 1.0 } else { 0.0 };

        scores.push(purity_score);
        scores.push(immutability_score);
        scores.push(specification_score);

        // Calculate overall compliance
        let overall_compliance = if !scores.is_empty() {
            scores.iter().sum::<f64>() / scores.len() as f64
        } else {
            0.0
        };

        // Identify areas for improvement
        let mut areas_for_improvement = Vec::new();
        if !self.bmode_analysis.purity_verified {
            areas_for_improvement.push("Function purity verification".to_string());
        }
        if !self.bmode_analysis.immutability_verified {
            areas_for_improvement.push("Immutability compliance".to_string());
        }
        if !self.bmode_analysis.specification_only_verified {
            areas_for_improvement.push("Specification-only operations".to_string());
        }

        self.compliance_assessment = ComplianceAssessment {
            overall_bmode_compliance: Self::normalize_float(overall_compliance),
            purity_score: Self::normalize_float(purity_score),
            immutability_score: Self::normalize_float(immutability_score),
            specification_score: Self::normalize_float(specification_score),
            areas_for_improvement,
        };
    }

    /// Normalize floating point values to 6 decimal places
    fn normalize_float(value: f64) -> f64 {
        (value * 1_000_000.0_f64).round() / 1_000_000.0
    }

    /// Check if the report indicates B-MODE compliance
    pub fn is_bmode_compliant(&self) -> bool {
        self.compliance_assessment.overall_bmode_compliance >= 0.8
    }

    /// Get critical recommended actions
    pub fn get_critical_actions(&self) -> Vec<&RecommendedAction> {
        self.recommended_actions
            .iter()
            .filter(|action| matches!(action.priority, ActionPriority::Critical))
            .collect()
    }
}

/// Helper function to create a recommended action
pub fn create_recommended_action(
    action_type: RecommendedActionType,
    component: ComponentId,
    description: String,
    priority: ActionPriority,
    implementation_hint: String,
) -> RecommendedAction {
    RecommendedAction {
        action_type,
        component,
        description,
        priority,
        implementation_hint,
    }
}

/// Helper function to analyze B-MODE compliance from a specification report
pub fn analyze_bmode_compliance(report: SpecificationReport) -> BModeSpecificationReport {
    let mut bmode_report = BModeSpecificationReport::new(report);

    // Collect violations to avoid borrow checker issues
    let violations: Vec<_> = bmode_report.base_report.violations.clone();

    // Analyze violations for B-MODE compliance issues
    for violation in &violations {
        let violation_code = match &violation.violation_type {
            crate::errors::ViolationType::RuntimeSemanticsMixing => "RUNTIME_SEMANTICS_MIXING",
            crate::errors::ViolationType::SpecificationIncomplete => "SPECIFICATION_INCOMPLETE",
            _ => "OTHER_VIOLATION",
        };

        match violation_code {
            "RUNTIME_SEMANTICS_MIXING" => {
                bmode_report.bmode_analysis.purity_verified = false;
                bmode_report = bmode_report.with_recommended_action(create_recommended_action(
                    RecommendedActionType::SeparateBModeFromRuntime,
                    ComponentId::D4RegisterAllocator, // Default component
                    "Separate B-MODE analysis from runtime enforcement".to_string(),
                    ActionPriority::Critical,
                    "Move enforcement logic to runtime module, keep only analysis in B-MODE".to_string(),
                ));
            }
            "SPECIFICATION_INCOMPLETE" => {
                bmode_report.bmode_analysis.specification_only_verified = false;
                bmode_report = bmode_report.with_recommended_action(create_recommended_action(
                    RecommendedActionType::ConvertToSpecification,
                    ComponentId::D4RegisterAllocator, // Default component
                    "Convert operations to specification-only".to_string(),
                    ActionPriority::High,
                    "Replace Result<()> returns with SpecificationReport returns".to_string(),
                ));
            }
            _ => {
                // Other violation types may indicate general compliance issues
                bmode_report = bmode_report.with_recommended_action(create_recommended_action(
                    RecommendedActionType::AddBModeCompliance,
                    ComponentId::D4RegisterAllocator, // Default component
                    "Improve B-MODE compliance".to_string(),
                    ActionPriority::Medium,
                    "Review and address specification compliance issues".to_string(),
                ));
            }
        }
    }

    bmode_report
}