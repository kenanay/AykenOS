//! B-MODE Specification Reports for D4 Constitutional Framework
//!
//! This module defines B-MODE specific report types that extend the base
//! SpecificationReport with B-MODE specific analysis and recommendations.
//! 
//! MERGED FROM ROOT: Also contains the pure B-MODE output types that replace
//! Result<()> returns for all specification operations.

use crate::types::{ComponentId, DeterministicClock, SpecLocation, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Single truth output for all B-MODE operations (MERGED FROM ROOT)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecificationReport {
    pub compliant: bool,
    pub violations: Vec<SpecViolation>,
    pub notes: Vec<String>,
}

impl SpecificationReport {
    /// Create a new compliant report
    pub fn compliant() -> Self {
        Self {
            compliant: true,
            violations: vec![],
            notes: vec![],
        }
    }
    
    /// Create a new non-compliant report with violations
    pub fn non_compliant(violations: Vec<SpecViolation>) -> Self {
        Self {
            compliant: violations.is_empty(),
            violations,
            notes: vec![],
        }
    }
    
    /// Add a note to the report
    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }
    
    /// Add multiple notes to the report
    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes.extend(notes);
        self
    }
}

/// Specification violation (not runtime error) (MERGED FROM ROOT)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecViolation {
    pub code: String,
    pub severity: Severity,
    pub location: SpecLocation,
    pub message: String,
}

impl SpecViolation {
    /// Create a new specification violation
    pub fn new(code: &str, severity: Severity, location: SpecLocation, message: &str) -> Self {
        Self {
            code: code.to_string(),
            severity,
            location,
            message: message.to_string(),
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceAssessment {
    pub overall_bmode_compliance: f64, // Normalized to 6 decimal places
    pub purity_score: f64,
    pub immutability_score: f64,
    pub specification_score: f64,
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

    /// Add a recommended action
    pub fn add_recommended_action(&mut self, action: RecommendedAction) {
        self.recommended_actions.push(action);
        self.update_compliance_assessment();
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
        match violation.code.as_str() {
            "RUNTIME_SEMANTICS_MIXING" => {
                bmode_report.bmode_analysis.purity_verified = false;
                bmode_report.add_recommended_action(create_recommended_action(
                    RecommendedActionType::SeparateBModeFromRuntime,
                    ComponentId::D4RegisterAllocator, // Default component
                    "Separate B-MODE analysis from runtime enforcement".to_string(),
                    ActionPriority::Critical,
                    "Move enforcement logic to runtime module, keep only analysis in B-MODE".to_string(),
                ));
            }
            "SPECIFICATION_INCOMPLETE" => {
                bmode_report.bmode_analysis.specification_only_verified = false;
                bmode_report.add_recommended_action(create_recommended_action(
                    RecommendedActionType::ConvertToSpecification,
                    ComponentId::D4RegisterAllocator, // Default component
                    "Convert operations to specification-only".to_string(),
                    ActionPriority::High,
                    "Replace Result<()> returns with SpecificationReport returns".to_string(),
                ));
            }
            _ => {
                // Other violation types may indicate general compliance issues
                bmode_report.add_recommended_action(create_recommended_action(
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