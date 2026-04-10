//! Performance metrics collection for parallel execution
//!
//! This module provides comprehensive performance measurement capabilities for the
//! D2 Parallelism Architecture. The metrics collector tracks all components of
//! execution time to calculate Net Speedup and make adaptive parallelism decisions.
//!
//! ## Design Principles
//!
//! 1. **Net Speedup Calculation**: Accounts for all overhead costs (ordering, sync, merge)
//! 2. **Percentile-Based Metrics**: Uses P50/P75 instead of averages for robustness
//! 3. **Comprehensive Tracking**: Measures sequential, parallel, and overhead times
//! 4. **Adaptive Decision Support**: Provides data for blacklist management
//!
//! **Design Reference:** D2 Parallelism Architecture - Performance Metrics section
//! **Requirements:** 12.1-12.6

use crate::parallelism::types::ExecutionMetrics;
use std::time::{Duration, Instant};

/// Execution phases for detailed performance tracking.
///
/// Each phase represents a distinct component of parallel execution that
/// contributes to the overall execution time. By tracking phases separately,
/// the adaptive decision engine can identify performance bottlenecks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPhase {
    /// Sequential execution (baseline for comparison)
    Sequential,
    /// Parallel computation (excluding overhead)
    Parallel,
    /// Overhead from maintaining stable index mapping
    Ordering,
    /// Cost of thread synchronization (barriers, locks, etc.)
    Synchronization,
    /// Cost of merging parallel results into final output
    Merge,
}

/// Trait for collecting and analyzing performance metrics.
///
/// The `MetricsCollector` provides a comprehensive interface for measuring
/// parallel execution performance. It tracks all components of execution time
/// and calculates the Net Speedup metric used by the adaptive decision engine.
///
/// # Net Speedup Formula
///
/// ```text
/// net_speedup = sequential_time / (parallel_time + ordering_overhead + sync_cost + merge_cost)
/// ```
///
/// If `net_speedup < 2.0x`, the adaptive system disables parallelism for that operation.
///
/// **Validates: Requirements 12.1-12.6**
pub trait MetricsCollector {
    /// Starts a new measurement session.
    ///
    /// This method initializes the collector for a new execution measurement.
    /// All previous measurements are cleared, and timing starts fresh.
    fn start_measurement(&mut self);

    /// Records the duration of a specific execution phase.
    ///
    /// # Arguments
    ///
    /// * `phase` - The execution phase that was measured
    /// * `duration` - The time taken for this phase
    ///
    /// **Validates: Requirement 12.2 (Phase Tracking)**
    fn record_phase(&mut self, phase: ExecutionPhase, duration: Duration);

    /// Calculates the net speedup including all overhead costs.
    ///
    /// This is the primary metric used by the adaptive decision engine.
    /// The calculation accounts for all overhead costs to provide an
    /// accurate measure of parallelism benefit.
    ///
    /// # Returns
    ///
    /// The net speedup ratio. Values >= 2.0 indicate beneficial parallelism.
    ///
    /// **Validates: Requirements 12.5, 4.1 (Net Speedup Calculation)**
    fn calculate_net_speedup(&self) -> f64;

    /// Generates a comprehensive execution metrics report.
    ///
    /// This method returns an `ExecutionMetrics` struct containing all
    /// measured timing data and calculated ratios.
    ///
    /// **Validates: Requirement 12.6 (Metrics Reporting)**
    fn report(&self) -> ExecutionMetrics;
}

/// Default implementation of metrics collector.
///
/// This collector provides comprehensive timing measurement with support for
/// multiple measurement sessions and statistical analysis. It tracks all
/// execution phases and calculates derived metrics.
///
/// # Usage Pattern
///
/// ```rust,ignore
/// let mut collector = DefaultMetricsCollector::new();
///
/// collector.start_measurement();
///
/// // Measure sequential execution
/// let start = Instant::now();
/// execute_sequential(&block, &data);
/// collector.record_phase(ExecutionPhase::Sequential, start.elapsed());
///
/// // Measure parallel execution phases
/// let start = Instant::now();
/// let results = execute_parallel(&block, &partitions);
/// collector.record_phase(ExecutionPhase::Parallel, start.elapsed());
///
/// // Calculate net speedup
/// let speedup = collector.calculate_net_speedup();
/// if speedup >= 2.0 {
///     // Parallelism is beneficial
/// }
/// ```
///
/// **Validates: Requirements 12.1-12.6**
#[derive(Debug, Clone)]
pub struct DefaultMetricsCollector {
    /// Current measurement session data
    current_session: MeasurementSession,
    /// Historical measurements for statistical analysis
    history: Vec<ExecutionMetrics>,
}

/// Single measurement session data.
#[derive(Debug, Clone, Default)]
struct MeasurementSession {
    sequential_time: Option<Duration>,
    parallel_time: Option<Duration>,
    ordering_overhead: Option<Duration>,
    sync_cost: Option<Duration>,
    merge_cost: Option<Duration>,
    measurement_start: Option<Instant>,
}

impl DefaultMetricsCollector {
    /// Creates a new metrics collector.
    pub fn new() -> Self {
        Self {
            current_session: MeasurementSession::default(),
            history: Vec::new(),
        }
    }

    /// Returns the number of historical measurements.
    pub fn measurement_count(&self) -> usize {
        self.history.len()
    }

    /// Calculates the P50 (median) net speedup from historical measurements.
    ///
    /// This method provides a robust statistical measure of parallelism
    /// performance that is less sensitive to outliers than the mean.
    ///
    /// **Validates: Requirement 4.3 (Percentile Metrics)**
    pub fn p50_net_speedup(&self) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }

        let mut speedups: Vec<f64> = self.history.iter().map(|m| m.net_speedup()).collect();

        speedups.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = speedups.len() / 2;
        Some(speedups[mid])
    }

    /// Calculates the P75 (75th percentile) net speedup from historical measurements.
    ///
    /// This method provides a conservative estimate of parallelism performance,
    /// useful for making robust adaptive decisions.
    ///
    /// **Validates: Requirement 4.3 (Percentile Metrics)**
    pub fn p75_net_speedup(&self) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }

        let mut speedups: Vec<f64> = self.history.iter().map(|m| m.net_speedup()).collect();

        speedups.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p75_idx = (speedups.len() as f64 * 0.75) as usize;
        let idx = p75_idx.min(speedups.len() - 1);
        Some(speedups[idx])
    }

    /// Adds the current measurement to historical data.
    ///
    /// This method finalizes the current measurement session and adds it
    /// to the historical data for statistical analysis.
    fn finalize_current_measurement(&mut self) {
        let metrics = self.report();
        self.history.push(metrics);
    }
}

impl Default for DefaultMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector for DefaultMetricsCollector {
    fn start_measurement(&mut self) {
        // Finalize previous measurement if it exists
        if self.current_session.measurement_start.is_some() {
            self.finalize_current_measurement();
        }

        // Start new measurement session
        self.current_session = MeasurementSession {
            measurement_start: Some(Instant::now()),
            ..Default::default()
        };
    }

    fn record_phase(&mut self, phase: ExecutionPhase, duration: Duration) {
        match phase {
            ExecutionPhase::Sequential => {
                self.current_session.sequential_time = Some(duration);
            }
            ExecutionPhase::Parallel => {
                self.current_session.parallel_time = Some(duration);
            }
            ExecutionPhase::Ordering => {
                self.current_session.ordering_overhead = Some(duration);
            }
            ExecutionPhase::Synchronization => {
                self.current_session.sync_cost = Some(duration);
            }
            ExecutionPhase::Merge => {
                self.current_session.merge_cost = Some(duration);
            }
        }
    }

    fn calculate_net_speedup(&self) -> f64 {
        let metrics = self.report();
        metrics.net_speedup()
    }

    fn report(&self) -> ExecutionMetrics {
        ExecutionMetrics {
            sequential_time: self
                .current_session
                .sequential_time
                .unwrap_or(Duration::ZERO),
            parallel_time: self.current_session.parallel_time.unwrap_or(Duration::ZERO),
            ordering_overhead: self
                .current_session
                .ordering_overhead
                .unwrap_or(Duration::ZERO),
            sync_cost: self.current_session.sync_cost.unwrap_or(Duration::ZERO),
            merge_cost: self.current_session.merge_cost.unwrap_or(Duration::ZERO),
        }
    }
}

/// Convenience function for measuring execution time of a closure.
///
/// This function provides a simple way to measure the execution time of
/// any operation, returning both the result and the elapsed time.
///
/// # Example
///
/// ```rust,ignore
/// let (result, duration) = measure_execution(|| {
///     expensive_computation()
/// });
///
/// collector.record_phase(ExecutionPhase::Parallel, duration);
/// ```
pub fn measure_execution<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ===== Basic Functionality Tests =====

    #[test]
    fn test_collector_creation() {
        let collector = DefaultMetricsCollector::new();
        assert_eq!(collector.measurement_count(), 0);
    }

    #[test]
    fn test_start_measurement() {
        let mut collector = DefaultMetricsCollector::new();
        collector.start_measurement();

        // Should have started a measurement session
        assert!(collector.current_session.measurement_start.is_some());
    }

    #[test]
    fn test_record_phase() {
        let mut collector = DefaultMetricsCollector::new();
        collector.start_measurement();

        let duration = Duration::from_millis(100);
        collector.record_phase(ExecutionPhase::Sequential, duration);

        let metrics = collector.report();
        assert_eq!(metrics.sequential_time, duration);
    }

    #[test]
    fn test_record_all_phases() {
        let mut collector = DefaultMetricsCollector::new();
        collector.start_measurement();

        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
        collector.record_phase(ExecutionPhase::Parallel, Duration::from_millis(300));
        collector.record_phase(ExecutionPhase::Ordering, Duration::from_millis(50));
        collector.record_phase(ExecutionPhase::Synchronization, Duration::from_millis(20));
        collector.record_phase(ExecutionPhase::Merge, Duration::from_millis(30));

        let metrics = collector.report();
        assert_eq!(metrics.sequential_time, Duration::from_millis(1000));
        assert_eq!(metrics.parallel_time, Duration::from_millis(300));
        assert_eq!(metrics.ordering_overhead, Duration::from_millis(50));
        assert_eq!(metrics.sync_cost, Duration::from_millis(20));
        assert_eq!(metrics.merge_cost, Duration::from_millis(30));
    }

    // ===== Net Speedup Calculation Tests =====

    #[test]
    fn test_calculate_net_speedup_beneficial() {
        let mut collector = DefaultMetricsCollector::new();
        collector.start_measurement();

        // Scenario: 1000ms sequential, 400ms total parallel (2.5x speedup)
        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
        collector.record_phase(ExecutionPhase::Parallel, Duration::from_millis(300));
        collector.record_phase(ExecutionPhase::Ordering, Duration::from_millis(50));
        collector.record_phase(ExecutionPhase::Synchronization, Duration::from_millis(20));
        collector.record_phase(ExecutionPhase::Merge, Duration::from_millis(30));

        let speedup = collector.calculate_net_speedup();
        assert!((speedup - 2.5).abs() < 0.01);
        assert!(speedup >= 2.0);
    }

    #[test]
    fn test_calculate_net_speedup_marginal() {
        let mut collector = DefaultMetricsCollector::new();
        collector.start_measurement();

        // Scenario: 1000ms sequential, 600ms total parallel (1.67x speedup)
        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
        collector.record_phase(ExecutionPhase::Parallel, Duration::from_millis(400));
        collector.record_phase(ExecutionPhase::Ordering, Duration::from_millis(100));
        collector.record_phase(ExecutionPhase::Synchronization, Duration::from_millis(50));
        collector.record_phase(ExecutionPhase::Merge, Duration::from_millis(50));

        let speedup = collector.calculate_net_speedup();
        assert!((speedup - 1.666).abs() < 0.01);
        assert!(speedup < 2.0);
    }

    #[test]
    fn test_calculate_net_speedup_zero_parallel_time() {
        let mut collector = DefaultMetricsCollector::new();
        collector.start_measurement();

        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
        // No parallel phases recorded

        let speedup = collector.calculate_net_speedup();
        assert_eq!(speedup, 0.0);
    }

    // ===== Historical Data Tests =====

    #[test]
    fn test_multiple_measurements() {
        let mut collector = DefaultMetricsCollector::new();

        // First measurement
        collector.start_measurement();
        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
        collector.record_phase(ExecutionPhase::Parallel, Duration::from_millis(400));

        // Second measurement (should finalize first)
        collector.start_measurement();
        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(800));
        collector.record_phase(ExecutionPhase::Parallel, Duration::from_millis(300));

        assert_eq!(collector.measurement_count(), 1); // First measurement finalized
    }

    #[test]
    fn test_p50_net_speedup() {
        let mut collector = DefaultMetricsCollector::new();

        // Add several measurements with different speedups
        let speedups = vec![1.5, 2.0, 2.5, 3.0, 1.8];

        for speedup in speedups {
            collector.start_measurement();
            collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
            let parallel_time = (1000.0 / speedup) as u64;
            collector.record_phase(
                ExecutionPhase::Parallel,
                Duration::from_millis(parallel_time),
            );
        }

        // Finalize last measurement
        collector.start_measurement();

        let p50 = collector.p50_net_speedup().unwrap();
        // Median of [1.5, 1.8, 2.0, 2.5, 3.0] is 2.0
        assert!((p50 - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_p75_net_speedup() {
        let mut collector = DefaultMetricsCollector::new();

        // Add several measurements
        let speedups = vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5];

        for speedup in speedups {
            collector.start_measurement();
            collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(1000));
            let parallel_time = (1000.0 / speedup) as u64;
            collector.record_phase(
                ExecutionPhase::Parallel,
                Duration::from_millis(parallel_time),
            );
        }

        // Finalize last measurement
        collector.start_measurement();

        let p75 = collector.p75_net_speedup().unwrap();
        // P75 of 8 values should be around index 6 (75% of 8 = 6)
        assert!(p75 >= 3.0);
    }

    #[test]
    fn test_empty_history_percentiles() {
        let collector = DefaultMetricsCollector::new();
        assert!(collector.p50_net_speedup().is_none());
        assert!(collector.p75_net_speedup().is_none());
    }

    // ===== Utility Function Tests =====

    #[test]
    fn test_measure_execution() {
        let (result, duration) = measure_execution(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
        assert!(duration < Duration::from_millis(50)); // Should be reasonably close
    }

    // ===== Integration Tests =====

    #[test]
    fn test_trait_implementation() {
        let mut collector = DefaultMetricsCollector::new();
        let _: &mut dyn MetricsCollector = &mut collector;

        // Verify trait methods work
        collector.start_measurement();
        collector.record_phase(ExecutionPhase::Sequential, Duration::from_millis(100));
        let speedup = collector.calculate_net_speedup();
        let _metrics = collector.report();

        assert_eq!(speedup, 0.0); // No parallel time recorded
    }

    #[test]
    fn test_execution_phase_enum() {
        // Test that ExecutionPhase enum has all required variants
        let phases = vec![
            ExecutionPhase::Sequential,
            ExecutionPhase::Parallel,
            ExecutionPhase::Ordering,
            ExecutionPhase::Synchronization,
            ExecutionPhase::Merge,
        ];

        // Test Debug trait
        for phase in &phases {
            let debug_str = format!("{:?}", phase);
            assert!(!debug_str.is_empty());
        }

        // Test PartialEq trait
        assert_eq!(ExecutionPhase::Sequential, ExecutionPhase::Sequential);
        assert_ne!(ExecutionPhase::Sequential, ExecutionPhase::Parallel);
    }
}
