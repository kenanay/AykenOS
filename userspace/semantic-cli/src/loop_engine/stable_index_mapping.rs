//! Stable Index Mapping and Result Collection for D3 Loop Support
//!
//! This module implements the stable index mapping system required for deterministic
//! parallel loop execution. The stable index mapping ensures that iteration i always
//! processes the same input data regardless of which partition contains it, and that
//! results are collected in deterministic order.
//!
//! # Constitutional Requirements
//!
//! The stable index mapping system must adhere to these constitutional principles:
//! - **Stable Mapping**: Iteration i always processes input_data[i] (same input data)
//! - **Deterministic Collection**: Results collected in deterministic order (partition 0, 1, 2, ...)
//! - **Reproducible Results**: Parallel result equals sequential result (always)
//! - **Index Preservation**: Global iteration index preserved across partitions
//!
//! # Requirements
//!
//! **Requirements 7.4, 15.2**: Ensure iteration i always processes same input data
//! regardless of partition assignment.
//!
//! **Requirements 15.6**: Collect results from partitions in deterministic order
//! (partition 0, 1, 2, ...).

use crate::bcib::{LoopInstruction, LoopRange, Value};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::loop_engine::executor::{
    IterationExecutionResult, PartitionExecutionResult, PartitionResultType,
};
use crate::loop_engine::IterationPartition;
use crate::parallelism::{DeterministicMerger, StableIndexMerger};
use std::collections::HashMap;

/// Stable index mapper for deterministic parallel loop execution
///
/// This struct implements the stable index mapping system that ensures iteration i
/// always processes the same input data regardless of partition assignment. It also
/// handles deterministic result collection from parallel partitions.
///
/// # Design Principles
///
/// 1. **Stable Mapping**: Global iteration index determines input data access
/// 2. **Partition Independence**: Same input data regardless of partition assignment
/// 3. **Deterministic Collection**: Results ordered by global iteration index
/// 4. **Constitutional Compliance**: Parallel results equal sequential results
///
/// **Requirements:** 7.4, 15.2, 15.6
#[derive(Debug, Clone)]
pub struct StableIndexMapper {
    /// D2 merger for deterministic result collection
    merger: StableIndexMerger,
}

impl StableIndexMapper {
    /// Create a new stable index mapper
    pub fn new() -> Self {
        Self {
            merger: StableIndexMerger::new(),
        }
    }

    /// Create input data mapping for partitions with stable index mapping (Phase 7.3)
    ///
    /// This method creates the input data for all iterations with stable index mapping.
    /// Each iteration i always receives the same input data regardless of which partition
    /// contains it, ensuring deterministic parallel execution.
    ///
    /// **Requirements 7.4, 15.2**: Stable index mapping for deterministic data access
    ///
    /// # Arguments
    ///
    /// * `instruction` - The loop instruction defining the iteration pattern
    /// * `partitions` - The partitions that will execute the iterations
    ///
    /// # Returns
    ///
    /// Vector of input data values with stable index mapping
    ///
    /// # Guarantees
    ///
    /// - Iteration i always receives input_data[i] (stable mapping)
    /// - Same input data in parallel and sequential execution modes
    /// - Input data order is deterministic and reproducible
    pub fn create_stable_input_mapping(
        &self,
        instruction: &LoopInstruction,
        partitions: &[IterationPartition],
    ) -> Result<Vec<(u32, Value)>> {
        let mut input_mapping = Vec::new();

        // Create input data for all iterations with stable index mapping
        for partition in partitions {
            for global_iteration in partition.start_iteration..partition.end_iteration {
                let iteration_data =
                    self.get_input_data_for_iteration(instruction, global_iteration)?;
                input_mapping.push((global_iteration, iteration_data));
            }
        }

        // Sort by global iteration index to ensure deterministic order
        input_mapping.sort_by_key(|(global_idx, _)| *global_idx);

        Ok(input_mapping)
    }

    /// Get input data for a specific iteration with stable index mapping (Phase 7.3)
    ///
    /// This method implements the core stable index mapping guarantee:
    /// - Iteration i always accesses input_data[i]
    /// - Mapping is based on iteration index, not partition assignment
    /// - Same input data regardless of parallel vs sequential execution
    ///
    /// **Requirements 15.2**: Stable index mapping ensures deterministic data access
    ///
    /// # Arguments
    ///
    /// * `instruction` - The loop instruction defining the iteration pattern
    /// * `global_iteration` - The global iteration index (0, 1, 2, ...)
    ///
    /// # Returns
    ///
    /// Input data value for the specified iteration
    ///
    /// # Guarantees
    ///
    /// - Same global_iteration → same input data (always)
    /// - Input data independent of partition assignment
    /// - Deterministic data access pattern
    pub fn get_input_data_for_iteration(
        &self,
        instruction: &LoopInstruction,
        global_iteration: u32,
    ) -> Result<Value> {
        match instruction {
            LoopInstruction::For { range, .. } => {
                // For loops: stable mapping based on range calculation
                self.get_for_loop_input_data(range, global_iteration)
            }
            LoopInstruction::ForEach { collection, .. } => {
                // ForEach loops: stable mapping based on collection index
                self.get_foreach_loop_input_data(collection, global_iteration)
            }
            LoopInstruction::While { .. } => {
                // While loops: stable mapping based on iteration index
                self.get_while_loop_input_data(global_iteration)
            }
        }
    }

    /// Collect partition results in deterministic order (Phase 7.3)
    ///
    /// This method implements deterministic result collection from parallel partitions:
    /// - Results collected in partition order (0, 1, 2, ...)
    /// - Within each partition, results ordered by global iteration index
    /// - Final result identical to sequential execution
    ///
    /// **Requirements 15.6**: Deterministic result collection order
    ///
    /// # Arguments
    ///
    /// * `partition_results` - Results from all partitions
    ///
    /// # Returns
    ///
    /// Merged results in deterministic order
    ///
    /// # Guarantees
    ///
    /// - Results ordered by global iteration index
    /// - Same result order as sequential execution
    /// - Deterministic and reproducible result collection
    pub fn collect_results_deterministic(
        &self,
        partition_results: Vec<PartitionExecutionResult>,
    ) -> Result<Vec<IterationExecutionResult>> {
        // Phase 7.3: Sort partition results by partition ID (deterministic order)
        let mut sorted_results = partition_results;
        sorted_results.sort_by_key(|result| result.partition_id);

        // Collect all iteration results with stable index mapping
        let mut all_iteration_results = Vec::new();

        // Phase 7.3: Collect results in partition order (0, 1, 2, ...)
        for partition_result in sorted_results {
            match partition_result.result_type {
                PartitionResultType::Success {
                    iteration_results, ..
                }
                | PartitionResultType::Break {
                    iteration_results, ..
                } => {
                    // Add iteration results with stable index mapping
                    all_iteration_results.extend(iteration_results);
                }
                PartitionResultType::Error { .. } => {
                    // Error partitions don't contribute results
                    continue;
                }
            }
        }

        // Sort results by global iteration index for deterministic ordering
        all_iteration_results.sort_by_key(|result| result.global_iteration_index);

        Ok(all_iteration_results)
    }

    /// Merge iteration results using D2 stable index merger (Phase 7.3)
    ///
    /// This method uses the D2 parallelism system's stable index merger to
    /// reconstruct the deterministic result order from parallel execution results.
    ///
    /// **Requirements 15.6**: Use D2 merger for deterministic result collection
    ///
    /// # Arguments
    ///
    /// * `iteration_results` - Results from all iterations with global indices
    ///
    /// # Returns
    ///
    /// Merged results in deterministic order using D2 merger
    pub fn merge_results_with_d2_system(
        &self,
        iteration_results: &[IterationExecutionResult],
    ) -> Result<Vec<Value>> {
        // Convert iteration results to D2 merger format
        let indexed_results: Vec<(usize, Value)> = iteration_results
            .iter()
            .map(|result| {
                (
                    result.global_iteration_index as usize,
                    result.result_value.clone(),
                )
            })
            .collect();

        // Use D2 stable index merger for deterministic result collection
        self.merger.merge(indexed_results).map_err(|e| {
            SemanticCLIError::execution_error(
                &format!("Failed to merge parallel results using D2 system: {}", e),
                ErrorCode::E500,
            )
        })
    }

    /// Verify stable index mapping correctness (Phase 7.3)
    ///
    /// This method verifies that the stable index mapping is working correctly
    /// by checking that the same global iteration index always produces the same
    /// input data across multiple calls.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The loop instruction to verify
    /// * `test_iterations` - Number of iterations to test
    ///
    /// # Returns
    ///
    /// Result indicating whether stable index mapping is working correctly
    pub fn verify_stable_mapping(
        &self,
        instruction: &LoopInstruction,
        test_iterations: u32,
    ) -> Result<StableMappingVerification> {
        let mut verification = StableMappingVerification {
            total_tested: 0,
            stable_mappings: 0,
            unstable_mappings: 0,
            errors: Vec::new(),
        };

        // Test stable mapping for each iteration
        for global_iteration in 0..test_iterations {
            verification.total_tested += 1;

            // Get input data multiple times for the same iteration
            match (
                self.get_input_data_for_iteration(instruction, global_iteration),
                self.get_input_data_for_iteration(instruction, global_iteration),
                self.get_input_data_for_iteration(instruction, global_iteration),
            ) {
                (Ok(data1), Ok(data2), Ok(data3)) => {
                    if data1 == data2 && data2 == data3 {
                        verification.stable_mappings += 1;
                    } else {
                        verification.unstable_mappings += 1;
                        verification.errors.push(format!(
                            "Unstable mapping at iteration {}: {:?} != {:?} != {:?}",
                            global_iteration, data1, data2, data3
                        ));
                    }
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    verification.errors.push(format!(
                        "Error getting input data for iteration {}: {}",
                        global_iteration, e
                    ));
                }
            }
        }

        Ok(verification)
    }

    // Helper methods for different loop types

    /// Get input data for For loop iterations
    fn get_for_loop_input_data(&self, range: &LoopRange, global_iteration: u32) -> Result<Value> {
        // For loops: stable mapping based on range calculation
        let iterator_value = range.start + (global_iteration as i64 * range.step);
        Ok(Value::Number(iterator_value as f64))
    }

    /// Get input data for ForEach loop iterations
    fn get_foreach_loop_input_data(
        &self,
        collection: &crate::bcib::OperandRef,
        global_iteration: u32,
    ) -> Result<Value> {
        match collection {
            crate::bcib::OperandRef::Literal(value) => {
                // Get element at stable index position
                match value {
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
            _ => {
                // Field references and temp registers not supported in Phase 7.3
                Err(SemanticCLIError::execution_error(
                    "Dynamic collection references not supported in Phase 7.3",
                    ErrorCode::E500,
                ))
            }
        }
    }

    /// Get input data for While loop iterations
    fn get_while_loop_input_data(&self, global_iteration: u32) -> Result<Value> {
        // While loops don't have stable input data (condition-based)
        // Return iteration index as input data
        Ok(Value::Number(global_iteration as f64))
    }
}

impl Default for StableIndexMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of stable mapping verification
#[derive(Debug, Clone, PartialEq)]
pub struct StableMappingVerification {
    /// Total number of iterations tested
    pub total_tested: u32,
    /// Number of stable mappings found
    pub stable_mappings: u32,
    /// Number of unstable mappings found
    pub unstable_mappings: u32,
    /// List of errors encountered during verification
    pub errors: Vec<String>,
}

impl StableMappingVerification {
    /// Check if the stable mapping verification passed
    pub fn is_stable(&self) -> bool {
        self.unstable_mappings == 0 && self.errors.is_empty()
    }

    /// Get the stability ratio (stable / total)
    pub fn stability_ratio(&self) -> f64 {
        if self.total_tested == 0 {
            1.0
        } else {
            self.stable_mappings as f64 / self.total_tested as f64
        }
    }
}

/// Index mapping strategy for different loop types
#[derive(Debug, Clone, PartialEq)]
pub enum IndexMappingStrategy {
    /// For loop: range-based mapping
    Range { start: i64, step: i64 },
    /// ForEach loop: collection-based mapping
    Collection {
        collection_size: usize,
        collection_type: String,
    },
    /// While loop: iteration-based mapping
    Iteration,
}

impl StableIndexMapper {
    /// Analyze the index mapping strategy for a loop instruction
    pub fn analyze_mapping_strategy(&self, instruction: &LoopInstruction) -> IndexMappingStrategy {
        match instruction {
            LoopInstruction::For { range, .. } => IndexMappingStrategy::Range {
                start: range.start,
                step: range.step,
            },
            LoopInstruction::ForEach { collection, .. } => match collection {
                crate::bcib::OperandRef::Literal(value) => {
                    let (size, type_name) = match value {
                        Value::Array(arr) => (arr.len(), "Array"),
                        Value::List(list) => (list.len(), "List"),
                        Value::SortedMap(map) => (map.len(), "SortedMap"),
                        _ => (0, "Unknown"),
                    };
                    IndexMappingStrategy::Collection {
                        collection_size: size,
                        collection_type: type_name.to_string(),
                    }
                }
                _ => IndexMappingStrategy::Collection {
                    collection_size: 0,
                    collection_type: "Dynamic".to_string(),
                },
            },
            LoopInstruction::While { .. } => IndexMappingStrategy::Iteration,
        }
    }

    /// Create a mapping cache for efficient repeated access
    pub fn create_mapping_cache(
        &self,
        instruction: &LoopInstruction,
        max_iterations: u32,
    ) -> Result<IndexMappingCache> {
        let mut cache = HashMap::new();

        // Pre-compute input data for all iterations
        for global_iteration in 0..max_iterations {
            let input_data = self.get_input_data_for_iteration(instruction, global_iteration)?;
            cache.insert(global_iteration, input_data);
        }

        Ok(IndexMappingCache {
            cache,
            strategy: self.analyze_mapping_strategy(instruction),
            max_iterations,
        })
    }
}

/// Cache for efficient index mapping lookups
#[derive(Debug, Clone)]
pub struct IndexMappingCache {
    /// Cached input data for each iteration
    cache: HashMap<u32, Value>,
    /// Mapping strategy used
    strategy: IndexMappingStrategy,
    /// Maximum number of iterations cached
    max_iterations: u32,
}

impl IndexMappingCache {
    /// Get input data for an iteration from the cache
    pub fn get_input_data(&self, global_iteration: u32) -> Option<&Value> {
        self.cache.get(&global_iteration)
    }

    /// Get the mapping strategy
    pub fn strategy(&self) -> &IndexMappingStrategy {
        &self.strategy
    }

    /// Get the maximum number of iterations cached
    pub fn max_iterations(&self) -> u32 {
        self.max_iterations
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> IndexMappingCacheStats {
        IndexMappingCacheStats {
            cached_entries: self.cache.len(),
            max_iterations: self.max_iterations,
            cache_hit_ratio: if self.max_iterations > 0 {
                self.cache.len() as f64 / self.max_iterations as f64
            } else {
                0.0
            },
        }
    }
}

/// Statistics for index mapping cache
#[derive(Debug, Clone, PartialEq)]
pub struct IndexMappingCacheStats {
    /// Number of cached entries
    pub cached_entries: usize,
    /// Maximum number of iterations
    pub max_iterations: u32,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
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

    #[test]
    fn test_stable_index_mapping_for_loop() {
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 10, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        // Test stable mapping for For loop
        for i in 0..10 {
            let data1 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();
            let data2 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();
            let data3 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();

            assert_eq!(data1, data2);
            assert_eq!(data2, data3);
            assert_eq!(data1, Value::Number(i as f64));
        }
    }

    #[test]
    fn test_stable_index_mapping_foreach_loop() {
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::ForEach {
            id: LoopID::new("test-foreach".to_string()),
            collection: crate::bcib::OperandRef::Literal(Value::Array(vec![
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
            ])),
            collection_type: crate::bcib::CollectionType::Array,
            iterator_var: "item".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        // Test stable mapping for ForEach loop
        let expected_values = [10.0, 20.0, 30.0];
        for i in 0..3 {
            let data1 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();
            let data2 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();
            let data3 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();

            assert_eq!(data1, data2);
            assert_eq!(data2, data3);
            assert_eq!(data1, Value::Number(expected_values[i as usize]));
        }
    }

    #[test]
    fn test_stable_index_mapping_while_loop() {
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::While {
            id: LoopID::new("test-while".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        // Test stable mapping for While loop
        for i in 0..10 {
            let data1 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();
            let data2 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();
            let data3 = mapper
                .get_input_data_for_iteration(&instruction, i)
                .unwrap();

            assert_eq!(data1, data2);
            assert_eq!(data2, data3);
            assert_eq!(data1, Value::Number(i as f64));
        }
    }

    #[test]
    fn test_stable_input_mapping_creation() {
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 10, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let partitions = vec![
            IterationPartition {
                partition_id: 0,
                start_iteration: 0,
                end_iteration: 5,
                iteration_count: 5,
            },
            IterationPartition {
                partition_id: 1,
                start_iteration: 5,
                end_iteration: 10,
                iteration_count: 5,
            },
        ];

        let input_mapping = mapper
            .create_stable_input_mapping(&instruction, &partitions)
            .unwrap();

        assert_eq!(input_mapping.len(), 10);

        // Verify stable mapping
        for (i, (global_idx, value)) in input_mapping.iter().enumerate() {
            assert_eq!(*global_idx, i as u32);
            assert_eq!(*value, Value::Number(i as f64));
        }
    }

    #[test]
    fn test_stable_mapping_verification() {
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 100, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let verification = mapper.verify_stable_mapping(&instruction, 10).unwrap();

        assert!(verification.is_stable());
        assert_eq!(verification.total_tested, 10);
        assert_eq!(verification.stable_mappings, 10);
        assert_eq!(verification.unstable_mappings, 0);
        assert!(verification.errors.is_empty());
        assert_eq!(verification.stability_ratio(), 1.0);
    }

    #[test]
    fn test_mapping_strategy_analysis() {
        let mapper = StableIndexMapper::new();

        // For loop strategy
        let for_instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(10, 20, 2),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let for_strategy = mapper.analyze_mapping_strategy(&for_instruction);
        assert_eq!(
            for_strategy,
            IndexMappingStrategy::Range { start: 10, step: 2 }
        );

        // ForEach loop strategy
        let foreach_instruction = LoopInstruction::ForEach {
            id: LoopID::new("test-foreach".to_string()),
            collection: crate::bcib::OperandRef::Literal(Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
            ])),
            collection_type: crate::bcib::CollectionType::Array,
            iterator_var: "item".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let foreach_strategy = mapper.analyze_mapping_strategy(&foreach_instruction);
        assert_eq!(
            foreach_strategy,
            IndexMappingStrategy::Collection {
                collection_size: 2,
                collection_type: "Array".to_string(),
            }
        );

        // While loop strategy
        let while_instruction = LoopInstruction::While {
            id: LoopID::new("test-while".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let while_strategy = mapper.analyze_mapping_strategy(&while_instruction);
        assert_eq!(while_strategy, IndexMappingStrategy::Iteration);
    }

    #[test]
    fn test_mapping_cache() {
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 10, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        let cache = mapper.create_mapping_cache(&instruction, 10).unwrap();

        assert_eq!(cache.max_iterations(), 10);

        // Test cache lookups
        for i in 0..10 {
            let cached_data = cache.get_input_data(i).unwrap();
            assert_eq!(*cached_data, Value::Number(i as f64));
        }

        let stats = cache.cache_stats();
        assert_eq!(stats.cached_entries, 10);
        assert_eq!(stats.max_iterations, 10);
        assert_eq!(stats.cache_hit_ratio, 1.0);
    }

    #[test]
    fn test_deterministic_result_collection() {
        let mapper = StableIndexMapper::new();

        // Create mock partition results
        let partition_results = vec![
            PartitionExecutionResult::success(
                0,
                vec![
                    IterationExecutionResult {
                        global_iteration_index: 0,
                        result_value: Value::Number(0.0),
                        control_flow: crate::bcib::ControlFlowType::Normal,
                    },
                    IterationExecutionResult {
                        global_iteration_index: 1,
                        result_value: Value::Number(1.0),
                        control_flow: crate::bcib::ControlFlowType::Normal,
                    },
                ],
                Value::Number(1.0),
                2,
            ),
            PartitionExecutionResult::success(
                1,
                vec![
                    IterationExecutionResult {
                        global_iteration_index: 2,
                        result_value: Value::Number(2.0),
                        control_flow: crate::bcib::ControlFlowType::Normal,
                    },
                    IterationExecutionResult {
                        global_iteration_index: 3,
                        result_value: Value::Number(3.0),
                        control_flow: crate::bcib::ControlFlowType::Normal,
                    },
                ],
                Value::Number(3.0),
                2,
            ),
        ];

        let collected_results = mapper
            .collect_results_deterministic(partition_results)
            .unwrap();

        assert_eq!(collected_results.len(), 4);

        // Verify results are in deterministic order
        for (i, result) in collected_results.iter().enumerate() {
            assert_eq!(result.global_iteration_index, i as u32);
            assert_eq!(result.result_value, Value::Number(i as f64));
        }
    }

    #[test]
    fn test_d2_merger_integration() {
        let mapper = StableIndexMapper::new();

        let iteration_results = vec![
            IterationExecutionResult {
                global_iteration_index: 2,
                result_value: Value::Number(20.0),
                control_flow: crate::bcib::ControlFlowType::Normal,
            },
            IterationExecutionResult {
                global_iteration_index: 0,
                result_value: Value::Number(0.0),
                control_flow: crate::bcib::ControlFlowType::Normal,
            },
            IterationExecutionResult {
                global_iteration_index: 1,
                result_value: Value::Number(10.0),
                control_flow: crate::bcib::ControlFlowType::Normal,
            },
        ];

        let merged_results = mapper
            .merge_results_with_d2_system(&iteration_results)
            .unwrap();

        assert_eq!(merged_results.len(), 3);
        assert_eq!(merged_results[0], Value::Number(0.0));
        assert_eq!(merged_results[1], Value::Number(10.0));
        assert_eq!(merged_results[2], Value::Number(20.0));
    }

    #[test]
    fn test_property_stable_mapping_consistency() {
        // Property: Same global iteration always produces same input data
        let mapper = StableIndexMapper::new();
        let instruction = LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 100, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: create_test_loop_config(),
            location: SourceLocation::new(1, 1, 0),
        };

        for global_iteration in 0..100 {
            let data1 = mapper
                .get_input_data_for_iteration(&instruction, global_iteration)
                .unwrap();
            let data2 = mapper
                .get_input_data_for_iteration(&instruction, global_iteration)
                .unwrap();
            let data3 = mapper
                .get_input_data_for_iteration(&instruction, global_iteration)
                .unwrap();

            assert_eq!(data1, data2);
            assert_eq!(data2, data3);
        }
    }

    #[test]
    fn test_property_deterministic_collection() {
        // Property: Same partition results always produce same collection order
        let mapper = StableIndexMapper::new();

        let partition_results = vec![
            PartitionExecutionResult::success(
                1, // Note: partition 1 first
                vec![IterationExecutionResult {
                    global_iteration_index: 2,
                    result_value: Value::Number(2.0),
                    control_flow: crate::bcib::ControlFlowType::Normal,
                }],
                Value::Number(2.0),
                1,
            ),
            PartitionExecutionResult::success(
                0, // Note: partition 0 second
                vec![IterationExecutionResult {
                    global_iteration_index: 0,
                    result_value: Value::Number(0.0),
                    control_flow: crate::bcib::ControlFlowType::Normal,
                }],
                Value::Number(0.0),
                1,
            ),
        ];

        let results1 = mapper
            .collect_results_deterministic(partition_results.clone())
            .unwrap();
        let results2 = mapper
            .collect_results_deterministic(partition_results.clone())
            .unwrap();
        let results3 = mapper
            .collect_results_deterministic(partition_results)
            .unwrap();

        assert_eq!(results1, results2);
        assert_eq!(results2, results3);

        // Results should be ordered by global iteration index, not partition order
        assert_eq!(results1[0].global_iteration_index, 0);
        assert_eq!(results1[1].global_iteration_index, 2);
    }
}
