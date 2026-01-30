//! Parallel Aggregation Example
//!
//! This example demonstrates how to use the D2 Parallelism Architecture
//! for parallel aggregation operations like sum, count, min, max, and average.
//!
//! **Usage:**
//! ```bash
//! cargo run --example parallel_aggregation --features phase2-implementation
//! ```

use semantic_cli::bcib::Value;
use semantic_cli::parallelism::{
    DefaultReductionHandler, ReductionHandler, ReductionType, operations,
    measure_execution
};
use rayon::prelude::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("📊 Parallel Aggregation Example");
    println!("================================");
    
    // Create test datasets
    let numeric_data = create_numeric_dataset(1_000_000);
    let mixed_data = create_mixed_dataset(500_000);
    
    println!("📈 Created datasets:");
    println!("   - Numeric: {} elements", numeric_data.len());
    println!("   - Mixed: {} elements", mixed_data.len());
    
    // Example 1: Sum aggregation
    println!("\n1️⃣ Sum Aggregation");
    demonstrate_sum_aggregation(&numeric_data)?;
    
    // Example 2: Count aggregation
    println!("\n2️⃣ Count Aggregation");
    demonstrate_count_aggregation(&mixed_data)?;
    
    // Example 3: Min/Max aggregation
    println!("\n3️⃣ Min/Max Aggregation");
    demonstrate_min_max_aggregation(&numeric_data)?;
    
    // Example 4: Average aggregation
    println!("\n4️⃣ Average Aggregation");
    demonstrate_average_aggregation(&numeric_data)?;
    
    // Example 5: Complex aggregation (variance)
    println!("\n5️⃣ Complex Aggregation (Variance)");
    demonstrate_variance_aggregation(&numeric_data)?;
    
    // Example 6: String concatenation (non-commutative)
    println!("\n6️⃣ String Concatenation (Non-Commutative)");
    demonstrate_string_concatenation()?;
    
    // Example 7: Grouped aggregation
    println!("\n7️⃣ Grouped Aggregation");
    demonstrate_grouped_aggregation(&mixed_data)?;
    
    // Example 8: Performance comparison
    println!("\n8️⃣ Performance Comparison");
    compare_aggregation_performance(&numeric_data)?;
    
    println!("\n🎉 All aggregation examples completed!");
    Ok(())
}

/// Creates a large numeric dataset for testing
fn create_numeric_dataset(size: usize) -> Vec<Value> {
    (1..=size)
        .map(|i| Value::Number(i as f64))
        .collect()
}

/// Creates a mixed dataset with different value types
fn create_mixed_dataset(size: usize) -> Vec<Value> {
    (0..size)
        .map(|i| match i % 4 {
            0 => Value::Number(i as f64),
            1 => Value::String(format!("item_{}", i)),
            2 => Value::Boolean(i % 2 == 0),
            _ => Value::Number((i as f64).sqrt()),
        })
        .collect()
}

/// Demonstrates sum aggregation with commutative reduction
fn demonstrate_sum_aggregation(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    let handler = DefaultReductionHandler::new();
    
    // Extract numeric values
    let numeric_values: Vec<Value> = data
        .iter()
        .filter_map(|v| match v {
            Value::Number(_) => Some(v.clone()),
            _ => None,
        })
        .collect();
    
    println!("   📊 Summing {} numeric values", numeric_values.len());
    
    // Sequential sum
    let (sequential_sum, sequential_time) = measure_execution(|| {
        numeric_values.iter().fold(0.0, |acc, v| match v {
            Value::Number(n) => acc + n,
            _ => acc,
        })
    });
    
    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!("   🔢 Sequential sum: {:.2}", sequential_sum);
    
    // Parallel sum using reduction handler
    let (parallel_result, parallel_time) = measure_execution(|| {
        operations::sum(&handler, numeric_values.clone())
    });
    
    let parallel_sum = match parallel_result? {
        Value::Number(n) => n,
        _ => 0.0,
    };
    
    println!("   ⏱️  Parallel time: {:?}", parallel_time);
    println!("   🔢 Parallel sum: {:.2}", parallel_sum);
    
    // Calculate speedup
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);
    
    // Verify correctness
    let difference = (sequential_sum - parallel_sum).abs();
    if difference < 1e-6 {
        println!("   ✅ Results match (difference: {:.2e})", difference);
    } else {
        println!("   ❌ Results differ by {:.2e}", difference);
    }
    
    Ok(())
}

/// Demonstrates count aggregation
fn demonstrate_count_aggregation(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Count different value types
    let counts = count_value_types(data);
    
    println!("   📊 Value type counts:");
    println!("      Numbers: {}", counts.numbers);
    println!("      Strings: {}", counts.strings);
    println!("      Booleans: {}", counts.booleans);
    println!("      Total: {}", counts.total());
    
    // Parallel count using Rayon
    let (parallel_counts, parallel_time) = measure_execution(|| {
        count_value_types_parallel(data)
    });
    
    println!("   ⏱️  Parallel count time: {:?}", parallel_time);
    println!("   📊 Parallel counts match: {}", counts == parallel_counts);
    
    Ok(())
}

/// Demonstrates min/max aggregation
fn demonstrate_min_max_aggregation(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    let handler = DefaultReductionHandler::new();
    
    // Take subset for faster demonstration
    let subset: Vec<Value> = data.iter().take(100_000).cloned().collect();
    
    // Sequential min/max
    let (sequential_result, sequential_time) = measure_execution(|| {
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        
        for value in &subset {
            if let Value::Number(n) = value {
                min_val = min_val.min(*n);
                max_val = max_val.max(*n);
            }
        }
        
        (min_val, max_val)
    });
    
    let (seq_min, seq_max) = sequential_result;
    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!("   📊 Sequential min: {:.2}, max: {:.2}", seq_min, seq_max);
    
    // Parallel min/max using reduction operations
    let (parallel_min_result, min_time) = measure_execution(|| {
        operations::min(&handler, subset.clone())
    });
    
    let (parallel_max_result, max_time) = measure_execution(|| {
        operations::max(&handler, subset.clone())
    });
    
    let parallel_min = match parallel_min_result? {
        Value::Number(n) => n,
        _ => f64::NAN,
    };
    
    let parallel_max = match parallel_max_result? {
        Value::Number(n) => n,
        _ => f64::NAN,
    };
    
    let total_parallel_time = min_time + max_time;
    println!("   ⏱️  Parallel time: {:?}", total_parallel_time);
    println!("   📊 Parallel min: {:.2}, max: {:.2}", parallel_min, parallel_max);
    
    // Verify correctness
    let min_match = (seq_min - parallel_min).abs() < 1e-6;
    let max_match = (seq_max - parallel_max).abs() < 1e-6;
    
    if min_match && max_match {
        println!("   ✅ Min/Max results match");
    } else {
        println!("   ❌ Min/Max results differ");
    }
    
    Ok(())
}

/// Demonstrates average aggregation
fn demonstrate_average_aggregation(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Take subset for demonstration
    let subset: Vec<Value> = data.iter().take(100_000).cloned().collect();
    
    // Sequential average
    let (sequential_avg, sequential_time) = measure_execution(|| {
        let mut sum = 0.0;
        let mut count = 0;
        
        for value in &subset {
            if let Value::Number(n) = value {
                sum += n;
                count += 1;
            }
        }
        
        if count > 0 { sum / count as f64 } else { 0.0 }
    });
    
    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!("   📊 Sequential average: {:.2}", sequential_avg);
    
    // Parallel average using Rayon
    let (parallel_avg, parallel_time) = measure_execution(|| {
        let numeric_values: Vec<f64> = subset
            .par_iter()
            .filter_map(|v| match v {
                Value::Number(n) => Some(*n),
                _ => None,
            })
            .collect();
        
        if numeric_values.is_empty() {
            0.0
        } else {
            let sum: f64 = numeric_values.par_iter().sum();
            sum / numeric_values.len() as f64
        }
    });
    
    println!("   ⏱️  Parallel time: {:?}", parallel_time);
    println!("   📊 Parallel average: {:.2}", parallel_avg);
    
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);
    
    // Verify correctness
    let difference = (sequential_avg - parallel_avg).abs();
    if difference < 1e-6 {
        println!("   ✅ Average results match");
    } else {
        println!("   ❌ Average results differ by {:.2e}", difference);
    }
    
    Ok(())
}

/// Demonstrates variance calculation (complex aggregation)
fn demonstrate_variance_aggregation(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Take smaller subset for complex calculation
    let subset: Vec<Value> = data.iter().take(50_000).cloned().collect();
    
    // Sequential variance
    let (sequential_variance, sequential_time) = measure_execution(|| {
        calculate_variance_sequential(&subset)
    });
    
    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!("   📊 Sequential variance: {:.2}", sequential_variance);
    
    // Parallel variance
    let (parallel_variance, parallel_time) = measure_execution(|| {
        calculate_variance_parallel(&subset)
    });
    
    println!("   ⏱️  Parallel time: {:?}", parallel_time);
    println!("   📊 Parallel variance: {:.2}", parallel_variance);
    
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("   🚀 Speedup: {:.2}x", speedup);
    
    // Verify correctness
    let difference = (sequential_variance - parallel_variance).abs();
    if difference < 1e-3 {
        println!("   ✅ Variance results match");
    } else {
        println!("   ❌ Variance results differ by {:.2e}", difference);
    }
    
    Ok(())
}

/// Demonstrates string concatenation (non-commutative operation)
fn demonstrate_string_concatenation() -> Result<(), Box<dyn std::error::Error>> {
    let handler = DefaultReductionHandler::new();
    
    // Create string dataset
    let strings: Vec<Value> = (0..1000)
        .map(|i| Value::String(format!("part{}", i)))
        .collect();
    
    println!("   📝 Concatenating {} strings", strings.len());
    
    // Sequential concatenation
    let (sequential_result, sequential_time) = measure_execution(|| {
        strings.iter().fold(String::new(), |mut acc, v| {
            if let Value::String(s) = v {
                if !acc.is_empty() {
                    acc.push(',');
                }
                acc.push_str(s);
            }
            acc
        })
    });
    
    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!("   📝 Sequential result length: {} chars", sequential_result.len());
    
    // Parallel concatenation using non-commutative reduction
    let indexed_strings: Vec<(usize, Value)> = strings
        .into_iter()
        .enumerate()
        .collect();
    
    let (parallel_result, parallel_time) = measure_execution(|| {
        operations::concat(&handler, indexed_strings)
    });
    
    let parallel_string = match parallel_result? {
        Value::String(s) => s,
        _ => String::new(),
    };
    
    println!("   ⏱️  Parallel time: {:?}", parallel_time);
    println!("   📝 Parallel result length: {} chars", parallel_string.len());
    
    // Verify order preservation (check first few parts)
    let seq_starts_correctly = sequential_result.starts_with("part0,part1,part2");
    let par_starts_correctly = parallel_string.starts_with("part0part1part2");
    
    if seq_starts_correctly && par_starts_correctly {
        println!("   ✅ Order preserved in both implementations");
    } else {
        println!("   ❌ Order preservation issue detected");
    }
    
    Ok(())
}

/// Demonstrates grouped aggregation
fn demonstrate_grouped_aggregation(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    // Group by value type and calculate statistics
    let (groups, grouping_time) = measure_execution(|| {
        group_by_type_parallel(data)
    });
    
    println!("   ⏱️  Grouping time: {:?}", grouping_time);
    println!("   📊 Groups created:");
    
    for (group_name, values) in &groups {
        println!("      {}: {} items", group_name, values.len());
        
        if group_name == "numbers" && !values.is_empty() {
            // Calculate statistics for numeric group
            let sum: f64 = values.par_iter()
                .filter_map(|v| match v {
                    Value::Number(n) => Some(*n),
                    _ => None,
                })
                .sum();
            
            let avg = sum / values.len() as f64;
            println!("         Average: {:.2}", avg);
        }
    }
    
    Ok(())
}

/// Compares performance of different aggregation methods
fn compare_aggregation_performance(data: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    let subset: Vec<Value> = data.iter().take(100_000).cloned().collect();
    
    println!("   📈 Performance comparison for {} elements:", subset.len());
    println!("   Operation    | Sequential | Parallel   | Speedup");
    println!("   -------------|------------|------------|--------");
    
    // Sum comparison
    let (_, seq_sum_time) = measure_execution(|| {
        subset.iter().fold(0.0, |acc, v| match v {
            Value::Number(n) => acc + n,
            _ => acc,
        })
    });
    
    let (_, par_sum_time) = measure_execution(|| {
        subset.par_iter().filter_map(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).sum::<f64>()
    });
    
    let sum_speedup = seq_sum_time.as_secs_f64() / par_sum_time.as_secs_f64();
    
    println!("   Sum          | {:>8.2}ms | {:>8.2}ms | {:>5.2}x",
             seq_sum_time.as_secs_f64() * 1000.0,
             par_sum_time.as_secs_f64() * 1000.0,
             sum_speedup);
    
    // Count comparison
    let (_, seq_count_time) = measure_execution(|| {
        count_value_types(&subset)
    });
    
    let (_, par_count_time) = measure_execution(|| {
        count_value_types_parallel(&subset)
    });
    
    let count_speedup = seq_count_time.as_secs_f64() / par_count_time.as_secs_f64();
    
    println!("   Count        | {:>8.2}ms | {:>8.2}ms | {:>5.2}x",
             seq_count_time.as_secs_f64() * 1000.0,
             par_count_time.as_secs_f64() * 1000.0,
             count_speedup);
    
    // Min/Max comparison
    let (_, seq_minmax_time) = measure_execution(|| {
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        for value in &subset {
            if let Value::Number(n) = value {
                min_val = min_val.min(*n);
                max_val = max_val.max(*n);
            }
        }
        (min_val, max_val)
    });
    
    let (_, par_minmax_time) = measure_execution(|| {
        let numeric: Vec<f64> = subset.par_iter().filter_map(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).collect();
        
        let min_val = numeric.par_iter().cloned().reduce(|| f64::INFINITY, f64::min);
        let max_val = numeric.par_iter().cloned().reduce(|| f64::NEG_INFINITY, f64::max);
        (min_val, max_val)
    });
    
    let minmax_speedup = seq_minmax_time.as_secs_f64() / par_minmax_time.as_secs_f64();
    
    println!("   Min/Max      | {:>8.2}ms | {:>8.2}ms | {:>5.2}x",
             seq_minmax_time.as_secs_f64() * 1000.0,
             par_minmax_time.as_secs_f64() * 1000.0,
             minmax_speedup);
    
    Ok(())
}

// Helper functions

#[derive(Debug, PartialEq)]
struct ValueTypeCounts {
    numbers: usize,
    strings: usize,
    booleans: usize,
}

impl ValueTypeCounts {
    fn total(&self) -> usize {
        self.numbers + self.strings + self.booleans
    }
}

fn count_value_types(data: &[Value]) -> ValueTypeCounts {
    let mut counts = ValueTypeCounts {
        numbers: 0,
        strings: 0,
        booleans: 0,
    };
    
    for value in data {
        match value {
            Value::Number(_) => counts.numbers += 1,
            Value::String(_) => counts.strings += 1,
            Value::Boolean(_) => counts.booleans += 1,
            Value::Array(_) | Value::List(_) | Value::SortedMap(_) => {
                // Composite values are not allowed at this phase
                // This is enforced by Gate C determinism guarantees
            }
        }
    }
    
    counts
}

fn count_value_types_parallel(data: &[Value]) -> ValueTypeCounts {
    use rayon::prelude::*;
    
    let counts: Vec<ValueTypeCounts> = data
        .par_chunks(1000)
        .map(|chunk| count_value_types(chunk))
        .collect();
    
    counts.into_iter().fold(
        ValueTypeCounts { numbers: 0, strings: 0, booleans: 0 },
        |mut acc, count| {
            acc.numbers += count.numbers;
            acc.strings += count.strings;
            acc.booleans += count.booleans;
            acc
        }
    )
}

fn calculate_variance_sequential(data: &[Value]) -> f64 {
    let numeric_values: Vec<f64> = data
        .iter()
        .filter_map(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })
        .collect();
    
    if numeric_values.is_empty() {
        return 0.0;
    }
    
    let mean = numeric_values.iter().sum::<f64>() / numeric_values.len() as f64;
    let variance = numeric_values
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / numeric_values.len() as f64;
    
    variance
}

fn calculate_variance_parallel(data: &[Value]) -> f64 {
    let numeric_values: Vec<f64> = data
        .par_iter()
        .filter_map(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })
        .collect();
    
    if numeric_values.is_empty() {
        return 0.0;
    }
    
    let mean = numeric_values.par_iter().sum::<f64>() / numeric_values.len() as f64;
    let variance = numeric_values
        .par_iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / numeric_values.len() as f64;
    
    variance
}

fn group_by_type_parallel(data: &[Value]) -> std::collections::HashMap<String, Vec<Value>> {
    use rayon::prelude::*;
    use std::collections::HashMap;
    
    let chunks: Vec<HashMap<String, Vec<Value>>> = data
        .par_chunks(10000)
        .map(|chunk| {
            let mut local_groups: HashMap<String, Vec<Value>> = HashMap::new();
            
            for value in chunk {
                let group_name = match value {
                    Value::Number(_) => "numbers",
                    Value::String(_) => "strings",
                    Value::Boolean(_) => "booleans",
                    Value::Array(_) | Value::List(_) | Value::SortedMap(_) => "composite",
                };
                
                local_groups
                    .entry(group_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(value.clone());
            }
            
            local_groups
        })
        .collect();
    
    // Merge chunks
    let mut final_groups: HashMap<String, Vec<Value>> = HashMap::new();
    for chunk_groups in chunks {
        for (group_name, mut values) in chunk_groups {
            final_groups
                .entry(group_name)
                .or_insert_with(Vec::new)
                .append(&mut values);
        }
    }
    
    final_groups
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sum_aggregation() {
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::String("ignore".to_string()),
        ];
        
        let handler = DefaultReductionHandler::new();
        let numeric_values: Vec<Value> = data
            .iter()
            .filter_map(|v| match v {
                Value::Number(_) => Some(v.clone()),
                _ => None,
            })
            .collect();
        
        let result = operations::sum(&handler, numeric_values).unwrap();
        
        assert_eq!(result, Value::Number(6.0));
    }
    
    #[test]
    fn test_count_aggregation() {
        let data = vec![
            Value::Number(1.0),
            Value::String("test".to_string()),
            Value::Boolean(true),
            Value::Number(2.0),
        ];
        
        let counts = count_value_types(&data);
        
        assert_eq!(counts.numbers, 2);
        assert_eq!(counts.strings, 1);
        assert_eq!(counts.booleans, 1);
        assert_eq!(counts.total(), 4);
    }
    
    #[test]
    fn test_min_max_aggregation() {
        let data = vec![
            Value::Number(5.0),
            Value::Number(1.0),
            Value::Number(9.0),
            Value::Number(3.0),
        ];
        
        let handler = DefaultReductionHandler::new();
        
        let min_result = operations::min(&handler, data.clone()).unwrap();
        let max_result = operations::max(&handler, data).unwrap();
        
        assert_eq!(min_result, Value::Number(1.0));
        assert_eq!(max_result, Value::Number(9.0));
    }
    
    #[test]
    fn test_variance_calculation() {
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        
        let variance = calculate_variance_sequential(&data);
        
        // Variance of [1,2,3,4,5] should be 2.0
        assert!((variance - 2.0).abs() < 1e-10);
    }
}