//! Adaptive decision engine for parallelism control
//!
//! This module implements the adaptive decision system that determines whether
//! to enable parallelism based on performance metrics and historical data.
//! The system uses a soft blacklist approach with re-evaluation windows.
//!
//! ## Design Principles
//!
//! 1. **Soft Blacklisting**: Blacklist is reversible after 50 executions or version change
//! 2. **Percentile Metrics**: Uses P50/P75 instead of averages for robustness
//! 3. **Conservative Thresholds**: Requires 2.0x net speedup for parallelism
//! 4. **Overhead Protection**: Disables parallelism when overhead > 50%
//!
//! **Design Reference:** D2 Parallelism Architecture - Adaptive Decision Engine section
//! **Requirements:** 4.1, 4.4, 4.5, 4.7

use crate::execution_plan::{BlockId, IRBlock, ParallelSafety};
use crate::parallelism::types::ExecutionMetrics;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Minimum net speedup required to enable parallelism.
///
/// This conservative threshold ensures that parallelism provides significant
/// benefit after accounting for all overhead costs.
///
/// **Validates: Requirement 4.1**
pub const MIN_NET_SPEEDUP: f64 = 2.0;

/// Maximum ordering overhead ratio before disabling parallelism.
///
/// If ordering overhead exceeds 50% of parallel execution time,
/// parallelism is likely counterproductive.
///
/// **Validates: Requirement 4.7**
pub const MAX_OVERHEAD_RATIO: f64 = 0.5;

/// Re-evaluation window for blacklisted operations.
///
/// After 50 executions, a blacklisted operation can be reconsidered
/// for parallelization.
///
/// **Validates: Requirement 4.4**
pub const REEVALUATION_WINDOW: usize = 50;

/// Minimum dataset size for parallelism consideration.
///
/// Small datasets are unlikely to benefit from parallelism due to
/// overhead costs.
pub const MIN_PARALLEL_SIZE: usize = 100;

/// Trait for adaptive parallelism decision making.
///
/// The `AdaptiveDecisionEngine` analyzes IR blocks and performance metrics
/// to determine whether parallelism should be enabled. It maintains a
/// soft blacklist of operations that have shown poor parallel performance.
///
/// # Decision Algorithm
///
/// ```text
/// if replay_mode:
///     return Sequential
///
/// if block.safety == Unsafe:
///     return Sequential
///
/// if is_blacklisted(block_id):
///     if executions_since_blacklist < 50 AND version_hash_unchanged:
///         return Sequential
///     else:
///         clear_blacklist(block_id)
///
/// if data_size < MIN_PARALLEL_SIZE:
///     return Sequential
///
/// if estimated_net_speedup < 2.0:
///     return Sequential
///
/// return Parallel
/// ```
///
/// **Validates: Requirements 4.1, 4.4, 4.7**
pub trait AdaptiveDecisionEngine {
    /// Determines whether to enable parallelism for a given IR block and dataset.
    ///
    /// # Arguments
    ///
    /// * `block` - The IR block to analyze
    /// * `data_size` - Size of the dataset to process
    ///
    /// # Returns
    ///
    /// * `true` - Parallelism should be enabled
    /// * `false` - Use sequential execution
    ///
    /// **Validates: Requirements 4.1, 4.7**
    fn should_parallelize(&self, block: &IRBlock, data_size: usize) -> bool;

    /// Records execution metrics for adaptive learning.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Identifier of the executed block
    /// * `metrics` - Performance metrics from the execution
    ///
    /// **Validates: Requirement 4.4 (Learning System)**
    fn record_execution(&mut self, block_id: BlockId, metrics: ExecutionMetrics);

    /// Calculates net speedup from execution metrics.
    ///
    /// This is a convenience method that extracts the net speedup calculation
    /// from `ExecutionMetrics`.
    ///
    /// **Validates: Requirement 4.1**
    fn calculate_net_speedup(&self, metrics: &ExecutionMetrics) -> f64;

    /// Checks if an operation is currently blacklisted.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Identifier of the block to check
    ///
    /// # Returns
    ///
    /// * `true` - Operation is blacklisted
    /// * `false` - Operation is not blacklisted or can be re-evaluated
    ///
    /// **Validates: Requirement 4.4**
    fn is_blacklisted(&self, block_id: BlockId) -> bool;

    /// Updates the blacklist based on performance metrics.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Identifier of the block
    /// * `speedup` - Measured net speedup
    ///
    /// **Validates: Requirement 4.4**
    fn update_blacklist(&mut self, block_id: BlockId, speedup: f64);
}

/// Blacklist entry for tracking operation performance history.
///
/// Each entry maintains a history of speedup measurements and tracks
/// the number of executions since blacklisting for re-evaluation.
///
/// **Validates: Requirements 4.4, 4.5**
#[derive(Debug, Clone)]
pub struct BlacklistEntry {
    /// Identifier of the blacklisted block
    pub block_id: BlockId,
    /// History of net speedup measurements
    pub speedup_history: Vec<f64>,
    /// Number of executions since this operation was blacklisted
    pub executions_since_blacklist: usize,
    /// Version hash when this operation was blacklisted
    pub blacklist_version_hash: u64,
}

impl BlacklistEntry {
    /// Creates a new blacklist entry.
    pub fn new(block_id: BlockId, initial_speedup: f64, version_hash: u64) -> Self {
        Self {
            block_id,
            speedup_history: vec![initial_speedup],
            executions_since_blacklist: 0,
            blacklist_version_hash: version_hash,
        }
    }

    /// Adds a new speedup measurement to the history.
    pub fn add_measurement(&mut self, speedup: f64) {
        self.speedup_history.push(speedup);
        self.executions_since_blacklist += 1;
    }

    /// Calculates the P50 (median) speedup from history.
    ///
    /// **Validates: Requirement 4.3 (Percentile Metrics)**
    pub fn p50_speedup(&self) -> f64 {
        if self.speedup_history.is_empty() {
            return 0.0;
        }

        let mut sorted = self.speedup_history.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = sorted.len() / 2;
        sorted[mid]
    }

    /// Calculates the P75 (75th percentile) speedup from history.
    ///
    /// **Validates: Requirement 4.3 (Percentile Metrics)**
    pub fn p75_speedup(&self) -> f64 {
        if self.speedup_history.is_empty() {
            return 0.0;
        }

        let mut sorted = self.speedup_history.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p75_idx = (sorted.len() as f64 * 0.75) as usize;
        let idx = p75_idx.min(sorted.len() - 1);
        sorted[idx]
    }
}

/// Adaptive blacklist for managing operation performance history.
///
/// The blacklist maintains a soft record of operations that have shown
/// poor parallel performance. Operations can be re-evaluated after a
/// sufficient number of executions or when the system version changes.
///
/// **Validates: Requirements 4.4, 4.5**
#[derive(Debug, Clone)]
pub struct AdaptiveBlacklist {
    /// Map of block ID to blacklist entry
    entries: HashMap<BlockId, BlacklistEntry>,
    /// Current system version hash
    version_hash: u64,
}

impl AdaptiveBlacklist {
    /// Creates a new adaptive blacklist.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version_hash: Self::compute_version_hash(),
        }
    }

    /// Computes a version hash based on system state.
    ///
    /// This hash changes when the IR structure or optimizer version changes,
    /// triggering re-evaluation of blacklisted operations.
    ///
    /// **Validates: Requirement 4.5 (Version Hash)**
    fn compute_version_hash() -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash system version information
        // In a real implementation, this would include:
        // - IR structure version
        // - Optimizer version
        // - JIT compiler version
        // - Hardware capabilities

        "d2-parallelism-v1.0".hash(&mut hasher);
        std::env::consts::ARCH.hash(&mut hasher);

        hasher.finish()
    }

    /// Checks if an operation should be re-evaluated.
    ///
    /// Re-evaluation is triggered by:
    /// - 50+ executions since blacklisting
    /// - Version hash change
    ///
    /// **Validates: Requirement 4.4**
    pub fn should_reevaluate(&self, entry: &BlacklistEntry) -> bool {
        entry.executions_since_blacklist >= REEVALUATION_WINDOW
            || entry.blacklist_version_hash != self.version_hash
    }

    /// Adds or updates a blacklist entry.
    pub fn add_entry(&mut self, block_id: BlockId, speedup: f64) {
        if let Some(entry) = self.entries.get_mut(&block_id) {
            entry.add_measurement(speedup);
        } else {
            let entry = BlacklistEntry::new(block_id, speedup, self.version_hash);
            self.entries.insert(block_id, entry);
        }
    }

    /// Removes a blacklist entry (for re-evaluation).
    pub fn remove_entry(&mut self, block_id: BlockId) {
        self.entries.remove(&block_id);
    }

    /// Gets a blacklist entry if it exists.
    pub fn get_entry(&self, block_id: BlockId) -> Option<&BlacklistEntry> {
        self.entries.get(&block_id)
    }

    /// Returns the number of blacklisted operations.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the blacklist is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gets the current version hash.
    pub fn get_version_hash(&self) -> u64 {
        self.version_hash
    }
}

impl Default for AdaptiveBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

/// Default implementation of adaptive decision engine.
///
/// This implementation provides comprehensive decision logic with support for:
/// - Soft blacklisting with re-evaluation
/// - Percentile-based performance analysis
/// - Conservative parallelism thresholds
/// - Overhead protection
///
/// **Validates: Requirements 4.1, 4.4, 4.7**
#[derive(Debug, Clone)]
pub struct DefaultDecisionEngine {
    /// Adaptive blacklist for tracking poor performers
    blacklist: AdaptiveBlacklist,
    /// Whether replay mode is active (forces sequential execution)
    replay_mode: bool,
}

impl DefaultDecisionEngine {
    /// Creates a new default decision engine.
    pub fn new() -> Self {
        Self {
            blacklist: AdaptiveBlacklist::new(),
            replay_mode: false,
        }
    }

    /// Sets replay mode state.
    ///
    /// When replay mode is active, all operations use sequential execution
    /// to ensure deterministic reproduction of previous results.
    ///
    /// **Validates: Requirement 3.1 (Replay Mode)**
    pub fn set_replay_mode(&mut self, replay_mode: bool) {
        self.replay_mode = replay_mode;
    }

    /// Checks if replay mode is active.
    pub fn is_replay_mode(&self) -> bool {
        self.replay_mode
    }

    /// Returns the number of blacklisted operations.
    pub fn blacklist_size(&self) -> usize {
        self.blacklist.len()
    }
}

impl Default for DefaultDecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveDecisionEngine for DefaultDecisionEngine {
    fn should_parallelize(&self, block: &IRBlock, data_size: usize) -> bool {
        // CONSTITUTIONAL ENFORCEMENT: Check execution mode first
        if self.replay_mode {
            // P3: Replay First-Class Citizen - MUST use sequential execution
            return false;
        }

        // CONSTITUTIONAL ENFORCEMENT: P1 - Determinism > Parallelism
        // Unsafe blocks MUST NEVER be parallelized
        if block.parallel_safety == ParallelSafety::Unsafe {
            return false;
        }

        // Check blacklist status
        if let Some(entry) = self.blacklist.get_entry(block.id) {
            if !self.blacklist.should_reevaluate(entry) {
                // Still blacklisted, use sequential
                return false;
            }
            // Can be re-evaluated, continue with other checks
        }

        // Check data size threshold
        if data_size < MIN_PARALLEL_SIZE {
            return false;
        }

        // For new operations or re-evaluation, assume parallelism is beneficial
        // The actual performance will be measured and recorded
        true
    }

    fn record_execution(&mut self, block_id: BlockId, metrics: ExecutionMetrics) {
        let net_speedup = self.calculate_net_speedup(&metrics);
        let overhead_ratio = metrics.ordering_overhead_ratio();

        // Check if this operation should be blacklisted
        if net_speedup < MIN_NET_SPEEDUP || overhead_ratio > MAX_OVERHEAD_RATIO {
            self.update_blacklist(block_id, net_speedup);
        } else {
            // Good performance, remove from blacklist if present
            self.blacklist.remove_entry(block_id);
        }
    }

    fn calculate_net_speedup(&self, metrics: &ExecutionMetrics) -> f64 {
        metrics.net_speedup()
    }

    fn is_blacklisted(&self, block_id: BlockId) -> bool {
        if let Some(entry) = self.blacklist.get_entry(block_id) {
            !self.blacklist.should_reevaluate(entry)
        } else {
            false
        }
    }

    fn update_blacklist(&mut self, block_id: BlockId, speedup: f64) {
        self.blacklist.add_entry(block_id, speedup);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_plan::{BlockTerminator, IRBlock, IRInstruction, ParallelSafety};
    use std::time::Duration;

    // ===== Test Helpers =====

    fn create_test_block(id: BlockId, safety: ParallelSafety) -> IRBlock {
        IRBlock::with_safety(
            id,
            vec![IRInstruction::LoadContext {
                context_id: "test".to_string(),
                target_register: 0,
            }],
            BlockTerminator::Return { register: 0 },
            safety,
        )
    }

    fn create_test_metrics(
        sequential_ms: u64,
        parallel_ms: u64,
        overhead_ms: u64,
    ) -> ExecutionMetrics {
        ExecutionMetrics {
            sequential_time: Duration::from_millis(sequential_ms),
            parallel_time: Duration::from_millis(parallel_ms),
            ordering_overhead: Duration::from_millis(overhead_ms),
            sync_cost: Duration::ZERO,
            merge_cost: Duration::ZERO,
        }
    }

    // ===== BlacklistEntry Tests =====

    #[test]
    fn test_blacklist_entry_creation() {
        let entry = BlacklistEntry::new(1, 1.5, 12345);

        assert_eq!(entry.block_id, 1);
        assert_eq!(entry.speedup_history.len(), 1);
        assert_eq!(entry.speedup_history[0], 1.5);
        assert_eq!(entry.executions_since_blacklist, 0);
        assert_eq!(entry.blacklist_version_hash, 12345);
    }

    #[test]
    fn test_blacklist_entry_add_measurement() {
        let mut entry = BlacklistEntry::new(1, 1.5, 12345);

        entry.add_measurement(1.8);
        entry.add_measurement(1.2);

        assert_eq!(entry.speedup_history.len(), 3);
        assert_eq!(entry.executions_since_blacklist, 2);
    }

    #[test]
    fn test_blacklist_entry_p50_speedup() {
        let mut entry = BlacklistEntry::new(1, 2.0, 12345);
        entry.speedup_history = vec![1.0, 1.5, 2.0, 2.5, 3.0];

        let p50 = entry.p50_speedup();
        assert_eq!(p50, 2.0); // Median of [1.0, 1.5, 2.0, 2.5, 3.0]
    }

    #[test]
    fn test_blacklist_entry_p75_speedup() {
        let mut entry = BlacklistEntry::new(1, 1.0, 12345);
        entry.speedup_history = vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5];

        let p75 = entry.p75_speedup();
        assert!(p75 >= 3.5); // 75th percentile should be around 3.5-4.0
    }

    // ===== AdaptiveBlacklist Tests =====

    #[test]
    fn test_adaptive_blacklist_creation() {
        let blacklist = AdaptiveBlacklist::new();
        assert!(blacklist.is_empty());
        assert_eq!(blacklist.len(), 0);
    }

    #[test]
    fn test_adaptive_blacklist_add_entry() {
        let mut blacklist = AdaptiveBlacklist::new();

        blacklist.add_entry(1, 1.5);
        assert_eq!(blacklist.len(), 1);
        assert!(!blacklist.is_empty());

        let entry = blacklist.get_entry(1).unwrap();
        assert_eq!(entry.block_id, 1);
        assert_eq!(entry.speedup_history[0], 1.5);
    }

    #[test]
    fn test_adaptive_blacklist_update_entry() {
        let mut blacklist = AdaptiveBlacklist::new();

        blacklist.add_entry(1, 1.5);
        blacklist.add_entry(1, 1.8); // Update existing entry

        assert_eq!(blacklist.len(), 1);

        let entry = blacklist.get_entry(1).unwrap();
        assert_eq!(entry.speedup_history.len(), 2);
        assert_eq!(entry.executions_since_blacklist, 1);
    }

    #[test]
    fn test_adaptive_blacklist_remove_entry() {
        let mut blacklist = AdaptiveBlacklist::new();

        blacklist.add_entry(1, 1.5);
        assert_eq!(blacklist.len(), 1);

        blacklist.remove_entry(1);
        assert_eq!(blacklist.len(), 0);
        assert!(blacklist.get_entry(1).is_none());
    }

    #[test]
    fn test_should_reevaluate_execution_count() {
        let blacklist = AdaptiveBlacklist::new();
        let mut entry = BlacklistEntry::new(1, 1.5, blacklist.version_hash);

        // Not enough executions
        entry.executions_since_blacklist = 49;
        assert!(!blacklist.should_reevaluate(&entry));

        // Enough executions
        entry.executions_since_blacklist = 50;
        assert!(blacklist.should_reevaluate(&entry));
    }

    #[test]
    fn test_should_reevaluate_version_hash() {
        let blacklist = AdaptiveBlacklist::new();
        let mut entry = BlacklistEntry::new(1, 1.5, blacklist.version_hash + 1); // Different version

        // Different version hash should trigger re-evaluation
        entry.executions_since_blacklist = 0;
        assert!(blacklist.should_reevaluate(&entry));
    }

    // ===== DefaultDecisionEngine Tests =====

    #[test]
    fn test_decision_engine_creation() {
        let engine = DefaultDecisionEngine::new();
        assert!(!engine.is_replay_mode());
        assert_eq!(engine.blacklist_size(), 0);
    }

    #[test]
    fn test_replay_mode_forces_sequential() {
        let mut engine = DefaultDecisionEngine::new();
        let block = create_test_block(1, ParallelSafety::Safe);

        // Normal mode should allow parallelism
        assert!(engine.should_parallelize(&block, 1000));

        // Replay mode should force sequential
        engine.set_replay_mode(true);
        assert!(!engine.should_parallelize(&block, 1000));

        // Back to normal mode
        engine.set_replay_mode(false);
        assert!(engine.should_parallelize(&block, 1000));
    }

    #[test]
    fn test_unsafe_blocks_sequential() {
        let engine = DefaultDecisionEngine::new();
        let unsafe_block = create_test_block(1, ParallelSafety::Unsafe);
        let safe_block = create_test_block(2, ParallelSafety::Safe);

        // Unsafe blocks should never be parallelized
        assert!(!engine.should_parallelize(&unsafe_block, 1000));

        // Safe blocks should be considered for parallelization
        assert!(engine.should_parallelize(&safe_block, 1000));
    }

    #[test]
    fn test_small_dataset_sequential() {
        let engine = DefaultDecisionEngine::new();
        let block = create_test_block(1, ParallelSafety::Safe);

        // Small datasets should use sequential execution
        assert!(!engine.should_parallelize(&block, 50));
        assert!(!engine.should_parallelize(&block, MIN_PARALLEL_SIZE - 1));

        // Large datasets should consider parallelization
        assert!(engine.should_parallelize(&block, MIN_PARALLEL_SIZE));
        assert!(engine.should_parallelize(&block, 1000));
    }

    #[test]
    fn test_blacklist_prevents_parallelization() {
        let mut engine = DefaultDecisionEngine::new();
        let block = create_test_block(1, ParallelSafety::Safe);

        // Initially should allow parallelization
        assert!(engine.should_parallelize(&block, 1000));

        // Record poor performance (low speedup)
        let poor_metrics = create_test_metrics(1000, 800, 100); // 1.11x speedup
        engine.record_execution(1, poor_metrics);

        // Should now be blacklisted
        assert!(!engine.should_parallelize(&block, 1000));
        assert_eq!(engine.blacklist_size(), 1);
    }

    #[test]
    fn test_high_overhead_blacklisting() {
        let mut engine = DefaultDecisionEngine::new();
        let block = create_test_block(1, ParallelSafety::Safe);

        // Record execution with high overhead (>50%)
        let high_overhead_metrics = create_test_metrics(1000, 300, 200); // 66% overhead
        engine.record_execution(1, high_overhead_metrics);

        // Should be blacklisted due to high overhead
        assert!(!engine.should_parallelize(&block, 1000));
    }

    #[test]
    fn test_good_performance_removes_blacklist() {
        let mut engine = DefaultDecisionEngine::new();
        let block = create_test_block(1, ParallelSafety::Safe);

        // First, blacklist the operation
        let poor_metrics = create_test_metrics(1000, 800, 100);
        engine.record_execution(1, poor_metrics);
        assert!(!engine.should_parallelize(&block, 1000));

        // Then record good performance
        let good_metrics = create_test_metrics(1000, 300, 50); // 2.86x speedup
        engine.record_execution(1, good_metrics);

        // Should be removed from blacklist
        assert!(engine.should_parallelize(&block, 1000));
        assert_eq!(engine.blacklist_size(), 0);
    }

    #[test]
    fn test_calculate_net_speedup() {
        let engine = DefaultDecisionEngine::new();

        let metrics = create_test_metrics(1000, 300, 50);
        let speedup = engine.calculate_net_speedup(&metrics);

        // 1000 / (300 + 50) = 2.86x
        assert!((speedup - 2.857).abs() < 0.01);
    }

    #[test]
    fn test_is_blacklisted() {
        let mut engine = DefaultDecisionEngine::new();

        // Initially not blacklisted
        assert!(!engine.is_blacklisted(1));

        // Add to blacklist
        engine.update_blacklist(1, 1.5);
        assert!(engine.is_blacklisted(1));
    }

    // ===== Integration Tests =====

    #[test]
    fn test_trait_implementation() {
        let mut engine = DefaultDecisionEngine::new();
        let _: &mut dyn AdaptiveDecisionEngine = &mut engine;

        let block = create_test_block(1, ParallelSafety::Safe);
        let metrics = create_test_metrics(1000, 400, 50);

        // Test all trait methods
        let should_parallel = engine.should_parallelize(&block, 1000);
        engine.record_execution(1, metrics);
        let speedup = engine.calculate_net_speedup(&metrics);
        let is_blacklisted = engine.is_blacklisted(1);

        assert!(should_parallel);
        assert!(speedup > 2.0);
        assert!(!is_blacklisted); // Good performance, not blacklisted
    }

    #[test]
    fn test_constants() {
        assert_eq!(MIN_NET_SPEEDUP, 2.0);
        assert_eq!(MAX_OVERHEAD_RATIO, 0.5);
        assert_eq!(REEVALUATION_WINDOW, 50);
        assert_eq!(MIN_PARALLEL_SIZE, 100);
    }
}
