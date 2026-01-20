//! Loop Engine Module - D3 Loop Support (Phase 1 - SEMANTIC CORE LOCKED)
//!
//! This module provides the core loop execution engine for the AykenOS Semantic CLI.
//! It implements bounded, deterministic iteration with constitutional guarantees.
//!
//! # 🔒 SEMANTIC CORE LOCK (Phase 0.5 Final)
//!
//! The following files are LOCKED and cannot have semantic changes:
//! - accumulator.rs - Accumulator state transition model
//! - state.rs - Loop state management and exactness guarantees  
//! - errors.rs - Error taxonomy and recovery policies
//!
//! These implement the constitutional decisions and changing them would break:
//! - Property-based test semantics
//! - Fingerprint cache behavior
//! - Retry/partial result policies
//! - JIT compilation assumptions
//!
//! # Phase 1 Status
//!
//! - ✅ LOCKED: Data structures and semantic core
//! - ✅ LOCKED: Type safety validation
//! - ✅ Phase 2.1: Loop execution logic (executor.rs)
//! - ✅ Phase 2.2: Budget timeout enforcement (executor.rs)
//! - ✅ Phase 2.3: Break/continue control flow (executor.rs)
//! - ✅ Phase 2.4: While loop condition evaluation (executor.rs)
//! - ✅ Phase 3.1: Collection determinism support (executor.rs)
//! - ✅ Phase 4.1-4.3: Safety analysis system (safety_analyzer.rs)
//! - ✅ Phase 5.1: Loop unrolling optimization (unroller.rs)
//! - ❌ TODO: Fingerprinting (fingerprint.rs)
//! - ✅ Phase 6.1: Hot loop detection and monitoring (monitoring.rs)

// Public modules (part of stable API)
pub mod state;
pub mod accumulator;
pub mod errors;
pub mod executor;
pub mod safety_analyzer;
pub mod unroller;
pub mod monitoring;
pub mod jit_integration;
pub mod fingerprint;
pub mod d2_integration;
pub mod deterministic_partitioner;
pub mod stable_index_mapping;
pub mod tests;

// Private modules with enforced boundaries
mod core;
mod control;
mod reduction;

// Re-export core types
pub use state::{LoopState, LoopContext};
pub use accumulator::{LoopAccumulator, AccumulatorPattern};
pub use reduction::{
    AccumulatorManager, AccumulatorTransition, DataFingerprint, TypeTag, TransitionType,
    DataTransformer, ValueTransformation
};
pub use errors::{
    LoopError, EnvironmentFault, LoopResult, PartialResult, TerminationReason, ControlFlowResult,
    RichLoopExecutionResult, LoopExecutionStatus, ExecutionMode
};
pub use core::{LoopBodyFn, LoopBodyResult};
pub use control::{ControlFlow, ControlFlowDecision, ControlDecision, BudgetCalculator, RangeIterator};
pub use executor::{LoopExecutor, ParallelizationDecision, IterationPartition};
pub use safety_analyzer::{
    SafetyAnalyzer, SafetyAnalysisResult, SafetyClass, SideEffect, LoopCarriedDependency, 
    LoopAnalysisContext, CacheStats, CacheMetrics, CacheConfig, CacheAlert, EvictionPolicy
};
pub use unroller::{LoopUnroller, UnrollResult, UnrollSkipReason, UnrollConfig, UnrollStats};
pub use monitoring::{
    LoopMonitor, LoopExecutionStats, HotLoopInfo, JITCompilationStatus, MonitoringConfig,
    GlobalMonitoringStats, LoopExecutionTracker, LoopExecutionResult, JITCompilationResult,
    MonitoringSummary, HOT_LOOP_THRESHOLD, LoopMonitoringAPI, PerformanceSummary, 
    LoopQueryCriteria, MetricType, LoopAlert, AlertSeverity, LoopProfilingData, 
    MemoryStats, PerformanceTrend
};
pub use jit_integration::{
    JITIntegration, JITConfig, JITCacheKey, CompiledLoopBody, NativeCode, 
    CompilationMetadata, JITStats
};
pub use fingerprint::{
    Fingerprint, ShapeFingerprint, LoopType,
    VerificationMode, VerificationResult, MismatchType,
    CanonicalEncoder, Blake3Computer, FingerprintVerifier, FingerprintCache,
    AuditTrailLogger, VerificationManager, VerificationStats
};
pub use d2_integration::D2LoopIntegration;
pub use deterministic_partitioner::{DeterministicPartitioner, PartitionerConfig, PartitionAnalysis, LoadBalanceMetrics};
pub use stable_index_mapping::{
    StableIndexMapper, StableMappingVerification, IndexMappingStrategy, 
    IndexMappingCache, IndexMappingCacheStats
};

// Re-exports from private modules (enforced boundaries)
// pub use core::*;
// pub use control::*;
pub use reduction::*;

use crate::bcib::LoopInstruction;
use crate::error::Result;

/// Loop execution engine (Phase 2.1 - Core Implementation)
pub struct LoopEngine {
    executor: LoopExecutor,
    safety_analyzer: SafetyAnalyzer,
    unroller: LoopUnroller,
    monitor: LoopMonitor,
    jit_integration: JITIntegration, // Phase 6.2: D1 JIT Integration
    d2_integration: D2LoopIntegration, // Phase 7: D2 Parallelism Integration
}

impl LoopEngine {
    /// Create a new loop engine
    pub fn new() -> Self {
        Self {
            executor: LoopExecutor::new(),
            safety_analyzer: SafetyAnalyzer::new(),
            unroller: LoopUnroller::new(),
            monitor: LoopMonitor::new(),
            jit_integration: JITIntegration::new(), // Phase 6.2: Initialize JIT integration
            d2_integration: D2LoopIntegration::new(), // Phase 7: Initialize D2 integration
        }
    }

    /// Execute a loop instruction (Phase 2.1 - Implemented)
    pub fn execute_loop(&mut self, instruction: &LoopInstruction, body_fn: LoopBodyFn) -> Result<RichLoopExecutionResult> {
        // Phase 6.1: Start monitoring
        let loop_id = match instruction {
            LoopInstruction::While { id, .. } => id,
            LoopInstruction::For { id, .. } => id,
            LoopInstruction::ForEach { id, .. } => id,
        };
        let tracker = self.monitor.record_loop_start(loop_id, instruction);
        
        // Execute the loop
        let result = self.executor.execute_loop(instruction, body_fn);
        
        // Phase 6.1: Record monitoring results
        match &result {
            Ok(loop_result) => {
                let execution_result = if loop_result.is_success() {
                    monitoring::LoopExecutionResult::Success
                } else if loop_result.is_break() {
                    monitoring::LoopExecutionResult::Break
                } else if loop_result.is_error() {
                    // Determine error type from loop result
                    match loop_result {
                        crate::loop_engine::LoopResult::Error(error) => {
                            match error {
                                LoopError::IterationLimitExceeded { .. } => monitoring::LoopExecutionResult::IterationLimitExceeded,
                                LoopError::BudgetTimeoutExceeded { .. } => monitoring::LoopExecutionResult::BudgetTimeoutExceeded,
                                _ => monitoring::LoopExecutionResult::Error(error.to_string()),
                            }
                        }
                        _ => monitoring::LoopExecutionResult::Error("Unknown error".to_string()),
                    }
                } else {
                    monitoring::LoopExecutionResult::Success
                };
                
                // Record completion with iteration count
                let iterations_completed = loop_result.get_iterations_completed();
                if let Err(monitor_error) = self.monitor.record_loop_completion(tracker, iterations_completed, execution_result) {
                    // Log monitoring error but don't fail the loop execution
                    eprintln!("Warning: Failed to record loop monitoring data: {}", monitor_error);
                }
                
                // Check if this loop became hot and trigger JIT compilation if needed
                if self.monitor.is_hot_loop(loop_id) {
                    let hot_loop_info = self.monitor.get_hot_loop_info(loop_id);
                    if let Some(info) = hot_loop_info {
                        if info.jit_triggered && info.jit_status == monitoring::JITCompilationStatus::Compiling {
                            // Create a basic LoopContext for JIT compilation
                            let loop_config = match instruction {
                                LoopInstruction::While { config, .. } => config,
                                LoopInstruction::For { config, .. } => config,
                                LoopInstruction::ForEach { config, .. } => config,
                            };
                            let loop_context = crate::loop_engine::state::LoopContext::new(
                                loop_id.clone(),
                                loop_config,
                                "jit_body".to_string(), // Placeholder body
                            );
                            
                            // Trigger actual JIT compilation
                            match self.jit_integration.compile_loop_body(instruction, &loop_context) {
                                Ok(jit_result) => {
                                    // Update status to successful
                                    if let Err(update_error) = self.monitor.record_jit_compilation_result(loop_id, jit_result) {
                                        eprintln!("Warning: Failed to update JIT compilation status: {}", update_error);
                                    }
                                }
                                Err(jit_error) => {
                                    eprintln!("Warning: JIT compilation failed: {}", jit_error);
                                    // Update status to failed
                                    let failure_result = monitoring::JITCompilationResult::Failure {
                                        reason: jit_error.to_string(),
                                    };
                                    if let Err(update_error) = self.monitor.record_jit_compilation_result(loop_id, failure_result) {
                                        eprintln!("Warning: Failed to update JIT compilation status: {}", update_error);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Loop execution failed - record with 0 iterations
                if let Err(monitor_error) = self.monitor.record_loop_completion(
                    tracker, 
                    0, 
                    monitoring::LoopExecutionResult::Error("Execution failed".to_string())
                ) {
                    eprintln!("Warning: Failed to record loop monitoring data: {}", monitor_error);
                }
            }
        }
        
        // Return rich execution result (Phase 6.2 - JIT Integration)
        match result {
            Ok(loop_result) => {
                // Convert LoopResult to RichLoopExecutionResult with execution mode
                let execution_mode = ExecutionMode::Interpreted; // TODO: Detect JIT mode
                Ok(RichLoopExecutionResult::from_loop_result(loop_result, execution_mode))
            }
            Err(e) => Err(e),
        }
    }

    /// Analyze loop body safety for parallelization (Phase 4.1 - Safety Analysis)
    pub fn analyze_loop_safety(
        &mut self,
        loop_body: &str,
        context: &LoopAnalysisContext,
    ) -> Result<SafetyAnalysisResult> {
        self.safety_analyzer.analyze_loop_safety(loop_body, context)
    }

    /// Get safety analyzer cache statistics
    pub fn get_safety_cache_stats(&self) -> crate::loop_engine::safety_analyzer::CacheStats {
        self.safety_analyzer.cache_stats()
    }

    /// Clear safety analysis cache
    pub fn clear_safety_cache(&mut self) {
        self.safety_analyzer.clear_cache();
    }

    /// Analyze a loop for unrolling optimization (Phase 5.1 - Loop Unrolling)
    pub fn analyze_loop_unrolling(&mut self, instruction: &LoopInstruction) -> Result<UnrollResult> {
        self.unroller.analyze_loop(instruction)
    }

    /// Check if a loop should be unrolled (Phase 5.1 - Loop Unrolling)
    pub fn should_unroll_loop(&self, instruction: &LoopInstruction) -> Result<bool> {
        self.unroller.should_unroll(instruction)
    }

    /// Get loop unrolling statistics
    pub fn get_unroll_stats(&self) -> &UnrollStats {
        self.unroller.get_stats()
    }

    /// Reset loop unrolling statistics
    pub fn reset_unroll_stats(&mut self) {
        self.unroller.reset_stats();
    }

    // Phase 6.1: Hot Loop Detection and Monitoring Methods

    /// Check if a loop is considered hot (Phase 6.1)
    pub fn is_hot_loop(&self, loop_id: &crate::bcib::LoopID) -> bool {
        self.monitor.is_hot_loop(loop_id)
    }

    /// Get loop execution statistics (Phase 6.1)
    pub fn get_loop_stats(&self, loop_id: &crate::bcib::LoopID) -> Option<&LoopExecutionStats> {
        self.monitor.get_loop_stats(loop_id)
    }

    /// Get hot loop information (Phase 6.1)
    pub fn get_hot_loop_info(&self, loop_id: &crate::bcib::LoopID) -> Option<&HotLoopInfo> {
        self.monitor.get_hot_loop_info(loop_id)
    }

    /// Get all hot loops (Phase 6.1)
    pub fn get_all_hot_loops(&self) -> Vec<&HotLoopInfo> {
        self.monitor.get_all_hot_loops()
    }

    /// Get global monitoring statistics (Phase 6.1)
    pub fn get_global_monitoring_stats(&self) -> &GlobalMonitoringStats {
        self.monitor.get_global_stats()
    }

    /// Get monitoring summary (Phase 6.1)
    pub fn get_monitoring_summary(&self) -> MonitoringSummary {
        self.monitor.get_monitoring_summary()
    }

    /// Update monitoring configuration (Phase 6.1)
    pub fn update_monitoring_config(&mut self, config: MonitoringConfig) {
        self.monitor.update_config(config);
    }

    /// Trigger JIT compilation for a hot loop (Phase 6.1)
    pub fn trigger_jit_compilation(&mut self, loop_id: &crate::bcib::LoopID) -> Result<()> {
        self.monitor.trigger_jit_compilation(loop_id)
    }

    /// Record JIT compilation result (Phase 6.1) - Semantic Model
    pub fn record_jit_compilation_result(
        &mut self,
        loop_id: &crate::bcib::LoopID,
        result: JITCompilationResult,
    ) -> Result<()> {
        self.monitor.record_jit_compilation_result(loop_id, result)
    }

    /// Clear all monitoring data (Phase 6.1)
    pub fn clear_monitoring_data(&mut self) {
        self.monitor.clear_monitoring_data();
    }

    // Phase 6.2: D1 JIT Integration Methods

    /// Compile a hot loop body using D1 JIT pipeline (Phase 6.2)
    pub fn compile_hot_loop_body(
        &mut self,
        instruction: &LoopInstruction,
        loop_context: &LoopContext,
    ) -> Result<JITCompilationResult> {
        self.jit_integration.compile_loop_body(instruction, loop_context)
    }

    /// Check if a loop is eligible for JIT compilation (Phase 6.2)
    pub fn is_jit_eligible(&self, instruction: &LoopInstruction) -> bool {
        self.jit_integration.is_jit_eligible(instruction)
    }

    /// Get JIT compilation statistics (Phase 6.2)
    pub fn get_jit_stats(&self) -> JITStats {
        self.jit_integration.get_stats()
    }

    /// Get JIT configuration (Phase 6.2)
    pub fn get_jit_config(&self) -> &JITConfig {
        self.jit_integration.get_config()
    }

    /// Update JIT configuration (Phase 6.2)
    pub fn update_jit_config(&mut self, config: JITConfig) {
        self.jit_integration.update_config(config);
    }

    /// Clear JIT cache (Phase 6.2)
    pub fn clear_jit_cache(&mut self) {
        self.jit_integration.clear_cache();
    }

    /// Determine if a loop should be parallelized (Phase 7.1 - Parallelization trigger logic)
    /// 
    /// This method implements the parallelization decision system that determines when loops
    /// are eligible for parallel execution based on safety analysis and loop type constraints.
    /// 
    /// Requirements 7.1: Only parallelize Safe loop bodies, exclude While loops,
    /// support For and ForEach loops with statically known iteration counts,
    /// fall back to sequential execution for Unsafe loops.
    pub fn should_parallelize_loop(
        &self,
        instruction: &LoopInstruction,
        safety_result: &SafetyAnalysisResult,
    ) -> ParallelizationDecision {
        self.d2_integration.should_parallelize_loop(instruction, safety_result)
    }

    /// Get static iteration count for a loop (Phase 7.1)
    /// 
    /// This is a convenience method that exposes the static iteration count analysis
    /// for external use (e.g., by optimization systems or monitoring).
    pub fn get_static_iteration_count(&self, instruction: &LoopInstruction) -> Option<u32> {
        self.d2_integration.get_static_iteration_count(instruction)
    }

    /// Partition iterations deterministically for parallel execution (Phase 7.2)
    /// 
    /// Requirements 7.5, 15.1, 15.3: Partition iterations based on iteration count only,
    /// use fixed chunk size algorithm, treat available parallelism as upper bound.
    /// 
    /// This method implements the constitutional requirement for deterministic partitioning:
    /// - Same iteration count → same partitions (always)
    /// - Available parallelism (core count) is optimization hint, not semantic input
    /// - Fixed chunk size algorithm ensures reproducible partition boundaries
    /// 
    /// # Arguments
    /// 
    /// * `total_iterations` - Total number of iterations to partition
    /// * `available_parallelism` - Available parallel workers (upper bound only)
    /// 
    /// # Returns
    /// 
    /// Vector of `IterationPartition` structs defining deterministic partition boundaries
    pub fn partition_iterations_deterministic(
        &self,
        total_iterations: u32,
        available_parallelism: usize,
    ) -> Vec<IterationPartition> {
        self.d2_integration.partition_iterations_deterministic(total_iterations, available_parallelism)
    }

    /// Execute loop with deterministic parallel partitioning (Phase 7.2)
    /// 
    /// This method implements the complete parallel loop execution workflow:
    /// 1. Partition iterations deterministically
    /// 2. Execute partitions in parallel using D2 system
    /// 3. Collect results in deterministic order
    /// 
    /// Requirements 7.5, 15.2, 15.6: Deterministic partitioning, stable index mapping,
    /// deterministic result collection order
    pub fn execute_loop_parallel(
        &mut self,
        instruction: &LoopInstruction,
        body_fn: LoopBodyFn,
        iteration_count: u32,
        available_parallelism: usize,
    ) -> Result<RichLoopExecutionResult> {
        // Execute with deterministic partitioning using D2 integration
        let result = self.d2_integration.execute_loop_parallel(instruction, body_fn, iteration_count, available_parallelism);
        
        // Convert to rich execution result
        match result {
            Ok(loop_result) => {
                let execution_mode = ExecutionMode::Parallel; // Mark as parallel execution
                Ok(RichLoopExecutionResult::from_loop_result(loop_result, execution_mode))
            }
            Err(e) => Err(e),
        }
    }

    /// Trigger JIT compilation for a hot loop with integrated workflow (Phase 6.2)
    /// 
    /// This method integrates hot loop detection with JIT compilation:
    /// 1. Check if loop is hot (using monitoring system)
    /// 2. Check if loop is JIT eligible
    /// 3. Compile loop body using D1 JIT pipeline
    /// 4. Record compilation result in monitoring system
    pub fn trigger_integrated_jit_compilation(&mut self, loop_id: &crate::bcib::LoopID, instruction: &LoopInstruction) -> Result<()> {
        // 1. Check if loop is hot
        if !self.monitor.is_hot_loop(loop_id) {
            return Err(crate::error::SemanticCLIError::execution_error(
                "Loop is not hot - JIT compilation not triggered",
                crate::error::ErrorCode::E400,
            ));
        }

        // 2. Check if loop is JIT eligible
        if !self.jit_integration.is_jit_eligible(instruction) {
            return Err(crate::error::SemanticCLIError::execution_error(
                "Loop is not eligible for JIT compilation",
                crate::error::ErrorCode::E400,
            ));
        }

        // 3. Create loop context for compilation
        let loop_context = self.create_loop_context_for_jit(instruction)?;

        // 4. Compile loop body using D1 JIT pipeline
        let compilation_result = self.jit_integration.compile_loop_body(instruction, &loop_context)?;

        // 5. Record compilation result in monitoring system
        self.monitor.record_jit_compilation_result(loop_id, compilation_result)?;

        Ok(())
    }

    /// Create loop context for JIT compilation
    fn create_loop_context_for_jit(&self, instruction: &LoopInstruction) -> Result<LoopContext> {
        let config = instruction.get_config();
        let loop_id = instruction.get_loop_id().clone();
        let loop_body = format!("jit-loop-body-{}", loop_id.0);

        Ok(LoopContext::new(loop_id, config, loop_body))
    }
}

// Extension trait for LoopInstruction to extract data (needed for JIT integration)
trait LoopInstructionExt {
    fn get_config(&self) -> &crate::bcib::LoopConfig;
    fn get_loop_id(&self) -> &crate::bcib::LoopID;
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
}

impl Default for LoopEngine {
    fn default() -> Self {
        Self::new()
    }
}