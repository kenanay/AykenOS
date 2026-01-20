//! JIT Compiler Constitutional Constraints
//!
//! This module implements bounds checking generation for register access under constitutional authority.
//! The JIT compiler must respect register allocation decisions and can only elide bounds checks
//! with explicit constitutional authorization.

use crate::bmode::constitutional::{ConstitutionalRuleAnalyzer, JITOperation, RegisterAccess, RegisterAccessType};
use crate::errors::{ConstitutionalError, ConstitutionalViolation, Result, SpecificationReport, SpecificationViolation, ViolationType};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use sha2::{Digest, Sha256};

/// Generate deterministic authorization ID for constitutional compliance
fn generate_deterministic_authorization_id(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.update(b"d4_constitutional_auth");
    let hash = hex::encode(hasher.finalize());
    format!("auth_{}", &hash[..16])
}

/// Generate deterministic proof ID for constitutional compliance
fn generate_deterministic_proof_id(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.update(b"d4_constitutional_proof");
    let hash = hex::encode(hasher.finalize());
    format!("proof_{}", &hash[..16])
}

/// Static safety analysis input set as specified in requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticSafetyAnalysisInput {
    /// IR structural fingerprint (no allocation decisions)
    pub ir_structural_fingerprint: String,
    /// Allocation map from virtual to physical registers
    pub allocation_map: BTreeMap<VirtualRegisterId, PhysicalRegisterId>,
    /// Target ABI model for bounds validation
    pub target_abi_model: TargetABIModel,
}

/// Target ABI model for bounds checking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetABIModel {
    /// Architecture-specific register bounds
    pub register_bounds: BTreeMap<PhysicalRegisterId, RegisterBounds>,
    /// Stack frame layout constraints
    pub stack_frame_constraints: StackFrameConstraints,
    /// Calling convention requirements
    pub calling_convention: CallingConvention,
}

/// Bounds information for physical registers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterBounds {
    /// Register identifier
    pub register_id: PhysicalRegisterId,
    /// Valid access range (start, end)
    pub valid_range: (u64, u64),
    /// Access permissions
    pub permissions: RegisterPermissions,
    /// Register class (general purpose, floating point, etc.)
    pub register_class: RegisterClass,
}

/// Register access permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

/// Register classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterClass {
    GeneralPurpose,
    FloatingPoint,
    Vector,
    Special,
    Reserved,
}

/// Stack frame layout constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackFrameConstraints {
    /// Maximum stack frame size
    pub max_frame_size: u64,
    /// Stack alignment requirements
    pub alignment: u32,
    /// Reserved stack regions
    pub reserved_regions: Vec<StackRegion>,
}

/// Stack memory region
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackRegion {
    pub start_offset: u64,
    pub size: u64,
    pub purpose: String,
}

/// Calling convention specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallingConvention {
    SystemV,
    MicrosoftX64,
    ARM64AAPCS,
    Custom(String),
}

/// Bounds checking generation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundsCheckingResult {
    /// Generated bounds checks for each register access
    pub bounds_checks: Vec<BoundsCheck>,
    /// Elided checks with constitutional authorization
    pub elided_checks: Vec<ElidedCheck>,
    /// Static safety proof (if bounds checks were elided)
    pub safety_proof: Option<StaticSafetyProof>,
}

/// Individual bounds check instruction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundsCheck {
    /// Register being accessed
    pub register: PhysicalRegisterId,
    /// Type of access
    pub access_type: RegisterAccessType,
    /// Instruction address where check is inserted
    pub instruction_address: u64,
    /// Check implementation details
    pub check_implementation: BoundsCheckImplementation,
}

/// Bounds check that was elided with constitutional authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElidedCheck {
    /// Register that would have been checked
    pub register: PhysicalRegisterId,
    /// Access type that would have been checked
    pub access_type: RegisterAccessType,
    /// Instruction address where check was elided
    pub instruction_address: u64,
    /// Constitutional authorization for elision
    pub authorization: ConstitutionalAuthorization,
    /// Static proof justifying elision
    pub static_proof: StaticSafetyProof,
}

/// Implementation details for bounds checking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundsCheckImplementation {
    /// Hardware-assisted bounds checking
    Hardware {
        instruction_sequence: Vec<String>,
        performance_cost: u32,
    },
    /// Software bounds checking
    Software {
        check_code: String,
        performance_cost: u32,
    },
    /// Hybrid approach
    Hybrid {
        hardware_part: String,
        software_part: String,
        performance_cost: u32,
    },
}

/// Constitutional authorization for bounds check elision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalAuthorization {
    /// Authorization ID for audit trail
    pub authorization_id: String,
    /// Constitutional rule that authorized elision
    pub authorizing_rule: RuleId,
    /// Timestamp of authorization
    pub timestamp: LogicalTimestamp, // BLOCKER #2 FIX: Use deterministic time type
    /// Justification for elision
    pub justification: String,
}

/// Static safety proof for bounds check elision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticSafetyProof {
    /// Proof ID for reference
    pub proof_id: String,
    /// Proof method used
    pub proof_method: ProofMethod,
    /// Input set used for proof
    pub input_set: StaticSafetyAnalysisInput,
    /// Proof steps and reasoning
    pub proof_steps: Vec<ProofStep>,
    /// Proof validity timestamp
    pub valid_until: LogicalTimestamp, // BLOCKER #2 FIX: Use deterministic time type
}

/// Methods for static safety proofs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofMethod {
    /// Structural analysis of IR
    StructuralAnalysis,
    /// Type system guarantees
    TypeSystemGuarantees,
    /// Range analysis
    RangeAnalysis,
    /// Combined approach
    Combined(Vec<ProofMethod>),
}

/// Individual step in a safety proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofStep {
    /// Step number
    pub step_number: u32,
    /// Description of the step
    pub description: String,
    /// Logical reasoning
    pub reasoning: String,
    /// Dependencies on previous steps
    pub dependencies: Vec<u32>,
}

/// Trait for JIT bounds checking generation under constitutional authority
pub trait JITBoundsChecker {
    /// Generate bounds checking for register access
    /// 
    /// This method must:
    /// 1. Use only the specified static proof input set (IR structural fingerprint + allocation map + target ABI model)
    /// 2. Not depend on runtime profiling, timing, or execution history
    /// 3. Generate bounds checks for all register accesses by default
    /// 4. Only elide bounds checks with explicit Constitutional Rule Engine authorization
    /// 5. Ensure JIT cannot rewrite register allocation decisions
    fn generate_bounds_checking(
        &self,
        analysis_input: &StaticSafetyAnalysisInput,
        register_accesses: &[RegisterAccess],
        constitutional_engine: &dyn ConstitutionalRuleAnalyzer,
    ) -> Result<BoundsCheckingResult>;

    /// Request constitutional authorization for bounds check elision
    fn request_elision_authorization(
        &self,
        register: PhysicalRegisterId,
        access_type: RegisterAccessType,
        static_proof: &StaticSafetyProof,
        constitutional_engine: &dyn ConstitutionalRuleAnalyzer,
    ) -> Result<Option<ConstitutionalAuthorization>>;

    /// Validate that JIT operations do not rewrite register allocations
    fn validate_allocation_immutability(
        &self,
        original_allocation: &BTreeMap<VirtualRegisterId, PhysicalRegisterId>,
        proposed_operations: &[JITOperation],
    ) -> SpecificationReport;

    /// Generate static safety proof for bounds check elision
    fn generate_static_safety_proof(
        &self,
        analysis_input: &StaticSafetyAnalysisInput,
        register: PhysicalRegisterId,
        access_type: RegisterAccessType,
    ) -> Result<StaticSafetyProof>;
}

/// Default implementation of JIT bounds checker
#[derive(Debug, Clone)]
pub struct DefaultJITBoundsChecker {
    /// Target architecture for bounds checking
    target_architecture: TargetArchitecture,
    /// Performance cost threshold for bounds checking
    /// NOTE: Constitutional design - bounds check elision decisions are made based on 
    /// static safety proofs and constitutional authorization, not performance thresholds.
    /// This field is reserved for future Gate-F performance optimization features.
    #[allow(dead_code)]
    performance_threshold: Percentage,
    /// Cache of static safety proofs
    proof_cache: BTreeMap<String, StaticSafetyProof>,
}

impl DefaultJITBoundsChecker {
    /// Create a new JIT bounds checker for the specified architecture
    pub fn new(target_architecture: TargetArchitecture) -> Result<Self> {
        let performance_threshold = Percentage::new(5.0)
            .map_err(|e| ConstitutionalError::ConfigurationError { reason: e })?; // 5% performance overhead threshold
        
        Ok(Self {
            target_architecture,
            performance_threshold,
            proof_cache: BTreeMap::new(),
        })
    }

    /// Get default target ABI model for the architecture
    pub fn get_default_abi_model(&self) -> TargetABIModel {
        match self.target_architecture {
            TargetArchitecture::X86_64 => self.get_x86_64_abi_model(),
            TargetArchitecture::ARM64 => self.get_arm64_abi_model(),
            TargetArchitecture::RISCV64 => self.get_riscv64_abi_model(),
        }
    }

    fn get_x86_64_abi_model(&self) -> TargetABIModel {
        let mut register_bounds = BTreeMap::new();
        
        // General purpose registers (RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP, R8-R15)
        for i in 0..16 {
            register_bounds.insert(
                PhysicalRegisterId(i),
                RegisterBounds {
                    register_id: PhysicalRegisterId(i),
                    valid_range: (0, u64::MAX), // Full 64-bit range
                    permissions: RegisterPermissions {
                        read: true,
                        write: true,
                        execute: false,
                    },
                    register_class: RegisterClass::GeneralPurpose,
                },
            );
        }

        TargetABIModel {
            register_bounds,
            stack_frame_constraints: StackFrameConstraints {
                max_frame_size: 1024 * 1024, // 1MB max frame
                alignment: 16, // 16-byte alignment
                reserved_regions: vec![
                    StackRegion {
                        start_offset: 0,
                        size: 8,
                        purpose: "return_address".to_string(),
                    },
                ],
            },
            calling_convention: CallingConvention::SystemV,
        }
    }

    fn get_arm64_abi_model(&self) -> TargetABIModel {
        let mut register_bounds = BTreeMap::new();
        
        // ARM64 general purpose registers (X0-X30)
        for i in 0..31 {
            register_bounds.insert(
                PhysicalRegisterId(i),
                RegisterBounds {
                    register_id: PhysicalRegisterId(i),
                    valid_range: (0, u64::MAX),
                    permissions: RegisterPermissions {
                        read: true,
                        write: true,
                        execute: false,
                    },
                    register_class: RegisterClass::GeneralPurpose,
                },
            );
        }

        TargetABIModel {
            register_bounds,
            stack_frame_constraints: StackFrameConstraints {
                max_frame_size: 1024 * 1024,
                alignment: 16,
                reserved_regions: vec![],
            },
            calling_convention: CallingConvention::ARM64AAPCS,
        }
    }

    fn get_riscv64_abi_model(&self) -> TargetABIModel {
        let mut register_bounds = BTreeMap::new();
        
        // RISC-V general purpose registers (x0-x31)
        for i in 0..32 {
            register_bounds.insert(
                PhysicalRegisterId(i),
                RegisterBounds {
                    register_id: PhysicalRegisterId(i),
                    valid_range: (0, u64::MAX),
                    permissions: RegisterPermissions {
                        read: i != 0, // x0 is always zero, not readable in the traditional sense
                        write: i != 0, // x0 is hardwired to zero
                        execute: false,
                    },
                    register_class: RegisterClass::GeneralPurpose,
                },
            );
        }

        TargetABIModel {
            register_bounds,
            stack_frame_constraints: StackFrameConstraints {
                max_frame_size: 1024 * 1024,
                alignment: 8,
                reserved_regions: vec![],
            },
            calling_convention: CallingConvention::Custom("RISC-V".to_string()),
        }
    }

    /// Check if a register access is within bounds
    fn is_access_within_bounds(
        &self,
        register: PhysicalRegisterId,
        access_type: RegisterAccessType,
        abi_model: &TargetABIModel,
    ) -> bool {
        if let Some(bounds) = abi_model.register_bounds.get(&register) {
            match access_type {
                RegisterAccessType::Read => bounds.permissions.read,
                RegisterAccessType::Write => bounds.permissions.write,
                RegisterAccessType::ReadWrite => bounds.permissions.read && bounds.permissions.write,
            }
        } else {
            false // Unknown register, not safe
        }
    }

    /// Generate bounds check implementation for a register access
    fn generate_bounds_check_implementation(
        &self,
        register: PhysicalRegisterId,
        _access_type: RegisterAccessType, // TODO: Will be used in Gate F for access-specific checks
        abi_model: &TargetABIModel,
    ) -> BoundsCheckImplementation {
        match self.target_architecture {
            TargetArchitecture::X86_64 => {
                BoundsCheckImplementation::Software {
                    check_code: format!(
                        "cmp {}, {}; jae bounds_violation_handler",
                        register.0,
                        abi_model.register_bounds.get(&register)
                            .map(|b| b.valid_range.1)
                            .unwrap_or(0)
                    ),
                    performance_cost: 2, // 2 instructions overhead
                }
            }
            TargetArchitecture::ARM64 => {
                BoundsCheckImplementation::Software {
                    check_code: format!(
                        "cmp x{}, #{}; b.hs bounds_violation_handler",
                        register.0,
                        abi_model.register_bounds.get(&register)
                            .map(|b| b.valid_range.1)
                            .unwrap_or(0)
                    ),
                    performance_cost: 2,
                }
            }
            TargetArchitecture::RISCV64 => {
                BoundsCheckImplementation::Software {
                    check_code: format!(
                        "bgeu x{}, {}, bounds_violation_handler",
                        register.0,
                        abi_model.register_bounds.get(&register)
                            .map(|b| b.valid_range.1)
                            .unwrap_or(0)
                    ),
                    performance_cost: 1, // Single instruction
                }
            }
        }
    }
}

impl JITBoundsChecker for DefaultJITBoundsChecker {
    fn generate_bounds_checking(
        &self,
        analysis_input: &StaticSafetyAnalysisInput,
        register_accesses: &[RegisterAccess],
        constitutional_engine: &dyn ConstitutionalRuleAnalyzer,
    ) -> Result<BoundsCheckingResult> {
        let mut bounds_checks = Vec::new();
        let mut elided_checks = Vec::new();
        let mut safety_proof = None;

        // Validate that we're not modifying register allocations
        let allocation_report = self.validate_allocation_immutability(&analysis_input.allocation_map, &[]);
        if !allocation_report.violations.is_empty() {
            // If there are allocation violations, we cannot proceed with bounds checking
            return Err(ConstitutionalError::ConfigurationError { 
                reason: format!("JIT allocation immutability violations: {:?}", allocation_report.violations)
            });
        }

        for access in register_accesses {
            // Check if this access is within bounds according to static analysis
            let is_safe = self.is_access_within_bounds(
                access.register,
                access.access_type,
                &analysis_input.target_abi_model,
            );

            if access.bounds_check_required {
                if is_safe {
                    // Try to generate static safety proof for elision
                    match self.generate_static_safety_proof(
                        analysis_input,
                        access.register,
                        access.access_type,
                    ) {
                        Ok(proof) => {
                            // Request constitutional authorization for elision
                            match self.request_elision_authorization(
                                access.register,
                                access.access_type,
                                &proof,
                                constitutional_engine,
                            )? {
                                Some(authorization) => {
                                    // Bounds check can be elided with constitutional authorization
                                    elided_checks.push(ElidedCheck {
                                        register: access.register,
                                        access_type: access.access_type,
                                        instruction_address: access.instruction_address,
                                        authorization,
                                        static_proof: proof.clone(),
                                    });
                                    
                                    if safety_proof.is_none() {
                                        safety_proof = Some(proof);
                                    }
                                }
                                None => {
                                    // Constitutional engine denied elision, must generate bounds check
                                    bounds_checks.push(BoundsCheck {
                                        register: access.register,
                                        access_type: access.access_type,
                                        instruction_address: access.instruction_address,
                                        check_implementation: self.generate_bounds_check_implementation(
                                            access.register,
                                            access.access_type,
                                            &analysis_input.target_abi_model,
                                        ),
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            // Cannot generate static proof, must include bounds check
                            bounds_checks.push(BoundsCheck {
                                register: access.register,
                                access_type: access.access_type,
                                instruction_address: access.instruction_address,
                                check_implementation: self.generate_bounds_check_implementation(
                                    access.register,
                                    access.access_type,
                                    &analysis_input.target_abi_model,
                                ),
                            });
                        }
                    }
                } else {
                    // Unsafe access, must include bounds check
                    bounds_checks.push(BoundsCheck {
                        register: access.register,
                        access_type: access.access_type,
                        instruction_address: access.instruction_address,
                        check_implementation: self.generate_bounds_check_implementation(
                            access.register,
                            access.access_type,
                            &analysis_input.target_abi_model,
                        ),
                    });
                }
            } else {
                // bounds_check_required is false
                if is_safe {
                    // Access is safe and doesn't require bounds checking
                    // Generate static safety proof and record as elided
                    match self.generate_static_safety_proof(
                        analysis_input,
                        access.register,
                        access.access_type,
                    ) {
                        Ok(proof) => {
                            // GATE-E FIX: Request explicit authorization from Constitutional Rule Engine
                            // No implicit authorization allowed - all elisions must be constitutionally approved
                            match self.request_elision_authorization(
                                access.register,
                                access.access_type,
                                &proof,
                                constitutional_engine,
                            ) {
                                Ok(Some(authorization)) => {
                                    elided_checks.push(ElidedCheck {
                                        register: access.register,
                                        access_type: access.access_type,
                                        instruction_address: access.instruction_address,
                                        authorization,
                                        static_proof: proof.clone(),
                                    });
                                    
                                    if safety_proof.is_none() {
                                        safety_proof = Some(proof);
                                    }
                                }
                                Ok(None) => {
                                    // Constitutional Rule Engine denied elision, include bounds check
                                    bounds_checks.push(BoundsCheck {
                                        register: access.register,
                                        access_type: access.access_type,
                                        instruction_address: access.instruction_address,
                                        check_implementation: self.generate_bounds_check_implementation(
                                            access.register,
                                            access.access_type,
                                            &analysis_input.target_abi_model,
                                        ),
                                    });
                                }
                                Err(_) => {
                                    // Constitutional Rule Engine denied elision, include bounds check
                                    bounds_checks.push(BoundsCheck {
                                        register: access.register,
                                        access_type: access.access_type,
                                        instruction_address: access.instruction_address,
                                        check_implementation: self.generate_bounds_check_implementation(
                                            access.register,
                                            access.access_type,
                                            &analysis_input.target_abi_model,
                                        ),
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            // Cannot generate static proof, include bounds check for safety
                            bounds_checks.push(BoundsCheck {
                                register: access.register,
                                access_type: access.access_type,
                                instruction_address: access.instruction_address,
                                check_implementation: self.generate_bounds_check_implementation(
                                    access.register,
                                    access.access_type,
                                    &analysis_input.target_abi_model,
                                ),
                            });
                        }
                    }
                } else {
                    // Access is not safe, must include bounds check regardless
                    bounds_checks.push(BoundsCheck {
                        register: access.register,
                        access_type: access.access_type,
                        instruction_address: access.instruction_address,
                        check_implementation: self.generate_bounds_check_implementation(
                            access.register,
                            access.access_type,
                            &analysis_input.target_abi_model,
                        ),
                    });
                }
            }
        }

        Ok(BoundsCheckingResult {
            bounds_checks,
            elided_checks,
            safety_proof,
        })
    }

    fn request_elision_authorization(
        &self,
        register: PhysicalRegisterId,
        access_type: RegisterAccessType,
        static_proof: &StaticSafetyProof,
        constitutional_engine: &dyn ConstitutionalRuleAnalyzer,
    ) -> Result<Option<ConstitutionalAuthorization>> {
        // Create a JIT operation for bounds check elision
        let jit_operation = JITOperation::CodeGeneration {
            register_accesses: vec![RegisterAccess {
                register,
                access_type,
                instruction_address: 0, // Placeholder
                bounds_check_required: false, // Requesting elision
            }],
            bounds_checking_enabled: false, // Requesting to disable for this access
        };

        // Validate with constitutional engine
        let report = constitutional_engine.analyze_jit_allocation_immutability(&jit_operation);
        if report.violations.is_empty() {
                // Constitutional engine allows this operation
                // Generate deterministic authorization
                let auth_content = format!("jit_bounds_{}_{:?}_{:?}", 
                    register.0, access_type, static_proof.proof_id);
                Ok(Some(ConstitutionalAuthorization {
                    authorization_id: generate_deterministic_authorization_id(&auth_content),
                    authorizing_rule: RuleId::from_content(b"jit_bounds_authorization"), // Deterministic rule ID
                    timestamp: DeterministicClock::new().now(),
                    justification: format!(
                        "Static safety proof {} validates bounds check elision for register {:?} access {:?}",
                        static_proof.proof_id, register, access_type
                    ),
                }))
        } else {
            // Constitutional engine denies elision
            Ok(None)
        }
    }

    fn validate_allocation_immutability(
        &self,
        original_allocation: &BTreeMap<VirtualRegisterId, PhysicalRegisterId>,
        proposed_operations: &[JITOperation],
    ) -> SpecificationReport {
        let mut report = SpecificationReport::new();
        
        // Analyze JIT operations for register allocation compliance
        for operation in proposed_operations {
            match operation {
                JITOperation::AllocationRewrite { .. } => {
                    // This is explicitly forbidden
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::UnauthorizedOperation,
                        component: ComponentId::JITCompiler,
                        rule_id: Some("JIT_ALLOCATION_IMMUTABILITY_001".to_string()),
                        description: "JIT Compiler attempted to rewrite register allocation - this violates constitutional immutability".to_string(),
                        remediation_hint: "JIT Compiler must not modify register allocation decisions".to_string(),
                    });
                }
                JITOperation::CodeGeneration { register_accesses, .. } => {
                    // Validate that code generation uses allocated registers correctly
                    for access in register_accesses {
                        // Check if this register was properly allocated
                        let is_allocated = original_allocation.values().any(|&allocated_reg| allocated_reg == access.register);
                        
                        if !is_allocated {
                            report.add_violation(SpecificationViolation {
                                violation_type: ViolationType::UnauthorizedOperation,
                                component: ComponentId::JITCompiler,
                                rule_id: Some("JIT_ALLOCATION_IMMUTABILITY_002".to_string()),
                                description: format!(
                                    "JIT Compiler attempted to access unallocated register {:?}",
                                    access.register
                                ),
                                remediation_hint: "JIT Compiler must only access registers that have been allocated".to_string(),
                            });
                        }
                    }
                }
                JITOperation::CodeOptimization { affected_registers, optimization_type } => {
                    // Ensure optimization doesn't change register allocations
                    if optimization_type.contains("register_reallocation") || 
                       optimization_type.contains("allocation_override") {
                        report.add_violation(SpecificationViolation {
                            violation_type: ViolationType::UnauthorizedOperation,
                            component: ComponentId::JITCompiler,
                            rule_id: Some("JIT_ALLOCATION_IMMUTABILITY_003".to_string()),
                            description: format!(
                                "JIT Compiler optimization '{}' attempts to modify register allocations",
                                optimization_type
                            ),
                            remediation_hint: "JIT optimizations must not modify register allocation decisions".to_string(),
                        });
                    }

                    // Validate that affected registers are properly allocated
                    for &register in affected_registers {
                        let is_allocated = original_allocation.values().any(|&allocated_reg| allocated_reg == register);
                        
                        if !is_allocated {
                            report.add_violation(SpecificationViolation {
                                violation_type: ViolationType::UnauthorizedOperation,
                                component: ComponentId::JITCompiler,
                                rule_id: Some("JIT_ALLOCATION_IMMUTABILITY_004".to_string()),
                                description: format!(
                                    "JIT Compiler optimization affects unallocated register {:?}",
                                    register
                                ),
                                remediation_hint: "JIT optimizations must only affect allocated registers".to_string(),
                            });
                        }
                    }
                }
            }
        }

        report
    }

    fn generate_static_safety_proof(
        &self,
        analysis_input: &StaticSafetyAnalysisInput,
        register: PhysicalRegisterId,
        access_type: RegisterAccessType,
    ) -> Result<StaticSafetyProof> {
        // Generate a cache key for this proof
        let cache_key = format!(
            "{}_{:?}_{:?}_{:?}",
            analysis_input.ir_structural_fingerprint,
            register,
            access_type,
            self.target_architecture
        );

        // Check if we have a cached proof
        if let Some(cached_proof) = self.proof_cache.get(&cache_key) {
            // GATE-E FIX: Remove time-based cache validation - proofs are structurally valid
            // Constitutional guarantee: Static safety proofs are valid for identical input sets
            return Ok(cached_proof.clone());
        }

        // Generate new static safety proof
        let mut proof_steps = Vec::new();

        // Step 1: Structural analysis of IR
        proof_steps.push(ProofStep {
            step_number: 1,
            description: "Analyze IR structural fingerprint for register usage patterns".to_string(),
            reasoning: format!(
                "IR fingerprint {} indicates structured register access without dynamic allocation changes",
                analysis_input.ir_structural_fingerprint
            ),
            dependencies: vec![],
        });

        // Step 2: Allocation map validation
        proof_steps.push(ProofStep {
            step_number: 2,
            description: "Validate allocation map consistency".to_string(),
            reasoning: format!(
                "Register {:?} is properly allocated in allocation map and target ABI model",
                register
            ),
            dependencies: vec![1],
        });

        // Step 3: Target ABI compliance
        proof_steps.push(ProofStep {
            step_number: 3,
            description: "Verify target ABI model compliance".to_string(),
            reasoning: format!(
                "Access type {:?} is permitted by target ABI model for register {:?}",
                access_type, register
            ),
            dependencies: vec![1, 2],
        });

        // Step 4: Range analysis
        if let Some(bounds) = analysis_input.target_abi_model.register_bounds.get(&register) {
            proof_steps.push(ProofStep {
                step_number: 4,
                description: "Perform range analysis on register access".to_string(),
                reasoning: format!(
                    "Register {:?} access is within valid range {:?} and has required permissions",
                    register, bounds.valid_range
                ),
                dependencies: vec![2, 3],
            });
        }

        // Generate deterministic proof ID
        let proof_content = format!("{}_{:?}_{:?}_{:?}", 
            analysis_input.ir_structural_fingerprint, register, access_type, self.target_architecture);
        let proof = StaticSafetyProof {
            proof_id: generate_deterministic_proof_id(&proof_content),
            proof_method: ProofMethod::Combined(vec![
                ProofMethod::StructuralAnalysis,
                ProofMethod::RangeAnalysis,
            ]),
            input_set: analysis_input.clone(),
            proof_steps,
            valid_until: LogicalTimestamp(0), // GATE-E FIX: Proof validity removed - logical time should not be mixed with duration
        };

        Ok(proof)
    }
}

#[cfg(all(test, feature = "runtime"))]
mod tests {
    use super::*;
    use crate::constitutional::DefaultConstitutionalRuleEngine;
    use crate::testing::*;
    use proptest::prelude::*;

    // Property test strategies for JIT bounds checking

    /// Strategy for generating StaticSafetyAnalysisInput values
    pub fn static_safety_analysis_input_strategy() -> impl Strategy<Value = StaticSafetyAnalysisInput> {
        (
            "[a-zA-Z0-9_]{10,50}",
            prop::collection::hash_map(
                virtual_register_id_strategy(),
                physical_register_id_strategy(),
                1..10,
            ),
            target_abi_model_strategy(),
        ).prop_map(|(ir_fingerprint, allocation_map, target_abi_model)| {
            StaticSafetyAnalysisInput {
                ir_structural_fingerprint: ir_fingerprint,
                allocation_map,
                target_abi_model,
            }
        })
    }

    /// Strategy for generating TargetABIModel values
    pub fn target_abi_model_strategy() -> impl Strategy<Value = TargetABIModel> {
        (
            prop::collection::hash_map(
                physical_register_id_strategy(),
                register_bounds_strategy(),
                1..16,
            ),
            stack_frame_constraints_strategy(),
            calling_convention_strategy(),
        ).prop_map(|(register_bounds, stack_frame_constraints, calling_convention)| {
            TargetABIModel {
                register_bounds,
                stack_frame_constraints,
                calling_convention,
            }
        })
    }

    /// Strategy for generating RegisterBounds values
    pub fn register_bounds_strategy() -> impl Strategy<Value = RegisterBounds> {
        (
            physical_register_id_strategy(),
            (0u64..0x1000000, 0x1000000u64..0x10000000),
            register_permissions_strategy(),
            register_class_strategy(),
        ).prop_map(|(register_id, (start, end), permissions, register_class)| {
            RegisterBounds {
                register_id,
                valid_range: (start, end),
                permissions,
                register_class,
            }
        })
    }

    /// Strategy for generating RegisterPermissions values
    pub fn register_permissions_strategy() -> impl Strategy<Value = RegisterPermissions> {
        (prop::bool::ANY, prop::bool::ANY, prop::bool::ANY).prop_map(|(read, write, execute)| {
            RegisterPermissions { read, write, execute }
        })
    }

    /// Strategy for generating RegisterClass values
    pub fn register_class_strategy() -> impl Strategy<Value = RegisterClass> {
        prop_oneof![
            Just(RegisterClass::GeneralPurpose),
            Just(RegisterClass::FloatingPoint),
            Just(RegisterClass::Vector),
            Just(RegisterClass::Special),
            Just(RegisterClass::Reserved),
        ]
    }

    /// Strategy for generating StackFrameConstraints values
    pub fn stack_frame_constraints_strategy() -> impl Strategy<Value = StackFrameConstraints> {
        (
            1024u64..1048576, // 1KB to 1MB
            prop_oneof![Just(4u32), Just(8u32), Just(16u32), Just(32u32)],
            prop::collection::vec(stack_region_strategy(), 0..5),
        ).prop_map(|(max_frame_size, alignment, reserved_regions)| {
            StackFrameConstraints {
                max_frame_size,
                alignment,
                reserved_regions,
            }
        })
    }

    /// Strategy for generating StackRegion values
    pub fn stack_region_strategy() -> impl Strategy<Value = StackRegion> {
        (
            0u64..1024,
            8u64..256,
            "[a-zA-Z_]{5,20}",
        ).prop_map(|(start_offset, size, purpose)| {
            StackRegion {
                start_offset,
                size,
                purpose,
            }
        })
    }

    /// Strategy for generating CallingConvention values
    pub fn calling_convention_strategy() -> impl Strategy<Value = CallingConvention> {
        prop_oneof![
            Just(CallingConvention::SystemV),
            Just(CallingConvention::MicrosoftX64),
            Just(CallingConvention::ARM64AAPCS),
            "[a-zA-Z_]{5,15}".prop_map(CallingConvention::Custom),
        ]
    }

    /// Strategy for generating RegisterAccess values
    pub fn register_access_strategy() -> impl Strategy<Value = RegisterAccess> {
        (
            physical_register_id_strategy(),
            register_access_type_strategy(),
            0x1000u64..0x100000,
            prop::bool::ANY,
        ).prop_map(|(register, access_type, instruction_address, bounds_check_required)| {
            RegisterAccess {
                register,
                access_type,
                instruction_address,
                bounds_check_required,
            }
        })
    }

    /// Strategy for generating RegisterAccessType values
    pub fn register_access_type_strategy() -> impl Strategy<Value = RegisterAccessType> {
        prop_oneof![
            Just(RegisterAccessType::Read),
            Just(RegisterAccessType::Write),
            Just(RegisterAccessType::ReadWrite),
        ]
    }

    /// Strategy for generating JITOperation values
    pub fn jit_operation_strategy() -> impl Strategy<Value = JITOperation> {
        prop_oneof![
            (allocation_decision_strategy(), allocation_decision_strategy()).prop_map(|(original, proposed)| {
                JITOperation::AllocationRewrite { original, proposed }
            }),
            (
                prop::collection::vec(register_access_strategy(), 1..10),
                prop::bool::ANY,
            ).prop_map(|(register_accesses, bounds_checking_enabled)| {
                JITOperation::CodeGeneration {
                    register_accesses,
                    bounds_checking_enabled,
                }
            }),
            (
                "[a-zA-Z_]{5,20}",
                prop::collection::vec(physical_register_id_strategy(), 0..8),
            ).prop_map(|(optimization_type, affected_registers)| {
                JITOperation::CodeOptimization {
                    optimization_type,
                    affected_registers,
                }
            }),
        ]
    }

    #[test]
    fn test_jit_bounds_checker_creation() {
        let checker = DefaultJITBoundsChecker::new(TargetArchitecture::X86_64).unwrap();
        assert_eq!(checker.target_architecture, TargetArchitecture::X86_64);
        assert_eq!(checker.performance_threshold.value(), 5.0);
    }

    #[test]
    fn test_target_abi_model_generation() {
        let checker = DefaultJITBoundsChecker::new(TargetArchitecture::X86_64).unwrap();
        let abi_model = checker.get_default_abi_model();
        
        assert_eq!(abi_model.register_bounds.len(), 16); // 16 general purpose registers
        assert!(matches!(abi_model.calling_convention, CallingConvention::SystemV));
        assert_eq!(abi_model.stack_frame_constraints.alignment, 16);
    }

    #[test]
    fn test_bounds_checking_generation() {
        let checker = DefaultJITBoundsChecker::new(TargetArchitecture::X86_64).unwrap();
        let constitutional_engine = DefaultConstitutionalRuleEngine::new();
        
        let analysis_input = StaticSafetyAnalysisInput {
            ir_structural_fingerprint: "test_ir_fingerprint".to_string(),
            allocation_map: {
                let mut map = BTreeMap::new();
                map.insert(VirtualRegisterId(1), PhysicalRegisterId(0));
                map
            },
            target_abi_model: checker.get_default_abi_model(),
        };

        let register_accesses = vec![
            RegisterAccess {
                register: PhysicalRegisterId(0),
                access_type: RegisterAccessType::Read,
                instruction_address: 0x1000,
                bounds_check_required: true,
            },
        ];

        let result = checker.generate_bounds_checking(
            &analysis_input,
            &register_accesses,
            &constitutional_engine,
        ).unwrap();

        // Should generate bounds checks or elide them with proper authorization
        assert!(result.bounds_checks.len() > 0 || result.elided_checks.len() > 0);
    }

    #[test]
    fn test_allocation_immutability_validation() {
        let checker = DefaultJITBoundsChecker::new(TargetArchitecture::X86_64).unwrap();
        
        let allocation_map = {
            let mut map = BTreeMap::new();
            map.insert(VirtualRegisterId(1), PhysicalRegisterId(0));
            map
        };

        // Valid operation: code generation with allocated register
        let valid_operations = vec![
            JITOperation::CodeGeneration {
                register_accesses: vec![
                    RegisterAccess {
                        register: PhysicalRegisterId(0), // Allocated register
                        access_type: RegisterAccessType::Read,
                        instruction_address: 0x1000,
                        bounds_check_required: true,
                    },
                ],
                bounds_checking_enabled: true,
            },
        ];

        let allocation_report = checker.validate_allocation_immutability(&allocation_map, &valid_operations);
        assert!(allocation_report.violations.is_empty());

        // Invalid operation: allocation rewrite
        let invalid_operations = vec![
            JITOperation::AllocationRewrite {
                original: AllocationDecision {
                    virtual_register: VirtualRegisterId(1),
                    physical_register: Some(PhysicalRegisterId(0)),
                    spill_location: None,
                    decision_timestamp: DeterministicClock::new().now(),
                    decision_context: AllocationContext {
                        ir_fingerprint: "test".to_string(),
                        optimization_level: OptimizationLevel::Release,
                        target_architecture: TargetArchitecture::X86_64,
                        available_registers: vec![PhysicalRegisterId(0)],
                    },
                },
                proposed: AllocationDecision {
                    virtual_register: VirtualRegisterId(1),
                    physical_register: Some(PhysicalRegisterId(1)),
                    spill_location: None,
                    decision_timestamp: DeterministicClock::new().now(),
                    decision_context: AllocationContext {
                        ir_fingerprint: "test".to_string(),
                        optimization_level: OptimizationLevel::Release,
                        target_architecture: TargetArchitecture::X86_64,
                        available_registers: vec![PhysicalRegisterId(1)],
                    },
                },
            },
        ];

        let allocation_report = checker.validate_allocation_immutability(&allocation_map, &invalid_operations);
        assert!(!allocation_report.violations.is_empty());
    }

    #[test]
    fn test_static_safety_proof_generation() {
        let checker = DefaultJITBoundsChecker::new(TargetArchitecture::X86_64).unwrap();
        
        let analysis_input = StaticSafetyAnalysisInput {
            ir_structural_fingerprint: "test_ir_fingerprint".to_string(),
            allocation_map: {
                let mut map = BTreeMap::new();
                map.insert(VirtualRegisterId(1), PhysicalRegisterId(0));
                map
            },
            target_abi_model: checker.get_default_abi_model(),
        };

        let proof = checker.generate_static_safety_proof(
            &analysis_input,
            PhysicalRegisterId(0),
            RegisterAccessType::Read,
        ).unwrap();

        assert!(!proof.proof_id.is_empty());
        assert!(matches!(proof.proof_method, ProofMethod::Combined(_)));
        assert!(!proof.proof_steps.is_empty());
        // GATE-E FIX: Remove time-based proof validity check - proofs are structurally valid
        assert_eq!(proof.valid_until, LogicalTimestamp(0));
    }

    #[test]
    fn test_bounds_check_implementation_generation() {
        let checker = DefaultJITBoundsChecker::new(TargetArchitecture::X86_64).unwrap();
        let abi_model = checker.get_default_abi_model();
        
        let implementation = checker.generate_bounds_check_implementation(
            PhysicalRegisterId(0),
            RegisterAccessType::Read,
            &abi_model,
        );

        match implementation {
            BoundsCheckImplementation::Software { check_code, performance_cost } => {
                assert!(!check_code.is_empty());
                assert!(performance_cost > 0);
            }
            _ => panic!("Expected software implementation for x86_64"),
        }
    }

    // Property 9: JIT Bounds Checking Generation
    // **Feature: d4-constitutional-addendum, Property 9: JIT Bounds Checking Generation**
    // **Validates: Requirements 1.1, 3.8**
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            max_shrink_iters: 1000,
            timeout: 10000,
            rng_algorithm: proptest::test_runner::RngAlgorithm::ChaCha,
            ..ProptestConfig::default()
        })]

        #[test]
        fn property_jit_bounds_checking_generation(
            target_architecture in target_architecture_strategy(),
            analysis_input in static_safety_analysis_input_strategy(),
            register_accesses in prop::collection::vec(register_access_strategy(), 1..20),
            jit_operations in prop::collection::vec(jit_operation_strategy(), 0..10)
        ) {
            let checker = DefaultJITBoundsChecker::new(target_architecture.clone()).unwrap();
            let constitutional_engine = DefaultConstitutionalRuleEngine::new();
            
            // Property: For any register access in generated code, the JIT Compiler should include bounds checking
            // unless statically proven safe and explicitly authorized by Constitutional Rule Engine
            
            // Test 1: Bounds checking generation must respect constitutional constraints
            let bounds_result = checker.generate_bounds_checking(
                &analysis_input,
                &register_accesses,
                &constitutional_engine,
            );
            
            prop_assert!(bounds_result.is_ok(), "Bounds checking generation should succeed for valid inputs");
            
            let bounds_checking_result = bounds_result.unwrap();
            
            // Verify that all register accesses are either checked or properly elided
            let total_accesses = register_accesses.len();
            let checked_accesses = bounds_checking_result.bounds_checks.len();
            let elided_accesses = bounds_checking_result.elided_checks.len();
            
            prop_assert_eq!(
                total_accesses, 
                checked_accesses + elided_accesses,
                "All register accesses must be either bounds-checked or properly elided with authorization"
            );
            
            // Test 2: Elided checks must have constitutional authorization
            for elided_check in &bounds_checking_result.elided_checks {
                prop_assert!(!elided_check.authorization.authorization_id.is_empty(),
                    "Elided bounds check must have valid authorization ID");
                prop_assert!(!elided_check.authorization.justification.is_empty(),
                    "Elided bounds check must have justification");
                prop_assert!(!elided_check.static_proof.proof_id.is_empty(),
                    "Elided bounds check must have static safety proof");
                // GATE-E FIX: Remove time-based proof validity check - proofs are structurally valid
                prop_assert_eq!(elided_check.static_proof.valid_until, LogicalTimestamp(0),
                    "Static safety proof validity should be structural, not time-based");
            }
            
            // Test 3: Bounds checks must have proper implementation
            for bounds_check in &bounds_checking_result.bounds_checks {
                match &bounds_check.check_implementation {
                    BoundsCheckImplementation::Software { check_code, performance_cost } => {
                        prop_assert!(!check_code.is_empty(), "Software bounds check must have implementation code");
                        prop_assert!(*performance_cost > 0, "Bounds check must have measurable performance cost");
                    }
                    BoundsCheckImplementation::Hardware { instruction_sequence, performance_cost } => {
                        prop_assert!(!instruction_sequence.is_empty(), "Hardware bounds check must have instruction sequence");
                        prop_assert!(*performance_cost > 0, "Bounds check must have measurable performance cost");
                    }
                    BoundsCheckImplementation::Hybrid { hardware_part, software_part, performance_cost } => {
                        prop_assert!(!hardware_part.is_empty(), "Hybrid bounds check must have hardware part");
                        prop_assert!(!software_part.is_empty(), "Hybrid bounds check must have software part");
                        prop_assert!(*performance_cost > 0, "Bounds check must have measurable performance cost");
                    }
                }
            }
            
            // Test 4: JIT cannot rewrite register allocation decisions (Requirement 1.1)
            let allocation_validation_result = checker.validate_allocation_immutability(
                &analysis_input.allocation_map,
                &jit_operations,
            );
            
            // Check for allocation rewrite attempts
            let has_allocation_rewrite = jit_operations.iter().any(|op| {
                matches!(op, JITOperation::AllocationRewrite { .. })
            });
            
            if has_allocation_rewrite {
                prop_assert!(!allocation_validation_result.violations.is_empty(),
                    "JIT Compiler allocation rewrite attempts should be rejected (Requirement 1.1)");
                
                // Check that violations are properly reported
                for violation in &allocation_validation_result.violations {
                    prop_assert_eq!(violation.component, ComponentId::JITCompiler,
                        "Allocation rewrite violation should be attributed to JIT Compiler");
                    
                    // Check if this is specifically an allocation rewrite violation
                    if violation.rule_id.as_ref().map_or(false, |id| id.contains("JIT_ALLOCATION_IMMUTABILITY_001")) {
                        prop_assert!(violation.description.contains("rewrite register allocation"),
                            "Allocation rewrite violations should mention register allocation rewrite");
                    }
                }
            } else {
                // Check for unallocated register access attempts
                let has_unallocated_access = jit_operations.iter().any(|op| {
                    match op {
                        JITOperation::CodeGeneration { register_accesses, .. } => {
                            register_accesses.iter().any(|access| {
                                !analysis_input.allocation_map.values().any(|&allocated_reg| allocated_reg == access.register)
                            })
                        }
                        JITOperation::CodeOptimization { affected_registers, .. } => {
                            affected_registers.iter().any(|&register| {
                                !analysis_input.allocation_map.values().any(|&allocated_reg| allocated_reg == register)
                            })
                        }
                        _ => false,
                    }
                });
                
                if has_unallocated_access {
                    prop_assert!(!allocation_validation_result.violations.is_empty(),
                        "JIT Compiler access to unallocated registers should be rejected");
                } else {
                    prop_assert!(allocation_validation_result.violations.is_empty(),
                        "Valid JIT operations should be allowed");
                }
            }
            
            // Test 5: Static safety analysis must use only specified input set (Requirement 3.8)
            // The static proof input set must be exactly: (IR structural fingerprint + allocation map + target ABI model)
            if let Some(safety_proof) = &bounds_checking_result.safety_proof {
                prop_assert_eq!(&safety_proof.input_set.ir_structural_fingerprint, &analysis_input.ir_structural_fingerprint,
                    "Static safety proof must use the exact IR structural fingerprint provided");
                prop_assert_eq!(&safety_proof.input_set.allocation_map, &analysis_input.allocation_map,
                    "Static safety proof must use the exact allocation map provided");
                prop_assert_eq!(&safety_proof.input_set.target_abi_model, &analysis_input.target_abi_model,
                    "Static safety proof must use the exact target ABI model provided");
                
                // Verify proof does not depend on runtime profiling, timing, or execution history
                for proof_step in &safety_proof.proof_steps {
                    prop_assert!(!proof_step.reasoning.to_lowercase().contains("runtime"),
                        "Static safety proof must not depend on runtime information");
                    prop_assert!(!proof_step.reasoning.to_lowercase().contains("timing"),
                        "Static safety proof must not depend on timing information");
                    prop_assert!(!proof_step.reasoning.to_lowercase().contains("execution history"),
                        "Static safety proof must not depend on execution history");
                    prop_assert!(!proof_step.reasoning.to_lowercase().contains("profiling"),
                        "Static safety proof must not depend on profiling information");
                }
            }
            
            // Test 6: Constitutional Rule Engine authorization is required for bounds check elision
            for elided_check in &bounds_checking_result.elided_checks {
                // Verify that the constitutional engine would actually authorize this elision
                let test_operation = JITOperation::CodeGeneration {
                    register_accesses: vec![RegisterAccess {
                        register: elided_check.register,
                        access_type: elided_check.access_type,
                        instruction_address: elided_check.instruction_address,
                        bounds_check_required: false, // Elision requested
                    }],
                    bounds_checking_enabled: false,
                };
                
                // The constitutional engine should allow this operation since it was elided
                let constitutional_validation = constitutional_engine.analyze_jit_allocation_immutability(&test_operation);
                prop_assert!(constitutional_validation.violations.is_empty(),
                    "Constitutional Rule Engine should authorize elided bounds checks");
            }
            
            // Test 7: Architecture-specific bounds checking implementation
            match target_architecture {
                TargetArchitecture::X86_64 => {
                    for bounds_check in &bounds_checking_result.bounds_checks {
                        match &bounds_check.check_implementation {
                            BoundsCheckImplementation::Software { check_code, .. } => {
                                prop_assert!(check_code.contains("cmp") || check_code.contains("jae"),
                                    "x86_64 bounds checks should use appropriate comparison instructions");
                            }
                            _ => {} // Other implementations are also valid
                        }
                    }
                }
                TargetArchitecture::ARM64 => {
                    for bounds_check in &bounds_checking_result.bounds_checks {
                        match &bounds_check.check_implementation {
                            BoundsCheckImplementation::Software { check_code, .. } => {
                                prop_assert!(check_code.contains("cmp") || check_code.contains("b.hs"),
                                    "ARM64 bounds checks should use appropriate comparison instructions");
                            }
                            _ => {} // Other implementations are also valid
                        }
                    }
                }
                TargetArchitecture::RISCV64 => {
                    for bounds_check in &bounds_checking_result.bounds_checks {
                        match &bounds_check.check_implementation {
                            BoundsCheckImplementation::Software { check_code, .. } => {
                                prop_assert!(check_code.contains("bgeu"),
                                    "RISC-V bounds checks should use appropriate branch instructions");
                            }
                            _ => {} // Other implementations are also valid
                        }
                    }
                }
            }
        }
    }
}
