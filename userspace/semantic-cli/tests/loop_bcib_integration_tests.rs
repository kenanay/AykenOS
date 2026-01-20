//! Integration Tests for D3 Loop Support with BCIB System (Task 12.1)
//!
//! This test suite validates the integration between the D3 Loop Support system
//! and the existing BCIB instruction system, ensuring:
//! 
//! 1. Loop instructions work with existing transformer
//! 2. Serialization/deserialization of loop instructions
//! 3. Capability requirements for loop operations
//!
//! **Requirements Validated:**
//! - Requirements 1.1, 1.2, 1.3, 1.4, 1.5 (Loop representation in IR)
//! - Requirements 2.1-2.8 (Bounded iteration enforcement)
//! - Requirements 3.1-3.8 (Timeout enforcement)
//! - Requirements 13.1-13.6 (Break/continue control flow)

use semantic_cli::bcib::{
    BCIBInstruction, BCIBSequence, LoopInstruction, ControlFlowInstruction,
    LoopID, LoopConfig, LoopRange, Value, ValueType, CollectionType, OperandRef,
    BudgetMeasurement, ErrorRecoveryPolicy, Capability
};
use semantic_cli::types::SourceLocation;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1, 0)
}

/// Test that loop instructions integrate properly with the BCIB system
#[test]
fn test_loop_instruction_bcib_integration() {
    // Create a While loop instruction
    let while_loop = LoopInstruction::While {
        id: LoopID::new("test-while".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "while-body-block".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: test_location(),
    };

    // Wrap in BCIB instruction
    let bcib_instruction = BCIBInstruction::Loop(while_loop);

    // Validate the instruction
    assert!(bcib_instruction.validate().is_ok());

    // Check phase compatibility
    assert!(bcib_instruction.is_phase_compatible());

    // Check capability requirements
    assert_eq!(bcib_instruction.required_capability(), Some(Capability::Execute));
}

/// Test For loop instruction integration
#[test]
fn test_for_loop_bcib_integration() {
    let for_loop = LoopInstruction::For {
        id: LoopID::new("test-for".to_string()),
        range: LoopRange::new(0, 10, 1),
        iterator_var: "i".to_string(),
        body: "for-body-block".to_string(),
        config: LoopConfig::new(Value::Array(vec![]), ValueType::Array),
        location: test_location(),
    };

    let bcib_instruction = BCIBInstruction::Loop(for_loop);

    assert!(bcib_instruction.validate().is_ok());
    assert!(bcib_instruction.is_phase_compatible());
    assert_eq!(bcib_instruction.required_capability(), Some(Capability::Execute));
}

/// Test ForEach loop instruction integration
#[test]
fn test_foreach_loop_bcib_integration() {
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test-foreach".to_string()),
        collection: OperandRef::Literal(Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "foreach-body-block".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: test_location(),
    };

    let bcib_instruction = BCIBInstruction::Loop(foreach_loop);

    assert!(bcib_instruction.validate().is_ok());
    assert!(bcib_instruction.is_phase_compatible());
    assert_eq!(bcib_instruction.required_capability(), Some(Capability::Execute));
}

/// Test control flow instructions integration
#[test]
fn test_control_flow_bcib_integration() {
    // Test Break instruction
    let break_instruction = ControlFlowInstruction::Break {
        location: test_location(),
    };
    let bcib_break = BCIBInstruction::ControlFlow(break_instruction);

    assert!(bcib_break.validate().is_ok());
    assert!(bcib_break.is_phase_compatible());
    assert_eq!(bcib_break.required_capability(), Some(Capability::Execute));

    // Test Continue instruction
    let continue_instruction = ControlFlowInstruction::Continue {
        location: test_location(),
    };
    let bcib_continue = BCIBInstruction::ControlFlow(continue_instruction);

    assert!(bcib_continue.validate().is_ok());
    assert!(bcib_continue.is_phase_compatible());
    assert_eq!(bcib_continue.required_capability(), Some(Capability::Execute));
}

/// Test BCIB sequence with loop instructions
#[test]
fn test_bcib_sequence_with_loops() {
    let instructions = vec![
        BCIBInstruction::Loop(LoopInstruction::For {
            id: LoopID::new("test-for".to_string()),
            range: LoopRange::new(0, 5, 1),
            iterator_var: "i".to_string(),
            body: "for-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: test_location(),
        }),
        BCIBInstruction::ControlFlow(ControlFlowInstruction::Break {
            location: test_location(),
        }),
    ];

    let sequence = BCIBSequence::new(instructions);
    
    // Validate the entire sequence
    assert!(sequence.validate().is_ok());

    // Check required capabilities
    let capabilities = sequence.required_capabilities();
    assert!(capabilities.contains(&Capability::Execute));
}

/// Test serialization and deserialization of loop instructions (Requirements 1.4)
#[test]
fn test_loop_instruction_serialization() {
    let original_loop = LoopInstruction::While {
        id: LoopID::new("serialization-test".to_string()),
        condition: OperandRef::Field("active".to_string()),
        body: "while-body".to_string(),
        config: LoopConfig {
            iteration_limit: 1000,
            budget_timeout: 50000,
            budget_measurement: BudgetMeasurement::IterationCount,
            initial_accumulator: Value::Number(42.0),
            accumulator_type: ValueType::Number,
            error_recovery: ErrorRecoveryPolicy::Abort,
        },
        location: test_location(),
    };

    let bcib_instruction = BCIBInstruction::Loop(original_loop.clone());
    let sequence = BCIBSequence::new(vec![bcib_instruction]);

    // Test JSON serialization round-trip
    let json = sequence.to_json().expect("JSON serialization failed");
    let deserialized_sequence = BCIBSequence::from_json(&json).expect("JSON deserialization failed");

    // Verify the loop instruction is preserved
    assert_eq!(sequence.instructions.len(), deserialized_sequence.instructions.len());
    
    match (&sequence.instructions[0], &deserialized_sequence.instructions[0]) {
        (BCIBInstruction::Loop(original), BCIBInstruction::Loop(deserialized)) => {
            assert_eq!(original, deserialized);
        }
        _ => panic!("Expected loop instructions"),
    }

    // Test binary serialization round-trip
    let binary = sequence.to_binary().expect("Binary serialization failed");
    let deserialized_binary = BCIBSequence::from_binary(&binary).expect("Binary deserialization failed");

    // Verify binary serialization preserves data
    match (&sequence.instructions[0], &deserialized_binary.instructions[0]) {
        (BCIBInstruction::Loop(original), BCIBInstruction::Loop(deserialized)) => {
            assert_eq!(original, deserialized);
        }
        _ => panic!("Expected loop instructions"),
    }
}

/// Test loop configuration validation (Requirements 2.1-2.8, 3.1-3.8)
#[test]
fn test_loop_config_validation() {
    // Valid configuration
    let valid_config = LoopConfig {
        iteration_limit: 5000,
        budget_timeout: 100000,
        budget_measurement: BudgetMeasurement::InstructionCount { weight: 10 },
        initial_accumulator: Value::String("initial".to_string()),
        accumulator_type: ValueType::String,
        error_recovery: ErrorRecoveryPolicy::ReturnPartialResults { include_error_info: true },
    };
    assert!(valid_config.validate().is_ok());

    // Invalid configuration - zero iteration limit
    let invalid_config = LoopConfig {
        iteration_limit: 0,
        budget_timeout: 100000,
        budget_measurement: BudgetMeasurement::IterationCount,
        initial_accumulator: Value::Number(0.0),
        accumulator_type: ValueType::Number,
        error_recovery: ErrorRecoveryPolicy::Abort,
    };
    assert!(invalid_config.validate().is_err());

    // Invalid configuration - exceeds constitutional maximum
    let excessive_config = LoopConfig {
        iteration_limit: 20000, // Exceeds 10,000 limit
        budget_timeout: 100000,
        budget_measurement: BudgetMeasurement::IterationCount,
        initial_accumulator: Value::Number(0.0),
        accumulator_type: ValueType::Number,
        error_recovery: ErrorRecoveryPolicy::Abort,
    };
    assert!(excessive_config.validate().is_err());

    // Invalid configuration - zero budget timeout
    let zero_budget_config = LoopConfig {
        iteration_limit: 1000,
        budget_timeout: 0,
        budget_measurement: BudgetMeasurement::IterationCount,
        initial_accumulator: Value::Number(0.0),
        accumulator_type: ValueType::Number,
        error_recovery: ErrorRecoveryPolicy::Abort,
    };
    assert!(zero_budget_config.validate().is_err());
}

/// Test loop range validation (Requirements 1.8)
#[test]
fn test_loop_range_validation() {
    // Valid ranges
    let valid_range = LoopRange::new(0, 10, 1);
    assert!(valid_range.validate().is_ok());
    assert_eq!(valid_range.iteration_count(), 10);

    let reverse_range = LoopRange::new(10, 0, -1);
    assert!(reverse_range.validate().is_ok());
    assert_eq!(reverse_range.iteration_count(), 10);

    // Invalid range - zero step
    let zero_step_range = LoopRange::new(0, 10, 0);
    assert!(zero_step_range.validate().is_err());

    // Invalid range - infinite loop (positive step, start >= end)
    let infinite_range = LoopRange::new(10, 5, 1);
    assert!(infinite_range.validate().is_err());

    // Invalid range - infinite loop (negative step, start <= end)
    let infinite_reverse_range = LoopRange::new(5, 10, -1);
    assert!(infinite_reverse_range.validate().is_err());
}

/// Test collection type validation (Requirements 1.6, 1.7)
#[test]
fn test_collection_type_validation() {
    // Valid collection types
    assert!(CollectionType::Array.validate().is_ok());
    assert!(CollectionType::List.validate().is_ok());
    assert!(CollectionType::SortedMap.validate().is_ok());

    // Valid hash collections with canonical ordering
    let ordered_hashmap = CollectionType::HashMap { 
        canonical_ordering: Some("key_sort".to_string()) 
    };
    assert!(ordered_hashmap.validate().is_ok());

    let ordered_hashset = CollectionType::HashSet { 
        canonical_ordering: Some("value_sort".to_string()) 
    };
    assert!(ordered_hashset.validate().is_ok());

    // Invalid hash collections without canonical ordering
    let unordered_hashmap = CollectionType::HashMap { 
        canonical_ordering: None 
    };
    assert!(unordered_hashmap.validate().is_err());

    let unordered_hashset = CollectionType::HashSet { 
        canonical_ordering: None 
    };
    assert!(unordered_hashset.validate().is_err());
}

/// Test error recovery policy validation (Requirements 8.6, 8.7, 8.8)
#[test]
fn test_error_recovery_policy_validation() {
    // Valid policies
    assert!(ErrorRecoveryPolicy::Abort.validate().is_ok());
    
    let valid_retry = ErrorRecoveryPolicy::RetryWithIncreasedLimit {
        new_limit: 5000,
        max_retries: 2,
    };
    assert!(valid_retry.validate().is_ok());

    let valid_partial = ErrorRecoveryPolicy::ReturnPartialResults {
        include_error_info: false,
    };
    assert!(valid_partial.validate().is_ok());

    // Invalid retry policy - exceeds constitutional maximum limit
    let excessive_retry = ErrorRecoveryPolicy::RetryWithIncreasedLimit {
        new_limit: 15000, // Exceeds 10,000
        max_retries: 1,
    };
    assert!(excessive_retry.validate().is_err());

    // Invalid retry policy - exceeds maximum retries
    let excessive_retries = ErrorRecoveryPolicy::RetryWithIncreasedLimit {
        new_limit: 5000,
        max_retries: 5, // Exceeds 3
    };
    assert!(excessive_retries.validate().is_err());
}

/// Test loop type detection for parallelization decisions
#[test]
fn test_loop_type_detection() {
    use semantic_cli::bcib::LoopType;

    let while_loop = LoopInstruction::While {
        id: LoopID::new("test".to_string()),
        condition: OperandRef::Literal(Value::Boolean(true)),
        body: "body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: test_location(),
    };
    assert_eq!(while_loop.loop_type(), LoopType::While);

    let for_loop = LoopInstruction::For {
        id: LoopID::new("test".to_string()),
        range: LoopRange::new(0, 10, 1),
        iterator_var: "i".to_string(),
        body: "body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: test_location(),
    };
    assert_eq!(for_loop.loop_type(), LoopType::For);

    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("test".to_string()),
        collection: OperandRef::Literal(Value::Array(vec![])),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: test_location(),
    };
    assert_eq!(foreach_loop.loop_type(), LoopType::ForEach);
}

/// Test control flow type detection
#[test]
fn test_control_flow_type_detection() {
    use semantic_cli::bcib::ControlFlowType;

    let break_instruction = ControlFlowInstruction::Break {
        location: test_location(),
    };
    assert_eq!(break_instruction.control_flow_type(), ControlFlowType::Break);

    let continue_instruction = ControlFlowInstruction::Continue {
        location: test_location(),
    };
    assert_eq!(continue_instruction.control_flow_type(), ControlFlowType::Continue);
}

/// Test Value collection iteration (Requirements 1.6, 1.8)
#[test]
fn test_value_collection_iteration() {
    // Test array iteration (index order: 0, 1, 2, ...)
    let array_value = Value::Array(vec![
        Value::String("first".to_string()),
        Value::String("second".to_string()),
        Value::String("third".to_string()),
    ]);

    assert!(array_value.is_collection());
    assert_eq!(array_value.collection_size(), Some(3));
    assert_eq!(array_value.collection_type(), Some(CollectionType::Array));

    let mut iterator = array_value.iter_collection().unwrap();
    
    let first = iterator.next().unwrap();
    assert_eq!(first.index(), Some(0));
    assert_eq!(first.value(), &Value::String("first".to_string()));

    let second = iterator.next().unwrap();
    assert_eq!(second.index(), Some(1));
    assert_eq!(second.value(), &Value::String("second".to_string()));

    let third = iterator.next().unwrap();
    assert_eq!(third.index(), Some(2));
    assert_eq!(third.value(), &Value::String("third".to_string()));

    assert!(iterator.next().is_none());

    // Test sorted map iteration (key sort order)
    let mut map = std::collections::BTreeMap::new();
    map.insert("zebra".to_string(), Value::Number(3.0));
    map.insert("alpha".to_string(), Value::Number(1.0));
    map.insert("beta".to_string(), Value::Number(2.0));

    let sorted_map_value = Value::SortedMap(map);
    assert!(sorted_map_value.is_collection());
    assert_eq!(sorted_map_value.collection_size(), Some(3));

    let mut map_iterator = sorted_map_value.iter_collection().unwrap();
    
    // Should iterate in key sort order: alpha, beta, zebra
    let first_entry = map_iterator.next().unwrap();
    assert_eq!(first_entry.key(), Some(&"alpha".to_string()));
    assert_eq!(first_entry.value(), &Value::Number(1.0));

    let second_entry = map_iterator.next().unwrap();
    assert_eq!(second_entry.key(), Some(&"beta".to_string()));
    assert_eq!(second_entry.value(), &Value::Number(2.0));

    let third_entry = map_iterator.next().unwrap();
    assert_eq!(third_entry.key(), Some(&"zebra".to_string()));
    assert_eq!(third_entry.value(), &Value::Number(3.0));

    assert!(map_iterator.next().is_none());
}

/// Test complex loop instruction with all features
#[test]
fn test_complex_loop_instruction_integration() {
    let complex_config = LoopConfig {
        iteration_limit: 2500,
        budget_timeout: 75000,
        budget_measurement: BudgetMeasurement::Hybrid { multiplier: 1.5 },
        initial_accumulator: Value::List(vec![Value::String("start".to_string())]),
        accumulator_type: ValueType::List,
        error_recovery: ErrorRecoveryPolicy::RetryWithIncreasedLimit {
            new_limit: 5000,
            max_retries: 2,
        },
    };

    let complex_loop = LoopInstruction::ForEach {
        id: LoopID::new("complex-integration-test".to_string()),
        collection: OperandRef::Field("data_collection".to_string()),
        collection_type: CollectionType::SortedMap,
        iterator_var: "entry".to_string(),
        body: "complex-processing-body".to_string(),
        config: complex_config,
        location: test_location(),
    };

    // Validate the complex loop
    assert!(complex_loop.validate().is_ok());

    // Create BCIB sequence with complex loop and control flow
    let instructions = vec![
        BCIBInstruction::Loop(complex_loop),
        BCIBInstruction::ControlFlow(ControlFlowInstruction::Continue {
            location: test_location(),
        }),
        BCIBInstruction::ControlFlow(ControlFlowInstruction::Break {
            location: test_location(),
        }),
    ];

    let sequence = BCIBSequence::new(instructions);
    assert!(sequence.validate().is_ok());

    // Test serialization of complex sequence
    let json = sequence.to_json().expect("Complex sequence JSON serialization failed");
    let deserialized = BCIBSequence::from_json(&json).expect("Complex sequence JSON deserialization failed");
    
    assert_eq!(sequence.instructions.len(), deserialized.instructions.len());
    
    // Verify all instructions are preserved correctly
    for (original, deserialized) in sequence.instructions.iter().zip(deserialized.instructions.iter()) {
        assert_eq!(original, deserialized);
    }
}