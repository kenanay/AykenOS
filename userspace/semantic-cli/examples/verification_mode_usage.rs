//! Verification Mode Usage Example
//!
//! This example demonstrates how to use verification mode to detect
//! determinism violations and debug parallel execution issues.
//!
//! **Usage:**
//! ```bash
//! cargo run --example verification_mode_usage --features phase2-implementation
//! ```

use semantic_cli::bcib::Value;
use semantic_cli::execution_plan::dataflow::DataflowGraph;
use semantic_cli::execution_plan::{BlockTerminator, IRBlock, IRInstruction, ParallelSafety};
use semantic_cli::execution_plan::{ExecutionMetadata, ExecutionPlan};
use semantic_cli::normalizer::RegisterAllocation;
use semantic_cli::parallelism::{
    verification::{
        execute_with_verification, DefaultVerificationExecutor, ValueMismatch,
        VerificationDiagnostics, VerificationExecutor, VerificationResult,
    },
    ContiguousPartitioner, ExecutionConfig, ImmutableContext, RayonParallelExecutor,
    StableIndexMerger,
};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🔍 Verification Mode Usage Example");
    println!("===================================");

    // Example 1: Basic verification (should pass)
    println!("\n1️⃣ Basic Verification - Correct Implementation");
    demonstrate_correct_verification()?;

    // Example 2: Simulated determinism violation
    println!("\n2️⃣ Simulated Determinism Violation");
    demonstrate_determinism_violation()?;

    // Example 3: Performance verification
    println!("\n3️⃣ Performance Verification");
    demonstrate_performance_verification()?;

    // Example 4: Large dataset verification
    println!("\n4️⃣ Large Dataset Verification");
    demonstrate_large_dataset_verification()?;

    // Example 5: Verification with different data types
    println!("\n5️⃣ Mixed Data Type Verification");
    demonstrate_mixed_data_verification()?;

    // Example 6: Verification diagnostics analysis
    println!("\n6️⃣ Verification Diagnostics Analysis");
    demonstrate_diagnostics_analysis()?;

    // Example 7: Continuous verification testing
    println!("\n7️⃣ Continuous Verification Testing");
    demonstrate_continuous_verification()?;

    println!("\n🎉 All verification examples completed!");
    Ok(())
}

/// Demonstrates basic verification that should pass
fn demonstrate_correct_verification() -> Result<(), Box<dyn std::error::Error>> {
    let data = create_test_dataset(1000);
    let block = create_safe_ir_block();
    let context = create_test_context();

    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();

    let start = Instant::now();
    let result = execute_with_verification(&block, &data, &context, &executor, &merger)?;
    let verification_time = start.elapsed();

    match result {
        VerificationResult::Match {
            result,
            sequential_time,
            parallel_time,
            verification_overhead,
        } => {
            println!("   ✅ Verification PASSED!");
            println!("      Result size: {} elements", result.len());
            println!("      Sequential time: {:?}", sequential_time);
            println!("      Parallel time: {:?}", parallel_time);
            println!("      Verification overhead: {:?}", verification_overhead);
            println!("      Total verification time: {:?}", verification_time);

            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            println!("      Measured speedup: {:.2}x", speedup);

            if speedup >= 1.0 {
                println!("      🚀 Parallelism is beneficial");
            } else {
                println!("      ⚠️  Sequential execution is faster");
            }
        }
        VerificationResult::Mismatch { diagnostics, .. } => {
            println!("   ❌ Unexpected verification failure!");
            print_diagnostics(&diagnostics);
        }
    }

    Ok(())
}

/// Demonstrates a simulated determinism violation
fn demonstrate_determinism_violation() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mock verification result with intentional mismatch
    let diagnostics = create_mock_mismatch_diagnostics();

    println!("   🔍 Simulating determinism violation...");

    let mismatch_result = VerificationResult::Mismatch {
        parallel_result: vec![
            Value::Number(1.0),
            Value::Number(3.0), // Wrong value
            Value::Number(3.0),
        ],
        sequential_result: vec![
            Value::Number(1.0),
            Value::Number(2.0), // Correct value
            Value::Number(3.0),
        ],
        diagnostics: diagnostics.clone(),
    };

    match mismatch_result {
        VerificationResult::Mismatch {
            parallel_result,
            sequential_result,
            diagnostics,
        } => {
            println!("   ❌ Verification FAILED!");
            println!("      Parallel result length: {}", parallel_result.len());
            println!(
                "      Sequential result length: {}",
                sequential_result.len()
            );

            print_diagnostics(&diagnostics);

            // Show detailed comparison
            println!("   🔍 Detailed Comparison:");
            for (i, (par, seq)) in parallel_result
                .iter()
                .zip(sequential_result.iter())
                .enumerate()
            {
                if par != seq {
                    println!(
                        "      Index {}: parallel={:?}, sequential={:?} ❌",
                        i, par, seq
                    );
                } else {
                    println!("      Index {}: {:?} ✅", i, par);
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Demonstrates performance-focused verification
fn demonstrate_performance_verification() -> Result<(), Box<dyn std::error::Error>> {
    let sizes = vec![100, 1_000, 10_000, 50_000];

    println!("   📈 Performance verification across dataset sizes:");
    println!("   Size     | Sequential | Parallel   | Speedup | Status");
    println!("   ---------|------------|------------|---------|-------");

    for size in sizes {
        let data = create_test_dataset(size);
        let block = create_safe_ir_block();
        let context = create_test_context();

        let executor = RayonParallelExecutor::new();
        let merger = StableIndexMerger::new();

        let result = execute_with_verification(&block, &data, &context, &executor, &merger)?;

        match result {
            VerificationResult::Match {
                sequential_time,
                parallel_time,
                ..
            } => {
                let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
                let status = if speedup >= 2.0 {
                    "Excellent"
                } else if speedup >= 1.5 {
                    "Good"
                } else if speedup >= 1.0 {
                    "Marginal"
                } else {
                    "Poor"
                };

                println!(
                    "   {:>8} | {:>8.2}ms | {:>8.2}ms | {:>5.2}x | {}",
                    format_size(size),
                    sequential_time.as_secs_f64() * 1000.0,
                    parallel_time.as_secs_f64() * 1000.0,
                    speedup,
                    status
                );
            }
            VerificationResult::Mismatch { .. } => {
                println!(
                    "   {:>8} | ERROR      | ERROR      | ERROR   | Failed",
                    format_size(size)
                );
            }
        }
    }

    Ok(())
}

/// Demonstrates verification with large datasets
fn demonstrate_large_dataset_verification() -> Result<(), Box<dyn std::error::Error>> {
    let large_data = create_test_dataset(100_000);
    println!(
        "   📊 Verifying large dataset with {} elements",
        large_data.len()
    );

    let block = create_safe_ir_block();
    let context = create_test_context();

    let verifier = DefaultVerificationExecutor::new();
    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();

    let start = Instant::now();
    let result =
        verifier.execute_with_verification(&block, &large_data, &context, &executor, &merger)?;
    let total_time = start.elapsed();

    match result {
        VerificationResult::Match {
            result,
            sequential_time,
            parallel_time,
            verification_overhead,
        } => {
            println!("   ✅ Large dataset verification PASSED!");
            println!("      Verified {} elements", result.len());
            println!("      Sequential time: {:?}", sequential_time);
            println!("      Parallel time: {:?}", parallel_time);
            println!("      Verification overhead: {:?}", verification_overhead);
            println!("      Total time: {:?}", total_time);

            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            let overhead_ratio = verification_overhead.as_secs_f64() / total_time.as_secs_f64();

            println!("      Speedup: {:.2}x", speedup);
            println!(
                "      Verification overhead: {:.1}%",
                overhead_ratio * 100.0
            );

            // Performance assessment
            if speedup >= 2.0 && overhead_ratio <= 0.2 {
                println!("      🎯 Excellent performance with low verification overhead");
            } else if speedup >= 1.5 {
                println!("      👍 Good performance");
            } else {
                println!("      ⚠️  Consider sequential execution for this workload");
            }
        }
        VerificationResult::Mismatch { diagnostics, .. } => {
            println!("   ❌ Large dataset verification FAILED!");
            print_diagnostics(&diagnostics);
        }
    }

    Ok(())
}

/// Demonstrates verification with mixed data types
fn demonstrate_mixed_data_verification() -> Result<(), Box<dyn std::error::Error>> {
    let mixed_data = create_mixed_dataset(5000);
    println!(
        "   🎭 Verifying mixed data types ({} elements)",
        mixed_data.len()
    );

    let block = create_safe_ir_block();
    let context = create_test_context();

    let executor = RayonParallelExecutor::new();
    let merger = StableIndexMerger::new();

    let result = execute_with_verification(&block, &mixed_data, &context, &executor, &merger)?;

    match result {
        VerificationResult::Match { result, .. } => {
            println!("   ✅ Mixed data verification PASSED!");

            // Analyze result composition
            let mut type_counts = HashMap::new();
            for value in &result {
                let type_name = match value {
                    Value::Number(_) => "Number",
                    Value::String(_) => "String",
                    Value::Boolean(_) => "Boolean",
                    Value::Array(_) | Value::List(_) | Value::SortedMap(_) => "Composite",
                };
                *type_counts.entry(type_name).or_insert(0) += 1;
            }

            println!("      Result composition:");
            for (type_name, count) in type_counts {
                println!("        {}: {} elements", type_name, count);
            }
        }
        VerificationResult::Mismatch { diagnostics, .. } => {
            println!("   ❌ Mixed data verification FAILED!");
            print_diagnostics(&diagnostics);

            // Analyze which data types caused issues
            analyze_type_specific_mismatches(&diagnostics);
        }
    }

    Ok(())
}

/// Demonstrates detailed diagnostics analysis
fn demonstrate_diagnostics_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔬 Analyzing verification diagnostics capabilities...");

    // Create comprehensive mock diagnostics
    let diagnostics = create_comprehensive_diagnostics();

    println!("   📋 Diagnostic Information:");
    print_diagnostics(&diagnostics);

    // Demonstrate diagnostic analysis
    analyze_diagnostics(&diagnostics);

    Ok(())
}

/// Demonstrates continuous verification testing
fn demonstrate_continuous_verification() -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔄 Running continuous verification tests...");

    let test_cases = vec![
        ("Small dataset", create_test_dataset(100)),
        ("Medium dataset", create_test_dataset(1000)),
        ("Large dataset", create_test_dataset(10000)),
        ("Mixed types", create_mixed_dataset(1000)),
        ("Edge case - empty", vec![]),
        ("Edge case - single", vec![Value::Number(42.0)]),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (test_name, data) in test_cases {
        print!("   Testing {}: ", test_name);

        if data.is_empty() {
            println!("SKIPPED (empty dataset)");
            continue;
        }

        let block = create_safe_ir_block();
        let context = create_test_context();
        let executor = RayonParallelExecutor::new();
        let merger = StableIndexMerger::new();

        match execute_with_verification(&block, &data, &context, &executor, &merger) {
            Ok(VerificationResult::Match { .. }) => {
                println!("PASSED ✅");
                passed += 1;
            }
            Ok(VerificationResult::Mismatch { .. }) => {
                println!("FAILED ❌");
                failed += 1;
            }
            Err(e) => {
                println!("ERROR: {} ⚠️", e);
                failed += 1;
            }
        }
    }

    println!("   📊 Continuous verification results:");
    println!("      Passed: {}", passed);
    println!("      Failed: {}", failed);
    println!(
        "      Success rate: {:.1}%",
        (passed as f64 / (passed + failed) as f64) * 100.0
    );

    if failed == 0 {
        println!("      🎉 All verification tests passed!");
    } else {
        println!("      ⚠️  Some verification tests failed - investigation needed");
    }

    Ok(())
}

// Helper functions

fn create_test_dataset(size: usize) -> Vec<Value> {
    (0..size).map(|i| Value::Number(i as f64)).collect()
}

fn create_mixed_dataset(size: usize) -> Vec<Value> {
    (0..size)
        .map(|i| match i % 3 {
            0 => Value::Number(i as f64),
            1 => Value::String(format!("item_{}", i)),
            _ => Value::Boolean(i % 2 == 0),
        })
        .collect()
}

fn create_safe_ir_block() -> IRBlock {
    IRBlock::with_safety(
        1,
        vec![
            IRInstruction::LoadContext {
                context_id: "test".to_string(),
                target_register: 0,
            },
            IRInstruction::LoadLiteral {
                value: Value::Number(1.0),
                target_register: 1,
            },
        ],
        BlockTerminator::Return { register: 0 },
        ParallelSafety::Safe,
    )
}

fn create_test_context() -> ImmutableContext {
    let execution_plan = ExecutionPlan::new(
        vec![],
        0,
        RegisterAllocation {
            allocated_registers: vec![],
            register_dependencies: HashMap::new(),
            next_register: 0,
        },
        DataflowGraph::new(),
        ExecutionMetadata::new("verification_test".to_string(), 0, 0, 0),
    );

    ImmutableContext {
        execution_plan,
        config: ExecutionConfig::default(),
    }
}

fn create_mock_mismatch_diagnostics() -> VerificationDiagnostics {
    VerificationDiagnostics {
        input_data: vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
        block_id: 1,
        partition_count: 2,
        first_mismatch_index: Some(1),
        value_mismatches: vec![ValueMismatch {
            index: 1,
            parallel_value: Value::Number(3.0),
            sequential_value: Value::Number(2.0),
            description: "Parallel execution produced 3.0, sequential produced 2.0".to_string(),
        }],
        context_info: "Simulated determinism violation for demonstration".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

fn create_comprehensive_diagnostics() -> VerificationDiagnostics {
    VerificationDiagnostics {
        input_data: vec![
            Value::Number(1.0),
            Value::String("test".to_string()),
            Value::Boolean(true),
            Value::Number(2.0),
        ],
        block_id: 42,
        partition_count: 4,
        first_mismatch_index: Some(2),
        value_mismatches: vec![
            ValueMismatch {
                index: 2,
                parallel_value: Value::Boolean(false),
                sequential_value: Value::Boolean(true),
                description: "Boolean value mismatch: parallel=false, sequential=true".to_string(),
            },
            ValueMismatch {
                index: 3,
                parallel_value: Value::Number(2.1),
                sequential_value: Value::Number(2.0),
                description: "Numerical precision issue: parallel=2.1, sequential=2.0".to_string(),
            },
        ],
        context_info: "Comprehensive diagnostics example with multiple mismatch types".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

fn print_diagnostics(diagnostics: &VerificationDiagnostics) {
    println!("      📋 Diagnostics:");
    println!("         Block ID: {}", diagnostics.block_id);
    println!("         Partition count: {}", diagnostics.partition_count);
    println!("         Input data size: {}", diagnostics.input_data.len());
    println!(
        "         First mismatch index: {:?}",
        diagnostics.first_mismatch_index
    );
    println!(
        "         Total mismatches: {}",
        diagnostics.value_mismatches.len()
    );
    println!("         Context: {}", diagnostics.context_info);
    println!("         Timestamp: {}", diagnostics.timestamp);

    if !diagnostics.value_mismatches.is_empty() {
        println!("         🔍 Value Mismatches:");
        for (i, mismatch) in diagnostics.value_mismatches.iter().take(5).enumerate() {
            println!("            {}: {}", i + 1, mismatch.description);
        }

        if diagnostics.value_mismatches.len() > 5 {
            println!(
                "            ... and {} more mismatches",
                diagnostics.value_mismatches.len() - 5
            );
        }
    }
}

fn analyze_diagnostics(diagnostics: &VerificationDiagnostics) {
    println!("   🔬 Diagnostic Analysis:");

    // Analyze mismatch patterns
    let mut type_mismatches = HashMap::new();
    for mismatch in &diagnostics.value_mismatches {
        let parallel_type = get_value_type(&mismatch.parallel_value);
        let sequential_type = get_value_type(&mismatch.sequential_value);

        if parallel_type != sequential_type {
            *type_mismatches.entry("Type mismatch").or_insert(0) += 1;
        } else {
            *type_mismatches.entry("Value mismatch").or_insert(0) += 1;
        }
    }

    for (category, count) in type_mismatches {
        println!("      {}: {} occurrences", category, count);
    }

    // Analyze mismatch distribution
    let mismatch_ratio =
        diagnostics.value_mismatches.len() as f64 / diagnostics.input_data.len() as f64;
    println!("      Mismatch ratio: {:.1}%", mismatch_ratio * 100.0);

    if mismatch_ratio > 0.5 {
        println!("      ⚠️  High mismatch ratio - systematic issue likely");
    } else if mismatch_ratio > 0.1 {
        println!("      ⚠️  Moderate mismatch ratio - investigate specific cases");
    } else {
        println!("      ✅ Low mismatch ratio - isolated issues");
    }

    // Suggest debugging steps
    println!("      💡 Debugging suggestions:");
    if diagnostics.partition_count > 1 {
        println!("         - Check partition boundary handling");
        println!("         - Verify index mapping correctness");
    }
    if !diagnostics.value_mismatches.is_empty() {
        println!("         - Review parallel execution logic");
        println!("         - Check for race conditions");
    }
}

fn analyze_type_specific_mismatches(diagnostics: &VerificationDiagnostics) {
    println!("      🎭 Type-specific mismatch analysis:");

    let mut type_issues = HashMap::new();
    for mismatch in &diagnostics.value_mismatches {
        let value_type = get_value_type(&mismatch.parallel_value);
        *type_issues.entry(value_type).or_insert(0) += 1;
    }

    for (value_type, count) in type_issues {
        println!("         {}: {} mismatches", value_type, count);
    }
}

fn get_value_type(value: &Value) -> &'static str {
    match value {
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Boolean(_) => "Boolean",
        Value::Array(_) | Value::List(_) | Value::SortedMap(_) => "Composite",
    }
}

fn format_size(size: usize) -> String {
    if size >= 1_000_000 {
        format!("{:.1}M", size as f64 / 1_000_000.0)
    } else if size >= 1_000 {
        format!("{:.1}K", size as f64 / 1_000.0)
    } else {
        size.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_with_matching_results() {
        let data = vec![Value::Number(1.0), Value::Number(2.0)];
        let block = create_safe_ir_block();
        let context = create_test_context();
        let executor = RayonParallelExecutor::new();
        let merger = StableIndexMerger::new();

        let result = execute_with_verification(&block, &data, &context, &executor, &merger);

        // Should not panic and should return a result
        assert!(result.is_ok());
    }

    #[test]
    fn test_diagnostics_creation() {
        let diagnostics = create_mock_mismatch_diagnostics();

        assert_eq!(diagnostics.block_id, 1);
        assert_eq!(diagnostics.partition_count, 2);
        assert_eq!(diagnostics.first_mismatch_index, Some(1));
        assert_eq!(diagnostics.value_mismatches.len(), 1);
    }

    #[test]
    fn test_mixed_dataset_creation() {
        let data = create_mixed_dataset(6);

        assert_eq!(data.len(), 6);

        // Check type distribution
        let mut type_counts = HashMap::new();
        for value in &data {
            let type_name = get_value_type(value);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        assert!(type_counts.contains_key("Number"));
        assert!(type_counts.contains_key("String"));
        assert!(type_counts.contains_key("Boolean"));
    }

    #[test]
    fn test_diagnostic_analysis() {
        let diagnostics = create_comprehensive_diagnostics();

        // Should not panic
        analyze_diagnostics(&diagnostics);
        analyze_type_specific_mismatches(&diagnostics);
    }
}
