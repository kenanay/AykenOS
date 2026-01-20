//! BCIB Integration Tests
//!
//! Tests the BCIB instruction system comprehensively with AR-1 to AR-4 architectural requirements.

use semantic_cli::bcib::*;
use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::types::SourceLocation;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1, 0)
}

#[test]
fn test_bcib_instruction_construction() {
    // Context instruction (AR-4: Contextual capabilities)
    let context_inst = BCIBInstruction::Context(ContextInstruction::LoadContext {
        path: "data.users".to_string(),
        location: test_location(),
    });
    
    assert!(context_inst.validate().is_ok());
    assert!(context_inst.is_phase_compatible());
    assert_eq!(
        context_inst.required_capability(), 
        Some(Capability::Read { context: "data.users".to_string() })
    );

    // Query instruction (AR-2: Updated FilterExpression with OperandRef)
    let query_inst = BCIBInstruction::Query(QueryInstruction::ApplyFilter {
        expression: FilterExpression::new(
            "age".to_string(),
            ComparisonOp::GreaterThan,
            OperandRef::Literal(Value::Number(18.0)),
        ),
        location: test_location(),
    });
    
    assert!(query_inst.validate().is_ok());
    assert!(query_inst.is_phase_compatible());
    assert_eq!(query_inst.required_capability(), None); // Context-dependent

    // System instruction (AR-4: Scoped capabilities)
    let system_inst = BCIBInstruction::System(SystemInstruction::SystemStatus {
        location: test_location(),
    });
    
    assert!(system_inst.validate().is_ok());
    assert!(system_inst.is_phase_compatible());
    assert_eq!(
        system_inst.required_capability(), 
        Some(Capability::System { scope: SystemScope::Status })
    );

    // Debug instruction
    let debug_inst = BCIBInstruction::Debug(DebugInstruction::History {
        location: test_location(),
    });
    
    assert!(debug_inst.validate().is_ok());
    assert!(debug_inst.is_phase_compatible());
    assert_eq!(debug_inst.required_capability(), Some(Capability::Debug));
}

#[test]
fn test_bcib_sequence_complete_workflow() {
    // Create a complete BCIB sequence for: query data.users {age > 18}
    let instructions = vec![
        // Load context (AR-4: Contextual capability)
        BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        }),
        // Apply filter (AR-2: Updated with OperandRef)
        BCIBInstruction::Query(QueryInstruction::ApplyFilter {
            expression: FilterExpression::new(
                "age".to_string(),
                ComparisonOp::GreaterThan,
                OperandRef::Literal(Value::Number(18.0)),
            ),
            location: test_location(),
        }),
        // Return result
        BCIBInstruction::Context(ContextInstruction::Return {
            location: test_location(),
        }),
    ];

    let sequence = BCIBSequence::new(instructions);
    
    // Validate sequence
    assert!(sequence.validate().is_ok());
    
    // Check capabilities (AR-4: Contextual)
    let capabilities = sequence.required_capabilities();
    assert!(capabilities.contains(&Capability::Read { context: "data.users".to_string() }));
    
    // Check metadata
    assert!(!sequence.metadata.sequence_id.is_empty());
    assert_eq!(sequence.metadata.phase, "3.5.1");
    assert_eq!(sequence.metadata.determinism, DeterminismLevel::Deterministic);
}

#[test]
fn test_bcib_complex_query_sequence() {
    // Create BCIB for: query data.users {age > 18 and active == true}
    // AR-1: Flat instruction graph with register generation
    let instructions = vec![
        // Load context (AR-4: Contextual capability)
        BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        }),
        // First comparison: age > 18 (AR-1: using OperandRef with target register)
        BCIBInstruction::Query(QueryInstruction::Compare {
            left: OperandRef::Field("age".to_string()),
            operator: ComparisonOp::GreaterThan,
            right: OperandRef::Literal(Value::Number(18.0)),
            target_register: 0,
            location: test_location(),
        }),
        // Second comparison: active == true (AR-1: using OperandRef with target register)
        BCIBInstruction::Query(QueryInstruction::Compare {
            left: OperandRef::Field("active".to_string()),
            operator: ComparisonOp::Equal,
            right: OperandRef::Literal(Value::Boolean(true)),
            target_register: 1,
            location: test_location(),
        }),
        // Logical AND (AR-1: using OperandRef for temp registers with target register)
        BCIBInstruction::Query(QueryInstruction::LogicalOp {
            operator: LogicalOperator::And,
            operands: vec![OperandRef::TempRegister(0), OperandRef::TempRegister(1)], // Results from comparisons
            target_register: 2,
            location: test_location(),
        }),
        // Apply filter using boolean register (AR-1: Register-based filtering)
        BCIBInstruction::Query(QueryInstruction::ApplyFilterBool {
            filter_register: 2,
            location: test_location(),
        }),
        // Return result
        BCIBInstruction::Context(ContextInstruction::Return {
            location: test_location(),
        }),
    ];

    let sequence = BCIBSequence::new(instructions);
    assert!(sequence.validate().is_ok());
    
    let capabilities = sequence.required_capabilities();
    assert_eq!(capabilities.len(), 1);
    assert!(capabilities.contains(&Capability::Read { context: "data.users".to_string() }));
}

#[test]
fn test_bcib_debug_sequence() {
    // Create BCIB for: explain status (AR-3: Using sequence references)
    let mut registry = BCIBSequenceRegistry::new();
    
    // Create target sequence
    let target_instructions = vec![
        BCIBInstruction::System(SystemInstruction::SystemStatus {
            location: test_location(),
        }),
    ];
    let target_sequence = BCIBSequence::new(target_instructions);
    let target_id = registry.register(target_sequence);

    // Create debug sequence that references the target
    let debug_sequence = vec![
        BCIBInstruction::Debug(DebugInstruction::Explain {
            target_sequence_id: target_id.clone(),
            location: test_location(),
        }),
    ];

    let sequence = BCIBSequence::new(debug_sequence);
    assert!(sequence.validate().is_ok());
    
    let capabilities = sequence.required_capabilities();
    assert!(capabilities.contains(&Capability::Debug));
    
    // Verify registry works
    assert!(registry.contains(&target_id));
    let retrieved = registry.get(&target_id).unwrap();
    assert_eq!(retrieved.instructions.len(), 1);
}

#[test]
fn test_bcib_serialization_comprehensive() {
    // Create a complex sequence (AR-4: Contextual capabilities)
    let instructions = vec![
        BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "system.processes".to_string(),
            location: test_location(),
        }),
        BCIBInstruction::Query(QueryInstruction::ApplyFilter {
            expression: FilterExpression::new(
                "cpu_usage".to_string(),
                ComparisonOp::GreaterThan,
                OperandRef::Literal(Value::Number(80.0)),
            ),
            location: test_location(),
        }),
        BCIBInstruction::Context(ContextInstruction::Return {
            location: test_location(),
        }),
    ];

    let original = BCIBSequence::new(instructions);
    
    // Test JSON serialization
    let json = original.to_json().unwrap();
    assert!(json.contains("system.processes"));
    assert!(json.contains("cpu_usage"));
    assert!(json.contains("GreaterThan"));
    
    let from_json = BCIBSequence::from_json(&json).unwrap();
    assert_eq!(original.instructions.len(), from_json.instructions.len());
    
    // Test binary serialization
    let binary = original.to_binary().unwrap();
    assert!(!binary.is_empty());
    
    let from_binary = BCIBSequence::from_binary(&binary).unwrap();
    assert_eq!(original.instructions.len(), from_binary.instructions.len());
}

#[test]
fn test_bcib_validation_errors() {
    // Empty context path
    let invalid_context = BCIBInstruction::Context(ContextInstruction::LoadContext {
        path: "".to_string(),
        location: test_location(),
    });
    assert!(invalid_context.validate().is_err());

    // Invalid context path format
    let invalid_format = BCIBInstruction::Context(ContextInstruction::LoadContext {
        path: "users".to_string(), // No dot
        location: test_location(),
    });
    assert!(invalid_format.validate().is_err());

    // Invalid logical operation - wrong operand count (AR-1: Updated with target register)
    let invalid_logical = BCIBInstruction::Query(QueryInstruction::LogicalOp {
        operator: LogicalOperator::Not,
        operands: vec![OperandRef::Literal(Value::Boolean(true)), OperandRef::Literal(Value::Boolean(false))], // Should be 1 operand
        target_register: 0,
        location: test_location(),
    });
    assert!(invalid_logical.validate().is_err());

    // Empty debug instruction (AR-3: Updated to use sequence ID)
    let invalid_debug = BCIBInstruction::Debug(DebugInstruction::Explain {
        target_sequence_id: "".to_string(), // Empty sequence ID
        location: test_location(),
    });
    assert!(invalid_debug.validate().is_err());
}

#[test]
fn test_bcib_value_validation() {
    // Valid values (AR-1: Field removed from Value)
    assert!(Value::String("test".to_string()).validate().is_ok());
    assert!(Value::Number(42.0).validate().is_ok());
    assert!(Value::Boolean(true).validate().is_ok());

    // Invalid values
    assert!(Value::Number(f64::NAN).validate().is_err());
    assert!(Value::Number(f64::INFINITY).validate().is_err());
    assert!(Value::Number(f64::NEG_INFINITY).validate().is_err());
}

#[test]
fn test_bcib_filter_expression_validation() {
    // Valid filter (AR-2: Updated with OperandRef)
    let valid_filter = FilterExpression::new(
        "age".to_string(),
        ComparisonOp::GreaterThan,
        OperandRef::Literal(Value::Number(18.0)),
    );
    assert!(valid_filter.validate().is_ok());
    assert!(!valid_filter.normalized); // Default to not normalized

    // Valid normalized filter (AR-2)
    let normalized_filter = FilterExpression::new_normalized(
        "status".to_string(),
        ComparisonOp::Equal,
        OperandRef::Literal(Value::String("active".to_string())),
    );
    assert!(normalized_filter.validate().is_ok());
    assert!(normalized_filter.normalized);

    // Invalid - empty field
    let invalid_filter = FilterExpression::new(
        "".to_string(),
        ComparisonOp::Equal,
        OperandRef::Literal(Value::String("test".to_string())),
    );
    assert!(invalid_filter.validate().is_err());

    // Invalid - NaN value in OperandRef
    let invalid_value_filter = FilterExpression::new(
        "score".to_string(),
        ComparisonOp::GreaterThan,
        OperandRef::Literal(Value::Number(f64::NAN)),
    );
    assert!(invalid_value_filter.validate().is_err());
}

#[test]
fn test_bcib_capability_system() {
    // Test all capability types (AR-4: Contextual capabilities)
    let read_inst = BCIBInstruction::Context(ContextInstruction::LoadContext {
        path: "data.users".to_string(),
        location: test_location(),
    });
    assert_eq!(
        read_inst.required_capability(), 
        Some(Capability::Read { context: "data.users".to_string() })
    );

    let system_inst = BCIBInstruction::System(SystemInstruction::ListAgents {
        location: test_location(),
    });
    assert_eq!(
        system_inst.required_capability(), 
        Some(Capability::System { scope: SystemScope::Agents })
    );

    let debug_inst = BCIBInstruction::Debug(DebugInstruction::DryRun {
        target_sequence_id: "test-sequence-123".to_string(),
        location: test_location(),
    });
    assert_eq!(debug_inst.required_capability(), Some(Capability::Debug));

    // Test sequence capability aggregation
    let sequence = BCIBSequence::new(vec![read_inst, system_inst, debug_inst]);
    let capabilities = sequence.required_capabilities();
    
    assert_eq!(capabilities.len(), 3);
    assert!(capabilities.contains(&Capability::Read { context: "data.users".to_string() }));
    assert!(capabilities.contains(&Capability::System { scope: SystemScope::Agents }));
    assert!(capabilities.contains(&Capability::Debug));
}

#[test]
fn test_bcib_comparison_operators() {
    let operators = vec![
        ComparisonOp::Equal,
        ComparisonOp::NotEqual,
        ComparisonOp::LessThan,
        ComparisonOp::LessThanOrEqual,
        ComparisonOp::GreaterThan,
        ComparisonOp::GreaterThanOrEqual,
    ];

    for op in operators {
        let compare_inst = BCIBInstruction::Query(QueryInstruction::Compare {
            left: OperandRef::Field("age".to_string()),
            operator: op,
            right: OperandRef::Literal(Value::Number(18.0)),
            target_register: 0,
            location: test_location(),
        });
        
        assert!(compare_inst.validate().is_ok());
        assert_eq!(compare_inst.required_capability(), None); // Context-dependent
    }
}

#[test]
fn test_bcib_logical_operators() {
    // Test AND (AR-1: Updated with target register)
    let and_inst = BCIBInstruction::Query(QueryInstruction::LogicalOp {
        operator: LogicalOperator::And,
        operands: vec![OperandRef::Literal(Value::Boolean(true)), OperandRef::Literal(Value::Boolean(false))],
        target_register: 0,
        location: test_location(),
    });
    assert!(and_inst.validate().is_ok());

    // Test OR (AR-1: Updated with target register)
    let or_inst = BCIBInstruction::Query(QueryInstruction::LogicalOp {
        operator: LogicalOperator::Or,
        operands: vec![OperandRef::Literal(Value::Boolean(true)), OperandRef::Literal(Value::Boolean(false))],
        target_register: 1,
        location: test_location(),
    });
    assert!(or_inst.validate().is_ok());

    // Test NOT (AR-1: Updated with target register)
    let not_inst = BCIBInstruction::Query(QueryInstruction::LogicalOp {
        operator: LogicalOperator::Not,
        operands: vec![OperandRef::Literal(Value::Boolean(true))],
        target_register: 2,
        location: test_location(),
    });
    assert!(not_inst.validate().is_ok());
}

#[test]
fn test_bcib_metadata_properties() {
    let sequence = BCIBSequence::new(vec![
        BCIBInstruction::System(SystemInstruction::SystemStatus {
            location: test_location(),
        }),
    ]);

    let metadata = &sequence.metadata;
    
    // Check UUID format (should be valid UUID)
    assert!(uuid::Uuid::parse_str(&metadata.sequence_id).is_ok());
    
    // Check timestamp (should be recent)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(metadata.created_at <= now);
    assert!(metadata.created_at > now - 60); // Within last minute
    
    // Check phase
    assert_eq!(metadata.phase, "3.5.1");
    
    // Check determinism
    assert_eq!(metadata.determinism, DeterminismLevel::Deterministic);
}

#[test]
fn test_bcib_phase_compatibility() {
    // All Phase 3.5.1 instructions should be compatible (AR-4: Updated capabilities)
    let phase_compatible_instructions = vec![
        BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        }),
        BCIBInstruction::Query(QueryInstruction::ApplyFilter {
            expression: FilterExpression::new(
                "age".to_string(),
                ComparisonOp::GreaterThan,
                OperandRef::Literal(Value::Number(18.0)),
            ),
            location: test_location(),
        }),
        BCIBInstruction::System(SystemInstruction::SystemStatus {
            location: test_location(),
        }),
        BCIBInstruction::Debug(DebugInstruction::History {
            location: test_location(),
        }),
    ];

    for instruction in phase_compatible_instructions {
        assert!(instruction.is_phase_compatible(), 
                "Instruction should be Phase 3.5.1 compatible: {:?}", instruction);
    }
}

#[test]
fn test_bcib_error_codes() {
    // Test that validation errors have correct error codes
    let invalid_context = BCIBInstruction::Context(ContextInstruction::LoadContext {
        path: "".to_string(),
        location: test_location(),
    });
    
    let result = invalid_context.validate();
    assert!(result.is_err());
    
    if let Err(SemanticCLIError::ValidationError { code, .. }) = result {
        assert_eq!(code, ErrorCode::E300);
    } else {
        panic!("Expected ValidationError with E300");
    }
}

#[test]
fn test_bcib_performance() {
    // Create a complex sequence (AR-1: Flat instruction graph)
    let instructions = vec![
        BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        }),
        BCIBInstruction::Query(QueryInstruction::Compare {
            left: OperandRef::Field("age".to_string()),
            operator: ComparisonOp::GreaterThan,
            right: OperandRef::Literal(Value::Number(18.0)),
            target_register: 0,
            location: test_location(),
        }),
        BCIBInstruction::Query(QueryInstruction::Compare {
            left: OperandRef::Field("active".to_string()),
            operator: ComparisonOp::Equal,
            right: OperandRef::Literal(Value::Boolean(true)),
            target_register: 1,
            location: test_location(),
        }),
        BCIBInstruction::Query(QueryInstruction::LogicalOp {
            operator: LogicalOperator::And,
            operands: vec![OperandRef::TempRegister(0), OperandRef::TempRegister(1)],
            target_register: 2,
            location: test_location(),
        }),
        BCIBInstruction::Context(ContextInstruction::Return {
            location: test_location(),
        }),
    ];

    let sequence = BCIBSequence::new(instructions);
    
    // Time validation
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        assert!(sequence.validate().is_ok());
    }
    
    let duration = start.elapsed();
    let avg_per_validation = duration / 1000;
    
    println!("1000 BCIB validations completed in {:?}", duration);
    println!("Average per validation: {:?}", avg_per_validation);
    
    // Should be very fast (< 1ms per validation)
    assert!(avg_per_validation.as_millis() < 1, "BCIB validation too slow: {:?}", avg_per_validation);
}