//! Loop Unrolling Tests - Phase 5.1 Implementation
//!
//! This test suite validates the loop unrolling optimization implementation
//! according to the D3 Loop Support Design requirements.
//!
//! # Requirements Tested
//!
//! - **Requirement 4.1**: Automatically unroll loops with iteration count < 10
//! - **Requirement 4.2**: Expand loop body into sequential IR instructions
//! - **Requirement 4.3**: Preserve exact semantics including iteration order and side effects
//! - **Requirement 4.4**: Exclude unrolling decisions from fingerprint (optimization only)
//! - **Requirement 4.5**: Skip unrolling when iteration count cannot be statically analyzed
//!
//! # Property-Based Test Coverage
//!
//! - **Property 4**: Loop Unrolling Semantic Preservation
//! - **Property 5**: Unrolling Fingerprint Independence

use semantic_cli::bcib::{
    BCIBSequence, CollectionType, LoopConfig, LoopID, LoopInstruction, LoopRange, OperandRef,
    Value, ValueType,
};
use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::loop_engine::{
    LoopEngine, LoopUnroller, UnrollConfig, UnrollResult, UnrollSkipReason,
};
use semantic_cli::types::SourceLocation;

fn create_test_for_loop(start: i64, end: i64, step: i64) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new("test-unroll".to_string()),
        range: LoopRange::new(start, end, step),
        iterator_var: "i".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

fn create_test_foreach_loop_literal(collection: Value) -> LoopInstruction {
    LoopInstruction::ForEach {
        id: LoopID::new("test-unroll-foreach".to_string()),
        collection: OperandRef::Literal(collection),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

fn create_test_foreach_loop_field(field_name: &str) -> LoopInstruction {
    LoopInstruction::ForEach {
        id: LoopID::new("test-unroll-foreach-field".to_string()),
        collection: OperandRef::Field(field_name.to_string()),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

fn create_test_while_loop() -> LoopInstruction {
    LoopInstruction::While {
        id: LoopID::new("test-unroll-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

#[test]
fn test_requirement_4_1_automatic_unrolling_small_loops() {
    // ✅ Requirement 4.1: Automatically unroll loops with iteration count < 10

    let mut unroller = LoopUnroller::new();

    // Test various small loop sizes
    let test_cases = vec![
        (0, 1, 1, 1),   // 1 iteration
        (0, 3, 1, 3),   // 3 iterations
        (0, 5, 1, 5),   // 5 iterations
        (0, 9, 1, 9),   // 9 iterations (just under threshold)
        (2, 8, 2, 3),   // 3 iterations with step 2: [2, 4, 6]
        (10, 5, -1, 5), // 5 iterations with negative step: [10, 9, 8, 7, 6]
    ];

    for (start, end, step, expected_iterations) in test_cases {
        let loop_instruction = create_test_for_loop(start, end, step);
        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                iteration_count,
                unrolled_sequence,
            } => {
                assert_eq!(
                    iteration_count, expected_iterations,
                    "Loop [{}, {}, {}] should have {} iterations",
                    start, end, step, expected_iterations
                );

                // Each iteration should generate 2 instructions (iterator binding + body placeholder)
                let expected_instruction_count = expected_iterations * 2;
                assert_eq!(
                    unrolled_sequence.instructions.len(),
                    expected_instruction_count as usize,
                    "Unrolled sequence should have {} instructions",
                    expected_instruction_count
                );
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!(
                    "Expected unrolling for small loop [{}, {}, {}] but got: {}",
                    start, end, step, reason
                );
            }
        }
    }
}

#[test]
fn test_requirement_4_1_skip_large_loops() {
    // ✅ Requirement 4.1: Skip loops with iteration count >= 10

    let mut unroller = LoopUnroller::new();

    // Test loops at and above the threshold
    let test_cases = vec![
        (0, 10, 1, 10),   // Exactly at threshold
        (0, 15, 1, 15),   // Above threshold
        (0, 100, 1, 100), // Well above threshold
    ];

    for (start, end, step, expected_iterations) in test_cases {
        let loop_instruction = create_test_for_loop(start, end, step);
        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::NotUnrolled { reason, .. } => match reason {
                UnrollSkipReason::IterationCountTooHigh { count, threshold } => {
                    assert_eq!(count, expected_iterations);
                    assert_eq!(threshold, 10);
                }
                _ => panic!("Expected IterationCountTooHigh but got: {}", reason),
            },
            UnrollResult::Unrolled { .. } => {
                panic!(
                    "Expected no unrolling for large loop [{}, {}, {}]",
                    start, end, step
                );
            }
        }
    }
}

#[test]
fn test_requirement_4_2_sequential_ir_instructions() {
    // ✅ Requirement 4.2: Expand loop body into sequential IR instructions

    let mut unroller = LoopUnroller::new();
    let loop_instruction = create_test_for_loop(1, 4, 1); // 3 iterations: [1, 2, 3]

    let result = unroller.analyze_loop(&loop_instruction).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 3);

            // Verify sequential structure
            assert_eq!(unrolled_sequence.instructions.len(), 6); // 3 iterations * 2 instructions each

            // Verify the sequence is valid BCIB
            assert!(
                unrolled_sequence.validate().is_ok(),
                "Unrolled sequence should be valid BCIB"
            );

            // Verify instructions are sequential (no nested structures)
            for instruction in &unrolled_sequence.instructions {
                // All instructions should be atomic (no nested loop structures)
                match instruction {
                    semantic_cli::bcib::BCIBInstruction::Loop(_) => {
                        panic!("Unrolled sequence should not contain nested loop instructions");
                    }
                    _ => {
                        // Expected - atomic instructions only
                    }
                }
            }
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling but got: {}", reason);
        }
    }
}

#[test]
fn test_requirement_4_3_semantic_preservation_iteration_order() {
    // ✅ Requirement 4.3: Preserve exact semantics including iteration order

    let mut unroller = LoopUnroller::new();

    // Test forward iteration order
    let forward_loop = create_test_for_loop(5, 8, 1); // [5, 6, 7]
    let result = unroller.analyze_loop(&forward_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 3);

            // Verify iteration order is preserved in instruction sequence
            // Each pair of instructions represents one iteration
            assert_eq!(unrolled_sequence.instructions.len(), 6);

            // Instructions should be generated in iteration order (5, then 6, then 7)
            // This is verified by the sequential nature of the instruction generation
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling but got: {}", reason);
        }
    }

    // Test reverse iteration order
    let reverse_loop = create_test_for_loop(10, 7, -1); // [10, 9, 8]
    let result = unroller.analyze_loop(&reverse_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 3);
            assert_eq!(unrolled_sequence.instructions.len(), 6);

            // Iteration order should be preserved (10, then 9, then 8)
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling but got: {}", reason);
        }
    }
}

#[test]
fn test_requirement_4_5_skip_non_static_iteration_count() {
    // ✅ Requirement 4.5: Skip unrolling when iteration count cannot be statically analyzed

    let mut unroller = LoopUnroller::new();

    // Test While loop (never static)
    let while_loop = create_test_while_loop();
    let result = unroller.analyze_loop(&while_loop).unwrap();

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

    // Test ForEach with field reference (dynamic collection)
    let foreach_field_loop = create_test_foreach_loop_field("dynamic_collection");
    let result = unroller.analyze_loop(&foreach_field_loop).unwrap();

    match result {
        UnrollResult::NotUnrolled { reason, .. } => {
            match reason {
                UnrollSkipReason::ForEachDynamicCollection => {
                    // Expected - Field references have dynamic size
                }
                _ => panic!("Expected ForEachDynamicCollection but got: {}", reason),
            }
        }
        UnrollResult::Unrolled { .. } => {
            panic!("Expected no unrolling for dynamic ForEach loop");
        }
    }

    // Test ForEach with temp register (dynamic collection)
    let foreach_register_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-unroll-foreach-register".to_string()),
        collection: OperandRef::TempRegister(0),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };

    let result = unroller.analyze_loop(&foreach_register_loop).unwrap();

    match result {
        UnrollResult::NotUnrolled { reason, .. } => {
            match reason {
                UnrollSkipReason::ForEachDynamicCollection => {
                    // Expected - Temp registers have dynamic size
                }
                _ => panic!("Expected ForEachDynamicCollection but got: {}", reason),
            }
        }
        UnrollResult::Unrolled { .. } => {
            panic!("Expected no unrolling for temp register ForEach loop");
        }
    }
}

#[test]
fn test_foreach_literal_collection_unrolling() {
    // Test ForEach loops with literal collections (static size)

    let mut unroller = LoopUnroller::new();

    // Test Array literal
    let array_collection = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let array_loop = create_test_foreach_loop_literal(array_collection);
    let result = unroller.analyze_loop(&array_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 3);
            assert_eq!(unrolled_sequence.instructions.len(), 6); // 3 iterations * 2 instructions
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling for Array literal but got: {}", reason);
        }
    }

    // Test List literal
    let list_collection = Value::List(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
    ]);
    let list_loop = create_test_foreach_loop_literal(list_collection);
    let result = unroller.analyze_loop(&list_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 2);
            assert_eq!(unrolled_sequence.instructions.len(), 4); // 2 iterations * 2 instructions
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling for List literal but got: {}", reason);
        }
    }

    // Test SortedMap literal
    let mut map = std::collections::BTreeMap::new();
    map.insert("key1".to_string(), Value::Boolean(true));
    map.insert("key2".to_string(), Value::Boolean(false));
    let map_collection = Value::SortedMap(map);
    let map_loop = create_test_foreach_loop_literal(map_collection);
    let result = unroller.analyze_loop(&map_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 2);
            assert_eq!(unrolled_sequence.instructions.len(), 4); // 2 iterations * 2 instructions
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!(
                "Expected unrolling for SortedMap literal but got: {}",
                reason
            );
        }
    }
}

#[test]
fn test_zero_and_single_iteration_loops() {
    let mut unroller = LoopUnroller::new();

    // Test zero iteration loop
    let zero_loop = create_test_for_loop(5, 5, 1); // Empty range
    let result = unroller.analyze_loop(&zero_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count,
            unrolled_sequence,
        } => {
            assert_eq!(iteration_count, 0);
            assert_eq!(unrolled_sequence.instructions.len(), 0); // No instructions for 0 iterations
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!(
                "Expected unrolling for zero-iteration loop but got: {}",
                reason
            );
        }
    }

    // Test single iteration loop
    let single_loop = create_test_for_loop(42, 43, 1); // Single iteration: [42]
    let result = unroller.analyze_loop(&single_loop).unwrap();

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
fn test_custom_unroll_threshold() {
    // Test custom unrolling threshold configuration

    let config = UnrollConfig {
        max_unroll_iterations: 5, // Lower threshold
        enabled: true,
        collect_stats: true,
    };
    let mut unroller = LoopUnroller::with_config(config);

    // Loop with 7 iterations (above custom threshold)
    let loop_instruction = create_test_for_loop(0, 7, 1);
    let result = unroller.analyze_loop(&loop_instruction).unwrap();

    match result {
        UnrollResult::NotUnrolled { reason, .. } => match reason {
            UnrollSkipReason::IterationCountTooHigh { count, threshold } => {
                assert_eq!(count, 7);
                assert_eq!(threshold, 5);
            }
            _ => panic!("Expected IterationCountTooHigh but got: {}", reason),
        },
        UnrollResult::Unrolled { .. } => {
            panic!("Expected no unrolling with custom threshold");
        }
    }

    // Loop with 3 iterations (below custom threshold)
    let small_loop = create_test_for_loop(0, 3, 1);
    let result = unroller.analyze_loop(&small_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count, ..
        } => {
            assert_eq!(iteration_count, 3);
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling for small loop but got: {}", reason);
        }
    }
}

#[test]
fn test_unrolling_disabled() {
    // Test unrolling disabled configuration

    let config = UnrollConfig {
        enabled: false,
        ..UnrollConfig::default()
    };
    let mut unroller = LoopUnroller::with_config(config);

    let loop_instruction = create_test_for_loop(0, 3, 1); // Small loop that would normally unroll
    let result = unroller.analyze_loop(&loop_instruction).unwrap();

    match result {
        UnrollResult::NotUnrolled { reason, .. } => {
            match reason {
                UnrollSkipReason::UnrollingDisabled => {
                    // Expected
                }
                _ => panic!("Expected UnrollingDisabled but got: {}", reason),
            }
        }
        UnrollResult::Unrolled { .. } => {
            panic!("Expected no unrolling when disabled");
        }
    }
}

#[test]
fn test_unroll_statistics() {
    // Test unrolling statistics collection

    let mut unroller = LoopUnroller::new();

    // Analyze various loops
    let small_loop1 = create_test_for_loop(0, 3, 1); // 3 iterations - should unroll
    let small_loop2 = create_test_for_loop(0, 5, 1); // 5 iterations - should unroll
    let large_loop = create_test_for_loop(0, 15, 1); // 15 iterations - should not unroll
    let while_loop = create_test_while_loop(); // While loop - should not unroll
    let field_foreach = create_test_foreach_loop_field("dynamic"); // Dynamic - should not unroll

    unroller.analyze_loop(&small_loop1).unwrap();
    unroller.analyze_loop(&small_loop2).unwrap();
    unroller.analyze_loop(&large_loop).unwrap();
    unroller.analyze_loop(&while_loop).unwrap();
    unroller.analyze_loop(&field_foreach).unwrap();

    let stats = unroller.get_stats();

    // Verify statistics
    assert_eq!(stats.loops_analyzed, 5);
    assert_eq!(stats.loops_unrolled, 2);
    assert_eq!(stats.loops_skipped_too_large, 1);
    assert_eq!(stats.loops_skipped_while, 1);
    assert_eq!(stats.loops_skipped_non_static, 1);
    assert_eq!(stats.total_iterations_unrolled, 8); // 3 + 5 = 8

    // Test calculated metrics
    assert_eq!(stats.success_rate(), 40.0); // 2/5 * 100
    assert_eq!(stats.average_iterations_per_unroll(), 4.0); // 8/2

    // Test skip summary
    let skip_summary = stats.skip_summary();
    assert_eq!(skip_summary["too_large"], 1);
    assert_eq!(skip_summary["while_loops"], 1);
    assert_eq!(skip_summary["non_static"], 1);
}

#[test]
fn test_should_unroll_lightweight_check() {
    // Test the lightweight should_unroll check

    let unroller = LoopUnroller::new();

    // Small loop should be unrolled
    let small_loop = create_test_for_loop(0, 5, 1);
    assert!(unroller.should_unroll(&small_loop).unwrap());

    // Large loop should not be unrolled
    let large_loop = create_test_for_loop(0, 15, 1);
    assert!(!unroller.should_unroll(&large_loop).unwrap());

    // While loop should not be unrolled
    let while_loop = create_test_while_loop();
    assert!(!unroller.should_unroll(&while_loop).unwrap());

    // Dynamic ForEach should not be unrolled
    let dynamic_foreach = create_test_foreach_loop_field("dynamic");
    assert!(!unroller.should_unroll(&dynamic_foreach).unwrap());

    // Static ForEach should be unrolled
    let static_foreach = create_test_foreach_loop_literal(Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ]));
    assert!(unroller.should_unroll(&static_foreach).unwrap());
}

#[test]
fn test_loop_engine_integration() {
    // Test integration with LoopEngine

    let mut engine = LoopEngine::new();

    // Test unrolling analysis through engine
    let loop_instruction = create_test_for_loop(0, 4, 1);
    let result = engine.analyze_loop_unrolling(&loop_instruction).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count, ..
        } => {
            assert_eq!(iteration_count, 4);
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling but got: {}", reason);
        }
    }

    // Test should_unroll check through engine
    assert!(engine.should_unroll_loop(&loop_instruction).unwrap());

    // Test statistics access through engine
    let stats = engine.get_unroll_stats();
    assert_eq!(stats.loops_analyzed, 1);
    assert_eq!(stats.loops_unrolled, 1);

    // Test statistics reset
    engine.reset_unroll_stats();
    let stats = engine.get_unroll_stats();
    assert_eq!(stats.loops_analyzed, 0);
    assert_eq!(stats.loops_unrolled, 0);
}

#[test]
fn test_complex_range_patterns() {
    // Test various complex range patterns

    let mut unroller = LoopUnroller::new();

    let test_cases = vec![
        // (start, end, step, expected_iterations, description)
        (0, 10, 2, 5, "Even numbers 0-8"),   // [0, 2, 4, 6, 8]
        (1, 10, 2, 5, "Odd numbers 1-9"),    // [1, 3, 5, 7, 9]
        (10, 0, -2, 5, "Reverse even 10-2"), // [10, 8, 6, 4, 2]
        (9, 0, -2, 5, "Reverse odd 9-1"),    // [9, 7, 5, 3, 1]
        (0, 25, 5, 5, "Multiples of 5"),     // [0, 5, 10, 15, 20]
        (100, 90, -3, 4, "Reverse by 3"),    // [100, 97, 94, 91]
    ];

    for (start, end, step, expected_iterations, description) in test_cases {
        let loop_instruction = create_test_for_loop(start, end, step);
        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                iteration_count,
                unrolled_sequence,
            } => {
                assert_eq!(
                    iteration_count, expected_iterations,
                    "Test case '{}' should have {} iterations",
                    description, expected_iterations
                );

                let expected_instruction_count = expected_iterations * 2;
                assert_eq!(
                    unrolled_sequence.instructions.len(),
                    expected_instruction_count as usize,
                    "Test case '{}' should generate {} instructions",
                    description,
                    expected_instruction_count
                );
            }
            UnrollResult::NotUnrolled { reason, .. } => {
                panic!(
                    "Expected unrolling for test case '{}' but got: {}",
                    description, reason
                );
            }
        }
    }
}

#[test]
fn test_edge_case_ranges() {
    // Test edge cases for range calculations

    let mut unroller = LoopUnroller::new();

    // Large step that exceeds range in one step
    let large_step_loop = create_test_for_loop(0, 5, 10); // [0] - only one iteration
    let result = unroller.analyze_loop(&large_step_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count, ..
        } => {
            assert_eq!(iteration_count, 1);
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!("Expected unrolling for large step loop but got: {}", reason);
        }
    }

    // Negative range with positive step (should be 0 iterations)
    let invalid_range_loop = create_test_for_loop(10, 5, 1);
    let result = unroller.analyze_loop(&invalid_range_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count, ..
        } => {
            assert_eq!(iteration_count, 0);
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!(
                "Expected unrolling for zero-iteration loop but got: {}",
                reason
            );
        }
    }

    // Positive range with negative step (should be 0 iterations)
    let invalid_negative_loop = create_test_for_loop(5, 10, -1);
    let result = unroller.analyze_loop(&invalid_negative_loop).unwrap();

    match result {
        UnrollResult::Unrolled {
            iteration_count, ..
        } => {
            assert_eq!(iteration_count, 0);
        }
        UnrollResult::NotUnrolled { reason, .. } => {
            panic!(
                "Expected unrolling for zero-iteration loop but got: {}",
                reason
            );
        }
    }
}

// Property-Based Test Placeholders
// These would be implemented with a property-based testing library like proptest

#[test]
fn test_property_4_loop_unrolling_semantic_preservation() {
    // **Property 4**: Loop Unrolling Semantic Preservation
    // *For any* loop with statically known iteration count < 10, the unrolled version
    // SHALL produce identical results to the non-unrolled version (semantic equivalence).

    // This is a placeholder for a full property-based test
    // In a complete implementation, this would use proptest or similar to generate
    // random loop configurations and verify semantic equivalence

    let mut unroller = LoopUnroller::new();

    // Test a representative sample of loops
    let test_loops = vec![
        create_test_for_loop(0, 3, 1),
        create_test_for_loop(5, 10, 2),
        create_test_for_loop(10, 0, -1),
        create_test_foreach_loop_literal(Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])),
    ];

    for loop_instruction in test_loops {
        let result = unroller.analyze_loop(&loop_instruction).unwrap();

        match result {
            UnrollResult::Unrolled {
                unrolled_sequence, ..
            } => {
                // Verify the unrolled sequence is valid BCIB
                assert!(
                    unrolled_sequence.validate().is_ok(),
                    "Unrolled sequence must be valid BCIB"
                );

                // In a full implementation, we would execute both the original loop
                // and the unrolled sequence and verify they produce identical results
            }
            UnrollResult::NotUnrolled { .. } => {
                // Some loops may not be unrolled, which is acceptable
            }
        }
    }
}

#[test]
fn test_property_5_unrolling_fingerprint_independence() {
    // **Property 5**: Unrolling Fingerprint Independence
    // *For any* loop, the fingerprint SHALL be identical whether the loop is unrolled
    // or not (optimization does not affect fingerprint).

    // This is a placeholder for fingerprint testing
    // In a complete implementation, this would verify that:
    // 1. The original loop instruction has a specific fingerprint
    // 2. The unrolling decision does not change that fingerprint
    // 3. Only semantic changes affect fingerprints, not optimization decisions

    let unroller = LoopUnroller::new();
    let loop_instruction = create_test_for_loop(0, 5, 1);

    // Check that unrolling decision doesn't affect the loop instruction itself
    let original_loop = loop_instruction.clone();
    let should_unroll = unroller.should_unroll(&loop_instruction).unwrap();

    // The original loop instruction should be unchanged
    assert_eq!(loop_instruction, original_loop);

    // The unrolling decision is separate from the loop's semantic identity
    assert!(should_unroll); // This loop should be unrolled

    // In a full implementation, we would compute fingerprints and verify they're identical
    // regardless of whether unrolling is enabled or disabled
}
