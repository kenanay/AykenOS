//! Template Specification System for D4 Constitutional Framework (B-MODE)
//!
//! This module implements pure B-MODE template specification and analysis system that provides
//! immutable template catalog and analysis capabilities without stateful registries or caches.
//!
//! B-MODE PRINCIPLES:
//! - Immutable template catalog (&'static or immutable references)
//! - All operations return SpecificationReport, never Result<()> for spec violations
//! - No registration/mutation operations, only specification and analysis
//! - Template availability analysis, not template management

use crate::errors::{SpecificationReport, SpecificationViolation, SpecificationFinding, ViolationType, FindingType};
use crate::types::{ComponentId, DeterministicClock, Severity};
use crate::bmode::validation_location::ValidationLocation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pure B-MODE template specification analyzer interface
pub trait TemplateSpecAnalyzer {
    /// Analyze template completeness for a given template type (B-MODE)
    fn analyze_template_completeness(&self, template_type: TemplateType) -> SpecificationReport;

    /// Analyze template availability in the catalog (B-MODE)
    fn analyze_template_availability(&self, template_type: TemplateType) -> TemplateAvailabilityReport;

    /// Specify template registration requirements (B-MODE)
    fn specify_template_registration(&self, template_spec: &TemplateSpecification) -> SpecificationReport;

    /// Get immutable template catalog for analysis
    fn catalog(&self) -> &TemplateCatalog;
}

/// Template types supported by the constitutional framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TemplateType {
    FailureMatrix,
    SemanticLockSpecification,
    CoreContract,
    GateValidator,
    ComponentInterface,
    ValidationRule,
}

/// Template specification (B-MODE)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateSpecification {
    pub template_type: TemplateType,
    pub specification_version: String,
    pub required_fields: Vec<TemplateField>,
    pub optional_fields: Vec<TemplateField>,
    pub validation_rules: Vec<TemplateValidationRule>,
    pub gate_e_compatibility: bool,
    pub metadata: BTreeMap<String, String>,
}

/// Template field specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateField {
    pub field_name: String,
    pub field_type: FieldType,
    pub description: String,
    pub constraints: Vec<FieldConstraint>,
}

/// Field types for template specifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    ComponentId,
    RuleId,
    Timestamp,
    Array(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
}

/// Field constraints for validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldConstraint {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Range(i64, i64),
    Pattern(String),
    Enum(Vec<String>),
}

/// Template validation rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateValidationRule {
    pub rule_id: String,
    pub description: String,
    pub validation_expression: String,
    pub severity: Severity,
}

/// Template availability analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateAvailabilityReport {
    pub template_type: TemplateType,
    pub is_available: bool,
    pub specification_version: Option<String>,
    pub completeness_score: f64, // Normalized to 6 decimal places
    pub missing_fields: Vec<String>,
    pub validation_issues: Vec<String>,
    pub gate_e_compatible: bool,
}

/// Immutable template catalog
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateCatalog {
    pub specifications: BTreeMap<TemplateType, TemplateSpecification>,
    pub catalog_version: String,
    pub last_updated: crate::types::LogicalTimestamp,
}

/// Default implementation of template specification analyzer (B-MODE)
#[derive(Debug, Clone)]
pub struct DefaultTemplateSpecAnalyzer {
    catalog: TemplateCatalog,
}

impl DefaultTemplateSpecAnalyzer {
    /// Create a new template specification analyzer with default catalog
    pub fn new() -> Self {
        Self {
            catalog: Self::create_default_catalog(),
        }
    }

    /// Create the default immutable template catalog
    fn create_default_catalog() -> TemplateCatalog {
        let mut specifications = BTreeMap::new();

        // FailureMatrix template specification (Requirement 2.1)
        let failure_matrix_spec = TemplateSpecification {
            template_type: TemplateType::FailureMatrix,
            specification_version: "1.0.0".to_string(),
            required_fields: vec![
                TemplateField {
                    field_name: "failure_type".to_string(),
                    field_type: FieldType::String,
                    description: "Type of failure scenario".to_string(),
                    constraints: vec![
                        FieldConstraint::Required,
                        FieldConstraint::Enum(vec![
                            "SpillStorageFull".to_string(),
                            "NativeCacheBoundsFail".to_string(),
                            "LoopCarriedMisuse".to_string(),
                            "ConstitutionalViolation".to_string(),
                            "SemanticLockViolation".to_string(),
                            "AllocationOverhead".to_string(),
                        ]),
                    ],
                },
                TemplateField {
                    field_name: "trigger_conditions".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::String)),
                    description: "Conditions that trigger this failure scenario".to_string(),
                    constraints: vec![FieldConstraint::Required, FieldConstraint::MinLength(1)],
                },
                TemplateField {
                    field_name: "recommended_responses".to_string(),
                    field_type: FieldType::Map(Box::new(FieldType::ComponentId), Box::new(FieldType::String)),
                    description: "Recommended responses per component".to_string(),
                    constraints: vec![FieldConstraint::Required],
                },
                TemplateField {
                    field_name: "determinism_requirements".to_string(),
                    field_type: FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::Boolean)),
                    description: "Determinism requirements for failure handling".to_string(),
                    constraints: vec![FieldConstraint::Required],
                },
            ],
            optional_fields: vec![
                TemplateField {
                    field_name: "recovery_path".to_string(),
                    field_type: FieldType::String,
                    description: "Optional recovery path specification".to_string(),
                    constraints: vec![],
                },
            ],
            validation_rules: vec![
                TemplateValidationRule {
                    rule_id: "FAILURE_MATRIX_COMPLETENESS".to_string(),
                    description: "All required fields must be present and valid".to_string(),
                    validation_expression: "required_fields.all_present() && determinism_requirements.response_must_be_reproducible == true".to_string(),
                    severity: Severity::Error,
                },
            ],
            gate_e_compatibility: true,
            metadata: {
                let mut meta = BTreeMap::new();
                meta.insert("requirement".to_string(), "2.1".to_string());
                meta.insert("criticality".to_string(), "constitutional".to_string());
                meta
            },
        };

        // SemanticLockSpecification template specification (Requirement 2.2)
        let semantic_lock_spec = TemplateSpecification {
            template_type: TemplateType::SemanticLockSpecification,
            specification_version: "1.0.0".to_string(),
            required_fields: vec![
                TemplateField {
                    field_name: "lock_type".to_string(),
                    field_type: FieldType::String,
                    description: "Type of semantic lock".to_string(),
                    constraints: vec![
                        FieldConstraint::Required,
                        FieldConstraint::Enum(vec![
                            "AllocationImmutability".to_string(),
                            "AuthorityHierarchy".to_string(),
                            "ComponentBoundary".to_string(),
                            "DeterminismGuarantee".to_string(),
                        ]),
                    ],
                },
                TemplateField {
                    field_name: "protected_components".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::ComponentId)),
                    description: "Components protected by this semantic lock".to_string(),
                    constraints: vec![FieldConstraint::Required, FieldConstraint::MinLength(1)],
                },
                TemplateField {
                    field_name: "lock_conditions".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::String)),
                    description: "Conditions under which the lock is active".to_string(),
                    constraints: vec![FieldConstraint::Required],
                },
                TemplateField {
                    field_name: "violation_response".to_string(),
                    field_type: FieldType::String,
                    description: "Response to semantic lock violations".to_string(),
                    constraints: vec![
                        FieldConstraint::Required,
                        FieldConstraint::Enum(vec![
                            "RecommendReject".to_string(),
                            "RecommendWarn".to_string(),
                            "RecommendEscalate".to_string(),
                        ]),
                    ],
                },
            ],
            optional_fields: vec![
                TemplateField {
                    field_name: "bypass_conditions".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::String)),
                    description: "Conditions under which the lock can be bypassed".to_string(),
                    constraints: vec![],
                },
            ],
            validation_rules: vec![
                TemplateValidationRule {
                    rule_id: "SEMANTIC_LOCK_COMPLETENESS".to_string(),
                    description: "Semantic lock must have valid protected components and conditions".to_string(),
                    validation_expression: "protected_components.length > 0 && lock_conditions.all_valid()".to_string(),
                    severity: Severity::Error,
                },
            ],
            gate_e_compatibility: true,
            metadata: {
                let mut meta = BTreeMap::new();
                meta.insert("requirement".to_string(), "2.2".to_string());
                meta.insert("criticality".to_string(), "constitutional".to_string());
                meta
            },
        };

        // CoreContract template specification
        let core_contract_spec = TemplateSpecification {
            template_type: TemplateType::CoreContract,
            specification_version: "1.0.0".to_string(),
            required_fields: vec![
                TemplateField {
                    field_name: "contract_id".to_string(),
                    field_type: FieldType::String,
                    description: "Unique identifier for the contract".to_string(),
                    constraints: vec![FieldConstraint::Required, FieldConstraint::MinLength(1)],
                },
                TemplateField {
                    field_name: "contracting_components".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::ComponentId)),
                    description: "Components bound by this contract".to_string(),
                    constraints: vec![FieldConstraint::Required, FieldConstraint::MinLength(2)],
                },
                TemplateField {
                    field_name: "contract_terms".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::String)),
                    description: "Terms and conditions of the contract".to_string(),
                    constraints: vec![FieldConstraint::Required],
                },
            ],
            optional_fields: vec![],
            validation_rules: vec![
                TemplateValidationRule {
                    rule_id: "CORE_CONTRACT_VALIDITY".to_string(),
                    description: "Contract must have at least two components and valid terms".to_string(),
                    validation_expression: "contracting_components.length >= 2 && contract_terms.all_valid()".to_string(),
                    severity: Severity::Error,
                },
            ],
            gate_e_compatibility: false,
            metadata: BTreeMap::new(),
        };

        // GateValidator template specification
        let gate_validator_spec = TemplateSpecification {
            template_type: TemplateType::GateValidator,
            specification_version: "1.0.0".to_string(),
            required_fields: vec![
                TemplateField {
                    field_name: "gate_id".to_string(),
                    field_type: FieldType::String,
                    description: "Unique identifier for the gate".to_string(),
                    constraints: vec![FieldConstraint::Required, FieldConstraint::MinLength(1)],
                },
                TemplateField {
                    field_name: "validation_criteria".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::String)),
                    description: "Criteria for gate validation".to_string(),
                    constraints: vec![FieldConstraint::Required],
                },
                TemplateField {
                    field_name: "required_components".to_string(),
                    field_type: FieldType::Array(Box::new(FieldType::ComponentId)),
                    description: "Components required for gate validation".to_string(),
                    constraints: vec![FieldConstraint::Required],
                },
            ],
            optional_fields: vec![],
            validation_rules: vec![
                TemplateValidationRule {
                    rule_id: "GATE_VALIDATOR_COMPLETENESS".to_string(),
                    description: "Gate validator must have valid criteria and components".to_string(),
                    validation_expression: "validation_criteria.length > 0 && required_components.length > 0".to_string(),
                    severity: Severity::Error,
                },
            ],
            gate_e_compatibility: true,
            metadata: BTreeMap::new(),
        };

        specifications.insert(TemplateType::FailureMatrix, failure_matrix_spec);
        specifications.insert(TemplateType::SemanticLockSpecification, semantic_lock_spec);
        specifications.insert(TemplateType::CoreContract, core_contract_spec);
        specifications.insert(TemplateType::GateValidator, gate_validator_spec);

        TemplateCatalog {
            specifications,
            catalog_version: "1.0.0".to_string(),
            last_updated: DeterministicClock::new().now(),
        }
    }

    /// Normalize floating point values to 6 decimal places for deterministic comparison
    fn normalize_float(value: f64) -> f64 {
        (value * 1_000_000.0_f64).round() / 1_000_000.0
    }
}

impl TemplateSpecAnalyzer for DefaultTemplateSpecAnalyzer {
    fn analyze_template_completeness(&self, template_type: TemplateType) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        if let Some(spec) = self.catalog.specifications.get(&template_type) {
            // Analyze template specification completeness
            if spec.required_fields.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::TemplateSpecRegistry,
                    rule_id: Some("TEMPLATE_REQUIRED_FIELDS".to_string()),
                    description: format!("Template {:?} has no required fields", template_type),
                    remediation_hint: "Add required fields to template specification".to_string(),
                });
            } else {
                report.add_finding(SpecificationFinding {
                    finding_type: FindingType::SpecificationCompliance,
                    component: ComponentId::TemplateSpecRegistry,
                    description: format!("Template {:?} has {} required fields", template_type, spec.required_fields.len()),
                    severity: Severity::Info,
                    location: ValidationLocation::new(ComponentId::TemplateSpecRegistry),
                });
            }

            // Analyze validation rules
            if spec.validation_rules.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::TemplateSpecRegistry,
                    rule_id: Some("TEMPLATE_VALIDATION_RULES".to_string()),
                    description: format!("Template {:?} has no validation rules", template_type),
                    remediation_hint: "Add validation rules to template specification".to_string(),
                });
            }

            // Analyze Gate E compatibility for required templates
            if matches!(template_type, TemplateType::FailureMatrix | TemplateType::SemanticLockSpecification) {
                if !spec.gate_e_compatibility {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationIncomplete,
                        component: ComponentId::TemplateSpecRegistry,
                        rule_id: Some("GATE_E_COMPATIBILITY".to_string()),
                        description: format!("Template {:?} is not Gate E compatible", template_type),
                        remediation_hint: "Ensure template meets Gate E compatibility requirements".to_string(),
                    });
                } else {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::SpecificationCompliance,
                        component: ComponentId::TemplateSpecRegistry,
                        description: format!("Template {:?} is Gate E compatible", template_type),
                        severity: Severity::Info,
                        location: ValidationLocation::new(ComponentId::TemplateSpecRegistry),
                    });
                }
            }
        } else {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::TemplateSpecRegistry,
                rule_id: Some("TEMPLATE_NOT_FOUND".to_string()),
                description: format!("Template {:?} not found in catalog", template_type),
                remediation_hint: "Add template specification to catalog".to_string(),
            });
        }

        report
    }

    fn analyze_template_availability(&self, template_type: TemplateType) -> TemplateAvailabilityReport {
        if let Some(spec) = self.catalog.specifications.get(&template_type) {
            // Calculate completeness score based on specification quality
            let mut completeness_factors = Vec::new();
            
            // Required fields factor
            completeness_factors.push(if spec.required_fields.is_empty() { 0.0 } else { 1.0 });
            
            // Validation rules factor
            completeness_factors.push(if spec.validation_rules.is_empty() { 0.0 } else { 1.0 });
            
            // Gate E compatibility factor (for required templates)
            if matches!(template_type, TemplateType::FailureMatrix | TemplateType::SemanticLockSpecification) {
                completeness_factors.push(if spec.gate_e_compatibility { 1.0 } else { 0.0 });
            }

            let completeness_score = if !completeness_factors.is_empty() {
                completeness_factors.iter().sum::<f64>() / completeness_factors.len() as f64
            } else {
                0.0
            };

            // Analyze missing fields (for demonstration, assume all fields are present)
            let missing_fields = Vec::new();

            // Analyze validation issues
            let mut validation_issues = Vec::new();
            if spec.validation_rules.is_empty() {
                validation_issues.push("No validation rules defined".to_string());
            }
            if spec.required_fields.is_empty() {
                validation_issues.push("No required fields defined".to_string());
            }

            TemplateAvailabilityReport {
                template_type,
                is_available: true,
                specification_version: Some(spec.specification_version.clone()),
                completeness_score: Self::normalize_float(completeness_score),
                missing_fields,
                validation_issues,
                gate_e_compatible: spec.gate_e_compatibility,
            }
        } else {
            TemplateAvailabilityReport {
                template_type,
                is_available: false,
                specification_version: None,
                completeness_score: 0.0,
                missing_fields: vec!["Template not found in catalog".to_string()],
                validation_issues: vec!["Template specification missing".to_string()],
                gate_e_compatible: false,
            }
        }
    }

    fn specify_template_registration(&self, template_spec: &TemplateSpecification) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // In B-MODE, we analyze registration requirements rather than performing registration
        
        // Analyze template specification validity
        if template_spec.required_fields.is_empty() {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::TemplateSpecRegistry,
                rule_id: Some("REGISTRATION_REQUIRED_FIELDS".to_string()),
                description: "Template registration requires at least one required field".to_string(),
                remediation_hint: "Add required fields to template specification before registration".to_string(),
            });
        }

        if template_spec.specification_version.is_empty() {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::TemplateSpecRegistry,
                rule_id: Some("REGISTRATION_VERSION".to_string()),
                description: "Template registration requires specification version".to_string(),
                remediation_hint: "Add specification version to template".to_string(),
            });
        }

        // Analyze Gate E compatibility requirements
        if matches!(template_spec.template_type, TemplateType::FailureMatrix | TemplateType::SemanticLockSpecification) {
            if !template_spec.gate_e_compatibility {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::TemplateSpecRegistry,
                    rule_id: Some("REGISTRATION_GATE_E_COMPATIBILITY".to_string()),
                    description: format!("Template {:?} must be Gate E compatible for registration", template_spec.template_type),
                    remediation_hint: "Ensure template meets Gate E compatibility requirements".to_string(),
                });
            }
        }

        // Check for conflicts with existing templates
        if let Some(existing_spec) = self.catalog.specifications.get(&template_spec.template_type) {
            if existing_spec.specification_version == template_spec.specification_version {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::TemplateSpecRegistry,
                    rule_id: Some("REGISTRATION_VERSION_CONFLICT".to_string()),
                    description: format!("Template {:?} version {} already exists", template_spec.template_type, template_spec.specification_version),
                    remediation_hint: "Use a different version number for template registration".to_string(),
                });
            }
        }

        // If all checks pass, indicate successful registration specification
        if report.violations.is_empty() {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::TemplateSpecRegistry,
                description: format!("Template {:?} registration specification is valid", template_spec.template_type),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::TemplateSpecRegistry),
            });
        }

        report
    }

    fn catalog(&self) -> &TemplateCatalog {
        &self.catalog
    }
}

/// Helper function to create a template specification
pub fn create_template_specification(
    template_type: TemplateType,
    version: String,
    required_fields: Vec<TemplateField>,
    optional_fields: Vec<TemplateField>,
    validation_rules: Vec<TemplateValidationRule>,
    gate_e_compatibility: bool,
) -> TemplateSpecification {
    TemplateSpecification {
        template_type,
        specification_version: version,
        required_fields,
        optional_fields,
        validation_rules,
        gate_e_compatibility,
        metadata: BTreeMap::new(),
    }
}

/// Helper function to create a template field
pub fn create_template_field(
    name: String,
    field_type: FieldType,
    description: String,
    constraints: Vec<FieldConstraint>,
) -> TemplateField {
    TemplateField {
        field_name: name,
        field_type,
        description,
        constraints,
    }
}

/// Helper function to create a template validation rule
pub fn create_validation_rule(
    rule_id: String,
    description: String,
    validation_expression: String,
    severity: Severity,
) -> TemplateValidationRule {
    TemplateValidationRule {
        rule_id,
        description,
        validation_expression,
        severity,
    }
}