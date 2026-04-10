//! # Gate C Error Types
//!
//! Deterministic error types for Gate C submission bridge functionality.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use thiserror::Error;

/// Result type for Gate C operations
pub type GateCResult<T> = Result<T, GateCError>;

/// Error code trait for stable error identification across versions
pub trait ErrorCode {
    /// Get stable error code for deterministic testing, audit logs, and tooling integration
    fn code(&self) -> &'static str;
}

/// Main error type for Gate C operations
#[derive(Debug, Clone, PartialEq, Error)]
pub enum GateCError {
    /// Submission-related errors
    #[error("Submission error: {0}")]
    Submission(#[from] SubmissionError),

    /// Mutation-related errors
    #[error("Mutation error: {0}")]
    Mutation(#[from] MutationError),

    /// Pipeline-related errors
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] PipelineError),

    /// IR planning errors
    #[error("IR error: {0}")]
    IR(#[from] IRError),

    /// Normalization errors
    #[error("Normalization error: {0}")]
    Normalization(#[from] NormalizationError),

    /// Security-related errors
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    /// REPL rendering errors
    #[error("Render error: {0}")]
    Render(#[from] RenderError),
}

impl ErrorCode for GateCError {
    fn code(&self) -> &'static str {
        match self {
            GateCError::Submission(e) => e.code(),
            GateCError::Mutation(e) => e.code(),
            GateCError::Pipeline(e) => e.code(),
            GateCError::IR(e) => e.code(),
            GateCError::Normalization(e) => e.code(),
            GateCError::Security(e) => e.code(),
            GateCError::Render(e) => e.code(),
        }
    }
}

/// Submission bridge errors (deterministic)
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SubmissionError {
    /// Orchestrator is unavailable
    #[error("Orchestrator unavailable")]
    OrchestratorUnavailable,

    /// Invalid plan submitted
    #[error("Invalid plan: {0}")]
    InvalidPlan(String),

    /// Capability denied
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Optional error codes for standardization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubmissionErrorCode {
    /// Orchestrator unavailable
    OrchUnavailable,
    /// Invalid plan
    InvalidPlan,
    /// Capability denied
    CapabilityDenied,
    /// Network error
    NetworkError,
    /// Serialization error
    SerializationError,
}

impl SubmissionError {
    /// Get stable error code for deterministic testing and audit logs
    pub fn error_code(&self) -> SubmissionErrorCode {
        match self {
            SubmissionError::OrchestratorUnavailable => SubmissionErrorCode::OrchUnavailable,
            SubmissionError::InvalidPlan(_) => SubmissionErrorCode::InvalidPlan,
            SubmissionError::CapabilityDenied(_) => SubmissionErrorCode::CapabilityDenied,
            SubmissionError::NetworkError(_) => SubmissionErrorCode::NetworkError,
            SubmissionError::SerializationError(_) => SubmissionErrorCode::SerializationError,
        }
    }
}

impl ErrorCode for SubmissionError {
    fn code(&self) -> &'static str {
        match self.error_code() {
            SubmissionErrorCode::OrchUnavailable => "GATE_C_SUBMISSION_ORCH_UNAVAILABLE",
            SubmissionErrorCode::InvalidPlan => "GATE_C_SUBMISSION_INVALID_PLAN",
            SubmissionErrorCode::CapabilityDenied => "GATE_C_SUBMISSION_CAPABILITY_DENIED",
            SubmissionErrorCode::NetworkError => "GATE_C_SUBMISSION_NETWORK_ERROR",
            SubmissionErrorCode::SerializationError => "GATE_C_SUBMISSION_SERIALIZATION_ERROR",
        }
    }
}

/// Mutation intent errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MutationError {
    /// Mutation conflict detected
    #[error("Mutation conflict: {0}")]
    MutationConflict(String),

    /// Capability denied for mutation
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    /// Invalid mutation intent
    #[error("Invalid mutation intent: {0}")]
    InvalidIntent(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

impl ErrorCode for MutationError {
    fn code(&self) -> &'static str {
        match self {
            MutationError::MutationConflict(_) => "GATE_C_MUTATION_CONFLICT",
            MutationError::CapabilityDenied(_) => "GATE_C_MUTATION_CAPABILITY_DENIED",
            MutationError::InvalidIntent(_) => "GATE_C_MUTATION_INVALID_INTENT",
            MutationError::ResourceNotFound(_) => "GATE_C_MUTATION_RESOURCE_NOT_FOUND",
        }
    }
}

/// Pipeline planning errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PipelineError {
    /// Dependency cycle detected
    #[error("Dependency cycle detected: {0}")]
    CycleDetected(String),

    /// Pipeline too large
    #[error("Pipeline too large: {steps} steps exceeds limit of {limit}")]
    PipelineTooLarge { steps: usize, limit: usize },

    /// Invalid step reference
    #[error("Invalid step reference: {0}")]
    InvalidStepReference(String),

    /// Ambiguous ordering
    #[error("Ambiguous ordering: {0}")]
    AmbiguousOrdering(String),
}

impl ErrorCode for PipelineError {
    fn code(&self) -> &'static str {
        match self {
            PipelineError::CycleDetected(_) => "GATE_C_PIPELINE_CYCLE_DETECTED",
            PipelineError::PipelineTooLarge { .. } => "GATE_C_PIPELINE_TOO_LARGE",
            PipelineError::InvalidStepReference(_) => "GATE_C_PIPELINE_INVALID_STEP_REFERENCE",
            PipelineError::AmbiguousOrdering(_) => "GATE_C_PIPELINE_AMBIGUOUS_ORDERING",
        }
    }
}

/// IR planner errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum IRError {
    /// Invalid BCIB instruction
    #[error("Invalid BCIB instruction: {0}")]
    InvalidInstruction(String),

    /// Dependency analysis failed
    #[error("Dependency analysis failed: {0}")]
    DependencyAnalysisFailed(String),

    /// Ordering hint generation failed
    #[error("Ordering hint generation failed: {0}")]
    OrderingHintFailed(String),

    /// Parallelism analysis failed
    #[error("Parallelism analysis failed: {0}")]
    ParallelismAnalysisFailed(String),

    /// Plan too complex for analysis
    #[error("Plan too complex: {0}")]
    TooComplex(String),

    /// Invalid plan structure
    #[error("Invalid plan: {0}")]
    InvalidPlan(String),

    /// Inconsistent hints detected
    #[error("Inconsistent hints: {0}")]
    InconsistentHints(String),
}

impl ErrorCode for IRError {
    fn code(&self) -> &'static str {
        match self {
            IRError::InvalidInstruction(_) => "GATE_C_IR_INVALID_INSTRUCTION",
            IRError::DependencyAnalysisFailed(_) => "GATE_C_IR_DEPENDENCY_ANALYSIS_FAILED",
            IRError::OrderingHintFailed(_) => "GATE_C_IR_ORDERING_HINT_FAILED",
            IRError::ParallelismAnalysisFailed(_) => "GATE_C_IR_PARALLELISM_ANALYSIS_FAILED",
            IRError::TooComplex(_) => "GATE_C_IR_TOO_COMPLEX",
            IRError::InvalidPlan(_) => "GATE_C_IR_INVALID_PLAN",
            IRError::InconsistentHints(_) => "GATE_C_IR_INCONSISTENT_HINTS",
        }
    }
}

/// Normalization errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizationError {
    /// Ambiguous plan (cannot determine canonical form)
    #[error("Ambiguous plan: {0}")]
    AmbiguousPlan(String),

    /// Invalid reference in plan
    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    /// Structural error in plan
    #[error("Structural error: {0}")]
    StructuralError(String),

    /// Plan too complex
    #[error("Plan too complex: {0}")]
    TooComplex(String),
}

impl ErrorCode for NormalizationError {
    fn code(&self) -> &'static str {
        match self {
            NormalizationError::AmbiguousPlan(_) => "GATE_C_NORMALIZATION_AMBIGUOUS_PLAN",
            NormalizationError::InvalidReference(_) => "GATE_C_NORMALIZATION_INVALID_REFERENCE",
            NormalizationError::StructuralError(_) => "GATE_C_NORMALIZATION_STRUCTURAL_ERROR",
            NormalizationError::TooComplex(_) => "GATE_C_NORMALIZATION_TOO_COMPLEX",
        }
    }
}

/// Security operation errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SecurityError {
    /// Capability check failed
    #[error("Capability check failed: {0}")]
    CapabilityCheckFailed(String),

    /// Redaction failed
    #[error("Redaction failed: {0}")]
    RedactionFailed(String),

    /// Audit logging failed
    #[error("Audit logging failed: {0}")]
    AuditLoggingFailed(String),

    /// Security inspection failed
    #[error("Security inspection failed: {0}")]
    InspectionFailed(String),
}

impl ErrorCode for SecurityError {
    fn code(&self) -> &'static str {
        match self {
            SecurityError::CapabilityCheckFailed(_) => "GATE_C_SECURITY_CAPABILITY_CHECK_FAILED",
            SecurityError::RedactionFailed(_) => "GATE_C_SECURITY_REDACTION_FAILED",
            SecurityError::AuditLoggingFailed(_) => "GATE_C_SECURITY_AUDIT_LOGGING_FAILED",
            SecurityError::InspectionFailed(_) => "GATE_C_SECURITY_INSPECTION_FAILED",
        }
    }
}

/// REPL rendering errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RenderError {
    /// Rendering failed
    #[error("Rendering failed: {0}")]
    RenderingFailed(String),

    /// Output too large
    #[error("Output too large: {size} bytes exceeds limit of {limit}")]
    OutputTooLarge { size: usize, limit: usize },

    /// Visualization failed
    #[error("Visualization failed: {0}")]
    VisualizationFailed(String),

    /// Explanation generation failed
    #[error("Explanation generation failed: {0}")]
    ExplanationFailed(String),
}

impl ErrorCode for RenderError {
    fn code(&self) -> &'static str {
        match self {
            RenderError::RenderingFailed(_) => "GATE_C_RENDER_RENDERING_FAILED",
            RenderError::OutputTooLarge { .. } => "GATE_C_RENDER_OUTPUT_TOO_LARGE",
            RenderError::VisualizationFailed(_) => "GATE_C_RENDER_VISUALIZATION_FAILED",
            RenderError::ExplanationFailed(_) => "GATE_C_RENDER_EXPLANATION_FAILED",
        }
    }
}

/// Cycle error for dependency analysis
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CycleError {
    /// Cycle detected in dependency graph
    #[error("Cycle detected: {0}")]
    CycleDetected(String),
}

/// Order error for canonicalization
#[derive(Debug, Clone, PartialEq, Error)]
pub enum OrderError {
    /// Cannot determine canonical order
    #[error("Cannot determine canonical order: {0}")]
    CannotDetermineOrder(String),

    /// Ambiguous ordering
    #[error("Ambiguous ordering: {0}")]
    AmbiguousOrdering(String),
}

/// Validation error for structural validation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ValidationError {
    /// Invalid structure
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    /// Invalid field value
    #[error("Invalid field value: {0}")]
    InvalidFieldValue(String),
}

/// Capability error for permission validation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CapabilityError {
    /// Insufficient permissions
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    /// Capability not found
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    /// Capability validation failed
    #[error("Capability validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submission_error_codes() {
        let error = SubmissionError::OrchestratorUnavailable;
        assert_eq!(error.error_code(), SubmissionErrorCode::OrchUnavailable);
        assert_eq!(error.code(), "GATE_C_SUBMISSION_ORCH_UNAVAILABLE");

        let error = SubmissionError::InvalidPlan("test".to_string());
        assert_eq!(error.error_code(), SubmissionErrorCode::InvalidPlan);
        assert_eq!(error.code(), "GATE_C_SUBMISSION_INVALID_PLAN");

        let error = SubmissionError::CapabilityDenied("test".to_string());
        assert_eq!(error.error_code(), SubmissionErrorCode::CapabilityDenied);
        assert_eq!(error.code(), "GATE_C_SUBMISSION_CAPABILITY_DENIED");
    }

    #[test]
    fn test_error_determinism() {
        let error1 = SubmissionError::OrchestratorUnavailable;
        let error2 = SubmissionError::OrchestratorUnavailable;
        assert_eq!(error1, error2);
        assert_eq!(error1.error_code(), error2.error_code());
        assert_eq!(error1.code(), error2.code());
    }

    #[test]
    fn test_gate_c_error_codes() {
        use crate::gate_c::error::ErrorCode;

        let mutation_error = MutationError::MutationConflict("test".to_string());
        assert_eq!(mutation_error.code(), "GATE_C_MUTATION_CONFLICT");

        let pipeline_error = PipelineError::CycleDetected("test".to_string());
        assert_eq!(pipeline_error.code(), "GATE_C_PIPELINE_CYCLE_DETECTED");

        let gate_c_error = GateCError::Mutation(mutation_error);
        assert_eq!(gate_c_error.code(), "GATE_C_MUTATION_CONFLICT");
    }

    #[test]
    fn test_pipeline_error_formatting() {
        let error = PipelineError::PipelineTooLarge {
            steps: 150,
            limit: 128,
        };
        let formatted = format!("{}", error);
        assert!(formatted.contains("150"));
        assert!(formatted.contains("128"));
    }

    #[test]
    fn test_render_error_formatting() {
        let error = RenderError::OutputTooLarge {
            size: 70000,
            limit: 64000,
        };
        let formatted = format!("{}", error);
        assert!(formatted.contains("70000"));
        assert!(formatted.contains("64000"));
    }
}
