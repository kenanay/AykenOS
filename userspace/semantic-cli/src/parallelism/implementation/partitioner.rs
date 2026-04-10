//! Data partitioning for parallel execution
//!
//! This module provides the `DataPartitioner` trait and implementations for dividing
//! input data into independent partitions that can be processed in parallel.
//!
//! The partitioning strategy is critical for the D2 Parallelism Architecture because
//! it determines how work is distributed across parallel workers and establishes the
//! stable index mapping used for deterministic result ordering.
//!
//! **Design Reference:** D2 Parallelism Architecture - Data Partitioner section
//! **Requirements:** 1.2, 1.3, 1.4

use crate::bcib::Value;
use crate::parallelism::types::DataPartition;

/// Trait for partitioning data into independent chunks for parallel processing.
///
/// The `DataPartitioner` is responsible for dividing input data into `DataPartition`
/// instances that can be processed independently by parallel workers. The partitioning
/// strategy must ensure:
///
/// 1. **Determinism**: Same input → same partitions (always)
/// 2. **Completeness**: All elements are covered exactly once
/// 3. **Non-overlapping**: Partition index ranges do not overlap
/// 4. **Balance**: Partition sizes are balanced (within 1 element)
///
/// # Design Principles
///
/// - **Contiguous Partitions**: Data is divided into contiguous slices for cache efficiency
/// - **Stable Index Mapping**: Each partition has deterministic start/end indices
/// - **Zero-Copy**: Uses borrowed slices to avoid data copying overhead
///
/// **Validates: Requirements 1.2, 1.3, 1.4**
pub trait DataPartitioner {
    /// Partitions the input data into independent chunks for parallel processing.
    ///
    /// # Arguments
    ///
    /// * `data` - The input data to partition
    /// * `num_workers` - The number of parallel workers (determines number of partitions)
    ///
    /// # Returns
    ///
    /// A vector of `DataPartition` instances, one for each worker. The partitions:
    /// - Cover all elements in `data` exactly once
    /// - Have non-overlapping index ranges
    /// - Are balanced in size (within 1 element)
    /// - Are deterministic (same inputs → same partitions)
    ///
    /// # Properties
    ///
    /// - **Property: Completeness** - All elements covered exactly once
    /// - **Property: Non-overlapping** - Partition indices are non-overlapping
    /// - **Property: Determinism** - Same input → same partitions
    ///
    /// **Validates: Requirements 1.2, 1.4**
    fn partition<'a>(&self, data: &'a [Value], num_workers: usize) -> Vec<DataPartition<'a>>;

    /// Calculates the size of each partition given the data size and number of workers.
    ///
    /// This is a helper method used by partition implementations to determine
    /// how to divide the data. The calculation ensures balanced partition sizes.
    ///
    /// # Arguments
    ///
    /// * `data_size` - The total number of elements in the input data
    /// * `num_workers` - The number of parallel workers
    ///
    /// # Returns
    ///
    /// The size of each partition (rounded up to ensure all elements are covered)
    ///
    /// **Validates: Requirement 1.3**
    fn calculate_partition_size(&self, data_size: usize, num_workers: usize) -> usize;
}

/// Contiguous partitioner that divides data into contiguous chunks.
///
/// This is the default and recommended partitioning strategy. It divides the input
/// data into contiguous slices, which provides:
///
/// - **Cache Efficiency**: Contiguous memory access patterns
/// - **Simplicity**: Straightforward implementation and reasoning
/// - **Determinism**: Trivial to verify deterministic behavior
///
/// # Partitioning Strategy
///
/// ```text
/// Input:  [e0, e1, e2, e3, e4, e5, e6, e7]
///          |         |         |         |
/// Partition: [0,1]    [2,3]     [4,5]     [6,7]
///          |         |         |         |
/// Workers:  W1        W2        W3        W4
/// ```
///
/// The partition size is calculated as `ceil(data_size / num_workers)`, ensuring
/// that all elements are covered and partition sizes are balanced (within 1 element).
///
/// # Example
///
/// ```rust,ignore
/// use semantic_cli::parallelism::{ContiguousPartitioner, DataPartitioner};
/// use semantic_cli::bcib::Value;
///
/// let partitioner = ContiguousPartitioner;
/// let data = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
/// let partitions = partitioner.partition(&data, 2);
///
/// assert_eq!(partitions.len(), 2);
/// assert_eq!(partitions[0].size(), 2); // [0, 1]
/// assert_eq!(partitions[1].size(), 1); // [2]
/// ```
///
/// **Validates: Requirements 1.2, 1.3, 1.4**
#[derive(Debug, Clone, Copy, Default)]
pub struct ContiguousPartitioner;

impl DataPartitioner for ContiguousPartitioner {
    fn partition<'a>(&self, data: &'a [Value], num_workers: usize) -> Vec<DataPartition<'a>> {
        // Handle edge cases
        if data.is_empty() {
            return Vec::new();
        }

        if num_workers == 0 {
            return Vec::new();
        }

        // If we have more workers than data elements, limit workers to data size
        let effective_workers = num_workers.min(data.len());

        // Calculate base partition size and remainder
        // This ensures balanced partitions (within 1 element)
        let base_size = data.len() / effective_workers;
        let remainder = data.len() % effective_workers;

        // Create partitions
        // First 'remainder' partitions get (base_size + 1) elements
        // Remaining partitions get base_size elements
        let mut partitions = Vec::with_capacity(effective_workers);
        let mut start = 0;

        for i in 0..effective_workers {
            // First 'remainder' partitions get an extra element
            let size = if i < remainder {
                base_size + 1
            } else {
                base_size
            };

            let end = start + size;

            partitions.push(DataPartition {
                data: &data[start..end],
                start_index: start,
                end_index: end,
            });

            start = end;
        }

        partitions
    }

    fn calculate_partition_size(&self, data_size: usize, num_workers: usize) -> usize {
        if num_workers == 0 {
            return 0;
        }

        // Calculate base partition size
        // Note: This returns the base size, not accounting for remainder distribution
        // The actual partition() method handles remainder distribution properly
        data_size / num_workers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;

    // ===== ContiguousPartitioner Unit Tests =====

    #[test]
    fn test_partition_empty_data() {
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = vec![];
        let partitions = partitioner.partition(&data, 4);

        assert_eq!(partitions.len(), 0);
    }

    #[test]
    fn test_partition_zero_workers() {
        let partitioner = ContiguousPartitioner;
        let data = vec![Value::Number(1.0), Value::Number(2.0)];
        let partitions = partitioner.partition(&data, 0);

        assert_eq!(partitions.len(), 0);
    }

    #[test]
    fn test_partition_single_worker() {
        let partitioner = ContiguousPartitioner;
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        let partitions = partitioner.partition(&data, 1);

        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].start_index, 0);
        assert_eq!(partitions[0].end_index, 4);
        assert_eq!(partitions[0].size(), 4);
    }

    #[test]
    fn test_partition_balanced() {
        let partitioner = ContiguousPartitioner;
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        let partitions = partitioner.partition(&data, 2);

        assert_eq!(partitions.len(), 2);

        // First partition: [0, 1]
        assert_eq!(partitions[0].start_index, 0);
        assert_eq!(partitions[0].end_index, 2);
        assert_eq!(partitions[0].size(), 2);

        // Second partition: [2, 3]
        assert_eq!(partitions[1].start_index, 2);
        assert_eq!(partitions[1].end_index, 4);
        assert_eq!(partitions[1].size(), 2);
    }

    #[test]
    fn test_partition_uneven() {
        let partitioner = ContiguousPartitioner;
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        let partitions = partitioner.partition(&data, 2);

        assert_eq!(partitions.len(), 2);

        // First partition: [0, 1, 2] (size 3)
        assert_eq!(partitions[0].start_index, 0);
        assert_eq!(partitions[0].end_index, 3);
        assert_eq!(partitions[0].size(), 3);

        // Second partition: [3, 4] (size 2)
        assert_eq!(partitions[1].start_index, 3);
        assert_eq!(partitions[1].end_index, 5);
        assert_eq!(partitions[1].size(), 2);

        // Verify balance: sizes differ by at most 1
        let size_diff = (partitions[0].size() as i32 - partitions[1].size() as i32).abs();
        assert!(size_diff <= 1);
    }

    #[test]
    fn test_partition_more_workers_than_data() {
        let partitioner = ContiguousPartitioner;
        let data = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let partitions = partitioner.partition(&data, 10);

        // Should create only 3 partitions (one per element)
        assert_eq!(partitions.len(), 3);

        for (i, partition) in partitions.iter().enumerate() {
            assert_eq!(partition.start_index, i);
            assert_eq!(partition.end_index, i + 1);
            assert_eq!(partition.size(), 1);
        }
    }

    #[test]
    fn test_partition_large_dataset() {
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = (0..1000).map(|i| Value::Number(i as f64)).collect();
        let partitions = partitioner.partition(&data, 8);

        assert_eq!(partitions.len(), 8);

        // 1000 / 8 = 125 exactly, so all partitions should have 125 elements
        for partition in &partitions {
            assert_eq!(partition.size(), 125);
        }
    }

    #[test]
    fn test_calculate_partition_size() {
        let partitioner = ContiguousPartitioner;

        // Evenly divisible
        assert_eq!(partitioner.calculate_partition_size(100, 10), 10);

        // Not evenly divisible - returns base size
        assert_eq!(partitioner.calculate_partition_size(100, 7), 14); // floor(100/7) = 14
        assert_eq!(partitioner.calculate_partition_size(10, 3), 3); // floor(10/3) = 3

        // Edge cases
        assert_eq!(partitioner.calculate_partition_size(0, 10), 0);
        assert_eq!(partitioner.calculate_partition_size(10, 0), 0);
        assert_eq!(partitioner.calculate_partition_size(1, 1), 1);
    }

    // ===== Property Validation Tests =====
    // These tests validate the core properties required by the design

    #[test]
    fn test_property_completeness() {
        // Property: All elements covered exactly once
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = (0..100).map(|i| Value::Number(i as f64)).collect();
        let partitions = partitioner.partition(&data, 7);

        // Collect all indices covered by partitions
        let mut covered_indices = Vec::new();
        for partition in &partitions {
            for i in partition.start_index..partition.end_index {
                covered_indices.push(i);
            }
        }

        // Sort to check for completeness
        covered_indices.sort();

        // Verify all indices from 0 to 99 are covered exactly once
        assert_eq!(covered_indices.len(), 100);
        for (i, &idx) in covered_indices.iter().enumerate() {
            assert_eq!(idx, i);
        }
    }

    #[test]
    fn test_property_non_overlapping() {
        // Property: Partition indices are non-overlapping
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = (0..100).map(|i| Value::Number(i as f64)).collect();
        let partitions = partitioner.partition(&data, 7);

        // Check that each partition's start is the previous partition's end
        for i in 1..partitions.len() {
            assert_eq!(partitions[i].start_index, partitions[i - 1].end_index);
        }

        // Check that no two partitions overlap
        for i in 0..partitions.len() {
            for j in (i + 1)..partitions.len() {
                let p1 = &partitions[i];
                let p2 = &partitions[j];

                // p1 should end before or at p2 starts
                assert!(p1.end_index <= p2.start_index);
            }
        }
    }

    #[test]
    fn test_property_determinism() {
        // Property: Same input → same partitions (determinism)
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = (0..100).map(|i| Value::Number(i as f64)).collect();

        // Partition the same data multiple times
        let partitions1 = partitioner.partition(&data, 7);
        let partitions2 = partitioner.partition(&data, 7);
        let partitions3 = partitioner.partition(&data, 7);

        // Verify all partitions are identical
        assert_eq!(partitions1.len(), partitions2.len());
        assert_eq!(partitions1.len(), partitions3.len());

        for i in 0..partitions1.len() {
            assert_eq!(partitions1[i].start_index, partitions2[i].start_index);
            assert_eq!(partitions1[i].end_index, partitions2[i].end_index);

            assert_eq!(partitions1[i].start_index, partitions3[i].start_index);
            assert_eq!(partitions1[i].end_index, partitions3[i].end_index);
        }
    }

    #[test]
    fn test_property_balance() {
        // Property: Partition sizes are balanced (within 1 element)
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = (0..100).map(|i| Value::Number(i as f64)).collect();
        let partitions = partitioner.partition(&data, 7);

        // Find min and max partition sizes
        let sizes: Vec<usize> = partitions.iter().map(|p| p.size()).collect();
        let min_size = *sizes.iter().min().unwrap();
        let max_size = *sizes.iter().max().unwrap();

        // Verify sizes differ by at most 1
        assert!(max_size - min_size <= 1);
    }

    #[test]
    fn test_partition_validity() {
        // Verify all partitions are well-formed
        let partitioner = ContiguousPartitioner;
        let data: Vec<Value> = (0..100).map(|i| Value::Number(i as f64)).collect();
        let partitions = partitioner.partition(&data, 7);

        for partition in &partitions {
            assert!(partition.is_valid());
        }
    }
}
