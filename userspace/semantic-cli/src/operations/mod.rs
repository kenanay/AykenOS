//! Operations Module
//!
//! This module implements all operation types for the Semantic CLI.
//! Operations execute BCIB instructions and return formatted results.
//!
//! # Phase 3.5.1.a Operations
//!
//! - **Query Operations**: query, list, show (read-only)
//! - **System Operations**: status, agents (read-only)
//! - **Debug Operations**: explain, dry-run, history
//!
//! # Phase 3.5.1.b Operations (Optional)
//!
//! - **Mutation Operations**: add, update, delete (stubs)
//! - **Pipeline Operations**: pipeline execution (skeleton)
//! - **Security Operations**: permissions, sandbox (structure)
//!
//! # Design Principles
//!
//! 1. **BCIB-driven**: All operations execute BCIB instructions
//! 2. **Contextual capabilities**: Access control via AR-4
//! 3. **Performance**: < 100ms for typical operations
//! 4. **Error handling**: Clear, actionable error messages
//! 5. **Result formatting**: Human-readable output

pub mod query;
pub mod filter;
pub mod system;
pub mod debug;

// Re-exports for convenience
pub use query::{QueryExecutor, QueryResult};
pub use filter::{FilterEvaluator, FilterResult};
pub use system::{SystemExecutor, SystemResult};
pub use debug::{DebugExecutor, DebugResult};

/// Operation result trait for consistent result handling
pub trait OperationResult {
    /// Format result for display
    fn format(&self) -> String;
    
    /// Get result metadata
    fn metadata(&self) -> std::collections::HashMap<String, serde_json::Value>;
    
    /// Check if operation was successful
    fn is_success(&self) -> bool;
}

/// Operation executor trait for consistent execution interface
pub trait OperationExecutor {
    type Input;
    type Output: OperationResult;
    type Error;
    
    /// Execute operation with given input
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}