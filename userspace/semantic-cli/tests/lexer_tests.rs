//! Unit tests for lexer functionality
//!
//! Tests:
//! - Valid tokens recognized
//! - Invalid tokens rejected
//! - Source location accurate
//! - Performance benchmarks

use semantic_cli::error::ErrorCode;
use semantic_cli::lexer::{Lexer, TokenKind};

#[test]
fn test_empty_input() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_whitespace_handling() {
    let mut lexer = Lexer::new("  \t  query  \t  ");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // query + EOF
    assert_eq!(tokens[0].kind, TokenKind::Query);
}

#[test]
fn test_all_keywords() {
    let input = "query list show add update delete pipeline status agents orchestrate explain dry-run history permissions sandbox and or not";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Query);
    assert_eq!(tokens[1].kind, TokenKind::List);
    assert_eq!(tokens[2].kind, TokenKind::Show);
    assert_eq!(tokens[3].kind, TokenKind::Add);
    assert_eq!(tokens[4].kind, TokenKind::Update);
    assert_eq!(tokens[5].kind, TokenKind::Delete);
    assert_eq!(tokens[6].kind, TokenKind::Pipeline);
    assert_eq!(tokens[7].kind, TokenKind::Status);
    assert_eq!(tokens[8].kind, TokenKind::Agents);
    assert_eq!(tokens[9].kind, TokenKind::Orchestrate);
    assert_eq!(tokens[10].kind, TokenKind::Explain);
    assert_eq!(tokens[11].kind, TokenKind::DryRun);
    assert_eq!(tokens[12].kind, TokenKind::History);
    assert_eq!(tokens[13].kind, TokenKind::Permissions);
    assert_eq!(tokens[14].kind, TokenKind::Sandbox);
    assert_eq!(tokens[15].kind, TokenKind::And);
    assert_eq!(tokens[16].kind, TokenKind::Or);
    assert_eq!(tokens[17].kind, TokenKind::Not);
}

#[test]
fn test_all_operators() {
    let input = ". , : | { } [ ] ( ) == != < <= > >= + - * /";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Dot);
    assert_eq!(tokens[1].kind, TokenKind::Comma);
    assert_eq!(tokens[2].kind, TokenKind::Colon);
    assert_eq!(tokens[3].kind, TokenKind::Pipe);
    assert_eq!(tokens[4].kind, TokenKind::LBrace);
    assert_eq!(tokens[5].kind, TokenKind::RBrace);
    assert_eq!(tokens[6].kind, TokenKind::LBracket);
    assert_eq!(tokens[7].kind, TokenKind::RBracket);
    assert_eq!(tokens[8].kind, TokenKind::LParen);
    assert_eq!(tokens[9].kind, TokenKind::RParen);
    assert_eq!(tokens[10].kind, TokenKind::Eq);
    assert_eq!(tokens[11].kind, TokenKind::Ne);
    assert_eq!(tokens[12].kind, TokenKind::Lt);
    assert_eq!(tokens[13].kind, TokenKind::Le);
    assert_eq!(tokens[14].kind, TokenKind::Gt);
    assert_eq!(tokens[15].kind, TokenKind::Ge);
    assert_eq!(tokens[16].kind, TokenKind::Plus);
    assert_eq!(tokens[17].kind, TokenKind::Minus);
    assert_eq!(tokens[18].kind, TokenKind::Star);
    assert_eq!(tokens[19].kind, TokenKind::Slash);
}

#[test]
fn test_identifiers() {
    let input = "data users_table my_var _private var123";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Identifier("data".to_string()));
    assert_eq!(
        tokens[1].kind,
        TokenKind::Identifier("users_table".to_string())
    );
    assert_eq!(tokens[2].kind, TokenKind::Identifier("my_var".to_string()));
    assert_eq!(
        tokens[3].kind,
        TokenKind::Identifier("_private".to_string())
    );
    assert_eq!(tokens[4].kind, TokenKind::Identifier("var123".to_string()));
}

#[test]
fn test_string_literals() {
    let input = r#""hello" "world" "hello world""#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::String("world".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::String("hello world".to_string()));
}

#[test]
fn test_string_escape_sequences() {
    let input = r#""line1\nline2" "tab\there" "quote\"test" "backslash\\test""#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::String("line1\nline2".to_string())
    );
    assert_eq!(tokens[1].kind, TokenKind::String("tab\there".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::String("quote\"test".to_string()));
    assert_eq!(
        tokens[3].kind,
        TokenKind::String("backslash\\test".to_string())
    );
}

#[test]
fn test_unterminated_string() {
    let mut lexer = Lexer::new(r#""hello"#);
    let result = lexer.tokenize();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), Some(ErrorCode::E003));
}

#[test]
fn test_string_with_newline() {
    let mut lexer = Lexer::new("\"hello\nworld\"");
    let result = lexer.tokenize();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), Some(ErrorCode::E003));
}

#[test]
fn test_invalid_escape_sequence() {
    let mut lexer = Lexer::new(r#""hello\xworld""#);
    let result = lexer.tokenize();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), Some(ErrorCode::E005));
}

#[test]
fn test_number_integers() {
    let input = "0 42 123 999";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Number("0".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Number("42".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Number("123".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Number("999".to_string()));
}

#[test]
fn test_number_decimals() {
    let input = "0.0 3.14 123.456";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Number("0.0".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Number("3.14".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Number("123.456".to_string()));
}

#[test]
fn test_boolean_literals() {
    let input = "true false";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Boolean(true));
    assert_eq!(tokens[1].kind, TokenKind::Boolean(false));
}

#[test]
fn test_query_command() {
    let input = r#"query data.users {age > 18}"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Query);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("data".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Dot);
    assert_eq!(tokens[3].kind, TokenKind::Identifier("users".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::LBrace);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("age".to_string()));
    assert_eq!(tokens[6].kind, TokenKind::Gt);
    assert_eq!(tokens[7].kind, TokenKind::Number("18".to_string()));
    assert_eq!(tokens[8].kind, TokenKind::RBrace);
    assert_eq!(tokens[9].kind, TokenKind::Eof);
}

#[test]
fn test_list_command() {
    let input = "list data.users";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::List);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("data".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Dot);
    assert_eq!(tokens[3].kind, TokenKind::Identifier("users".to_string()));
}

#[test]
fn test_add_command() {
    let input = r#"add data.users {name: "John", age: 25}"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Add);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("data".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Dot);
    assert_eq!(tokens[3].kind, TokenKind::Identifier("users".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::LBrace);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("name".to_string()));
    assert_eq!(tokens[6].kind, TokenKind::Colon);
    assert_eq!(tokens[7].kind, TokenKind::String("John".to_string()));
    assert_eq!(tokens[8].kind, TokenKind::Comma);
    assert_eq!(tokens[9].kind, TokenKind::Identifier("age".to_string()));
    assert_eq!(tokens[10].kind, TokenKind::Colon);
    assert_eq!(tokens[11].kind, TokenKind::Number("25".to_string()));
    assert_eq!(tokens[12].kind, TokenKind::RBrace);
}

#[test]
fn test_pipeline_command() {
    let input = "pipeline[load data.users | filter {age > 18} | save results]";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Pipeline);
    assert_eq!(tokens[1].kind, TokenKind::LBracket);
    assert_eq!(tokens[2].kind, TokenKind::Identifier("load".to_string()));
    // ... more assertions
}

#[test]
fn test_complex_expression() {
    let input = "(age > 18 and age < 65) or state == \"active\"";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::LParen);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("age".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Gt);
    assert_eq!(tokens[3].kind, TokenKind::Number("18".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::And);
    assert_eq!(tokens[5].kind, TokenKind::Identifier("age".to_string()));
    assert_eq!(tokens[6].kind, TokenKind::Lt);
    assert_eq!(tokens[7].kind, TokenKind::Number("65".to_string()));
    assert_eq!(tokens[8].kind, TokenKind::RParen);
    assert_eq!(tokens[9].kind, TokenKind::Or);
    assert_eq!(tokens[10].kind, TokenKind::Identifier("state".to_string()));
    assert_eq!(tokens[11].kind, TokenKind::Eq);
    assert_eq!(tokens[12].kind, TokenKind::String("active".to_string()));
}

#[test]
fn test_source_location_single_line() {
    let input = "query data.users";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].location.line, 1);
    assert_eq!(tokens[0].location.column, 1);
    assert_eq!(tokens[0].location.offset, 0);

    assert_eq!(tokens[1].location.line, 1);
    assert_eq!(tokens[1].location.column, 7);
    assert_eq!(tokens[1].location.offset, 6);
}

#[test]
fn test_source_location_multi_line() {
    let input = "query\nlist\nshow";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].location.line, 1);
    assert_eq!(tokens[1].location.line, 1); // newline
    assert_eq!(tokens[2].location.line, 2);
    assert_eq!(tokens[3].location.line, 2); // newline
    assert_eq!(tokens[4].location.line, 3);
}

#[test]
fn test_invalid_single_equals() {
    let mut lexer = Lexer::new("age = 18");
    let result = lexer.tokenize();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), Some(ErrorCode::E001));
}

#[test]
fn test_invalid_single_exclamation() {
    let mut lexer = Lexer::new("! true");
    let result = lexer.tokenize();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), Some(ErrorCode::E001));
}

#[test]
fn test_invalid_character() {
    let mut lexer = Lexer::new("@invalid");
    let result = lexer.tokenize();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), Some(ErrorCode::E001));
}

#[test]
fn test_lexeme_preservation() {
    let input = "query data.users";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].lexeme, "query");
    assert_eq!(tokens[1].lexeme, "data");
    assert_eq!(tokens[2].lexeme, ".");
    assert_eq!(tokens[3].lexeme, "users");
}

#[test]
fn test_performance_simple_command() {
    let input = "query data.users {age > 18}";
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize().unwrap();
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_micros() / 1000;

    // Target: < 5ms per tokenization
    // 1000 iterations should take < 5000ms = 5s
    assert!(elapsed.as_secs() < 5, "Lexer too slow: {:?}", elapsed);
    println!("Average tokenization time: {}μs", avg_time);
}

#[test]
fn test_performance_complex_command() {
    let input = r#"pipeline[load data.users | filter {(age > 18 and age < 65) or status == "active"} | transform {name: upper(name)} | save results]"#;
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let mut lexer = Lexer::new(input);
        let _ = lexer.tokenize().unwrap();
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_micros() / 1000;

    // Target: < 5ms per tokenization
    assert!(elapsed.as_secs() < 5, "Lexer too slow: {:?}", elapsed);
    println!("Average tokenization time (complex): {}μs", avg_time);
}
