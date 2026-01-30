//! Validation Location Analysis for D4 Constitutional Framework (B-MODE)
//!
//! This module implements pure B-MODE validation location analysis that provides
//! immutable location context analysis without stateful tracking or mutations.
//!
//! B-MODE PRINCIPLES:
//! - All operations return reports, never panic or Result<()> for spec violations
//! - Immutable location analysis (&self), no state mutations
//! - Location context analysis, not location tracking
//! - No set_location or mutate operations, only analyze_location_context

use crate::errors::{SpecificationReport, SpecificationViolation, SpecificationFinding, ViolationType, FindingType};
use crate::types::{ComponentId, DeterministicClock, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pure B-MODE validation location analyzer interface
pub trait ValidationLocationAnalyzer {
    /// Analyze current location context (B-MODE)
    fn analyze_current_location(&self, context: &LocationContext) -> LocationAnalysisReport;

    /// Analyze location context accuracy (B-MODE)
    fn analyze_location_context(&self, ctx: &LocationContext) -> SpecificationReport;

    /// Create location context wrapper (B-MODE immutable)
    fn with_location_context(&self, new_location: ValidationLocation) -> Self;

    /// Get current location for analysis
    fn get_current_location(&self) -> ValidationLocation;
}

/// Validation location for constitutional analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationLocation {
    pub component: ComponentId,
    pub validation_phase: ValidationPhase,
    pub method_name: Option<String>,
    pub line_number: Option<u32>,
    pub stack_trace: Vec<String>,
    pub metadata: BTreeMap<String, String>,
}

/// Phases of validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationPhase {
    StructuralValidation,
    SemanticValidation,
    ContractValidation,
    TemplateApplication,
    GateTransitionCheck,
    ComplianceVerification,
}

/// Location context for analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationContext {
    pub current_location: ValidationLocation,
    pub parent_context: Option<Box<LocationContext>>,
    pub analysis_depth: usize,
    pub context_metadata: BTreeMap<String, String>,
}

/// Location analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationAnalysisReport {
    pub location: ValidationLocation,
    pub accuracy_score: f64, // Normalized to 6 decimal places
    pub context_depth: usize,
    pub component_verified: bool,
    pub phase_appropriate: bool,
    pub analysis_findings: Vec<LocationFinding>,
    pub analysis_timestamp: crate::types::LogicalTimestamp,
}

/// Location analysis findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationFinding {
    pub finding_type: LocationFindingType,
    pub description: String,
    pub severity: Severity,
    pub remediation_hint: Option<String>,
}

/// Types of location findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LocationFindingType {
    ComponentAccuracy,
    PhaseConsistency,
    ContextDepth,
    MetadataCompleteness,
    StackTraceValidity,
}

/// Default implementation of validation location analyzer (B-MODE)
#[derive(Debug, Clone)]
pub struct DefaultValidationLocationAnalyzer {
    current_location: ValidationLocation,
}

impl DefaultValidationLocationAnalyzer {
    /// Create a new validation location analyzer
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            current_location: ValidationLocation::new(component_id),
        }
    }

    /// Create analyzer with specific location
    pub fn with_location(location: ValidationLocation) -> Self {
        Self {
            current_location: location,
        }
    }

    /// Normalize floating point values to 6 decimal places for deterministic comparison
    fn normalize_float(value: f64) -> f64 {
        (value * 1_000_000.0_f64).round() / 1_000_000.0
    }

    /// Analyze component accuracy against expected component
    fn analyze_component_accuracy(&self, location: &ValidationLocation) -> LocationFinding {
        // Rule-based component validation instead of string heuristics
        let component_phase_match = match (&location.component, &location.validation_phase) {
            (ComponentId::TemplateSpecRegistry, ValidationPhase::TemplateApplication) => true,
            (ComponentId::ConstitutionalRuleEngine, ValidationPhase::ContractValidation) => true,
            (ComponentId::ConstitutionalRuleEngine, ValidationPhase::ComplianceVerification) => true,
            (ComponentId::DeterminismEngine, ValidationPhase::SemanticValidation) => true,
            (ComponentId::SemanticSpecificationRegistry, ValidationPhase::SemanticValidation) => true,
            (ComponentId::D4RegisterAllocator, _) => true, // D4 can handle any phase
            _ => false, // Other combinations need verification
        };

        if component_phase_match {
            LocationFinding {
                finding_type: LocationFindingType::ComponentAccuracy,
                description: format!("Component {:?} is appropriately matched for {:?} phase", 
                    location.component, location.validation_phase),
                severity: Severity::Info,
                remediation_hint: None,
            }
        } else {
            LocationFinding {
                finding_type: LocationFindingType::ComponentAccuracy,
                description: format!(
                    "Component {:?} may not be the most appropriate for {:?} phase - consider using specialized component",
                    location.component, location.validation_phase
                ),
                severity: Severity::Warning,
                remediation_hint: Some("Use component that specializes in the current validation phase".to_string()),
            }
        }
    }

    /// Analyze validation phase consistency
    fn analyze_phase_consistency(&self, location: &ValidationLocation) -> LocationFinding {
        // Analyze if validation phase is appropriate for the component
        let phase_appropriate = match location.component {
            ComponentId::ConstitutionalRuleEngine => {
                matches!(location.validation_phase, 
                    ValidationPhase::ContractValidation | 
                    ValidationPhase::ComplianceVerification
                )
            }
            ComponentId::TemplateSpecRegistry => {
                matches!(location.validation_phase, ValidationPhase::TemplateApplication)
            }
            ComponentId::DeterminismEngine => {
                matches!(location.validation_phase, 
                    ValidationPhase::SemanticValidation | 
                    ValidationPhase::ComplianceVerification
                )
            }
            _ => true, // Other components can use any phase
        };

        if phase_appropriate {
            LocationFinding {
                finding_type: LocationFindingType::PhaseConsistency,
                description: format!("Validation phase {:?} is appropriate for component {:?}", 
                    location.validation_phase, location.component),
                severity: Severity::Info,
                remediation_hint: None,
            }
        } else {
            LocationFinding {
                finding_type: LocationFindingType::PhaseConsistency,
                description: format!("Validation phase {:?} may not be appropriate for component {:?}", 
                    location.validation_phase, location.component),
                severity: Severity::Warning,
                remediation_hint: Some("Use appropriate validation phase for component type".to_string()),
            }
        }
    }

    /// Analyze context depth
    fn analyze_context_depth(&self, context: &LocationContext) -> LocationFinding {
        if context.analysis_depth > 10 {
            LocationFinding {
                finding_type: LocationFindingType::ContextDepth,
                description: format!("Analysis depth {} is very deep", context.analysis_depth),
                severity: Severity::Warning,
                remediation_hint: Some("Consider flattening analysis context to avoid deep nesting".to_string()),
            }
        } else if context.analysis_depth == 0 {
            LocationFinding {
                finding_type: LocationFindingType::ContextDepth,
                description: "Analysis depth is zero - may indicate missing context".to_string(),
                severity: Severity::Warning,
                remediation_hint: Some("Ensure analysis context depth is properly tracked".to_string()),
            }
        } else {
            LocationFinding {
                finding_type: LocationFindingType::ContextDepth,
                description: format!("Analysis depth {} is appropriate", context.analysis_depth),
                severity: Severity::Info,
                remediation_hint: None,
            }
        }
    }

    /// Analyze metadata completeness
    fn analyze_metadata_completeness(&self, location: &ValidationLocation) -> LocationFinding {
        if location.metadata.is_empty() {
            LocationFinding {
                finding_type: LocationFindingType::MetadataCompleteness,
                description: "Location metadata is empty - may lack context information".to_string(),
                severity: Severity::Info,
                remediation_hint: Some("Consider adding relevant metadata for better context tracking".to_string()),
            }
        } else {
            LocationFinding {
                finding_type: LocationFindingType::MetadataCompleteness,
                description: format!("Location has {} metadata entries", location.metadata.len()),
                severity: Severity::Info,
                remediation_hint: None,
            }
        }
    }

    /// Analyze stack trace validity
    fn analyze_stack_trace_validity(&self, location: &ValidationLocation) -> LocationFinding {
        if location.stack_trace.is_empty() {
            LocationFinding {
                finding_type: LocationFindingType::StackTraceValidity,
                description: "Stack trace is empty - may lack execution context".to_string(),
                severity: Severity::Info,
                remediation_hint: Some("Consider adding stack trace information for better debugging".to_string()),
            }
        } else if location.stack_trace.len() > 50 {
            LocationFinding {
                finding_type: LocationFindingType::StackTraceValidity,
                description: format!("Stack trace has {} entries - may be too verbose", location.stack_trace.len()),
                severity: Severity::Warning,
                remediation_hint: Some("Consider limiting stack trace depth for performance".to_string()),
            }
        } else {
            LocationFinding {
                finding_type: LocationFindingType::StackTraceValidity,
                description: format!("Stack trace has {} entries", location.stack_trace.len()),
                severity: Severity::Info,
                remediation_hint: None,
            }
        }
    }
}

impl ValidationLocationAnalyzer for DefaultValidationLocationAnalyzer {
    fn analyze_current_location(&self, context: &LocationContext) -> LocationAnalysisReport {
        let mut analysis_findings = Vec::new();

        // Analyze component accuracy
        let component_finding = self.analyze_component_accuracy(&context.current_location);
        let component_verified = matches!(component_finding.severity, Severity::Info);
        analysis_findings.push(component_finding);

        // Analyze phase consistency
        let phase_finding = self.analyze_phase_consistency(&context.current_location);
        let phase_appropriate = matches!(phase_finding.severity, Severity::Info);
        analysis_findings.push(phase_finding);

        // Analyze context depth
        let depth_finding = self.analyze_context_depth(context);
        analysis_findings.push(depth_finding);

        // Analyze metadata completeness
        let metadata_finding = self.analyze_metadata_completeness(&context.current_location);
        analysis_findings.push(metadata_finding);

        // Analyze stack trace validity
        let stack_finding = self.analyze_stack_trace_validity(&context.current_location);
        analysis_findings.push(stack_finding);

        // Calculate accuracy score based on findings
        let info_findings = analysis_findings.iter().filter(|f| matches!(f.severity, Severity::Info)).count();
        let total_findings = analysis_findings.len();
        let accuracy_score = if total_findings > 0 {
            info_findings as f64 / total_findings as f64
        } else {
            1.0
        };

        LocationAnalysisReport {
            location: context.current_location.clone(),
            accuracy_score: Self::normalize_float(accuracy_score),
            context_depth: context.analysis_depth,
            component_verified,
            phase_appropriate,
            analysis_findings,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }

    fn analyze_location_context(&self, ctx: &LocationContext) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze location context for B-MODE compliance
        let analysis_report = self.analyze_current_location(ctx);

        // Convert location findings to specification findings/violations
        for finding in &analysis_report.analysis_findings {
            match finding.severity {
                Severity::Error => {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationIncomplete,
                        component: ctx.current_location.component,
                        rule_id: Some(format!("LOCATION_{:?}", finding.finding_type)),
                        description: finding.description.clone(),
                        remediation_hint: finding.remediation_hint.clone().unwrap_or_default(),
                    });
                }
                Severity::Critical => {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationIncomplete,
                        component: ctx.current_location.component,
                        rule_id: Some(format!("LOCATION_CRITICAL_{:?}", finding.finding_type)),
                        description: finding.description.clone(),
                        remediation_hint: finding.remediation_hint.clone().unwrap_or_default(),
                    });
                }
                Severity::Warning => {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::BoundaryViolation,
                        component: ctx.current_location.component,
                        description: finding.description.clone(),
                        severity: finding.severity,
                        location: ValidationLocation::new(ctx.current_location.component),
                    });
                }
                Severity::Info => {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::SpecificationCompliance,
                        component: ctx.current_location.component,
                        description: finding.description.clone(),
                        severity: finding.severity,
                        location: ValidationLocation::new(ctx.current_location.component),
                    });
                }
            }
        }

        // Add overall accuracy assessment
        if analysis_report.accuracy_score >= 0.8 {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ctx.current_location.component,
                description: format!("Location context accuracy score: {:.6}", analysis_report.accuracy_score),
                severity: Severity::Info,
                location: ValidationLocation::new(ctx.current_location.component),
            });
        } else {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ctx.current_location.component,
                rule_id: Some("LOCATION_ACCURACY_LOW".to_string()),
                description: format!("Location context accuracy score too low: {:.6}", analysis_report.accuracy_score),
                remediation_hint: "Improve location context accuracy by addressing identified issues".to_string(),
            });
        }

        report
    }

    fn with_location_context(&self, new_location: ValidationLocation) -> Self {
        // In B-MODE, we create a new analyzer instance with updated location
        // This maintains immutability while providing location updates
        Self {
            current_location: new_location,
        }
    }

    fn get_current_location(&self) -> ValidationLocation {
        self.current_location.clone()
    }
}

impl ValidationLocation {
    /// Create a new validation location
    pub fn new(component: ComponentId) -> Self {
        Self {
            component,
            validation_phase: ValidationPhase::StructuralValidation,
            method_name: None,
            line_number: None,
            stack_trace: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Create location with validation phase
    pub fn with_validation_phase(mut self, phase: ValidationPhase) -> Self {
        self.validation_phase = phase;
        self
    }

    /// Create location with method name
    pub fn with_method(mut self, method: String) -> Self {
        self.method_name = Some(method);
        self
    }

    /// Create location with line number
    pub fn with_line(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Create location with stack trace entry
    pub fn with_stack_entry(mut self, entry: String) -> Self {
        self.stack_trace.push(entry);
        self
    }

    /// Create location with metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Create location context from this location
    pub fn create_context(&self) -> LocationContext {
        LocationContext {
            current_location: self.clone(),
            parent_context: None,
            analysis_depth: 1,
            context_metadata: BTreeMap::new(),
        }
    }

    /// Create nested location context
    pub fn create_nested_context(&self, parent: LocationContext) -> LocationContext {
        LocationContext {
            current_location: self.clone(),
            parent_context: Some(Box::new(parent.clone())),
            analysis_depth: parent.analysis_depth + 1,
            context_metadata: BTreeMap::new(),
        }
    }
}

impl LocationContext {
    /// Create a new location context
    pub fn new(location: ValidationLocation) -> Self {
        Self {
            current_location: location,
            parent_context: None,
            analysis_depth: 1,
            context_metadata: BTreeMap::new(),
        }
    }

    /// Create context with parent
    pub fn with_parent(mut self, parent: LocationContext) -> Self {
        self.analysis_depth = parent.analysis_depth + 1;
        self.parent_context = Some(Box::new(parent));
        self
    }

    /// Create context with metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.context_metadata.insert(key, value);
        self
    }

    /// Get the root context (top-level parent)
    pub fn get_root_context(&self) -> &LocationContext {
        if let Some(ref parent) = self.parent_context {
            parent.get_root_context()
        } else {
            self
        }
    }

    /// Get context chain depth
    pub fn get_chain_depth(&self) -> usize {
        if let Some(ref parent) = self.parent_context {
            1 + parent.get_chain_depth()
        } else {
            1
        }
    }
}

/// Helper function to create a validation location
pub fn create_validation_location(
    component: ComponentId,
    phase: ValidationPhase,
) -> ValidationLocation {
    ValidationLocation::new(component).with_validation_phase(phase)
}

/// Helper function to create a location context
pub fn create_location_context(
    component: ComponentId,
    phase: ValidationPhase,
) -> LocationContext {
    LocationContext::new(create_validation_location(component, phase))
}