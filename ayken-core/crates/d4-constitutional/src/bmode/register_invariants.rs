//! Register Invariants Specification for D4 Constitutional Framework (B-MODE)
//!
//! This module implements pure B-MODE register allocation invariant specification that provides
//! immutable invariant analysis without stateful validation or enforcement.
//!
//! B-MODE PRINCIPLES:
//! - All operations return SpecificationReport, never Result<()> for spec violations
//! - Immutable invariant analysis (&self), no state mutations
//! - Specification and analysis only, no invariant enforcement
//! - No allocation validation operations, only specification analysis

use crate::errors::{SpecificationReport, SpecificationViolation, SpecificationFinding, ViolationType, FindingType};
use crate::types::{ComponentId, DeterministicClock, Severity, VirtualRegisterId, PhysicalRegisterId, AllocationDecision, RegisterBinding, SpillLocation, TargetArchitecture, PerformanceRequirements, AllocationConstraints};
use crate::bmode::validation_location::ValidationLocation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pure B-MODE register invariants specification analyzer interface
pub trait RegisterInvariantsAnalyzer {
    /// Analyze allocation uniqueness invariant specification (B-MODE)
    fn analyze_allocation_uniqueness_invariant(&self, allocations: &[AllocationDecision]) -> SpecificationReport;

    /// Analyze mapping consistency invariant specification (B-MODE)
    fn analyze_mapping_consistency_invariant(&self, allocations: &[AllocationDecision]) -> SpecificationReport;

    /// Analyze spill overhead invariant specification (B-MODE)
    fn analyze_spill_overhead_invariant(&self, allocations: &[AllocationDecision], performance_requirements: &PerformanceRequirements) -> SpecificationReport;

    /// Specify register invariant requirements (B-MODE)
    fn specify_register_invariant_requirements(&self, invariant_type: RegisterInvariantType) -> RegisterInvariantRequirementsReport;

    /// Get immutable register invariants catalog for analysis
    fn register_invariants_catalog(&self) -> &RegisterInvariantsCatalog;
}

/// Register invariant types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RegisterInvariantType {
    AllocationUniqueness,
    MappingConsistency,
    SpillOverheadLimit,
    PhysicalRegisterRange,
    SpillLocationAlignment,
    ConstraintCompliance,
}

/// Register invariant specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterInvariantSpec {
    pub invariant_id: String,
    pub invariant_type: RegisterInvariantType,
    pub description: String,
    pub formal_specification: String,
    pub threshold_values: BTreeMap<String, f64>, // Normalized to 6 decimal places
    pub target_architectures: Vec<TargetArchitecture>,
    pub constitutional_level: ConstitutionalLevel,
}

/// Constitutional levels for invariants
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstitutionalLevel {
    Constitutional,
    Administrative,
    Operational,
    Advisory,
}

/// Register invariant requirements analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterInvariantRequirementsReport {
    pub invariant_type: RegisterInvariantType,
    pub specification_required: bool,
    pub constitutional_level: ConstitutionalLevel,
    pub threshold_specifications: BTreeMap<String, f64>, // Normalized to 6 decimal places
    pub validation_requirements: Vec<String>,
    pub compliance_criteria: Vec<String>,
    pub analysis_timestamp: crate::types::LogicalTimestamp,
}

/// Allocation uniqueness analysis specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationUniquenessAnalysisSpec {
    pub virtual_registers: Vec<VirtualRegisterId>,
    pub physical_register_mappings: BTreeMap<PhysicalRegisterId, VirtualRegisterId>,
    pub conflict_analysis: ConflictAnalysisSpec,
    pub uniqueness_compliance: bool,
}

/// Conflict analysis specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictAnalysisSpec {
    pub detected_conflicts: Vec<RegisterConflictSpec>,
    pub conflict_resolution_recommendations: Vec<String>,
}

/// Register conflict specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterConflictSpec {
    pub conflicting_virtual_registers: Vec<VirtualRegisterId>,
    pub shared_physical_register: PhysicalRegisterId,
    pub conflict_severity: Severity,
}

/// Spill overhead analysis specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpillOverheadAnalysisSpec {
    pub total_allocations: usize,
    pub spilled_allocations: usize,
    pub calculated_overhead_percentage: f64, // Normalized to 6 decimal places
    pub threshold_percentage: f64, // Normalized to 6 decimal places
    pub overhead_compliance: bool,
    pub optimization_recommendations: Vec<String>,
}

/// Physical register range analysis specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalRegisterRangeAnalysisSpec {
    pub target_architecture: TargetArchitecture,
    pub max_register_id: u32,
    pub analyzed_registers: Vec<PhysicalRegisterId>,
    pub out_of_range_registers: Vec<PhysicalRegisterId>,
    pub range_compliance: bool,
}

/// Spill location analysis specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpillLocationAnalysisSpec {
    pub spill_locations: Vec<SpillLocationSpec>,
    pub alignment_compliance: bool,
    pub size_compliance: bool,
    pub address_compliance: bool,
}

/// Spill location specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpillLocationSpec {
    pub memory_address: u64,
    pub size_bytes: u32,
    pub alignment: u32,
    pub is_power_of_two_size: bool,
    pub is_properly_aligned: bool,
}

/// Immutable register invariants catalog
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterInvariantsCatalog {
    pub invariant_specifications: BTreeMap<RegisterInvariantType, RegisterInvariantSpec>,
    pub catalog_version: String,
    pub last_updated: crate::types::LogicalTimestamp,
}

/// Default implementation of register invariants analyzer (B-MODE)
#[derive(Debug, Clone)]
pub struct DefaultRegisterInvariantsAnalyzer {
    catalog: RegisterInvariantsCatalog,
}

impl DefaultRegisterInvariantsAnalyzer {
    /// Create a new register invariants analyzer with default catalog
    pub fn new() -> Self {
        Self {
            catalog: Self::create_default_catalog(),
        }
    }

    /// Create the default immutable register invariants catalog
    fn create_default_catalog() -> RegisterInvariantsCatalog {
        let mut invariant_specifications = BTreeMap::new();

        // Allocation uniqueness invariant specification
        let allocation_uniqueness_spec = RegisterInvariantSpec {
            invariant_id: "REG_ALLOC_UNIQUENESS".to_string(),
            invariant_type: RegisterInvariantType::AllocationUniqueness,
            description: "No two virtual registers share the same physical register".to_string(),
            formal_specification: "∀ v1, v2 ∈ VirtualRegisters, v1 ≠ v2 → physical_mapping(v1) ≠ physical_mapping(v2)".to_string(),
            threshold_values: BTreeMap::new(),
            target_architectures: vec![
                TargetArchitecture::X86_64,
                TargetArchitecture::ARM64,
                TargetArchitecture::RISCV64,
            ],
            constitutional_level: ConstitutionalLevel::Constitutional,
        };

        // Spill overhead limit invariant specification
        let mut spill_overhead_thresholds = BTreeMap::new();
        spill_overhead_thresholds.insert("max_spill_percentage".to_string(), Self::normalize_float(5.0));
        spill_overhead_thresholds.insert("warning_threshold".to_string(), Self::normalize_float(3.0));

        let spill_overhead_spec = RegisterInvariantSpec {
            invariant_id: "SPILL_OVERHEAD_LIMIT".to_string(),
            invariant_type: RegisterInvariantType::SpillOverheadLimit,
            description: "Spill overhead must not exceed 5% threshold".to_string(),
            formal_specification: "spill_overhead_percentage ≤ 5.0".to_string(),
            threshold_values: spill_overhead_thresholds,
            target_architectures: vec![
                TargetArchitecture::X86_64,
                TargetArchitecture::ARM64,
                TargetArchitecture::RISCV64,
            ],
            constitutional_level: ConstitutionalLevel::Constitutional,
        };

        // Physical register range invariant specification
        let mut register_range_thresholds = BTreeMap::new();
        register_range_thresholds.insert("x86_64_max_registers".to_string(), Self::normalize_float(16.0));
        register_range_thresholds.insert("arm64_max_registers".to_string(), Self::normalize_float(31.0));
        register_range_thresholds.insert("riscv64_max_registers".to_string(), Self::normalize_float(32.0));

        let physical_register_range_spec = RegisterInvariantSpec {
            invariant_id: "PHYSICAL_REGISTER_RANGE".to_string(),
            invariant_type: RegisterInvariantType::PhysicalRegisterRange,
            description: "Physical register IDs must be within valid range for target architecture".to_string(),
            formal_specification: "∀ pr ∈ PhysicalRegisters, 0 ≤ pr.id < max_registers(architecture)".to_string(),
            threshold_values: register_range_thresholds,
            target_architectures: vec![
                TargetArchitecture::X86_64,
                TargetArchitecture::ARM64,
                TargetArchitecture::RISCV64,
            ],
            constitutional_level: ConstitutionalLevel::Administrative,
        };

        invariant_specifications.insert(RegisterInvariantType::AllocationUniqueness, allocation_uniqueness_spec);
        invariant_specifications.insert(RegisterInvariantType::SpillOverheadLimit, spill_overhead_spec);
        invariant_specifications.insert(RegisterInvariantType::PhysicalRegisterRange, physical_register_range_spec);

        RegisterInvariantsCatalog {
            invariant_specifications,
            catalog_version: "1.0.0".to_string(),
            last_updated: DeterministicClock::new().now(),
        }
    }

    /// Normalize floating point values to 6 decimal places for deterministic comparison
    fn normalize_float(value: f64) -> f64 {
        (value * 1_000_000.0_f64).round() / 1_000_000.0
    }

    /// Calculate spill overhead percentage from allocations
    fn calculate_spill_overhead(&self, allocations: &[AllocationDecision]) -> f64 {
        if allocations.is_empty() {
            return 0.0;
        }

        let total_allocations = allocations.len();
        let spilled_allocations = allocations
            .iter()
            .filter(|alloc| matches!(alloc.binding, RegisterBinding::Spilled(_)))
            .count();

        Self::normalize_float((spilled_allocations as f64 / total_allocations as f64) * 100.0)
    }

    /// Get maximum register count for target architecture
    fn get_max_registers_for_architecture(&self, architecture: TargetArchitecture) -> u32 {
        match architecture {
            TargetArchitecture::X86_64 => 16,  // 16 general-purpose registers in x86-64
            TargetArchitecture::ARM64 => 31,   // 31 general-purpose registers in ARM64 (X0-X30)
            TargetArchitecture::RISCV64 => 32, // 32 general-purpose registers in RISC-V (x0-x31)
        }
    }
}

impl RegisterInvariantsAnalyzer for DefaultRegisterInvariantsAnalyzer {
    fn analyze_allocation_uniqueness_invariant(&self, allocations: &[AllocationDecision]) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Build physical register mapping for analysis
        let mut physical_register_mappings = BTreeMap::new();
        let mut detected_conflicts = Vec::new();

        for allocation in allocations {
            if let RegisterBinding::Physical(physical_register) = allocation.binding {
                if let Some(existing_virtual) = physical_register_mappings.get(&physical_register) {
                    // Detected conflict
                    detected_conflicts.push(RegisterConflictSpec {
                        conflicting_virtual_registers: vec![allocation.virtual_register, *existing_virtual],
                        shared_physical_register: physical_register,
                        conflict_severity: Severity::Critical,
                    });
                } else {
                    physical_register_mappings.insert(physical_register, allocation.virtual_register);
                }
            }
        }

        if !detected_conflicts.is_empty() {
            for conflict in &detected_conflicts {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationViolation,
                    component: ComponentId::D4RegisterAllocator,
                    rule_id: Some("ALLOCATION_UNIQUENESS_VIOLATION".to_string()),
                    description: format!(
                        "Allocation uniqueness violated: Virtual registers {:?} both mapped to physical register {:?}",
                        conflict.conflicting_virtual_registers, conflict.shared_physical_register
                    ),
                    remediation_hint: "Ensure each physical register is mapped to at most one virtual register".to_string(),
                });
            }
        } else {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::D4RegisterAllocator,
                description: format!("Allocation uniqueness invariant satisfied for {} allocations", allocations.len()),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::D4RegisterAllocator),
            });
        }

        report
    }

    fn analyze_mapping_consistency_invariant(&self, allocations: &[AllocationDecision]) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        for allocation in allocations {
            // Analyze constraint compliance
            if let RegisterBinding::Physical(physical_register) = allocation.binding {
                let constraints = &allocation.decision_context.constraints;
                
                if constraints.forbidden_registers.contains(&physical_register) {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationViolation,
                        component: ComponentId::D4RegisterAllocator,
                        rule_id: Some("FORBIDDEN_REGISTER_ALLOCATION".to_string()),
                        description: format!(
                            "Virtual register {:?} allocated to forbidden register {:?}",
                            allocation.virtual_register, physical_register
                        ),
                        remediation_hint: "Remove forbidden register from allocation or choose different physical register".to_string(),
                    });
                }

                if constraints.excluded_registers.contains(&physical_register) {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationViolation,
                        component: ComponentId::D4RegisterAllocator,
                        rule_id: Some("EXCLUDED_REGISTER_ALLOCATION".to_string()),
                        description: format!(
                            "Virtual register {:?} allocated to excluded register {:?}",
                            allocation.virtual_register, physical_register
                        ),
                        remediation_hint: "Remove excluded register from allocation or choose different physical register".to_string(),
                    });
                }
            }

            // Analyze spill location if present
            if let RegisterBinding::Spilled(ref spill_location) = allocation.binding {
                // Analyze spill location alignment
                if spill_location.size_bytes == 0 {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationInconsistent,
                        component: ComponentId::D4RegisterAllocator,
                        rule_id: Some("ZERO_SPILL_SIZE".to_string()),
                        description: "Spill location has zero size".to_string(),
                        remediation_hint: "Set appropriate non-zero size for spill location".to_string(),
                    });
                }

                if !spill_location.size_bytes.is_power_of_two() {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationInconsistent,
                        component: ComponentId::D4RegisterAllocator,
                        rule_id: Some("NON_POWER_OF_TWO_SPILL_SIZE".to_string()),
                        description: format!(
                            "Spill location size {} bytes is not a power of 2",
                            spill_location.size_bytes
                        ),
                        remediation_hint: "Use power-of-2 size for proper memory alignment".to_string(),
                    });
                }

                if spill_location.memory_address % (spill_location.size_bytes as u64) != 0 {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationInconsistent,
                        component: ComponentId::D4RegisterAllocator,
                        rule_id: Some("MISALIGNED_SPILL_ADDRESS".to_string()),
                        description: format!(
                            "Spill location address 0x{:x} is not aligned to {} byte boundary",
                            spill_location.memory_address, spill_location.size_bytes
                        ),
                        remediation_hint: "Align spill location address to match size boundary".to_string(),
                    });
                }
            }
        }

        if report.violations.is_empty() {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::D4RegisterAllocator,
                description: "Mapping consistency invariant satisfied for all allocations".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::D4RegisterAllocator),
            });
        }

        report
    }

    fn analyze_spill_overhead_invariant(&self, allocations: &[AllocationDecision], performance_requirements: &PerformanceRequirements) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        let calculated_overhead = self.calculate_spill_overhead(allocations);
        
        // Get threshold from catalog or performance requirements
        let threshold = if let Some(max_spill_rate) = &performance_requirements.max_spill_rate {
            *max_spill_rate
        } else if let Some(spec) = self.catalog.invariant_specifications.get(&RegisterInvariantType::SpillOverheadLimit) {
            spec.threshold_values.get("max_spill_percentage").copied().unwrap_or(5.0)
        } else {
            5.0
        };

        if calculated_overhead > threshold {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationViolation,
                component: ComponentId::D4RegisterAllocator,
                rule_id: Some("SPILL_OVERHEAD_THRESHOLD_EXCEEDED".to_string()),
                description: format!(
                    "Spill overhead {:.6}% exceeds threshold of {:.6}%",
                    calculated_overhead, threshold
                ),
                remediation_hint: "Reduce spill overhead or disable register optimization for this IR block".to_string(),
            });
        } else {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: ComponentId::D4RegisterAllocator,
                description: format!(
                    "Spill overhead {:.6}% is within threshold of {:.6}%",
                    calculated_overhead, threshold
                ),
                severity: Severity::Info,
                location: ValidationLocation::new(ComponentId::D4RegisterAllocator),
            });
        }

        report
    }

    fn specify_register_invariant_requirements(&self, invariant_type: RegisterInvariantType) -> RegisterInvariantRequirementsReport {
        let specification_required = matches!(
            invariant_type,
            RegisterInvariantType::AllocationUniqueness |
            RegisterInvariantType::SpillOverheadLimit |
            RegisterInvariantType::MappingConsistency
        );

        let constitutional_level = if matches!(
            invariant_type,
            RegisterInvariantType::AllocationUniqueness |
            RegisterInvariantType::SpillOverheadLimit
        ) {
            ConstitutionalLevel::Constitutional
        } else {
            ConstitutionalLevel::Administrative
        };

        let threshold_specifications = if let Some(spec) = self.catalog.invariant_specifications.get(&invariant_type) {
            spec.threshold_values.clone()
        } else {
            BTreeMap::new()
        };

        let validation_requirements = match invariant_type {
            RegisterInvariantType::AllocationUniqueness => vec![
                "unique_physical_register_mapping".to_string(),
                "conflict_detection".to_string(),
            ],
            RegisterInvariantType::SpillOverheadLimit => vec![
                "spill_percentage_calculation".to_string(),
                "threshold_comparison".to_string(),
            ],
            RegisterInvariantType::MappingConsistency => vec![
                "constraint_compliance_check".to_string(),
                "spill_location_validation".to_string(),
            ],
            RegisterInvariantType::PhysicalRegisterRange => vec![
                "architecture_specific_range_check".to_string(),
            ],
            RegisterInvariantType::SpillLocationAlignment => vec![
                "power_of_two_size_check".to_string(),
                "address_alignment_check".to_string(),
            ],
            RegisterInvariantType::ConstraintCompliance => vec![
                "forbidden_register_check".to_string(),
                "excluded_register_check".to_string(),
            ],
        };

        let compliance_criteria = match invariant_type {
            RegisterInvariantType::AllocationUniqueness => vec![
                "no_physical_register_conflicts".to_string(),
                "one_to_one_mapping_maintained".to_string(),
            ],
            RegisterInvariantType::SpillOverheadLimit => vec![
                "overhead_below_threshold".to_string(),
                "optimization_recommendations_provided".to_string(),
            ],
            _ => vec!["specification_requirements_met".to_string()],
        };

        RegisterInvariantRequirementsReport {
            invariant_type,
            specification_required,
            constitutional_level,
            threshold_specifications,
            validation_requirements,
            compliance_criteria,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }

    fn register_invariants_catalog(&self) -> &RegisterInvariantsCatalog {
        &self.catalog
    }
}

/// Helper function to create a register invariant specification
pub fn create_register_invariant_specification(
    invariant_id: String,
    invariant_type: RegisterInvariantType,
    description: String,
    formal_specification: String,
) -> RegisterInvariantSpec {
    RegisterInvariantSpec {
        invariant_id,
        invariant_type,
        description,
        formal_specification,
        threshold_values: BTreeMap::new(),
        target_architectures: vec![
            TargetArchitecture::X86_64,
            TargetArchitecture::ARM64,
            TargetArchitecture::RISCV64,
        ],
        constitutional_level: ConstitutionalLevel::Administrative,
    }
}

/// Helper function to create an allocation uniqueness analysis specification
pub fn create_allocation_uniqueness_analysis_specification(
    allocations: &[AllocationDecision],
) -> AllocationUniquenessAnalysisSpec {
    let mut physical_register_mappings = BTreeMap::new();
    let mut detected_conflicts = Vec::new();

    for allocation in allocations {
        if let RegisterBinding::Physical(physical_register) = allocation.binding {
            if let Some(existing_virtual) = physical_register_mappings.get(&physical_register) {
                detected_conflicts.push(RegisterConflictSpec {
                    conflicting_virtual_registers: vec![allocation.virtual_register, *existing_virtual],
                    shared_physical_register: physical_register,
                    conflict_severity: Severity::Critical,
                });
            } else {
                physical_register_mappings.insert(physical_register, allocation.virtual_register);
            }
        }
    }

    let uniqueness_compliance = detected_conflicts.is_empty();

    AllocationUniquenessAnalysisSpec {
        virtual_registers: allocations.iter().map(|a| a.virtual_register).collect(),
        physical_register_mappings,
        conflict_analysis: ConflictAnalysisSpec {
            detected_conflicts,
            conflict_resolution_recommendations: vec![
                "reassign_conflicting_registers".to_string(),
                "use_spill_locations_for_conflicts".to_string(),
            ],
        },
        uniqueness_compliance,
    }
}

/// Helper function to create spill overhead analysis specification
pub fn create_spill_overhead_analysis_specification(
    allocations: &[AllocationDecision],
    threshold_percentage: f64,
) -> SpillOverheadAnalysisSpec {
    let total_allocations = allocations.len();
    let spilled_allocations = allocations
        .iter()
        .filter(|alloc| matches!(alloc.binding, RegisterBinding::Spilled(_)))
        .count();

    let calculated_overhead_percentage = if total_allocations > 0 {
        DefaultRegisterInvariantsAnalyzer::normalize_float(
            (spilled_allocations as f64 / total_allocations as f64) * 100.0
        )
    } else {
        0.0
    };

    let overhead_compliance = calculated_overhead_percentage <= DefaultRegisterInvariantsAnalyzer::normalize_float(threshold_percentage);

    let optimization_recommendations = if !overhead_compliance {
        vec![
            "disable_register_optimization_for_current_ir_block".to_string(),
            "increase_available_physical_registers".to_string(),
            "reduce_register_pressure_through_code_restructuring".to_string(),
        ]
    } else {
        vec!["maintain_current_allocation_strategy".to_string()]
    };

    SpillOverheadAnalysisSpec {
        total_allocations,
        spilled_allocations,
        calculated_overhead_percentage,
        threshold_percentage: DefaultRegisterInvariantsAnalyzer::normalize_float(threshold_percentage),
        overhead_compliance,
        optimization_recommendations,
    }
}

/// Helper function to create physical register range analysis specification
pub fn create_physical_register_range_analysis_specification(
    allocations: &[AllocationDecision],
    target_architecture: TargetArchitecture,
) -> PhysicalRegisterRangeAnalysisSpec {
    let max_register_id = match target_architecture {
        TargetArchitecture::X86_64 => 15,  // 0-15 for x86-64
        TargetArchitecture::ARM64 => 30,   // 0-30 for ARM64 (X0-X30)
        TargetArchitecture::RISCV64 => 31, // 0-31 for RISC-V (x0-x31)
    };

    let mut analyzed_registers = Vec::new();
    let mut out_of_range_registers = Vec::new();

    for allocation in allocations {
        if let RegisterBinding::Physical(physical_register) = allocation.binding {
            analyzed_registers.push(physical_register);
            if physical_register.0 > max_register_id {
                out_of_range_registers.push(physical_register);
            }
        }
    }

    let range_compliance = out_of_range_registers.is_empty();

    PhysicalRegisterRangeAnalysisSpec {
        target_architecture,
        max_register_id,
        analyzed_registers,
        out_of_range_registers,
        range_compliance,
    }
}

/// Helper function to create spill location analysis specification
pub fn create_spill_location_analysis_specification(
    allocations: &[AllocationDecision],
) -> SpillLocationAnalysisSpec {
    let mut spill_locations = Vec::new();
    let mut alignment_compliance = true;
    let mut size_compliance = true;
    let mut address_compliance = true;

    for allocation in allocations {
        if let RegisterBinding::Spilled(ref spill_location) = allocation.binding {
            let is_power_of_two_size = spill_location.size_bytes.is_power_of_two();
            let is_properly_aligned = spill_location.memory_address % (spill_location.size_bytes as u64) == 0;

            if !is_power_of_two_size {
                size_compliance = false;
            }
            if !is_properly_aligned {
                alignment_compliance = false;
            }
            if spill_location.size_bytes == 0 {
                size_compliance = false;
                address_compliance = false;
            }

            spill_locations.push(SpillLocationSpec {
                memory_address: spill_location.memory_address,
                size_bytes: spill_location.size_bytes,
                alignment: spill_location.size_bytes,
                is_power_of_two_size,
                is_properly_aligned,
            });
        }
    }

    SpillLocationAnalysisSpec {
        spill_locations,
        alignment_compliance,
        size_compliance,
        address_compliance,
    }
}