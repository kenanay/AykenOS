//! Parallel Safety Analyzer
//!
//! This module provides analysis of IR blocks to determine their parallel safety classification.
//! The analyzer examines all operations in a block and classifies it as Safe, Unsafe, or ReductionOnly
//! based on the operations it contains.
//!
//! **Design Reference:** D2 Parallelism Architecture - Section "Parallel Safety Classification"
//! **Requirements:** 1.5, 1.6

use crate::execution_plan::{IRBlock, IRInstruction, ParallelSafety};

/// Trait for analyzing IR blocks to determine parallel safety.
///
/// The analyzer examines all operations in a block and classifies it based on:
/// - **Safe**: Pure data transformations with no side effects (map, filter, projection)
/// - **Unsafe**: Operations with side effects, IO, or order-sensitive operations
/// - **ReductionOnly**: Reducible but not mappable (e.g., fold with state)
///
/// **Validates: Requirements 1.5, 1.6**
pub trait ParallelSafetyAnalyzer {
    /// Analyze an IR block and return its parallel safety classification.
    ///
    /// This method examines all instructions in the block and determines whether
    /// the block can be safely parallelized.
    ///
    /// # Classification Rules
    ///
    /// - If any operation has side effects → Unsafe
    /// - If all operations are pure data transformations → Safe
    /// - If block contains stateful reduction → ReductionOnly
    fn analyze_block(&self, block: &IRBlock) -> ParallelSafety;

    /// Check if an operation is pure (no side effects, deterministic).
    ///
    /// Pure operations include:
    /// - LoadContext, LoadField, LoadLiteral (data loading)
    /// - Compare, LogicalOp (pure computations)
    /// - ApplyFilter (pure data transformation)
    fn is_pure_operation(&self, op: &IRInstruction) -> bool;

    /// Check if an operation has side effects.
    ///
    /// Side effects include:
    /// - IO operations
    /// - Mutable state modifications
    /// - Non-deterministic operations
    fn has_side_effects(&self, op: &IRInstruction) -> bool;
}

/// Default implementation of parallel safety analyzer.
///
/// This analyzer uses conservative classification rules to ensure safety:
/// - Pure data operations are classified as Safe
/// - Control flow operations are analyzed in context
/// - Unknown or potentially unsafe operations default to Unsafe
///
/// **Validates: Requirements 1.5, 1.6**
#[derive(Debug, Clone, Default)]
pub struct DefaultSafetyAnalyzer;

impl DefaultSafetyAnalyzer {
    /// Create a new default safety analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Check if an instruction is a control flow operation.
    ///
    /// Control flow operations (Branch, Return) are safe in themselves,
    /// but their safety depends on the operations they control.
    fn is_control_flow(&self, op: &IRInstruction) -> bool {
        matches!(op, IRInstruction::Branch { .. } | IRInstruction::Return { .. })
    }
}

impl ParallelSafetyAnalyzer for DefaultSafetyAnalyzer {
    fn analyze_block(&self, block: &IRBlock) -> ParallelSafety {
        // If the block is empty, it's safe (no operations to execute)
        if block.instructions.is_empty() {
            return ParallelSafety::Safe;
        }

        // Scan all instructions in the block
        for instruction in &block.instructions {
            // If any instruction has side effects, the block is unsafe
            if self.has_side_effects(instruction) {
                return ParallelSafety::Unsafe;
            }

            // Control flow operations are safe in themselves
            // (their safety is determined by the operations they control)
            if self.is_control_flow(instruction) {
                continue;
            }

            // If we find a non-pure, non-control-flow operation, it's unsafe
            if !self.is_pure_operation(instruction) {
                return ParallelSafety::Unsafe;
            }
        }

        // If all operations are pure, the block is safe for parallelization
        ParallelSafety::Safe
    }

    fn is_pure_operation(&self, op: &IRInstruction) -> bool {
        match op {
            // Data loading operations are pure (deterministic, no side effects)
            IRInstruction::LoadContext { .. } => true,
            IRInstruction::LoadField { .. } => true,
            IRInstruction::LoadLiteral { .. } => true,

            // Comparison and logical operations are pure
            IRInstruction::Compare { .. } => true,
            IRInstruction::LogicalOp { .. } => true,

            // Filter operations are pure data transformations
            // (they don't modify the input, just select elements)
            IRInstruction::ApplyFilter { .. } => true,

            // Control flow operations are handled separately
            IRInstruction::Branch { .. } => true,
            IRInstruction::Return { .. } => true,
        }
    }

    fn has_side_effects(&self, op: &IRInstruction) -> bool {
        // Currently, all our IR instructions are pure
        // This method is here for future extensibility when we add:
        // - IO operations (file read/write, network)
        // - Mutable state operations (variable assignment)
        // - Non-deterministic operations (random number generation, timestamps)
        
        match op {
            // All current operations are pure
            IRInstruction::LoadContext { .. } => false,
            IRInstruction::LoadField { .. } => false,
            IRInstruction::LoadLiteral { .. } => false,
            IRInstruction::Compare { .. } => false,
            IRInstruction::LogicalOp { .. } => false,
            IRInstruction::ApplyFilter { .. } => false,
            IRInstruction::Branch { .. } => false,
            IRInstruction::Return { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{ComparisonOp, FilterExpression, LogicalOperator, Value};
    use crate::execution_plan::{BlockTerminator, IRBlock, IRInstruction, ParallelSafety};

    // ===== Helper Functions =====

    fn create_test_block(instructions: Vec<IRInstruction>) -> IRBlock {
        IRBlock::new(
            0,
            instructions,
            BlockTerminator::Return { register: 0 },
        )
    }

    // ===== Pure Operation Tests =====

    #[test]
    fn test_load_context_is_pure() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::LoadContext {
            context_id: "users".to_string(),
            target_register: 0,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
    }

    #[test]
    fn test_load_field_is_pure() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::LoadField {
            source_register: 0,
            field_name: "name".to_string(),
            target_register: 1,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
    }

    #[test]
    fn test_load_literal_is_pure() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::LoadLiteral {
            value: Value::Number(42.0),
            target_register: 0,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
    }

    #[test]
    fn test_compare_is_pure() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::Compare {
            left_register: 0,
            operator: ComparisonOp::Equal,
            right_register: 1,
            target_register: 2,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
    }

    #[test]
    fn test_logical_op_is_pure() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::LogicalOp {
            operation: LogicalOperator::And,
            operand_registers: vec![0, 1],
            target_register: 2,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
    }

    #[test]
    fn test_apply_filter_is_pure() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::ApplyFilter {
            context_register: 0,
            filter_expression: FilterExpression::new(
                "age".to_string(),
                ComparisonOp::GreaterThan,
                crate::bcib::OperandRef::Literal(Value::Number(18.0)),
            ),
            target_register: 1,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
    }

    #[test]
    fn test_branch_is_control_flow() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::Branch {
            condition_register: 0,
            true_block: 1,
            false_block: 2,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
        assert!(analyzer.is_control_flow(&instruction));
    }

    #[test]
    fn test_return_is_control_flow() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let instruction = IRInstruction::Return {
            source_register: 0,
        };

        assert!(analyzer.is_pure_operation(&instruction));
        assert!(!analyzer.has_side_effects(&instruction));
        assert!(analyzer.is_control_flow(&instruction));
    }

    // ===== Block Analysis Tests =====

    #[test]
    fn test_empty_block_is_safe() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let block = create_test_block(vec![]);

        let safety = analyzer.analyze_block(&block);
        assert_eq!(safety, ParallelSafety::Safe);
    }

    #[test]
    fn test_pure_operations_block_is_safe() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let block = create_test_block(vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
            IRInstruction::LoadField {
                source_register: 0,
                field_name: "age".to_string(),
                target_register: 1,
            },
            IRInstruction::LoadLiteral {
                value: Value::Number(18.0),
                target_register: 2,
            },
            IRInstruction::Compare {
                left_register: 1,
                operator: ComparisonOp::GreaterThan,
                right_register: 2,
                target_register: 3,
            },
        ]);

        let safety = analyzer.analyze_block(&block);
        assert_eq!(safety, ParallelSafety::Safe);
    }

    #[test]
    fn test_filter_operation_block_is_safe() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let block = create_test_block(vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
            IRInstruction::ApplyFilter {
                context_register: 0,
                filter_expression: FilterExpression::new(
                    "age".to_string(),
                    ComparisonOp::GreaterThan,
                    crate::bcib::OperandRef::Literal(Value::Number(18.0)),
                ),
                target_register: 1,
            },
        ]);

        let safety = analyzer.analyze_block(&block);
        assert_eq!(safety, ParallelSafety::Safe);
    }

    #[test]
    fn test_complex_pure_block_is_safe() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let block = create_test_block(vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
            IRInstruction::LoadField {
                source_register: 0,
                field_name: "age".to_string(),
                target_register: 1,
            },
            IRInstruction::LoadLiteral {
                value: Value::Number(18.0),
                target_register: 2,
            },
            IRInstruction::Compare {
                left_register: 1,
                operator: ComparisonOp::GreaterThan,
                right_register: 2,
                target_register: 3,
            },
            IRInstruction::LoadField {
                source_register: 0,
                field_name: "active".to_string(),
                target_register: 4,
            },
            IRInstruction::LogicalOp {
                operation: LogicalOperator::And,
                operand_registers: vec![3, 4],
                target_register: 5,
            },
        ]);

        let safety = analyzer.analyze_block(&block);
        assert_eq!(safety, ParallelSafety::Safe);
    }

    // ===== Integration Tests =====

    #[test]
    fn test_analyzer_trait_implementation() {
        let analyzer = DefaultSafetyAnalyzer::new();
        let block = create_test_block(vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
        ]);

        // Test that the trait methods work correctly
        let _: &dyn ParallelSafetyAnalyzer = &analyzer;
        let safety = analyzer.analyze_block(&block);
        assert_eq!(safety, ParallelSafety::Safe);
    }

    #[test]
    fn test_default_trait_implementation() {
        let analyzer = DefaultSafetyAnalyzer::default();
        let block = create_test_block(vec![
            IRInstruction::LoadLiteral {
                value: Value::Number(42.0),
                target_register: 0,
            },
        ]);

        let safety = analyzer.analyze_block(&block);
        assert_eq!(safety, ParallelSafety::Safe);
    }
}
