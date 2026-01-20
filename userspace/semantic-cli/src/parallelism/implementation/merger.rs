//! Deterministic merger for parallel execution results
//!
//! This module implements the deterministic merging strategy for parallel execution results.
//! The merger uses a Stable Index Map to reconstruct logical ordering from parallel results,
//! ensuring that the output is identical to sequential execution regardless of thread
//! scheduling or completion order.
//!
//! ## Design Principles
//!
//! 1. **Stable Index Mapping**: Logical ordering is determined at partition time, not merge time
//! 2. **Cache-Line Safety**: Uses chunk-local buffers followed by single-threaded merge (ADR-3)
//! 3. **Completeness Verification**: Verifies all indices are present before returning results
//! 4. **No Sorting**: Index-based merge avoids sorting overhead
//!
//! **Design Reference:** D2 Parallelism Architecture - Deterministic Merger section
//! **Requirements:** 2.2, 2.4, 7.1, 7.2, 7.3
//! **ADR:** ADR-3 (Chunk-Local Buffers for Cache-Line Safety)

use crate::bcib::Value;
use crate::parallelism::error::ParallelismError;

/// Trait for deterministic merging of parallel execution results.
///
/// The merger takes indexed results from parallel workers and reconstructs the
/// logical ordering using the Stable Index Map pattern. This ensures deterministic
/// output regardless of thread scheduling or completion order.
///
/// # Design Pattern: Stable Index Map
///
/// ```text
/// Input:  [e0, e1, e2, e3, e4, e5, e6, e7]
///          |         |         |         |
/// Partition: [0,1]    [2,3]     [4,5]     [6,7]
///          |         |         |         |
/// Workers:  W1        W2        W3        W4
///          |         |         |         |
/// Index:   [0,1]    [2,3]     [4,5]     [6,7]  <- Stable mapping
///          |         |         |         |
/// Merge:  [r0, r1, r2, r3, r4, r5, r6, r7]  <- Deterministic order
/// ```
///
/// **Validates: Requirements 2.2, 2.4**
pub trait DeterministicMerger {
    /// Merges indexed results from parallel workers into a deterministically ordered output.
    ///
    /// # Arguments
    ///
    /// * `results` - Vector of (index, value) pairs from parallel workers
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Value>)` - Merged results in logical order
    /// * `Err(ParallelismError)` - If indices are incomplete or invalid
    ///
    /// # Guarantees
    ///
    /// - Output order is determined by indices, not arrival order
    /// - All indices from 0 to max_index must be present
    /// - Same indices always produce same output order
    ///
    /// **Validates: Requirement 2.2 (Stable Index Map)**
    fn merge(&self, results: Vec<(usize, Value)>) -> Result<Vec<Value>, ParallelismError>;
    
    /// Verifies that all expected indices are present in the results.
    ///
    /// # Arguments
    ///
    /// * `results` - Slice of (index, value) pairs to verify
    /// * `expected_size` - Expected number of results (0 to expected_size-1)
    ///
    /// # Returns
    ///
    /// * `true` - All indices from 0 to expected_size-1 are present
    /// * `false` - Some indices are missing
    ///
    /// **Validates: Requirement 2.4 (Completeness Verification)**
    fn verify_completeness(&self, results: &[(usize, Value)], expected_size: usize) -> bool;
}

/// Implementation of deterministic merger using chunk-local buffer strategy.
///
/// This implementation follows ADR-3 (Chunk-Local Buffers for Cache-Line Safety):
/// - Workers write to thread-local Vec<(idx, Value)> buffers
/// - Merge is single-threaded to avoid false sharing
/// - No padding or alignment complexity required
///
/// # Cache-Line Safety
///
/// The chunk-local buffer strategy guarantees cache-line safety:
/// 1. Each worker writes only to its own local buffer (no contention)
/// 2. Merge phase is single-threaded (no concurrent writes)
/// 3. No false sharing between workers
///
/// # Performance Characteristics
///
/// - Time Complexity: O(n) where n is total number of results
/// - Space Complexity: O(n) for pre-allocated output vector
/// - Cache Efficiency: Sequential writes during merge phase
///
/// **Validates: Requirements 2.2, 2.4, 7.1, 7.2, 7.3**
/// **ADR: ADR-3 (Chunk-Local Buffers for Cache-Line Safety)**
#[derive(Debug, Clone, Default)]
pub struct StableIndexMerger;

impl StableIndexMerger {
    /// Creates a new StableIndexMerger instance.
    pub fn new() -> Self {
        Self
    }
}

impl DeterministicMerger for StableIndexMerger {
    fn merge(&self, results: Vec<(usize, Value)>) -> Result<Vec<Value>, ParallelismError> {
        // Handle empty results
        if results.is_empty() {
            return Ok(Vec::new());
        }
        
        // Find the maximum index to determine output size
        let max_idx = results.iter()
            .map(|(idx, _)| *idx)
            .max()
            .unwrap(); // Safe because we checked results is not empty
        
        let expected_size = max_idx + 1;
        
        // Pre-allocate output vector with None values
        // This is the single-threaded merge phase (no false sharing)
        let mut output: Vec<Option<Value>> = vec![None; expected_size];
        
        // Single-threaded merge: place each result at its index position
        // This is cache-friendly because we write sequentially to the output vector
        for (idx, value) in results {
            if idx >= expected_size {
                return Err(ParallelismError::ExecutionError {
                    worker_id: None,
                    partition_start: None,
                    partition_end: None,
                    message: format!(
                        "Invalid index: {} exceeds maximum index {}",
                        idx, max_idx
                    ),
                    source: None,
                });
            }
            
            // Check for duplicate indices
            if output[idx].is_some() {
                return Err(ParallelismError::ExecutionError {
                    worker_id: None,
                    partition_start: None,
                    partition_end: None,
                    message: format!("Duplicate index: {} appears multiple times", idx),
                    source: None,
                });
            }
            
            output[idx] = Some(value);
        }
        
        // Verify completeness: check if any indices are missing
        for (idx, opt) in output.iter().enumerate() {
            if opt.is_none() {
                return Err(ParallelismError::ExecutionError {
                    worker_id: None,
                    partition_start: None,
                    partition_end: None,
                    message: format!(
                        "Incomplete results: missing result at index {}",
                        idx
                    ),
                    source: None,
                });
            }
        }
        
        // Convert Option<Value> to Value
        // This should never fail because we verified completeness above
        Ok(output.into_iter()
            .map(|opt| opt.unwrap())
            .collect())
    }
    
    fn verify_completeness(&self, results: &[(usize, Value)], expected_size: usize) -> bool {
        // Check that we have the expected number of results
        if results.len() != expected_size {
            return false;
        }
        
        // If expected_size is 0, we're done
        if expected_size == 0 {
            return true;
        }
        
        // Create a boolean array to track which indices are present
        let mut present = vec![false; expected_size];
        
        // Mark each index as present
        for (idx, _) in results {
            if *idx >= expected_size {
                return false; // Index out of bounds
            }
            if present[*idx] {
                return false; // Duplicate index
            }
            present[*idx] = true;
        }
        
        // Verify all indices are present
        present.iter().all(|&p| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;

    // ===== Correctness Tests =====

    #[test]
    fn test_merge_empty_results() {
        let merger = StableIndexMerger::new();
        let results = vec![];
        
        let merged = merger.merge(results).unwrap();
        assert_eq!(merged.len(), 0);
    }

    #[test]
    fn test_merge_single_element() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(42.0)),
        ];
        
        let merged = merger.merge(results).unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], Value::Number(42.0));
    }

    #[test]
    fn test_merge_ordered_results() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (2, Value::Number(3.0)),
            (3, Value::Number(4.0)),
        ];
        
        let merged = merger.merge(results).unwrap();
        assert_eq!(merged.len(), 4);
        assert_eq!(merged[0], Value::Number(1.0));
        assert_eq!(merged[1], Value::Number(2.0));
        assert_eq!(merged[2], Value::Number(3.0));
        assert_eq!(merged[3], Value::Number(4.0));
    }

    #[test]
    fn test_merge_unordered_results() {
        // This is the key test: results arrive in arbitrary order,
        // but merge reconstructs logical order using indices
        let merger = StableIndexMerger::new();
        let results = vec![
            (2, Value::Number(3.0)),
            (0, Value::Number(1.0)),
            (3, Value::Number(4.0)),
            (1, Value::Number(2.0)),
        ];
        
        let merged = merger.merge(results).unwrap();
        assert_eq!(merged.len(), 4);
        assert_eq!(merged[0], Value::Number(1.0));
        assert_eq!(merged[1], Value::Number(2.0));
        assert_eq!(merged[2], Value::Number(3.0));
        assert_eq!(merged[3], Value::Number(4.0));
    }

    #[test]
    fn test_merge_large_dataset() {
        let merger = StableIndexMerger::new();
        
        // Create 1000 results in reverse order
        let mut results = Vec::new();
        for i in (0..1000).rev() {
            results.push((i, Value::Number(i as f64)));
        }
        
        let merged = merger.merge(results).unwrap();
        assert_eq!(merged.len(), 1000);
        
        // Verify correct ordering
        for (i, value) in merged.iter().enumerate() {
            assert_eq!(*value, Value::Number(i as f64));
        }
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_merge_missing_index() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            // Missing index 2
            (3, Value::Number(4.0)),
        ];
        
        let result = merger.merge(results);
        assert!(result.is_err());
        
        match result {
            Err(ParallelismError::ExecutionError { message, .. }) => {
                assert!(message.contains("Incomplete results"));
            }
            _ => panic!("Expected ExecutionError for missing index"),
        }
    }

    #[test]
    fn test_merge_duplicate_index() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (1, Value::Number(3.0)), // Duplicate index 1
            (2, Value::Number(4.0)),
        ];
        
        let result = merger.merge(results);
        assert!(result.is_err());
        
        match result {
            Err(ParallelismError::ExecutionError { message, .. }) => {
                assert!(message.contains("Duplicate index"));
            }
            _ => panic!("Expected ExecutionError for duplicate index"),
        }
    }

    #[test]
    fn test_merge_invalid_index() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (2, Value::Number(3.0)),
            (10, Value::Number(4.0)), // Gap in indices
        ];
        
        let result = merger.merge(results);
        assert!(result.is_err());
    }

    // ===== Completeness Verification Tests =====

    #[test]
    fn test_verify_completeness_valid() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (2, Value::Number(3.0)),
        ];
        
        assert!(merger.verify_completeness(&results, 3));
    }

    #[test]
    fn test_verify_completeness_missing_index() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (2, Value::Number(3.0)),
        ];
        
        assert!(!merger.verify_completeness(&results, 3));
    }

    #[test]
    fn test_verify_completeness_duplicate_index() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (1, Value::Number(3.0)),
        ];
        
        assert!(!merger.verify_completeness(&results, 3));
    }

    #[test]
    fn test_verify_completeness_out_of_bounds() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (5, Value::Number(3.0)),
        ];
        
        assert!(!merger.verify_completeness(&results, 3));
    }

    #[test]
    fn test_verify_completeness_wrong_count() {
        let merger = StableIndexMerger::new();
        let results = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
        ];
        
        assert!(!merger.verify_completeness(&results, 3));
    }

    // ===== Property-Based Test Helpers =====
    // These tests validate the key properties of the merger

    #[test]
    fn test_property_order_independence() {
        // Property: Merge result should be independent of input order
        let merger = StableIndexMerger::new();
        
        let results1 = vec![
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
            (2, Value::Number(3.0)),
        ];
        
        let results2 = vec![
            (2, Value::Number(3.0)),
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
        ];
        
        let results3 = vec![
            (1, Value::Number(2.0)),
            (2, Value::Number(3.0)),
            (0, Value::Number(1.0)),
        ];
        
        let merged1 = merger.merge(results1).unwrap();
        let merged2 = merger.merge(results2).unwrap();
        let merged3 = merger.merge(results3).unwrap();
        
        assert_eq!(merged1, merged2);
        assert_eq!(merged2, merged3);
    }

    #[test]
    fn test_property_index_preservation() {
        // Property: Value at index i in output should correspond to (i, value) in input
        let merger = StableIndexMerger::new();
        
        let results = vec![
            (3, Value::Number(30.0)),
            (1, Value::Number(10.0)),
            (0, Value::Number(0.0)),
            (2, Value::Number(20.0)),
        ];
        
        let merged = merger.merge(results).unwrap();
        
        assert_eq!(merged[0], Value::Number(0.0));
        assert_eq!(merged[1], Value::Number(10.0));
        assert_eq!(merged[2], Value::Number(20.0));
        assert_eq!(merged[3], Value::Number(30.0));
    }

    #[test]
    fn test_property_determinism() {
        // Property: Same input should always produce same output
        let merger = StableIndexMerger::new();
        
        let results = vec![
            (2, Value::Number(3.0)),
            (0, Value::Number(1.0)),
            (1, Value::Number(2.0)),
        ];
        
        let merged1 = merger.merge(results.clone()).unwrap();
        let merged2 = merger.merge(results.clone()).unwrap();
        let merged3 = merger.merge(results).unwrap();
        
        assert_eq!(merged1, merged2);
        assert_eq!(merged2, merged3);
    }
}
