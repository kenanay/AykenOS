//! Loop Error Types and Recovery Policies - Constitutional Alignment (Phase 0.5)
//!
//! This module implements the error taxonomy and recovery policies following
//! the constitutional decisions from Phase 0.5.
//!
//! # Constitutional Error Taxonomy (LOCKED)
//!
//! ```text
//! LoopError
//! ├── IterationLimitExceeded
//! ├── BudgetTimeoutExceeded
//! ├── TypeMismatch
//! ├── UnorderedCollectionRejected
//! ├── LoopBodyError
//! ├── BreakSignal
//! ├── ContinueSignal
//!
//! EnvironmentFault
//! ├── WallClockKill
//! ```
//!
//! # Constitutional Guarantees
//!
//! - Error recovery policies are explicit only (no automatic recovery)
//! - Retry limits are bounded (max 3 retries, new limit ≤ 10,000)
//! - Partial results require explicit configuration
//! - Environment faults are separate from semantic errors

use crate::bcib::{Value, ValueType};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error recovery policies (Constitutional: explicit only)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorRecoveryPolicy {
    /// Fail immediately (default)
    FailFast,
    /// Return partial results
    ReturnPartial,
    /// Retry with bounded attempts (max 3)
    RetryBounded { max_attempts: u8 }, // max_attempts ≤ 3
}

impl Default for ErrorRecoveryPolicy {
    fn default() -> Self {
        Self::FailFast
    }
}

/// Recovery actions for custom recovery policies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Fail with the original error
    Fail,
    /// Return partial results
    ReturnPartial,
    /// Retry with new configuration
    Retry {
        new_limit: Option<u32>,
        new_budget: Option<u64>,
    },
}

impl ErrorRecoveryPolicy {
    /// Create a retry policy with bounded attempts (Constitutional limit: max 3)
    pub fn retry_bounded(max_attempts: u8) -> Result<Self, String> {
        if max_attempts == 0 {
            return Err("Retry attempts must be at least 1".to_string());
        }
        if max_attempts > 3 {
            return Err("Constitutional limit: maximum 3 retry attempts".to_string());
        }
        Ok(Self::RetryBounded { max_attempts })
    }

    /// Check if this policy allows retries
    pub fn allows_retries(&self) -> bool {
        matches!(self, Self::RetryBounded { .. })
    }

    /// Get maximum retry attempts (0 if no retries allowed)
    pub fn max_retries(&self) -> u8 {
        match self {
            Self::RetryBounded { max_attempts } => *max_attempts,
            _ => 0,
        }
    }
}

/// Loop execution errors (Constitutional Alignment - Phase 0.5)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoopError {
    /// Iteration limit exceeded (Constitutional: exactness guarantee)
    IterationLimitExceeded { limit: u32, completed: u32 },
    /// Budget timeout exceeded (Constitutional: deterministic)
    BudgetTimeoutExceeded {
        budget: u64,
        consumed: u64,
        iterations_completed: u32,
    },
    /// Accumulator type mismatch during iteration (LOCKED NAME - Phase 0.5)
    AccumulatorTypeMismatch {
        expected: ValueType,
        actual: ValueType,
        iteration: u32,
        accumulator_name: String,
    },
    /// Unordered collection rejected for deterministic iteration
    UnorderedCollectionRejected {
        collection_type: String,
        reason: String,
    },
    /// Error in loop body execution
    LoopBodyError { iteration: u32, error: String },
    /// Break signal (early termination)
    BreakSignal { iteration: u32, accumulator: Value },
    /// Continue signal (skip iteration)
    ContinueSignal { iteration: u32 },
}

impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IterationLimitExceeded { limit, completed } => {
                write!(
                    f,
                    "Iteration limit of {} exceeded (completed: {})",
                    limit, completed
                )
            }
            Self::BudgetTimeoutExceeded {
                budget,
                consumed,
                iterations_completed,
            } => {
                write!(
                    f,
                    "Budget timeout of {} exceeded (consumed: {}, iterations: {})",
                    budget, consumed, iterations_completed
                )
            }
            Self::AccumulatorTypeMismatch {
                expected,
                actual,
                iteration,
                accumulator_name,
            } => {
                write!(
                    f,
                    "Accumulator type mismatch in '{}' at iteration {}: expected {:?}, got {:?}",
                    accumulator_name, iteration, expected, actual
                )
            }
            Self::UnorderedCollectionRejected {
                collection_type,
                reason,
            } => {
                write!(
                    f,
                    "Unordered collection '{}' rejected: {}",
                    collection_type, reason
                )
            }
            Self::LoopBodyError { iteration, error } => {
                write!(f, "Loop body error at iteration {}: {}", iteration, error)
            }
            Self::BreakSignal { iteration, .. } => {
                write!(f, "Break signal at iteration {}", iteration)
            }
            Self::ContinueSignal { iteration } => {
                write!(f, "Continue signal at iteration {}", iteration)
            }
        }
    }
}

impl std::error::Error for LoopError {}

impl LoopError {
    /// Get the canonical error code for this error (Constitutional requirement)
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::IterationLimitExceeded { .. } => "LE001",
            Self::BudgetTimeoutExceeded { .. } => "LE002",
            Self::AccumulatorTypeMismatch { .. } => "LE003",
            Self::UnorderedCollectionRejected { .. } => "LE004",
            Self::LoopBodyError { .. } => "LE005",
            Self::BreakSignal { .. } => "LE006",
            Self::ContinueSignal { .. } => "LE007",
        }
    }

    /// Check if this error is recoverable (Constitutional classification)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::IterationLimitExceeded { .. } => true,
            Self::BudgetTimeoutExceeded { .. } => true,
            Self::AccumulatorTypeMismatch { .. } => false,
            Self::UnorderedCollectionRejected { .. } => false,
            Self::LoopBodyError { .. } => true,   // Configurable
            Self::BreakSignal { .. } => false,    // Control flow, not error
            Self::ContinueSignal { .. } => false, // Control flow, not error
        }
    }

    /// Check if this error supports partial results (Constitutional guarantee)
    pub fn supports_partial_results(&self) -> bool {
        match self {
            Self::IterationLimitExceeded { .. } => true, // POST-INCREMENT
            Self::BudgetTimeoutExceeded { .. } => true,  // POST-INCREMENT
            Self::AccumulatorTypeMismatch { .. } => true, // PRE-COMMIT
            Self::UnorderedCollectionRejected { .. } => false,
            Self::LoopBodyError { .. } => true, // Configurable
            Self::BreakSignal { .. } => true,   // Break value
            Self::ContinueSignal { .. } => false,
        }
    }
}

/// Environment faults (Constitutional: separate from semantic errors)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvironmentFault {
    /// Wall-clock kill switch triggered (Constitutional: non-semantic)
    WallClockKill { elapsed_ms: u64, limit_ms: u64 },
}

impl fmt::Display for EnvironmentFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WallClockKill {
                elapsed_ms,
                limit_ms,
            } => {
                write!(
                    f,
                    "Wall-clock kill switch triggered: {}ms elapsed (limit: {}ms)",
                    elapsed_ms, limit_ms
                )
            }
        }
    }
}

impl std::error::Error for EnvironmentFault {}

impl EnvironmentFault {
    /// Get the canonical error code for this fault (Constitutional requirement)
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::WallClockKill { .. } => "EF001",
        }
    }

    /// Environment faults are never recoverable (Constitutional principle)
    pub fn is_recoverable(&self) -> bool {
        false
    }

    /// Environment faults support partial results (current state capture)
    pub fn supports_partial_results(&self) -> bool {
        true
    }
}

/// Partial results with termination metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialResult {
    /// Final accumulator state (Constitutional: POST-INCREMENT capture)
    pub accumulator: Value,
    /// Number of completed iterations
    pub iterations_completed: u32,
    /// Reason for termination
    pub termination_reason: TerminationReason,
    /// Optional error information
    pub error_info: Option<String>,
}

impl PartialResult {
    /// Create a new partial result
    pub fn new(
        accumulator: Value,
        iterations_completed: u32,
        termination_reason: TerminationReason,
    ) -> Self {
        Self {
            accumulator,
            iterations_completed,
            termination_reason,
            error_info: None,
        }
    }

    /// Create a partial result with error information
    pub fn with_error(
        accumulator: Value,
        iterations_completed: u32,
        termination_reason: TerminationReason,
        error_info: String,
    ) -> Self {
        Self {
            accumulator,
            iterations_completed,
            termination_reason,
            error_info: Some(error_info),
        }
    }
}

/// Termination reasons for partial results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationReason {
    /// Iteration limit reached
    IterationLimitReached,
    /// Budget timeout reached
    BudgetTimeoutReached,
    /// Break statement executed
    BreakExecuted,
    /// Loop condition became false (While loops)
    ConditionFalse,
    /// Loop body error occurred
    LoopBodyError,
    /// Environment fault (wall-clock kill switch)
    EnvironmentFault,
}

impl fmt::Display for TerminationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IterationLimitReached => write!(f, "iteration limit reached"),
            Self::BudgetTimeoutReached => write!(f, "budget timeout reached"),
            Self::BreakExecuted => write!(f, "break executed"),
            Self::ConditionFalse => write!(f, "condition became false"),
            Self::LoopBodyError => write!(f, "loop body error"),
            Self::EnvironmentFault => write!(f, "environment fault"),
        }
    }
}

/// Loop execution result (success or partial)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoopResult {
    /// Successful completion
    Success {
        accumulator: Value,
        iterations_completed: u32,
    },
    /// Partial result (with configured policy)
    Partial(PartialResult),
    /// Error (no partial results)
    Error(LoopError),
    /// Environment fault
    EnvironmentFault(EnvironmentFault),
    /// Control flow result (break/continue)
    ControlFlow(ControlFlowResult),
}

/// Rich loop execution result with execution semantics (Phase 6.2 - JIT Integration)
///
/// This type formalizes the execution contract between the loop executor and JIT system.
/// It provides rich execution metadata while maintaining constitutional compliance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichLoopExecutionResult {
    /// Execution status
    pub status: LoopExecutionStatus,
    /// Number of iterations completed
    pub iterations_completed: u32,
    /// Final accumulator value
    pub accumulator: Value,
    /// Execution mode used
    pub execution_mode: ExecutionMode,
}

/// Loop execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopExecutionStatus {
    /// Loop completed successfully
    Success,
    /// Loop terminated due to budget timeout
    BudgetExceeded,
    /// Loop terminated due to iteration limit
    IterationLimitReached,
    /// Loop terminated due to type error
    TypeError,
    /// Loop terminated due to break statement
    Break,
    /// Loop terminated due to environment fault
    EnvironmentFault,
}

/// Execution mode used for the loop
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Interpreted execution
    Interpreted,
    /// JIT compiled execution
    JIT,
    /// Parallel execution (Phase 7.2)
    Parallel,
}

/// Control flow execution results (Phase 2.3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlFlowResult {
    /// Break result - early termination
    Break {
        accumulator: Value,
        iterations_completed: u32,
    },
    /// Continue result - skip remaining body
    Continue { iterations_completed: u32 },
}

impl RichLoopExecutionResult {
    /// Create a successful execution result
    pub fn success(
        accumulator: Value,
        iterations_completed: u32,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            status: LoopExecutionStatus::Success,
            iterations_completed,
            accumulator,
            execution_mode,
        }
    }

    /// Create a budget exceeded result
    pub fn budget_exceeded(
        accumulator: Value,
        iterations_completed: u32,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            status: LoopExecutionStatus::BudgetExceeded,
            iterations_completed,
            accumulator,
            execution_mode,
        }
    }

    /// Create an iteration limit reached result
    pub fn iteration_limit_reached(
        accumulator: Value,
        iterations_completed: u32,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            status: LoopExecutionStatus::IterationLimitReached,
            iterations_completed,
            accumulator,
            execution_mode,
        }
    }

    /// Create a type error result
    pub fn type_error(
        accumulator: Value,
        iterations_completed: u32,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            status: LoopExecutionStatus::TypeError,
            iterations_completed,
            accumulator,
            execution_mode,
        }
    }

    /// Create a break result
    pub fn break_result(
        accumulator: Value,
        iterations_completed: u32,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            status: LoopExecutionStatus::Break,
            iterations_completed,
            accumulator,
            execution_mode,
        }
    }

    /// Create an environment fault result
    pub fn environment_fault(
        accumulator: Value,
        iterations_completed: u32,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            status: LoopExecutionStatus::EnvironmentFault,
            iterations_completed,
            accumulator,
            execution_mode,
        }
    }

    /// Check if the execution was successful
    pub fn is_success(&self) -> bool {
        matches!(self.status, LoopExecutionStatus::Success)
    }

    /// Check if the execution was terminated due to break
    pub fn is_break(&self) -> bool {
        matches!(self.status, LoopExecutionStatus::Break)
    }

    /// Check if the execution encountered an error
    pub fn is_error(&self) -> bool {
        matches!(
            self.status,
            LoopExecutionStatus::BudgetExceeded
                | LoopExecutionStatus::IterationLimitReached
                | LoopExecutionStatus::TypeError
                | LoopExecutionStatus::EnvironmentFault
        )
    }

    /// Convert from LoopResult to LoopExecutionResult
    pub fn from_loop_result(result: LoopResult, execution_mode: ExecutionMode) -> Self {
        match result {
            LoopResult::Success {
                accumulator,
                iterations_completed,
            } => Self::success(accumulator, iterations_completed, execution_mode),
            LoopResult::Partial(partial) => match partial.termination_reason {
                TerminationReason::BudgetTimeoutReached => Self::budget_exceeded(
                    partial.accumulator,
                    partial.iterations_completed,
                    execution_mode,
                ),
                TerminationReason::IterationLimitReached => Self::iteration_limit_reached(
                    partial.accumulator,
                    partial.iterations_completed,
                    execution_mode,
                ),
                TerminationReason::BreakExecuted => Self::break_result(
                    partial.accumulator,
                    partial.iterations_completed,
                    execution_mode,
                ),
                TerminationReason::EnvironmentFault => Self::environment_fault(
                    partial.accumulator,
                    partial.iterations_completed,
                    execution_mode,
                ),
                _ => Self::success(
                    partial.accumulator,
                    partial.iterations_completed,
                    execution_mode,
                ),
            },
            LoopResult::Error(error) => {
                // For errors, we need to provide a default accumulator and iteration count
                // This represents the state at the point of error
                let (accumulator, iterations) = match &error {
                    LoopError::IterationLimitExceeded { completed, .. } => {
                        (Value::Number(0.0), *completed) // Default accumulator for limit errors
                    }
                    LoopError::BudgetTimeoutExceeded {
                        iterations_completed,
                        ..
                    } => {
                        (Value::Number(0.0), *iterations_completed) // Default accumulator for timeout errors
                    }
                    LoopError::AccumulatorTypeMismatch { iteration, .. } => {
                        (Value::Number(0.0), *iteration) // Default accumulator for type errors
                    }
                    _ => (Value::Number(0.0), 0), // Default for other errors
                };

                match error {
                    LoopError::IterationLimitExceeded { .. } => {
                        Self::iteration_limit_reached(accumulator, iterations, execution_mode)
                    }
                    LoopError::BudgetTimeoutExceeded { .. } => {
                        Self::budget_exceeded(accumulator, iterations, execution_mode)
                    }
                    LoopError::AccumulatorTypeMismatch { .. } => {
                        Self::type_error(accumulator, iterations, execution_mode)
                    }
                    _ => Self::type_error(accumulator, iterations, execution_mode),
                }
            }
            LoopResult::EnvironmentFault(_) => {
                Self::environment_fault(Value::Number(0.0), 0, execution_mode)
            }
            LoopResult::ControlFlow(control_flow) => {
                match control_flow {
                    ControlFlowResult::Break {
                        accumulator,
                        iterations_completed,
                    } => Self::break_result(accumulator, iterations_completed, execution_mode),
                    ControlFlowResult::Continue {
                        iterations_completed,
                    } => {
                        // Continue doesn't have an accumulator, so we use a default
                        Self::success(Value::Number(0.0), iterations_completed, execution_mode)
                    }
                }
            }
        }
    }
}

impl LoopResult {
    /// Create a success result
    pub fn success(accumulator: Value, iterations_completed: u32) -> Self {
        Self::Success {
            accumulator,
            iterations_completed,
        }
    }

    /// Create a partial result
    pub fn partial(partial: PartialResult) -> Self {
        Self::Partial(partial)
    }

    /// Create an error result
    pub fn error(error: LoopError) -> Self {
        Self::Error(error)
    }

    /// Create an environment fault result
    pub fn environment_fault(fault: EnvironmentFault) -> Self {
        Self::EnvironmentFault(fault)
    }

    /// Create a break result
    pub fn break_result(accumulator: Value, iterations_completed: u32) -> Self {
        Self::ControlFlow(ControlFlowResult::Break {
            accumulator,
            iterations_completed,
        })
    }

    /// Create a continue result
    pub fn continue_result(iterations_completed: u32) -> Self {
        Self::ControlFlow(ControlFlowResult::Continue {
            iterations_completed,
        })
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if the result is partial
    pub fn is_partial(&self) -> bool {
        matches!(self, Self::Partial(_))
    }

    /// Check if the result is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Check if the result is an environment fault
    pub fn is_environment_fault(&self) -> bool {
        matches!(self, Self::EnvironmentFault(_))
    }

    /// Check if the result is a control flow result
    pub fn is_control_flow(&self) -> bool {
        matches!(self, Self::ControlFlow(_))
    }

    /// Check if the result is a break
    pub fn is_break(&self) -> bool {
        matches!(self, Self::ControlFlow(ControlFlowResult::Break { .. }))
    }

    /// Check if the result is a continue
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::ControlFlow(ControlFlowResult::Continue { .. }))
    }

    /// Get the final accumulator value (if available)
    pub fn get_accumulator(&self) -> Option<&Value> {
        match self {
            Self::Success { accumulator, .. } => Some(accumulator),
            Self::Partial(partial) => Some(&partial.accumulator),
            Self::ControlFlow(ControlFlowResult::Break { accumulator, .. }) => Some(accumulator),
            Self::Error(_)
            | Self::EnvironmentFault(_)
            | Self::ControlFlow(ControlFlowResult::Continue { .. }) => None,
        }
    }

    /// Get the number of completed iterations
    pub fn get_iterations_completed(&self) -> u32 {
        match self {
            Self::Success {
                iterations_completed,
                ..
            } => *iterations_completed,
            Self::Partial(partial) => partial.iterations_completed,
            Self::ControlFlow(ControlFlowResult::Break {
                iterations_completed,
                ..
            }) => *iterations_completed,
            Self::ControlFlow(ControlFlowResult::Continue {
                iterations_completed,
                ..
            }) => *iterations_completed,
            Self::Error(_) | Self::EnvironmentFault(_) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{Value, ValueType};

    #[test]
    fn test_loop_error_display() {
        let error = LoopError::IterationLimitExceeded {
            limit: 100,
            completed: 100,
        };
        assert!(error
            .to_string()
            .contains("Iteration limit of 100 exceeded"));

        let error = LoopError::BudgetTimeoutExceeded {
            budget: 1000,
            consumed: 1000,
            iterations_completed: 50,
        };
        assert!(error
            .to_string()
            .contains("Budget timeout of 1000 exceeded"));

        let error = LoopError::AccumulatorTypeMismatch {
            expected: ValueType::Number,
            actual: ValueType::String,
            iteration: 5,
            accumulator_name: "counter".to_string(),
        };
        assert!(error
            .to_string()
            .contains("Accumulator type mismatch in 'counter'"));
    }

    #[test]
    fn test_environment_fault_display() {
        let fault = EnvironmentFault::WallClockKill {
            elapsed_ms: 5000,
            limit_ms: 3000,
        };
        assert!(fault
            .to_string()
            .contains("Wall-clock kill switch triggered"));
    }

    #[test]
    fn test_partial_result() {
        let partial = PartialResult::new(
            Value::Number(42.0),
            10,
            TerminationReason::IterationLimitReached,
        );

        assert_eq!(partial.accumulator, Value::Number(42.0));
        assert_eq!(partial.iterations_completed, 10);
        assert_eq!(
            partial.termination_reason,
            TerminationReason::IterationLimitReached
        );
        assert!(partial.error_info.is_none());

        let partial_with_error = PartialResult::with_error(
            Value::Boolean(true),
            5,
            TerminationReason::LoopBodyError,
            "Division by zero".to_string(),
        );

        assert_eq!(
            partial_with_error.error_info,
            Some("Division by zero".to_string())
        );
    }

    #[test]
    fn test_termination_reason_display() {
        assert_eq!(
            TerminationReason::IterationLimitReached.to_string(),
            "iteration limit reached"
        );
        assert_eq!(
            TerminationReason::BudgetTimeoutReached.to_string(),
            "budget timeout reached"
        );
        assert_eq!(
            TerminationReason::BreakExecuted.to_string(),
            "break executed"
        );
        assert_eq!(
            TerminationReason::ConditionFalse.to_string(),
            "condition became false"
        );
        assert_eq!(
            TerminationReason::LoopBodyError.to_string(),
            "loop body error"
        );
        assert_eq!(
            TerminationReason::EnvironmentFault.to_string(),
            "environment fault"
        );
    }

    #[test]
    fn test_loop_result() {
        // Test success result
        let success = LoopResult::success(Value::Number(100.0), 50);
        assert!(success.is_success());
        assert!(!success.is_partial());
        assert!(!success.is_error());
        assert!(!success.is_environment_fault());
        assert!(!success.is_control_flow());
        assert_eq!(success.get_accumulator(), Some(&Value::Number(100.0)));
        assert_eq!(success.get_iterations_completed(), 50);

        // Test partial result
        let partial = PartialResult::new(
            Value::String("partial".to_string()),
            25,
            TerminationReason::BudgetTimeoutReached,
        );
        let partial_result = LoopResult::partial(partial);
        assert!(!partial_result.is_success());
        assert!(partial_result.is_partial());
        assert!(!partial_result.is_error());
        assert!(!partial_result.is_environment_fault());
        assert!(!partial_result.is_control_flow());
        assert_eq!(
            partial_result.get_accumulator(),
            Some(&Value::String("partial".to_string()))
        );
        assert_eq!(partial_result.get_iterations_completed(), 25);

        // Test error result
        let error = LoopError::AccumulatorTypeMismatch {
            expected: ValueType::Number,
            actual: ValueType::String,
            iteration: 10,
            accumulator_name: "test".to_string(),
        };
        let error_result = LoopResult::error(error);
        assert!(!error_result.is_success());
        assert!(!error_result.is_partial());
        assert!(error_result.is_error());
        assert!(!error_result.is_environment_fault());
        assert!(!error_result.is_control_flow());
        assert_eq!(error_result.get_accumulator(), None);
        assert_eq!(error_result.get_iterations_completed(), 0);

        // Test environment fault result
        let fault = EnvironmentFault::WallClockKill {
            elapsed_ms: 5000,
            limit_ms: 3000,
        };
        let fault_result = LoopResult::environment_fault(fault);
        assert!(!fault_result.is_success());
        assert!(!fault_result.is_partial());
        assert!(!fault_result.is_error());
        assert!(fault_result.is_environment_fault());
        assert!(!fault_result.is_control_flow());
        assert_eq!(fault_result.get_accumulator(), None);
        assert_eq!(fault_result.get_iterations_completed(), 0);

        // Test break result
        let break_result = LoopResult::break_result(Value::Boolean(true), 15);
        assert!(!break_result.is_success());
        assert!(!break_result.is_partial());
        assert!(!break_result.is_error());
        assert!(!break_result.is_environment_fault());
        assert!(break_result.is_control_flow());
        assert!(break_result.is_break());
        assert!(!break_result.is_continue());
        assert_eq!(break_result.get_accumulator(), Some(&Value::Boolean(true)));
        assert_eq!(break_result.get_iterations_completed(), 15);

        // Test continue result
        let continue_result = LoopResult::continue_result(10);
        assert!(!continue_result.is_success());
        assert!(!continue_result.is_partial());
        assert!(!continue_result.is_error());
        assert!(!continue_result.is_environment_fault());
        assert!(continue_result.is_control_flow());
        assert!(!continue_result.is_break());
        assert!(continue_result.is_continue());
        assert_eq!(continue_result.get_accumulator(), None); // Continue doesn't return accumulator
        assert_eq!(continue_result.get_iterations_completed(), 10);
    }

    #[test]
    fn test_error_codes() {
        // Test LoopError codes
        let error = LoopError::IterationLimitExceeded {
            limit: 100,
            completed: 100,
        };
        assert_eq!(error.error_code(), "LE001");

        let error = LoopError::BudgetTimeoutExceeded {
            budget: 1000,
            consumed: 1000,
            iterations_completed: 50,
        };
        assert_eq!(error.error_code(), "LE002");

        let error = LoopError::AccumulatorTypeMismatch {
            expected: ValueType::Number,
            actual: ValueType::String,
            iteration: 5,
            accumulator_name: "test".to_string(),
        };
        assert_eq!(error.error_code(), "LE003");

        let error = LoopError::UnorderedCollectionRejected {
            collection_type: "HashMap".to_string(),
            reason: "Non-deterministic iteration order".to_string(),
        };
        assert_eq!(error.error_code(), "LE004");

        let error = LoopError::LoopBodyError {
            iteration: 10,
            error: "Division by zero".to_string(),
        };
        assert_eq!(error.error_code(), "LE005");

        let error = LoopError::BreakSignal {
            iteration: 5,
            accumulator: Value::Number(42.0),
        };
        assert_eq!(error.error_code(), "LE006");

        let error = LoopError::ContinueSignal { iteration: 3 };
        assert_eq!(error.error_code(), "LE007");

        // Test EnvironmentFault codes
        let fault = EnvironmentFault::WallClockKill {
            elapsed_ms: 5000,
            limit_ms: 3000,
        };
        assert_eq!(fault.error_code(), "EF001");
    }

    #[test]
    fn test_error_recoverability() {
        // Recoverable errors
        let error = LoopError::IterationLimitExceeded {
            limit: 100,
            completed: 100,
        };
        assert!(error.is_recoverable());
        assert!(error.supports_partial_results());

        let error = LoopError::BudgetTimeoutExceeded {
            budget: 1000,
            consumed: 1000,
            iterations_completed: 50,
        };
        assert!(error.is_recoverable());
        assert!(error.supports_partial_results());

        let error = LoopError::LoopBodyError {
            iteration: 10,
            error: "Test error".to_string(),
        };
        assert!(error.is_recoverable());
        assert!(error.supports_partial_results());

        // Non-recoverable errors
        let error = LoopError::AccumulatorTypeMismatch {
            expected: ValueType::Number,
            actual: ValueType::String,
            iteration: 5,
            accumulator_name: "test".to_string(),
        };
        assert!(!error.is_recoverable());
        assert!(error.supports_partial_results()); // PRE-COMMIT

        let error = LoopError::UnorderedCollectionRejected {
            collection_type: "HashMap".to_string(),
            reason: "Non-deterministic".to_string(),
        };
        assert!(!error.is_recoverable());
        assert!(!error.supports_partial_results());

        // Control flow (not errors)
        let error = LoopError::BreakSignal {
            iteration: 5,
            accumulator: Value::Number(42.0),
        };
        assert!(!error.is_recoverable()); // Control flow, not error
        assert!(error.supports_partial_results()); // Break value

        let error = LoopError::ContinueSignal { iteration: 3 };
        assert!(!error.is_recoverable()); // Control flow, not error
        assert!(!error.supports_partial_results());

        // Environment faults
        let fault = EnvironmentFault::WallClockKill {
            elapsed_ms: 5000,
            limit_ms: 3000,
        };
        assert!(!fault.is_recoverable());
        assert!(fault.supports_partial_results());
    }

    #[test]
    fn test_error_recovery_policy() {
        // Default policy
        let policy = ErrorRecoveryPolicy::default();
        assert_eq!(policy, ErrorRecoveryPolicy::FailFast);
        assert!(!policy.allows_retries());
        assert_eq!(policy.max_retries(), 0);

        // Return partial policy
        let policy = ErrorRecoveryPolicy::ReturnPartial;
        assert!(!policy.allows_retries());
        assert_eq!(policy.max_retries(), 0);

        // Retry bounded policy
        let policy = ErrorRecoveryPolicy::retry_bounded(2).unwrap();
        assert!(policy.allows_retries());
        assert_eq!(policy.max_retries(), 2);

        // Constitutional limits
        assert!(ErrorRecoveryPolicy::retry_bounded(0).is_err());
        assert!(ErrorRecoveryPolicy::retry_bounded(4).is_err());
        assert!(ErrorRecoveryPolicy::retry_bounded(3).is_ok());

        // Custom policy (removed for simplicity)
        // let policy = ErrorRecoveryPolicy::Custom(None);
        // assert!(policy.allows_retries());
        // assert_eq!(policy.max_retries(), 3); // Constitutional maximum
    }

    #[test]
    fn test_recovery_action() {
        let action = RecoveryAction::Fail;
        assert_eq!(action, RecoveryAction::Fail);

        let action = RecoveryAction::ReturnPartial;
        assert_eq!(action, RecoveryAction::ReturnPartial);

        let action = RecoveryAction::Retry {
            new_limit: Some(200),
            new_budget: Some(2000),
        };
        if let RecoveryAction::Retry {
            new_limit,
            new_budget,
        } = action
        {
            assert_eq!(new_limit, Some(200));
            assert_eq!(new_budget, Some(2000));
        } else {
            panic!("Expected Retry action");
        }
    }
}
