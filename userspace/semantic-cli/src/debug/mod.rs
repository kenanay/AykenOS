//! Debug and developer mode support for semantic CLI
//!
//! This module provides comprehensive debugging capabilities including:
//! - Developer mode with plan generation without execution
//! - Detailed tracing of semantic processing pipeline
//! - Dry-run execution and simulation capabilities
//! - Token-level streaming inspection and timing analysis

pub mod developer;
pub mod tracing;
pub mod dry_run;
pub mod inspection;

pub use developer::*;
pub use tracing::*;
pub use dry_run::*;
pub use inspection::*;

use crate::types::*;
use crate::error::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Debug session manager that coordinates all debugging activities
pub struct DebugSession {
    /// Unique session identifier
    pub session_id: Uuid,
    /// Developer mode controller
    pub developer: DeveloperController,
    /// Tracing system
    pub tracer: PipelineTracer,
    /// Dry-run executor
    pub dry_runner: DryRunExecutor,
    /// Token inspector for streaming analysis
    pub inspector: TokenInspector,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Debug configuration
    pub config: DebugConfig,
}

/// Configuration for debug session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Enable detailed tracing
    pub enable_tracing: bool,
    /// Trace level
    pub trace_level: TracingLevel,
    /// Enable dry-run by default
    pub default_dry_run: bool,
    /// Enable token inspection
    pub enable_token_inspection: bool,
    /// Maximum trace history to keep
    pub max_trace_history: usize,
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            trace_level: TracingLevel::Debug,
            default_dry_run: true,
            enable_token_inspection: true,
            max_trace_history: 1000,
            performance_monitoring: true,
        }
    }
}

impl DebugSession {
    /// Create a new debug session
    pub fn new() -> Self {
        let config = DebugConfig::default();
        
        Self {
            session_id: Uuid::new_v4(),
            developer: DeveloperController::new(),
            tracer: PipelineTracer::new(config.trace_level.clone()),
            dry_runner: DryRunExecutor::new(),
            inspector: TokenInspector::new(),
            start_time: Utc::now(),
            config,
        }
    }

    /// Create debug session with custom configuration
    pub fn with_config(config: DebugConfig) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            developer: DeveloperController::new(),
            tracer: PipelineTracer::new(config.trace_level.clone()),
            dry_runner: DryRunExecutor::new(),
            inspector: TokenInspector::new(),
            start_time: Utc::now(),
            config,
        }
    }

    /// Start debugging an intent processing pipeline
    pub async fn start_intent_debug(&mut self, intent: &Intent) -> Result<DebugHandle, DebugError> {
        let handle = DebugHandle::new(intent.id);
        
        if self.config.enable_tracing {
            self.tracer.start_trace(intent.id, "intent_processing").await?;
        }
        
        if self.config.performance_monitoring {
            self.tracer.start_performance_monitoring(intent.id).await?;
        }
        
        Ok(handle)
    }

    /// Generate plan without execution (developer mode)
    pub async fn generate_plan_only(&mut self, intent: &Intent) -> Result<ExecutionPlan, DebugError> {
        self.developer.generate_plan_without_execution(intent).await
    }

    /// Perform dry-run execution
    pub async fn dry_run_execution(&mut self, plan: &ExecutionPlan) -> Result<DryRunResult, DebugError> {
        self.dry_runner.simulate_execution(plan).await
    }

    /// Get comprehensive debug report
    pub fn get_debug_report(&self, intent_id: IntentId) -> Option<DebugReport> {
        let trace = self.tracer.get_trace(intent_id)?;
        let performance = self.tracer.get_performance_metrics(intent_id)?;
        
        Some(DebugReport {
            intent_id,
            session_id: self.session_id,
            trace: trace.clone(),
            performance_metrics: performance.clone(),
            dry_run_results: self.dry_runner.get_results(intent_id).cloned(),
            token_analysis: self.inspector.get_analysis(intent_id).cloned(),
            timestamp: Utc::now(),
        })
    }

    /// Clear debug history for memory management
    pub fn clear_history(&mut self) {
        self.tracer.clear_old_traces(self.config.max_trace_history);
        self.dry_runner.clear_old_results(self.config.max_trace_history);
        self.inspector.clear_old_analysis(self.config.max_trace_history);
    }
}

/// Handle for tracking a specific debug session
#[derive(Debug, Clone)]
pub struct DebugHandle {
    /// Intent being debugged
    pub intent_id: IntentId,
    /// Handle creation time
    pub created_at: Instant,
    /// Debug session ID
    pub session_id: Uuid,
}

impl DebugHandle {
    fn new(intent_id: IntentId) -> Self {
        Self {
            intent_id,
            created_at: Instant::now(),
            session_id: Uuid::new_v4(),
        }
    }

    /// Get elapsed time since debug started
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Comprehensive debug report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugReport {
    /// Intent that was debugged
    pub intent_id: IntentId,
    /// Debug session ID
    pub session_id: Uuid,
    /// Pipeline trace information
    pub trace: PipelineTrace,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Dry-run results if available
    pub dry_run_results: Option<DryRunResult>,
    /// Token analysis if available
    pub token_analysis: Option<TokenAnalysis>,
    /// Report generation timestamp
    pub timestamp: DateTime<Utc>,
}

impl DebugReport {
    /// Generate human-readable summary
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Debug Report for Intent: {}\n", self.intent_id));
        summary.push_str(&format!("Session: {}\n", self.session_id));
        summary.push_str(&format!("Generated: {}\n\n", self.timestamp));
        
        // Pipeline trace summary
        summary.push_str("Pipeline Trace:\n");
        for step in &self.trace.steps {
            summary.push_str(&format!("  {} ({}ms): {}\n", 
                step.name, 
                step.duration.as_millis(),
                step.status
            ));
        }
        
        // Performance summary
        summary.push_str(&format!("\nPerformance:\n"));
        summary.push_str(&format!("  Total Time: {}ms\n", self.performance_metrics.total_time.as_millis()));
        summary.push_str(&format!("  Parsing: {}ms\n", self.performance_metrics.parsing_time.as_millis()));
        summary.push_str(&format!("  Planning: {}ms\n", self.performance_metrics.planning_time.as_millis()));
        summary.push_str(&format!("  Compilation: {}ms\n", self.performance_metrics.compilation_time.as_millis()));
        
        // Dry-run results
        if let Some(dry_run) = &self.dry_run_results {
            summary.push_str(&format!("\nDry-run: {} ({}ms)\n", 
                if dry_run.would_succeed { "SUCCESS" } else { "FAILURE" },
                dry_run.simulation_time.as_millis()
            ));
        }
        
        summary
    }

    /// Export to JSON format
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Error types for debug operations
#[derive(Debug, thiserror::Error)]
pub enum DebugError {
    #[error("Tracing error: {0}")]
    TracingError(String),
    
    #[error("Dry-run execution failed: {0}")]
    DryRunFailed(String),
    
    #[error("Developer mode error: {0}")]
    DeveloperModeError(String),
    
    #[error("Token inspection error: {0}")]
    TokenInspectionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_session_creation() {
        let session = DebugSession::new();
        assert!(session.config.enable_tracing);
        assert!(session.config.default_dry_run);
        assert_eq!(session.config.max_trace_history, 1000);
    }

    #[tokio::test]
    async fn test_debug_handle_creation() {
        let intent_id = Uuid::new_v4();
        let handle = DebugHandle::new(intent_id);
        assert_eq!(handle.intent_id, intent_id);
        assert!(handle.elapsed().as_nanos() > 0);
    }

    #[test]
    fn test_debug_config_default() {
        let config = DebugConfig::default();
        assert!(config.enable_tracing);
        assert!(config.default_dry_run);
        assert_eq!(config.max_trace_history, 1000);
    }
}