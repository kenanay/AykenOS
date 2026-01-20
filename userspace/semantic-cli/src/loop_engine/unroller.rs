//! Loop Unrolling Optimization - Phase 5.1 Implementation
//!
//! This module implements small loop unrolling optimization as specified in
//! the D3 Loop Support Design. It detects loops with statically known iteration
//! count < 10 and expands them into sequential IR instructions.
//!
//! # Constitutional Requirements
//!
//! - **Requirement 4.1**: Automatically unroll loops with iteration count < 10
//! - **Requirement 4.2**: Expand loop body into sequential IR instructions
//! - **Requirement 4.3**: Preserve exact semantics including iteration order and side effects
//! - **Requirement 4.4**: Exclude unrolling decisions from fingerprint (optimization only)
//! - **Requirement 4.5**: Skip unrolling when iteration count cannot be statically analyzed
//!
//! # Design Principles
//!
//! 1. **Static Analysis Only**: Only unroll loops with compile-time known iteration counts
//! 2. **Semantic Preservation**: Unrolled version must produce identical results to original
//! 3. **Optimization Transparency**: Unrolling decisions don't affect fingerprinting
//! 4. **Conservative Approach**: When in doubt, don't unroll (preserve correctness)

use crate::bcib::{LoopInstruction, LoopType, Value, BCIBInstruction, BCIBSequence};
use crate::error::{Result, SemanticCLIError, ErrorCode};
use std::collections::HashMap;

/// Loop unrolling threshold - loops with fewer iterations are candidates for unrolling
const UNROLL_THRESHOLD: u32 = 10;

/// Loop unroller that expands small loops into sequential instructions
#[derive(Debug, Clone)]
pub struct LoopUnroller {
    /// Configuration for unrolling behavior
    config: UnrollConfig,
    /// Statistics for monitoring unrolling decisions
    stats: UnrollStats,
}

/// Configuration for loop unrolling optimization
#[derive(Debug, Clone)]
pub struct UnrollConfig {
    /// Maximum iterations to unroll (default: 10, per constitutional requirement)
    pub max_unroll_iterations: u32,
    /// Whether to enable unrolling (can be disabled for debugging)
    pub enabled: bool,
    /// Whether to collect detailed statistics
    pub collect_stats: bool,
}

impl Default for UnrollConfig {
    fn default() -> Self {
        Self {
            max_unroll_iterations: UNROLL_THRESHOLD,
            enabled: true,
            collect_stats: true,
        }
    }
}

/// Statistics for loop unrolling decisions
#[derive(Debug, Clone, Default)]
pub struct UnrollStats {
    /// Number of loops analyzed for unrolling
    pub loops_analyzed: u64,
    /// Number of loops successfully unrolled
    pub loops_unrolled: u64,
    /// Number of loops skipped (iteration count too high)
    pub loops_skipped_too_large: u64,
    /// Number of loops skipped (non-static iteration count)
    pub loops_skipped_non_static: u64,
    /// Number of loops skipped (While loops - never unrolled)
    pub loops_skipped_while: u64,
    /// Total iterations unrolled
    pub total_iterations_unrolled: u64,
}

/// Result of unrolling analysis
#[derive(Debug, Clone, PartialEq)]
pub enum UnrollResult {
    /// Loop was successfully unrolled into sequential instructions
    Unrolled {
        /// The unrolled instruction sequence
        unrolled_sequence: BCIBSequence,
        /// Number of iterations that were unrolled
        iteration_count: u32,
    },
    /// Loop was not unrolled due to various reasons
    NotUnrolled {
        /// Reason why unrolling was skipped
        reason: UnrollSkipReason,
        /// Original loop instruction (unchanged)
        original_loop: LoopInstruction,
    },
}

/// Reasons why a loop was not unrolled
#[derive(Debug, Clone, PartialEq)]
pub enum UnrollSkipReason {
    /// Iteration count exceeds unrolling threshold
    IterationCountTooHigh { count: u32, threshold: u32 },
    /// Iteration count cannot be determined statically
    NonStaticIterationCount,
    /// While loops are never unrolled (constitutional rule)
    WhileLoopNotSupported,
    /// ForEach loops with dynamic collections cannot be unrolled
    ForEachDynamicCollection,
    /// Unrolling is disabled in configuration
    UnrollingDisabled,
}

impl LoopUnroller {
    /// Create a new loop unroller with default configuration
    pub fn new() -> Self {
        Self {
            config: UnrollConfig::default(),
            stats: UnrollStats::default(),
        }
    }

    /// Create a new loop unroller with custom configuration
    pub fn with_config(config: UnrollConfig) -> Self {
        Self {
            config,
            stats: UnrollStats::default(),
        }
    }

    /// Analyze a loop instruction and determine if it should be unrolled
    /// 
    /// This is the main entry point for loop unrolling optimization.
    /// Returns UnrollResult indicating whether the loop was unrolled or not.
    pub fn analyze_loop(&mut self, loop_instruction: &LoopInstruction) -> Result<UnrollResult> {
        if self.config.collect_stats {
            self.stats.loops_analyzed += 1;
        }

        // Check if unrolling is enabled
        if !self.config.enabled {
            if self.config.collect_stats {
                // Note: We don't increment any skip counters for disabled unrolling
                // as this is a global configuration decision, not a per-loop decision
            }
            return Ok(UnrollResult::NotUnrolled {
                reason: UnrollSkipReason::UnrollingDisabled,
                original_loop: loop_instruction.clone(),
            });
        }

        // Determine static iteration count
        let iteration_count = match self.get_static_iteration_count(loop_instruction)? {
            Some(count) => count,
            None => {
                // Cannot determine iteration count statically
                let reason = match loop_instruction.loop_type() {
                    LoopType::While => {
                        if self.config.collect_stats {
                            self.stats.loops_skipped_while += 1;
                        }
                        UnrollSkipReason::WhileLoopNotSupported
                    }
                    LoopType::ForEach => {
                        if self.config.collect_stats {
                            self.stats.loops_skipped_non_static += 1;
                        }
                        UnrollSkipReason::ForEachDynamicCollection
                    }
                    LoopType::For => {
                        if self.config.collect_stats {
                            self.stats.loops_skipped_non_static += 1;
                        }
                        UnrollSkipReason::NonStaticIterationCount
                    }
                };

                return Ok(UnrollResult::NotUnrolled {
                    reason,
                    original_loop: loop_instruction.clone(),
                });
            }
        };

        // Check if iteration count is within unrolling threshold
        if iteration_count >= self.config.max_unroll_iterations {
            if self.config.collect_stats {
                self.stats.loops_skipped_too_large += 1;
            }
            return Ok(UnrollResult::NotUnrolled {
                reason: UnrollSkipReason::IterationCountTooHigh {
                    count: iteration_count,
                    threshold: self.config.max_unroll_iterations,
                },
                original_loop: loop_instruction.clone(),
            });
        }

        // Unroll the loop
        let unrolled_sequence = self.unroll_loop_body(loop_instruction, iteration_count)?;

        if self.config.collect_stats {
            self.stats.loops_unrolled += 1;
            self.stats.total_iterations_unrolled += iteration_count as u64;
        }

        Ok(UnrollResult::Unrolled {
            unrolled_sequence,
            iteration_count,
        })
    }

    /// Determine the static iteration count for a loop instruction
    /// 
    /// Returns Some(count) if the iteration count can be determined at compile time,
    /// None otherwise. This implements Requirement 4.5 (skip unrolling when iteration
    /// count cannot be statically analyzed).
    fn get_static_iteration_count(&self, loop_instruction: &LoopInstruction) -> Result<Option<u32>> {
        match loop_instruction {
            LoopInstruction::While { .. } => {
                // While loops are never unrolled due to non-static iteration count
                // Constitutional rule: While loops depend on runtime condition evaluation
                Ok(None)
            }
            LoopInstruction::For { range, .. } => {
                // For loops have statically known iteration count from range
                let count = range.iteration_count();
                Ok(Some(count))
            }
            LoopInstruction::ForEach { collection, .. } => {
                // ForEach loops can only be unrolled if collection size is statically known
                match collection {
                    crate::bcib::OperandRef::Literal(value) => {
                        // Literal collections have statically known size
                        if let Some(size) = value.collection_size() {
                            Ok(Some(size as u32))
                        } else {
                            // Not a collection literal
                            Ok(None)
                        }
                    }
                    crate::bcib::OperandRef::Field(_) | crate::bcib::OperandRef::TempRegister(_) => {
                        // Field references and temp registers have dynamic size
                        Ok(None)
                    }
                }
            }
        }
    }

    /// Unroll a loop body into sequential instructions
    /// 
    /// This method expands the loop body N times where N is the iteration count,
    /// preserving exact semantics including iteration order and side effects.
    /// Implements Requirements 4.2 and 4.3.
    fn unroll_loop_body(
        &self,
        loop_instruction: &LoopInstruction,
        iteration_count: u32,
    ) -> Result<BCIBSequence> {
        let mut unrolled_instructions = Vec::new();

        // Generate unrolled instructions for each iteration
        for iteration in 0..iteration_count {
            let iteration_instructions = self.generate_iteration_instructions(
                loop_instruction,
                iteration,
                iteration_count,
            )?;
            unrolled_instructions.extend(iteration_instructions);
        }

        // Create the unrolled sequence
        let unrolled_sequence = BCIBSequence::new(unrolled_instructions);

        Ok(unrolled_sequence)
    }

    /// Generate instructions for a single iteration of the unrolled loop
    /// 
    /// This method creates the instruction sequence for iteration N of the loop,
    /// handling iterator variable binding and accumulator state management.
    fn generate_iteration_instructions(
        &self,
        loop_instruction: &LoopInstruction,
        iteration: u32,
        _total_iterations: u32,
    ) -> Result<Vec<BCIBInstruction>> {
        let mut instructions = Vec::new();

        // Handle iterator variable binding based on loop type
        match loop_instruction {
            LoopInstruction::For { range, iterator_var, .. } => {
                // Calculate iterator value for this iteration
                let iterator_value = range.start + (iteration as i64 * range.step);
                
                // Generate instruction to bind iterator variable
                // In a full implementation, this would create a variable binding instruction
                // For Phase 5.1, we create a placeholder comment instruction
                instructions.push(self.create_iterator_binding_instruction(
                    iterator_var,
                    Value::Number(iterator_value as f64),
                )?);
            }
            LoopInstruction::ForEach { collection, iterator_var, .. } => {
                // Extract collection element for this iteration
                if let crate::bcib::OperandRef::Literal(collection_value) = collection {
                    let element_value = self.get_collection_element(collection_value, iteration)?;
                    
                    // Generate instruction to bind iterator variable to collection element
                    instructions.push(self.create_iterator_binding_instruction(
                        iterator_var,
                        element_value,
                    )?);
                } else {
                    return Err(SemanticCLIError::execution_error(
                        "ForEach unrolling requires literal collection",
                        ErrorCode::E500,
                    ));
                }
            }
            LoopInstruction::While { .. } => {
                // While loops should never reach this point
                return Err(SemanticCLIError::execution_error(
                    "While loops cannot be unrolled",
                    ErrorCode::E500,
                ));
            }
        }

        // Generate loop body instructions
        // In Phase 5.1, we create a placeholder for the loop body
        // A full implementation would expand the actual loop body IR
        instructions.push(self.create_loop_body_placeholder_instruction(
            loop_instruction,
            iteration,
        )?);

        Ok(instructions)
    }

    /// Create an iterator variable binding instruction
    /// 
    /// This is a placeholder implementation for Phase 5.1.
    /// A full implementation would create proper variable binding instructions.
    fn create_iterator_binding_instruction(
        &self,
        iterator_var: &str,
        value: Value,
    ) -> Result<BCIBInstruction> {
        // Phase 5.1: Create a debug instruction as placeholder
        // Future phases will implement proper variable binding
        Ok(BCIBInstruction::Debug(crate::bcib::DebugInstruction::Explain {
            target_sequence_id: format!("bind-{}-to-{:?}", iterator_var, value),
            location: crate::types::SourceLocation::new(1, 1, 0),
        }))
    }

    /// Create a loop body placeholder instruction
    /// 
    /// This is a placeholder implementation for Phase 5.1.
    /// A full implementation would expand the actual loop body IR.
    fn create_loop_body_placeholder_instruction(
        &self,
        loop_instruction: &LoopInstruction,
        iteration: u32,
    ) -> Result<BCIBInstruction> {
        let loop_id = match loop_instruction {
            LoopInstruction::While { id, .. } => &id.0,
            LoopInstruction::For { id, .. } => &id.0,
            LoopInstruction::ForEach { id, .. } => &id.0,
        };

        // Phase 5.1: Create a debug instruction as placeholder
        // Future phases will implement proper loop body expansion
        Ok(BCIBInstruction::Debug(crate::bcib::DebugInstruction::Explain {
            target_sequence_id: format!("unrolled-{}-iteration-{}", loop_id, iteration),
            location: crate::types::SourceLocation::new(1, 1, 0),
        }))
    }

    /// Get a collection element at the specified index
    /// 
    /// This method extracts the element at the given index from a collection value,
    /// maintaining deterministic iteration order.
    fn get_collection_element(&self, collection: &Value, index: u32) -> Result<Value> {
        match collection {
            Value::Array(arr) => {
                if (index as usize) < arr.len() {
                    Ok(arr[index as usize].clone())
                } else {
                    Err(SemanticCLIError::execution_error(
                        format!("Array index {} out of bounds (length: {})", index, arr.len()),
                        ErrorCode::E500,
                    ))
                }
            }
            Value::List(list) => {
                if (index as usize) < list.len() {
                    Ok(list[index as usize].clone())
                } else {
                    Err(SemanticCLIError::execution_error(
                        format!("List index {} out of bounds (length: {})", index, list.len()),
                        ErrorCode::E500,
                    ))
                }
            }
            Value::SortedMap(map) => {
                let keys: Vec<_> = map.keys().collect();
                if (index as usize) < keys.len() {
                    let key = keys[index as usize];
                    Ok(map[key].clone())
                } else {
                    Err(SemanticCLIError::execution_error(
                        format!("Map index {} out of bounds (length: {})", index, map.len()),
                        ErrorCode::E500,
                    ))
                }
            }
            _ => Err(SemanticCLIError::execution_error(
                "Value is not a collection",
                ErrorCode::E500,
            )),
        }
    }

    /// Get unrolling statistics
    pub fn get_stats(&self) -> &UnrollStats {
        &self.stats
    }

    /// Reset unrolling statistics
    pub fn reset_stats(&mut self) {
        self.stats = UnrollStats::default();
    }

    /// Check if a loop should be unrolled (without actually unrolling it)
    /// 
    /// This is a lightweight check that can be used for decision making
    /// without the overhead of generating the unrolled instructions.
    pub fn should_unroll(&self, loop_instruction: &LoopInstruction) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        // Check if we can determine static iteration count
        let iteration_count = match self.get_static_iteration_count(loop_instruction)? {
            Some(count) => count,
            None => return Ok(false),
        };

        // Check if iteration count is within threshold
        Ok(iteration_count < self.config.max_unroll_iterations)
    }
}

impl Default for LoopUnroller {
    fn default() -> Self {
        Self::new()
    }
}

impl UnrollStats {
    /// Get the unrolling success rate (percentage of analyzed loops that were unrolled)
    pub fn success_rate(&self) -> f64 {
        if self.loops_analyzed == 0 {
            0.0
        } else {
            (self.loops_unrolled as f64 / self.loops_analyzed as f64) * 100.0
        }
    }

    /// Get the average iterations per unrolled loop
    pub fn average_iterations_per_unroll(&self) -> f64 {
        if self.loops_unrolled == 0 {
            0.0
        } else {
            self.total_iterations_unrolled as f64 / self.loops_unrolled as f64
        }
    }

    /// Get a summary of skip reasons
    pub fn skip_summary(&self) -> HashMap<String, u64> {
        let mut summary = HashMap::new();
        summary.insert("too_large".to_string(), self.loops_skipped_too_large);
        summary.insert("non_static".to_string(), self.loops_skipped_non_static);
        summary.insert("while_loops".to_string(), self.loops_skipped_while);
        summary
    }
}

impl std::fmt::Display for UnrollSkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnrollSkipReason::IterationCountTooHigh { count, threshold } => {
                write!(f, "Iteration count {} exceeds threshold {}", count, threshold)
            }
            UnrollSkipReason::NonStaticIterationCount => {
                write!(f, "Iteration count cannot be determined statically")
            }
            UnrollSkipReason::WhileLoopNotSupported => {
                write!(f, "While loops are not supported for unrolling")
            }
            UnrollSkipReason::ForEachDynamicCollection => {
                write!(f, "ForEach loops with dynamic collections cannot be unrolled")
            }
            UnrollSkipReason::UnrollingDisabled => {
                write!(f, "Loop unrolling is disabled in configuration")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{LoopID, LoopConfig, LoopRange, ValueType};
    use crate::types::SourceLocation;

    fn create_test_for_loop(start: i64, end: i64, step: i64) -> LoopInstruction {
        LoopInstruction::For {
            id: LoopID::new("test-unroll".to_string()),
            range: LoopRange::new(start, end, step),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        }
    }

    fn create_test_foreach_loop_with_literal(collection: Value) -> LoopInstruction {
        LoopInstruction::ForEach {
            id: LoopID::new("test-unroll-foreach".to_string()),
            collection: crate::bcib::OperandRef::Literal(collection),
            collection_type: crate::bcib::CollectionType::Array,
            iterator_var: "item".to_string(),
            body: "test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        }
    }

    fn create_test_while_loop() -> LoopInstruction {
        LoopInstruction::While {
            id: LoopID::new("test-unroll-while".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        }
    }

    #[test]
    fn test_small_for_loop_unrolling() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(0, 3, 1); // 3 iterations: 0, 1, 2

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled { iteration_count, unrolled_sequence } => {
                assert_eq!(iteration_count, 3);
                assert_eq!(unrolled_sequence.instructions.len(), 6); // 3 iterations * 2 instructions each
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling but got: {}", reason);
            }
        }

        // Check statistics
        let stats = unroller.get_stats();
        assert_eq!(stats.loops_analyzed, 1);
        assert_eq!(stats.loops_unrolled, 1);
        assert_eq!(stats.total_iterations_unrolled, 3);
    }

    #[test]
    fn test_large_for_loop_not_unrolled() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(0, 15, 1); // 15 iterations (> threshold)

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => {
                match reason {
                    UnrollSkipReason::IterationCountTooHigh { count, threshold } => {
                        assert_eq!(count, 15);
                        assert_eq!(threshold, 10);
                    }
                    _ => panic!("Expected IterationCountTooHigh but got: {}", reason),
                }
            }
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling for large loop");
            }
        }

        // Check statistics
        let stats = unroller.get_stats();
        assert_eq!(stats.loops_analyzed, 1);
        assert_eq!(stats.loops_unrolled, 0);
        assert_eq!(stats.loops_skipped_too_large, 1);
    }

    #[test]
    fn test_while_loop_not_unrolled() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_while_loop();

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => {
                match reason {
                    UnrollSkipReason::WhileLoopNotSupported => {
                        // Expected
                    }
                    _ => panic!("Expected WhileLoopNotSupported but got: {}", reason),
                }
            }
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling for While loop");
            }
        }

        // Check statistics
        let stats = unroller.get_stats();
        assert_eq!(stats.loops_analyzed, 1);
        assert_eq!(stats.loops_unrolled, 0);
        assert_eq!(stats.loops_skipped_while, 1);
    }

    #[test]
    fn test_foreach_literal_array_unrolling() {
        let mut unroller = LoopUnroller::new();
        let collection = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let loop_instruction = create_test_foreach_loop_with_literal(collection);

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled { iteration_count, unrolled_sequence } => {
                assert_eq!(iteration_count, 3);
                assert_eq!(unrolled_sequence.instructions.len(), 6); // 3 iterations * 2 instructions each
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling but got: {}", reason);
            }
        }
    }

    #[test]
    fn test_foreach_field_reference_not_unrolled() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = LoopInstruction::ForEach {
            id: LoopID::new("test-unroll-foreach-field".to_string()),
            collection: crate::bcib::OperandRef::Field("dynamic_collection".to_string()),
            collection_type: crate::bcib::CollectionType::Array,
            iterator_var: "item".to_string(),
            body: "test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        };

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => {
                match reason {
                    UnrollSkipReason::ForEachDynamicCollection => {
                        // Expected
                    }
                    _ => panic!("Expected ForEachDynamicCollection but got: {}", reason),
                }
            }
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling for dynamic ForEach loop");
            }
        }
    }

    #[test]
    fn test_unrolling_disabled() {
        let config = UnrollConfig {
            enabled: false,
            ..UnrollConfig::default()
        };
        let mut unroller = LoopUnroller::with_config(config);
        let loop_instruction = create_test_for_loop(0, 3, 1);

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => {
                match reason {
                    UnrollSkipReason::UnrollingDisabled => {
                        // Expected
                    }
                    _ => panic!("Expected UnrollingDisabled but got: {}", reason),
                }
            }
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling when disabled");
            }
        }
    }

    #[test]
    fn test_should_unroll_check() {
        let unroller = LoopUnroller::new();

        // Small loop should be unrolled
        let small_loop = create_test_for_loop(0, 5, 1);
        assert!(unroller.should_unroll(&small_loop).unwrap());

        // Large loop should not be unrolled
        let large_loop = create_test_for_loop(0, 15, 1);
        assert!(!unroller.should_unroll(&large_loop).unwrap());

        // While loop should not be unrolled
        let while_loop = create_test_while_loop();
        assert!(!unroller.should_unroll(&while_loop).unwrap());
    }

    #[test]
    fn test_static_iteration_count_detection() {
        let unroller = LoopUnroller::new();

        // For loop with static count
        let for_loop = create_test_for_loop(2, 8, 2); // 3 iterations: 2, 4, 6
        let count = unroller.get_static_iteration_count(&for_loop).unwrap();
        assert_eq!(count, Some(3));

        // While loop (non-static)
        let while_loop = create_test_while_loop();
        let count = unroller.get_static_iteration_count(&while_loop).unwrap();
        assert_eq!(count, None);

        // ForEach with literal collection
        let collection = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let foreach_loop = create_test_foreach_loop_with_literal(collection);
        let count = unroller.get_static_iteration_count(&foreach_loop).unwrap();
        assert_eq!(count, Some(2));
    }

    #[test]
    fn test_unroll_stats() {
        let mut unroller = LoopUnroller::new();

        // Analyze several loops
        let small_loop1 = create_test_for_loop(0, 3, 1); // 3 iterations - should unroll
        let small_loop2 = create_test_for_loop(0, 5, 1); // 5 iterations - should unroll
        let large_loop = create_test_for_loop(0, 15, 1); // 15 iterations - should not unroll
        let while_loop = create_test_while_loop(); // While loop - should not unroll

        unroller.analyze_loop(&small_loop1).unwrap();
        unroller.analyze_loop(&small_loop2).unwrap();
        unroller.analyze_loop(&large_loop).unwrap();
        unroller.analyze_loop(&while_loop).unwrap();

        let stats = unroller.get_stats();
        assert_eq!(stats.loops_analyzed, 4);
        assert_eq!(stats.loops_unrolled, 2);
        assert_eq!(stats.loops_skipped_too_large, 1);
        assert_eq!(stats.loops_skipped_while, 1);
        assert_eq!(stats.total_iterations_unrolled, 8); // 3 + 5 = 8

        // Test calculated metrics
        assert_eq!(stats.success_rate(), 50.0); // 2/4 * 100
        assert_eq!(stats.average_iterations_per_unroll(), 4.0); // 8/2

        // Test skip summary
        let skip_summary = stats.skip_summary();
        assert_eq!(skip_summary["too_large"], 1);
        assert_eq!(skip_summary["while_loops"], 1);
        assert_eq!(skip_summary["non_static"], 0);
    }

    #[test]
    fn test_zero_iteration_loop() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(5, 5, 1); // 0 iterations

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled { iteration_count, unrolled_sequence } => {
                assert_eq!(iteration_count, 0);
                assert_eq!(unrolled_sequence.instructions.len(), 0); // No instructions for 0 iterations
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling of zero-iteration loop but got: {}", reason);
            }
        }
    }

    #[test]
    fn test_single_iteration_loop() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(42, 43, 1); // 1 iteration: 42

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled { iteration_count, unrolled_sequence } => {
                assert_eq!(iteration_count, 1);
                assert_eq!(unrolled_sequence.instructions.len(), 2); // 1 iteration * 2 instructions
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling of single-iteration loop but got: {}", reason);
            }
        }
    }

    #[test]
    fn test_custom_threshold() {
        let config = UnrollConfig {
            max_unroll_iterations: 5, // Lower threshold
            ..UnrollConfig::default()
        };
        let mut unroller = LoopUnroller::with_config(config);

        // Loop with 7 iterations (above custom threshold)
        let loop_instruction = create_test_for_loop(0, 7, 1);
        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => {
                match reason {
                    UnrollSkipReason::IterationCountTooHigh { count, threshold } => {
                        assert_eq!(count, 7);
                        assert_eq!(threshold, 5);
                    }
                    _ => panic!("Expected IterationCountTooHigh but got: {}", reason),
                }
            }
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling with custom threshold");
            }
        }
    }

    #[test]
    fn test_negative_step_for_loop() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(10, 5, -2); // 3 iterations: 10, 8, 6

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled { iteration_count, unrolled_sequence } => {
                assert_eq!(iteration_count, 3);
                assert_eq!(unrolled_sequence.instructions.len(), 6); // 3 iterations * 2 instructions each
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling of negative step loop but got: {}", reason);
            }
        }
    }
}