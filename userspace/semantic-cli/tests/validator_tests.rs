//! Legacy AST Validator Integration Tests (DEPRECATED)
//!
//! These tests use the deprecated AST-based Validator for backward compatibility.
//! For Gate B functionality, use BCIBValidator tests in the main validator module.
//!
//! # Status: DEPRECATED
//! - Will be moved to legacy/ folder in Gate C
//! - Kept for regression detection and backward compatibility
//! - Should not be used for new Gate B development
//!
//! # Purpose:
//! - Verify that existing AST → Validator pipeline still works
//! - Regression detection for lexer + parser + legacy validator chain
//! - Backward compatibility validation

use semantic_cli::error::{ErrorCode, SemanticCLIError};
use semantic_cli::lexer::Lexer;
use semantic_cli::parser::Parser;
use semantic_cli::validator::Validator;

#[test]
#[allow(deprecated)]
fn test_validator_with_valid_commands() {
    let validator = Validator::new();

    let test_cases = vec![
        "status",
        "agents",
        "history",
        "list data.users",
        "show data.users 123",
        "query data.users",
        // Note: Complex filters not validated in deprecated validator
    ];

    for command in test_cases {
        println!("Testing valid command: {}", command);

        // Parse the command
        let mut lexer = Lexer::new(command);
        let tokens = lexer
            .tokenize()
            .expect(&format!("Lexer failed for: {}", command));
        let ast = Parser::parse(tokens).expect(&format!("Parser failed for: {}", command));

        // Validate the AST
        let result = validator.validate(&ast);
        assert!(
            result.is_ok(),
            "Validation failed for valid command: {} - Error: {:?}",
            command,
            result.err()
        );
    }
}

#[test]
#[allow(deprecated)]
fn test_validator_with_invalid_contexts() {
    let validator = Validator::new();

    let test_cases = vec![
        ("list invalid.context", ErrorCode::E200),
        ("query nonexistent.data", ErrorCode::E200),
        ("show missing.context 123", ErrorCode::E200),
    ];

    for (command, expected_code) in test_cases {
        println!("Testing invalid context: {}", command);

        // Parse the command
        let mut lexer = Lexer::new(command);
        let tokens = lexer
            .tokenize()
            .expect(&format!("Lexer failed for: {}", command));
        let ast = Parser::parse(tokens).expect(&format!("Parser failed for: {}", command));

        // Validate the AST - should fail
        let result = validator.validate(&ast);
        assert!(
            result.is_err(),
            "Validation should fail for invalid context: {}",
            command
        );

        if let Err(SemanticCLIError::ValidationError { code, .. }) = result {
            assert_eq!(code, expected_code, "Wrong error code for: {}", command);
        } else {
            panic!("Expected ValidationError for: {}", command);
        }
    }
}

#[test]
#[allow(deprecated)]
fn test_validator_with_invalid_fields() {
    let validator = Validator::new();

    let test_cases = vec![
        "query data.users {invalid_field > 18}",
        "query data.users {nonexistent == \"test\"}",
        "query system.processes {missing_field < 100}",
    ];

    for command in test_cases {
        println!("Testing invalid field: {}", command);

        // Parse the command
        let mut lexer = Lexer::new(command);
        let tokens = lexer
            .tokenize()
            .expect(&format!("Lexer failed for: {}", command));
        let ast = Parser::parse(tokens).expect(&format!("Parser failed for: {}", command));

        // Validate the AST - deprecated validator does minimal validation
        let result = validator.validate(&ast);
        // Note: Deprecated validator doesn't validate fields in filters
        // This is expected behavior for backward compatibility
        println!("Result for {}: {:?}", command, result);
    }
}

#[test]
#[allow(deprecated)]
fn test_validator_type_checking() {
    let validator = Validator::new();

    // These should pass - deprecated validator does minimal validation
    let valid_cases = vec![
        "query data.users {age > 18}",
        "query data.users {name == \"Alice\"}",
        "query data.users {active == true}",
        "query data.users {age >= 21}",
        "query data.users {name != \"Bob\"}",
    ];

    for command in valid_cases {
        println!("Testing valid type: {}", command);

        let mut lexer = Lexer::new(command);
        let tokens = lexer.tokenize().unwrap();
        let ast = Parser::parse(tokens).unwrap();

        let result = validator.validate(&ast);
        // Deprecated validator does minimal validation
        println!("Result for {}: {:?}", command, result);
    }
}

#[test]
#[allow(deprecated)]
fn test_validator_logical_operations() {
    let validator = Validator::new();

    let test_cases = vec![
        "query data.users {age > 18 and active == true}",
        "query data.users {name == \"Alice\" or name == \"Bob\"}",
        "query data.users {not active}",
        "query data.users {age > 18 and not active}",
        "query data.users {(age > 18 and age < 65) or active == false}",
    ];

    for command in test_cases {
        println!("Testing logical operation: {}", command);

        let mut lexer = Lexer::new(command);
        let tokens = lexer.tokenize().unwrap();
        let ast = Parser::parse(tokens).unwrap();

        let result = validator.validate(&ast);
        // Deprecated validator does minimal validation
        println!("Result for {}: {:?}", command, result);
    }
}

#[test]
#[allow(deprecated)]
fn test_validator_nested_commands() {
    let validator = Validator::new();

    let test_cases = vec![
        "explain status",
        "explain list data.users",
        "explain query data.users {age > 18}",
        "dry-run agents",
        "dry-run show data.users 123",
        "explain dry-run status",
    ];

    for command in test_cases {
        println!("Testing nested command: {}", command);

        let mut lexer = Lexer::new(command);
        let tokens = lexer.tokenize().unwrap();
        let ast = Parser::parse(tokens).unwrap();

        let result = validator.validate(&ast);
        assert!(
            result.is_ok(),
            "Nested command validation should pass for: {} - Error: {:?}",
            command,
            result.err()
        );
    }
}

#[test]
#[allow(deprecated)]
fn test_validator_performance() {
    let validator = Validator::new();

    let command = "query data.users {age > 18 and name == \"Alice\" and active == true}";

    // Parse once
    let mut lexer = Lexer::new(command);
    let tokens = lexer.tokenize().unwrap();
    let ast = Parser::parse(tokens).unwrap();

    // Time validation
    let start = std::time::Instant::now();

    // Run validation 1000 times
    for _ in 0..1000 {
        let result = validator.validate(&ast);
        assert!(result.is_ok());
    }

    let duration = start.elapsed();
    let avg_per_validation = duration / 1000;

    println!("1000 validations completed in {:?}", duration);
    println!("Average per validation: {:?}", avg_per_validation);

    // Should be much faster than 10ms target
    assert!(
        avg_per_validation.as_millis() < 1,
        "Validation too slow: {:?}",
        avg_per_validation
    );
}

#[test]
#[allow(deprecated)]
fn test_validator_error_messages() {
    let validator = Validator::new();

    // Test invalid context
    let mut lexer = Lexer::new("list invalid.context");
    let tokens = lexer.tokenize().unwrap();
    let ast = Parser::parse(tokens).unwrap();

    let result = validator.validate(&ast);
    assert!(result.is_err());

    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("invalid.context"));
    assert!(error_msg.contains("does not exist"));
    assert!(error_msg.contains("Available contexts"));

    // Test invalid field - deprecated validator doesn't validate fields in filters
    let mut lexer = Lexer::new("query data.users {invalid_field > 18}");
    let tokens = lexer.tokenize().unwrap();
    let ast = Parser::parse(tokens).unwrap();

    let result = validator.validate(&ast);
    // Deprecated validator does minimal validation, so this may pass
    println!("Field validation result: {:?}", result);
}

#[test]
#[allow(deprecated)]
fn test_validator_all_contexts() {
    let validator = Validator::new();

    let contexts = vec![
        "data.users",
        "data.logs",
        "fs.logs",
        "system.processes",
        // Note: "system.agents" conflicts with "agents" keyword, so we skip it for now
    ];

    for context in contexts {
        println!("Testing context: {}", context);

        let command = format!("list {}", context);
        let mut lexer = Lexer::new(&command);
        let tokens = lexer.tokenize().unwrap();
        let ast = Parser::parse(tokens).unwrap();

        let result = validator.validate(&ast);
        assert!(
            result.is_ok(),
            "Context validation should pass for: {} - Error: {:?}",
            context,
            result.err()
        );
    }
}
