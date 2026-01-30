//! # AykenOS Semantic CLI
//!
//! Natural language-inspired DSL that compiles to BCIB instructions.
//!
//! ## Architecture
//!
//! ```text
//! User Input (DSL)
//!     ↓
//! Lexer (Tokenize)
//!     ↓
//! Parser (Build AST)
//!     ↓
//! Validator (Check semantics)
//!     ↓
//! Transformer (AST → BCIB)
//!     ↓
//! Executor (Run BCIB via orchestrator)
//!     ↓
//! Result
//! ```
//!
//! ## Architectural Rules
//!
//! **CRITICAL:** This implementation MUST follow Phase 3.5 Architectural Rules.
//!
//! - **RULE 0:** AI components MUST NOT emit BCIB (AI → DSL only)
//! - **RULE 2:** BCIB is execution truth (no bypassing)
//! - **RULE 8:** DSL Core is LOCKED (minimal, stable)
//!
//! See: `_ayken/specs/phase3-5-semantic-interaction/ARCHITECTURAL_RULES.md`
//!
//! ## Manifesto
//!
//! > **AykenOS is an operating system where intent, execution and authority are strictly separated.**
//!
//! ## Phase
//!
//! Phase 3.5.1: Semantic CLI Core (AI-less)
//!
//! **NO AI in this phase.** Only DSL → BCIB → Execution.

// Module declarations
pub mod types;
pub mod error;

// Lexer module (tokenization)
pub mod lexer;

// Parser module (AST construction)
pub mod parser;
pub mod ast;

// Validator module (semantic analysis)
pub mod validator;

// Transformer module (AST → BCIB)
pub mod transformer;
pub mod bcib;

// Normalizer module (BCIB → NormalizedBCIB) - Gate C
pub mod normalizer;
pub mod execution_plan;
pub mod ir_planner;

// Parallelism module (D2 Parallelism Architecture) - Gate D
pub mod parallelism;

// Loop Engine module (D3 Loop Support) - Gate D
pub mod loop_engine;

// Context manager (data loading, caching)
pub mod context;

// Memory optimization (object pooling, allocation patterns) - Phase 4.3
pub mod memory;

// Performance Management (intelligent resource allocation, scheduling) - Phase 4.4
pub mod performance_management;

// Constitutional Integration (unified governance, evidence-based decisions) - Phase CI
pub mod constitutional;

// Orchestration (multi-agent coordination) - Phase 3.4
pub mod orchestration;

// Operations (query, mutation, pipeline, etc.)
pub mod operations;

// Submission Bridge (BCIB submission via orchestrator) - Gate C
pub mod submission_bridge;

// Gate C: Submission Bridge (Phase 3.5)
pub mod gate_c;

// REPL (interactive interface)
pub mod repl;

// Re-exports for convenience
pub use error::{SemanticCLIError, Result};
pub use types::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Phase identifier
pub const PHASE: &str = "3.5.1";

/// Phase name
pub const PHASE_NAME: &str = "Semantic CLI Core (AI-less)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_phase() {
        assert_eq!(PHASE, "3.5.1");
        assert_eq!(PHASE_NAME, "Semantic CLI Core (AI-less)");
    }
}
