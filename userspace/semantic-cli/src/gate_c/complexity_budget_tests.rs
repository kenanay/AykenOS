//! # Complexity Budget Tests
//!
//! Constitutional CI Guards for Phase 4.0 Baseline Hardening
//! 
//! **CRITICAL:** These tests prevent O(n²) complexity attacks and ensure
//! Gate C operations complete within reasonable time bounds even for
//! large plans. Failure indicates a DoS vulnerability.

use crate::gate_c::{
    normalizer::PlanNormalizer,
    types::*,
    ir::{SemanticAnalyzer, IRPlanner},
    limits::MAX_PLAN_STEPS,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Create a plan with N steps for complexity testing
fn create_plan_with_n_steps(n: usize) -> ExecutionPlan {
    let mut steps = Vec::new();
    let mut dependencies = Vec::new();
    
    for i in 1..=n {
        let step_id = format!("step-{:04}", i);
        
        // Create realistic operations
        let operation = match i % 4 {
            0 => Operation::Query {
                target: format!("table-{}", i % 10),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("id".to_string(), format!("{}", i));
                    p.insert("filter".to_string(), format!("active = true AND id > {}", i.saturating_sub(100)));
                    p
                },
            },
            1 => Operation::Compute {
                function: format!("process-{}", i % 5),
                arguments: vec![
                    format!("input-{}", i),
                    "normalize".to_string(),
                    "validate".to_string(),
                    format!("config-{}", i % 3),
                ],
            },
            2 => Operation::Mutation {
                intent: MutationIntent::UpdateIntent {
                    target: ResourcePath {
                        segments: vec![
                            "cache".to_string(),
                            format!("partition-{}", i % 8),
                            format!("item-{}", i),
                        ],
                    },
                    changes: ChangeSet {
                        updates: {
                            let mut updates = HashMap::new();
                            updates.insert("status".to_string(), "processed".to_string());
                            updates.insert("updated_at".to_string(), "0".to_string());
                            updates.insert("version".to_string(), format!("{}", i));
                            updates
                        },
                        removals: vec!["temp_data".to_string(), "cache_key".to_string()],
                    },
                },
            },
            _ => Operation::Query {
                target: "metadata".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".to_string(), format!("{}", i));
                    p.insert("include_history".to_string(), "false".to_string());
                    p
                },
            },
        };
        
        // Create inputs (depend on previous 1-3 steps)
        let inputs = if i > 1 {
            let num_deps = std::cmp::min(3, i - 1);
            (0..num_deps).map(|j| {
                let dep_step = i - j - 1;
                DataRef {
                    id: format!("data-{:04}", dep_step),
                    data_type: "json".to_string(),
                    source_step: Some(format!("step-{:04}", dep_step)),
                }
            }).collect()
        } else {
            vec![]
        };
        
        let outputs = vec![DataRef {
            id: format!("data-{:04}", i),
            data_type: "json".to_string(),
            source_step: Some(step_id.clone()),
        }];
        
        steps.push(PlanStep {
            id: step_id.clone(),
            operation,
            inputs,
            outputs,
        });
        
        // Add dependencies (create realistic dependency graph)
        if i > 1 {
            let num_deps = std::cmp::min(2, i - 1);
            for j in 0..num_deps {
                let dep_step = i - j - 1;
                dependencies.push(Dependency {
                    from: format!("step-{:04}", dep_step),
                    to: step_id.clone(),
                    dependency_type: if j == 0 { DependencyType::Data } else { DependencyType::Control },
                });
            }
        }
    }
    
    ExecutionPlan {
        id: format!("complexity-test-{}", n),
        steps,
        metadata: PlanMetadata {
            name: format!("Complexity Test Plan (N={})", n),
            description: Some(format!("Generated plan with {} steps for complexity testing", n)),
            created_at: 0, // DETERMINISTIC
            version: "1.0.0".to_string(),
            extra: {
                let mut extra = HashMap::new();
                extra.insert("test_type".to_string(), "complexity".to_string());
                extra.insert("step_count".to_string(), format!("{}", n));
                extra
            },
        },
        dependencies,
    }
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_normalization_complexity_budget_1k() {
    // **CONSTITUTIONAL GUARD:** Normalization must complete in reasonable time for 1K steps
    
    let normalizer = PlanNormalizer::new();
    let plan = create_plan_with_n_steps(1000);
    
    let start = Instant::now();
    let result = normalizer.normalize(&plan);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Normalization should succeed for 1K steps");
    
    // Budget: 1K steps should normalize in < 100ms on reasonable hardware
    assert!(
        duration < Duration::from_millis(100),
        "Normalization took too long: {:?} for 1K steps", duration
    );
    
    let canonical = result.unwrap();
    assert_eq!(canonical.normalized_steps.len(), 1000);
    assert_ne!(canonical.fingerprint().hash, 0);
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_normalization_complexity_budget_5k() {
    // **CONSTITUTIONAL GUARD:** Normalization scaling test for 5K steps
    
    let normalizer = PlanNormalizer::new();
    let plan = create_plan_with_n_steps(5000);
    
    let start = Instant::now();
    let result = normalizer.normalize(&plan);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Normalization should succeed for 5K steps");
    
    // Budget: 5K steps should normalize in < 500ms (linear scaling)
    assert!(
        duration < Duration::from_millis(500),
        "Normalization took too long: {:?} for 5K steps", duration
    );
    
    let canonical = result.unwrap();
    assert_eq!(canonical.normalized_steps.len(), 5000);
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_normalization_complexity_budget_max() {
    // **CONSTITUTIONAL GUARD:** Test at maximum allowed plan size
    
    let normalizer = PlanNormalizer::new();
    let plan = create_plan_with_n_steps(MAX_PLAN_STEPS);
    
    let start = Instant::now();
    let result = normalizer.normalize(&plan);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Normalization should succeed at MAX_PLAN_STEPS");
    
    // Budget: MAX_PLAN_STEPS should normalize in < 1s
    assert!(
        duration < Duration::from_secs(1),
        "Normalization took too long: {:?} for MAX_PLAN_STEPS", duration
    );
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_ir_analysis_complexity_budget() {
    // **CONSTITUTIONAL GUARD:** IR analysis complexity budget
    
    let analyzer = SemanticAnalyzer::new();
    let plan = create_plan_with_n_steps(1000);
    
    let start = Instant::now();
    let result = analyzer.analyze_semantic_dependencies(&plan);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "IR analysis should succeed for 1K steps");
    
    // Budget: IR analysis should complete in < 200ms for 1K steps
    assert!(
        duration < Duration::from_millis(200),
        "IR analysis took too long: {:?} for 1K steps", duration
    );
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_parallelism_analysis_complexity_budget() {
    // **CONSTITUTIONAL GUARD:** Parallelism analysis O(n²) prevention
    
    let planner = IRPlanner::new();
    let plan = create_plan_with_n_steps(500); // Smaller for parallelism analysis
    
    let start = Instant::now();
    let result = planner.analyze_plan(&plan);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Parallelism analysis should succeed for 500 steps");
    
    // Budget: Parallelism analysis should complete in < 300ms for 500 steps
    // This is more expensive due to dependency analysis but should still be reasonable
    assert!(
        duration < Duration::from_millis(300),
        "Parallelism analysis took too long: {:?} for 500 steps", duration
    );
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_complexity_scaling_linearity() {
    // **CONSTITUTIONAL GUARD:** Verify linear scaling (not quadratic)
    
    let normalizer = PlanNormalizer::new();
    
    // Test different sizes and measure scaling
    let sizes = vec![100, 200, 400, 800];
    let mut durations = Vec::new();
    
    for size in sizes {
        let plan = create_plan_with_n_steps(size);
        
        let start = Instant::now();
        let result = normalizer.normalize(&plan);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Normalization should succeed for {} steps", size);
        durations.push((size, duration));
    }
    
    // Check that scaling is roughly linear (not quadratic)
    // If it were O(n²), 800 steps would take 64x longer than 100 steps
    // For linear scaling, it should take ~8x longer
    
    let (size_100, duration_100) = durations[0];
    let (size_800, duration_800) = durations[3];
    
    let size_ratio = size_800 as f64 / size_100 as f64; // 8.0
    let time_ratio = duration_800.as_nanos() as f64 / duration_100.as_nanos() as f64;
    
    // Allow some overhead, but time ratio should be much closer to size ratio than size²
    let quadratic_ratio = size_ratio * size_ratio; // 64.0
    
    assert!(
        time_ratio < quadratic_ratio / 2.0,
        "Scaling appears quadratic: {}x size increase took {}x time (expected < {}x)",
        size_ratio, time_ratio, quadratic_ratio / 2.0
    );
    
    println!("Scaling test: {}x size increase took {:.2}x time (linear would be {:.1}x)",
             size_ratio, time_ratio, size_ratio);
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_memory_usage_budget() {
    // **CONSTITUTIONAL GUARD:** Memory usage should not explode
    
    let normalizer = PlanNormalizer::new();
    
    // Test memory usage for large plans
    let plan = create_plan_with_n_steps(2000);
    
    // This is a basic test - in production you'd use more sophisticated memory monitoring
    let result = normalizer.normalize(&plan);
    assert!(result.is_ok(), "Should handle 2K steps without memory issues");
    
    let canonical = result.unwrap();
    
    // Basic sanity checks
    assert_eq!(canonical.normalized_steps.len(), 2000);
    assert!(canonical.fingerprint().hash != 0);
    
    // The canonical plan should not be dramatically larger than the input
    // (This is a rough heuristic - exact ratios depend on the data structure)
    let input_steps = plan.steps.len();
    let output_steps = canonical.normalized_steps.len();
    assert_eq!(input_steps, output_steps, "Canonical plan should preserve step count");
}

#[test]
fn test_complexity_budget_fallback_behavior() {
    // **CONSTITUTIONAL GUARD:** System should gracefully handle complexity limits
    
    let normalizer = PlanNormalizer::new();
    
    // Test plan that exceeds MAX_PLAN_STEPS
    let oversized_plan = create_plan_with_n_steps(MAX_PLAN_STEPS + 1);
    
    let result = normalizer.normalize(&oversized_plan);
    
    // Should fail gracefully with appropriate error
    assert!(result.is_err(), "Should reject oversized plans");
    
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(
        error_msg.contains("exceeds limit") || error_msg.contains("TooComplex"),
        "Error should indicate size limit exceeded: {}", error_msg
    );
}

#[cfg(test)]
mod complexity_regression_tests {
    use super::*;
    
    #[test]
    fn test_small_plan_performance_baseline() {
        // Baseline performance test for small plans (should be very fast)
        let normalizer = PlanNormalizer::new();
        let plan = create_plan_with_n_steps(10);
        
        let start = Instant::now();
        let result = normalizer.normalize(&plan);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        
        // Small plans should be very fast (< 10ms)
        assert!(
            duration < Duration::from_millis(10),
            "Small plan normalization took too long: {:?}", duration
        );
    }
    
    #[test]
    fn test_empty_plan_performance() {
        // Edge case: empty plan should be instant
        let normalizer = PlanNormalizer::new();
        let empty_plan = ExecutionPlan {
            id: "empty".to_string(),
            steps: vec![],
            metadata: PlanMetadata {
                name: "Empty".to_string(),
                description: None,
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };
        
        let start = Instant::now();
        let result = normalizer.normalize(&empty_plan);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(10), "Empty plan should be very fast: {:?}", duration);
        
        let canonical = result.unwrap();
        assert_eq!(canonical.normalized_steps.len(), 0);
    }
}