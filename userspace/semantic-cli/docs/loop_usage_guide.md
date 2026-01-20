# D3 Loop Support Usage Guide

**Version:** Phase 3.5 - BCIB Architecture  
**Status:** Production Ready  
**Scope:** Semantic CLI runtime (userspace). Kernel-side execution excluded.  
**Last Updated:** January 2026

## Overview

The D3 Loop Support system provides bounded, deterministic iteration capabilities in AykenOS Semantic CLI. All loops are constitutional-compliant with mandatory iteration limits and deterministic timeout enforcement.

**Critical Note:** This guide assumes familiarity with BCIB (Binary/Bounded/Capability-aware Instruction Block) architecture. Loop conditions and expressions are pre-resolved at the parser layer before BCIB instruction creation. String expressions like `"x < 10"` are NOT valid BCIB operands - they must be resolved to concrete `Value` types.

## Quick Start

### Basic For Loop

```rust
use semantic_cli::bcib::{LoopInstruction, LoopID, LoopRange, LoopConfig, Value, ValueType};
use semantic_cli::loop_engine::LoopEngine;

let mut engine = LoopEngine::new();

// Create a simple For loop: sum numbers 1 to 5
let for_loop = LoopInstruction::For {
    id: LoopID::new("sum-example".to_string()),
    range: LoopRange::new(1, 6, 1), // 1,2,3,4,5
    iterator_var: "i".to_string(),
    body: "sum += i".to_string(),
    config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
    location: SourceLocation::new(1, 1, 0),
};

// Execute with body function
let result = engine.execute_loop(&for_loop, Box::new(|accumulator, iteration| {
    let current_sum = match accumulator {
        Value::Number(n) => *n,
        _ => 0.0,
    };
    let new_sum = current_sum + iteration as f64;
    Ok(LoopBodyResult::Continue(Value::Number(new_sum)))
}));

// Result: Value::Number(15.0) - sum of 1+2+3+4+5
```

**Note:** `iteration` is a zero-based executor counter and does not reflect `LoopRange.start`.

### Basic While Loop

```rust
// While loop with pre-resolved condition
// NOTE: While loop conditions are NOT string expressions in BCIB.
// Conditions must be resolved at parser layer before BCIB instruction creation.
let while_loop = LoopInstruction::While {
    id: LoopID::new("countdown".to_string()),
    condition: OperandRef::Literal(Value::Boolean(true)), // Pre-resolved condition
    body: "counter -= 1".to_string(),
    config: {
        let mut config = LoopConfig::new(Value::Number(10.0), ValueType::Number);
        config.iteration_limit = 100;
        config.budget_timeout = 1000;
        config.budget_measurement = BudgetMeasurement::IterationCount;
        config.error_recovery = ErrorRecoveryPolicy::Abort;
        config
    },
    location: SourceLocation::new(1, 1, 0),
};
```

**Important:** While loop conditions in BCIB are pre-resolved Values, not string expressions. Runtime condition evaluation happens at the executor level, not in the BCIB instruction.

**Note:** While loops are intentionally conservative and excluded from parallelization for determinism and safety.

### ForEach Loop with Arrays

```rust
// ForEach over deterministic collection
let foreach_loop = LoopInstruction::ForEach {
    id: LoopID::new("process-items".to_string()),
    collection: OperandRef::Literal(Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
    ])),
    collection_type: CollectionType::Array,
    iterator_var: "item".to_string(),
    body: "total += item".to_string(),
    config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
    location: SourceLocation::new(1, 1, 0),
};
```

## Constitutional Compliance

### Bounded Iteration (Mandatory)

All loops MUST have iteration limits. Default: 10,000 iterations.

```rust
// Use builder pattern for LoopConfig (constitutional compliance)
let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
config.iteration_limit = 1000;  // Never exceeded
config.budget_timeout = 10000;
config.error_recovery = ErrorRecoveryPolicy::Abort;
```

**Guarantee:** Loop terminates at exactly `iteration_limit`, never exceeds.

### Deterministic Timeout (Mandatory)

Budget timeout is measured deterministically, not by wall-clock time.

```rust
let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
config.budget_timeout = 5000;
config.budget_measurement = BudgetMeasurement::IterationCount;
```

**Options:**
- `IterationCount`: Simple iteration counting
- `InstructionCount { weight }`: Instruction-level measurement
- `Hybrid { multiplier }`: Iteration × average instruction count

### Error Recovery Policies (Explicit Only)

```rust
// Default: Abort on error
error_recovery: ErrorRecoveryPolicy::Abort,

// Explicit retry with increased limit
error_recovery: ErrorRecoveryPolicy::RetryWithIncreasedLimit {
    new_limit: 2000,    // Must not exceed 10,000
    max_retries: 2,     // Must not exceed 3
},

// Explicit partial results
error_recovery: ErrorRecoveryPolicy::ReturnPartialResults {
    include_error_info: true,
},
```

## Collection Determinism

### Supported Collections (Deterministic Order)

```rust
// ✅ Arrays - index order (0, 1, 2, ...)
CollectionType::Array

// ✅ Lists - insertion order
CollectionType::List

// ✅ Sorted Maps - key sort order
CollectionType::SortedMap
```

### Rejected Collections (Non-Deterministic)

```rust
// ❌ Hash Maps - unless canonical ordering provided
CollectionType::HashMap { canonical_ordering: None }  // REJECTED

// ✅ Hash Maps - with explicit ordering
CollectionType::HashMap { 
    canonical_ordering: Some("key_sort".to_string()) 
}  // ACCEPTED
```

## Performance Characteristics

**Note:** All performance figures are order-of-magnitude estimates from development environments.

### Loop Execution Overhead

| Loop Type | Overhead | Notes |
|-----------|----------|-------|
| Small For loops (< 10 iterations) | ~1μs | Automatically unrolled |
| Regular loops | ~10-50μs | Interpreted execution |
| Hot loops (> 1,000 iterations) | ~5-20μs | JIT compiled |
| Parallel loops | ~100μs + execution | D2 parallelism overhead |

### Memory Usage

| Component | Memory per Loop | Notes |
|-----------|----------------|-------|
| Loop state | ~200 bytes | Context + accumulator |
| Safety cache | ~1KB per unique body | Cached analysis results |
| JIT cache | ~5-50KB per compiled body | Native code cache |
| Monitoring | ~100 bytes per execution | Stats tracking |

**Note:** JIT cache memory is shared across executions and may appear persistent.

### Optimization Triggers

```rust
// Automatic unrolling for small loops
if static_iteration_count < 10 {
    // Loop body expanded to sequential instructions
}

// Hot loop detection
if total_iterations > 1000 {
    // JIT compilation triggered
}

// Parallelization (Safe loops only)
if safety_class == Safe && loop_type != While {
    // D2 parallel execution
}
```

## Error Handling

### Common Errors

```rust
// Handle loop execution errors (API-agnostic approach)
match result {
    Ok(value) => {
        // Process successful result
    }
    Err(error) => {
        let error_msg = error.to_string();
        
        if error_msg.contains("iteration") {
            // Handle iteration limit exceeded
            println!("Loop hit iteration limit");
        } else if error_msg.contains("budget") || error_msg.contains("timeout") {
            // Handle budget timeout
            println!("Loop exceeded budget timeout");
        } else if error_msg.contains("deterministic") || error_msg.contains("ordering") {
            // Handle collection ordering issues
            println!("Use ordered collection or provide canonical ordering");
        } else {
            // Handle other execution errors
            println!("Loop execution failed: {}", error);
        }
    }
}
```

**Note:** Error handling uses string pattern matching for robustness across different error wrapper types.

### Error Recovery Examples

```rust
// Retry with increased limit
let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
config.error_recovery = ErrorRecoveryPolicy::RetryWithIncreasedLimit {
    new_limit: 2000,
    max_retries: 1,
};

// Get partial results on timeout
let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
config.error_recovery = ErrorRecoveryPolicy::ReturnPartialResults {
    include_error_info: true,
};
```

## Monitoring and Stats

### Loop Statistics

```rust
let stats = engine.get_loop_stats(&loop_id);
if let Some(stats) = stats {
    println!("Executions: {}", stats.execution_count);
    println!("Total iterations: {}", stats.total_iterations);
    println!("Avg time: {:?}", stats.average_execution_time);
    println!("Was parallelized: {}", stats.was_parallelized);
    println!("Was JIT compiled: {}", stats.was_jit_compiled);
}
```

### Hot Loop Detection

**Note:** Hot loop detection and JIT compilation availability depends on system configuration and D1 integration status.

```rust
// Check if hot loop monitoring is available and enabled
let loop_id = LoopID::new("my-loop".to_string());

if engine.is_hot_loop(&loop_id) {
    if let Some(hot_info) = engine.get_hot_loop_info(&loop_id) {
        println!("JIT status: {:?}", hot_info.jit_status);
        println!("Compilation time: {:?}", hot_info.compilation_time);
    }
}
```

### Safety Analysis Cache

```rust
let cache_stats = engine.get_safety_cache_stats();
println!("Cache entries: {}", cache_stats.entries);
println!("Hit rate: {:.2}%", cache_stats.hit_rate * 100.0);
```

## Advanced Usage

### Custom Budget Measurement

```rust
let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
config.budget_measurement = BudgetMeasurement::InstructionCount { weight: 10 };
config.budget_timeout = 50000; // 5000 instructions × 10 weight
```

### Loop Body with Break/Continue

```rust
let body = Box::new(|accumulator, iteration| {
    if iteration > 100 {
        // Early termination
        Ok(LoopBodyResult::Break(accumulator.clone()))
    } else if iteration % 2 == 0 {
        // Skip even iterations
        Ok(LoopBodyResult::Continue(accumulator.clone()))
    } else {
        // Normal processing
        let new_value = process_iteration(accumulator, iteration);
        Ok(LoopBodyResult::Continue(new_value))
    }
});
```

### Safety Analysis Integration

```rust
let context = LoopAnalysisContext::new();
let safety_result = engine.analyze_loop_safety(&loop_body, &context)?;

match safety_result.classification {
    SafetyClass::Safe => {
        // Loop may be eligible for parallelization
        // (subject to runtime conditions and D2 availability)
        println!("Loop is safe for potential parallelization");
    }
    SafetyClass::Unsafe => {
        // Sequential execution only
        println!("Loop has side effects: {}", safety_result.reason);
    }
}
```

## Best Practices

### 1. Use Appropriate Loop Types

```rust
// ✅ For loops for known ranges
LoopInstruction::For { range: LoopRange::new(0, 100, 1), .. }

// ✅ ForEach for collections
LoopInstruction::ForEach { collection_type: CollectionType::Array, .. }

// ⚠️ While loops (never parallelized)
LoopInstruction::While { .. }  // Use sparingly
```

### 2. Optimize for Performance

```rust
// Small loops: let unroller handle them
if iterations < 10 {
    // Will be automatically unrolled
}

// Large loops: design for parallelization
// - Avoid side effects
// - Avoid loop-carried dependencies
// - Use deterministic collections
```

### 3. Handle Errors Gracefully

```rust
match engine.execute_loop(&instruction, body) {
    Ok(result) => {
        // Process successful result
    }
    Err(error) => {
        if error.to_string().contains("iteration") {
            // Increase limit or use partial results
        } else if error.to_string().contains("deterministic") {
            // Fix collection ordering
        } else {
            // Handle other errors
        }
    }
}
```

### 4. Monitor Performance

```rust
// Check if loop became hot
if engine.is_hot_loop(&loop_id) {
    // JIT compilation occurred - performance improved
}

// Monitor cache efficiency
let cache_stats = engine.get_safety_cache_stats();
if cache_stats.hit_rate < 0.8 {
    // Consider cache tuning
}
```

## Troubleshooting

See [Loop Troubleshooting Guide](./loop_troubleshooting_guide.md) for common issues and solutions.

## Integration Examples

See [Loop Integration Examples](./loop_integration_examples.md) for real-world usage patterns with D1 JIT and D2 parallelism systems.