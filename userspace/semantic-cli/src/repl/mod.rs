//! Minimal REPL Implementation (Phase 3.5.1.a)
//!
//! This module implements a basic REPL (Read-Eval-Print Loop) for the Semantic CLI.
//! Gate B scope: basic functionality only, no advanced features.
//!
//! # Phase 3.5.1.a Scope
//! - Basic command input/output
//! - Command execution pipeline (Lexer → Parser → Transformer → Executor)
//! - Result display
//! - Basic error handling
//!
//! # NOT INCLUDED (Phase 3.5.1.b)
//! - Tab completion
//! - Syntax highlighting
//! - Command history search
//! - Multi-line editing

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::transformer::Transformer;
use crate::validator::BCIBValidator;
use crate::operations::{QueryExecutor, SystemExecutor, DebugExecutor, OperationExecutor, OperationResult};
use crate::bcib::{BCIBInstruction, BCIBSequenceRegistry};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use rustyline::Editor;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Minimal REPL for Semantic CLI
/// 
/// **Gate B Scope:** Basic functionality only
/// - Command input/output
/// - Execution pipeline
/// - Result display
/// - Error handling
pub struct MinimalREPL {
    /// Command line editor
    editor: Editor<(), rustyline::history::DefaultHistory>,
    /// Transformer for BCIB generation
    transformer: Transformer,
    /// Validator for BCIB validation
    validator: BCIBValidator,
    /// Query executor
    query_executor: QueryExecutor,
    /// System executor
    system_executor: SystemExecutor,
    /// Debug executor
    debug_executor: DebugExecutor,
    /// Sequence registry for debug operations
    sequence_registry: Arc<Mutex<BCIBSequenceRegistry>>,
    /// Command counter for session tracking
    command_count: u32,
}

/// REPL execution result
#[derive(Debug)]
pub struct REPLResult {
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
    pub command_count: u32,
}

impl MinimalREPL {
    /// Create new minimal REPL
    pub fn new() -> Result<Self> {
        let editor = Editor::new()
            .map_err(|e| SemanticCLIError::execution_error(
                format!("Failed to initialize REPL editor: {}", e),
                ErrorCode::E500,
            ))?;

        let sequence_registry = Arc::new(Mutex::new(BCIBSequenceRegistry::new()));
        let transformer = Transformer::with_registry(Arc::clone(&sequence_registry));
        let debug_executor = DebugExecutor::with_registry(Arc::clone(&sequence_registry));

        Ok(Self {
            editor,
            transformer,
            validator: BCIBValidator::new(),
            query_executor: QueryExecutor::new(),
            system_executor: SystemExecutor::new(),
            debug_executor,
            sequence_registry,
            command_count: 0,
        })
    }

    /// Start the REPL loop
    pub fn run(&mut self) -> Result<()> {
        println!("🚀 Semantic CLI v0.1.0 (Phase 3.5.1.a - Gate B)");
        println!("Type 'help' for available commands, 'exit' to quit.\n");

        loop {
            // Read command
            let readline = self.editor.readline("semantic> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    // Handle special commands
                    if line.is_empty() {
                        continue;
                    }
                    
                    if line == "exit" || line == "quit" {
                        println!("Goodbye! 👋");
                        break;
                    }
                    
                    if line == "help" {
                        self.show_help();
                        continue;
                    }

                    if line == "clear" {
                        print!("\x1B[2J\x1B[1;1H"); // Clear screen
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line); // Intentional ignore - UX decision
                    
                    // Execute command
                    let result = self.execute_command(line);
                    self.display_result(result);
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("Goodbye! 👋");
                    break;
                }
                Err(err) => {
                    eprintln!("REPL Error: {}", err);
                    return Err(SemanticCLIError::execution_error(
                        format!("REPL failed: {}", err),
                        ErrorCode::E500,
                    ));
                }
            }
        }

        Ok(())
    }

    /// Execute a single command through the full pipeline
    fn execute_command(&mut self, input: &str) -> REPLResult {
        let start_time = Instant::now();
        self.command_count += 1;

        // Phase 1: Lexical Analysis
        let mut lexer = Lexer::new(input);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                return REPLResult {
                    success: false,
                    output: format!("❌ Lexer Error: {}", e),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    command_count: self.command_count,
                };
            }
        };

        // Phase 2: Syntax Analysis
        let ast = match Parser::parse(tokens) {
            Ok(ast) => ast,
            Err(e) => {
                return REPLResult {
                    success: false,
                    output: format!("❌ Parser Error: {}", e),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    command_count: self.command_count,
                };
            }
        };

        // Phase 3: BCIB Transformation
        let bcib_sequence = match self.transformer.transform(&ast) {
            Ok(sequence) => sequence,
            Err(e) => {
                return REPLResult {
                    success: false,
                    output: format!("❌ Transformer Error: {}", e),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    command_count: self.command_count,
                };
            }
        };

        // Phase 4: BCIB Validation
        if let Err(e) = self.validator.validate_sequence(&bcib_sequence) {
            return REPLResult {
                success: false,
                output: format!("❌ Validation Error: {}", e),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                command_count: self.command_count,
            };
        }

        // Phase 5: Execution
        let execution_result = self.execute_bcib_sequence(&bcib_sequence);
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Add to debug history if successful
        if execution_result.success {
            self.debug_executor.add_to_history(
                bcib_sequence.metadata.sequence_id.clone(),
                input.to_string(),
                true,
                execution_time,
            );
        }

        REPLResult {
            success: execution_result.success,
            output: execution_result.output,
            execution_time_ms: execution_time,
            command_count: self.command_count,
        }
    }

    /// Execute BCIB sequence using appropriate executor
    fn execute_bcib_sequence(&mut self, sequence: &crate::bcib::BCIBSequence) -> REPLResult {
        // Determine operation type from first instruction
        let operation_type = self.determine_operation_type(&sequence.instructions);
        
        match operation_type.as_str() {
            "query" => {
                let input = crate::operations::query::QueryInput {
                    instructions: sequence.instructions.clone(),
                };
                
                match self.query_executor.execute(input) {
                    Ok(result) => REPLResult {
                        success: true,
                        output: result.format(),
                        execution_time_ms: result.execution_time_ms,
                        command_count: self.command_count,
                    },
                    Err(e) => REPLResult {
                        success: false,
                        output: format!("❌ Query Execution Error: {}", e),
                        execution_time_ms: 0,
                        command_count: self.command_count,
                    }
                }
            }
            "system" => {
                let input = crate::operations::system::SystemInput {
                    instructions: sequence.instructions.clone(),
                };
                
                match self.system_executor.execute(input) {
                    Ok(result) => REPLResult {
                        success: true,
                        output: result.format(),
                        execution_time_ms: result.execution_time_ms,
                        command_count: self.command_count,
                    },
                    Err(e) => REPLResult {
                        success: false,
                        output: format!("❌ System Execution Error: {}", e),
                        execution_time_ms: 0,
                        command_count: self.command_count,
                    }
                }
            }
            "debug" => {
                let input = crate::operations::debug::DebugInput {
                    instructions: sequence.instructions.clone(),
                };
                
                match self.debug_executor.execute(input) {
                    Ok(result) => REPLResult {
                        success: true,
                        output: result.format(),
                        execution_time_ms: result.execution_time_ms,
                        command_count: self.command_count,
                    },
                    Err(e) => REPLResult {
                        success: false,
                        output: format!("❌ Debug Execution Error: {}", e),
                        execution_time_ms: 0,
                        command_count: self.command_count,
                    }
                }
            }
            _ => REPLResult {
                success: false,
                output: format!("❌ Unknown operation type: {}", operation_type),
                execution_time_ms: 0,
                command_count: self.command_count,
            }
        }
    }

    /// Determine operation type from BCIB instructions
    fn determine_operation_type(&self, instructions: &[BCIBInstruction]) -> String {
        for instruction in instructions {
            match instruction {
                BCIBInstruction::Query(_) => return "query".to_string(),
                BCIBInstruction::System(_) => return "system".to_string(),
                BCIBInstruction::Debug(_) => return "debug".to_string(),
                BCIBInstruction::Context(_) => continue, // Context instructions don't determine type
                BCIBInstruction::Loop(_) => return "loop".to_string(),
                BCIBInstruction::ControlFlow(_) => return "control_flow".to_string(),
            }
        }
        "unknown".to_string()
    }

    /// Display execution result
    fn display_result(&self, result: REPLResult) {
        if result.success {
            println!("{}", result.output);
            println!("✅ Command #{} completed in {}ms", result.command_count, result.execution_time_ms);
        } else {
            println!("{}", result.output);
            println!("💥 Command #{} failed in {}ms", result.command_count, result.execution_time_ms);
        }
        println!(); // Empty line for readability
    }

    /// Show help information
    fn show_help(&self) {
        println!("📖 Semantic CLI Help (Phase 3.5.1.a - Gate B)");
        println!();
        println!("🔍 Query Operations:");
        println!("  query users where age > 25     - Filter users by age");
        println!("  list users                     - List all users");
        println!("  show user 123                  - Show specific user");
        println!();
        println!("⚙️  System Operations:");
        println!("  status                         - Show system status");
        println!("  agents                         - List active agents");
        println!();
        println!("🐛 Debug Operations:");
        println!("  explain <sequence-id>          - Explain BCIB sequence");
        println!("  dry-run <sequence-id>          - Simulate sequence execution");
        println!("  history                        - Show command history");
        println!();
        println!("🎛️  REPL Commands:");
        println!("  help                           - Show this help");
        println!("  clear                          - Clear screen");
        println!("  exit / quit                    - Exit REPL");
        println!();
        println!("💡 Examples:");
        println!("  semantic> query data.users where role = \"admin\"");
        println!("  semantic> status");
        println!("  semantic> history");
        println!();
        println!("📋 Available Contexts:");
        println!("  data.users, data.logs, fs.logs, system.processes, system.agents");
        println!();
    }
}

impl Default for MinimalREPL {
    fn default() -> Self {
        Self::new().expect("Failed to create default REPL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = MinimalREPL::new();
        assert!(repl.is_ok());
    }

    #[test]
    fn test_operation_type_determination() {
        let repl = MinimalREPL::new().unwrap();
        
        // Query operation
        let query_instructions = vec![
            BCIBInstruction::Context(crate::bcib::ContextInstruction::LoadContext {
                path: "data.users".to_string(),
                location: crate::types::SourceLocation::new(1, 1, 0),
            }),
            BCIBInstruction::Query(crate::bcib::QueryInstruction::ApplyFilter {
                expression: crate::bcib::FilterExpression::new(
                    "age".to_string(),
                    crate::bcib::ComparisonOp::GreaterThan,
                    crate::bcib::OperandRef::Literal(crate::bcib::Value::Number(25.0)),
                ),
                location: crate::types::SourceLocation::new(1, 1, 0),
            }),
        ];
        
        assert_eq!(repl.determine_operation_type(&query_instructions), "query");
        
        // System operation
        let system_instructions = vec![
            BCIBInstruction::System(crate::bcib::SystemInstruction::SystemStatus {
                location: crate::types::SourceLocation::new(1, 1, 0),
            }),
        ];
        
        assert_eq!(repl.determine_operation_type(&system_instructions), "system");
        
        // Debug operation
        let debug_instructions = vec![
            BCIBInstruction::Debug(crate::bcib::DebugInstruction::History {
                location: crate::types::SourceLocation::new(1, 1, 0),
            }),
        ];
        
        assert_eq!(repl.determine_operation_type(&debug_instructions), "debug");
        
        // Unknown operation (context only)
        let context_instructions = vec![
            BCIBInstruction::Context(crate::bcib::ContextInstruction::Return {
                location: crate::types::SourceLocation::new(1, 1, 0),
            }),
        ];
        
        assert_eq!(repl.determine_operation_type(&context_instructions), "unknown");
    }

    #[test]
    fn test_repl_result_creation() {
        let result = REPLResult {
            success: true,
            output: "Test output".to_string(),
            execution_time_ms: 50,
            command_count: 1,
        };
        
        assert!(result.success);
        assert_eq!(result.output, "Test output");
        assert_eq!(result.execution_time_ms, 50);
        assert_eq!(result.command_count, 1);
    }
}