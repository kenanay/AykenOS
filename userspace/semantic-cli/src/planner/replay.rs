//! Deterministic replay mode for audit and debugging
//!
//! This module provides functionality for deterministic replay of planning
//! operations, enabling audit trails and debugging capabilities.

use crate::types::*;
use crate::error::PlanningError;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{debug, info};

/// Replay controller for deterministic plan generation
pub struct ReplayController {
    /// Whether deterministic mode is enabled
    deterministic_mode: bool,
    /// Cache of plans for replay
    plan_cache: HashMap<String, CachedPlan>,
    /// Replay session information
    session_info: ReplaySession,
    /// Configuration for replay behavior
    config: ReplayConfig,
}

/// Configuration for replay behavior
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Maximum number of cached plans
    pub max_cached_plans: usize,
    /// Whether to use strict determinism (exact input matching)
    pub strict_determinism: bool,
    /// Cache expiration time in seconds
    pub cache_expiration_secs: u64,
    /// Whether to log replay operations
    pub log_replay_operations: bool,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            max_cached_plans: 1000,
            strict_determinism: true,
            cache_expiration_secs: 3600, // 1 hour
            log_replay_operations: true,
        }
    }
}

/// Cached plan for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPlan {
    /// The cached execution plan
    pub plan: ExecutionPlan,
    /// Original intent that generated this plan
    pub original_intent: Intent,
    /// When this plan was cached
    pub cached_at: DateTime<Utc>,
    /// Replay metadata
    pub metadata: ReplayMetadata,
    /// Hash of the input for deterministic matching
    pub input_hash: String,
}

/// Replay metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMetadata {
    /// Planning session ID
    pub session_id: String,
    /// User context at time of planning
    pub user_context: UserContext,
    /// Planning configuration used
    pub planning_config: String, // Serialized config
    /// Number of times this plan has been replayed
    pub replay_count: u32,
    /// Last replay timestamp
    pub last_replayed: Option<DateTime<Utc>>,
}

/// Replay session information
#[derive(Debug, Clone)]
pub struct ReplaySession {
    /// Session ID
    pub session_id: String,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Number of plans generated in this session
    pub plans_generated: u32,
    /// Number of cache hits in this session
    pub cache_hits: u32,
    /// Session configuration
    pub config: ReplayConfig,
}

/// Replay statistics
#[derive(Debug, Clone)]
pub struct ReplayStatistics {
    /// Total number of cached plans
    pub total_cached_plans: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f32,
    /// Average plan generation time
    pub avg_generation_time: std::time::Duration,
    /// Memory usage of cache
    pub cache_memory_usage: u64,
}

impl ReplayController {
    /// Create a new replay controller
    pub fn new() -> Self {
        Self {
            deterministic_mode: false,
            plan_cache: HashMap::new(),
            session_info: ReplaySession {
                session_id: Uuid::new_v4().to_string(),
                start_time: Utc::now(),
                plans_generated: 0,
                cache_hits: 0,
                config: ReplayConfig::default(),
            },
            config: ReplayConfig::default(),
        }
    }

    /// Create replay controller with custom configuration
    pub fn with_config(config: ReplayConfig) -> Self {
        let mut controller = Self::new();
        controller.config = config.clone();
        controller.session_info.config = config;
        controller
    }

    /// Set deterministic mode
    pub fn set_deterministic_mode(&mut self, enabled: bool) {
        info!("Setting deterministic replay mode: {}", enabled);
        self.deterministic_mode = enabled;
        
        if enabled && self.config.log_replay_operations {
            info!("Deterministic replay mode enabled for session: {}", self.session_info.session_id);
        }
    }

    /// Check if deterministic mode is enabled
    pub fn is_deterministic_mode(&self) -> bool {
        self.deterministic_mode
    }

    /// Get cached plan for an intent if available
    pub async fn get_cached_plan(&mut self, intent: &Intent) -> Result<Option<ExecutionPlan>, PlanningError> {
        if !self.deterministic_mode {
            return Ok(None);
        }

        let input_hash = self.calculate_input_hash(intent)?;
        
        debug!("Looking for cached plan with hash: {}", input_hash);

        // Check if cache entry exists and is valid
        let is_valid = if let Some(cached_plan) = self.plan_cache.get(&input_hash) {
            self.is_cache_entry_valid(cached_plan)
        } else {
            false
        };

        if is_valid {
            // Get the cached plan and update statistics
            if let Some(cached_plan) = self.plan_cache.get_mut(&input_hash) {
                cached_plan.metadata.replay_count += 1;
                cached_plan.metadata.last_replayed = Some(Utc::now());
                self.session_info.cache_hits += 1;

                if self.config.log_replay_operations {
                    info!("Cache hit for intent: {} (replay count: {})", 
                          intent.id, cached_plan.metadata.replay_count);
                }

                return Ok(Some(cached_plan.plan.clone()));
            }
        } else if self.plan_cache.contains_key(&input_hash) {
            // Remove expired cache entry
            self.plan_cache.remove(&input_hash);
            debug!("Removed expired cache entry for hash: {}", input_hash);
        }

        debug!("No valid cached plan found for hash: {}", input_hash);
        Ok(None)
    }

    /// Cache a plan for future replay
    pub async fn cache_plan(&mut self, intent: &Intent, plan: &ExecutionPlan) -> Result<(), PlanningError> {
        if !self.deterministic_mode {
            return Ok(());
        }

        let input_hash = self.calculate_input_hash(intent)?;
        
        // Check cache size limit
        if self.plan_cache.len() >= self.config.max_cached_plans {
            self.evict_oldest_cache_entry();
        }

        let cached_plan = CachedPlan {
            plan: plan.clone(),
            original_intent: intent.clone(),
            cached_at: Utc::now(),
            metadata: ReplayMetadata {
                session_id: self.session_info.session_id.clone(),
                user_context: intent.context.clone(),
                planning_config: "default".to_string(), // Would serialize actual config
                replay_count: 0,
                last_replayed: None,
            },
            input_hash: input_hash.clone(),
        };

        self.plan_cache.insert(input_hash.clone(), cached_plan);
        self.session_info.plans_generated += 1;

        if self.config.log_replay_operations {
            info!("Cached plan for intent: {} with hash: {}", intent.id, input_hash);
        }

        debug!("Plan cached successfully. Cache size: {}", self.plan_cache.len());
        Ok(())
    }

    /// Calculate deterministic hash for an intent
    fn calculate_input_hash(&self, intent: &Intent) -> Result<String, PlanningError> {
        if self.config.strict_determinism {
            // Strict mode: hash the exact input and context
            self.calculate_strict_hash(intent)
        } else {
            // Relaxed mode: hash normalized input
            self.calculate_normalized_hash(intent)
        }
    }

    /// Calculate strict hash (exact matching)
    fn calculate_strict_hash(&self, intent: &Intent) -> Result<String, PlanningError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Hash the raw input exactly
        intent.raw_input.hash(&mut hasher);
        
        // Hash the action type
        format!("{:?}", intent.action).hash(&mut hasher);
        
        // Hash parameters in a deterministic order
        let mut param_keys: Vec<_> = intent.parameters.keys().collect();
        param_keys.sort();
        for key in param_keys {
            key.hash(&mut hasher);
            if let Ok(value_str) = serde_json::to_string(&intent.parameters[key]) {
                value_str.hash(&mut hasher);
            }
        }
        
        // Hash targets
        for target in &intent.targets {
            format!("{:?}", target.target_type).hash(&mut hasher);
            target.identifier.hash(&mut hasher);
        }
        
        // Hash relevant context
        intent.context.working_directory.hash(&mut hasher);
        intent.context.session_id.hash(&mut hasher);

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Calculate normalized hash (semantic matching)
    fn calculate_normalized_hash(&self, intent: &Intent) -> Result<String, PlanningError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Normalize and hash the input
        let normalized_input = self.normalize_input(&intent.raw_input);
        normalized_input.hash(&mut hasher);
        
        // Hash the action type
        format!("{:?}", intent.action).hash(&mut hasher);
        
        // Hash normalized parameters
        let mut param_keys: Vec<_> = intent.parameters.keys().collect();
        param_keys.sort();
        for key in param_keys {
            let normalized_key = key.to_lowercase().trim().to_string();
            normalized_key.hash(&mut hasher);
            if let Ok(value_str) = serde_json::to_string(&intent.parameters[key]) {
                value_str.to_lowercase().hash(&mut hasher);
            }
        }
        
        // Hash target types (ignore specific identifiers in relaxed mode)
        for target in &intent.targets {
            format!("{:?}", target.target_type).hash(&mut hasher);
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Normalize input for semantic matching
    fn normalize_input(&self, input: &str) -> String {
        input
            .to_lowercase()
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if cache entry is still valid
    fn is_cache_entry_valid(&self, cached_plan: &CachedPlan) -> bool {
        let now = Utc::now();
        let expiration_duration = chrono::Duration::seconds(self.config.cache_expiration_secs as i64);
        
        now.signed_duration_since(cached_plan.cached_at) < expiration_duration
    }

    /// Evict oldest cache entry to make room
    fn evict_oldest_cache_entry(&mut self) {
        if let Some((oldest_key, _)) = self.plan_cache
            .iter()
            .min_by_key(|(_, cached_plan)| cached_plan.cached_at)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.plan_cache.remove(&oldest_key);
            debug!("Evicted oldest cache entry: {}", oldest_key);
        }
    }

    /// Clear all cached plans
    pub fn clear_cache(&mut self) {
        let cache_size = self.plan_cache.len();
        self.plan_cache.clear();
        
        if self.config.log_replay_operations {
            info!("Cleared replay cache ({} entries)", cache_size);
        }
    }

    /// Get replay statistics
    pub fn get_statistics(&self) -> ReplayStatistics {
        let cache_hit_rate = if self.session_info.plans_generated > 0 {
            self.session_info.cache_hits as f32 / self.session_info.plans_generated as f32
        } else {
            0.0
        };

        // Estimate memory usage (rough calculation)
        let estimated_memory_per_plan = 1024; // 1KB per plan (rough estimate)
        let cache_memory_usage = self.plan_cache.len() as u64 * estimated_memory_per_plan;

        ReplayStatistics {
            total_cached_plans: self.plan_cache.len(),
            cache_hit_rate,
            avg_generation_time: std::time::Duration::from_millis(100), // Placeholder
            cache_memory_usage,
        }
    }

    /// Export cached plans for analysis
    pub async fn export_cache(&self) -> Result<Vec<CachedPlan>, PlanningError> {
        Ok(self.plan_cache.values().cloned().collect())
    }

    /// Import cached plans from previous session
    pub async fn import_cache(&mut self, cached_plans: Vec<CachedPlan>) -> Result<(), PlanningError> {
        for cached_plan in cached_plans {
            if self.is_cache_entry_valid(&cached_plan) {
                self.plan_cache.insert(cached_plan.input_hash.clone(), cached_plan);
            }
        }
        
        if self.config.log_replay_operations {
            info!("Imported {} cached plans", self.plan_cache.len());
        }
        
        Ok(())
    }

    /// Get session information
    pub fn get_session_info(&self) -> &ReplaySession {
        &self.session_info
    }

    /// Start new replay session
    pub fn start_new_session(&mut self) {
        let old_session_id = self.session_info.session_id.clone();
        
        self.session_info = ReplaySession {
            session_id: Uuid::new_v4().to_string(),
            start_time: Utc::now(),
            plans_generated: 0,
            cache_hits: 0,
            config: self.config.clone(),
        };

        if self.config.log_replay_operations {
            info!("Started new replay session: {} (previous: {})", 
                  self.session_info.session_id, old_session_id);
        }
    }

    /// Validate cache integrity
    pub async fn validate_cache_integrity(&self) -> Result<Vec<String>, PlanningError> {
        let mut issues = Vec::new();

        for (hash, cached_plan) in &self.plan_cache {
            // Verify hash matches
            let recalculated_hash = self.calculate_input_hash(&cached_plan.original_intent)?;
            if recalculated_hash != *hash {
                issues.push(format!("Hash mismatch for cached plan: {} != {}", hash, recalculated_hash));
            }

            // Verify plan integrity
            if cached_plan.plan.steps.is_empty() {
                issues.push(format!("Cached plan {} has no steps", hash));
            }

            // Verify metadata consistency
            if cached_plan.metadata.replay_count > 1000 {
                issues.push(format!("Cached plan {} has unusually high replay count: {}", 
                                   hash, cached_plan.metadata.replay_count));
            }
        }

        Ok(issues)
    }

    /// Optimize cache by removing least used entries
    pub fn optimize_cache(&mut self) {
        let target_size = self.config.max_cached_plans * 3 / 4; // Keep 75% of max size
        
        if self.plan_cache.len() <= target_size {
            return;
        }

        // Sort by usage (replay count and last accessed time)
        let mut entries: Vec<_> = self.plan_cache.iter()
            .map(|(hash, cached_plan)| (hash.clone(), self.calculate_usage_score(cached_plan)))
            .collect();
        
        entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Remove least used entries
        let to_remove = self.plan_cache.len() - target_size;
        for (hash, _) in entries.iter().take(to_remove) {
            self.plan_cache.remove(hash);
        }

        if self.config.log_replay_operations {
            info!("Optimized cache: removed {} entries, {} remaining", 
                  to_remove, self.plan_cache.len());
        }
    }

    /// Calculate usage score for cache optimization
    fn calculate_usage_score(&self, cached_plan: &CachedPlan) -> f32 {
        let now = Utc::now();
        let age_hours = now.signed_duration_since(cached_plan.cached_at).num_hours() as f32;
        let replay_count = cached_plan.metadata.replay_count as f32;
        
        // Score based on usage frequency and recency
        let recency_score = 1.0 / (1.0 + age_hours / 24.0); // Decay over days
        let usage_score = replay_count.ln_1p(); // Logarithmic scaling of usage
        
        recency_score * usage_score
    }
}

impl Default for ReplayController {
    fn default() -> Self {
        Self::new()
    }
}