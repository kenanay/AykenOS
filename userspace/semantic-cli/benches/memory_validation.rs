//! D3 Loop Memory Usage Validation
//!
//! Task 13.2: Validate memory usage
//!
//! NOTE:
//! These benchmarks observe relative allocation behavior.
//! They do NOT prove absence of memory leaks.
//! They are intended to detect memory behavior regressions.
//!
//! This module provides memory usage validation for the D3 Loop Support Design using
//! simple memory pattern observation. It focuses on relative memory usage patterns
//! without complex OS-level measurements.
//!
//! ## CRITICAL LIMITATION NOTICE
//! 
//! **This benchmark does NOT prove absence of memory leaks.**
//! **It detects abnormal allocation behavior regressions.**
//! 
//! What this benchmark CAN detect:
//! - Sudden increases in execution time (indicating memory pressure)
//! - Performance regression patterns across loop sizes
//! - Relative memory behavior changes between releases
//! 
//! What this benchmark CANNOT detect:
//! - Slow memory leaks that don't affect performance immediately
//! - Memory leaks smaller than system noise threshold
//! - Cross-session memory accumulation
//! 
//! For comprehensive memory leak detection, use dedicated tools:
//! - Valgrind (Linux)
//! - AddressSanitizer 
//! - OS-specific memory profilers
//!
//! ARCHITECTURAL COMPLIANCE:
//! - No global atomic counters
//! - No fake memory tracking
//! - Focuses on relative memory usage patterns
//! - Simple memory pattern observation

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use semantic_cli::bcib::{LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType, OperandRef};
use semantic_cli::loop_engine::{LoopEngine, LoopBodyFn, LoopBodyResult};
use semantic_cli::types::SourceLocation;

// ===== Memory Validation Configuration =====

/// Loop sizes for memory validation
const MEMORY_TEST_SIZES: &[u32] = &[100, 500, 1000, 5000];

// ===== Test Loop Creation =====

/// Create a For loop for memory testing
fn create_memory_test_for_loop(iterations: u32, loop_id: &str) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new(format!("memory-test-for-{}", loop_id)),
        range: LoopRange::new(0, iterations as i64, 1),
        iterator_var: "i".to_string(),
        body: "memory-test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Create a ForEach loop for memory testing
fn create_memory_test_foreach_loop(collection_size: u32, loop_id: &str) -> LoopInstruction {
    let collection_values: Vec<Value> = (0..collection_size)
        .map(|i| Value::Number(i as f64))
        .collect();
    
    LoopInstruction::ForEach {
        id: LoopID::new(format!("memory-test-foreach-{}", loop_id)),
        collection: OperandRef::Literal(Value::Array(collection_values)),
        collection_type: semantic_cli::bcib::CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "memory-test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

// ===== Loop Body Functions =====

/// Memory-light body function factory for baseline measurement
/// Constitutional compliance: Creates new function each time - NO CLONING
fn memory_light_body() -> LoopBodyFn {
    Box::new(|accumulator, iteration| {
        let current_value = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // Minimal computation, minimal allocation
        let result = current_value + iteration as f64;
        
        Ok(LoopBodyResult::Normal(Value::Number(result)))
    })
}

/// Memory-heavy body function factory for memory usage testing
/// Constitutional compliance: Creates new function each time - NO CLONING
fn memory_heavy_body() -> LoopBodyFn {
    Box::new(|accumulator, iteration| {
        let current_value = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // Allocate temporary memory that should be cleaned up
        let temp_data: Vec<f64> = (0..1000).map(|i| i as f64 + iteration as f64).collect();
        let sum: f64 = temp_data.iter().sum();
        
        let result = current_value + sum / 1000.0;
        
        Ok(LoopBodyResult::Normal(Value::Number(result)))
    })
}

/// String-heavy body function factory for testing string allocation patterns
/// Constitutional compliance: Creates new function each time - NO CLONING
fn string_heavy_body() -> LoopBodyFn {
    Box::new(|accumulator, iteration| {
        let current_value = match accumulator {
            Value::String(s) => s.clone(),
            _ => String::new(),
        };
        
        // Create and manipulate strings
        let temp_string = format!("iteration_{}_data_{}", iteration, current_value);
        let processed = temp_string.repeat(10);
        let result = processed.chars().take(100).collect::<String>();
        
        Ok(LoopBodyResult::Normal(Value::String(result)))
    })
}

// ===== Memory Validation Tests =====

/// Validate loop state memory overhead through execution time patterns
/// 
/// NOTE: Uses warm-up to establish JIT steady-state before measurement
fn validate_loop_state_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_validation_loop_state");
    
    for &size in MEMORY_TEST_SIZES {
        group.bench_with_input(
            BenchmarkId::new("for_loop_memory_light", size),
            &size,
            |b, &iterations| {
                // ✅ FIXED: Engine outside iteration to preserve JIT cache
                let mut loop_engine = LoopEngine::new();
                let instruction = create_memory_test_for_loop(iterations, "light");
                
                // Warm-up: Trigger JIT compilation once
                let warmup_body = memory_light_body();
                let _ = loop_engine.execute_loop(&instruction, warmup_body);
                
                b.iter(|| {
                    // Constitutional compliance: Create body function each iteration
                    let body_fn = memory_light_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("for_loop_memory_heavy", size),
            &size,
            |b, &iterations| {
                // ✅ FIXED: Engine outside iteration to preserve JIT cache
                let mut loop_engine = LoopEngine::new();
                let instruction = create_memory_test_for_loop(iterations, "heavy");
                
                // Warm-up: Trigger JIT compilation once
                let warmup_body = memory_heavy_body();
                let _ = loop_engine.execute_loop(&instruction, warmup_body);
                
                b.iter(|| {
                    // Constitutional compliance: Create body function each iteration
                    let body_fn = memory_heavy_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Validate memory cleanup through repeated execution patterns
/// 
/// NOTE: No warm-up used here - testing drop behavior and cleanup patterns
/// across fresh engine instances (intentional cold-start behavior)
fn validate_memory_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_validation_cleanup");
    
    group.bench_function("memory_cleanup_validation", |b| {
        b.iter(|| {
            // Execute multiple loops to test cleanup patterns
            for i in 0..5 {
                let mut loop_engine = LoopEngine::new();
                let instruction = create_memory_test_for_loop(1000, &format!("cleanup-{}", i));
                
                // Constitutional compliance: Create body function each iteration
                let body_fn = memory_heavy_body();
                let result = loop_engine.execute_loop(&instruction, body_fn);
                black_box(result.unwrap());
                
                // Explicitly drop to test cleanup
                drop(loop_engine);
            }
        });
    });
    
    group.finish();
}

/// Validate long-running memory stability
/// 
/// NOTE: Uses warm-up to establish JIT steady-state before measurement
fn validate_long_running_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_validation_long_running");
    
    group.bench_function("long_running_memory_stability", |b| {
        // ✅ FIXED: Engine outside iteration to preserve JIT cache
        let mut loop_engine = LoopEngine::new();
        let instruction = create_memory_test_for_loop(10000, "long-running");
        
        // Warm-up: Trigger JIT compilation once
        let warmup_body = memory_heavy_body();
        let _ = loop_engine.execute_loop(&instruction, warmup_body);
        
        b.iter(|| {
            // Constitutional compliance: Create body function each iteration
            let body_fn = memory_heavy_body();
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.finish();
}

/// Validate memory usage patterns across different loop types
/// 
/// NOTE: Uses warm-up to establish JIT steady-state before measurement
fn validate_loop_type_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_validation_loop_types");
    
    let test_size = 1000u32;
    
    group.bench_function("for_loop_memory_pattern", |b| {
        // ✅ FIXED: Engine outside iteration to preserve JIT cache
        let mut loop_engine = LoopEngine::new();
        let instruction = create_memory_test_for_loop(test_size, "pattern");
        
        // Warm-up: Trigger JIT compilation once
        let warmup_body = string_heavy_body();
        let _ = loop_engine.execute_loop(&instruction, warmup_body);
        
        b.iter(|| {
            // Constitutional compliance: Create body function each iteration
            let body_fn = string_heavy_body();
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.bench_function("foreach_loop_memory_pattern", |b| {
        // ✅ FIXED: Engine outside iteration to preserve JIT cache
        let mut loop_engine = LoopEngine::new();
        let instruction = create_memory_test_foreach_loop(test_size, "pattern");
        
        // Warm-up: Trigger JIT compilation once
        let warmup_body = string_heavy_body();
        let _ = loop_engine.execute_loop(&instruction, warmup_body);
        
        b.iter(|| {
            // Constitutional compliance: Create body function each iteration
            let body_fn = string_heavy_body();
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.finish();
}

/// Validate memory scalability with increasing loop sizes
/// 
/// NOTE: This test runs with JIT enabled, measuring "loop + JIT runtime memory"
/// not pure loop memory. For Phase 3, consider separating:
/// - memory_validation_no_jit (pure loop memory)
/// - memory_validation_with_jit (loop + JIT memory)
fn validate_memory_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_validation_scalability");
    
    for &size in MEMORY_TEST_SIZES {
        group.bench_with_input(
            BenchmarkId::new("memory_scalability", size),
            &size,
            |b, &iterations| {
                // ✅ FIXED: Engine outside iteration to preserve JIT cache
                let mut loop_engine = LoopEngine::new();
                let instruction = create_memory_test_for_loop(iterations, "scalability");
                
                // Warm-up: Trigger JIT compilation once
                let warmup_body = memory_heavy_body();
                let _ = loop_engine.execute_loop(&instruction, warmup_body);
                
                b.iter(|| {
                    // ✅ CONSTITUTIONAL COMPLIANCE: Create body function each iteration - NO CLONING
                    // This measures steady-state execution with JIT enabled.
                    // Cache-key behavior (LoopID-only vs LoopID+body signature) is engine-defined.
                    // If cache key changes, update this benchmark's interpretation.
                    let body_fn = memory_heavy_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

// ===== Benchmark Groups =====

criterion_group!(
    memory_validation_benchmarks,
    validate_loop_state_memory_overhead,
    validate_memory_cleanup,
    validate_long_running_memory,
    validate_loop_type_memory_patterns,
    validate_memory_scalability
);

criterion_main!(memory_validation_benchmarks);