//! Property tests for parser correctness
//!
//! Tests the property: parse → unparse → parse (round-trip)
//! This ensures that parsing is deterministic and preserves information.

use semantic_cli::ast::{AstNode, BinaryOp, CommandNode, Expr, UnaryOp};
use semantic_cli::error::SemanticCLIError;
use semantic_cli::lexer::Lexer;
use semantic_cli::parser::Parser;

/// Helper function to parse command from string
fn parse_command(input: &str) -> Result<AstNode, SemanticCLIError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    Parser::parse(tokens)
}

/// Unparse AST back to string (for round-trip testing)
fn unparse_ast(ast: &AstNode) -> String {
    unparse_command(&ast.command)
}

/// Unparse command node to string
fn unparse_command(cmd: &CommandNode) -> String {
    match cmd {
        CommandNode::Query {
            context, filter, ..
        } => {
            let mut result = format!("query {}", context.join("."));
            if let Some(filter) = filter {
                result.push_str(&format!(" {{{}}}", unparse_expr(filter)));
            }
            result
        }
        CommandNode::List { context, .. } => {
            format!("list {}", context.join("."))
        }
        CommandNode::Show { context, id, .. } => {
            format!("show {} {}", context.join("."), unparse_expr(id))
        }
        CommandNode::Status { .. } => "status".to_string(),
        CommandNode::Agents { .. } => "agents".to_string(),
        CommandNode::Explain { command, .. } => {
            format!("explain {}", unparse_command(command))
        }
        CommandNode::DryRun { command, .. } => {
            format!("dry-run {}", unparse_command(command))
        }
        CommandNode::History { .. } => "history".to_string(),
    }
}

/// Unparse expression to string
fn unparse_expr(expr: &Expr) -> String {
    match expr {
        Expr::Identifier { name, .. } => name.clone(),
        Expr::Number { value, .. } => value.clone(),
        Expr::String { value, .. } => format!("\"{}\"", value),
        Expr::Boolean { value, .. } => value.to_string(),
        Expr::Binary {
            left, op, right, ..
        } => {
            let op_str = match op {
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
            };
            format!("{} {} {}", unparse_expr(left), op_str, unparse_expr(right))
        }
        Expr::Unary { op, operand, .. } => {
            let op_str = match op {
                UnaryOp::Not => "not",
            };
            format!("{} {}", op_str, unparse_expr(operand))
        }
    }
}

#[test]
fn test_round_trip_simple_commands() {
    let test_cases = vec![
        "status",
        "agents",
        "history",
        "list data.users",
        "show data.users 123",
        "query data.users",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_query_with_filters() {
    let test_cases = vec![
        "query data.users {age > 18}",
        "query data.users {name == \"Alice\"}",
        "query data.users {active == true}",
        "query data.users {count != 0}",
        "query data.users {score >= 80}",
        "query data.users {age <= 65}",
        "query data.users {rating < 5}",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_logical_expressions() {
    let test_cases = vec![
        "query data.users {age > 18 and active == true}",
        "query data.users {name == \"Alice\" or name == \"Bob\"}",
        "query data.users {not active}",
        "query data.users {age > 18 and name == \"Alice\" or not active}",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_nested_commands() {
    let test_cases = vec![
        "explain status",
        "explain agents",
        "explain query data.users",
        "dry-run status",
        "dry-run list data.users",
        "explain dry-run status",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_complex_contexts() {
    let test_cases = vec![
        "list system.processes.running",
        "query data.users.active.premium",
        "show fs.logs.error.recent 123",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_preserves_semantics() {
    // Test that operator precedence is preserved
    let input = "query data.users {age > 18 and name == \"Alice\" or not active}";
    let ast1 = parse_command(input).unwrap();
    let unparsed = unparse_ast(&ast1);
    let ast2 = parse_command(&unparsed).unwrap();

    assert_eq!(ast1, ast2);

    // Verify the structure is correct (OR at top level)
    match ast1.command {
        CommandNode::Query {
            filter: Some(expr), ..
        } => match expr {
            Expr::Binary {
                op: BinaryOp::Or, ..
            } => {}
            _ => panic!("Expected Or as top-level operator"),
        },
        _ => panic!("Expected Query with filter"),
    }
}

#[test]
fn test_round_trip_source_location_consistency() {
    let input = "query data.users {age > 18}";
    let ast1 = parse_command(input).unwrap();
    let unparsed = unparse_ast(&ast1);
    let ast2 = parse_command(&unparsed).unwrap();

    // AST structure should be identical
    assert_eq!(ast1, ast2);

    // Both should have valid source locations
    assert!(ast1.location().line > 0);
    assert!(ast2.location().line > 0);
}

#[test]
fn test_idempotent_parsing() {
    // Multiple parse cycles should produce identical results
    let input = "query data.users {age > 18 and name == \"Alice\"}";

    let mut current = input.to_string();
    let mut asts = Vec::new();

    // Parse → unparse → parse multiple times
    for _ in 0..5 {
        let ast = parse_command(&current).unwrap();
        asts.push(ast.clone());
        current = unparse_ast(&ast);
    }

    // All ASTs should be identical
    for ast in &asts[1..] {
        assert_eq!(&asts[0], ast, "Parsing is not idempotent");
    }
}

#[test]
fn test_round_trip_all_token_types() {
    // Test that all Core DSL token types survive round-trip
    let test_cases = vec![
        // All commands
        "status",
        "agents",
        "history",
        "list data.users",
        "show data.users 123",
        "query data.users",
        "explain status",
        "dry-run agents",
        // All operators
        "query data.users {age == 18}",
        "query data.users {age != 18}",
        "query data.users {age < 18}",
        "query data.users {age <= 18}",
        "query data.users {age > 18}",
        "query data.users {age >= 18}",
        "query data.users {active and premium}",
        "query data.users {active or premium}",
        "query data.users {not active}",
        // All literal types
        "query data.users {name == \"Alice\"}",
        "query data.users {age == 25}",
        "query data.users {score == 3.14}",
        "query data.users {active == true}",
        "query data.users {inactive == false}",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_edge_cases() {
    let test_cases = vec![
        // Empty string literals
        "query data.users {name == \"\"}",
        // Single character identifiers
        "query a {b == c}",
        // Numbers with decimals
        "query data.users {score == 0.0}",
        "query data.users {ratio == 1.5}",
        // Complex nested contexts
        "query a.b.c.d.e.f",
        // Deeply nested commands
        "explain dry-run query data.users {age > 18}",
    ];

    for input in test_cases {
        let ast1 = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast1);
        let ast2 = parse_command(&unparsed).unwrap();

        assert_eq!(ast1, ast2, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_performance() {
    use std::time::Instant;

    let input = "query data.users {age > 18 and name == \"Alice\" or not active}";

    let start = Instant::now();
    for _ in 0..100 {
        let ast = parse_command(input).unwrap();
        let unparsed = unparse_ast(&ast);
        let _ast2 = parse_command(&unparsed).unwrap();
    }
    let duration = start.elapsed();

    // 100 round-trips should complete in reasonable time
    assert!(
        duration.as_millis() < 100,
        "Round-trip performance too slow: {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_extended_dsl_rejection_consistency() {
    // Extended DSL tokens should be consistently rejected
    let extended_commands = vec![
        "add data.users {name: \"Alice\"}",
        "update data.users 123 {age: 26}",
        "delete data.users 123",
        "pipeline[load | filter | save]",
        "orchestrate \"analyze logs\"",
        "permissions data.users",
        "sandbox status",
    ];

    let extended_operators = vec![
        "query data.users {age + 5 > 18}",
        "query data.users {score - penalty < 50}",
        "query data.users {count * 2 == 10}",
        "query data.users {total / count > 5}",
    ];

    for input in extended_commands.iter().chain(extended_operators.iter()) {
        let result = parse_command(input);
        assert!(
            result.is_err(),
            "Extended DSL should be rejected: {}",
            input
        );

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Extended DSL") || err.to_string().contains("Phase 3.5.1")
        );
    }
}
