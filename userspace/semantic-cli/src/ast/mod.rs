//! Abstract Syntax Tree (AST) for Semantic CLI
//!
//! This module defines the AST node types that represent parsed commands.
//!
//! # Phase 3.5.1 Scope
//!
//! **CORE DSL ONLY:**
//! - Query operations: `query`, `list`, `show`
//! - System operations: `status`, `agents`
//! - Expressions: identifiers, literals, comparisons, logical operators
//!
//! **EXTENDED DSL (NOT IN THIS PHASE):**
//! - Mutation operations: `add`, `update`, `delete`
//! - Pipeline operations: `pipeline`
//! - Orchestration: `orchestrate`
//! - Arithmetic operators: `+`, `-`, `*`, `/`
//!
//! These will be added in Phase 3.5.2+.

pub mod nodes;

// Re-export for convenience
pub use nodes::{AstNode, BinaryOp, CommandNode, Expr, UnaryOp};
