//! Replay System Implementation
//! 
//! **Created By:** Kenan AY
//! **Date:** 16 Ocak 2026
//! **Architectural Reference:** C4 Section "Replay System Integration"
//! 
//! Step-by-step execution recording and replay for deterministic debugging.

use super::{ExecutionState, RegisterId};
use super::register_file::{RegisterFile, RegisterValue};
use crate::context::ContextData;
use std::collections::HashMap;

/// Replay recorder for capturing execution steps
#[derive(Debug, Clone)]
pub struct ReplayRecorder {
    /// Whether recording is enabled
    recording_enabled: bool,
    /// Execution plan fingerprint
    execution_plan_fingerprint: String,
    /// Recorded execution steps
    steps: Vec<ReplayStep>,
    /// Context snapshots for deterministic replay
    context_snapshots: HashMap<String, ContextData>,
}

impl ReplayRecorder {
    /// Create new replay recorder
    pub fn new() -> Self {
        Self {
            recording_enabled: false,
            execution_plan_fingerprint: String::new(),
            steps: Vec::new(),
            context_snapshots: HashMap::new(),
        }
    }
    
    /// Enable recording
    pub fn enable_recording(&mut self) {
        self.recording_enabled = true;
    }
    
    /// Disable recording
    pub fn disable_recording(&mut self) {
        self.recording_enabled = false;
    }
    
    /// Initialize recording for execution plan
    pub fn initialize(&mut self, fingerprint: String) {
        self.execution_plan_fingerprint = fingerprint;
        self.steps.clear();
        self.context_snapshots.clear();
    }
    
    /// Set context snapshots for deterministic replay
    pub fn set_context_snapshots(&mut self, snapshots: HashMap<String, ContextData>) {
        self.context_snapshots = snapshots;
    }
    
    /// Record execution step
    pub fn record_step(&mut self, execution_state: &ExecutionState, register_file: &RegisterFile) -> Result<(), ReplayError> {
        if !self.recording_enabled {
            return Ok(());
        }
        
        let step = ReplayStep {
            step_number: execution_state.execution_step,
            execution_state: execution_state.clone(),
            register_state: register_file.clone_state(),
        };
        
        self.steps.push(step);
        
        Ok(())
    }
    
    /// Record error during execution
    pub fn record_error(&mut self, _error: &super::ExecutionError) {
        // Error recording implementation
        // For Gate C, we keep it simple
    }
    
    /// Finalize recording and create replay trace
    pub fn finalize_trace(&self) -> Result<ReplayTrace, ReplayError> {
        if !self.recording_enabled {
            return Err(ReplayError::RecordingNotEnabled);
        }
        
        Ok(ReplayTrace {
            execution_plan_fingerprint: self.execution_plan_fingerprint.clone(),
            steps: self.steps.clone(),
            context_snapshots: self.context_snapshots.clone(),
            total_steps: self.steps.len(),
        })
    }
    
    /// Get current step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
    
    /// Check if recording is enabled
    pub fn is_recording(&self) -> bool {
        self.recording_enabled
    }
    
    /// Clear all recorded data for pooling reuse
    /// 
    /// **Constitutional Rule:** Must clear all state to prevent cross-run leakage
    pub fn clear(&mut self) {
        self.recording_enabled = false;
        self.execution_plan_fingerprint.clear();
        self.steps.clear();
        self.context_snapshots.clear();
    }
}

impl Default for ReplayRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete replay trace for execution
/// 
/// **Architectural Reference:** C4 Section "Step-by-Step Replay"
#[derive(Debug, Clone)]
pub struct ReplayTrace {
    /// Execution plan fingerprint for validation
    pub execution_plan_fingerprint: String,
    /// All execution steps
    pub steps: Vec<ReplayStep>,
    /// Context snapshots for deterministic replay
    pub context_snapshots: HashMap<String, ContextData>,
    /// Total number of steps
    pub total_steps: usize,
}

impl ReplayTrace {
    /// Create new replay trace
    pub fn new(fingerprint: String) -> Self {
        Self {
            execution_plan_fingerprint: fingerprint,
            steps: Vec::new(),
            context_snapshots: HashMap::new(),
            total_steps: 0,
        }
    }
    
    /// Validate trace integrity
    pub fn validate(&self) -> Result<(), ReplayError> {
        // Check fingerprint is present
        if self.execution_plan_fingerprint.is_empty() {
            return Err(ReplayError::InvalidTrace { 
                reason: "Missing execution plan fingerprint".to_string() 
            });
        }
        
        // Check step sequence
        for (i, step) in self.steps.iter().enumerate() {
            if step.step_number != (i as u64) {
                return Err(ReplayError::InvalidTrace { 
                    reason: format!("Step sequence mismatch at index {}: expected {}, got {}", 
                        i, i, step.step_number) 
                });
            }
        }
        
        // Check total steps matches
        if self.total_steps != self.steps.len() {
            return Err(ReplayError::InvalidTrace { 
                reason: format!("Total steps mismatch: expected {}, got {}", 
                    self.total_steps, self.steps.len()) 
            });
        }
        
        Ok(())
    }
    
    /// Get step by number
    pub fn get_step(&self, step_number: u64) -> Option<&ReplayStep> {
        self.steps.get(step_number as usize)
    }
    
    /// Get execution summary
    pub fn get_summary(&self) -> ReplayTraceSummary {
        let mut register_usage = HashMap::new();
        let mut max_registers = 0;
        
        for step in &self.steps {
            let register_count = step.register_state.len();
            if register_count > max_registers {
                max_registers = register_count;
            }
            
            for (_register_id, value) in &step.register_state {
                let value_type = value.value_type();
                *register_usage.entry(value_type).or_insert(0) += 1;
            }
        }
        
        ReplayTraceSummary {
            total_steps: self.total_steps,
            max_registers_used: max_registers,
            context_count: self.context_snapshots.len(),
            register_type_usage: register_usage,
        }
    }
    
    /// Serialize trace to JSON
    pub fn to_json(&self) -> Result<String, ReplayError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ReplayError::SerializationFailed { 
                reason: e.to_string() 
            })
    }
    
    /// Deserialize trace from JSON
    pub fn from_json(json: &str) -> Result<Self, ReplayError> {
        let trace: ReplayTrace = serde_json::from_str(json)
            .map_err(|e| ReplayError::SerializationFailed { 
                reason: e.to_string() 
            })?;
        
        trace.validate()?;
        Ok(trace)
    }
}

/// Single execution step in replay trace
#[derive(Debug, Clone)]
pub struct ReplayStep {
    /// Step number in execution sequence
    pub step_number: u64,
    /// Execution state at this step
    pub execution_state: ExecutionState,
    /// Register file state at this step
    pub register_state: HashMap<RegisterId, RegisterValue>,
}

impl ReplayStep {
    /// Create new replay step
    pub fn new(step_number: u64, execution_state: ExecutionState, register_state: HashMap<RegisterId, RegisterValue>) -> Self {
        Self {
            step_number,
            execution_state,
            register_state,
        }
    }
    
    /// Get register value at this step
    pub fn get_register_value(&self, register_id: RegisterId) -> Option<&RegisterValue> {
        self.register_state.get(&register_id)
    }
    
    /// Get all defined registers at this step
    pub fn defined_registers(&self) -> Vec<RegisterId> {
        self.register_state.keys().copied().collect()
    }
}

/// Replay trace summary for analysis
#[derive(Debug, Clone)]
pub struct ReplayTraceSummary {
    pub total_steps: usize,
    pub max_registers_used: usize,
    pub context_count: usize,
    pub register_type_usage: HashMap<super::register_file::ValueType, usize>,
}

/// Replay system for deterministic execution replay
pub struct ReplaySystem {
    /// Current replay trace
    current_trace: Option<ReplayTrace>,
    /// Replay position
    replay_position: usize,
}

impl ReplaySystem {
    /// Create new replay system
    pub fn new() -> Self {
        Self {
            current_trace: None,
            replay_position: 0,
        }
    }
    
    /// Load replay trace
    pub fn load_trace(&mut self, trace: ReplayTrace) -> Result<(), ReplayError> {
        trace.validate()?;
        self.current_trace = Some(trace);
        self.replay_position = 0;
        Ok(())
    }
    
    /// Get next replay step
    pub fn next_step(&mut self) -> Option<&ReplayStep> {
        if let Some(trace) = &self.current_trace {
            if self.replay_position < trace.steps.len() {
                let step = &trace.steps[self.replay_position];
                self.replay_position += 1;
                Some(step)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Reset replay to beginning
    pub fn reset(&mut self) {
        self.replay_position = 0;
    }
    
    /// Check if replay is complete
    pub fn is_complete(&self) -> bool {
        if let Some(trace) = &self.current_trace {
            self.replay_position >= trace.steps.len()
        } else {
            true
        }
    }
    
    /// Get current replay position
    pub fn current_position(&self) -> usize {
        self.replay_position
    }
    
    /// Get total steps in current trace
    pub fn total_steps(&self) -> usize {
        if let Some(trace) = &self.current_trace {
            trace.steps.len()
        } else {
            0
        }
    }
    
    /// Seek to specific step
    pub fn seek_to_step(&mut self, step_number: usize) -> Result<(), ReplayError> {
        if let Some(trace) = &self.current_trace {
            if step_number <= trace.steps.len() {
                self.replay_position = step_number;
                Ok(())
            } else {
                Err(ReplayError::InvalidSeekPosition { 
                    position: step_number,
                    max_position: trace.steps.len(),
                })
            }
        } else {
            Err(ReplayError::NoTraceLoaded)
        }
    }
}

impl Default for ReplaySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Replay errors
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("Recording not enabled")]
    RecordingNotEnabled,
    
    #[error("Invalid replay trace: {reason}")]
    InvalidTrace { reason: String },
    
    #[error("Serialization failed: {reason}")]
    SerializationFailed { reason: String },
    
    #[error("No trace loaded")]
    NoTraceLoaded,
    
    #[error("Invalid seek position {position}, max position is {max_position}")]
    InvalidSeekPosition { position: usize, max_position: usize },
    
    #[error("Replay step mismatch at step {step}: expected {expected:?}, got {actual:?}")]
    StepMismatch { 
        step: u64, 
        expected: HashMap<RegisterId, RegisterValue>, 
        actual: HashMap<RegisterId, RegisterValue> 
    },
    
    #[error("Replay operation failed: {reason}")]
    OperationFailed { reason: String },
}

// Implement serde traits for serialization (simplified for Gate C)
impl serde::Serialize for ReplayTrace {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ReplayTrace", 4)?;
        state.serialize_field("execution_plan_fingerprint", &self.execution_plan_fingerprint)?;
        state.serialize_field("total_steps", &self.total_steps)?;
        state.serialize_field("context_count", &self.context_snapshots.len())?;
        state.serialize_field("step_count", &self.steps.len())?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ReplayTrace {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Simplified deserialization for Gate C
        // Full implementation would deserialize all fields
        Ok(ReplayTrace::new("deserialized".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[cfg(feature = "phase2-implementation")]
    use crate::ir_planner::{IRExecutor, RegisterValue};
    
    #[cfg(feature = "phase2-implementation")]
    fn create_test_execution_plan() -> crate::execution_plan::ExecutionPlan {
        use crate::execution_plan::{ExecutionPlan, IRBlock, IRInstruction, BlockTerminator, ExecutionMetadata, ParallelSafety};
        use crate::execution_plan::dataflow::DataflowGraph;
        use crate::normalizer::RegisterAllocation;
        use crate::bcib::Value;
        use std::collections::HashMap;
        
        let block = IRBlock::with_safety(
            0,
            vec![
                IRInstruction::LoadLiteral {
                    value: Value::String("test".to_string()),
                    target_register: 0,
                },
            ],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Safe,
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
    fn test_replay_recorder_creation() {
        let recorder = ReplayRecorder::new();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.step_count(), 0);
    }
    
    #[test]
    fn test_replay_recording() {
        let mut recorder = ReplayRecorder::new();
        recorder.enable_recording();
        recorder.initialize("test_fingerprint".to_string());
        
        assert!(recorder.is_recording());
        
        let execution_state = ExecutionState::new();
        let register_file = RegisterFile::new();
        
        let result = recorder.record_step(&execution_state, &register_file);
        assert!(result.is_ok());
        assert_eq!(recorder.step_count(), 1);
    }
    
    #[test]
    fn test_replay_trace_creation() {
        let trace = ReplayTrace::new("test_fingerprint".to_string());
        assert_eq!(trace.execution_plan_fingerprint, "test_fingerprint");
        assert_eq!(trace.steps.len(), 0);
        assert_eq!(trace.total_steps, 0);
    }
    
    #[test]
    fn test_replay_trace_validation() {
        let mut trace = ReplayTrace::new("test_fingerprint".to_string());
        
        // Valid trace
        let result = trace.validate();
        assert!(result.is_ok());
        
        // Invalid trace - empty fingerprint
        trace.execution_plan_fingerprint = String::new();
        let result = trace.validate();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_replay_step_creation() {
        let execution_state = ExecutionState::new();
        let register_state = HashMap::new();
        
        let step = ReplayStep::new(0, execution_state, register_state);
        assert_eq!(step.step_number, 0);
        assert_eq!(step.defined_registers().len(), 0);
    }
    
    #[test]
    fn test_replay_system() {
        let mut replay_system = ReplaySystem::new();
        
        // No trace loaded initially
        assert!(replay_system.is_complete());
        assert_eq!(replay_system.total_steps(), 0);
        
        // Load trace
        let trace = ReplayTrace::new("test".to_string());
        let result = replay_system.load_trace(trace);
        assert!(result.is_ok());
        
        assert_eq!(replay_system.current_position(), 0);
    }
    
    #[test]
    fn test_replay_system_seeking() {
        let mut replay_system = ReplaySystem::new();
        let trace = ReplayTrace::new("test".to_string());
        replay_system.load_trace(trace).unwrap();
        
        // Valid seek
        let result = replay_system.seek_to_step(0);
        assert!(result.is_ok());
        assert_eq!(replay_system.current_position(), 0);
        
        // Invalid seek
        let result = replay_system.seek_to_step(100);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_replay_trace_summary() {
        let trace = ReplayTrace::new("test".to_string());
        let summary = trace.get_summary();
        
        assert_eq!(summary.total_steps, 0);
        assert_eq!(summary.max_registers_used, 0);
        assert_eq!(summary.context_count, 0);
    }
    
    #[test]
    fn test_replay_trace_serialization() {
        let trace = ReplayTrace::new("test_fingerprint".to_string());
        
        let json_result = trace.to_json();
        assert!(json_result.is_ok());
        
        let json = json_result.unwrap();
        assert!(json.contains("test_fingerprint"));
    }
    
    // ===== Parallelism Integration Tests =====
    
    #[cfg(feature = "phase2-implementation")]
    #[test]
    fn test_ir_executor_with_parallelism() {
        let executor = IRExecutor::new().with_parallelism();
        assert!(executor.is_parallelism_enabled());
    }
    
    #[cfg(not(feature = "phase2-implementation"))]
    #[test]
    fn test_ir_executor_without_parallelism_feature() {
        let executor = IRExecutor::new();
        assert!(!executor.is_parallelism_enabled());
    }
    
    #[test]
    fn test_ir_executor_parallelism_disabled_by_default() {
        let executor = IRExecutor::new();
        // Without calling with_parallelism(), parallelism should be disabled
        assert!(!executor.is_parallelism_enabled());
    }
    
    #[cfg(feature = "phase2-implementation")]
    #[test]
    fn test_replay_mode_forces_sequential_execution() {
        let mut executor = IRExecutor::new().with_parallelism();
        let plan = create_test_execution_plan();
        
        // Enable replay recording
        executor.replay_recorder.enable_recording();
        executor.replay_recorder.initialize("test_fingerprint".to_string());
        
        // Should use sequential execution due to replay mode
        let should_parallel = executor.should_use_parallel_execution(&plan);
        assert!(should_parallel.is_ok());
        assert!(!should_parallel.unwrap());
    }
    
    #[cfg(feature = "phase2-implementation")]
    #[test]
    fn test_parallel_execution_path_fallback() {
        let mut executor = IRExecutor::new().with_parallelism();
        let plan = create_test_execution_plan();
        
        // For now, parallel path should fall back to sequential
        let result = executor.execute_parallel_path(plan);
        assert!(result.is_ok());
        
        let execution_result = result.unwrap();
        match execution_result.value {
            RegisterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_sequential_execution_still_works() {
        let mut executor = IRExecutor::new();
        let plan = create_test_execution_plan();
        
        let result = executor.execute_sequential_path(plan);
        assert!(result.is_ok());
        
        let execution_result = result.unwrap();
        match execution_result.value {
            RegisterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }
}