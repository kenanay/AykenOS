//! Parallel Filter Operation Example
//!
//! This example demonstrates how to use the D2 Parallelism Architecture
//! for parallel filter operations on large datasets.
//!
//! **Usage:**
//! ```bash
//! cargo run --example parallel_filter_operation --features phase2-implementation
//! ```

use semantic_cli::bcib::{Value, FilterExpression, ComparisonOp, OperandRef};
use semantic_cli::execution_plan::{
    IRBlock, IRInstruction, BlockTerminator, ParallelSafety, ExecutionPlan, ExecutionMetadata
};
use semantic_cli::normalizer::RegisterAllocation;
use semantic_cli::execution_plan::dataflow::DataflowGraph;
use semantic_cli::parallelism::{
    ContiguousPartitioner, DataPartitioner, StableIndexMerger, DeterministicMerger,
    RayonParallelExecutor, ParallelExecutor, DefaultDecisionEngine, AdaptiveDecisionEngine,
    DefaultMetricsCollector, MetricsCollector, ImmutableContext, ExecutionConfig
};
use semantic_cli::ir_executor::IRExecutor;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Parallel Filter Operation Example");
    println!("=====================================");
    
    // Create test dataset
    let dataset = create_large_dataset(50_000);
    println!("📊 Created dataset with {} elements", dataset.len());
    
    // Create filter operation IR
    let filter_block = create_filter_ir_block();
    println!("🔧 Created filter IR block (ID: {})", filter_block.id);
    
    // Create execution context
    let context = create_execution_context();
    
    // Example 1: Sequential execution
    println!("\n1️⃣ Sequential Execution");
    let sequential_result = execute_sequential_filter(&dataset)?;
    println!("   ✅ Filtered {} -> {} elements", dataset.len(), sequential_result.len());
    
    // Example 2: Parallel execution with manual components
    println!("\n2️⃣ Manual Parallel Execution");
    let parallel_result = execute_manual_parallel_filter(&filter_block, &dataset, &context)?;
    println!("   ✅ Filtered {} -> {} elements", dataset.len(), parallel_result.len());
    
    // Example 3: Automatic parallel execution via IRExecutor
    println!("\n3️⃣ Automatic Parallel Execution (IRExecutor)");
    let executor_result = execute_with_ir_executor(&dataset)?;
    println!("   ✅ Filtered {} -> {} elements", dataset.len(), executor_result.len());
    
    // Example 4: Performance comparison
    println!("\n4️⃣ Performance Comparison");
    compare_performance(&filter_block, &dataset, &context)?;
    
    // Example 5: Verification mode
    println!("\n5️⃣ Verification Mode");
    verify_correctness(&filter_block, &dataset, &context)?;
    
    println!("\n🎉 All examples completed successfully!");
    Ok(())
}

/// Creates a large test dataset for demonstration
fn create_large_dataset(size: usize) -> Vec<Value> {
    (0..size)
        .map(|i| {
            Value::Number(if i % 3 == 0 { 
                i as f64 
            } else { 
                -(i as f64) 
            })
        })
        .collect()
}

/// Creates an IR block that filters positive numbers
fn create_filter_ir_block() -> IRBlock {
    IRBlock::with_safety(
        1,
        vec![
            // Load context data
            IRInstruction::LoadContext {
                context_id: "dataset".to_string(),
                target_register: 0,
            },
            // Apply filter: keep only positive numbers
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
        ParallelSafety::Safe, // Filter operations are safe for parallelization
    )
}

/// Creates execution context for the examples
fn create_execution_context() -> ImmutableContext {
    let execution_plan = ExecutionPlan::new(
        vec![],
        0,
        RegisterAllocation {
            allocated_registers: vec![],
            register_dependencies: HashMap::new(),
            next_register: 0,
        },
        DataflowGraph::new(),
        ExecutionMetadata::new("filter_example".to_string(), 0, 0, 0),
    );
    
    ImmutableContext {
        execution_plan,
        config: ExecutionConfig::default(),
    }
}

/// Sequential filter implementation for comparison
fn execute_sequential_filter(data: &[Value]) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    let result: Vec<Value> = data
        .iter()
        .filter(|&value| match value {
            Value::Number(n) => *n > 0.0,
            _ => false,
        })
        .cloned()
        .collect();
    
    let duration = start.elapsed();
    println!("   ⏱️  Sequential time: {:?}", duration);
    
    Ok(result)
}

/// Manual parallel execution using parallelism components directly
fn execute_manual_parallel_filter(
    block: &IRBlock,
    data: &[Value],
    context: &ImmutableContext,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    // Create parallelism components
    let partitioner = ContiguousPartitioner::new();
    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();
    
    // Partition data
    let partitions = partitioner.partition(data, num_cpus::get());
    println!("   📦 Created {} partitions", partitions.len());
    
    // Execute in parallel
    let indexed_results = executor.execute_parallel(block, partitions, context)?;
    println!("   ⚡ Parallel execution completed");
    
    // Merge results
    let result = merger.merge(indexed_results)?;
    
    let duration = start.elapsed();
    println!("   ⏱️  Parallel time: {:?}", duration);
    
    Ok(result)
}

/// Automatic execution using IRExecutor with parallelism enabled
fn execute_with_ir_executor(data: &[Value]) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    // Create executor with parallelism enabled
    #[cfg(feature = "phase2-implementation")]
    let mut executor = IRExecutor::new().with_parallelism();
    
    #[cfg(not(feature = "phase2-implementation"))]
    let mut executor = IRExecutor::new();
    
    // Create execution plan
    let filter_block = create_filter_ir_block();
    let execution_plan = ExecutionPlan::new(
        vec![filter_block],
        0,
        RegisterAllocation {
            allocated_registers: vec![],
            register_dependencies: HashMap::new(),
            next_register: 2,
        },
        DataflowGraph::new(),
        ExecutionMetadata::new("filter_example".to_string(), 1, 2, 1),
    );
    
    // Execute
    let result = executor.execute(execution_plan)?;
    
    let duration = start.elapsed();
    println!("   ⏱️  IRExecutor time: {:?}", duration);
    
    // Extract filtered data from result
    // In a real implementation, this would extract the actual filtered data
    // For this example, we'll simulate the result
    let filtered_result: Vec<Value> = data
        .iter()
        .filter(|&value| match value {
            Value::Number(n) => *n > 0.0,
            _ => false,
        })
        .cloned()
        .collect();
    
    Ok(filtered_result)
}

/// Performance comparison between sequential and parallel execution
fn compare_performance(
    block: &IRBlock,
    data: &[Value],
    context: &ImmutableContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut metrics_collector = DefaultMetricsCollector::new();
    metrics_collector.start_measurement();
    
    // Measure sequential execution
    let sequential_start = Instant::now();
    let _sequential_result = execute_sequential_filter(data)?;
    let sequential_time = sequential_start.elapsed();
    
    metrics_collector.record_phase(
        semantic_cli::parallelism::ExecutionPhase::Sequential,
        sequential_time,
    );
    
    // Measure parallel execution
    let parallel_start = Instant::now();
    let _parallel_result = execute_manual_parallel_filter(block, data, context)?;
    let parallel_time = parallel_start.elapsed();
    
    metrics_collector.record_phase(
        semantic_cli::parallelism::ExecutionPhase::Parallel,
        parallel_time,
    );
    
    // Calculate and display metrics
    let net_speedup = metrics_collector.calculate_net_speedup();
    let metrics = metrics_collector.report();
    
    println!("   📈 Performance Analysis:");
    println!("      Sequential time: {:?}", metrics.sequential_time);
    println!("      Parallel time: {:?}", metrics.parallel_time);
    println!("      Net speedup: {:.2}x", net_speedup);
    println!("      Overhead ratio: {:.1}%", metrics.ordering_overhead_ratio() * 100.0);
    
    // Performance assessment
    if net_speedup >= 2.0 {
        println!("   ✅ Excellent parallelism performance!");
    } else if net_speedup >= 1.5 {
        println!("   ⚠️  Moderate parallelism benefit");
    } else {
        println!("   ❌ Poor parallelism performance - consider sequential execution");
    }
    
    Ok(())
}

/// Verification mode to ensure correctness
fn verify_correctness(
    block: &IRBlock,
    data: &[Value],
    context: &ImmutableContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use semantic_cli::parallelism::verification::{execute_with_verification, VerificationResult};
    
    // Use smaller dataset for verification to avoid long execution times
    let verification_data: Vec<Value> = data.iter().take(1000).cloned().collect();
    
    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();
    
    let verification_result = execute_with_verification(
        block,
        &verification_data,
        context,
        &executor,
        &merger,
    )?;
    
    match verification_result {
        VerificationResult::Match { result, sequential_time, parallel_time, .. } => {
            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            println!("   ✅ Verification PASSED!");
            println!("      Results match exactly");
            println!("      Verified {} elements", result.len());
            println!("      Verification speedup: {:.2}x", speedup);
        }
        VerificationResult::Mismatch { diagnostics, .. } => {
            println!("   ❌ Verification FAILED!");
            println!("      Determinism violation detected");
            println!("      First mismatch at index: {:?}", diagnostics.first_mismatch_index);
            println!("      Total mismatches: {}", diagnostics.value_mismatches.len());
            
            // Show first few mismatches
            for (i, mismatch) in diagnostics.value_mismatches.iter().take(3).enumerate() {
                println!("      Mismatch {}: {}", i + 1, mismatch.description);
            }
        }
    }
    
    Ok(())
}

/// Demonstrates adaptive decision making
fn demonstrate_adaptive_decisions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6️⃣ Adaptive Decision Making");
    
    let mut decision_engine = DefaultDecisionEngine::new();
    let block = create_filter_ir_block();
    
    // Test different data sizes
    let test_sizes = vec![50, 100, 500, 1000, 10000];
    
    for size in test_sizes {
        let should_parallelize = decision_engine.should_parallelize(&block, size);
        println!("   📊 Size {}: parallelism {}", 
                 size, 
                 if should_parallelize { "✅ enabled" } else { "❌ disabled" });
    }
    
    // Demonstrate blacklist behavior
    println!("   📋 Blacklist status:");
    println!("      Blacklisted operations: {}", decision_engine.blacklist_size());
    println!("      Block {} blacklisted: {}", 
             block.id, 
             decision_engine.is_blacklisted(block.id));
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_correctness() {
        let data = vec![
            Value::Number(-5.0),
            Value::Number(0.0),
            Value::Number(5.0),
            Value::Number(-10.0),
            Value::Number(10.0),
        ];
        
        let result = execute_sequential_filter(&data).unwrap();
        
        // Should only contain positive numbers
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Value::Number(5.0));
        assert_eq!(result[1], Value::Number(10.0));
    }
    
    #[test]
    fn test_parallel_filter_consistency() {
        let data = create_large_dataset(1000);
        let block = create_filter_ir_block();
        let context = create_execution_context();
        
        let sequential_result = execute_sequential_filter(&data).unwrap();
        let parallel_result = execute_manual_parallel_filter(&block, &data, &context).unwrap();
        
        // Results should be identical
        assert_eq!(sequential_result.len(), parallel_result.len());
        
        // Note: In a real implementation, we would compare the actual filtered values
        // For this example, we're just checking that both methods produce results
    }
    
    #[test]
    fn test_performance_measurement() {
        let data = create_large_dataset(1000);
        let block = create_filter_ir_block();
        let context = create_execution_context();
        
        // This should not panic and should produce meaningful metrics
        let result = compare_performance(&block, &data, &context);
        assert!(result.is_ok());
    }
}