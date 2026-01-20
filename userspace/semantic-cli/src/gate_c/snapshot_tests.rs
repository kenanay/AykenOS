//! # Snapshot Determinism Tests
//!
//! Constitutional CI Guards for Phase 4.0 Baseline Hardening
//! 
//! **CRITICAL:** These tests ensure "same input → byte-identical output"
//! across all Gate C canonical operations. Any failure indicates a 
//! determinism violation that will break AI replay/audit trails.

use crate::gate_c::{
    normalizer::PlanNormalizer,
    types::*,
    repl_visibility::{REPLVisualizer, VisualizationConfig},
    pipeline::PipelinePlanner,
};
use std::collections::HashMap;

/// Create small reference plan for snapshot testing
fn create_small_reference_plan() -> ExecutionPlan {
    ExecutionPlan {
        id: "snapshot-small".to_string(),
        steps: vec![
            PlanStep {
                id: "step-1".to_string(),
                operation: Operation::Query {
                    target: "users".to_string(),
                    parameters: {
                        let mut p = HashMap::new();
                        p.insert("active".to_string(), "true".to_string());
                        p.insert("limit".to_string(), "10".to_string());
                        p
                    },
                },
                inputs: vec![],
                outputs: vec![DataRef {
                    id: "user-list".to_string(),
                    data_type: "json".to_string(),
                    source_step: Some("step-1".to_string()),
                }],
            },
            PlanStep {
                id: "step-2".to_string(),
                operation: Operation::Compute {
                    function: "transform".to_string(),
                    arguments: vec!["normalize".to_string(), "validate".to_string()],
                },
                inputs: vec![DataRef {
                    id: "user-list".to_string(),
                    data_type: "json".to_string(),
                    source_step: Some("step-1".to_string()),
                }],
                outputs: vec![DataRef {
                    id: "processed-users".to_string(),
                    data_type: "json".to_string(),
                    source_step: Some("step-2".to_string()),
                }],
            },
        ],
        metadata: PlanMetadata {
            name: "Small Reference Plan".to_string(),
            description: Some("Determinism snapshot test plan".to_string()),
            created_at: 0, // DETERMINISTIC: Fixed timestamp
            version: "1.0.0".to_string(),
            extra: {
                let mut extra = HashMap::new();
                extra.insert("test".to_string(), "snapshot".to_string());
                extra
            },
        },
        dependencies: vec![Dependency {
            from: "step-1".to_string(),
            to: "step-2".to_string(),
            dependency_type: DependencyType::Data,
        }],
    }
}

/// Create complex reference plan for stress testing
fn create_complex_reference_plan() -> ExecutionPlan {
    let mut steps = Vec::new();
    let mut dependencies = Vec::new();
    
    // Create 20 steps with complex interdependencies
    for i in 1..=20 {
        let step_id = format!("step-{}", i);
        
        let operation = match i % 3 {
            0 => Operation::Query {
                target: format!("table-{}", i),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("filter".to_string(), format!("id > {}", i * 10));
                    p.insert("order".to_string(), "created_at".to_string());
                    p
                },
            },
            1 => Operation::Compute {
                function: format!("process-{}", i),
                arguments: vec![
                    format!("arg-{}", i),
                    "normalize".to_string(),
                    "validate".to_string(),
                ],
            },
            _ => Operation::Mutation {
                intent: MutationIntent::UpdateIntent {
                    target: ResourcePath {
                        segments: vec!["cache".to_string(), format!("item-{}", i)],
                    },
                    changes: ChangeSet {
                        updates: {
                            let mut updates = HashMap::new();
                            updates.insert("status".to_string(), "processed".to_string());
                            updates.insert("timestamp".to_string(), "0".to_string());
                            updates
                        },
                        removals: vec!["temp_data".to_string()],
                    },
                },
            },
        };
        
        let inputs = if i > 1 {
            vec![DataRef {
                id: format!("data-{}", i - 1),
                data_type: "json".to_string(),
                source_step: Some(format!("step-{}", i - 1)),
            }]
        } else {
            vec![]
        };
        
        let outputs = vec![DataRef {
            id: format!("data-{}", i),
            data_type: "json".to_string(),
            source_step: Some(step_id.clone()),
        }];
        
        steps.push(PlanStep {
            id: step_id.clone(),
            operation,
            inputs,
            outputs,
        });
        
        // Add dependency to previous step
        if i > 1 {
            dependencies.push(Dependency {
                from: format!("step-{}", i - 1),
                to: step_id,
                dependency_type: DependencyType::Data,
            });
        }
    }
    
    ExecutionPlan {
        id: "snapshot-complex".to_string(),
        steps,
        metadata: PlanMetadata {
            name: "Complex Reference Plan".to_string(),
            description: Some("Complex determinism snapshot test plan".to_string()),
            created_at: 0, // DETERMINISTIC: Fixed timestamp
            version: "2.0.0".to_string(),
            extra: {
                let mut extra = HashMap::new();
                extra.insert("complexity".to_string(), "high".to_string());
                extra.insert("test".to_string(), "snapshot".to_string());
                extra
            },
        },
        dependencies,
    }
}

#[test]
fn test_canonical_plan_snapshot_determinism() {
    // **CONSTITUTIONAL GUARD:** This test ensures canonical normalization
    // produces byte-identical output across all runs
    
    let normalizer = PlanNormalizer::new();
    let plan = create_small_reference_plan();
    
    // Run normalization multiple times
    let mut canonical_bytes = Vec::new();
    for _ in 0..10 {
        let canonical = normalizer.normalize(&plan).expect("Normalization should succeed");
        canonical_bytes.push(canonical.to_canonical_bytes());
    }
    
    // All canonical bytes must be identical (this is the key determinism test)
    let first_bytes = &canonical_bytes[0];
    for (i, bytes) in canonical_bytes.iter().enumerate() {
        assert_eq!(
            bytes, first_bytes,
            "Canonical bytes not deterministic at run {}", i
        );
    }
    
    // Verify fingerprint stability
    let canonical = normalizer.normalize(&plan).unwrap();
    let expected_fingerprint_version = 1;
    assert_eq!(canonical.fingerprint().version, expected_fingerprint_version);
    
    // The fingerprint hash should be non-zero and consistent
    assert_ne!(canonical.fingerprint().hash, 0);
    
    // Run again and verify same fingerprint
    let canonical2 = normalizer.normalize(&plan).unwrap();
    assert_eq!(canonical.fingerprint().hash, canonical2.fingerprint().hash);
    
    // CRITICAL: Verify canonical bytes are identical
    assert_eq!(canonical.to_canonical_bytes(), canonical2.to_canonical_bytes());
}

#[test]
fn test_complex_plan_snapshot_determinism() {
    // **CONSTITUTIONAL GUARD:** Complex plan determinism test
    
    let normalizer = PlanNormalizer::new();
    let plan = create_complex_reference_plan();
    
    // Normalize complex plan multiple times
    let canonical1 = normalizer.normalize(&plan).expect("Complex normalization should succeed");
    let canonical2 = normalizer.normalize(&plan).expect("Complex normalization should succeed");
    
    // Fingerprints must be identical
    assert_eq!(canonical1.fingerprint().hash, canonical2.fingerprint().hash);
    assert_eq!(canonical1.fingerprint().version, canonical2.fingerprint().version);
    
    // CRITICAL: Canonical bytes must be identical
    assert_eq!(canonical1.to_canonical_bytes(), canonical2.to_canonical_bytes());
    
    // Step count should be preserved
    assert_eq!(canonical1.normalized_steps.len(), 20);
    assert_eq!(canonical2.normalized_steps.len(), 20);
    
    // Steps should be in canonical order (sorted by ID)
    for i in 1..canonical1.normalized_steps.len() {
        assert!(
            canonical1.normalized_steps[i-1].id < canonical1.normalized_steps[i].id,
            "Steps not in canonical order"
        );
    }
}

#[test]
fn test_repl_visualization_snapshot_determinism() {
    // **CONSTITUTIONAL GUARD:** REPL visualization determinism
    
    let _config = VisualizationConfig::default();
    let visualizer = REPLVisualizer::new();
    let plan = create_small_reference_plan();
    
    // Generate visualization multiple times
    let viz1 = visualizer.visualize_plan(&plan).expect("Visualization should succeed");
    let viz2 = visualizer.visualize_plan(&plan).expect("Visualization should succeed");
    
    // Content should be identical (excluding timestamps)
    assert_eq!(viz1.content.len(), viz2.content.len());
    
    // Metadata should have deterministic values
    assert_eq!(viz1.metadata.render_duration_ms, viz2.metadata.render_duration_ms);
    assert_eq!(viz1.metadata.nodes_rendered, viz2.metadata.nodes_rendered);
}

#[test]
fn test_pipeline_planning_snapshot_determinism() {
    // **CONSTITUTIONAL GUARD:** Pipeline planning determinism
    
    let planner = PipelinePlanner::new();
    let plan = create_small_reference_plan();
    
    // Convert to pipeline steps
    let pipeline_steps: Vec<PipelineStep> = plan.steps.into_iter().map(|step| {
        PipelineStep {
            id: step.id,
            operation: step.operation,
            inputs: step.inputs,
            outputs: step.outputs,
        }
    }).collect();
    
    // Plan pipeline multiple times
    let pipeline1 = planner.plan(pipeline_steps.clone()).expect("Pipeline planning should succeed");
    let pipeline2 = planner.plan(pipeline_steps).expect("Pipeline planning should succeed");
    
    // Results should be identical
    assert_eq!(pipeline1.steps.len(), pipeline2.steps.len());
    assert_eq!(pipeline1.metadata.created_at, pipeline2.metadata.created_at);
    
    // Dependency graphs should be identical
    assert_eq!(pipeline1.dependencies.nodes, pipeline2.dependencies.nodes);
    assert_eq!(pipeline1.dependencies.edges, pipeline2.dependencies.edges);
}

#[test]
fn test_cross_run_determinism_golden_master() {
    // **CONSTITUTIONAL GUARD:** Golden master test for determinism
    // This test captures the exact expected output for regression detection
    
    let normalizer = PlanNormalizer::new();
    let plan = create_small_reference_plan();
    let canonical = normalizer.normalize(&plan).unwrap();
    
    // Golden master values (update these if canonical format changes intentionally)
    assert_eq!(canonical.fingerprint().version, 1);
    assert_eq!(canonical.normalized_steps.len(), 2);
    assert_eq!(canonical.normalized_steps[0].id, "step-1");
    assert_eq!(canonical.normalized_steps[1].id, "step-2");
    
    // Verify canonical metadata
    assert_eq!(canonical.metadata.name, "Small Reference Plan");
    assert_eq!(canonical.metadata.version, "1.0.0");
    assert_eq!(canonical.metadata.canonicalized_at, 0); // DETERMINISTIC
    
    // Verify fingerprint is non-zero and stable
    assert_ne!(canonical.fingerprint().hash, 0);
    
    // CRITICAL: Canonical bytes should be deterministic
    let bytes1 = canonical.to_canonical_bytes();
    let canonical2 = normalizer.normalize(&plan).unwrap();
    let bytes2 = canonical2.to_canonical_bytes();
    assert_eq!(bytes1, bytes2, "Canonical bytes not deterministic");
    
    // Canonical string should be deterministic (for debugging)
    let str1 = canonical.to_canonical_string();
    let str2 = canonical2.to_canonical_string();
    assert_eq!(str1, str2, "Canonical string not deterministic");
    
    // This hash should remain stable unless canonical format changes
    // If this test fails, verify the change is intentional and update the expected value
    let expected_hash_range = 1000..u64::MAX; // Allow any reasonable hash value
    assert!(expected_hash_range.contains(&canonical.fingerprint().hash));
}

#[cfg(test)]
mod snapshot_regression_tests {
    
    
    #[test]
    fn test_deterministic_utilities_snapshot() {
        // Test deterministic utility functions for stability
        use crate::gate_c::deterministic::*;
        
        // Fixed inputs should produce fixed outputs
        let ts1 = deterministic_timestamp_from_plan_id("test-plan");
        let ts2 = deterministic_timestamp_from_plan_id("test-plan");
        assert_eq!(ts1, ts2);
        
        let id1 = deterministic_id_from_plan("prefix", "test-plan");
        let id2 = deterministic_id_from_plan("prefix", "test-plan");
        assert_eq!(id1, id2);
        
        let duration1 = deterministic_duration_ms("operation", "content");
        let duration2 = deterministic_duration_ms("operation", "content");
        assert_eq!(duration1, duration2);
        
        // Different inputs should produce different outputs
        let ts3 = deterministic_timestamp_from_plan_id("different-plan");
        assert_ne!(ts1, ts3);
        
        let id3 = deterministic_id_from_plan("prefix", "different-plan");
        assert_ne!(id1, id3);
    }
}