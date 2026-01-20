//! Framework-level errors for D4 Constitutional Framework
//!
//! This module contains ONLY framework initialization, IO, and system-level errors.
//! Specification violations are handled in specification_reports.rs, NOT here.

use crate::errors::specification_reports::PropertyTestFailureInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for constitutional framework operations (ONLY for framework init/IO issues)
pub type Result<T> = std::result::Result<T, ConstitutionalError>;

/// Constitutional framework errors (ONLY for framework init/IO issues, NOT specification violations)
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstitutionalError {
    /// System initialization failure
    #[error("System initialization failed: {reason}")]
    SystemInitializationFailure { reason: String },

    /// Configuration loading error
    #[error("Configuration load error: {reason}")]
    ConfigurationLoadError { reason: String },

    /// Configuration error
    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },

    /// IO operation error
    #[error("IO error: {reason}")]
    IOError { reason: String },

    /// Framework corruption detected
    #[error("Framework corruption: {reason}")]
    FrameworkCorruption { reason: String },

    /// Critical invariant violation (framework-level)
    #[error("Critical invariant violation: {reason}")]
    CriticalInvariantViolation { reason: String },

    /// Serialization/deserialization error
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    /// Property test failure (framework-level)
    #[error("Property test failure: {reason}")]
    PropertyTestFailure { reason: String },
}

/// Legacy error context trait (DEPRECATED - use ReportContext from specification_reports)
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| ConstitutionalError::SystemInitializationFailure {
            reason: format!("{}: {}", context, e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constitutional_error_serialization() {
        let error = ConstitutionalError::ConfigurationLoadError {
            reason: "Test configuration error".to_string(),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: ConstitutionalError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn test_error_context_trait() {
        let result: std::result::Result<(), &str> = Err("test error");
        let with_context = result.with_context("test context");
        
        assert!(with_context.is_err());
        if let Err(ConstitutionalError::SystemInitializationFailure { reason }) = with_context {
            assert!(reason.contains("test context"));
            assert!(reason.contains("test error"));
        } else {
            panic!("Expected SystemInitializationFailure");
        }
    }
}

impl From<PropertyTestFailureInfo> for ConstitutionalError {
    fn from(failure: PropertyTestFailureInfo) -> Self {
        ConstitutionalError::PropertyTestFailure {
            reason: format!("Property test '{}' failed: {} (seed: {}, case: {})", 
                failure.property_name, 
                failure.failure_reason, 
                failure.seed, 
                failure.test_case_id),
        }
    }
}