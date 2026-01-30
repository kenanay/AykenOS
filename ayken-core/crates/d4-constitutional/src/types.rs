//! Core Types for D4 Constitutional Framework
//!
//! This module defines the fundamental types used throughout the constitutional framework.
//! All types are designed to be deterministic and support B-MODE purity principles.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Component identifier for constitutional framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ComponentId {
    D1Component,
    D2Component,
    D3Component,
    D4RegisterAllocator,
    JITCompiler,
    LoopOptimizer,
    UnrollOptimizer,
    NativeCache,
    ConstitutionalRuleEngine,
    TemplateSpecRegistry,
    DeterminismEngine,
    FailureMatrix,
    SemanticSpecificationRegistry,
}

/// Severity levels for findings and violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Virtual register identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VirtualRegisterId(pub u32);

/// Physical register identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PhysicalRegisterId(pub u32);

/// Target architecture for compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetArchitecture {
    X86_64,
    ARM64,
    RISCV64,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Aggressive,
}

/// Register binding type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterBinding {
    Physical(PhysicalRegisterId),
    Spilled(SpillLocation),
}

/// Spill location specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpillLocation {
    pub memory_address: u64,
    pub size_bytes: u32,
    pub alignment: u32,
    pub access_pattern: String,
}

/// Allocation decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationDecision {
    pub virtual_register: VirtualRegisterId,
    pub binding: RegisterBinding,
    pub decision_context: AllocationContext,
}

/// Allocation context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationContext {
    pub pressure_level: u32,
    pub optimization_level: OptimizationLevel,
    pub constraints: AllocationConstraints,
    pub performance_requirements: PerformanceRequirements,
}

/// Allocation constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationConstraints {
    pub preferred_registers: Vec<PhysicalRegisterId>,
    pub excluded_registers: Vec<PhysicalRegisterId>,
    pub alignment_requirements: Vec<u32>,
    pub forbidden_registers: Vec<PhysicalRegisterId>,
    pub lifetime_requirements: Vec<String>,
    pub performance_hints: Vec<String>,
}

impl Default for AllocationConstraints {
    fn default() -> Self {
        Self {
            preferred_registers: Vec::new(),
            excluded_registers: Vec::new(),
            alignment_requirements: Vec::new(),
            forbidden_registers: Vec::new(),
            lifetime_requirements: Vec::new(),
            performance_hints: Vec::new(),
        }
    }
}

/// Performance requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    pub max_spill_rate: Option<f64>,
    pub max_register_pressure: Option<u32>,
    pub cache_locality: Option<String>,
}

/// Percentage type for constitutional compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Percentage {
    value: f64,
}

impl Percentage {
    pub fn new(value: f64) -> Result<Self, String> {
        if value >= 0.0 && value <= 100.0 {
            Ok(Self { value })
        } else {
            Err(format!("Percentage must be between 0.0 and 100.0, got {}", value))
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Deterministic logical timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogicalTimestamp(pub u64);

impl LogicalTimestamp {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Deterministic clock for constitutional compliance
#[derive(Debug, Clone)]
pub struct DeterministicClock {
    current_time: LogicalTimestamp,
}

impl DeterministicClock {
    pub fn new() -> Self {
        Self {
            current_time: LogicalTimestamp(0),
        }
    }

    pub fn now(&self) -> LogicalTimestamp {
        self.current_time
    }

    pub fn advance(&mut self) -> LogicalTimestamp {
        self.current_time.0 += 1;
        self.current_time
    }
}

impl Default for DeterministicClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule identifier for constitutional rules
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RuleId(String);

impl RuleId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Create deterministic rule ID from content
    pub fn from_content(content: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        hasher.update(b"d4_constitutional_rule");
        let hash = hex::encode(hasher.finalize());
        Self(format!("rule_{}", &hash[..16]))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Authorization level for constitutional operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationLevel {
    Component,
    System,
    Constitutional,
    Administrative,
}

/// Rule type for constitutional rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    JITAllocationImmutability,
    NativeCacheDeterministicDisable,
    AuthorityHierarchyEnforcement,
}

/// Enforcement level for constitutional rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Component,
    System,
    Constitutional,
    Administrative,
}

/// Operation type for constitutional analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    AllocationRewrite,
    CodeGeneration,
    CodeOptimization,
}

// Note: ContractFinding, ContractViolationType, GatePhase, ReadinessStatus, 
// TransitionFindingType, PermissionType, and FingerprintValidity are now 
// defined in errors::specification_reports to avoid duplication

/// Locked behavior specification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LockedBehavior {
    RegisterAllocationAlgorithm,
    FailureHandlingProcedures,
    AuthorityHierarchy,
    GateTransitionLogic,
    DeterminismRequirements,
}

/// Specification location for error reporting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub context: String,
}

/// Cache operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheOperationType {
    Enable,
    Disable,
    BoundsCheck,
    Access,
}

/// Cache target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheTarget {
    NativeCache,
    InstructionCache,
    DataCache,
}

/// Interaction type between components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    AllocationRequest,
    AllocationDecision,
    AllocationResponse,
    OptimizationHint,
    FailureNotification,
    StateQuery,
    ConstraintDeclaration,
}

/// Proposal operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalOperationType {
    AllocationConstraint,
    OptimizationHint,
    ExecutionHint,
    AllocationRewrite,
}

/// Operation types for constitutional analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    RegisterAllocation {
        virtual_register: VirtualRegisterId,
        physical_register: PhysicalRegisterId,
    },
    RegisterRewrite {
        original: PhysicalRegisterId,
        new: PhysicalRegisterId,
    },
    CacheOperation {
        operation_type: CacheOperationType,
        target: CacheTarget,
    },
    ComponentInteraction {
        source: ComponentId,
        target: ComponentId,
        interaction_type: InteractionType,
    },
}

/// Component interaction payload
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InteractionPayload {
    AllocationRequest {
        virtual_registers: Vec<VirtualRegisterId>,
        constraints: AllocationConstraints,
    },
    AllocationDecision {
        decisions: Vec<AllocationDecision>,
    },
    OptimizationHint {
        hint_type: String,
        parameters: BTreeMap<String, String>,
    },
    FailureNotification {
        failure_type: String,
        context: String,
    },
    StateQuery {
        query_type: String,
        parameters: BTreeMap<String, String>,
    },
}

/// Component interaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInteraction {
    pub source: ComponentId,
    pub target: ComponentId,
    pub interaction_type: InteractionType,
    pub payload: InteractionPayload,
    pub timestamp: LogicalTimestamp,
}

/// Register access type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterAccessType {
    Read,
    Write,
    ReadWrite,
}

/// Register access specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterAccess {
    pub register: PhysicalRegisterId,
    pub access_type: RegisterAccessType,
    pub instruction_address: u64,
    pub bounds_check_required: bool,
}

/// JIT operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JITOperation {
    AllocationRewrite {
        original: AllocationDecision,
        proposed: AllocationDecision,
    },
    CodeGeneration {
        register_accesses: Vec<RegisterAccess>,
        bounds_checking_enabled: bool,
    },
    CodeOptimization {
        optimization_type: String,
        affected_registers: Vec<PhysicalRegisterId>,
    },
}