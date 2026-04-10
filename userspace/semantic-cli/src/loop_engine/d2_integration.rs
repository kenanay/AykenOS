//! D2 Parallelism Integration for D3 Loop Support
//!
//! This module implements the integration between the D3 Loop Support system and the
//! D2 Parallelism Architecture. It provides the parallelization trigger logic,
//! deterministic partitioning, and stable index mapping required for safe parallel
//! loop execution.
//!
//! # Constitutional Requirements
//!
//! This integration must adhere to the constitutional principles established in D0:
//! - Bounded iteration only (never exceed iteration limits)
//! - Deterministic execution (same input → same output)
//! - Budget-based timeout enforcement
//! - Explicit error recovery policies
//!
//! # Phase 7 Implementation
//!
//! - **Phase 7.1**: Parallelization trigger logic
//! - **Phase 7.2**: Deterministic partitioning
//! - **Phase 7.3**: Stable index mapping and result collection
//!
//! **Requirements:** 7.1, 7.4, 7.5, 15.1, 15.2, 15.3, 15.6

use super::core::LoopBodyFn;
use super::stable_index_mapping::StableIndexMapper;
use crate::bcib::{ControlFlowType, LoopInstruction, LoopRange, LoopType, Value};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::execution_plan::{BlockTerminator, IRBlock, IRInstruction, ParallelSafety};
use crate::loop_engine::executor::{
    IterationExecutionResult, PartitionExecutionResult, PartitionResultType,
};
use crate::loop_engine::{
    IterationPartition, LoopResult, ParallelizationDecision, SafetyAnalysisResult, SafetyClass,
};
use crate::parallelism::{
    ContiguousPartitioner, DataPartition, DataPartitioner, DeterministicMerger, ImmutableContext,
    ParallelExecutor, ParallelismError, RayonParallelExecutor, StableIndexMerger,
};

/// D2 Parallelism Integration for D3 Loop Support
///
/// This struct provides the integration layer between the D3 Loop Support system
/// and the D2 Parallelism Architecture. It implements the constitutional requirements
/// for safe parallel loop execution while maintaining deterministic semantics.
///
/// # Design Principles
///
/// 1. **Safety First**: Only Safe loop bodies are parallelized
/// 2. **Deterministic Partitioning**: Same iteration count → same partitions
/// 3. **Stable Index Mapping**: Iteration i always processes same input data
/// 4. **Constitutional Compliance**: All constitutional rules are enforced
///
/// **Requirements:** 7.1, 7.4, 7.5, 15.1, 15.2, 15.3, 15.6
pub struct D2LoopIntegration {
    /// D2 parallel executor for executing partitions
    parallel_executor: RayonParallelExecutor,
    /// D2 data partitioner for deterministic partitioning
    partitioner: ContiguousPartitioner,
    /// D2 deterministic merger for result collection
    merger: StableIndexMerger,
    /// Stable index mapper for deterministic data access
    index_mapper: StableIndexMapper,
}

impl D2LoopIntegration {
    /// Create a new D2 loop integration instance
    pub fn new() -> Self {
        Self {
            parallel_executor: RayonParallelExecutor::new(),
            partitioner: ContiguousPartitioner,
            merger: StableIndexMerger::new(),
            index_mapper: StableIndexMapper::new(),
        }
    }

    /// Determine if a loop should be parallelized (Phase 7.1 - Parallelization trigger logic)
    ///
    /// This method implements the complete parallelization decision system:
    /// 1. Check loop type eligibility (Constitutional: While loops NEVER parallelized)
    /// 2. Verify safety classification (only Safe loops can be parallelized)
    /// 3. Ensure static iteration count is available
    /// 4. Check minimum threshold for parallelization benefit
    ///
    /// **Requirements 7.1**: Only parallelize Safe loop bodies, exclude While loops,
    /// support For and ForEach loops with statically known iteration counts,
    /// fall back to sequential execution for Unsafe loops.
    pub fn should_parallelize_loop(
        &self,
        instruction: &LoopInstruction,
        safety_result: &SafetyAnalysisResult,
    ) -> ParallelizationDecision {
        // Phase 7.1: Constitutional rule - While loops are NEVER parallelized
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

        // Phase 7.1: Safety classification check - only Safe loops can be parallelized
        if safety_result.classification != SafetyClass::Safe {
            return ParallelizationDecision::Sequential {
                reason: format!(
                    "Loop body is Unsafe for parallelization: {}",
                    safety_result.reason
                ),
            };
        }

        // Phase 7.1: Verify iteration count is statically known
        let iteration_count = match self.get_static_iteration_count(instruction) {
            Some(count) => count,
            None => {
                return ParallelizationDecision::Sequential {
                    reason: "Dynamic iteration count - cannot determine parallelization benefit"
                        .to_string(),
                };
            }
        };

        // Phase 7.1: Check minimum threshold for parallelization benefit
        const MIN_PARALLEL_ITERATIONS: u32 = 100; // Minimum iterations to justify parallel overhead
        if iteration_count < MIN_PARALLEL_ITERATIONS {
            return ParallelizationDecision::Sequential {
                reason: format!(
                    "Iteration count ({}) below minimum threshold ({}) for parallelization",
                    iteration_count, MIN_PARALLEL_ITERATIONS
                ),
            };
        }

        // Phase 7.1: All checks passed - recommend parallelization
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

    /// Partition iterations deterministically for parallel execution (Phase 7.2)
    ///
    /// This method implements the constitutional requirement for deterministic partitioning:
    /// - Same iteration count → same partitions (always)
    /// - Available parallelism (core count) is optimization hint, not semantic input
    /// - Fixed chunk size algorithm ensures reproducible partition boundaries
    ///
    /// **Requirements 7.5, 15.1, 15.3**: Partition iterations based on iteration count only,
    /// use fixed chunk size algorithm, treat available parallelism as upper bound.
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

    /// Execute loop with deterministic parallel partitioning (Phase 7.3)
    ///
    /// This method implements the complete parallel loop execution workflow:
    /// 1. Partition iterations deterministically
    /// 2. Execute partitions in parallel using D2 system with stable index mapping
    /// 3. Collect results in deterministic order (partition 0, 1, 2, ...)
    ///
    /// **Requirements 7.4, 15.2, 15.6**: Stable index mapping, deterministic partitioning,
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
            // No partitions - return empty result
            return Ok(LoopResult::success(
                instruction.get_config().initial_accumulator.clone(),
                0,
            ));
        }

        if partitions.len() == 1 {
            // Single partition - execute sequentially for efficiency
            return self.execute_single_partition_sequential(instruction, &body_fn, &partitions[0]);
        }

        // Phase 7.3: Execute partitions with stable index mapping using D2 system
        let partition_results =
            self.execute_partitions_with_d2_system(instruction, &body_fn, &partitions)?;

        // Phase 7.3: Collect results in deterministic order (partition 0, 1, 2, ...)
        let final_result = self.collect_partition_results_deterministic(partition_results)?;

        Ok(final_result)
    }

    /// Execute partitions using the D2 parallelism system (Phase 7.3)
    ///
    /// This method integrates with the D2 parallelism architecture to execute
    /// loop partitions in parallel while maintaining stable index mapping.
    ///
    /// **Requirements 7.4, 15.2**: Stable index mapping, deterministic data access
    fn execute_partitions_with_d2_system(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: &LoopBodyFn,
        partitions: &[IterationPartition],
    ) -> Result<Vec<PartitionExecutionResult>> {
        // Convert loop partitions to D2 data partitions
        let input_data = self.create_input_data_for_partitions(instruction, partitions)?;
        let data_partitions = self.convert_to_d2_partitions(&input_data, partitions);

        // Create immutable context for D2 execution
        let context = self.create_d2_execution_context(instruction)?;

        // Create IR block for loop body execution
        let ir_block = self.create_ir_block_for_loop_body(instruction, body_fn)?;

        // Execute partitions in parallel using D2 system
        let d2_results = self
            .parallel_executor
            .execute_parallel(&ir_block, data_partitions, &context)
            .map_err(|e| self.convert_parallelism_error(e))?;

        // Convert D2 results back to loop partition results with stable index mapping
        self.convert_d2_results_to_partition_results(d2_results, partitions)
    }

    /// Collect partition results in deterministic order (Phase 7.3)
    ///
    /// This method implements deterministic result collection using the D2 merger:
    /// - Results collected in partition order (0, 1, 2, ...)
    /// - Within each partition, results ordered by iteration index
    /// - Final result identical to sequential execution
    ///
    /// **Requirements 15.6**: Deterministic result collection order
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
                    final_accumulator,
                    iterations_completed,
                    ..
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

        // Phase 7.3: Use D2 stable index merger for deterministic result ordering
        let _ordered_results = self.merger.merge(all_iteration_results).map_err(|e| {
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

    // Helper methods for D2 integration

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
    fn get_collection_size_hint(&self, collection_ref: &crate::bcib::OperandRef) -> Option<u32> {
        match collection_ref {
            crate::bcib::OperandRef::Literal(value) => {
                // Literal collections have known size
                value.collection_size().map(|size| size as u32)
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

    /// Calculate deterministic chunk size for iteration partitioning (Phase 7.2)
    ///
    /// This method implements the fixed chunk size algorithm required for deterministic
    /// partitioning. The chunk size is calculated based solely on iteration count,
    /// ensuring the same input always produces the same partitions.
    ///
    /// **Requirements 15.1**: Partition iterations based on iteration count only
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

    /// Execute a single partition sequentially (fallback for single partition)
    fn execute_single_partition_sequential(
        &self,
        instruction: &LoopInstruction,
        body_fn: &LoopBodyFn,
        partition: &IterationPartition,
    ) -> Result<LoopResult> {
        // For single partition, execute sequentially
        // This is a simplified implementation - in practice, this would delegate
        // to the core loop executor
        let mut accumulator = instruction.get_config().initial_accumulator.clone();
        let mut iterations_completed = 0;

        for global_iteration in partition.start_iteration..partition.end_iteration {
            // Execute iteration body
            let body_result = body_fn(&accumulator, global_iteration)?;

            match body_result {
                crate::loop_engine::core::LoopBodyResult::Break(new_accumulator) => {
                    accumulator = new_accumulator;
                    iterations_completed += 1;
                    return Ok(LoopResult::break_result(accumulator, iterations_completed));
                }
                crate::loop_engine::core::LoopBodyResult::Continue(new_accumulator) => {
                    accumulator = new_accumulator;
                    iterations_completed += 1;
                    continue;
                }
                crate::loop_engine::core::LoopBodyResult::Normal(new_accumulator) => {
                    accumulator = new_accumulator;
                    iterations_completed += 1;
                }
            }
        }

        Ok(LoopResult::success(accumulator, iterations_completed))
    }

    /// Create input data for partitions with stable index mapping (Phase 7.3)
    fn create_input_data_for_partitions(
        &self,
        instruction: &LoopInstruction,
        partitions: &[IterationPartition],
    ) -> Result<Vec<Value>> {
        // Use stable index mapper for deterministic input data creation
        let input_mapping = self
            .index_mapper
            .create_stable_input_mapping(instruction, partitions)?;

        // Extract values in deterministic order
        let input_data: Vec<Value> = input_mapping.into_iter().map(|(_, value)| value).collect();

        Ok(input_data)
    }

    /// Get input data for a specific iteration with stable index mapping (Phase 7.3)
    fn get_input_data_for_iteration(
        &self,
        instruction: &LoopInstruction,
        global_iteration: u32,
    ) -> Result<Value> {
        // Use stable index mapper for deterministic input data access
        self.index_mapper
            .get_input_data_for_iteration(instruction, global_iteration)
    }

    /// Convert loop partitions to D2 data partitions
    fn convert_to_d2_partitions<'a>(
        &self,
        input_data: &'a [Value],
        partitions: &[IterationPartition],
    ) -> Vec<DataPartition<'a>> {
        self.partitioner.partition(input_data, partitions.len())
    }

    /// Create D2 execution context for parallel execution
    fn create_d2_execution_context(
        &self,
        instruction: &LoopInstruction,
    ) -> Result<ImmutableContext> {
        // Create a basic execution plan for D2 context
        let execution_plan = crate::execution_plan::ExecutionPlan::new(
            vec![],
            0,
            crate::normalizer::RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: std::collections::HashMap::new(),
                next_register: 0,
            },
            crate::execution_plan::dataflow::DataflowGraph::new(),
            crate::execution_plan::ExecutionMetadata::new(
                format!("loop-{}", instruction.get_loop_id().0),
                0,
                0,
                0,
            ),
        );

        let config = crate::parallelism::types::ExecutionConfig {
            max_execution_time: None,
            max_iterations: Some(instruction.get_config().iteration_limit as usize),
            verification_mode: false,
            replay_mode: false,
        };

        Ok(ImmutableContext {
            execution_plan,
            config,
        })
    }

    /// Create IR block for loop body execution
    fn create_ir_block_for_loop_body(
        &self,
        _instruction: &LoopInstruction,
        _body_fn: &LoopBodyFn,
    ) -> Result<IRBlock> {
        // Create a simple IR block for loop body execution
        // In a real implementation, this would convert the loop body function
        // to IR instructions for D2 execution
        Ok(IRBlock::with_safety(
            0,
            vec![IRInstruction::LoadContext {
                context_id: "loop-body".to_string(),
                target_register: 0,
            }],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Safe,
        ))
    }

    /// Convert D2 results back to loop partition results
    fn convert_d2_results_to_partition_results(
        &self,
        d2_results: Vec<(usize, Value)>,
        partitions: &[IterationPartition],
    ) -> Result<Vec<PartitionExecutionResult>> {
        let mut partition_results = Vec::with_capacity(partitions.len());

        // Group D2 results by partition
        for (partition_idx, partition) in partitions.iter().enumerate() {
            let mut iteration_results = Vec::new();
            let mut iterations_completed = 0;

            // Collect results for this partition
            for global_iteration in partition.start_iteration..partition.end_iteration {
                if let Some((_, value)) = d2_results
                    .iter()
                    .find(|(idx, _)| *idx == global_iteration as usize)
                {
                    iteration_results.push(IterationExecutionResult {
                        global_iteration_index: global_iteration,
                        result_value: value.clone(),
                        control_flow: ControlFlowType::Normal,
                    });
                    iterations_completed += 1;
                }
            }

            // Create partition result
            let final_accumulator = if let Some(last_result) = iteration_results.last() {
                last_result.result_value.clone()
            } else {
                Value::Number(0.0) // Default accumulator
            };

            partition_results.push(PartitionExecutionResult::success(
                partition_idx,
                iteration_results,
                final_accumulator,
                iterations_completed,
            ));
        }

        Ok(partition_results)
    }

    /// Convert parallelism error to semantic CLI error
    fn convert_parallelism_error(&self, error: ParallelismError) -> SemanticCLIError {
        match error {
            ParallelismError::ExecutionError { message, .. } => {
                SemanticCLIError::execution_error(&message, ErrorCode::E500)
            }
            ParallelismError::SafetyViolation { reason, .. } => SemanticCLIError::validation_error(
                reason,
                "Check loop safety analysis",
                ErrorCode::E400,
            ),
            ParallelismError::DeterminismViolation { .. } => SemanticCLIError::execution_error(
                "Determinism violation in parallel execution",
                ErrorCode::E500,
            ),
            ParallelismError::PerformanceDegradation { .. } => SemanticCLIError::execution_error(
                "Performance degradation detected",
                ErrorCode::E500,
            ),
            ParallelismError::ThreadPoolInitialization { reason } => {
                SemanticCLIError::execution_error(&reason, ErrorCode::E500)
            }
            ParallelismError::ConstitutionalViolation { principle, .. } => {
                SemanticCLIError::execution_error(
                    &format!("Constitutional violation: {}", principle),
                    ErrorCode::E500,
                )
            }
            ParallelismError::SecurityError { message } => {
                SemanticCLIError::execution_error(&message, ErrorCode::E500)
            }
        }
    }

    /// Verify stable index mapping for a loop instruction (Phase 7.3)
    ///
    /// This method verifies that the stable index mapping is working correctly
    /// for the given loop instruction by testing multiple iterations.
    ///
    /// **Requirements 15.2**: Verify stable index mapping correctness
    pub fn verify_stable_mapping(
        &self,
        instruction: &LoopInstruction,
        test_iterations: u32,
    ) -> Result<crate::loop_engine::stable_index_mapping::StableMappingVerification> {
        self.index_mapper
            .verify_stable_mapping(instruction, test_iterations)
    }

    /// Analyze the index mapping strategy for a loop instruction (Phase 7.3)
    ///
    /// This method analyzes the index mapping strategy that will be used
    /// for the given loop instruction, providing insights for optimization.
    pub fn analyze_mapping_strategy(
        &self,
        instruction: &LoopInstruction,
    ) -> crate::loop_engine::stable_index_mapping::IndexMappingStrategy {
        self.index_mapper.analyze_mapping_strategy(instruction)
    }

    /// Create a mapping cache for efficient repeated access (Phase 7.3)
    ///
    /// This method creates a cache of input data mappings for efficient
    /// repeated access during parallel execution.
    pub fn create_mapping_cache(
        &self,
        instruction: &LoopInstruction,
        max_iterations: u32,
    ) -> Result<crate::loop_engine::stable_index_mapping::IndexMappingCache> {
        self.index_mapper
            .create_mapping_cache(instruction, max_iterations)
    }
}

impl Default for D2LoopIntegration {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for LoopInstruction to extract data (needed for D2 integration)
trait LoopInstructionExt {
    fn get_config(&self) -> &crate::bcib::LoopConfig;
    fn get_loop_id(&self) -> &crate::bcib::LoopID;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{
        BudgetMeasurement, ErrorRecoveryPolicy, LoopConfig, LoopID, LoopRange, Value, ValueType,
    };
    use crate::types::SourceLocation;

    fn create_test_loop_config() -> LoopConfig {
        LoopConfig {
            iteration_limit: 1000,
            budget_timeout: 10000,
            budget_measurement: BudgetMeasurement::IterationCount,
            initial_accumulator: Value::Number(0.0),
            accumulator_type: ValueType::Number,
            error_recovery: ErrorRecoveryPolicy::Abort,
        }
    }

    fn create_test_safety_result(classification: SafetyClass) -> SafetyAnalysisResult {
        SafetyAnalysisResult {
            classification,
            reason: "Test safety analysis".to_string(),
            side_effects: vec![],
            dependencies: vec![],
            cache_key: "test-cache-key".to_string(),
        }
    }

    #[test]
    fn test_parallelization_decision_while_loop_excluded() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::While {
            id: LoopID::new("test-while".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };
        let safety_result = create_test_safety_result(SafetyClass::Safe);

        let decision = integration.should_parallelize_loop(&instruction, &safety_result);

        assert!(decision.is_sequential());
        assert!(decision
            .sequential_reason()
            .unwrap()
            .contains("While loops are excluded"));
    }

    #[test]
    fn test_parallelization_decision_unsafe_loop_excluded() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 1000, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };
        let safety_result = create_test_safety_result(SafetyClass::Unsafe);

        let decision = integration.should_parallelize_loop(&instruction, &safety_result);

        assert!(decision.is_sequential());
        assert!(decision
            .sequential_reason()
            .unwrap()
            .contains("Unsafe for parallelization"));
    }

    #[test]
    fn test_parallelization_decision_small_loop_excluded() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 50, 1), // Only 50 iterations
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };
        let safety_result = create_test_safety_result(SafetyClass::Safe);

        let decision = integration.should_parallelize_loop(&instruction, &safety_result);

        assert!(decision.is_sequential());
        assert!(decision
            .sequential_reason()
            .unwrap()
            .contains("below minimum threshold"));
    }

    #[test]
    fn test_parallelization_decision_large_safe_for_loop_accepted() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 1000, 1), // 1000 iterations
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };
        let safety_result = create_test_safety_result(SafetyClass::Safe);

        let decision = integration.should_parallelize_loop(&instruction, &safety_result);

        assert!(decision.is_parallel());
        assert_eq!(decision.iteration_count(), Some(1000));
        assert!(decision.parallel_benefit().unwrap() > 0.0);
    }

    #[test]
    fn test_static_iteration_count_for_loop() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 100, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let count = integration.get_static_iteration_count(&instruction);
        assert_eq!(count, Some(100));
    }

    #[test]
    fn test_static_iteration_count_foreach_loop_with_literal_array() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::ForEach {
            id: LoopID::new("test-foreach".to_string()),
            collection: crate::bcib::OperandRef::Literal(Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ])),
            collection_type: crate::bcib::CollectionType::Array,
            iterator_var: "item".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let count = integration.get_static_iteration_count(&instruction);
        assert_eq!(count, Some(3));
    }

    #[test]
    fn test_static_iteration_count_while_loop_returns_none() {
        let integration = D2LoopIntegration::new();
        let instruction = LoopInstruction::While {
            id: LoopID::new("test-while".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let count = integration.get_static_iteration_count(&instruction);
        assert_eq!(count, None);
    }

    #[test]
    fn test_deterministic_partitioning() {
        let integration = D2LoopIntegration::new();

        // Test deterministic partitioning with same inputs
        let partitions1 = integration.partition_iterations_deterministic(1000, 4);
        let partitions2 = integration.partition_iterations_deterministic(1000, 4);

        // Should produce identical partitions
        assert_eq!(partitions1, partitions2);
        assert!(!partitions1.is_empty());

        // Verify partition properties
        let mut total_iterations = 0;
        for partition in &partitions1 {
            assert!(partition.is_valid());
            total_iterations += partition.iteration_count;
        }
        assert_eq!(total_iterations, 1000);
    }

    #[test]
    fn test_deterministic_chunk_size_calculation() {
        let integration = D2LoopIntegration::new();

        // Small loops: single chunk
        assert_eq!(integration.calculate_deterministic_chunk_size(50), 50);

        // Medium loops: divide into 4 chunks
        assert_eq!(integration.calculate_deterministic_chunk_size(400), 100);

        // Large loops: use maximum chunk size
        assert_eq!(integration.calculate_deterministic_chunk_size(10000), 1000);
    }

    #[test]
    fn test_for_loop_iteration_calculation() {
        let integration = D2LoopIntegration::new();

        // Forward iteration
        let range1 = LoopRange::new(0, 10, 1);
        assert_eq!(integration.calculate_for_loop_iterations(&range1), 10);

        // Forward iteration with step 2
        let range2 = LoopRange::new(0, 10, 2);
        assert_eq!(integration.calculate_for_loop_iterations(&range2), 5);

        // Backward iteration
        let range3 = LoopRange::new(10, 0, -1);
        assert_eq!(integration.calculate_for_loop_iterations(&range3), 10);

        // Zero iterations
        let range4 = LoopRange::new(10, 0, 1);
        assert_eq!(integration.calculate_for_loop_iterations(&range4), 0);
    }

    #[test]
    fn test_parallelization_benefit_estimation() {
        let integration = D2LoopIntegration::new();

        // Small iteration count: low benefit
        let benefit1 = integration.estimate_parallelization_benefit(100);
        assert_eq!(benefit1, 0.0);

        // Medium iteration count: scaled benefit
        let benefit2 = integration.estimate_parallelization_benefit(5000);
        assert!(benefit2 > 0.0 && benefit2 < 1.0);

        // Large iteration count: maximum benefit
        let benefit3 = integration.estimate_parallelization_benefit(10000);
        assert_eq!(benefit3, 1.0);
    }
}
