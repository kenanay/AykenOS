//! Property-based tests for DataPartitioner
//!
//! This test module validates the core correctness properties of the DataPartitioner
//! implementation using property-based testing with proptest.
//!
//! **Design Reference:** D2 Parallelism Architecture - Task 6.4
//! **Requirements:** 1.2, 1.3, 1.4
//!
//! # Properties Tested
//!
//! 1. **Completeness**: All elements covered exactly once
//! 2. **Non-overlapping**: Partition indices are non-overlapping
//! 3. **Determinism**: Same input → same partitions
//! 4. **Balance**: Partition sizes differ by at most 1 element

use proptest::prelude::*;
use semantic_cli::bcib::Value;
use semantic_cli::parallelism::{ContiguousPartitioner, DataPartitioner};

// ===== Property Test Generators =====

/// Generates arbitrary Value instances for testing
fn arbitrary_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<f64>().prop_map(Value::Number),
        any::<String>().prop_map(Value::String),
        any::<bool>().prop_map(Value::Boolean),
    ]
}

/// Generates arbitrary datasets (vectors of Values)
fn arbitrary_dataset(size_range: std::ops::Range<usize>) -> impl Strategy<Value = Vec<Value>> {
    prop::collection::vec(arbitrary_value(), size_range)
}

/// Generates arbitrary number of workers (1 to 16)
fn arbitrary_num_workers() -> impl Strategy<Value = usize> {
    1usize..=16
}

// ===== Property Tests =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 1: Completeness**
    ///
    /// For any input dataset and number of workers, all elements must be covered
    /// exactly once across all partitions.
    ///
    /// **Validates: Requirement 1.2 (Partition Independence)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Completeness**
    #[test]
    fn property_partition_completeness(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        // Collect all indices covered by partitions
        let mut covered_indices = Vec::new();
        for partition in &partitions {
            for i in partition.start_index..partition.end_index {
                covered_indices.push(i);
            }
        }
        
        // Sort to check for completeness
        covered_indices.sort_unstable();
        
        // Verify all indices from 0 to data.len()-1 are covered exactly once
        prop_assert_eq!(covered_indices.len(), data.len(), 
            "Number of covered indices must equal data size");
        
        for (i, &idx) in covered_indices.iter().enumerate() {
            prop_assert_eq!(idx, i, 
                "Index {} should be covered, but index {} was found instead", i, idx);
        }
    }

    /// **Property 2: Non-overlapping**
    ///
    /// For any input dataset and number of workers, partition index ranges must
    /// not overlap. Each element should belong to exactly one partition.
    ///
    /// **Validates: Requirement 1.2 (Partition Independence)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Non-overlapping**
    #[test]
    fn property_partition_non_overlapping(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        // Check that partitions are contiguous (each partition's start is the previous partition's end)
        for i in 1..partitions.len() {
            prop_assert_eq!(
                partitions[i].start_index, 
                partitions[i - 1].end_index,
                "Partition {} should start where partition {} ends", i, i - 1
            );
        }
        
        // Check that no two partitions overlap
        for i in 0..partitions.len() {
            for j in (i + 1)..partitions.len() {
                let p1 = &partitions[i];
                let p2 = &partitions[j];
                
                // p1 should end before or at p2 starts
                prop_assert!(
                    p1.end_index <= p2.start_index,
                    "Partition {} ({}..{}) overlaps with partition {} ({}..{})",
                    i, p1.start_index, p1.end_index,
                    j, p2.start_index, p2.end_index
                );
            }
        }
    }

    /// **Property 3: Determinism**
    ///
    /// For any input dataset and number of workers, partitioning the same data
    /// multiple times must produce identical partition boundaries.
    ///
    /// **Validates: Requirement 1.4 (Deterministic Partition Mapping)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Determinism**
    #[test]
    fn property_partition_determinism(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        
        // Partition the same data multiple times
        let partitions1 = partitioner.partition(&data, num_workers);
        let partitions2 = partitioner.partition(&data, num_workers);
        let partitions3 = partitioner.partition(&data, num_workers);
        
        // Verify all partitions are identical
        prop_assert_eq!(partitions1.len(), partitions2.len(), 
            "First and second partitioning produced different number of partitions");
        prop_assert_eq!(partitions1.len(), partitions3.len(),
            "First and third partitioning produced different number of partitions");
        
        for i in 0..partitions1.len() {
            prop_assert_eq!(
                partitions1[i].start_index, 
                partitions2[i].start_index,
                "Partition {} start index differs between first and second partitioning", i
            );
            prop_assert_eq!(
                partitions1[i].end_index, 
                partitions2[i].end_index,
                "Partition {} end index differs between first and second partitioning", i
            );
            
            prop_assert_eq!(
                partitions1[i].start_index, 
                partitions3[i].start_index,
                "Partition {} start index differs between first and third partitioning", i
            );
            prop_assert_eq!(
                partitions1[i].end_index, 
                partitions3[i].end_index,
                "Partition {} end index differs between first and third partitioning", i
            );
        }
    }

    /// **Property 4: Balance**
    ///
    /// For any input dataset and number of workers, partition sizes must be
    /// balanced (differ by at most 1 element).
    ///
    /// **Validates: Requirement 1.3 (Balanced Partition Sizes)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Balance**
    #[test]
    fn property_partition_balance(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        if partitions.is_empty() {
            // Empty data produces no partitions, which is valid
            return Ok(());
        }
        
        // Find min and max partition sizes
        let sizes: Vec<usize> = partitions.iter().map(|p| p.size()).collect();
        let min_size = *sizes.iter().min().unwrap();
        let max_size = *sizes.iter().max().unwrap();
        
        // Verify sizes differ by at most 1
        prop_assert!(
            max_size - min_size <= 1,
            "Partition sizes differ by more than 1: min={}, max={}, diff={}",
            min_size, max_size, max_size - min_size
        );
    }

    /// **Property 5: Partition Validity**
    ///
    /// For any input dataset and number of workers, all partitions must be
    /// well-formed (start_index <= end_index, data.len() == size()).
    ///
    /// **Validates: Requirement 1.2 (Partition Independence)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Validity**
    #[test]
    fn property_partition_validity(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        for (i, partition) in partitions.iter().enumerate() {
            prop_assert!(
                partition.is_valid(),
                "Partition {} is not valid: start={}, end={}, data.len()={}",
                i, partition.start_index, partition.end_index, partition.data.len()
            );
        }
    }

    /// **Property 6: First Partition Starts at Zero**
    ///
    /// For any non-empty input dataset, the first partition must start at index 0.
    ///
    /// **Validates: Requirement 1.4 (Deterministic Partition Mapping)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Start Index**
    #[test]
    fn property_first_partition_starts_at_zero(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        if !partitions.is_empty() {
            prop_assert_eq!(
                partitions[0].start_index, 
                0,
                "First partition must start at index 0"
            );
        }
    }

    /// **Property 7: Last Partition Ends at Data Length**
    ///
    /// For any non-empty input dataset, the last partition must end at data.len().
    ///
    /// **Validates: Requirement 1.2 (Partition Independence - Completeness)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition End Index**
    #[test]
    fn property_last_partition_ends_at_data_length(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        if !partitions.is_empty() {
            prop_assert_eq!(
                partitions.last().unwrap().end_index,
                data.len(),
                "Last partition must end at data.len()"
            );
        }
    }

    /// **Property 8: Number of Partitions**
    ///
    /// For any input dataset and number of workers, the number of partitions
    /// should be min(num_workers, data.len()).
    ///
    /// **Validates: Requirement 1.3 (Balanced Partition Sizes)**
    ///
    /// **Feature: d2-parallelism-architecture, Property: Partition Count**
    #[test]
    fn property_partition_count(
        data in arbitrary_dataset(1..1000),
        num_workers in arbitrary_num_workers()
    ) {
        let partitioner = ContiguousPartitioner;
        let partitions = partitioner.partition(&data, num_workers);
        
        let expected_count = num_workers.min(data.len());
        prop_assert_eq!(
            partitions.len(),
            expected_count,
            "Expected {} partitions, got {}",
            expected_count, partitions.len()
        );
    }
}

// ===== Edge Case Tests =====
// These tests verify behavior for edge cases that might not be covered by property tests

#[test]
fn test_empty_data() {
    let partitioner = ContiguousPartitioner;
    let data: Vec<Value> = vec![];
    let partitions = partitioner.partition(&data, 4);
    
    assert_eq!(partitions.len(), 0);
}

#[test]
fn test_zero_workers() {
    let partitioner = ContiguousPartitioner;
    let data = vec![Value::Number(1.0), Value::Number(2.0)];
    let partitions = partitioner.partition(&data, 0);
    
    assert_eq!(partitions.len(), 0);
}

#[test]
fn test_single_element() {
    let partitioner = ContiguousPartitioner;
    let data = vec![Value::Number(42.0)];
    let partitions = partitioner.partition(&data, 1);
    
    assert_eq!(partitions.len(), 1);
    assert_eq!(partitions[0].start_index, 0);
    assert_eq!(partitions[0].end_index, 1);
    assert_eq!(partitions[0].size(), 1);
}

#[test]
fn test_more_workers_than_elements() {
    let partitioner = ContiguousPartitioner;
    let data = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let partitions = partitioner.partition(&data, 100);
    
    // Should create only 3 partitions (one per element)
    assert_eq!(partitions.len(), 3);
    
    for (i, partition) in partitions.iter().enumerate() {
        assert_eq!(partition.start_index, i);
        assert_eq!(partition.end_index, i + 1);
        assert_eq!(partition.size(), 1);
    }
}
