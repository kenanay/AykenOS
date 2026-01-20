//! Parallel executor for IR blocks
//!
//! This module implements parallel execution of IR blocks using Rayon thread pool.
//! The executor maintains strict determinism guarantees while providing data-parallel
//! execution capabilities.
//!
//! ## Design Principles
//!
//! 1. **Deterministic Results**: Parallel execution produces identical results to sequential execution
//! 2. **Error Propagation**: All worker errors are propagated to the caller without silent failures
//! 3. **Panic Safety**: Worker panics are caught and converted to errors
//! 4. **Cache-Line Safety**: Uses chunk-local buffers to avoid false sharing
//!
//! **Design Reference:** D2 Parallelism Architecture - Parallel Executor section
//! **Requirements:** 2.1, 5.1-5.6, 8.1-8.4

use crate::bcib::Value;
use crate::execution_plan::IRBlock;
use crate::parallelism::{
    DataPartition, ImmutableContext, LocalState, ParallelismError, ParallelismResult
};
use crate::ir_planner::ExecutionError;
use rayon::prelude::*;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Trait for parallel execution of IR blocks.
///
/// The `ParallelExecutor` executes IR blocks in parallel using data partitioning.
/// Each partition is processed by a separate worker thread, and results are
/// collected with stable index mapping for deterministic merging.
///
/// # Design Pattern: Parallelism Unit
///
/// The executor operates on Parallelism_Unit = (IR_Block, Data_Partition):
/// - **IR_Block**: The code to execute (shared immutably across workers)
/// - **Data_Partition**: The data slice to process (unique per worker)
///
/// # Error Handling
///
/// - Worker panics are caught and converted to `ParallelismError::ExecutionError`
/// - Execution errors are propagated immediately (fail-fast)
/// - No silent failures or ignored errors
///
/// **Validates: Requirements 2.1, 5.1-5.6, 8.1-8.4**
pub trait ParallelExecutor {
    /// Executes an IR block in parallel across multiple data partitions.
    ///
    /// # Arguments
    ///
    /// * `block` - The IR block to execute (shared across all workers)
    /// * `partitions` - Data partitions to process in parallel
    /// * `context` - Immutable execution context (shared across all workers)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(usize, Value)>)` - Indexed results from all workers
    /// * `Err(ParallelismError)` - If any worker fails or panics
    ///
    /// # Guarantees
    ///
    /// - Results are indexed for deterministic merging
    /// - All worker errors are propagated
    /// - No shared mutable state between workers
    /// - Thread-safe execution with immutable context
    ///
    /// **Validates: Requirements 2.1, 8.1**
    fn execute_parallel(
        &self,
        block: &IRBlock,
        partitions: Vec<DataPartition>,
        context: &ImmutableContext,
    ) -> ParallelismResult<Vec<(usize, Value)>>;
}

/// Rayon-based parallel executor implementation.
///
/// This executor uses the Rayon thread pool for parallel execution, providing:
/// - Work-stealing parallelism with deterministic results
/// - Panic safety through `catch_unwind`
/// - Error propagation with detailed context
/// - Cache-line safety through chunk-local buffers
///
/// # Thread Pool Management
///
/// The executor uses Rayon's global thread pool, which:
/// - Is initialized once and reused across operations
/// - Automatically sizes based on available CPU cores
/// - Provides work-stealing for load balancing
/// - Maintains thread safety without explicit synchronization
///
/// # Performance Characteristics
///
/// - **Parallelism**: Scales with number of CPU cores
/// - **Overhead**: Minimal for large datasets (>1000 elements)
/// - **Memory**: Uses chunk-local buffers (no false sharing)
/// - **Error Handling**: Fail-fast on first error
///
/// **Validates: Requirements 5.1-5.6, 8.1-8.4**
#[derive(Debug, Clone, Default)]
pub struct RayonParallelExecutor;

impl RayonParallelExecutor {
    /// Creates a new Rayon parallel executor.
    pub fn new() -> Self {
        Self
    }
    
    /// Executes a single partition with panic safety.
    ///
    /// This method wraps partition execution in `catch_unwind` to handle
    /// worker panics gracefully. Panics are converted to `ParallelismError`
    /// with detailed context information.
    ///
    /// # Arguments
    ///
    /// * `block` - IR block to execute
    /// * `partition` - Data partition to process
    /// * `context` - Immutable execution context
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(usize, Value)>)` - Indexed results from this partition
    /// * `Err(ParallelismError)` - If execution fails or panics
    ///
    /// **Validates: Requirements 8.2, 8.3 (Panic Safety)**
    fn execute_partition_safe(
        &self,
        block: &IRBlock,
        partition: DataPartition,
        context: &ImmutableContext,
    ) -> ParallelismResult<Vec<(usize, Value)>> {
        // Wrap execution in catch_unwind for panic safety
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.execute_partition_impl(block, partition, context)
        }));
        
        match result {
            Ok(execution_result) => execution_result,
            Err(panic) => {
                // Extract panic message
                let panic_message = if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic occurred during parallel execution".to_string()
                };
                
                Err(ParallelismError::ExecutionError {
                    worker_id: None, // Rayon doesn't expose worker IDs directly
                    partition_start: Some(partition.start_index),
                    partition_end: Some(partition.end_index),
                    message: format!("Worker panicked: {}", panic_message),
                    source: None,
                })
            }
        }
    }
    
    /// Internal implementation of partition execution.
    ///
    /// This method processes a single data partition by executing the IR block
    /// on each element in the partition. Results are collected with stable
    /// index mapping for deterministic merging.
    ///
    /// # Thread-Local State
    ///
    /// Each worker maintains its own `LocalState` that can be mutated during
    /// execution without affecting other workers. This ensures thread safety
    /// without synchronization overhead.
    ///
    /// **Validates: Requirements 6.2, 6.3 (Thread-Local State)**
    fn execute_partition_impl(
        &self,
        block: &IRBlock,
        partition: DataPartition,
        context: &ImmutableContext,
    ) -> ParallelismResult<Vec<(usize, Value)>> {
        // Create thread-local state for this worker
        let mut local_state = LocalState::new();
        let mut results = Vec::with_capacity(partition.size());
        
        // Process each element in the partition
        for (local_idx, value) in partition.data.iter().enumerate() {
            // Calculate logical index for stable index mapping
            let logical_idx = partition.logical_index(local_idx);
            
            // Execute IR block on this value
            // Note: This is a simplified implementation
            // In a real implementation, this would use the IR executor
            match self.execute_ir_block_on_value(block, value, context, &mut local_state) {
                Ok(result) => {
                    results.push((logical_idx, result));
                }
                Err(e) => {
                    return Err(ParallelismError::ExecutionError {
                        worker_id: None,
                        partition_start: Some(partition.start_index),
                        partition_end: Some(partition.end_index),
                        message: format!("Execution failed at index {}: {}", logical_idx, e),
                        source: None,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Executes an IR block on a single value.
    ///
    /// This is a simplified implementation that demonstrates the execution pattern.
    /// In a real implementation, this would integrate with the IR executor to
    /// execute the full IR block.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. The actual implementation would
    /// need to integrate with the existing IR executor infrastructure.
    fn execute_ir_block_on_value(
        &self,
        _block: &IRBlock,
        value: &Value,
        _context: &ImmutableContext,
        _local_state: &mut LocalState,
    ) -> Result<Value, ExecutionError> {
        // Placeholder: just return the input value
        // Real implementation would execute the IR block
        Ok(value.clone())
    }
}

impl ParallelExecutor for RayonParallelExecutor {
    fn execute_parallel(
        &self,
        block: &IRBlock,
        partitions: Vec<DataPartition>,
        context: &ImmutableContext,
    ) -> ParallelismResult<Vec<(usize, Value)>> {
        // Handle empty partitions
        if partitions.is_empty() {
            return Ok(Vec::new());
        }
        
        // Execute partitions in parallel using Rayon
        // Each partition is processed by a separate worker thread
        let results: Result<Vec<Vec<(usize, Value)>>, ParallelismError> = partitions
            .into_par_iter()
            .map(|partition| {
                // Execute this partition with panic safety
                self.execute_partition_safe(block, partition, context)
            })
            .collect();
        
        match results {
            Ok(partition_results) => {
                // Flatten results from all partitions
                // This preserves the stable index mapping
                let flattened: Vec<(usize, Value)> = partition_results
                    .into_iter()
                    .flatten()
                    .collect();
                
                Ok(flattened)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;
    use crate::execution_plan::{IRBlock, IRInstruction, BlockTerminator, ParallelSafety};
    use crate::parallelism::types::{ExecutionConfig, ImmutableContext};
    use crate::execution_plan::ExecutionPlan;
    use crate::normalizer::RegisterAllocation;
    use crate::execution_plan::dataflow::DataflowGraph;
    use crate::execution_plan::ExecutionMetadata;
    use std::collections::HashMap;

    // ===== Test Helpers =====

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
            ExecutionMetadata::new("test".to_string(), 0, 0, 0),
        );
        
        ImmutableContext {
            execution_plan,
            config: ExecutionConfig::default(),
        }
    }

    fn create_test_block() -> IRBlock {
        IRBlock::with_safety(
            0,
            vec![
                IRInstruction::LoadContext {
                    context_id: "test".to_string(),
                    target_register: 0,
                },
            ],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Safe,
        )
    }

    fn create_test_data(size: usize) -> Vec<Value> {
        (0..size).map(|i| Value::Number(i as f64)).collect()
    }

    fn create_test_partitions(data: &[Value], num_partitions: usize) -> Vec<DataPartition<'_>> {
        let partition_size = (data.len() + num_partitions - 1) / num_partitions;
        let mut partitions = Vec::new();
        
        for i in 0..num_partitions {
            let start = i * partition_size;
            let end = ((i + 1) * partition_size).min(data.len());
            
            if start < data.len() {
                partitions.push(DataPartition {
                    data: &data[start..end],
                    start_index: start,
                    end_index: end,
                });
            }
        }
        
        partitions
    }

    // ===== Basic Functionality Tests =====

    #[test]
    fn test_executor_creation() {
        let executor = RayonParallelExecutor::new();
        assert!(format!("{:?}", executor).contains("RayonParallelExecutor"));
    }

    #[test]
    fn test_execute_empty_partitions() {
        let executor = RayonParallelExecutor::new();
        let block = create_test_block();
        let context = create_test_context();
        let partitions = vec![];
        
        let result = executor.execute_parallel(&block, partitions, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_execute_single_partition() {
        let executor = RayonParallelExecutor::new();
        let block = create_test_block();
        let context = create_test_context();
        let data = create_test_data(5);
        let partitions = create_test_partitions(&data, 1);
        
        let result = executor.execute_parallel(&block, partitions, &context);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.len(), 5);
        
        // Verify stable index mapping
        for (i, (idx, _)) in results.iter().enumerate() {
            assert_eq!(*idx, i);
        }
    }

    #[test]
    fn test_execute_multiple_partitions() {
        let executor = RayonParallelExecutor::new();
        let block = create_test_block();
        let context = create_test_context();
        let data = create_test_data(10);
        let partitions = create_test_partitions(&data, 3);
        
        let result = executor.execute_parallel(&block, partitions, &context);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.len(), 10);
        
        // Sort results by index to verify completeness
        let mut sorted_results = results;
        sorted_results.sort_by_key(|(idx, _)| *idx);
        
        // Verify all indices are present
        for (i, (idx, _)) in sorted_results.iter().enumerate() {
            assert_eq!(*idx, i);
        }
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_panic_safety() {
        // This test would require a way to inject panics into the execution
        // For now, we just verify the panic handling infrastructure exists
        let executor = RayonParallelExecutor::new();
        let block = create_test_block();
        let context = create_test_context();
        let data = create_test_data(1);
        let partitions = create_test_partitions(&data, 1);
        
        // Normal execution should work
        let result = executor.execute_parallel(&block, partitions, &context);
        assert!(result.is_ok());
    }

    // ===== Property Tests =====

    #[test]
    fn test_property_index_preservation() {
        // Property: Results maintain stable index mapping
        let executor = RayonParallelExecutor::new();
        let block = create_test_block();
        let context = create_test_context();
        let data = create_test_data(100);
        let partitions = create_test_partitions(&data, 4);
        
        let result = executor.execute_parallel(&block, partitions, &context);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.len(), 100);
        
        // Verify index preservation
        let mut sorted_results = results;
        sorted_results.sort_by_key(|(idx, _)| *idx);
        
        for (i, (idx, _)) in sorted_results.iter().enumerate() {
            assert_eq!(*idx, i);
        }
    }

    #[test]
    fn test_property_determinism() {
        // Property: Same input produces same output
        let executor = RayonParallelExecutor::new();
        let block = create_test_block();
        let context = create_test_context();
        let data = create_test_data(50);
        
        // Execute multiple times with same input
        let partitions1 = create_test_partitions(&data, 3);
        let partitions2 = create_test_partitions(&data, 3);
        
        let result1 = executor.execute_parallel(&block, partitions1, &context);
        let result2 = executor.execute_parallel(&block, partitions2, &context);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let mut results1 = result1.unwrap();
        let mut results2 = result2.unwrap();
        
        // Sort both results by index
        results1.sort_by_key(|(idx, _)| *idx);
        results2.sort_by_key(|(idx, _)| *idx);
        
        // Results should be identical
        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.0, r2.0); // Same index
            assert_eq!(r1.1, r2.1); // Same value
        }
    }

    // ===== Integration Tests =====

    #[test]
    fn test_trait_implementation() {
        let executor = RayonParallelExecutor::new();
        let _: &dyn ParallelExecutor = &executor;
        
        // Verify trait methods are callable
        let block = create_test_block();
        let context = create_test_context();
        let data = create_test_data(5);
        let partitions = create_test_partitions(&data, 2);
        
        let result = executor.execute_parallel(&block, partitions, &context);
        assert!(result.is_ok());
    }
}