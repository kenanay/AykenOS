# Performance Tuning Guide

**D2 Parallelism Architecture - Performance Optimization**  
**Created By:** Kenan AY  
**Date:** 17 Ocak 2026  
**Version:** v1.0

## Overview

This guide provides comprehensive guidance for optimizing the performance of the D2 Parallelism Architecture. It covers adaptive decision engine configuration, blacklist management, metrics interpretation, and system tuning.

## Performance Fundamentals

### Net Speedup Formula

The core performance metric is Net Speedup, calculated as:

```
net_speedup = sequential_time / (parallel_time + ordering_overhead + sync_cost + merge_cost)
```

**Target**: ≥2.0x for beneficial parallelism

### Key Performance Factors

1. **Data Size**: Larger datasets benefit more from parallelism
2. **Overhead Ratio**: Keep ordering overhead ≤50% of parallel time
3. **Thread Utilization**: Optimal thread count = physical CPU cores
4. **Memory Locality**: Contiguous partitions improve cache performance

## Adaptive Decision Engine Configuration

### Basic Configuration

```rust
use semantic_cli::parallelism::{DefaultDecisionEngine, MIN_NET_SPEEDUP, MAX_OVERHEAD_RATIO};

let mut decision_engine = DefaultDecisionEngine::new();

// Check current thresholds
println!("Minimum speedup threshold: {}x", MIN_NET_SPEEDUP);
println!("Maximum overhead threshold: {:.1}%", MAX_OVERHEAD_RATIO * 100.0);
```

### Replay Mode Management

```rust
// Disable parallelism for deterministic replay
decision_engine.set_replay_mode(true);

// Re-enable for normal execution
decision_engine.set_replay_mode(false);

// Check current mode
if decision_engine.is_replay_mode() {
    println!("Running in replay mode - parallelism disabled");
}
```

### Decision Monitoring

```rust
// Monitor decision outcomes
let should_parallel = decision_engine.should_parallelize(&block, data.len());
println!("Parallelism decision for {} elements: {}", data.len(), should_parallel);

// Track blacklist size
println!("Blacklisted operations: {}", decision_engine.blacklist_size());
```

## Blacklist Management

### Understanding the Blacklist

The adaptive blacklist temporarily disables parallelism for operations that show poor performance:

- **Trigger**: Net speedup <2.0x OR overhead ratio >50%
- **Duration**: 50 executions or version hash change
- **Recovery**: Automatic re-evaluation after trigger conditions

### Monitoring Blacklist Status

```rust
// Check if specific operation is blacklisted
if decision_engine.is_blacklisted(block_id) {
    println!("Block {} is blacklisted", block_id);
}

// Monitor blacklist growth
let initial_size = decision_engine.blacklist_size();
// ... execute operations ...
let final_size = decision_engine.blacklist_size();

if final_size > initial_size {
    println!("Warning: {} new operations blacklisted", final_size - initial_size);
}
```

### Blacklist Optimization Strategies

#### 1. Identify Root Causes

```rust
// Use verification mode to debug blacklisted operations
let verification_result = execute_with_verification(
    &blacklisted_block,
    &test_data,
    &context,
    &parallel_executor,
    &merger,
)?;

match verification_result {
    VerificationResult::Match { sequential_time, parallel_time, .. } => {
        let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
        println!("Actual speedup: {:.2}x", speedup);
    }
    VerificationResult::Mismatch { .. } => {
        println!("Determinism violation detected");
    }
}
```

#### 2. Data Size Analysis

```rust
// Test different data sizes to find optimal threshold
let test_sizes = vec![50, 100, 500, 1000, 5000, 10000];

for size in test_sizes {
    let test_data = create_test_dataset(size);
    let should_parallel = decision_engine.should_parallelize(&block, test_data.len());
    println!("Size {}: parallelism {}", size, if should_parallel { "enabled" } else { "disabled" });
}
```

#### 3. Force Re-evaluation

```rust
// Simulate 50 executions to trigger re-evaluation
for _ in 0..50 {
    let metrics = ExecutionMetrics {
        sequential_time: Duration::from_millis(1000),
        parallel_time: Duration::from_millis(300),  // Better performance
        ordering_overhead: Duration::from_millis(50),
        sync_cost: Duration::from_millis(20),
        merge_cost: Duration::from_millis(30),
    };
    
    decision_engine.record_execution(block_id, metrics);
}

// Check if re-evaluation occurred
if !decision_engine.is_blacklisted(block_id) {
    println!("Operation {} removed from blacklist", block_id);
}
```

## Metrics Interpretation

### Basic Metrics Analysis

```rust
use semantic_cli::parallelism::{DefaultMetricsCollector, MetricsCollector};

let mut collector = DefaultMetricsCollector::new();

// After execution
let metrics = collector.report();
let net_speedup = collector.calculate_net_speedup();

println!("=== Performance Analysis ===");
println!("Sequential time: {:?}", metrics.sequential_time);
println!("Parallel time: {:?}", metrics.parallel_time);
println!("Total parallel time: {:?}", metrics.total_parallel_time());
println!("Net speedup: {:.2}x", net_speedup);
println!("Overhead ratio: {:.1}%", metrics.ordering_overhead_ratio() * 100.0);
```

### Advanced Metrics Analysis

```rust
// Historical performance analysis
if collector.measurement_count() > 10 {
    if let Some(p50) = collector.p50_net_speedup() {
        println!("P50 speedup: {:.2}x", p50);
    }
    
    if let Some(p75) = collector.p75_net_speedup() {
        println!("P75 speedup: {:.2}x", p75);
    }
    
    // Performance stability check
    if let (Some(p50), Some(p75)) = (collector.p50_net_speedup(), collector.p75_net_speedup()) {
        let stability = p50 / p75;
        if stability < 0.8 {
            println!("Warning: Performance is unstable (P50/P75 = {:.2})", stability);
        }
    }
}
```

### Performance Regression Detection

```rust
// Store baseline metrics
struct PerformanceBaseline {
    baseline_speedup: f64,
    baseline_overhead: f64,
}

impl PerformanceBaseline {
    fn check_regression(&self, current_metrics: &ExecutionMetrics) -> bool {
        let current_speedup = current_metrics.net_speedup();
        let current_overhead = current_metrics.ordering_overhead_ratio();
        
        // Check for >10% performance degradation
        let speedup_regression = current_speedup < self.baseline_speedup * 0.9;
        let overhead_increase = current_overhead > self.baseline_overhead * 1.1;
        
        speedup_regression || overhead_increase
    }
}
```

## Thread Pool Optimization

### Optimal Thread Count

```rust
use semantic_cli::parallelism::RayonParallelExecutor;

// Default: Use all available cores
let executor = RayonParallelExecutor::new();
println!("Using {} threads", executor.thread_count());

// Custom thread count for specific workloads
let cpu_cores = num_cpus::get();
let optimal_threads = match cpu_cores {
    1..=2 => cpu_cores,           // Use all cores for small systems
    3..=8 => cpu_cores - 1,       // Leave one core for system tasks
    _ => cpu_cores * 3 / 4,       // Use 75% of cores for large systems
};

let executor = RayonParallelExecutor::with_threads(optimal_threads);
```

### Thread Pool Sizing Guidelines

| System Type | Recommended Thread Count | Reasoning |
|-------------|-------------------------|-----------|
| Single-core | 1 (sequential only) | No parallelism benefit |
| Dual-core | 2 | Use all available cores |
| Quad-core | 3-4 | Leave 1 core for OS tasks |
| 8+ cores | 75% of cores | Balance parallelism with system responsiveness |
| NUMA systems | Cores per NUMA node | Avoid cross-NUMA memory access |

### Dynamic Thread Adjustment

```rust
// Monitor system load and adjust thread count
fn adjust_thread_count_based_on_load() -> usize {
    let cpu_cores = num_cpus::get();
    let system_load = get_system_load(); // Hypothetical function
    
    match system_load {
        load if load < 0.5 => cpu_cores,           // Low load: use all cores
        load if load < 0.8 => cpu_cores * 3 / 4,   // Medium load: reduce threads
        _ => cpu_cores / 2,                        // High load: minimal threads
    }
}
```

## Data Partitioning Optimization

### Partition Size Analysis

```rust
use semantic_cli::parallelism::{ContiguousPartitioner, DataPartitioner};

let partitioner = ContiguousPartitioner::new();

// Analyze partition distribution
fn analyze_partitions(data: &[Value], num_workers: usize) {
    let partitions = partitioner.partition(data, num_workers);
    
    println!("=== Partition Analysis ===");
    println!("Total elements: {}", data.len());
    println!("Number of partitions: {}", partitions.len());
    
    let sizes: Vec<usize> = partitions.iter().map(|p| p.size()).collect();
    let min_size = sizes.iter().min().unwrap_or(&0);
    let max_size = sizes.iter().max().unwrap_or(&0);
    let avg_size = sizes.iter().sum::<usize>() / sizes.len().max(1);
    
    println!("Partition sizes - Min: {}, Max: {}, Avg: {}", min_size, max_size, avg_size);
    
    // Check for load imbalance
    let imbalance = (*max_size as f64 - *min_size as f64) / avg_size as f64;
    if imbalance > 0.1 {
        println!("Warning: Load imbalance detected ({:.1}%)", imbalance * 100.0);
    }
}
```

### Optimal Partition Sizing

```rust
// Calculate optimal partition size based on data characteristics
fn calculate_optimal_partition_size(data_size: usize, num_workers: usize) -> usize {
    let base_size = (data_size + num_workers - 1) / num_workers;
    
    // Adjust based on data size
    match data_size {
        0..=1000 => base_size,                    // Small data: use calculated size
        1001..=100000 => base_size.max(100),      // Medium data: minimum 100 elements
        _ => base_size.max(1000),                 // Large data: minimum 1000 elements
    }
}
```

## Memory Optimization

### Memory Usage Monitoring

```rust
// Monitor memory usage during partitioning
fn monitor_partition_memory_usage(data: &[Value]) {
    let initial_memory = get_memory_usage(); // Hypothetical function
    
    let partitioner = ContiguousPartitioner::new();
    let partitions = partitioner.partition(data, num_cpus::get());
    
    let partition_memory = get_memory_usage() - initial_memory;
    
    println!("Partition memory overhead: {} bytes", partition_memory);
    println!("Memory per partition: {} bytes", partition_memory / partitions.len().max(1));
    
    // Check for excessive memory usage
    let data_size_bytes = data.len() * std::mem::size_of::<Value>();
    let overhead_ratio = partition_memory as f64 / data_size_bytes as f64;
    
    if overhead_ratio > 0.1 {
        println!("Warning: High memory overhead ({:.1}%)", overhead_ratio * 100.0);
    }
}
```

### Memory-Efficient Strategies

```rust
// Use streaming processing for large datasets
fn process_large_dataset_streaming(data: &[Value], chunk_size: usize) {
    for chunk in data.chunks(chunk_size) {
        // Process each chunk independently
        let partitions = partitioner.partition(chunk, num_cpus::get());
        let results = parallel_executor.execute_parallel(&block, partitions, &context)?;
        let merged = merger.merge(results)?;
        
        // Process results immediately to free memory
        process_chunk_results(merged);
    }
}
```

## Overhead Reduction Strategies

### Minimize Ordering Overhead

```rust
// Use commutative operations when possible
use semantic_cli::parallelism::{DefaultReductionHandler, ReductionType, operations};

let handler = DefaultReductionHandler::new();

// Check if operation can be optimized
let reduction_type = handler.classify_reduction(&instruction);
match reduction_type {
    ReductionType::Commutative => {
        println!("Operation can skip ordering overhead");
        // Use parallel reduction without ordering
        let result = operations::sum(&handler, values)?;
    }
    ReductionType::NonCommutative => {
        println!("Operation requires ordering preservation");
        // Use ordered merge with stable indices
    }
}
```

### Reduce Synchronization Costs

```rust
// Minimize thread synchronization points
fn optimize_synchronization() {
    // 1. Use thread-local storage for intermediate results
    // 2. Batch operations to reduce sync frequency
    // 3. Use lock-free data structures when possible
    
    // Example: Batch multiple operations
    let batch_size = 100;
    for batch in operations.chunks(batch_size) {
        // Process batch with single synchronization point
        process_batch_parallel(batch);
    }
}
```

### Optimize Merge Operations

```rust
// Use efficient merging strategies
use semantic_cli::parallelism::{StableIndexMerger, DeterministicMerger};

let merger = StableIndexMerger::new();

// Pre-sort results if possible to speed up merging
fn optimize_merge(mut indexed_results: Vec<(usize, Value)>) -> Vec<Value> {
    // Sort by index for faster merging
    indexed_results.sort_by_key(|(idx, _)| *idx);
    
    // Verify completeness before merging
    if !merger.verify_completeness(&indexed_results, expected_size) {
        panic!("Incomplete results detected");
    }
    
    merger.merge(indexed_results).unwrap()
}
```

## System-Level Optimization

### CPU Affinity

```rust
// Set CPU affinity for optimal performance (platform-specific)
#[cfg(target_os = "linux")]
fn set_cpu_affinity() {
    // Bind threads to specific CPU cores to improve cache locality
    // Implementation would use platform-specific APIs
}
```

### NUMA Awareness

```rust
// NUMA-aware partitioning for large systems
fn numa_aware_partitioning(data: &[Value]) -> Vec<DataPartition> {
    let numa_nodes = get_numa_node_count(); // Hypothetical function
    let cores_per_node = num_cpus::get() / numa_nodes;
    
    // Create partitions aligned with NUMA topology
    let partition_size = data.len() / numa_nodes;
    
    // Implementation would consider NUMA topology
    todo!("NUMA-aware partitioning implementation")
}
```

### I/O Optimization

```rust
// Optimize I/O operations in parallel context
fn optimize_io_operations() {
    // 1. Use async I/O for non-blocking operations
    // 2. Batch I/O operations to reduce syscall overhead
    // 3. Use memory-mapped files for large datasets
    // 4. Implement read-ahead strategies
}
```

## Performance Testing and Validation

### Automated Performance Testing

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[test]
    fn test_performance_regression() {
        let baseline_speedup = 2.5; // Stored baseline
        
        let mut collector = DefaultMetricsCollector::new();
        // ... execute test workload ...
        
        let current_speedup = collector.calculate_net_speedup();
        
        // Allow 10% performance degradation
        assert!(
            current_speedup >= baseline_speedup * 0.9,
            "Performance regression detected: {:.2}x < {:.2}x",
            current_speedup,
            baseline_speedup * 0.9
        );
    }
    
    #[test]
    fn test_overhead_threshold() {
        let mut collector = DefaultMetricsCollector::new();
        // ... execute test workload ...
        
        let metrics = collector.report();
        let overhead_ratio = metrics.ordering_overhead_ratio();
        
        assert!(
            overhead_ratio <= MAX_OVERHEAD_RATIO,
            "Overhead threshold exceeded: {:.1}% > {:.1}%",
            overhead_ratio * 100.0,
            MAX_OVERHEAD_RATIO * 100.0
        );
    }
}
```

### Continuous Performance Monitoring

```rust
// Implement performance monitoring in production
struct PerformanceMonitor {
    baseline_metrics: HashMap<String, f64>,
    alert_threshold: f64,
}

impl PerformanceMonitor {
    fn check_performance(&self, operation: &str, current_speedup: f64) {
        if let Some(&baseline) = self.baseline_metrics.get(operation) {
            let degradation = (baseline - current_speedup) / baseline;
            
            if degradation > self.alert_threshold {
                log::warn!(
                    "Performance degradation detected for {}: {:.2}x -> {:.2}x ({:.1}%)",
                    operation, baseline, current_speedup, degradation * 100.0
                );
            }
        }
    }
}
```

## Troubleshooting Performance Issues

### Common Performance Problems

#### 1. Low Speedup

**Symptoms**: Net speedup <2.0x
**Causes**: 
- Small dataset size
- High overhead costs
- Poor thread utilization

**Solutions**:
```rust
// Increase minimum data size threshold
const CUSTOM_MIN_PARALLEL_SIZE: usize = 500; // Instead of 100

// Reduce overhead by optimizing operations
// Use commutative reductions when possible
// Minimize synchronization points
```

#### 2. High Overhead

**Symptoms**: Ordering overhead >50%
**Causes**:
- Excessive partitioning
- Complex merge operations
- Thread contention

**Solutions**:
```rust
// Reduce partition count
let optimal_partitions = (data.len() / 1000).max(1).min(num_cpus::get());

// Use simpler merge strategies
// Optimize data structures for cache locality
```

#### 3. Memory Issues

**Symptoms**: High memory usage, GC pressure
**Causes**:
- Large partition overhead
- Memory fragmentation
- Inefficient data structures

**Solutions**:
```rust
// Use streaming processing
// Implement memory pooling
// Optimize data layout for cache efficiency
```

### Performance Debugging Tools

```rust
// Enable detailed performance logging
fn enable_performance_debugging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    log::debug!("Performance debugging enabled");
}

// Use profiling tools
fn profile_parallel_execution() {
    // Use tools like:
    // - perf (Linux)
    // - Instruments (macOS)  
    // - VTune (Intel)
    // - cargo flamegraph
}
```

## Best Practices Summary

### Configuration Best Practices

1. **Thread Count**: Use 75% of CPU cores for systems with >4 cores
2. **Data Size**: Only parallelize datasets >100 elements
3. **Overhead Monitoring**: Keep ordering overhead ≤50%
4. **Blacklist Management**: Monitor and address blacklisted operations

### Performance Best Practices

1. **Measurement**: Always collect comprehensive metrics
2. **Baselines**: Establish and monitor performance baselines
3. **Testing**: Include performance tests in CI/CD pipeline
4. **Profiling**: Regular profiling to identify bottlenecks

### Optimization Best Practices

1. **Commutative Operations**: Use when possible to skip ordering
2. **Memory Efficiency**: Minimize partition overhead
3. **Cache Locality**: Use contiguous partitions
4. **Load Balancing**: Ensure even partition sizes

### Monitoring Best Practices

1. **Continuous Monitoring**: Track performance in production
2. **Alerting**: Set up alerts for performance degradation
3. **Historical Analysis**: Use percentile metrics for stability
4. **Regression Detection**: Automated performance regression tests

## See Also

- [Parallelism Usage Guide](parallelism_usage_guide.md)
- [Parallelism API Reference](parallelism_api_reference.md)
- [Benchmarking Guide](benchmarking_guide.md)
- [Property Testing Guide](property_testing_guide.md)