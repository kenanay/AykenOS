//! # Gate C: Submission Bridge
//!
//! **Phase:** 3.5 Gate C  
//! **Author:** Kenan AY  
//! **Mission:** "Anlamlı etki PLANLAMASI"
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! Gate C implements semantic planning and submission without execution.
//!
//! ## Gate C Boundaries
//!
//! Gate C:
//! - ✅ Plan üretir (generates plans)
//! - ✅ Submit eder (submits plans)  
//! - ✅ Görünür kılar (provides visibility)
//! - ❌ Uygulamaz (does not execute)
//! - ❌ Beklemez (does not wait)
//! - ❌ Optimize etmez (does not optimize runtime)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Gate C Architecture                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  REPL Visibility    │  Security Ops     │  Submission Bridge │
//! │  - Plan Preview     │  - Inspect/Audit  │  - Submit Only     │
//! │  - Semantic Explain │  - Capability     │  - No Wait/Poll    │
//! │  - Dry-run Preview  │  - Redaction      │  - Deterministic   │
//! ├─────────────────────────────────────────────────────────────┤
//! │              IR Planner (Semantic/Structural)              │
//! │              - Ordering Hints                               │
//! │              - Dependency Analysis                          │
//! │              - Parallelism Hints (No Execution)            │
//! ├─────────────────────────────────────────────────────────────┤
//! │                     Normalizer                              │
//! │                     - Canonicalization                      │
//! │                     - Structural Validation                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Pipeline Planning  │  Mutation Intent  │  Core Types       │
//! │  - Dependency Graph │  - Invalidate     │  - Error Types    │
//! │  - Stateless Chain  │  - Conflict Check │  - Limits         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                   Existing BCIB/DSL Core                   │
//! │                   (No Modifications)                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Steering Compliance
//!
//! This module strictly follows:
//! - `_ayken/steering/GATE_C_RULES_LOCK.md` (LOCK)
//! - `_ayken/steering/GATE_C_BEHAVIOR_MATRIX.md` (LOCK)  
//! - `_ayken/steering/GATE_C_TEST_AND_LIMITS.md` (LOCK)

// Core types and error definitions
pub mod deterministic;
pub mod error;
pub mod limits;
pub mod types; // DETERMINISM UTILITIES

// Submission Bridge (Submit-Only)
pub mod submission;

// Mutation Intent Planning
pub mod mutation;

// Pipeline Planning (Stateless)
pub mod pipeline;

// Security Operations (Inspect/Audit/Check-Only)
pub mod security_ops;

// IR Planner (Semantic/Structural Only)
pub mod ir;

// Normalizer Integration
pub mod normalizer;

// REPL Semantic Visibility
pub mod repl_visibility;

// Performance Measurement Infrastructure - Phase 4.2
pub mod performance;

// Constitutional CI Guards - Phase 4.0 Baseline Hardening
#[cfg(test)]
pub mod complexity_budget_tests;
#[cfg(test)]
pub mod optimized_complexity_budget_tests;
#[cfg(test)]
pub mod snapshot_tests;

// Re-exports for convenience
pub use error::{GateCError, GateCResult};
pub use limits::*;
pub use types::*;

/// Gate C version
pub const GATE_C_VERSION: &str = "3.5.0";

/// Gate C mission statement
pub const GATE_C_MISSION: &str = "Anlamlı etki PLANLAMASI";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_c_version() {
        assert_eq!(GATE_C_VERSION, "3.5.0");
    }

    #[test]
    fn test_gate_c_mission() {
        assert_eq!(GATE_C_MISSION, "Anlamlı etki PLANLAMASI");
    }
}
