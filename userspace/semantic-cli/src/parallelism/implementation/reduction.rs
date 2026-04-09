//! Reduction operations for parallel execution
//!
//! This module handles commutative and non-commutative reduction operations
//! in parallel execution. It provides optimized implementations that either
//! preserve ordering (for non-commutative operations) or skip ordering overhead
//! (for commutative operations).
//!
//! ## Design Principles
//!
//! 1. **Operation Classification**: Distinguish commutative from non-commutative operations
//! 2. **Optimization**: Skip ordering overhead for commutative operations
//! 3. **Correctness**: Preserve left-to-right order for non-commutative operations
//! 4. **Parallel Efficiency**: Use Rayon's parallel reduction for commutative operations
//!
//! **Design Reference:** D2 Parallelism Architecture - Reduction Handler section
//! **Requirements:** 10.1, 10.2, 10.3

use crate::bcib::Value;
use crate::execution_plan::IRInstruction;
use crate::parallelism::ParallelismResult;

/// Classification of reduction operations based on commutativity.
///
/// This enum determines the optimization strategy for parallel reductions:
/// - **Commutative**: Order-independent operations that can be optimized
/// - **NonCommutative**: Order-dependent operations that must preserve sequence
///
/// **Validates: Requirements 10.1, 10.2**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReductionType {
    /// Order-independent operations (sum, max, min, logical AND/OR)
    /// 
    /// These operations can be executed in any order without affecting the result:
    /// - `a + b + c = (a + b) + c = a + (b + c)`
    /// - `max(a, b, c) = max(max(a, b), c) = max(a, max(b, c))`
    /// 
    /// **Optimization**: Can skip ordering overhead and use parallel reduction
    Commutative,
    
    /// Order-dependent operations (fold, string concatenation, list construction)
    /// 
    /// These operations depend on the order of operands:
    /// - `"a" + "b" + "c" = "abc"` but `"c" + "a" + "b" = "cab"`
    /// - `fold(list, init, f)` depends on left-to-right evaluation
    /// 
    /// **Requirement**: Must preserve left-to-right merge order using Stable Index Map
    NonCommutative,
}

/// Trait for handling reduction operations in parallel execution.
///
/// The `ReductionHandler` provides classification and execution strategies
/// for different types of reduction operations. It optimizes commutative
/// operations while preserving correctness for non-commutative operations.
///
/// **Validates: Requirements 10.1, 10.2, 10.3**
pub trait ReductionHandler {
    /// Classifies a reduction operation as commutative or non-commutative.
    ///
    /// # Arguments
    ///
    /// * `operation` - The IR instruction to classify
    ///
    /// # Returns
    ///
    /// The reduction type classification
    ///
    /// **Validates: Requirement 10.3**
    fn classify_reduction(&self, operation: &IRInstruction) -> ReductionType;
    
    /// Performs parallel reduction for commutative operations.
    ///
    /// This method uses Rayon's parallel reduction to efficiently combine
    /// values without ordering overhead. The reduction can proceed in any
    /// order since the operation is commutative.
    ///
    /// # Arguments
    ///
    /// * `values` - Values to reduce (order doesn't matter)
    /// * `identity` - Identity element for the operation
    /// * `combine` - Binary combination function
    ///
    /// # Returns
    ///
    /// The reduced result
    ///
    /// **Validates: Requirement 10.1**
    fn reduce_commutative<F>(&self, values: Vec<Value>, identity: Value, combine: F) -> ParallelismResult<Value>
    where
        F: Fn(Value, Value) -> Value + Sync + Send;
    
    /// Performs ordered reduction for non-commutative operations.
    ///
    /// This method preserves left-to-right order using the Stable Index Map.
    /// Values are sorted by their logical indices before reduction to ensure
    /// the result is identical to sequential execution.
    ///
    /// # Arguments
    ///
    /// * `indexed_values` - Values with their logical indices
    /// * `identity` - Identity element for the operation
    /// * `combine` - Binary combination function (order-sensitive)
    ///
    /// # Returns
    ///
    /// The reduced result in correct order
    ///
    /// **Validates: Requirement 10.2**
    fn reduce_non_commutative<F>(
        &self, 
        indexed_values: Vec<(usize, Value)>, 
        identity: Value, 
        combine: F
    ) -> ParallelismResult<Value>
    where
        F: Fn(Value, Value) -> Value;
}

/// Default implementation of reduction handler.
///
/// This implementation provides conservative classification rules and
/// efficient reduction strategies for both commutative and non-commutative
/// operations.
///
/// # Classification Strategy
///
/// - **Arithmetic operations**: Addition, multiplication → Commutative
/// - **Comparison operations**: Max, min → Commutative  
/// - **Logical operations**: AND, OR → Commutative
/// - **String operations**: Concatenation → Non-commutative
/// - **List operations**: Construction, folding → Non-commutative
/// - **Unknown operations**: Default to Non-commutative for safety
///
/// **Validates: Requirements 10.1, 10.2, 10.3**
#[derive(Debug, Clone, Default)]
pub struct DefaultReductionHandler;

impl DefaultReductionHandler {
    /// Creates a new default reduction handler.
    pub fn new() -> Self {
        Self
    }

    fn canonicalize_value(&self, value: Value) -> Value {
        match value {
            Value::Number(number) => Value::Number(self.canonicalize_number(number)),
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(|item| self.canonicalize_value(item))
                    .collect(),
            ),
            Value::List(items) => Value::List(
                items
                    .into_iter()
                    .map(|item| self.canonicalize_value(item))
                    .collect(),
            ),
            Value::SortedMap(map) => Value::SortedMap(
                map.into_iter()
                    .map(|(key, value)| (key, self.canonicalize_value(value)))
                    .collect(),
            ),
            other => other,
        }
    }

    fn canonicalize_number(&self, value: f64) -> f64 {
        if value.is_nan() {
            f64::from_bits(0x7FF8_0000_0000_0000)
        } else if value == 0.0 {
            0.0
        } else {
            value
        }
    }

    fn canonical_sort_key(&self, value: &Value) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.append_canonical_bytes(value, &mut bytes);
        bytes
    }

    fn append_canonical_bytes(&self, value: &Value, bytes: &mut Vec<u8>) {
        match value {
            Value::String(text) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
                bytes.extend_from_slice(text.as_bytes());
            }
            Value::Number(number) => {
                bytes.push(0x02);
                bytes.extend_from_slice(&self.canonicalize_number(*number).to_le_bytes());
            }
            Value::Boolean(flag) => {
                bytes.push(0x03);
                bytes.push(u8::from(*flag));
            }
            Value::Array(items) => {
                bytes.push(0x04);
                bytes.extend_from_slice(&(items.len() as u32).to_le_bytes());
                for item in items {
                    let item_key = self.canonical_sort_key(item);
                    bytes.extend_from_slice(&(item_key.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&item_key);
                }
            }
            Value::List(items) => {
                bytes.push(0x05);
                bytes.extend_from_slice(&(items.len() as u32).to_le_bytes());
                for item in items {
                    let item_key = self.canonical_sort_key(item);
                    bytes.extend_from_slice(&(item_key.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&item_key);
                }
            }
            Value::SortedMap(map) => {
                bytes.push(0x06);
                bytes.extend_from_slice(&(map.len() as u32).to_le_bytes());
                for (key, value) in map {
                    bytes.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    let value_key = self.canonical_sort_key(value);
                    bytes.extend_from_slice(&(value_key.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&value_key);
                }
            }
        }
    }
    
    /// Checks if a value represents a numeric operation.
    fn is_numeric_operation(&self, _operation: &IRInstruction) -> bool {
        // In a real implementation, this would analyze the IR instruction
        // to determine if it performs numeric operations like addition,
        // multiplication, etc.
        
        // For now, we'll be conservative and assume most operations
        // are non-commutative unless explicitly known to be commutative
        false
    }
    
    /// Checks if a value represents a comparison operation.
    fn is_comparison_operation(&self, operation: &IRInstruction) -> bool {
        matches!(operation, IRInstruction::Compare { .. })
    }
    
    /// Checks if a value represents a logical operation.
    fn is_logical_operation(&self, operation: &IRInstruction) -> bool {
        matches!(operation, IRInstruction::LogicalOp { .. })
    }
}

impl ReductionHandler for DefaultReductionHandler {
    fn classify_reduction(&self, operation: &IRInstruction) -> ReductionType {
        // Classify based on operation type
        if self.is_comparison_operation(operation) {
            // Comparison operations like max, min are typically commutative
            ReductionType::Commutative
        } else if self.is_logical_operation(operation) {
            // Logical operations like AND, OR are commutative
            ReductionType::Commutative
        } else if self.is_numeric_operation(operation) {
            // Numeric operations like sum, product are commutative
            ReductionType::Commutative
        } else {
            // Default to non-commutative for safety
            // This includes string concatenation, list construction, etc.
            ReductionType::NonCommutative
        }
    }
    
    fn reduce_commutative<F>(&self, values: Vec<Value>, identity: Value, combine: F) -> ParallelismResult<Value>
    where
        F: Fn(Value, Value) -> Value + Sync + Send,
    {
        if values.is_empty() {
            return Ok(self.canonicalize_value(identity));
        }

        // Determinism is constitutional. Canonicalize and sort inputs so the
        // merge order is independent from caller order and worker scheduling.
        let mut ordered_values: Vec<(Vec<u8>, Value)> = values
            .into_iter()
            .map(|value| {
                let canonical = self.canonicalize_value(value);
                let key = self.canonical_sort_key(&canonical);
                (key, canonical)
            })
            .collect();
        ordered_values.sort_by(|left, right| left.0.cmp(&right.0));

        let result = ordered_values
            .into_iter()
            .map(|(_, value)| value)
            .fold(self.canonicalize_value(identity), |acc, value| {
                self.canonicalize_value(combine(acc, value))
            });

        Ok(result)
    }
    
    fn reduce_non_commutative<F>(
        &self, 
        mut indexed_values: Vec<(usize, Value)>, 
        identity: Value, 
        combine: F
    ) -> ParallelismResult<Value>
    where
        F: Fn(Value, Value) -> Value,
    {
        if indexed_values.is_empty() {
            return Ok(identity);
        }
        
        // Sort by index to preserve left-to-right order
        // This is the key difference from commutative reduction
        indexed_values.sort_by_key(|(idx, _)| *idx);
        
        // Perform sequential reduction in correct order
        let result = indexed_values
            .into_iter()
            .map(|(_, value)| value)
            .fold(identity, |acc, value| combine(acc, value));
        
        Ok(result)
    }
}

/// Common reduction operations for convenience.
///
/// This module provides pre-implemented reduction functions for common
/// operations like sum, product, max, min, etc.
pub mod operations {
    use super::*;
    
    /// Reduces values by addition (commutative).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let values = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    /// let result = sum(values); // Value::Number(6.0)
    /// ```
    pub fn sum(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value> {
        let identity = Value::Number(0.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            _ => Value::Number(0.0), // Error case, should be handled properly
        };
        
        handler.reduce_commutative(values, identity, combine)
    }
    
    /// Reduces values by multiplication (commutative).
    pub fn product(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value> {
        let identity = Value::Number(1.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x * y),
            _ => Value::Number(1.0), // Error case
        };
        
        handler.reduce_commutative(values, identity, combine)
    }
    
    /// Reduces values by finding maximum (commutative).
    pub fn max(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value> {
        if values.is_empty() {
            return Ok(Value::Number(f64::NEG_INFINITY));
        }
        
        let identity = Value::Number(f64::NEG_INFINITY);
        let combine = |a: Value, b: Value| match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x.max(*y)),
            _ => a, // Keep first value on error
        };
        
        handler.reduce_commutative(values, identity, combine)
    }
    
    /// Reduces values by finding minimum (commutative).
    pub fn min(handler: &DefaultReductionHandler, values: Vec<Value>) -> ParallelismResult<Value> {
        if values.is_empty() {
            return Ok(Value::Number(f64::INFINITY));
        }
        
        let identity = Value::Number(f64::INFINITY);
        let combine = |a: Value, b: Value| match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x.min(*y)),
            _ => a, // Keep first value on error
        };
        
        handler.reduce_commutative(values, identity, combine)
    }
    
    /// Reduces strings by concatenation (non-commutative).
    ///
    /// This operation preserves left-to-right order, so "a" + "b" + "c" = "abc".
    pub fn concat(
        handler: &DefaultReductionHandler, 
        indexed_values: Vec<(usize, Value)>
    ) -> ParallelismResult<Value> {
        let identity = Value::String("".to_string());
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::String(x), Value::String(y)) => Value::String(x + &y),
            (Value::String(x), _) => Value::String(x), // Keep string on type mismatch
            (_, Value::String(y)) => Value::String(y),
            _ => Value::String("".to_string()), // Error case
        };
        
        handler.reduce_non_commutative(indexed_values, identity, combine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{Value, ComparisonOp, LogicalOperator};
    use crate::execution_plan::IRInstruction;

    // ===== ReductionType Tests =====

    #[test]
    fn test_reduction_type_enum() {
        let commutative = ReductionType::Commutative;
        let non_commutative = ReductionType::NonCommutative;
        
        // Test Debug trait
        assert_eq!(format!("{:?}", commutative), "Commutative");
        assert_eq!(format!("{:?}", non_commutative), "NonCommutative");
        
        // Test PartialEq trait
        assert_eq!(commutative, ReductionType::Commutative);
        assert_ne!(commutative, non_commutative);
    }

    // ===== DefaultReductionHandler Tests =====

    #[test]
    fn test_handler_creation() {
        let handler = DefaultReductionHandler::new();
        assert!(format!("{:?}", handler).contains("DefaultReductionHandler"));
    }

    #[test]
    fn test_classify_comparison_operation() {
        let handler = DefaultReductionHandler::new();
        let compare_op = IRInstruction::Compare {
            left_register: 0,
            operator: ComparisonOp::GreaterThan,
            right_register: 1,
            target_register: 2,
        };
        
        let classification = handler.classify_reduction(&compare_op);
        assert_eq!(classification, ReductionType::Commutative);
    }

    #[test]
    fn test_classify_logical_operation() {
        let handler = DefaultReductionHandler::new();
        let logical_op = IRInstruction::LogicalOp {
            operation: LogicalOperator::And,
            operand_registers: vec![0, 1],
            target_register: 2,
        };
        
        let classification = handler.classify_reduction(&logical_op);
        assert_eq!(classification, ReductionType::Commutative);
    }

    #[test]
    fn test_classify_unknown_operation() {
        let handler = DefaultReductionHandler::new();
        let load_op = IRInstruction::LoadContext {
            context_id: "test".to_string(),
            target_register: 0,
        };
        
        // Unknown operations should default to non-commutative for safety
        let classification = handler.classify_reduction(&load_op);
        assert_eq!(classification, ReductionType::NonCommutative);
    }

    // ===== Commutative Reduction Tests =====

    #[test]
    fn test_reduce_commutative_empty() {
        let handler = DefaultReductionHandler::new();
        let values = vec![];
        let identity = Value::Number(0.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            _ => Value::Number(0.0),
        };
        
        let result = handler.reduce_commutative(values, identity, combine).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_reduce_commutative_single_value() {
        let handler = DefaultReductionHandler::new();
        let values = vec![Value::Number(42.0)];
        let identity = Value::Number(0.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            _ => Value::Number(0.0),
        };
        
        let result = handler.reduce_commutative(values, identity, combine).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_reduce_commutative_multiple_values() {
        let handler = DefaultReductionHandler::new();
        let values = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        let identity = Value::Number(0.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            _ => Value::Number(0.0),
        };
        
        let result = handler.reduce_commutative(values, identity, combine).unwrap();
        assert_eq!(result, Value::Number(10.0)); // 1 + 2 + 3 + 4 = 10
    }

    // ===== Non-Commutative Reduction Tests =====

    #[test]
    fn test_reduce_non_commutative_empty() {
        let handler = DefaultReductionHandler::new();
        let indexed_values = vec![];
        let identity = Value::String("".to_string());
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::String(x), Value::String(y)) => Value::String(x + &y),
            _ => Value::String("".to_string()),
        };
        
        let result = handler.reduce_non_commutative(indexed_values, identity, combine).unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn test_reduce_non_commutative_ordered() {
        let handler = DefaultReductionHandler::new();
        let indexed_values = vec![
            (0, Value::String("a".to_string())),
            (1, Value::String("b".to_string())),
            (2, Value::String("c".to_string())),
        ];
        let identity = Value::String("".to_string());
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::String(x), Value::String(y)) => Value::String(x + &y),
            _ => Value::String("".to_string()),
        };
        
        let result = handler.reduce_non_commutative(indexed_values, identity, combine).unwrap();
        assert_eq!(result, Value::String("abc".to_string()));
    }

    #[test]
    fn test_reduce_non_commutative_unordered_input() {
        let handler = DefaultReductionHandler::new();
        // Input in wrong order - should be sorted by index
        let indexed_values = vec![
            (2, Value::String("c".to_string())),
            (0, Value::String("a".to_string())),
            (1, Value::String("b".to_string())),
        ];
        let identity = Value::String("".to_string());
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::String(x), Value::String(y)) => Value::String(x + &y),
            _ => Value::String("".to_string()),
        };
        
        let result = handler.reduce_non_commutative(indexed_values, identity, combine).unwrap();
        // Should still produce "abc" because indices are sorted
        assert_eq!(result, Value::String("abc".to_string()));
    }

    // ===== Common Operations Tests =====

    #[test]
    fn test_sum_operation() {
        let handler = DefaultReductionHandler::new();
        let values = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ];
        
        let result = operations::sum(&handler, values).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_sum_operation_extreme_magnitudes_is_deterministic() {
        let handler = DefaultReductionHandler::new();
        let values1 = vec![
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(1.671_359_021_813_273_5e298),
            Value::Number(2.876_743_867_543_314_6e302),
            Value::Number(0.0),
            Value::Number(-8.678_323_354_081_092e304),
        ];
        let mut values2 = values1.clone();
        values2.reverse();

        let result1 = operations::sum(&handler, values1).unwrap();
        let result2 = operations::sum(&handler, values2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_product_operation() {
        let handler = DefaultReductionHandler::new();
        let values = vec![
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        
        let result = operations::product(&handler, values).unwrap();
        assert_eq!(result, Value::Number(24.0));
    }

    #[test]
    fn test_max_operation() {
        let handler = DefaultReductionHandler::new();
        let values = vec![
            Value::Number(1.0),
            Value::Number(5.0),
            Value::Number(3.0),
        ];
        
        let result = operations::max(&handler, values).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_min_operation() {
        let handler = DefaultReductionHandler::new();
        let values = vec![
            Value::Number(5.0),
            Value::Number(1.0),
            Value::Number(3.0),
        ];
        
        let result = operations::min(&handler, values).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_concat_operation() {
        let handler = DefaultReductionHandler::new();
        let indexed_values = vec![
            (0, Value::String("Hello".to_string())),
            (1, Value::String(" ".to_string())),
            (2, Value::String("World".to_string())),
        ];
        
        let result = operations::concat(&handler, indexed_values).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_concat_operation_unordered() {
        let handler = DefaultReductionHandler::new();
        // Input in wrong order
        let indexed_values = vec![
            (2, Value::String("World".to_string())),
            (0, Value::String("Hello".to_string())),
            (1, Value::String(" ".to_string())),
        ];
        
        let result = operations::concat(&handler, indexed_values).unwrap();
        // Should still produce correct order
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    // ===== Property Tests =====

    #[test]
    fn test_property_commutative_order_independence() {
        // Property: Commutative operations should produce same result regardless of input order
        let handler = DefaultReductionHandler::new();
        let identity = Value::Number(0.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            _ => Value::Number(0.0),
        };
        
        let values1 = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let values2 = vec![Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)];
        let values3 = vec![Value::Number(2.0), Value::Number(3.0), Value::Number(1.0)];
        
        let result1 = handler.reduce_commutative(values1, identity.clone(), &combine).unwrap();
        let result2 = handler.reduce_commutative(values2, identity.clone(), &combine).unwrap();
        let result3 = handler.reduce_commutative(values3, identity, &combine).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
        assert_eq!(result1, Value::Number(6.0));
    }

    #[test]
    fn test_property_non_commutative_order_preservation() {
        // Property: Non-commutative operations must preserve index order
        let handler = DefaultReductionHandler::new();
        let identity = Value::String("".to_string());
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::String(x), Value::String(y)) => Value::String(x + &y),
            _ => Value::String("".to_string()),
        };
        
        // Different input orders should produce same result when indices are preserved
        let indexed_values1 = vec![
            (0, Value::String("a".to_string())),
            (1, Value::String("b".to_string())),
            (2, Value::String("c".to_string())),
        ];
        
        let indexed_values2 = vec![
            (2, Value::String("c".to_string())),
            (0, Value::String("a".to_string())),
            (1, Value::String("b".to_string())),
        ];
        
        let result1 = handler.reduce_non_commutative(indexed_values1, identity.clone(), &combine).unwrap();
        let result2 = handler.reduce_non_commutative(indexed_values2, identity, &combine).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(result1, Value::String("abc".to_string()));
    }

    // ===== Integration Tests =====

    #[test]
    fn test_concrete_implementation() {
        let handler = DefaultReductionHandler::new();
        
        // Test all trait methods
        let load_op = IRInstruction::LoadContext {
            context_id: "test".to_string(),
            target_register: 0,
        };
        
        let classification = handler.classify_reduction(&load_op);
        assert_eq!(classification, ReductionType::NonCommutative);
        
        let values = vec![Value::Number(1.0), Value::Number(2.0)];
        let identity = Value::Number(0.0);
        let combine = |a: Value, b: Value| match (a, b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            _ => Value::Number(0.0),
        };
        
        let result = handler.reduce_commutative(values, identity, combine).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }
}
