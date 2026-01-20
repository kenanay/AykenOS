# Parallelism API Reference

**D2 Parallelism Architecture - API Documentation**  
**Created By:** Kenan AY  
**Date:** 17 Ocak 2026  
**Version:** v1.0

## Module Overview

The parallelism module provides comprehensive data-parallel execution capabilities while maintaining strict determinism guarantees.

```rust
use semantic_cli::parallelism::*;
```

## Core Traits

### `ParallelExecutor`

Executes IR blocks in parallel across data partitions.

```rust
pub trait ParallelExecutor {
    fn execute_parallel(
        &self,
        block: &IRBlock,
        partitions: Vec<DataPartition>,
        context: &ImmutableContext,
    ) -> Result<Vec<(usize, Value)>, ParallelismError>;
}
```

**Implementations:**
- `RayonParallelExecutor` - Uses Rayon thread pool for parallel execution

### `DataPartitioner`

Partitions input data for parallel processing.

```rust
pub trait DataPartitioner {
    fn partition(&self, data: &[Value], num_workers: usize) -> Vec<DataPartition>;
    fn calculate_partition_size(&self, data_size: usize, num_workers: usize) -> usize;
}
```

**Implementations:**
- `ContiguousPartitioner` - Divides data into contiguous chunks

### `DeterministicMerger`

Merges parallel results while preserving order.

```rust
pub trait DeterministicMerger {
    fn merge(&self, results: Vec<(usize, Value)>) -> Result<Vec<Value>, ParallelismError>;
    fn verify_completeness(&self, results: &[(usize, Value)], expected_size: usize) -> bool;
}
```

**Implementations:**
- `StableIndexMerger` - Uses stable index mapping for deterministic merging

### `AdaptiveDecisionEngine`

Makes intelligent decisions about when to use parallelism.

```rust
pub trait AdaptiveDecisionEngine {
    fn should_parallelize(&self, block: &IRBlock, data_size: usize) -> bool;
    fn record_execution(&mut self, block_id: BlockId, metrics: ExecutionMetrics);
    fn is_blacklisted(&self, block_id: BlockId) -> bool;
    fn update_blacklist(&mut self, block_id: BlockId, speedup: f64);
}
```

**Implementations:**
- `DefaultDecisionEngine` - Conservative decision making with soft blacklisting

### `MetricsCollector`

Collects and analyzes performance metrics.

```rust
pub trait MetricsCollector {
    fn start_measurement(&mut self);
    fn record_phase(&mut self, phase: ExecutionPhase, duration: Duration);
    fn calculate_net_speedup(&self) -> f64;
    fn report(&self) -> ExecutionMetrics;
}
```

**Implementations:**
- `DefaultMetricsCollector` - Comprehensive metrics collection with percentile analysis

### `ReductionHandler`

Handles commutative and non-commutative reduction operations.

```rust
pub trait ReductionHandler {
    fn classify_reduction(&self, operation: &IRInstruction) -> ReductionType;
    fn reduce_commutative<F>(&self, values: Vec<Value>, identity: Value, combine: F) -> ParallelismResult<Value>
    where F: Fn(Value, Value) -> Value + Sync + Send;
    fn reduce_non_commutative<F>(&self, indexed_values: Vec<(usize, Value)>, identity: Value, combine: F) -> ParallelismResult<Value>
    where F: Fn(Value, Value) -> Value;
}
```

**Implementations:**
- `DefaultReductionHandler` - Conservative classification with optimized reductions

### `VerificationExecutor`

Executes verification mode for correctness testing.

```rust
pub trait VerificationExecutor {
    fn execute_with_verification<P, M>(
        &self,
        block: &IRBlock,
        data: &[Value],
        context: &ImmutableContext,
        parallel_executor: &P,
        merger: &M,
    ) -> ParallelismResult<VerificationResult>
    where P: ParallelExecutor, M: DeterministicMerger;
}
```

**Implementations:**
- `DefaultVerificationExecutor` - Comprehensive verification with detailed diagnostics

## Data Types

### `DataPartition<'a>`

Represents a partition of input data for parallel processing.

```rust
pub struct DataPartition<'a> {
    pub data: &'a [Value],
    pub start_index: usize,
    pub end_index: usize,
}

impl<'a> DataPartition<'a> {
    pub fn size(&self) -> usize;
    pub fn is_valid(&self) -> bool;
    pub fn is_empty(&self) -> bool;
}
```

### `ExecutionMetrics`

Performance metrics for parallel execution.

```rust
pub struct ExecutionMetrics {
    pub sequential_time: Duration,
    pub parallel_time: Duration,
    pub ordering_overhead: Duration,
    pub sync_cost: Duration,
    pub merge_cost: Duration,
}

impl ExecutionMetrics {
    pub fn net_speedup(&self) -> f64;
    pub fn total_parallel_time(&self) -> Duration;
    pub fn ordering_overhead_ratio(&self) -> f64;
}
```

### `ImmutableContext`

Immutable execution context for parallel operations.

```rust
pub struct ImmutableContext {
    pub execution_plan: ExecutionPlan,
    pub config: ExecutionConfig,
}
```

### `ExecutionConfig`

Configuration for parallel execution.

```rust
pub struct ExecutionConfig {
    pub thread_pool_size: Option<usize>,
    pub enable_verification: bool,
    pub metrics_collection: bool,
    pub adaptive_decisions: bool,
}

impl Default for ExecutionConfig;
```

## Enums

### `ExecutionPhase`

Phases of parallel execution for metrics tracking.

```rust
pub enum ExecutionPhase {
    Sequential,
    Parallel,
    Ordering,
    Synchronization,
    Merge,
}
```

### `ReductionType`

Classification of reduction operations.

```rust
pub enum ReductionType {
    Commutative,    // Order-independent (sum, max, min)
    NonCommutative, // Order-dependent (concat, fold)
}
```

### `VerificationResult`

Result of verification mode execution.

```rust
pub enum VerificationResult {
    Match {
        result: Vec<Value>,
        sequential_time: Duration,
        parallel_time: Duration,
        verification_overhead: Duration,
    },
    Mismatch {
        parallel_result: Vec<Value>,
        sequential_result: Vec<Value>,
        diagnostics: VerificationDiagnostics,
    },
}
```

### `ParallelismError`

Errors that can occur during parallel execution.

```rust
pub enum ParallelismError {
    SafetyViolation { reason: String },
    ExecutionError { source: Box<dyn std::error::Error + Send + Sync> },
    DeterminismViolation { reason: String },
    PerformanceDegradation { speedup: f64 },
}
```

## Constants

### Performance Thresholds

```rust
/// Minimum net speedup required to enable parallelism
pub const MIN_NET_SPEEDUP: f64 = 2.0;

/// Maximum ordering overhead ratio before disabling parallelism
pub const MAX_OVERHEAD_RATIO: f64 = 0.5;

/// Re-evaluation window for blacklisted operations
pub const REEVALUATION_WINDOW: usize = 50;

/// Minimum dataset size for parallelism consideration
pub const MIN_PARALLEL_SIZE: usize = 100;
```

## Implementations

### `RayonParallelExecutor`

High-performance parallel executor using Rayon.

```rust
impl RayonParallelExecutor {
    pub fn new() -> Self;
    pub fn with_threads(num_threads: usize) -> Self;
    pub fn thread_count(&self) -> usize;
}

impl ParallelExecutor for RayonParallelExecutor {
    // Implementation uses rayon::par_iter() for optimal performance
}
```

### `ContiguousPartitioner`

Divides data into contiguous, balanced partitions.

```rust
impl ContiguousPartitioner {
    pub fn new() -> Self;
}

impl DataPartitioner for ContiguousPartitioner {
    // Ensures partitions are within 1 element of each other in size
    // Assigns deterministic start/end indices
}
```

### `StableIndexMerger`

Merges results using stable index mapping.

```rust
impl StableIndexMerger {
    pub fn new() -> Self;
}

impl DeterministicMerger for StableIndexMerger {
    // Uses chunk-local buffer strategy (ADR-3)
    // Single-threaded merge to avoid false sharing
    // Verifies all indices present before returning
}
```

### `DefaultDecisionEngine`

Conservative decision engine with adaptive blacklisting.

```rust
impl DefaultDecisionEngine {
    pub fn new() -> Self;
    pub fn set_replay_mode(&mut self, replay_mode: bool);
    pub fn is_replay_mode(&self) -> bool;
    pub fn blacklist_size(&self) -> usize;
}

impl AdaptiveDecisionEngine for DefaultDecisionEngine {
    // Decision algorithm:
    // 1. Check replay mode → sequential
    // 2. Check ParallelSafety::Unsafe → sequential  
    // 3. Check blacklist status
    // 4. Check data size threshold
    // 5. Estimate net speedup
}
```

### `DefaultMetricsCollector`

Comprehensive metrics collection with statistical analysis.

```rust
impl DefaultMetricsCollector {
    pub fn new() -> Self;
    pub fn measurement_count(&self) -> usize;
    pub fn p50_net_speedup(&self) -> Option<f64>;
    pub fn p75_net_speedup(&self) -> Option<f64>;
}

impl MetricsCollector for DefaultMetricsCollector {
    // Tracks all execution phases
    // Calculates net speedup with overhead costs
    // Maintains historical data for percentile analysis
}
```

### `DefaultReductionHandler`

Conservative reduction handler with optimized implementations.

```rust
impl DefaultReductionHandler {
    pub fn new() -> Self;
}

impl ReductionHandler for DefaultReductionHandler {
    // Classification strategy:
    // - Comparison operations → Commutative
    // - Logical operations → Commutative  
    // - Unknown operations → NonCommutative (safe default)
}
```

### `DefaultVerificationExecutor`

Comprehensive verification with detailed diagnostics.

```rust
impl DefaultVerificationExecutor {
    pub fn new() -> Self;
}

impl VerificationExecutor for DefaultVerificationExecutor {
    // Executes both parallel and sequential paths
    // Performs bitwise comparison of results
    // Generates detailed diagnostic information
}
```

## Utility Functions

### Convenience Functions

```rust
/// Convenience function for verification mode execution
pub fn execute_with_verification<P, M>(
    block: &IRBlock,
    data: &[Value],
    context: &ImmutableContext,
    parallel_executor: &P,
    merger: &M,
) -> ParallelismResult<VerificationResult>
where P: ParallelExecutor, M: DeterministicMerger;

/// Measures execution time of a closure
pub fn measure_execution<F, T>(f: F) -> (T, Duration)
where F: FnOnce() -> T;
```

### Common Reduction Operations

```rust
pub mod operations {
    /// Sum reduction (commutative)
    pub fn sum(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value>;
    
    /// Product reduction (commutative)
    pub fn product(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value>;
    
    /// Maximum reduction (commutative)
    pub fn max(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value>;
    
    /// Minimum reduction (commutative)
    pub fn min(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value>;
    
    /// String concatenation (non-commutative)
    pub fn concat(handler: &DefaultReductionHandler, indexed_values: Vec<(usize, Value)>) -> ParallelismResult<Value>;
}
```

## Feature Flags

### Phase Enforcement

```rust
#[cfg(feature = "phase2-implementation")]
// Parallelism features are only available with this feature flag
```

### Constitutional Compliance

```rust
#[cfg(feature = "constitutional-compliance")]
// Additional constitutional enforcement checks
```

## Integration with IRExecutor

### Basic Integration

```rust
use semantic_cli::ir_executor::IRExecutor;

// Enable parallelism with default components
let executor = IRExecutor::new().with_parallelism();

// Check if parallelism is enabled
if executor.is_parallelism_enabled() {
    println!("Parallelism is enabled");
}
```

### Custom Integration

```rust
// Custom parallelism components
let parallel_executor = Box::new(RayonParallelExecutor::new());
let decision_engine = Box::new(DefaultDecisionEngine::new());

let executor = IRExecutor::new()
    .with_custom_parallelism(parallel_executor, decision_engine);
```

## Testing Support

### Property Test Generators

```rust
// Available in test builds
#[cfg(test)]
pub mod test_utils {
    pub fn arbitrary_safe_ir_block() -> impl Strategy<Value = IRBlock>;
    pub fn arbitrary_dataset(size_range: Range<usize>) -> impl Strategy<Value = Vec<Value>>;
    pub fn create_test_context() -> ImmutableContext;
}
```

### Mock Implementations

```rust
#[cfg(test)]
pub struct MockParallelExecutor;

#[cfg(test)]
impl ParallelExecutor for MockParallelExecutor {
    // Test implementation
}
```

## Performance Considerations

### Thread Pool Sizing

- Default: Uses `num_cpus::get()` for optimal performance
- Recommendation: Don't exceed physical core count
- Consider NUMA topology for large systems

### Memory Usage

- Partitions hold references, not copies of data
- Minimal memory overhead per partition
- Consider cache locality when sizing partitions

### Overhead Costs

- Partitioning: O(1) time, O(partitions) space
- Merging: O(n log n) time for sorting by index
- Synchronization: Depends on thread pool implementation

## Error Handling Best Practices

### Error Propagation

```rust
// Errors are propagated from first failing worker
match parallel_result {
    Err(ParallelismError::ExecutionError { source }) => {
        // Handle execution error from worker thread
    }
    _ => {}
}
```

### Panic Safety

```rust
// Panics in worker threads are caught and converted to errors
// System remains stable even with worker panics
```

### Recovery Strategies

```rust
// Automatic fallback to sequential execution on parallel failure
if parallel_execution_fails {
    fallback_to_sequential();
}
```

## Debugging and Profiling

### Debug Logging

```rust
// Enable debug logging
env_logger::init();
log::debug!("Parallel execution started with {} partitions", partition_count);
```

### Performance Profiling

```rust
// Use criterion benchmarks for detailed performance analysis
cargo bench --bench parallelism_benchmarks
```

### Verification Debugging

```rust
// Use verification mode to debug determinism issues
let verification_result = execute_with_verification(...)?;
match verification_result {
    VerificationResult::Mismatch { diagnostics, .. } => {
        println!("Debug info: {}", diagnostics.context_info);
    }
    _ => {}
}
```

## Version Compatibility

- **Minimum Rust Version**: 1.70.0
- **Rayon Version**: 1.8+
- **Proptest Version**: 1.4+ (for testing)
- **Criterion Version**: 0.5+ (for benchmarking)

## See Also

- [Parallelism Usage Guide](parallelism_usage_guide.md)
- [D2 Parallelism Architecture Design Document](../../../docs/phase2/FAZ_2_ABDF_BCIB.md)
- [Property-Based Testing Guide](property_testing_guide.md)
- [Performance Tuning Guide](performance_tuning_guide.md)