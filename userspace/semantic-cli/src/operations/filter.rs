//! Filter Evaluation Implementation
//!
//! This module implements filter evaluation using the OperandRef model with normalization flags.
//! Supports comparison operations, logical operations, and field access.

use crate::bcib::{ComparisonOp, FilterExpression, LogicalOperator, OperandRef, Value};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::time::Instant;

/// Filter evaluator for BCIB filter expressions
pub struct FilterEvaluator {
    // NOTE: Registers are reserved for Gate C (normalized BCIB execution)
    // In Phase 3.5.1, filters use direct field evaluation without registers
    _registers_placeholder: HashMap<u16, JsonValue>, // Gate C placeholder
    _next_register_placeholder: u16,                 // Gate C placeholder
}

/// Filter evaluation result
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub items: Vec<JsonValue>,
    pub total_evaluated: usize,
    pub matched_count: usize,
    pub evaluation_time_ms: u64,
    pub normalized: bool,
}

impl FilterEvaluator {
    /// Create new filter evaluator
    pub fn new() -> Self {
        Self {
            _registers_placeholder: HashMap::new(), // Gate C placeholder
            _next_register_placeholder: 1,          // Gate C placeholder
        }
    }

    /// Evaluate filter expression against data items
    pub fn evaluate_filter(
        &mut self,
        filter: &FilterExpression,
        items: &[JsonValue],
    ) -> Result<FilterResult> {
        let start_time = Instant::now();

        let mut matched_items = Vec::new();
        let total_evaluated = items.len();

        for item in items {
            if self.evaluate_filter_for_item(filter, item)? {
                matched_items.push(item.clone());
            }
        }

        let matched_count = matched_items.len();
        let evaluation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(FilterResult {
            items: matched_items,
            total_evaluated,
            matched_count,
            evaluation_time_ms,
            normalized: filter.normalized,
        })
    }

    /// Evaluate filter expression for a single item
    fn evaluate_filter_for_item(
        &mut self,
        filter: &FilterExpression,
        item: &JsonValue,
    ) -> Result<bool> {
        // Check if filter is normalized (Phase 3.5.1 constraint)
        if filter.normalized {
            return Err(SemanticCLIError::validation_error(
                "Normalized filters not supported in Phase 3.5.1",
                "Use non-normalized filters in Phase 3.5.1",
                ErrorCode::E400,
            ));
        }

        // **GATE B FIX:** Use canonical FilterExpression API
        // FilterExpression has: field (String) + operator + value (OperandRef)
        let field_value = self.get_field_value(&filter.field, item);
        let comparison_value = self.resolve_operand_value(&filter.value, item)?;

        // Perform comparison
        self.compare_values(&field_value, &filter.operator, &comparison_value)
    }

    /// Get field value from item (canonical field access)
    fn get_field_value(&self, field_name: &str, item: &JsonValue) -> JsonValue {
        item.get(field_name).cloned().unwrap_or(JsonValue::Null)
    }

    /// Resolve operand value (canonical operand resolution)
    fn resolve_operand_value(&self, operand: &OperandRef, item: &JsonValue) -> Result<JsonValue> {
        match operand {
            OperandRef::Field(field_name) => Ok(self.get_field_value(field_name, item)),
            OperandRef::Literal(literal) => self.bcib_value_to_json(literal),
            OperandRef::TempRegister(register_id) => {
                // In Phase 3.5.1, temp registers in filters are not allowed
                Err(SemanticCLIError::validation_error(
                    format!(
                        "Temp register {} not allowed in Phase 3.5.1 filters",
                        register_id
                    ),
                    "Use Field or Literal operands in Phase 3.5.1 filters",
                    ErrorCode::E400,
                ))
            }
        }
    }

    /// Convert BCIB Value to JsonValue
    fn bcib_value_to_json(&self, value: &Value) -> Result<JsonValue> {
        match value {
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Number(n) => Ok(JsonValue::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
            )),
            Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
            // Collections are not supported in filter operations (Phase 3.1)
            Value::Array(_) | Value::List(_) | Value::SortedMap(_) => {
                Err(SemanticCLIError::execution_error(
                    "Collection values cannot be used in filter operations",
                    ErrorCode::E500,
                ))
            }
        }
    }

    /// Compare two JSON values using comparison operator
    fn compare_values(
        &self,
        left: &JsonValue,
        operator: &ComparisonOp,
        right: &JsonValue,
    ) -> Result<bool> {
        match operator {
            ComparisonOp::Equal => Ok(self.values_equal(left, right)),
            ComparisonOp::NotEqual => Ok(!self.values_equal(left, right)),
            ComparisonOp::LessThan => self.values_less_than(left, right),
            ComparisonOp::LessThanOrEqual => {
                Ok(self.values_equal(left, right) || self.values_less_than(left, right)?)
            }
            ComparisonOp::GreaterThan => {
                Ok(!self.values_equal(left, right) && !self.values_less_than(left, right)?)
            }
            ComparisonOp::GreaterThanOrEqual => {
                Ok(self.values_equal(left, right) || !self.values_less_than(left, right)?)
            }
        }
    }

    /// Check if two JSON values are equal
    fn values_equal(&self, left: &JsonValue, right: &JsonValue) -> bool {
        match (left, right) {
            (JsonValue::String(l), JsonValue::String(r)) => l == r,
            (JsonValue::Number(l), JsonValue::Number(r)) => {
                l.as_f64().unwrap_or(0.0) == r.as_f64().unwrap_or(0.0)
            }
            (JsonValue::Bool(l), JsonValue::Bool(r)) => l == r,
            (JsonValue::Null, JsonValue::Null) => true,
            // Type coercion for mixed comparisons
            (JsonValue::String(s), JsonValue::Number(n)) => s
                .parse::<f64>()
                .map(|parsed| parsed == n.as_f64().unwrap_or(0.0))
                .unwrap_or(false),
            (JsonValue::Number(n), JsonValue::String(s)) => s
                .parse::<f64>()
                .map(|parsed| parsed == n.as_f64().unwrap_or(0.0))
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Check if left value is less than right value
    fn values_less_than(&self, left: &JsonValue, right: &JsonValue) -> Result<bool> {
        match (left, right) {
            (JsonValue::String(l), JsonValue::String(r)) => Ok(l < r),
            (JsonValue::Number(l), JsonValue::Number(r)) => {
                Ok(l.as_f64().unwrap_or(0.0) < r.as_f64().unwrap_or(0.0))
            }
            (JsonValue::Bool(l), JsonValue::Bool(r)) => Ok(l < r), // false < true
            // Type coercion for mixed comparisons
            (JsonValue::String(s), JsonValue::Number(n)) => match s.parse::<f64>() {
                Ok(parsed) => Ok(parsed < n.as_f64().unwrap_or(0.0)),
                Err(_) => Err(SemanticCLIError::validation_error(
                    format!("Cannot compare string '{}' with number", s),
                    "Use compatible types for comparison",
                    ErrorCode::E400,
                )),
            },
            (JsonValue::Number(n), JsonValue::String(s)) => match s.parse::<f64>() {
                Ok(parsed) => Ok(n.as_f64().unwrap_or(0.0) < parsed),
                Err(_) => Err(SemanticCLIError::validation_error(
                    format!("Cannot compare number with string '{}'", s),
                    "Use compatible types for comparison",
                    ErrorCode::E400,
                )),
            },
            _ => Err(SemanticCLIError::validation_error(
                format!("Cannot compare {:?} with {:?}", left, right),
                "Use compatible types for comparison",
                ErrorCode::E400,
            )),
        }
    }

    /// Evaluate complex filter with logical operations (for future use)
    pub fn evaluate_logical_filter(
        &mut self,
        left_filter: &FilterExpression,
        operator: LogicalOperator,
        right_filter: &FilterExpression,
        items: &[JsonValue],
    ) -> Result<FilterResult> {
        let start_time = Instant::now();

        let mut matched_items = Vec::new();
        let total_evaluated = items.len();

        for item in items {
            let left_result = self.evaluate_filter_for_item(left_filter, item)?;
            let right_result = self.evaluate_filter_for_item(right_filter, item)?;

            let combined_result = match operator {
                LogicalOperator::And => left_result && right_result,
                LogicalOperator::Or => left_result || right_result,
                LogicalOperator::Not => {
                    // NOT operator should only have one operand (left_result)
                    !left_result
                }
            };

            if combined_result {
                matched_items.push(item.clone());
            }
        }

        let matched_count = matched_items.len();
        let evaluation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(FilterResult {
            items: matched_items,
            total_evaluated,
            matched_count,
            evaluation_time_ms,
            normalized: left_filter.normalized && right_filter.normalized,
        })
    }
}

impl Default for FilterEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterResult {
    /// Create empty filter result
    pub fn empty(evaluation_time_ms: u64) -> Self {
        Self {
            items: Vec::new(),
            total_evaluated: 0,
            matched_count: 0,
            evaluation_time_ms,
            normalized: false,
        }
    }

    /// Get filter efficiency (matched / total)
    pub fn efficiency(&self) -> f64 {
        if self.total_evaluated == 0 {
            0.0
        } else {
            self.matched_count as f64 / self.total_evaluated as f64
        }
    }

    /// Format result summary
    pub fn summary(&self) -> String {
        format!(
            "Filtered {} items to {} matches in {}ms (efficiency: {:.1}%)",
            self.total_evaluated,
            self.matched_count,
            self.evaluation_time_ms,
            self.efficiency() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_data() -> Vec<JsonValue> {
        vec![
            json!({"id": "user_001", "name": "Alice", "age": 28, "active": true}),
            json!({"id": "user_002", "name": "Bob", "age": 34, "active": true}),
            json!({"id": "user_003", "name": "Carol", "age": 29, "active": false}),
            json!({"id": "user_004", "name": "David", "age": 42, "active": true}),
        ]
    }

    #[test]
    fn test_filter_evaluator_creation() {
        let evaluator = FilterEvaluator::new();
        // Gate C placeholder registers should be initialized
        assert_eq!(evaluator._next_register_placeholder, 1);
        assert!(evaluator._registers_placeholder.is_empty());
    }

    #[test]
    fn test_string_equality_filter() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "name".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("Alice".to_string())),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 1);
        assert_eq!(filter_result.total_evaluated, 4);
        assert_eq!(filter_result.items[0]["name"], "Alice");
    }

    #[test]
    fn test_number_comparison_filter() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "age".to_string(),
            ComparisonOp::GreaterThan,
            OperandRef::Literal(Value::Number(30.0)),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 2); // Bob (34) and David (42)

        for item in &filter_result.items {
            let age = item["age"].as_f64().unwrap();
            assert!(age > 30.0);
        }
    }

    #[test]
    fn test_boolean_filter() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "active".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::Boolean(true)),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 3); // Alice, Bob, David

        for item in &filter_result.items {
            assert_eq!(item["active"].as_bool().unwrap(), true);
        }
    }

    #[test]
    fn test_not_equal_filter() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "name".to_string(),
            ComparisonOp::NotEqual,
            OperandRef::Literal(Value::String("Alice".to_string())),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 3); // Bob, Carol, David

        for item in &filter_result.items {
            assert_ne!(item["name"].as_str().unwrap(), "Alice");
        }
    }

    #[test]
    fn test_less_than_or_equal_filter() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "age".to_string(),
            ComparisonOp::LessThanOrEqual,
            OperandRef::Literal(Value::Number(29.0)),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 2); // Alice (28) and Carol (29)

        for item in &filter_result.items {
            let age = item["age"].as_f64().unwrap();
            assert!(age <= 29.0);
        }
    }

    #[test]
    fn test_field_not_found() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "nonexistent_field".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("test".to_string())),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 0); // No matches (null != "test")
    }

    #[test]
    fn test_temp_register_not_allowed() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let filter = FilterExpression::new(
            "temp_register_field".to_string(), // Dummy field name for temp register test
            ComparisonOp::Equal,
            OperandRef::TempRegister(1), // This should cause error
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_err());

        if let Err(SemanticCLIError::ValidationError { .. }) = result {
            // Expected validation error
        } else {
            panic!("Expected ValidationError for temp register in filter");
        }
    }

    #[test]
    fn test_normalized_filter_not_allowed() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let mut filter = FilterExpression::new(
            "name".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("Alice".to_string())),
        );
        filter.normalized = true; // Set normalized flag

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_err());

        if let Err(SemanticCLIError::ValidationError { .. }) = result {
            // Expected validation error
        } else {
            panic!("Expected ValidationError for normalized filter");
        }
    }

    #[test]
    fn test_logical_filter_and() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let left_filter = FilterExpression::new(
            "active".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::Boolean(true)),
        );

        let right_filter = FilterExpression::new(
            "age".to_string(),
            ComparisonOp::GreaterThan,
            OperandRef::Literal(Value::Number(30.0)),
        );

        let result = evaluator.evaluate_logical_filter(
            &left_filter,
            LogicalOperator::And,
            &right_filter,
            &data,
        );
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 2); // Bob and David (active AND age > 30)

        for item in &filter_result.items {
            assert_eq!(item["active"].as_bool().unwrap(), true);
            assert!(item["age"].as_f64().unwrap() > 30.0);
        }
    }

    #[test]
    fn test_logical_filter_or() {
        let mut evaluator = FilterEvaluator::new();
        let data = create_test_data();

        let left_filter = FilterExpression::new(
            "name".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("Alice".to_string())),
        );

        let right_filter = FilterExpression::new(
            "name".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("Carol".to_string())),
        );

        let result = evaluator.evaluate_logical_filter(
            &left_filter,
            LogicalOperator::Or,
            &right_filter,
            &data,
        );
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 2); // Alice OR Carol

        let names: Vec<&str> = filter_result
            .items
            .iter()
            .map(|item| item["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Carol"));
    }

    #[test]
    fn test_type_coercion_string_number() {
        let mut evaluator = FilterEvaluator::new();
        let data = vec![
            json!({"id": "1", "value": "42"}),
            json!({"id": "2", "value": "not_a_number"}),
        ];

        let filter = FilterExpression::new(
            "value".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::Number(42.0)),
        );

        let result = evaluator.evaluate_filter(&filter, &data);
        assert!(result.is_ok());

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 1); // "42" == 42.0
        assert_eq!(filter_result.items[0]["id"], "1");
    }

    #[test]
    fn test_filter_result_efficiency() {
        let filter_result = FilterResult {
            items: vec![json!({"test": true})],
            total_evaluated: 10,
            matched_count: 2,
            evaluation_time_ms: 5,
            normalized: false,
        };

        assert_eq!(filter_result.efficiency(), 0.2); // 2/10 = 0.2
        assert!(filter_result.summary().contains("20.0%"));
    }

    #[test]
    fn test_empty_filter_result() {
        let result = FilterResult::empty(10);
        assert_eq!(result.matched_count, 0);
        assert_eq!(result.total_evaluated, 0);
        assert_eq!(result.evaluation_time_ms, 10);
        assert_eq!(result.efficiency(), 0.0);
    }

    #[test]
    fn test_filter_performance() {
        let mut evaluator = FilterEvaluator::new();

        // Create larger dataset
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push(json!({
                "id": format!("item_{}", i),
                "value": i,
                "active": i % 2 == 0
            }));
        }

        let filter = FilterExpression::new(
            "active".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::Boolean(true)),
        );

        let start = Instant::now();
        let result = evaluator.evaluate_filter(&filter, &data);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 100); // Performance target: < 100ms

        let filter_result = result.unwrap();
        assert_eq!(filter_result.matched_count, 500); // Half should be active
        assert!(filter_result.evaluation_time_ms < 100);
    }
}
