//! JIT Performance Benchmarks
//!
//! Constitutional Compliance: JIT Benchmark Separation Policy
//!
//! This module provides separate benchmarks for:
//! 1. JIT compilation latency (cold start)
//! 2. JIT runtime performance (warm execution)
//! 3. JIT cache effectiveness
//!
//! ARCHITECTURAL COMPLIANCE:
//! - JIT compilation and execution measured separately
//! - Engine lifecycle properly managed
//! - Cache behavior validated independently

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use semantic_cli::bcib::{LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType};
use semantic_cli::loop_engine::{LoopEngine, LoopBodyFn, LoopBodyResult};
use semantic_cli::types::SourceLocation;
use std::time::{Duration, Instant};

// ===== JIT Test Configuration =====

/// Loop sizes for JIT performance testing
const JIT_TEST_SIZES: &[u32] = &[100, 1000, 5000, 10000];

// ===== Test Loop Creation =====

/// Create a stable loop for JIT cache testing (same ID)
/// 
/// CRITICAL ASSUMPTION: JIT cache key is based on LoopID only.
/// If cache key changes to include body hash, bytecode fingerprint, 
/// or config flags, this benchmark must be updated.
/// 
/// Current cache key components (as of Phase 2.3):
/// - Loop ID (stable)
/// - Loop structure (stable) 
/// - Body function pointer (varies - handled by pre-creation)
fn create_stable_jit_loop(iterations: u32) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new("jit-performance-stable".to_string()),
        range: LoopRange::new(0, iterations as i64, 1),
        iterator_var: "i".to_string(),
        body: "jit-test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

/// Create a unique loop for JIT compilation testing (different ID each time)
fn create_unique_jit_loop(unique_id: u64, iterations: u32) -> LoopInstruction {
    LoopInstruction::For {
        id: LoopID::new(format!("jit-compile-unique-{}", unique_id)),
        range: LoopRange::new(0, iterations as i64, 1),
        iterator_var: "i".to_string(),
        body: "jit-test-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    }
}

// ===== Loop Body Functions =====

/// Computational body for JIT testing
fn jit_test_body() -> LoopBodyFn {
    Box::new(|accumulator, iteration| {
        let current_value = match accumulator {
            Value::Number(n) => *n,
            _ => 0.0,
        };
        
        // Moderate computation to trigger JIT
        let result = current_value + (iteration as f64).sqrt() * 1.5;
        
        Ok(LoopBodyResult::Normal(Value::Number(result)))
    })
}

// ===== JIT Compilation Benchmarks =====

/// Benchmark JIT compilation latency (cold start)
fn benchmark_jit_compilation_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_compilation_latency");
    
    for &size in JIT_TEST_SIZES {
        group.bench_with_input(
            BenchmarkId::new("jit_compile_cold_start", size),
            &size,
            |b, &iterations| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;
                    
                    for i in 0..iters {
                        // Fresh engine = empty JIT cache
                        let mut engine = LoopEngine::new();
                        let instruction = create_unique_jit_loop(i, iterations);
                        let body_fn = jit_test_body();
                        
                        let start = Instant::now();
                        let result = engine.execute_loop(&instruction, body_fn);
                        total_duration += start.elapsed();
                        
                        black_box(result.unwrap());
                    }
                    
                    total_duration
                });
            },
        );
    }
    
    group.finish();
}

// ===== JIT Runtime Performance Benchmarks =====

/// Benchmark JIT runtime performance (warm execution)
fn benchmark_jit_runtime_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_runtime_performance");
    
    for &size in JIT_TEST_SIZES {
        group.bench_with_input(
            BenchmarkId::new("jit_runtime_warm", size),
            &size,
            |b, &iterations| {
                // Single engine to preserve JIT cache
                let mut engine = LoopEngine::new();
                let instruction = create_stable_jit_loop(iterations);
                
                // Warm-up: Trigger JIT compilation
                let warmup_body = jit_test_body();
                let _ = engine.execute_loop(&instruction, warmup_body);
                
                // Measure only runtime performance
                b.iter(|| {
                    let body_fn = jit_test_body();
                    let result = engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

// ===== JIT Cache Effectiveness Benchmarks =====

/// Benchmark JIT cache hit vs miss performance
fn benchmark_jit_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_cache_effectiveness");
    
    let test_iterations = 1000u32;
    
    // Cache hit scenario
    group.bench_function("jit_cache_hit", |b| {
        let mut engine = LoopEngine::new();
        let instruction = create_stable_jit_loop(test_iterations);
        
        // Warm-up: Ensure JIT compilation
        let warmup_body = jit_test_body();
        let _ = engine.execute_loop(&instruction, warmup_body);
        
        b.iter(|| {
            // Same loop ID = cache hit
            let body_fn = jit_test_body();
            let result = engine.execute_loop(&instruction, body_fn);
            black_box(result.unwrap());
        });
    });
    
    // Cache miss scenario
    group.bench_function("jit_cache_miss", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;
            
            for i in 0..iters {
                let mut engine = LoopEngine::new();
                let instruction = create_unique_jit_loop(i, test_iterations);
                let body_fn = jit_test_body();
                
                let start = Instant::now();
                let result = engine.execute_loop(&instruction, body_fn);
                total_duration += start.elapsed();
                
                black_box(result.unwrap());
            }
            
            total_duration
        });
    });
    
    group.finish();
}

// ===== JIT Scalability Benchmarks =====

/// Benchmark JIT performance scaling with loop size
fn benchmark_jit_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_scalability");
    
    // Compare JIT vs non-JIT performance across sizes
    for &size in JIT_TEST_SIZES {
        // JIT enabled (warm)
        group.bench_with_input(
            BenchmarkId::new("jit_enabled_warm", size),
            &size,
            |b, &iterations| {
                let mut engine = LoopEngine::new();
                let instruction = create_stable_jit_loop(iterations);
                
                // Warm-up
                let warmup_body = jit_test_body();
                let _ = engine.execute_loop(&instruction, warmup_body);
                
                b.iter(|| {
                    let body_fn = jit_test_body();
                    let result = engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
        
        // JIT disabled (for comparison)
        group.bench_with_input(
            BenchmarkId::new("jit_disabled", size),
            &size,
            |b, &iterations| {
                let mut engine = LoopEngine::new();
                
                // Disable JIT for this engine
                let mut disabled_config = engine.get_jit_config().clone();
                disabled_config.enabled = false;
                engine.update_jit_config(disabled_config);
                
                let instruction = create_stable_jit_loop(iterations);
                
                b.iter(|| {
                    let body_fn = jit_test_body();
                    let result = engine.execute_loop(&instruction, body_fn);
                    black_box(result.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

// ===== Benchmark Groups =====

criterion_group!(
    jit_performance_benchmarks,
    benchmark_jit_compilation_latency,
    benchmark_jit_runtime_performance,
    benchmark_jit_cache_effectiveness,
    benchmark_jit_scalability
);

criterion_main!(jit_performance_benchmarks);