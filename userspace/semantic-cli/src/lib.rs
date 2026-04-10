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
pub mod error;
pub mod types;

// Lexer module (tokenization)
pub mod lexer;

// Parser module (AST construction)
pub mod ast;
pub mod parser;

// Validator module (semantic analysis)
pub mod validator;

// Transformer module (AST → BCIB)
pub mod bcib;
pub mod bcib_simple; // Simplified BCIB for Phase-16A
pub mod bcib_serialization; // BCIB serialization for kernel submission
pub mod transformer;

// Normalizer module (BCIB → NormalizedBCIB) - Gate C
pub mod execution_plan;
pub mod ir_planner;
pub mod memory;
pub mod normalizer;
pub mod performance_management;
pub mod canonical_query;
pub mod canonical_query_lowering;
pub mod submission_validation;
pub mod proof_chain;
pub mod submit_only_router;

// Parallelism module (D2 Parallelism Architecture) - Gate D
pub mod parallelism;

// Loop Engine module (D3 Loop Support) - Gate D
pub mod loop_engine;

// Context manager (data loading, caching)
pub mod context;

// Operations (query, mutation, pipeline, etc.)
pub mod operations;

// Submission Bridge (BCIB submission via orchestrator) - Gate C
pub mod submission_bridge;

// Replay Verification (deterministic execution verification)
pub mod replay_verification;

// Kernel Submit Adapter (Ring3 → Ring0 boundary)
pub mod kernel_submit_adapter;

// Capability Derivation Audit (capability derivation verification)
pub mod capability_derivation_audit;

// Gate C: Submission Bridge (Phase 3.5)
pub mod gate_c;

// REPL (interactive interface)
pub mod repl;

pub mod ir_executor {
    pub use crate::ir_planner::register_file::{RegisterFile, RegisterValue};
    pub use crate::ir_planner::replay::{ReplayRecorder, ReplayTrace};
    pub use crate::ir_planner::{ExecutionError, ExecutionResult, ExecutionState, IRExecutor};
}

// Re-exports for convenience
pub use error::{Result, SemanticCLIError};
pub use canonical_query::{
    build_canonical_plan, build_canonical_plan_from_command, parse_canonical_plan,
    CanonicalCommandKind, CanonicalIrInstruction, CanonicalPlan, CanonicalPredicate,
    CanonicalPredicateKind, CanonicalQueryBinding,
};
pub use canonical_query_lowering::{
    lower_canonical_query_to_bcib, lower_canonical_query_to_bcib_with_options,
    validate_canonical_query_bcib, CanonicalQueryLoweringOptions,
    LoweredBcibInstruction, LoweredCanonicalQuery,
};
pub use proof_chain::{build_proof_chain_record, ProofChainRecord, ProofReplayBinding};
pub use submission_validation::{
    derive_required_capabilities, SubmissionCapability, SubmissionCapabilityScope,
    SubmissionValidationInput, SubmissionValidationReport, SubmissionValidator,
};
pub use submit_only_router::{
    CanonicalQuerySubmission, CanonicalQuerySubmissionRequest, DeterministicSubmitAdapter,
    SubmitAdapter, SubmitOnlyRouter,
};
pub use replay_verification::{ReplayVerificationResult, ReplayVerifier};
pub use kernel_submit_adapter::KernelSubmitAdapter;
pub use capability_derivation_audit::{
    CapabilityDerivationAudit, CapabilityDerivationAuditor, DerivationStep,
};
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
