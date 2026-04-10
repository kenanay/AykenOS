//! AST Node Definitions
//!
//! Defines all AST node types for the Semantic CLI.
//!
//! # Design Principles
//!
//! 1. **Immutable:** AST nodes are immutable after construction
//! 2. **Location tracking:** All nodes preserve source location for error reporting
//! 3. **Type-safe:** Rust's type system ensures correctness
//! 4. **Phase-aware:** Only Core DSL nodes in Phase 3.5.1

use crate::types::SourceLocation;
use serde::{Deserialize, Serialize};

/// Root AST node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode {
    pub command: CommandNode,
}

impl AstNode {
    /// Create a new AST node
    pub fn new(command: CommandNode) -> Self {
        Self { command }
    }

    /// Get the source location of this node
    pub fn location(&self) -> SourceLocation {
        self.command.location()
    }
}

/// Command node types (Core DSL only - Phase 3.5.1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandNode {
    /// Query command: `query <context> <filter>`
    Query {
        location: SourceLocation,
        context: Vec<String>,
        filter: Option<Expr>,
    },

    /// List command: `list <context>`
    List {
        location: SourceLocation,
        context: Vec<String>,
    },

    /// Show command: `show <context> <id>`
    Show {
        location: SourceLocation,
        context: Vec<String>,
        id: Expr,
    },

    /// Status command: `status`
    Status { location: SourceLocation },

    /// Agents command: `agents`
    Agents { location: SourceLocation },

    /// Explain command: `explain <command>`
    Explain {
        location: SourceLocation,
        command: Box<CommandNode>,
    },

    /// Dry-run command: `dry-run <command>`
    DryRun {
        location: SourceLocation,
        command: Box<CommandNode>,
    },

    /// History command: `history`
    History { location: SourceLocation },
}

impl CommandNode {
    /// Get the source location of this command
    pub fn location(&self) -> SourceLocation {
        match self {
            Self::Query { location, .. } => *location,
            Self::List { location, .. } => *location,
            Self::Show { location, .. } => *location,
            Self::Status { location } => *location,
            Self::Agents { location } => *location,
            Self::Explain { location, .. } => *location,
            Self::DryRun { location, .. } => *location,
            Self::History { location } => *location,
        }
    }
}

/// Expression node types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Identifier: `age`, `name`, etc.
    Identifier {
        name: String,
        location: SourceLocation,
    },

    /// Number literal: `42`, `3.14`
    Number {
        value: String,
        location: SourceLocation,
    },

    /// String literal: `"hello"`
    String {
        value: String,
        location: SourceLocation,
    },

    /// Boolean literal: `true`, `false`
    Boolean {
        value: bool,
        location: SourceLocation,
    },

    /// Binary operation: `age > 18`, `name == "Alice"`
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        location: SourceLocation,
    },

    /// Unary operation: `not active`
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        location: SourceLocation,
    },
}

impl Expr {
    /// Get the source location of this expression
    pub fn location(&self) -> SourceLocation {
        match self {
            Self::Identifier { location, .. } => *location,
            Self::Number { location, .. } => *location,
            Self::String { location, .. } => *location,
            Self::Boolean { location, .. } => *location,
            Self::Binary { location, .. } => *location,
            Self::Unary { location, .. } => *location,
        }
    }
}

/// Binary operators (Core DSL only - Phase 3.5.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Comparison operators (CORE)
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=

    // Logical operators (CORE)
    And, // and
    Or,  // or
}

impl BinaryOp {
    /// Get operator precedence (higher = tighter binding)
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::Ne => 3,
            Self::Lt | Self::Le | Self::Gt | Self::Ge => 4,
        }
    }

    /// Check if operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        true // All Core DSL operators are left-associative
    }
}

/// Unary operators (Core DSL only - Phase 3.5.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not, // not
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let cmd = CommandNode::Status {
            location: SourceLocation::new(1, 1, 0),
        };
        let ast = AstNode::new(cmd);
        assert_eq!(ast.location().line, 1);
    }

    #[test]
    fn test_query_command() {
        let cmd = CommandNode::Query {
            location: SourceLocation::new(1, 1, 0),
            context: vec!["data".to_string(), "users".to_string()],
            filter: None,
        };
        assert_eq!(cmd.location().line, 1);
    }

    #[test]
    fn test_expression_location() {
        let expr = Expr::Number {
            value: "42".to_string(),
            location: SourceLocation::new(1, 5, 4),
        };
        assert_eq!(expr.location().column, 5);
    }

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
        assert!(BinaryOp::Eq.precedence() > BinaryOp::And.precedence());
        assert!(BinaryOp::Lt.precedence() > BinaryOp::Eq.precedence());
    }

    #[test]
    fn test_binary_op_associativity() {
        assert!(BinaryOp::And.is_left_associative());
        assert!(BinaryOp::Eq.is_left_associative());
    }
}
