//! Loop Performance Monitoring - Phase 6.1 Hot Loop Detection
//!
//! This module implements hot loop detection and performance monitoring for the
//! AykenOS Semantic CLI loop system. It tracks loop execution frequency and
//! identifies loops that are candidates for JIT compilation.
//!
//! # Hot Loop Detection
//!
//! - Tracks loop execution frequency with 1,000 iteration threshold
//! - Marks frequently executed loops as hot loop candidates
//! - Logs hot loop detection events for monitoring
//! - Integrates with D1 JIT compilation system
//!
//! # Requirements Validation
//!
//! - Requirements 6.1: Hot loop detection with 1,000 iteration threshold
//! - Requirements 9.3: Log hot loop detection events for monitoring

use crate::bcib::{LoopID, LoopInstruction};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Hot loop detection threshold (Requirements 6.1)
pub const HOT_LOOP_THRESHOLD: u32 = 1_000;

/// Monitoring API trait for exposing loop performance metrics (Requirements 9.2)
/// 
/// This trait provides a standardized interface for accessing loop performance
/// metrics, enabling integration with monitoring dashboards, alerting systems,
/// and performance analysis tools.
pub trait LoopMonitoringAPI {
    /// Get real-time metrics for an active loop
    fn get_active_loop_metrics(&self, loop_id: &LoopID) -> Option<&LoopExecutionStats>;
    
    /// Get all currently active loop IDs
    fn get_all_active_loops(&self) -> Vec<LoopID>;
    
    /// Get historical metrics for a completed loop
    fn get_completed_loop_metrics(&self, loop_id: &LoopID) -> Option<&LoopExecutionStats>;
    
    /// Get recent completed loops (up to specified count)
    fn get_recent_completed_loops(&self, count: usize) -> Vec<&LoopExecutionStats>;
    
    /// Get global monitoring statistics
    fn get_global_stats(&self) -> &GlobalMonitoringStats;
    
    /// Get performance summary for a time window
    fn get_performance_summary(&self, time_window: Duration) -> PerformanceSummary;
    
    /// Query loops by specific criteria
    fn query_loops_by_criteria(&self, criteria: &LoopQueryCriteria) -> Vec<&LoopExecutionStats>;
    
    /// Get top loops by a specific metric
    fn get_top_loops_by_metric(&self, metric: MetricType, count: usize) -> Vec<&LoopExecutionStats>;
    
    /// Get current alerts for timeout and iteration limit violations (Requirements 9.4)
    fn get_current_alerts(&self) -> Vec<LoopAlert>;
    
    /// Check if a loop has any active alerts
    fn has_alerts(&self, loop_id: &LoopID) -> bool;
}

/// Performance summary for a specific time window
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Time window for this summary
    pub time_window: Duration,
    /// Total number of loops executed
    pub total_loops: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Percentage of loops that were hot
    pub hot_loops_percentage: f64,
    /// JIT compilation success rate
    pub jit_success_rate: f64,
    /// Parallelization rate (percentage of loops parallelized)
    pub parallelization_rate: f64,
    /// Top slowest loops in this time window
    pub top_slow_loops: Vec<LoopExecutionStats>,
}

/// Query criteria for filtering loop metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopQueryCriteria {
    /// Minimum iteration count filter
    pub min_iteration_count: Option<u32>,
    /// Maximum iteration count filter
    pub max_iteration_count: Option<u32>,
    /// Minimum execution time filter (milliseconds)
    pub min_execution_time_ms: Option<u64>,
    /// Maximum execution time filter (milliseconds)
    pub max_execution_time_ms: Option<u64>,
    /// JIT compilation status filter
    pub jit_status: Option<JITCompilationStatus>,
    /// Time range filter (start, end) - not serialized due to Instant limitations
    #[serde(skip, default)]
    pub time_range: Option<(Instant, Instant)>,
}

/// Metric types for sorting and filtering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricType {
    /// Total execution time
    ExecutionTime,
    /// Total iteration count
    IterationCount,
    /// Average time per iteration
    AverageIterationTime,
    /// Number of executions
    ExecutionCount,
    /// JIT compilation time
    JITCompilationTime,
}

/// Loop monitoring alerts for timeout and iteration limit violations (Requirements 9.4)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoopAlert {
    /// Iteration limit exceeded alert
    IterationLimitViolation {
        /// Loop that violated the limit
        loop_id: LoopID,
        /// Configured iteration limit
        limit: u32,
        /// Actual iterations completed
        completed: u32,
        /// When the violation occurred (not serialized)
        #[serde(skip, default = "Instant::now")]
        timestamp: Instant,
        /// Severity level
        severity: AlertSeverity,
    },
    /// Budget timeout exceeded alert
    BudgetTimeoutViolation {
        /// Loop that violated the timeout
        loop_id: LoopID,
        /// Configured budget timeout
        budget: u64,
        /// Actual budget consumed
        consumed: u64,
        /// Iterations completed before timeout
        iterations_completed: u32,
        /// When the violation occurred (not serialized)
        #[serde(skip, default = "Instant::now")]
        timestamp: Instant,
        /// Severity level
        severity: AlertSeverity,
    },
    /// Hot loop detection alert
    HotLoopDetected {
        /// Loop that became hot
        loop_id: LoopID,
        /// Iteration count that triggered detection
        iteration_count: u32,
        /// When the loop was detected as hot (not serialized)
        #[serde(skip, default = "Instant::now")]
        timestamp: Instant,
        /// Whether JIT compilation was triggered
        jit_triggered: bool,
    },
    /// JIT compilation failure alert
    JITCompilationFailed {
        /// Loop with failed compilation
        loop_id: LoopID,
        /// Failure reason
        reason: String,
        /// When the failure occurred (not serialized)
        #[serde(skip, default = "Instant::now")]
        timestamp: Instant,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Critical alert requiring attention
    Critical,
}

/// Loop performance monitoring system
#[derive(Debug)]
pub struct LoopMonitor {
    /// Loop execution statistics
    loop_stats: HashMap<LoopID, LoopExecutionStats>,
    /// Hot loop candidates
    hot_loops: HashMap<LoopID, HotLoopInfo>,
    /// Monitoring configuration
    config: MonitoringConfig,
    /// Global monitoring statistics
    global_stats: GlobalMonitoringStats,
    /// Active alerts for violations and issues
    active_alerts: Vec<LoopAlert>,
    /// Alert history (limited to prevent memory growth)
    alert_history: Vec<LoopAlert>,
    /// Maximum number of alerts to keep in history
    max_alert_history: usize,
}

/// Statistics for a single loop execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopExecutionStats {
    /// Loop identifier
    pub loop_id: LoopID,
    /// Total number of times this loop has been executed
    pub execution_count: u64,
    /// Total iterations across all executions
    pub total_iterations: u64,
    /// Total execution time across all executions
    pub total_execution_time: Duration,
    /// Maximum iterations in a single execution
    pub max_iterations_per_execution: u32,
    /// Average iterations per execution
    pub avg_iterations_per_execution: f64,
    /// Last execution timestamp (not serialized)
    #[serde(skip, default)]
    pub last_execution: Option<Instant>,
    /// Whether this loop has been marked as hot
    pub is_hot_loop: bool,
    /// JIT compilation status
    pub jit_compilation_status: JITCompilationStatus,
    /// Per-loop profiling data (Requirements 9.5)
    pub profiling_data: LoopProfilingData,
}

/// Per-loop profiling data providing detailed performance metrics (Requirements 9.5)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopProfilingData {
    /// Average iteration time in nanoseconds
    pub average_iteration_time_ns: u64,
    /// Total execution time across all runs
    pub total_execution_time: Duration,
    /// Minimum execution time for a single run
    pub min_execution_time: Duration,
    /// Maximum execution time for a single run
    pub max_execution_time: Duration,
    /// Standard deviation of execution times
    pub execution_time_std_dev: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Performance trend over recent executions
    pub performance_trend: PerformanceTrend,
}

/// Memory usage statistics for loop execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage during loop execution (bytes)
    pub peak_memory_bytes: u64,
    /// Average memory usage (bytes)
    pub avg_memory_bytes: u64,
    /// Memory allocations during execution
    pub allocations_count: u64,
    /// Memory deallocations during execution
    pub deallocations_count: u64,
}

/// Performance trend analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceTrend {
    /// Performance is improving over time
    Improving { improvement_rate: f64 },
    /// Performance is degrading over time
    Degrading { degradation_rate: f64 },
    /// Performance is stable
    Stable,
    /// Not enough data to determine trend
    Insufficient,
}

/// Hot loop information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotLoopInfo {
    /// Loop identifier
    pub loop_id: LoopID,
    /// When this loop was first detected as hot (not serialized)
    #[serde(skip, default = "Instant::now")]
    pub detected_at: Instant,
    /// Total iterations when detected as hot
    pub detection_iteration_count: u64,
    /// Number of times this loop has exceeded the hot threshold
    pub hot_detection_count: u32,
    /// Whether JIT compilation has been triggered
    pub jit_triggered: bool,
    /// JIT compilation result
    pub jit_status: JITCompilationStatus,
}

/// JIT compilation status for loops
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JITCompilationStatus {
    /// Not eligible for JIT compilation
    NotEligible,
    /// Eligible but not yet compiled
    Eligible,
    /// JIT compilation in progress
    Compiling,
    /// Successfully compiled to native code
    Compiled {
        /// Compilation timestamp (not serialized)
        #[serde(skip, default = "Instant::now")]
        compiled_at: Instant,
        /// Compilation time
        compilation_time: Duration,
    },
    /// JIT compilation failed
    Failed {
        /// Failure timestamp (not serialized)
        #[serde(skip, default = "Instant::now")]
        failed_at: Instant,
        /// Failure reason
        reason: String,
    },
}

/// Monitoring configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Hot loop detection threshold
    pub hot_loop_threshold: u32,
    /// Whether to enable detailed performance tracking
    pub enable_detailed_tracking: bool,
    /// Whether to log hot loop detection events
    pub enable_hot_loop_logging: bool,
    /// Maximum number of loop stats to keep in memory
    pub max_loop_stats_entries: usize,
    /// Whether to automatically trigger JIT compilation for hot loops
    pub auto_trigger_jit: bool,
}

/// Global monitoring statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalMonitoringStats {
    /// Total number of loops monitored
    pub total_loops_monitored: u64,
    /// Total number of loop executions
    pub total_loop_executions: u64,
    /// Total iterations across all loops
    pub total_iterations: u64,
    /// Total execution time across all loops
    pub total_execution_time: Duration,
    /// Number of hot loops detected
    pub hot_loops_detected: u32,
    /// Number of JIT compilations triggered
    pub jit_compilations_triggered: u32,
    /// Number of successful JIT compilations
    pub successful_jit_compilations: u32,
    /// Monitoring start time (not serialized)
    #[serde(skip, default = "Instant::now")]
    pub monitoring_start_time: Instant,
}

impl LoopMonitor {
    /// Create a new loop monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    /// Create a new loop monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        Self {
            loop_stats: HashMap::new(),
            hot_loops: HashMap::new(),
            config,
            global_stats: GlobalMonitoringStats::new(),
            active_alerts: Vec::new(),
            alert_history: Vec::new(),
            max_alert_history: 1000, // Keep last 1000 alerts
        }
    }

    /// Record the start of a loop execution
    pub fn record_loop_start(&mut self, loop_id: &LoopID, _instruction: &LoopInstruction) -> LoopExecutionTracker {
        // Update global stats
        self.global_stats.total_loop_executions += 1;

        // Get or create loop stats
        let stats = self.loop_stats.entry(loop_id.clone()).or_insert_with(|| {
            self.global_stats.total_loops_monitored += 1;
            LoopExecutionStats::new(loop_id.clone())
        });

        // Update execution count
        stats.execution_count += 1;
        stats.last_execution = Some(Instant::now());

        // Create execution tracker
        LoopExecutionTracker::new(loop_id.clone(), Instant::now())
    }

    /// Record the completion of a loop execution
    pub fn record_loop_completion(
        &mut self,
        tracker: LoopExecutionTracker,
        iterations_completed: u32,
        execution_result: LoopExecutionResult,
    ) -> Result<()> {
        let execution_time = tracker.start_time.elapsed();
        let loop_id = &tracker.loop_id;

        // Handle alerting for violations (Requirements 9.4)
        self.handle_execution_result_alerting(loop_id, iterations_completed, &execution_result)?;

        // Check for hot loop detection first (before mutable borrow)
        let should_detect_hot = if let Some(stats) = self.loop_stats.get(loop_id) {
            iterations_completed >= self.config.hot_loop_threshold && !stats.is_hot_loop
        } else {
            false
        };

        // Update loop stats
        if let Some(stats) = self.loop_stats.get_mut(loop_id) {
            stats.total_iterations += iterations_completed as u64;
            stats.total_execution_time += execution_time;
            stats.max_iterations_per_execution = stats.max_iterations_per_execution.max(iterations_completed);
            stats.avg_iterations_per_execution = stats.total_iterations as f64 / stats.execution_count as f64;

            // Update profiling data (Requirements 9.5) - separate method to avoid borrow issues
            Self::update_profiling_data_static(&mut stats.profiling_data, execution_time, iterations_completed, stats.execution_count);

            // Update global stats
            self.global_stats.total_iterations += iterations_completed as u64;
            self.global_stats.total_execution_time += execution_time;

            // Check for hot loop detection (Requirements 6.1)
            if should_detect_hot {
                // Clone the loop_id to avoid borrow issues
                let loop_id_clone = loop_id.clone();
                // Mark as hot first
                stats.is_hot_loop = true;
                stats.jit_compilation_status = JITCompilationStatus::Eligible;
                
                // Now handle hot loop detection
                self.handle_hot_loop_detection(loop_id_clone, iterations_completed)?;
            }
        }

        Ok(())
    }

    /// Handle alerting for execution results (Requirements 9.4)
    fn handle_execution_result_alerting(
        &mut self,
        loop_id: &LoopID,
        iterations_completed: u32,
        execution_result: &LoopExecutionResult,
    ) -> Result<()> {
        match execution_result {
            LoopExecutionResult::IterationLimitExceeded => {
                let alert = LoopAlert::IterationLimitViolation {
                    loop_id: loop_id.clone(),
                    limit: iterations_completed, // The limit was reached exactly
                    completed: iterations_completed,
                    timestamp: Instant::now(),
                    severity: AlertSeverity::Warning,
                };
                self.add_alert(alert);
            }
            LoopExecutionResult::BudgetTimeoutExceeded => {
                // We don't have the exact budget values here, so we'll create a generic alert
                // In a real implementation, this would be passed from the execution context
                let alert = LoopAlert::BudgetTimeoutViolation {
                    loop_id: loop_id.clone(),
                    budget: 0, // Would be filled from execution context
                    consumed: 0, // Would be filled from execution context
                    iterations_completed,
                    timestamp: Instant::now(),
                    severity: AlertSeverity::Critical,
                };
                self.add_alert(alert);
            }
            LoopExecutionResult::Error(error_msg) => {
                // Check if this is a JIT compilation error
                if error_msg.contains("JIT") || error_msg.contains("compilation") {
                    let alert = LoopAlert::JITCompilationFailed {
                        loop_id: loop_id.clone(),
                        reason: error_msg.clone(),
                        timestamp: Instant::now(),
                    };
                    self.add_alert(alert);
                }
            }
            _ => {
                // No alerting needed for successful executions or breaks
            }
        }
        Ok(())
    }

    /// Update profiling data for a loop (Requirements 9.5) - static method to avoid borrow issues
    fn update_profiling_data_static(
        profiling: &mut LoopProfilingData,
        execution_time: Duration,
        iterations_completed: u32,
        execution_count: u64,
    ) {
        // Update average iteration time
        if iterations_completed > 0 {
            let iteration_time_ns = execution_time.as_nanos() / iterations_completed as u128;
            profiling.average_iteration_time_ns = iteration_time_ns as u64;
        }
        
        // Update execution time statistics
        profiling.total_execution_time += execution_time;
        
        if profiling.min_execution_time == Duration::ZERO || execution_time < profiling.min_execution_time {
            profiling.min_execution_time = execution_time;
        }
        
        if execution_time > profiling.max_execution_time {
            profiling.max_execution_time = execution_time;
        }
        
        // Update performance trend (simplified implementation)
        profiling.performance_trend = Self::calculate_performance_trend_static(execution_count, execution_time);
        
        // Update memory stats (placeholder - would integrate with actual memory tracking)
        profiling.memory_stats.peak_memory_bytes = profiling.memory_stats.peak_memory_bytes.max(1024 * iterations_completed as u64);
        profiling.memory_stats.avg_memory_bytes = profiling.memory_stats.peak_memory_bytes / 2;
        profiling.memory_stats.allocations_count += iterations_completed as u64;
    }

    /// Calculate performance trend for a loop - static version
    fn calculate_performance_trend_static(execution_count: u64, execution_time: Duration) -> PerformanceTrend {
        if execution_count < 3 {
            return PerformanceTrend::Insufficient;
        }
        
        // Simplified trend calculation based on execution time
        let time_ms = execution_time.as_millis() as f64;
        
        if time_ms < 100.0 {
            PerformanceTrend::Improving { improvement_rate: 0.1 }
        } else if time_ms > 1000.0 {
            PerformanceTrend::Degrading { degradation_rate: 0.1 }
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Add an alert to the monitoring system
    fn add_alert(&mut self, alert: LoopAlert) {
        // Add to active alerts
        self.active_alerts.push(alert.clone());
        
        // Add to history
        self.alert_history.push(alert);
        
        // Limit history size
        if self.alert_history.len() > self.max_alert_history {
            self.alert_history.remove(0);
        }
        
        // Log the alert if logging is enabled
        if self.config.enable_hot_loop_logging {
            self.log_alert(&self.alert_history.last().unwrap());
        }
    }

    /// Log an alert
    fn log_alert(&self, alert: &LoopAlert) {
        match alert {
            LoopAlert::IterationLimitViolation { loop_id, limit, completed, severity, .. } => {
                println!(
                    "[ALERT_{:?}] Loop {} exceeded iteration limit: {}/{} iterations",
                    severity, loop_id.0, completed, limit
                );
            }
            LoopAlert::BudgetTimeoutViolation { loop_id, budget, consumed, iterations_completed, severity, .. } => {
                println!(
                    "[ALERT_{:?}] Loop {} exceeded budget timeout: {}/{} budget units ({} iterations)",
                    severity, loop_id.0, consumed, budget, iterations_completed
                );
            }
            LoopAlert::HotLoopDetected { loop_id, iteration_count, jit_triggered, .. } => {
                println!(
                    "[ALERT_INFO] Hot loop detected: {} with {} iterations (JIT triggered: {})",
                    loop_id.0, iteration_count, jit_triggered
                );
            }
            LoopAlert::JITCompilationFailed { loop_id, reason, .. } => {
                println!(
                    "[ALERT_CRITICAL] JIT compilation failed for loop {}: {}",
                    loop_id.0, reason
                );
            }
        }
    }

    /// Handle hot loop detection separately to avoid borrow checker issues
    fn handle_hot_loop_detection(&mut self, loop_id: LoopID, iteration_count: u32) -> Result<()> {
        // Create hot loop info
        let hot_loop_info = HotLoopInfo {
            loop_id: loop_id.clone(),
            detected_at: Instant::now(),
            detection_iteration_count: iteration_count as u64,
            hot_detection_count: 1,
            jit_triggered: false,
            jit_status: JITCompilationStatus::Eligible,
        };

        // Register hot loop
        self.hot_loops.insert(loop_id.clone(), hot_loop_info);
        self.global_stats.hot_loops_detected += 1;

        // Log hot loop detection event (Requirements 9.3)
        if self.config.enable_hot_loop_logging {
            self.log_hot_loop_detection(&loop_id, iteration_count)?;
        }

        // Automatically trigger JIT compilation if enabled
        if self.config.auto_trigger_jit {
            // For now, just call the regular trigger method
            // The LoopEngine will need to override this behavior
            self.trigger_jit_compilation(&loop_id)?;
        }

        Ok(())
    }

    /// Trigger JIT compilation for a hot loop (Phase 6.2 Integration)
    /// 
    /// This method now serves as a bridge between monitoring and JIT integration.
    /// The actual JIT compilation is handled by the JITIntegration system.
    pub fn trigger_jit_compilation(&mut self, loop_id: &LoopID) -> Result<()> {
        if let Some(hot_loop_info) = self.hot_loops.get_mut(loop_id) {
            if !hot_loop_info.jit_triggered {
                hot_loop_info.jit_triggered = true;
                hot_loop_info.jit_status = JITCompilationStatus::Compiling;
                self.global_stats.jit_compilations_triggered += 1;

                // Update loop stats as well
                if let Some(stats) = self.loop_stats.get_mut(loop_id) {
                    stats.jit_compilation_status = JITCompilationStatus::Compiling;
                }

                // Log JIT compilation trigger
                if self.config.enable_hot_loop_logging {
                    println!(
                        "[JIT_COMPILATION_TRIGGERED] Loop {} marked for JIT compilation",
                        loop_id.0
                    );
                }

                // NOTE: Actual JIT compilation is now handled by LoopEngine.trigger_integrated_jit_compilation()
                // This method only updates monitoring state and triggers the compilation request
            }
        }

        Ok(())
    }

    /// Record JIT compilation result (semantic model)
    pub fn record_jit_compilation_result(
        &mut self,
        loop_id: &LoopID,
        result: JITCompilationResult,
    ) -> Result<()> {
        let status = match &result {
            JITCompilationResult::Success { compilation_time } => {
                self.global_stats.successful_jit_compilations += 1;
                JITCompilationStatus::Compiled {
                    compiled_at: std::time::Instant::now(),
                    compilation_time: *compilation_time,
                }
            }
            JITCompilationResult::Failure { reason } => {
                JITCompilationStatus::Failed {
                    failed_at: std::time::Instant::now(),
                    reason: reason.clone(),
                }
            }
        };

        // Update hot loop info
        if let Some(hot_loop_info) = self.hot_loops.get_mut(loop_id) {
            hot_loop_info.jit_status = status.clone();
        }

        // Update loop stats
        if let Some(stats) = self.loop_stats.get_mut(loop_id) {
            stats.jit_compilation_status = status;
        }

        // Log compilation result
        if self.config.enable_hot_loop_logging {
            match &result {
                JITCompilationResult::Success { compilation_time } => {
                    println!(
                        "[JIT_COMPILATION_SUCCESS] Loop {} compiled in {:?}",
                        loop_id.0,
                        compilation_time
                    );
                }
                JITCompilationResult::Failure { reason } => {
                    println!(
                        "[JIT_COMPILATION_FAILED] Loop {} compilation failed: {}",
                        loop_id.0,
                        reason
                    );
                }
            }
        }

        Ok(())
    }

    /// Trigger JIT compilation with integration callback
    /// This method allows the LoopEngine to provide a callback for actual JIT compilation
    pub fn trigger_jit_compilation_with_callback<F>(&mut self, loop_id: &LoopID, jit_callback: F) -> Result<()> 
    where
        F: FnOnce(&LoopID) -> Result<()>,
    {
        if let Some(hot_loop_info) = self.hot_loops.get_mut(loop_id) {
            if !hot_loop_info.jit_triggered {
                hot_loop_info.jit_triggered = true;
                hot_loop_info.jit_status = JITCompilationStatus::Compiling;
                self.global_stats.jit_compilations_triggered += 1;

                // Update loop stats as well
                if let Some(stats) = self.loop_stats.get_mut(loop_id) {
                    stats.jit_compilation_status = JITCompilationStatus::Compiling;
                }

                // Log JIT compilation trigger
                if self.config.enable_hot_loop_logging {
                    println!(
                        "[JIT_COMPILATION_TRIGGERED] Loop {} marked for JIT compilation",
                        loop_id.0
                    );
                }

                // Actually trigger JIT compilation via callback
                jit_callback(loop_id)?;
            }
        }

        Ok(())
    }

    /// Log hot loop detection event (Requirements 9.3)
    fn log_hot_loop_detection(&self, loop_id: &LoopID, iteration_count: u32) -> Result<()> {
        // In a real implementation, this would use a proper logging framework
        // For now, we'll use println! for demonstration
        println!(
            "[HOT_LOOP_DETECTED] Loop {} exceeded threshold with {} iterations (threshold: {})",
            loop_id.0,
            iteration_count,
            self.config.hot_loop_threshold
        );

        // TODO: Integrate with proper logging system
        // - Log to structured logging framework
        // - Include additional context (execution time, loop type, etc.)
        // - Support different log levels and destinations
        // - Enable log aggregation and monitoring

        Ok(())
    }

    /// Check if a loop is considered hot
    pub fn is_hot_loop(&self, loop_id: &LoopID) -> bool {
        self.hot_loops.contains_key(loop_id)
    }

    /// Get loop execution statistics
    pub fn get_loop_stats(&self, loop_id: &LoopID) -> Option<&LoopExecutionStats> {
        self.loop_stats.get(loop_id)
    }

    /// Get hot loop information
    pub fn get_hot_loop_info(&self, loop_id: &LoopID) -> Option<&HotLoopInfo> {
        self.hot_loops.get(loop_id)
    }

    /// Get all hot loops
    pub fn get_all_hot_loops(&self) -> Vec<&HotLoopInfo> {
        self.hot_loops.values().collect()
    }

    /// Get global monitoring statistics
    pub fn get_global_stats(&self) -> &GlobalMonitoringStats {
        &self.global_stats
    }

    /// Get monitoring configuration
    pub fn get_config(&self) -> &MonitoringConfig {
        &self.config
    }

    /// Update monitoring configuration
    pub fn update_config(&mut self, config: MonitoringConfig) {
        self.config = config;
    }

    /// Clear all monitoring data
    pub fn clear_monitoring_data(&mut self) {
        self.loop_stats.clear();
        self.hot_loops.clear();
        self.global_stats = GlobalMonitoringStats::new();
        self.active_alerts.clear();
        self.alert_history.clear();
    }

    /// Get monitoring summary for reporting
    pub fn get_monitoring_summary(&self) -> MonitoringSummary {
        MonitoringSummary {
            total_loops_monitored: self.global_stats.total_loops_monitored,
            total_loop_executions: self.global_stats.total_loop_executions,
            total_iterations: self.global_stats.total_iterations,
            hot_loops_detected: self.global_stats.hot_loops_detected,
            jit_compilations_triggered: self.global_stats.jit_compilations_triggered,
            successful_jit_compilations: self.global_stats.successful_jit_compilations,
            average_iterations_per_execution: if self.global_stats.total_loop_executions > 0 {
                self.global_stats.total_iterations as f64 / self.global_stats.total_loop_executions as f64
            } else {
                0.0
            },
            monitoring_duration: self.global_stats.monitoring_start_time.elapsed(),
        }
    }
}

/// Implementation of the LoopMonitoringAPI trait for LoopMonitor (Requirements 9.2)
impl LoopMonitoringAPI for LoopMonitor {
    fn get_active_loop_metrics(&self, loop_id: &LoopID) -> Option<&LoopExecutionStats> {
        // For now, we consider all loops in loop_stats as potentially active
        // In a real implementation, we'd track which loops are currently executing
        self.loop_stats.get(loop_id)
    }
    
    fn get_all_active_loops(&self) -> Vec<LoopID> {
        // Return all loop IDs that have been executed
        // In a real implementation, this would only return currently executing loops
        self.loop_stats.keys().cloned().collect()
    }
    
    fn get_completed_loop_metrics(&self, loop_id: &LoopID) -> Option<&LoopExecutionStats> {
        self.loop_stats.get(loop_id)
    }
    
    fn get_recent_completed_loops(&self, count: usize) -> Vec<&LoopExecutionStats> {
        let mut loops: Vec<&LoopExecutionStats> = self.loop_stats.values().collect();
        
        // Sort by last execution time (most recent first)
        loops.sort_by(|a, b| {
            match (a.last_execution, b.last_execution) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        
        loops.into_iter().take(count).collect()
    }
    
    fn get_global_stats(&self) -> &GlobalMonitoringStats {
        &self.global_stats
    }
    
    fn get_performance_summary(&self, time_window: Duration) -> PerformanceSummary {
        let cutoff_time = Instant::now() - time_window;
        
        // Filter loops executed within the time window
        let recent_loops: Vec<&LoopExecutionStats> = self.loop_stats.values()
            .filter(|stats| {
                stats.last_execution
                    .map(|time| time >= cutoff_time)
                    .unwrap_or(false)
            })
            .collect();
        
        let total_loops = recent_loops.len() as u64;
        let hot_loops_count = recent_loops.iter()
            .filter(|stats| stats.is_hot_loop)
            .count() as u64;
        
        let jit_success_count = recent_loops.iter()
            .filter(|stats| matches!(stats.jit_compilation_status, JITCompilationStatus::Compiled { .. }))
            .count() as u64;
        
        let jit_attempted_count = recent_loops.iter()
            .filter(|stats| !matches!(stats.jit_compilation_status, JITCompilationStatus::NotEligible))
            .count() as u64;
        
        let average_execution_time_ms = if total_loops > 0 {
            recent_loops.iter()
                .map(|stats| stats.avg_execution_time().as_millis() as f64)
                .sum::<f64>() / total_loops as f64
        } else {
            0.0
        };
        
        // Get top 5 slowest loops
        let mut top_slow_loops: Vec<LoopExecutionStats> = recent_loops.iter()
            .map(|&stats| stats.clone())
            .collect();
        top_slow_loops.sort_by(|a, b| b.avg_execution_time().cmp(&a.avg_execution_time()));
        top_slow_loops.truncate(5);
        
        PerformanceSummary {
            time_window,
            total_loops,
            average_execution_time_ms,
            hot_loops_percentage: if total_loops > 0 {
                (hot_loops_count as f64 / total_loops as f64) * 100.0
            } else {
                0.0
            },
            jit_success_rate: if jit_attempted_count > 0 {
                (jit_success_count as f64 / jit_attempted_count as f64) * 100.0
            } else {
                0.0
            },
            parallelization_rate: 0.0, // Would be calculated from execution mode data
            top_slow_loops,
        }
    }
    
    fn query_loops_by_criteria(&self, criteria: &LoopQueryCriteria) -> Vec<&LoopExecutionStats> {
        self.loop_stats.values()
            .filter(|stats| {
                // Apply iteration count filters
                if let Some(min_iterations) = criteria.min_iteration_count {
                    if stats.total_iterations < min_iterations as u64 {
                        return false;
                    }
                }
                
                if let Some(max_iterations) = criteria.max_iteration_count {
                    if stats.total_iterations > max_iterations as u64 {
                        return false;
                    }
                }
                
                // Apply execution time filters
                let avg_time_ms = stats.avg_execution_time().as_millis() as u64;
                
                if let Some(min_time) = criteria.min_execution_time_ms {
                    if avg_time_ms < min_time {
                        return false;
                    }
                }
                
                if let Some(max_time) = criteria.max_execution_time_ms {
                    if avg_time_ms > max_time {
                        return false;
                    }
                }
                
                // Apply JIT status filter
                if let Some(ref jit_status) = criteria.jit_status {
                    if std::mem::discriminant(&stats.jit_compilation_status) != std::mem::discriminant(jit_status) {
                        return false;
                    }
                }
                
                // Apply time range filter
                if let Some((start_time, end_time)) = criteria.time_range {
                    if let Some(last_exec) = stats.last_execution {
                        if last_exec < start_time || last_exec > end_time {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }
    
    fn get_top_loops_by_metric(&self, metric: MetricType, count: usize) -> Vec<&LoopExecutionStats> {
        let mut loops: Vec<&LoopExecutionStats> = self.loop_stats.values().collect();
        
        // Sort by the specified metric
        loops.sort_by(|a, b| {
            match metric {
                MetricType::ExecutionTime => b.total_execution_time.cmp(&a.total_execution_time),
                MetricType::IterationCount => b.total_iterations.cmp(&a.total_iterations),
                MetricType::AverageIterationTime => b.avg_time_per_iteration().cmp(&a.avg_time_per_iteration()),
                MetricType::ExecutionCount => b.execution_count.cmp(&a.execution_count),
                MetricType::JITCompilationTime => {
                    // Compare JIT compilation times if available
                    match (&a.jit_compilation_status, &b.jit_compilation_status) {
                        (JITCompilationStatus::Compiled { compilation_time: a_time, .. },
                         JITCompilationStatus::Compiled { compilation_time: b_time, .. }) => {
                            b_time.cmp(a_time)
                        }
                        (JITCompilationStatus::Compiled { .. }, _) => std::cmp::Ordering::Less,
                        (_, JITCompilationStatus::Compiled { .. }) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                }
            }
        });
        
        loops.into_iter().take(count).collect()
    }
    
    fn get_current_alerts(&self) -> Vec<LoopAlert> {
        self.active_alerts.clone()
    }
    
    fn has_alerts(&self, loop_id: &LoopID) -> bool {
        self.active_alerts.iter().any(|alert| {
            match alert {
                LoopAlert::IterationLimitViolation { loop_id: alert_loop_id, .. } |
                LoopAlert::BudgetTimeoutViolation { loop_id: alert_loop_id, .. } |
                LoopAlert::HotLoopDetected { loop_id: alert_loop_id, .. } |
                LoopAlert::JITCompilationFailed { loop_id: alert_loop_id, .. } => {
                    alert_loop_id == loop_id
                }
            }
        })
    }
}

/// Loop execution tracker for timing measurements
#[derive(Debug, Clone)]
pub struct LoopExecutionTracker {
    /// Loop identifier
    pub loop_id: LoopID,
    /// Execution start time
    pub start_time: Instant,
}

impl LoopExecutionTracker {
    /// Create a new execution tracker
    pub fn new(loop_id: LoopID, start_time: Instant) -> Self {
        Self { loop_id, start_time }
    }
}

/// Result of loop execution for monitoring
#[derive(Debug, Clone, PartialEq)]
pub enum LoopExecutionResult {
    /// Loop completed successfully
    Success,
    /// Loop terminated due to iteration limit
    IterationLimitExceeded,
    /// Loop terminated due to budget timeout
    BudgetTimeoutExceeded,
    /// Loop terminated due to break statement
    Break,
    /// Loop failed with error
    Error(String),
}

/// Result of JIT compilation
#[derive(Debug, Clone, PartialEq)]
pub enum JITCompilationResult {
    /// Compilation succeeded
    Success {
        /// Time taken to compile
        compilation_time: Duration,
    },
    /// Compilation failed
    Failure {
        /// Failure reason
        reason: String,
    },
}

/// Monitoring summary for reporting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoringSummary {
    /// Total number of loops monitored
    pub total_loops_monitored: u64,
    /// Total number of loop executions
    pub total_loop_executions: u64,
    /// Total iterations across all loops
    pub total_iterations: u64,
    /// Number of hot loops detected
    pub hot_loops_detected: u32,
    /// Number of JIT compilations triggered
    pub jit_compilations_triggered: u32,
    /// Number of successful JIT compilations
    pub successful_jit_compilations: u32,
    /// Average iterations per execution
    pub average_iterations_per_execution: f64,
    /// Total monitoring duration
    pub monitoring_duration: Duration,
}

impl LoopProfilingData {
    /// Create new profiling data with default values
    pub fn new() -> Self {
        Self {
            average_iteration_time_ns: 0,
            total_execution_time: Duration::ZERO,
            min_execution_time: Duration::ZERO,
            max_execution_time: Duration::ZERO,
            execution_time_std_dev: 0.0,
            memory_stats: MemoryStats::new(),
            performance_trend: PerformanceTrend::Insufficient,
        }
    }
}

impl MemoryStats {
    /// Create new memory statistics with default values
    pub fn new() -> Self {
        Self {
            peak_memory_bytes: 0,
            avg_memory_bytes: 0,
            allocations_count: 0,
            deallocations_count: 0,
        }
    }
}

impl LoopExecutionStats {
    /// Create new loop execution statistics
    pub fn new(loop_id: LoopID) -> Self {
        Self {
            loop_id,
            execution_count: 0,
            total_iterations: 0,
            total_execution_time: Duration::ZERO,
            max_iterations_per_execution: 0,
            avg_iterations_per_execution: 0.0,
            last_execution: None,
            is_hot_loop: false,
            jit_compilation_status: JITCompilationStatus::NotEligible,
            profiling_data: LoopProfilingData::new(),
        }
    }

    /// Get the average execution time per loop run
    pub fn avg_execution_time(&self) -> Duration {
        if self.execution_count > 0 {
            self.total_execution_time / self.execution_count as u32
        } else {
            Duration::ZERO
        }
    }

    /// Get the average execution time per iteration
    pub fn avg_time_per_iteration(&self) -> Duration {
        if self.total_iterations > 0 {
            self.total_execution_time / self.total_iterations as u32
        } else {
            Duration::ZERO
        }
    }
}

impl MonitoringConfig {
    /// Create default monitoring configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            hot_loop_threshold: HOT_LOOP_THRESHOLD,
            enable_detailed_tracking: false, // Minimal overhead
            enable_hot_loop_logging: true,
            max_loop_stats_entries: 1000,
            auto_trigger_jit: true,
        }
    }

    /// Create configuration optimized for debugging
    pub fn debug_optimized() -> Self {
        Self {
            hot_loop_threshold: 100, // Lower threshold for debugging
            enable_detailed_tracking: true,
            enable_hot_loop_logging: true,
            max_loop_stats_entries: 10000,
            auto_trigger_jit: false, // Manual JIT control
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            hot_loop_threshold: HOT_LOOP_THRESHOLD,
            enable_detailed_tracking: true,
            enable_hot_loop_logging: true,
            max_loop_stats_entries: 5000,
            auto_trigger_jit: true,
        }
    }
}

impl GlobalMonitoringStats {
    /// Create new global monitoring statistics
    pub fn new() -> Self {
        Self {
            total_loops_monitored: 0,
            total_loop_executions: 0,
            total_iterations: 0,
            total_execution_time: Duration::ZERO,
            hot_loops_detected: 0,
            jit_compilations_triggered: 0,
            successful_jit_compilations: 0,
            monitoring_start_time: Instant::now(),
        }
    }

    /// Get the average execution time per loop
    pub fn avg_execution_time_per_loop(&self) -> Duration {
        if self.total_loop_executions > 0 {
            self.total_execution_time / self.total_loop_executions as u32
        } else {
            Duration::ZERO
        }
    }

    /// Get the hot loop detection rate
    pub fn hot_loop_detection_rate(&self) -> f64 {
        if self.total_loops_monitored > 0 {
            self.hot_loops_detected as f64 / self.total_loops_monitored as f64 * 100.0
        } else {
            0.0
        }
    }

    /// Get the JIT compilation success rate
    pub fn jit_success_rate(&self) -> f64 {
        if self.jit_compilations_triggered > 0 {
            self.successful_jit_compilations as f64 / self.jit_compilations_triggered as f64 * 100.0
        } else {
            0.0
        }
    }
}

impl Default for LoopMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{LoopConfig, Value, ValueType};
    use crate::types::SourceLocation;
    use std::thread;
    use std::time::Duration;

    fn create_test_loop_instruction() -> LoopInstruction {
        LoopInstruction::For {
            id: LoopID::new("test-loop".to_string()),
            range: crate::bcib::LoopRange::new(0, 5, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        }
    }

    #[test]
    fn test_loop_monitor_creation() {
        let monitor = LoopMonitor::new();
        assert_eq!(monitor.get_global_stats().total_loops_monitored, 0);
        assert_eq!(monitor.get_global_stats().hot_loops_detected, 0);
        assert_eq!(monitor.get_config().hot_loop_threshold, HOT_LOOP_THRESHOLD);
    }

    #[test]
    fn test_loop_execution_tracking() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("test-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Record loop start
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        assert_eq!(tracker.loop_id, loop_id);

        // Simulate some execution time
        thread::sleep(Duration::from_millis(1));

        // Record loop completion
        monitor.record_loop_completion(
            tracker,
            500, // iterations completed
            LoopExecutionResult::Success,
        ).unwrap();

        // Check statistics
        let stats = monitor.get_loop_stats(&loop_id).unwrap();
        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.total_iterations, 500);
        assert_eq!(stats.max_iterations_per_execution, 500);
        assert!(!stats.is_hot_loop); // Below threshold

        // Check profiling data
        assert_eq!(stats.profiling_data.total_execution_time, stats.total_execution_time);
        assert!(stats.profiling_data.average_iteration_time_ns > 0);

        // Check global stats
        let global_stats = monitor.get_global_stats();
        assert_eq!(global_stats.total_loops_monitored, 1);
        assert_eq!(global_stats.total_loop_executions, 1);
        assert_eq!(global_stats.total_iterations, 500);
    }

    #[test]
    fn test_hot_loop_detection() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("hot-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Record loop execution that exceeds hot loop threshold
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(
            tracker,
            1500, // Exceeds HOT_LOOP_THRESHOLD (1000)
            LoopExecutionResult::Success,
        ).unwrap();

        // Check that loop was detected as hot
        assert!(monitor.is_hot_loop(&loop_id));
        
        let stats = monitor.get_loop_stats(&loop_id).unwrap();
        assert!(stats.is_hot_loop);
        assert_eq!(stats.jit_compilation_status, JITCompilationStatus::Compiling);

        let hot_loop_info = monitor.get_hot_loop_info(&loop_id).unwrap();
        assert_eq!(hot_loop_info.detection_iteration_count, 1500);
        assert_eq!(hot_loop_info.hot_detection_count, 1);
        // Since auto_trigger_jit is enabled by default, JIT should be triggered
        assert!(hot_loop_info.jit_triggered);
        assert_eq!(hot_loop_info.jit_status, JITCompilationStatus::Compiling);

        // Check global stats
        let global_stats = monitor.get_global_stats();
        assert_eq!(global_stats.hot_loops_detected, 1);
    }

    #[test]
    fn test_hot_loop_threshold_boundary() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("boundary-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Test exactly at threshold
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(
            tracker,
            HOT_LOOP_THRESHOLD, // Exactly at threshold
            LoopExecutionResult::Success,
        ).unwrap();

        // Should be detected as hot
        assert!(monitor.is_hot_loop(&loop_id));
        assert_eq!(monitor.get_global_stats().hot_loops_detected, 1);

        // Test just below threshold
        let loop_id2 = LoopID::new("below-threshold-loop".to_string());
        let tracker2 = monitor.record_loop_start(&loop_id2, &instruction);
        monitor.record_loop_completion(
            tracker2,
            HOT_LOOP_THRESHOLD - 1, // Just below threshold
            LoopExecutionResult::Success,
        ).unwrap();

        // Should NOT be detected as hot
        assert!(!monitor.is_hot_loop(&loop_id2));
        assert_eq!(monitor.get_global_stats().hot_loops_detected, 1); // Still 1
    }

    #[test]
    fn test_jit_compilation_triggering() {
        let config = MonitoringConfig {
            auto_trigger_jit: true,
            ..MonitoringConfig::default()
        };
        let mut monitor = LoopMonitor::with_config(config);
        let loop_id = LoopID::new("jit-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Record hot loop execution
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(
            tracker,
            2000, // Hot loop
            LoopExecutionResult::Success,
        ).unwrap();

        // Check that JIT compilation was triggered
        let hot_loop_info = monitor.get_hot_loop_info(&loop_id).unwrap();
        assert!(hot_loop_info.jit_triggered);
        assert_eq!(hot_loop_info.jit_status, JITCompilationStatus::Compiling);

        // Check global stats
        assert_eq!(monitor.get_global_stats().jit_compilations_triggered, 1);
    }

    #[test]
    fn test_jit_compilation_result_recording() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("jit-result-loop".to_string());
        let instruction = create_test_loop_instruction();

        // First record a hot loop execution to create the hot loop info
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(tracker, 1500, LoopExecutionResult::Success).unwrap();

        // Manually trigger JIT compilation
        monitor.trigger_jit_compilation(&loop_id).unwrap();

        // Record successful compilation
        monitor.record_jit_compilation_result(
            &loop_id,
            JITCompilationResult::Success {
                compilation_time: Duration::from_millis(100),
            },
        ).unwrap();

        // Check compilation status
        let hot_loop_info = monitor.get_hot_loop_info(&loop_id).unwrap();
        match &hot_loop_info.jit_status {
            JITCompilationStatus::Compiled { compilation_time, .. } => {
                assert_eq!(*compilation_time, Duration::from_millis(100));
            }
            _ => panic!("Expected Compiled status"),
        }

        // Check global stats
        assert_eq!(monitor.get_global_stats().successful_jit_compilations, 1);
    }

    #[test]
    fn test_jit_compilation_failure_recording() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("jit-fail-loop".to_string());
        let instruction = create_test_loop_instruction();

        // First record a hot loop execution to create the hot loop info
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(tracker, 1500, LoopExecutionResult::Success).unwrap();

        // Manually trigger JIT compilation
        monitor.trigger_jit_compilation(&loop_id).unwrap();

        // Record failed compilation
        monitor.record_jit_compilation_result(
            &loop_id,
            JITCompilationResult::Failure {
                reason: "Compilation error".to_string(),
            },
        ).unwrap();

        // Check compilation status
        let hot_loop_info = monitor.get_hot_loop_info(&loop_id).unwrap();
        match &hot_loop_info.jit_status {
            JITCompilationStatus::Failed { reason, .. } => {
                assert_eq!(reason, "Compilation error");
            }
            _ => panic!("Expected Failed status"),
        }

        // Check global stats
        assert_eq!(monitor.get_global_stats().successful_jit_compilations, 0);
        assert_eq!(monitor.get_global_stats().jit_compilations_triggered, 1);
    }

    #[test]
    fn test_multiple_loop_executions() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("multi-exec-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Execute loop multiple times
        for i in 1..=5 {
            let tracker = monitor.record_loop_start(&loop_id, &instruction);
            monitor.record_loop_completion(
                tracker,
                i * 200, // Increasing iterations: 200, 400, 600, 800, 1000
                LoopExecutionResult::Success,
            ).unwrap();
        }

        // Check accumulated statistics
        let stats = monitor.get_loop_stats(&loop_id).unwrap();
        assert_eq!(stats.execution_count, 5);
        assert_eq!(stats.total_iterations, 3000); // 200+400+600+800+1000
        assert_eq!(stats.max_iterations_per_execution, 1000);
        assert_eq!(stats.avg_iterations_per_execution, 600.0);

        // Should be hot after 5th execution (1000 iterations == threshold)
        assert!(stats.is_hot_loop);
    }

    #[test]
    fn test_monitoring_summary() {
        let mut monitor = LoopMonitor::new();
        let loop_id1 = LoopID::new("loop1".to_string());
        let loop_id2 = LoopID::new("loop2".to_string());
        let instruction = create_test_loop_instruction();

        // Execute multiple loops
        let tracker1 = monitor.record_loop_start(&loop_id1, &instruction);
        monitor.record_loop_completion(tracker1, 1200, LoopExecutionResult::Success).unwrap();

        let tracker2 = monitor.record_loop_start(&loop_id2, &instruction);
        monitor.record_loop_completion(tracker2, 800, LoopExecutionResult::Success).unwrap();

        // Get monitoring summary
        let summary = monitor.get_monitoring_summary();
        assert_eq!(summary.total_loops_monitored, 2);
        assert_eq!(summary.total_loop_executions, 2);
        assert_eq!(summary.total_iterations, 2000);
        assert_eq!(summary.hot_loops_detected, 1); // Only loop1 is hot
        assert_eq!(summary.average_iterations_per_execution, 1000.0);
    }

    #[test]
    fn test_monitoring_config_variants() {
        // Test performance optimized config
        let perf_config = MonitoringConfig::performance_optimized();
        assert!(!perf_config.enable_detailed_tracking);
        assert!(perf_config.auto_trigger_jit);
        assert_eq!(perf_config.hot_loop_threshold, HOT_LOOP_THRESHOLD);

        // Test debug optimized config
        let debug_config = MonitoringConfig::debug_optimized();
        assert!(debug_config.enable_detailed_tracking);
        assert!(!debug_config.auto_trigger_jit);
        assert_eq!(debug_config.hot_loop_threshold, 100);
    }

    #[test]
    fn test_loop_execution_stats_calculations() {
        let loop_id = LoopID::new("calc-test-loop".to_string());
        let mut stats = LoopExecutionStats::new(loop_id);

        // Simulate multiple executions
        stats.execution_count = 3;
        stats.total_iterations = 1500;
        stats.total_execution_time = Duration::from_millis(300);

        // Test calculations
        assert_eq!(stats.avg_execution_time(), Duration::from_millis(100));
        assert_eq!(stats.avg_time_per_iteration(), Duration::from_nanos(200_000));
    }

    #[test]
    fn test_global_stats_calculations() {
        let mut stats = GlobalMonitoringStats::new();
        
        // Simulate monitoring data
        stats.total_loops_monitored = 10;
        stats.total_loop_executions = 25;
        stats.total_execution_time = Duration::from_millis(5000);
        stats.hot_loops_detected = 3;
        stats.jit_compilations_triggered = 2;
        stats.successful_jit_compilations = 1;

        // Test calculations
        assert_eq!(stats.avg_execution_time_per_loop(), Duration::from_millis(200));
        assert_eq!(stats.hot_loop_detection_rate(), 30.0);
        assert_eq!(stats.jit_success_rate(), 50.0);
    }

    #[test]
    fn test_clear_monitoring_data() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("clear-test-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Add some data
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(tracker, 1500, LoopExecutionResult::Success).unwrap();

        // Verify data exists
        assert!(monitor.is_hot_loop(&loop_id));
        assert_eq!(monitor.get_global_stats().total_loops_monitored, 1);

        // Clear data
        monitor.clear_monitoring_data();

        // Verify data is cleared
        assert!(!monitor.is_hot_loop(&loop_id));
        assert_eq!(monitor.get_global_stats().total_loops_monitored, 0);
        assert_eq!(monitor.get_global_stats().hot_loops_detected, 0);
        assert!(monitor.get_current_alerts().is_empty());
    }

    #[test]
    fn test_monitoring_api_interface() {
        let mut monitor = LoopMonitor::new();
        let loop_id1 = LoopID::new("api-test-loop1".to_string());
        let loop_id2 = LoopID::new("api-test-loop2".to_string());
        let instruction = create_test_loop_instruction();

        // Execute some loops
        let tracker1 = monitor.record_loop_start(&loop_id1, &instruction);
        monitor.record_loop_completion(tracker1, 1200, LoopExecutionResult::Success).unwrap();

        let tracker2 = monitor.record_loop_start(&loop_id2, &instruction);
        monitor.record_loop_completion(tracker2, 800, LoopExecutionResult::Success).unwrap();

        // Test monitoring API methods
        let api: &dyn LoopMonitoringAPI = &monitor;
        
        // Test get_completed_loop_metrics
        let stats1 = api.get_completed_loop_metrics(&loop_id1);
        assert!(stats1.is_some());
        assert_eq!(stats1.unwrap().total_iterations, 1200);

        // Test get_all_active_loops
        let active_loops = api.get_all_active_loops();
        assert_eq!(active_loops.len(), 2);
        assert!(active_loops.contains(&loop_id1));
        assert!(active_loops.contains(&loop_id2));

        // Test get_recent_completed_loops
        let recent_loops = api.get_recent_completed_loops(1);
        assert_eq!(recent_loops.len(), 1);

        // Test get_performance_summary
        let summary = api.get_performance_summary(Duration::from_secs(60));
        assert_eq!(summary.total_loops, 2);
        assert!(summary.hot_loops_percentage > 0.0); // Should have some hot loops

        // Test query_loops_by_criteria
        let criteria = LoopQueryCriteria {
            min_iteration_count: Some(1000),
            max_iteration_count: None,
            min_execution_time_ms: None,
            max_execution_time_ms: None,
            jit_status: None,
            time_range: None,
        };
        let filtered_loops = api.query_loops_by_criteria(&criteria);
        assert_eq!(filtered_loops.len(), 1); // Only loop1 meets criteria

        // Test get_top_loops_by_metric
        let top_by_iterations = api.get_top_loops_by_metric(MetricType::IterationCount, 2);
        assert_eq!(top_by_iterations.len(), 2);
        assert_eq!(top_by_iterations[0].loop_id, loop_id1); // Should be first (more iterations)
    }

    #[test]
    fn test_alerting_system() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("alert-test-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Test iteration limit violation alert
        let tracker = monitor.record_loop_start(&loop_id, &instruction);
        monitor.record_loop_completion(
            tracker,
            1000,
            LoopExecutionResult::IterationLimitExceeded,
        ).unwrap();

        // Check that alert was created
        let alerts = monitor.get_current_alerts();
        assert_eq!(alerts.len(), 1);
        match &alerts[0] {
            LoopAlert::IterationLimitViolation { loop_id: alert_loop_id, limit, completed, severity, .. } => {
                assert_eq!(alert_loop_id, &loop_id);
                assert_eq!(*limit, 1000);
                assert_eq!(*completed, 1000);
                assert_eq!(*severity, AlertSeverity::Warning);
            }
            _ => panic!("Expected IterationLimitViolation alert"),
        }

        // Test has_alerts method
        assert!(monitor.has_alerts(&loop_id));
        
        let other_loop_id = LoopID::new("other-loop".to_string());
        assert!(!monitor.has_alerts(&other_loop_id));
    }

    #[test]
    fn test_profiling_data_updates() {
        let mut monitor = LoopMonitor::new();
        let loop_id = LoopID::new("profiling-test-loop".to_string());
        let instruction = create_test_loop_instruction();

        // Execute loop multiple times to build profiling data
        for i in 1..=3 {
            let tracker = monitor.record_loop_start(&loop_id, &instruction);
            thread::sleep(Duration::from_millis(i)); // Variable execution time
            monitor.record_loop_completion(
                tracker,
                i as u32 * 100, // Variable iteration count
                LoopExecutionResult::Success,
            ).unwrap();
        }

        // Check profiling data
        let stats = monitor.get_loop_stats(&loop_id).unwrap();
        let profiling = &stats.profiling_data;
        
        assert!(profiling.average_iteration_time_ns > 0);
        assert!(profiling.total_execution_time > Duration::ZERO);
        assert!(profiling.min_execution_time > Duration::ZERO);
        assert!(profiling.max_execution_time >= profiling.min_execution_time);
        assert!(profiling.memory_stats.peak_memory_bytes > 0);
        assert!(profiling.memory_stats.allocations_count > 0);
        
        // Performance trend should be calculated after 3 executions
        assert!(!matches!(profiling.performance_trend, PerformanceTrend::Insufficient));
    }
}