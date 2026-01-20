//! # Thread Pool Configuration
//!
//! This module provides configuration for the Rayon thread pool used in parallel execution.
//!
//! ## Requirements
//!
//! - **Requirement 5.2**: Thread pool SHALL be initialized once and reused across operations
//! - **Requirement 5.3**: Thread pool size SHALL be configured based on available CPU cores
//! - **Requirement 5.4**: Thread pool errors SHALL be handled gracefully without silent failures
//!
//! ## Design
//!
//! The thread pool is configured once at initialization and reused throughout the application
//! lifecycle. The pool size is determined by the number of available CPU cores, with
//! configurable overrides for testing and tuning.

use std::num::NonZeroUsize;
use std::sync::OnceLock;
use rayon::ThreadPoolBuilder;
use crate::parallelism::error::{ParallelismError, ParallelismResult};

/// Global thread pool configuration
static THREAD_POOL_CONFIG: OnceLock<ThreadPoolConfig> = OnceLock::new();

/// Thread pool configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Number of worker threads in the pool
    /// If None, uses the number of available CPU cores
    pub num_threads: Option<NonZeroUsize>,
    
    /// Thread name prefix for debugging
    pub thread_name_prefix: String,
    
    /// Stack size per thread (in bytes)
    /// If None, uses Rayon's default
    pub stack_size: Option<usize>,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            num_threads: None, // Use CPU core count
            thread_name_prefix: "semantic-worker".to_string(),
            stack_size: None, // Use Rayon default
        }
    }
}

impl ThreadPoolConfig {
    /// Create a new thread pool configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the number of worker threads
    ///
    /// If not set, the thread pool will use the number of available CPU cores.
    pub fn with_num_threads(mut self, num_threads: NonZeroUsize) -> Self {
        self.num_threads = Some(num_threads);
        self
    }
    
    /// Set the thread name prefix for debugging
    pub fn with_thread_name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.thread_name_prefix = prefix.into();
        self
    }
    
    /// Set the stack size per thread
    pub fn with_stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = Some(stack_size);
        self
    }
    
    /// Get the effective number of threads
    ///
    /// Returns the configured number of threads, or the number of CPU cores if not configured.
    pub fn effective_num_threads(&self) -> usize {
        self.num_threads
            .map(|n| n.get())
            .unwrap_or_else(num_cpus)
    }
    
    /// Initialize the global Rayon thread pool with this configuration
    ///
    /// This should be called once at application startup. Subsequent calls will
    /// return an error.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The thread pool has already been initialized
    /// - Rayon fails to build the thread pool
    ///
    /// # Requirements
    ///
    /// - **Validates: Requirement 5.2** - Thread pool initialized once and reused
    /// - **Validates: Requirement 5.3** - Thread pool size based on CPU cores
    /// - **Validates: Requirement 5.4** - Graceful error handling
    pub fn initialize_global_pool(&self) -> ParallelismResult<()> {
        let mut builder = ThreadPoolBuilder::new();
        
        // Configure number of threads based on CPU cores
        if let Some(num_threads) = self.num_threads {
            builder = builder.num_threads(num_threads.get());
        } else {
            // Use number of CPU cores (Rayon's default behavior)
            builder = builder.num_threads(num_cpus());
        }
        
        // Configure thread naming for debugging
        // Clone the prefix to move into the closure (required for 'static lifetime)
        let thread_name_prefix = self.thread_name_prefix.clone();
        builder = builder.thread_name(move |idx| {
            format!("{}-{}", thread_name_prefix, idx)
        });
        
        // Configure stack size if specified
        if let Some(stack_size) = self.stack_size {
            builder = builder.stack_size(stack_size);
        }
        
        // Build and install the global thread pool
        builder.build_global()
            .map_err(|e| ParallelismError::ThreadPoolInitialization {
                reason: format!("Failed to initialize Rayon thread pool: {}", e),
            })
    }
}

/// Get the number of available CPU cores
///
/// This uses `std::thread::available_parallelism()` to determine the number
/// of hardware threads available to the process.
///
/// # Requirements
///
/// - **Validates: Requirement 5.3** - Thread pool size based on CPU cores
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) // Fallback to 1 if detection fails
}

/// Initialize the global thread pool with default configuration
///
/// This is a convenience function that initializes the thread pool with
/// default settings (number of threads = CPU cores).
///
/// # Errors
///
/// Returns an error if the thread pool has already been initialized or
/// if Rayon fails to build the thread pool.
///
/// # Example
///
/// ```rust,ignore
/// use semantic_cli::parallelism::config::initialize_default_thread_pool;
///
/// // Call once at application startup
/// initialize_default_thread_pool()?;
/// ```
pub fn initialize_default_thread_pool() -> ParallelismResult<()> {
    ThreadPoolConfig::default().initialize_global_pool()
}

/// Initialize the global thread pool with custom configuration
///
/// # Errors
///
/// Returns an error if the thread pool has already been initialized or
/// if Rayon fails to build the thread pool.
///
/// # Example
///
/// ```rust,ignore
/// use semantic_cli::parallelism::config::{initialize_thread_pool, ThreadPoolConfig};
/// use std::num::NonZeroUsize;
///
/// let config = ThreadPoolConfig::new()
///     .with_num_threads(NonZeroUsize::new(4).unwrap())
///     .with_thread_name_prefix("my-worker");
///
/// initialize_thread_pool(config)?;
/// ```
pub fn initialize_thread_pool(config: ThreadPoolConfig) -> ParallelismResult<()> {
    config.initialize_global_pool()
}

/// Get the number of threads in the current thread pool
///
/// Returns the number of threads configured in the Rayon global thread pool.
/// If the pool hasn't been initialized, returns the number of CPU cores.
pub fn current_num_threads() -> usize {
    rayon::current_num_threads()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ThreadPoolConfig::default();
        assert_eq!(config.thread_name_prefix, "semantic-worker");
        assert!(config.num_threads.is_none());
        assert!(config.stack_size.is_none());
    }
    
    #[test]
    fn test_config_builder() {
        let config = ThreadPoolConfig::new()
            .with_num_threads(NonZeroUsize::new(4).unwrap())
            .with_thread_name_prefix("test-worker")
            .with_stack_size(2 * 1024 * 1024);
        
        assert_eq!(config.num_threads.unwrap().get(), 4);
        assert_eq!(config.thread_name_prefix, "test-worker");
        assert_eq!(config.stack_size.unwrap(), 2 * 1024 * 1024);
    }
    
    #[test]
    fn test_effective_num_threads() {
        // With explicit configuration
        let config = ThreadPoolConfig::new()
            .with_num_threads(NonZeroUsize::new(8).unwrap());
        assert_eq!(config.effective_num_threads(), 8);
        
        // With default (CPU cores)
        let config = ThreadPoolConfig::default();
        let num_threads = config.effective_num_threads();
        assert!(num_threads >= 1, "Should have at least 1 thread");
    }
    
    #[test]
    fn test_num_cpus() {
        let cpus = num_cpus();
        assert!(cpus >= 1, "Should detect at least 1 CPU core");
    }
}
