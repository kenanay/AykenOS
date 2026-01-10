//! # AykenOS Hierarchical DSL Parser
//! 
//! This module implements the hierarchical Domain Specific Language (DSL) parser
//! for AykenOS Phase 2 according to the data-centric architecture specification.
//! 
//! ## Grammar
//! 
//! The DSL supports three levels of command hierarchy:
//! 
//! - `>` : Context selection (e.g., `> data.users`, `> sys.hw`, `> ai`)
//! - `>>` : Context-specific actions (e.g., `>> add {...}`, `>> query filter=...`)
//! - `>[ ]` : Batch/parallel operations (e.g., `>[ ] cmd1 | cmd2 | cmd3`)
//! 
//! ## Examples
//! 
//! ```rust
//! use dsl_parser::DslParser;
//! 
//! let mut parser = DslParser::new();
//! 
//! // Select a data context
//! let result = parser.parse_command("> data.users").unwrap();
//! 
//! // Create a schema in the selected context
//! let result = parser.parse_command(">> create schema=[id:int,name:string,age:int]").unwrap();
//! 
//! // Add data to the container
//! let result = parser.parse_command(">> add {\"id\":1,\"name\":\"Ahmet\",\"age\":34}").unwrap();
//! 
//! // Query the data
//! let result = parser.parse_command(">> query filter=\"age > 30\"").unwrap();
//! ```
//! 
//! ## Supported Contexts
//! 
//! - `data.*` : Data container operations (tabular, text, etc.)
//! - `sys.*` : System information and hardware operations
//! - `ui.*` : User interface and rendering operations
//! - `ai` : AI-powered operations and queries
//! 
//! ## Supported Actions
//! 
//! - `create` : Create new data containers with schema
//! - `add` : Add data to containers (JSON format)
//! - `query` : Query data with filters
//! - `list` : List available containers or metadata
//! - `help` : Get help information
//! - `info` : Get context-specific information
//! - `render` : Render UI components
//! - `ask` : Ask AI-powered questions
//! - `exit`/`quit` : Exit commands

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Context selection
    SelectContext { target: String },
    
    // Data operations
    Create { schema: String },
    Add { payload: String },
    Query { filter: String },
    
    // System operations
    Info,
    Render,
    
    // AI operations
    AiAsk { prompt: String },
    
    // Batch operations
    Batch(Vec<String>),
    
    // Additional Phase 2 commands
    List { target: Option<String> },
    Help { topic: Option<String> },
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchRequest {
    pub ctx: Option<String>,
    pub command: Command,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    pub current: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    InvalidSyntax,
    MissingContext,
    UnknownAction(String),
    MissingPayload(&'static str),
    InvalidJson(String),
    InvalidSchema(String),
    UnsupportedContext(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "input is empty"),
            ParseError::InvalidSyntax => write!(f, "invalid DSL syntax - use >, >>, or >[ ] prefixes"),
            ParseError::MissingContext => write!(f, "context not selected (use > <context> first)"),
            ParseError::UnknownAction(a) => write!(f, "unknown action: '{}' - supported: create, add, query, list, help, info, render, ask, exit", a),
            ParseError::MissingPayload(p) => write!(f, "missing payload for '{}' command", p),
            ParseError::InvalidJson(msg) => write!(f, "invalid JSON format: {}", msg),
            ParseError::InvalidSchema(msg) => write!(f, "invalid schema definition: {}", msg),
            ParseError::UnsupportedContext(ctx) => write!(f, "unsupported context: '{}' - supported: data.*, sys.*, ui.*, ai", ctx),
        }
    }
}

impl std::error::Error for ParseError {}

pub struct DslParser {
    pub context: ExecutionContext,
}

impl DslParser {
    pub fn new() -> Self {
        Self { context: ExecutionContext::default() }
    }

    /// Parse a DSL command according to AykenOS Phase 2 hierarchical grammar:
    /// - `>` : Context selection (e.g., "> data.users", "> sys.hw", "> ai")
    /// - `>>` : Context-specific actions (e.g., ">> add {...}", ">> query filter=...")
    /// - `>[ ]` : Batch/parallel operations (e.g., ">[ ] cmd1 | cmd2 | cmd3")
    pub fn parse_command(&mut self, input: &str) -> Result<DispatchRequest, ParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        // Parse hierarchical DSL commands according to Phase 2 documentation
        match trimmed {
            cmd if cmd.starts_with(">>") => self.parse_context_command(cmd),
            cmd if cmd.starts_with(">[ ]") => self.parse_batch_command(cmd),
            cmd if cmd.starts_with(">") => self.parse_simple_command(cmd),
            _ => Err(ParseError::InvalidSyntax)
        }
    }

    /// Get the current execution context
    pub fn current_context(&self) -> Option<&String> {
        self.context.current.as_ref()
    }

    /// Reset the execution context
    pub fn reset_context(&mut self) {
        self.context.current = None;
    }

    /// Check if a context is currently selected
    pub fn has_context(&self) -> bool {
        self.context.current.is_some()
    }

    fn parse_simple_command(&mut self, cmd: &str) -> Result<DispatchRequest, ParseError> {
        // Simple context selection: "> data.users" or "> sys.hw" or "> ai"
        let target = cmd.trim_start_matches('>').trim();
        if target.is_empty() {
            return Err(ParseError::InvalidSyntax);
        }
        
        // Validate context format (basic validation)
        if !self.is_valid_context(target) {
            return Err(ParseError::UnsupportedContext(target.to_string()));
        }
        
        // Update current context
        self.context.current = Some(target.to_string());
        
        Ok(DispatchRequest {
            ctx: self.context.current.clone(),
            command: Command::SelectContext { target: target.to_string() },
        })
    }

    fn parse_context_command(&mut self, cmd: &str) -> Result<DispatchRequest, ParseError> {
        // Context-specific actions: ">> add {...}", ">> query filter=...", ">> info"
        let ctx = self.context.current.clone().ok_or(ParseError::MissingContext)?;
        let body = cmd.trim_start_matches(">>").trim();
        if body.is_empty() {
            return Err(ParseError::InvalidSyntax);
        }

        let mut parts = body.splitn(2, ' ');
        let action = parts.next().unwrap_or("").trim();
        let rest = parts.next().unwrap_or("").trim();

        let command = match action {
            "create" => {
                // Parse schema definition: ">> create schema=[id:int,name:string,age:int]"
                let schema = if rest.starts_with("schema=") {
                    rest.trim_start_matches("schema=").trim().to_string()
                } else {
                    rest.to_string()
                };
                if schema.is_empty() {
                    return Err(ParseError::MissingPayload("create"));
                }
                
                // Basic schema validation
                if !self.is_valid_schema(&schema) {
                    return Err(ParseError::InvalidSchema(format!("expected format: [field:type,...]")));
                }
                
                Command::Create { schema }
            }
            "add" => {
                // Parse JSON payload: ">> add {"id":1,"name":"Ahmet","age":34}"
                if rest.is_empty() {
                    return Err(ParseError::MissingPayload("add"));
                }
                
                // Basic JSON validation
                if !self.is_valid_json_like(rest) {
                    return Err(ParseError::InvalidJson("expected JSON object format".to_string()));
                }
                
                Command::Add { payload: rest.to_string() }
            }
            "query" => {
                // Parse query filter: ">> query filter="age > 30"" or ">> query "age>30""
                let filter = if rest.starts_with("filter=") {
                    rest.trim_start_matches("filter=").trim_matches('"').trim().to_string()
                } else {
                    rest.trim_matches('"').trim().to_string()
                };
                if filter.is_empty() {
                    return Err(ParseError::MissingPayload("query"));
                }
                Command::Query { filter }
            }
            "list" => {
                // Parse list command: ">> list" or ">> list data" or ">> list meta"
                let target = if rest.is_empty() { None } else { Some(rest.to_string()) };
                Command::List { target }
            }
            "help" => {
                // Parse help command: ">> help" or ">> help commands"
                let topic = if rest.is_empty() { None } else { Some(rest.to_string()) };
                Command::Help { topic }
            }
            "info" => Command::Info,
            "render" => Command::Render,
            "ask" => {
                // Parse AI query: ">> ask "natural language query""
                if rest.is_empty() {
                    return Err(ParseError::MissingPayload("ask"));
                }
                let prompt = rest.trim_matches('"').to_string();
                Command::AiAsk { prompt }
            }
            "exit" | "quit" => Command::Exit,
            other => return Err(ParseError::UnknownAction(other.to_string())),
        };

        Ok(DispatchRequest { ctx: Some(ctx), command })
    }

    fn parse_batch_command(&mut self, cmd: &str) -> Result<DispatchRequest, ParseError> {
        // Batch/parallel operations: ">[ ] cmd1 | cmd2 | cmd3"
        let body = cmd.trim_start_matches(">[ ]").trim();
        if body.is_empty() {
            return Err(ParseError::MissingPayload("batch"));
        }
        
        let items: Vec<String> = body
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
            
        if items.is_empty() {
            return Err(ParseError::MissingPayload("batch"));
        }
        
        Ok(DispatchRequest { 
            ctx: self.context.current.clone(), 
            command: Command::Batch(items) 
        })
    }

    /// Basic validation for context names
    fn is_valid_context(&self, context: &str) -> bool {
        // Support Phase 2 documented contexts: data.*, sys.*, ui.*, ai
        context.starts_with("data.") || 
        context.starts_with("sys.") || 
        context.starts_with("ui.") || 
        context == "ai"
    }

    /// Basic validation for schema format
    fn is_valid_schema(&self, schema: &str) -> bool {
        // Very basic validation - should start with [ and end with ]
        schema.starts_with('[') && schema.ends_with(']') && schema.len() > 2
    }

    /// Basic validation for JSON-like format
    fn is_valid_json_like(&self, json: &str) -> bool {
        // Very basic validation - should start with { and end with }
        json.trim().starts_with('{') && json.trim().ends_with('}')
    }
}
