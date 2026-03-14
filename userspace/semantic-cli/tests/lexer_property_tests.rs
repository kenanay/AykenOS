//! Invariant and round-trip tests for lexer
//!
//! Tests lexer correctness through round-trip invariants:
//! 1. Tokenization is reversible (no information loss)
//! 2. Lexemes are preserved correctly
//! 3. Tokenization is idempotent
//!
//! **Note:** These are example-based tests, not property-based tests.
//! For true property-based testing, use proptest or quickcheck.

use semantic_cli::lexer::{Lexer, Token, TokenKind};

/// Detokenize tokens back to exact original string
///
/// This reconstructs the input by concatenating lexemes directly.
/// No spaces added - this is the TRUE round-trip test.
fn detokenize_exact(tokens: &[Token]) -> String {
    tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Eof | TokenKind::Newline))
        .map(|t| t.lexeme.as_str())
        .collect::<String>()
}

#[test]
fn test_round_trip_keywords() {
    let inputs = vec![
        "query",
        "list",
        "show",
        "add",
        "update",
        "delete",
        "pipeline",
        "status",
        "agents",
        "orchestrate",
        "explain",
        "dry-run",
        "history",
        "permissions",
        "sandbox",
        "and",
        "or",
        "not",
    ];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_operators() {
    let inputs = vec![
        ".", ",", ":", "|", "{", "}", "[", "]", "(", ")", "==", "!=", "<", "<=", ">", ">=", "+",
        "-", "*", "/",
    ];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_identifiers() {
    let inputs = vec![
        "data",
        "users",
        "my_var",
        "_private",
        "var123",
        "CamelCase",
        "snake_case",
    ];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_numbers() {
    let inputs = vec!["0", "42", "123", "3.14", "0.0", "123.456"];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_strings() {
    let inputs = vec![
        r#""hello""#,
        r#""world""#,
        r#""hello world""#,
        r#""line1\nline2""#,
        r#""tab\there""#,
        r#""quote\"test""#,
        r#""backslash\\test""#,
    ];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_booleans() {
    let inputs = vec!["true", "false"];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_simple_commands() {
    let inputs = vec!["query", "list", "show", "status", "agents", "history"];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let output = detokenize_exact(&tokens);
        assert_eq!(input, output, "Round-trip failed for: {}", input);
    }
}

#[test]
fn test_round_trip_query_with_filter() {
    let input = "query{age>18}";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let output = detokenize_exact(&tokens);
    assert_eq!(input, output);
}

#[test]
fn test_round_trip_add_command() {
    let input = r#"add{name:"John",age:25}"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let output = detokenize_exact(&tokens);
    assert_eq!(input, output);
}

#[test]
fn test_round_trip_complex_expression() {
    let input = r#"(age>18andage<65)orstate=="active""#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let output = detokenize_exact(&tokens);
    assert_eq!(input, output);
}

#[test]
fn test_round_trip_pipeline() {
    let input = "pipeline[load|filter{age>18}|save]";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let output = detokenize_exact(&tokens);
    assert_eq!(input, output);
}

#[test]
fn test_round_trip_nested_expressions() {
    let input = "((a>1)and(b<2))or(c==3)";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let output = detokenize_exact(&tokens);
    assert_eq!(input, output);
}

#[test]
fn test_round_trip_arithmetic() {
    let input = "a+b*c-d/e";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let output = detokenize_exact(&tokens);
    assert_eq!(input, output);
}

#[test]
fn test_token_count_consistency() {
    let inputs = vec![
        ("query", 2),  // query, EOF
        ("list", 2),   // list, EOF
        ("status", 2), // status, EOF
        ("a+b", 4),    // a, +, b, EOF
    ];

    for (input, expected_count) in inputs {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens.len(),
            expected_count,
            "Token count mismatch for: {}",
            input
        );
    }
}

#[test]
fn test_lexeme_preservation() {
    let input = "query{age>18}";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // Check that lexemes match original tokens
    assert_eq!(tokens[0].lexeme, "query");
    assert_eq!(tokens[1].lexeme, "{");
    assert_eq!(tokens[2].lexeme, "age");
    assert_eq!(tokens[3].lexeme, ">");
    assert_eq!(tokens[4].lexeme, "18");
    assert_eq!(tokens[5].lexeme, "}");
}

#[test]
fn test_source_location_consistency() {
    let input = "query";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // First tokenization
    let locations1: Vec<_> = tokens.iter().map(|t| t.location).collect();

    // Second tokenization
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let locations2: Vec<_> = tokens.iter().map(|t| t.location).collect();

    // Locations should be identical
    assert_eq!(locations1, locations2);
}

#[test]
fn test_idempotent_tokenization() {
    let input = "query{age>18}";

    // First tokenization
    let mut lexer1 = Lexer::new(input);
    let tokens1 = lexer1.tokenize().unwrap();

    // Second tokenization
    let mut lexer2 = Lexer::new(input);
    let tokens2 = lexer2.tokenize().unwrap();

    // Should produce identical tokens
    assert_eq!(tokens1.len(), tokens2.len());
    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.kind, t2.kind);
        assert_eq!(t1.lexeme, t2.lexeme);
        assert_eq!(t1.location, t2.location);
    }
}

#[test]
fn test_all_token_types_covered() {
    // Ensure all token types can be tokenized and round-tripped
    let input = r#"query list show add update delete pipeline status agents orchestrate explain dry-run history permissions sandbox and or not data . , : | { } [ ] ( ) == != < <= > >= + - * / "string" 42 3.14 true false"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // Check that we have a diverse set of token types
    let mut has_keyword = false;
    let mut has_identifier = false;
    let mut has_string = false;
    let mut has_number = false;
    let mut has_boolean = false;
    let mut has_operator = false;
    let mut has_delimiter = false;

    for token in &tokens {
        match &token.kind {
            TokenKind::Query
            | TokenKind::List
            | TokenKind::Show
            | TokenKind::Add
            | TokenKind::Update
            | TokenKind::Delete
            | TokenKind::Pipeline
            | TokenKind::Status
            | TokenKind::Agents
            | TokenKind::Orchestrate
            | TokenKind::Explain
            | TokenKind::DryRun
            | TokenKind::History
            | TokenKind::Permissions
            | TokenKind::Sandbox
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Not => has_keyword = true,
            TokenKind::Identifier(_) => has_identifier = true,
            TokenKind::String(_) => has_string = true,
            TokenKind::Number(_) => has_number = true,
            TokenKind::Boolean(_) => has_boolean = true,
            TokenKind::Dot
            | TokenKind::Comma
            | TokenKind::Colon
            | TokenKind::Pipe
            | TokenKind::Eq
            | TokenKind::Ne
            | TokenKind::Lt
            | TokenKind::Le
            | TokenKind::Gt
            | TokenKind::Ge
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash => has_operator = true,
            TokenKind::LBrace
            | TokenKind::RBrace
            | TokenKind::LBracket
            | TokenKind::RBracket
            | TokenKind::LParen
            | TokenKind::RParen => has_delimiter = true,
            _ => {}
        }
    }

    assert!(has_keyword, "Missing keyword tokens");
    assert!(has_identifier, "Missing identifier tokens");
    assert!(has_string, "Missing string tokens");
    assert!(has_number, "Missing number tokens");
    assert!(has_boolean, "Missing boolean tokens");
    assert!(has_operator, "Missing operator tokens");
    assert!(has_delimiter, "Missing delimiter tokens");
}
