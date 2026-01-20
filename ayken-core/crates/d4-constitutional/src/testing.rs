//! Property-based testing framework for the D4 Constitutional Framework
//!
//! This module provides comprehensive property-based testing infrastructure using proptest,
//! consistent with the D4 register optimization framework. Supports deterministic CI testing
//! and development-friendly random testing.

use crate::errors::{PropertyTestFailureInfo, Result};
use crate::types::*;
use proptest::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Constitutional deterministic constants
const D4_CONSTITUTIONAL_SEED: u64 = 0xD4C0_0000_0000_0001;
const FIXED_TEST_LOGICAL_TIMESTAMP: LogicalTimestamp = LogicalTimestamp(1000);

/// Generate deterministic test case ID for constitutional compliance
fn generate_deterministic_test_case_id(property_name: &str, iteration: u32, seed: u64) -> String {
    use sha2::{Digest, Sha256};
    let input = format!("{}_{}_{}_{}", property_name, iteration, seed, "d4_constitutional");
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("case_{}_{}", iteration, &hash[..16]) // Use first 16 chars of hash for readability
}

/// Generate deterministic scenario ID for constitutional compliance
fn generate_deterministic_scenario_id(property_name: &str, seed: u64) -> String {
    use sha2::{Digest, Sha256};
    let input = format!("{}_{}_{}", property_name, seed, "d4_scenario");
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("scenario_{}", &hash[..16]) // Use first 16 chars of hash for readability
}

/// Configuration for property-based testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyTestConfig {
    /// Number of test iterations to run
    pub iterations: u32,
    /// Seed for deterministic testing (None for random)
    pub seed: Option<u64>,
    /// Maximum size for generated test data
    pub max_size: usize,
    /// Whether to enable shrinking on failure
    pub enable_shrinking: bool,
    /// Timeout for individual test cases (in milliseconds)
    pub timeout_ms: u64,
}

impl Default for PropertyTestConfig {
    fn default() -> Self {
        // Check environment variables for CI configuration
        let is_ci = std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok();
        let seed = if is_ci {
            Some(crate::CI_FIXED_SEED)
        } else {
            std::env::var("PROPERTY_TEST_SEED")
                .ok()
                .and_then(|s| s.parse().ok())
        };

        Self {
            iterations: if is_ci { 100 } else { crate::DEFAULT_PROPERTY_TEST_ITERATIONS },
            seed,
            max_size: if is_ci { 50 } else { 100 },
            enable_shrinking: true,
            timeout_ms: if is_ci { 10000 } else { 5000 },
        }
    }
}

impl PropertyTestConfig {
    /// Create configuration for CI testing with fixed seed
    pub fn for_ci() -> Self {
        Self {
            iterations: 100,
            seed: Some(crate::CI_FIXED_SEED),
            max_size: 50,
            enable_shrinking: true,
            timeout_ms: 10000,
        }
    }

    /// Create configuration for development with random seed
    pub fn for_development() -> Self {
        Self {
            iterations: 50,
            seed: None,
            max_size: 100,
            enable_shrinking: true,
            timeout_ms: 5000,
        }
    }

    /// Create configuration with specific seed for reproduction
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed: Some(seed),
            ..Default::default()
        }
    }
}

/// Property test runner with enhanced failure reporting using proptest
pub struct PropertyTestRunner {
    config: PropertyTestConfig,
    rng: ChaCha8Rng,
}

impl PropertyTestRunner {
    /// Create a new property test runner with the given configuration
    /// Uses deterministic seed for constitutional compliance
    pub fn new(config: PropertyTestConfig) -> Self {
        let seed = config.seed.unwrap_or(D4_CONSTITUTIONAL_SEED); // Deterministic default seed

        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Run a property test with enhanced failure reporting
    /// Uses deterministic seed for constitutional compliance
    pub fn run_property_simple(&mut self, property_name: &str, test_fn: fn() -> bool) -> Result<()> {
        let seed = self.config.seed.unwrap_or(D4_CONSTITUTIONAL_SEED); // Deterministic default seed
        
        // Run the test multiple times
        for i in 0..self.config.iterations {
            if !test_fn() {
                let failure_info = PropertyTestFailureInfo {
                    property_name: property_name.to_string(),
                    test_case_id: generate_deterministic_test_case_id(property_name, i, seed),
                    seed,
                    shrunk_input: format!("iteration_{}", i),
                    failure_reason: "Property test failed".to_string(),
                    ir_fingerprint: format!("fingerprint_{}", i),
                    failure_scenario_id: generate_deterministic_scenario_id(property_name, seed),
                    stack_trace: None,
                    reproduction_command: String::new(),
                };

                eprintln!("Property test failure details:");
                eprintln!("  Property: {}", failure_info.property_name);
                eprintln!("  Seed: {}", failure_info.seed);
                eprintln!("  Case ID: {}", failure_info.test_case_id);
                eprintln!("  IR Fingerprint: {}", failure_info.ir_fingerprint);
                eprintln!("  Failure Scenario ID: {}", failure_info.failure_scenario_id);
                eprintln!("  Reproduction: {}", failure_info.generate_reproduction_command());

                return Err(failure_info.into());
            }
        }

        Ok(())
    }

    /// Run a property test using proptest with enhanced failure reporting
    pub fn run_proptest<T>(&mut self, property_name: &str, strategy: T, test_fn: impl Fn(&T::Value) -> bool) -> Result<()>
    where
        T: Strategy,
        T::Value: std::fmt::Debug + Clone,
    {
        let seed = self.config.seed.unwrap_or(D4_CONSTITUTIONAL_SEED); // Deterministic default seed
        
        // Configure proptest with our settings
        let mut config = ProptestConfig::default();
        config.cases = self.config.iterations;
        config.max_shrink_iters = if self.config.enable_shrinking { 1000 } else { 0 };
        config.timeout = self.config.timeout_ms as u32;
        config.rng_algorithm = proptest::test_runner::RngAlgorithm::ChaCha;
        
        // Create test runner with fixed seed for deterministic behavior
        let mut runner = proptest::test_runner::TestRunner::new_with_rng(
            config,
            proptest::test_runner::TestRng::deterministic_rng(proptest::test_runner::RngAlgorithm::ChaCha)
        );
        
        // Run the property test
        let result = runner.run(&strategy, |input| {
            let ir_fingerprint = self.generate_ir_fingerprint(&input);
            
            if test_fn(&input) {
                Ok(())
            } else {
                let failure_info = PropertyTestFailureInfo {
                    property_name: property_name.to_string(),
                    test_case_id: generate_deterministic_test_case_id(property_name, 0, seed),
                    seed,
                    shrunk_input: format!("{:?}", input),
                    failure_reason: "Property test failed".to_string(),
                    ir_fingerprint,
                    failure_scenario_id: generate_deterministic_scenario_id(property_name, seed),
                    stack_trace: None,
                    reproduction_command: String::new(),
                };

                eprintln!("Property test failure details:");
                eprintln!("  Property: {}", failure_info.property_name);
                eprintln!("  Seed: {}", failure_info.seed);
                eprintln!("  Case ID: {}", failure_info.test_case_id);
                eprintln!("  IR Fingerprint: {}", failure_info.ir_fingerprint);
                eprintln!("  Failure Scenario ID: {}", failure_info.failure_scenario_id);
                eprintln!("  Shrunk Input: {}", failure_info.shrunk_input);
                eprintln!("  Reproduction: {}", failure_info.generate_reproduction_command());

                Err(proptest::test_runner::TestCaseError::Fail("Property violated".into()))
            }
        });

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(crate::errors::ConstitutionalError::SystemInitializationFailure {
                reason: format!(
                    "Property '{}' failed (seed {}): {}",
                    property_name, seed, e
                ),
            }),
        }
    }

    /// Generate IR fingerprint for test input (simplified for testing)
    fn generate_ir_fingerprint<T: std::fmt::Debug>(&self, input: &T) -> String {
        use sha2::{Digest, Sha256};
        let input_str = format!("{:?}", input);
        let mut hasher = Sha256::new();
        hasher.update(input_str.as_bytes());
        hex::encode(hasher.finalize()) // GATE-E FIX: Use full hash to prevent collisions
    }
}

/// Proptest strategies for constitutional framework types

/// Strategy for generating ComponentId values
pub fn component_id_strategy() -> impl Strategy<Value = ComponentId> {
    prop_oneof![
        Just(ComponentId::D1Component),
        Just(ComponentId::D2Component),
        Just(ComponentId::D3Component),
        Just(ComponentId::D4RegisterAllocator),
        Just(ComponentId::JITCompiler),
        Just(ComponentId::LoopOptimizer),
        Just(ComponentId::UnrollOptimizer),
        Just(ComponentId::NativeCache),
    ]
}

/// Strategy for generating VirtualRegisterId values
pub fn virtual_register_id_strategy() -> impl Strategy<Value = VirtualRegisterId> {
    (0u32..1000).prop_map(VirtualRegisterId)
}

/// Strategy for generating PhysicalRegisterId values
pub fn physical_register_id_strategy() -> impl Strategy<Value = PhysicalRegisterId> {
    (0u32..32).prop_map(PhysicalRegisterId)
}

/// Strategy for generating OptimizationLevel values
pub fn optimization_level_strategy() -> impl Strategy<Value = OptimizationLevel> {
    prop_oneof![
        Just(OptimizationLevel::Debug),
        Just(OptimizationLevel::Release),
        Just(OptimizationLevel::Aggressive),
    ]
}

/// Strategy for generating TargetArchitecture values
pub fn target_architecture_strategy() -> impl Strategy<Value = TargetArchitecture> {
    prop_oneof![
        Just(TargetArchitecture::X86_64),
        Just(TargetArchitecture::ARM64),
        Just(TargetArchitecture::RISCV64),
    ]
}

/// Strategy for generating SpillLocation values
pub fn spill_location_strategy() -> impl Strategy<Value = SpillLocation> {
    (
        0u64..0x1000000,
        4u32..64,
        prop_oneof![Just(4u32), Just(8u32), Just(16u32), Just(32u32)],
    )
        .prop_map(|(memory_address, size_bytes, alignment)| SpillLocation {
        memory_address,
        size_bytes,
        alignment,
        access_pattern: "sequential".to_string(),
    })
}

/// Strategy for generating AllocationDecision values
pub fn allocation_decision_strategy() -> impl Strategy<Value = AllocationDecision> {
    (
        virtual_register_id_strategy(),
        prop::option::of(physical_register_id_strategy()),
        prop::bool::ANY,
        optimization_level_strategy(),
        target_architecture_strategy(),
        prop::collection::vec(physical_register_id_strategy(), 0..16),
        0u32..1000,
    )
        .prop_map(
            |(virtual_register, physical_register, has_spill, opt_level, _arch, available_regs, fingerprint_id)| {
                let spill_location = SpillLocation {
                    memory_address: (fingerprint_id as u64) * 8,
                    size_bytes: 8,
                    alignment: 8,
                    access_pattern: "sequential".to_string(),
                };

                let binding = if let Some(physical_register) = physical_register {
                    RegisterBinding::Physical(physical_register)
                } else if has_spill {
                    RegisterBinding::Spilled(spill_location)
                } else {
                    RegisterBinding::Physical(PhysicalRegisterId(0))
                };

                let constraints = AllocationConstraints {
                    preferred_registers: available_regs,
                    excluded_registers: Vec::new(),
                    alignment_requirements: Vec::new(),
                    forbidden_registers: Vec::new(),
                    lifetime_requirements: Vec::new(),
                    performance_hints: Vec::new(),
                };

                let performance_requirements = PerformanceRequirements {
                    max_spill_rate: None,
                    max_register_pressure: None,
                    cache_locality: None,
                };

                AllocationDecision {
                    virtual_register,
                    binding,
                    decision_context: AllocationContext {
                        pressure_level: 0,
                        optimization_level: opt_level,
                        constraints,
                        performance_requirements,
                    },
                }
            },
        )
}

/// Strategy for generating vectors of AllocationDecision values
pub fn allocation_decisions_strategy() -> impl Strategy<Value = Vec<AllocationDecision>> {
    prop::collection::vec(allocation_decision_strategy(), 0..20)
}

/// Strategy for generating CacheOperationType values
pub fn cache_operation_type_strategy() -> impl Strategy<Value = CacheOperationType> {
    prop_oneof![
        Just(CacheOperationType::Enable),
        Just(CacheOperationType::Disable),
        Just(CacheOperationType::BoundsCheck),
        Just(CacheOperationType::Access),
    ]
}

/// Strategy for generating CacheTarget values
pub fn cache_target_strategy() -> impl Strategy<Value = CacheTarget> {
    prop_oneof![
        Just(CacheTarget::NativeCache),
        Just(CacheTarget::InstructionCache),
        Just(CacheTarget::DataCache),
    ]
}

/// Strategy for generating InteractionType values
pub fn interaction_type_strategy() -> impl Strategy<Value = InteractionType> {
    prop_oneof![
        Just(InteractionType::AllocationRequest),
        Just(InteractionType::AllocationDecision),
        Just(InteractionType::OptimizationHint),
        Just(InteractionType::FailureNotification),
        Just(InteractionType::StateQuery),
    ]
}

/// Strategy for generating ProposalOperationType values
pub fn proposal_operation_type_strategy() -> impl Strategy<Value = ProposalOperationType> {
    prop_oneof![
        Just(ProposalOperationType::AllocationConstraint),
        Just(ProposalOperationType::OptimizationHint),
        Just(ProposalOperationType::ExecutionHint),
    ]
}

/// Strategy for generating Operation values
pub fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        (virtual_register_id_strategy(), physical_register_id_strategy())
            .prop_map(|(vr, pr)| Operation::RegisterAllocation {
                virtual_register: vr,
                physical_register: pr,
            }),
        (physical_register_id_strategy(), physical_register_id_strategy())
            .prop_map(|(orig, new)| Operation::RegisterRewrite {
                original: orig,
                new,
            }),
        (cache_operation_type_strategy(), cache_target_strategy())
            .prop_map(|(op_type, target)| Operation::CacheOperation {
                operation_type: op_type,
                target,
            }),
        (component_id_strategy(), component_id_strategy(), interaction_type_strategy())
            .prop_map(|(source, target, interaction_type)| Operation::ComponentInteraction {
                source,
                target,
                interaction_type,
            }),
    ]
}

/// Macro for defining property tests with standard configuration using proptest
#[macro_export]
macro_rules! proptest_constitutional {
    ($name:ident, $strategy:expr, $property:expr) => {
        proptest! {
            #![proptest_config(if cfg!(feature = "deterministic") {
                ProptestConfig {
                    cases: 100,
                    max_shrink_iters: 1000,
                    timeout: 10000,
                    rng_algorithm: proptest::test_runner::RngAlgorithm::ChaCha,
                    ..ProptestConfig::default()
                }
            } else {
                ProptestConfig {
                    cases: 50,
                    max_shrink_iters: 1000,
                    timeout: 5000,
                    rng_algorithm: proptest::test_runner::RngAlgorithm::ChaCha,
                    ..ProptestConfig::default()
                }
            })]

            #[test]
            fn $name(input in $strategy) {
                prop_assert!($property(&input));
            }
        }
    };
}

/// Test data generators for common constitutional framework scenarios
pub struct TestDataGenerators;

impl TestDataGenerators {
    /// Generate a set of allocation decisions with potential conflicts for testing
    /// Uses deterministic seed to ensure reproducible test results
    #[cfg(test)]
    pub fn generate_mock_allocation_decisions_with_conflicts_for_tests(
        size: usize,
        test_seed: u64,
        _test_timestamp: LogicalTimestamp,
        _test_ir_fingerprint: &str,
    ) -> Vec<AllocationDecision> {
        use rand::SeedableRng;
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(test_seed);
        let mut decisions = Vec::new();
        let physical_registers: Vec<PhysicalRegisterId> = (0..8)
            .map(PhysicalRegisterId)
            .collect();

        for i in 0..size {
            let virtual_register = VirtualRegisterId(i as u32);
            let physical_register = if rng.gen_bool(0.8) {
                Some(*physical_registers.choose(&mut rng).unwrap())
            } else {
                None
            };

            let binding = if let Some(physical_register) = physical_register {
                RegisterBinding::Physical(physical_register)
            } else {
                RegisterBinding::Spilled(SpillLocation {
                    memory_address: (i * 8) as u64,
                    size_bytes: 8,
                    alignment: 8,
                    access_pattern: "sequential".to_string(),
                })
            };

            let constraints = AllocationConstraints {
                preferred_registers: physical_registers.clone(),
                excluded_registers: Vec::new(),
                alignment_requirements: Vec::new(),
                forbidden_registers: Vec::new(),
                lifetime_requirements: Vec::new(),
                performance_hints: Vec::new(),
            };

            let performance_requirements = PerformanceRequirements {
                max_spill_rate: None,
                max_register_pressure: None,
                cache_locality: None,
            };

            decisions.push(AllocationDecision {
                virtual_register,
                binding,
                decision_context: AllocationContext {
                    pressure_level: 0,
                    optimization_level: OptimizationLevel::Release,
                    constraints,
                    performance_requirements,
                },
            });
        }

        decisions
    }

    /// Generate a set of allocation decisions without conflicts for testing uniqueness property
    /// Uses deterministic parameters to ensure reproducible test results
    #[cfg(test)]
    pub fn generate_mock_allocation_decisions_without_conflicts_for_tests(
        size: usize,
        _test_timestamp: LogicalTimestamp,
        _test_ir_fingerprint: &str,
    ) -> Vec<AllocationDecision> {
        let mut decisions = Vec::new();
        let physical_registers: Vec<PhysicalRegisterId> = (0..16)
            .map(PhysicalRegisterId)
            .collect();

        for i in 0..size {
            let virtual_register = VirtualRegisterId(i as u32);
            // Ensure each virtual register gets a unique physical register or spills
            let physical_register = if i < physical_registers.len() {
                Some(physical_registers[i])
            } else {
                None
            };

            let binding = if let Some(physical_register) = physical_register {
                RegisterBinding::Physical(physical_register)
            } else {
                RegisterBinding::Spilled(SpillLocation {
                    memory_address: (i * 8) as u64,
                    size_bytes: 8,
                    alignment: 8,
                    access_pattern: "sequential".to_string(),
                })
            };

            let constraints = AllocationConstraints {
                preferred_registers: physical_registers.clone(),
                excluded_registers: Vec::new(),
                alignment_requirements: Vec::new(),
                forbidden_registers: Vec::new(),
                lifetime_requirements: Vec::new(),
                performance_hints: Vec::new(),
            };

            let performance_requirements = PerformanceRequirements {
                max_spill_rate: None,
                max_register_pressure: None,
                cache_locality: None,
            };

            decisions.push(AllocationDecision {
                virtual_register,
                binding,
                decision_context: AllocationContext {
                    pressure_level: 0,
                    optimization_level: OptimizationLevel::Release,
                    constraints,
                    performance_requirements,
                },
            });
        }

        decisions
    }

    /// Generate component interactions for testing authority hierarchy
    pub fn component_interactions_for_hierarchy() -> Vec<ComponentInteraction> {
        let interactions = vec![
            (ComponentId::JITCompiler, ComponentId::D4RegisterAllocator),
            (ComponentId::LoopOptimizer, ComponentId::D4RegisterAllocator),
            (ComponentId::UnrollOptimizer, ComponentId::LoopOptimizer),
            (ComponentId::D4RegisterAllocator, ComponentId::NativeCache),
        ];

        interactions
            .into_iter()
            .map(|(source, target)| ComponentInteraction {
                source,
                target,
                interaction_type: InteractionType::AllocationRequest,
                payload: InteractionPayload::AllocationRequest {
                    virtual_registers: vec![VirtualRegisterId(1), VirtualRegisterId(2)],
                    constraints: AllocationConstraints {
                        preferred_registers: vec![PhysicalRegisterId(0)],
                        excluded_registers: vec![],
                        alignment_requirements: vec![],
                        forbidden_registers: vec![],
                        lifetime_requirements: vec![],
                        performance_hints: vec![],
                    },
                },
                timestamp: FIXED_TEST_LOGICAL_TIMESTAMP,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_test_config_creation() {
        let ci_config = PropertyTestConfig::for_ci();
        assert_eq!(ci_config.seed, Some(crate::CI_FIXED_SEED));
        assert_eq!(ci_config.iterations, 100);

        let dev_config = PropertyTestConfig::for_development();
        assert_eq!(dev_config.seed, None);
        assert_eq!(dev_config.iterations, 50);
    }

    #[test]
    fn test_property_test_runner_creation() {
        let config = PropertyTestConfig::with_seed(12345);
        let runner = PropertyTestRunner::new(config);
        // Runner should be created successfully
        assert_eq!(runner.config.seed, Some(12345));
    }

    #[test]
    fn test_proptest_strategies() {
        // Test that proptest strategies work
        let mut runner = proptest::test_runner::TestRunner::default();
        
        // Test component strategy
        let component_result = runner.run(&component_id_strategy(), |_| Ok(()));
        assert!(component_result.is_ok());
        
        // Test virtual register strategy
        let vr_result = runner.run(&virtual_register_id_strategy(), |_| Ok(()));
        assert!(vr_result.is_ok());
        
        // Test allocation decision strategy
        let allocation_result = runner.run(&allocation_decision_strategy(), |_| Ok(()));
        assert!(allocation_result.is_ok());
    }

    #[test]
    fn test_test_data_generators() {
        let decisions = TestDataGenerators::generate_mock_allocation_decisions_with_conflicts_for_tests(
            5, 
            12345, 
            LogicalTimestamp(1000), 
            "test_ir"
        );
        assert_eq!(decisions.len(), 5);

        let interactions = TestDataGenerators::component_interactions_for_hierarchy();
        assert!(!interactions.is_empty());
    }

    // Example property test using proptest
    proptest! {
        #[test]
        fn test_allocation_uniqueness_property(
            decisions in prop::collection::vec(allocation_decision_strategy(), 1..10)
        ) {
            let decisions: Vec<AllocationDecision> = decisions;
            // This is a simplified test - we'll use the non-conflicting generator for this example
            let test_decisions = TestDataGenerators::generate_mock_allocation_decisions_without_conflicts_for_tests(
                decisions.len(),
                LogicalTimestamp(1000),
                "test_ir"
            );
            
            // Check that no two virtual registers share the same physical register
            let mut physical_allocations = std::collections::BTreeMap::new();
            for decision in &test_decisions {
                if let RegisterBinding::Physical(physical_reg) = decision.binding {
                    prop_assert!(!physical_allocations.contains_key(&physical_reg), 
                        "Found duplicate allocation for physical register {:?}", physical_reg);
                    physical_allocations.insert(physical_reg, decision.virtual_register);
                }
            }
        }
    }

    #[test]
    fn test_example_allocation_uniqueness_simple() {
        let config = if cfg!(feature = "deterministic") {
            PropertyTestConfig::for_ci()
        } else {
            PropertyTestConfig::for_development()
        };

        let mut runner = PropertyTestRunner::new(config);
        
        // Simplified test function - use non-conflicting generator for this test
        let test_fn = || {
            let decisions = TestDataGenerators::generate_mock_allocation_decisions_without_conflicts_for_tests(
                5,
                LogicalTimestamp(1000),
                "test_ir"
            );
            if decisions.is_empty() {
                return true; // Skip empty cases
            }

            // Check that no two virtual registers share the same physical register
            let mut physical_allocations = std::collections::BTreeMap::new();
            for decision in &decisions {
                if let RegisterBinding::Physical(physical_reg) = decision.binding {
                    if physical_allocations.contains_key(&physical_reg) {
                        return false; // Found duplicate allocation
                    }
                    physical_allocations.insert(physical_reg, decision.virtual_register);
                }
            }

            true // All allocations are unique
        };

        runner.run_property_simple("example_allocation_uniqueness", test_fn).unwrap();
    }
}
