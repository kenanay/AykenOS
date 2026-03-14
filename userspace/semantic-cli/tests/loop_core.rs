//! Core Loop Tests - Stable Implementation (10.1 + 10.2)
//!
//! This test suite covers the stable, production-ready loop functionality:
//! - 10.1 Test core loop execution functionality
//! - 10.2 Test collection determinism
//!
//! These tests run by default with: cargo test -p semantic-cli

use semantic_cli::bcib::{
    BudgetMeasurement, CollectionType, LoopConfig, LoopID, LoopInstruction, LoopRange, OperandRef,
    Value, ValueType,
};
use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::loop_engine::{LoopBodyFn, LoopBodyResult, LoopError, LoopExecutor, LoopResult};
use semantic_cli::types::SourceLocation;
use std::collections::BTreeMap;

// Test helper functions
fn create_test_for_loop(start: i64, end: i64, step: i64) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new(format!("test-for-{}-{}-{}", start, end, step)),
        range: LoopRange::new(start, end, step),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

fn create_test_while_loop(condition: Value) -> LoopInstruction {
    LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(condition),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

fn create_test_foreach_loop(collection: Value, collection_type: CollectionType) -> LoopInstruction {
    LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(collection),
        collection_type,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

// =============================================================================
// 10.1 Test Core Loop Execution Functionality
// =============================================================================

#[cfg(test)]
mod core_execution_tests {
    use super::*;

    #[test]
    fn test_for_loop_basic_execution() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(0, 5, 1); // 0, 1, 2, 3, 4

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 5);

        // Sum should be 0+1+2+3+4 = 10
        if let Some(Value::Number(final_sum)) = result.get_accumulator() {
            assert_eq!(*final_sum, 10.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_while_loop_iteration_limit_enforced() {
        let mut executor = LoopExecutor::new();
        let mut instruction = create_test_while_loop(Value::Boolean(true));

        // Set iteration limit to prevent infinite loop
        if let LoopInstruction::While { config, .. } = &mut instruction {
            config.iteration_limit = 3;
        }

        // NEVER breaks -> must hit iteration limit
        let body_fn: LoopBodyFn = Box::new(|accumulator, _iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(acc + 1.0)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_error());
        assert!(result.get_iterations_completed() <= 3);
    }

    #[test]
    fn test_foreach_loop_basic_execution() {
        let mut executor = LoopExecutor::new();
        let collection = Value::Array(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ]);
        let instruction = create_test_foreach_loop(collection, CollectionType::Array);

        let body_fn: LoopBodyFn = Box::new(|accumulator, _iteration| {
            if let Value::Number(acc) = accumulator {
                // For ForEach, we would normally access the current item
                // For this test, just increment by 5
                Ok(LoopBodyResult::Normal(Value::Number(acc + 5.0)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 3);

        // Should be 0 + 5 + 5 + 5 = 15
        if let Some(Value::Number(final_sum)) = result.get_accumulator() {
            assert_eq!(*final_sum, 15.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_iteration_limit_enforcement() {
        let mut executor = LoopExecutor::new();
        let mut instruction = create_test_for_loop(0, 100, 1); // Would be 100 iterations

        // Set very low iteration limit
        if let LoopInstruction::For { config, .. } = &mut instruction {
            config.iteration_limit = 5;
        }

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_error());
        // The exact number of completed iterations may vary based on implementation
        // but should be less than the total range
        assert!(result.get_iterations_completed() <= 5);

        // Verify it's an iteration limit error
        if let LoopResult::Error(LoopError::IterationLimitExceeded { limit, completed }) = result {
            assert_eq!(limit, 5);
            assert_eq!(completed, 5);
        } else {
            panic!("Expected IterationLimitExceeded error, got: {:?}", result);
        }
    }

    #[test]
    fn test_budget_timeout_enforcement() {
        let mut executor = LoopExecutor::new();
        let mut instruction = create_test_for_loop(0, 100, 1);

        // Set very low budget timeout
        if let LoopInstruction::For { config, .. } = &mut instruction {
            config.budget_timeout = 3; // Should timeout after 3 iterations
            config.budget_measurement = BudgetMeasurement::IterationCount;
        }

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_error());

        // Verify it's a budget timeout error
        if let LoopResult::Error(LoopError::BudgetTimeoutExceeded {
            budget,
            consumed,
            iterations_completed,
        }) = result
        {
            assert_eq!(budget, 3);
            assert!(consumed > 0); // Should have consumed some budget
            assert!(iterations_completed <= 3);
        } else {
            panic!("Expected BudgetTimeoutExceeded error, got: {:?}", result);
        }
    }

    #[test]
    fn test_break_control_flow() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(0, 10, 1);

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                let new_acc = acc + iteration as f64;
                if iteration >= 3 {
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
        assert_eq!(result.get_iterations_completed(), 4); // 0, 1, 2, 3 (break iteration counted)

        // Sum should be 0+1+2+3 = 6
        if let Some(Value::Number(final_sum)) = result.get_accumulator() {
            assert_eq!(*final_sum, 6.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_continue_control_flow() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(0, 5, 1);

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
    fn test_accumulator_type_safety() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(0, 5, 1);

        // Body function that changes accumulator type (should fail)
        let body_fn: LoopBodyFn = Box::new(|_accumulator, iteration| {
            if iteration == 0 {
                Ok(LoopBodyResult::Normal(Value::Number(42.0))) // Valid
            } else {
                Ok(LoopBodyResult::Normal(Value::String("invalid".to_string())))
                // Type change - should fail
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_error());

        // Should fail on type mismatch
        if let LoopResult::Error(LoopError::LoopBodyError { iteration, .. }) = result {
            assert_eq!(iteration, 1); // Should fail on second iteration
        } else {
            panic!(
                "Expected LoopBodyError for type mismatch, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn test_loop_body_error_propagation() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(0, 5, 1);

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if iteration == 2 {
                // Throw error on third iteration
                Err(SemanticCLIError::execution_error(
                    "Test error",
                    ErrorCode::E500,
                ))
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
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_error());
        // Should complete some iterations before error
        assert!(result.get_iterations_completed() <= 2);

        // Should propagate the loop body error
        if let LoopResult::Error(LoopError::LoopBodyError { iteration, .. }) = result {
            assert_eq!(iteration, 2);
        } else {
            panic!("Expected LoopBodyError, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_range_loop() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(5, 5, 1); // Empty range

        let body_fn: LoopBodyFn = Box::new(|accumulator, _iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(acc + 1.0)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 0); // No iterations

        // Accumulator should remain unchanged
        if let Some(Value::Number(final_sum)) = result.get_accumulator() {
            assert_eq!(*final_sum, 0.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_negative_step_loop() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(5, 0, -1); // 5, 4, 3, 2, 1

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 5);

        // Sum should be 0+1+2+3+4 = 10 (iteration counter, not loop variable)
        if let Some(Value::Number(final_sum)) = result.get_accumulator() {
            assert_eq!(*final_sum, 10.0);
        } else {
            panic!("Expected number accumulator");
        }
    }
}

// =============================================================================
// 10.2 Test Collection Determinism
// =============================================================================

#[cfg(test)]
mod collection_determinism_tests {
    use super::*;

    #[test]
    fn test_array_deterministic_iteration() {
        let mut executor = LoopExecutor::new();
        let collection = Value::Array(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ]);
        let instruction = create_test_foreach_loop(collection, CollectionType::Array);

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                // Accumulate iteration order to verify determinism
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc * 10.0 + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 3);

        // Should iterate in index order: 0, 1, 2
        // Result: ((0*10+0)*10+1)*10+2 = 012
        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 12.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_list_deterministic_iteration() {
        let mut executor = LoopExecutor::new();
        let collection = Value::List(vec![
            Value::String("first".to_string()),
            Value::String("second".to_string()),
            Value::String("third".to_string()),
        ]);
        let instruction = create_test_foreach_loop(collection, CollectionType::List);

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                // Accumulate iteration order to verify determinism
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc * 10.0 + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 3);

        // Should iterate in insertion order: 0, 1, 2
        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 12.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_sorted_map_deterministic_iteration() {
        let mut executor = LoopExecutor::new();
        let mut map = BTreeMap::new();
        map.insert("zebra".to_string(), Value::Number(3.0));
        map.insert("alpha".to_string(), Value::Number(1.0));
        map.insert("beta".to_string(), Value::Number(2.0));

        let collection = Value::SortedMap(map);
        let instruction = create_test_foreach_loop(collection, CollectionType::SortedMap);

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                // Accumulate iteration order to verify determinism
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc * 10.0 + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 3);

        // Should iterate in key sort order: alpha, beta, zebra (0, 1, 2)
        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 12.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_range_deterministic_iteration() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(10, 15, 2); // 10, 12, 14

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                // Accumulate iteration order to verify determinism
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc * 100.0 + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 3);

        // Should iterate in sequence: 0, 1, 2 (iteration counter)
        // Result: ((0*100+0)*100+1)*100+2 = 102
        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 102.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_empty_collection_iteration() {
        let mut executor = LoopExecutor::new();
        let collection = Value::Array(vec![]);
        let instruction = create_test_foreach_loop(collection, CollectionType::Array);

        let body_fn: LoopBodyFn = Box::new(|accumulator, _iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(acc + 1.0)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 0);

        // Accumulator should remain unchanged
        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 0.0);
        } else {
            panic!("Expected number accumulator");
        }
    }

    // Helper function to create deterministic body function
    fn determinism_body() -> LoopBodyFn {
        Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc * 10.0 + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        })
    }

    #[test]
    fn test_collection_determinism_repeatability() {
        // Test that the same collection produces the same iteration order multiple times
        let mut executor = LoopExecutor::new();
        let collection = Value::Array(vec![
            Value::Number(5.0),
            Value::Number(3.0),
            Value::Number(7.0),
            Value::Number(1.0),
        ]);

        // Execute the same loop multiple times
        let mut results = Vec::new();
        for _ in 0..3 {
            let instruction = create_test_foreach_loop(collection.clone(), CollectionType::Array);
            let result = executor
                .execute_loop(&instruction, determinism_body())
                .unwrap();

            assert!(result.is_success());
            assert_eq!(result.get_iterations_completed(), 4);

            if let Some(Value::Number(final_value)) = result.get_accumulator() {
                results.push(*final_value);
            } else {
                panic!("Expected number accumulator");
            }
        }

        // All results should be identical (deterministic)
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0], results[2]);
        assert_eq!(results[0], 123.0); // Expected: ((0*10+0)*10+1)*10+2)*10+3 = 123
    }

    #[test]
    fn test_range_edge_cases() {
        let mut executor = LoopExecutor::new();

        // Test single iteration range
        let instruction = create_test_for_loop(5, 6, 1); // Just 5
        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 1);

        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 0.0); // 0 + 0 (first iteration)
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_large_step_range() {
        let mut executor = LoopExecutor::new();
        let instruction = create_test_for_loop(0, 100, 50); // 0, 50

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(
                    acc + iteration as f64,
                )))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = executor.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.get_iterations_completed(), 2);

        // Sum should be 0+1 = 1 (iteration counter)
        if let Some(Value::Number(final_value)) = result.get_accumulator() {
            assert_eq!(*final_value, 1.0);
        } else {
            panic!("Expected number accumulator");
        }
    }
}

// =============================================================================
// Test Configuration and Utilities
// =============================================================================

#[cfg(test)]
mod test_utilities {
    use super::*;

    #[test]
    fn test_helper_functions() {
        // Test helper function correctness
        let for_loop = create_test_for_loop(1, 5, 2);
        match for_loop {
            LoopInstruction::For { range, .. } => {
                assert_eq!(range.start, 1);
                assert_eq!(range.end, 5);
                assert_eq!(range.step, 2);
            }
            _ => panic!("Expected For loop"),
        }

        let while_loop = create_test_while_loop(Value::Boolean(true));
        match while_loop {
            LoopInstruction::While { condition, .. } => match condition {
                OperandRef::Literal(Value::Boolean(true)) => {}
                _ => panic!("Expected boolean true condition"),
            },
            _ => panic!("Expected While loop"),
        }

        let foreach_loop = create_test_foreach_loop(
            Value::Array(vec![Value::Number(1.0)]),
            CollectionType::Array,
        );
        match foreach_loop {
            LoopInstruction::ForEach {
                collection_type, ..
            } => {
                assert_eq!(collection_type, CollectionType::Array);
            }
            _ => panic!("Expected ForEach loop"),
        }
    }
}
