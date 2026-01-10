//! Integration tests for DSL parser hierarchical commands
//! 
//! This module contains comprehensive integration tests to verify that the DSL parser
//! correctly handles all hierarchical command patterns according to AykenOS Phase 2 requirements.

use crate::parser::{DslParser, Command, ParseError};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_hierarchical_workflow() {
        let mut parser = DslParser::new();
        
        // Test complete workflow: context selection -> operations -> batch operations
        
        // 1. Context selection (Level 1: >)
        let result = parser.parse_command("> data.users").unwrap();
        assert_eq!(result.ctx, Some("data.users".to_string()));
        match result.command {
            Command::SelectContext { target } => assert_eq!(target, "data.users"),
            _ => panic!("Expected SelectContext command"),
        }
        
        // 2. Context-specific operations (Level 2: >>)
        let result = parser.parse_command(">> create schema=[id:int,name:string,age:int]").unwrap();
        assert_eq!(result.ctx, Some("data.users".to_string()));
        match result.command {
            Command::Create { schema } => assert_eq!(schema, "[id:int,name:string,age:int]"),
            _ => panic!("Expected Create command"),
        }
        
        let result = parser.parse_command(">> add {\"id\":1,\"name\":\"Alice\",\"age\":30}").unwrap();
        match result.command {
            Command::Add { payload } => assert_eq!(payload, "{\"id\":1,\"name\":\"Alice\",\"age\":30}"),
            _ => panic!("Expected Add command"),
        }
        
        let result = parser.parse_command(">> query filter=\"age > 25\"").unwrap();
        match result.command {
            Command::Query { filter } => assert_eq!(filter, "age > 25"),
            _ => panic!("Expected Query command"),
        }
        
        // 3. Batch operations (Level 3: >[ ])
        let result = parser.parse_command(">[ ] add {\"id\":2,\"name\":\"Bob\"} | query filter=\"name like 'B%'\" | list").unwrap();
        match result.command {
            Command::Batch(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "add {\"id\":2,\"name\":\"Bob\"}");
                assert_eq!(items[1], "query filter=\"name like 'B%'\"");
                assert_eq!(items[2], "list");
            },
            _ => panic!("Expected Batch command"),
        }
    }

    #[test]
    fn test_all_supported_contexts() {
        let mut parser = DslParser::new();
        
        // Test all supported context patterns according to Phase 2 documentation
        let contexts = vec![
            "data.users",
            "data.products", 
            "data.logs",
            "sys.hw",
            "sys.memory",
            "sys.processes",
            "ui.scene.dashboard",
            "ui.scene.terminal",
            "ui.components.chart",
            "ai",
        ];
        
        for context in contexts {
            let result = parser.parse_command(&format!("> {}", context));
            assert!(result.is_ok(), "Context '{}' should be valid", context);
            
            match result.unwrap().command {
                Command::SelectContext { target } => assert_eq!(target, context),
                _ => panic!("Expected SelectContext for '{}'", context),
            }
        }
    }

    #[test]
    fn test_all_context_operations() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.test").unwrap();
        
        // Test all supported context operations
        let operations = vec![
            (">> create schema=[id:int]", "Create"),
            (">> add {\"id\":1}", "Add"),
            (">> query filter=\"id = 1\"", "Query"),
            (">> list", "List"),
            (">> help", "Help"),
            (">> info", "Info"),
            (">> render", "Render"),
            (">> ask \"test question\"", "AiAsk"),
            (">> exit", "Exit"),
        ];
        
        for (cmd, expected_type) in operations {
            let result = parser.parse_command(cmd);
            assert!(result.is_ok(), "Command '{}' should parse successfully", cmd);
            
            let dispatch = result.unwrap();
            assert_eq!(dispatch.ctx, Some("data.test".to_string()));
            
            // Verify command type matches expectation
            match (&dispatch.command, expected_type) {
                (Command::Create { .. }, "Create") => {},
                (Command::Add { .. }, "Add") => {},
                (Command::Query { .. }, "Query") => {},
                (Command::List { .. }, "List") => {},
                (Command::Help { .. }, "Help") => {},
                (Command::Info, "Info") => {},
                (Command::Render, "Render") => {},
                (Command::AiAsk { .. }, "AiAsk") => {},
                (Command::Exit, "Exit") => {},
                _ => panic!("Command type mismatch for '{}': expected {}", cmd, expected_type),
            }
        }
    }

    #[test]
    fn test_hierarchical_error_handling() {
        let mut parser = DslParser::new();
        
        // Test error cases for each hierarchical level
        
        // Level 1 errors (>)
        assert!(matches!(parser.parse_command("> invalid.context"), Err(ParseError::UnsupportedContext(_))));
        assert!(matches!(parser.parse_command(">"), Err(ParseError::InvalidSyntax)));
        
        // Level 2 errors (>>) - no context selected
        assert!(matches!(parser.parse_command(">> add {}"), Err(ParseError::MissingContext)));
        
        // Select context for further tests
        parser.parse_command("> data.test").unwrap();
        
        // Level 2 errors (>>) - invalid operations
        assert!(matches!(parser.parse_command(">> unknown"), Err(ParseError::UnknownAction(_))));
        assert!(matches!(parser.parse_command(">> add"), Err(ParseError::MissingPayload(_))));
        assert!(matches!(parser.parse_command(">> create"), Err(ParseError::MissingPayload(_))));
        assert!(matches!(parser.parse_command(">> query"), Err(ParseError::MissingPayload(_))));
        assert!(matches!(parser.parse_command(">> ask"), Err(ParseError::MissingPayload(_))));
        
        // Level 3 errors (>[ ])
        assert!(matches!(parser.parse_command(">[ ]"), Err(ParseError::MissingPayload(_))));
        assert!(matches!(parser.parse_command(">[ ] "), Err(ParseError::MissingPayload(_))));
    }

    #[test]
    fn test_context_switching_behavior() {
        let mut parser = DslParser::new();
        
        // Test context switching preserves state correctly
        assert_eq!(parser.current_context(), None);
        assert!(!parser.has_context());
        
        // Switch to first context
        parser.parse_command("> data.users").unwrap();
        assert_eq!(parser.current_context(), Some(&"data.users".to_string()));
        assert!(parser.has_context());
        
        // Switch to second context
        parser.parse_command("> sys.hw").unwrap();
        assert_eq!(parser.current_context(), Some(&"sys.hw".to_string()));
        
        // Operations should use current context
        let result = parser.parse_command(">> info").unwrap();
        assert_eq!(result.ctx, Some("sys.hw".to_string()));
        
        // Reset context
        parser.reset_context();
        assert_eq!(parser.current_context(), None);
        assert!(!parser.has_context());
        
        // Operations should fail without context
        assert!(matches!(parser.parse_command(">> info"), Err(ParseError::MissingContext)));
    }

    #[test]
    fn test_batch_operation_parsing() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.test").unwrap();
        
        // Test various batch operation formats
        let batch_tests = vec![
            (">[ ] cmd1", vec!["cmd1"]),
            (">[ ] cmd1 | cmd2", vec!["cmd1", "cmd2"]),
            (">[ ] cmd1 | cmd2 | cmd3", vec!["cmd1", "cmd2", "cmd3"]),
            (">[ ]  cmd1  |  cmd2  |  cmd3  ", vec!["cmd1", "cmd2", "cmd3"]), // with spaces
            (">[ ] query filter=\"test\" | add {\"id\":1} | list", vec!["query filter=\"test\"", "add {\"id\":1}", "list"]),
        ];
        
        for (cmd, expected_items) in batch_tests {
            let result = parser.parse_command(cmd).unwrap();
            match result.command {
                Command::Batch(items) => {
                    assert_eq!(items.len(), expected_items.len(), "Batch command '{}' item count mismatch", cmd);
                    for (i, expected) in expected_items.iter().enumerate() {
                        assert_eq!(items[i], *expected, "Batch command '{}' item {} mismatch", cmd, i);
                    }
                },
                _ => panic!("Expected Batch command for '{}'", cmd),
            }
        }
    }

    #[test]
    fn test_parameter_parsing_edge_cases() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.test").unwrap();
        
        // Test edge cases in parameter parsing
        
        // Schema with various formats
        assert!(parser.parse_command(">> create schema=[id:int,name:string]").is_ok());
        assert!(parser.parse_command(">> create [complex:json,data:array]").is_ok());
        
        // JSON with various formats
        assert!(parser.parse_command(">> add {\"simple\":\"value\"}").is_ok());
        assert!(parser.parse_command(">> add { \"spaced\" : \"value\" }").is_ok());
        assert!(parser.parse_command(">> add {\"nested\":{\"key\":\"value\"}}").is_ok());
        
        // Query filters with various formats
        assert!(parser.parse_command(">> query filter=\"simple = 1\"").is_ok());
        assert!(parser.parse_command(">> query \"complex filter\"").is_ok());
        assert!(parser.parse_command(">> query filter=\"name like 'test%'\"").is_ok());
        
        // AI prompts with various formats
        parser.parse_command("> ai").unwrap();
        assert!(parser.parse_command(">> ask \"Simple question\"").is_ok());
        assert!(parser.parse_command(">> ask \"Complex question with 'quotes' and symbols!\"").is_ok());
        
        // Help and list with optional parameters
        assert!(parser.parse_command(">> help").is_ok());
        assert!(parser.parse_command(">> help commands").is_ok());
        assert!(parser.parse_command(">> list").is_ok());
        assert!(parser.parse_command(">> list data").is_ok());
    }

    #[test]
    fn test_phase2_compliance() {
        let mut parser = DslParser::new();
        
        // Test compliance with Phase 2 documentation requirements
        
        // FR-2.4.1: DSL parser hiyerarşik komutları çözebilmeli (>, >>, >[])
        
        // Test > (context selection)
        assert!(parser.parse_command("> data.users").is_ok());
        assert!(parser.parse_command("> sys.hw").is_ok());
        assert!(parser.parse_command("> ui.scene.dashboard").is_ok());
        assert!(parser.parse_command("> ai").is_ok());
        
        // Test >> (context operations)
        assert!(parser.parse_command(">> create schema=[id:int]").is_ok());
        assert!(parser.parse_command(">> add {\"id\":1}").is_ok());
        assert!(parser.parse_command(">> query filter=\"id=1\"").is_ok());
        assert!(parser.parse_command(">> list").is_ok());
        assert!(parser.parse_command(">> help").is_ok());
        assert!(parser.parse_command(">> info").is_ok());
        assert!(parser.parse_command(">> render").is_ok());
        assert!(parser.parse_command(">> ask \"question\"").is_ok());
        
        // Test >[ ] (batch operations)
        assert!(parser.parse_command(">[ ] cmd1 | cmd2 | cmd3").is_ok());
        
        // Verify error handling for invalid syntax
        assert!(parser.parse_command("invalid").is_err());
        assert!(parser.parse_command(">>> invalid").is_err());
        assert!(parser.parse_command(">[]").is_err());
    }
}