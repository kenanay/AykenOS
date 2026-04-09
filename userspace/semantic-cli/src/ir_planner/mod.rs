//! IR Executor Implementation (C7)
//! 
//! **Created By:** Kenan AY
//! **Date:** 16 Ocak 2026
//! **Architectural Reference:** C4 - Execution Engine Architecture
//! 
//! Single-threaded deterministic IR interpreter that executes ExecutionPlan IR
//! with guaranteed reproducible results.
//! 
//! **Key Principle:** Pull execution model with step-by-step replay capability.

use crate::execution_plan::{ExecutionPlan, IRInstruction, BlockTerminator, RegisterId, BlockId};
use crate::bcib::{Value, ComparisonOp, LogicalOperator, FilterExpression};
use crate::context::ContextManager;
use crate::memory::ExecutionPools;
use std::collections::HashMap;

// Parallelism imports (optional - only available when phase2-implementation feature is enabled)
#[cfg(feature = "phase2-implementation")]
use crate::parallelism::{
    ParallelExecutor, AdaptiveDecisionEngine, RayonParallelExecutor, DefaultDecisionEngine
};

pub mod register_file;
pub mod replay;

use register_file::{RegisterFile, RegisterValue};
use replay::{ReplayRecorder, ReplayTrace};

/// Instruction identifier for execution tracking
pub type InstructionId = u32;

/// Main IR Executor - deterministic interpreter for ExecutionPlan IR
pub struct IRExecutor {
    /// Register file for virtual register storage
    register_file: RegisterFile,
    /// Context manager for loading context data
    context_manager: ContextManager,
    /// Current execution state
    execution_state: ExecutionState,
    /// Replay recorder for step-by-step debugging
    replay_recorder: ReplayRecorder,
    /// Object pools for memory optimization (Phase 4.3.3.1)
    pools: ExecutionPools,
    /// Optional parallel executor (only available with phase2-implementation feature)
    #[cfg(feature = "phase2-implementation")]
    parallel_executor: Option<Box<dyn ParallelExecutor>>,
    /// Optional adaptive decision engine (only available with phase2-implementation feature)
    #[cfg(feature = "phase2-implementation")]
    decision_engine: Option<Box<dyn AdaptiveDecisionEngine>>,
}

impl IRExecutor {
    /// Create new IR executor
    pub fn new() -> Self {
        Self {
            register_file: RegisterFile::new(),
            context_manager: ContextManager::new(),
            execution_state: ExecutionState::new(),
            replay_recorder: ReplayRecorder::new(),
            pools: ExecutionPools::with_capacity(16), // Pre-allocate for warmup
            #[cfg(feature = "phase2-implementation")]
            parallel_executor: None,
            #[cfg(feature = "phase2-implementation")]
            decision_engine: None,
        }
    }
    
    /// Create IR executor with custom context manager
    pub fn with_context_manager(context_manager: ContextManager) -> Self {
        Self {
            register_file: RegisterFile::new(),
            context_manager,
            execution_state: ExecutionState::new(),
            replay_recorder: ReplayRecorder::new(),
            pools: ExecutionPools::with_capacity(16), // Pre-allocate for warmup
            #[cfg(feature = "phase2-implementation")]
            parallel_executor: None,
            #[cfg(feature = "phase2-implementation")]
            decision_engine: None,
        }
    }
    
    /// Enable parallelism with default components (only available with phase2-implementation feature)
    #[cfg(feature = "phase2-implementation")]
    pub fn with_parallelism(mut self) -> Self {
        self.parallel_executor = Some(Box::new(RayonParallelExecutor::new()));
        self.decision_engine = Some(Box::new(DefaultDecisionEngine::new()));
        self
    }
    
    /// Enable parallelism with custom components (only available with phase2-implementation feature)
    #[cfg(feature = "phase2-implementation")]
    pub fn with_custom_parallelism(
        mut self,
        parallel_executor: Box<dyn ParallelExecutor>,
        decision_engine: Box<dyn AdaptiveDecisionEngine>,
    ) -> Self {
        self.parallel_executor = Some(parallel_executor);
        self.decision_engine = Some(decision_engine);
        self
    }
    
    /// Get pool statistics for performance monitoring (Phase 4.3.3.1)
    pub fn pool_stats(&self) -> &crate::memory::PoolStats {
        self.pools.stats()
    }
    
    /// Get current pool sizes for monitoring (Phase 4.3.3.1)
    pub fn pool_sizes(&self) -> crate::memory::PoolSizes {
        self.pools.pool_sizes()
    }
    
    /// Manually clear pools (for testing or explicit cleanup)
    /// 
    /// **Constitutional Rule:** Pools are automatically cleared after each execution
    pub fn clear_pools(&mut self) {
        self.pools.clear_all();
    }
    
    /// Check if parallelism is enabled
    #[cfg(feature = "phase2-implementation")]
    pub fn is_parallelism_enabled(&self) -> bool {
        self.parallel_executor.is_some() && self.decision_engine.is_some()
    }
    
    /// Check if parallelism is enabled (always false without phase2-implementation feature)
    #[cfg(not(feature = "phase2-implementation"))]
    pub fn is_parallelism_enabled(&self) -> bool {
        false
    }
    
    /// Execute ExecutionPlan with deterministic results
    /// 
    /// **Architectural Reference:** C4 Section "Execution Flow"
    pub fn execute(&mut self, plan: ExecutionPlan) -> Result<ExecutionResult, ExecutionError> {
        // Initialize execution
        self.initialize_execution(&plan)?;
        
        // Check if we should use parallel execution
        #[cfg(feature = "phase2-implementation")]
        if self.should_use_parallel_execution(&plan)? {
            let result = self.execute_parallel_path(plan);
            // Constitutional Rule: Clear pools after execution (no cross-run leakage)
            self.pools.clear_all();
            return result;
        }
        
        // Use sequential execution (default path)
        let result = self.execute_sequential_path(plan);
        
        // Constitutional Rule: Clear pools after execution (no cross-run leakage)
        self.pools.clear_all();
        
        result
    }
    
    /// Execute using sequential path (original implementation)
    fn execute_sequential_path(&mut self, plan: ExecutionPlan) -> Result<ExecutionResult, ExecutionError> {
        // Main execution loop (pull model)
        while !self.execution_state.is_terminated {
            // Fetch next instruction
            let instruction = self.fetch_next_instruction(&plan)?;
            
            // Execute instruction
            self.execute_instruction(instruction)?;
            
            // Advance execution state
            self.advance_execution_state(&plan)?;
            
            // Record for replay
            self.replay_recorder.record_step(&self.execution_state, &self.register_file)?;
            
            // Break if terminated (critical for deterministic execution)
            if self.execution_state.is_terminated {
                break;
            }
        }
        
        // Finalize and return result
        self.finalize_execution()
    }
    
    /// Execute using parallel path (only available with phase2-implementation feature)
    #[cfg(feature = "phase2-implementation")]
    fn execute_parallel_path(&mut self, plan: ExecutionPlan) -> Result<ExecutionResult, ExecutionError> {
        // For now, this is a placeholder that falls back to sequential execution
        // Full parallel execution would require significant refactoring of the execution model
        // to support data-parallel operations on IR blocks
        
        // TODO: Implement full parallel execution path
        // This would involve:
        // 1. Identifying parallelizable IR blocks
        // 2. Partitioning input data
        // 3. Executing blocks in parallel using the parallel executor
        // 4. Merging results deterministically
        
        // For Task 14, we implement the integration points but fall back to sequential
        self.execute_sequential_path(plan)
    }
    
    /// Determine if parallel execution should be used
    #[cfg(feature = "phase2-implementation")]
    fn should_use_parallel_execution(&self, _plan: &ExecutionPlan) -> Result<bool, ExecutionError> {
        // Check if parallelism is enabled
        if !self.is_parallelism_enabled() {
            return Ok(false);
        }
        
        // CONSTITUTIONAL ENFORCEMENT: P3 - Replay First-Class Citizen
        // Replay mode MUST use sequential execution for deterministic reproduction
        if self.replay_recorder.is_recording() {
            return Ok(false);
        }
        
        // For now, always use sequential execution
        // Full implementation would analyze the execution plan and make decisions
        // based on the adaptive decision engine
        
        // TODO: Implement full decision logic
        // This would involve:
        // 1. Analyzing IR blocks for parallel safety
        // 2. Estimating data sizes
        // 3. Consulting the adaptive decision engine
        // 4. Checking blacklist status
        
        Ok(false)
    }
    
    /// Execute with full replay trace recording
    /// 
    /// **Architectural Reference:** C4 Section "Replay System Integration"
    pub fn execute_with_replay(&mut self, plan: ExecutionPlan) -> Result<(ExecutionResult, ReplayTrace), ExecutionError> {
        // Enable replay recording
        self.replay_recorder.enable_recording();
        
        // Capture context snapshots for deterministic replay
        let context_snapshots = self.capture_context_snapshots(&plan)?;
        self.replay_recorder.set_context_snapshots(context_snapshots);
        
        // Execute with step recording
        let result = self.execute(plan)?;
        
        // Get replay trace
        let trace = self.replay_recorder.finalize_trace()?;
        
        // Constitutional Rule: Clear pools after execution (no cross-run leakage)
        self.pools.clear_all();
        
        Ok((result, trace))
    }
    
    /// Initialize execution state
    fn initialize_execution(&mut self, plan: &ExecutionPlan) -> Result<(), ExecutionError> {
        // Validate execution plan
        plan.validate().map_err(|e| ExecutionError::InvalidExecutionPlan { 
            reason: e.to_string() 
        })?;

        let replay_enabled = self.replay_recorder.is_recording();
        let plan_fingerprint = plan.compute_determinism_fingerprint();
        
        // Reset execution state using pooling (Phase 4.3.3.1)
        let mut new_execution_state = self.pools.borrow_execution_state();
        new_execution_state.current_block = plan.entry_block;
        self.execution_state = new_execution_state;
        
        // Initialize register file using pooling (Phase 4.3.3.1)
        self.register_file = self.pools.borrow_register_file();
        
        // Initialize replay recorder using pooling (Phase 4.3.3.1)
        if replay_enabled {
            self.replay_recorder.initialize(plan_fingerprint);
            self.replay_recorder.enable_recording();
        } else {
            self.replay_recorder = self.pools.borrow_replay_recorder();
            self.replay_recorder.initialize(plan_fingerprint);
        }
        
        Ok(())
    }
    
    /// Fetch next instruction using pull model
    /// 
    /// **Architectural Reference:** C4 Section "Instruction Execution Cycle"
    fn fetch_next_instruction<'a>(&self, plan: &'a ExecutionPlan) -> Result<&'a IRInstruction, ExecutionError> {
        let current_block = plan.get_block(self.execution_state.current_block)
            .ok_or(ExecutionError::InvalidBlock { 
                block_id: self.execution_state.current_block 
            })?;
        
        let instruction_index = self.execution_state.instruction_index;
        
        current_block.instructions.get(instruction_index)
            .ok_or(ExecutionError::InstructionIndexOutOfBounds { 
                block_id: self.execution_state.current_block,
                index: instruction_index,
                max_index: current_block.instructions.len(),
            })
    }
    
    /// Execute single IR instruction
    /// 
    /// **Architectural Reference:** C4 Section "Instruction Execution Cycle"
    fn execute_instruction(&mut self, instruction: &IRInstruction) -> Result<(), ExecutionError> {
        match instruction {
            IRInstruction::LoadContext { context_id, target_register } => {
                self.execute_load_context(context_id, *target_register)
            },
            
            IRInstruction::LoadField { source_register, field_name, target_register } => {
                self.execute_load_field(*source_register, field_name, *target_register)
            },
            
            IRInstruction::LoadLiteral { value, target_register } => {
                self.execute_load_literal(value, *target_register)
            },
            
            IRInstruction::Compare { left_register, operator, right_register, target_register } => {
                self.execute_compare(*left_register, *operator, *right_register, *target_register)
            },
            
            IRInstruction::LogicalOp { operation, operand_registers, target_register } => {
                self.execute_logical_op(*operation, operand_registers, *target_register)
            },
            
            IRInstruction::ApplyFilter { context_register, filter_expression, target_register } => {
                // C9: Pass filter expression for per-item evaluation
                self.execute_apply_filter(*context_register, filter_expression, *target_register)
            },
            
            IRInstruction::Return { source_register: _ } => {
                // Return is handled by block terminator, not instruction
                Err(ExecutionError::InvalidInstruction { 
                    instruction: "Return instruction should be block terminator".to_string() 
                })
            },
            
            IRInstruction::Branch { .. } => {
                // Branch is handled by block terminator, not instruction
                Err(ExecutionError::InvalidInstruction { 
                    instruction: "Branch instruction should be block terminator".to_string() 
                })
            },
        }
    }
    
    /// Execute LoadContext instruction
    fn execute_load_context(&mut self, _context_id: &str, target_register: RegisterId) -> Result<(), ExecutionError> {
        // For Gate C, simplified context loading
        // Full implementation will use proper ContextManager
        let context_data = crate::context::ContextData {
            items: vec![
                serde_json::json!({"name": "Alice", "age": 25, "active": true}),
                serde_json::json!({"name": "Bob", "age": 30, "active": false}),
                serde_json::json!({"name": "Charlie", "age": 35, "active": true}),
                serde_json::json!({"name": "Diana", "age": 40, "active": false}),
                serde_json::json!({"name": "Eve", "age": 45, "active": true}),
            ],
            loaded_at: std::time::Instant::now(),
            ttl: std::time::Duration::from_secs(300),
        };
        
        let register_value = RegisterValue::ContextData(context_data);
        self.register_file.set_register(target_register, register_value)?;
        
        Ok(())
    }
    
    /// Execute LoadField instruction (C9: Uses current_item cursor)
    fn execute_load_field(&mut self, source_register: RegisterId, field_name: &str, target_register: RegisterId) -> Result<(), ExecutionError> {
        // C9: Check if we have a current_item (per-item evaluation context)
        if let Some(ref current_item) = self.execution_state.current_item {
            // Load field from current item
            let field_value = current_item.get(field_name)
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            
            // Convert JsonValue to RegisterValue
            let register_value = match field_value {
                serde_json::Value::String(s) => RegisterValue::String(s),
                serde_json::Value::Number(n) => RegisterValue::Number(n.as_f64().unwrap_or(0.0)),
                serde_json::Value::Bool(b) => RegisterValue::Boolean(b),
                serde_json::Value::Null => RegisterValue::Null,
                _ => RegisterValue::String(field_value.to_string()),
            };
            
            self.register_file.set_register(target_register, register_value)?;
            return Ok(());
        }
        
        // C9: Fallback for single-item context (test_register_operations)
        let context_value = self.register_file.get_register(source_register)?;
        
        let field_value = match context_value {
            RegisterValue::ContextData(context_data) => {
                // If single-item context, use that item as current_item
                if context_data.items().len() == 1 {
                    let item = &context_data.items()[0];
                    let field_val = item.get(field_name)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    
                    match field_val {
                        serde_json::Value::String(s) => RegisterValue::String(s),
                        serde_json::Value::Number(n) => RegisterValue::Number(n.as_f64().unwrap_or(0.0)),
                        serde_json::Value::Bool(b) => RegisterValue::Boolean(b),
                        serde_json::Value::Null => RegisterValue::Null,
                        _ => RegisterValue::String(field_val.to_string()),
                    }
                } else {
                    return Err(ExecutionError::InvalidOperation {
                        operation: format!("LoadField on multi-item context without current_item cursor (items: {})", context_data.items().len()),
                    });
                }
            },
            _ => {
                return Err(ExecutionError::TypeMismatch { 
                    expected: "ContextData or current_item".to_string(),
                    actual: context_value.type_name().to_string(),
                });
            }
        };
        
        self.register_file.set_register(target_register, field_value)?;
        
        Ok(())
    }
    
    /// Execute LoadLiteral instruction
    fn execute_load_literal(&mut self, value: &Value, target_register: RegisterId) -> Result<(), ExecutionError> {
        let register_value = RegisterValue::from_bcib_value(value.clone())?;
        self.register_file.set_register(target_register, register_value)?;
        
        Ok(())
    }
    
    /// Execute Compare instruction
    fn execute_compare(&mut self, left_register: RegisterId, operator: ComparisonOp, right_register: RegisterId, target_register: RegisterId) -> Result<(), ExecutionError> {
        let left_value = self.register_file.get_register(left_register)?;
        let right_value = self.register_file.get_register(right_register)?;
        
        let result = self.perform_comparison(left_value, operator, right_value)?;
        
        self.register_file.set_register(target_register, RegisterValue::Boolean(result))?;
        
        Ok(())
    }
    
    /// Execute LogicalOp instruction
    fn execute_logical_op(&mut self, operation: LogicalOperator, operand_registers: &[RegisterId], target_register: RegisterId) -> Result<(), ExecutionError> {
        let operand_values: Result<Vec<_>, _> = operand_registers.iter()
            .map(|reg| self.register_file.get_register(*reg))
            .collect();
        
        let operand_values = operand_values?;
        
        let result = self.perform_logical_operation(operation, &operand_values)?;
        
        self.register_file.set_register(target_register, RegisterValue::Boolean(result))?;
        
        Ok(())
    }
    
    /// Execute ApplyFilter instruction (C9: Per-item evaluation with cursor)
    fn execute_apply_filter(
        &mut self, 
        context_register: RegisterId, 
        filter_expression: &FilterExpression, 
        target_register: RegisterId
    ) -> Result<(), ExecutionError> {
        let context_value = self.register_file.get_register(context_register)?;
        
        // Get context data
        let context_data = match context_value {
            RegisterValue::ContextData(data) => data,
            _ => {
                return Err(ExecutionError::TypeMismatch { 
                    expected: "ContextData".to_string(),
                    actual: context_value.type_name().to_string(),
                });
            }
        };
        
        // C9: Evaluate filter for EACH item using cursor
        let mut filtered_items = Vec::new();
        
        for item in context_data.items() {
            // Set current_item cursor
            self.execution_state.current_item = Some(item.clone());
            
            // Evaluate filter expression for this item
            if self.evaluate_filter_expression(filter_expression)? {
                filtered_items.push(item.clone());
            }
        }
        
        // Clear current_item cursor
        self.execution_state.current_item = None;
        
        // Create filtered context
        let filtered_context = crate::context::ContextData {
            items: filtered_items,
            loaded_at: context_data.loaded_at,
            ttl: context_data.ttl,
        };
        
        self.register_file.update_register(
            target_register,
            RegisterValue::ContextData(filtered_context),
        )?;
        
        Ok(())
    }
    
    /// Evaluate filter expression using current_item cursor (C9)
    fn evaluate_filter_expression(
        &self,
        filter_expression: &FilterExpression,
    ) -> Result<bool, ExecutionError> {
        use crate::bcib::OperandRef;
        
        // Get current item (must be set by ApplyFilter)
        let current_item = self.execution_state.current_item.as_ref()
            .ok_or_else(|| ExecutionError::InvalidOperation {
                operation: "evaluate_filter_expression called without current_item".to_string(),
            })?;
        
        // Get field value from current item
        let field_value = current_item.get(&filter_expression.field)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        
        // Get comparison value
        let comparison_value = match &filter_expression.value {
            OperandRef::Literal(lit) => {
                // Convert BCIB Value to JsonValue
                match lit {
                    Value::String(s) => serde_json::Value::String(s.clone()),
                    Value::Number(n) => serde_json::json!(n),
                    Value::Boolean(b) => serde_json::Value::Bool(*b),
                    // Collections are not supported in filter expressions (Phase 3.1)
                    Value::Array(_) | Value::List(_) | Value::SortedMap(_) => {
                        return Err(ExecutionError::InvalidOperation {
                            operation: "Collection values cannot be used in filter expressions".to_string()
                        });
                    }
                }
            },
            OperandRef::Field(field_name) => {
                current_item.get(field_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            },
            OperandRef::TempRegister(reg_id) => {
                // Get value from register
                let reg_value = self.register_file.get_register(*reg_id)?;
                self.register_value_to_json(reg_value)?
            },
        };
        
        // Perform comparison
        self.compare_json_values(&field_value, filter_expression.operator, &comparison_value)
    }
    
    /// Convert RegisterValue to JsonValue (C9 helper)
    fn register_value_to_json(&self, value: &RegisterValue) -> Result<serde_json::Value, ExecutionError> {
        match value {
            RegisterValue::String(s) => Ok(serde_json::Value::String(s.clone())),
            RegisterValue::Number(n) => Ok(serde_json::json!(n)),
            RegisterValue::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
            RegisterValue::Null => Ok(serde_json::Value::Null),
            _ => Err(ExecutionError::InvalidOperation {
                operation: format!("Cannot convert {} to JSON", value.type_name()),
            }),
        }
    }
    
    /// Compare JSON values (C9 helper)
    fn compare_json_values(
        &self,
        left: &serde_json::Value,
        operator: crate::bcib::ComparisonOp,
        right: &serde_json::Value,
    ) -> Result<bool, ExecutionError> {
        use crate::bcib::ComparisonOp;
        
        match (left, right) {
            (serde_json::Value::String(l), serde_json::Value::String(r)) => {
                Ok(match operator {
                    ComparisonOp::Equal => l == r,
                    ComparisonOp::NotEqual => l != r,
                    ComparisonOp::LessThan => l < r,
                    ComparisonOp::LessThanOrEqual => l <= r,
                    ComparisonOp::GreaterThan => l > r,
                    ComparisonOp::GreaterThanOrEqual => l >= r,
                })
            },
            (serde_json::Value::Number(l), serde_json::Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                Ok(match operator {
                    ComparisonOp::Equal => (l_f64 - r_f64).abs() < f64::EPSILON,
                    ComparisonOp::NotEqual => (l_f64 - r_f64).abs() >= f64::EPSILON,
                    ComparisonOp::LessThan => l_f64 < r_f64,
                    ComparisonOp::LessThanOrEqual => l_f64 <= r_f64,
                    ComparisonOp::GreaterThan => l_f64 > r_f64,
                    ComparisonOp::GreaterThanOrEqual => l_f64 >= r_f64,
                })
            },
            (serde_json::Value::Bool(l), serde_json::Value::Bool(r)) => {
                Ok(match operator {
                    ComparisonOp::Equal => l == r,
                    ComparisonOp::NotEqual => l != r,
                    _ => return Err(ExecutionError::InvalidOperation {
                        operation: format!("Boolean comparison with {:?}", operator),
                    }),
                })
            },
            _ => Err(ExecutionError::TypeMismatch {
                expected: "Compatible types for comparison".to_string(),
                actual: format!("{:?} and {:?}", left, right),
            }),
        }
    }
    
    /// Perform comparison operation
    fn perform_comparison(&self, left: &RegisterValue, operator: ComparisonOp, right: &RegisterValue) -> Result<bool, ExecutionError> {
        match (left, right) {
            (RegisterValue::String(l), RegisterValue::String(r)) => {
                Ok(match operator {
                    ComparisonOp::Equal => l == r,
                    ComparisonOp::NotEqual => l != r,
                    ComparisonOp::LessThan => l < r,
                    ComparisonOp::LessThanOrEqual => l <= r,
                    ComparisonOp::GreaterThan => l > r,
                    ComparisonOp::GreaterThanOrEqual => l >= r,
                })
            },
            
            (RegisterValue::Number(l), RegisterValue::Number(r)) => {
                Ok(match operator {
                    ComparisonOp::Equal => (l - r).abs() < f64::EPSILON,
                    ComparisonOp::NotEqual => (l - r).abs() >= f64::EPSILON,
                    ComparisonOp::LessThan => l < r,
                    ComparisonOp::LessThanOrEqual => l <= r,
                    ComparisonOp::GreaterThan => l > r,
                    ComparisonOp::GreaterThanOrEqual => l >= r,
                })
            },
            
            (RegisterValue::Boolean(l), RegisterValue::Boolean(r)) => {
                Ok(match operator {
                    ComparisonOp::Equal => l == r,
                    ComparisonOp::NotEqual => l != r,
                    _ => return Err(ExecutionError::InvalidOperation { 
                        operation: format!("Boolean comparison with {:?}", operator) 
                    }),
                })
            },
            
            _ => Err(ExecutionError::TypeMismatch { 
                expected: "Compatible types for comparison".to_string(),
                actual: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }
    
    /// Perform logical operation
    fn perform_logical_operation(&self, operation: LogicalOperator, operands: &[&RegisterValue]) -> Result<bool, ExecutionError> {
        match operation {
            LogicalOperator::And => {
                if operands.len() != 2 {
                    return Err(ExecutionError::InvalidOperation { 
                        operation: format!("AND requires 2 operands, got {}", operands.len()) 
                    });
                }
                
                let left = operands[0].as_boolean()?;
                let right = operands[1].as_boolean()?;
                Ok(left && right)
            },
            
            LogicalOperator::Or => {
                if operands.len() != 2 {
                    return Err(ExecutionError::InvalidOperation { 
                        operation: format!("OR requires 2 operands, got {}", operands.len()) 
                    });
                }
                
                let left = operands[0].as_boolean()?;
                let right = operands[1].as_boolean()?;
                Ok(left || right)
            },
            
            LogicalOperator::Not => {
                if operands.len() != 1 {
                    return Err(ExecutionError::InvalidOperation { 
                        operation: format!("NOT requires 1 operand, got {}", operands.len()) 
                    });
                }
                
                let operand = operands[0].as_boolean()?;
                Ok(!operand)
            },
        }
    }
    
    /// Advance execution state after instruction
    /// 
    /// **Architectural Reference:** C4 Section "Instruction Execution Cycle"
    fn advance_execution_state(&mut self, plan: &ExecutionPlan) -> Result<(), ExecutionError> {
        self.execution_state.execution_step += 1;
        
        let current_block = plan.get_block(self.execution_state.current_block)
            .ok_or(ExecutionError::InvalidBlock { 
                block_id: self.execution_state.current_block 
            })?;
        
        // Move to next instruction in current block
        self.execution_state.instruction_index += 1;
        
        // Check if we've reached the end of current block
        if self.execution_state.instruction_index >= current_block.instructions.len() {
            // Handle block terminator
            self.handle_block_terminator(&current_block.terminator)?;
        }
        
        Ok(())
    }
    
    /// Handle block terminator
    fn handle_block_terminator(&mut self, terminator: &BlockTerminator) -> Result<(), ExecutionError> {
        match terminator {
            BlockTerminator::Return { register } => {
                self.execution_state.is_terminated = true;
                self.execution_state.termination_reason = Some(TerminationReason::Return { 
                    register: *register 
                });
            },
            
            BlockTerminator::Branch { condition, true_block, false_block } => {
                let condition_value = self.register_file.get_register(*condition)?;
                let condition_bool = condition_value.as_boolean()?;
                
                let next_block = if condition_bool {
                    *true_block
                } else {
                    *false_block
                };
                
                self.execution_state.current_block = next_block;
                self.execution_state.instruction_index = 0;
            },
            
            BlockTerminator::Jump { target_block } => {
                self.execution_state.current_block = *target_block;
                self.execution_state.instruction_index = 0;
            },
        }
        
        Ok(())
    }
    
    /// Finalize execution and return result
    fn finalize_execution(&self) -> Result<ExecutionResult, ExecutionError> {
        match &self.execution_state.termination_reason {
            Some(TerminationReason::Return { register }) => {
                let return_value = self.register_file.get_register(*register)?;
                Ok(ExecutionResult {
                    value: return_value.clone(),
                    execution_steps: self.execution_state.execution_step,
                    register_state: self.register_file.dump_state(),
                })
            },
            Some(TerminationReason::Error { error }) => {
                Err(error.clone())
            },
            Some(TerminationReason::Timeout) => {
                Err(ExecutionError::ExecutionTimeout { 
                    steps: self.execution_state.execution_step 
                })
            },
            None => {
                Err(ExecutionError::UnexpectedTermination)
            },
        }
    }
    
    /// Capture context snapshots for replay
    fn capture_context_snapshots(&self, _plan: &ExecutionPlan) -> Result<HashMap<String, crate::context::ContextData>, ExecutionError> {
        // For Gate C, simplified context snapshots
        let mut snapshots = HashMap::new();
        
        // Create a simple test context
        let test_context = crate::context::ContextData {
            items: vec![serde_json::json!({"name": "test", "value": 42})],
            loaded_at: std::time::Instant::now(),
            ttl: std::time::Duration::from_secs(300),
        };
        
        snapshots.insert("test".to_string(), test_context);
        
        Ok(snapshots)
    }
}

impl Default for IRExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Current execution state
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionState {
    /// Current block being executed
    pub current_block: BlockId,
    /// Current instruction index within block
    pub instruction_index: usize,
    /// Total execution steps taken
    pub execution_step: u64,
    /// Whether execution has terminated
    pub is_terminated: bool,
    /// Reason for termination
    pub termination_reason: Option<TerminationReason>,
    /// Current item being evaluated (C9: For per-item filter evaluation)
    /// Note: Cannot use reference due to lifetime issues, using owned value
    pub current_item: Option<serde_json::Value>,
}

impl ExecutionState {
    /// Create new execution state
    pub fn new() -> Self {
        Self {
            current_block: 0,
            instruction_index: 0,
            execution_step: 0,
            is_terminated: false,
            termination_reason: None,
            current_item: None,  // C9: Initialize as None
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution termination reasons
#[derive(Debug, Clone, PartialEq)]
pub enum TerminationReason {
    /// Normal return with value
    Return { register: RegisterId },
    /// Error during execution
    Error { error: ExecutionError },
    /// Execution timeout
    Timeout,
}

/// Execution result
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    /// Final return value
    pub value: RegisterValue,
    /// Number of execution steps
    pub execution_steps: u64,
    /// Final register state (for debugging)
    pub register_state: String,
}

/// Execution errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExecutionError {
    #[error("Invalid execution plan: {reason}")]
    InvalidExecutionPlan { reason: String },
    
    #[error("Invalid block: {block_id}")]
    InvalidBlock { block_id: BlockId },
    
    #[error("Instruction index out of bounds: block {block_id}, index {index}, max {max_index}")]
    InstructionIndexOutOfBounds { block_id: BlockId, index: usize, max_index: usize },
    
    #[error("Undefined register: {register:?}")]
    UndefinedRegister { register: RegisterId },
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Context loading failed for '{context_id}': {reason}")]
    ContextLoadFailed { context_id: String, reason: String },
    
    #[error("Field access failed for '{field}': {reason}")]
    FieldAccessFailed { field: String, reason: String },
    
    #[error("Invalid operation: {operation}")]
    InvalidOperation { operation: String },
    
    #[error("Invalid instruction: {instruction}")]
    InvalidInstruction { instruction: String },
    
    #[error("Execution timeout after {steps} steps")]
    ExecutionTimeout { steps: u64 },
    
    #[error("Unexpected termination")]
    UnexpectedTermination,
    
    #[error("Register file error: {reason}")]
    RegisterFileError { reason: String },
    
    #[error("Invalid register value: {0}")]
    InvalidRegisterValue(String),
    
    #[error("Replay error: {reason}")]
    ReplayError { reason: String },
}

// Error conversion implementations
impl From<register_file::RegisterFileError> for ExecutionError {
    fn from(error: register_file::RegisterFileError) -> Self {
        ExecutionError::RegisterFileError { reason: error.to_string() }
    }
}

impl From<replay::ReplayError> for ExecutionError {
    fn from(error: replay::ReplayError) -> Self {
        ExecutionError::ReplayError { reason: error.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_plan::{ExecutionPlan, IRBlock, ExecutionMetadata, ParallelSafety};
    use crate::execution_plan::dataflow::DataflowGraph;
    use crate::normalizer::RegisterAllocation;
    use crate::bcib::Value;
    use std::collections::HashMap;
    
    fn create_test_execution_plan() -> ExecutionPlan {
        let block = IRBlock::with_safety(
            0,
            vec![
                IRInstruction::LoadLiteral {
                    value: Value::String("test".to_string()),
                    target_register: 0,
                },
            ],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Safe, // Pure literal load
        );
        
        ExecutionPlan::new(
            vec![block],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 1,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 1, 1, 1),
        )
    }
    
    #[test]
    fn test_ir_executor_creation() {
        let executor = IRExecutor::new();
        assert_eq!(executor.execution_state.current_block, 0);
        assert_eq!(executor.execution_state.instruction_index, 0);
        assert_eq!(executor.execution_state.execution_step, 0);
        assert!(!executor.execution_state.is_terminated);
    }
    
    #[test]
    fn test_simple_execution() {
        let mut executor = IRExecutor::new();
        let plan = create_test_execution_plan();
        
        let result = executor.execute(plan);
        assert!(result.is_ok());
        
        let execution_result = result.unwrap();
        match execution_result.value {
            RegisterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
        
        assert_eq!(execution_result.execution_steps, 1);
    }
    
    #[test]
    fn test_load_literal_instruction() {
        let mut executor = IRExecutor::new();
        
        let instruction = IRInstruction::LoadLiteral {
            value: Value::Number(42.0),
            target_register: 0,
        };
        
        let result = executor.execute_instruction(&instruction);
        assert!(result.is_ok());
        
        let register_value = executor.register_file.get_register(0).unwrap();
        match register_value {
            RegisterValue::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number value"),
        }
    }
    
    #[test]
    fn test_comparison_operations() {
        let mut executor = IRExecutor::new();
        
        // Set up registers for comparison
        executor.register_file.set_register(0, RegisterValue::Number(10.0)).unwrap();
        executor.register_file.set_register(1, RegisterValue::Number(20.0)).unwrap();
        
        let result = executor.perform_comparison(
            &RegisterValue::Number(10.0),
            ComparisonOp::LessThan,
            &RegisterValue::Number(20.0),
        );
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[test]
    fn test_logical_operations() {
        let executor = IRExecutor::new();
        
        let operands = vec![&RegisterValue::Boolean(true), &RegisterValue::Boolean(false)];
        
        let and_result = executor.perform_logical_operation(LogicalOperator::And, &operands);
        assert!(and_result.is_ok());
        assert!(!and_result.unwrap());
        
        let or_result = executor.perform_logical_operation(LogicalOperator::Or, &operands);
        assert!(or_result.is_ok());
        assert!(or_result.unwrap());
        
        let not_operands = vec![&RegisterValue::Boolean(true)];
        let not_result = executor.perform_logical_operation(LogicalOperator::Not, &not_operands);
        assert!(not_result.is_ok());
        assert!(!not_result.unwrap());
    }
    
    #[test]
    fn test_execution_state_advancement() {
        let mut executor = IRExecutor::new();
        let plan = create_test_execution_plan();
        
        executor.initialize_execution(&plan).unwrap();
        
        assert_eq!(executor.execution_state.current_block, 0);
        assert_eq!(executor.execution_state.instruction_index, 0);
        assert_eq!(executor.execution_state.execution_step, 0);
        
        // Execute one step
        let instruction = executor.fetch_next_instruction(&plan).unwrap();
        let instruction_clone = instruction.clone();
        executor.execute_instruction(&instruction_clone).unwrap();
        executor.advance_execution_state(&plan).unwrap();
        
        assert_eq!(executor.execution_state.execution_step, 1);
        assert!(executor.execution_state.is_terminated); // Should terminate on return
    }
    
    #[test]
    fn test_invalid_register_access() {
        let executor = IRExecutor::new();
        
        let result = executor.register_file.get_register(999);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_type_mismatch_error() {
        let executor = IRExecutor::new();
        
        let result = executor.perform_comparison(
            &RegisterValue::String("test".to_string()),
            ComparisonOp::LessThan,
            &RegisterValue::Number(42.0),
        );
        
        assert!(result.is_err());
    }
    
    // ===== Parallelism Integration Tests =====
    
    #[cfg(feature = "phase2-implementation")]
    #[test]
    fn test_ir_executor_with_parallelism_enabled() {
        let executor = IRExecutor::new().with_parallelism();
        assert!(executor.is_parallelism_enabled());
    }
    
    #[cfg(not(feature = "phase2-implementation"))]
    #[test]
    fn test_ir_executor_parallelism_feature_disabled() {
        let executor = IRExecutor::new();
        assert!(!executor.is_parallelism_enabled());
    }
    
    #[test]
    fn test_ir_executor_parallelism_disabled_by_default() {
        let executor = IRExecutor::new();
        assert!(!executor.is_parallelism_enabled());
    }
    
    #[cfg(feature = "phase2-implementation")]
    #[test]
    fn test_replay_mode_prevents_parallel_execution() {
        let mut executor = IRExecutor::new().with_parallelism();
        let plan = create_test_execution_plan();
        
        // Enable replay recording to simulate replay mode
        executor.replay_recorder.enable_recording();
        executor.replay_recorder.initialize("test_fingerprint".to_string());
        
        // Should not use parallel execution due to replay mode
        let should_parallel = executor.should_use_parallel_execution(&plan);
        assert!(should_parallel.is_ok());
        assert!(!should_parallel.unwrap());
    }
    
    #[cfg(feature = "phase2-implementation")]
    #[test]
    fn test_parallel_execution_integration() {
        let mut executor = IRExecutor::new().with_parallelism();
        let plan = create_test_execution_plan();
        
        // Execute with parallelism enabled (should fall back to sequential for now)
        let result = executor.execute(plan);
        assert!(result.is_ok());
        
        let execution_result = result.unwrap();
        match execution_result.value {
            RegisterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_sequential_execution_unchanged() {
        let mut executor = IRExecutor::new();
        let plan = create_test_execution_plan();
        
        // Sequential execution should work exactly as before
        let result = executor.execute(plan);
        assert!(result.is_ok());
        
        let execution_result = result.unwrap();
        match execution_result.value {
            RegisterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
        
        assert_eq!(execution_result.execution_steps, 1);
    }
}
