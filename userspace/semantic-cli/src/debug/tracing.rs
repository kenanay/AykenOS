//! Detailed tracing system for semantic processing pipeline
//!
//! Provides comprehensive tracing of all pipeline stages including
//! parsing, planning, compilation, and execution with performance metrics.

use crate::types::*;
use super::DebugError;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};

/// Pipeline tracer that captures detailed execution traces
pub struct PipelineTracer {
    /// Active traces by intent ID
    active_traces: HashMap<IntentId, ActiveTrace>,
    /// Completed traces
    completed_traces: HashMap<IntentId, PipelineTrace>,
    /// Performance metrics
    performance_metrics: HashMap<IntentId, PerformanceMetrics>,
    /// Tracing configuration
    config: TracingConfig,
}

/// Configuration for pipeline tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Tracing level
    pub level: TracingLevel,
    /// Maximum number of traces to keep
    pub max_traces: usize,
    /// Enable performance monitoring
    pub enable_performance: bool,
    /// Enable detailed step tracing
    pub enable_step_tracing: bool,
    /// Capture input/output data
    pub capture_io_data: bool,
}

/// Active trace being recorded
#[derive(Debug, Clone)]
struct ActiveTrace {
    /// Trace ID
    trace_id: Uuid,
    /// Intent being traced
    intent_id: IntentId,
    /// Start time
    start_time: Instant,
    /// Current pipeline steps
    steps: Vec<TraceStep>,
    /// Performance timers
    timers: HashMap<String, Instant>,
}

/// Complete pipeline trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTrace {
    /// Trace ID
    pub trace_id: Uuid,
    /// Intent that was traced
    pub intent_id: IntentId,
    /// All pipeline steps
    pub steps: Vec<TraceStep>,
    /// Total execution time
    pub total_time: Duration,
    /// Trace completion timestamp
    pub completed_at: DateTime<Utc>,
    /// Trace metadata
    pub metadata: TraceMetadata,
}

/// Individual step in the pipeline trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    /// Step name/identifier
    pub name: String,
    /// Step type
    pub step_type: StepType,
    /// Step start time (relative to trace start)
    pub start_offset: Duration,
    /// Step duration
    pub duration: Duration,
    /// Step status
    pub status: StepStatus,
    /// Input data (if captured)
    pub input_data: Option<String>,
    /// Output data (if captured)
    pub output_data: Option<String>,
    /// Error information if step failed
    pub error_info: Option<String>,
    /// Sub-steps if any
    pub sub_steps: Vec<TraceStep>,
}

/// Types of pipeline steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Intent parsing
    Parsing,
    /// Plan generation
    Planning,
    /// Command compilation
    Compilation,
    /// Validation
    Validation,
    /// Execution
    Execution,
    /// Custom step
    Custom(String),
}

/// Status of a pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step started
    Started,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Started => write!(f, "STARTED"),
            StepStatus::Completed => write!(f, "COMPLETED"),
            StepStatus::Failed => write!(f, "FAILED"),
            StepStatus::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// Metadata for a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// Tracing level used
    pub tracing_level: TracingLevel,
    /// Whether performance was monitored
    pub performance_monitored: bool,
    /// Number of steps traced
    pub step_count: usize,
    /// Total data captured (bytes)
    pub data_captured_bytes: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: TracingLevel::Debug,
            max_traces: 100,
            enable_performance: true,
            enable_step_tracing: true,
            capture_io_data: true,
        }
    }
}

impl PipelineTracer {
    /// Create a new pipeline tracer
    pub fn new(level: TracingLevel) -> Self {
        Self {
            active_traces: HashMap::new(),
            completed_traces: HashMap::new(),
            performance_metrics: HashMap::new(),
            config: TracingConfig {
                level,
                ..Default::default()
            },
        }
    }

    /// Start tracing a pipeline execution
    pub async fn start_trace(&mut self, intent_id: IntentId, trace_name: &str) -> Result<Uuid, DebugError> {
        let trace_id = Uuid::new_v4();
        
        info!("Starting pipeline trace: {} for intent: {}", trace_name, intent_id);
        
        let active_trace = ActiveTrace {
            trace_id,
            intent_id,
            start_time: Instant::now(),
            steps: Vec::new(),
            timers: HashMap::new(),
        };
        
        self.active_traces.insert(intent_id, active_trace);
        
        // Start initial step
        self.start_step(intent_id, "pipeline_start", StepType::Custom("initialization".to_string())).await?;
        
        Ok(trace_id)
    }

    /// Start a new step in the trace
    pub async fn start_step(&mut self, intent_id: IntentId, step_name: &str, step_type: StepType) -> Result<(), DebugError> {
        if let Some(trace) = self.active_traces.get_mut(&intent_id) {
            let start_offset = trace.start_time.elapsed();
            
            let step = TraceStep {
                name: step_name.to_string(),
                step_type,
                start_offset,
                duration: Duration::from_nanos(0), // Will be updated when step completes
                status: StepStatus::Started,
                input_data: None,
                output_data: None,
                error_info: None,
                sub_steps: Vec::new(),
            };
            
            trace.steps.push(step.clone());
            trace.timers.insert(step_name.to_string(), Instant::now());
            
            debug!("Started trace step: {} (type: {:?})", step_name, step.step_type);
        } else {
            return Err(DebugError::TracingError(format!("No active trace for intent: {}", intent_id)));
        }
        
        Ok(())
    }

    /// Complete a step in the trace
    pub async fn complete_step(&mut self, intent_id: IntentId, step_name: &str, status: StepStatus) -> Result<(), DebugError> {
        if let Some(trace) = self.active_traces.get_mut(&intent_id) {
            // Find the step and update it
            if let Some(step) = trace.steps.iter_mut().find(|s| s.name == step_name && matches!(s.status, StepStatus::Started)) {
                if let Some(start_time) = trace.timers.get(step_name) {
                    step.duration = start_time.elapsed();
                }
                step.status = status.clone();
                
                debug!("Completed trace step: {} with status: {:?} in {}ms", 
                    step_name, status, step.duration.as_millis());
            } else {
                return Err(DebugError::TracingError(format!("Step not found or not started: {}", step_name)));
            }
        } else {
            return Err(DebugError::TracingError(format!("No active trace for intent: {}", intent_id)));
        }
        
        Ok(())
    }

    /// Add input data to current step
    pub async fn add_step_input(&mut self, intent_id: IntentId, step_name: &str, input_data: &str) -> Result<(), DebugError> {
        if !self.config.capture_io_data {
            return Ok(());
        }
        
        if let Some(trace) = self.active_traces.get_mut(&intent_id) {
            if let Some(step) = trace.steps.iter_mut().find(|s| s.name == step_name) {
                step.input_data = Some(input_data.to_string());
                trace!("Added input data to step: {} ({} bytes)", step_name, input_data.len());
            }
        }
        
        Ok(())
    }

    /// Add output data to current step
    pub async fn add_step_output(&mut self, intent_id: IntentId, step_name: &str, output_data: &str) -> Result<(), DebugError> {
        if !self.config.capture_io_data {
            return Ok(());
        }
        
        if let Some(trace) = self.active_traces.get_mut(&intent_id) {
            if let Some(step) = trace.steps.iter_mut().find(|s| s.name == step_name) {
                step.output_data = Some(output_data.to_string());
                trace!("Added output data to step: {} ({} bytes)", step_name, output_data.len());
            }
        }
        
        Ok(())
    }

    /// Add error information to current step
    pub async fn add_step_error(&mut self, intent_id: IntentId, step_name: &str, error_info: &str) -> Result<(), DebugError> {
        if let Some(trace) = self.active_traces.get_mut(&intent_id) {
            if let Some(step) = trace.steps.iter_mut().find(|s| s.name == step_name) {
                step.error_info = Some(error_info.to_string());
                step.status = StepStatus::Failed;
                debug!("Added error to step: {} - {}", step_name, error_info);
            }
        }
        
        Ok(())
    }

    /// Complete the entire trace
    pub async fn complete_trace(&mut self, intent_id: IntentId) -> Result<PipelineTrace, DebugError> {
        if let Some(active_trace) = self.active_traces.remove(&intent_id) {
            let total_time = active_trace.start_time.elapsed();
            
            // Calculate metadata
            let data_captured_bytes = active_trace.steps.iter()
                .map(|step| {
                    let input_size = step.input_data.as_ref().map(|s| s.len()).unwrap_or(0);
                    let output_size = step.output_data.as_ref().map(|s| s.len()).unwrap_or(0);
                    input_size + output_size
                })
                .sum();
            
            let metadata = TraceMetadata {
                tracing_level: self.config.level.clone(),
                performance_monitored: self.config.enable_performance,
                step_count: active_trace.steps.len(),
                data_captured_bytes,
            };
            
            let completed_trace = PipelineTrace {
                trace_id: active_trace.trace_id,
                intent_id,
                steps: active_trace.steps,
                total_time,
                completed_at: Utc::now(),
                metadata,
            };
            
            info!("Completed pipeline trace: {} in {}ms ({} steps)", 
                completed_trace.trace_id, 
                total_time.as_millis(),
                completed_trace.steps.len()
            );
            
            self.completed_traces.insert(intent_id, completed_trace.clone());
            
            // Clean up old traces if needed
            self.cleanup_old_traces();
            
            Ok(completed_trace)
        } else {
            Err(DebugError::TracingError(format!("No active trace for intent: {}", intent_id)))
        }
    }

    /// Start performance monitoring for an intent
    pub async fn start_performance_monitoring(&mut self, intent_id: IntentId) -> Result<(), DebugError> {
        if !self.config.enable_performance {
            return Ok(());
        }
        
        let metrics = PerformanceMetrics {
            total_time: Duration::from_nanos(0),
            parsing_time: Duration::from_nanos(0),
            planning_time: Duration::from_nanos(0),
            compilation_time: Duration::from_nanos(0),
        };
        
        self.performance_metrics.insert(intent_id, metrics);
        debug!("Started performance monitoring for intent: {}", intent_id);
        
        Ok(())
    }

    /// Update performance metrics
    pub async fn update_performance_metric(&mut self, intent_id: IntentId, metric_type: &str, duration: Duration) -> Result<(), DebugError> {
        if let Some(metrics) = self.performance_metrics.get_mut(&intent_id) {
            match metric_type {
                "parsing" => metrics.parsing_time = duration,
                "planning" => metrics.planning_time = duration,
                "compilation" => metrics.compilation_time = duration,
                "total" => metrics.total_time = duration,
                _ => debug!("Unknown performance metric type: {}", metric_type),
            }
        }
        
        Ok(())
    }

    /// Get completed trace
    pub fn get_trace(&self, intent_id: IntentId) -> Option<&PipelineTrace> {
        self.completed_traces.get(&intent_id)
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self, intent_id: IntentId) -> Option<&PerformanceMetrics> {
        self.performance_metrics.get(&intent_id)
    }

    /// Get all completed traces
    pub fn get_all_traces(&self) -> Vec<&PipelineTrace> {
        self.completed_traces.values().collect()
    }

    /// Clear old traces to manage memory
    pub fn clear_old_traces(&mut self, max_to_keep: usize) {
        if self.completed_traces.len() > max_to_keep {
            // Keep only the most recent traces
            let mut traces: Vec<_> = self.completed_traces.iter().collect();
            traces.sort_by(|a, b| b.1.completed_at.cmp(&a.1.completed_at));
            
            let to_remove: Vec<_> = traces.iter()
                .skip(max_to_keep)
                .map(|(intent_id, _)| **intent_id)
                .collect();
            
            for intent_id in to_remove {
                self.completed_traces.remove(&intent_id);
                self.performance_metrics.remove(&intent_id);
            }
            
            info!("Cleaned up old traces, keeping {} most recent", max_to_keep);
        }
    }

    /// Cleanup old traces based on configuration
    fn cleanup_old_traces(&mut self) {
        self.clear_old_traces(self.config.max_traces);
    }

    /// Generate trace summary
    pub fn generate_trace_summary(&self, intent_id: IntentId) -> Option<TraceSummary> {
        let trace = self.get_trace(intent_id)?;
        let performance = self.get_performance_metrics(intent_id);
        
        let successful_steps = trace.steps.iter()
            .filter(|step| matches!(step.status, StepStatus::Completed))
            .count();
        
        let failed_steps = trace.steps.iter()
            .filter(|step| matches!(step.status, StepStatus::Failed))
            .count();
        
        Some(TraceSummary {
            trace_id: trace.trace_id,
            intent_id,
            total_steps: trace.steps.len(),
            successful_steps,
            failed_steps,
            total_time: trace.total_time,
            performance_metrics: performance.cloned(),
            data_captured_bytes: trace.metadata.data_captured_bytes,
        })
    }
}

/// Summary of a pipeline trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    /// Trace ID
    pub trace_id: Uuid,
    /// Intent ID
    pub intent_id: IntentId,
    /// Total number of steps
    pub total_steps: usize,
    /// Number of successful steps
    pub successful_steps: usize,
    /// Number of failed steps
    pub failed_steps: usize,
    /// Total execution time
    pub total_time: Duration,
    /// Performance metrics if available
    pub performance_metrics: Option<PerformanceMetrics>,
    /// Total data captured in bytes
    pub data_captured_bytes: usize,
}

impl Default for PipelineTracer {
    fn default() -> Self {
        Self::new(TracingLevel::Debug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_tracer_creation() {
        let tracer = PipelineTracer::new(TracingLevel::Debug);
        assert_eq!(tracer.config.level, TracingLevel::Debug);
        assert!(tracer.config.enable_performance);
        assert!(tracer.active_traces.is_empty());
    }

    #[tokio::test]
    async fn test_trace_lifecycle() {
        let mut tracer = PipelineTracer::new(TracingLevel::Debug);
        let intent_id = Uuid::new_v4();
        
        // Start trace
        let trace_id = tracer.start_trace(intent_id, "test_trace").await.unwrap();
        assert!(tracer.active_traces.contains_key(&intent_id));
        
        // Add step
        tracer.start_step(intent_id, "test_step", StepType::Parsing).await.unwrap();
        tracer.complete_step(intent_id, "test_step", StepStatus::Completed).await.unwrap();
        
        // Complete trace
        let completed_trace = tracer.complete_trace(intent_id).await.unwrap();
        assert_eq!(completed_trace.trace_id, trace_id);
        assert!(!tracer.active_traces.contains_key(&intent_id));
        assert!(tracer.completed_traces.contains_key(&intent_id));
    }

    #[tokio::test]
    async fn test_step_data_capture() {
        let mut tracer = PipelineTracer::new(TracingLevel::Debug);
        let intent_id = Uuid::new_v4();
        
        tracer.start_trace(intent_id, "test_trace").await.unwrap();
        tracer.start_step(intent_id, "test_step", StepType::Parsing).await.unwrap();
        
        // Add input and output data
        tracer.add_step_input(intent_id, "test_step", "input_data").await.unwrap();
        tracer.add_step_output(intent_id, "test_step", "output_data").await.unwrap();
        
        tracer.complete_step(intent_id, "test_step", StepStatus::Completed).await.unwrap();
        let trace = tracer.complete_trace(intent_id).await.unwrap();
        
        let step = &trace.steps[1]; // Skip pipeline_start step
        assert_eq!(step.input_data, Some("input_data".to_string()));
        assert_eq!(step.output_data, Some("output_data".to_string()));
    }

    #[tokio::test]
    async fn test_performance_monitoring() {
        let mut tracer = PipelineTracer::new(TracingLevel::Debug);
        let intent_id = Uuid::new_v4();
        
        tracer.start_performance_monitoring(intent_id).await.unwrap();
        tracer.update_performance_metric(intent_id, "parsing", Duration::from_millis(100)).await.unwrap();
        
        let metrics = tracer.get_performance_metrics(intent_id).unwrap();
        assert_eq!(metrics.parsing_time, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_trace_cleanup() {
        let mut tracer = PipelineTracer::new(TracingLevel::Debug);
        tracer.config.max_traces = 2;
        
        // Create 3 traces
        for i in 0..3 {
            let intent_id = Uuid::new_v4();
            tracer.start_trace(intent_id, &format!("trace_{}", i)).await.unwrap();
            tracer.complete_trace(intent_id).await.unwrap();
        }
        
        // Should only keep 2 most recent
        assert_eq!(tracer.completed_traces.len(), 2);
    }
}