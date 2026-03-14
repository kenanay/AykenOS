//! Test module for DSL parser
//! This module contains tests to verify the hierarchical DSL parser implementation

use crate::parser::{Command, DslParser, ParseError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_context_selection() {
        let mut parser = DslParser::new();

        // Test valid context selections
        let result = parser.parse_command("> data.users").unwrap();
        assert_eq!(result.ctx, Some("data.users".to_string()));
        match result.command {
            Command::SelectContext { target } => assert_eq!(target, "data.users"),
            _ => panic!("Expected SelectContext command"),
        }

        // Test other valid contexts
        assert!(parser.parse_command("> sys.hw").is_ok());
        assert!(parser.parse_command("> ui.scene.sysdash").is_ok());
        assert!(parser.parse_command("> ai").is_ok());
    }

    #[test]
    fn test_context_commands() {
        let mut parser = DslParser::new();

        // First select a context
        parser.parse_command("> data.users").unwrap();

        // Test create command
        let result = parser
            .parse_command(">> create schema=[id:int,name:string,age:int]")
            .unwrap();
        match result.command {
            Command::Create { schema } => assert_eq!(schema, "[id:int,name:string,age:int]"),
            _ => panic!("Expected Create command"),
        }

        // Test add command
        let result = parser
            .parse_command(">> add {\"id\":1,\"name\":\"Ahmet\",\"age\":34}")
            .unwrap();
        match result.command {
            Command::Add { payload } => {
                assert_eq!(payload, "{\"id\":1,\"name\":\"Ahmet\",\"age\":34}")
            }
            _ => panic!("Expected Add command"),
        }

        // Test query command
        let result = parser
            .parse_command(">> query filter=\"age > 30\"")
            .unwrap();
        match result.command {
            Command::Query { filter } => assert_eq!(filter, "age > 30"),
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_batch_commands() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();

        let result = parser.parse_command(">[ ] cmd1 | cmd2 | cmd3").unwrap();
        match result.command {
            Command::Batch(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "cmd1");
                assert_eq!(items[1], "cmd2");
                assert_eq!(items[2], "cmd3");
            }
            _ => panic!("Expected Batch command"),
        }
    }

    #[test]
    fn test_error_cases() {
        let mut parser = DslParser::new();

        // Test empty input
        assert!(matches!(
            parser.parse_command(""),
            Err(ParseError::EmptyInput)
        ));
        assert!(matches!(
            parser.parse_command("   "),
            Err(ParseError::EmptyInput)
        ));

        // Test invalid syntax
        assert!(matches!(
            parser.parse_command("invalid"),
            Err(ParseError::InvalidSyntax)
        ));

        // Test missing context for context commands
        assert!(matches!(
            parser.parse_command(">> add {}"),
            Err(ParseError::MissingContext)
        ));

        // Test unknown action
        parser.parse_command("> data.users").unwrap();
        assert!(matches!(
            parser.parse_command(">> unknown"),
            Err(ParseError::UnknownAction(_))
        ));

        // Test missing payload
        assert!(matches!(
            parser.parse_command(">> add"),
            Err(ParseError::MissingPayload(_))
        ));
        assert!(matches!(
            parser.parse_command(">> create"),
            Err(ParseError::MissingPayload(_))
        ));
    }

    #[test]
    fn test_ai_commands() {
        let mut parser = DslParser::new();
        parser.parse_command("> ai").unwrap();

        let result = parser
            .parse_command(">> ask \"What is the weather today?\"")
            .unwrap();
        match result.command {
            Command::AiAsk { prompt } => assert_eq!(prompt, "What is the weather today?"),
            _ => panic!("Expected AiAsk command"),
        }
    }

    #[test]
    fn test_help_and_list_commands() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();

        // Test help command
        let result = parser.parse_command(">> help").unwrap();
        match result.command {
            Command::Help { topic } => assert_eq!(topic, None),
            _ => panic!("Expected Help command"),
        }

        let result = parser.parse_command(">> help commands").unwrap();
        match result.command {
            Command::Help { topic } => assert_eq!(topic, Some("commands".to_string())),
            _ => panic!("Expected Help command with topic"),
        }

        // Test list command
        let result = parser.parse_command(">> list").unwrap();
        match result.command {
            Command::List { target } => assert_eq!(target, None),
            _ => panic!("Expected List command"),
        }

        let result = parser.parse_command(">> list data").unwrap();
        match result.command {
            Command::List { target } => assert_eq!(target, Some("data".to_string())),
            _ => panic!("Expected List command with target"),
        }
    }

    #[test]
    fn test_context_validation() {
        let mut parser = DslParser::new();

        // Test valid contexts
        assert!(parser.parse_command("> data.users").is_ok());
        assert!(parser.parse_command("> sys.hw").is_ok());
        assert!(parser.parse_command("> ui.scene.dashboard").is_ok());
        assert!(parser.parse_command("> ai").is_ok());

        // Test invalid contexts
        assert!(matches!(
            parser.parse_command("> invalid.context"),
            Err(ParseError::UnsupportedContext(_))
        ));
        assert!(matches!(
            parser.parse_command("> random"),
            Err(ParseError::UnsupportedContext(_))
        ));
    }

    #[test]
    fn test_json_validation() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();

        // Test valid JSON-like format
        assert!(parser.parse_command(">> add {\"id\":1}").is_ok());
        assert!(parser
            .parse_command(">> add { \"name\": \"test\" }")
            .is_ok());

        // Test invalid JSON-like format
        assert!(matches!(
            parser.parse_command(">> add invalid"),
            Err(ParseError::InvalidJson(_))
        ));
        assert!(matches!(
            parser.parse_command(">> add [1,2,3]"),
            Err(ParseError::InvalidJson(_))
        ));
    }

    #[test]
    fn test_schema_validation() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();

        // Test valid schema format
        assert!(parser
            .parse_command(">> create schema=[id:int,name:string]")
            .is_ok());
        assert!(parser
            .parse_command(">> create [id:int,name:string,age:int]")
            .is_ok());

        // Test invalid schema format
        assert!(matches!(
            parser.parse_command(">> create schema=invalid"),
            Err(ParseError::InvalidSchema(_))
        ));
        assert!(matches!(
            parser.parse_command(">> create invalid"),
            Err(ParseError::InvalidSchema(_))
        ));
    }
}
