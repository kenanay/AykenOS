//! Parallel Map Operation Example
//!
//! This example demonstrates how to use the D2 Parallelism Architecture
//! for parallel map operations that transform data elements.
//!
//! **Usage:**
//! ```bash
//! cargo run --example parallel_map_operation --features phase2-implementation
//! ```

use semantic_cli::bcib::Value;
use semantic_cli::execution_plan::dataflow::DataflowGraph;
use semantic_cli::execution_plan::{
    BlockTerminator, ExecutionMetadata, ExecutionPlan, IRBlock, IRInstruction, ParallelSafety,
};
use semantic_cli::normalizer::RegisterAllocation;
use semantic_cli::parallelism::{
    measure_execution, ContiguousPartitioner, DataPartitioner, DefaultMetricsCollector,
    DeterministicMerger, ExecutionConfig, ImmutableContext, MetricsCollector, ParallelExecutor,
    RayonParallelExecutor, StableIndexMerger,
};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🗺️  Parallel Map Operation Example");
    println!("===================================");

    // Create test dataset
    let dataset = create_numeric_dataset(100_000);
    println!("📊 Created dataset with {} elements", dataset.len());

    // Example 1: Simple transformation (x * 2 + 1)
    println!("\n1️⃣ Simple Transformation: f(x) = x * 2 + 1");
    demonstrate_simple_map(&dataset)?;

    // Example 2: Complex transformation (mathematical function)
    println!("\n2️⃣ Complex Transformation: f(x) = sin(x) + cos(x²)");
    demonstrate_complex_map(&dataset)?;

    // Example 3: String transformation
    println!("\n3️⃣ String Transformation");
    let string_dataset = create_string_dataset(50_000);
    demonstrate_string_map(&string_dataset)?;

    // Example 4: Conditional transformation
    println!("\n4️⃣ Conditional Transformation");
    demonstrate_conditional_map(&dataset)?;

    // Example 5: Performance scaling analysis
    println!("\n5️⃣ Performance Scaling Analysis");
    analyze_scaling_performance()?;

    println!("\n🎉 All map operation examples completed!");
    Ok(())
}

/// Creates a numeric dataset for testing
fn create_numeric_dataset(size: usize) -> Vec<Value> {
    (0..size)
        .map(|i| Value::Number(i as f64 / 100.0)) // Values from 0.0 to size/100
        .collect()
}

/// Creates a string dataset for testing
fn create_string_dataset(size: usize) -> Vec<Value> {
    (0..size)
        .map(|i| Value::String(format!("item_{:06}", i)))
        .collect()
}

/// Creates an IR block for simple numeric transformation
fn create_simple_transform_block() -> IRBlock {
    IRBlock::with_safety(
        1,
        vec![
            IRInstruction::LoadContext {
                context_id: "dataset".to_string(),
                target_register: 0,
            },
            // Load multiplier (2.0)
            IRInstruction::LoadLiteral {
                value: Value::Number(2.0),
                target_register: 1,
            },
            // Load addend (1.0)
            IRInstruction::LoadLiteral {
                value: Value::Number(1.0),
                target_register: 2,
            },
            // Note: In a real implementation, we would have arithmetic instructions
            // For this example, we'll simulate the transformation
        ],
        BlockTerminator::Return { register: 0 },
        ParallelSafety::Safe, // Pure mathematical operations are safe
    )
}

/// Demonstrates simple parallel map operation
fn demonstrate_simple_map(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Sequential implementation
    let (sequential_result, sequential_time) = measure_execution(|| {
        data.iter()
            .map(|value| match value {
                Value::Number(n) => Value::Number(n * 2.0 + 1.0),
                _ => value.clone(),
            })
            .collect::<Vec<_>>()
    });

    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!("   📊 Transformed {} elements", sequential_result.len());

    // Parallel implementation
    let (parallel_result, parallel_time) = measure_execution(|| {
        execute_parallel_map(data, |value| match value {
            Value::Number(n) => Value::Number(n * 2.0 + 1.0),
            _ => value.clone(),
        })
    });

    let parallel_result = parallel_result?;
    println!("   ⏱️  Parallel time: {:?}", parallel_time);
    println!("   📊 Transformed {} elements", parallel_result.len());

    // Calculate speedup
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);

    // Verify correctness (check first few elements)
    let matches = sequential_result
        .iter()
        .zip(parallel_result.iter())
        .take(10)
        .all(|(a, b)| a == b);

    if matches {
        println!("   ✅ Results match (verified first 10 elements)");
    } else {
        println!("   ❌ Results mismatch detected!");
    }

    Ok(())
}

/// Demonstrates complex mathematical transformation
fn demonstrate_complex_map(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Complex transformation: sin(x) + cos(x²)
    let transform_fn = |value: &Value| match value {
        Value::Number(n) => {
            let x = *n;
            let result = x.sin() + (x * x).cos();
            Value::Number(result)
        }
        _ => value.clone(),
    };

    // Sequential implementation
    let (sequential_result, sequential_time) =
        measure_execution(|| data.iter().map(transform_fn).collect::<Vec<_>>());

    println!("   ⏱️  Sequential time: {:?}", sequential_time);

    // Parallel implementation
    let (parallel_result, parallel_time) =
        measure_execution(|| execute_parallel_map(data, transform_fn));

    let parallel_result = parallel_result?;
    println!("   ⏱️  Parallel time: {:?}", parallel_time);

    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);

    // Verify numerical accuracy (allowing for floating-point precision)
    let numerical_matches = sequential_result
        .iter()
        .zip(parallel_result.iter())
        .take(100)
        .all(|(a, b)| match (a, b) {
            (Value::Number(x), Value::Number(y)) => (x - y).abs() < 1e-10,
            _ => a == b,
        });

    if numerical_matches {
        println!("   ✅ Numerical results match (within precision)");
    } else {
        println!("   ❌ Numerical precision issues detected!");
    }

    Ok(())
}

/// Demonstrates string transformation
fn demonstrate_string_map(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // String transformation: uppercase + prefix
    let transform_fn = |value: &Value| match value {
        Value::String(s) => Value::String(format!("PROCESSED_{}", s.to_uppercase())),
        _ => value.clone(),
    };

    // Sequential implementation
    let (sequential_result, sequential_time) =
        measure_execution(|| data.iter().map(transform_fn).collect::<Vec<_>>());

    println!("   ⏱️  Sequential time: {:?}", sequential_time);

    // Parallel implementation
    let (parallel_result, parallel_time) =
        measure_execution(|| execute_parallel_map(data, transform_fn));

    let parallel_result = parallel_result?;
    println!("   ⏱️  Parallel time: {:?}", parallel_time);

    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);

    // Show sample transformations
    println!("   📝 Sample transformations:");
    for i in 0..3 {
        if let (Some(original), Some(transformed)) = (data.get(i), parallel_result.get(i)) {
            println!(
                "      {} -> {}",
                format_value_short(original),
                format_value_short(transformed)
            );
        }
    }

    Ok(())
}

/// Demonstrates conditional transformation
fn demonstrate_conditional_map(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Conditional transformation: square positive numbers, negate negative numbers
    let transform_fn = |value: &Value| match value {
        Value::Number(n) => {
            if *n >= 0.0 {
                Value::Number(n * n) // Square positive numbers
            } else {
                Value::Number(-n) // Make negative numbers positive
            }
        }
        _ => value.clone(),
    };

    // Create mixed dataset
    let mixed_data: Vec<Value> = (0..10000)
        .map(|i| Value::Number(if i % 2 == 0 { i as f64 } else { -(i as f64) }))
        .collect();

    // Sequential implementation
    let (sequential_result, sequential_time) =
        measure_execution(|| mixed_data.iter().map(transform_fn).collect::<Vec<_>>());

    println!("   ⏱️  Sequential time: {:?}", sequential_time);

    // Parallel implementation
    let (parallel_result, parallel_time) =
        measure_execution(|| execute_parallel_map(&mixed_data, transform_fn));

    let parallel_result = parallel_result?;
    println!("   ⏱️  Parallel time: {:?}", parallel_time);

    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);

    // Verify all results are non-negative
    let all_positive = parallel_result.iter().all(|value| match value {
        Value::Number(n) => *n >= 0.0,
        _ => true,
    });

    if all_positive {
        println!("   ✅ All results are non-negative (transformation correct)");
    } else {
        println!("   ❌ Some results are negative (transformation failed)");
    }

    Ok(())
}

/// Analyzes performance scaling with different dataset sizes
fn analyze_scaling_performance() -> Result<(), Box<dyn std::error::Error>> {
    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    println!("   📈 Performance scaling analysis:");
    println!("   Size      | Sequential | Parallel   | Speedup");
    println!("   ----------|------------|------------|--------");

    for size in sizes {
        let data = create_numeric_dataset(size);

        // Simple transformation for consistent measurement
        let transform_fn = |value: &Value| match value {
            Value::Number(n) => Value::Number(n * 2.0 + 1.0),
            _ => value.clone(),
        };

        // Measure sequential
        let (_, sequential_time) =
            measure_execution(|| data.iter().map(transform_fn).collect::<Vec<_>>());

        // Measure parallel
        let (parallel_result, parallel_time) =
            measure_execution(|| execute_parallel_map(&data, transform_fn));

        if let Ok(_) = parallel_result {
            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            println!(
                "   {:>9} | {:>8.2}ms | {:>8.2}ms | {:>5.2}x",
                format_number(size),
                sequential_time.as_secs_f64() * 1000.0,
                parallel_time.as_secs_f64() * 1000.0,
                speedup
            );
        }
    }

    Ok(())
}

/// Generic parallel map implementation using parallelism components
fn execute_parallel_map<F>(
    data: &[Value],
    transform_fn: F,
) -> Result<Vec<Value>, Box<dyn std::error::Error>>
where
    F: Fn(&Value) -> Value + Sync + Send,
{
    use rayon::prelude::*;

    // Use Rayon for simple parallel map
    // In a real implementation, this would use the full parallelism architecture
    let result: Vec<Value> = data.par_iter().map(|value| transform_fn(value)).collect();

    Ok(result)
}

/// Formats a value for short display
fn format_value_short(value: &Value) -> String {
    match value {
        Value::String(s) => {
            if s.len() > 20 {
                format!("{}...", &s[..17])
            } else {
                s.clone()
            }
        }
        Value::Number(n) => format!("{:.2}", n),
        Value::Boolean(b) => b.to_string(),
        Value::Array(_) | Value::List(_) | Value::SortedMap(_) => "[Composite]".to_string(),
    }
}

/// Formats a number with thousand separators
fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Demonstrates memory-efficient streaming map for very large datasets
fn demonstrate_streaming_map() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6️⃣ Memory-Efficient Streaming Map");

    const CHUNK_SIZE: usize = 10_000;
    const TOTAL_SIZE: usize = 1_000_000;

    let start = Instant::now();
    let mut total_processed = 0;

    // Process data in chunks to avoid memory issues
    for chunk_start in (0..TOTAL_SIZE).step_by(CHUNK_SIZE) {
        let chunk_end = (chunk_start + CHUNK_SIZE).min(TOTAL_SIZE);
        let chunk_size = chunk_end - chunk_start;

        // Create chunk
        let chunk: Vec<Value> = (chunk_start..chunk_end)
            .map(|i| Value::Number(i as f64))
            .collect();

        // Process chunk in parallel
        let _processed_chunk = execute_parallel_map(&chunk, |value| match value {
            Value::Number(n) => Value::Number(n.sqrt()),
            _ => value.clone(),
        })?;

        total_processed += chunk_size;

        if chunk_start % (CHUNK_SIZE * 10) == 0 {
            println!(
                "   📊 Processed {}/{} elements ({:.1}%)",
                total_processed,
                TOTAL_SIZE,
                (total_processed as f64 / TOTAL_SIZE as f64) * 100.0
            );
        }
    }

    let total_time = start.elapsed();
    println!("   ✅ Streaming processing completed in {:?}", total_time);
    println!(
        "   📈 Throughput: {:.0} elements/second",
        TOTAL_SIZE as f64 / total_time.as_secs_f64()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_map_correctness() {
        let data = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];

        let result = execute_parallel_map(&data, |value| match value {
            Value::Number(n) => Value::Number(n * 2.0 + 1.0),
            _ => value.clone(),
        })
        .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Value::Number(3.0)); // 1*2+1
        assert_eq!(result[1], Value::Number(5.0)); // 2*2+1
        assert_eq!(result[2], Value::Number(7.0)); // 3*2+1
    }

    #[test]
    fn test_string_map_correctness() {
        let data = vec![
            Value::String("hello".to_string()),
            Value::String("world".to_string()),
        ];

        let result = execute_parallel_map(&data, |value| match value {
            Value::String(s) => Value::String(s.to_uppercase()),
            _ => value.clone(),
        })
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Value::String("HELLO".to_string()));
        assert_eq!(result[1], Value::String("WORLD".to_string()));
    }

    #[test]
    fn test_empty_dataset() {
        let data: Vec<Value> = vec![];

        let result = execute_parallel_map(&data, |value| match value {
            Value::Number(n) => Value::Number(n * 2.0),
            _ => value.clone(),
        })
        .unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_large_dataset_performance() {
        let data = create_numeric_dataset(10_000);

        let start = Instant::now();
        let result = execute_parallel_map(&data, |value| match value {
            Value::Number(n) => Value::Number(n * n),
            _ => value.clone(),
        })
        .unwrap();
        let duration = start.elapsed();

        assert_eq!(result.len(), data.len());

        // Should complete within reasonable time (adjust based on system)
        assert!(
            duration.as_secs() < 5,
            "Processing took too long: {:?}",
            duration
        );
    }
}
