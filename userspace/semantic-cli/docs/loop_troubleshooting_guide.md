# Loop Troubleshooting Guide

**Version:** Phase 3.5 - BCIB Architecture  
**Last Updated:** January 2026

## Common Issues and Solutions

### 1. Compilation Errors

#### Error: "no variant named `Integer` found for enum `Value`"

**Problem:** Using old Value::Integer syntax.

```rust
// ❌ Wrong (pre-BCIB)
Value::Integer(42)
ValueType::Integer

// ✅ Correct (BCIB)
Value::Number(42.0)
ValueType::Number
```

**Solution:** Use `Value::Number(f64)` for all numeric values.

#### Error: "expected `Value`, found `String`"

**Problem:** Incorrect OperandRef usage.

```rust
// ❌ Wrong
OperandRef::Literal("condition".to_string())

// ✅ Correct
OperandRef::Literal(Value::String("condition".to_string()))
```

**Solution:** Wrap string literals in `Value::String()`.

#### Error: "closure is expected to take 2 arguments, but it takes 1"

**Problem:** Incorrect loop body function signature.

```rust
// ❌ Wrong (old API)
Box::new(|state| { ... })

// ✅ Correct (current API)
Box::new(|accumulator: &Value, iteration: u32| -> Result<LoopBodyResult> {
    // Process accumulator and iteration
    Ok(LoopBodyResult::Continue(new_accumulator))
})
```

### 2. Runtime Errors

#### Error: "Iteration limit exceeded"

**Symptoms:**
- Loop terminates before completion
- Error message contains "iteration limit"

**Causes:**
1. Loop requires more iterations than limit allows
2. Infinite loop condition
3. Incorrect iteration counting

**Solutions:**

```rust
// Option 1: Increase iteration limit (within constitutional bounds)
let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
config.iteration_limit = 5000; // Max: 10,000

// Option 2: Use partial results policy
config.error_recovery = ErrorRecoveryPolicy::ReturnPartialResults {
    include_error_info: true,
};

// Option 3: Use retry policy
config.error_recovery = ErrorRecoveryPolicy::RetryWithIncreasedLimit {
    new_limit: 2000,
    max_retries: 1,
};
```

#### Error: "Budget timeout exceeded"

**Symptoms:**
- Loop terminates due to timeout
- Error message contains "budget" or "timeout"

**Causes:**
1. Loop body is too expensive
2. Budget timeout too low
3. Inefficient budget measurement

**Solutions:**

```rust
// Option 1: Increase budget timeout
config.budget_timeout = 10000;

// Option 2: Use more efficient budget measurement
config.budget_measurement = BudgetMeasurement::IterationCount; // Fastest

// Option 3: Optimize loop body
let optimized_body = Box::new(|acc, iter| {
    // Minimize work per iteration
    let result = simple_calculation(acc, iter);
    Ok(LoopBodyResult::Continue(result))
});
```

#### Error: "Unordered collection requires canonical ordering"

**Symptoms:**
- ForEach loop validation fails
- Error mentions "deterministic" or "ordering"

**Cause:** Using HashMap or HashSet without explicit ordering.

**Solutions:**

```rust
// Option 1: Use ordered collection
CollectionType::Array  // or List, SortedMap

// Option 2: Provide canonical ordering
CollectionType::HashMap { 
    canonical_ordering: Some("key_sort".to_string()) 
}

// Option 3: Convert to ordered collection
let ordered_data = Value::Array(
    hashmap.into_iter()
           .collect::<Vec<_>>()
           .sort_by_key(|(k, _)| k)
           .into_iter()
           .map(|(_, v)| v)
           .collect()
);
```

### 3. Performance Issues

#### Issue: Loop execution is slow

**Symptoms:**
- High execution time
- Poor throughput

**Diagnostic Steps:**

```rust
// 1. Check if loop is being optimized
let stats = engine.get_loop_stats(&loop_id).unwrap();
println!("JIT compiled: {}", stats.was_jit_compiled);
println!("Parallelized: {}", stats.was_parallelized);

// 2. Check hot loop status
if engine.is_hot_loop(&loop_id) {
    let hot_info = engine.get_hot_loop_info(&loop_id).unwrap();
    println!("JIT status: {:?}", hot_info.jit_status);
}

// 3. Check safety analysis
let context = LoopAnalysisContext::new();
let safety = engine.analyze_loop_safety(&loop_body, &context).unwrap();
println!("Safety class: {:?}", safety.classification);
```

**Solutions:**

```rust
// 1. Ensure loop qualifies for JIT (> 1000 iterations)
config.iteration_limit = 2000; // Above JIT threshold

// 2. Make loop body safe for parallelization
// - Remove I/O operations
// - Avoid external mutations
// - Eliminate loop-carried dependencies

// 3. Use efficient accumulator patterns
let body = Box::new(|acc, iter| {
    // Avoid expensive operations
    // Use simple arithmetic
    // Minimize allocations
    Ok(LoopBodyResult::Continue(simple_update(acc, iter)))
});
```

#### Issue: Memory usage is high

**Symptoms:**
- Increasing memory consumption
- Out of memory errors

**Diagnostic Steps:**

```rust
// Check cache sizes
let cache_stats = engine.get_safety_cache_stats();
println!("Safety cache entries: {}", cache_stats.entries);

let jit_stats = engine.get_jit_stats();
println!("JIT cache size: {} KB", jit_stats.cache_size_kb);
```

**Solutions:**

```rust
// 1. Clear caches periodically
engine.clear_safety_cache();

// 2. Limit loop complexity
// - Avoid large accumulator values
// - Use streaming patterns for large data

// 3. Monitor and limit concurrent loops
let global_stats = engine.get_global_monitoring_stats();
if global_stats.active_loops > 100 {
    // Wait for completion or limit new loops
}
```

### 4. Integration Issues

#### Issue: Safety analysis cache misses

**Symptoms:**
- Low cache hit rate
- Repeated safety analysis for same code

**Diagnostic:**

```rust
let cache_stats = engine.get_safety_cache_stats();
println!("Hit rate: {:.2}%", cache_stats.hit_rate * 100.0);
```

**Solutions:**

```rust
// 1. Ensure consistent loop body strings
let body = "standardized_body_format"; // Same string = cache hit

// 2. Avoid dynamic body generation
// ❌ Wrong
let body = format!("process_{}", dynamic_var);

// ✅ Correct
let body = "process_item"; // Static string
```

#### Issue: JIT compilation failures

**Symptoms:**
- Hot loops not getting JIT compiled
- JIT compilation errors in logs

**Diagnostic:**

```rust
let hot_info = engine.get_hot_loop_info(&loop_id);
if let Some(info) = hot_info {
    match info.jit_status {
        JITCompilationStatus::Failed => {
            println!("JIT failed: {}", info.failure_reason);
        }
        _ => {}
    }
}
```

**Solutions:**

```rust
// 1. Simplify loop body for JIT compatibility
// - Avoid complex control flow
// - Use simple arithmetic operations
// - Minimize external function calls

// 2. Check JIT configuration
let jit_config = JITConfig {
    enable_optimization: true,
    max_compilation_time: Duration::from_secs(5),
    // ...
};
```

### 5. Debugging Techniques

#### Enable Detailed Logging

```rust
// Set environment variable
std::env::set_var("LOOP_ENGINE_LOG", "debug");

// Or use monitoring API
let monitoring_config = MonitoringConfig {
    enable_detailed_logging: true,
    log_level: LogLevel::Debug,
};
```

#### Inspect Loop State

```rust
// Get detailed execution stats
let stats = engine.get_loop_stats(&loop_id).unwrap();
println!("Execution count: {}", stats.execution_count);
println!("Total iterations: {}", stats.total_iterations);
println!("Average time: {:?}", stats.average_execution_time);
println!("Max iterations per execution: {}", stats.max_iterations_per_execution);

// Check for performance anomalies
if stats.average_execution_time > Duration::from_millis(100) {
    println!("WARNING: Loop execution is slow");
}
```

#### Validate Loop Configuration

```rust
// Check constitutional compliance
assert!(config.iteration_limit <= 10_000, "Exceeds constitutional limit");
assert!(config.budget_timeout > 0, "Budget timeout must be positive");

// Validate error recovery policy
match &config.error_recovery {
    ErrorRecoveryPolicy::RetryWithIncreasedLimit { new_limit, max_retries } => {
        assert!(*new_limit <= 10_000, "Retry limit exceeds constitutional maximum");
        assert!(*max_retries <= 3, "Max retries exceeds constitutional maximum");
    }
    _ => {}
}
```

### 6. Performance Optimization Checklist

#### Loop Design
- [ ] Use For loops for known ranges
- [ ] Use ForEach for deterministic collections
- [ ] Avoid While loops when possible (never parallelized)
- [ ] Keep loop body simple and side-effect free

#### Configuration
- [ ] Set appropriate iteration limits (not too low, not too high)
- [ ] Use efficient budget measurement (IterationCount for simple loops)
- [ ] Configure error recovery policies explicitly
- [ ] Enable partial results for long-running loops

#### Monitoring
- [ ] Check hot loop detection is working
- [ ] Monitor JIT compilation success rate
- [ ] Track safety analysis cache hit rate
- [ ] Watch for memory usage growth

#### Integration
- [ ] Ensure safety analysis cache is effective
- [ ] Verify parallelization is occurring for safe loops
- [ ] Check D1 JIT integration is working
- [ ] Monitor D2 parallelism performance

### 7. Error Code Reference

| Error Pattern | Meaning | Solution |
|---------------|---------|----------|
| `iteration limit` | Loop exceeded iteration limit | Increase limit or use partial results |
| `budget timeout` | Loop exceeded budget timeout | Increase budget or optimize body |
| `deterministic` | Collection ordering issue | Use ordered collection or provide canonical ordering |
| `type mismatch` | Accumulator type changed | Ensure consistent accumulator types |
| `validation` | BCIB instruction invalid | Fix instruction parameters |
| `compilation` | JIT compilation failed | Simplify loop body |
| `parallelization` | Parallel execution failed | Check safety analysis results |

### 8. Getting Help

For additional support:

1. **Check existing tests:** Look at `tests/loop_*_tests.rs` for working examples
2. **Review documentation:** See `docs/loop_usage_guide.md` for usage patterns
3. **Enable debug logging:** Set `LOOP_ENGINE_LOG=debug` for detailed output
4. **Use monitoring API:** Get runtime statistics and performance data
5. **Validate configuration:** Ensure constitutional compliance

### 9. Known Limitations

#### Constitutional Limits (Cannot be Changed)
- Maximum iteration limit: 10,000
- Maximum retry attempts: 3
- Budget timeout must be deterministic
- While loops cannot be parallelized

#### Current Implementation Limits
- JIT compilation requires > 1,000 iterations
- Safety analysis cache has fixed size
- Parallel execution requires D2 system availability
- Wall-clock kill switch is environment-dependent

#### Workarounds
- Break large loops into smaller chunks
- Use streaming patterns for large datasets
- Implement custom accumulator patterns for complex state
- Use explicit error recovery policies for robustness