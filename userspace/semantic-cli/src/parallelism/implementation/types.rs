//! Core data models for parallelism architecture
//!
//! This module defines the fundamental types used in the D2 Parallelism Architecture:
//! - DataPartition: Represents a contiguous slice of input data for parallel processing
//! - ExecutionMetrics: Performance metrics for adaptive decision making
//! - ImmutableContext: Shared read-only context for parallel workers
//! - LocalState: Thread-local mutable state for parallel workers
//!
//! **Design Reference:** D2 Parallelism Architecture - Data Models section
//! **Requirements:** 1.2, 1.4, 6.1, 6.2, 12.1-12.5

use crate::bcib::Value;
use crate::execution_plan::ExecutionPlan;
use std::time::Duration;

/// Represents a contiguous slice of input data that can be processed independently
/// by a parallel worker.
///
/// This struct is a core data model for the parallelism architecture, representing
/// the "Data" component of the Parallelism_Unit = (IR_Block, Data_Partition) model.
///
/// # Design Principles
///
/// - **Contiguous Partitions**: Data is divided into contiguous slices for cache efficiency
/// - **Deterministic Indexing**: Each partition has stable start/end indices for deterministic merging
/// - **Zero-Copy**: Uses borrowed slices to avoid data copying overhead
///
/// **Validates: Requirements 1.2, 1.4**
#[derive(Debug, Clone, Copy)]
pub struct DataPartition<'a> {
    /// Borrowed slice of input data to be processed by this partition
    pub data: &'a [Value],
    
    /// Starting index in the original dataset (inclusive)
    /// Used for stable index mapping during deterministic merge
    pub start_index: usize,
    
    /// Ending index in the original dataset (exclusive)
    /// Used for stable index mapping during deterministic merge
    pub end_index: usize,
}

impl<'a> DataPartition<'a> {
    /// Returns the number of elements in this partition
    #[inline]
    pub fn size(&self) -> usize {
        self.end_index - self.start_index
    }
    
    /// Converts a local index (within this partition) to a logical index
    /// (in the original dataset).
    ///
    /// This is critical for the Stable Index Map pattern, which ensures
    /// deterministic ordering of parallel results.
    ///
    /// **Validates: Requirement 2.2 (Stable Index Map)**
    #[inline]
    pub fn logical_index(&self, local_index: usize) -> usize {
        self.start_index + local_index
    }
    
    /// Validates that the partition is well-formed
    pub fn is_valid(&self) -> bool {
        self.start_index <= self.end_index && self.data.len() == self.size()
    }
}

/// Performance metrics for parallel execution, used by the adaptive decision engine
/// to determine whether parallelism provides sufficient benefit.
///
/// This struct tracks all components of execution time to calculate the **Net Speedup**,
/// which accounts for all overhead costs (ordering, synchronization, merging).
///
/// # Net Speedup Formula
///
/// ```text
/// net_speedup = sequential_time / (parallel_time + ordering_overhead + sync_cost + merge_cost)
/// ```
///
/// If `net_speedup < 2.0x`, the adaptive system disables parallelism for that operation.
///
/// **Validates: Requirements 12.1-12.5, 4.1, 4.7**
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExecutionMetrics {
    /// Time taken for sequential execution (baseline for comparison)
    pub sequential_time: Duration,
    
    /// Time taken for parallel computation (excluding overhead)
    pub parallel_time: Duration,
    
    /// Overhead from maintaining stable index mapping for deterministic ordering
    pub ordering_overhead: Duration,
    
    /// Cost of thread synchronization (barriers, locks, etc.)
    pub sync_cost: Duration,
    
    /// Cost of merging parallel results into final output
    pub merge_cost: Duration,
}

impl ExecutionMetrics {
    /// Calculates the net speedup, accounting for all overhead costs.
    ///
    /// **Validates: Requirements 12.5, 4.1 (Net Speedup Calculation)**
    pub fn net_speedup(&self) -> f64 {
        let total_parallel = self.parallel_time 
            + self.ordering_overhead 
            + self.sync_cost 
            + self.merge_cost;
        
        // Avoid division by zero
        if total_parallel.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.sequential_time.as_secs_f64() / total_parallel.as_secs_f64()
    }
    
    /// Calculates the ratio of ordering overhead to parallel execution time.
    ///
    /// **Validates: Requirement 4.7 (Ordering Overhead Protection)**
    pub fn ordering_overhead_ratio(&self) -> f64 {
        // Avoid division by zero
        if self.parallel_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.ordering_overhead.as_secs_f64() / self.parallel_time.as_secs_f64()
    }
    
    /// Returns the total parallel execution time including all overhead.
    pub fn total_parallel_time(&self) -> Duration {
        self.parallel_time + self.ordering_overhead + self.sync_cost + self.merge_cost
    }
}

/// Immutable context shared across all parallel workers.
///
/// This struct contains read-only data that all workers need access to during
/// parallel execution. By making this context immutable, we ensure thread safety
/// without synchronization overhead.
///
/// **Validates: Requirements 6.1, 6.2 (Shared State Model)**
#[derive(Debug, Clone)]
pub struct ImmutableContext {
    /// The execution plan containing IR blocks and metadata
    pub execution_plan: ExecutionPlan,
    
    /// Execution configuration (timeouts, limits, etc.)
    pub config: ExecutionConfig,
}

/// Execution configuration parameters.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum execution time before timeout
    pub max_execution_time: Option<Duration>,
    
    /// Maximum number of iterations for loops
    pub max_iterations: Option<usize>,
    
    /// Whether to enable verification mode (compare parallel vs sequential)
    pub verification_mode: bool,
    
    /// Whether replay mode is active (forces sequential execution)
    pub replay_mode: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_execution_time: None,
            max_iterations: Some(10_000),
            verification_mode: false,
            replay_mode: false,
        }
    }
}

/// Thread-local mutable state for parallel workers.
///
/// Each worker maintains its own `LocalState` that can be mutated during execution
/// without affecting other workers. This enables safe parallel execution without
/// shared mutable state.
///
/// **Validates: Requirements 6.2, 6.3 (Thread-Local State)**
#[derive(Debug, Clone)]
pub struct LocalState {
    /// Intermediate computation results stored during execution
    pub intermediate_values: Vec<Value>,
    
    /// Local performance metrics for this worker
    pub local_metrics: LocalMetrics,
}

impl LocalState {
    /// Creates a new empty local state for a worker.
    pub fn new() -> Self {
        Self {
            intermediate_values: Vec::new(),
            local_metrics: LocalMetrics::default(),
        }
    }
    
    /// Clears all intermediate values and resets metrics.
    pub fn clear(&mut self) {
        self.intermediate_values.clear();
        self.local_metrics = LocalMetrics::default();
    }
}

impl Default for LocalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Local performance metrics tracked by each worker.
#[derive(Debug, Clone, Default)]
pub struct LocalMetrics {
    /// Number of values processed by this worker
    pub values_processed: usize,
    
    /// Number of operations executed by this worker
    pub operations_executed: usize,
    
    /// Time spent in computation (excluding synchronization)
    pub computation_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;
    use std::time::Duration;

    // ===== DataPartition Tests =====

    #[test]
    fn test_data_partition_size() {
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        
        let partition = DataPartition {
            data: &data[0..3],
            start_index: 0,
            end_index: 3,
        };
        
        assert_eq!(partition.size(), 3);
    }

    #[test]
    fn test_data_partition_logical_index() {
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        
        let partition = DataPartition {
            data: &data[2..5],
            start_index: 2,
            end_index: 5,
        };
        
        assert_eq!(partition.logical_index(0), 2);
        assert_eq!(partition.logical_index(1), 3);
        assert_eq!(partition.logical_index(2), 4);
    }

    #[test]
    fn test_data_partition_is_valid() {
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ];
        
        let valid_partition = DataPartition {
            data: &data[0..2],
            start_index: 0,
            end_index: 2,
        };
        assert!(valid_partition.is_valid());
        
        let invalid_partition = DataPartition {
            data: &data[0..2],
            start_index: 0,
            end_index: 3,
        };
        assert!(!invalid_partition.is_valid());
    }

    // ===== ExecutionMetrics Tests =====

    #[test]
    fn test_execution_metrics_net_speedup_beneficial() {
        let metrics = ExecutionMetrics {
            sequential_time: Duration::from_millis(1000),
            parallel_time: Duration::from_millis(300),
            ordering_overhead: Duration::from_millis(50),
            sync_cost: Duration::from_millis(20),
            merge_cost: Duration::from_millis(30),
        };
        
        let speedup = metrics.net_speedup();
        assert!((speedup - 2.5).abs() < 0.01);
        assert!(speedup >= 2.0);
    }

    #[test]
    fn test_execution_metrics_net_speedup_marginal() {
        let metrics = ExecutionMetrics {
            sequential_time: Duration::from_millis(1000),
            parallel_time: Duration::from_millis(400),
            ordering_overhead: Duration::from_millis(100),
            sync_cost: Duration::from_millis(50),
            merge_cost: Duration::from_millis(50),
        };
        
        let speedup = metrics.net_speedup();
        assert!((speedup - 1.666).abs() < 0.01);
        assert!(speedup < 2.0);
    }

    #[test]
    fn test_execution_metrics_ordering_overhead_ratio() {
        let metrics = ExecutionMetrics {
            sequential_time: Duration::from_millis(1000),
            parallel_time: Duration::from_millis(400),
            ordering_overhead: Duration::from_millis(250),
            sync_cost: Duration::from_millis(0),
            merge_cost: Duration::from_millis(0),
        };
        
        let ratio = metrics.ordering_overhead_ratio();
        assert!((ratio - 0.625).abs() < 0.01);
        assert!(ratio > 0.5);
    }

    // ===== LocalState Tests =====

    #[test]
    fn test_local_state_new() {
        let state = LocalState::new();
        assert_eq!(state.intermediate_values.len(), 0);
        assert_eq!(state.local_metrics.values_processed, 0);
    }

    #[test]
    fn test_local_state_clear() {
        let mut state = LocalState::new();
        state.intermediate_values.push(Value::Number(42.0));
        state.local_metrics.values_processed = 10;
        
        state.clear();
        
        assert_eq!(state.intermediate_values.len(), 0);
        assert_eq!(state.local_metrics.values_processed, 0);
    }

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_iterations, Some(10_000));
        assert_eq!(config.verification_mode, false);
        assert_eq!(config.replay_mode, false);
    }
}
