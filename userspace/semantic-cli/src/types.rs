//! Core type definitions for Semantic CLI
//!
//! This module defines the fundamental types used throughout the Semantic CLI:
//! - Source location tracking
//! - Token types
//! - AST node types
//! - BCIB instruction types
//! - Execution context types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source location in input text
///
/// Used for error reporting and debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in source (0-indexed)
    pub offset: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create a source location at the start of input
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self::start()
    }
}

/// Determinism level for BCIB execution
///
/// Used for distributed execution, GPU offloading, and replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeterminismLevel {
    /// Same input → same output (required for replay)
    Deterministic,
    /// May vary (e.g., network latency, GPU scheduling)
    BestEffort,
    /// Explicitly non-deterministic (e.g., random, time)
    NonDeterministic,
}

impl Default for DeterminismLevel {
    fn default() -> Self {
        Self::Deterministic
    }
}

impl fmt::Display for DeterminismLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deterministic => write!(f, "DETERMINISTIC"),
            Self::BestEffort => write!(f, "BEST_EFFORT"),
            Self::NonDeterministic => write!(f, "NON_DETERMINISTIC"),
        }
    }
}

/// BCIB metadata for replay security and audit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BCIBMetadata {
    /// Unique execution ID (prevents replay attacks)
    pub nonce: u64,
    /// Expiration timestamp (optional)
    pub expiry: Option<i64>,
    /// Hash of execution context (validates context)
    pub execution_context_hash: Vec<u8>,
    /// Whether this BCIB can be replayed
    pub replay_allowed: bool,
}

impl BCIBMetadata {
    /// Create new metadata with nonce
    pub fn new(nonce: u64) -> Self {
        Self {
            nonce,
            expiry: None,
            execution_context_hash: Vec::new(),
            replay_allowed: false,
        }
    }

    /// Create metadata with expiry
    pub fn with_expiry(nonce: u64, expiry: i64) -> Self {
        Self {
            nonce,
            expiry: Some(expiry),
            execution_context_hash: Vec::new(),
            replay_allowed: false,
        }
    }

    /// Check if metadata is expired
    pub fn is_expired(&self, current_time: i64) -> bool {
        self.expiry.map_or(false, |exp| current_time > exp)
    }

    /// Set execution context hash
    pub fn with_context_hash(mut self, hash: Vec<u8>) -> Self {
        self.execution_context_hash = hash;
        self
    }

    /// Allow replay
    pub fn allow_replay(mut self) -> Self {
        self.replay_allowed = true;
        self
    }
}

/// Execution scope identifier
pub type ScopeId = String;

/// Context path (e.g., "data.users")
pub type ContextPath = String;

/// Capability name (e.g., "read", "write", "delete")
pub type Capability = String;

/// Agent type identifier
pub type AgentType = String;

/// Task description
pub type TaskDescription = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new(10, 5, 42);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.offset, 42);
        assert_eq!(format!("{}", loc), "10:5");
    }

    #[test]
    fn test_source_location_start() {
        let loc = SourceLocation::start();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);
        assert_eq!(loc.offset, 0);
    }

    #[test]
    fn test_determinism_level() {
        assert_eq!(
            format!("{}", DeterminismLevel::Deterministic),
            "DETERMINISTIC"
        );
        assert_eq!(format!("{}", DeterminismLevel::BestEffort), "BEST_EFFORT");
        assert_eq!(
            format!("{}", DeterminismLevel::NonDeterministic),
            "NON_DETERMINISTIC"
        );
    }

    #[test]
    fn test_bcib_metadata() {
        let meta = BCIBMetadata::new(42);
        assert_eq!(meta.nonce, 42);
        assert_eq!(meta.expiry, None);
        assert!(!meta.replay_allowed);
    }

    #[test]
    fn test_bcib_metadata_expiry() {
        let meta = BCIBMetadata::with_expiry(42, 1000);
        assert!(!meta.is_expired(500));
        assert!(meta.is_expired(1001));
    }

    #[test]
    fn test_bcib_metadata_builder() {
        let meta = BCIBMetadata::new(42)
            .with_context_hash(vec![1, 2, 3])
            .allow_replay();
        assert_eq!(meta.execution_context_hash, vec![1, 2, 3]);
        assert!(meta.replay_allowed);
    }
}
