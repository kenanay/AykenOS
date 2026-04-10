//! Error types for parallelism operations
//!
//! This module defines the error types used throughout the D2 Parallelism Architecture
//! to handle various failure modes. The error types provide comprehensive diagnostic
//! information for debugging and error handling.
//!
//! **Design Reference:** D2 Parallelism Architecture - Error Handling section
//! **Requirements:** 8.1, 8.2

use crate::bcib::Value;
use crate::execution_plan::BlockId;
use crate::ir_planner::ExecutionError;
use std::error::Error;
use std::fmt;

/// Result type alias for parallelism operations.
///
/// This is a convenience type alias that uses `ParallelismError` as the error type.
pub type ParallelismResult<T> = Result<T, ParallelismError>;

/// Error types for parallelism operations.
///
/// This enum categorizes all possible errors that can occur during parallel execution,
/// providing detailed context for each failure mode.
///
/// # Error Categories
///
/// 1. **SafetyViolation**: Attempting to parallelize unsafe operations
/// 2. **ExecutionError**: Runtime errors during parallel execution
/// 3. **DeterminismViolation**: Parallel results differ from sequential results
/// 4. **PerformanceDegradation**: Parallelism provides insufficient benefit
/// 5. **ConstitutionalViolation**: Violation of constitutional principles (FATAL)
/// 6. **SecurityError**: Security boundary violations
///
/// **CONSTITUTIONAL:** Determinism and Safety violations are FATAL errors.
///
/// **Validates: Requirements 8.1, 8.2**
#[derive(Debug)]
pub enum ParallelismError {
    /// Attempting to parallelize an operation that is not safe for parallel execution.
    ///
    /// This error occurs when:
    /// - An IR block marked as `ParallelSafety::Unsafe` is submitted for parallel execution
    /// - Shared mutable state is detected between workers
    /// - Native code purity constraints are violated
    ///
    /// **Response:** Fail fast with detailed error message, disable parallelism for the operation
    SafetyViolation {
        /// Identifier of the IR block that violated safety constraints
        block_id: BlockId,

        /// Human-readable explanation of the safety violation
        reason: String,
    },

    /// Runtime error during parallel execution.
    ///
    /// This error occurs when:
    /// - A worker panics during parallel execution
    /// - Thread pool exhaustion or failure
    /// - Memory allocation failures
    /// - Execution errors within a worker
    ///
    /// **Response:** Propagate error to caller with full context, use catch_unwind for panic containment
    ExecutionError {
        /// Identifier of the worker that encountered the error (if applicable)
        worker_id: Option<usize>,

        /// Starting index of the partition being processed when error occurred
        partition_start: Option<usize>,

        /// Ending index of the partition being processed when error occurred
        partition_end: Option<usize>,

        /// Detailed error message
        message: String,

        /// Optional source error for error chaining
        #[allow(dead_code)]
        source: Option<Box<dyn Error + Send + Sync>>,
    },

    /// Verification mode detected that parallel execution produced different results
    /// than sequential execution.
    ///
    /// This error occurs when:
    /// - Verification mode detects parallel != sequential
    /// - Replay produces different results than original
    /// - Index map produces non-deterministic ordering
    ///
    /// **Response:** Log detailed diagnostics, disable parallelism, fall back to sequential execution
    DeterminismViolation {
        /// Result produced by parallel execution
        parallel_result: Value,

        /// Result produced by sequential execution (expected result)
        sequential_result: Value,

        /// Input value that produced the mismatch
        input: Value,

        /// Additional diagnostic information
        diagnostics: String,
    },

    /// Parallelism provides insufficient performance benefit.
    ///
    /// This error occurs when:
    /// - Net speedup < 2.0x
    /// - Ordering overhead > 50%
    /// - Cache-line contention detected
    ///
    /// **Response:** Blacklist operation, use sequential execution, log performance metrics
    PerformanceDegradation {
        /// Measured net speedup (including all overhead)
        net_speedup: f64,

        /// Ratio of ordering overhead to parallel execution time
        overhead_ratio: f64,

        /// Human-readable explanation of the performance issue
        reason: String,
    },

    /// Thread pool initialization failure.
    ///
    /// This error occurs when:
    /// - The thread pool has already been initialized
    /// - Rayon fails to build the thread pool
    /// - Invalid thread pool configuration
    ///
    /// **Response:** Fail fast with detailed error message
    ///
    /// **Validates: Requirement 5.4** - Thread pool errors handled gracefully
    ThreadPoolInitialization {
        /// Human-readable explanation of the initialization failure
        reason: String,
    },

    /// Constitutional principle violation (FATAL).
    ///
    /// This error occurs when:
    /// - Determinism enforcement is disabled in production
    /// - Replay capability is disabled
    /// - Safety verification is bypassed
    /// - Any constitutional principle is violated
    ///
    /// **CONSTITUTIONAL:** These errors are ALWAYS fatal and cause immediate system shutdown.
    ///
    /// **Response:** Immediate panic with detailed constitutional violation report
    ConstitutionalViolation {
        /// The constitutional principle that was violated
        principle: String,

        /// Detailed description of the violation
        violation: String,
    },

    /// Security boundary violation.
    ///
    /// This error occurs when:
    /// - Unauthorized access to privileged metrics
    /// - Constitutional authority verification fails
    /// - Security boundary enforcement fails
    ///
    /// **Response:** Log security event, deny access, potentially blacklist operation
    SecurityError {
        /// Detailed description of the security violation
        message: String,
    },
}

impl fmt::Display for ParallelismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParallelismError::SafetyViolation { block_id, reason } => {
                write!(
                    f,
                    "Parallelism safety violation in block {:?}: {}",
                    block_id, reason
                )
            }
            ParallelismError::ExecutionError {
                worker_id,
                partition_start,
                partition_end,
                message,
                ..
            } => {
                write!(f, "Parallel execution error")?;
                if let Some(id) = worker_id {
                    write!(f, " in worker {}", id)?;
                }
                if let (Some(start), Some(end)) = (partition_start, partition_end) {
                    write!(f, " (partition {}..{})", start, end)?;
                }
                write!(f, ": {}", message)
            }
            ParallelismError::DeterminismViolation {
                parallel_result,
                sequential_result,
                input,
                diagnostics,
            } => {
                write!(
                    f,
                    "Determinism violation: parallel result {:?} != sequential result {:?} for input {:?}. {}",
                    parallel_result, sequential_result, input, diagnostics
                )
            }
            ParallelismError::PerformanceDegradation {
                net_speedup,
                overhead_ratio,
                reason,
            } => {
                write!(
                    f,
                    "Performance degradation: net speedup {:.2}x, overhead ratio {:.2}%. {}",
                    net_speedup,
                    overhead_ratio * 100.0,
                    reason
                )
            }
            ParallelismError::ThreadPoolInitialization { reason } => {
                write!(f, "Thread pool initialization failed: {}", reason)
            }
            ParallelismError::ConstitutionalViolation {
                principle,
                violation,
            } => {
                write!(f, "CONSTITUTIONAL VIOLATION: {} - {}", principle, violation)
            }
            ParallelismError::SecurityError { message } => {
                write!(f, "Security error: {}", message)
            }
        }
    }
}

impl Error for ParallelismError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParallelismError::ExecutionError { source: _, .. } => {
                // Note: We can't return the source here because it's Box<dyn Error + Send + Sync>
                // and we need &(dyn Error + 'static). This is a known limitation.
                // In practice, the error message will contain the source information.
                None
            }
            _ => None,
        }
    }
}

// Convenience constructors for common error scenarios
impl ParallelismError {
    /// Creates a safety violation error for an unsafe IR block.
    pub fn unsafe_block(block_id: BlockId) -> Self {
        ParallelismError::SafetyViolation {
            block_id,
            reason: "IR block is marked as ParallelSafety::Unsafe and cannot be parallelized"
                .to_string(),
        }
    }

    /// Creates a safety violation error for shared mutable state.
    pub fn shared_mutable_state(block_id: BlockId) -> Self {
        ParallelismError::SafetyViolation {
            block_id,
            reason: "Shared mutable state detected between parallel workers".to_string(),
        }
    }

    /// Creates a safety violation error for native code purity constraint violation.
    pub fn native_code_impurity(block_id: BlockId, details: String) -> Self {
        ParallelismError::SafetyViolation {
            block_id,
            reason: format!("Native code purity constraint violated: {}", details),
        }
    }

    /// Creates an execution error from a worker panic.
    pub fn worker_panic(
        worker_id: usize,
        partition_start: usize,
        partition_end: usize,
        panic_message: String,
    ) -> Self {
        ParallelismError::ExecutionError {
            worker_id: Some(worker_id),
            partition_start: Some(partition_start),
            partition_end: Some(partition_end),
            message: format!("Worker panicked: {}", panic_message),
            source: None,
        }
    }

    /// Creates an execution error from a thread pool failure.
    pub fn thread_pool_failure(message: String) -> Self {
        ParallelismError::ExecutionError {
            worker_id: None,
            partition_start: None,
            partition_end: None,
            message: format!("Thread pool failure: {}", message),
            source: None,
        }
    }

    /// Creates a determinism violation error with diagnostic information.
    pub fn determinism_mismatch(
        parallel_result: Value,
        sequential_result: Value,
        input: Value,
        context: String,
    ) -> Self {
        ParallelismError::DeterminismViolation {
            parallel_result,
            sequential_result,
            input,
            diagnostics: context,
        }
    }

    /// Creates a performance degradation error for low speedup.
    pub fn low_speedup(net_speedup: f64, overhead_ratio: f64) -> Self {
        ParallelismError::PerformanceDegradation {
            net_speedup,
            overhead_ratio,
            reason: format!(
                "Net speedup {:.2}x is below the 2.0x threshold required for parallelism",
                net_speedup
            ),
        }
    }

    /// Creates a performance degradation error for high overhead.
    pub fn high_overhead(overhead_ratio: f64) -> Self {
        ParallelismError::PerformanceDegradation {
            net_speedup: 0.0,
            overhead_ratio,
            reason: format!(
                "Ordering overhead {:.1}% exceeds the 50% threshold",
                overhead_ratio * 100.0
            ),
        }
    }

    /// Creates a constitutional violation error (FATAL).
    ///
    /// **CONSTITUTIONAL:** This error type is reserved for violations of
    /// constitutional principles and MUST cause immediate system shutdown.
    pub fn constitutional_violation(principle: String, violation: String) -> Self {
        ParallelismError::ConstitutionalViolation {
            principle,
            violation,
        }
    }

    /// Creates a security error for unauthorized access.
    pub fn security_violation(message: String) -> Self {
        ParallelismError::SecurityError { message }
    }

    /// Checks if this error represents a constitutional violation.
    ///
    /// **CONSTITUTIONAL:** Constitutional violations are FATAL and must
    /// cause immediate system shutdown.
    pub fn is_constitutional_violation(&self) -> bool {
        matches!(self, ParallelismError::ConstitutionalViolation { .. })
    }

    /// Checks if this error should cause operation blacklisting.
    pub fn should_blacklist(&self) -> bool {
        matches!(
            self,
            ParallelismError::PerformanceDegradation { .. }
                | ParallelismError::ExecutionError { .. }
        )
    }

    /// Checks if this error should cause fallback to sequential execution.
    pub fn should_fallback(&self) -> bool {
        matches!(
            self,
            ParallelismError::SafetyViolation { .. }
                | ParallelismError::DeterminismViolation { .. }
                | ParallelismError::ThreadPoolInitialization { .. }
        )
    }

    /// Checks if this error is fatal and should cause system shutdown.
    ///
    /// **CONSTITUTIONAL:** Constitutional violations are always fatal.
    pub fn is_fatal(&self) -> bool {
        matches!(self, ParallelismError::ConstitutionalViolation { .. })
    }
}

// ===== Error Conversions =====
//
// These conversions allow seamless integration with existing error types
// in the semantic-cli codebase.

/// Convert ExecutionError to ParallelismError.
///
/// This conversion wraps execution errors that occur during parallel execution
/// into the ParallelismError::ExecutionError variant, preserving the original
/// error message for debugging.
///
/// **Validates: Requirement 8.1 (Error Propagation)**
impl From<ExecutionError> for ParallelismError {
    fn from(error: ExecutionError) -> Self {
        ParallelismError::ExecutionError {
            worker_id: None,
            partition_start: None,
            partition_end: None,
            message: error.to_string(),
            source: None,
        }
    }
}

/// Convert from std::io::Error for thread pool and system-level errors.
impl From<std::io::Error> for ParallelismError {
    fn from(error: std::io::Error) -> Self {
        ParallelismError::thread_pool_failure(format!("IO error: {}", error))
    }
}

/// Convert from Box<dyn Error> for generic error handling.
impl From<Box<dyn Error + Send + Sync>> for ParallelismError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        ParallelismError::ExecutionError {
            worker_id: None,
            partition_start: None,
            partition_end: None,
            message: error.to_string(),
            source: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;

    #[test]
    fn test_safety_violation_display() {
        let error = ParallelismError::SafetyViolation {
            block_id: 42,
            reason: "Test reason".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("safety violation"));
        assert!(display.contains("42"));
        assert!(display.contains("Test reason"));
    }

    #[test]
    fn test_execution_error_display() {
        let error = ParallelismError::ExecutionError {
            worker_id: Some(3),
            partition_start: Some(100),
            partition_end: Some(200),
            message: "Test error".to_string(),
            source: None,
        };

        let display = format!("{}", error);
        assert!(display.contains("execution error"));
        assert!(display.contains("worker 3"));
        assert!(display.contains("partition 100..200"));
        assert!(display.contains("Test error"));
    }

    #[test]
    fn test_determinism_violation_display() {
        let error = ParallelismError::DeterminismViolation {
            parallel_result: Value::Number(1.0),
            sequential_result: Value::Number(2.0),
            input: Value::Number(3.0),
            diagnostics: "Test diagnostics".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Determinism violation"));
        assert!(display.contains("Test diagnostics"));
    }

    #[test]
    fn test_performance_degradation_display() {
        let error = ParallelismError::PerformanceDegradation {
            net_speedup: 1.5,
            overhead_ratio: 0.6,
            reason: "Test reason".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Performance degradation"));
        assert!(display.contains("1.50x"));
        assert!(display.contains("60.")); // More flexible - could be 60.0% or 60.00%
        assert!(display.contains("Test reason"));
    }

    #[test]
    fn test_unsafe_block_constructor() {
        let error = ParallelismError::unsafe_block(10);

        match error {
            ParallelismError::SafetyViolation { block_id, reason } => {
                assert_eq!(block_id, 10);
                assert!(reason.contains("ParallelSafety::Unsafe"));
            }
            _ => panic!("Expected SafetyViolation"),
        }
    }

    #[test]
    fn test_worker_panic_constructor() {
        let error = ParallelismError::worker_panic(2, 50, 100, "Division by zero".to_string());

        match error {
            ParallelismError::ExecutionError {
                worker_id,
                partition_start,
                partition_end,
                message,
                ..
            } => {
                assert_eq!(worker_id, Some(2));
                assert_eq!(partition_start, Some(50));
                assert_eq!(partition_end, Some(100));
                assert!(message.contains("Division by zero"));
            }
            _ => panic!("Expected ExecutionError"),
        }
    }

    #[test]
    fn test_low_speedup_constructor() {
        let error = ParallelismError::low_speedup(1.5, 0.3);

        match error {
            ParallelismError::PerformanceDegradation {
                net_speedup,
                overhead_ratio,
                reason,
            } => {
                assert_eq!(net_speedup, 1.5);
                assert_eq!(overhead_ratio, 0.3);
                assert!(reason.contains("2.0x threshold"));
            }
            _ => panic!("Expected PerformanceDegradation"),
        }
    }

    #[test]
    fn test_high_overhead_constructor() {
        let error = ParallelismError::high_overhead(0.65);

        match error {
            ParallelismError::PerformanceDegradation {
                overhead_ratio,
                reason,
                ..
            } => {
                assert_eq!(overhead_ratio, 0.65);
                assert!(reason.contains("50% threshold"));
            }
            _ => panic!("Expected PerformanceDegradation"),
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = ParallelismError::unsafe_block(1);

        // Test that Error trait is implemented
        let _: &dyn Error = &error;

        // Test Display through Error trait
        let display = format!("{}", error);
        assert!(!display.is_empty());
    }

    // ===== Error Conversion Tests =====

    #[test]
    fn test_from_execution_error() {
        let exec_error = ExecutionError::InvalidOperation {
            operation: "test_op".to_string(),
        };

        let parallelism_error: ParallelismError = exec_error.into();

        match parallelism_error {
            ParallelismError::ExecutionError { message, .. } => {
                assert!(message.contains("test_op"));
            }
            _ => panic!("Expected ExecutionError variant"),
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test IO error");

        let parallelism_error: ParallelismError = io_error.into();

        match parallelism_error {
            ParallelismError::ExecutionError { message, .. } => {
                assert!(message.contains("IO error"));
                assert!(message.contains("test IO error"));
            }
            _ => panic!("Expected ExecutionError variant"),
        }
    }

    #[test]
    fn test_from_boxed_error() {
        let boxed_error: Box<dyn Error + Send + Sync> = Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "boxed error",
        ));

        let parallelism_error: ParallelismError = boxed_error.into();

        match parallelism_error {
            ParallelismError::ExecutionError {
                message, source, ..
            } => {
                assert!(message.contains("boxed error"));
                assert!(source.is_some());
            }
            _ => panic!("Expected ExecutionError variant"),
        }
    }
}
