//! Contract Specification and Analysis for D4 Constitutional Framework (B-MODE)
//!
//! This module implements pure B-MODE contract specification and analysis that provides
//! immutable contract analysis capabilities without stateful generation or enforcement.
//!
//! B-MODE PRINCIPLES:
//! - All operations return SpecificationReport, never Result<()> for spec violations
//! - Immutable contract analysis (&self), no state mutations
//! - Specification and analysis only, no contract enforcement
//! - No contract generation/registration operations, only analysis

use crate::errors::{SpecificationReport, SpecificationViolation, SpecificationFinding, ViolationType, FindingType};
use crate::types::{ComponentId, DeterministicClock, Severity};
use crate::bmode::validation_location::ValidationLocation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pure B-MODE contract specification analyzer interface
pub trait ContractSpecAnalyzer {
    /// Analyze contract completeness for a given component (B-MODE)
    fn analyze_contract_completeness(&self, component: ComponentId) -> SpecificationReport;

    /// Analyze contract specification validity (B-MODE)
    fn analyze_contract_specification(&self, contract_spec: &ContractSpecification) -> SpecificationReport;

    /// Specify contract requirements for Gate E validation (B-MODE)
    fn specify_contract_requirements(&self, component: ComponentId) -> ContractRequirementsReport;

    /// Get immutable contract catalog for analysis
    fn contract_catalog(&self) -> &ContractCatalog;
}

/// Contract specification for constitutional analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractSpecification {
    pub contract_id: String,
    pub target_component: ComponentId,
    pub struct_specifications: Vec<StructSpecification>,
    pub trait_specifications: Vec<TraitSpecification>,
    pub invariant_specifications: Vec<InvariantSpecification>,
    pub property_test_specifications: Vec<PropertyTestSpecification>,
    pub performance_specifications: PerformanceSpecification,
}

/// Struct specification for contracts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructSpecification {
    pub name: String,
    pub fields: Vec<FieldSpecification>,
    pub visibility: VisibilitySpecification,
    pub documentation: String,
    pub constitutional_constraints: Vec<String>,
}

/// Field specification within structs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpecification {
    pub name: String,
    pub field_type: String,
    pub visibility: VisibilitySpecification,
    pub documentation: String,
    pub constraints: Vec<String>,
}

/// Trait specification for contracts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitSpecification {
    pub name: String,
    pub methods: Vec<MethodSpecification>,
    pub associated_types: Vec<AssociatedTypeSpecification>,
    pub documentation: String,
    pub constitutional_requirements: Vec<String>,
}

/// Method specification within traits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodSpecification {
    pub name: String,
    pub parameters: Vec<ParameterSpecification>,
    pub return_type: Option<String>,
    pub documentation: String,
    pub b_mode_compliance: bool,
}

/// Parameter specification for methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterSpecification {
    pub name: String,
    pub param_type: String,
    pub is_mutable: bool,
}

/// Associated type specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssociatedTypeSpecification {
    pub name: String,
    pub bounds: Vec<String>,
    pub documentation: String,
}

/// Visibility specification levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilitySpecification {
    Public,
    Private,
    Crate,
    Module(String),
}

/// Invariant specification for contracts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantSpecification {
    pub invariant_id: String,
    pub description: String,
    pub formal_specification: String,
    pub validation_method: ValidationMethodSpecification,
    pub violation_consequences: Vec<String>,
    pub constitutional_level: ConstitutionalLevel,
}

/// Validation method specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationMethodSpecification {
    PropertyTest(String),
    UnitTest(String),
    StaticAnalysis(String),
    ComplianceCheck(String),
}

/// Constitutional levels for invariants
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstitutionalLevel {
    Constitutional,
    Administrative,
    Operational,
    Advisory,
}

/// Property test specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyTestSpecification {
    pub test_name: String,
    pub property_description: String,
    pub input_generators: Vec<String>,
    pub test_iterations: u32,
    pub shrinking_enabled: bool,
    pub b_mode_compliance: bool,
}

/// Performance specification for contracts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSpecification {
    pub allocation_overhead_target: f64, // Normalized to 6 decimal places
    pub reuse_rate_capability: f64,      // Normalized to 6 decimal places
    pub spill_overhead_threshold: f64,   // Normalized to 6 decimal places
    pub measurement_methods: Vec<String>,
    pub validation_frequency: ValidationFrequency,
}

/// Validation frequency specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationFrequency {
    PerCommit,
    Daily,
    Weekly,
    OnDemand,
}

/// Contract requirements analysis report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractRequirementsReport {
    pub component: ComponentId,
    pub required_structs: Vec<String>,
    pub required_traits: Vec<String>,
    pub required_invariants: Vec<String>,
    pub required_property_tests: Vec<String>,
    pub constitutional_requirements: Vec<String>,
    pub gate_e_readiness: bool,
    pub analysis_timestamp: crate::types::LogicalTimestamp,
}

/// Immutable contract catalog
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractCatalog {
    pub specifications: BTreeMap<ComponentId, ContractSpecification>,
    pub catalog_version: String,
    pub last_updated: crate::types::LogicalTimestamp,
}

/// Default implementation of contract specification analyzer (B-MODE)
#[derive(Debug, Clone)]
pub struct DefaultContractSpecAnalyzer {
    catalog: ContractCatalog,
}

impl DefaultContractSpecAnalyzer {
    /// Create a new contract specification analyzer with default catalog
    pub fn new() -> Self {
        Self {
            catalog: Self::create_default_catalog(),
        }
    }

    /// Create the default immutable contract catalog
    fn create_default_catalog() -> ContractCatalog {
        let mut specifications = BTreeMap::new();

        // D4 Register Allocator contract specification
        let d4_register_allocator_spec = ContractSpecification {
            contract_id: "d4_register_allocator_contract".to_string(),
            target_component: ComponentId::D4RegisterAllocator,
            struct_specifications: vec![
                StructSpecification {
                    name: "RegisterAllocator".to_string(),
                    fields: vec![
                        FieldSpecification {
                            name: "allocation_map".to_string(),
                            field_type: "BTreeMap<VirtualRegisterId, PhysicalRegisterId>".to_string(),
                            visibility: VisibilitySpecification::Private,
                            documentation: "Maps virtual registers to physical registers".to_string(),
                            constraints: vec!["immutable_after_allocation".to_string()],
                        },
                        FieldSpecification {
                            name: "spill_locations".to_string(),
                            field_type: "BTreeMap<VirtualRegisterId, SpillLocation>".to_string(),
                            visibility: VisibilitySpecification::Private,
                            documentation: "Tracks spilled register locations".to_string(),
                            constraints: vec!["deterministic_ordering".to_string()],
                        },
                    ],
                    visibility: VisibilitySpecification::Public,
                    documentation: "Core register allocator implementation".to_string(),
                    constitutional_constraints: vec!["allocation_immutability".to_string()],
                },
            ],
            trait_specifications: vec![
                TraitSpecification {
                    name: "RegisterAllocationStrategy".to_string(),
                    methods: vec![
                        MethodSpecification {
                            name: "analyze_allocation_requirements".to_string(),
                            parameters: vec![
                                ParameterSpecification {
                                    name: "self".to_string(),
                                    param_type: "&self".to_string(),
                                    is_mutable: false,
                                },
                                ParameterSpecification {
                                    name: "virtual_registers".to_string(),
                                    param_type: "&[VirtualRegisterId]".to_string(),
                                    is_mutable: false,
                                },
                            ],
                            return_type: Some("SpecificationReport".to_string()),
                            documentation: "Analyze allocation requirements for virtual registers".to_string(),
                            b_mode_compliance: true,
                        },
                    ],
                    associated_types: vec![],
                    documentation: "Strategy trait for register allocation analysis".to_string(),
                    constitutional_requirements: vec!["b_mode_purity".to_string()],
                },
            ],
            invariant_specifications: vec![
                InvariantSpecification {
                    invariant_id: "REG_ALLOC_UNIQUENESS".to_string(),
                    description: "No two virtual registers share the same physical register".to_string(),
                    formal_specification: "∀ v1, v2 ∈ VirtualRegisters, v1 ≠ v2 → physical_mapping(v1) ≠ physical_mapping(v2)".to_string(),
                    validation_method: ValidationMethodSpecification::PropertyTest("Property 5: Register Allocation Uniqueness".to_string()),
                    violation_consequences: vec![
                        "Register corruption".to_string(),
                        "Undefined behavior".to_string(),
                    ],
                    constitutional_level: ConstitutionalLevel::Constitutional,
                },
            ],
            property_test_specifications: vec![
                PropertyTestSpecification {
                    test_name: "test_reg_alloc_uniqueness".to_string(),
                    property_description: "Register allocation uniqueness property".to_string(),
                    input_generators: vec![
                        "virtual_register_id_generator()".to_string(),
                        "allocation_constraints_generator()".to_string(),
                    ],
                    test_iterations: 100,
                    shrinking_enabled: true,
                    b_mode_compliance: true,
                },
            ],
            performance_specifications: PerformanceSpecification {
                allocation_overhead_target: Self::normalize_float(1.0),
                reuse_rate_capability: Self::normalize_float(80.0),
                spill_overhead_threshold: Self::normalize_float(5.0),
                measurement_methods: vec!["benchmark_analysis".to_string()],
                validation_frequency: ValidationFrequency::PerCommit,
            },
        };

        // JIT Compiler contract specification
        let jit_compiler_spec = ContractSpecification {
            contract_id: "jit_compiler_contract".to_string(),
            target_component: ComponentId::JITCompiler,
            struct_specifications: vec![
                StructSpecification {
                    name: "JITCompiler".to_string(),
                    fields: vec![
                        FieldSpecification {
                            name: "bounds_checker".to_string(),
                            field_type: "BoundsChecker".to_string(),
                            visibility: VisibilitySpecification::Private,
                            documentation: "Analyzes bounds checking requirements".to_string(),
                            constraints: vec!["immutable_analysis".to_string()],
                        },
                    ],
                    visibility: VisibilitySpecification::Public,
                    documentation: "Just-In-Time compiler with constitutional constraints".to_string(),
                    constitutional_constraints: vec!["allocation_immutability".to_string()],
                },
            ],
            trait_specifications: vec![
                TraitSpecification {
                    name: "ConstitutionalJITAnalyzer".to_string(),
                    methods: vec![
                        MethodSpecification {
                            name: "analyze_compilation_requirements".to_string(),
                            parameters: vec![
                                ParameterSpecification {
                                    name: "self".to_string(),
                                    param_type: "&self".to_string(),
                                    is_mutable: false,
                                },
                            ],
                            return_type: Some("SpecificationReport".to_string()),
                            documentation: "Analyze compilation requirements with constitutional compliance".to_string(),
                            b_mode_compliance: true,
                        },
                    ],
                    associated_types: vec![],
                    documentation: "JIT compiler that analyzes constitutional constraints".to_string(),
                    constitutional_requirements: vec!["b_mode_purity".to_string()],
                },
            ],
            invariant_specifications: vec![
                InvariantSpecification {
                    invariant_id: "JIT_ALLOCATION_IMMUTABILITY".to_string(),
                    description: "JIT compiler cannot rewrite register allocation decisions".to_string(),
                    formal_specification: "∀ allocation_decision ∈ AllocationDecisions, JIT_cannot_modify(allocation_decision)".to_string(),
                    validation_method: ValidationMethodSpecification::PropertyTest("Property 1: Constitutional Rule Enforcement".to_string()),
                    violation_consequences: vec![
                        "Constitutional violation".to_string(),
                        "System state corruption".to_string(),
                    ],
                    constitutional_level: ConstitutionalLevel::Constitutional,
                },
            ],
            property_test_specifications: vec![
                PropertyTestSpecification {
                    test_name: "test_jit_allocation_immutability".to_string(),
                    property_description: "JIT allocation immutability property".to_string(),
                    input_generators: vec![
                        "allocation_decision_generator()".to_string(),
                        "jit_operation_generator()".to_string(),
                    ],
                    test_iterations: 100,
                    shrinking_enabled: true,
                    b_mode_compliance: true,
                },
            ],
            performance_specifications: PerformanceSpecification {
                allocation_overhead_target: Self::normalize_float(1.0),
                reuse_rate_capability: Self::normalize_float(80.0),
                spill_overhead_threshold: Self::normalize_float(5.0),
                measurement_methods: vec!["constitutional_compliance_analysis".to_string()],
                validation_frequency: ValidationFrequency::PerCommit,
            },
        };

        specifications.insert(ComponentId::D4RegisterAllocator, d4_register_allocator_spec);
        specifications.insert(ComponentId::JITCompiler, jit_compiler_spec);

        ContractCatalog {
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

impl ContractSpecAnalyzer for DefaultContractSpecAnalyzer {
    fn analyze_contract_completeness(&self, component: ComponentId) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        if let Some(spec) = self.catalog.specifications.get(&component) {
            // Analyze struct specifications completeness
            if spec.struct_specifications.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component,
                    rule_id: Some("CONTRACT_STRUCT_COMPLETENESS".to_string()),
                    description: format!("Component {:?} has no struct specifications", component),
                    remediation_hint: "Add required struct specifications to contract".to_string(),
                });
            } else {
                report.add_finding(SpecificationFinding {
                    finding_type: FindingType::SpecificationCompliance,
                    component,
                    description: format!("Component {:?} has {} struct specifications", component, spec.struct_specifications.len()),
                    severity: Severity::Info,
                    location: ValidationLocation::new(component),
                });
            }

            // Analyze invariant specifications completeness
            if spec.invariant_specifications.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component,
                    rule_id: Some("CONTRACT_INVARIANT_COMPLETENESS".to_string()),
                    description: format!("Component {:?} has no invariant specifications", component),
                    remediation_hint: "Add required invariant specifications to contract".to_string(),
                });
            }

            // Analyze B-MODE compliance
            let b_mode_compliant_methods = spec.trait_specifications.iter()
                .flat_map(|t| &t.methods)
                .filter(|m| m.b_mode_compliance)
                .count();

            if b_mode_compliant_methods == 0 && !spec.trait_specifications.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationViolation,
                    component,
                    rule_id: Some("CONTRACT_BMODE_COMPLIANCE".to_string()),
                    description: format!("Component {:?} has no B-MODE compliant methods", component),
                    remediation_hint: "Ensure all methods follow B-MODE principles (return SpecificationReport, use &self)".to_string(),
                });
            }
        } else {
            report.add_violation(SpecificationViolation {
                violation_type: ViolationType::SpecificationIncomplete,
                component,
                rule_id: Some("CONTRACT_NOT_FOUND".to_string()),
                description: format!("Component {:?} has no contract specification", component),
                remediation_hint: "Create contract specification for component".to_string(),
            });
        }

        report
    }

    fn analyze_contract_specification(&self, contract_spec: &ContractSpecification) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        // Analyze struct specifications
        for struct_spec in &contract_spec.struct_specifications {
            if struct_spec.fields.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: contract_spec.target_component,
                    rule_id: Some("STRUCT_EMPTY_FIELDS".to_string()),
                    description: format!("Struct '{}' has no fields specified", struct_spec.name),
                    remediation_hint: "Add field specifications to struct".to_string(),
                });
            }

            if struct_spec.documentation.trim().is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: contract_spec.target_component,
                    rule_id: Some("STRUCT_MISSING_DOCUMENTATION".to_string()),
                    description: format!("Struct '{}' is missing documentation", struct_spec.name),
                    remediation_hint: "Add comprehensive documentation for struct".to_string(),
                });
            }
        }

        // Analyze trait specifications for B-MODE compliance
        for trait_spec in &contract_spec.trait_specifications {
            for method in &trait_spec.methods {
                if !method.b_mode_compliance {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationViolation,
                        component: contract_spec.target_component,
                        rule_id: Some("METHOD_BMODE_VIOLATION".to_string()),
                        description: format!("Method '{}' in trait '{}' is not B-MODE compliant", method.name, trait_spec.name),
                        remediation_hint: "Ensure method returns SpecificationReport and uses &self".to_string(),
                    });
                }

                // Check for mutable self parameters (B-MODE violation)
                if method.parameters.iter().any(|p| p.param_type.contains("&mut self")) {
                    report.add_violation(SpecificationViolation {
                        violation_type: ViolationType::SpecificationViolation,
                        component: contract_spec.target_component,
                        rule_id: Some("METHOD_MUTABLE_SELF".to_string()),
                        description: format!("Method '{}' uses &mut self - violates B-MODE immutability", method.name),
                        remediation_hint: "Use &self instead of &mut self for B-MODE compliance".to_string(),
                    });
                }
            }
        }

        // Analyze invariant specifications
        for invariant in &contract_spec.invariant_specifications {
            if invariant.formal_specification.trim().is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: contract_spec.target_component,
                    rule_id: Some("INVARIANT_MISSING_FORMAL_SPEC".to_string()),
                    description: format!("Invariant '{}' is missing formal specification", invariant.invariant_id),
                    remediation_hint: "Add formal mathematical specification for invariant".to_string(),
                });
            }

            if invariant.violation_consequences.is_empty() {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationIncomplete,
                    component: contract_spec.target_component,
                    rule_id: Some("INVARIANT_MISSING_CONSEQUENCES".to_string()),
                    description: format!("Invariant '{}' has no violation consequences specified", invariant.invariant_id),
                    remediation_hint: "Define consequences of violating this invariant".to_string(),
                });
            }
        }

        // Analyze property test specifications for B-MODE compliance
        for property_test in &contract_spec.property_test_specifications {
            if !property_test.b_mode_compliance {
                report.add_violation(SpecificationViolation {
                    violation_type: ViolationType::SpecificationViolation,
                    component: contract_spec.target_component,
                    rule_id: Some("PROPERTY_TEST_BMODE_VIOLATION".to_string()),
                    description: format!("Property test '{}' is not B-MODE compliant", property_test.test_name),
                    remediation_hint: "Ensure property test follows B-MODE principles".to_string(),
                });
            }
        }

        if report.violations.is_empty() {
            report.add_finding(SpecificationFinding {
                finding_type: FindingType::SpecificationCompliance,
                component: contract_spec.target_component,
                description: "Contract specification is complete and B-MODE compliant".to_string(),
                severity: Severity::Info,
                location: ValidationLocation::new(contract_spec.target_component),
            });
        }

        report
    }

    fn specify_contract_requirements(&self, component: ComponentId) -> ContractRequirementsReport {
        let mut required_structs = Vec::new();
        let mut required_traits = Vec::new();
        let mut required_invariants = Vec::new();
        let mut required_property_tests = Vec::new();
        let mut constitutional_requirements = Vec::new();

        match component {
            ComponentId::D4RegisterAllocator => {
                required_structs.push("RegisterAllocator".to_string());
                required_structs.push("AllocationResult".to_string());
                required_traits.push("RegisterAllocationStrategy".to_string());
                required_invariants.push("REG_ALLOC_UNIQUENESS".to_string());
                required_invariants.push("SPILL_OVERHEAD_LIMIT".to_string());
                required_property_tests.push("test_reg_alloc_uniqueness".to_string());
                constitutional_requirements.push("allocation_immutability".to_string());
                constitutional_requirements.push("b_mode_purity".to_string());
            }
            ComponentId::JITCompiler => {
                required_structs.push("JITCompiler".to_string());
                required_traits.push("ConstitutionalJITAnalyzer".to_string());
                required_invariants.push("JIT_ALLOCATION_IMMUTABILITY".to_string());
                required_property_tests.push("test_jit_allocation_immutability".to_string());
                constitutional_requirements.push("allocation_immutability".to_string());
                constitutional_requirements.push("b_mode_purity".to_string());
            }
            ComponentId::ConstitutionalRuleEngine => {
                required_structs.push("ConstitutionalRuleEngine".to_string());
                required_traits.push("ConstitutionalRuleAnalyzer".to_string());
                required_invariants.push("CONSTITUTIONAL_RULE_ENFORCEMENT".to_string());
                required_property_tests.push("test_constitutional_rule_enforcement".to_string());
                constitutional_requirements.push("rule_immutability".to_string());
                constitutional_requirements.push("b_mode_purity".to_string());
            }
            ComponentId::DeterminismEngine => {
                required_structs.push("DeterminismEngine".to_string());
                required_traits.push("DeterminismAnalyzer".to_string());
                required_invariants.push("DETERMINISTIC_ALLOCATION_REPRODUCIBILITY".to_string());
                required_property_tests.push("test_deterministic_allocation_reproducibility".to_string());
                constitutional_requirements.push("determinism_guarantee".to_string());
                constitutional_requirements.push("b_mode_purity".to_string());
            }
            _ => {
                // Default requirements for other components
                constitutional_requirements.push("b_mode_purity".to_string());
            }
        }

        // Check if component has specification in catalog
        let gate_e_readiness = self.catalog.specifications.contains_key(&component)
            && !required_structs.is_empty()
            && !required_invariants.is_empty();

        ContractRequirementsReport {
            component,
            required_structs,
            required_traits,
            required_invariants,
            required_property_tests,
            constitutional_requirements,
            gate_e_readiness,
            analysis_timestamp: DeterministicClock::new().now(),
        }
    }

    fn contract_catalog(&self) -> &ContractCatalog {
        &self.catalog
    }
}

/// Helper function to create a contract specification
pub fn create_contract_specification(
    contract_id: String,
    target_component: ComponentId,
    struct_specifications: Vec<StructSpecification>,
    trait_specifications: Vec<TraitSpecification>,
    invariant_specifications: Vec<InvariantSpecification>,
) -> ContractSpecification {
    ContractSpecification {
        contract_id,
        target_component,
        struct_specifications,
        trait_specifications,
        invariant_specifications,
        property_test_specifications: Vec::new(),
        performance_specifications: PerformanceSpecification {
            allocation_overhead_target: 1.0,
            reuse_rate_capability: 80.0,
            spill_overhead_threshold: 5.0,
            measurement_methods: Vec::new(),
            validation_frequency: ValidationFrequency::PerCommit,
        },
    }
}

/// Helper function to create a struct specification
pub fn create_struct_specification(
    name: String,
    fields: Vec<FieldSpecification>,
    documentation: String,
) -> StructSpecification {
    StructSpecification {
        name,
        fields,
        visibility: VisibilitySpecification::Public,
        documentation,
        constitutional_constraints: Vec::new(),
    }
}

/// Helper function to create an invariant specification
pub fn create_invariant_specification(
    invariant_id: String,
    description: String,
    formal_specification: String,
    constitutional_level: ConstitutionalLevel,
) -> InvariantSpecification {
    InvariantSpecification {
        invariant_id,
        description,
        formal_specification,
        validation_method: ValidationMethodSpecification::PropertyTest("Property test".to_string()),
        violation_consequences: Vec::new(),
        constitutional_level,
    }
}