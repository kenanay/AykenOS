//! Test Support Utilities and Assertion Macros
//!
//! This module provides the core test support infrastructure including:
//! - Assertion macros for loop results (assert_loop_error!, assert_break!, assert_iterations!)
//! - Fingerprint testing utilities
//! - Helper functions for test data creation
//!
//! # Requirements Satisfied
//!
//! - Requirements 1.5: Standardized assertion macros
//! - Requirements 2.2: Helper assertion functions with detailed failure context
//! - Requirements 2.4: Expected vs actual values and iteration index in error messages

use crate::bcib::{LoopConfig, LoopID, LoopInstruction, LoopRange, Value, ValueType};
use crate::loop_engine::{ControlFlowResult, LoopError, LoopResult};
use crate::types::SourceLocation;

/// Assert that a LoopResult is an error matching the expected pattern
///
/// This macro provides detailed error reporting when the assertion fails,
/// including the expected error pattern and the actual result.
///
/// # Examples
///
/// ```rust
/// use semantic_cli::loop_engine::tests::support::assert_loop_error;
/// use semantic_cli::loop_engine::{LoopResult, LoopError};
///
/// let result = LoopResult::Error(LoopError::IterationLimitExceeded { limit: 100, completed: 100 });
/// assert_loop_error!(result, LoopError::IterationLimitExceeded { .. });
/// ```
#[macro_export]
macro_rules! assert_loop_error {
    ($result:expr, $expected_error:pat) => {
        match &$result {
            $crate::loop_engine::LoopResult::Error(error) => {
                if !matches!(error, $expected_error) {
                    panic!(
                        "Expected error pattern {}, but got error: {:?}\nFull result: {:?}",
                        stringify!($expected_error),
                        error,
                        $result
                    );
                }
            }
            other => {
                panic!(
                    "Expected error result matching pattern {}, but got: {:?}",
                    stringify!($expected_error),
                    other
                );
            }
        }
    };
    ($result:expr, $expected_error:pat, $context:expr) => {
        match &$result {
            $crate::loop_engine::LoopResult::Error(error) => {
                if !matches!(error, $expected_error) {
                    panic!(
                        "Expected error pattern {} in context '{}', but got error: {:?}\nFull result: {:?}",
                        stringify!($expected_error),
                        $context,
                        error,
                        $result
                    );
                }
            }
            other => {
                panic!(
                    "Expected error result matching pattern {} in context '{}', but got: {:?}",
                    stringify!($expected_error),
                    $context,
                    other
                );
            }
        }
    };
}

/// Assert that a LoopResult is a break with the expected value
///
/// This macro validates both that the result is a break and that the
/// accumulator value matches the expected value.
///
/// # Examples
///
/// ```rust
/// use semantic_cli::loop_engine::tests::support::assert_break;
/// use semantic_cli::loop_engine::LoopResult;
/// use semantic_cli::bcib::Value;
///
/// let result = LoopResult::break_result(Value::Number(42.0), 5);
/// assert_break!(result, Value::Number(42.0));
/// ```
#[macro_export]
macro_rules! assert_break {
    ($result:expr, $expected_value:expr) => {
        match &$result {
            $crate::loop_engine::LoopResult::ControlFlow(
                $crate::loop_engine::ControlFlowResult::Break { accumulator, .. }
            ) => {
                if accumulator != &$expected_value {
                    panic!(
                        "Expected break with value {:?}, but got break with value: {:?}\nFull result: {:?}",
                        $expected_value,
                        accumulator,
                        $result
                    );
                }
            }
            other => {
                panic!(
                    "Expected break result with value {:?}, but got: {:?}",
                    $expected_value,
                    other
                );
            }
        }
    };
    ($result:expr, $expected_value:expr, $expected_iterations:expr) => {
        match &$result {
            $crate::loop_engine::LoopResult::ControlFlow(
                $crate::loop_engine::ControlFlowResult::Break { accumulator, iterations_completed }
            ) => {
                if accumulator != &$expected_value {
                    panic!(
                        "Expected break with value {:?}, but got break with value: {:?}\nFull result: {:?}",
                        $expected_value,
                        accumulator,
                        $result
                    );
                }
                if iterations_completed != &$expected_iterations {
                    panic!(
                        "Expected break with {} iterations, but got {} iterations\nFull result: {:?}",
                        $expected_iterations,
                        iterations_completed,
                        $result
                    );
                }
            }
            other => {
                panic!(
                    "Expected break result with value {:?} and {} iterations, but got: {:?}",
                    $expected_value,
                    $expected_iterations,
                    other
                );
            }
        }
    };
}

/// Assert that a LoopResult has the expected number of completed iterations
///
/// This macro works with any LoopResult variant that tracks iteration count.
///
/// # Examples
///
/// ```rust
/// use semantic_cli::loop_engine::tests::support::assert_iterations;
/// use semantic_cli::loop_engine::LoopResult;
/// use semantic_cli::bcib::Value;
///
/// let result = LoopResult::success(Value::Number(100.0), 10);
/// assert_iterations!(result, 10);
/// ```
#[macro_export]
macro_rules! assert_iterations {
    ($result:expr, $expected_count:expr) => {
        let actual_count = $result.get_iterations_completed();
        if actual_count != $expected_count {
            panic!(
                "Expected {} iterations, but got {} iterations\nFull result: {:?}",
                $expected_count, actual_count, $result
            );
        }
    };
    ($result:expr, $expected_count:expr, $context:expr) => {
        let actual_count = $result.get_iterations_completed();
        if actual_count != $expected_count {
            panic!(
                "Expected {} iterations in context '{}', but got {} iterations\nFull result: {:?}",
                $expected_count, $context, actual_count, $result
            );
        }
    };
}

// Re-export macros for easier access
pub use assert_break;
pub use assert_iterations;
pub use assert_loop_error;

/// Fingerprint testing utilities
pub mod fingerprint {
    #[allow(unused_imports)]
    use super::*;

    /// Placeholder for enhanced fingerprint types (will be implemented in Phase 2)
    /// This provides the interface that will be used for fingerprint testing
    #[derive(Debug, Clone, PartialEq)]
    pub struct TestFingerprint {
        pub version: u8,
        pub shape_hash: [u8; 32],
        pub control_hash: [u8; 32],
        pub data_hash: [u8; 32],
        pub combined_hash: [u8; 32],
    }

    /// Mismatch types for fingerprint validation
    #[derive(Debug, Clone, PartialEq)]
    pub enum FingerprintMismatchType {
        Shape {
            field: String,
            expected: String,
            actual: String,
        },
        Control {
            decision_index: u64,
            expected: String,
            actual: String,
        },
        Data {
            transition_index: u64,
            expected: Vec<u8>,
            actual: Vec<u8>,
        },
        Combined {
            expected_hash: [u8; 32],
            actual_hash: [u8; 32],
        },
    }

    /// Create a test fingerprint with specified components
    pub fn create_test_fingerprint(
        version: u8,
        shape_data: &[u8],
        control_data: &[u8],
        data_data: &[u8],
    ) -> TestFingerprint {
        // Simple hash simulation for testing (will be replaced with BLAKE3 in Phase 2)
        let mut shape_hash = [0u8; 32];
        let mut control_hash = [0u8; 32];
        let mut data_hash = [0u8; 32];
        let mut combined_hash = [0u8; 32];

        // Fill with simple patterns for testing
        for (i, &byte) in shape_data.iter().enumerate() {
            if i < 32 {
                shape_hash[i] = byte;
            }
        }
        for (i, &byte) in control_data.iter().enumerate() {
            if i < 32 {
                control_hash[i] = byte;
            }
        }
        for (i, &byte) in data_data.iter().enumerate() {
            if i < 32 {
                data_hash[i] = byte;
            }
        }

        // Combined hash is XOR of all components for testing
        for i in 0..32 {
            combined_hash[i] = shape_hash[i] ^ control_hash[i] ^ data_hash[i];
        }

        TestFingerprint {
            version,
            shape_hash,
            control_hash,
            data_hash,
            combined_hash,
        }
    }

    /// Assert that two fingerprints have the expected mismatch type
    pub fn assert_fingerprint_mismatch(
        expected: &TestFingerprint,
        actual: &TestFingerprint,
        expected_mismatch_type: FingerprintMismatchType,
    ) {
        if expected == actual {
            panic!("Expected fingerprint mismatch, but fingerprints are identical");
        }

        // Determine actual mismatch type
        let actual_mismatch = if expected.shape_hash != actual.shape_hash {
            FingerprintMismatchType::Shape {
                field: "shape_hash".to_string(),
                expected: format!("{:?}", expected.shape_hash),
                actual: format!("{:?}", actual.shape_hash),
            }
        } else if expected.control_hash != actual.control_hash {
            FingerprintMismatchType::Control {
                decision_index: 0, // Simplified for testing
                expected: format!("{:?}", expected.control_hash),
                actual: format!("{:?}", actual.control_hash),
            }
        } else if expected.data_hash != actual.data_hash {
            FingerprintMismatchType::Data {
                transition_index: 0, // Simplified for testing
                expected: expected.data_hash.to_vec(),
                actual: actual.data_hash.to_vec(),
            }
        } else {
            FingerprintMismatchType::Combined {
                expected_hash: expected.combined_hash,
                actual_hash: actual.combined_hash,
            }
        };

        // Validate mismatch type matches expectation
        match (&expected_mismatch_type, &actual_mismatch) {
            (FingerprintMismatchType::Shape { .. }, FingerprintMismatchType::Shape { .. }) => {}
            (FingerprintMismatchType::Control { .. }, FingerprintMismatchType::Control { .. }) => {}
            (FingerprintMismatchType::Data { .. }, FingerprintMismatchType::Data { .. }) => {}
            (
                FingerprintMismatchType::Combined { .. },
                FingerprintMismatchType::Combined { .. },
            ) => {}
            _ => {
                panic!(
                    "Expected mismatch type {:?}, but got {:?}",
                    expected_mismatch_type, actual_mismatch
                );
            }
        }
    }
}

// Re-export fingerprint utilities
pub use fingerprint::{
    assert_fingerprint_mismatch, create_test_fingerprint, FingerprintMismatchType, TestFingerprint,
};

/// Helper function to extract iteration count from any LoopResult
pub fn extract_iteration_count(result: &LoopResult) -> u32 {
    result.get_iterations_completed()
}

/// Format a LoopResult for debug output with detailed information
pub fn format_loop_result_debug(result: &LoopResult) -> String {
    match result {
        LoopResult::Success {
            accumulator,
            iterations_completed,
        } => {
            format!(
                "Success(accumulator: {:?}, iterations: {})",
                accumulator, iterations_completed
            )
        }
        LoopResult::Partial(partial) => {
            format!(
                "Partial(accumulator: {:?}, iterations: {}, reason: {:?})",
                partial.accumulator, partial.iterations_completed, partial.termination_reason
            )
        }
        LoopResult::Error(error) => {
            format!("Error({:?})", error)
        }
        LoopResult::EnvironmentFault(fault) => {
            format!("EnvironmentFault({:?})", fault)
        }
        LoopResult::ControlFlow(control_flow) => match control_flow {
            ControlFlowResult::Break {
                accumulator,
                iterations_completed,
            } => {
                format!(
                    "Break(accumulator: {:?}, iterations: {})",
                    accumulator, iterations_completed
                )
            }
            ControlFlowResult::Continue {
                iterations_completed,
            } => {
                format!("Continue(iterations: {})", iterations_completed)
            }
        },
    }
}

/// Create a test loop configuration with specified parameters
pub fn create_test_loop_config(
    initial_accumulator: Value,
    accumulator_type: ValueType,
    iteration_limit: Option<u32>,
    budget_timeout: Option<u64>,
) -> LoopConfig {
    let mut config = LoopConfig::new(initial_accumulator, accumulator_type);

    if let Some(limit) = iteration_limit {
        config.iteration_limit = limit;
    }

    if let Some(budget) = budget_timeout {
        config.budget_timeout = budget;
    }

    config
}

/// Create a test For loop instruction with custom configuration
pub fn create_test_for_loop(
    loop_id: &str,
    start: i64,
    end: i64,
    step: i64,
    config: LoopConfig,
) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new(loop_id.to_string()),
        range: LoopRange::new(start, end, step),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config,
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Create a test While loop instruction with custom configuration
pub fn create_test_while_loop(
    loop_id: &str,
    condition_value: bool,
    config: LoopConfig,
) -> LoopInstruction {
    LoopInstruction::While {
        id: LoopID::new(loop_id.to_string()),
        condition: crate::bcib::OperandRef::Literal(Value::Boolean(condition_value)),
        body: "test-body".to_string(),
        config,
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Test scenario builders for common loop testing patterns
pub mod scenarios {
    use super::*;
    use crate::error::{ErrorCode, SemanticCLIError};
    use crate::loop_engine::{LoopBodyFn, LoopBodyResult};

    /// Create a simple accumulator body function that sums iteration indices
    pub fn create_sum_body_fn() -> LoopBodyFn {
        Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type for sum operation",
                    ErrorCode::E500,
                ))
            }
        })
    }

    /// Create a body function that breaks after a specified iteration
    pub fn create_break_after_body_fn(break_iteration: u32) -> LoopBodyFn {
        Box::new(move |accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                let new_acc = acc + iteration as f64;
                if iteration >= break_iteration {
                    Ok(LoopBodyResult::Break(Value::Number(new_acc)))
                } else {
                    Ok(LoopBodyResult::Normal(Value::Number(new_acc)))
                }
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type for break operation",
                    ErrorCode::E500,
                ))
            }
        })
    }

    /// Create a body function that continues (skips) on even iterations
    pub fn create_continue_even_body_fn() -> LoopBodyFn {
        Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                if iteration % 2 == 0 {
                    Ok(LoopBodyResult::Continue(Value::Number(*acc)))
                } else {
                    Ok(LoopBodyResult::Normal(Value::Number(
                        acc + iteration as f64,
                    )))
                }
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type for continue operation",
                    ErrorCode::E500,
                ))
            }
        })
    }

    /// Create a body function that causes a type error after specified iterations
    pub fn create_type_error_body_fn(error_iteration: u32) -> LoopBodyFn {
        Box::new(move |accumulator, iteration| {
            if iteration == error_iteration {
                // Return wrong type to trigger type error
                Ok(LoopBodyResult::Normal(Value::String(
                    "type_error".to_string(),
                )))
            } else if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;
    use crate::loop_engine::{LoopBodyResult, LoopResult};

    #[test]
    fn test_assert_loop_error_macro() {
        let result = LoopResult::Error(LoopError::IterationLimitExceeded {
            limit: 100,
            completed: 50,
        });

        // This should not panic
        assert_loop_error!(result, LoopError::IterationLimitExceeded { .. });
    }

    #[test]
    fn test_assert_break_macro() {
        let result = LoopResult::break_result(Value::Number(42.0), 5);

        // This should not panic
        assert_break!(result, Value::Number(42.0));
        assert_break!(result, Value::Number(42.0), 5);
    }

    #[test]
    fn test_assert_iterations_macro() {
        let result = LoopResult::success(Value::Number(100.0), 10);

        // This should not panic
        assert_iterations!(result, 10);
    }

    #[test]
    fn test_extract_iteration_count() {
        let success_result = LoopResult::success(Value::Number(42.0), 15);
        assert_eq!(extract_iteration_count(&success_result), 15);

        let break_result = LoopResult::break_result(Value::Number(10.0), 7);
        assert_eq!(extract_iteration_count(&break_result), 7);
    }

    #[test]
    fn test_format_loop_result_debug() {
        let success_result = LoopResult::success(Value::Number(42.0), 10);
        let debug_str = format_loop_result_debug(&success_result);
        assert!(debug_str.contains("Success"));
        assert!(debug_str.contains("42.0"));
        assert!(debug_str.contains("10"));

        let error_result = LoopResult::Error(LoopError::IterationLimitExceeded {
            limit: 100,
            completed: 50,
        });
        let debug_str = format_loop_result_debug(&error_result);
        assert!(debug_str.contains("Error"));
        assert!(debug_str.contains("IterationLimitExceeded"));
    }

    #[test]
    fn test_create_test_loop_config() {
        let config =
            create_test_loop_config(Value::Number(0.0), ValueType::Number, Some(500), Some(2000));

        assert_eq!(config.iteration_limit, 500);
        assert_eq!(config.budget_timeout, 2000);
    }

    #[test]
    fn test_create_test_fingerprint() {
        let fingerprint = create_test_fingerprint(1, &[1, 2, 3], &[4, 5, 6], &[7, 8, 9]);

        assert_eq!(fingerprint.version, 1);
        assert_eq!(fingerprint.shape_hash[0], 1);
        assert_eq!(fingerprint.control_hash[0], 4);
        assert_eq!(fingerprint.data_hash[0], 7);
        // Combined hash should be XOR: 1 ^ 4 ^ 7 = 2
        assert_eq!(fingerprint.combined_hash[0], 2);
    }

    #[test]
    fn test_fingerprint_mismatch_assertion() {
        let fp1 = create_test_fingerprint(1, &[1, 2, 3], &[4, 5, 6], &[7, 8, 9]);
        let fp2 = create_test_fingerprint(1, &[1, 2, 4], &[4, 5, 6], &[7, 8, 9]); // Different shape

        // This should not panic - we expect a shape mismatch
        assert_fingerprint_mismatch(
            &fp1,
            &fp2,
            FingerprintMismatchType::Shape {
                field: "shape_hash".to_string(),
                expected: "expected".to_string(),
                actual: "actual".to_string(),
            },
        );
    }

    #[test]
    fn test_scenario_builders() {
        // Test that scenario builders create valid functions
        let sum_fn = scenarios::create_sum_body_fn();
        let result = sum_fn(&Value::Number(10.0), 5);
        assert!(result.is_ok());

        let break_fn = scenarios::create_break_after_body_fn(3);
        let result = break_fn(&Value::Number(10.0), 3);
        assert!(result.is_ok());
        if let Ok(LoopBodyResult::Break(Value::Number(val))) = result {
            assert_eq!(val, 13.0); // 10 + 3
        } else {
            panic!("Expected break result");
        }
    }
}
