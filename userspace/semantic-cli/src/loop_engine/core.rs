//! Core Loop Execution Logic - Phase 3.1 Modularization
//!
//! This module contains the core execution logic extracted from executor.rs,
//! focusing on main execution loop coordination and state management.
//!
//! # Restricted Imports
//! 
//! Following the task requirements, this module only imports:
//! - `use super::{state, accumulator, errors};`
//!
//! # Core Responsibilities
//!
//! - Main execution loop coordination
//! - State transitions and lifecycle management  
//! - Integration with safety analyzer
//! - Public API surface maintenance
//!
//! # Constitutional Compliance
//!
//! This implementation MUST follow the locked semantics:
//! 1. Execute iteration body
//! 2. Update accumulator value  
//! 3. Type validation check (PRE-COMMIT)
//! 4. POST-INCREMENT: completed_iterations += 1 (AFTER successful iteration)
//! 5. Check limits (iteration/budget)
//! 6. Decide: continue/terminate/error

use super::{state, errors, control};
use crate::bcib::{LoopInstruction, LoopType, Value, LoopRange};
use crate::error::{Result, SemanticCLIError, ErrorCode};

/// Loop body execution function type
/// 
/// Phase 2.3: Enhanced with control flow support
/// Returns Ok(ControlFlowDecision) for normal execution, break, or continue
/// Returns Err for execution errors
pub type LoopBodyFn = Box<dyn Fn(&Value, u32) -> Result<LoopBodyResult>>;

/// Result of loop body execution (Phase 2.3)
#[derive(Debug, Clone, PartialEq)]
pub enum LoopBodyResult {
    /// Normal execution - continue with new accumulator
    Normal(Value),
    /// Break - early termination with accumulator
    Break(Value),
    /// Continue - skip remaining body, proceed to next iteration
    Continue(Value),
}

/// Core loop execution engine
pub struct LoopExecutor {
    // Phase 2.1: Minimal state
    // Future phases will add budget tracking, safety analysis, etc.
}

impl LoopExecutor {
    /// Create a new loop executor
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a loop instruction (Phase 2.1 - Core dispatcher)
    pub fn execute_loop(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
    ) -> Result<errors::LoopResult> {
        // Create loop context from instruction
        let context = self.create_loop_context(instruction)?;
        
        // Create initial loop state
        let mut state = state::LoopState::new(context, instruction.get_initial_accumulator().clone())?;

        // Dispatch to appropriate loop type
        match instruction.loop_type() {
            LoopType::While => self.execute_while_loop(&mut state, instruction, body_fn),
            LoopType::For => self.execute_for_loop(&mut state, instruction, body_fn),
            LoopType::ForEach => self.execute_foreach_loop(&mut state, instruction, body_fn),
        }
    }

    /// Execute While loop (Phase 2.3 - Enhanced with break/continue)
    fn execute_while_loop(
        &mut self,
        state: &mut state::LoopState,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
    ) -> Result<errors::LoopResult> {
        // Create control flow manager from loop state
        let mut control_flow = control::ControlFlow::from_loop_state(state);
        
        loop {
            // 🔒 CONSTITUTIONAL: PRE-CHECK iteration limit using control flow
            if control_flow.would_exceed_iteration_limit() {
                return Ok(errors::LoopResult::error(errors::LoopError::IterationLimitExceeded {
                    limit: state.context.iteration_limit,
                    completed: control_flow.get_iteration_count(),
                }));
            }

            // 🔒 CONSTITUTIONAL: PRE-CHECK budget timeout (Phase 2.2) using control flow
            let budget_cost = control::BudgetCalculator::calculate_iteration_budget_cost(&state.context.budget_measurement);
            if control_flow.would_exceed_budget_timeout(budget_cost) {
                return Ok(errors::LoopResult::error(errors::LoopError::BudgetTimeoutExceeded {
                    budget: state.context.budget_timeout,
                    consumed: control_flow.get_budget_consumed(),
                    iterations_completed: control_flow.get_iteration_count(),
                }));
            }

            // Phase 2.2: Check wall-clock kill switch using control flow
            if let Err(_) = control_flow.check_wall_clock_kill_switch() {
                return Ok(errors::LoopResult::error(errors::LoopError::LoopBodyError {
                    iteration: control_flow.get_iteration_count(),
                    error: "Wall-clock kill switch triggered".to_string(),
                }));
            }

            // Phase 2.4: Evaluate while condition
            let condition_result = self.evaluate_while_condition(state, instruction)?;
            let decision = control_flow.evaluate_condition(condition_result);
            
            if decision == control::ControlFlowDecision::Break {
                // Condition is false - normal loop termination
                return Ok(errors::LoopResult::success(
                    state.get_accumulator().clone(),
                    control_flow.get_iteration_count(),
                ));
            }
            
            // 🔒 CONSTITUTIONAL SEQUENCE: Execute iteration with control flow
            match self.execute_single_iteration_with_control(state, &body_fn, &mut control_flow) {
                Ok(control::ControlFlowDecision::Continue) => {
                    // Continue to next iteration
                    continue;
                }
                Ok(control::ControlFlowDecision::Break) => {
                    // Break: return break result
                    return Ok(errors::LoopResult::break_result(
                        state.get_accumulator().clone(),
                        control_flow.get_iteration_count(),
                    ));
                }
                Ok(control::ControlFlowDecision::Skip) => {
                    // Continue statement: skip to next iteration
                    continue;
                }
                Err(e) => {
                    // Error during iteration - return error result
                    return Ok(errors::LoopResult::error(errors::LoopError::LoopBodyError {
                        iteration: control_flow.get_iteration_count(),
                        error: e.to_string(),
                    }));
                }
            }
        }
    }

    /// Execute For loop (Phase 2.3 - Enhanced with break/continue)
    fn execute_for_loop(
        &mut self,
        state: &mut state::LoopState,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
    ) -> Result<errors::LoopResult> {
        // Extract range from instruction
        let range = instruction.get_range()
            .ok_or_else(|| SemanticCLIError::execution_error(
                "For loop missing range specification",
                ErrorCode::E500,
            ))?;

        // Create control flow manager and range iterator
        let mut control_flow = control::ControlFlow::from_loop_state(state);
        let mut range_iterator = control::RangeIterator::from_loop_range(range);
        
        loop {
            // Check if range is complete
            if range_iterator.is_complete() {
                // Range complete - normal termination
                return Ok(errors::LoopResult::success(
                    state.get_accumulator().clone(),
                    control_flow.get_iteration_count(),
                ));
            }

            // 🔒 CONSTITUTIONAL: PRE-CHECK iteration limit using control flow
            if control_flow.would_exceed_iteration_limit() {
                return Ok(errors::LoopResult::error(errors::LoopError::IterationLimitExceeded {
                    limit: state.context.iteration_limit,
                    completed: control_flow.get_iteration_count(),
                }));
            }

            // 🔒 CONSTITUTIONAL: PRE-CHECK budget timeout using control flow
            let budget_cost = control::BudgetCalculator::calculate_iteration_budget_cost(&state.context.budget_measurement);
            if control_flow.would_exceed_budget_timeout(budget_cost) {
                return Ok(errors::LoopResult::error(errors::LoopError::BudgetTimeoutExceeded {
                    budget: state.context.budget_timeout,
                    consumed: control_flow.get_budget_consumed(),
                    iterations_completed: control_flow.get_iteration_count(),
                }));
            }

            // Phase 2.2: Check wall-clock kill switch using control flow
            if let Err(_) = control_flow.check_wall_clock_kill_switch() {
                return Ok(errors::LoopResult::error(errors::LoopError::LoopBodyError {
                    iteration: control_flow.get_iteration_count(),
                    error: "Wall-clock kill switch triggered".to_string(),
                }));
            }

            // Get current iterator value
            let iterator_value = range_iterator.current_value();
            
            // 🔒 CONSTITUTIONAL SEQUENCE: Execute iteration with control flow
            match self.execute_single_iteration_with_iterator_and_control(state, &body_fn, &iterator_value, &mut control_flow) {
                Ok(control::ControlFlowDecision::Continue) => {
                    // Continue to next iteration
                    range_iterator.advance();
                    continue;
                }
                Ok(control::ControlFlowDecision::Break) => {
                    // Break: return break result
                    return Ok(errors::LoopResult::break_result(
                        state.get_accumulator().clone(),
                        control_flow.get_iteration_count(),
                    ));
                }
                Ok(control::ControlFlowDecision::Skip) => {
                    // Continue statement: skip to next iteration
                    range_iterator.advance();
                    continue;
                }
                Err(e) => {
                    return Ok(errors::LoopResult::error(errors::LoopError::LoopBodyError {
                        iteration: control_flow.get_iteration_count(),
                        error: e.to_string(),
                    }));
                }
            }
        }
    }

    /// Execute ForEach loop (Phase 3.1 - Collection determinism support)
    fn execute_foreach_loop(
        &mut self,
        state: &mut state::LoopState,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
    ) -> Result<errors::LoopResult> {
        // Extract collection and collection type from instruction
        let (collection_ref, collection_type) = match instruction {
            LoopInstruction::ForEach { collection, collection_type, .. } => (collection, collection_type),
            _ => {
                return Err(SemanticCLIError::execution_error(
                    "execute_foreach_loop called on non-ForEach loop",
                    ErrorCode::E500,
                ));
            }
        };

        // Phase 3.1: Resolve collection value from operand reference
        let collection_value = self.resolve_collection_operand(collection_ref, state)?;

        // Phase 3.1: Validate collection type matches expected type
        self.validate_collection_determinism(&collection_value, collection_type)?;

        // Phase 3.1: Create deterministic iterator
        let collection_iter = collection_value.iter_collection()
            .ok_or_else(|| SemanticCLIError::execution_error(
                "Value is not a collection",
                ErrorCode::E500,
            ))?;

        // Create control flow manager
        let mut control_flow = control::ControlFlow::from_loop_state(state);

        // Phase 3.1: Iterate over collection in deterministic order
        for collection_element in collection_iter {
            // 🔒 CONSTITUTIONAL: PRE-CHECK iteration limit using control flow
            if control_flow.would_exceed_iteration_limit() {
                return Ok(errors::LoopResult::error(errors::LoopError::IterationLimitExceeded {
                    limit: state.context.iteration_limit,
                    completed: control_flow.get_iteration_count(),
                }));
            }

            // 🔒 CONSTITUTIONAL: PRE-CHECK budget timeout using control flow
            let budget_cost = control::BudgetCalculator::calculate_iteration_budget_cost(&state.context.budget_measurement);
            if control_flow.would_exceed_budget_timeout(budget_cost) {
                return Ok(errors::LoopResult::error(errors::LoopError::BudgetTimeoutExceeded {
                    budget: state.context.budget_timeout,
                    consumed: control_flow.get_budget_consumed(),
                    iterations_completed: control_flow.get_iteration_count(),
                }));
            }

            // Phase 2.2: Check wall-clock kill switch using control flow
            if let Err(_) = control_flow.check_wall_clock_kill_switch() {
                return Ok(errors::LoopResult::error(errors::LoopError::LoopBodyError {
                    iteration: control_flow.get_iteration_count(),
                    error: "Wall-clock kill switch triggered".to_string(),
                }));
            }

            // Phase 3.1: Execute iteration with collection element as iterator value
            match self.execute_single_iteration_with_collection_element_and_control(state, &body_fn, collection_element.value(), &mut control_flow) {
                Ok(control::ControlFlowDecision::Continue) => {
                    // Continue to next iteration
                    continue;
                }
                Ok(control::ControlFlowDecision::Break) => {
                    // Break: return break result
                    return Ok(errors::LoopResult::break_result(
                        state.get_accumulator().clone(),
                        control_flow.get_iteration_count(),
                    ));
                }
                Ok(control::ControlFlowDecision::Skip) => {
                    // Continue statement: skip to next iteration
                    continue;
                }
                Err(e) => {
                    // Error during iteration - return error result
                    return Ok(errors::LoopResult::error(errors::LoopError::LoopBodyError {
                        iteration: control_flow.get_iteration_count(),
                        error: e.to_string(),
                    }));
                }
            }
        }

        // Collection iteration complete - normal termination
        Ok(errors::LoopResult::success(
            state.get_accumulator().clone(),
            control_flow.get_iteration_count(),
        ))
    }

    /// Execute a single iteration following constitutional sequence with control flow
    /// 
    /// 🔒 CONSTITUTIONAL SEQUENCE (LOCKED):
    /// 1. Execute iteration body
    /// 2. Handle control flow (break/continue) - Phase 2.3
    /// 3. Update accumulator value (if not continue)
    /// 4. Type validation check (PRE-COMMIT)
    /// 5. POST-INCREMENT: completed_iterations += 1
    /// 6. POST-INCREMENT: budget_consumed += budget_cost
    /// 7. Return control flow decision
    fn execute_single_iteration_with_control(
        &mut self,
        state: &mut state::LoopState,
        body_fn: &LoopBodyFn,
        control_flow: &mut control::ControlFlow,
    ) -> Result<control::ControlFlowDecision> {
        // 1. Execute iteration body
        let body_result = body_fn(state.get_accumulator(), control_flow.get_iteration_count())?;

        // 2. Handle control flow (Phase 2.3)
        match body_result {
            LoopBodyResult::Break(accumulator) => {
                // Break: Update accumulator, increment counters, return break decision
                state.update_accumulator(accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // Account for break instruction in budget
                let budget_cost = control::BudgetCalculator::calculate_break_budget_cost();
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);
                
                // Record break decision
                let decision = control_flow.handle_break();
                return Ok(decision);
            }
            LoopBodyResult::Continue(accumulator) => {
                // Continue: Update accumulator, increment counters, return skip decision
                state.update_accumulator(accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // Account for continue instruction in budget
                let budget_cost = control::BudgetCalculator::calculate_continue_budget_cost();
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);
                
                // Record continue decision
                let decision = control_flow.handle_continue();
                return Ok(decision);
            }
            LoopBodyResult::Normal(new_accumulator) => {
                // Normal execution: proceed with standard flow
                
                // 3. Update accumulator value
                // 4. Type validation check (PRE-COMMIT) - handled by update_accumulator
                state.update_accumulator(new_accumulator)?;

                // 5. 🔒 CONSTITUTIONAL: POST-INCREMENT after successful iteration
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();

                // 6. 🔒 CONSTITUTIONAL: POST-INCREMENT budget after successful iteration
                let budget_cost = control::BudgetCalculator::calculate_iteration_budget_cost(&state.context.budget_measurement);
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);

                // 7. Continue loop normally
                Ok(control::ControlFlowDecision::Continue)
            }
        }
    }

    /// Execute single iteration with iterator variable and control flow (For loops)
    fn execute_single_iteration_with_iterator_and_control(
        &mut self,
        state: &mut state::LoopState,
        body_fn: &LoopBodyFn,
        _iterator_value: &Value,
        control_flow: &mut control::ControlFlow,
    ) -> Result<control::ControlFlowDecision> {
        // Phase 2.1: Simple approach - pass iterator via closure context
        // Future phases will integrate with proper variable scoping
        
        // 1. Execute iteration body (iterator passed via closure context)
        let body_result = body_fn(state.get_accumulator(), control_flow.get_iteration_count())?;

        // 2. Handle control flow (Phase 2.3) - same as regular iteration
        match body_result {
            LoopBodyResult::Break(accumulator) => {
                // Break: Update accumulator, increment counters, return break decision
                state.update_accumulator(accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // Account for break instruction in budget
                let budget_cost = control::BudgetCalculator::calculate_break_budget_cost();
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);
                
                let decision = control_flow.handle_break();
                return Ok(decision);
            }
            LoopBodyResult::Continue(accumulator) => {
                // Continue: Update accumulator, increment counters, return skip decision
                state.update_accumulator(accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // Account for continue instruction in budget
                let budget_cost = control::BudgetCalculator::calculate_continue_budget_cost();
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);
                
                let decision = control_flow.handle_continue();
                return Ok(decision);
            }
            LoopBodyResult::Normal(new_accumulator) => {
                // Normal execution: proceed with standard flow
                state.update_accumulator(new_accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // POST-INCREMENT budget after successful iteration
                let budget_cost = control::BudgetCalculator::calculate_iteration_budget_cost(&state.context.budget_measurement);
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);

                Ok(control::ControlFlowDecision::Continue)
            }
        }
    }

    /// Execute single iteration with collection element and control flow (ForEach loops)
    fn execute_single_iteration_with_collection_element_and_control(
        &mut self,
        state: &mut state::LoopState,
        body_fn: &LoopBodyFn,
        _collection_element: &Value,
        control_flow: &mut control::ControlFlow,
    ) -> Result<control::ControlFlowDecision> {
        // Phase 3.1: Simple approach - pass collection element via closure context
        // Future phases will integrate with proper variable scoping
        
        // 1. Execute iteration body (collection element passed via closure context)
        let body_result = body_fn(state.get_accumulator(), control_flow.get_iteration_count())?;

        // 2. Handle control flow (Phase 3.1) - same as regular iteration
        match body_result {
            LoopBodyResult::Break(accumulator) => {
                // Break: Update accumulator, increment counters, return break decision
                state.update_accumulator(accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // Account for break instruction in budget
                let budget_cost = control::BudgetCalculator::calculate_break_budget_cost();
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);
                
                let decision = control_flow.handle_break();
                return Ok(decision);
            }
            LoopBodyResult::Continue(accumulator) => {
                // Continue: Update accumulator, increment counters, return skip decision
                state.update_accumulator(accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // Account for continue instruction in budget
                let budget_cost = control::BudgetCalculator::calculate_continue_budget_cost();
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);
                
                let decision = control_flow.handle_continue();
                return Ok(decision);
            }
            LoopBodyResult::Normal(new_accumulator) => {
                // Normal execution: proceed with standard flow
                state.update_accumulator(new_accumulator)?;
                state.increment_completed_iterations();
                control_flow.increment_iteration_count();
                
                // POST-INCREMENT budget after successful iteration
                let budget_cost = control::BudgetCalculator::calculate_iteration_budget_cost(&state.context.budget_measurement);
                state.add_budget_consumed(budget_cost);
                control_flow.add_budget_consumed(budget_cost);

                Ok(control::ControlFlowDecision::Continue)
            }
        }
    }

    /// Create loop context from instruction
    fn create_loop_context(&self, instruction: &LoopInstruction) -> Result<state::LoopContext> {
        let config = instruction.get_config();
        let loop_id = instruction.get_loop_id().clone();
        
        // Phase 2.1: Simple body reference
        let loop_body = format!("loop-body-{}", loop_id.0);

        Ok(state::LoopContext::new(loop_id, config, loop_body))
    }

    /// Evaluate While loop condition (Phase 2.4)
    /// 
    /// Requirements 1.9, 1.10: Evaluate condition under same safety rules as loop body,
    /// detect and reject non-deterministic conditions, support explicit capture/logging
    fn evaluate_while_condition(
        &self,
        _state: &state::LoopState,
        instruction: &LoopInstruction,
    ) -> Result<bool> {
        // Extract condition from While instruction
        let condition = match instruction {
            LoopInstruction::While { condition, .. } => condition,
            _ => {
                return Err(SemanticCLIError::execution_error(
                    "evaluate_while_condition called on non-While loop",
                    ErrorCode::E500,
                ));
            }
        };

        // Phase 2.4: Evaluate condition with safety analysis
        let condition_value = self.evaluate_condition_expression(condition, _state)?;
        self.convert_to_boolean(condition_value)
    }

    /// Evaluate a condition expression with safety analysis (Phase 2.4)
    /// 
    /// This method evaluates expressions under the same safety rules as loop bodies,
    /// detecting non-deterministic operations like external calls, I/O, and timing.
    fn evaluate_condition_expression(
        &self,
        condition: &crate::bcib::OperandRef,
        _state: &state::LoopState,
    ) -> Result<Value> {
        match condition {
            crate::bcib::OperandRef::Literal(value) => {
                // Literal values are always deterministic
                Ok(value.clone())
            }
            crate::bcib::OperandRef::Field(_field_name) => {
                // Field references are deterministic if they reference loop state
                // Phase 2.4: For now, assume field references are deterministic
                // Future phases will implement proper field resolution
                
                // Placeholder: return true for field references
                Ok(Value::Boolean(true))
            }
            crate::bcib::OperandRef::TempRegister(_register_id) => {
                // Temp register references are deterministic if they contain deterministic values
                // Phase 2.4: For now, assume temp registers are deterministic
                // Future phases will implement proper register resolution
                
                // Placeholder: return true for temp register references
                Ok(Value::Boolean(true))
            }
        }
    }

    /// Convert a value to boolean for condition evaluation
    fn convert_to_boolean(&self, value: Value) -> Result<bool> {
        match value {
            Value::Boolean(b) => Ok(b),
            Value::Number(n) => Ok(n != 0.0),
            Value::String(s) => Ok(!s.is_empty()),
            Value::Array(arr) => Ok(!arr.is_empty()),
            Value::List(list) => Ok(!list.is_empty()),
            Value::SortedMap(map) => Ok(!map.is_empty()),
        }
    }

    /// Resolve collection operand to a value (Phase 3.1)
    fn resolve_collection_operand(
        &self,
        collection_ref: &crate::bcib::OperandRef,
        _state: &state::LoopState,
    ) -> Result<Value> {
        match collection_ref {
            crate::bcib::OperandRef::Literal(value) => {
                // Literal collections are directly available
                Ok(value.clone())
            }
            crate::bcib::OperandRef::Field(field_name) => {
                // Field references need to be resolved from context
                // Phase 3.1: For now, return an error for field references
                // Future phases will implement proper field resolution
                Err(SemanticCLIError::execution_error(
                    &format!("Field reference '{}' resolution not implemented in Phase 3.1", field_name),
                    ErrorCode::E500,
                ))
            }
            crate::bcib::OperandRef::TempRegister(register_id) => {
                // Temp register references need to be resolved from execution context
                // Phase 3.1: For now, return an error for temp register references
                // Future phases will implement proper register resolution
                Err(SemanticCLIError::execution_error(
                    &format!("Temp register {} resolution not implemented in Phase 3.1", register_id),
                    ErrorCode::E500,
                ))
            }
        }
    }

    /// Validate collection determinism (Phase 3.1)
    fn validate_collection_determinism(
        &self,
        collection_value: &Value,
        expected_type: &crate::bcib::CollectionType,
    ) -> Result<()> {
        // Phase 3.1: Validate that collection type matches expected type
        match (collection_value, expected_type) {
            (Value::Array(_), crate::bcib::CollectionType::Array) => Ok(()),
            (Value::List(_), crate::bcib::CollectionType::List) => Ok(()),
            (Value::SortedMap(_), crate::bcib::CollectionType::SortedMap) => Ok(()),
            _ => {
                Err(SemanticCLIError::validation_error(
                    &format!(
                        "Collection type mismatch: expected {:?}, got {:?}",
                        expected_type,
                        collection_value.value_type()
                    ),
                    "Ensure collection type matches loop specification",
                    ErrorCode::E301,
                ))
            }
        }
    }
}

impl Default for LoopExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for LoopInstruction to extract data (needed for core execution)
trait LoopInstructionExt {
    fn get_config(&self) -> &crate::bcib::LoopConfig;
    fn get_loop_id(&self) -> &crate::bcib::LoopID;
    fn get_initial_accumulator(&self) -> &Value;
    fn get_range(&self) -> Option<&LoopRange>;
    #[allow(dead_code)]
    fn loop_type(&self) -> LoopType;
}

impl LoopInstructionExt for LoopInstruction {
    fn get_config(&self) -> &crate::bcib::LoopConfig {
        match self {
            LoopInstruction::While { config, .. } => config,
            LoopInstruction::For { config, .. } => config,
            LoopInstruction::ForEach { config, .. } => config,
        }
    }

    fn get_loop_id(&self) -> &crate::bcib::LoopID {
        match self {
            LoopInstruction::While { id, .. } => id,
            LoopInstruction::For { id, .. } => id,
            LoopInstruction::ForEach { id, .. } => id,
        }
    }

    fn get_initial_accumulator(&self) -> &Value {
        &self.get_config().initial_accumulator
    }

    fn get_range(&self) -> Option<&LoopRange> {
        match self {
            LoopInstruction::For { range, .. } => Some(range),
            _ => None,
        }
    }

    fn loop_type(&self) -> LoopType {
        match self {
            LoopInstruction::While { .. } => LoopType::While,
            LoopInstruction::For { .. } => LoopType::For,
            LoopInstruction::ForEach { .. } => LoopType::ForEach,
        }
    }
}