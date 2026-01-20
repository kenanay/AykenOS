//! Semantic Specification Registry for D4 Constitutional Framework (B-MODE)
//!
//! This module implements pure B-MODE semantic specification registry that provides
//! immutable semantic lock specification analysis without stateful management or enforcement.
//!
//! B-MODE PRINCIPLES:
//! - All operations return SpecificationReport, never Result<()> for spec violations
//! - Immutable specification analysis (&self), no state mutations
//! - Specification and analysis only, no lock management/enforcement
//! - No lock registration/release operations, only analysis

use crate::errors::{SpecificationReport, SpecificationViolation, SpecificationFinding, ViolationType, FindingType};
use crate::types::{ComponentId, DeterministicClock, Severity, LockedBehavior, AuthorizationLevel};
use crate::bmode::validation_location::ValidationLocation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pure B-MODE semantic specification registry interface
pub trait SemanticSpecRegistry {
    /// Analyze semantic lock specification completeness (B-MODE)
    fn analyze_semantic_lock_specification(&self, behavior: LockedBehavior) -> SpecificationReport;

    /// Analyze lock specification coverage (B-MODE)
    fn analyze_lock_specification_coverage(&self, behaviors: &[LockedBehavior]) -> SpecificationReport;

    /// Specify semantic lock requirements (B-MODE)
    fn specify_semantic_lock_requirements(&self, behavior: LockedBehavior) -> SemanticLockRequirementsReport;

    /// Get immutable semantic specification catalog for analysis
    fn semantic_specification_catalog(&self) -> &SemanticSpecificationCatalog;
}

/// Semantic lock specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticLockSpecification {
    pub lock_id: String,
    pub locked_behavior: LockedBehavior,
    pub authorization_level: AuthorizationLevel,
    pub protected_components: Vec<ComponentId>,
    pub lock_conditions: Vec<String>,
    pub violation_response: ViolationResponseSpec,
    pub bypass_conditions: Vec<String>,
    pub maintenance_authorization: MaintenanceAuthorizationSpec,
}

/// Violation response specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationResponseSpec {
    RecommendReject,
    RecommendWarn,
    RecommendEscalate(ComponentId),
    RecommendCustomActions(Vec<String>),
}

/// Maintenance authorization specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceAuthorizationSpec {
    pub authorized_components: Vec<ComponentId>,
    pub authorization_conditions: Vec<String>,
    pub maintenance_window_required: bool,
    pub approval_process: ApprovalProcessSpec,
}

/// Approval process specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalProcessSpec {
    AutomaticApproval,
    ComponentApproval(ComponentId),
    MultiComponentApproval(Vec<ComponentId>),
    ConstitutionalApproval,
}

/// Semantic lock requirements analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticLockRequirementsReport {
    pub behavior: LockedBehavior,
    pub lock_required: bool,
    pub authorization_level: AuthorizationLevel,
    pub protected_components: Vec<ComponentId>,
    pub constitutional_compliance: bool,
    pub maintenance_authorization_required: bool,
    pub closed_world_behavior_extension: bool,
    pub analysis_timestamp: crate::types::LogicalTimestamp,
}

/// Immutable semantic specification catalog
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSpecificationCatalog {
    pub lock_specifications: BTreeMap<LockedBehavior, SemanticLockSpecification>,
    pub catalog_version: String,
    pub last_updated: crate::types::LogicalTimestamp,
}

/// Default implementation of semantic specification registry (B-MODE)
#[derive(Debug, Clone)]
pub struct DefaultSemanticSpecRegistry {
    catalog: SemanticSpecificationCatalog,
}

impl DefaultSemanticSpecRegistry {
    /// Create a new semantic specification registry with default catalog
    pub fn new() -> Self {
        Self {
            catalog: Self::create_default_catalog(),
        }
    }

    /// Create the default immutable semantic specification catalog
    fn create_default_catalog() -> SemanticSpecificationCatalog {
        let mut lock_specifications = BTreeMap::new();

        // Register Allocation Algorithm lock specification
        let register_allocation_lock = SemanticLockSpecification {
            lock_id: "register_allocation_algorithm_lock".to_string(),
            locked_behavior: LockedBehavior::RegisterAllocationAlgorithm,
            authorization_level: AuthorizationLevel::Constitutional,
            protected_components: vec![ComponentId::D4RegisterAllocator],
            lock_conditions: vec![
                "allocation_immutability_active".to_string(),
                "constitutional_rules_enforced".to_string(),
            ],
            violation_response: ViolationResponseSpec::RecommendReject,
            bypass_conditions: vec![], // No bypass allowed for constitutional locks
            maintenance_authorization: MaintenanceAuthorizationSpec {
                authorized_components: vec![ComponentId::ConstitutionalRuleEngine],
                authorization_conditions: vec![
                    "maintenance_window_active".to_string(),
                    "constitutional_approval_granted".to_string(),
                ],
                maintenance_window_required: true,
                approval_process: ApprovalProcessSpec::ConstitutionalApproval,
            },
        };

        // Failure Handling Procedures lock specification
        let failure_handling_lock = SemanticLockSpecification {
            lock_id: "failure_handling_procedures_lock".to_string(),
            locked_behavior: LockedBehavior::FailureHandlingProcedures,
            authorization_level: AuthorizationLevel::Constitutional,
            protected_components: vec![
                ComponentId::FailureMatrix,
                ComponentId::D1Component,
                ComponentId::D2Component,
                ComponentId::D3Component,
                ComponentId::D4RegisterAllocator,
            ],
            lock_conditions: vec![
                "deterministic_failure_handling_active".to_string(),
                "failure_matrix_complete".to_string(),
            ],
            violation_response: ViolationResponseSpec::RecommendReject,
            bypass_conditions: vec![], // No bypass allowed for constitutional locks
            maintenance_authorization: MaintenanceAuthorizationSpec {
                authorized_components: vec![ComponentId::ConstitutionalRuleEngine],
                authorization_conditions: vec![
                    "maintenance_window_active".to_string(),
                    "failure_matrix_validation_passed".to_string(),
                ],
                maintenance_window_required: true,
                approval_process: ApprovalProcessSpec::ConstitutionalApproval,
            },
        };

        // Authority Hierarchy lock specification
        let authority_hierarchy_lock = SemanticLockSpecification {
            lock_id: "authority_hierarchy_lock".to_string(),
            locked_behavior: LockedBehavior::AuthorityHierarchy,
            authorization_level: AuthorizationLevel::Constitutional,
            protected_components: vec![
                ComponentId::D4RegisterAllocator,
                ComponentId::LoopOptimizer,
                ComponentId::UnrollOptimizer,
                ComponentId::JITCompiler,
            ],
            lock_conditions: vec![
                "authority_hierarchy_enforced".to_string(),
                "component_precedence_defined".to_string(),
            ],
            violation_response: ViolationResponseSpec::RecommendReject,
            bypass_conditions: vec![], // No bypass allowed for constitutional locks
            maintenance_authorization: MaintenanceAuthorizationSpec {
                authorized_components: vec![ComponentId::ConstitutionalRuleEngine],
                authorization_conditions: vec![
                    "maintenance_window_active".to_string(),
                    "hierarchy_validation_passed".to_string(),
                ],
                maintenance_window_required: true,
                approval_process: ApprovalProcessSpec::ConstitutionalApproval,
            },
        };

        // Gate Transition Logic lock specification
        let gate_transition_lock = SemanticLockSpecification {
            lock_id: "gate_transition_logic_lock".to_string(),
            locked_behavior: LockedBehavior::GateTransitionLogic,
            authorization_level: AuthorizationLevel::Administrative,
            protected_components: vec![ComponentId::ConstitutionalRuleEngine],
            lock_conditions: vec![
                "gate_validation_active".to_string(),
                "transition_criteria_defined".to_string(),
            ],
            violation_response: ViolationResponseSpec::RecommendWarn,
            bypass_conditions: vec![
                "emergency_override_authorized".to_string(),
            ],
            maintenance_authorization: MaintenanceAuthorizationSpec {
                authorized_components: vec![
                    ComponentId::ConstitutionalRuleEngine,
                    ComponentId::DeterminismEngine,
                ],
                authorization_conditions: vec![
                    "gate_validation_passed".to_string(),
                ],
                maintenance_window_required: false,
                approval_process: ApprovalProcessSpec::ComponentApproval(ComponentId::ConstitutionalRuleEngine),
            },
        };

        // Determinism Requirements lock specification
        let determinism_requirements_lock = SemanticLockSpecification {
            lock_id: "determinism_requirements_lock".to_string(),
            locked_behavior: LockedBehavior::DeterminismRequirements,
            authorization_level: AuthorizationLevel::Constitutional,
            protected_components: vec![
                ComponentId::DeterminismEngine,
                ComponentId::D4RegisterAllocator,
            ],
            lock_conditions: vec![
                "deterministic_behavior_enforced".to_string(),
                "reproducibility_guaranteed".to_string(),
            ],
            violation_response: ViolationResponseSpec::RecommendReject,
            bypass_conditions: vec![], // No bypass allowed for constitutional locks
            maintenance_authorization: MaintenanceAuthorizationSpec {
                authorized_components: vec![ComponentId::ConstitutionalRuleEngine],
                authorization_conditions: vec![
                    "maintenance_window_active".to_string(),
                    "determinism_validation_passed".to_string(),
                ],
                maintenance_window_required: true,
                approval_process: ApprovalProcessSpec::ConstitutionalApproval,
            },
        };

        lock_specifications.insert(LockedBehavior::RegisterAllocationAlgorithm, register_allocation_lock);
        lock_specifications.insert(LockedBehavior::FailureHandlingProcedures, failure_handling_lock);
        lock_specifications.insert(LockedBehavior::AuthorityHierarchy, authority_hierarchy_lock);
        lock_specifications.insert(LockedBehavior::GateTransitionLogic, gate_transition_lock);
        lock_specifications.insert(LockedBehavior::DeterminismRequirements, determinism_requirements_lock);

        SemanticSpecificationCatalog {
            lock_specifications,
            catalog_version: "1.0.0".to_string(),
            last_updated: DeterministicClock::new().now(),
        }
    }
}

impl SemanticSpecRegistry for DefaultSemanticSpecRegistry {
    fn analyze_semantic_lock_specification(&self, behavior: LockedBehavior) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        if let Some(lock_spec) = self.catalog.lock_specifications.get(&behavior) {
            // Analyze lock specification completeness
            if lock_spec.protected_components.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::SemanticSpecificationRegistry,
                    rule_id: Some("LOCK_PROTECTED_COMPONENTS".to_string()),
                    description: format!("Semantic lock for {:?} has no protected components", behavior),
                    remediation_hint: "Add protected components to semantic lock specification".to_string(),
                });
            }

            if lock_spec.lock_conditions.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::SemanticSpecificationRegistry,
                    rule_id: Some("LOCK_CONDITIONS".to_string()),
                    description: format!("Semantic lock for {:?} has no lock conditions", behavior),
                    remediation_hint: "Add lock conditions to semantic lock specification".to_string(),
                });
            }

            // Analyze constitutional compliance
            if lock_spec.authorization_level == AuthorizationLevel::Constitutional {
                if !lock_spec.bypass_conditions.is_empty() {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationViolation,
                        component: ComponentId::SemanticSpecificationRegistry,
                        rule_id: Some("CONSTITUTIONAL_LOCK_BYPASS".to_string()),
                        description: format!("Constitutional lock for {:?} should not have bypass conditions", behavior),
                        remediation_hint: "Remove bypass conditions from constitutional locks".to_string(),
                    });
                }

                if !matches!(lock_spec.maintenance_authorization.approval_process, ApprovalProcessSpec::ConstitutionalApproval) {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationViolation,
                        component: ComponentId::SemanticSpecificationRegistry,
                        rule_id: Some("CONSTITUTIONAL_LOCK_APPROVAL".to_string()),
                        description: format!("Constitutional lock for {:?} must require constitutional approval", behavior),
                        remediation_hint: "Set approval process to ConstitutionalApproval for constitutional locks".to_string(),
                    });
                }
            }

            // Analyze maintenance authorization
            if lock_spec.maintenance_authorization.authorized_components.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::SemanticSpecificationRegistry,
                    rule_id: Some("MAINTENANCE_AUTHORIZATION".to_string()),
                    description: format!("Semantic lock for {:?} has no maintenance authorization", behavior),
                    remediation_hint: "Add maintenance authorization to semantic lock specification".to_string(),
                });
            }

            if report.violations.is_empty() {
                report.add_finding(SpecificationFinding {
                    finding_type: FindingType::SpecificationCompliance,
                    component: ComponentId::SemanticSpecificationRegistry,
                    description: format!("Semantic lock specification for {:?} is complete and compliant", behavior),
                    severity: Severity::Info,
                    location: ValidationLocation::new(ComponentId::SemanticSpecificationRegistry),
                });
            }
        } else {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::SemanticSpecificationRegistry,
                rule_id: Some("LOCK_SPECIFICATION_NOT_FOUND".to_string()),
                description: format!("No semantic lock specification found for behavior {:?}", behavior),
                remediation_hint: "Add semantic lock specification for this behavior".to_string(),
            });
        }

        report
    }

    fn analyze_lock_specification_coverage(&self, behaviors: &[LockedBehavior]) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        for behavior in behaviors {
            let has_specification = self.catalog.lock_specifications.contains_key(behavior);

            if !has_specification {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::SemanticSpecificationRegistry,
                    rule_id: Some("MISSING_LOCK_SPECIFICATION".to_string()),
                    description: format!("No semantic lock specification found for behavior {:?}", behavior),
                    remediation_hint: "Add semantic lock specification for this behavior".to_string(),
                });
            } else {
                report.add_finding(SpecificationFinding {
                    finding_type: FindingType::SpecificationCompliance,
                    component: ComponentId::SemanticSpecificationRegistry,
                    description: format!("Semantic lock specification coverage confirmed for behavior {:?}", behavior),
                    severity: Severity::Info,
                    location: ValidationLocation::new(ComponentId::SemanticSpecificationRegistry),
                });
            }
        }

        report
    }

    fn specify_semantic_lock_requirements(&self, behavior: LockedBehavior) -> SemanticLockRequirementsReport {
        let lock_required = matches!(
            behavior,
            LockedBehavior::RegisterAllocationAlgorithm |
            LockedBehavior::FailureHandlingProcedures |
            LockedBehavior::AuthorityHierarchy |
            LockedBehavior::DeterminismRequirements
        );

        let authorization_level = if matches!(
            behavior,
            LockedBehavior::RegisterAllocationAlgorithm |
            LockedBehavior::FailureHandlingProcedures |
            LockedBehavior::AuthorityHierarchy |
            LockedBehavior::DeterminismRequirements
        ) {
            AuthorizationLevel::Constitutional
        } else {
            AuthorizationLevel::Administrative
        };

        let protected_components = match behavior {
            LockedBehavior::RegisterAllocationAlgorithm => vec![ComponentId::D4RegisterAllocator],
            LockedBehavior::FailureHandlingProcedures => vec![
                ComponentId::FailureMatrix,
                ComponentId::D1Component,
                ComponentId::D2Component,
                ComponentId::D3Component,
                ComponentId::D4RegisterAllocator,
            ],
            LockedBehavior::AuthorityHierarchy => vec![
                ComponentId::D4RegisterAllocator,
                ComponentId::LoopOptimizer,
                ComponentId::UnrollOptimizer,
                ComponentId::JITCompiler,
            ],
            LockedBehavior::GateTransitionLogic => vec![ComponentId::ConstitutionalRuleEngine],
            LockedBehavior::DeterminismRequirements => vec![
                ComponentId::DeterminismEngine,
                ComponentId::D4RegisterAllocator,
            ],
        };

        let constitutional_compliance = authorization_level == AuthorizationLevel::Constitutional;
        let maintenance_authorization_required = lock_required;
        let closed_world_behavior_extension = matches!(behavior, LockedBehavior::DeterminismRequirements);

        SemanticLockRequirementsReport {
            behavior,
            lock_required,
            authorization_level,
            protected_components,
            constitutional_compliance,
            maintenance_authorization_required,
            closed_world_behavior_extension,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }

    fn semantic_specification_catalog(&self) -> &SemanticSpecificationCatalog {
        &self.catalog
    }
}

/// Helper function to create a semantic lock specification
pub fn create_semantic_lock_specification(
    lock_id: String,
    locked_behavior: LockedBehavior,
    authorization_level: AuthorizationLevel,
    protected_components: Vec<ComponentId>,
) -> SemanticLockSpecification {
    SemanticLockSpecification {
        lock_id,
        locked_behavior,
        authorization_level,
        protected_components,
        lock_conditions: Vec::new(),
        violation_response: ViolationResponseSpec::RecommendReject,
        bypass_conditions: Vec::new(),
        maintenance_authorization: MaintenanceAuthorizationSpec {
            authorized_components: vec![ComponentId::ConstitutionalRuleEngine],
            authorization_conditions: Vec::new(),
            maintenance_window_required: true,
            approval_process: ApprovalProcessSpec::ConstitutionalApproval,
        },
    }
}

/// Helper function to create maintenance authorization specification
pub fn create_maintenance_authorization_specification(
    authorized_components: Vec<ComponentId>,
    approval_process: ApprovalProcessSpec,
) -> MaintenanceAuthorizationSpec {
    MaintenanceAuthorizationSpec {
        authorized_components,
        authorization_conditions: Vec::new(),
        maintenance_window_required: true,
        approval_process,
    }
}