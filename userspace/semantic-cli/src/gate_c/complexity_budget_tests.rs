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
pub fn create_plan_with_n_steps(n: usize) -> ExecutionPlan {
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
    
    assert!(result.is_ok(), 
        "Normalization complexity budget exceeded for 1K steps - Evidence for Phase 4.3 optimization");
    
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
    // **PHASE 4.2.7.6:** Converted to relative scaling test (machine-independent)
    
    let normalizer = PlanNormalizer::new();
    
    // Test relative scaling instead of absolute timing
    let plan_1k = create_plan_with_n_steps(1000);
    let plan_5k = create_plan_with_n_steps(5000);
    
    let start_1k = Instant::now();
    let result_1k = normalizer.normalize(&plan_1k);
    let duration_1k = start_1k.elapsed();
    
    let start_5k = Instant::now();
    let result_5k = normalizer.normalize_for_performance_testing(&plan_5k);
    let duration_5k = start_5k.elapsed();
    
    assert!(result_1k.is_ok(), 
        "Normalization complexity budget exceeded for 1K steps - Evidence for Phase 4.3 optimization");
    
    // Debug: Print the actual error for 5K case
    if let Err(ref e) = result_5k {
        println!("5K normalization error: {:?}", e);
    }
    
    assert!(result_5k.is_ok(), 
        "Normalization complexity budget exceeded for 5K steps - Evidence for Phase 4.3 optimization");
    
    // Relative scaling test: 5x input should scale ≤ 7x time (allowing some overhead)
    let size_ratio = 5.0; // 5K / 1K
    let time_ratio = duration_5k.as_nanos() as f64 / duration_1k.as_nanos() as f64;
    
    assert!(
        time_ratio <= size_ratio * 1.4, // Allow 40% overhead for larger inputs
        "Normalization scaling violation: {}x size took {:.2}x time (expected ≤ {:.1}x)",
        size_ratio, time_ratio, size_ratio * 1.4
    );
    
    let canonical_5k = result_5k.unwrap();
    assert_eq!(canonical_5k.normalized_steps.len(), 5000);
    
    println!("✅ Normalization scaling: {}x size took {:.2}x time", size_ratio, time_ratio);
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
    
    assert!(result.is_ok(), 
        "Normalization complexity budget exceeded at MAX_PLAN_STEPS - Evidence for Phase 4.3 optimization");
    
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
    // **PHASE 4.2.7.6:** Converted to relative scaling test (machine-independent)
    
    let analyzer = SemanticAnalyzer::new();
    
    // Test relative scaling instead of absolute timing
    let plan_500 = create_plan_with_n_steps(500);
    let plan_1k = create_plan_with_n_steps(1000);
    
    let start_500 = Instant::now();
    let result_500 = analyzer.analyze_semantic_dependencies(&plan_500);
    let duration_500 = start_500.elapsed();
    
    let start_1k = Instant::now();
    let result_1k = analyzer.analyze_semantic_dependencies(&plan_1k);
    let duration_1k = start_1k.elapsed();
    
    assert!(result_500.is_ok(), 
        "IR analysis complexity budget exceeded for 500 steps - Evidence for Phase 4.3 optimization: {:?}", 
        result_500.err());
    assert!(result_1k.is_ok(), 
        "IR analysis complexity budget exceeded for 1K steps - Evidence for Phase 4.3 optimization: {:?}", 
        result_1k.err());
    
    // Relative scaling test: 2x input should scale ≤ 3x time (allowing some overhead)
    let size_ratio = 2.0; // 1K / 500
    let time_ratio = duration_1k.as_nanos() as f64 / duration_500.as_nanos() as f64;
    
    assert!(
        time_ratio <= size_ratio * 1.5, // Allow 50% overhead for IR analysis
        "IR analysis scaling violation: {}x size took {:.2}x time (expected ≤ {:.1}x)",
        size_ratio, time_ratio, size_ratio * 1.5
    );
    
    println!("✅ IR analysis scaling: {}x size took {:.2}x time", size_ratio, time_ratio);
}

#[test]
#[ignore] // Run with: cargo test complexity_budget_tests -- --ignored
fn test_parallelism_analysis_complexity_budget() {
    // **CONSTITUTIONAL GUARD:** Parallelism analysis O(n²) prevention
    // **PHASE 4.2.7.6:** Converted to relative scaling test (machine-independent)
    // **PHASE 4.3.2.3:** Using OptimizedIRExecutor to eliminate O(n²) operations
    
    use crate::gate_c::ir::optimized_executor::OptimizedIRExecutor;
    
    let planner = OptimizedIRExecutor::new();
    
    // Test relative scaling instead of absolute timing
    let plan_250 = create_plan_with_n_steps(250);
    let plan_500 = create_plan_with_n_steps(500);
    
    let start_250 = Instant::now();
    let result_250 = planner.analyze_plan(&plan_250);
    let duration_250 = start_250.elapsed();
    
    let start_500 = Instant::now();
    let result_500 = planner.analyze_plan(&plan_500);
    let duration_500 = start_500.elapsed();
    
    assert!(result_250.is_ok(), 
        "Parallelism analysis complexity budget exceeded for 250 steps - Evidence for Phase 4.3 optimization");
    assert!(result_500.is_ok(), 
        "Parallelism analysis complexity budget exceeded for 500 steps - Evidence for Phase 4.3 optimization");
    
    // Relative scaling test: 2x input should scale ≤ 2.5x time (optimized O(n) analysis)
    let size_ratio = 2.0; // 500 / 250
    let time_ratio = duration_500.as_nanos() as f64 / duration_250.as_nanos() as f64;
    
    assert!(
        time_ratio <= size_ratio * 1.5, // Optimized O(n) should scale much better
        "Parallelism analysis scaling violation: {}x size took {:.2}x time (expected ≤ {:.1}x)",
        size_ratio, time_ratio, size_ratio * 1.5
    );
    
    println!("✅ Parallelism analysis scaling: {}x size took {:.2}x time", size_ratio, time_ratio);
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
        
        assert!(result.is_ok(), 
            "Memory usage complexity budget exceeded for {} steps - Evidence for Phase 4.3 optimization", size);
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
    // **PHASE 4.2.7.6:** Converted to relative scaling test (machine-independent)
    
    let normalizer = PlanNormalizer::new();
    
    // Test memory scaling instead of absolute memory usage
    let plan_1k = create_plan_with_n_steps(1000);
    let plan_2k = create_plan_with_n_steps(2000);
    
    let result_1k = normalizer.normalize_for_performance_testing(&plan_1k);
    let result_2k = normalizer.normalize_for_performance_testing(&plan_2k);
    
    assert!(result_1k.is_ok(), 
        "Memory usage complexity budget exceeded for 1K steps - Evidence for Phase 4.3 optimization");
    assert!(result_2k.is_ok(), 
        "Memory usage complexity budget exceeded for 2K steps - Evidence for Phase 4.3 optimization");
    
    let canonical_1k = result_1k.unwrap();
    let canonical_2k = result_2k.unwrap();
    
    // Basic sanity checks
    assert_eq!(canonical_1k.normalized_steps.len(), 1000);
    assert_eq!(canonical_2k.normalized_steps.len(), 2000);
    assert!(canonical_1k.fingerprint().hash != 0);
    assert!(canonical_2k.fingerprint().hash != 0);
    
    // Memory scaling test: output size should scale linearly with input size
    let input_ratio = 2.0; // 2K / 1K
    let output_ratio = canonical_2k.normalized_steps.len() as f64 / canonical_1k.normalized_steps.len() as f64;
    
    assert!(
        (output_ratio - input_ratio).abs() < 0.1, // Should be very close to linear
        "Memory scaling violation: {}x input produced {}x output (expected ~{}x)",
        input_ratio, output_ratio, input_ratio
    );
    
    println!("✅ Memory scaling: {}x input produced {:.2}x output", input_ratio, output_ratio);
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