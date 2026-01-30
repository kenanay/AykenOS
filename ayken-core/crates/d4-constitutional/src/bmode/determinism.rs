//! Determinism Analysis for the D4 Constitutional Framework (B-MODE)
//!
//! This module implements pure B-MODE deterministic behavior analysis and specification reporting
//! to ensure reproducible system behavior and constitutional compliance analysis.
//!
//! B-MODE PRINCIPLES:
//! - All operations return SpecificationReport, never Result<()> for spec violations
//! - Immutable analysis (&self), no state mutations
//! - Specification and analysis only, no enforcement
//! - No stateful engines, caches, or audit logs

use crate::errors::{SpecificationReport, SpecificationViolation, SpecificationFinding, ViolationType, FindingType};
use crate::types::{*, Severity};
use crate::bmode::validation_location::ValidationLocation;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Program point for deterministic analysis (B-MODE)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProgramPoint {
    pub function: String,
    pub offset: u64,
}

/// Pure B-MODE determinism analysis interface
pub trait DeterminismAnalyzer {
    /// Analyze allocation decisions for reproducibility compliance (B-MODE)
    fn analyze_allocation_reproducibility(&self, inputs: &AllocationInputs) -> SpecificationReport;

    /// Analyze state changes for constitutional compliance (B-MODE)
    fn analyze_state_change_compliance(&self, change: &StateChange) -> SpecificationReport;

    /// Specify audit log requirements for constitutional actions (B-MODE)
    fn specify_audit_log_requirements(&self, action: &ConstitutionalAction) -> SpecificationReport;

    /// Analyze allocation independence from IR fingerprint for caching (B-MODE)
    fn analyze_allocation_fingerprint_independence(
        &self,
        allocation: &AllocationDecision,
        fingerprint: &IRFingerprint,
    ) -> SpecificationReport;

    /// Analyze failure scenario reproducibility requirements (B-MODE)
    fn analyze_failure_scenario_reproducibility(
        &self,
        scenario: &FailureScenario,
        system_state: &SystemState,
    ) -> SpecificationReport;

    /// Analyze audit log specification requirements (B-MODE)
    fn analyze_audit_log_specification(&self, log_spec: &AuditLogSpec) -> SpecificationReport;
}

/// Inputs to allocation decisions that must be deterministic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationInputs {
    pub ir_fingerprint: String,
    pub virtual_registers: Vec<VirtualRegisterId>,
    pub constraints: AllocationConstraints,
    pub optimization_level: OptimizationLevel,
    pub target_architecture: TargetArchitecture,
}

/// State changes that must be validated for constitutional compliance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateChange {
    pub change_type: StateChangeType,
    pub component: ComponentId,
    pub before_state: String, // Serialized state
    pub after_state: String,  // Serialized state
    pub timestamp: LogicalTimestamp,
}

/// Types of state changes that can occur
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateChangeType {
    AllocationDecision,
    CacheStateChange,
    OptimizationLevelChange,
    ComponentConfiguration,
}

/// Constitutional actions that must be audited
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalAction {
    pub action_type: ActionType,
    pub component: ComponentId,
    pub rule_id: Option<RuleId>,
    pub description: String,
    pub timestamp: LogicalTimestamp,
    pub context: BTreeMap<String, String>,
}

/// Types of constitutional actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    RuleEnforcement,
    ViolationRejection,
    StateValidation,
    AuditLogEntry,
    FailureScenarioHandling,
    AllocationDecision,
}

/// System state for failure scenario validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemState {
    pub component_states: BTreeMap<ComponentId, ComponentState>,
    pub active_allocations: Vec<AllocationDecision>,
    pub cache_state: CacheState,
    pub optimization_state: OptimizationState,
    pub timestamp: LogicalTimestamp,
}

/// Individual component state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentState {
    pub component_id: ComponentId,
    pub state_data: BTreeMap<String, String>,
    pub last_update: LogicalTimestamp,
}

/// Cache system state
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CacheState {
    Enabled,
    Disabled,
    TemporarilyDisabled,
    Failed,
}

/// Optimization system state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationState {
    pub current_level: OptimizationLevel,
    pub active_optimizers: Vec<ComponentId>,
    pub optimization_context: BTreeMap<String, String>,
}

/// Failure scenario definition for deterministic handling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureScenario {
    pub scenario_id: String,
    pub scenario_type: FailureType,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub expected_responses: BTreeMap<ComponentId, RecommendedSystemResponse>,
    pub determinism_requirements: DeterminismRequirements,
}

/// Types of system failures that must be handled deterministically
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureType {
    SpillStorageFull,
    NativeCacheBoundsFail,
    LoopCarriedMisuse,
    ConstitutionalViolation,
    SemanticLockViolation,
    AllocationOverhead,
}

/// Conditions that trigger failure scenarios
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub condition_type: String,
    pub threshold: Option<f64>,
    pub component: ComponentId,
    pub parameters: BTreeMap<String, String>,
}

/// Recommended system responses in response to failures (B-MODE)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedSystemResponse {
    RecommendDisableOptimization,
    RecommendFallbackToSafeMode,
    RecommendLogAndContinue,
    RecommendEscalateToHigherLevel,
    RecommendTermination,
    RecommendDisableCache,
    RecommendReduceOptimizationLevel,
}

/// Determinism requirements for failure handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismRequirements {
    pub response_must_be_reproducible: bool,
    pub state_changes_must_be_auditable: bool,
    pub recovery_path_must_be_defined: bool,
    pub timing_must_be_deterministic: bool,
}

/// IR fingerprint that excludes allocation decisions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IRFingerprint {
    pub structural_hash: String,
    pub lifetime_analysis_cache: Option<LifetimeCache>,
    // Allocation decisions are explicitly excluded
}

/// Cached lifetime analysis information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifetimeCache {
    pub analysis_version: String,
    pub cached_lifetimes: BTreeMap<VirtualRegisterId, LifetimeInfo>,
    pub cache_timestamp: LogicalTimestamp,
}

/// Lifetime information for virtual registers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifetimeInfo {
    pub start_point: ProgramPoint,
    pub end_point: ProgramPoint,
    pub interference_set: Vec<VirtualRegisterId>,
}

/// Audit log specification for B-MODE analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogSpec {
    pub schema_version: String,
    pub required_fields: Vec<String>,
    pub hash_chain_required: bool,
    pub monotonic_counter_required: bool,
    pub append_only_guarantee: bool,
    pub integrity_verification_method: String,
}

/// Default implementation of the determinism analyzer (B-MODE)
#[derive(Debug, Clone)]
pub struct DefaultDeterminismAnalyzer {
    determinism_seed: u64,
}

impl DefaultDeterminismAnalyzer {
    /// Create a new determinism analyzer with deterministic seed
    pub fn new() -> Self {
        Self::with_seed(0x1234567890ABCDEF) // Default CI seed
    }

    /// Create a new determinism analyzer with specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            determinism_seed: seed,
        }
    }

    /// Get the determinism seed used by this analyzer
    pub fn get_determinism_seed(&self) -> u64 {
        self.determinism_seed
    }

    /// Compute deterministic hash of allocation inputs
    fn compute_input_hash(&self, inputs: &AllocationInputs) -> String {
        let mut hasher = Sha256::new();
        
        // Hash inputs in deterministic order
        hasher.update(inputs.ir_fingerprint.as_bytes());
        hasher.update(format!("{:?}", inputs.virtual_registers).as_bytes());
        hasher.update(format!("{:?}", inputs.constraints).as_bytes());
        hasher.update(format!("{:?}", inputs.optimization_level).as_bytes());
        hasher.update(format!("{:?}", inputs.target_architecture).as_bytes());
        hasher.update(self.determinism_seed.to_string().as_bytes());
        
        hex::encode(hasher.finalize())
    }

    /// Analyze input determinism (no random elements)
    fn analyze_input_determinism(&self, inputs: &AllocationInputs) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Check for non-deterministic elements in inputs
        if inputs.ir_fingerprint.is_empty() {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("DETERMINISM_INPUT_001".to_string()),
                description: "IR fingerprint is empty - cannot ensure deterministic allocation".to_string(),
                remediation_hint: "Provide valid IR fingerprint for deterministic allocation decisions".to_string(),
            });
        }

        if inputs.virtual_registers.is_empty() {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("DETERMINISM_INPUT_002".to_string()),
                description: "No virtual registers specified - cannot perform allocation analysis".to_string(),
                remediation_hint: "Provide virtual register list for allocation analysis".to_string(),
            });
        }

        // Check for deterministic ordering of virtual registers
        let mut sorted_registers = inputs.virtual_registers.clone();
        sorted_registers.sort();
        if sorted_registers != inputs.virtual_registers {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::BoundaryViolation,
                component: ComponentId::DeterminismEngine,
                description: "Virtual registers are not in deterministic order".to_string(),
                severity: Severity::Warning,
                location: ValidationLocation::new(ComponentId::DeterminismEngine),
            });
        }

        report
    }

    /// Analyze IR fingerprint independence from allocation decisions
    fn analyze_ir_fingerprint_independence(&self, inputs: &AllocationInputs) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze whether IR fingerprint contains allocation-dependent data
        if inputs.ir_fingerprint.contains("allocation") || inputs.ir_fingerprint.contains("register_") {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("FINGERPRINT_INDEPENDENCE_001".to_string()),
                description: "IR fingerprint appears to contain allocation-dependent data".to_string(),
                remediation_hint: "Ensure IR fingerprint only contains structural information, not allocation decisions".to_string(),
            });
        } else {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::DeterminismEngine,
                description: "IR fingerprint appears to be allocation-independent".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::DeterminismEngine),
            });
        }

        report
    }

    /// Analyze cache state determinism
    fn analyze_cache_state_determinism(&self, change: &StateChange) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Parse cache state from serialized data
        if change.before_state == change.after_state {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: change.component,
                description: "Cache state change is deterministic (no actual change)".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(change.component),
            });
        } else {
            // Analyze the nature of the cache state change
            if change.after_state.contains("disabled") {
                report.add_finding(SpecificationFinding {
                    finding_type: FindingType::SpecificationCompliance,
                    component: change.component,
                    description: "Cache state change to disabled is deterministic".to_string(),
                    severity: Severity::Info,
                    location: ValidationLocation::new(change.component),
                });
            } else if change.after_state.contains("random") || change.after_state.contains("nondeterministic") {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::DeterminismViolation,
                    component: change.component,
                    rule_id: Some("CACHE_DETERMINISM_001".to_string()),
                    description: "Cache state change contains non-deterministic elements".to_string(),
                    remediation_hint: "Ensure all cache state changes are deterministic and reproducible".to_string(),
                });
            }
        }

        report
    }

    /// Analyze component configuration determinism
    fn analyze_component_configuration_determinism(&self, change: &StateChange) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze configuration changes for determinism
        if change.before_state.len() != change.after_state.len() {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: change.component,
                description: "Component configuration size changed deterministically".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(change.component),
            });
        }

        // Check for non-deterministic configuration elements
        if change.after_state.contains("uuid") || change.after_state.contains("timestamp") {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: change.component,
                rule_id: Some("CONFIG_DETERMINISM_001".to_string()),
                description: "Component configuration contains non-deterministic elements (UUID/timestamp)".to_string(),
                remediation_hint: "Use deterministic identifiers and logical timestamps instead of UUIDs and wall-clock time".to_string(),
            });
        }

        report
    }

    /// Analyze temporal consistency of state changes
    fn analyze_temporal_consistency(&self, change: &StateChange) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze timestamp for deterministic ordering
        if change.timestamp.value() == 0 {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: change.component,
                rule_id: Some("TEMPORAL_CONSISTENCY_001".to_string()),
                description: "State change has invalid timestamp (0) - cannot ensure temporal ordering".to_string(),
                remediation_hint: "Use valid logical timestamps for all state changes".to_string(),
            });
        } else {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: change.component,
                description: "State change has valid timestamp for temporal ordering".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(change.component),
            });
        }

        report
    }

    /// Analyze lifetime cache independence
    fn analyze_lifetime_cache_independence(&self, cache: &LifetimeCache) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze cache for allocation independence
        if cache.analysis_version.contains("allocation") {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("LIFETIME_CACHE_INDEPENDENCE_001".to_string()),
                description: "Lifetime cache analysis version contains allocation-dependent information".to_string(),
                remediation_hint: "Ensure lifetime cache is independent of allocation decisions".to_string(),
            });
        }

        // Check cache timestamp for determinism
        if cache.cache_timestamp.value() == 0 {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("LIFETIME_CACHE_TIMESTAMP_001".to_string()),
                description: "Lifetime cache has invalid timestamp - cannot ensure cache validity".to_string(),
                remediation_hint: "Use valid logical timestamps for cache entries".to_string(),
            });
        }

        report
    }

    /// Compute failure scenario hash for reproducibility analysis
    fn compute_failure_scenario_hash(&self, scenario: &FailureScenario, system_state: &SystemState) -> String {
        let mut hasher = Sha256::new();
        
        hasher.update(scenario.scenario_id.as_bytes());
        hasher.update(format!("{:?}", scenario.scenario_type).as_bytes());
        hasher.update(format!("{:?}", scenario.trigger_conditions).as_bytes());
        hasher.update(format!("{:?}", system_state.component_states).as_bytes());
        hasher.update(self.determinism_seed.to_string().as_bytes());
        
        hex::encode(hasher.finalize())
    }

    /// Extract structural data only (no allocation decisions)
    fn extract_structural_data_only(&self, context: &BTreeMap<String, String>) -> String {
        let mut structural_data = String::new();
        
        // Only include structural information, exclude allocation decisions
        for (key, value) in context {
            if !key.contains("allocation") && !key.contains("register_binding") {
                structural_data.push_str(&format!("{}:{};", key, value));
            }
        }
        
        structural_data
    }
}

impl DeterminismAnalyzer for DefaultDeterminismAnalyzer {
    fn analyze_allocation_reproducibility(&self, inputs: &AllocationInputs) -> SpecificationReport {
        let mut report = SpecificationReport::new();
        
        // Analyze input determinism (no random elements)
        let determinism_analysis = self.analyze_input_determinism(inputs);
        report.merge(determinism_analysis);
        
        // Analyze IR fingerprint independence from allocation decisions
        let independence_analysis = self.analyze_ir_fingerprint_independence(inputs);
        report.merge(independence_analysis);
        
        // Create deterministic hash for reproducibility verification
        let input_hash = self.compute_input_hash(inputs);
        report.add_finding(SpecificationFinding {
            finding_type: FindingType::SpecificationCompliance,
            component: ComponentId::DeterminismEngine,
            description: format!("Allocation inputs produce deterministic hash: {}", &input_hash[..16]),
            severity: Severity::Info,
            location: ValidationLocation::new(ComponentId::DeterminismEngine),
        });
        
        report
    }

    fn analyze_state_change_compliance(&self, change: &StateChange) -> SpecificationReport {
        let mut report = SpecificationReport::new();
        
        // Analyze state change constitutional compliance
        match change.change_type {
            StateChangeType::AllocationDecision => {
                // Allocation decisions must come from authorized components
                if change.component != ComponentId::D4RegisterAllocator {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::UnauthorizedOperation,
                        component: change.component,
                        rule_id: Some("DETERMINISM_AUTH_001".to_string()),
                        description: format!(
                            "Allocation decision attempted by unauthorized component: {:?}",
                            change.component
                        ),
                        remediation_hint: "Only D4RegisterAllocator may make allocation decisions".to_string(),
                    });
                } else {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::SpecificationCompliance,
                        component: change.component,
                        description: "Allocation decision from authorized component (D4RegisterAllocator)".to_string(),
                        severity: Severity::Info,
                        location: ValidationLocation::new(change.component),
                    });
                }
            }
            StateChangeType::CacheStateChange => {
                // Cache state changes must be deterministic and from authorized components
                if change.component != ComponentId::NativeCache {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::UnauthorizedOperation,
                        component: change.component,
                        rule_id: Some("DETERMINISM_CACHE_001".to_string()),
                        description: "Cache state change from non-cache component".to_string(),
                        remediation_hint: "Only NativeCache component may change cache state".to_string(),
                    });
                }
                
                // Analyze cache state determinism
                let cache_analysis = self.analyze_cache_state_determinism(change);
                report.merge(cache_analysis);
            }
            StateChangeType::OptimizationLevelChange => {
                // Optimization level changes must be from authorized optimizers
                match change.component {
                    ComponentId::LoopOptimizer | ComponentId::UnrollOptimizer | ComponentId::D4RegisterAllocator => {
                        // Authorized components - add compliance finding
                        report.add_finding(SpecificationFinding {
                            finding_type: FindingType::SpecificationCompliance,
                            component: change.component,
                            description: "Optimization level change from authorized component".to_string(),
                            severity: Severity::Info,
                            location: ValidationLocation::new(change.component),
                        });
                    }
                    _ => {
                        report.add_violation(SpecificationViolation {
                            violation_type: ViolationType::UnauthorizedOperation,
                            component: change.component,
                            rule_id: Some("DETERMINISM_OPT_001".to_string()),
                            description: format!(
                                "Optimization level change from unauthorized component: {:?}",
                                change.component
                            ),
                            remediation_hint: "Only authorized optimizers may change optimization levels".to_string(),
                        });
                    }
                }
            }
            StateChangeType::ComponentConfiguration => {
                // Component configuration changes must be deterministic
                let config_analysis = self.analyze_component_configuration_determinism(change);
                report.merge(config_analysis);
            }
        }
        
        // Analyze temporal consistency (changes must be in proper order)
        let temporal_analysis = self.analyze_temporal_consistency(change);
        report.merge(temporal_analysis);
        
        report
    }

    fn specify_audit_log_requirements(&self, action: &ConstitutionalAction) -> SpecificationReport {
        let mut report = SpecificationReport::new();
        
        // Specify audit log requirements for this action
        report.add_finding(SpecificationFinding {
            finding_type: FindingType::SpecificationCompliance,
            component: ComponentId::DeterminismEngine,
            description: format!("Audit log entry required for action: {:?}", action.action_type),
            severity: Severity::Info,
            location: ValidationLocation::new(ComponentId::DeterminismEngine),
        });
        
        // Analyze action for audit requirements
        if action.context.is_empty() {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::IncompleteSpecification,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("AUDIT_CONTEXT_001".to_string()),
                description: "Constitutional action must include context for audit trail".to_string(),
                remediation_hint: "Add context information to constitutional action".to_string(),
            });
        }

        // Analyze action timestamp for determinism
        if action.timestamp.value() == 0 {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: action.component,
                rule_id: Some("AUDIT_TIMESTAMP_001".to_string()),
                description: "Constitutional action has invalid timestamp for audit log".to_string(),
                remediation_hint: "Use valid logical timestamp for constitutional actions".to_string(),
            });
        }
        
        report
    }

    fn analyze_allocation_fingerprint_independence(
        &self,
        allocation: &AllocationDecision,
        fingerprint: &IRFingerprint,
    ) -> SpecificationReport {
        let mut report = SpecificationReport::new();
        
        // Analyze whether allocation decisions affect the IR fingerprint
        // The fingerprint should only contain structural information
        
        // Analyze structural data extraction
        let structural_data = self.extract_structural_data_only(&allocation.decision_context.constraints.performance_hints.iter().map(|h| (h.clone(), "hint".to_string())).collect());
        let mut hasher = Sha256::new();
        hasher.update(structural_data.as_bytes());
        hasher.update("allocation_independent_v1".as_bytes());
        hasher.update(self.determinism_seed.to_string().as_bytes());
        
        let expected_hash = hex::encode(hasher.finalize());
        
        if fingerprint.structural_hash != expected_hash {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("FINGERPRINT_INDEPENDENCE_001".to_string()),
                description: "IR fingerprint contains allocation-dependent data".to_string(),
                remediation_hint: "Ensure IR fingerprint only contains structural information".to_string(),
            });
        } else {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::DeterminismEngine,
                description: "IR fingerprint is properly independent of allocation decisions".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::DeterminismEngine),
            });
        }
        
        // Analyze lifetime analysis cache for allocation independence
        if let Some(ref cache) = fingerprint.lifetime_analysis_cache {
            let cache_analysis = self.analyze_lifetime_cache_independence(cache);
            report.merge(cache_analysis);
        }
        
        report
    }

    fn analyze_failure_scenario_reproducibility(
        &self,
        scenario: &FailureScenario,
        system_state: &SystemState,
    ) -> SpecificationReport {
        let mut report = SpecificationReport::new();
        
        // Analyze failure scenario for deterministic behavior
        if !scenario.determinism_requirements.response_must_be_reproducible {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("FAILURE_REPRODUCIBILITY_001".to_string()),
                description: "Failure scenario does not require reproducible responses".to_string(),
                remediation_hint: "All failure scenarios must have reproducible responses".to_string(),
            });
        }

        // Analyze system state for deterministic failure handling
        let scenario_hash = self.compute_failure_scenario_hash(scenario, system_state);
        report.add_finding(SpecificationFinding {
            finding_type: FindingType::SpecificationCompliance,
            component: ComponentId::DeterminismEngine,
            description: format!("Failure scenario produces deterministic hash: {}", &scenario_hash[..16]),
            severity: Severity::Info,
            location: ValidationLocation::new(ComponentId::DeterminismEngine),
        });

        // Analyze expected responses for B-MODE compliance
        for (component, action) in &scenario.expected_responses {
            match action {
                RecommendedSystemResponse::RecommendDisableOptimization => {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::SpecificationCompliance,
                        component: *component,
                        description: "Failure response recommends disabling optimization (B-MODE compliant)".to_string(),
                        severity: Severity::Info,
                        location: ValidationLocation::new(*component),
                    });
                }
                RecommendedSystemResponse::RecommendTermination => {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::SpecificationCompliance,
                        component: *component,
                        description: "Failure response recommends termination (B-MODE compliant)".to_string(),
                        severity: Severity::Info,
                        location: ValidationLocation::new(*component),
                    });
                }
                _ => {
                    report.add_finding(SpecificationFinding {
                        finding_type: FindingType::SpecificationCompliance,
                        component: *component,
                        description: format!("Failure response {:?} is B-MODE compliant", action),
                        severity: Severity::Info,
                        location: ValidationLocation::new(*component),
                    });
                }
            }
        }

        // Analyze timing determinism requirements
        if !scenario.determinism_requirements.timing_must_be_deterministic {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::DeterminismViolation,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("TIMING_DETERMINISM_001".to_string()),
                description: "Failure scenario does not require deterministic timing".to_string(),
                remediation_hint: "All failure scenarios must have deterministic timing requirements".to_string(),
            });
        }
        
        report
    }

    fn analyze_audit_log_specification(&self, log_spec: &AuditLogSpec) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze audit log specification for B-MODE compliance
        if !log_spec.hash_chain_required {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("AUDIT_HASH_CHAIN_001".to_string()),
                description: "Audit log specification does not require hash chain".to_string(),
                remediation_hint: "Hash chain is required for audit log integrity".to_string(),
            });
        }

        if !log_spec.monotonic_counter_required {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("AUDIT_MONOTONIC_COUNTER_001".to_string()),
                description: "Audit log specification does not require monotonic counter".to_string(),
                remediation_hint: "Monotonic counter is required for audit log ordering".to_string(),
            });
        }

        if !log_spec.append_only_guarantee {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component: ComponentId::DeterminismEngine,
                rule_id: Some("AUDIT_APPEND_ONLY_001".to_string()),
                description: "Audit log specification does not guarantee append-only behavior".to_string(),
                remediation_hint: "Append-only guarantee is required for audit log immutability".to_string(),
            });
        }

        // Check required fields
        let expected_fields = vec![
            "prev_hash".to_string(),
            "event_type".to_string(),
            "component_id".to_string(),
            "ir_fingerprint".to_string(),
            "failure_scenario_id".to_string(),
            "decision_hash".to_string(),
            "monotonic_counter".to_string(),
        ];

        for field in &expected_fields {
            if !log_spec.required_fields.contains(field) {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: ComponentId::DeterminismEngine,
                    rule_id: Some("AUDIT_REQUIRED_FIELD_001".to_string()),
                    description: format!("Audit log specification missing required field: {}", field),
                    remediation_hint: format!("Add {} to required fields list", field),
                });
            }
        }

        if log_spec.required_fields.len() == expected_fields.len() {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::DeterminismEngine,
                description: "Audit log specification includes all required fields".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::DeterminismEngine),
            });
        }

        report
    }
}