//! Loop Engine Test Support Infrastructure
//!
//! This module provides comprehensive test support infrastructure for the loop engine
//! architectural improvements. It includes assertion macros, fingerprint testing utilities,
//! and helper functions for creating test scenarios.
//!
//! # Test Support Structure
//!
//! - `support.rs` - Core test utilities and assertion macros
//! - `property_tests.rs` - Property-based tests for universal correctness
//! - Helper functions for creating test data and scenarios
//! - Fingerprint testing utilities for validation
//!
//! # Requirements Satisfied
//!
//! - Requirements 1.5: Test support submodule with standardized assertion macros
//! - Requirements 2.2: Helper assertion functions in tests::support module
//! - Requirements 2.4: Detailed failure context including expected vs actual values

#[cfg(test)]
pub mod architecture_preservation_tests;
#[cfg(test)]
pub mod property_tests;
pub mod support;

// Re-export core test utilities for easy access
pub use support::{
    assert_break,
    assert_fingerprint_mismatch,
    assert_iterations,
    // Assertion macros
    assert_loop_error,
    // Fingerprint testing utilities
    create_test_fingerprint,
    create_test_for_loop,
    // Helper functions
    create_test_loop_config,
    create_test_while_loop,
    extract_iteration_count,
    format_loop_result_debug,
};

use crate::bcib::{LoopConfig, LoopID, LoopInstruction, LoopRange, Value, ValueType};
use crate::loop_engine::{LoopError, LoopResult};
use crate::types::SourceLocation;

/// Test configuration constants
pub const DEFAULT_ITERATION_LIMIT: u32 = 1000;
pub const DEFAULT_BUDGET_TIMEOUT: u64 = 10000;
pub const TEST_LOOP_ID_PREFIX: &str = "test-loop";

/// Create a standard test loop configuration
pub fn create_standard_test_config() -> LoopConfig {
    LoopConfig::new(Value::Number(0.0), ValueType::Number)
}

/// Create a test For loop instruction with default settings
pub fn create_default_for_loop(start: i64, end: i64, step: i64) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new(format!(
            "{}-for-{}-{}-{}",
            TEST_LOOP_ID_PREFIX, start, end, step
        )),
        range: LoopRange::new(start, end, step),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: create_standard_test_config(),
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Create a test While loop instruction with default settings
pub fn create_default_while_loop(condition_value: bool) -> LoopInstruction {
    LoopInstruction::While {
        id: LoopID::new(format!("{}-while-{}", TEST_LOOP_ID_PREFIX, condition_value)),
        condition: crate::bcib::OperandRef::Literal(Value::Boolean(condition_value)),
        body: "test-body".to_string(),
        config: create_standard_test_config(),
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Test result validation utilities
pub mod validation {
    use super::*;

    /// Validate that a loop result matches expected success criteria
    pub fn validate_success_result(
        result: &LoopResult,
        expected_iterations: u32,
        expected_accumulator: Option<&Value>,
    ) -> Result<(), String> {
        if !result.is_success() {
            return Err(format!("Expected success result, got: {:?}", result));
        }

        let actual_iterations = result.get_iterations_completed();
        if actual_iterations != expected_iterations {
            return Err(format!(
                "Expected {} iterations, got {}",
                expected_iterations, actual_iterations
            ));
        }

        if let Some(expected_acc) = expected_accumulator {
            match result.get_accumulator() {
                Some(actual_acc) if actual_acc == expected_acc => Ok(()),
                Some(actual_acc) => Err(format!(
                    "Expected accumulator {:?}, got {:?}",
                    expected_acc, actual_acc
                )),
                None => Err("Expected accumulator value, got None".to_string()),
            }
        } else {
            Ok(())
        }
    }

    /// Validate that a loop result matches expected error criteria
    pub fn validate_error_result(
        result: &LoopResult,
        expected_error_pattern: fn(&LoopError) -> bool,
    ) -> Result<(), String> {
        match result {
            LoopResult::Error(error) => {
                if expected_error_pattern(error) {
                    Ok(())
                } else {
                    Err(format!("Error doesn't match expected pattern: {:?}", error))
                }
            }
            _ => Err(format!("Expected error result, got: {:?}", result)),
        }
    }

    /// Validate that a loop result matches expected break criteria
    pub fn validate_break_result(
        result: &LoopResult,
        expected_iterations: u32,
        expected_accumulator: Option<&Value>,
    ) -> Result<(), String> {
        if !result.is_break() {
            return Err(format!("Expected break result, got: {:?}", result));
        }

        let actual_iterations = result.get_iterations_completed();
        if actual_iterations != expected_iterations {
            return Err(format!(
                "Expected {} iterations, got {}",
                expected_iterations, actual_iterations
            ));
        }

        if let Some(expected_acc) = expected_accumulator {
            match result.get_accumulator() {
                Some(actual_acc) if actual_acc == expected_acc => Ok(()),
                Some(actual_acc) => Err(format!(
                    "Expected accumulator {:?}, got {:?}",
                    expected_acc, actual_acc
                )),
                None => Err("Expected accumulator value, got None".to_string()),
            }
        } else {
            Ok(())
        }
    }
}

/// Test data generation utilities
pub mod generators {
    use super::*;

    /// Generate a sequence of test values for accumulator testing
    pub fn generate_number_sequence(start: f64, count: usize, step: f64) -> Vec<Value> {
        (0..count)
            .map(|i| Value::Number(start + (i as f64 * step)))
            .collect()
    }

    /// Generate a sequence of test string values
    pub fn generate_string_sequence(prefix: &str, count: usize) -> Vec<Value> {
        (0..count)
            .map(|i| Value::String(format!("{}{}", prefix, i)))
            .collect()
    }

    /// Generate a sequence of test boolean values (alternating)
    pub fn generate_boolean_sequence(count: usize, start_with: bool) -> Vec<Value> {
        (0..count)
            .map(|i| Value::Boolean(if i % 2 == 0 { start_with } else { !start_with }))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_for_loop() {
        let loop_inst = create_default_for_loop(0, 5, 1);

        match loop_inst {
            LoopInstruction::For { range, .. } => {
                assert_eq!(range.start, 0);
                assert_eq!(range.end, 5);
                assert_eq!(range.step, 1);
            }
            _ => panic!("Expected For loop instruction"),
        }
    }

    #[test]
    fn test_create_default_while_loop() {
        let loop_inst = create_default_while_loop(true);

        match loop_inst {
            LoopInstruction::While { condition, .. } => match condition {
                crate::bcib::OperandRef::Literal(Value::Boolean(true)) => {}
                _ => panic!("Expected boolean true condition"),
            },
            _ => panic!("Expected While loop instruction"),
        }
    }

    #[test]
    fn test_validation_success_result() {
        let result = LoopResult::success(Value::Number(42.0), 10);

        // Valid success validation
        assert!(
            validation::validate_success_result(&result, 10, Some(&Value::Number(42.0))).is_ok()
        );

        // Invalid iteration count
        assert!(
            validation::validate_success_result(&result, 5, Some(&Value::Number(42.0))).is_err()
        );

        // Invalid accumulator
        assert!(
            validation::validate_success_result(&result, 10, Some(&Value::Number(100.0))).is_err()
        );
    }

    #[test]
    fn test_generators() {
        let numbers = generators::generate_number_sequence(1.0, 3, 2.0);
        assert_eq!(
            numbers,
            vec![Value::Number(1.0), Value::Number(3.0), Value::Number(5.0)]
        );

        let strings = generators::generate_string_sequence("test", 2);
        assert_eq!(
            strings,
            vec![
                Value::String("test0".to_string()),
                Value::String("test1".to_string())
            ]
        );

        let booleans = generators::generate_boolean_sequence(4, true);
        assert_eq!(
            booleans,
            vec![
                Value::Boolean(true),
                Value::Boolean(false),
                Value::Boolean(true),
                Value::Boolean(false)
            ]
        );
    }
}
