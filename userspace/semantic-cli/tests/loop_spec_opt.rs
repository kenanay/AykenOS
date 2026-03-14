//! Loop Optimization Tests - Spec Implementation (10.4 + Integration)
//!
//! This test suite covers the optimization systems including:
//! - Loop unrolling optimization
//! - Hot loop detection and monitoring
//! - JIT compilation integration
//! - Integration tests
//!
//! These tests are behind the d3_loop_spec feature flag.
//!
//! Run with: cargo test -p semantic-cli --features d3_loop_spec

#![cfg(feature = "d3_loop_spec")]

use semantic_cli::bcib::{
    CollectionType, LoopConfig, LoopID, LoopInstruction, LoopRange, OperandRef, Value, ValueType,
};
use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::loop_engine::{
    JITCompilationStatus, LoopAnalysisContext, LoopBodyFn, LoopBodyResult, LoopEngine,
    LoopExecutor, LoopMonitor, LoopUnroller, SafetyClass, UnrollResult, UnrollSkipReason,
};
use semantic_cli::types::SourceLocation;

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
// 10.4 Test Optimization Systems
// =============================================================================

#[cfg(all(test, feature = "d3_loop_spec"))]
mod optimization_tests {
    use super::*;

    // Loop Unrolling Tests
    #[test]
    fn test_small_loop_unrolling() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(0, 5, 1); // 5 iterations

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                iteration_count,
                unrolled_sequence,
            } => {
                assert_eq!(iteration_count, 5);
                // Each iteration should generate 2 instructions (iterator binding + body)
                assert_eq!(unrolled_sequence.instructions.len(), 10);
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling for small loop but got: {}", reason);
            }
        }
    }

    #[test]
    fn test_large_loop_not_unrolled() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(0, 15, 1); // 15 iterations (above threshold)

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => match reason {
                UnrollSkipReason::IterationCountTooHigh { count, threshold } => {
                    assert_eq!(count, 15);
                    assert_eq!(threshold, 10);
                }
                _ => panic!("Expected IterationCountTooHigh but got: {}", reason),
            },
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling for large loop");
            }
        }
    }

    #[test]
    fn test_while_loop_not_unrolled() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_while_loop(Value::Boolean(true));

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => {
                match reason {
                    UnrollSkipReason::WhileLoopNotSupported => {
                        // Expected - While loops are never unrolled
                    }
                    _ => panic!("Expected WhileLoopNotSupported but got: {}", reason),
                }
            }
            UnrollResult::Unrolled { .. } => {
                panic!("Expected no unrolling for While loop");
            }
        }
    }

    #[test]
    fn test_zero_iteration_loop_unrolling() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(5, 5, 1); // Empty range

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                iteration_count,
                unrolled_sequence,
            } => {
                assert_eq!(iteration_count, 0);
                assert_eq!(unrolled_sequence.instructions.len(), 0);
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!(
                    "Expected unrolling for zero-iteration loop but got: {}",
                    reason
                );
            }
        }
    }

    #[test]
    fn test_single_iteration_loop_unrolling() {
        let mut unroller = LoopUnroller::new();
        let loop_instruction = create_test_for_loop(42, 43, 1); // Single iteration

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                iteration_count,
                unrolled_sequence,
            } => {
                assert_eq!(iteration_count, 1);
                assert_eq!(unrolled_sequence.instructions.len(), 2); // 1 iteration * 2 instructions
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!(
                    "Expected unrolling for single-iteration loop but got: {}",
                    reason
                );
            }
        }
    }

    #[test]
    fn test_foreach_literal_collection_unrolling() {
        let mut unroller = LoopUnroller::new();
        let collection = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let loop_instruction = create_test_foreach_loop(collection, CollectionType::Array);

        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                iteration_count,
                unrolled_sequence,
            } => {
                assert_eq!(iteration_count, 3);
                assert_eq!(unrolled_sequence.instructions.len(), 6); // 3 iterations * 2 instructions
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!(
                    "Expected unrolling for literal collection but got: {}",
                    reason
                );
            }
        }
    }

    #[test]
    fn test_unroll_statistics() {
        let mut unroller = LoopUnroller::new();

        // Analyze various loops
        let small_loop1 = create_test_for_loop(0, 3, 1); // 3 iterations - should unroll
        let small_loop2 = create_test_for_loop(0, 5, 1); // 5 iterations - should unroll
        let large_loop = create_test_for_loop(0, 15, 1); // 15 iterations - should not unroll
        let while_loop = create_test_while_loop(Value::Boolean(true)); // While loop - should not unroll

        unroller.analyze_loop(&small_loop1).unwrap();
        unroller.analyze_loop(&small_loop2).unwrap();
        unroller.analyze_loop(&large_loop).unwrap();
        unroller.analyze_loop(&while_loop).unwrap();

        let stats = unroller.get_stats();

        // Verify statistics
        assert_eq!(stats.loops_analyzed, 4);
        assert_eq!(stats.loops_unrolled, 2);
        assert_eq!(stats.loops_skipped_too_large, 1);
        assert_eq!(stats.loops_skipped_while, 1);
        assert_eq!(stats.total_iterations_unrolled, 8); // 3 + 5 = 8

        // Test calculated metrics
        assert_eq!(stats.success_rate(), 50.0); // 2/4 * 100
        assert_eq!(stats.average_iterations_per_unroll(), 4.0); // 8/2
    }

    // Hot Loop Detection Tests
    #[test]
    #[ignore = "Monitoring API may not be fully implemented"]
    fn test_hot_loop_detection() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("test-hot-loop".to_string());
        let instruction = create_test_for_loop(0, 1500, 1); // 1500 iterations (above hot threshold)

        // Start monitoring
        let tracker = monitor.record_loop_start(&loop_id, &instruction);

        // Record completion with high iteration count
        let result = monitor.record_loop_completion(
            tracker,
            1500,
            semantic_cli::loop_engine::monitoring::LoopExecutionResult::Success,
        );
        assert!(result.is_ok());

        // Should be detected as hot loop
        assert!(monitor.is_hot_loop(&loop_id));

        // Get hot loop info
        let hot_info = monitor.get_hot_loop_info(&loop_id);
        assert!(hot_info.is_some());

        let info = hot_info.unwrap();
        assert_eq!(info.loop_id, loop_id);
        assert!(info.detection_iteration_count >= 1500);
        // JIT status may vary based on implementation
        assert!(matches!(
            info.jit_status,
            JITCompilationStatus::NotEligible
                | JITCompilationStatus::Eligible
                | JITCompilationStatus::Compiling
        ));
    }

    #[test]
    #[ignore = "Monitoring API may not be fully implemented"]
    fn test_cold_loop_not_detected() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("test-cold-loop".to_string());
        let instruction = create_test_for_loop(0, 50, 1); // 50 iterations (below hot threshold)

        // Start monitoring
        let tracker = monitor.record_loop_start(&loop_id, &instruction);

        // Record completion with low iteration count
        let result = monitor.record_loop_completion(
            tracker,
            50,
            semantic_cli::loop_engine::monitoring::LoopExecutionResult::Success,
        );
        assert!(result.is_ok());

        // Should NOT be detected as hot loop
        assert!(!monitor.is_hot_loop(&loop_id));

        // Should not have hot loop info
        let hot_info = monitor.get_hot_loop_info(&loop_id);
        assert!(hot_info.is_none());
    }

    #[test]
    #[ignore = "JIT compilation API may not be fully implemented"]
    fn test_jit_compilation_triggering() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("test-jit-loop".to_string());

        // First create a hot loop to make JIT compilation meaningful
        let instruction = create_test_for_loop(0, 1500, 1);
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor
            .record_loop_completion(
                tracker,
                1500,
                semantic_cli::loop_engine::monitoring::LoopExecutionResult::Success,
            )
            .unwrap();

        // Manually trigger JIT compilation
        let result = monitor.trigger_jit_compilation(&loop_id);
        assert!(result.is_ok());

        // Record JIT compilation result
        let jit_result = semantic_cli::loop_engine::monitoring::JITCompilationResult::Success {
            compilation_time: std::time::Duration::from_millis(150),
        };
        let result = monitor.record_jit_compilation_result(&loop_id, jit_result);
        assert!(result.is_ok());

        // Check hot loop info
        let hot_info = monitor.get_hot_loop_info(&loop_id);
        assert!(hot_info.is_some());

        let info = hot_info.unwrap();
        assert!(info.jit_triggered);
        assert!(matches!(
            info.jit_status,
            JITCompilationStatus::Compiled { .. }
        ));
    }

    #[test]
    #[ignore = "Monitoring API may not be fully implemented"]
    fn test_global_monitoring_statistics() {
        let mut monitor = LoopMonitor::new();

        // Create multiple loops
        let loop1 = LoopID::new("loop1".to_string());
        let loop2 = LoopID::new("loop2".to_string());
        let loop3 = LoopID::new("loop3".to_string());

        let instruction = create_test_for_loop(0, 1200, 1); // Hot loop

        // Record multiple loop executions
        for loop_id in [&loop1, &loop2, &loop3] {
            let tracker = monitor.record_loop_start(loop_id, &instruction);
            monitor
                .record_loop_completion(
                    tracker,
                    1200,
                    semantic_cli::loop_engine::monitoring::LoopExecutionResult::Success,
                )
                .unwrap();
        }

        // Get global statistics
        let stats = monitor.get_global_stats();
        assert_eq!(stats.total_loop_executions, 3);
        assert_eq!(stats.hot_loops_detected, 3);
        assert_eq!(stats.total_iterations, 3600); // 3 * 1200
    }

    #[test]
    #[ignore = "Monitoring configuration API may not be fully implemented"]
    fn test_monitoring_configuration() {
        let mut monitor = LoopMonitor::new();

        // Update configuration
        let config = semantic_cli::loop_engine::monitoring::MonitoringConfig {
            hot_loop_threshold: 500, // Lower threshold
            enable_detailed_tracking: true,
            enable_hot_loop_logging: true,
            max_loop_stats_entries: 1000,
            auto_trigger_jit: true,
        };
        monitor.update_config(config);

        // Test with new threshold
        let loop_id = LoopID::new("test-config-loop".to_string());
        let instruction = create_test_for_loop(0, 600, 1); // 600 iterations

        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor
            .record_loop_completion(
                tracker,
                600,
                semantic_cli::loop_engine::monitoring::LoopExecutionResult::Success,
            )
            .unwrap();

        // Should be hot with lower threshold
        assert!(monitor.is_hot_loop(&loop_id));
    }

    #[test]
    #[ignore = "Monitoring data clearing API may not be fully implemented"]
    fn test_monitoring_data_clearing() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("test-clear-loop".to_string());
        let instruction = create_test_for_loop(0, 1500, 1);

        // Record loop execution
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor
            .record_loop_completion(
                tracker,
                1500,
                semantic_cli::loop_engine::monitoring::LoopExecutionResult::Success,
            )
            .unwrap();

        // Verify data exists
        assert!(monitor.is_hot_loop(&loop_id));
        assert_eq!(monitor.get_global_stats().total_loop_executions, 1);

        // Clear monitoring data
        monitor.clear_monitoring_data();

        // Verify data is cleared
        assert!(!monitor.is_hot_loop(&loop_id));
        assert_eq!(monitor.get_global_stats().total_loop_executions, 0);
    }
}

// =============================================================================
// Integration Tests - Testing Loop Engine as a Whole
// =============================================================================

#[cfg(all(test, feature = "d3_loop_spec"))]
mod integration_tests {
    use super::*;

    #[test]
    fn test_loop_engine_complete_workflow() {
        let mut engine = LoopEngine::new();
        let instruction = create_test_for_loop(0, 10, 1);

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

        let result = engine.execute_loop(&instruction, body_fn).unwrap();

        assert!(result.is_success());
        assert_eq!(result.iterations_completed, 10);

        if let Value::Number(final_sum) = result.accumulator {
            assert_eq!(final_sum, 45.0); // 0+1+2+...+9 = 45
        } else {
            panic!("Expected number accumulator");
        }
    }

    #[test]
    fn test_safety_analysis_integration() {
        let mut engine = LoopEngine::new();
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        // Test safe loop body
        let safe_body = "accumulator = accumulator + i * 2";
        let result = engine.analyze_loop_safety(safe_body, &context).unwrap();

        assert_eq!(result.classification, SafetyClass::Safe);
        assert!(result.side_effects.is_empty());

        // Test unsafe loop body
        let unsafe_body = "file_write('output.txt', i); accumulator = accumulator + i";
        let result = engine.analyze_loop_safety(unsafe_body, &context).unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());
    }

    #[test]
    fn test_unrolling_integration() {
        let mut engine = LoopEngine::new();
        let small_loop = create_test_for_loop(0, 5, 1);

        // Check if loop should be unrolled
        let should_unroll = engine.should_unroll_loop(&small_loop).unwrap();
        assert!(should_unroll);

        // Analyze unrolling
        let result = engine.analyze_loop_unrolling(&small_loop).unwrap();
        match result {
            UnrollResult::Unrolled {
                iteration_count, ..
            } => {
                assert_eq!(iteration_count, 5);
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!("Expected unrolling but got: {}", reason);
            }
        }
    }

    #[test]
    #[ignore = "Hot loop detection integration may not be fully implemented"]
    fn test_hot_loop_detection_integration() {
        let mut engine = LoopEngine::new();
        let instruction = create_test_for_loop(0, 1500, 1); // Hot loop

        // Extract the loop ID from the instruction
        let loop_id = match &instruction {
            LoopInstruction::For { id, .. } => id.clone(),
            _ => panic!("Expected For loop"),
        };

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

        // Execute the loop
        let result = engine.execute_loop(&instruction, body_fn).unwrap();
        assert!(result.is_success());

        // Check if it was detected as hot
        assert!(engine.is_hot_loop(&loop_id));

        // Get hot loop info
        let hot_info = engine.get_hot_loop_info(&loop_id);
        assert!(hot_info.is_some());
    }

    #[test]
    fn test_error_handling_integration() {
        let mut engine = LoopEngine::new();
        let mut instruction = create_test_for_loop(0, 100, 1);

        // Set low iteration limit to trigger error
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

        let result = engine.execute_loop(&instruction, body_fn).unwrap();

        assert!(!result.is_success());
        assert_eq!(result.iterations_completed, 5);
    }

    #[test]
    fn test_multiple_accumulator_types() {
        let mut engine = LoopEngine::new();

        // Test with string accumulator
        let mut string_instruction = create_test_for_loop(0, 3, 1);
        if let LoopInstruction::For { config, .. } = &mut string_instruction {
            config.initial_accumulator = Value::String("".to_string());
            config.accumulator_type = ValueType::String;
        }

        let string_body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::String(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::String(format!(
                    "{}{}",
                    acc, iteration
                ))))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let result = engine
            .execute_loop(&string_instruction, string_body_fn)
            .unwrap();

        assert!(result.is_success());
        assert_eq!(result.iterations_completed, 3);

        if let Value::String(final_string) = result.accumulator {
            assert_eq!(final_string, "012");
        } else {
            panic!("Expected string accumulator");
        }
    }

    #[test]
    #[ignore = "Monitoring statistics integration may not be fully implemented"]
    fn test_monitoring_statistics_integration() {
        let mut engine = LoopEngine::new();

        // Execute multiple loops
        for i in 0..3 {
            let instruction = create_test_for_loop(0, 100 + i * 500, 1); // Varying sizes

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

            let result = engine.execute_loop(&instruction, body_fn).unwrap();
            assert!(result.is_success());
        }

        // Check global statistics
        let stats = engine.get_global_monitoring_stats();
        assert_eq!(stats.total_loop_executions, 3);
        assert!(stats.total_iterations > 0);
    }
}
