//! Transformer Integration Tests
//!
//! Tests the AST → BCIB transformation with architectural requirements (AR-1 to AR-4).

use semantic_cli::ast::{AstNode, BinaryOp, CommandNode, Expr};
use semantic_cli::bcib::{
    BCIBInstruction, Capability, ComparisonOp, ContextInstruction, DebugInstruction,
    FilterExpression, LogicalOperator, OperandRef, QueryInstruction, SystemInstruction,
    SystemScope, Value,
};
use semantic_cli::transformer::Transformer;
use semantic_cli::types::SourceLocation;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1, 0)
}

#[test]
fn test_transformer_end_to_end_query() {
    let mut transformer = Transformer::new();

    // Create AST for: query data.users {age > 18}
    let filter = Expr::Binary {
        left: Box::new(Expr::Identifier {
            name: "age".to_string(),
            location: test_location(),
        }),
        op: BinaryOp::Gt,
        right: Box::new(Expr::Number {
            value: "18".to_string(),
            location: test_location(),
        }),
        location: test_location(),
    };

    let ast = AstNode::new(CommandNode::Query {
        location: test_location(),
        context: vec!["data".to_string(), "users".to_string()],
        filter: Some(filter),
    });

    // Transform to BCIB
    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 3); // LoadContext + ApplyFilter + Return

    // Validate instruction sequence
    match &sequence.instructions[0] {
        BCIBInstruction::Context(ContextInstruction::LoadContext { path, .. }) => {
            assert_eq!(path, "data.users");
        }
        _ => panic!("Expected LoadContext instruction"),
    }

    match &sequence.instructions[1] {
        BCIBInstruction::Query(QueryInstruction::ApplyFilter { expression, .. }) => {
            assert_eq!(expression.field, "age");
            assert_eq!(expression.operator, ComparisonOp::GreaterThan);
            match &expression.value {
                OperandRef::Literal(Value::Number(n)) => assert_eq!(*n, 18.0),
                _ => panic!("Expected number literal operand"),
            }
            assert!(!expression.normalized); // Should not be normalized initially (AR-2)
        }
        _ => panic!("Expected ApplyFilter instruction"),
    }

    match &sequence.instructions[2] {
        BCIBInstruction::Context(ContextInstruction::Return { .. }) => {}
        _ => panic!("Expected Return instruction"),
    }
}

#[test]
fn test_transformer_complex_logical_filter() {
    let mut transformer = Transformer::new();

    // Create AST for: query data.users {age > 18 and active == true}
    let filter = Expr::Binary {
        left: Box::new(Expr::Binary {
            left: Box::new(Expr::Identifier {
                name: "age".to_string(),
                location: test_location(),
            }),
            op: BinaryOp::Gt,
            right: Box::new(Expr::Number {
                value: "18".to_string(),
                location: test_location(),
            }),
            location: test_location(),
        }),
        op: BinaryOp::And,
        right: Box::new(Expr::Binary {
            left: Box::new(Expr::Identifier {
                name: "active".to_string(),
                location: test_location(),
            }),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Boolean {
                value: true,
                location: test_location(),
            }),
            location: test_location(),
        }),
        location: test_location(),
    };

    let ast = AstNode::new(CommandNode::Query {
        location: test_location(),
        context: vec!["data".to_string(), "users".to_string()],
        filter: Some(filter),
    });

    // Transform to BCIB
    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    // Should have multiple instructions for complex filter (flat instruction graph - AR-1)
    assert!(sequence.instructions.len() > 3);

    // First instruction should be LoadContext
    match &sequence.instructions[0] {
        BCIBInstruction::Context(ContextInstruction::LoadContext { path, .. }) => {
            assert_eq!(path, "data.users");
        }
        _ => panic!("Expected LoadContext instruction"),
    }

    // Should contain Compare and LogicalOp instructions (AR-1: Flat instruction graph)
    let has_compare = sequence.instructions.iter().any(|inst| {
        matches!(
            inst,
            BCIBInstruction::Query(QueryInstruction::Compare { .. })
        )
    });
    let has_logical_op = sequence.instructions.iter().any(|inst| {
        matches!(
            inst,
            BCIBInstruction::Query(QueryInstruction::LogicalOp { .. })
        )
    });

    assert!(
        has_compare,
        "Should contain Compare instruction for flat graph"
    );
    assert!(
        has_logical_op,
        "Should contain LogicalOp instruction for flat graph"
    );
}

#[test]
fn test_transformer_system_commands() {
    let mut transformer = Transformer::new();

    // Test status command
    let status_ast = AstNode::new(CommandNode::Status {
        location: test_location(),
    });

    let result = transformer.transform(&status_ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 1);

    match &sequence.instructions[0] {
        BCIBInstruction::System(SystemInstruction::SystemStatus { .. }) => {}
        _ => panic!("Expected SystemStatus instruction"),
    }

    // Test agents command
    let agents_ast = AstNode::new(CommandNode::Agents {
        location: test_location(),
    });

    let result = transformer.transform(&agents_ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 1);

    match &sequence.instructions[0] {
        BCIBInstruction::System(SystemInstruction::ListAgents { .. }) => {}
        _ => panic!("Expected ListAgents instruction"),
    }
}

#[test]
fn test_transformer_debug_commands_with_sequence_references() {
    let mut transformer = Transformer::new();

    // Create target command for explain
    let target_command = CommandNode::Status {
        location: test_location(),
    };

    // Test explain command (AR-3: Sequence references)
    let explain_ast = AstNode::new(CommandNode::Explain {
        location: test_location(),
        command: Box::new(target_command.clone()),
    });

    let result = transformer.transform(&explain_ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 1);

    match &sequence.instructions[0] {
        BCIBInstruction::Debug(DebugInstruction::Explain {
            target_sequence_id, ..
        }) => {
            assert!(!target_sequence_id.is_empty());

            // Verify sequence was registered (AR-3)
            let registry = transformer.sequence_registry();
            let registry_lock = registry.lock().unwrap();
            assert!(registry_lock.contains(target_sequence_id));
        }
        _ => panic!("Expected Explain instruction"),
    }

    // Test dry-run command (AR-3: Sequence references)
    let dry_run_ast = AstNode::new(CommandNode::DryRun {
        location: test_location(),
        command: Box::new(target_command),
    });

    let result = transformer.transform(&dry_run_ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 1);

    match &sequence.instructions[0] {
        BCIBInstruction::Debug(DebugInstruction::DryRun {
            target_sequence_id, ..
        }) => {
            assert!(!target_sequence_id.is_empty());
        }
        _ => panic!("Expected DryRun instruction"),
    }
}

#[test]
fn test_transformer_show_command_with_filter() {
    let mut transformer = Transformer::new();

    let id_expr = Expr::String {
        value: "user123".to_string(),
        location: test_location(),
    };

    let ast = AstNode::new(CommandNode::Show {
        location: test_location(),
        context: vec!["data".to_string(), "users".to_string()],
        id: id_expr,
    });

    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 3); // LoadContext + ApplyFilter + Return

    // Check that show creates an ID filter
    match &sequence.instructions[1] {
        BCIBInstruction::Query(QueryInstruction::ApplyFilter { expression, .. }) => {
            assert_eq!(expression.field, "id");
            assert_eq!(expression.operator, ComparisonOp::Equal);
            match &expression.value {
                OperandRef::Literal(Value::String(s)) => assert_eq!(s, "user123"),
                _ => panic!("Expected string literal operand"),
            }
        }
        _ => panic!("Expected ApplyFilter instruction for show command"),
    }
}

#[test]
fn test_transformer_list_command_no_filter() {
    let mut transformer = Transformer::new();

    let ast = AstNode::new(CommandNode::List {
        location: test_location(),
        context: vec!["fs".to_string(), "logs".to_string()],
    });

    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 2); // LoadContext + Return (no filter)

    match &sequence.instructions[0] {
        BCIBInstruction::Context(ContextInstruction::LoadContext { path, .. }) => {
            assert_eq!(path, "fs.logs");
        }
        _ => panic!("Expected LoadContext instruction"),
    }

    match &sequence.instructions[1] {
        BCIBInstruction::Context(ContextInstruction::Return { .. }) => {}
        _ => panic!("Expected Return instruction"),
    }
}

#[test]
fn test_transformer_history_command() {
    let mut transformer = Transformer::new();

    let ast = AstNode::new(CommandNode::History {
        location: test_location(),
    });

    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert_eq!(sequence.instructions.len(), 1);

    match &sequence.instructions[0] {
        BCIBInstruction::Debug(DebugInstruction::History { .. }) => {}
        _ => panic!("Expected History instruction"),
    }
}

#[test]
fn test_transformer_operand_ref_model_compliance() {
    let mut transformer = Transformer::new();

    // Create a filter that should use OperandRef model (AR-1)
    let filter = Expr::Binary {
        left: Box::new(Expr::Identifier {
            name: "status".to_string(),
            location: test_location(),
        }),
        op: BinaryOp::Eq,
        right: Box::new(Expr::String {
            value: "active".to_string(),
            location: test_location(),
        }),
        location: test_location(),
    };

    let ast = AstNode::new(CommandNode::Query {
        location: test_location(),
        context: vec!["data".to_string(), "users".to_string()],
        filter: Some(filter),
    });

    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();

    // Find the ApplyFilter instruction and verify OperandRef usage
    let filter_instruction = sequence.instructions.iter().find(|inst| {
        matches!(
            inst,
            BCIBInstruction::Query(QueryInstruction::ApplyFilter { .. })
        )
    });

    assert!(filter_instruction.is_some());

    if let Some(BCIBInstruction::Query(QueryInstruction::ApplyFilter { expression, .. })) =
        filter_instruction
    {
        // Verify OperandRef model compliance (AR-1)
        match &expression.value {
            OperandRef::Literal(Value::String(s)) => assert_eq!(s, "active"),
            _ => panic!("Expected OperandRef::Literal for value"),
        }

        // Field should be referenced by name (not OperandRef in FilterExpression)
        assert_eq!(expression.field, "status");
    }
}

#[test]
fn test_transformer_performance() {
    let mut transformer = Transformer::new();

    // Create a moderately complex AST
    let filter = Expr::Binary {
        left: Box::new(Expr::Binary {
            left: Box::new(Expr::Identifier {
                name: "age".to_string(),
                location: test_location(),
            }),
            op: BinaryOp::Ge,
            right: Box::new(Expr::Number {
                value: "18".to_string(),
                location: test_location(),
            }),
            location: test_location(),
        }),
        op: BinaryOp::And,
        right: Box::new(Expr::Binary {
            left: Box::new(Expr::Identifier {
                name: "active".to_string(),
                location: test_location(),
            }),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Boolean {
                value: true,
                location: test_location(),
            }),
            location: test_location(),
        }),
        location: test_location(),
    };

    let ast = AstNode::new(CommandNode::Query {
        location: test_location(),
        context: vec!["data".to_string(), "users".to_string()],
        filter: Some(filter),
    });

    // Measure transformation time
    let start = std::time::Instant::now();
    let result = transformer.transform(&ast);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(
        duration.as_millis() < 50,
        "Transformation should be < 50ms, was {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_transformer_sequence_validation() {
    let mut transformer = Transformer::new();

    let ast = AstNode::new(CommandNode::Query {
        location: test_location(),
        context: vec!["data".to_string(), "users".to_string()],
        filter: None,
    });

    let result = transformer.transform(&ast);
    assert!(result.is_ok());

    let sequence = result.unwrap();

    // Validate the generated sequence
    let validation_result = sequence.validate();
    assert!(
        validation_result.is_ok(),
        "Generated BCIB sequence should be valid"
    );

    // Check required capabilities (AR-4: Contextual capabilities)
    let capabilities = sequence.required_capabilities();
    assert!(
        !capabilities.is_empty(),
        "Sequence should require capabilities"
    );

    // Should require Read capability for data.users context
    let has_read_capability = capabilities
        .iter()
        .any(|cap| matches!(cap, Capability::Read { context } if context == "data.users"));
    assert!(
        has_read_capability,
        "Should require Read capability for data.users context"
    );
}
