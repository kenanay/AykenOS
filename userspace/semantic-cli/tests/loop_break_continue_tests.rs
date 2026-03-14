//! Break/Continue Control Flow Tests - Phase 2.3
//!
//! This module tests the break and continue control flow functionality
//! implemented in Phase 2.3 of the D3 Loop Support Design.

use semantic_cli::bcib::{LoopConfig, LoopID, LoopInstruction, LoopRange, Value, ValueType};
use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::loop_engine::{LoopBodyFn, LoopBodyResult, LoopExecutor};
use semantic_cli::types::SourceLocation;

fn create_test_for_loop() -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 5, 1), // 0, 1, 2, 3, 4
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

#[test]
fn test_break_control_flow() {
    let mut executor = LoopExecutor::new();
    let instruction = create_test_for_loop();

    // Body function that breaks after 2 iterations
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Number(acc) = accumulator {
            let new_acc = acc + iteration as f64;
            if iteration >= 2 {
                // Break after 2 iterations (0, 1, 2)
                Ok(LoopBodyResult::Break(Value::Number(new_acc)))
            } else {
                Ok(LoopBodyResult::Normal(Value::Number(new_acc)))
            }
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_break());
    assert_eq!(result.get_iterations_completed(), 3); // 0, 1, 2 (break iteration counted)

    // Accumulator should be 0+0 + 1+1 + 2+2 = 3 (0 + 1 + 2)
    if let Some(Value::Number(final_sum)) = result.get_accumulator() {
        assert_eq!(*final_sum, 3.0);
    } else {
        panic!("Expected number accumulator");
    }
}

#[test]
fn test_continue_control_flow() {
    let mut executor = LoopExecutor::new();
    let instruction = create_test_for_loop();

    // Body function that continues (skips) on even iterations
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Number(acc) = accumulator {
            if iteration % 2 == 0 {
                // Continue on even iterations (skip adding to accumulator)
                Ok(LoopBodyResult::Continue(Value::Number(*acc)))
            } else {
                // Normal execution on odd iterations
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            }
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // All iterations completed

    // Accumulator should only include odd iterations: 0 + 1 + 3 = 4
    if let Some(Value::Number(final_sum)) = result.get_accumulator() {
        assert_eq!(*final_sum, 4.0);
    } else {
        panic!("Expected number accumulator");
    }
}

#[test]
fn test_break_continue_budget_accounting() {
    let mut executor = LoopExecutor::new();
    let mut instruction = create_test_for_loop();

    // Set budget that should be exceeded with break/continue costs
    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.budget_timeout = 8; // Should allow 3 iterations + break/continue costs
    }

    // Body function that uses break and continue
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Number(acc) = accumulator {
            match iteration {
                0 => Ok(LoopBodyResult::Continue(Value::Number(*acc))), // Continue costs 1 budget
                1 => Ok(LoopBodyResult::Normal(Value::Number(acc + 1.0))), // Normal costs 1 budget
                2 => Ok(LoopBodyResult::Break(Value::Number(acc + 2.0))), // Break costs 1 budget
                _ => Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                ))),
            }
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    // Should break successfully (budget accounting working correctly)
    assert!(result.is_break());
    assert_eq!(result.get_iterations_completed(), 3); // 0 (continue), 1 (normal), 2 (break)

    // Accumulator should be 0 + 1 + 2 = 3
    if let Some(Value::Number(final_sum)) = result.get_accumulator() {
        assert_eq!(*final_sum, 3.0);
    } else {
        panic!("Expected number accumulator");
    }
}

#[test]
fn test_break_continue_iteration_counting() {
    let mut executor = LoopExecutor::new();
    let instruction = create_test_for_loop();

    // Body function that tests iteration counting with break/continue
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Number(acc) = accumulator {
            match iteration {
                0 => Ok(LoopBodyResult::Continue(Value::Number(acc + 100.0))), // Continue: iteration 0 counted
                1 => Ok(LoopBodyResult::Normal(Value::Number(acc + 200.0))), // Normal: iteration 1 counted
                2 => Ok(LoopBodyResult::Break(Value::Number(acc + 300.0))), // Break: iteration 2 counted
                _ => Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                ))),
            }
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_break());
    // Constitutional rule: both break and continue count toward completed iterations
    assert_eq!(result.get_iterations_completed(), 3); // 0 (continue), 1 (normal), 2 (break)

    // Accumulator should be 0 + 100 + 200 + 300 = 600
    if let Some(Value::Number(final_sum)) = result.get_accumulator() {
        assert_eq!(*final_sum, 600.0);
    } else {
        panic!("Expected number accumulator");
    }
}
