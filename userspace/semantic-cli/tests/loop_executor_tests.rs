//! Loop Executor Tests - Phase 2.2 Budget Timeout Enforcement
//!
//! This test file validates the budget timeout enforcement implementation
//! for the D3 Loop Support Design.

use semantic_cli::bcib::{
    BudgetMeasurement, LoopConfig, LoopID, LoopInstruction, LoopRange, Value, ValueType,
};
use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::loop_engine::{LoopBodyFn, LoopExecutor};
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

fn create_test_while_loop() -> LoopInstruction {
    LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: semantic_cli::bcib::OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

#[test]
fn test_budget_timeout_enforcement() {
    let mut executor = LoopExecutor::new();
    let mut instruction = create_test_for_loop();

    // Set very low budget timeout (3 iterations with IterationCount measurement)
    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.budget_timeout = 3; // Should timeout after 3 iterations
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, _| {
        if let Value::Number(acc) = accumulator {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(acc + 1.0),
            ))
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_error());
    // Should timeout before completing all 5 iterations
    assert!(result.get_iterations_completed() < 5);
}

#[test]
fn test_budget_measurement_iteration_count() {
    let mut executor = LoopExecutor::new();
    let mut instruction = create_test_for_loop();

    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.budget_measurement = BudgetMeasurement::IterationCount;
        config.budget_timeout = 3; // 3 iterations
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, _| {
        if let Value::Number(acc) = accumulator {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(acc + 1.0),
            ))
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    assert!(result.is_error()); // Should timeout after 3 iterations
}

#[test]
fn test_budget_measurement_instruction_count() {
    let mut executor = LoopExecutor::new();
    let mut instruction = create_test_for_loop();

    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.budget_measurement = BudgetMeasurement::InstructionCount { weight: 2 };
        config.budget_timeout = 5; // 5 instruction units (2.5 iterations)
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, _| {
        if let Value::Number(acc) = accumulator {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(acc + 1.0),
            ))
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    assert!(result.is_error()); // Should timeout after 2 iterations (4 instruction units)
}

#[test]
fn test_budget_measurement_hybrid() {
    let mut executor = LoopExecutor::new();
    let mut instruction = create_test_for_loop();

    if let LoopInstruction::For { config, .. } = &mut instruction {
        config.budget_measurement = BudgetMeasurement::Hybrid { multiplier: 1.5 };
        config.budget_timeout = 4; // 4 units (2.67 iterations)
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, _| {
        if let Value::Number(acc) = accumulator {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(acc + 1.0),
            ))
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();
    assert!(result.is_error()); // Should timeout after 2 iterations (3 units)
}

#[test]
fn test_for_loop_basic_execution_no_timeout() {
    let mut executor = LoopExecutor::new();
    let instruction = create_test_for_loop();

    // Simple accumulator: sum numbers
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Number(acc) = accumulator {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(acc + iteration as f64),
            ))
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_success());
    assert_eq!(result.get_iterations_completed(), 5); // 0,1,2,3,4

    // Sum should be 0+0 + 1+1 + 2+2 + 3+3 + 4+4 = 20
    if let Some(Value::Number(final_sum)) = result.get_accumulator() {
        assert_eq!(*final_sum, 10.0); // 0+1+2+3+4 = 10
    } else {
        panic!("Expected number accumulator");
    }
}

#[test]
fn test_iteration_limit_enforcement() {
    let mut executor = LoopExecutor::new();
    let mut instruction = create_test_while_loop();

    // Set very low iteration limit
    if let LoopInstruction::While { config, .. } = &mut instruction {
        config.iteration_limit = 3;
    }

    let body_fn: LoopBodyFn = Box::new(|accumulator, _| {
        if let Value::Number(acc) = accumulator {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(acc + 1.0),
            ))
        } else {
            Err(SemanticCLIError::execution_error(
                "Invalid accumulator type",
                ErrorCode::E500,
            ))
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_error());
    assert_eq!(result.get_iterations_completed(), 0); // Error before any iterations
}

#[test]
fn test_type_safety_enforcement() {
    let mut executor = LoopExecutor::new();
    let instruction = create_test_for_loop();

    // Body function that changes accumulator type (should fail)
    let body_fn: LoopBodyFn = Box::new(|_accumulator, iteration| {
        if iteration == 0 {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::Number(42.0),
            )) // Valid
        } else {
            Ok(semantic_cli::loop_engine::LoopBodyResult::Normal(
                Value::String("invalid".to_string()),
            )) // Type change - should fail
        }
    });

    let result = executor.execute_loop(&instruction, body_fn).unwrap();

    assert!(result.is_error());
    // Should complete first iteration, fail on second
    assert_eq!(result.get_iterations_completed(), 0);
}
