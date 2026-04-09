//! Context Cache Implementation
//!
//! LRU cache with TTL for efficient context data access.

use crate::context::CacheStats;
use lru::LruCache;
use serde_json::Value;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

/// Context data with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct ContextData {
    pub items: Vec<Value>,
    pub loaded_at: Instant,
    pub ttl: Duration,
}

impl ContextData {
    /// Create new context data with default TTL (5 minutes)
    pub fn new(items: Vec<Value>) -> Self {
        Self {
            items,
            loaded_at: Instant::now(),
            ttl: Duration::from_secs(300), // 5 minutes default TTL
        }
    }

    /// Create new context data with custom TTL
    pub fn new_with_ttl(items: Vec<Value>, ttl: Duration) -> Self {
        Self {
            items,
            loaded_at: Instant::now(),
            ttl,
        }
    }

    /// Get items reference (C9: Accessor for encapsulation)
    pub fn items(&self) -> &Vec<Value> {
        &self.items
    }

    /// Check if data is expired
    pub fn is_expired(&self) -> bool {
        self.loaded_at.elapsed() > self.ttl
    }

    /// Get age of data
    pub fn age(&self) -> Duration {
        self.loaded_at.elapsed()
    }

    /// Get remaining TTL
    pub fn remaining_ttl(&self) -> Duration {
        if self.is_expired() {
            Duration::ZERO
        } else {
            self.ttl - self.loaded_at.elapsed()
        }
    }
}

/// LRU cache entry with access tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    data: ContextData,
    last_accessed: Instant,
    access_count: u64,
}

impl CacheEntry {
    fn new(data: ContextData) -> Self {
        Self {
            data,
            last_accessed: Instant::now(),
            access_count: 1,
        }
    }

    fn access(&mut self) -> &ContextData {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.data
    }

    fn is_expired(&self) -> bool {
        self.data.is_expired()
    }
}

/// LRU cache with TTL for context data
pub struct ContextCache {
    entries: LruCache<String, CacheEntry>,
    capacity: usize,
    stats: CacheStats,
}

impl ContextCache {
    /// Create new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let normalized_capacity = capacity.max(1);
        Self {
            entries: LruCache::new(
                NonZeroUsize::new(normalized_capacity).expect("cache capacity must be non-zero"),
            ),
            capacity: normalized_capacity,
            stats: CacheStats::default(),
        }
    }

    /// Get data from cache (returns None if expired or not found)
    pub fn get(&mut self, key: &str) -> Option<ContextData> {
        let mut expired = false;
        let data = match self.entries.get_mut(key) {
            Some(entry) => {
                if entry.is_expired() {
                    expired = true;
                    None
                } else {
                    Some(entry.access().clone())
                }
            }
            None => None,
        };

        if expired {
            let _ = self.entries.pop(key);
            self.stats.size = self.entries.len();
            self.stats.misses += 1;
            None
        } else if let Some(data) = data {
            self.stats.hits += 1;
            Some(data)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert data into cache
    pub fn insert(&mut self, key: String, data: ContextData) {
        self.entries.put(key, CacheEntry::new(data));
        self.stats.size = self.entries.len();
    }

    /// Remove entry from cache
    pub fn remove(&mut self, key: &str) -> Option<ContextData> {
        if let Some(entry) = self.entries.pop(key) {
            self.stats.size = self.entries.len();
            Some(entry.data)
        } else {
            None
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.stats.hits,
            misses: self.stats.misses,
            size: self.entries.len(),
            capacity: self.capacity,
        }
    }

    /// Clean expired entries
    pub fn clean_expired(&mut self) {
        let expired_keys: Vec<String> = self.entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            let _ = self.entries.pop(&key);
        }
        
        self.stats.size = self.entries.len();
    }

    /// Get cache utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        self.entries.len() as f64 / self.capacity as f64
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get detailed cache information for debugging
    pub fn debug_info(&self) -> HashMap<String, serde_json::Value> {
        let mut info = HashMap::new();
        
        info.insert("capacity".to_string(), serde_json::json!(self.capacity));
        info.insert("size".to_string(), serde_json::json!(self.entries.len()));
        info.insert("utilization".to_string(), serde_json::json!(self.utilization()));
        info.insert("hit_rate".to_string(), serde_json::json!(self.stats().hit_rate()));
        info.insert("total_hits".to_string(), serde_json::json!(self.stats.hits));
        info.insert("total_misses".to_string(), serde_json::json!(self.stats.misses));
        
        // Entry details
        let mut entries_info = Vec::new();
        for (key, entry) in &self.entries {
            entries_info.push(serde_json::json!({
                "key": key,
                "age_ms": entry.data.age().as_millis(),
                "remaining_ttl_ms": entry.data.remaining_ttl().as_millis(),
                "access_count": entry.access_count,
                "expired": entry.is_expired(),
                "item_count": entry.data.items.len()
            }));
        }
        info.insert("entries".to_string(), serde_json::json!(entries_info));
        
        info
    }
}

impl Default for ContextCache {
    fn default() -> Self {
        Self::new(100) // Default capacity of 100 entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_data() -> ContextData {
        let items = vec![
            json!({"id": "1", "name": "test1"}),
            json!({"id": "2", "name": "test2"}),
        ];
        ContextData::new(items)
    }

    fn create_test_data_with_ttl(ttl: Duration) -> ContextData {
        let items = vec![
            json!({"id": "1", "name": "test1"}),
        ];
        ContextData::new_with_ttl(items, ttl)
    }

    #[test]
    fn test_context_data_creation() {
        let data = create_test_data();
        assert_eq!(data.items.len(), 2);
        assert!(!data.is_expired());
        assert!(data.age() < Duration::from_millis(100));
        assert!(data.remaining_ttl() > Duration::from_secs(290));
    }

    #[test]
    fn test_context_data_expiration() {
        let data = create_test_data_with_ttl(Duration::from_millis(1));
        assert!(!data.is_expired());
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));
        assert!(data.is_expired());
        assert_eq!(data.remaining_ttl(), Duration::ZERO);
    }

    #[test]
    fn test_cache_creation() {
        let cache = ContextCache::new(10);
        assert_eq!(cache.capacity(), 10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.utilization(), 0.0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ContextCache::new(10);
        let data = create_test_data();
        
        // Insert data
        cache.insert("test_key".to_string(), data.clone());
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
        
        // Get data
        let retrieved = cache.get("test_key");
        assert!(retrieved.is_some());
        let retrieved_data = retrieved.unwrap();
        assert_eq!(retrieved_data.items.len(), data.items.len());
        
        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = ContextCache::new(10);
        
        let result = cache.get("nonexistent_key");
        assert!(result.is_none());
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = ContextCache::new(10);
        let data = create_test_data_with_ttl(Duration::from_millis(1));
        
        // Insert data
        cache.insert("test_key".to_string(), data);
        assert_eq!(cache.len(), 1);
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));
        
        // Try to get expired data
        let result = cache.get("test_key");
        assert!(result.is_none());
        assert_eq!(cache.len(), 0); // Should be removed
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = ContextCache::new(2); // Small capacity
        
        // Insert 3 items (should evict first)
        cache.insert("key1".to_string(), create_test_data());
        cache.insert("key2".to_string(), create_test_data());
        cache.insert("key3".to_string(), create_test_data());
        
        assert_eq!(cache.len(), 2);
        
        // key1 should be evicted
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_access_order() {
        let mut cache = ContextCache::new(2);
        
        // Insert 2 items
        cache.insert("key1".to_string(), create_test_data());
        cache.insert("key2".to_string(), create_test_data());
        
        // Access key1 to make it most recently used
        let _ = cache.get("key1");
        
        // Insert key3 (should evict key2, not key1)
        cache.insert("key3".to_string(), create_test_data());
        
        assert!(cache.get("key1").is_some()); // Still there
        assert!(cache.get("key2").is_none());  // Evicted
        assert!(cache.get("key3").is_some()); // New item
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = ContextCache::new(10);
        let data = create_test_data();
        
        cache.insert("test_key".to_string(), data.clone());
        assert_eq!(cache.len(), 1);
        
        let removed = cache.remove("test_key");
        assert!(removed.is_some());
        assert_eq!(cache.len(), 0);
        
        // Try to remove again
        let removed_again = cache.remove("test_key");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ContextCache::new(10);
        
        cache.insert("key1".to_string(), create_test_data());
        cache.insert("key2".to_string(), create_test_data());
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_clean_expired() {
        let mut cache = ContextCache::new(10);
        
        // Insert mix of expired and valid data
        cache.insert("valid".to_string(), create_test_data());
        cache.insert("expired".to_string(), create_test_data_with_ttl(Duration::from_millis(1)));
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));
        
        assert_eq!(cache.len(), 2);
        cache.clean_expired();
        assert_eq!(cache.len(), 1);
        
        // Valid data should still be there
        assert!(cache.get("valid").is_some());
        assert!(cache.get("expired").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ContextCache::new(10);
        let data = create_test_data();
        
        cache.insert("key1".to_string(), data);
        
        // Generate some hits and misses
        let _ = cache.get("key1"); // hit
        let _ = cache.get("key1"); // hit
        let _ = cache.get("key2"); // miss
        let _ = cache.get("key3"); // miss
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.size, 1);
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_utilization() {
        let mut cache = ContextCache::new(4);
        
        assert_eq!(cache.utilization(), 0.0);
        
        cache.insert("key1".to_string(), create_test_data());
        assert_eq!(cache.utilization(), 0.25);
        
        cache.insert("key2".to_string(), create_test_data());
        assert_eq!(cache.utilization(), 0.5);
        
        cache.insert("key3".to_string(), create_test_data());
        cache.insert("key4".to_string(), create_test_data());
        assert_eq!(cache.utilization(), 1.0);
    }

    #[test]
    fn test_cache_debug_info() {
        let mut cache = ContextCache::new(10);
        cache.insert("test_key".to_string(), create_test_data());
        
        let debug_info = cache.debug_info();
        
        assert!(debug_info.contains_key("capacity"));
        assert!(debug_info.contains_key("size"));
        assert!(debug_info.contains_key("utilization"));
        assert!(debug_info.contains_key("hit_rate"));
        assert!(debug_info.contains_key("entries"));
        
        let entries = debug_info.get("entries").unwrap().as_array().unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        assert!(entry.get("key").is_some());
        assert!(entry.get("age_ms").is_some());
        assert!(entry.get("remaining_ttl_ms").is_some());
        assert!(entry.get("access_count").is_some());
        assert!(entry.get("expired").is_some());
        assert!(entry.get("item_count").is_some());
    }

    #[test]
    fn test_cache_performance() {
        let mut cache = ContextCache::new(1000);
        
        // Insert many items
        let start = Instant::now();
        for i in 0..1000 {
            cache.insert(format!("key_{}", i), create_test_data());
        }
        let insert_duration = start.elapsed();
        
        // Should be fast (< 100ms for 1000 inserts)
        assert!(insert_duration.as_millis() < 100);
        
        // Access items
        let start = Instant::now();
        for i in 0..1000 {
            let _ = cache.get(&format!("key_{}", i));
        }
        let access_duration = start.elapsed();
        
        // Should be very fast (< 50ms for 1000 accesses)
        assert!(access_duration.as_millis() < 50);
    }
}
