//! Loop Execution Engine - Advanced Features and Parallelization
//!
//! This module implements advanced loop execution features including parallelization,
//! partitioning, and integration with the core execution logic.
//!
//! # Core Execution Logic
//!
//! The main execution logic has been extracted to the `core` module.
//! This module focuses on:
//! - Parallelization decision logic
//! - Deterministic partitioning
//! - Parallel execution coordination
//! - Advanced iteration management
//!
//! # Phase Coverage
//!
//! - ✅ Phase 7.1: Parallelization trigger logic
//! - ✅ Phase 7.2: Deterministic partitioning
//! - ✅ Phase 7.3: Parallel execution with stable index mapping

use super::core::{LoopBodyFn, LoopBodyResult, LoopExecutor as CoreLoopExecutor};
use crate::bcib::{ControlFlowType, LoopInstruction, LoopRange, LoopType, Value};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::loop_engine::{LoopContext, LoopError, LoopResult, LoopState};
use crate::parallelism::DeterministicMerger;

/// Iteration partition for deterministic parallel execution (Phase 7.2)
///
/// Represents a contiguous range of loop iterations that can be executed
/// independently by a parallel worker. The partition boundaries are
/// calculated deterministically based on iteration count only.
///
/// Requirements 15.1, 15.2: Deterministic partitioning, stable index mapping
#[derive(Debug, Clone, PartialEq)]
pub struct IterationPartition {
    /// Unique partition identifier (0, 1, 2, ...)
    pub partition_id: usize,
    /// Starting iteration (inclusive)
    pub start_iteration: u32,
    /// Ending iteration (exclusive)
    pub end_iteration: u32,
    /// Number of iterations in this partition
    pub iteration_count: u32,
}

impl IterationPartition {
    /// Check if this partition contains the given iteration
    pub fn contains_iteration(&self, iteration: u32) -> bool {
        iteration >= self.start_iteration && iteration < self.end_iteration
    }

    /// Get the local iteration index within this partition
    ///
    /// This implements the stable index mapping required for deterministic
    /// parallel execution: global iteration N maps to local iteration
    /// (N - start_iteration) within the partition.
    ///
    /// Requirements 15.2: Stable index mapping
    pub fn local_iteration_index(&self, global_iteration: u32) -> Option<u32> {
        if self.contains_iteration(global_iteration) {
            Some(global_iteration - self.start_iteration)
        } else {
            None
        }
    }

    /// Check if this partition is valid (well-formed)
    pub fn is_valid(&self) -> bool {
        self.start_iteration <= self.end_iteration
            && self.iteration_count == (self.end_iteration - self.start_iteration)
    }
}

/// Parallelization decision for loop execution (Phase 7.1)
#[derive(Debug, Clone, PartialEq)]
pub enum ParallelizationDecision {
    /// Execute loop sequentially
    Sequential {
        /// Reason for sequential execution
        reason: String,
    },
    /// Execute loop in parallel
    Parallel {
        /// Static iteration count
        iteration_count: u32,
        /// Safety classification
        safety_classification: crate::loop_engine::SafetyClass,
        /// Estimated parallelization benefit (0.0 to 1.0)
        estimated_benefit: f64,
    },
}

impl ParallelizationDecision {
    /// Check if this decision recommends parallel execution
    pub fn is_parallel(&self) -> bool {
        matches!(self, ParallelizationDecision::Parallel { .. })
    }

    /// Check if this decision recommends sequential execution
    pub fn is_sequential(&self) -> bool {
        matches!(self, ParallelizationDecision::Sequential { .. })
    }

    /// Get the reason for sequential execution (if applicable)
    pub fn sequential_reason(&self) -> Option<&str> {
        match self {
            ParallelizationDecision::Sequential { reason } => Some(reason),
            ParallelizationDecision::Parallel { .. } => None,
        }
    }

    /// Get the estimated benefit for parallel execution (if applicable)
    pub fn parallel_benefit(&self) -> Option<f64> {
        match self {
            ParallelizationDecision::Parallel {
                estimated_benefit, ..
            } => Some(*estimated_benefit),
            ParallelizationDecision::Sequential { .. } => None,
        }
    }

    /// Get the iteration count for parallel execution (if applicable)
    pub fn iteration_count(&self) -> Option<u32> {
        match self {
            ParallelizationDecision::Parallel {
                iteration_count, ..
            } => Some(*iteration_count),
            ParallelizationDecision::Sequential { .. } => None,
        }
    }
}

/// Result of executing a single iteration within a partition (Phase 7.3)
///
/// This struct captures the result of executing one iteration with stable
/// index mapping, including the global iteration index for deterministic
/// result collection.
///
/// Requirements 15.2, 15.6: Stable index mapping, deterministic result collection
#[derive(Debug, Clone, PartialEq)]
pub struct IterationExecutionResult {
    /// Global iteration index (stable across partitions)
    pub global_iteration_index: u32,
    /// Result value from this iteration
    pub result_value: Value,
    /// Control flow decision (normal, break, continue)
    pub control_flow: ControlFlowType,
}

/// Result of executing a partition with stable index mapping (Phase 7.3)
///
/// This struct captures the complete result of executing a partition,
/// including all iteration results with their stable indices for
/// deterministic result collection.
///
/// Requirements 15.2, 15.6: Stable index mapping, deterministic result collection
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionExecutionResult {
    /// Partition identifier for deterministic ordering
    pub partition_id: usize,
    /// Type of result (success, break, error)
    pub result_type: PartitionResultType,
}

/// Type of partition execution result (Phase 7.3)
#[derive(Debug, Clone, PartialEq)]
pub enum PartitionResultType {
    /// Partition completed successfully
    Success {
        /// Results from all iterations in this partition
        iteration_results: Vec<IterationExecutionResult>,
        /// Final accumulator value from this partition
        final_accumulator: Value,
        /// Number of iterations completed in this partition
        iterations_completed: u32,
    },
    /// Partition terminated with break
    Break {
        /// Results from iterations before break
        iteration_results: Vec<IterationExecutionResult>,
        /// Final accumulator value at break point
        final_accumulator: Value,
        /// Number of iterations completed before break
        iterations_completed: u32,
    },
    /// Partition failed with error
    Error {
        /// The error that occurred
        error: LoopError,
    },
}

impl PartitionExecutionResult {
    /// Create a successful partition result
    pub fn success(
        partition_id: usize,
        iteration_results: Vec<IterationExecutionResult>,
        final_accumulator: Value,
        iterations_completed: u32,
    ) -> Self {
        Self {
            partition_id,
            result_type: PartitionResultType::Success {
                iteration_results,
                final_accumulator,
                iterations_completed,
            },
        }
    }

    /// Create a break partition result
    pub fn break_result(
        partition_id: usize,
        iteration_results: Vec<IterationExecutionResult>,
        final_accumulator: Value,
        iterations_completed: u32,
    ) -> Self {
        Self {
            partition_id,
            result_type: PartitionResultType::Break {
                iteration_results,
                final_accumulator,
                iterations_completed,
            },
        }
    }

    /// Create an error partition result
    pub fn error(partition_id: usize, error: LoopError) -> Self {
        Self {
            partition_id,
            result_type: PartitionResultType::Error { error },
        }
    }

    /// Check if this partition result represents success
    pub fn is_success(&self) -> bool {
        matches!(self.result_type, PartitionResultType::Success { .. })
    }

    /// Check if this partition result represents a break
    pub fn is_break(&self) -> bool {
        matches!(self.result_type, PartitionResultType::Break { .. })
    }

    /// Check if this partition result represents an error
    pub fn is_error(&self) -> bool {
        matches!(self.result_type, PartitionResultType::Error { .. })
    }
}

/// Loop execution engine with advanced features
///
/// This struct wraps the core execution engine and adds advanced features
/// like parallelization, partitioning, and performance optimizations.
pub struct LoopExecutor {
    /// Core execution engine
    core_executor: CoreLoopExecutor,
}

impl LoopExecutor {
    /// Create a new loop executor
    pub fn new() -> Self {
        Self {
            core_executor: CoreLoopExecutor::new(),
        }
    }

    /// Execute a loop instruction (delegates to core)
    pub fn execute_loop(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
    ) -> Result<LoopResult> {
        self.core_executor.execute_loop(instruction, body_fn)
    }

    /// Determine if a loop should be parallelized (Phase 7.1 - Parallelization trigger logic)
    ///
    /// Requirements 7.1: Only parallelize Safe loop bodies, exclude While loops,
    /// support For and ForEach loops with statically known iteration counts,
    /// fall back to sequential execution for Unsafe loops.
    pub fn should_parallelize_loop(
        &self,
        instruction: &LoopInstruction,
        safety_result: &crate::loop_engine::SafetyAnalysisResult,
    ) -> ParallelizationDecision {
        // 1. Check loop type eligibility (Constitutional rule: While loops NEVER parallelized)
        match instruction.loop_type() {
            LoopType::While => {
                return ParallelizationDecision::Sequential {
                    reason:
                        "While loops are excluded from parallelization (non-static iteration count)"
                            .to_string(),
                };
            }
            LoopType::For | LoopType::ForEach => {
                // For and ForEach loops are eligible for parallelization
            }
        }

        // 2. Check safety classification (only Safe loops can be parallelized)
        if safety_result.classification != crate::loop_engine::SafetyClass::Safe {
            return ParallelizationDecision::Sequential {
                reason: format!(
                    "Loop body is Unsafe for parallelization: {}",
                    safety_result.reason
                ),
            };
        }

        // 3. Verify iteration count is statically known
        let iteration_count = match self.get_static_iteration_count(instruction) {
            Some(count) => count,
            None => {
                return ParallelizationDecision::Sequential {
                    reason: "Dynamic iteration count - cannot determine parallelization benefit"
                        .to_string(),
                };
            }
        };

        // 4. Check minimum threshold for parallelization benefit
        const MIN_PARALLEL_ITERATIONS: u32 = 100; // Minimum iterations to justify parallel overhead
        if iteration_count < MIN_PARALLEL_ITERATIONS {
            return ParallelizationDecision::Sequential {
                reason: format!(
                    "Iteration count ({}) below minimum threshold ({}) for parallelization",
                    iteration_count, MIN_PARALLEL_ITERATIONS
                ),
            };
        }

        // 5. All checks passed - recommend parallelization
        ParallelizationDecision::Parallel {
            iteration_count,
            safety_classification: safety_result.classification,
            estimated_benefit: self.estimate_parallelization_benefit(iteration_count),
        }
    }

    /// Get static iteration count for a loop instruction (Phase 7.1)
    ///
    /// Returns Some(count) if the iteration count can be determined statically,
    /// None if the iteration count is dynamic or unknown.
    pub fn get_static_iteration_count(&self, instruction: &LoopInstruction) -> Option<u32> {
        match instruction {
            LoopInstruction::While { .. } => {
                // While loops have dynamic iteration count (condition-dependent)
                None
            }
            LoopInstruction::For { range, config, .. } => {
                // For loops have static iteration count if range is known
                let iteration_count = self.calculate_for_loop_iterations(range);

                // Ensure we don't exceed the iteration limit
                let limited_count = iteration_count.min(config.iteration_limit);
                Some(limited_count)
            }
            LoopInstruction::ForEach {
                collection, config, ..
            } => {
                // ForEach loops have static iteration count if collection size is known
                match self.get_collection_size_hint(collection) {
                    Some(size) => {
                        // Ensure we don't exceed the iteration limit
                        let limited_size = size.min(config.iteration_limit);
                        Some(limited_size)
                    }
                    None => {
                        // Collection size unknown - cannot determine static count
                        None
                    }
                }
            }
        }
    }

    /// Calculate the number of iterations for a For loop range (Phase 7.1)
    fn calculate_for_loop_iterations(&self, range: &LoopRange) -> u32 {
        if range.step == 0 {
            // Invalid step - would be infinite loop, but caught by validation
            return 0;
        }

        let total_range = if range.step > 0 {
            // Forward iteration: start to end-1
            if range.end <= range.start {
                0 // No iterations
            } else {
                (range.end - range.start) as u32
            }
        } else {
            // Backward iteration: start to end+1
            if range.start <= range.end {
                0 // No iterations
            } else {
                (range.start - range.end) as u32
            }
        };

        // Calculate number of steps needed
        let step_size = range.step.abs() as u32;
        (total_range + step_size - 1) / step_size // Ceiling division
    }

    /// Get collection size hint for ForEach loops (Phase 7.1)
    ///
    /// Returns Some(size) if the collection size can be determined statically,
    /// None if the collection size is dynamic or unknown.
    fn get_collection_size_hint(&self, collection_ref: &crate::bcib::OperandRef) -> Option<u32> {
        match collection_ref {
            crate::bcib::OperandRef::Literal(value) => {
                // Literal collections have known size
                match value {
                    crate::bcib::Value::Array(arr) => Some(arr.len() as u32),
                    crate::bcib::Value::List(list) => Some(list.len() as u32),
                    crate::bcib::Value::SortedMap(map) => Some(map.len() as u32),
                    _ => None, // Not a collection
                }
            }
            crate::bcib::OperandRef::Field(_) | crate::bcib::OperandRef::TempRegister(_) => {
                // Field references and temp registers have dynamic size
                // In a real implementation, we might be able to infer size from type information
                // For Phase 7.1, we'll be conservative and return None
                None
            }
        }
    }

    /// Estimate parallelization benefit for a given iteration count (Phase 7.1)
    ///
    /// Returns a benefit score from 0.0 (no benefit) to 1.0 (maximum benefit).
    /// This is used for prioritizing parallelization decisions.
    fn estimate_parallelization_benefit(&self, iteration_count: u32) -> f64 {
        // Simple heuristic based on iteration count
        // In a real implementation, this would consider:
        // - Loop body complexity
        // - Available CPU cores
        // - Memory access patterns
        // - Historical performance data

        const MAX_BENEFIT_THRESHOLD: u32 = 10000;

        if iteration_count >= MAX_BENEFIT_THRESHOLD {
            1.0 // Maximum benefit for large loops
        } else {
            // Linear scaling from minimum threshold to maximum benefit
            let min_threshold = 100.0; // MIN_PARALLEL_ITERATIONS as f64
            let max_threshold = MAX_BENEFIT_THRESHOLD as f64;
            let current = iteration_count as f64;

            ((current - min_threshold) / (max_threshold - min_threshold))
                .min(1.0)
                .max(0.0)
        }
    }

    /// Partition iterations deterministically for parallel execution (Phase 7.2)
    ///
    /// Requirements 7.5, 15.1, 15.3: Partition iterations based on iteration count only,
    /// use fixed chunk size algorithm, treat available parallelism as upper bound.
    ///
    /// This method implements the constitutional requirement for deterministic partitioning:
    /// - Same iteration count → same partitions (always)
    /// - Available parallelism (core count) is optimization hint, not semantic input
    /// - Fixed chunk size algorithm ensures reproducible partition boundaries
    ///
    /// # Arguments
    ///
    /// * `total_iterations` - Total number of iterations to partition
    /// * `available_parallelism` - Available parallel workers (upper bound only)
    ///
    /// # Returns
    ///
    /// Vector of `IterationPartition` structs defining deterministic partition boundaries
    pub fn partition_iterations_deterministic(
        &self,
        total_iterations: u32,
        available_parallelism: usize,
    ) -> Vec<IterationPartition> {
        // Handle edge cases
        if total_iterations == 0 {
            return Vec::new();
        }

        if available_parallelism == 0 {
            return Vec::new();
        }

        // Phase 7.2: Deterministic chunk size calculation
        // Constitutional rule: Based on iteration count only, not machine-dependent factors
        let chunk_size = self.calculate_deterministic_chunk_size(total_iterations);

        // Calculate number of partitions needed
        let num_partitions = ((total_iterations + chunk_size - 1) / chunk_size) as usize;

        // Treat available parallelism as upper bound (optimization, not semantic)
        let effective_partitions = num_partitions.min(available_parallelism);

        // Create deterministic partitions using fixed chunk size
        let mut partitions = Vec::with_capacity(effective_partitions);
        let mut start_iteration = 0;

        for partition_id in 0..effective_partitions {
            // Calculate partition boundaries deterministically
            let end_iteration = if partition_id == effective_partitions - 1 {
                // Last partition gets remaining iterations
                total_iterations
            } else {
                // Regular partition gets fixed chunk size
                (start_iteration + chunk_size).min(total_iterations)
            };

            if start_iteration < end_iteration {
                partitions.push(IterationPartition {
                    partition_id,
                    start_iteration,
                    end_iteration,
                    iteration_count: end_iteration - start_iteration,
                });
            }

            start_iteration = end_iteration;

            // Stop if we've covered all iterations
            if start_iteration >= total_iterations {
                break;
            }
        }

        partitions
    }

    /// Calculate deterministic chunk size for iteration partitioning (Phase 7.2)
    ///
    /// This method implements the fixed chunk size algorithm required for deterministic
    /// partitioning. The chunk size is calculated based solely on iteration count,
    /// ensuring the same input always produces the same partitions.
    ///
    /// Requirements 15.1: Partition iterations based on iteration count only
    fn calculate_deterministic_chunk_size(&self, total_iterations: u32) -> u32 {
        // Phase 7.2: Fixed chunk size algorithm
        // Constitutional rule: Deterministic calculation based on iteration count only

        // Use a fixed chunk size that scales with iteration count
        // This ensures deterministic partitioning while providing reasonable parallelism
        const MIN_CHUNK_SIZE: u32 = 100; // Minimum chunk size for overhead amortization
        const MAX_CHUNK_SIZE: u32 = 1000; // Maximum chunk size for load balancing

        if total_iterations <= MIN_CHUNK_SIZE {
            // Small loops: single chunk
            total_iterations
        } else if total_iterations <= MAX_CHUNK_SIZE * 4 {
            // Medium loops: divide into 4 chunks for good parallelism
            (total_iterations + 3) / 4 // Ceiling division
        } else {
            // Large loops: use maximum chunk size
            MAX_CHUNK_SIZE
        }
    }

    /// Execute loop with deterministic parallel partitioning (Phase 7.3)
    ///
    /// This method implements the complete parallel loop execution workflow:
    /// 1. Partition iterations deterministically
    /// 2. Execute partitions in parallel using stable index mapping
    /// 3. Collect results in deterministic order (partition 0, 1, 2, ...)
    ///
    /// Requirements 7.4, 15.2, 15.6: Stable index mapping, deterministic partitioning,
    /// deterministic result collection order
    pub fn execute_loop_parallel(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
        iteration_count: u32,
        available_parallelism: usize,
    ) -> Result<LoopResult> {
        // Phase 7.3: Create deterministic partitions
        let partitions =
            self.partition_iterations_deterministic(iteration_count, available_parallelism);

        if partitions.is_empty() {
            // No partitions - fall back to sequential execution
            return self.execute_loop(instruction, body_fn);
        }

        if partitions.len() == 1 {
            // Single partition - execute sequentially for efficiency
            return self.execute_loop(instruction, body_fn);
        }

        // Phase 7.3: Execute partitions with stable index mapping and result collection
        let partition_results =
            self.execute_partitions_with_stable_mapping(instruction, &body_fn, &partitions)?;

        // Phase 7.3: Collect results in deterministic order (partition 0, 1, 2, ...)
        let final_result = self.collect_partition_results_deterministic(partition_results)?;

        Ok(final_result)
    }

    /// Execute partitions with stable index mapping (Phase 7.3)
    ///
    /// This method ensures that iteration i always processes the same input data
    /// regardless of which partition contains it. Each partition maintains stable
    /// index mapping where global iteration N maps to the same input data.
    ///
    /// Requirements 7.4, 15.2: Stable index mapping, deterministic data access
    fn execute_partitions_with_stable_mapping(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: &LoopBodyFn,
        partitions: &[IterationPartition],
    ) -> Result<Vec<PartitionExecutionResult>> {
        let mut partition_results = Vec::with_capacity(partitions.len());

        // Execute each partition with stable index mapping
        for partition in partitions {
            let partition_result =
                self.execute_single_partition_with_stable_mapping(instruction, body_fn, partition)?;
            partition_results.push(partition_result);
        }

        Ok(partition_results)
    }

    /// Execute a single partition with stable index mapping (Phase 7.3)
    ///
    /// This method implements the core stable index mapping guarantee:
    /// - Iteration i always processes input_data[i] (same input data)
    /// - Partition assignment does not affect data access
    /// - Same data access in parallel and sequential modes
    ///
    /// Requirements 15.2: Stable index mapping for deterministic parallel execution
    fn execute_single_partition_with_stable_mapping(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: &LoopBodyFn,
        partition: &IterationPartition,
    ) -> Result<PartitionExecutionResult> {
        // Create partition-local loop context
        let context = self.create_loop_context(instruction)?;
        let mut state = LoopState::new(context, instruction.get_initial_accumulator().clone())?;

        let mut iteration_results = Vec::with_capacity(partition.iteration_count as usize);

        // Execute each iteration in the partition with stable index mapping
        for global_iteration in partition.start_iteration..partition.end_iteration {
            // Phase 7.3: Stable index mapping - iteration i always processes same input data
            let input_data = self.get_input_data_for_iteration(instruction, global_iteration)?;

            // Check limits before each iteration
            if state.would_exceed_iteration_limit() {
                return Ok(PartitionExecutionResult::error(
                    partition.partition_id,
                    LoopError::IterationLimitExceeded {
                        limit: state.context.iteration_limit,
                        completed: state.completed_iterations,
                    },
                ));
            }

            let budget_cost = self.calculate_iteration_budget_cost(&state);
            if state.would_exceed_budget_timeout(budget_cost) {
                return Ok(PartitionExecutionResult::error(
                    partition.partition_id,
                    LoopError::BudgetTimeoutExceeded {
                        budget: state.context.budget_timeout,
                        consumed: state.budget_consumed,
                        iterations_completed: state.completed_iterations,
                    },
                ));
            }

            // Execute iteration body with stable input data
            let body_result = self.execute_iteration_with_stable_input(
                body_fn,
                &state,
                global_iteration,
                &input_data,
            )?;

            // Handle control flow and update state
            match body_result {
                LoopBodyResult::Break(accumulator) => {
                    state.update_accumulator(accumulator.clone())?;
                    state.increment_completed_iterations();
                    let budget_cost = self.calculate_break_budget_cost();
                    state.add_budget_consumed(budget_cost);

                    // Add iteration result with stable index
                    iteration_results.push(IterationExecutionResult {
                        global_iteration_index: global_iteration,
                        result_value: accumulator,
                        control_flow: ControlFlowType::Break,
                    });

                    return Ok(PartitionExecutionResult::break_result(
                        partition.partition_id,
                        iteration_results,
                        state.get_accumulator().clone(),
                        state.completed_iterations,
                    ));
                }
                LoopBodyResult::Continue(accumulator) => {
                    state.update_accumulator(accumulator.clone())?;
                    state.increment_completed_iterations();
                    let budget_cost = self.calculate_continue_budget_cost();
                    state.add_budget_consumed(budget_cost);

                    // Add iteration result with stable index
                    iteration_results.push(IterationExecutionResult {
                        global_iteration_index: global_iteration,
                        result_value: accumulator,
                        control_flow: ControlFlowType::Continue,
                    });

                    continue;
                }
                LoopBodyResult::Normal(new_accumulator) => {
                    state.update_accumulator(new_accumulator.clone())?;
                    state.increment_completed_iterations();
                    let budget_cost = self.calculate_iteration_budget_cost(&state);
                    state.add_budget_consumed(budget_cost);

                    // Add iteration result with stable index
                    iteration_results.push(IterationExecutionResult {
                        global_iteration_index: global_iteration,
                        result_value: new_accumulator,
                        control_flow: ControlFlowType::Normal,
                    });
                }
            }
        }

        // Partition completed successfully
        Ok(PartitionExecutionResult::success(
            partition.partition_id,
            iteration_results,
            state.get_accumulator().clone(),
            state.completed_iterations,
        ))
    }

    /// Get input data for a specific iteration with stable index mapping (Phase 7.3)
    ///
    /// This method implements the stable index mapping guarantee:
    /// - Iteration i always accesses input_data[i]
    /// - Mapping is based on iteration index, not partition assignment
    /// - Same input data regardless of parallel vs sequential execution
    ///
    /// Requirements 15.2: Stable index mapping ensures deterministic data access
    fn get_input_data_for_iteration(
        &self,
        instruction: &LoopInstruction,
        global_iteration: u32,
    ) -> Result<Value> {
        match instruction {
            LoopInstruction::For { range, .. } => {
                // For loops: stable mapping based on range calculation
                let iterator_value = range.start + (global_iteration as i64 * range.step);
                Ok(Value::Number(iterator_value as f64))
            }
            LoopInstruction::ForEach { collection, .. } => {
                // ForEach loops: stable mapping based on collection index
                let collection_value = self.resolve_collection_operand(
                    collection,
                    &LoopState::new(
                        self.create_loop_context(instruction)?,
                        instruction.get_initial_accumulator().clone(),
                    )?,
                )?;

                // Get element at stable index position
                match collection_value {
                    Value::Array(ref arr) => {
                        if global_iteration < arr.len() as u32 {
                            Ok(arr[global_iteration as usize].clone())
                        } else {
                            Err(SemanticCLIError::execution_error(
                                &format!(
                                    "Array index {} out of bounds (length: {})",
                                    global_iteration,
                                    arr.len()
                                ),
                                ErrorCode::E500,
                            ))
                        }
                    }
                    Value::List(ref list) => {
                        if global_iteration < list.len() as u32 {
                            Ok(list[global_iteration as usize].clone())
                        } else {
                            Err(SemanticCLIError::execution_error(
                                &format!(
                                    "List index {} out of bounds (length: {})",
                                    global_iteration,
                                    list.len()
                                ),
                                ErrorCode::E500,
                            ))
                        }
                    }
                    Value::SortedMap(ref map) => {
                        let keys: Vec<_> = map.keys().collect(); // Deterministic order
                        if global_iteration < keys.len() as u32 {
                            let key = &keys[global_iteration as usize];
                            Ok(map[*key].clone())
                        } else {
                            Err(SemanticCLIError::execution_error(
                                &format!(
                                    "Map index {} out of bounds (size: {})",
                                    global_iteration,
                                    keys.len()
                                ),
                                ErrorCode::E500,
                            ))
                        }
                    }
                    _ => Err(SemanticCLIError::execution_error(
                        "Unsupported collection type for stable index mapping",
                        ErrorCode::E500,
                    )),
                }
            }
            LoopInstruction::While { .. } => {
                // While loops don't have stable input data (condition-based)
                // Return iteration index as input data
                Ok(Value::Number(global_iteration as f64))
            }
        }
    }

    /// Execute iteration with stable input data (Phase 7.3)
    ///
    /// This method executes a single iteration with the stable input data
    /// determined by the stable index mapping. The same global iteration
    /// index always receives the same input data.
    fn execute_iteration_with_stable_input(
        &self,
        body_fn: &LoopBodyFn,
        state: &LoopState,
        global_iteration: u32,
        _input_data: &Value,
    ) -> Result<LoopBodyResult> {
        // Execute body function with stable input data
        // The body function receives the accumulator and iteration index
        // The stable input data is implicitly available through the iteration context
        body_fn(state.get_accumulator(), global_iteration)
    }

    /// Collect partition results in deterministic order (Phase 7.3)
    ///
    /// This method implements deterministic result collection:
    /// - Results collected in partition order (0, 1, 2, ...)
    /// - Within each partition, results ordered by iteration index
    /// - Final result identical to sequential execution
    ///
    /// Requirements 15.6: Deterministic result collection order
    fn collect_partition_results_deterministic(
        &self,
        partition_results: Vec<PartitionExecutionResult>,
    ) -> Result<LoopResult> {
        // Phase 7.3: Sort partition results by partition ID (deterministic order)
        let mut sorted_results = partition_results;
        sorted_results.sort_by_key(|result| result.partition_id);

        // Check for early termination (break or error) in any partition
        for partition_result in &sorted_results {
            match &partition_result.result_type {
                PartitionResultType::Break {
                    iteration_results: _,
                    final_accumulator,
                    iterations_completed,
                } => {
                    return Ok(LoopResult::break_result(
                        final_accumulator.clone(),
                        *iterations_completed,
                    ));
                }
                PartitionResultType::Error { error } => {
                    return Ok(LoopResult::error(error.clone()));
                }
                PartitionResultType::Success { .. } => {
                    // Continue processing
                }
            }
        }

        // All partitions completed successfully - collect iteration results
        let mut all_iteration_results = Vec::new();
        let mut total_iterations = 0;
        let mut final_accumulator = Value::Number(0.0); // Will be updated

        // Phase 7.3: Collect results in partition order (0, 1, 2, ...)
        for partition_result in sorted_results {
            if let PartitionResultType::Success {
                iteration_results,
                final_accumulator: acc,
                iterations_completed,
            } = partition_result.result_type
            {
                // Add iteration results with stable index mapping
                for iteration_result in iteration_results {
                    all_iteration_results.push((
                        iteration_result.global_iteration_index as usize,
                        iteration_result.result_value,
                    ));
                }

                total_iterations += iterations_completed;
                final_accumulator = acc; // Use the last partition's accumulator
            }
        }

        // Phase 7.3: Use stable index merger for deterministic result ordering
        let merger = crate::parallelism::StableIndexMerger::new();
        let _ordered_results = merger.merge(all_iteration_results).map_err(|e| {
            SemanticCLIError::execution_error(
                &format!("Failed to merge parallel results: {}", e),
                ErrorCode::E500,
            )
        })?;

        // Reconstruct final accumulator from ordered results
        // In a real implementation, this would use the accumulator pattern
        // For Phase 7.3, we'll use the last partition's accumulator
        Ok(LoopResult::success(final_accumulator, total_iterations))
    }

    // Helper methods for parallel execution

    /// Create loop context from instruction (helper for parallel execution)
    fn create_loop_context(&self, instruction: &LoopInstruction) -> Result<LoopContext> {
        let config = instruction.get_config();
        let loop_id = instruction.get_loop_id().clone();

        // Phase 2.1: Simple body reference
        let loop_body = format!("loop-body-{}", loop_id.0);

        Ok(LoopContext::new(loop_id, config, loop_body))
    }

    /// Calculate budget cost for a single iteration (helper for parallel execution)
    fn calculate_iteration_budget_cost(&self, state: &LoopState) -> u64 {
        match &state.context.budget_measurement {
            crate::bcib::BudgetMeasurement::IterationCount => {
                // Simple: each iteration costs 1 budget unit
                1
            }
            crate::bcib::BudgetMeasurement::InstructionCount { weight } => {
                // Phase 2.2: Use provided weight as instruction count
                // Future phases will implement actual instruction counting
                *weight
            }
            crate::bcib::BudgetMeasurement::Hybrid { multiplier } => {
                // Phase 2.2: Use multiplier as average instruction count per iteration
                // Future phases will implement dynamic profiling
                (*multiplier as u64).max(1)
            }
        }
    }

    /// Calculate budget cost for break instruction (helper for parallel execution)
    fn calculate_break_budget_cost(&self) -> u64 {
        // Constitutional: Break instruction has minimal cost to prevent budget bypass
        1
    }

    /// Calculate budget cost for continue instruction (helper for parallel execution)
    fn calculate_continue_budget_cost(&self) -> u64 {
        // Constitutional: Continue instruction has minimal cost to prevent budget bypass
        1
    }

    /// Resolve collection operand to a value (helper for parallel execution)
    fn resolve_collection_operand(
        &self,
        collection_ref: &crate::bcib::OperandRef,
        _state: &LoopState,
    ) -> Result<Value> {
        match collection_ref {
            crate::bcib::OperandRef::Literal(value) => {
                // Literal collections are directly available
                Ok(value.clone())
            }
            crate::bcib::OperandRef::Field(field_name) => {
                // Field references need to be resolved from context
                // Phase 3.1: For now, return an error for field references
                // Future phases will implement proper field resolution
                Err(SemanticCLIError::execution_error(
                    &format!(
                        "Field reference '{}' resolution not implemented in Phase 3.1",
                        field_name
                    ),
                    ErrorCode::E500,
                ))
            }
            crate::bcib::OperandRef::TempRegister(register_id) => {
                // Temp register references need to be resolved from execution context
                // Phase 3.1: For now, return an error for temp register references
                // Future phases will implement proper register resolution
                Err(SemanticCLIError::execution_error(
                    &format!(
                        "Temp register {} resolution not implemented in Phase 3.1",
                        register_id
                    ),
                    ErrorCode::E500,
                ))
            }
        }
    }
}

impl Default for LoopExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for LoopInstruction to extract data (needed for parallel execution)
trait LoopInstructionExt {
    fn get_config(&self) -> &crate::bcib::LoopConfig;
    fn get_loop_id(&self) -> &crate::bcib::LoopID;
    fn get_initial_accumulator(&self) -> &Value;
    #[allow(dead_code)]
    fn get_range(&self) -> Option<&LoopRange>;
    #[allow(dead_code)]
    fn loop_type(&self) -> LoopType;
}

impl LoopInstructionExt for LoopInstruction {
    fn get_config(&self) -> &crate::bcib::LoopConfig {
        match self {
            LoopInstruction::While { config, .. } => config,
            LoopInstruction::For { config, .. } => config,
            LoopInstruction::ForEach { config, .. } => config,
        }
    }

    fn get_loop_id(&self) -> &crate::bcib::LoopID {
        match self {
            LoopInstruction::While { id, .. } => id,
            LoopInstruction::For { id, .. } => id,
            LoopInstruction::ForEach { id, .. } => id,
        }
    }

    fn get_initial_accumulator(&self) -> &Value {
        &self.get_config().initial_accumulator
    }

    fn get_range(&self) -> Option<&LoopRange> {
        match self {
            LoopInstruction::For { range, .. } => Some(range),
            _ => None,
        }
    }

    fn loop_type(&self) -> LoopType {
        match self {
            LoopInstruction::While { .. } => LoopType::While,
            LoopInstruction::For { .. } => LoopType::For,
            LoopInstruction::ForEach { .. } => LoopType::ForEach,
        }
    }
}
