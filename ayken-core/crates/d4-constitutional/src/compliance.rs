//! Validation Compliance Analysis - PURE B-MODE
//!
//! This module provides mathematically correct compliance analysis for validation operations.
//! It NEVER executes or enforces - only analyzes and reports compliance.
//!
//! PURE B-MODE PRINCIPLES:
//! - Analysis only, no execution
//! - Reports only, no errors for spec violations
//! - Immutable analysis, no state mutations
//! - Compliance specifications, not runtime enforcement

use crate::types::{ComponentId, Severity, LogicalTimestamp, DeterministicClock};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Input data for validation compliance analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationInput {
    pub component_id: ComponentId,
    pub validation_rules: Vec<ValidationRule>,
    pub penalties: Vec<Penalty>,
    pub bonuses: Vec<Bonus>,
    pub metadata: BTreeMap<String, String>,
}

/// A validation rule that can be checked for compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_id: String,
    pub description: String,
    pub severity: Severity,
    pub weight: f64,
}

/// A penalty applied to compliance calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Penalty {
    pub penalty_id: String,
    pub description: String,
    pub severity: Severity,
    pub impact: f64, // Normalized to 0.0-1.0 range
    pub source_rule: Option<String>,
}

/// A bonus applied to compliance calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bonus {
    pub bonus_id: String,
    pub description: String,
    pub impact: f64, // Normalized to 0.0-1.0 range
    pub source_rule: Option<String>,
}

/// Analysis metadata for compliance calculations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analysis_timestamp: LogicalTimestamp,
    pub component_context: ComponentId,
    pub total_rules_analyzed: usize,
    pub calculation_method: String,
}

/// Comprehensive compliance analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceAnalysisReport {
    /// Base compliance before penalties/bonuses (normalized to 6 decimal places)
    pub base_compliance: f64,
    /// All penalties applied (each counted exactly once)
    pub penalties: Vec<Penalty>,
    /// All bonuses applied (each counted exactly once)
    pub bonuses: Vec<Bonus>,
    /// Final compliance index (normalized to 6 decimal places)
    /// Higher values indicate better specification compliance
    pub compliance_index: f64,
    /// Analysis metadata and audit trail
    pub analysis_metadata: AnalysisMetadata,
}

/// Trait for analyzing validation compliance in B-MODE
pub trait ComplianceAnalyzer {
    /// Analyze compliance for given validation input
    /// Returns comprehensive compliance analysis report
    fn analyze_compliance(validation_input: &ValidationInput) -> ComplianceAnalysisReport;
    
    /// Analyze penalty impact on compliance
    /// Returns detailed penalty impact analysis
    fn analyze_penalty_impact(penalties: &[Penalty]) -> PenaltyImpactReport;
    
    /// Analyze bonus impact on compliance
    /// Returns detailed bonus impact analysis
    fn analyze_bonus_impact(bonuses: &[Bonus]) -> BonusImpactReport;
    
    /// Generate comprehensive compliance report
    /// Returns complete compliance analysis with all components
    fn generate_compliance_report(
        base_compliance: f64,
        penalties: &[Penalty],
        bonuses: &[Bonus],
        metadata: AnalysisMetadata,
    ) -> ComplianceAnalysisReport;
}

/// Report analyzing penalty impact on compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PenaltyImpactReport {
    pub total_penalty_impact: f64, // Normalized to 6 decimal places
    pub penalty_breakdown: Vec<PenaltyBreakdown>,
    pub severity_distribution: BTreeMap<Severity, f64>,
}

/// Report analyzing bonus impact on compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BonusImpactReport {
    pub total_bonus_impact: f64, // Normalized to 6 decimal places
    pub bonus_breakdown: Vec<BonusBreakdown>,
    pub impact_distribution: BTreeMap<String, f64>,
}

/// Detailed breakdown of individual penalty impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PenaltyBreakdown {
    pub penalty_id: String,
    pub normalized_impact: f64, // Normalized to 6 decimal places
    pub severity_weight: f64,
    pub contribution_percentage: f64,
}

/// Detailed breakdown of individual bonus impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BonusBreakdown {
    pub bonus_id: String,
    pub normalized_impact: f64, // Normalized to 6 decimal places
    pub contribution_percentage: f64,
}

/// Default implementation of ComplianceAnalyzer
pub struct ValidationComplianceAnalyzer;

impl ComplianceAnalyzer for ValidationComplianceAnalyzer {
    fn analyze_compliance(validation_input: &ValidationInput) -> ComplianceAnalysisReport {
        // Calculate base compliance from validation rules
        let base_compliance = Self::calculate_base_compliance(&validation_input.validation_rules);
        
        // Analyze penalty and bonus impacts
        let penalty_report = Self::analyze_penalty_impact(&validation_input.penalties);
        let bonus_report = Self::analyze_bonus_impact(&validation_input.bonuses);
        
        // Calculate final compliance index with proper mathematical handling
        let compliance_index = Self::calculate_final_compliance_index(
            base_compliance,
            penalty_report.total_penalty_impact,
            bonus_report.total_bonus_impact,
        );
        
        // Create analysis metadata
        let metadata = AnalysisMetadata {
            analysis_timestamp: DeterministicClock::new().now(),
            component_context: validation_input.component_id.clone(),
            total_rules_analyzed: validation_input.validation_rules.len(),
            calculation_method: "mathematical_compliance_v1".to_string(),
        };
        
        Self::generate_compliance_report(
            base_compliance,
            &validation_input.penalties,
            &validation_input.bonuses,
            metadata,
        )
    }
    
    fn analyze_penalty_impact(penalties: &[Penalty]) -> PenaltyImpactReport {
        if penalties.is_empty() {
            return PenaltyImpactReport {
                total_penalty_impact: Self::normalize_float(0.0),
                penalty_breakdown: Vec::new(),
                severity_distribution: BTreeMap::new(),
            };
        }
        
        // Calculate total penalty impact without double-counting
        let mut penalty_breakdown = Vec::new();
        let mut severity_totals: BTreeMap<Severity, f64> = BTreeMap::new();
        let mut total_impact = 0.0;
        
        for penalty in penalties {
            // Each penalty counted exactly once
            let severity_weight = Self::get_severity_weight(&penalty.severity);
            let normalized_impact = Self::normalize_float(penalty.impact * severity_weight);
            
            total_impact += normalized_impact;
            
            // Track severity distribution
            *severity_totals.entry(penalty.severity.clone()).or_insert(0.0) += normalized_impact;
            
            penalty_breakdown.push(PenaltyBreakdown {
                penalty_id: penalty.penalty_id.clone(),
                normalized_impact,
                severity_weight,
                contribution_percentage: 0.0, // Will be calculated after total is known
            });
        }
        
        // Normalize total impact and calculate contribution percentages
        let normalized_total = Self::normalize_float(total_impact);
        
        // Update contribution percentages
        for breakdown in &mut penalty_breakdown {
            breakdown.contribution_percentage = if normalized_total > 0.0 {
                Self::normalize_float((breakdown.normalized_impact / normalized_total) * 100.0)
            } else {
                0.0
            };
        }
        
        PenaltyImpactReport {
            total_penalty_impact: normalized_total,
            penalty_breakdown,
            severity_distribution: severity_totals.into_iter()
                .map(|(k, v)| (k, Self::normalize_float(v)))
                .collect(),
        }
    }
    
    fn analyze_bonus_impact(bonuses: &[Bonus]) -> BonusImpactReport {
        if bonuses.is_empty() {
            return BonusImpactReport {
                total_bonus_impact: Self::normalize_float(0.0),
                bonus_breakdown: Vec::new(),
                impact_distribution: BTreeMap::new(),
            };
        }
        
        // Calculate total bonus impact without double-counting
        let mut bonus_breakdown = Vec::new();
        let mut impact_distribution: BTreeMap<String, f64> = BTreeMap::new();
        let mut total_impact = 0.0;
        
        for bonus in bonuses {
            // Each bonus counted exactly once
            let normalized_impact = Self::normalize_float(bonus.impact);
            total_impact += normalized_impact;
            
            // Track impact distribution by bonus type
            let bonus_type = bonus.source_rule.as_deref().unwrap_or("unspecified").to_string();
            *impact_distribution.entry(bonus_type).or_insert(0.0) += normalized_impact;
            
            bonus_breakdown.push(BonusBreakdown {
                bonus_id: bonus.bonus_id.clone(),
                normalized_impact,
                contribution_percentage: 0.0, // Will be calculated after total is known
            });
        }
        
        // Normalize total impact and calculate contribution percentages
        let normalized_total = Self::normalize_float(total_impact);
        
        // Update contribution percentages
        for breakdown in &mut bonus_breakdown {
            breakdown.contribution_percentage = if normalized_total > 0.0 {
                Self::normalize_float((breakdown.normalized_impact / normalized_total) * 100.0)
            } else {
                0.0
            };
        }
        
        BonusImpactReport {
            total_bonus_impact: normalized_total,
            bonus_breakdown,
            impact_distribution: impact_distribution.into_iter()
                .map(|(k, v)| (k, Self::normalize_float(v)))
                .collect(),
        }
    }
    
    fn generate_compliance_report(
        base_compliance: f64,
        penalties: &[Penalty],
        bonuses: &[Bonus],
        metadata: AnalysisMetadata,
    ) -> ComplianceAnalysisReport {
        let penalty_report = Self::analyze_penalty_impact(penalties);
        let bonus_report = Self::analyze_bonus_impact(bonuses);
        
        let compliance_index = Self::calculate_final_compliance_index(
            base_compliance,
            penalty_report.total_penalty_impact,
            bonus_report.total_bonus_impact,
        );
        
        ComplianceAnalysisReport {
            base_compliance: Self::normalize_float(base_compliance),
            penalties: penalties.to_vec(),
            bonuses: bonuses.to_vec(),
            compliance_index,
            analysis_metadata: metadata,
        }
    }
}

impl ValidationComplianceAnalyzer {
    /// Calculate base compliance from validation rules
    fn calculate_base_compliance(rules: &[ValidationRule]) -> f64 {
        if rules.is_empty() {
            return 1.0; // Perfect compliance when no rules to violate
        }
        
        // Base compliance starts at 1.0 (perfect) and is reduced by rule violations
        // This is a simplified calculation - in practice, this would be based on
        // actual rule evaluation results
        let total_weight: f64 = rules.iter().map(|r| r.weight).sum();
        if total_weight <= 0.0 {
            return 1.0;
        }
        
        // For this implementation, assume all rules pass (base case)
        // In practice, this would be calculated from actual validation results
        1.0
    }
    
    /// Calculate final compliance index with proper mathematical handling
    fn calculate_final_compliance_index(
        base_compliance: f64,
        penalty_impact: f64,
        bonus_impact: f64,
    ) -> f64 {
        // Ensure inputs are in valid ranges
        let base = base_compliance.clamp(0.0, 1.0);
        let penalties = penalty_impact.max(0.0);
        let bonuses = bonus_impact.max(0.0);
        
        // Apply penalties first (multiplicative to avoid going negative)
        let after_penalties = base * (1.0 - penalties.min(1.0));
        
        // Apply bonuses (additive but capped at 1.0)
        let final_compliance = (after_penalties + bonuses).min(1.0);
        
        Self::normalize_float(final_compliance)
    }
    
    /// Get severity weight for penalty calculations
    fn get_severity_weight(severity: &Severity) -> f64 {
        match severity {
            Severity::Critical => 1.0,
            Severity::Error => 0.8,
            Severity::Warning => 0.5,
            Severity::Info => 0.2,
        }
    }
    
    /// Normalize floating point values to 6 decimal places for deterministic comparison
    fn normalize_float(value: f64) -> f64 {
        (value * 1_000_000.0).round() / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComponentId;
    
    #[test]
    fn test_empty_validation_input() {
        let input = ValidationInput {
            component_id: ComponentId::D4RegisterAllocator,
            validation_rules: Vec::new(),
            penalties: Vec::new(),
            bonuses: Vec::new(),
            metadata: BTreeMap::new(),
        };
        
        let report = ValidationComplianceAnalyzer::analyze_compliance(&input);
        
        assert_eq!(report.base_compliance, 1.0);
        assert_eq!(report.compliance_index, 1.0);
        assert!(report.penalties.is_empty());
        assert!(report.bonuses.is_empty());
    }
    
    #[test]
    fn test_penalty_impact_calculation() {
        let penalties = vec![
            Penalty {
                penalty_id: "P1".to_string(),
                description: "Critical issue".to_string(),
                severity: Severity::Critical,
                impact: 0.5,
                source_rule: Some("R1".to_string()),
            },
            Penalty {
                penalty_id: "P2".to_string(),
                description: "Warning issue".to_string(),
                severity: Severity::Warning,
                impact: 0.3,
                source_rule: Some("R2".to_string()),
            },
        ];
        
        let report = ValidationComplianceAnalyzer::analyze_penalty_impact(&penalties);
        
        // Critical: 0.5 * 1.0 = 0.5
        // Warning: 0.3 * 0.5 = 0.15
        // Total: 0.65
        assert_eq!(report.total_penalty_impact, 0.65);
        assert_eq!(report.penalty_breakdown.len(), 2);
    }
    
    #[test]
    fn test_bonus_impact_calculation() {
        let bonuses = vec![
            Bonus {
                bonus_id: "B1".to_string(),
                description: "Good practice".to_string(),
                impact: 0.1,
                source_rule: Some("R1".to_string()),
            },
            Bonus {
                bonus_id: "B2".to_string(),
                description: "Excellent implementation".to_string(),
                impact: 0.05,
                source_rule: Some("R2".to_string()),
            },
        ];
        
        let report = ValidationComplianceAnalyzer::analyze_bonus_impact(&bonuses);
        
        assert_eq!(report.total_bonus_impact, 0.15);
        assert_eq!(report.bonus_breakdown.len(), 2);
    }
    
    #[test]
    fn test_final_compliance_calculation() {
        let base = 0.9;
        let penalties = 0.2;
        let bonuses = 0.1;
        
        let result = ValidationComplianceAnalyzer::calculate_final_compliance_index(
            base, penalties, bonuses
        );
        
        // 0.9 * (1.0 - 0.2) + 0.1 = 0.9 * 0.8 + 0.1 = 0.72 + 0.1 = 0.82
        assert_eq!(result, 0.82);
    }
    
    #[test]
    fn test_edge_case_extreme_penalties() {
        let result = ValidationComplianceAnalyzer::calculate_final_compliance_index(
            1.0, 2.0, 0.0 // Penalty > 1.0 should be clamped
        );
        
        // 1.0 * (1.0 - 1.0) + 0.0 = 0.0
        assert_eq!(result, 0.0);
    }
    
    #[test]
    fn test_edge_case_extreme_bonuses() {
        let result = ValidationComplianceAnalyzer::calculate_final_compliance_index(
            0.5, 0.0, 2.0 // Large bonus should be capped at 1.0
        );
        
        // (0.5 + 2.0).min(1.0) = 1.0
        assert_eq!(result, 1.0);
    }
    
    #[test]
    fn test_float_normalization() {
        let value = 0.123456789;
        let normalized = ValidationComplianceAnalyzer::normalize_float(value);
        assert_eq!(normalized, 0.123457); // Rounded to 6 decimal places
    }
}