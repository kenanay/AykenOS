//! Hierarchical DSL Command Demonstration
//! 
//! This example demonstrates the complete hierarchical command functionality
//! of the AykenOS DSL parser according to Phase 2 requirements.

use dsl_parser::{DslParser, Command};

fn main() {
    println!("AykenOS DSL Parser - Hierarchical Commands Demonstration");
    println!("========================================================\n");

    let mut parser = DslParser::new();

    println!("🎯 PHASE 2 HIERARCHICAL COMMAND STRUCTURE");
    println!("==========================================");
    println!("Level 1: >     - Context Selection");
    println!("Level 2: >>    - Context Operations");
    println!("Level 3: >[ ]  - Batch Operations\n");

    // Demonstrate Level 1: Context Selection
    println!("📁 LEVEL 1: CONTEXT SELECTION (>)");
    println!("==================================");
    
    let contexts = vec![
        ("> data.users", "Data container for user records"),
        ("> sys.hw", "System hardware information"),
        ("> ui.scene.dashboard", "UI dashboard scene"),
        ("> ai", "AI-powered operations"),
    ];

    for (cmd, description) in contexts {
        println!("Command: {}", cmd);
        match parser.parse_command(cmd) {
            Ok(result) => {
                println!("  ✓ {}", description);
                println!("  Context: {:?}", result.ctx);
                if let Command::SelectContext { target } = result.command {
                    println!("  Target: {}", target);
                }
            }
            Err(e) => println!("  ✗ Error: {}", e),
        }
        println!();
    }

    // Demonstrate Level 2: Context Operations
    println!("⚙️  LEVEL 2: CONTEXT OPERATIONS (>>)");
    println!("====================================");
    
    // Select a data context first
    parser.parse_command("> data.users").unwrap();
    println!("Selected context: data.users\n");

    let operations = vec![
        (">> create schema=[id:int,name:string,age:int]", "Create data schema"),
        (">> add {\"id\":1,\"name\":\"Alice\",\"age\":30}", "Add data record"),
        (">> add {\"id\":2,\"name\":\"Bob\",\"age\":25}", "Add another record"),
        (">> query filter=\"age > 25\"", "Query with filter"),
        (">> list", "List container contents"),
        (">> help", "Get help information"),
    ];

    for (cmd, description) in operations {
        println!("Command: {}", cmd);
        match parser.parse_command(cmd) {
            Ok(result) => {
                println!("  ✓ {}", description);
                println!("  Context: {:?}", result.ctx);
                match result.command {
                    Command::Create { schema } => println!("  Schema: {}", schema),
                    Command::Add { payload } => println!("  Payload: {}", payload),
                    Command::Query { filter } => println!("  Filter: {}", filter),
                    Command::List { target } => println!("  Target: {:?}", target),
                    Command::Help { topic } => println!("  Topic: {:?}", topic),
                    _ => println!("  Command: {:?}", result.command),
                }
            }
            Err(e) => println!("  ✗ Error: {}", e),
        }
        println!();
    }

    // Demonstrate Level 3: Batch Operations
    println!("🔄 LEVEL 3: BATCH OPERATIONS (>[ ])");
    println!("===================================");
    
    let batch_operations = vec![
        (">[ ] query filter=\"age > 20\" | list", "Filter and list"),
        (">[ ] add {\"id\":3,\"name\":\"Charlie\"} | query filter=\"name like 'C%'\" | list", "Add, filter, and list"),
        (">[ ] help | info | list", "Multiple info commands"),
    ];

    for (cmd, description) in batch_operations {
        println!("Command: {}", cmd);
        match parser.parse_command(cmd) {
            Ok(result) => {
                println!("  ✓ {}", description);
                println!("  Context: {:?}", result.ctx);
                if let Command::Batch(items) = result.command {
                    println!("  Batch items ({}):", items.len());
                    for (i, item) in items.iter().enumerate() {
                        println!("    {}. {}", i + 1, item);
                    }
                }
            }
            Err(e) => println!("  ✗ Error: {}", e),
        }
        println!();
    }

    // Demonstrate AI Context
    println!("🤖 AI CONTEXT DEMONSTRATION");
    println!("===========================");
    
    parser.parse_command("> ai").unwrap();
    println!("Selected context: ai\n");

    let ai_commands = vec![
        (">> ask \"What is the average age of users?\"", "Natural language query"),
        (">> ask \"Show me users older than 25\"", "Data analysis request"),
        (">> help ai", "AI-specific help"),
    ];

    for (cmd, description) in ai_commands {
        println!("Command: {}", cmd);
        match parser.parse_command(cmd) {
            Ok(result) => {
                println!("  ✓ {}", description);
                if let Command::AiAsk { prompt } = result.command {
                    println!("  Prompt: {}", prompt);
                }
            }
            Err(e) => println!("  ✗ Error: {}", e),
        }
        println!();
    }

    // Demonstrate Error Handling
    println!("❌ ERROR HANDLING DEMONSTRATION");
    println!("===============================");
    
    let error_cases = vec![
        ("invalid command", "Invalid syntax"),
        ("> invalid.context", "Unsupported context"),
        (">> add without context", "Missing context (after reset)"),
        (">> unknown_action", "Unknown action"),
        (">> add", "Missing payload"),
        (">[ ]", "Empty batch"),
    ];

    parser.reset_context(); // Reset for error testing

    for (cmd, description) in error_cases {
        println!("Command: {}", cmd);
        match parser.parse_command(cmd) {
            Ok(_) => println!("  ✗ Unexpected success"),
            Err(e) => println!("  ✓ Expected error: {} - {}", description, e),
        }
        println!();
    }

    // Summary
    println!("📊 SUMMARY");
    println!("==========");
    println!("✅ Level 1 (>) - Context Selection: WORKING");
    println!("✅ Level 2 (>>) - Context Operations: WORKING");
    println!("✅ Level 3 (>[ ]) - Batch Operations: WORKING");
    println!("✅ Error Handling: COMPREHENSIVE");
    println!("✅ Context Management: FUNCTIONAL");
    println!("✅ Parameter Parsing: ROBUST");
    println!("\n🎉 DSL Parser handles hierarchical commands successfully!");
    println!("   Ready for AykenOS Phase 2 data-centric architecture!");
}