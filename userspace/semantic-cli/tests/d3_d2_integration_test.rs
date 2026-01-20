//! Integration tests for D3 Loop Support with D2 Parallelism System
//!
//! This test suite verifies that the D3 Loop Support system correctly integrates
//! with the D2 Parallelism Architecture to provide safe, deterministic parallel
//! loop execution.
//!
//! # Test Coverage
//!
//! - Parallelization trigger logic (Phase 7.1)
//! - Deterministic partitioning (Phase 7.2)
//! - Stable index mapping and result collection (Phase 7.3)
//! - Constitutional compliance verification
//! - End-to-end parallel loop execution

use semantic_cli::loop_engine::{
    LoopEngine, D2LoopIntegration, DeterministicPartitioner, StableIndexMapper,
    SafetyAnalysisResult, SafetyClass
};
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
        reason: "No side effects detected".to_string(),
        side_effects: vec![],
        dependencies: vec![],
        cache_key: "test-safe-cache-key".to_string(),
    }
}

fn create_unsafe_safety_result() -> SafetyAnalysisResult {
    SafetyAnalysisResult {
        classification: SafetyClass::Unsafe,
        reason: "Side effects detected".to_string(),
        side_effects: vec![],
        dependencies: vec![],
        cache_key: "test-unsafe-cache-key".to_string(),
    }
}

#[test]
fn test_parallelization_trigger_logic_while_loop_excluded() {
    // Test Phase 7.1: While loops are NEVER parallelized (Constitutional rule)
    let integration = D2LoopIntegration::new();
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();

    let decision = integration.should_parallelize_loop(&while_loop, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("While loops are excluded"));
}

#[test]
fn test_parallelization_trigger_logic_unsafe_loop_excluded() {
    // Test Phase 7.1: Unsafe loops are excluded from parallelization
    let integration = D2LoopIntegration::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_unsafe_safety_result();

    let decision = integration.should_parallelize_loop(&for_loop, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("Unsafe for parallelization"));
}

#[test]
fn test_parallelization_trigger_logic_small_loop_excluded() {
    // Test Phase 7.1: Small loops below threshold are excluded
    let integration = D2LoopIntegration::new();
    let small_for_loop = LoopInstruction::For {
        id: LoopID::new("test-small-for".to_string()),
        range: LoopRange::new(0, 50, 1), // Only 50 iterations
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();

    let decision = integration.should_parallelize_loop(&small_for_loop, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("below minimum threshold"));
}

#[test]
fn test_parallelization_trigger_logic_large_safe_for_loop_accepted() {
    // Test Phase 7.1: Large safe For loops are accepted for parallelization
    let integration = D2LoopIntegration::new();
    let large_for_loop = LoopInstruction::For {
        id: LoopID::new("test-large-for".to_string()),
        range: LoopRange::new(0, 1000, 1), // 1000 iterations
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();

    let decision = integration.should_parallelize_loop(&large_for_loop, &safety_result);
    
    assert!(decision.is_parallel());
    assert_eq!(decision.iteration_count(), Some(1000));
    assert!(decision.parallel_benefit().unwrap() > 0.0);
}

#[test]
fn test_parallelization_trigger_logic_foreach_loop_with_literal_array() {
    // Test Phase 7.1: ForEach loops with literal arrays are accepted
    let integration = D2LoopIntegration::new();
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(
            (0..200).map(|i| Value::Number(i as f64)).collect()
        )),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    let safety_result = create_safe_safety_result();

    let decision = integration.should_parallelize_loop(&foreach_loop, &safety_result);
    
    assert!(decision.is_parallel());
    assert_eq!(decision.iteration_count(), Some(200));
}

#[test]
fn test_deterministic_partitioning_same_inputs_same_outputs() {
    // Test Phase 7.2: Same iteration count → same partitions (Constitutional requirement)
    let integration = D2LoopIntegration::new();
    
    // Test multiple times with same inputs
    let partitions1 = integration.partition_iterations_deterministic(1000, 4);
    let partitions2 = integration.partition_iterations_deterministic(1000, 4);
    let partitions3 = integration.partition_iterations_deterministic(1000, 4);
    
    // Should produce identical partitions
    assert_eq!(partitions1, partitions2);
    assert_eq!(partitions2, partitions3);
    assert!(!partitions1.is_empty());
}

#[test]
fn test_deterministic_partitioning_completeness() {
    // Test Phase 7.2: All iterations covered exactly once
    let integration = D2LoopIntegration::new();
    let partitions = integration.partition_iterations_deterministic(1000, 4);
    
    // Verify all iterations are covered exactly once
    let mut total_iterations = 0;
    let mut last_end = 0;
    
    for partition in &partitions {
        assert_eq!(partition.start_iteration, last_end);
        assert!(partition.is_valid());
        total_iterations += partition.iteration_count;
        last_end = partition.end_iteration;
    }
    
    assert_eq!(total_iterations, 1000);
    assert_eq!(last_end, 1000);
}

#[test]
fn test_deterministic_partitioning_available_parallelism_as_upper_bound() {
    // Test Phase 7.2: Available parallelism is upper bound, not semantic input
    let integration = D2LoopIntegration::new();
    
    // More parallelism than needed
    let partitions1 = integration.partition_iterations_deterministic(100, 10);
    // Less parallelism than optimal
    let partitions2 = integration.partition_iterations_deterministic(10000, 2);
    
    // Verify partitions are created appropriately
    assert!(!partitions1.is_empty());
    assert!(!partitions2.is_empty());
    assert_eq!(partitions2.len(), 2); // Limited by available parallelism
}

#[test]
fn test_stable_index_mapping_for_loop() {
    // Test Phase 7.3: Stable index mapping for For loops
    let mapper = StableIndexMapper::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(10, 20, 2), // start=10, step=2
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    // Test stable mapping: same iteration → same input data
    for i in 0..5 {
        let data1 = mapper.get_input_data_for_iteration(&for_loop, i).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&for_loop, i).unwrap();
        let data3 = mapper.get_input_data_for_iteration(&for_loop, i).unwrap();
        
        assert_eq!(data1, data2);
        assert_eq!(data2, data3);
        
        // Verify correct range calculation
        let expected_value = 10.0 + (i as f64 * 2.0);
        assert_eq!(data1, Value::Number(expected_value));
    }
}

#[test]
fn test_stable_index_mapping_foreach_loop() {
    // Test Phase 7.3: Stable index mapping for ForEach loops
    let mapper = StableIndexMapper::new();
    let test_array = vec![
        Value::Number(100.0),
        Value::Number(200.0),
        Value::Number(300.0),
        Value::Number(400.0),
    ];
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(test_array.clone())),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    // Test stable mapping for ForEach loop
    for i in 0..4 {
        let data1 = mapper.get_input_data_for_iteration(&foreach_loop, i).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&foreach_loop, i).unwrap();
        let data3 = mapper.get_input_data_for_iteration(&foreach_loop, i).unwrap();
        
        assert_eq!(data1, data2);
        assert_eq!(data2, data3);
        assert_eq!(data1, test_array[i as usize]);
    }
}

#[test]
fn test_stable_index_mapping_while_loop() {
    // Test Phase 7.3: Stable index mapping for While loops
    let mapper = StableIndexMapper::new();
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    // Test stable mapping for While loop (uses iteration index)
    for i in 0..10 {
        let data1 = mapper.get_input_data_for_iteration(&while_loop, i).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&while_loop, i).unwrap();
        let data3 = mapper.get_input_data_for_iteration(&while_loop, i).unwrap();
        
        assert_eq!(data1, data2);
        assert_eq!(data2, data3);
        assert_eq!(data1, Value::Number(i as f64));
    }
}

#[test]
fn test_stable_mapping_verification() {
    // Test Phase 7.3: Stable mapping verification system
    let integration = D2LoopIntegration::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 100, 1),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    let verification = integration.verify_stable_mapping(&for_loop, 50).unwrap();
    
    assert!(verification.is_stable());
    assert_eq!(verification.total_tested, 50);
    assert_eq!(verification.stable_mappings, 50);
    assert_eq!(verification.unstable_mappings, 0);
    assert!(verification.errors.is_empty());
    assert_eq!(verification.stability_ratio(), 1.0);
}

#[test]
fn test_mapping_strategy_analysis() {
    // Test Phase 7.3: Index mapping strategy analysis
    let integration = D2LoopIntegration::new();
    
    // For loop strategy
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(5, 15, 3),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
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
    
    // ForEach loop strategy
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])),
        collection_type: CollectionType::List,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let foreach_strategy = integration.analyze_mapping_strategy(&foreach_loop);
    match foreach_strategy {
        semantic_cli::loop_engine::IndexMappingStrategy::Collection { collection_size, collection_type } => {
            assert_eq!(collection_size, 3);
            assert_eq!(collection_type, "List");
        }
        _ => panic!("Expected Collection strategy for ForEach loop"),
    }
    
    // While loop strategy
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let while_strategy = integration.analyze_mapping_strategy(&while_loop);
    assert_eq!(while_strategy, semantic_cli::loop_engine::IndexMappingStrategy::Iteration);
}

#[test]
fn test_mapping_cache_creation_and_usage() {
    // Test Phase 7.3: Mapping cache for efficient repeated access
    let integration = D2LoopIntegration::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 20, 1),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    let cache = integration.create_mapping_cache(&for_loop, 20).unwrap();
    
    assert_eq!(cache.max_iterations(), 20);
    
    // Test cache lookups
    for i in 0..20 {
        let cached_data = cache.get_input_data(i).unwrap();
        assert_eq!(*cached_data, Value::Number(i as f64));
    }
    
    let stats = cache.cache_stats();
    assert_eq!(stats.cached_entries, 20);
    assert_eq!(stats.max_iterations, 20);
    assert_eq!(stats.cache_hit_ratio, 1.0);
}

#[test]
fn test_deterministic_partitioner_standalone() {
    // Test Phase 7.2: Standalone deterministic partitioner
    let partitioner = DeterministicPartitioner::new();
    
    // Test deterministic partitioning
    let partitions1 = partitioner.partition_iterations(1000, 4).unwrap();
    let partitions2 = partitioner.partition_iterations(1000, 4).unwrap();
    
    assert_eq!(partitions1, partitions2);
    
    // Test partition analysis
    let analysis = partitioner.analyze_partitioning(1000, 4).unwrap();
    assert_eq!(analysis.total_iterations, 1000);
    assert!(analysis.partition_count > 0);
    assert!(analysis.chunk_size > 0);
    assert_eq!(analysis.available_parallelism, 4);
    assert!(analysis.load_balance.balance_ratio > 0.0);
    assert!(analysis.load_balance.balance_ratio <= 1.0);
}

#[test]
fn test_constitutional_compliance_iteration_limit_exactness() {
    // Test Constitutional requirement: Iteration limit exactness
    let integration = D2LoopIntegration::new();
    
    // Test that iteration count never exceeds limit
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 10000, 1), // Would be 10000 iterations
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
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
    
    let static_count = integration.get_static_iteration_count(&for_loop);
    assert_eq!(static_count, Some(500)); // Should be limited to 500
}

#[test]
fn test_constitutional_compliance_deterministic_partitioning() {
    // Test Constitutional requirement: Deterministic partitioning
    let integration = D2LoopIntegration::new();
    
    // Test that same iteration count produces same partitions across different machines
    // (simulated by multiple calls)
    for total_iterations in [100, 500, 1000, 5000] {
        for available_parallelism in [1, 2, 4, 8] {
            let partitions1 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            let partitions2 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            let partitions3 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            
            assert_eq!(partitions1, partitions2);
            assert_eq!(partitions2, partitions3);
        }
    }
}

#[test]
fn test_constitutional_compliance_stable_index_mapping() {
    // Test Constitutional requirement: Stable index mapping
    let mapper = StableIndexMapper::new();
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };

    // Test that iteration i always processes same input data
    for global_iteration in 0..100 {
        let data1 = mapper.get_input_data_for_iteration(&for_loop, global_iteration).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&for_loop, global_iteration).unwrap();
        let data3 = mapper.get_input_data_for_iteration(&for_loop, global_iteration).unwrap();
        
        assert_eq!(data1, data2);
        assert_eq!(data2, data3);
        assert_eq!(data1, Value::Number(global_iteration as f64));
    }
}

#[test]
fn test_end_to_end_integration() {
    // Test end-to-end integration of D3 with D2 system
    let loop_engine = LoopEngine::new();
    
    // Create a large safe For loop
    let large_for_loop = LoopInstruction::For {
        id: LoopID::new("test-large-for".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Test safety analysis
    let safety_result = create_safe_safety_result();
    
    // Test parallelization decision
    let decision = loop_engine.should_parallelize_loop(&large_for_loop, &safety_result);
    assert!(decision.is_parallel());
    
    // Test static iteration count
    let static_count = loop_engine.get_static_iteration_count(&large_for_loop);
    assert_eq!(static_count, Some(1000));
    
    // Test deterministic partitioning
    let partitions = loop_engine.partition_iterations_deterministic(1000, 4);
    assert!(!partitions.is_empty());
    
    // Verify partition completeness
    let mut total_iterations = 0;
    for partition in &partitions {
        assert!(partition.is_valid());
        total_iterations += partition.iteration_count;
    }
    assert_eq!(total_iterations, 1000);
}

#[test]
fn test_property_determinism_across_calls() {
    // Property test: Same inputs always produce same outputs
    let integration = D2LoopIntegration::new();
    
    let test_cases = [
        (100, 2),
        (500, 4),
        (1000, 8),
        (5000, 16),
    ];
    
    for (total_iterations, available_parallelism) in test_cases {
        // Test multiple calls with same inputs
        let partitions1 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
        let partitions2 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
        let partitions3 = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
        
        assert_eq!(partitions1, partitions2);
        assert_eq!(partitions2, partitions3);
    }
}

#[test]
fn test_property_completeness_across_partition_sizes() {
    // Property test: All iterations covered exactly once
    let integration = D2LoopIntegration::new();
    
    for total_iterations in [1, 10, 100, 1000, 10000] {
        for available_parallelism in [1, 2, 4, 8] {
            let partitions = integration.partition_iterations_deterministic(total_iterations, available_parallelism);
            
            let mut covered_iterations = 0;
            let mut last_end = 0;
            
            for partition in &partitions {
                assert_eq!(partition.start_iteration, last_end);
                assert!(partition.is_valid());
                covered_iterations += partition.iteration_count;
                last_end = partition.end_iteration;
            }
            
            assert_eq!(covered_iterations, total_iterations);
            assert_eq!(last_end, total_iterations);
        }
    }
}

#[test]
fn test_property_stable_mapping_consistency() {
    // Property test: Stable mapping consistency across loop types
    let mapper = StableIndexMapper::new();
    
    // Test For loop
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 100, 1),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    for i in 0..100 {
        let data1 = mapper.get_input_data_for_iteration(&for_loop, i).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&for_loop, i).unwrap();
        assert_eq!(data1, data2);
    }
    
    // Test ForEach loop
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(
            (0..100).map(|i| Value::Number(i as f64)).collect()
        )),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    for i in 0..100 {
        let data1 = mapper.get_input_data_for_iteration(&foreach_loop, i).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&foreach_loop, i).unwrap();
        assert_eq!(data1, data2);
    }
    
    // Test While loop
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: create_test_loop_config(),
        location: SourceLocation::new(1, 1, 0),
    };
    
    for i in 0..100 {
        let data1 = mapper.get_input_data_for_iteration(&while_loop, i).unwrap();
        let data2 = mapper.get_input_data_for_iteration(&while_loop, i).unwrap();
        assert_eq!(data1, data2);
    }
}