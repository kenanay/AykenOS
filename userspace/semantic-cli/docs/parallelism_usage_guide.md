# Parallelism Usage Guide

**D2 Parallelism Architecture - User Guide**  
**Created By:** Kenan AY  
**Date:** 17 Ocak 2026  
**Version:** v1.0

## Overview

The D2 Parallelism Architecture enables data-parallel execution of IR operations while maintaining strict determinism guarantees. This guide explains how to enable, configure, and use parallelism in your applications.

## Quick Start

### Enabling Parallelism

```rust
use semantic_cli::ir_executor::IRExecutor;

// Create executor with parallelism enabled
let executor = IRExecutor::new().with_parallelism();

// Execute with automatic parallel/sequential decision making
let result = executor.execute(execution_plan)?;
```

### Custom Configuration

```rust
use semantic_cli::parallelism::{RayonParallelExecutor, DefaultDecisionEngine};

// Create executor with custom parallelism components
let parallel_executor = Box::new(RayonParallelExecutor::new());
let decision_engine = Box::new(DefaultDecisionEngine::new());

let executor = IRExecutor::new()
    .with_custom_parallelism(parallel_executor, decision_engine);
```

## Core Concepts

### Parallel Safety

Operations are classified into three safety levels:

- **Safe**: Pure operations that can be parallelized safely
- **Unsafe**: Operations with side effects that must run sequentially
- **ReductionOnly**: Operations that can only be parallelized for reductions

```rust
use semantic_cli::execution_plan::ParallelSafety;

// IR blocks are automatically annotated with safety levels
let block = IRBlock::with_safety(
    id,
    instructions,
    terminator,
    ParallelSafety::Safe,  // This block can be parallelized
);
```

### Adaptive Decision Making

The system automatically decides when to use parallelism based on:

- **Data Size**: Minimum 100 elements required
- **Net Speedup**: Must achieve at least 2.0x speedup
- **Overhead Ratio**: Ordering overhead must be ≤50%
- **Blacklist Status**: Poor performers are temporarily blacklisted

## Configuration

### Thread Pool Configuration

```rust
use semantic_cli::parallelism::RayonParallelExecutor;

// Default: Uses all available CPU cores
let executor = RayonParallelExecutor::new();

// Custom thread count
let executor = RayonParallelExecutor::with_threads(4);
```

### Decision Engine Tuning

```rust
use semantic_cli::parallelism::{DefaultDecisionEngine, MIN_NET_SPEEDUP};

let mut decision_engine = DefaultDecisionEngine::new();

// Enable replay mode (forces sequential execution)
decision_engine.set_replay_mode(true);

// Check current thresholds
println!("Minimum speedup: {}x", MIN_NET_SPEEDUP);
```

### Blacklist Management

```rust
// Check if an operation is blacklisted
if decision_engine.is_blacklisted(block_id) {
    println!("Operation {} is blacklisted", block_id);
}

// Blacklist size
println!("Blacklisted operations: {}", decision_engine.blacklist_size());
```

## Verification Mode

Verification mode runs both parallel and sequential paths and compares results to detect determinism violations.

### Basic Verification

```rust
use semantic_cli::parallelism::verification::execute_with_verification;

let result = execute_with_verification(
    &block,
    &data,
    &context,
    &parallel_executor,
    &merger,
)?;

match result {
    VerificationResult::Match { result, sequential_time, parallel_time, .. } => {
        println!("✅ Results match! Speedup: {:.2}x", 
                 sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
    }
    VerificationResult::Mismatch { diagnostics, .. } => {
        println!("❌ Determinism violation detected!");
        println!("First mismatch at index: {:?}", diagnostics.first_mismatch_index);
        for mismatch in &diagnostics.value_mismatches {
            println!("  Index {}: parallel={:?}, sequential={:?}", 
                     mismatch.index, mismatch.parallel_value, mismatch.sequential_value);
        }
    }
}
```

### Advanced Verification

```rust
use semantic_cli::parallelism::verification::{DefaultVerificationExecutor, VerificationExecutor};

let verifier = DefaultVerificationExecutor::new();

let result = verifier.execute_with_verification(
    &block,
    &data,
    &context,
    &parallel_executor,
    &merger,
)?;
```

## Performance Monitoring

### Metrics Collection

```rust
use semantic_cli::parallelism::{DefaultMetricsCollector, MetricsCollector, ExecutionPhase};

let mut collector = DefaultMetricsCollector::new();

collector.start_measurement();

// Record execution phases
collector.record_phase(ExecutionPhase::Sequential, sequential_time);
collector.record_phase(ExecutionPhase::Parallel, parallel_time);
collector.record_phase(ExecutionPhase::Ordering, ordering_time);
collector.record_phase(ExecutionPhase::Synchronization, sync_time);
collector.record_phase(ExecutionPhase::Merge, merge_time);

// Calculate performance metrics
let net_speedup = collector.calculate_net_speedup();
let metrics = collector.report();

println!("Net speedup: {:.2}x", net_speedup);
println!("Sequential time: {:?}", metrics.sequential_time);
println!("Parallel time: {:?}", metrics.parallel_time);
println!("Overhead ratio: {:.1}%", metrics.ordering_overhead_ratio() * 100.0);
```

### Historical Analysis

```rust
// Get percentile metrics from historical data
if let Some(p50) = collector.p50_net_speedup() {
    println!("P50 speedup: {:.2}x", p50);
}

if let Some(p75) = collector.p75_net_speedup() {
    println!("P75 speedup: {:.2}x", p75);
}

println!("Total measurements: {}", collector.measurement_count());
```

## Data Partitioning

### Contiguous Partitioning

```rust
use semantic_cli::parallelism::{ContiguousPartitioner, DataPartitioner};

let partitioner = ContiguousPartitioner::new();
let partitions = partitioner.partition(&data, num_cpus::get());

for (i, partition) in partitions.iter().enumerate() {
    println!("Partition {}: {} elements (indices {}-{})", 
             i, partition.size(), partition.start_index, partition.end_index);
}
```

### Custom Partitioning

```rust
// Implement custom partitioning strategy
struct CustomPartitioner;

impl DataPartitioner for CustomPartitioner {
    fn partition(&self, data: &[Value], num_workers: usize) -> Vec<DataPartition> {
        // Custom partitioning logic
        todo!()
    }
    
    fn calculate_partition_size(&self, data_size: usize, num_workers: usize) -> usize {
        (data_size + num_workers - 1) / num_workers
    }
}
```

## Result Merging

### Deterministic Merging

```rust
use semantic_cli::parallelism::{StableIndexMerger, DeterministicMerger};

let merger = StableIndexMerger::new();

// Merge indexed results from parallel execution
let indexed_results = vec![
    (0, Value::Number(1.0)),
    (1, Value::Number(2.0)),
    (2, Value::Number(3.0)),
];

let merged_result = merger.merge(indexed_results)?;
```

### Completeness Verification

```rust
// Verify all indices are present
let is_complete = merger.verify_completeness(&indexed_results, expected_size);
if !is_complete {
    return Err(ParallelismError::DeterminismViolation { 
        reason: "Missing indices in parallel results".to_string() 
    });
}
```

## Reduction Operations

### Commutative Reductions

```rust
use semantic_cli::parallelism::{DefaultReductionHandler, operations};

let handler = DefaultReductionHandler::new();

// Sum reduction (order-independent)
let values = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
let sum = operations::sum(&handler, values)?;

// Max reduction (order-independent)
let values = vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0)];
let max = operations::max(&handler, values)?;
```

### Non-Commutative Reductions

```rust
// String concatenation (order-dependent)
let indexed_values = vec![
    (0, Value::String("Hello".to_string())),
    (1, Value::String(" ".to_string())),
    (2, Value::String("World".to_string())),
];

let concat = operations::concat(&handler, indexed_values)?;
// Result: "Hello World"
```

## Error Handling

### Common Errors

```rust
use semantic_cli::parallelism::ParallelismError;

match result {
    Err(ParallelismError::SafetyViolation { reason }) => {
        println!("Safety violation: {}", reason);
    }
    Err(ParallelismError::DeterminismViolation { reason }) => {
        println!("Determinism violation: {}", reason);
    }
    Err(ParallelismError::PerformanceDegradation { speedup }) => {
        println!("Performance degradation: {:.2}x speedup", speedup);
    }
    Err(ParallelismError::ExecutionError { source }) => {
        println!("Execution error: {}", source);
    }
    Ok(result) => {
        println!("Success: {:?}", result);
    }
}
```

## Best Practices

### 1. Constitutional Compliance

- **Determinism First**: Never sacrifice determinism for performance
- **Replay Support**: Always test with replay mode enabled
- **Safety Classification**: Properly classify IR blocks for parallel safety

### 2. Performance Optimization

- **Data Size**: Use parallelism only for datasets >100 elements
- **Overhead Monitoring**: Keep ordering overhead below 50%
- **Blacklist Awareness**: Monitor and address blacklisted operations

### 3. Testing and Verification

- **Property Tests**: Use property-based testing for correctness
- **Verification Mode**: Regularly run verification to catch regressions
- **Benchmarking**: Monitor performance with comprehensive benchmarks

### 4. Debugging

- **Metrics Collection**: Always collect performance metrics
- **Diagnostic Information**: Use verification mode for detailed diagnostics
- **Logging**: Enable debug logging for troubleshooting

## Troubleshooting

### Performance Issues

**Problem**: Low or negative speedup
```rust
// Check metrics
let metrics = collector.report();
if metrics.net_speedup() < 2.0 {
    println!("Speedup too low: {:.2}x", metrics.net_speedup());
    println!("Overhead ratio: {:.1}%", metrics.ordering_overhead_ratio() * 100.0);
}
```

**Solution**: 
- Increase data size
- Reduce overhead costs
- Check for blacklisted operations

### Determinism Violations

**Problem**: Verification mode detects mismatches
```rust
// Enable detailed diagnostics
match verification_result {
    VerificationResult::Mismatch { diagnostics, .. } => {
        println!("Context: {}", diagnostics.context_info);
        println!("Timestamp: {}", diagnostics.timestamp);
        for mismatch in &diagnostics.value_mismatches {
            println!("{}", mismatch.description);
        }
    }
    _ => {}
}
```

**Solution**:
- Check for race conditions
- Verify IR block safety classification
- Review parallel execution logic

### Memory Issues

**Problem**: High memory usage with large datasets
```rust
// Monitor partition sizes
let partitions = partitioner.partition(&data, num_workers);
for (i, partition) in partitions.iter().enumerate() {
    println!("Partition {} size: {} elements", i, partition.size());
}
```

**Solution**:
- Adjust partition sizes
- Use streaming processing
- Implement memory-efficient algorithms

## Advanced Topics

### Custom Parallel Executors

```rust
use semantic_cli::parallelism::ParallelExecutor;

struct CustomParallelExecutor;

impl ParallelExecutor for CustomParallelExecutor {
    fn execute_parallel(
        &self,
        block: &IRBlock,
        partitions: Vec<DataPartition>,
        context: &ImmutableContext,
    ) -> Result<Vec<(usize, Value)>, ParallelismError> {
        // Custom parallel execution logic
        todo!()
    }
}
```

### Custom Decision Engines

```rust
use semantic_cli::parallelism::AdaptiveDecisionEngine;

struct CustomDecisionEngine;

impl AdaptiveDecisionEngine for CustomDecisionEngine {
    fn should_parallelize(&self, block: &IRBlock, data_size: usize) -> bool {
        // Custom decision logic
        todo!()
    }
    
    fn record_execution(&mut self, block_id: BlockId, metrics: ExecutionMetrics) {
        // Custom learning logic
        todo!()
    }
    
    // ... other methods
}
```

## API Reference

For complete API documentation, run:
```bash
cargo doc --open --package semantic-cli
```

## Examples

See the `examples/` directory for complete working examples:
- `parallel_filter_operation.rs`
- `parallel_map_operation.rs`
- `parallel_aggregation.rs`
- `verification_mode_usage.rs`

## Support

For issues and questions:
- Check the troubleshooting section above
- Review the property tests for usage patterns
- Consult the design document for architectural details