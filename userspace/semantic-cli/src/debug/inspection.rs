//! Token-level streaming inspection and timing analysis
//!
//! Provides detailed analysis of streaming operations including
//! token generation rates, buffer utilization, and performance metrics.

use crate::types::*;
use super::DebugError;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};

/// Token inspector for analyzing streaming operations
pub struct TokenInspector {
    /// Active inspections by intent ID
    active_inspections: HashMap<IntentId, ActiveInspection>,
    /// Completed analyses
    completed_analyses: HashMap<IntentId, TokenAnalysis>,
    /// Inspector configuration
    config: InspectionConfig,
}

/// Configuration for token inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionConfig {
    /// Enable detailed token logging
    pub enable_token_logging: bool,
    /// Enable timing analysis
    pub enable_timing_analysis: bool,
    /// Enable buffer monitoring
    pub enable_buffer_monitoring: bool,
    /// Maximum tokens to track per inspection
    pub max_tokens_tracked: usize,
    /// Sampling rate for performance metrics (0.0 to 1.0)
    pub sampling_rate: f32,
}

/// Active token inspection session
#[derive(Debug, Clone)]
struct ActiveInspection {
    /// Inspection ID
    inspection_id: Uuid,
    /// Intent being inspected
    intent_id: IntentId,
    /// Start time
    start_time: Instant,
    /// Token events
    token_events: VecDeque<TokenEvent>,
    /// Buffer state snapshots
    buffer_snapshots: VecDeque<BufferSnapshot>,
    /// Performance samples
    performance_samples: VecDeque<PerformanceSample>,
    /// Current statistics
    current_stats: InspectionStats,
}

/// Token event during streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    /// Event timestamp (relative to inspection start)
    pub timestamp: Duration,
    /// Event type
    pub event_type: TokenEventType,
    /// Token data if applicable
    pub token_data: Option<TokenData>,
    /// Buffer state at time of event
    pub buffer_state: Option<BufferState>,
    /// Performance metrics at time of event
    pub performance_metrics: Option<StreamMetrics>,
}

/// Types of token events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenEventType {
    /// Token generation started
    GenerationStarted,
    /// Token generated
    TokenGenerated,
    /// Token sent to buffer
    TokenBuffered,
    /// Token delivered to client
    TokenDelivered,
    /// Buffer overflow occurred
    BufferOverflow,
    /// Buffer underflow occurred
    BufferUnderflow,
    /// Stream paused
    StreamPaused,
    /// Stream resumed
    StreamResumed,
    /// Stream completed
    StreamCompleted,
    /// Stream cancelled
    StreamCancelled,
    /// Error occurred
    Error(String),
}

/// Token data for inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    /// Token ID or sequence number
    pub token_id: u64,
    /// Token content (may be truncated for privacy)
    pub content: String,
    /// Token length in characters
    pub length: usize,
    /// Token type if known
    pub token_type: Option<String>,
    /// Generation time for this token
    pub generation_time: Duration,
}

/// Buffer state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSnapshot {
    /// Snapshot timestamp
    pub timestamp: Duration,
    /// Input buffer utilization (0.0 to 1.0)
    pub input_utilization: f32,
    /// Output buffer utilization (0.0 to 1.0)
    pub output_utilization: f32,
    /// Total tokens in buffers
    pub total_buffered_tokens: usize,
    /// Buffer overflow count
    pub overflow_count: u64,
    /// Buffer underflow count
    pub underflow_count: u64,
}

/// Performance sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    /// Sample timestamp
    pub timestamp: Duration,
    /// Tokens per second at this moment
    pub tokens_per_second: f32,
    /// Latency percentiles
    pub latency_p50: Duration,
    pub latency_p95: Duration,
    pub latency_p99: Duration,
    /// Throughput in bytes per second
    pub throughput_bps: f32,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
}

/// Current inspection statistics
#[derive(Debug, Clone, Default)]
struct InspectionStats {
    /// Total tokens processed
    total_tokens: u64,
    /// Total generation time
    total_generation_time: Duration,
    /// Average tokens per second
    avg_tokens_per_second: f32,
    /// Peak tokens per second
    peak_tokens_per_second: f32,
    /// Total buffer overflows
    total_overflows: u64,
    /// Total buffer underflows
    total_underflows: u64,
    /// Error count
    error_count: u64,
}

/// Complete token analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysis {
    /// Analysis ID
    pub analysis_id: Uuid,
    /// Intent that was analyzed
    pub intent_id: IntentId,
    /// All token events
    pub token_events: Vec<TokenEvent>,
    /// Buffer utilization over time
    pub buffer_utilization: Vec<BufferSnapshot>,
    /// Performance metrics over time
    pub performance_timeline: Vec<PerformanceSample>,
    /// Summary statistics
    pub summary: AnalysisSummary,
    /// Analysis completion time
    pub completed_at: DateTime<Utc>,
}

/// Summary of token analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Total inspection duration
    pub total_duration: Duration,
    /// Total tokens processed
    pub total_tokens: u64,
    /// Average generation rate (tokens/sec)
    pub avg_generation_rate: f32,
    /// Peak generation rate (tokens/sec)
    pub peak_generation_rate: f32,
    /// Average buffer utilization
    pub avg_buffer_utilization: f32,
    /// Peak buffer utilization
    pub peak_buffer_utilization: f32,
    /// Total buffer events
    pub buffer_overflow_count: u64,
    pub buffer_underflow_count: u64,
    /// Performance characteristics
    pub avg_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    /// Quality metrics
    pub error_rate: f32,
    pub completion_rate: f32,
}

impl Default for InspectionConfig {
    fn default() -> Self {
        Self {
            enable_token_logging: true,
            enable_timing_analysis: true,
            enable_buffer_monitoring: true,
            max_tokens_tracked: 10000,
            sampling_rate: 1.0,
        }
    }
}

impl TokenInspector {
    /// Create a new token inspector
    pub fn new() -> Self {
        Self {
            active_inspections: HashMap::new(),
            completed_analyses: HashMap::new(),
            config: InspectionConfig::default(),
        }
    }

    /// Create token inspector with custom configuration
    pub fn with_config(config: InspectionConfig) -> Self {
        Self {
            active_inspections: HashMap::new(),
            completed_analyses: HashMap::new(),
            config,
        }
    }

    /// Start inspecting a streaming operation
    pub async fn start_inspection(&mut self, intent_id: IntentId) -> Result<Uuid, DebugError> {
        let inspection_id = Uuid::new_v4();
        
        info!("Starting token inspection for intent: {}", intent_id);
        
        let inspection = ActiveInspection {
            inspection_id,
            intent_id,
            start_time: Instant::now(),
            token_events: VecDeque::new(),
            buffer_snapshots: VecDeque::new(),
            performance_samples: VecDeque::new(),
            current_stats: InspectionStats::default(),
        };
        
        self.active_inspections.insert(intent_id, inspection);
        
        // Record inspection start event
        self.record_token_event(intent_id, TokenEventType::GenerationStarted, None).await?;
        
        Ok(inspection_id)
    }

    /// Record a token event
    pub async fn record_token_event(
        &mut self, 
        intent_id: IntentId, 
        event_type: TokenEventType, 
        token_data: Option<TokenData>
    ) -> Result<(), DebugError> {
        if let Some(inspection) = self.active_inspections.get_mut(&intent_id) {
            let timestamp = inspection.start_time.elapsed();
            
            // Update statistics
            match &event_type {
                TokenEventType::TokenGenerated => {
                    inspection.current_stats.total_tokens += 1;
                    if let Some(token) = &token_data {
                        inspection.current_stats.total_generation_time += token.generation_time;
                    }
                },
                TokenEventType::BufferOverflow => {
                    inspection.current_stats.total_overflows += 1;
                },
                TokenEventType::BufferUnderflow => {
                    inspection.current_stats.total_underflows += 1;
                },
                TokenEventType::Error(_) => {
                    inspection.current_stats.error_count += 1;
                },
                _ => {}
            }
            
            // Create event
            let event = TokenEvent {
                timestamp,
                event_type: event_type.clone(),
                token_data,
                buffer_state: None, // Will be filled by buffer monitoring
                performance_metrics: None, // Will be filled by performance monitoring
            };
            
            // Add to event queue (with size limit)
            inspection.token_events.push_back(event);
            if inspection.token_events.len() > self.config.max_tokens_tracked {
                inspection.token_events.pop_front();
            }
            
            trace!("Recorded token event: {:?} at {}ms", event_type, timestamp.as_millis());
        } else {
            return Err(DebugError::TokenInspectionError(
                format!("No active inspection for intent: {}", intent_id)
            ));
        }
        
        Ok(())
    }

    /// Record buffer state snapshot
    pub async fn record_buffer_snapshot(
        &mut self, 
        intent_id: IntentId, 
        buffer_state: BufferState
    ) -> Result<(), DebugError> {
        if !self.config.enable_buffer_monitoring {
            return Ok(());
        }
        
        if let Some(inspection) = self.active_inspections.get_mut(&intent_id) {
            let timestamp = inspection.start_time.elapsed();
            
            let snapshot = BufferSnapshot {
                timestamp,
                input_utilization: buffer_state.buffer_utilization,
                output_utilization: buffer_state.buffer_utilization, // Simplified for demo
                total_buffered_tokens: 0, // Would be calculated from actual buffer
                overflow_count: buffer_state.overflow_count,
                underflow_count: buffer_state.underflow_count,
            };
            
            inspection.buffer_snapshots.push_back(snapshot);
            
            // Limit snapshot history
            if inspection.buffer_snapshots.len() > 1000 {
                inspection.buffer_snapshots.pop_front();
            }
            
            trace!("Recorded buffer snapshot at {}ms: {:.2}% utilization", 
                timestamp.as_millis(), buffer_state.buffer_utilization * 100.0);
        }
        
        Ok(())
    }

    /// Record performance sample
    pub async fn record_performance_sample(
        &mut self, 
        intent_id: IntentId, 
        metrics: StreamMetrics
    ) -> Result<(), DebugError> {
        if !self.config.enable_timing_analysis {
            return Ok(());
        }
        
        // Sample based on configured rate
        if rand::random::<f32>() > self.config.sampling_rate {
            return Ok(());
        }
        
        if let Some(inspection) = self.active_inspections.get_mut(&intent_id) {
            let timestamp = inspection.start_time.elapsed();
            
            let sample = PerformanceSample {
                timestamp,
                tokens_per_second: metrics.tokens_per_second,
                latency_p50: metrics.latency_p50,
                latency_p95: metrics.latency_p95,
                latency_p99: metrics.latency_p99,
                throughput_bps: metrics.throughput_mbps * 1024.0 * 1024.0, // Convert to bytes/sec
                cpu_usage: 0.0, // Would be measured from system
                memory_usage: 0, // Would be measured from system
            };
            
            inspection.performance_samples.push_back(sample);
            
            // Update peak statistics
            if metrics.tokens_per_second > inspection.current_stats.peak_tokens_per_second {
                inspection.current_stats.peak_tokens_per_second = metrics.tokens_per_second;
            }
            
            // Limit sample history
            if inspection.performance_samples.len() > 1000 {
                inspection.performance_samples.pop_front();
            }
            
            trace!("Recorded performance sample at {}ms: {:.2} tokens/sec", 
                timestamp.as_millis(), metrics.tokens_per_second);
        }
        
        Ok(())
    }

    /// Complete inspection and generate analysis
    pub async fn complete_inspection(&mut self, intent_id: IntentId) -> Result<TokenAnalysis, DebugError> {
        if let Some(inspection) = self.active_inspections.remove(&intent_id) {
            info!("Completing token inspection for intent: {}", intent_id);
            
            let total_duration = inspection.start_time.elapsed();
            
            // Calculate summary statistics
            let summary = self.calculate_analysis_summary(&inspection, total_duration);
            
            let analysis = TokenAnalysis {
                analysis_id: inspection.inspection_id,
                intent_id,
                token_events: inspection.token_events.into_iter().collect(),
                buffer_utilization: inspection.buffer_snapshots.into_iter().collect(),
                performance_timeline: inspection.performance_samples.into_iter().collect(),
                summary,
                completed_at: Utc::now(),
            };
            
            self.completed_analyses.insert(intent_id, analysis.clone());
            
            Ok(analysis)
        } else {
            Err(DebugError::TokenInspectionError(
                format!("No active inspection for intent: {}", intent_id)
            ))
        }
    }

    /// Calculate analysis summary from inspection data
    fn calculate_analysis_summary(&self, inspection: &ActiveInspection, total_duration: Duration) -> AnalysisSummary {
        let total_tokens = inspection.current_stats.total_tokens;
        
        // Calculate average generation rate
        let avg_generation_rate = if total_duration.as_secs_f32() > 0.0 {
            total_tokens as f32 / total_duration.as_secs_f32()
        } else {
            0.0
        };
        
        // Calculate buffer utilization statistics
        let (avg_buffer_utilization, peak_buffer_utilization) = if !inspection.buffer_snapshots.is_empty() {
            let avg = inspection.buffer_snapshots.iter()
                .map(|s| s.input_utilization)
                .sum::<f32>() / inspection.buffer_snapshots.len() as f32;
            
            let peak = inspection.buffer_snapshots.iter()
                .map(|s| s.input_utilization)
                .fold(0.0f32, |acc, x| acc.max(x));
            
            (avg, peak)
        } else {
            (0.0, 0.0)
        };
        
        // Calculate latency statistics
        let (avg_latency, p95_latency, p99_latency) = if !inspection.performance_samples.is_empty() {
            let mut latencies: Vec<_> = inspection.performance_samples.iter()
                .map(|s| s.latency_p50)
                .collect();
            latencies.sort();
            
            let avg = Duration::from_nanos(
                (latencies.iter().map(|d| d.as_nanos()).sum::<u128>() / latencies.len() as u128).try_into().unwrap()
            );
            
            let p95_idx = (latencies.len() as f32 * 0.95) as usize;
            let p99_idx = (latencies.len() as f32 * 0.99) as usize;
            
            let p95 = latencies.get(p95_idx).copied().unwrap_or(Duration::from_nanos(0));
            let p99 = latencies.get(p99_idx).copied().unwrap_or(Duration::from_nanos(0));
            
            (avg, p95, p99)
        } else {
            (Duration::from_nanos(0), Duration::from_nanos(0), Duration::from_nanos(0))
        };
        
        // Calculate quality metrics
        let error_rate = if total_tokens > 0 {
            inspection.current_stats.error_count as f32 / total_tokens as f32
        } else {
            0.0
        };
        
        let completion_rate = 1.0 - error_rate; // Simplified calculation
        
        AnalysisSummary {
            total_duration,
            total_tokens,
            avg_generation_rate,
            peak_generation_rate: inspection.current_stats.peak_tokens_per_second,
            avg_buffer_utilization,
            peak_buffer_utilization,
            buffer_overflow_count: inspection.current_stats.total_overflows,
            buffer_underflow_count: inspection.current_stats.total_underflows,
            avg_latency,
            p95_latency,
            p99_latency,
            error_rate,
            completion_rate,
        }
    }

    /// Get analysis for an intent
    pub fn get_analysis(&self, intent_id: IntentId) -> Option<&TokenAnalysis> {
        self.completed_analyses.get(&intent_id)
    }

    /// Get all completed analyses
    pub fn get_all_analyses(&self) -> Vec<&TokenAnalysis> {
        self.completed_analyses.values().collect()
    }

    /// Clear old analyses
    pub fn clear_old_analysis(&mut self, max_to_keep: usize) {
        if self.completed_analyses.len() > max_to_keep {
            // Keep only the most recent analyses
            let mut analyses: Vec<_> = self.completed_analyses.iter().collect();
            analyses.sort_by(|a, b| b.1.completed_at.cmp(&a.1.completed_at));
            
            let to_remove: Vec<_> = analyses.iter()
                .skip(max_to_keep)
                .map(|(intent_id, _)| **intent_id)
                .collect();
            
            for intent_id in to_remove {
                self.completed_analyses.remove(&intent_id);
            }
            
            info!("Cleaned up old token analyses, keeping {} most recent", max_to_keep);
        }
    }

    /// Generate inspection report
    pub fn generate_inspection_report(&self, intent_id: IntentId) -> Option<String> {
        let analysis = self.get_analysis(intent_id)?;
        
        let mut report = String::new();
        report.push_str(&format!("Token Inspection Report\n"));
        report.push_str(&format!("======================\n\n"));
        report.push_str(&format!("Intent ID: {}\n", intent_id));
        report.push_str(&format!("Analysis ID: {}\n", analysis.analysis_id));
        report.push_str(&format!("Completed: {}\n\n", analysis.completed_at));
        
        report.push_str(&format!("Summary Statistics:\n"));
        report.push_str(&format!("  Duration: {}ms\n", analysis.summary.total_duration.as_millis()));
        report.push_str(&format!("  Total Tokens: {}\n", analysis.summary.total_tokens));
        report.push_str(&format!("  Avg Generation Rate: {:.2} tokens/sec\n", analysis.summary.avg_generation_rate));
        report.push_str(&format!("  Peak Generation Rate: {:.2} tokens/sec\n", analysis.summary.peak_generation_rate));
        report.push_str(&format!("  Avg Buffer Utilization: {:.1}%\n", analysis.summary.avg_buffer_utilization * 100.0));
        report.push_str(&format!("  Peak Buffer Utilization: {:.1}%\n", analysis.summary.peak_buffer_utilization * 100.0));
        report.push_str(&format!("  Buffer Overflows: {}\n", analysis.summary.buffer_overflow_count));
        report.push_str(&format!("  Buffer Underflows: {}\n", analysis.summary.buffer_underflow_count));
        report.push_str(&format!("  Avg Latency: {}ms\n", analysis.summary.avg_latency.as_millis()));
        report.push_str(&format!("  P95 Latency: {}ms\n", analysis.summary.p95_latency.as_millis()));
        report.push_str(&format!("  P99 Latency: {}ms\n", analysis.summary.p99_latency.as_millis()));
        report.push_str(&format!("  Error Rate: {:.2}%\n", analysis.summary.error_rate * 100.0));
        report.push_str(&format!("  Completion Rate: {:.2}%\n", analysis.summary.completion_rate * 100.0));
        
        Some(report)
    }
}

impl Default for TokenInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_inspector_creation() {
        let inspector = TokenInspector::new();
        assert!(inspector.config.enable_token_logging);
        assert!(inspector.active_inspections.is_empty());
    }

    #[tokio::test]
    async fn test_inspection_lifecycle() {
        let mut inspector = TokenInspector::new();
        let intent_id = Uuid::new_v4();
        
        // Start inspection
        let inspection_id = inspector.start_inspection(intent_id).await.unwrap();
        assert!(inspector.active_inspections.contains_key(&intent_id));
        
        // Record some events
        let token_data = TokenData {
            token_id: 1,
            content: "test".to_string(),
            length: 4,
            token_type: Some("word".to_string()),
            generation_time: Duration::from_millis(10),
        };
        
        inspector.record_token_event(intent_id, TokenEventType::TokenGenerated, Some(token_data)).await.unwrap();
        
        // Complete inspection
        let analysis = inspector.complete_inspection(intent_id).await.unwrap();
        assert_eq!(analysis.analysis_id, inspection_id);
        assert!(!inspector.active_inspections.contains_key(&intent_id));
        assert!(inspector.completed_analyses.contains_key(&intent_id));
    }

    #[tokio::test]
    async fn test_buffer_monitoring() {
        let mut inspector = TokenInspector::new();
        let intent_id = Uuid::new_v4();
        
        inspector.start_inspection(intent_id).await.unwrap();
        
        let buffer_state = BufferState {
            buffer_utilization: 0.75,
            overflow_count: 1,
            underflow_count: 0,
        };
        
        inspector.record_buffer_snapshot(intent_id, buffer_state).await.unwrap();
        
        let analysis = inspector.complete_inspection(intent_id).await.unwrap();
        assert!(!analysis.buffer_utilization.is_empty());
        assert_eq!(analysis.buffer_utilization[0].input_utilization, 0.75);
    }

    #[tokio::test]
    async fn test_performance_sampling() {
        let mut inspector = TokenInspector::new();
        let intent_id = Uuid::new_v4();
        
        inspector.start_inspection(intent_id).await.unwrap();
        
        let metrics = StreamMetrics {
            tokens_per_second: 50.0,
            latency_p50: Duration::from_millis(20),
            latency_p95: Duration::from_millis(50),
            latency_p99: Duration::from_millis(100),
            throughput_mbps: 1.0,
            error_rate: 0.01,
        };
        
        inspector.record_performance_sample(intent_id, metrics).await.unwrap();
        
        let analysis = inspector.complete_inspection(intent_id).await.unwrap();
        assert!(!analysis.performance_timeline.is_empty());
        assert_eq!(analysis.performance_timeline[0].tokens_per_second, 50.0);
    }

    #[test]
    fn test_analysis_summary_calculation() {
        let inspector = TokenInspector::new();
        let mut inspection = ActiveInspection {
            inspection_id: Uuid::new_v4(),
            intent_id: Uuid::new_v4(),
            start_time: Instant::now(),
            token_events: VecDeque::new(),
            buffer_snapshots: VecDeque::new(),
            performance_samples: VecDeque::new(),
            current_stats: InspectionStats {
                total_tokens: 100,
                peak_tokens_per_second: 75.0,
                total_overflows: 2,
                total_underflows: 1,
                error_count: 1,
                ..Default::default()
            },
        };
        
        // Add some buffer snapshots
        inspection.buffer_snapshots.push_back(BufferSnapshot {
            timestamp: Duration::from_millis(100),
            input_utilization: 0.5,
            output_utilization: 0.6,
            total_buffered_tokens: 10,
            overflow_count: 1,
            underflow_count: 0,
        });
        
        let summary = inspector.calculate_analysis_summary(&inspection, Duration::from_secs(2));
        
        assert_eq!(summary.total_tokens, 100);
        assert_eq!(summary.peak_generation_rate, 75.0);
        assert_eq!(summary.buffer_overflow_count, 2);
        assert_eq!(summary.buffer_underflow_count, 1);
        assert_eq!(summary.avg_generation_rate, 50.0); // 100 tokens / 2 seconds
    }
}