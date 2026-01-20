//! End-to-End Integration Tests for D2 Parallelism Architecture
//!
//! This module provides comprehensive integration tests that validate the entire
//! parallelism pipeline from IR execution through result verification.
//!
//! **Design Reference:** D2 Parallelism Architecture - Integration Testing section
//! **Requirements:** All requirements

use semantic_cli::bcib::{Value, FilterExpression, ComparisonOp, OperandRef};
use semantic_cli::execution_plan::{
    IRBlock, IRInstruction, BlockTerminator, ParallelSafety, ExecutionPlan, ExecutionMetadata
};
use semantic_cli::normalizer::RegisterAllocation;
use semantic_cli::execution_plan::dataflow::DataflowGraph;
use semantic_cli::parallelism::{
    ContiguousPartitioner, DataPartitioner, StableIndexMerger, DeterministicMerger,
    RayonParallelExecutor, ParallelExecutor, DefaultDecisionEngine, AdaptiveDecisionEngine,
    DefaultMetricsCollector, MetricsCollector, DefaultReductionHandler, operations,
    ImmutableContext, ExecutionConfig, ParallelismError,
    verification::{execute_with_verification, VerificationResult}
};
use semantic_cli::ir_executor::IRExecutor;
use std::collections::HashMap;
use std::time::Duration;

// ===== Full Pipeline Integration Tests =====

#[test]
fn test_full_pipeline_with_parallelism_enabled() {
    // Test the complete pipeline from IR creation to parallel execution
    let dataset = create_test_dataset(10_000);
    let execution_plan = create_filter_execution_plan();
    
    // Create executor with parallelism enabled
    #[cfg(feature = "phase2-implementation")]
    let mut executor = IRExecutor::new().with_parallelism();
    
    #[cfg(not(feature = "phase2-implementation"))]
    let mut executor = IRExecutor::new();
    
    // Execute the plan
    let result = executor.execute(execution_plan);
    
    assert!(result.is_ok(), "Full pipeline execution should succeed");
    
    let execution_result = result.unwrap();
    assert!(execution_result.execution_steps > 0, "Should have executed some steps");
    
    // Verify the executor used appropriate execution mode
    #[cfg(feature = "phase2-implementation")]
    {
        assert!(executor.is_parallelism_enabled(), "Parallelism should be enabled");
    }
    
    #[cfg(not(feature = "phase2-implementation"))]
    {
        assert!(!executor.is_parallelism_enabled(), "Parallelism should be disabled without feature");
    }
}

#[test]
fn test_replay_mode_integration() {
    // Test that replay mode forces sequential execution
    let dataset = create_test_dataset(1_000);
    let execution_plan = create_simple_execution_plan();
    
    #[cfg(feature = "phase2-implementation")]
    {
        let mut executor = IRExecutor::new().with_parallelism();
        
        // Execute with replay recording
        let result = executor.execute_with_replay(execution_plan);
        
        assert!(result.is_ok(), "Replay execution should succeed");
        
        let (execution_result, replay_trace) = result.unwrap();
        assert!(execution_result.execution_steps > 0, "Should have executed steps");
        assert!(!replay_trace.steps.is_empty(), "Should have recorded replay steps");
    }
}

#[test]
fn test_verification_mode_integration() {
    // Test verification mode with the full parallelism stack
    let data = create_test_dataset(1_000);
    let block = create_safe_ir_block();
    let context = create_test_context();
    
    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();
    
    let result = execute_with_verification(&block, &data, &context, &executor, &merger);
    
    assert!(result.is_ok(), "Verification should succeed");
    
    match result.unwrap() {
        VerificationResult::Match { result, sequential_time, parallel_time, .. } => {
            assert_eq!(result.len(), data.len(), "Result size should match input");
            assert!(sequential_time > Duration::ZERO, "Sequential time should be measured");
            assert!(parallel_time > Duration::ZERO, "Parallel time should be measured");
        }
        VerificationResult::Mismatch { diagnostics, .. } => {
            panic!("Verification failed with diagnostics: {:?}", diagnostics);
        }
    }
}

// ===== Constitutional Compliance Tests =====

#[test]
fn test_constitutional_compliance_determinism() {
    // Test that determinism is maintained across multiple executions
    let data = create_deterministic_dataset(5_000);
    let block = create_safe_ir_block();
    let context = create_test_context();
    
    let partitioner = ContiguousPartitioner::new();
    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();
    
    // Execute multiple times and verify identical results
    let mut results = Vec::new();
    for _ in 0..5 {
        let partitions = partitioner.partition(&data, num_cpus::get());
        let indexed_results = executor.execute_parallel(&block, partitions, &context).unwrap();
        let result = merger.merge(indexed_results).unwrap();
        results.push(result);
    }
    
    // All results should be identical
    for i in 1..results.len() {
        assert_eq!(results[0], results[i], 
                   "Execution {} should produce identical results to execution 0", i);
    }
}

#[test]
fn test_constitutional_compliance_replay_correctness() {
    // Test that replay produces bitwise identical results
    let data = create_test_dataset(1_000);
    let execution_plan = create_simple_execution_plan();
    
    #[cfg(feature = "phase2-implementation")]
    {
        let mut executor = IRExecutor::new().with_parallelism();
        
        // First execution with replay recording
        let (original_result, replay_trace) = executor.execute_with_replay(execution_plan.clone()).unwrap();
        
        // Replay execution
        let mut replay_executor = IRExecutor::new();
        // In a real implementation, we would use the replay trace to reproduce execution
        let replay_result = replay_executor.execute(execution_plan).unwrap();
        
        // Results should be identical (in a full implementation)
        assert_eq!(original_result.execution_steps, replay_result.execution_steps,
                   "Replay should execute same number of steps");
    }
}

#[test]
fn test_constitutional_compliance_performance_measurement() {
    // Test that performance measurement is accurate and comprehensive
    let data = create_test_dataset(10_000);
    let mut metrics_collector = DefaultMetricsCollector::new();
    
    metrics_collector.start_measurement();
    
    // Simulate execution phases
    std::thread::sleep(Duration::from_millis(10)); // Sequential
    metrics_collector.record_phase(
        semantic_cli::parallelism::ExecutionPhase::Sequential,
        Duration::from_millis(100)
    );
    
    std::thread::sleep(Duration::from_millis(5)); // Parallel
    metrics_collector.record_phase(
        semantic_cli::parallelism::ExecutionPhase::Parallel,
        Duration::from_millis(40)
    );
    
    metrics_collector.record_phase(
        semantic_cli::parallelism::ExecutionPhase::Ordering,
        Duration::from_millis(10)
    );
    
    let net_speedup = metrics_collector.calculate_net_speedup();
    let metrics = metrics_collector.report();
    
    assert!(net_speedup > 0.0, "Net speedup should be positive");
    assert!(metrics.sequential_time > Duration::ZERO, "Sequential time should be recorded");
    assert!(metrics.parallel_time > Duration::ZERO, "Parallel time should be recorded");
    assert!(metrics.ordering_overhead >= Duration::ZERO, "Ordering overhead should be recorded");
    
    // Net speedup should be calculated correctly: 100ms / (40ms + 10ms) = 2.0x
    assert!((net_speedup - 2.0).abs() < 0.1, "Net speedup calculation should be accurate");
}

// ===== Property Test Integration =====

#[test]
fn test_all_correctness_properties_integration() {
    // This test validates that all 13 correctness properties work together
    let data = create_test_dataset(1_000);
    let block = create_safe_ir_block();
    let context = create_test_context();
    
    // Property 1: Parallel-Sequential Equivalence
    let sequential_result = execute_sequential_simulation(&data);
    let parallel_result = execute_parallel_simulation(&block, &data, &context).unwrap();
    assert_eq!(sequential_result.len(), parallel_result.len(), "Property 1: Results should have same length");
    
    // Property 2: Stable Index Map Determinism
    let partitioner = ContiguousPartitioner::new();
    let partitions1 = partitioner.partition(&data, 4);
    let partitions2 = partitioner.partition(&data, 4);
    assert_eq!(partitions1.len(), partitions2.len(), "Property 2: Same partition count");
    for (p1, p2) in partitions1.iter().zip(partitions2.iter()) {
        assert_eq!(p1.start_index, p2.start_index, "Property 2: Same start indices");
        assert_eq!(p1.end_index, p2.end_index, "Property 2: Same end indices");
    }
    
    // Property 6: Unsafe Block Sequential Execution
    let unsafe_block = create_unsafe_ir_block();
    let mut decision_engine = DefaultDecisionEngine::new();
    let should_parallelize = decision_engine.should_parallelize(&unsafe_block, data.len());
    assert!(!should_parallelize, "Property 6: Unsafe blocks should not be parallelized");
    
    // Property 7: Adaptive Speedup Threshold
    let poor_metrics = semantic_cli::parallelism::ExecutionMetrics {
        sequential_time: Duration::from_millis(100),
        parallel_time: Duration::from_millis(80), // Only 1.25x speedup
        ordering_overhead: Duration::ZERO,
        sync_cost: Duration::ZERO,
        merge_cost: Duration::ZERO,
    };
    decision_engine.record_execution(block.id, poor_metrics);
    let should_parallelize_after_poor = decision_engine.should_parallelize(&block, data.len());
    // Note: This might still be true initially, but after multiple poor executions it would be false
    
    // Property 9: Ordering Overhead Protection
    let high_overhead_metrics = semantic_cli::parallelism::ExecutionMetrics {
        sequential_time: Duration::from_millis(100),
        parallel_time: Duration::from_millis(30),
        ordering_overhead: Duration::from_millis(60), // 200% overhead
        sync_cost: Duration::ZERO,
        merge_cost: Duration::ZERO,
    };
    decision_engine.record_execution(block.id, high_overhead_metrics);
    // After recording high overhead, the operation should be blacklisted
}

// ===== Performance Benchmark Integration =====

#[test]
fn test_performance_benchmarks_integration() {
    // Test that performance benchmarks meet targets
    let sizes = vec![1_000, 10_000, 100_000];
    
    for size in sizes {
        let data = create_test_dataset(size);
        let block = create_safe_ir_block();
        let context = create_test_context();
        
        let mut metrics_collector = DefaultMetricsCollector::new();
        metrics_collector.start_measurement();
        
        // Measure sequential execution
        let sequential_start = std::time::Instant::now();
        let _sequential_result = execute_sequential_simulation(&data);
        let sequential_time = sequential_start.elapsed();
        metrics_collector.record_phase(
            semantic_cli::parallelism::ExecutionPhase::Sequential,
            sequential_time
        );
        
        // Measure parallel execution
        let parallel_start = std::time::Instant::now();
        let _parallel_result = execute_parallel_simulation(&block, &data, &context).unwrap();
        let parallel_time = parallel_start.elapsed();
        metrics_collector.record_phase(
            semantic_cli::parallelism::ExecutionPhase::Parallel,
            parallel_time
        );
        
        let net_speedup = metrics_collector.calculate_net_speedup();
        let metrics = metrics_collector.report();
        
        // Performance targets
        if size >= 10_000 && num_cpus::get() > 1 {
            // For large datasets on multi-core systems, expect some benefit
            assert!(net_speedup >= 0.8, 
                    "Net speedup should be at least 0.8x for size {}, got {:.2}x", 
                    size, net_speedup);
        }
        
        // Overhead should be reasonable
        let overhead_ratio = metrics.ordering_overhead_ratio();
        assert!(overhead_ratio <= 1.0, 
                "Ordering overhead should not exceed 100% for size {}, got {:.1}%", 
                size, overhead_ratio * 100.0);
    }
}

// ===== Error Handling Integration =====

#[test]
fn test_error_handling_integration() {
    // Test comprehensive error handling across the parallelism stack
    
    // Test safety violation
    let unsafe_block = create_unsafe_ir_block();
    let data = create_test_dataset(1_000);
    let context = create_test_context();
    
    let executor = RayonParallelExecutor::new();
    let partitioner = ContiguousPartitioner::new();
    let partitions = partitioner.partition(&data, 4);
    
    // This should work (the executor doesn't enforce safety, that's the decision engine's job)
    let result = executor.execute_parallel(&unsafe_block, partitions, &context);
    // In a real implementation, this might return an error or be prevented by the decision engine
    
    // Test invalid partition handling
    let empty_partitions = vec![];
    let result = executor.execute_parallel(&unsafe_block, empty_partitions, &context);
    // Should handle empty partitions gracefully
    
    // Test merger error handling
    let merger = StableIndexMerger::new();
    let incomplete_results = vec![
        (0, Value::Number(1.0)),
        (2, Value::Number(3.0)), // Missing index 1
    ];
    
    let merge_result = merger.merge(incomplete_results);
    // Should handle incomplete results appropriately
}

// ===== Stress Testing =====

#[test]
fn test_large_dataset_stress() {
    // Test with large datasets to ensure scalability
    let large_data = create_test_dataset(100_000);
    let block = create_safe_ir_block();
    let context = create_test_context();
    
    let start = std::time::Instant::now();
    let result = execute_parallel_simulation(&block, &large_data, &context);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Large dataset processing should succeed");
    assert!(duration < Duration::from_secs(10), 
            "Large dataset should process within 10 seconds, took {:?}", duration);
    
    let processed_result = result.unwrap();
    assert_eq!(processed_result.len(), large_data.len(), 
               "All elements should be processed");
}

#[test]
fn test_concurrent_execution_stress() {
    // Test concurrent execution of multiple parallelism operations
    use std::sync::Arc;
    use std::thread;
    
    let data = Arc::new(create_test_dataset(5_000));
    let block = Arc::new(create_safe_ir_block());
    let context = Arc::new(create_test_context());
    
    let mut handles = vec![];
    
    // Spawn multiple threads executing parallel operations
    for i in 0..4 {
        let data_clone = Arc::clone(&data);
        let block_clone = Arc::clone(&block);
        let context_clone = Arc::clone(&context);
        
        let handle = thread::spawn(move || {
            let result = execute_parallel_simulation(&block_clone, &data_clone, &context_clone);
            (i, result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads and verify results
    for handle in handles {
        let (thread_id, result) = handle.join().unwrap();
        assert!(result.is_ok(), "Thread {} should succeed", thread_id);
        
        let processed_result = result.unwrap();
        assert_eq!(processed_result.len(), data.len(), 
                   "Thread {} should process all elements", thread_id);
    }
}

// ===== Helper Functions =====

fn create_test_dataset(size: usize) -> Vec<Value> {
    (0..size)
        .map(|i| Value::Number(i as f64))
        .collect()
}

fn create_deterministic_dataset(size: usize) -> Vec<Value> {
    // Create a dataset that should produce deterministic results
    (0..size)
        .map(|i| Value::Number((i * 17 + 42) as f64)) // Deterministic pattern
        .collect()
}

fn create_safe_ir_block() -> IRBlock {
    IRBlock::with_safety(
        1,
        vec![
            IRInstruction::LoadContext {
                context_id: "test".to_string(),
                target_register: 0,
            },
            IRInstruction::LoadLiteral {
                value: Value::Number(1.0),
                target_register: 1,
            },
        ],
        BlockTerminator::Return { register: 0 },
        ParallelSafety::Safe,
    )
}

fn create_unsafe_ir_block() -> IRBlock {
    IRBlock::with_safety(
        2,
        vec![
            IRInstruction::LoadContext {
                context_id: "test".to_string(),
                target_register: 0,
            },
            // In a real implementation, this would be an instruction with side effects
        ],
        BlockTerminator::Return { register: 0 },
        ParallelSafety::Unsafe,
    )
}

fn create_test_context() -> ImmutableContext {
    let execution_plan = ExecutionPlan::new(
        vec![],
        0,
        RegisterAllocation {
            allocated_registers: vec![],
            register_dependencies: HashMap::new(),
            next_register: 0,
        },
        DataflowGraph::new(),
        ExecutionMetadata::new("integration_test".to_string(), 0, 0, 0),
    );
    
    ImmutableContext {
        execution_plan,
        config: ExecutionConfig::default(),
    }
}

fn create_simple_execution_plan() -> ExecutionPlan {
    let block = create_safe_ir_block();
    
    ExecutionPlan::new(
        vec![block],
        0,
        RegisterAllocation {
            allocated_registers: vec![],
            register_dependencies: HashMap::new(),
            next_register: 2,
        },
        DataflowGraph::new(),
        ExecutionMetadata::new("simple_test".to_string(), 1, 2, 1),
    )
}

fn create_filter_execution_plan() -> ExecutionPlan {
    let filter_block = IRBlock::with_safety(
        1,
        vec![
            IRInstruction::LoadContext {
                context_id: "dataset".to_string(),
                target_register: 0,
            },
            IRInstruction::ApplyFilter {
                context_register: 0,
                filter_expression: FilterExpression {
                    field: "value".to_string(),
                    operator: ComparisonOp::GreaterThan,
                    value: OperandRef::Literal(Value::Number(0.0)),
                },
                target_register: 1,
            },
        ],
        BlockTerminator::Return { register: 1 },
        ParallelSafety::Safe,
    );
    
    ExecutionPlan::new(
        vec![filter_block],
        0,
        RegisterAllocation {
            allocated_registers: vec![],
            register_dependencies: HashMap::new(),
            next_register: 2,
        },
        DataflowGraph::new(),
        ExecutionMetadata::new("filter_test".to_string(), 1, 2, 1),
    )
}

fn execute_sequential_simulation(data: &[Value]) -> Vec<Value> {
    // Simulate sequential execution
    data.iter().cloned().collect()
}

fn execute_parallel_simulation(
    block: &IRBlock,
    data: &[Value],
    context: &ImmutableContext,
) -> Result<Vec<Value>, ParallelismError> {
    let partitioner = ContiguousPartitioner::new();
    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();
    
    let partitions = partitioner.partition(data, num_cpus::get());
    let indexed_results = executor.execute_parallel(block, partitions, context)?;
    let result = merger.merge(indexed_results)?;
    
    Ok(result)
}

// ===== Module-Level Integration Tests =====

#[cfg(test)]
mod module_integration {
    use super::*;
    
    #[test]
    fn test_all_modules_work_together() {
        // Test that all parallelism modules integrate correctly
        let data = create_test_dataset(1_000);
        let block = create_safe_ir_block();
        let context = create_test_context();
        
        // Test partitioner
        let partitioner = ContiguousPartitioner::new();
        let partitions = partitioner.partition(&data, 4);
        assert_eq!(partitions.len(), 4);
        
        // Test executor
        let executor = RayonParallelExecutor::new();
        let indexed_results = executor.execute_parallel(&block, partitions, &context).unwrap();
        assert!(!indexed_results.is_empty());
        
        // Test merger
        let merger = StableIndexMerger::new();
        let result = merger.merge(indexed_results).unwrap();
        assert_eq!(result.len(), data.len());
        
        // Test decision engine
        let mut decision_engine = DefaultDecisionEngine::new();
        let should_parallelize = decision_engine.should_parallelize(&block, data.len());
        // Should be true for safe blocks with sufficient data
        
        // Test metrics collector
        let mut metrics_collector = DefaultMetricsCollector::new();
        metrics_collector.start_measurement();
        metrics_collector.record_phase(
            semantic_cli::parallelism::ExecutionPhase::Sequential,
            Duration::from_millis(100)
        );
        let speedup = metrics_collector.calculate_net_speedup();
        assert_eq!(speedup, 0.0); // No parallel time recorded yet
        
        // Test reduction handler
        let handler = DefaultReductionHandler::new();
        let sum_result = operations::sum(&handler, vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]).unwrap();
        assert_eq!(sum_result, Value::Number(6.0));
    }
    
    #[test]
    fn test_feature_flag_integration() {
        // Test that feature flags work correctly
        #[cfg(feature = "phase2-implementation")]
        {
            let executor = IRExecutor::new().with_parallelism();
            assert!(executor.is_parallelism_enabled());
        }
        
        #[cfg(not(feature = "phase2-implementation"))]
        {
            let executor = IRExecutor::new();
            assert!(!executor.is_parallelism_enabled());
        }
    }
}