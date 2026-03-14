//! Basic usage example for the AykenOS DSL Parser
//!
//! This example demonstrates how to use the hierarchical DSL parser
//! to parse various AykenOS commands according to Phase 2 specifications.

use dsl_parser::{Command, DslParser, ParseError};

fn main() {
    println!("AykenOS DSL Parser - Basic Usage Example");
    println!("========================================\n");

    let mut parser = DslParser::new();

    // Example commands to demonstrate the parser
    let example_commands = vec![
        // Context selection
        "> data.users",
        "> sys.hw",
        "> ui.scene.dashboard",
        "> ai",
        // Data operations (requires context)
        ">> create schema=[id:int,name:string,age:int]",
        ">> add {\"id\":1,\"name\":\"Ahmet\",\"age\":34}",
        ">> add {\"id\":2,\"name\":\"Ayşe\",\"age\":28}",
        ">> query filter=\"age > 30\"",
        ">> list",
        ">> help",
        // AI operations
        ">> ask \"What is the average age of users?\"",
        // Batch operations
        ">[ ] query filter=\"age > 25\" | query filter=\"name like 'A%'\" | list",
        // System operations
        "> sys.hw",
        ">> info",
        // UI operations
        "> ui.scene.dashboard",
        ">> render",
        // Error cases
        "invalid command",
        ">> add without context",
        ">> unknown_action",
    ];

    for (i, cmd) in example_commands.iter().enumerate() {
        println!("{}. Command: {}", i + 1, cmd);

        match parser.parse_command(cmd) {
            Ok(dispatch_request) => {
                println!("   ✓ Parsed successfully");
                println!("   Context: {:?}", dispatch_request.ctx);
                println!("   Command: {:?}", dispatch_request.command);

                // Show current parser context
                if let Some(ctx) = parser.current_context() {
                    println!("   Current Context: {}", ctx);
                }
            }
            Err(error) => {
                println!("   ✗ Parse error: {}", error);
            }
        }
        println!();
    }

    // Demonstrate context management
    println!("Context Management Demo:");
    println!("=======================");

    println!("Initial context: {:?}", parser.current_context());

    parser.parse_command("> data.products").unwrap();
    println!("After '> data.products': {:?}", parser.current_context());

    parser.reset_context();
    println!("After reset: {:?}", parser.current_context());

    println!("Has context: {}", parser.has_context());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_commands() {
        let mut parser = DslParser::new();

        // Test that basic workflow works
        assert!(parser.parse_command("> data.users").is_ok());
        assert!(parser
            .parse_command(">> create schema=[id:int,name:string]")
            .is_ok());
        assert!(parser
            .parse_command(">> add {\"id\":1,\"name\":\"test\"}")
            .is_ok());
        assert!(parser.parse_command(">> query filter=\"id = 1\"").is_ok());
    }
}
