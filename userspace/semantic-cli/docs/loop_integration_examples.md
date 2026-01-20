# Loop Integration Examples

**Version:** Phase 3.5 - BCIB Architecture  
**Last Updated:** January 2026

## Real-World Integration Patterns

This document provides practical examples of D3 Loop Support integration with D1 JIT compilation and D2 parallelism systems.

## Example 1: Data Processing Pipeline

### Scenario
Process a large dataset with mathematical transformations, leveraging JIT compilation for hot loops.

```rust
use semantic_cli::bcib::*;
use semantic_cli::loop_engine::LoopEngine;
use semantic_cli::types::SourceLocation;

fn process_large_dataset() -> Result<Value, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    
    // Large dataset - will trigger JIT compilation
    let dataset = Value::Array((0..5000).map(|i| Value::Number(i as f64)).collect());
    
    let processing_loop = LoopInstruction::ForEach {
        id: LoopID::new("data-processing".to_string()),
        collection: OperandRef::Literal(dataset),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "result = transform(item)".to_string(),
        config: {
            let mut config = LoopConfig::new(Value::Array(vec![]), ValueType::Array);
            config.iteration_limit = 10_000;
            config.budget_timeout = 50_000;
            config.budget_measurement = BudgetMeasurement::IterationCount;
            config.error_recovery = ErrorRecoveryPolicy::ReturnPartialResults {
                include_error_info: true,
            };
            config
        },
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Execute with mathematical transformation
    let result = engine.execute_loop(&processing_loop, Box::new(|accumulator, iteration| {
        let mut results = match accumulator {
            Value::Array(arr) => arr.clone(),
            _ => vec![],
        };
        
        // Complex mathematical transformation (benefits from JIT)
        let transformed = Value::Number(
            (iteration as f64).sqrt() * 2.0 + (iteration as f64).sin()
        );
        
        results.push(transformed);
        Ok(LoopBodyResult::Continue(Value::Array(results)))
    }))?;
    
    // Check if JIT compilation occurred
    let loop_id = LoopID::new("data-processing".to_string());
    if engine.is_hot_loop(&loop_id) {
        println!("✅ JIT compilation activated for performance boost");
        
        let hot_info = engine.get_hot_loop_info(&loop_id).unwrap();
        println!("JIT compilation time: {:?}", hot_info.compilation_time);
    }
    
    Ok(result)
}
```

## Example 2: Parallel Aggregation

### Scenario
Aggregate data across multiple dimensions using D2 parallelism for safe operations.

```rust
fn parallel_aggregation() -> Result<Value, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    
    // Create aggregation loop - safe for parallelization
    let aggregation_loop = LoopInstruction::For {
        id: LoopID::new("parallel-sum".to_string()),
        range: LoopRange::new(0, 1000, 1),
        iterator_var: "i".to_string(),
        body: "sum += compute(i)".to_string(),
        config: LoopConfig {
            iteration_limit: 2000,
            budget_timeout: 20_000,
            budget_measurement: BudgetMeasurement::IterationCount,
            initial_accumulator: Value::Number(0.0),
            accumulator_type: ValueType::Number,
            error_recovery: ErrorRecoveryPolicy::Abort,
        },
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Analyze safety for parallelization
    let context = LoopAnalysisContext::new();
    let safety_result = engine.analyze_loop_safety("sum += compute(i)", &context)?;
    
    println!("Safety classification: {:?}", safety_result.classification);
    
    // Execute - will be parallelized if safe
    let result = engine.execute_loop(&aggregation_loop, Box::new(|accumulator, iteration| {
        let current_sum = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // Pure computation - no side effects (safe for parallelization)
        let computed_value = (iteration as f64).powi(2) + (iteration as f64).sqrt();
        let new_sum = current_sum + computed_value;
        
        Ok(LoopBodyResult::Continue(Value::Number(new_sum)))
    }))?;
    
    // Check parallelization status
    let stats = engine.get_loop_stats(&LoopID::new("parallel-sum".to_string())).unwrap();
    if stats.was_parallelized {
        println!("✅ Loop executed in parallel using D2 system");
        println!("Parallel efficiency: {:.2}%", stats.parallel_efficiency * 100.0);
    }
    
    Ok(result)
}
```

## Example 3: Streaming Data Processing

### Scenario
Process streaming data with bounded memory usage and error recovery.

```rust
fn streaming_processor() -> Result<Vec<Value>, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    let mut results = Vec::new();
    
    // Process data in chunks to manage memory
    const CHUNK_SIZE: usize = 100;
    let total_items = 10_000;
    
    for chunk_start in (0..total_items).step_by(CHUNK_SIZE) {
        let chunk_end = std::cmp::min(chunk_start + CHUNK_SIZE, total_items);
        
        let chunk_loop = LoopInstruction::For {
            id: LoopID::new(format!("chunk-{}", chunk_start)),
            range: LoopRange::new(chunk_start as i64, chunk_end as i64, 1),
            iterator_var: "item_idx".to_string(),
            body: "process_streaming_item(item_idx)".to_string(),
            config: LoopConfig {
                iteration_limit: CHUNK_SIZE as u32,
                budget_timeout: 5_000,
                budget_measurement: BudgetMeasurement::InstructionCount { weight: 10 },
                initial_accumulator: Value::Array(vec![]),
                accumulator_type: ValueType::Array,
                error_recovery: ErrorRecoveryPolicy::ReturnPartialResults {
                    include_error_info: true,
                },
            },
            location: SourceLocation::new(1, 1, 0),
        };
        
        match engine.execute_loop(&chunk_loop, Box::new(|accumulator, iteration| {
            let mut chunk_results = match accumulator {
                Value::Array(arr) => arr.clone(),
                _ => vec![],
            };
            
            // Simulate streaming data processing
            let processed_item = Value::String(format!("processed_{}", iteration));
            chunk_results.push(processed_item);
            
            Ok(LoopBodyResult::Continue(Value::Array(chunk_results)))
        })) {
            Ok(chunk_result) => {
                if let Value::Array(chunk_data) = chunk_result {
                    results.extend(chunk_data);
                }
            }
            Err(error) => {
                println!("Chunk {} failed: {}", chunk_start, error);
                // Continue with next chunk (resilient processing)
            }
        }
    }
    
    Ok(results)
}
```

## Example 4: Multi-Level Loop Optimization

### Scenario
Nested loop pattern with different optimization strategies per level.

```rust
fn multi_level_processing() -> Result<Value, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    
    // Outer loop: Small, will be unrolled
    let outer_loop = LoopInstruction::For {
        id: LoopID::new("outer-matrix".to_string()),
        range: LoopRange::new(0, 5, 1), // Small loop - automatic unrolling
        iterator_var: "row".to_string(),
        body: "process_row(row)".to_string(),
        config: LoopConfig::new(Value::Array(vec![]), ValueType::Array),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let result = engine.execute_loop(&outer_loop, Box::new(|accumulator, row| {
        let mut matrix_results = match accumulator {
            Value::Array(arr) => arr.clone(),
            _ => vec![],
        };
        
        // Inner loop: Large, will be JIT compiled and potentially parallelized
        let inner_loop = LoopInstruction::For {
            id: LoopID::new(format!("inner-row-{}", row)),
            range: LoopRange::new(0, 2000, 1), // Large loop - JIT + parallel
            iterator_var: "col".to_string(),
            body: "compute_cell(row, col)".to_string(),
            config: LoopConfig {
                iteration_limit: 3000,
                budget_timeout: 30_000,
                budget_measurement: BudgetMeasurement::IterationCount,
                initial_accumulator: Value::Array(vec![]),
                accumulator_type: ValueType::Array,
                error_recovery: ErrorRecoveryPolicy::Abort,
            },
            location: SourceLocation::new(2, 1, 0),
        };
        
        // Execute inner loop
        let mut inner_engine = LoopEngine::new(); // Separate engine for inner loop
        let row_result = inner_engine.execute_loop(&inner_loop, Box::new(move |acc, col| {
            let mut row_data = match acc {
                Value::Array(arr) => arr.clone(),
                _ => vec![],
            };
            
            // Matrix computation - pure math (safe for parallelization)
            let cell_value = Value::Number((row as f64) * (col as f64) + (col as f64).sin());
            row_data.push(cell_value);
            
            Ok(LoopBodyResult::Continue(Value::Array(row_data)))
        }))?;
        
        matrix_results.push(row_result);
        Ok(LoopBodyResult::Continue(Value::Array(matrix_results)))
    }))?;
    
    // Report optimization status
    let outer_stats = engine.get_loop_stats(&LoopID::new("outer-matrix".to_string()));
    if let Some(stats) = outer_stats {
        println!("Outer loop unrolled: {}", stats.was_unrolled);
    }
    
    Ok(result)
}
```

## Example 5: Error Recovery Patterns

### Scenario
Robust loop execution with comprehensive error handling and recovery.

```rust
fn robust_processing_with_recovery() -> Result<Value, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    
    // Configure loop with multiple recovery strategies
    let robust_loop = LoopInstruction::ForEach {
        id: LoopID::new("robust-processor".to_string()),
        collection: OperandRef::Literal(Value::Array(
            (0..1000).map(|i| Value::Number(i as f64)).collect()
        )),
        collection_type: CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "risky_operation(item)".to_string(),
        config: LoopConfig {
            iteration_limit: 1500,
            budget_timeout: 15_000,
            budget_measurement: BudgetMeasurement::IterationCount,
            initial_accumulator: Value::Array(vec![]),
            accumulator_type: ValueType::Array,
            // Try retry first, then partial results
            error_recovery: ErrorRecoveryPolicy::RetryWithIncreasedLimit {
                new_limit: 2000,
                max_retries: 2,
            },
        },
        location: SourceLocation::new(1, 1, 0),
    };
    
    let mut attempt = 0;
    loop {
        attempt += 1;
        println!("Processing attempt {}", attempt);
        
        match engine.execute_loop(&robust_loop, Box::new(|accumulator, iteration| {
            let mut results = match accumulator {
                Value::Array(arr) => arr.clone(),
                _ => vec![],
            };
            
            // Simulate risky operation that might fail
            if iteration % 100 == 99 && attempt == 1 {
                // Simulate failure on first attempt
                return Err(SemanticCLIError::execution_error(
                    "Simulated processing error",
                    "Retry with different parameters",
                ));
            }
            
            let processed = Value::Number((iteration as f64) * 1.5);
            results.push(processed);
            
            Ok(LoopBodyResult::Continue(Value::Array(results)))
        })) {
            Ok(result) => {
                println!("✅ Processing completed successfully on attempt {}", attempt);
                return Ok(result);
            }
            Err(error) => {
                println!("❌ Attempt {} failed: {}", attempt, error);
                
                if attempt >= 3 {
                    // Final attempt with partial results
                    let partial_config = LoopConfig {
                        iteration_limit: 1500,
                        budget_timeout: 15_000,
                        budget_measurement: BudgetMeasurement::IterationCount,
                        initial_accumulator: Value::Array(vec![]),
                        accumulator_type: ValueType::Array,
                        error_recovery: ErrorRecoveryPolicy::ReturnPartialResults {
                            include_error_info: true,
                        },
                    };
                    
                    let partial_loop = LoopInstruction::ForEach {
                        id: LoopID::new("partial-processor".to_string()),
                        collection: OperandRef::Literal(Value::Array(
                            (0..500).map(|i| Value::Number(i as f64)).collect() // Smaller dataset
                        )),
                        collection_type: CollectionType::Array,
                        iterator_var: "item".to_string(),
                        body: "safe_operation(item)".to_string(),
                        config: partial_config,
                        location: SourceLocation::new(1, 1, 0),
                    };
                    
                    match engine.execute_loop(&partial_loop, Box::new(|acc, iter| {
                        let mut results = match acc {
                            Value::Array(arr) => arr.clone(),
                            _ => vec![],
                        };
                        results.push(Value::Number(iter as f64));
                        Ok(LoopBodyResult::Continue(Value::Array(results)))
                    })) {
                        Ok(partial_result) => {
                            println!("⚠️ Returning partial results");
                            return Ok(partial_result);
                        }
                        Err(final_error) => {
                            return Err(final_error);
                        }
                    }
                }
                
                // Continue retry loop
            }
        }
    }
}
```

## Example 6: Performance Monitoring Integration

### Scenario
Comprehensive performance monitoring and optimization feedback loop.

```rust
fn monitored_processing() -> Result<Value, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    
    // Enable detailed monitoring
    let monitoring_config = MonitoringConfig {
        enable_detailed_logging: true,
        track_memory_usage: true,
        performance_alerts: true,
    };
    
    let monitored_loop = LoopInstruction::For {
        id: LoopID::new("monitored-computation".to_string()),
        range: LoopRange::new(0, 5000, 1),
        iterator_var: "i".to_string(),
        body: "complex_computation(i)".to_string(),
        config: LoopConfig {
            iteration_limit: 6000,
            budget_timeout: 60_000,
            budget_measurement: BudgetMeasurement::InstructionCount { weight: 5 },
            initial_accumulator: Value::Number(0.0),
            accumulator_type: ValueType::Number,
            error_recovery: ErrorRecoveryPolicy::Abort,
        },
        location: SourceLocation::new(1, 1, 0),
    };
    
    let start_time = std::time::Instant::now();
    
    let result = engine.execute_loop(&monitored_loop, Box::new(|accumulator, iteration| {
        let current = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // Complex computation that benefits from JIT
        let computed = (iteration as f64).powf(1.5) + (iteration as f64).ln_1p();
        let new_value = current + computed;
        
        Ok(LoopBodyResult::Continue(Value::Number(new_value)))
    }))?;
    
    let execution_time = start_time.elapsed();
    
    // Comprehensive performance analysis
    let loop_id = LoopID::new("monitored-computation".to_string());
    let stats = engine.get_loop_stats(&loop_id).unwrap();
    
    println!("=== Performance Report ===");
    println!("Total execution time: {:?}", execution_time);
    println!("Loop executions: {}", stats.execution_count);
    println!("Total iterations: {}", stats.total_iterations);
    println!("Average iteration time: {:?}", stats.average_iteration_time);
    println!("JIT compiled: {}", stats.was_jit_compiled);
    println!("Parallelized: {}", stats.was_parallelized);
    
    if stats.was_jit_compiled {
        let hot_info = engine.get_hot_loop_info(&loop_id).unwrap();
        println!("JIT compilation time: {:?}", hot_info.compilation_time);
        println!("Performance improvement: {:.2}x", hot_info.speedup_factor);
    }
    
    // Cache efficiency analysis
    let cache_stats = engine.get_safety_cache_stats();
    println!("Safety cache hit rate: {:.2}%", cache_stats.hit_rate * 100.0);
    
    let jit_stats = engine.get_jit_stats();
    println!("JIT cache entries: {}", jit_stats.cache_entries);
    println!("JIT compilation success rate: {:.2}%", jit_stats.success_rate * 100.0);
    
    // Performance recommendations
    if stats.average_iteration_time > std::time::Duration::from_micros(100) {
        println!("⚠️ RECOMMENDATION: Consider optimizing loop body");
    }
    
    if !stats.was_parallelized && stats.total_iterations > 1000 {
        println!("⚠️ RECOMMENDATION: Check if loop can be made safe for parallelization");
    }
    
    if cache_stats.hit_rate < 0.8 {
        println!("⚠️ RECOMMENDATION: Consider stabilizing loop body strings for better caching");
    }
    
    Ok(result)
}
```

## Integration Best Practices

### 1. JIT Compilation Optimization

```rust
// Design loops for JIT success
fn jit_friendly_loop() {
    // ✅ Good: Simple arithmetic, predictable patterns
    let body = Box::new(|acc, iter| {
        let current = extract_number(acc);
        let result = current + (iter as f64) * 2.0; // Simple math
        Ok(LoopBodyResult::Continue(Value::Number(result)))
    });
    
    // ❌ Avoid: Complex control flow, external calls
    let bad_body = Box::new(|acc, iter| {
        if iter % 7 == 0 {
            if iter % 14 == 0 {
                external_api_call(iter)?; // JIT can't optimize this
            }
        }
        // Complex nested conditions hurt JIT performance
        Ok(LoopBodyResult::Continue(acc.clone()))
    });
}
```

### 2. Parallelization Design

```rust
// Design for safe parallelization
fn parallel_safe_patterns() {
    // ✅ Pure computation - safe for parallelization
    let safe_body = Box::new(|acc, iter| {
        let sum = extract_number(acc);
        let computed = (iter as f64).sqrt() + (iter as f64).sin();
        Ok(LoopBodyResult::Continue(Value::Number(sum + computed)))
    });
    
    // ❌ Side effects - cannot be parallelized
    let unsafe_body = Box::new(|acc, iter| {
        println!("Processing {}", iter); // I/O side effect
        global_counter += 1; // External mutation
        Ok(LoopBodyResult::Continue(acc.clone()))
    });
}
```

### 3. Memory Management

```rust
// Efficient memory patterns
fn memory_efficient_loops() {
    // ✅ Streaming pattern for large datasets
    let streaming_body = Box::new(|_acc, iter| {
        // Process one item at a time, don't accumulate everything
        let result = process_item(iter);
        send_to_output_stream(result);
        Ok(LoopBodyResult::Continue(Value::Number(0.0))) // Minimal accumulator
    });
    
    // ❌ Memory-intensive accumulation
    let memory_heavy_body = Box::new(|acc, iter| {
        let mut all_results = extract_array(acc);
        all_results.push(create_large_object(iter)); // Memory grows unbounded
        Ok(LoopBodyResult::Continue(Value::Array(all_results)))
    });
}
```

### 4. Error Handling Integration

```rust
// Robust error handling with monitoring
fn error_aware_processing() -> Result<Value, SemanticCLIError> {
    let mut engine = LoopEngine::new();
    
    let config = LoopConfig {
        // Conservative limits for robust operation
        iteration_limit: 1000,
        budget_timeout: 10_000,
        budget_measurement: BudgetMeasurement::IterationCount,
        initial_accumulator: Value::Number(0.0),
        accumulator_type: ValueType::Number,
        // Graceful degradation strategy
        error_recovery: ErrorRecoveryPolicy::ReturnPartialResults {
            include_error_info: true,
        },
    };
    
    // Monitor error patterns
    let result = engine.execute_loop(&loop_instruction, body);
    
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            // Log error patterns for analysis
            log_error_pattern(&error);
            
            // Implement fallback strategy
            execute_fallback_processing()
        }
    }
}
```

These examples demonstrate real-world integration patterns that leverage the full capabilities of the D3 Loop Support system while maintaining constitutional compliance and optimal performance.