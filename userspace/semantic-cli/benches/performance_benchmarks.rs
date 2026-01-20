//! D3 Loop Execution Performance Benchmarks
//!
//! Task 13.1: Benchmark loop execution performance
//!
//! This module provides D3-compliant performance benchmarks that respect architectural boundaries:
//! - Measures sequential loop execution overhead
//! - Compares relative performance between loop types
//! - Observes JIT compilation effects (without forcing thresholds)
//! - Validates execution time characteristics
//!
//! ARCHITECTURAL COMPLIANCE:
//! - No LoopBodyFn cloning (constitutional violation)
//! - No forced parallelization parameters
//! - No hardcoded JIT thresholds
//! - No manual memory tracking
//! - No performance requirement assertions in benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use semantic_cli::bcib::{LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType, OperandRef};
use semantic_cli::loop_engine::{LoopEngine, LoopBodyFn, LoopBodyResult};
use semantic_cli::types::SourceLocation;

// ===== Benchmark Configuration =====

/// Small loop sizes for overhead measurement
const SMALL_LOOP_SIZES: &[u32] = &[10, 50, 100];

/// Large loop sizes for scalability testing
const LARGE_LOOP_SIZES: &[u32] = &[5000, 10000];

// ===== Test Loop Creation Utilities =====

/// Create a simple For loop for benchmarking
fn create_benchmark_for_loop(iterations: u32, loop_id: &str) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new(format!("benchmark-for-{}", loop_id)),
        range: LoopRange::new(0, iterations as i64, 1),
        iterator_var: "i".to_string(),
        body: "benchmark-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Create a ForEach loop for benchmarking
fn create_benchmark_foreach_loop(collection_size: u32, loop_id: &str) -> LoopInstruction {
    // Create a literal array collection for deterministic benchmarking
    let collection_values: Vec<Value> = (0..collection_size)
        .map(|i| Value::Number(i as f64))
        .collect();
    
    LoopInstruction::ForEach {
        id: LoopID::new(format!("benchmark-foreach-{}", loop_id)),
        collection: OperandRef::Literal(Value::Array(collection_values)),
        collection_type: semantic_cli::bcib::CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "benchmark-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

// ===== Loop Body Functions (Factories - Constitutional Compliance) =====

/// Simple computational body function factory
/// Creates a new function each time - NO CLONING
fn simple_computation_body() -> LoopBodyFn {
    Box::new(|accumulator, iteration| {
        // Simulate simple computation work
        let current_value = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // Simple arithmetic to simulate work
        let result = current_value + iteration as f64;
        
        Ok(LoopBodyResult::Normal(Value::Number(result)))
    })
}

/// CPU-intensive body function factory
/// Creates a new function each time - NO CLONING
fn cpu_intensive_body() -> LoopBodyFn {
    Box::new(|accumulator, iteration| {
        // Simulate CPU-intensive work
        let current_value = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // More complex computation
        let mut result = current_value;
        for i in 0..50 { // Reduced from 100 to avoid excessive benchmark time
            result += (i as f64 + iteration as f64).sin().cos();
        }
        
        Ok(LoopBodyResult::Normal(Value::Number(result)))
    })
}

// ===== Task 13.1: Benchmark loop execution performance =====

/// Benchmark sequential loop execution overhead
fn benchmark_sequential_loop_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_loop_overhead");
    
    for &size in SMALL_LOOP_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("for_loop", size),
            &size,
            |b, &iterations| {
                let mut loop_engine = LoopEngine::new();
                let instruction = create_benchmark_for_loop(iterations, "overhead");
                
                b.iter(|| {
                    // Constitutional compliance: Create body function each iteration
                    let body_fn = simple_computation_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("foreach_loop", size),
            &size,
            |b, &iterations| {
                let mut loop_engine = LoopEngine::new();
                let instruction = create_benchmark_foreach_loop(iterations, "overhead");
                
                b.iter(|| {
                    // Constitutional compliance: Create body function each iteration
                    let body_fn = simple_computation_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark loop execution scalability
fn benchmark_loop_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop_scalability");
    
    for &size in LARGE_LOOP_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        
        // For loops with CPU-intensive work
        group.bench_with_input(
            BenchmarkId::new("for_cpu_intensive", size),
            &size,
            |b, &iterations| {
                let mut loop_engine = LoopEngine::new();
                let instruction = create_benchmark_for_loop(iterations, "scalability");
                
                b.iter(|| {
                    // Constitutional compliance: Create body function each iteration
                    let body_fn = cpu_intensive_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
        
        // ForEach loops with CPU-intensive work
        group.bench_with_input(
            BenchmarkId::new("foreach_cpu_intensive", size),
            &size,
            |b, &iterations| {
                let mut loop_engine = LoopEngine::new();
                let instruction = create_benchmark_foreach_loop(iterations, "scalability");
                
                b.iter(|| {
                    // Constitutional compliance: Create body function each iteration
                    let body_fn = cpu_intensive_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark JIT compilation effects (observational, not forced)
fn benchmark_jit_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_effects");
    
    // Test repeated execution to observe JIT effects
    // We don't force JIT thresholds - we just observe what happens
    
    group.bench_function("cold_execution", |b| {
        b.iter(|| {
            // Fresh engine each time - no JIT warmup
            let mut loop_engine = LoopEngine::new();
            let instruction = create_benchmark_for_loop(1500, "cold");
            let body_fn = cpu_intensive_body();
            
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.bench_function("warm_execution", |b| {
        // Shared engine - potential JIT warmup
        let mut loop_engine = LoopEngine::new();
        
        b.iter(|| {
            let instruction = create_benchmark_for_loop(1500, "warm");
            let body_fn = cpu_intensive_body();
            
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.bench_function("repeated_execution", |b| {
        b.iter_custom(|iters| {
            let mut loop_engine = LoopEngine::new();
            let instruction = create_benchmark_for_loop(1500, "repeated");
            
            let start = std::time::Instant::now();
            
            for _ in 0..iters {
                let body_fn = cpu_intensive_body();
                let result = loop_engine.execute_loop(&instruction, body_fn);
                black_box(result.unwrap());
            }
            
            start.elapsed()
        });
    });
    
    group.finish();
}

/// Benchmark loop type comparison
fn benchmark_loop_type_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop_type_comparison");
    
    let test_size = 1000u32;
    group.throughput(Throughput::Elements(test_size as u64));
    
    group.bench_function("for_loop_baseline", |b| {
        let mut loop_engine = LoopEngine::new();
        let instruction = create_benchmark_for_loop(test_size, "comparison");
        
        b.iter(|| {
            let body_fn = simple_computation_body();
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.bench_function("foreach_loop_baseline", |b| {
        let mut loop_engine = LoopEngine::new();
        let instruction = create_benchmark_foreach_loop(test_size, "comparison");
        
        b.iter(|| {
            let body_fn = simple_computation_body();
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.finish();
}

/// Benchmark execution engine overhead
fn benchmark_execution_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution_overhead");
    
    // Measure the overhead of the loop execution system itself
    // by comparing very small loops
    
    for &size in &[1u32, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("minimal_loop", size),
            &size,
            |b, &iterations| {
                let mut loop_engine = LoopEngine::new();
                let instruction = create_benchmark_for_loop(iterations, "minimal");
                
                b.iter(|| {
                    let body_fn = simple_computation_body();
                    let result = loop_engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallelization effects (observational only)
/// This measures whether the engine chooses to parallelize and the effect on performance
fn benchmark_parallelization_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallelization_effects");
    
    let test_size = 5000u32;
    group.throughput(Throughput::Elements(test_size as u64));
    
    // We don't force parallelization - we observe what the engine decides
    group.bench_function("engine_decision_baseline", |b| {
        let mut loop_engine = LoopEngine::new();
        let instruction = create_benchmark_for_loop(test_size, "parallel-candidate");
        
        b.iter(|| {
            let body_fn = cpu_intensive_body();
            let result = loop_engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    group.finish();
}

// ===== Benchmark Groups =====

criterion_group!(
    d3_execution_benchmarks,
    benchmark_sequential_loop_overhead,
    benchmark_loop_scalability,
    benchmark_jit_effects,
    benchmark_loop_type_comparison,
    benchmark_execution_overhead,
    benchmark_parallelization_effects
);

criterion_main!(d3_execution_benchmarks);
