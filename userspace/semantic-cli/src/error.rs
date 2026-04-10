//! Error types for Semantic CLI
//!
//! This module defines all error types used throughout the Semantic CLI.
//! Errors are designed to be:
//! - Human-readable (not technical jargon)
//! - Actionable (suggest fixes)
//! - Include error codes (for programmatic handling)
//! - Include source location (for debugging)

use crate::types::SourceLocation;
use std::fmt;
use thiserror::Error;

/// Result type alias for Semantic CLI operations
pub type Result<T> = std::result::Result<T, SemanticCLIError>;

/// Main error type for Semantic CLI
#[derive(Debug, Error)]
pub enum SemanticCLIError {
    /// Syntax error (invalid DSL syntax)
    #[error("Syntax error at {location}: {message}\n{suggestion}")]
    SyntaxError {
        location: SourceLocation,
        message: String,
        suggestion: String,
        code: ErrorCode,
    },

    /// Semantic error (valid syntax but invalid semantics)
    #[error("Semantic error at {location}: {message}\n{suggestion}")]
    SemanticError {
        location: SourceLocation,
        message: String,
        suggestion: String,
        code: ErrorCode,
    },

    /// Validation error (command fails validation)
    #[error("Validation error: {message}\n{suggestion}")]
    ValidationError {
        message: String,
        suggestion: String,
        code: ErrorCode,
    },

    /// Transformation error (AST → BCIB transformation fails)
    #[error("Transformation error: {message}")]
    TransformError { message: String, code: ErrorCode },

    /// Execution error (BCIB execution fails)
    #[error("Execution error: {message}")]
    ExecutionError { message: String, code: ErrorCode },

    /// Context error (context loading/access fails)
    #[error("Context error: {message}")]
    ContextError { message: String, code: ErrorCode },

    /// Security error (capability check fails)
    #[error("Security error: {message}")]
    SecurityError { message: String, code: ErrorCode },

    /// Audit error (audit trail operations fail)
    #[error("Audit error: {message}")]
    AuditError { message: String, code: ErrorCode },

    /// Replay verification failed
    #[error("Replay verification failed: {0}")]
    ReplayVerificationFailed(String),

    /// Submission failed
    #[error("Submission failed: {0}")]
    SubmissionFailed(String),

    /// Capability derivation failed
    #[error("Capability derivation failed: {0}")]
    CapabilityDerivationFailed(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Error codes for programmatic handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Syntax errors (E001-E099)
    E001, // Invalid token
    E002, // Unexpected token
    E003, // Unclosed delimiter
    E004, // Invalid identifier
    E005, // Invalid literal

    // Semantic errors (E100-E199)
    E100, // Invalid context path
    E101, // Type mismatch
    E102, // Undefined identifier
    E103, // Invalid operation

    // Validation errors (E200-E299)
    E200, // Context not found
    E201, // Permission denied
    E202, // Type check failed
    E203, // Dependency missing

    // Transformation errors (E300-E399)
    E300, // Cannot transform node
    E301, // Invalid BCIB sequence

    // Execution errors (E400-E499)
    E400, // Execution failed
    E401, // Timeout
    E402, // Resource exhausted
    E420, // Evidence incomplete

    // Context errors (E500-E599)
    E500, // Context load failed
    E501, // Context not accessible

    // Security errors (E600-E699)
    E600, // Capability check failed
    E601, // Unauthorized access

    // Audit errors (E700-E799)
    E700, // Audit trail creation failed
    E701, // Audit integrity check failed
    E702, // Audit record corruption

    // Replay verification errors (E750-E759)
    E750, // Replay verification failed
    E751, // Replay deviation detected
    E752, // Replay binding integrity failed

    // Submission errors (E760-E769)
    E760, // Submission failed
    E761, // Submission validation failed
    E762, // Kernel endpoint unavailable

    // Capability derivation errors (E770-E779)
    E770, // Capability derivation failed
    E771, // Capability mismatch
    E772, // Capability audit failed

    // System errors (E800-E899)
    E800, // System timestamp error
    E801, // File system error
    E802, // Serialization error
    E803, // Network error

    // Constitutional errors (E900-E999)
    E900, // Constitutional violation - critical
    E901, // Evidence requirement violation
    E902, // Decision boundary violation
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SemanticCLIError {
    /// Create a syntax error
    pub fn syntax_error(
        location: SourceLocation,
        message: impl Into<String>,
        suggestion: impl Into<String>,
        code: ErrorCode,
    ) -> Self {
        Self::SyntaxError {
            location,
            message: message.into(),
            suggestion: suggestion.into(),
            code,
        }
    }

    /// Create a semantic error
    pub fn semantic_error(
        location: SourceLocation,
        message: impl Into<String>,
        suggestion: impl Into<String>,
        code: ErrorCode,
    ) -> Self {
        Self::SemanticError {
            location,
            message: message.into(),
            suggestion: suggestion.into(),
            code,
        }
    }

    /// Create a validation error
    pub fn validation_error(
        message: impl Into<String>,
        suggestion: impl Into<String>,
        code: ErrorCode,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            suggestion: suggestion.into(),
            code,
        }
    }

    /// Create a transformation error
    pub fn transform_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::TransformError {
            message: message.into(),
            code,
        }
    }

    /// Create a transformation error (alias for consistency)
    pub fn transformation_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::TransformError {
            message: message.into(),
            code,
        }
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::TransformError {
            message: message.into(),
            code,
        }
    }

    /// Create an execution error
    pub fn execution_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::ExecutionError {
            message: message.into(),
            code,
        }
    }

    /// Create a context error
    pub fn context_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::ContextError {
            message: message.into(),
            code,
        }
    }

    /// Create a security error
    pub fn security_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::SecurityError {
            message: message.into(),
            code,
        }
    }

    /// Create an audit error
    pub fn audit_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::AuditError {
            message: message.into(),
            code,
        }
    }

    /// Create a system error
    pub fn system_error(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::ExecutionError {
            message: message.into(),
            code,
        }
    }

    /// Get error code
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::SyntaxError { code, .. } => Some(*code),
            Self::SemanticError { code, .. } => Some(*code),
            Self::ValidationError { code, .. } => Some(*code),
            Self::TransformError { code, .. } => Some(*code),
            Self::ExecutionError { code, .. } => Some(*code),
            Self::ContextError { code, .. } => Some(*code),
            Self::SecurityError { code, .. } => Some(*code),
            Self::AuditError { code, .. } => Some(*code),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error() {
        let err = SemanticCLIError::syntax_error(
            SourceLocation::new(1, 5, 4),
            "unexpected token",
            "expected identifier",
            ErrorCode::E002,
        );
        assert!(matches!(err, SemanticCLIError::SyntaxError { .. }));
        assert_eq!(err.code(), Some(ErrorCode::E002));
    }

    #[test]
    fn test_validation_error() {
        let err = SemanticCLIError::validation_error(
            "context not found",
            "available contexts: data.users, fs.logs",
            ErrorCode::E200,
        );
        assert!(matches!(err, SemanticCLIError::ValidationError { .. }));
        assert_eq!(err.code(), Some(ErrorCode::E200));
    }

    #[test]
    fn test_error_display() {
        let err = SemanticCLIError::syntax_error(
            SourceLocation::new(1, 5, 4),
            "unexpected token",
            "expected identifier",
            ErrorCode::E002,
        );
        let display = format!("{}", err);
        assert!(display.contains("Syntax error"));
        assert!(display.contains("1:5"));
        assert!(display.contains("unexpected token"));
    }
}
