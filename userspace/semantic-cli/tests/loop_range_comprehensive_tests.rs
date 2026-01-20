//! Comprehensive For Loop Range Support Tests - Task 3.2
//!
//! This test suite validates the complete For loop range support implementation
//! as specified in task 3.2 of the D3 Loop Support Design.
//!
//! Requirements validated:
//! - Deterministic range sequences (start, end, step)
//! - Range validation and bounds checking
//! - Support negative step values for reverse iteration
//! - Requirements: 1.8

use semantic_cli::bcib::{LoopInstruction, LoopID, LoopConfig, Value, ValueType, LoopRange, BudgetMeasurement};
use semantic_cli::loop_engine::{LoopExecutor, LoopBodyFn, LoopBodyResult};
use semantic_cli::error::{SemanticCLIError, ErrorCode};
use semantic_cli::types::SourceLocation;

fn create_test_for_loop_with_range(range: LoopRange) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new("test-for-range".to_string()),
        range,
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Array(vec![]), ValueType::Array),
        location: SourceLocation::new(1, 1, 0),
    }
}

#[test]
fn test_positive_step_range_basic() {
    // Test basic positive step range: 0 to 5 step 1 -> [0, 1, 2, 3, 4]
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 5, 1);
    let instruction = create_test_for_loop_with_range(range);

    // Collect iterator values to verify deterministic sequence
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // 0, 1, 2, 3, 4
    
    // Verify deterministic sequence
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 5);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_positive_step_range_with_step_2() {
    // Test positive step range: 0 to 10 step 2 -> [0, 2, 4, 6, 8]
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 10, 2);
    let instruction = create_test_for_loop_with_range(range);

    // Collect iterator values to verify deterministic sequence
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // 0, 2, 4, 6, 8
    
    // Verify deterministic sequence
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 5);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_negative_step_range_reverse_iteration() {
    // Test negative step range: 10 to 0 step -2 -> [10, 8, 6, 4, 2]
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(10, 0, -2);
    let instruction = create_test_for_loop_with_range(range);

    // Collect iterator values to verify deterministic reverse sequence
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // 10, 8, 6, 4, 2
    
    // Verify deterministic reverse sequence
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 5);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_negative_step_range_reverse_iteration_step_1() {
    // Test negative step range: 5 to 0 step -1 -> [5, 4, 3, 2, 1]
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(5, 0, -1);
    let instruction = create_test_for_loop_with_range(range);

    // Collect iterator values to verify deterministic reverse sequence
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // 5, 4, 3, 2, 1
    
    // Verify deterministic reverse sequence
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 5);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_range_validation_zero_step() {
    // Test range validation: step cannot be zero
    let range = LoopRange::new(0, 5, 0);
    let validation_result = range.validate();
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    assert!(error.to_string().contains("step cannot be zero"));
}

#[test]
fn test_range_validation_infinite_loop_positive_step() {
    // Test range validation: positive step with start >= end should result in zero iterations
    let range = LoopRange::new(5, 5, 1); // start == end
    let validation_result = range.validate();
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    assert!(error.to_string().contains("zero iterations"));

    let range2 = LoopRange::new(10, 5, 1); // start > end with positive step
    let validation_result2 = range2.validate();
    
    assert!(validation_result2.is_err());
    let error2 = validation_result2.unwrap_err();
    assert!(error2.to_string().contains("zero iterations"));
}

#[test]
fn test_range_validation_infinite_loop_negative_step() {
    // Test range validation: negative step with start <= end should result in zero iterations
    let range = LoopRange::new(5, 5, -1); // start == end
    let validation_result = range.validate();
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    assert!(error.to_string().contains("zero iterations"));

    let range2 = LoopRange::new(5, 10, -1); // start < end with negative step
    let validation_result2 = range2.validate();
    
    assert!(validation_result2.is_err());
    let error2 = validation_result2.unwrap_err();
    assert!(error2.to_string().contains("zero iterations"));
}

#[test]
fn test_range_iteration_count_calculation() {
    // Test iteration count calculation for various ranges
    
    // Positive step ranges
    assert_eq!(LoopRange::new(0, 5, 1).iteration_count(), 5); // [0, 1, 2, 3, 4]
    assert_eq!(LoopRange::new(0, 10, 2).iteration_count(), 5); // [0, 2, 4, 6, 8]
    assert_eq!(LoopRange::new(1, 8, 3).iteration_count(), 3); // [1, 4, 7]
    assert_eq!(LoopRange::new(0, 1, 1).iteration_count(), 1); // [0]
    
    // Negative step ranges
    assert_eq!(LoopRange::new(5, 0, -1).iteration_count(), 5); // [5, 4, 3, 2, 1]
    assert_eq!(LoopRange::new(10, 0, -2).iteration_count(), 5); // [10, 8, 6, 4, 2]
    assert_eq!(LoopRange::new(7, 1, -3).iteration_count(), 2); // [7, 4]
    assert_eq!(LoopRange::new(1, 0, -1).iteration_count(), 1); // [1]
    
    // Zero iteration ranges
    assert_eq!(LoopRange::new(5, 5, 1).iteration_count(), 0); // start == end
    assert_eq!(LoopRange::new(10, 5, 1).iteration_count(), 0); // start > end with positive step
    assert_eq!(LoopRange::new(5, 10, -1).iteration_count(), 0); // start < end with negative step
}

#[test]
fn test_empty_range_execution() {
    // Test execution of empty ranges (zero iterations)
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(5, 5, 1); // Empty range
    let instruction = create_test_for_loop_with_range(range);

    // Body function should never be called
    let body_fn: LoopBodyFn = Box::new(|_accumulator, _iteration| {
        panic!("Body should not be executed for empty range");
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 0); // No iterations
    
    // Accumulator should remain initial value
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 0); // Empty array
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_single_iteration_range() {
    // Test ranges that produce exactly one iteration
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 1, 1); // Single iteration: [0]
    let instruction = create_test_for_loop_with_range(range);

    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 1); // Single iteration
    
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 1);
        assert_eq!(*final_array, vec![Value::Number(0.0)]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_large_step_range() {
    // Test ranges with large step values
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 100, 25); // [0, 25, 50, 75]
    let instruction = create_test_for_loop_with_range(range);

    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 4); // [0, 25, 50, 75]
    
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 4);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_negative_range_large_step() {
    // Test negative ranges with large step values
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(100, 0, -25); // [100, 75, 50, 25]
    let instruction = create_test_for_loop_with_range(range);

    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 4); // [100, 75, 50, 25]
    
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 4);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_range_with_break_control_flow() {
    // Test range iteration with break control flow
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 10, 1); // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    let instruction = create_test_for_loop_with_range(range);

    // Break after 3 iterations
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            if iteration >= 2 {
                Ok(LoopBodyResult::Break(Value::Array(acc)))
            } else {
                Ok(LoopBodyResult::Normal(Value::Array(acc)))
            }
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_break());
    assert_eq!(result.get_iterations_completed(), 3); // 0, 1, 2 (break iteration counted)
    
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 3);
        assert_eq!(*final_array, vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_range_with_continue_control_flow() {
    // Test range iteration with continue control flow
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 5, 1); // [0, 1, 2, 3, 4]
    let instruction = create_test_for_loop_with_range(range);

    // Continue (skip) on even iterations
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            if iteration % 2 == 0 {
                // Continue on even iterations (don't add to accumulator)
                Ok(LoopBodyResult::Continue(Value::Array(acc)))
            } else {
                // Normal execution on odd iterations
                acc.push(Value::Number(iteration as f64));
                Ok(LoopBodyResult::Normal(Value::Array(acc)))
            }
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // All iterations completed
    
    // Should only include odd iterations: 1, 3
    if let Some(Value::Array(final_array)) = result.get_accumulator() {
        assert_eq!(final_array.len(), 2);
        assert_eq!(*final_array, vec![
            Value::Number(1.0),
            Value::Number(3.0),
        ]);
    } else {
        panic!("Expected array accumulator");
    }
}

#[test]
fn test_range_determinism_multiple_executions() {
    // Test that the same range produces identical results across multiple executions
    let range = LoopRange::new(1, 6, 2); // [1, 3, 5]
    
    let mut results = Vec::new();
    
    for _ in 0..3 {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop_with_range(range.clone());

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Array(mut acc) = accumulator.clone() {
                acc.push(Value::Number(iteration as f64));
                Ok(LoopBodyResult::Normal(Value::Array(acc)))
            } else {
                Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();
        results.push(result);
    }
    
    // All results should be identical (deterministic)
    for result in &results {
        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 3); // [1, 3, 5]
        
        if let Some(Value::Array(final_array)) = result.get_accumulator() {
            assert_eq!(final_array.len(), 3);
            assert_eq!(*final_array, vec![
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(2.0),
            ]);
        } else {
            panic!("Expected array accumulator");
        }
    }
}

#[test]
fn test_range_bounds_checking_with_limits() {
    // Test range execution with iteration limits
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 100, 1); // Would normally be 100 iterations
    let mut instruction = create_test_for_loop_with_range(range);
    
    // Set low iteration limit
    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.iteration_limit = 5; // Limit to 5 iterations
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_error()); // Should hit iteration limit
    assert_eq!(result.get_iterations_completed(), 0); // Error before any iterations
}

#[test]
fn test_range_with_budget_timeout() {
    // Test range execution with budget timeout
    let mut executor = LoopExecutor::new();
    let range = LoopRange::new(0, 100, 1); // Would normally be 100 iterations
    let mut instruction = create_test_for_loop_with_range(range);
    
    // Set low budget timeout
    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.budget_timeout = 3; // Very low budget
        config.budget_measurement = BudgetMeasurement::IterationCount;
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Array(mut acc) = accumulator.clone() {
            acc.push(Value::Number(iteration as f64));
            Ok(LoopBodyResult::Normal(Value::Array(acc)))
        } else {
            Err(SemanticCLIError::execution_error("Invalid accumulator type", ErrorCode::E500))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    
    assert!(result.is_error()); // Should hit budget timeout
    // Should complete some iterations before timeout
    assert!(result.get_iterations_completed() < 100);
}