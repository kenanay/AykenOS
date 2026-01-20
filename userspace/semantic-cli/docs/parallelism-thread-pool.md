# Thread Pool Configuration

## Overview

The D2 Parallelism Architecture uses Rayon for data-parallel execution. The thread pool must be configured once at application startup and is reused throughout the application lifecycle.

## Requirements

- **Requirement 5.1**: Use a thread pool (Rayon) for parallel execution
- **Requirement 5.2**: Thread pool initialized once and reused across operations
- **Requirement 5.3**: Thread pool size configured based on available CPU cores
- **Requirement 5.4**: Thread pool errors handled gracefully without silent failures

## Basic Usage

### Default Configuration (Recommended)

The simplest way to initialize the thread pool is to use the default configuration, which automatically sets the number of threads to match the number of CPU cores:

```rust
use semantic_cli::parallelism::config::initialize_default_thread_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize thread pool at application startup
    initialize_default_thread_pool()?;
    
    // Rest of your application...
    Ok(())
}
```

### Custom Configuration

For more control over thread pool settings, you can create a custom configuration:

```rust
use semantic_cli::parallelism::config::{initialize_thread_pool, ThreadPoolConfig};
use std::num::NonZeroUsize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = ThreadPoolConfig::new()
        .with_num_threads(NonZeroUsize::new(4).unwrap())
        .with_thread_name_prefix("my-worker")
        .with_stack_size(2 * 1024 * 1024); // 2 MB stack per thread
    
    // Initialize thread pool with custom configuration
    initialize_thread_pool(config)?;
    
    // Rest of your application...
    Ok(())
}
```

## Configuration Options

### Number of Threads

By default, the thread pool uses the number of available CPU cores. You can override this:

```rust
let config = ThreadPoolConfig::new()
    .with_num_threads(NonZeroUsize::new(8).unwrap());
```

**When to customize:**
- Testing with specific thread counts
- Running in containerized environments with CPU limits
- Performance tuning for specific workloads

### Thread Name Prefix

Thread names are useful for debugging and profiling:

```rust
let config = ThreadPoolConfig::new()
    .with_thread_name_prefix("semantic-worker");
```

Threads will be named: `semantic-worker-0`, `semantic-worker-1`, etc.

### Stack Size

The default stack size is usually sufficient, but you can customize it if needed:

```rust
let config = ThreadPoolConfig::new()
    .with_stack_size(4 * 1024 * 1024); // 4 MB stack per thread
```

**When to customize:**
- Deep recursion in parallel operations
- Large stack-allocated data structures

## Error Handling

Thread pool initialization can fail in the following cases:

1. **Already Initialized**: The thread pool can only be initialized once
2. **Invalid Configuration**: Invalid thread count or stack size
3. **System Errors**: OS-level failures creating threads

All errors are returned as `ParallelismError::ThreadPoolInitialization`:

```rust
use semantic_cli::parallelism::config::initialize_default_thread_pool;

match initialize_default_thread_pool() {
    Ok(()) => println!("Thread pool initialized successfully"),
    Err(e) => eprintln!("Failed to initialize thread pool: {}", e),
}
```

## Querying Thread Pool Status

You can query the current number of threads in the pool:

```rust
use semantic_cli::parallelism::config::current_num_threads;

let num_threads = current_num_threads();
println!("Thread pool has {} threads", num_threads);
```

## Best Practices

### 1. Initialize Early

Initialize the thread pool as early as possible in your application:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize thread pool FIRST
    initialize_default_thread_pool()?;
    
    // Then initialize other components
    let config = load_config()?;
    let executor = create_executor()?;
    
    // Run application
    run_application(config, executor)?;
    
    Ok(())
}
```

### 2. Use Default Configuration

Unless you have specific requirements, use the default configuration:

```rust
// ✅ Good: Simple and correct
initialize_default_thread_pool()?;

// ❌ Avoid: Unnecessary complexity
let config = ThreadPoolConfig::new()
    .with_num_threads(NonZeroUsize::new(num_cpus::get()).unwrap());
initialize_thread_pool(config)?;
```

### 3. Handle Initialization Errors

Always handle initialization errors gracefully:

```rust
// ✅ Good: Proper error handling
if let Err(e) = initialize_default_thread_pool() {
    eprintln!("Failed to initialize thread pool: {}", e);
    std::process::exit(1);
}

// ❌ Avoid: Silent failures
let _ = initialize_default_thread_pool();
```

### 4. Don't Re-Initialize

The thread pool can only be initialized once. Attempting to re-initialize will fail:

```rust
// First initialization succeeds
initialize_default_thread_pool()?;

// Second initialization fails
match initialize_default_thread_pool() {
    Ok(()) => unreachable!("Should not succeed"),
    Err(e) => println!("Expected error: {}", e),
}
```

## Testing

For tests, you may want to control the number of threads:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    fn initialize_test_pool() {
        INIT.call_once(|| {
            let config = ThreadPoolConfig::new()
                .with_num_threads(NonZeroUsize::new(2).unwrap());
            initialize_thread_pool(config).ok();
        });
    }
    
    #[test]
    fn test_parallel_operation() {
        initialize_test_pool();
        // Test code here...
    }
}
```

## Performance Considerations

### CPU Core Count

The default configuration uses the number of CPU cores, which is optimal for most workloads:

- **CPU-bound tasks**: Number of cores is ideal
- **I/O-bound tasks**: May benefit from more threads
- **Mixed workloads**: Start with default, tune if needed

### Overhead

Thread pool overhead is minimal when properly configured:

- **Initialization**: One-time cost at startup
- **Task scheduling**: Rayon's work-stealing is efficient
- **Synchronization**: Minimal overhead for data-parallel operations

### Tuning

If you need to tune thread pool size:

1. Start with default (CPU cores)
2. Measure performance with your workload
3. Adjust thread count if needed
4. Re-measure to verify improvement

## Architecture Compliance

This implementation satisfies the following requirements from the D2 Parallelism Architecture:

- ✅ **Requirement 5.1**: Uses Rayon thread pool for parallel execution
- ✅ **Requirement 5.2**: Thread pool initialized once and reused
- ✅ **Requirement 5.3**: Thread pool size based on CPU cores
- ✅ **Requirement 5.4**: Graceful error handling without silent failures

## See Also

- [D2 Parallelism Architecture Design Document](../../../_ayken/specs/phase3-5-semantic-interaction/d2-parallelism-architecture/design.md)
- [Rayon Documentation](https://docs.rs/rayon/)
- [Thread Pool Configuration API](../src/parallelism/config.rs)
