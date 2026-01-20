//! Loop State Management - Constitutional Alignment (Phase 0.5)
//!
//! This module implements the loop state management types following the
//! accumulator pattern: immutable context + mutable accumulator.
//!
//! # Constitutional Guarantees
//!
//! - POST-INCREMENT iteration counting (completed_iterations)
//! - Exactness guarantee (never exceed limits)
//! - Deterministic budget measurement
//! - Type safety validation for accumulators

use crate::bcib::{LoopID, LoopConfig, Value, ValueType, BudgetMeasurement};
use crate::error::{Result, SemanticCLIError, ErrorCode};
use serde::{Deserialize, Serialize};

/// Immutable loop context (Constitutional Alignment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopContext {
    /// Unique loop identifier
    pub loop_id: LoopID,
    /// Maximum iterations allowed (Constitutional: never exceeded)
    pub iteration_limit: u32,
    /// Budget timeout in logical units (Constitutional: deterministic)
    pub budget_timeout: u64,
    /// Budget measurement method
    pub budget_measurement: BudgetMeasurement,
    /// Expected accumulator type for validation
    pub accumulator_type: ValueType,
    /// Loop body reference (Phase 1: string reference)
    pub loop_body: String,
}

impl LoopContext {
    /// Create a new loop context
    pub fn new(
        loop_id: LoopID,
        config: &LoopConfig,
        loop_body: String,
    ) -> Self {
        Self {
            loop_id,
            iteration_limit: config.iteration_limit,
            budget_timeout: config.budget_timeout,
            budget_measurement: config.budget_measurement.clone(),
            accumulator_type: config.accumulator_type,
            loop_body,
        }
    }

    /// Validate this loop context
    pub fn validate(&self) -> Result<()> {
        self.loop_id.validate()?;

        if self.iteration_limit == 0 {
            return Err(SemanticCLIError::validation_error(
                "Iteration limit must be greater than 0",
                "Provide a positive iteration limit",
                ErrorCode::E300,
            ));
        }

        if self.iteration_limit > 10_000 {
            return Err(SemanticCLIError::validation_error(
                "Iteration limit exceeds constitutional maximum of 10,000",
                "Use a limit within constitutional bounds",
                ErrorCode::E300,
            ));
        }

        if self.budget_timeout == 0 {
            return Err(SemanticCLIError::validation_error(
                "Budget timeout must be greater than 0",
                "Provide a positive budget timeout",
                ErrorCode::E300,
            ));
        }

        if self.loop_body.is_empty() {
            return Err(SemanticCLIError::validation_error(
                "Loop body cannot be empty",
                "Provide a valid loop body reference",
                ErrorCode::E300,
            ));
        }

        Ok(())
    }
}

/// Mutable loop execution state (Constitutional Alignment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopState {
    /// Immutable context
    pub context: LoopContext,
    /// Completed iterations counter (Constitutional: POST-INCREMENT)
    pub completed_iterations: u32,
    /// Budget consumed so far (Constitutional: deterministic)
    pub budget_consumed: u64,
    /// Current accumulator value
    pub accumulator: Value,
}

impl LoopState {
    /// Create a new loop state
    pub fn new(context: LoopContext, initial_accumulator: Value) -> Result<Self> {
        // Validate accumulator type matches context
        if initial_accumulator.value_type() != context.accumulator_type {
            return Err(SemanticCLIError::validation_error(
                format!(
                    "Initial accumulator type {:?} does not match expected type {:?}",
                    initial_accumulator.value_type(),
                    context.accumulator_type
                ),
                "Provide accumulator with correct type",
                ErrorCode::E300,
            ));
        }

        Ok(Self {
            context,
            completed_iterations: 0, // Constitutional: start at 0
            budget_consumed: 0,      // Constitutional: start at 0
            accumulator: initial_accumulator,
        })
    }

    /// Check if iteration limit would be exceeded (LOCKED: PRE-CHECK before iteration)
    pub fn would_exceed_iteration_limit(&self) -> bool {
        self.completed_iterations >= self.context.iteration_limit
    }

    /// Check if budget timeout would be exceeded (LOCKED: deterministic, every iteration)
    pub fn would_exceed_budget_timeout(&self, additional_budget: u64) -> bool {
        self.budget_consumed + additional_budget >= self.context.budget_timeout
    }

    /// Increment completed iterations (LOCKED: POST-INCREMENT after successful iteration)
    pub fn increment_completed_iterations(&mut self) {
        self.completed_iterations += 1;
    }

    /// Add to budget consumed (LOCKED: deterministic tracking every iteration)
    pub fn add_budget_consumed(&mut self, budget: u64) {
        self.budget_consumed += budget;
    }

    /// Update accumulator with type validation
    pub fn update_accumulator(&mut self, new_value: Value) -> Result<()> {
        // Constitutional: Type safety validation
        if new_value.value_type() != self.context.accumulator_type {
            return Err(SemanticCLIError::validation_error(
                format!(
                    "Accumulator type changed from {:?} to {:?} during iteration {}",
                    self.context.accumulator_type,
                    new_value.value_type(),
                    self.completed_iterations
                ),
                "Maintain consistent accumulator type across iterations",
                ErrorCode::E300,
            ));
        }

        self.accumulator = new_value;
        Ok(())
    }

    /// Get the current accumulator value (for partial results)
    pub fn get_accumulator(&self) -> &Value {
        &self.accumulator
    }

    /// Validate the current state
    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;

        // Validate accumulator type consistency
        if self.accumulator.value_type() != self.context.accumulator_type {
            return Err(SemanticCLIError::validation_error(
                "Accumulator type does not match context type",
                "Ensure type consistency",
                ErrorCode::E300,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{LoopID, Value, ValueType, BudgetMeasurement};

    fn create_test_context() -> LoopContext {
        LoopContext {
            loop_id: LoopID::new("test-loop".to_string()),
            iteration_limit: 100,
            budget_timeout: 1000,
            budget_measurement: BudgetMeasurement::IterationCount,
            accumulator_type: ValueType::Number,
            loop_body: "test-body".to_string(),
        }
    }

    #[test]
    fn test_loop_context_validation() {
        let context = create_test_context();
        assert!(context.validate().is_ok());

        // Test invalid iteration limit
        let mut invalid_context = context.clone();
        invalid_context.iteration_limit = 0;
        assert!(invalid_context.validate().is_err());

        // Test constitutional maximum exceeded
        let mut invalid_context = context.clone();
        invalid_context.iteration_limit = 20_000;
        assert!(invalid_context.validate().is_err());

        // Test invalid budget timeout
        let mut invalid_context = context.clone();
        invalid_context.budget_timeout = 0;
        assert!(invalid_context.validate().is_err());

        // Test empty loop body
        let mut invalid_context = context.clone();
        invalid_context.loop_body = "".to_string();
        assert!(invalid_context.validate().is_err());
    }

    #[test]
    fn test_loop_state_creation() {
        let context = create_test_context();
        let initial_accumulator = Value::Number(0.0);

        let state = LoopState::new(context, initial_accumulator).unwrap();
        assert_eq!(state.completed_iterations, 0);
        assert_eq!(state.budget_consumed, 0);
        assert_eq!(state.accumulator, Value::Number(0.0));
    }

    #[test]
    fn test_loop_state_type_validation() {
        let context = create_test_context();
        let wrong_type_accumulator = Value::String("wrong".to_string());

        // Should fail with wrong type
        let result = LoopState::new(context, wrong_type_accumulator);
        assert!(result.is_err());
    }

    #[test]
    fn test_iteration_limit_checking() {
        let context = create_test_context();
        let mut state = LoopState::new(context, Value::Number(0.0)).unwrap();

        // Should not exceed initially
        assert!(!state.would_exceed_iteration_limit());

        // Increment to limit
        for _ in 0..100 {
            state.increment_completed_iterations();
        }

        // Should now exceed
        assert!(state.would_exceed_iteration_limit());
    }

    #[test]
    fn test_budget_timeout_checking() {
        let context = create_test_context();
        let state = LoopState::new(context, Value::Number(0.0)).unwrap();

        // Should not exceed with small budget
        assert!(!state.would_exceed_budget_timeout(100));

        // Should exceed with large budget
        assert!(state.would_exceed_budget_timeout(2000));
    }

    #[test]
    fn test_accumulator_update_type_safety() {
        let context = create_test_context();
        let mut state = LoopState::new(context, Value::Number(0.0)).unwrap();

        // Valid update
        assert!(state.update_accumulator(Value::Number(42.0)).is_ok());
        assert_eq!(state.accumulator, Value::Number(42.0));

        // Invalid type update
        assert!(state.update_accumulator(Value::String("invalid".to_string())).is_err());
    }
}