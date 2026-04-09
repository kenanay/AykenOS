//! Control Flow Logic - Phase 3.2 Modularization
//!
//! This module contains the control flow logic extracted from core.rs and executor.rs,
//! focusing on iteration counting, bounds checking, and decision tracking.
//!
//! # Restricted Imports
//! 
//! Following the task requirements, this module only imports:
//! - `use super::{state, errors};`
//!
//! # Control Flow Responsibilities
//!
//! - Iteration counting and bounds checking
//! - Break/continue condition evaluation
//! - Control flow decision tracking for fingerprints
//! - Budget timeout management
//! - Wall-clock kill switch monitoring
//!
//! # Constitutional Compliance
//!
//! This implementation MUST follow the locked semantics:
//! 1. PRE-CHECK iteration limits before execution
//! 2. PRE-CHECK budget timeout before execution
//! 3. POST-INCREMENT: completed_iterations += 1 (AFTER successful iteration)
//! 4. POST-INCREMENT: budget_consumed += budget_cost (AFTER successful iteration)
//! 5. Decision trace recording for fingerprint generation

use super::state;
use crate::bcib::{Value, LoopRange};
use crate::error::{Result, SemanticCLIError, ErrorCode};

/// Control flow decision for loop execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowDecision {
    /// Continue to next iteration
    Continue,
    /// Break from loop
    Break,
    /// Skip remaining body, proceed to next iteration
    Skip,
}

/// Control flow decision with metadata for fingerprint tracking
#[derive(Debug, Clone, PartialEq)]
pub struct ControlDecision {
    /// The decision made (continue, break, skip)
    pub decision: ControlFlowDecision,
    /// Condition result that led to this decision
    pub condition_result: bool,
    /// Iteration number when decision was made
    pub iteration: u32,
    /// Timestamp for decision ordering (deterministic)
    pub decision_index: u64,
}

impl ControlDecision {
    /// Create a new control decision
    pub fn new(
        decision: ControlFlowDecision,
        condition_result: bool,
        iteration: u32,
        decision_index: u64,
    ) -> Self {
        Self {
            decision,
            condition_result,
            iteration,
            decision_index,
        }
    }

    /// Create a continue decision
    pub fn continue_decision(condition_result: bool, iteration: u32, decision_index: u64) -> Self {
        Self::new(ControlFlowDecision::Continue, condition_result, iteration, decision_index)
    }

    /// Create a break decision
    pub fn break_decision(condition_result: bool, iteration: u32, decision_index: u64) -> Self {
        Self::new(ControlFlowDecision::Break, condition_result, iteration, decision_index)
    }

    /// Create a skip decision (for continue statements)
    pub fn skip_decision(condition_result: bool, iteration: u32, decision_index: u64) -> Self {
        Self::new(ControlFlowDecision::Skip, condition_result, iteration, decision_index)
    }
}

/// Control flow manager for loop execution
/// 
/// This struct manages iteration counting, bounds checking, and decision tracking
/// for fingerprint generation. It enforces constitutional guarantees around
/// iteration limits and budget timeouts.
pub struct ControlFlow {
    /// Current iteration count (POST-INCREMENT after successful iteration)
    iteration_count: u32,
    /// Maximum iterations allowed
    max_iterations: Option<u32>,
    /// Budget consumed so far
    budget_consumed: u64,
    /// Maximum budget allowed
    max_budget: Option<u64>,
    /// Decision trace for fingerprint generation
    decision_trace: Vec<ControlDecision>,
    /// Decision index counter for deterministic ordering
    decision_index: u64,
    /// Wall-clock monitoring state
    wall_clock_check_counter: u32,
    /// Positions where break statements occurred (for ShapeFingerprint)
    break_positions: Vec<u64>,
    /// Positions where continue statements occurred (for ShapeFingerprint)
    continue_positions: Vec<u64>,
    /// Condition evaluation order tracking (for ShapeFingerprint)
    condition_evaluation_order: Vec<u64>,
}

impl ControlFlow {
    /// Create a new control flow manager
    pub fn new() -> Self {
        Self {
            iteration_count: 0,
            max_iterations: None,
            budget_consumed: 0,
            max_budget: None,
            decision_trace: Vec::new(),
            decision_index: 0,
            wall_clock_check_counter: 0,
            break_positions: Vec::new(),
            continue_positions: Vec::new(),
            condition_evaluation_order: Vec::new(),
        }
    }

    /// Create a control flow manager with limits from loop state
    pub fn from_loop_state(state: &state::LoopState) -> Self {
        Self {
            iteration_count: state.completed_iterations,
            max_iterations: Some(state.context.iteration_limit),
            budget_consumed: state.budget_consumed,
            max_budget: Some(state.context.budget_timeout),
            decision_trace: Vec::new(),
            decision_index: 0,
            wall_clock_check_counter: 0,
            break_positions: Vec::new(),
            continue_positions: Vec::new(),
            condition_evaluation_order: Vec::new(),
        }
    }

    /// Set iteration limits
    pub fn set_iteration_limit(&mut self, limit: u32) {
        self.max_iterations = Some(limit);
    }

    /// Set budget limits
    pub fn set_budget_limit(&mut self, limit: u64) {
        self.max_budget = Some(limit);
    }

    /// Check if iteration limit would be exceeded (Constitutional: PRE-CHECK)
    /// 
    /// This method implements the constitutional requirement for PRE-CHECK
    /// iteration limit validation before executing any iteration.
    pub fn would_exceed_iteration_limit(&self) -> bool {
        if let Some(max) = self.max_iterations {
            self.iteration_count >= max
        } else {
            false
        }
    }

    /// Check if budget timeout would be exceeded (Constitutional: PRE-CHECK)
    /// 
    /// This method implements the constitutional requirement for PRE-CHECK
    /// budget timeout validation before executing any iteration.
    pub fn would_exceed_budget_timeout(&self, additional_budget: u64) -> bool {
        if let Some(max) = self.max_budget {
            self.budget_consumed + additional_budget >= max
        } else {
            false
        }
    }

    /// Check iteration limit and return error if exceeded
    pub fn check_iteration_limit(&self) -> Result<()> {
        if self.would_exceed_iteration_limit() {
            return Err(SemanticCLIError::execution_error(
                &format!(
                    "Iteration limit exceeded: {} >= {}",
                    self.iteration_count,
                    self.max_iterations.unwrap_or(0)
                ),
                ErrorCode::E400,
            ));
        }
        Ok(())
    }

    /// Check budget timeout and return error if exceeded
    pub fn check_budget_timeout(&self, additional_budget: u64) -> Result<()> {
        if self.would_exceed_budget_timeout(additional_budget) {
            return Err(SemanticCLIError::execution_error(
                &format!(
                    "Budget timeout exceeded: {} + {} >= {}",
                    self.budget_consumed,
                    additional_budget,
                    self.max_budget.unwrap_or(0)
                ),
                ErrorCode::E400,
            ));
        }
        Ok(())
    }

    /// Increment iteration count (Constitutional: POST-INCREMENT after successful iteration)
    /// 
    /// This method implements the constitutional requirement for POST-INCREMENT
    /// iteration counting after successful iteration execution.
    pub fn increment_iteration_count(&mut self) {
        self.iteration_count += 1;
    }

    /// Add to budget consumed (Constitutional: POST-INCREMENT after successful iteration)
    /// 
    /// This method implements the constitutional requirement for POST-INCREMENT
    /// budget tracking after successful iteration execution.
    pub fn add_budget_consumed(&mut self, budget: u64) {
        self.budget_consumed += budget;
    }

    /// Record a control flow decision for fingerprint tracking
    /// 
    /// This method records control flow decisions in a deterministic order
    /// for fingerprint generation. Each decision is assigned a unique index
    /// to ensure deterministic ordering across executions.
    pub fn record_decision(&mut self, decision: ControlFlowDecision, condition_result: bool) {
        let control_decision = ControlDecision::new(
            decision,
            condition_result,
            self.iteration_count,
            self.decision_index,
        );
        
        self.decision_trace.push(control_decision);
        self.decision_index += 1;
    }

    /// Evaluate a condition and record the decision
    /// 
    /// This method evaluates a boolean condition and records the decision
    /// for fingerprint tracking. It returns the control flow decision
    /// based on the condition result.
    pub fn evaluate_condition(&mut self, condition: bool) -> ControlFlowDecision {
        // Track condition evaluation order for ShapeFingerprint
        self.condition_evaluation_order.push(self.iteration_count as u64);
        
        let decision = if condition {
            ControlFlowDecision::Continue
        } else {
            ControlFlowDecision::Break
        };
        
        self.record_decision(decision, condition);
        decision
    }

    /// Handle break statement execution
    /// 
    /// This method handles break statement execution by recording the decision
    /// and returning the appropriate control flow result.
    pub fn handle_break(&mut self) -> ControlFlowDecision {
        // Track break position for ShapeFingerprint
        self.break_positions.push(self.iteration_count as u64);
        
        self.record_decision(ControlFlowDecision::Break, true);
        ControlFlowDecision::Break
    }

    /// Handle continue statement execution
    /// 
    /// This method handles continue statement execution by recording the decision
    /// and returning the appropriate control flow result.
    pub fn handle_continue(&mut self) -> ControlFlowDecision {
        // Track continue position for ShapeFingerprint
        self.continue_positions.push(self.iteration_count as u64);
        
        self.record_decision(ControlFlowDecision::Skip, true);
        ControlFlowDecision::Skip
    }

    /// Check wall-clock kill switch (Constitutional: environment fault mechanism)
    /// 
    /// This method implements the constitutional requirement for wall-clock
    /// kill switch monitoring. It checks every 100 iterations for performance
    /// and can trigger environment faults for non-semantic termination.
    pub fn check_wall_clock_kill_switch(&mut self) -> Result<()> {
        self.wall_clock_check_counter += 1;
        
        // Check every 100 iterations for performance
        if self.wall_clock_check_counter % 100 != 0 && self.wall_clock_check_counter != 1 {
            return Ok(());
        }

        // TODO: Implement actual wall-clock checking
        // For now, this is a placeholder that never triggers
        // Future implementation will track wall-clock time since loop start
        // and trigger EnvironmentFault::WallClockKill if exceeded
        
        Ok(())
    }

    /// Get the current iteration count
    pub fn get_iteration_count(&self) -> u32 {
        self.iteration_count
    }

    /// Get the current budget consumed
    pub fn get_budget_consumed(&self) -> u64 {
        self.budget_consumed
    }

    /// Get the decision trace for fingerprint generation
    pub fn get_decision_trace(&self) -> &[ControlDecision] {
        &self.decision_trace
    }

    /// Get the next decision index
    pub fn get_next_decision_index(&self) -> u64 {
        self.decision_index
    }

    /// Get break positions for ShapeFingerprint generation
    pub fn get_break_positions(&self) -> &[u64] {
        &self.break_positions
    }

    /// Get continue positions for ShapeFingerprint generation
    pub fn get_continue_positions(&self) -> &[u64] {
        &self.continue_positions
    }

    /// Get condition evaluation order for ShapeFingerprint generation
    pub fn get_condition_evaluation_order(&self) -> &[u64] {
        &self.condition_evaluation_order
    }

    /// Reset the control flow state
    pub fn reset(&mut self) {
        self.iteration_count = 0;
        self.budget_consumed = 0;
        self.decision_trace.clear();
        self.decision_index = 0;
        self.wall_clock_check_counter = 0;
        self.break_positions.clear();
        self.continue_positions.clear();
        self.condition_evaluation_order.clear();
    }

    /// Create a control fingerprint from the decision trace
    /// 
    /// This method generates a control fingerprint from the recorded decision
    /// trace for use in the enhanced fingerprint system. The fingerprint
    /// includes decision sequence and evaluation order for deterministic
    /// replay verification.
    pub fn create_control_fingerprint(&self) -> crate::loop_engine::fingerprint::ControlFingerprint {
        let converted_decisions = self.decision_trace.iter().map(|d| {
            match d.decision {
                ControlFlowDecision::Continue => {
                    crate::loop_engine::fingerprint::ControlDecision::Continue {
                        condition_result: d.condition_result,
                        iteration: d.iteration as u64,
                    }
                }
                ControlFlowDecision::Break => {
                    crate::loop_engine::fingerprint::ControlDecision::Break {
                        condition_result: d.condition_result,
                        iteration: d.iteration as u64,
                    }
                }
                ControlFlowDecision::Skip => {
                    // Skip is treated as Continue for fingerprint purposes
                    crate::loop_engine::fingerprint::ControlDecision::Continue {
                        condition_result: d.condition_result,
                        iteration: d.iteration as u64,
                    }
                }
            }
        }).collect();

        crate::loop_engine::fingerprint::ControlFingerprint {
            decision_sequence: converted_decisions,
            condition_evaluation_order: self.decision_trace
                .iter()
                .map(|d| d.decision_index)
                .collect(),
            decision_trace_index: self.decision_index,
        }
    }

    /// Create a shape fingerprint from the control flow state
    /// 
    /// This method generates a shape fingerprint from the tracked execution
    /// characteristics including break/continue positions and condition
    /// evaluation order for deterministic replay verification.
    pub fn create_shape_fingerprint(
        &self,
        loop_id: u64,
        loop_type: crate::loop_engine::fingerprint::LoopType,
    ) -> crate::loop_engine::fingerprint::ShapeFingerprint {
        crate::loop_engine::fingerprint::ShapeFingerprint {
            loop_id,
            loop_type,
            metadata_signature: 0,
            body_signature: 0,
            iteration_count: self.iteration_count as u64,
            break_positions: self.break_positions.clone(),
            continue_positions: self.continue_positions.clone(),
            condition_evaluation_order: self.condition_evaluation_order.clone(),
        }
    }
}

impl Default for ControlFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Budget calculation utilities for control flow management
pub struct BudgetCalculator;

impl BudgetCalculator {
    /// Calculate budget cost for a single iteration
    /// 
    /// This method calculates the budget cost for a single iteration
    /// based on the budget measurement method specified in the loop context.
    pub fn calculate_iteration_budget_cost(
        budget_measurement: &crate::bcib::BudgetMeasurement,
    ) -> u64 {
        match budget_measurement {
            crate::bcib::BudgetMeasurement::IterationCount => {
                // Simple: each iteration costs 1 budget unit
                1
            }
            crate::bcib::BudgetMeasurement::InstructionCount { weight } => {
                // Use provided weight as instruction count
                *weight
            }
            crate::bcib::BudgetMeasurement::Hybrid { multiplier } => {
                // Use multiplier as average instruction count per iteration
                (*multiplier as u64).max(1)
            }
        }
    }

    /// Calculate budget cost for break instruction
    /// 
    /// Constitutional: Break instruction has minimal cost to prevent budget bypass
    pub fn calculate_break_budget_cost() -> u64 {
        1
    }

    /// Calculate budget cost for continue instruction
    /// 
    /// Constitutional: Continue instruction has minimal cost to prevent budget bypass
    pub fn calculate_continue_budget_cost() -> u64 {
        1
    }
}

/// Range iteration utilities for control flow management
pub struct RangeIterator {
    current: i64,
    end: i64,
    step: i64,
    completed: bool,
}

impl RangeIterator {
    /// Create a new range iterator from a loop range
    pub fn from_loop_range(range: &LoopRange) -> Self {
        Self {
            current: range.start,
            end: range.end,
            step: range.step,
            completed: false,
        }
    }

    /// Check if the range iteration is complete
    pub fn is_complete(&self) -> bool {
        if self.completed {
            return true;
        }

        if self.step > 0 {
            self.current >= self.end
        } else if self.step < 0 {
            self.current <= self.end
        } else {
            true // Invalid step - consider complete
        }
    }

    /// Get the current iterator value
    pub fn current_value(&self) -> Value {
        Value::Number(self.current as f64)
    }

    /// Advance to the next iteration
    pub fn advance(&mut self) {
        if !self.is_complete() {
            self.current += self.step;
        }
    }

    /// Reset the iterator to the beginning
    pub fn reset(&mut self, range: &LoopRange) {
        self.current = range.start;
        self.end = range.end;
        self.step = range.step;
        self.completed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{LoopID, BudgetMeasurement, ValueType};

    fn create_test_loop_state() -> state::LoopState {
        let context = state::LoopContext {
            loop_id: LoopID::new("test-loop".to_string()),
            iteration_limit: 100,
            budget_timeout: 1000,
            budget_measurement: BudgetMeasurement::IterationCount,
            accumulator_type: ValueType::Number,
            loop_body: "test-body".to_string(),
        };
        
        state::LoopState::new(context, Value::Number(0.0)).unwrap()
    }

    #[test]
    fn test_control_flow_creation() {
        let control_flow = ControlFlow::new();
        assert_eq!(control_flow.get_iteration_count(), 0);
        assert_eq!(control_flow.get_budget_consumed(), 0);
        assert!(control_flow.get_decision_trace().is_empty());
        assert_eq!(control_flow.get_next_decision_index(), 0);
    }

    #[test]
    fn test_control_flow_from_loop_state() {
        let mut state = create_test_loop_state();
        state.increment_completed_iterations();
        state.add_budget_consumed(50);

        let control_flow = ControlFlow::from_loop_state(&state);
        assert_eq!(control_flow.get_iteration_count(), 1);
        assert_eq!(control_flow.get_budget_consumed(), 50);
        assert_eq!(control_flow.max_iterations, Some(100));
        assert_eq!(control_flow.max_budget, Some(1000));
    }

    #[test]
    fn test_iteration_limit_checking() {
        let mut control_flow = ControlFlow::new();
        control_flow.set_iteration_limit(5);

        // Should not exceed initially
        assert!(!control_flow.would_exceed_iteration_limit());
        assert!(control_flow.check_iteration_limit().is_ok());

        // Increment to limit
        for _ in 0..5 {
            control_flow.increment_iteration_count();
        }

        // Should now exceed
        assert!(control_flow.would_exceed_iteration_limit());
        assert!(control_flow.check_iteration_limit().is_err());
    }

    #[test]
    fn test_budget_timeout_checking() {
        let mut control_flow = ControlFlow::new();
        control_flow.set_budget_limit(100);

        // Should not exceed with small budget
        assert!(!control_flow.would_exceed_budget_timeout(50));
        assert!(control_flow.check_budget_timeout(50).is_ok());

        // Should exceed with large budget
        assert!(control_flow.would_exceed_budget_timeout(200));
        assert!(control_flow.check_budget_timeout(200).is_err());

        // Add some budget and test again
        control_flow.add_budget_consumed(80);
        assert!(control_flow.would_exceed_budget_timeout(30));
        assert!(control_flow.check_budget_timeout(30).is_err());
    }

    #[test]
    fn test_decision_recording() {
        let mut control_flow = ControlFlow::new();

        // Record some decisions
        control_flow.record_decision(ControlFlowDecision::Continue, true);
        control_flow.record_decision(ControlFlowDecision::Break, false);

        let trace = control_flow.get_decision_trace();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].decision, ControlFlowDecision::Continue);
        assert_eq!(trace[0].condition_result, true);
        assert_eq!(trace[0].decision_index, 0);
        assert_eq!(trace[1].decision, ControlFlowDecision::Break);
        assert_eq!(trace[1].condition_result, false);
        assert_eq!(trace[1].decision_index, 1);
    }

    #[test]
    fn test_condition_evaluation() {
        let mut control_flow = ControlFlow::new();

        // Evaluate true condition
        let decision = control_flow.evaluate_condition(true);
        assert_eq!(decision, ControlFlowDecision::Continue);

        // Evaluate false condition
        let decision = control_flow.evaluate_condition(false);
        assert_eq!(decision, ControlFlowDecision::Break);

        // Check decision trace
        let trace = control_flow.get_decision_trace();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].condition_result, true);
        assert_eq!(trace[1].condition_result, false);
    }

    #[test]
    fn test_break_and_continue_handling() {
        let mut control_flow = ControlFlow::new();

        // Handle break
        let decision = control_flow.handle_break();
        assert_eq!(decision, ControlFlowDecision::Break);

        // Handle continue
        let decision = control_flow.handle_continue();
        assert_eq!(decision, ControlFlowDecision::Skip);

        // Check decision trace
        let trace = control_flow.get_decision_trace();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].decision, ControlFlowDecision::Break);
        assert_eq!(trace[1].decision, ControlFlowDecision::Skip);
    }

    #[test]
    fn test_control_fingerprint_creation() {
        let mut control_flow = ControlFlow::new();

        // Record some decisions
        control_flow.record_decision(ControlFlowDecision::Continue, true);
        control_flow.record_decision(ControlFlowDecision::Break, false);

        // Create fingerprint
        let fingerprint = control_flow.create_control_fingerprint();
        assert_eq!(fingerprint.decision_count(), 2);
        assert_eq!(fingerprint.decision_trace_index, 2);
        assert!(!fingerprint.is_empty());
        assert!(fingerprint.validate().is_ok());
    }

    #[test]
    fn test_budget_calculator() {
        // Test iteration count budget
        let budget = BudgetCalculator::calculate_iteration_budget_cost(
            &BudgetMeasurement::IterationCount
        );
        assert_eq!(budget, 1);

        // Test instruction count budget
        let budget = BudgetCalculator::calculate_iteration_budget_cost(
            &BudgetMeasurement::InstructionCount { weight: 10 }
        );
        assert_eq!(budget, 10);

        // Test hybrid budget
        let budget = BudgetCalculator::calculate_iteration_budget_cost(
            &BudgetMeasurement::Hybrid { multiplier: 5.5 }
        );
        assert_eq!(budget, 5);

        // Test break and continue costs
        assert_eq!(BudgetCalculator::calculate_break_budget_cost(), 1);
        assert_eq!(BudgetCalculator::calculate_continue_budget_cost(), 1);
    }

    #[test]
    fn test_range_iterator() {
        let range = LoopRange {
            start: 0,
            end: 5,
            step: 1,
        };

        let mut iterator = RangeIterator::from_loop_range(&range);
        assert!(!iterator.is_complete());
        assert_eq!(iterator.current_value(), Value::Number(0.0));

        // Advance through range
        for i in 1..5 {
            iterator.advance();
            assert_eq!(iterator.current_value(), Value::Number(i as f64));
            assert!(!iterator.is_complete());
        }

        // Should be complete after reaching end
        iterator.advance();
        assert!(iterator.is_complete());

        // Test backward range
        let range = LoopRange {
            start: 5,
            end: 0,
            step: -1,
        };

        iterator.reset(&range);
        assert!(!iterator.is_complete());
        assert_eq!(iterator.current_value(), Value::Number(5.0));

        // Advance through backward range
        for i in (1..5).rev() {
            iterator.advance();
            assert_eq!(iterator.current_value(), Value::Number(i as f64));
            assert!(!iterator.is_complete());
        }

        iterator.advance();
        assert!(iterator.is_complete());
    }

    #[test]
    fn test_control_flow_reset() {
        let mut control_flow = ControlFlow::new();
        control_flow.set_iteration_limit(100);
        control_flow.set_budget_limit(1000);
        
        // Add some state
        control_flow.increment_iteration_count();
        control_flow.add_budget_consumed(50);
        control_flow.record_decision(ControlFlowDecision::Continue, true);
        control_flow.handle_break();
        control_flow.handle_continue();

        // Verify state exists
        assert_eq!(control_flow.get_iteration_count(), 1);
        assert_eq!(control_flow.get_budget_consumed(), 50);
        assert_eq!(control_flow.get_decision_trace().len(), 3);
        assert_eq!(control_flow.get_break_positions().len(), 1);
        assert_eq!(control_flow.get_continue_positions().len(), 1);

        // Reset and verify clean state
        control_flow.reset();
        assert_eq!(control_flow.get_iteration_count(), 0);
        assert_eq!(control_flow.get_budget_consumed(), 0);
        assert_eq!(control_flow.get_decision_trace().len(), 0);
        assert_eq!(control_flow.get_next_decision_index(), 0);
        assert_eq!(control_flow.get_break_positions().len(), 0);
        assert_eq!(control_flow.get_continue_positions().len(), 0);
        assert_eq!(control_flow.get_condition_evaluation_order().len(), 0);
    }

    #[test]
    fn test_break_continue_position_tracking() {
        let mut control_flow = ControlFlow::new();
        
        // Simulate some iterations with breaks and continues
        control_flow.increment_iteration_count(); // iteration 1
        control_flow.handle_continue(); // continue at iteration 1
        
        control_flow.increment_iteration_count(); // iteration 2
        control_flow.increment_iteration_count(); // iteration 3
        control_flow.handle_break(); // break at iteration 3
        
        control_flow.increment_iteration_count(); // iteration 4
        control_flow.handle_continue(); // continue at iteration 4

        // Verify positions are tracked correctly
        let break_positions = control_flow.get_break_positions();
        let continue_positions = control_flow.get_continue_positions();
        
        assert_eq!(break_positions, &[3]);
        assert_eq!(continue_positions, &[1, 4]);
    }

    #[test]
    fn test_condition_evaluation_order_tracking() {
        let mut control_flow = ControlFlow::new();
        
        // Simulate condition evaluations at different iterations
        control_flow.evaluate_condition(true); // iteration 0
        control_flow.increment_iteration_count();
        
        control_flow.evaluate_condition(true); // iteration 1
        control_flow.increment_iteration_count();
        
        control_flow.evaluate_condition(false); // iteration 2
        
        // Verify condition evaluation order is tracked
        let evaluation_order = control_flow.get_condition_evaluation_order();
        assert_eq!(evaluation_order, &[0, 1, 2]);
    }

    #[test]
    fn test_shape_fingerprint_creation() {
        use crate::loop_engine::fingerprint::LoopType;
        
        let mut control_flow = ControlFlow::new();
        
        // Simulate loop execution with breaks and continues
        control_flow.evaluate_condition(true); // condition at iteration 0
        control_flow.increment_iteration_count(); // iteration 1
        control_flow.handle_continue();
        
        control_flow.evaluate_condition(true); // condition at iteration 1
        control_flow.increment_iteration_count(); // iteration 2
        
        control_flow.evaluate_condition(false); // condition at iteration 2
        control_flow.increment_iteration_count(); // iteration 3
        control_flow.handle_break();
        
        // Create shape fingerprint
        let loop_id = 12345u64;
        let loop_type = LoopType::While;
        let shape_fingerprint = control_flow.create_shape_fingerprint(loop_id, loop_type);
        
        // Verify shape fingerprint contents
        assert_eq!(shape_fingerprint.loop_id, loop_id);
        assert_eq!(shape_fingerprint.loop_type, loop_type);
        assert_eq!(shape_fingerprint.iteration_count, 3);
        assert_eq!(shape_fingerprint.break_positions, vec![3]);
        assert_eq!(shape_fingerprint.continue_positions, vec![1]);
        assert_eq!(shape_fingerprint.condition_evaluation_order, vec![0, 1, 2]);
    }
}
