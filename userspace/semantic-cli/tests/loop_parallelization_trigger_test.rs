//! Integration test for Loop Parallelization Trigger Logic (Task 7.1)
//!
//! This test validates the complete parallelization decision workflow:
//! 1. Safety analysis of loop bodies
//! 2. Parallelization trigger logic based on safety and loop type
//! 3. Integration between loop engine components

use semantic_cli::loop_engine::{
    LoopEngine, LoopAnalysisContext, ParallelizationDecision, SafetyClass
};
use semantic_cli::bcib::{
    LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType, CollectionType, OperandRef
};
use semantic_cli::types::SourceLocation;

/// Test the complete parallelization decision workflow
#[test]
fn test_parallelization_trigger_workflow() {
    let mut loop_engine = LoopEngine::new();
    
    // Test 1: Large Safe For Loop - Should be parallelized
    let large_for_loop = LoopInstruction::For {
        id: LoopID::new("large-safe-for".to_string()),
        range: LoopRange::new(0, 1000, 1), // 1000 iterations
        iterator_var: "i".to_string(),
        body: "accumulator = accumulator + i * 2".to_string(), // Safe computation
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Analyze safety
    let mut context = LoopAnalysisContext::new();
    context.add_loop_variable("i".to_string(), "number".to_string());
    context.add_loop_variable("accumulator".to_string(), "number".to_string());
    
    let safety_result = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + i * 2", 
        &context
    ).unwrap();
    
    assert_eq!(safety_result.classification, SafetyClass::Safe);
    
    // Check parallelization decision
    let decision = loop_engine.should_parallelize_loop(&large_for_loop, &safety_result);
    
    assert!(decision.is_parallel());
    assert_eq!(decision.iteration_count(), Some(1000));
    assert!(decision.parallel_benefit().unwrap() > 0.0);
    
    // Test 2: While Loop - Should never be parallelized (constitutional rule)
    let while_loop = LoopInstruction::While {
        id: LoopID::new("while-loop".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "accumulator = accumulator + 1".to_string(), // Safe computation
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let safety_result = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + 1", 
        &context
    ).unwrap();
    
    assert_eq!(safety_result.classification, SafetyClass::Safe);
    
    // While loops should never be parallelized
    let decision = loop_engine.should_parallelize_loop(&while_loop, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("While loops are excluded"));
    
    // Test 3: Unsafe For Loop - Should be sequential
    let unsafe_for_loop = LoopInstruction::For {
        id: LoopID::new("unsafe-for".to_string()),
        range: LoopRange::new(0, 1000, 1), // 1000 iterations
        iterator_var: "i".to_string(),
        body: "file_write('output.txt', i); accumulator = accumulator + i".to_string(), // Has I/O
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let safety_result = loop_engine.analyze_loop_safety(
        "file_write('output.txt', i); accumulator = accumulator + i", 
        &context
    ).unwrap();
    
    assert_eq!(safety_result.classification, SafetyClass::Unsafe);
    
    // Unsafe loops should be sequential
    let decision = loop_engine.should_parallelize_loop(&unsafe_for_loop, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("Unsafe for parallelization"));
    
    // Test 4: Small Safe For Loop - Should be sequential (below threshold)
    let small_for_loop = LoopInstruction::For {
        id: LoopID::new("small-safe-for".to_string()),
        range: LoopRange::new(0, 50, 1), // 50 iterations (below 100 threshold)
        iterator_var: "i".to_string(),
        body: "accumulator = accumulator + i".to_string(), // Safe computation
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let safety_result = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + i", 
        &context
    ).unwrap();
    
    assert_eq!(safety_result.classification, SafetyClass::Safe);
    
    // Small loops should be sequential (below threshold)
    let decision = loop_engine.should_parallelize_loop(&small_for_loop, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("below minimum threshold"));
    
    // Test 5: Large Safe ForEach Loop - Should be parallelized
    let large_array = (0..200).map(|i| Value::Number(i as f64)).collect();
    
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("large-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(large_array)),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "accumulator = accumulator + item".to_string(), // Safe computation
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let mut foreach_context = LoopAnalysisContext::new();
    foreach_context.add_loop_variable("item".to_string(), "number".to_string());
    foreach_context.add_loop_variable("accumulator".to_string(), "number".to_string());
    
    let safety_result = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + item", 
        &foreach_context
    ).unwrap();
    
    assert_eq!(safety_result.classification, SafetyClass::Safe);
    
    // Large safe ForEach loops should be parallelized
    let decision = loop_engine.should_parallelize_loop(&foreach_loop, &safety_result);
    
    assert!(decision.is_parallel());
    assert_eq!(decision.iteration_count(), Some(200));
    assert!(decision.parallel_benefit().unwrap() > 0.0);
    
    // Test 6: Dynamic Collection ForEach - Should be sequential (unknown size)
    let dynamic_foreach = LoopInstruction::ForEach {
        id: LoopID::new("dynamic-foreach".to_string()),
        collection: OperandRef::Field("dynamic_array".to_string()), // Dynamic size
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "accumulator = accumulator + item".to_string(), // Safe computation
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let safety_result = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + item", 
        &foreach_context
    ).unwrap();
    
    assert_eq!(safety_result.classification, SafetyClass::Safe);
    
    // Dynamic collections should be sequential (unknown size)
    let decision = loop_engine.should_parallelize_loop(&dynamic_foreach, &safety_result);
    
    assert!(decision.is_sequential());
    assert!(decision.sequential_reason().unwrap().contains("Dynamic iteration count"));
}

/// Test static iteration count analysis
#[test]
fn test_static_iteration_count_analysis() {
    let loop_engine = LoopEngine::new();
    
    // Test For loop with known range
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 100, 1), // 100 iterations
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let count = loop_engine.get_static_iteration_count(&for_loop);
    assert_eq!(count, Some(100));
    
    // Test ForEach loop with literal array
    let array = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(array)),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let count = loop_engine.get_static_iteration_count(&foreach_loop);
    assert_eq!(count, Some(3));
    
    // Test While loop (should always be None)
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let count = loop_engine.get_static_iteration_count(&while_loop);
    assert_eq!(count, None);
}

/// Test parallelization benefit estimation
#[test]
fn test_parallelization_benefit_estimation() {
    let mut loop_engine = LoopEngine::new();
    
    // Create a large safe For loop
    let large_for_loop = LoopInstruction::For {
        id: LoopID::new("benefit-test".to_string()),
        range: LoopRange::new(0, 5000, 1), // 5000 iterations
        iterator_var: "i".to_string(),
        body: "accumulator = accumulator + i".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let mut context = LoopAnalysisContext::new();
    context.add_loop_variable("i".to_string(), "number".to_string());
    context.add_loop_variable("accumulator".to_string(), "number".to_string());
    
    let safety_result = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + i", 
        &context
    ).unwrap();
    
    let decision = loop_engine.should_parallelize_loop(&large_for_loop, &safety_result);
    
    if let ParallelizationDecision::Parallel { estimated_benefit, .. } = decision {
        // 5000 iterations should have significant benefit (between 0.4 and 0.6)
        assert!(estimated_benefit > 0.4);
        assert!(estimated_benefit < 0.6);
    } else {
        panic!("Expected parallel decision for large safe loop");
    }
}

/// Test integration with safety analyzer caching
#[test]
fn test_safety_analysis_caching_integration() {
    let mut loop_engine = LoopEngine::new();
    
    let for_loop = LoopInstruction::For {
        id: LoopID::new("cache-test".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "accumulator = accumulator + i".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let mut context = LoopAnalysisContext::new();
    context.add_loop_variable("i".to_string(), "number".to_string());
    context.add_loop_variable("accumulator".to_string(), "number".to_string());
    
    // First analysis - should be a cache miss
    let safety_result1 = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + i", 
        &context
    ).unwrap();
    
    // Second analysis - should be a cache hit
    let safety_result2 = loop_engine.analyze_loop_safety(
        "accumulator = accumulator + i", 
        &context
    ).unwrap();
    
    // Results should be identical
    assert_eq!(safety_result1.classification, safety_result2.classification);
    assert_eq!(safety_result1.cache_key, safety_result2.cache_key);
    
    // Both should result in the same parallelization decision
    let decision1 = loop_engine.should_parallelize_loop(&for_loop, &safety_result1);
    let decision2 = loop_engine.should_parallelize_loop(&for_loop, &safety_result2);
    
    assert_eq!(decision1.is_parallel(), decision2.is_parallel());
    assert_eq!(decision1.iteration_count(), decision2.iteration_count());
    
    // Check cache statistics
    let cache_stats = loop_engine.get_safety_cache_stats();
    assert!(cache_stats.hit_count > 0); // Should have at least one cache hit
    assert!(cache_stats.entries > 0); // Should have cached entries
}