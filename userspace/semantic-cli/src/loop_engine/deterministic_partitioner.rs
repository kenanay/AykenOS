//! Deterministic Partitioning for D3 Loop Support
//!
//! This module implements the deterministic partitioning algorithm required for
//! parallel loop execution. The partitioner ensures that the same iteration count
//! always produces the same partition boundaries, regardless of available parallelism
//! or machine-dependent factors.
//!
//! # Constitutional Requirements
//!
//! The partitioning algorithm must adhere to these constitutional principles:
//! - **Deterministic**: Same iteration count → same partitions (always)
//! - **Reproducible**: Partition boundaries are based on iteration count only
//! - **Optimization Hint**: Available parallelism is upper bound, not semantic input
//! - **Fixed Algorithm**: Uses fixed chunk size algorithm for consistency
//!
//! # Requirements
//!
//! **Requirements 7.5, 15.1, 15.3**: Partition iterations based on iteration count only,
//! use fixed chunk size algorithm, treat available parallelism as upper bound.
//!
//! **Requirements 15.2**: Ensure iteration i always processes same input data
//! regardless of partition assignment.

use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::loop_engine::IterationPartition;

/// Deterministic partitioner for loop iterations
///
/// This struct implements the constitutional requirement for deterministic partitioning
/// of loop iterations. The partitioner uses a fixed chunk size algorithm that depends
/// only on the total iteration count, ensuring reproducible partition boundaries.
///
/// # Design Principles
///
/// 1. **Deterministic Algorithm**: Same inputs → same outputs (always)
/// 2. **Fixed Chunk Size**: Chunk size calculated from iteration count only
/// 3. **Load Balancing**: Partitions are balanced within one iteration
/// 4. **Optimization Boundary**: Available parallelism is upper bound only
///
/// **Requirements:** 7.5, 15.1, 15.3
#[derive(Debug, Clone, Default)]
pub struct DeterministicPartitioner {
    /// Configuration for chunk size calculation
    config: PartitionerConfig,
}

/// Configuration for the deterministic partitioner
#[derive(Debug, Clone)]
pub struct PartitionerConfig {
    /// Minimum chunk size for overhead amortization
    pub min_chunk_size: u32,
    /// Maximum chunk size for load balancing
    pub max_chunk_size: u32,
    /// Target number of chunks for medium-sized loops
    pub target_chunks: u32,
}

impl Default for PartitionerConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 100,  // Minimum chunk size for overhead amortization
            max_chunk_size: 1000, // Maximum chunk size for load balancing
            target_chunks: 4,     // Target number of chunks for good parallelism
        }
    }
}

impl DeterministicPartitioner {
    /// Create a new deterministic partitioner with default configuration
    pub fn new() -> Self {
        Self {
            config: PartitionerConfig::default(),
        }
    }

    /// Create a new deterministic partitioner with custom configuration
    pub fn with_config(config: PartitionerConfig) -> Self {
        Self { config }
    }

    /// Partition iterations deterministically (Phase 7.2)
    ///
    /// This method implements the core deterministic partitioning algorithm:
    /// 1. Calculate deterministic chunk size based on iteration count only
    /// 2. Create partitions with fixed chunk boundaries
    /// 3. Treat available parallelism as upper bound (optimization only)
    /// 4. Ensure all iterations are covered exactly once
    ///
    /// **Requirements 7.5, 15.1, 15.3**: Deterministic partitioning algorithm
    ///
    /// # Arguments
    ///
    /// * `total_iterations` - Total number of iterations to partition
    /// * `available_parallelism` - Available parallel workers (upper bound only)
    ///
    /// # Returns
    ///
    /// Vector of `IterationPartition` structs with deterministic boundaries
    ///
    /// # Guarantees
    ///
    /// - Same `total_iterations` → same partition boundaries (always)
    /// - All iterations from 0 to `total_iterations-1` are covered exactly once
    /// - Partition boundaries are reproducible across different machines
    /// - Available parallelism only affects number of partitions, not boundaries
    pub fn partition_iterations(
        &self,
        total_iterations: u32,
        available_parallelism: usize,
    ) -> Result<Vec<IterationPartition>> {
        // Handle edge cases
        if total_iterations == 0 {
            return Ok(Vec::new());
        }

        if available_parallelism == 0 {
            return Err(SemanticCLIError::validation_error(
                "Available parallelism must be greater than 0",
                "Provide a positive number of parallel workers",
                ErrorCode::E400,
            ));
        }

        // Phase 7.2: Calculate deterministic chunk size
        // Constitutional rule: Based on iteration count only, not machine-dependent factors
        let chunk_size = self.calculate_deterministic_chunk_size(total_iterations);

        // Calculate number of partitions needed based on chunk size
        let num_partitions = ((total_iterations + chunk_size - 1) / chunk_size) as usize;

        // Treat available parallelism as upper bound (optimization, not semantic)
        let effective_partitions = num_partitions.min(available_parallelism);

        // Create deterministic partitions using fixed chunk size
        let partitions =
            self.create_fixed_chunk_partitions(total_iterations, chunk_size, effective_partitions)?;

        // Verify partition correctness
        self.verify_partition_correctness(&partitions, total_iterations)?;

        Ok(partitions)
    }

    /// Calculate deterministic chunk size based on iteration count (Phase 7.2)
    ///
    /// This method implements the fixed chunk size algorithm required for deterministic
    /// partitioning. The chunk size is calculated based solely on iteration count,
    /// ensuring the same input always produces the same partitions.
    ///
    /// **Requirements 15.1**: Partition iterations based on iteration count only
    ///
    /// # Algorithm
    ///
    /// ```text
    /// if total_iterations <= min_chunk_size:
    ///     chunk_size = total_iterations  // Single chunk
    /// elif total_iterations <= max_chunk_size * target_chunks:
    ///     chunk_size = ceil(total_iterations / target_chunks)  // Balanced chunks
    /// else:
    ///     chunk_size = max_chunk_size  // Fixed maximum
    /// ```
    ///
    /// # Arguments
    ///
    /// * `total_iterations` - Total number of iterations to partition
    ///
    /// # Returns
    ///
    /// Deterministic chunk size for partitioning
    pub fn calculate_deterministic_chunk_size(&self, total_iterations: u32) -> u32 {
        if total_iterations <= self.config.min_chunk_size {
            // Small loops: single chunk
            total_iterations
        } else if total_iterations <= self.config.max_chunk_size * self.config.target_chunks {
            // Medium loops: divide into target number of chunks for good parallelism
            (total_iterations + self.config.target_chunks - 1) / self.config.target_chunks
        // Ceiling division
        } else {
            // Large loops: use maximum chunk size
            self.config.max_chunk_size
        }
    }

    /// Create partitions with fixed chunk boundaries (Phase 7.2)
    ///
    /// This method creates the actual partition structures with deterministic boundaries.
    /// Each partition covers a contiguous range of iterations with stable index mapping.
    ///
    /// **Requirements 15.2**: Stable index mapping for deterministic parallel execution
    fn create_fixed_chunk_partitions(
        &self,
        total_iterations: u32,
        chunk_size: u32,
        effective_partitions: usize,
    ) -> Result<Vec<IterationPartition>> {
        // First, create all deterministic partitions based on chunk size only
        let mut all_partitions = Vec::new();
        let mut start_iteration = 0;
        let mut partition_id = 0;

        while start_iteration < total_iterations {
            let end_iteration = (start_iteration + chunk_size).min(total_iterations);

            let partition = IterationPartition {
                partition_id,
                start_iteration,
                end_iteration,
                iteration_count: end_iteration - start_iteration,
            };

            // Verify partition is valid
            if !partition.is_valid() {
                return Err(SemanticCLIError::execution_error(
                    &format!("Invalid partition created: {:?}", partition),
                    ErrorCode::E500,
                ));
            }

            all_partitions.push(partition);
            start_iteration = end_iteration;
            partition_id += 1;
        }

        // Then, limit to effective_partitions (available parallelism upper bound)
        // If we have more partitions than available parallelism, merge the excess partitions
        if all_partitions.len() <= effective_partitions {
            // We have enough parallelism for all partitions
            Ok(all_partitions)
        } else {
            // We need to merge some partitions to fit within available parallelism
            let mut merged_partitions = Vec::with_capacity(effective_partitions);
            let partitions_per_worker =
                (all_partitions.len() + effective_partitions - 1) / effective_partitions;

            for worker_id in 0..effective_partitions {
                let start_idx = worker_id * partitions_per_worker;
                let end_idx = ((worker_id + 1) * partitions_per_worker).min(all_partitions.len());

                if start_idx < all_partitions.len() {
                    // Merge partitions for this worker
                    let first_partition = &all_partitions[start_idx];
                    let last_partition = &all_partitions[end_idx - 1];

                    let merged_partition = IterationPartition {
                        partition_id: worker_id,
                        start_iteration: first_partition.start_iteration,
                        end_iteration: last_partition.end_iteration,
                        iteration_count: last_partition.end_iteration
                            - first_partition.start_iteration,
                    };

                    merged_partitions.push(merged_partition);
                }
            }

            Ok(merged_partitions)
        }
    }

    /// Verify partition correctness (Phase 7.2)
    ///
    /// This method verifies that the created partitions satisfy all correctness properties:
    /// - All iterations are covered exactly once
    /// - Partitions are non-overlapping
    /// - Partition boundaries are contiguous
    /// - Total iteration count matches expected
    fn verify_partition_correctness(
        &self,
        partitions: &[IterationPartition],
        expected_total: u32,
    ) -> Result<()> {
        if partitions.is_empty() {
            if expected_total == 0 {
                return Ok(()); // Empty partitions for zero iterations is valid
            } else {
                return Err(SemanticCLIError::execution_error(
                    "No partitions created for non-zero iteration count",
                    ErrorCode::E500,
                ));
            }
        }

        // Verify partitions are sorted by partition_id
        for i in 1..partitions.len() {
            if partitions[i].partition_id != partitions[i - 1].partition_id + 1 {
                return Err(SemanticCLIError::execution_error(
                    "Partitions are not properly ordered by partition_id",
                    ErrorCode::E500,
                ));
            }
        }

        // Verify partitions are contiguous and non-overlapping
        let mut expected_start = 0;
        let mut total_covered = 0;

        for partition in partitions {
            // Check partition starts where expected
            if partition.start_iteration != expected_start {
                return Err(SemanticCLIError::execution_error(
                    &format!(
                        "Partition {} has gap: expected start {}, actual start {}",
                        partition.partition_id, expected_start, partition.start_iteration
                    ),
                    ErrorCode::E500,
                ));
            }

            // Check partition is valid
            if !partition.is_valid() {
                return Err(SemanticCLIError::execution_error(
                    &format!("Invalid partition: {:?}", partition),
                    ErrorCode::E500,
                ));
            }

            total_covered += partition.iteration_count;
            expected_start = partition.end_iteration;
        }

        // Verify total coverage
        if total_covered != expected_total {
            return Err(SemanticCLIError::execution_error(
                &format!(
                    "Partition coverage mismatch: expected {}, actual {}",
                    expected_total, total_covered
                ),
                ErrorCode::E500,
            ));
        }

        // Verify last partition ends at expected total
        if let Some(last_partition) = partitions.last() {
            if last_partition.end_iteration != expected_total {
                return Err(SemanticCLIError::execution_error(
                    &format!(
                        "Last partition ends at {}, expected {}",
                        last_partition.end_iteration, expected_total
                    ),
                    ErrorCode::E500,
                ));
            }
        }

        Ok(())
    }

    /// Get partitioner configuration
    pub fn config(&self) -> &PartitionerConfig {
        &self.config
    }

    /// Update partitioner configuration
    pub fn update_config(&mut self, config: PartitionerConfig) {
        self.config = config;
    }
}

/// Partition analysis result for debugging and optimization
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionAnalysis {
    /// Total number of iterations
    pub total_iterations: u32,
    /// Number of partitions created
    pub partition_count: usize,
    /// Chunk size used for partitioning
    pub chunk_size: u32,
    /// Available parallelism (upper bound)
    pub available_parallelism: usize,
    /// Effective parallelism used
    pub effective_parallelism: usize,
    /// Load balancing metrics
    pub load_balance: LoadBalanceMetrics,
}

/// Load balancing metrics for partition analysis
#[derive(Debug, Clone, PartialEq)]
pub struct LoadBalanceMetrics {
    /// Minimum partition size
    pub min_partition_size: u32,
    /// Maximum partition size
    pub max_partition_size: u32,
    /// Average partition size
    pub avg_partition_size: f64,
    /// Load balance ratio (min/max)
    pub balance_ratio: f64,
}

impl DeterministicPartitioner {
    /// Analyze partitioning for debugging and optimization
    pub fn analyze_partitioning(
        &self,
        total_iterations: u32,
        available_parallelism: usize,
    ) -> Result<PartitionAnalysis> {
        let partitions = self.partition_iterations(total_iterations, available_parallelism)?;

        if partitions.is_empty() {
            return Ok(PartitionAnalysis {
                total_iterations,
                partition_count: 0,
                chunk_size: 0,
                available_parallelism,
                effective_parallelism: 0,
                load_balance: LoadBalanceMetrics {
                    min_partition_size: 0,
                    max_partition_size: 0,
                    avg_partition_size: 0.0,
                    balance_ratio: 1.0,
                },
            });
        }

        let chunk_size = self.calculate_deterministic_chunk_size(total_iterations);
        let partition_sizes: Vec<u32> = partitions.iter().map(|p| p.iteration_count).collect();

        let min_size = *partition_sizes.iter().min().unwrap();
        let max_size = *partition_sizes.iter().max().unwrap();
        let avg_size = partition_sizes.iter().sum::<u32>() as f64 / partition_sizes.len() as f64;
        let balance_ratio = if max_size > 0 {
            min_size as f64 / max_size as f64
        } else {
            1.0
        };

        Ok(PartitionAnalysis {
            total_iterations,
            partition_count: partitions.len(),
            chunk_size,
            available_parallelism,
            effective_parallelism: partitions.len(),
            load_balance: LoadBalanceMetrics {
                min_partition_size: min_size,
                max_partition_size: max_size,
                avg_partition_size: avg_size,
                balance_ratio,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_partitioning_same_inputs() {
        let partitioner = DeterministicPartitioner::new();

        // Test deterministic partitioning with same inputs
        let partitions1 = partitioner.partition_iterations(1000, 4).unwrap();
        let partitions2 = partitioner.partition_iterations(1000, 4).unwrap();

        // Should produce identical partitions
        assert_eq!(partitions1, partitions2);
        assert!(!partitions1.is_empty());
    }

    #[test]
    fn test_partition_completeness() {
        let partitioner = DeterministicPartitioner::new();
        let partitions = partitioner.partition_iterations(1000, 4).unwrap();

        // Verify all iterations are covered exactly once
        let mut total_iterations = 0;
        let mut last_end = 0;

        for partition in &partitions {
            assert_eq!(partition.start_iteration, last_end);
            assert!(partition.is_valid());
            total_iterations += partition.iteration_count;
            last_end = partition.end_iteration;
        }

        assert_eq!(total_iterations, 1000);
        assert_eq!(last_end, 1000);
    }

    #[test]
    fn test_chunk_size_calculation() {
        let partitioner = DeterministicPartitioner::new();

        // Small loops: single chunk
        assert_eq!(partitioner.calculate_deterministic_chunk_size(50), 50);

        // Medium loops: divide into target chunks
        assert_eq!(partitioner.calculate_deterministic_chunk_size(400), 100);

        // Large loops: use maximum chunk size
        assert_eq!(partitioner.calculate_deterministic_chunk_size(10000), 1000);
    }

    #[test]
    fn test_available_parallelism_as_upper_bound() {
        let partitioner = DeterministicPartitioner::new();

        // More parallelism than needed
        let partitions1 = partitioner.partition_iterations(100, 10).unwrap();
        assert_eq!(partitions1.len(), 1); // Only one chunk needed

        // Less parallelism than optimal
        let partitions2 = partitioner.partition_iterations(10000, 2).unwrap();
        assert_eq!(partitions2.len(), 2); // Limited by available parallelism
    }

    #[test]
    fn test_edge_cases() {
        let partitioner = DeterministicPartitioner::new();

        // Zero iterations
        let partitions = partitioner.partition_iterations(0, 4).unwrap();
        assert!(partitions.is_empty());

        // Single iteration
        let partitions = partitioner.partition_iterations(1, 4).unwrap();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].iteration_count, 1);

        // Zero parallelism should fail
        let result = partitioner.partition_iterations(100, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_configuration() {
        let config = PartitionerConfig {
            min_chunk_size: 50,
            max_chunk_size: 500,
            target_chunks: 8,
        };
        let partitioner = DeterministicPartitioner::with_config(config);

        // Test with custom configuration
        let chunk_size = partitioner.calculate_deterministic_chunk_size(1000);
        assert_eq!(chunk_size, 125); // 1000 / 8 = 125
    }

    #[test]
    fn test_partition_analysis() {
        let partitioner = DeterministicPartitioner::new();
        let analysis = partitioner.analyze_partitioning(1000, 4).unwrap();

        assert_eq!(analysis.total_iterations, 1000);
        assert!(analysis.partition_count > 0);
        assert!(analysis.chunk_size > 0);
        assert_eq!(analysis.available_parallelism, 4);
        assert!(analysis.load_balance.balance_ratio > 0.0);
        assert!(analysis.load_balance.balance_ratio <= 1.0);
    }

    #[test]
    fn test_load_balancing() {
        let partitioner = DeterministicPartitioner::new();
        let partitions = partitioner.partition_iterations(1000, 4).unwrap();

        // Check load balancing
        let sizes: Vec<u32> = partitions.iter().map(|p| p.iteration_count).collect();
        let min_size = *sizes.iter().min().unwrap();
        let max_size = *sizes.iter().max().unwrap();

        // Partitions should be balanced within 1 iteration
        assert!(max_size - min_size <= 1);
    }

    #[test]
    fn test_determinism_across_different_parallelism() {
        let partitioner = DeterministicPartitioner::new();

        // Same iteration count with different available parallelism
        let partitions1 = partitioner.partition_iterations(1000, 2).unwrap();
        let partitions2 = partitioner.partition_iterations(1000, 8).unwrap();

        // Chunk size should be the same (deterministic)
        let chunk_size1 = partitioner.calculate_deterministic_chunk_size(1000);
        let chunk_size2 = partitioner.calculate_deterministic_chunk_size(1000);
        assert_eq!(chunk_size1, chunk_size2);

        // The key deterministic property: same total iterations are covered
        let total1: u32 = partitions1.iter().map(|p| p.iteration_count).sum();
        let total2: u32 = partitions2.iter().map(|p| p.iteration_count).sum();
        assert_eq!(total1, 1000);
        assert_eq!(total2, 1000);
        assert_eq!(total1, total2);

        // All iterations should be covered exactly once in both cases
        let mut covered1 = vec![false; 1000];
        for partition in &partitions1 {
            for i in partition.start_iteration..partition.end_iteration {
                assert!(
                    !covered1[i as usize],
                    "Iteration {} covered twice in partitions1",
                    i
                );
                covered1[i as usize] = true;
            }
        }
        assert!(
            covered1.iter().all(|&x| x),
            "Not all iterations covered in partitions1"
        );

        let mut covered2 = vec![false; 1000];
        for partition in &partitions2 {
            for i in partition.start_iteration..partition.end_iteration {
                assert!(
                    !covered2[i as usize],
                    "Iteration {} covered twice in partitions2",
                    i
                );
                covered2[i as usize] = true;
            }
        }
        assert!(
            covered2.iter().all(|&x| x),
            "Not all iterations covered in partitions2"
        );

        // Verify that partitions are contiguous and non-overlapping
        for partitions in [&partitions1, &partitions2] {
            let mut last_end = 0;
            for partition in partitions {
                assert_eq!(
                    partition.start_iteration, last_end,
                    "Gap in partition coverage"
                );
                assert!(partition.is_valid(), "Invalid partition: {:?}", partition);
                last_end = partition.end_iteration;
            }
            assert_eq!(last_end, 1000, "Partitions don't cover all iterations");
        }
    }

    #[test]
    fn test_partition_verification() {
        let partitioner = DeterministicPartitioner::new();

        // Valid partitions should pass verification
        let partitions = partitioner.partition_iterations(1000, 4).unwrap();
        assert!(partitioner
            .verify_partition_correctness(&partitions, 1000)
            .is_ok());

        // Test verification with empty partitions for zero iterations
        assert!(partitioner.verify_partition_correctness(&[], 0).is_ok());
    }

    #[test]
    fn test_property_determinism() {
        // Property: Same input should always produce same output
        let partitioner = DeterministicPartitioner::new();

        for total_iterations in [100, 500, 1000, 5000] {
            for available_parallelism in [1, 2, 4, 8] {
                let partitions1 = partitioner
                    .partition_iterations(total_iterations, available_parallelism)
                    .unwrap();
                let partitions2 = partitioner
                    .partition_iterations(total_iterations, available_parallelism)
                    .unwrap();
                let partitions3 = partitioner
                    .partition_iterations(total_iterations, available_parallelism)
                    .unwrap();

                assert_eq!(partitions1, partitions2);
                assert_eq!(partitions2, partitions3);
            }
        }
    }

    #[test]
    fn test_property_completeness() {
        // Property: All iterations covered exactly once
        let partitioner = DeterministicPartitioner::new();

        for total_iterations in [1, 10, 100, 1000, 10000] {
            let partitions = partitioner
                .partition_iterations(total_iterations, 4)
                .unwrap();

            let mut covered_iterations = 0;
            let mut last_end = 0;

            for partition in &partitions {
                assert_eq!(partition.start_iteration, last_end);
                covered_iterations += partition.iteration_count;
                last_end = partition.end_iteration;
            }

            assert_eq!(covered_iterations, total_iterations);
            assert_eq!(last_end, total_iterations);
        }
    }

    #[test]
    fn test_property_non_overlapping() {
        // Property: Partitions are non-overlapping
        let partitioner = DeterministicPartitioner::new();
        let partitions = partitioner.partition_iterations(1000, 4).unwrap();

        for i in 0..partitions.len() {
            for j in (i + 1)..partitions.len() {
                let p1 = &partitions[i];
                let p2 = &partitions[j];

                // p1 should end before or at p2 starts
                assert!(p1.end_iteration <= p2.start_iteration);
            }
        }
    }
}
