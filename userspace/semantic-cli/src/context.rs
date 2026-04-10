//! Context Management System
//!
//! This module implements context loading, caching, and access management for Phase 3.5.1.a.
//!
//! # Design Principles
//!
//! 1. **Read-only:** No mutation operations in Phase 3.5.1.a
//! 2. **Contextual capabilities:** Fine-grained access control using AR-4
//! 3. **Performance:** < 20ms cached, < 100ms uncached
//! 4. **Caching:** LRU cache with TTL for efficient data access
//! 5. **Lazy loading:** Load contexts on first access
//! 6. **Schema validation:** Type-safe context data access
//!
//! # Supported Contexts
//!
//! - `data.users` - User data (mock database)
//! - `data.logs` - System logs (mock log files)
//! - `fs.logs` - Filesystem logs (mock /var/log)
//! - `system.processes` - Running processes (mock /proc)
//! - `system.agents` - Active agents (mock orchestrator)
//!
//! # Phase 3.5.1.a Constraints
//!
//! - **Read-only:** No write/update/delete operations
//! - **Mock data:** All loaders return mock data for testing
//! - **Contextual capabilities:** Access control via Read{context} capabilities
//! - **No persistence:** Data is generated on-demand, not persisted

use crate::bcib::Capability;
use crate::error::{ErrorCode, Result, SemanticCLIError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub mod cache;
pub mod loaders;
pub mod registry;

pub use cache::{ContextCache, ContextData};
pub use loaders::{
    ContextLoader, MockAgentLoader, MockLogLoader, MockProcessLoader, MockUserLoader,
};
pub use registry::ContextRegistry;

/// Context Manager - Main interface for context operations
pub struct ContextManager {
    registry: Arc<ContextRegistry>,
    cache: Arc<Mutex<ContextCache>>,
}

impl ContextManager {
    /// Create a new context manager with default configuration
    pub fn new() -> Self {
        let registry = Arc::new(ContextRegistry::new());
        let cache = Arc::new(Mutex::new(ContextCache::new(100))); // 100 item LRU cache

        Self { registry, cache }
    }

    /// Load context data with caching and capability checking
    pub fn load_context(&self, path: &str, capability: &Capability) -> Result<Vec<Value>> {
        // Check capability first
        self.check_capability(path, capability)?;

        // Try cache first
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(data) = cache.get(path) {
                return Ok(data.items.clone());
            }
        }

        // Load from registry
        let data = self.registry.load_context(path)?;

        // Cache the result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(path.to_string(), data.clone());
        }

        Ok(data.items)
    }

    /// Check if context exists
    pub fn context_exists(&self, path: &str) -> bool {
        self.registry.context_exists(path)
    }

    /// List available contexts
    pub fn list_contexts(&self) -> Vec<String> {
        self.registry.list_contexts()
    }

    /// Get context schema information
    pub fn get_context_schema(&self, path: &str) -> Result<HashMap<String, String>> {
        self.registry.get_context_schema(path)
    }

    /// Check contextual capability for context access
    fn check_capability(&self, path: &str, capability: &Capability) -> Result<()> {
        match capability {
            Capability::Read { context } => {
                if context == path {
                    Ok(())
                } else {
                    Err(SemanticCLIError::security_error(
                        format!("Capability mismatch: required Read{{context: '{}'}}, got Read{{context: '{}'}}", path, context),
                        ErrorCode::E601,
                    ))
                }
            }
            _ => Err(SemanticCLIError::security_error(
                format!("Invalid capability for context access: {:?}", capability),
                ErrorCode::E601,
            )),
        }
    }

    /// Invalidate cache for a specific context
    pub fn invalidate_cache(&self, path: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(path);
        }
    }

    /// Clear entire cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        if let Ok(cache) = self.cache.lock() {
            cache.stats()
        } else {
            CacheStats::default()
        }
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new();
        assert!(manager.context_exists("data.users"));
        assert!(manager.context_exists("fs.logs"));
        assert!(manager.context_exists("system.processes"));
    }

    #[test]
    fn test_context_loading_with_capability() {
        let manager = ContextManager::new();
        let capability = Capability::Read {
            context: "data.users".to_string(),
        };

        let result = manager.load_context("data.users", &capability);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_capability_mismatch() {
        let manager = ContextManager::new();
        let capability = Capability::Read {
            context: "data.logs".to_string(),
        };

        let result = manager.load_context("data.users", &capability);
        assert!(result.is_err());

        if let Err(SemanticCLIError::SecurityError { .. }) = result {
            // Expected security error
        } else {
            panic!("Expected SecurityError");
        }
    }

    #[test]
    fn test_context_caching() {
        let manager = ContextManager::new();
        let capability = Capability::Read {
            context: "data.users".to_string(),
        };

        // First load (cache miss)
        let start = Instant::now();
        let result1 = manager.load_context("data.users", &capability);
        let first_duration = start.elapsed();
        assert!(result1.is_ok());

        // Second load (cache hit)
        let start = Instant::now();
        let result2 = manager.load_context("data.users", &capability);
        let second_duration = start.elapsed();
        assert!(result2.is_ok());

        // Cache hit should be faster
        assert!(second_duration < first_duration);

        // Data should be identical
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_list_contexts() {
        let manager = ContextManager::new();
        let contexts = manager.list_contexts();

        assert!(contexts.contains(&"data.users".to_string()));
        assert!(contexts.contains(&"fs.logs".to_string()));
        assert!(contexts.contains(&"system.processes".to_string()));
    }

    #[test]
    fn test_cache_invalidation() {
        let manager = ContextManager::new();
        let capability = Capability::Read {
            context: "data.users".to_string(),
        };

        // Load data to cache
        let _ = manager.load_context("data.users", &capability);

        // Invalidate cache
        manager.invalidate_cache("data.users");

        // Next load should be fresh
        let result = manager.load_context("data.users", &capability);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_stats() {
        let manager = ContextManager::new();
        let capability = Capability::Read {
            context: "data.users".to_string(),
        };

        // Initial stats
        let stats = manager.cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Load data (should be cache miss)
        let _ = manager.load_context("data.users", &capability);

        // Load again (should be cache hit)
        let _ = manager.load_context("data.users", &capability);

        let stats = manager.cache_stats();
        assert!(stats.hits > 0 || stats.misses > 0);
    }

    #[test]
    fn test_performance_targets() {
        let manager = ContextManager::new();
        let capability = Capability::Read {
            context: "data.users".to_string(),
        };

        // First load (uncached) - should be < 100ms
        let start = Instant::now();
        let _ = manager.load_context("data.users", &capability);
        let uncached_duration = start.elapsed();
        assert!(
            uncached_duration.as_millis() < 100,
            "Uncached load took {}ms",
            uncached_duration.as_millis()
        );

        // Second load (cached) - should be < 20ms
        let start = Instant::now();
        let _ = manager.load_context("data.users", &capability);
        let cached_duration = start.elapsed();
        assert!(
            cached_duration.as_millis() < 20,
            "Cached load took {}ms",
            cached_duration.as_millis()
        );
    }
}
