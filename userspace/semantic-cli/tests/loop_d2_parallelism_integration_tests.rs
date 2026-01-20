//! D2 Parallelism Integration Tests for D3 Loop Support (Task 12.3)
//!
//! This test suite verifies the integration between D3 Loop Support and the D2 Parallelism
//! system, focusing on parallel loop execution, deterministic result collection, and
//! fallback to sequential execution.
//!
//! # ARCHITECTURAL BOUNDARIES (CRITICAL)
//!
//! These tests follow D2 integration principles:
//! - **BLACK-BOX TESTING**: Test decisions, not internal heuristics
//! - **DECISION LEVEL**: Test "did it choose parallel?" not "why did it choose parallel?"
//! - **CORRECTNESS FOCUS**: Test deterministic behavior and constitutional compliance
//! - **API BOUNDARIES**: Only test public integration points, not private implementation details
//!
//! ## What We Test (D2 Integration Layer)
//! ✅ Parallelization decisions (parallel vs sequential)
//! ✅ Deterministic partitioning (same inputs → same partitions)
//! ✅ Constitutional compliance (bounded iteration, determinism)
//! ✅ Stable index mapping (iteration i → same input data)
//! ✅ Fallback mechanisms (unsafe loops → sequential execution)
//!
//! ## What We DON'T Test (Internal D3 Heuristics)
//! ❌ estimate_parallelization_benefit() - internal heuristic, not API
//! ❌ calculate_deterministic_chunk_size() - implementation detail, not contract
//! ❌ Internal decision algorithms - these belong in D3 property tests
//!
//! # Test Coverage
//!
//! - Parallel loop execution workflow
//! - Deterministic result collection from parallel partitions
//! - Fallback to sequential execution for unsafe loops
//! - D2 system integration points validation
//! - Constitutional compliance in parallel execution

use semantic_cli::loop_engine::d2_integration::D2LoopIntegration;
use semantic_cli::loop_engine::{SafetyAnalysisResult, SafetyClass};
use semantic_cli::bcib::{
    LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType, 
    BudgetMeasurement, ErrorRecoveryPolicy, OperandRef, CollectionType
};
use semantic_cli::types::SourceLocation;

fn create_test_loop_config() -> LoopConfig {
    LoopConfig {
        iteration_limit: 10000,
        budget_timeout: 100000,
        budget_measurement: BudgetMeasurement::IterationCount,
        initial_accumulator: Value::Number(0.0),
        accumulator_type: ValueType::Number,
        error_recovery: ErrorRecoveryPolicy::Abort,
    }
}

fn create_safe_safety_result() -> SafetyAnalysisResult {
    SafetyAnalysisResult {
        classification: SafetyClass::Safe,
        reason: "No side effects or dependencies detected".to_string(),
        side_effects: vec![],
        dependencies: vec![],
        cache_key: "test-safe-cache-key".to_string(),
    }
}

fn create_unsafe_safety_result() -> SafetyAnalysisResult {
    SafetyAnalysisResult {
        classification: SafetyClass::Unsafe,
        reason: "Side effects detected - I/O operations".to_string(),
        side_effects: vec![],
        dependencies: vec![],
        cache_key: "test-unsafe-cache-key".to_string(),
    }
}

#[test]
fn test_parallel_loop_execution_workflow_for_loop() {
    // Test parallel execution workflow for For loops
    let integration = D2LoopIntegration::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-parallel-for".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "accumulator + i".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();

    // Verify parallelization decision
    let decision = integration.should_parallelize_loop(&for_loop, &safety_result);
    assert!(decision.is_parallel());
    assert_eq!(decision.iteration_count(), Some(1000));

    // Verify static iteration count
    let static_count = integration.get_static_iteration_count(&for_loop);
    assert_eq!(static_count, Some(1000));

    // Verify deterministic partitioning
    let partitions = integration.partition_iterations_deterministic(1000, 4);
    assert!(!partitions.is_empty());
    assert!(partitions.len() <= 4); // Respects available parallelism

    // Verify partition completeness
    let mut total_iterations = 0;
    for partition in &partitions {
        assert!(partition.is_valid());
        total_iterations += partition.iteration_count;
    }
    assert_eq!(total_iterations, 1000);
}

#[test]
fn test_parallel_loop_execution_workflow_foreach_loop() {
    // Test parallel execution workflow for ForEach loops
    let integration = D2LoopIntegration::new();
    let test_array: Vec<Value> = (0..500).map(|i| Value::Number(i as f64)).collect();
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-parallel-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(test_array)),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "accumulator + item".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();

    // Verify parallelization decision
    let decision = integration.should_parallelize_loop(&foreach_loop, &safety_result);
    assert!(decision.is_parallel());
    assert_eq!(decision.iteration_count(), Some(500));

    // Verify static iteration count
    let static_count = integration.get_static_iteration_count(&foreach_loop);
    assert_eq!(static_count, Some(500));

    // Verify deterministic partitioning
    let partitions = integration.partition_iterations_deterministic(500, 4);
    assert!(!partitions.is_empty());

    // Verify partition completeness
    let mut total_iterations = 0;
    for partition in &partitions {
        assert!(partition.is_valid());
        total_iterations += partition.iteration_count;
    }
    assert_eq!(total_iterations, 500);
}

#[test]
fn test_deterministic_result_collection_from_parallel_partitions() {
    // Test that parallel execution produces deterministic results
    let integration = D2LoopIntegration::new();
    
    // Test multiple executions with same inputs
    for total_iterations in [100, 500, 1000] {
        for available_parallelism in [2, 4, 8] {
            let partitions1 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            let partitions2 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            let partitions3 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            
            // Same inputs should produce identical partitions
            assert_eq!(partitions1, partitions2);
            assert_eq!(partitions2, partitions3);
            
            // Verify deterministic ordering (partition 0, 1, 2, ...)
            for (i, partition) in partitions1.iter().enumerate() {
                assert_eq!(partition.partition_id, i);
            }
        }
    }
}

#[test]
fn test_deterministic_result_collection_ordering() {
    // Test that results are collected in deterministic order (partition 0, 1, 2, ...)
    let integration = D2LoopIntegration::new();
    let partitions = integration.partition_iterations_deterministic(1000, 4);
    
    // Verify partitions are ordered by ID
    for (i, partition) in partitions.iter().enumerate() {
        assert_eq!(partition.partition_id, i);
        if i > 0 {
            assert_eq!(partition.start_iteration, partitions[i-1].end_iteration);
        }
    }
    
    // Verify no gaps or overlaps
    let mut last_end = 0;
    for partition in &partitions {
        assert_eq!(partition.start_iteration, last_end);
        last_end = partition.end_iteration;
    }
    assert_eq!(last_end, 1000);
}

#[test]
fn test_fallback_to_sequential_execution_unsafe_loops() {
    // Test fallback to sequential execution for unsafe loops
    let integration = D2LoopIntegration::new();
    let unsafe_for_loop = LoopInstruction::For {
        id: LoopID::new("test-unsafe-for".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "print(i); accumulator + i".to_string(), // I/O operation makes it unsafe
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let unsafe_safety_result = create_unsafe_safety_result();

    // Verify fallback to sequential execution
    let decision = integration.should_parallelize_loop(&unsafe_for_loop, &unsafe_safety_result);
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("Unsafe for parallelization"));
}

#[test]
fn test_fallback_to_sequential_execution_while_loops() {
    // Test fallback to sequential execution for While loops (constitutional rule)
    let integration = D2LoopIntegration::new();
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "accumulator + 1".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safe_safety_result = create_safe_safety_result();

    // Verify While loops are never parallelized (constitutional rule)
    let decision = integration.should_parallelize_loop(&while_loop, &safe_safety_result);
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("While loops are excluded"));
}

#[test]
fn test_fallback_to_sequential_execution_small_loops() {
    // Test fallback to sequential execution for small loops below threshold
    let integration = D2LoopIntegration::new();
    let small_for_loop = LoopInstruction::For {
        id: LoopID::new("test-small-for".to_string()),
        range: LoopRange::new(0, 50, 1), // Only 50 iterations
        iterator_var: "i".to_string(),
        body: "accumulator + i".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safe_safety_result = create_safe_safety_result();

    // Verify small loops fall back to sequential execution
    let decision = integration.should_parallelize_loop(&small_for_loop, &safe_safety_result);
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("below minimum threshold"));
}

#[test]
fn test_d2_system_integration_points_partitioning() {
    // Test D2 system integration points - partitioning
    let integration = D2LoopIntegration::new();
    
    // Test that partitioning integrates correctly with D2 system
    let partitions = integration.partition_iterations_deterministic(1000, 4);
    
    // Verify D2 partitioning requirements
    assert!(!partitions.is_empty());
    assert!(partitions.len() <= 4); // Respects available parallelism
    
    // Verify each partition is valid for D2 execution
    for partition in &partitions {
        assert!(partition.is_valid());
        assert!(partition.iteration_count > 0);
        assert!(partition.start_iteration < partition.end_iteration);
    }
}

#[test]
fn test_d2_system_integration_points_stable_mapping() {
    // Test D2 system integration points - stable index mapping
    let integration = D2LoopIntegration::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-mapping-for".to_string()),
        range: LoopRange::new(10, 110, 2), // start=10, step=2
        iterator_var: "i".to_string(),
        body: "accumulator + i".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    // Test stable mapping verification
    let verification = integration.verify_stable_mapping(&for_loop, 50).unwrap();
    assert!(verification.is_stable());
    assert_eq!(verification.total_tested, 50);
    assert_eq!(verification.stable_mappings, 50);
    assert_eq!(verification.unstable_mappings, 0);
    assert_eq!(verification.stability_ratio(), 1.0);
}

#[test]
fn test_d2_system_integration_points_mapping_strategy() {
    // Test D2 system integration points - mapping strategy analysis
    let integration = D2LoopIntegration::new();
    
    // Test For loop mapping strategy
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-strategy-for".to_string()),
        range: LoopRange::new(5, 25, 3),
        iterator_var: "i".to_string(),
        body: "accumulator + i".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let for_strategy = integration.analyze_mapping_strategy(&for_loop);
    match for_strategy {
        semantic_cli::loop_engine::IndexMappingStrategy::Range { start, step } => {
            assert_eq!(start, 5);
            assert_eq!(step, 3);
        }
        _ => panic!("Expected Range strategy for For loop"),
    }
    
    // Test ForEach loop mapping strategy
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-strategy-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ])),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "accumulator + item".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let foreach_strategy = integration.analyze_mapping_strategy(&foreach_loop);
    match foreach_strategy {
        semantic_cli::loop_engine::IndexMappingStrategy::Collection { collection_size, collection_type } => {
            assert_eq!(collection_size, 4);
            assert_eq!(collection_type, "Array");
        }
        _ => panic!("Expected Collection strategy for ForEach loop"),
    }
}

#[test]
fn test_d2_system_integration_points_mapping_cache() {
    // Test D2 system integration points - mapping cache
    let integration = D2LoopIntegration::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-cache-for".to_string()),
        range: LoopRange::new(0, 100, 1),
        iterator_var: "i".to_string(),
        body: "accumulator + i".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    // Test mapping cache creation and usage
    let cache = integration.create_mapping_cache(&for_loop, 100).unwrap();
    assert_eq!(cache.max_iterations(), 100);
    
    // Test cache lookups
    for i in 0..100 {
        let cached_data = cache.get_input_data(i).unwrap();
        assert_eq!(*cached_data, Value::Number(i as f64));
    }
    
    // Verify cache statistics
    let stats = cache.cache_stats();
    assert_eq!(stats.cached_entries, 100);
    assert_eq!(stats.max_iterations, 100);
    assert_eq!(stats.cache_hit_ratio, 1.0);
}

#[test]
fn test_constitutional_compliance_parallel_execution() {
    // Test constitutional compliance in parallel execution
    let integration = D2LoopIntegration::new();
    
    // Test iteration limit exactness in parallel context
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-constitutional-for".to_string()),
        range: LoopRange::new(0, 10000, 1), // Would be 10000 iterations
        iterator_var: "i".to_string(),
        body: "accumulator + i".to_string(),
        config: LoopConfig {
            iteration_limit: 500, // But limit to 500
            budget_timeout: 100000,
            budget_measurement: BudgetMeasurement::IterationCount,
            initial_accumulator: Value::Number(0.0),
            accumulator_type: ValueType::Number,
            error_recovery: ErrorRecoveryPolicy::Abort,
        },
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Verify iteration limit is respected in parallel context
    let static_count = integration.get_static_iteration_count(&for_loop);
    assert_eq!(static_count, Some(500)); // Should be limited to 500
    
    // Verify partitioning respects the limit
    let partitions = integration.partition_iterations_deterministic(500, 4);
    let mut total_iterations = 0;
    for partition in &partitions {
        total_iterations += partition.iteration_count;
    }
    assert_eq!(total_iterations, 500); // Never exceeds limit
}

#[test]
fn test_constitutional_compliance_deterministic_execution() {
    // Test constitutional compliance - deterministic execution
    let integration = D2LoopIntegration::new();
    
    // Test that same inputs always produce same outputs (determinism)
    for total_iterations in [100, 500, 1000] {
        for available_parallelism in [2, 4, 8] {
            // Multiple executions with same inputs
            let partitions1 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            let partitions2 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            let partitions3 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            
            // Must produce identical results
            assert_eq!(partitions1, partitions2);
            assert_eq!(partitions2, partitions3);
        }
    }
}

#[test]
fn test_constitutional_compliance_bounded_iteration() {
    // Test constitutional compliance - bounded iteration only
    let integration = D2LoopIntegration::new();
    
    // Test that all loop types respect iteration bounds
    let test_cases = [
        (100, 4),
        (1000, 8),
        (5000, 16),
    ];
    
    for (total_iterations, available_parallelism) in test_cases {
        let partitions = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
        
        // Verify bounded iteration - no partition exceeds total
        let mut covered_iterations = 0;
        for partition in &partitions {
            assert!(partition.start_iteration < total_iterations);
            assert!(partition.end_iteration <= total_iterations);
            covered_iterations += partition.iteration_count;
        }
        
        // Verify exact coverage (no more, no less)
        assert_eq!(covered_iterations, total_iterations);
    }
}

#[test]
fn test_end_to_end_parallel_execution_workflow() {
    // Test complete end-to-end parallel execution workflow (BLACK-BOX D2 INTEGRATION)
    let integration = D2LoopIntegration::new();
    
    // Create a large safe For loop suitable for parallelization
    let parallel_for_loop = LoopInstruction::For {
        id: LoopID::new("test-end-to-end-for".to_string()),
        range: LoopRange::new(0, 2000, 1),
        iterator_var: "i".to_string(),
        body: "accumulator + (i * 2)".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();
    
    // Step 1: Test DECISION LEVEL - did it choose parallel?
    let decision = integration.should_parallelize_loop(&parallel_for_loop, &safety_result);
    assert!(decision.is_parallel(), "Large safe For loop should be parallelized");
    
    // Step 2: Test CORRECTNESS - can it determine iteration count?
    let static_count = integration.get_static_iteration_count(&parallel_for_loop);
    assert_eq!(static_count, Some(2000), "Static iteration count should be deterministic");
    
    // Step 3: Test DETERMINISM - same inputs produce same partitions
    let partitions1 = integration.partition_iterations_deterministic(2000, 8);
    let partitions2 = integration.partition_iterations_deterministic(2000, 8);
    assert_eq!(partitions1, partitions2, "Deterministic partitioning must be consistent");
    
    // Step 4: Test COMPLETENESS - all iterations covered exactly once
    let mut total_iterations = 0;
    let mut last_end = 0;
    for partition in &partitions1 {
        assert_eq!(partition.start_iteration, last_end, "Partitions must be contiguous");
        assert!(partition.is_valid(), "Each partition must be valid");
        total_iterations += partition.iteration_count;
        last_end = partition.end_iteration;
    }
    assert_eq!(total_iterations, 2000, "All iterations must be covered exactly once");
    assert_eq!(last_end, 2000, "Final partition must end at total iterations");
    
    // Step 5: Test STABILITY - mapping is consistent
    let verification = integration.verify_stable_mapping(&parallel_for_loop, 100).unwrap();
    assert!(verification.is_stable(), "Index mapping must be stable");
    assert_eq!(verification.stability_ratio(), 1.0, "All mappings must be stable");
}

#[test]
fn test_property_parallel_determinism_consistency() {
    // Property test: Parallel execution determinism consistency
    let integration = D2LoopIntegration::new();
    
    // Test that deterministic partitioning is consistent across multiple calls
    let test_cases = [
        (100, 2),
        (500, 4),
        (1000, 8),
        (2000, 16),
    ];
    
    for (total_iterations, available_parallelism) in test_cases {
        // Execute multiple times
        let results: Vec<_> = (0..10)
            .map(|_| integration.partition_iterations_deterministic(total_iterations, available_parallelism))
            .collect();
        
        // All results should be identical
        for i in 1..results.len() {
            assert_eq!(results[0], results[i]);
        }
    }
}

#[test]
fn test_property_parallel_completeness_guarantee() {
    // Property test: Parallel execution completeness guarantee
    let integration = D2LoopIntegration::new();
    
    // Test that all iterations are covered exactly once across different configurations
    for total_iterations in [1, 10, 100, 1000, 5000] {
        for available_parallelism in [1, 2, 4, 8, 16] {
            let partitions = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            
            // Verify completeness
            let mut covered_iterations = 0;
            let mut iteration_set = std::collections::HashSet::new();
            
            for partition in &partitions {
                for iter in partition.start_iteration..partition.end_iteration {
                    assert!(iteration_set.insert(iter), "Iteration {} covered multiple times", iter);
                    covered_iterations += 1;
                }
            }
            
            assert_eq!(covered_iterations, total_iterations);
            assert_eq!(iteration_set.len(), total_iterations as usize);
        }
    }
}