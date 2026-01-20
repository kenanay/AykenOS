//! Loop Accumulator Pattern - Constitutional Alignment (Phase 0.5)
//!
//! This module implements the accumulator pattern for loop state management:
//! immutable context + mutable accumulator with type safety validation.
//!
//! # Constitutional Guarantees
//!
//! - Type safety across iterations
//! - Multiple accumulator support with independent types
//! - Explicit initial values required
//! - POST-INCREMENT state capture for partial results

use crate::bcib::{Value, ValueType};
use crate::error::{Result, SemanticCLIError, ErrorCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Single loop accumulator with type safety
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopAccumulator {
    /// Current accumulator value
    pub value: Value,
    /// Expected type for validation
    pub expected_type: ValueType,
    /// Accumulator name/identifier
    pub name: String,
}

impl LoopAccumulator {
    /// Create a new loop accumulator
    pub fn new(name: String, initial_value: Value) -> Self {
        let expected_type = initial_value.value_type();
        Self {
            value: initial_value,
            expected_type,
            name,
        }
    }

    /// Update the accumulator value with type validation
    pub fn update(&mut self, new_value: Value) -> Result<()> {
        if new_value.value_type() != self.expected_type {
            return Err(SemanticCLIError::validation_error(
                format!(
                    "Accumulator '{}' type changed from {:?} to {:?}",
                    self.name,
                    self.expected_type,
                    new_value.value_type()
                ),
                "Maintain consistent accumulator type across iterations",
                ErrorCode::E300,
            ));
        }

        self.value = new_value;
        Ok(())
    }

    /// Get the current value
    pub fn get_value(&self) -> &Value {
        &self.value
    }

    /// Get the expected type
    pub fn get_expected_type(&self) -> ValueType {
        self.expected_type
    }

    /// Validate this accumulator
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(SemanticCLIError::validation_error(
                "Accumulator name cannot be empty",
                "Provide a valid accumulator name",
                ErrorCode::E300,
            ));
        }

        self.value.validate()?;

        if self.value.value_type() != self.expected_type {
            return Err(SemanticCLIError::validation_error(
                format!(
                    "Accumulator '{}' value type {:?} does not match expected type {:?}",
                    self.name,
                    self.value.value_type(),
                    self.expected_type
                ),
                "Ensure value type matches expected type",
                ErrorCode::E300,
            ));
        }

        Ok(())
    }
}

/// Multi-accumulator pattern for complex loop state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorPattern {
    /// Map of accumulator name to accumulator
    accumulators: HashMap<String, LoopAccumulator>,
}

impl AccumulatorPattern {
    /// Create a new accumulator pattern
    pub fn new() -> Self {
        Self {
            accumulators: HashMap::new(),
        }
    }

    /// Add an accumulator with explicit initial value
    pub fn add_accumulator(&mut self, name: String, initial_value: Value) -> Result<()> {
        if self.accumulators.contains_key(&name) {
            return Err(SemanticCLIError::validation_error(
                format!("Accumulator '{}' already exists", name),
                "Use unique accumulator names",
                ErrorCode::E300,
            ));
        }

        let accumulator = LoopAccumulator::new(name.clone(), initial_value);
        self.accumulators.insert(name, accumulator);
        Ok(())
    }

    /// Update an accumulator value with type validation
    pub fn update_accumulator(&mut self, name: &str, new_value: Value) -> Result<()> {
        let accumulator = self.accumulators.get_mut(name)
            .ok_or_else(|| SemanticCLIError::validation_error(
                format!("Accumulator '{}' not found", name),
                "Ensure accumulator exists before updating",
                ErrorCode::E300,
            ))?;

        accumulator.update(new_value)
    }

    /// Get an accumulator value
    pub fn get_accumulator(&self, name: &str) -> Result<&Value> {
        let accumulator = self.accumulators.get(name)
            .ok_or_else(|| SemanticCLIError::validation_error(
                format!("Accumulator '{}' not found", name),
                "Ensure accumulator exists",
                ErrorCode::E300,
            ))?;

        Ok(accumulator.get_value())
    }

    /// Get all accumulator names
    pub fn get_accumulator_names(&self) -> Vec<String> {
        self.accumulators.keys().cloned().collect()
    }

    /// Get the expected type for an accumulator
    pub fn get_expected_type(&self, name: &str) -> Result<ValueType> {
        let accumulator = self.accumulators.get(name)
            .ok_or_else(|| SemanticCLIError::validation_error(
                format!("Accumulator '{}' not found", name),
                "Ensure accumulator exists",
                ErrorCode::E300,
            ))?;

        Ok(accumulator.get_expected_type())
    }

    /// Check if an accumulator exists
    pub fn has_accumulator(&self, name: &str) -> bool {
        self.accumulators.contains_key(name)
    }

    /// Get the number of accumulators
    pub fn len(&self) -> usize {
        self.accumulators.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.accumulators.is_empty()
    }

    /// Validate all accumulators
    pub fn validate(&self) -> Result<()> {
        for accumulator in self.accumulators.values() {
            accumulator.validate()?;
        }
        Ok(())
    }

    /// Get all accumulator values as a map (for partial results)
    pub fn get_all_values(&self) -> HashMap<String, Value> {
        self.accumulators.iter()
            .map(|(name, acc)| (name.clone(), acc.value.clone()))
            .collect()
    }

    /// Create from a map of initial values
    pub fn from_initial_values(initial_values: HashMap<String, Value>) -> Result<Self> {
        let mut pattern = Self::new();
        
        for (name, value) in initial_values {
            pattern.add_accumulator(name, value)?;
        }
        
        Ok(pattern)
    }
}

impl Default for AccumulatorPattern {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{Value, ValueType};

    #[test]
    fn test_single_accumulator() {
        let mut acc = LoopAccumulator::new(
            "counter".to_string(),
            Value::Number(0.0)
        );

        assert_eq!(acc.get_value(), &Value::Number(0.0));
        assert_eq!(acc.get_expected_type(), ValueType::Number);

        // Valid update
        assert!(acc.update(Value::Number(42.0)).is_ok());
        assert_eq!(acc.get_value(), &Value::Number(42.0));

        // Invalid type update
        assert!(acc.update(Value::String("invalid".to_string())).is_err());
    }

    #[test]
    fn test_accumulator_validation() {
        let acc = LoopAccumulator::new(
            "test".to_string(),
            Value::Boolean(true)
        );
        assert!(acc.validate().is_ok());

        // Test empty name
        let invalid_acc = LoopAccumulator::new(
            "".to_string(),
            Value::Boolean(true)
        );
        assert!(invalid_acc.validate().is_err());
    }

    #[test]
    fn test_multi_accumulator_pattern() {
        let mut pattern = AccumulatorPattern::new();

        // Add multiple accumulators with different types
        assert!(pattern.add_accumulator("counter".to_string(), Value::Number(0.0)).is_ok());
        assert!(pattern.add_accumulator("flag".to_string(), Value::Boolean(false)).is_ok());
        assert!(pattern.add_accumulator("message".to_string(), Value::String("".to_string())).is_ok());

        assert_eq!(pattern.len(), 3);
        assert!(!pattern.is_empty());

        // Test updates
        assert!(pattern.update_accumulator("counter", Value::Number(10.0)).is_ok());
        assert!(pattern.update_accumulator("flag", Value::Boolean(true)).is_ok());
        assert!(pattern.update_accumulator("message", Value::String("hello".to_string())).is_ok());

        // Test type validation
        assert!(pattern.update_accumulator("counter", Value::String("invalid".to_string())).is_err());

        // Test retrieval
        assert_eq!(pattern.get_accumulator("counter").unwrap(), &Value::Number(10.0));
        assert_eq!(pattern.get_accumulator("flag").unwrap(), &Value::Boolean(true));
        assert_eq!(pattern.get_accumulator("message").unwrap(), &Value::String("hello".to_string()));

        // Test expected types
        assert_eq!(pattern.get_expected_type("counter").unwrap(), ValueType::Number);
        assert_eq!(pattern.get_expected_type("flag").unwrap(), ValueType::Boolean);
        assert_eq!(pattern.get_expected_type("message").unwrap(), ValueType::String);
    }

    #[test]
    fn test_accumulator_pattern_errors() {
        let mut pattern = AccumulatorPattern::new();

        // Test duplicate accumulator
        assert!(pattern.add_accumulator("test".to_string(), Value::Number(0.0)).is_ok());
        assert!(pattern.add_accumulator("test".to_string(), Value::Number(1.0)).is_err());

        // Test non-existent accumulator
        assert!(pattern.get_accumulator("nonexistent").is_err());
        assert!(pattern.update_accumulator("nonexistent", Value::Number(0.0)).is_err());
        assert!(pattern.get_expected_type("nonexistent").is_err());
    }

    #[test]
    fn test_accumulator_pattern_from_initial_values() {
        let mut initial_values = HashMap::new();
        initial_values.insert("a".to_string(), Value::Number(1.0));
        initial_values.insert("b".to_string(), Value::Boolean(true));
        initial_values.insert("c".to_string(), Value::String("test".to_string()));

        let pattern = AccumulatorPattern::from_initial_values(initial_values).unwrap();

        assert_eq!(pattern.len(), 3);
        assert!(pattern.has_accumulator("a"));
        assert!(pattern.has_accumulator("b"));
        assert!(pattern.has_accumulator("c"));

        assert_eq!(pattern.get_accumulator("a").unwrap(), &Value::Number(1.0));
        assert_eq!(pattern.get_accumulator("b").unwrap(), &Value::Boolean(true));
        assert_eq!(pattern.get_accumulator("c").unwrap(), &Value::String("test".to_string()));
    }

    #[test]
    fn test_get_all_values() {
        let mut pattern = AccumulatorPattern::new();
        pattern.add_accumulator("x".to_string(), Value::Number(10.0)).unwrap();
        pattern.add_accumulator("y".to_string(), Value::Boolean(false)).unwrap();

        let all_values = pattern.get_all_values();
        assert_eq!(all_values.len(), 2);
        assert_eq!(all_values.get("x"), Some(&Value::Number(10.0)));
        assert_eq!(all_values.get("y"), Some(&Value::Boolean(false)));
    }
}