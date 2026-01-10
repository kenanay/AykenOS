# AykenOS DSL Parser

A hierarchical Domain Specific Language (DSL) parser for AykenOS Phase 2 data-centric architecture.

## Overview

This parser implements the AykenOS hierarchical command grammar that enables context-aware, data-centric operations. It supports three levels of command hierarchy designed for intuitive interaction with data containers, system resources, and AI services.

## Grammar

### Command Hierarchy

- **`>`** : Context selection (global/context switching)
- **`>>`** : Context-specific actions (operations within selected context)
- **`>[ ]`** : Batch/parallel operations (pipeline-style commands)

### Supported Contexts

- **`data.*`** : Data container operations (e.g., `data.users`, `data.products`)
- **`sys.*`** : System information and hardware (e.g., `sys.hw`, `sys.memory`)
- **`ui.*`** : User interface operations (e.g., `ui.scene.dashboard`)
- **`ai`** : AI-powered operations and queries

### Supported Actions

| Action | Description | Example |
|--------|-------------|---------|
| `create` | Create data containers with schema | `>> create schema=[id:int,name:string]` |
| `add` | Add data to containers (JSON format) | `>> add {"id":1,"name":"John"}` |
| `query` | Query data with filters | `>> query filter="age > 30"` |
| `list` | List containers or metadata | `>> list` or `>> list data` |
| `help` | Get help information | `>> help` or `>> help commands` |
| `info` | Get context-specific information | `>> info` |
| `render` | Render UI components | `>> render` |
| `ask` | AI-powered questions | `>> ask "What is the average age?"` |
| `exit`/`quit` | Exit commands | `>> exit` |

## Usage Examples

### Basic Data Operations

```rust
use dsl_parser::DslParser;

let mut parser = DslParser::new();

// Select a data context
let result = parser.parse_command("> data.users")?;

// Create a schema
let result = parser.parse_command(">> create schema=[id:int,name:string,age:int]")?;

// Add data
let result = parser.parse_command(">> add {\"id\":1,\"name\":\"Ahmet\",\"age\":34}")?;

// Query data
let result = parser.parse_command(">> query filter=\"age > 30\"")?;
```

### AI Operations

```rust
// Switch to AI context
parser.parse_command("> ai")?;

// Ask AI questions
parser.parse_command(">> ask \"What is the weather today?\"")?;
```

### Batch Operations

```rust
// Execute multiple commands in parallel
parser.parse_command(">[ ] query filter=\"age > 25\" | query filter=\"name like 'A%'\" | list")?;
```

### System Operations

```rust
// System information
parser.parse_command("> sys.hw")?;
parser.parse_command(">> info")?;
```

## Error Handling

The parser provides detailed error messages for common issues:

- **Empty Input**: When no command is provided
- **Invalid Syntax**: When command doesn't match DSL grammar
- **Missing Context**: When context-specific action is used without selecting context
- **Unknown Action**: When unsupported action is used
- **Missing Payload**: When required parameters are missing
- **Invalid JSON**: When JSON format is malformed
- **Invalid Schema**: When schema definition is malformed
- **Unsupported Context**: When invalid context is selected

## Context Management

The parser maintains execution context state:

```rust
let mut parser = DslParser::new();

// Check current context
if let Some(ctx) = parser.current_context() {
    println!("Current context: {}", ctx);
}

// Check if context is set
if parser.has_context() {
    // Context-specific operations are allowed
}

// Reset context
parser.reset_context();
```

## Integration with AykenOS

This parser is designed to integrate with:

- **BCIB Runtime**: Commands can be converted to BCIB execution graphs
- **Data Containers**: Direct integration with tabular and text data types
- **AI Services**: Natural language processing and query assistance
- **Shell Interface**: REPL-style interactive command processing

## Phase 2 Compliance

This implementation follows the AykenOS Phase 2 specifications:

- ✅ Hierarchical DSL grammar (>, >>, >[ ])
- ✅ Context-aware command processing
- ✅ Data-centric operation support
- ✅ AI integration readiness
- ✅ Error handling and validation
- ✅ Extensible command system

## Testing

Run the test suite:

```bash
cargo test
```

Run the basic usage example:

```bash
cargo run --example basic_usage
```

## License

MIT License - See LICENSE file for details.