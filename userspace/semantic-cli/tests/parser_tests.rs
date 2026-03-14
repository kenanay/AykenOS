//! Unit tests for parser functionality
//!
//! Tests parser correctness, error handling, and Phase 3.5.1 compliance.

use semantic_cli::ast::{AstNode, BinaryOp, CommandNode, Expr};
use semantic_cli::error::SemanticCLIError;
use semantic_cli::lexer::Lexer;
use semantic_cli::parser::Parser;

/// Helper function to parse command from string
fn parse_command(input: &str) -> Result<AstNode, SemanticCLIError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    Parser::parse(tokens)
}

/// Helper function to expect parsing error
fn expect_error(input: &str) -> SemanticCLIError {
    parse_command(input).expect_err("Expected parsing to fail")
}

#[test]
fn test_valid_commands_parse_correctly() {
    // Query commands
    let ast = parse_command("query data.users").unwrap();
    match ast.command {
        CommandNode::Query {
            context, filter, ..
        } => {
            assert_eq!(context, vec!["data", "users"]);
            assert!(filter.is_none());
        }
        _ => panic!("Expected Query command"),
    }

    let ast = parse_command("query data.users {age > 18}").unwrap();
    match ast.command {
        CommandNode::Query {
            context, filter, ..
        } => {
            assert_eq!(context, vec!["data", "users"]);
            assert!(filter.is_some());
        }
        _ => panic!("Expected Query command"),
    }

    // List command
    let ast = parse_command("list fs.logs").unwrap();
    match ast.command {
        CommandNode::List { context, .. } => {
            assert_eq!(context, vec!["fs", "logs"]);
        }
        _ => panic!("Expected List command"),
    }

    // Show command
    let ast = parse_command("show data.users 123").unwrap();
    match ast.command {
        CommandNode::Show { context, .. } => {
            assert_eq!(context, vec!["data", "users"]);
        }
        _ => panic!("Expected Show command"),
    }

    // System commands
    parse_command("status").unwrap();
    parse_command("agents").unwrap();

    // Debug commands
    parse_command("explain status").unwrap();
    parse_command("dry-run agents").unwrap();
    parse_command("history").unwrap();
}

#[test]
fn test_invalid_syntax_detected() {
    // Empty input
    expect_error("");

    // Invalid command
    expect_error("invalid");

    // Missing context
    expect_error("query");
    expect_error("list");

    // Missing ID for show
    expect_error("show data.users");

    // Unclosed filter
    expect_error("query data.users {age > 18");

    // Invalid filter
    expect_error("query data.users {@invalid}");
}

#[test]
fn test_extended_dsl_rejected() {
    // Extended keywords should be rejected
    let err = expect_error("add data.users {name: \"Alice\"}");
    assert!(err.to_string().contains("Extended DSL keyword"));
    assert!(err.to_string().contains("Phase 3.5.1"));

    let err = expect_error("update data.users 123 {age: 26}");
    assert!(err.to_string().contains("Extended DSL keyword"));

    let err = expect_error("delete data.users 123");
    assert!(err.to_string().contains("Extended DSL keyword"));

    let err = expect_error("pipeline[load | filter | save]");
    assert!(err.to_string().contains("Extended DSL keyword"));

    let err = expect_error("orchestrate \"analyze logs\"");
    assert!(err.to_string().contains("Extended DSL keyword"));

    let err = expect_error("permissions data.users");
    assert!(err.to_string().contains("Extended DSL keyword"));

    let err = expect_error("sandbox status");
    assert!(err.to_string().contains("Extended DSL keyword"));

    // Extended operators should be rejected
    let err = expect_error("query data.users {age + 5 > 18}");
    assert!(err.to_string().contains("Extended DSL operator"));
    assert!(err.to_string().contains("Phase 3.5.1"));

    let err = expect_error("query data.users {score - penalty < 50}");
    assert!(err.to_string().contains("Extended DSL operator"));

    let err = expect_error("query data.users {count * 2 == 10}");
    assert!(err.to_string().contains("Extended DSL operator"));

    let err = expect_error("query data.users {total / count > 5}");
    assert!(err.to_string().contains("Extended DSL operator"));
}

#[test]
fn test_error_messages_helpful() {
    // Syntax errors should have helpful messages
    let err = expect_error("query");
    assert!(err.to_string().contains("expected identifier"));

    let err = expect_error("invalid");
    assert!(err.to_string().contains("unexpected token"));
    assert!(err.to_string().contains("expected command"));

    let err = expect_error("query data.users {");
    assert!(err.to_string().contains("unexpected"));

    // Extended DSL errors should suggest alternatives
    let err = expect_error("add data.users {name: \"Alice\"}");
    assert!(err.to_string().contains("Phase 3.5.2+"));

    let err = expect_error("query data.users {age + 5 > 18}");
    assert!(err.to_string().contains("comparison operators only"));
}

#[test]
fn test_complex_expressions() {
    let ast =
        parse_command("query data.users {age > 18 and name == \"Alice\" or not active}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => {
            // Should parse as: ((age > 18) and (name == "Alice")) or (not active)
            match expr {
                Expr::Binary {
                    op: BinaryOp::Or, ..
                } => {}
                _ => panic!("Expected Or as top-level operator"),
            }
        }
        _ => panic!("Expected Query with filter"),
    }
}

#[test]
fn test_nested_commands() {
    let ast = parse_command("explain query data.users {age > 18}").unwrap();
    match ast.command {
        CommandNode::Explain { command, .. } => match command.as_ref() {
            CommandNode::Query { .. } => {}
            _ => panic!("Expected Query inside Explain"),
        },
        _ => panic!("Expected Explain command"),
    }

    let ast = parse_command("dry-run explain status").unwrap();
    match ast.command {
        CommandNode::DryRun { command, .. } => match command.as_ref() {
            CommandNode::Explain { command, .. } => match command.as_ref() {
                CommandNode::Status { .. } => {}
                _ => panic!("Expected Status inside Explain inside DryRun"),
            },
            _ => panic!("Expected Explain inside DryRun"),
        },
        _ => panic!("Expected DryRun command"),
    }
}

#[test]
fn test_context_paths() {
    // Simple context
    let ast = parse_command("list users").unwrap();
    match ast.command {
        CommandNode::List { context, .. } => {
            assert_eq!(context, vec!["users"]);
        }
        _ => panic!("Expected List command"),
    }

    // Nested context
    let ast = parse_command("list data.users.active").unwrap();
    match ast.command {
        CommandNode::List { context, .. } => {
            assert_eq!(context, vec!["data", "users", "active"]);
        }
        _ => panic!("Expected List command"),
    }

    // Deep nesting
    let ast = parse_command("query system.processes.running.high_cpu").unwrap();
    match ast.command {
        CommandNode::Query { context, .. } => {
            assert_eq!(context, vec!["system", "processes", "running", "high_cpu"]);
        }
        _ => panic!("Expected Query command"),
    }
}

#[test]
fn test_expression_types() {
    let ast = parse_command("query data.users {age > 18}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Binary {
                left, op, right, ..
            } => {
                assert_eq!(op, BinaryOp::Gt);
                match left.as_ref() {
                    Expr::Identifier { name, .. } => assert_eq!(name, "age"),
                    _ => panic!("Expected identifier"),
                }
                match right.as_ref() {
                    Expr::Number { value, .. } => assert_eq!(value, "18"),
                    _ => panic!("Expected number"),
                }
            }
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected Query with filter"),
    }

    let ast = parse_command("query data.users {name == \"Alice\"}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Binary { right, .. } => match right.as_ref() {
                Expr::String { value, .. } => assert_eq!(value, "Alice"),
                _ => panic!("Expected string"),
            },
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected Query with filter"),
    }

    let ast = parse_command("query data.users {active == true}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Binary { right, .. } => match right.as_ref() {
                Expr::Boolean { value, .. } => assert!(*value),
                _ => panic!("Expected boolean"),
            },
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected Query with filter"),
    }
}

#[test]
fn test_operator_precedence() {
    // Test that comparison has higher precedence than logical operators
    let ast = parse_command("query data.users {age > 18 and active == true}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Binary {
                op: BinaryOp::And,
                left,
                right,
                ..
            } => {
                // Left should be (age > 18)
                match left.as_ref() {
                    Expr::Binary {
                        op: BinaryOp::Gt, ..
                    } => {}
                    _ => panic!("Expected Gt as left operand of And"),
                }
                // Right should be (active == true)
                match right.as_ref() {
                    Expr::Binary {
                        op: BinaryOp::Eq, ..
                    } => {}
                    _ => panic!("Expected Eq as right operand of And"),
                }
            }
            _ => panic!("Expected And expression"),
        },
        _ => panic!("Expected Query with filter"),
    }
}

#[test]
fn test_unary_operators() {
    let ast = parse_command("query data.users {not active}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Unary { operand, .. } => match operand.as_ref() {
                Expr::Identifier { name, .. } => assert_eq!(name, "active"),
                _ => panic!("Expected identifier"),
            },
            _ => panic!("Expected unary expression"),
        },
        _ => panic!("Expected Query with filter"),
    }
}

#[test]
fn test_parentheses() {
    let ast = parse_command("query data.users {(age > 18) and (name == \"Alice\")}").unwrap();
    match ast.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Binary {
                op: BinaryOp::And, ..
            } => {}
            _ => panic!("Expected And expression"),
        },
        _ => panic!("Expected Query with filter"),
    }
}

#[test]
fn test_source_location_preserved() {
    let ast = parse_command("query data.users").unwrap();
    let location = ast.location();
    assert_eq!(location.line, 1);
    assert_eq!(location.column, 1);
}

#[test]
fn test_performance_benchmark() {
    use std::time::Instant;

    let commands = vec![
        "status",
        "agents",
        "query data.users",
        "query data.users {age > 18}",
        "query data.users {age > 18 and name == \"Alice\" or not active}",
        "list fs.logs",
        "show system.processes 123",
        "explain query data.users {age > 18}",
        "dry-run status",
        "history",
    ];

    for command in commands {
        let start = Instant::now();
        let result = parse_command(command);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Command '{}' should parse successfully",
            command
        );
        assert!(
            duration.as_millis() < 5,
            "Parse time should be < 5ms, got {}ms for '{}'",
            duration.as_millis(),
            command
        );
    }
}
