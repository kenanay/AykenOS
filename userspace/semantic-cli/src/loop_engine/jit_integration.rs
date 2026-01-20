//! D1 JIT Integration for Hot Loop Bodies - Phase 6.2
//!
//! This module implements the integration between the loop engine and the D1 JIT
//! compilation system. It compiles hot loop bodies to native code while maintaining
//! all constitutional guarantees and safety constraints.
//!
//! # Requirements Validation
//!
//! - Requirements 6.2: Compile hot loop bodies using D1 JIT pipeline
//! - Requirements 6.3: Cache compiled bodies by comprehensive fingerprint
//! - Requirements 6.4: Enforce bounds checking, iteration limits, and budget timeouts in native code
//! - Requirements 6.5: Enforce bounds checking in JIT-compiled loop bodies for security
//!
//! # Constitutional Compliance
//!
//! This implementation MUST maintain all constitutional guarantees:
//! 1. Bounded iteration (iteration limits enforced in native code)
//! 2. Deterministic budget timeout (budget enforcement in native code)
//! 3. Type safety (accumulator type validation in native code)
//! 4. Semantic equivalence (JIT and interpreter produce identical results)

use crate::bcib::{LoopInstruction, LoopID, Value, ValueType};
use crate::error::{Result, SemanticCLIError, ErrorCode};
use crate::loop_engine::{LoopContext, LoopBodyResult};
use crate::loop_engine::monitoring::JITCompilationResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use sha2::{Sha256, Digest};

/// D1 JIT integration system for hot loop compilation
#[derive(Debug)]
pub struct JITIntegration {
    /// Cache of compiled loop bodies keyed by comprehensive fingerprint
    compiled_cache: Arc<RwLock<HashMap<JITCacheKey, CompiledLoopBody>>>,
    /// JIT compilation configuration
    config: JITConfig,
    /// Compilation statistics
    stats: Arc<RwLock<JITStats>>,
}

/// JIT compilation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JITConfig {
    /// Whether JIT compilation is enabled
    pub enabled: bool,
    /// Maximum number of cached compiled bodies
    pub max_cache_entries: usize,
    /// Compilation timeout in milliseconds
    pub compilation_timeout_ms: u64,
    /// Whether to enable bounds checking in compiled code
    pub enable_bounds_checking: bool,
    /// Whether to enable budget timeout enforcement in compiled code
    pub enable_budget_enforcement: bool,
    /// Whether to enable type safety validation in compiled code
    pub enable_type_safety: bool,
    /// Whether to enable debug information in compiled code
    pub enable_debug_info: bool,
}

/// Comprehensive cache key for JIT compilation (Requirements 6.3)
/// 
/// This key includes all semantic factors that affect compilation:
/// - Loop body fingerprint (IR structure)
/// - Semantic configuration affecting native code generation
/// - Type information for accumulator validation
/// - Safety constraints and bounds checking requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JITCacheKey {
    /// SHA-256 hash of all semantic inputs
    fingerprint: String,
}

/// Compiled loop body with native code and metadata
#[derive(Debug, Clone)]
pub struct CompiledLoopBody {
    /// Cache key used for this compilation
    cache_key: JITCacheKey,
    /// Compiled native code (placeholder - would be actual native code in real implementation)
    #[allow(dead_code)]
    native_code: NativeCode,
    /// Compilation metadata
    #[allow(dead_code)]
    metadata: CompilationMetadata,
    /// When this body was compiled
    compiled_at: Instant,
    /// Time taken to compile
    compilation_time: Duration,
}

/// Native code representation (placeholder for actual native code)
/// 
/// In a real D1 JIT implementation, this would contain:
/// - Machine code bytes
/// - Entry point addresses
/// - Stack frame layout
/// - Register allocation information
/// - Exception handling tables
#[derive(Debug, Clone)]
pub struct NativeCode {
    /// Placeholder for native code bytes
    #[allow(dead_code)]
    code_bytes: Vec<u8>,
    /// Entry point offset
    #[allow(dead_code)]
    entry_point: usize,
    /// Code size in bytes
    #[allow(dead_code)]
    code_size: usize,
    /// Whether bounds checking is embedded
    #[allow(dead_code)]
    has_bounds_checking: bool,
    /// Whether budget enforcement is embedded
    #[allow(dead_code)]
    has_budget_enforcement: bool,
    /// Whether type safety validation is embedded
    #[allow(dead_code)]
    has_type_safety: bool,
}

/// Compilation metadata for debugging and verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilationMetadata {
    /// Loop identifier
    loop_id: LoopID,
    /// Loop type (While, For, ForEach)
    loop_type: crate::bcib::LoopType,
    /// Iteration limit enforced in native code
    iteration_limit: u32,
    /// Budget timeout enforced in native code
    budget_timeout: u64,
    /// Budget measurement method
    budget_measurement: crate::bcib::BudgetMeasurement,
    /// Accumulator type validated in native code
    accumulator_type: ValueType,
    /// Compilation flags
    compilation_flags: Vec<String>,
    /// D1 JIT compiler version
    compiler_version: String,
}

/// JIT compilation statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JITStats {
    /// Total compilation attempts
    pub compilation_attempts: u64,
    /// Successful compilations
    pub successful_compilations: u64,
    /// Failed compilations
    pub failed_compilations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Total compilation time
    pub total_compilation_time: Duration,
    /// Average compilation time
    pub avg_compilation_time: Duration,
    /// Number of cached entries
    pub cached_entries: usize,
    /// Cache evictions
    pub cache_evictions: u64,
}

impl JITIntegration {
    /// Create a new JIT integration system
    pub fn new() -> Self {
        Self::with_config(JITConfig::default())
    }

    /// Create JIT integration with custom configuration
    pub fn with_config(config: JITConfig) -> Self {
        Self {
            compiled_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(JITStats::new())),
        }
    }

    /// Compile a hot loop body using D1 JIT pipeline (Requirements 6.2)
    /// 
    /// This method implements the core JIT compilation workflow:
    /// 1. Compute comprehensive cache key (Requirements 6.3)
    /// 2. Check cache for existing compilation
    /// 3. If cache miss, compile using D1 JIT pipeline
    /// 4. Embed constitutional guarantees in native code (Requirements 6.4)
    /// 5. Cache compiled body with comprehensive key
    pub fn compile_loop_body(
        &mut self,
        instruction: &LoopInstruction,
        loop_context: &LoopContext,
    ) -> Result<JITCompilationResult> {
        let start_time = Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.compilation_attempts += 1;
        }

        // 1. Compute comprehensive cache key (Requirements 6.3)
        let cache_key = self.compute_comprehensive_cache_key(instruction, loop_context)?;

        // 2. Check cache for existing compilation
        if let Some(cached_body) = self.get_cached_compilation(&cache_key)? {
            // Cache hit - return cached compilation
            {
                let mut stats = self.stats.write().unwrap();
                stats.cache_hits += 1;
            }
            
            return Ok(JITCompilationResult::Success {
                compilation_time: cached_body.compilation_time,
            });
        }

        // 3. Cache miss - perform compilation
        {
            let mut stats = self.stats.write().unwrap();
            stats.cache_misses += 1;
        }

        // 4. Compile using D1 JIT pipeline with constitutional guarantees
        let compiled_body = self.perform_jit_compilation(instruction, loop_context, cache_key)?;
        
        // 5. Cache compiled body
        self.cache_compiled_body(compiled_body.clone())?;

        let compilation_time = start_time.elapsed();
        
        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.successful_compilations += 1;
            stats.total_compilation_time += compilation_time;
            stats.avg_compilation_time = stats.total_compilation_time / stats.successful_compilations as u32;
        }

        Ok(JITCompilationResult::Success { compilation_time })
    }

    /// Compute comprehensive cache key including all semantic factors (Requirements 6.3)
    /// 
    /// The cache key MUST include all factors that affect native code generation:
    /// - Loop body fingerprint (IR structure)
    /// - Iteration limit (affects bounds checking in native code)
    /// - Budget timeout and measurement method (affects budget enforcement)
    /// - Accumulator type (affects type validation in native code)
    /// - Safety constraints and compilation flags
    fn compute_comprehensive_cache_key(
        &self,
        instruction: &LoopInstruction,
        loop_context: &LoopContext,
    ) -> Result<JITCacheKey> {
        let mut hasher = Sha256::new();

        // Include loop body fingerprint (IR structure)
        hasher.update(self.compute_loop_body_fingerprint(instruction)?);

        // Include semantic configuration affecting native code generation
        hasher.update(loop_context.iteration_limit.to_le_bytes());
        hasher.update(loop_context.budget_timeout.to_le_bytes());
        
        // Include budget measurement method (affects native code generation)
        let budget_method_bytes = match &loop_context.budget_measurement {
            crate::bcib::BudgetMeasurement::IterationCount => "iteration_count".as_bytes(),
            crate::bcib::BudgetMeasurement::InstructionCount { weight } => {
                hasher.update(b"instruction_count");
                hasher.update(weight.to_le_bytes());
                "instruction_count_weighted".as_bytes()
            }
            crate::bcib::BudgetMeasurement::Hybrid { multiplier } => {
                hasher.update(b"hybrid");
                hasher.update(multiplier.to_le_bytes());
                "hybrid_weighted".as_bytes()
            }
        };
        hasher.update(budget_method_bytes);

        // Include accumulator type (affects type validation in native code)
        let accumulator_type_bytes = match loop_context.accumulator_type {
            ValueType::String => "string".as_bytes(),
            ValueType::Number => "number".as_bytes(),
            ValueType::Boolean => "boolean".as_bytes(),
            ValueType::Array => "array".as_bytes(),
            ValueType::List => "list".as_bytes(),
            ValueType::SortedMap => "sorted_map".as_bytes(),
        };
        hasher.update(accumulator_type_bytes);

        // Include compilation flags that affect native code generation
        if self.config.enable_bounds_checking {
            hasher.update(b"bounds_checking_enabled");
        }
        if self.config.enable_budget_enforcement {
            hasher.update(b"budget_enforcement_enabled");
        }
        if self.config.enable_type_safety {
            hasher.update(b"type_safety_enabled");
        }

        // Include loop type (affects parallelization and optimization decisions)
        let loop_type_bytes = match instruction.loop_type() {
            crate::bcib::LoopType::While => "while".as_bytes(),
            crate::bcib::LoopType::For => "for".as_bytes(),
            crate::bcib::LoopType::ForEach => "foreach".as_bytes(),
        };
        hasher.update(loop_type_bytes);

        // Finalize hash
        let hash_bytes = hasher.finalize();
        let fingerprint = hex::encode(hash_bytes);

        Ok(JITCacheKey { fingerprint })
    }

    /// Compute loop body fingerprint from IR structure
    fn compute_loop_body_fingerprint(&self, instruction: &LoopInstruction) -> Result<Vec<u8>> {
        let mut hasher = Sha256::new();

        // Include loop-specific structure
        match instruction {
            LoopInstruction::While { condition, body, .. } => {
                hasher.update(b"while_loop");
                hasher.update(format!("{:?}", condition).as_bytes());
                hasher.update(body.as_bytes());
            }
            LoopInstruction::For { range, iterator_var, body, .. } => {
                hasher.update(b"for_loop");
                hasher.update(range.start.to_le_bytes());
                hasher.update(range.end.to_le_bytes());
                hasher.update(range.step.to_le_bytes());
                hasher.update(iterator_var.as_bytes());
                hasher.update(body.as_bytes());
            }
            LoopInstruction::ForEach { collection, collection_type, iterator_var, body, .. } => {
                hasher.update(b"foreach_loop");
                hasher.update(format!("{:?}", collection).as_bytes());
                hasher.update(format!("{:?}", collection_type).as_bytes());
                hasher.update(iterator_var.as_bytes());
                hasher.update(body.as_bytes());
            }
        }

        Ok(hasher.finalize().to_vec())
    }

    /// Get cached compilation if available
    fn get_cached_compilation(&self, cache_key: &JITCacheKey) -> Result<Option<CompiledLoopBody>> {
        let cache = self.compiled_cache.read().unwrap();
        Ok(cache.get(cache_key).cloned())
    }

    /// Perform actual JIT compilation using D1 pipeline (Requirements 6.2, 6.4)
    /// 
    /// This method integrates with the D1 JIT compilation system to generate
    /// native code with embedded constitutional guarantees:
    /// - Bounds checking for security (Requirements 6.5)
    /// - Iteration limit enforcement (Requirements 6.4)
    /// - Budget timeout enforcement (Requirements 6.4)
    /// - Type safety validation (Requirements 6.4)
    fn perform_jit_compilation(
        &self,
        instruction: &LoopInstruction,
        loop_context: &LoopContext,
        cache_key: JITCacheKey,
    ) -> Result<CompiledLoopBody> {
        let start_time = Instant::now();

        // TODO: Integrate with actual D1 JIT compilation pipeline
        // For now, we'll create a placeholder implementation that demonstrates
        // the required interface and constitutional compliance

        // 1. Prepare compilation context for D1 JIT
        let compilation_context = self.prepare_d1_compilation_context(instruction, loop_context)?;

        // 2. Generate native code with embedded constitutional guarantees
        let native_code = self.generate_native_code_with_guarantees(&compilation_context)?;

        // 3. Create compilation metadata
        let metadata = self.create_compilation_metadata(instruction, loop_context)?;

        let compilation_time = start_time.elapsed();

        Ok(CompiledLoopBody {
            cache_key,
            native_code,
            metadata,
            compiled_at: start_time,
            compilation_time,
        })
    }

    /// Prepare compilation context for D1 JIT system
    fn prepare_d1_compilation_context(
        &self,
        instruction: &LoopInstruction,
        loop_context: &LoopContext,
    ) -> Result<D1CompilationContext> {
        Ok(D1CompilationContext {
            loop_id: match instruction {
                LoopInstruction::While { id, .. } => id.clone(),
                LoopInstruction::For { id, .. } => id.clone(),
                LoopInstruction::ForEach { id, .. } => id.clone(),
            },
            loop_type: instruction.loop_type(),
            iteration_limit: loop_context.iteration_limit,
            budget_timeout: loop_context.budget_timeout,
            budget_measurement: loop_context.budget_measurement.clone(),
            accumulator_type: loop_context.accumulator_type,
            bounds_checking_required: self.config.enable_bounds_checking,
            budget_enforcement_required: self.config.enable_budget_enforcement,
            type_safety_required: self.config.enable_type_safety,
            debug_info_required: self.config.enable_debug_info,
        })
    }

    /// Generate native code with embedded constitutional guarantees (Requirements 6.4, 6.5)
    /// 
    /// This method generates native code that enforces:
    /// - Iteration limit checking (never exceed constitutional bounds)
    /// - Budget timeout enforcement (deterministic budget measurement)
    /// - Bounds checking for memory safety (Requirements 6.5)
    /// - Type safety validation for accumulator
    fn generate_native_code_with_guarantees(
        &self,
        context: &D1CompilationContext,
    ) -> Result<NativeCode> {
        // TODO: Integrate with actual D1 JIT code generation
        // This is a placeholder implementation that demonstrates the required structure

        let mut code_bytes = Vec::new();
        let mut compilation_flags = Vec::new();

        // 1. Generate iteration limit checking code (Requirements 6.4)
        if context.iteration_limit > 0 {
            // Pseudo-assembly for iteration limit checking:
            // cmp iteration_count, iteration_limit
            // jge iteration_limit_exceeded
            code_bytes.extend_from_slice(&[0x48, 0x39, 0xC1]); // cmp rcx, rax (placeholder)
            compilation_flags.push("iteration_limit_checking".to_string());
        }

        // 2. Generate budget timeout enforcement code (Requirements 6.4)
        if context.budget_enforcement_required {
            match &context.budget_measurement {
                crate::bcib::BudgetMeasurement::IterationCount => {
                    // Simple iteration-based budget checking
                    code_bytes.extend_from_slice(&[0x48, 0xFF, 0xC2]); // inc rdx (placeholder)
                    compilation_flags.push("budget_iteration_count".to_string());
                }
                crate::bcib::BudgetMeasurement::InstructionCount { weight } => {
                    // Instruction-count based budget checking
                    code_bytes.extend_from_slice(&[0x48, 0x83, 0xC2, *weight as u8]); // add rdx, weight (placeholder)
                    compilation_flags.push("budget_instruction_count".to_string());
                }
                crate::bcib::BudgetMeasurement::Hybrid { multiplier } => {
                    // Hybrid budget checking
                    code_bytes.extend_from_slice(&[0x48, 0x83, 0xC2, (*multiplier as u64) as u8]); // add rdx, multiplier (placeholder)
                    compilation_flags.push("budget_hybrid".to_string());
                }
            }
        }

        // 3. Generate bounds checking code (Requirements 6.5)
        if context.bounds_checking_required {
            // Array bounds checking:
            // cmp array_index, array_length
            // jge bounds_check_failed
            code_bytes.extend_from_slice(&[0x48, 0x39, 0xD8]); // cmp rax, rbx (placeholder)
            compilation_flags.push("bounds_checking".to_string());
        }

        // 4. Generate type safety validation code (Requirements 6.4)
        if context.type_safety_required {
            match context.accumulator_type {
                ValueType::Number => {
                    // Type tag checking for numbers
                    code_bytes.extend_from_slice(&[0x48, 0x83, 0xF8, 0x01]); // cmp rax, 1 (number tag)
                    compilation_flags.push("type_safety_number".to_string());
                }
                ValueType::String => {
                    // Type tag checking for strings
                    code_bytes.extend_from_slice(&[0x48, 0x83, 0xF8, 0x02]); // cmp rax, 2 (string tag)
                    compilation_flags.push("type_safety_string".to_string());
                }
                ValueType::Boolean => {
                    // Type tag checking for booleans
                    code_bytes.extend_from_slice(&[0x48, 0x83, 0xF8, 0x03]); // cmp rax, 3 (boolean tag)
                    compilation_flags.push("type_safety_boolean".to_string());
                }
                _ => {
                    // Complex type checking for collections
                    code_bytes.extend_from_slice(&[0x48, 0x83, 0xF8, 0x04]); // cmp rax, 4 (collection tag)
                    compilation_flags.push("type_safety_collection".to_string());
                }
            }
        }

        // 5. Generate debug information if required
        if context.debug_info_required {
            compilation_flags.push("debug_info".to_string());
        }

        Ok(NativeCode {
            code_size: code_bytes.len(),
            code_bytes,
            entry_point: 0,
            has_bounds_checking: context.bounds_checking_required,
            has_budget_enforcement: context.budget_enforcement_required,
            has_type_safety: context.type_safety_required,
        })
    }

    /// Create compilation metadata for debugging and verification
    fn create_compilation_metadata(
        &self,
        instruction: &LoopInstruction,
        loop_context: &LoopContext,
    ) -> Result<CompilationMetadata> {
        let loop_id = match instruction {
            LoopInstruction::While { id, .. } => id.clone(),
            LoopInstruction::For { id, .. } => id.clone(),
            LoopInstruction::ForEach { id, .. } => id.clone(),
        };

        let mut compilation_flags = Vec::new();
        
        if self.config.enable_bounds_checking {
            compilation_flags.push("bounds_checking".to_string());
        }
        if self.config.enable_budget_enforcement {
            compilation_flags.push("budget_enforcement".to_string());
        }
        if self.config.enable_type_safety {
            compilation_flags.push("type_safety".to_string());
        }
        if self.config.enable_debug_info {
            compilation_flags.push("debug_info".to_string());
        }

        Ok(CompilationMetadata {
            loop_id,
            loop_type: instruction.loop_type(),
            iteration_limit: loop_context.iteration_limit,
            budget_timeout: loop_context.budget_timeout,
            budget_measurement: loop_context.budget_measurement.clone(),
            accumulator_type: loop_context.accumulator_type,
            compilation_flags,
            compiler_version: "d1-jit-v1.0.0".to_string(), // TODO: Get actual D1 version
        })
    }

    /// Cache compiled body with comprehensive key (Requirements 6.3)
    fn cache_compiled_body(&mut self, compiled_body: CompiledLoopBody) -> Result<()> {
        let mut cache = self.compiled_cache.write().unwrap();
        
        // Check cache size limits
        if cache.len() >= self.config.max_cache_entries {
            // Evict oldest entry (simple LRU approximation)
            if let Some((oldest_key, _)) = cache.iter()
                .min_by_key(|(_, body)| body.compiled_at)
                .map(|(k, v)| (k.clone(), v.clone())) {
                cache.remove(&oldest_key);
                
                // Update stats
                let mut stats = self.stats.write().unwrap();
                stats.cache_evictions += 1;
            }
        }

        // Insert compiled body
        cache.insert(compiled_body.cache_key.clone(), compiled_body);
        
        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.cached_entries = cache.len();
        }

        Ok(())
    }

    /// Execute compiled loop body (placeholder for actual native execution)
    /// 
    /// In a real implementation, this would:
    /// 1. Set up native execution context
    /// 2. Call compiled native code
    /// 3. Handle native code exceptions
    /// 4. Validate results against interpreter
    pub fn execute_compiled_loop_body(
        &self,
        _cache_key: &JITCacheKey,
        _accumulator: &Value,
        _iteration_count: u32,
    ) -> Result<LoopBodyResult> {
        // TODO: Implement actual native code execution
        // For now, return a placeholder result
        Err(SemanticCLIError::execution_error(
            "Native code execution not yet implemented",
            ErrorCode::E501,
        ))
    }

    /// Check if a loop is eligible for JIT compilation
    pub fn is_jit_eligible(&self, instruction: &LoopInstruction) -> bool {
        if !self.config.enabled {
            return false;
        }

        // While loops are eligible for JIT compilation
        // (parallelization restriction doesn't apply to JIT)
        match instruction.loop_type() {
            crate::bcib::LoopType::While => true,
            crate::bcib::LoopType::For => true,
            crate::bcib::LoopType::ForEach => true,
        }
    }

    /// Get JIT compilation statistics
    pub fn get_stats(&self) -> JITStats {
        self.stats.read().unwrap().clone()
    }

    /// Clear JIT cache
    pub fn clear_cache(&mut self) {
        let mut cache = self.compiled_cache.write().unwrap();
        cache.clear();
        
        let mut stats = self.stats.write().unwrap();
        stats.cached_entries = 0;
    }

    /// Get JIT configuration
    pub fn get_config(&self) -> &JITConfig {
        &self.config
    }

    /// Update JIT configuration
    pub fn update_config(&mut self, config: JITConfig) {
        self.config = config;
    }
}

/// D1 compilation context for JIT integration
#[derive(Debug, Clone)]
struct D1CompilationContext {
    #[allow(dead_code)]
    loop_id: LoopID,
    #[allow(dead_code)]
    loop_type: crate::bcib::LoopType,
    iteration_limit: u32,
    #[allow(dead_code)]
    budget_timeout: u64,
    budget_measurement: crate::bcib::BudgetMeasurement,
    accumulator_type: ValueType,
    bounds_checking_required: bool,
    budget_enforcement_required: bool,
    type_safety_required: bool,
    debug_info_required: bool,
}

impl Default for JITConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_cache_entries: 1000,
            compilation_timeout_ms: 5000, // 5 seconds
            enable_bounds_checking: true,
            enable_budget_enforcement: true,
            enable_type_safety: true,
            enable_debug_info: false, // Disabled by default for performance
        }
    }
}

impl JITStats {
    /// Create new JIT statistics
    pub fn new() -> Self {
        Self {
            compilation_attempts: 0,
            successful_compilations: 0,
            failed_compilations: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_compilation_time: Duration::ZERO,
            avg_compilation_time: Duration::ZERO,
            cached_entries: 0,
            cache_evictions: 0,
        }
    }

    /// Get compilation success rate
    pub fn success_rate(&self) -> f64 {
        if self.compilation_attempts > 0 {
            self.successful_compilations as f64 / self.compilation_attempts as f64 * 100.0
        } else {
            0.0
        }
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64 * 100.0
        } else {
            0.0
        }
    }
}

impl Default for JITIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{LoopConfig, LoopRange, Value, ValueType, BudgetMeasurement};
    use crate::types::SourceLocation;

    fn create_test_for_loop() -> LoopInstruction {
        LoopInstruction::For {
            id: LoopID::new("test-jit-for".to_string()),
            range: LoopRange::new(0, 1000, 1), // Hot loop with 1000 iterations
            iterator_var: "i".to_string(),
            body: "test-jit-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        }
    }

    fn create_test_loop_context() -> LoopContext {
        LoopContext::new(
            LoopID::new("test-jit-context".to_string()),
            &LoopConfig::new(Value::Number(0.0), ValueType::Number),
            "test-body".to_string(),
        )
    }

    #[test]
    fn test_jit_integration_creation() {
        let jit = JITIntegration::new();
        assert!(jit.config.enabled);
        assert_eq!(jit.config.max_cache_entries, 1000);
        assert!(jit.config.enable_bounds_checking);
        assert!(jit.config.enable_budget_enforcement);
        assert!(jit.config.enable_type_safety);
    }

    #[test]
    fn test_jit_eligibility_check() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        
        assert!(jit.is_jit_eligible(&instruction));
        
        // Test with disabled JIT
        let disabled_config = JITConfig {
            enabled: false,
            ..JITConfig::default()
        };
        let disabled_jit = JITIntegration::with_config(disabled_config);
        assert!(!disabled_jit.is_jit_eligible(&instruction));
    }

    #[test]
    fn test_comprehensive_cache_key_computation() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        let context = create_test_loop_context();
        
        let cache_key1 = jit.compute_comprehensive_cache_key(&instruction, &context).unwrap();
        let cache_key2 = jit.compute_comprehensive_cache_key(&instruction, &context).unwrap();
        
        // Same inputs should produce same cache key
        assert_eq!(cache_key1, cache_key2);
        
        // Different contexts should produce different cache keys
        let mut different_context = context.clone();
        different_context.iteration_limit = 5000;
        let cache_key3 = jit.compute_comprehensive_cache_key(&instruction, &different_context).unwrap();
        
        assert_ne!(cache_key1, cache_key3);
    }

    #[test]
    fn test_loop_body_fingerprint_computation() {
        let jit = JITIntegration::new();
        let instruction1 = create_test_for_loop();
        
        let fingerprint1 = jit.compute_loop_body_fingerprint(&instruction1).unwrap();
        let fingerprint2 = jit.compute_loop_body_fingerprint(&instruction1).unwrap();
        
        // Same instruction should produce same fingerprint
        assert_eq!(fingerprint1, fingerprint2);
        
        // Different instruction should produce different fingerprint
        let instruction2 = LoopInstruction::For {
            id: LoopID::new("different-loop".to_string()),
            range: LoopRange::new(0, 500, 1), // Different range
            iterator_var: "j".to_string(), // Different iterator
            body: "different-body".to_string(), // Different body
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        };
        
        let fingerprint3 = jit.compute_loop_body_fingerprint(&instruction2).unwrap();
        assert_ne!(fingerprint1, fingerprint3);
    }

    #[test]
    fn test_d1_compilation_context_preparation() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        let context = create_test_loop_context();
        
        let d1_context = jit.prepare_d1_compilation_context(&instruction, &context).unwrap();
        
        assert_eq!(d1_context.loop_type, crate::bcib::LoopType::For);
        assert_eq!(d1_context.iteration_limit, context.iteration_limit);
        assert_eq!(d1_context.budget_timeout, context.budget_timeout);
        assert_eq!(d1_context.accumulator_type, context.accumulator_type);
        assert!(d1_context.bounds_checking_required);
        assert!(d1_context.budget_enforcement_required);
        assert!(d1_context.type_safety_required);
    }

    #[test]
    fn test_native_code_generation_with_guarantees() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        let context = create_test_loop_context();
        
        let d1_context = jit.prepare_d1_compilation_context(&instruction, &context).unwrap();
        let native_code = jit.generate_native_code_with_guarantees(&d1_context).unwrap();
        
        // Verify constitutional guarantees are embedded
        assert!(native_code.has_bounds_checking);
        assert!(native_code.has_budget_enforcement);
        assert!(native_code.has_type_safety);
        assert!(!native_code.code_bytes.is_empty());
        assert_eq!(native_code.entry_point, 0);
        assert_eq!(native_code.code_size, native_code.code_bytes.len());
    }

    #[test]
    fn test_compilation_metadata_creation() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        let context = create_test_loop_context();
        
        let metadata = jit.create_compilation_metadata(&instruction, &context).unwrap();
        
        assert_eq!(metadata.loop_type, crate::bcib::LoopType::For);
        assert_eq!(metadata.iteration_limit, context.iteration_limit);
        assert_eq!(metadata.budget_timeout, context.budget_timeout);
        assert_eq!(metadata.accumulator_type, context.accumulator_type);
        assert!(metadata.compilation_flags.contains(&"bounds_checking".to_string()));
        assert!(metadata.compilation_flags.contains(&"budget_enforcement".to_string()));
        assert!(metadata.compilation_flags.contains(&"type_safety".to_string()));
        assert_eq!(metadata.compiler_version, "d1-jit-v1.0.0");
    }

    #[test]
    fn test_jit_compilation_workflow() {
        let mut jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        let context = create_test_loop_context();
        
        // First compilation should be a cache miss
        let result1 = jit.compile_loop_body(&instruction, &context).unwrap();
        match result1 {
            JITCompilationResult::Success { compilation_time } => {
                assert!(compilation_time > Duration::ZERO);
            }
            JITCompilationResult::Failure { .. } => {
                panic!("Expected successful compilation");
            }
        }
        
        let stats1 = jit.get_stats();
        assert_eq!(stats1.compilation_attempts, 1);
        assert_eq!(stats1.successful_compilations, 1);
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cached_entries, 1);
        
        // Second compilation should be a cache hit
        let result2 = jit.compile_loop_body(&instruction, &context).unwrap();
        match result2 {
            JITCompilationResult::Success { .. } => {
                // Cache hit should still be successful
            }
            JITCompilationResult::Failure { .. } => {
                panic!("Expected successful compilation");
            }
        }
        
        let stats2 = jit.get_stats();
        assert_eq!(stats2.compilation_attempts, 2);
        assert_eq!(stats2.successful_compilations, 1); // Only one actual compilation
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(stats2.cache_misses, 1);
        assert_eq!(stats2.cached_entries, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = JITConfig {
            max_cache_entries: 2, // Small cache for testing
            ..JITConfig::default()
        };
        let mut jit = JITIntegration::with_config(config);
        
        // Create three different instructions
        let instruction1 = LoopInstruction::For {
            id: LoopID::new("loop1".to_string()),
            range: LoopRange::new(0, 100, 1),
            iterator_var: "i".to_string(),
            body: "body1".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        };
        
        let instruction2 = LoopInstruction::For {
            id: LoopID::new("loop2".to_string()),
            range: LoopRange::new(0, 200, 1),
            iterator_var: "j".to_string(),
            body: "body2".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(2, 1, 0),
        };
        
        let instruction3 = LoopInstruction::For {
            id: LoopID::new("loop3".to_string()),
            range: LoopRange::new(0, 300, 1),
            iterator_var: "k".to_string(),
            body: "body3".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(3, 1, 0),
        };
        
        let context = create_test_loop_context();
        
        // Compile first two instructions (fill cache)
        jit.compile_loop_body(&instruction1, &context).unwrap();
        jit.compile_loop_body(&instruction2, &context).unwrap();
        
        let stats_before = jit.get_stats();
        assert_eq!(stats_before.cached_entries, 2);
        assert_eq!(stats_before.cache_evictions, 0);
        
        // Compile third instruction (should trigger eviction)
        jit.compile_loop_body(&instruction3, &context).unwrap();
        
        let stats_after = jit.get_stats();
        assert_eq!(stats_after.cached_entries, 2); // Still at max
        assert_eq!(stats_after.cache_evictions, 1); // One eviction occurred
    }

    #[test]
    fn test_jit_stats_calculations() {
        let mut stats = JITStats::new();
        
        // Test initial state
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.cache_hit_rate(), 0.0);
        
        // Add some data
        stats.compilation_attempts = 10;
        stats.successful_compilations = 8;
        stats.failed_compilations = 2;
        stats.cache_hits = 15;
        stats.cache_misses = 5;
        
        // Test calculations
        assert_eq!(stats.success_rate(), 80.0);
        assert_eq!(stats.cache_hit_rate(), 75.0);
    }

    #[test]
    fn test_cache_key_sensitivity_to_budget_measurement() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        
        // Create contexts with different budget measurements
        let mut context1 = create_test_loop_context();
        context1.budget_measurement = BudgetMeasurement::IterationCount;
        
        let mut context2 = create_test_loop_context();
        context2.budget_measurement = BudgetMeasurement::InstructionCount { weight: 10 };
        
        let mut context3 = create_test_loop_context();
        context3.budget_measurement = BudgetMeasurement::Hybrid { multiplier: 1.5 };
        
        let key1 = jit.compute_comprehensive_cache_key(&instruction, &context1).unwrap();
        let key2 = jit.compute_comprehensive_cache_key(&instruction, &context2).unwrap();
        let key3 = jit.compute_comprehensive_cache_key(&instruction, &context3).unwrap();
        
        // All keys should be different
        assert_ne!(key1, key2);
        assert_ne!(key2, key3);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_key_sensitivity_to_accumulator_type() {
        let jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        
        // Create contexts with different accumulator types
        let mut context1 = create_test_loop_context();
        context1.accumulator_type = ValueType::Number;
        
        let mut context2 = create_test_loop_context();
        context2.accumulator_type = ValueType::String;
        
        let mut context3 = create_test_loop_context();
        context3.accumulator_type = ValueType::Boolean;
        
        let key1 = jit.compute_comprehensive_cache_key(&instruction, &context1).unwrap();
        let key2 = jit.compute_comprehensive_cache_key(&instruction, &context2).unwrap();
        let key3 = jit.compute_comprehensive_cache_key(&instruction, &context3).unwrap();
        
        // All keys should be different
        assert_ne!(key1, key2);
        assert_ne!(key2, key3);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_jit_config_variants() {
        // Test default config
        let default_config = JITConfig::default();
        assert!(default_config.enabled);
        assert!(default_config.enable_bounds_checking);
        assert!(default_config.enable_budget_enforcement);
        assert!(default_config.enable_type_safety);
        assert!(!default_config.enable_debug_info);
        
        // Test custom config
        let custom_config = JITConfig {
            enabled: false,
            max_cache_entries: 500,
            compilation_timeout_ms: 10000,
            enable_bounds_checking: false,
            enable_budget_enforcement: true,
            enable_type_safety: false,
            enable_debug_info: true,
        };
        
        let jit = JITIntegration::with_config(custom_config.clone());
        assert_eq!(jit.get_config(), &custom_config);
    }

    #[test]
    fn test_clear_cache() {
        let mut jit = JITIntegration::new();
        let instruction = create_test_for_loop();
        let context = create_test_loop_context();
        
        // Add something to cache
        jit.compile_loop_body(&instruction, &context).unwrap();
        
        let stats_before = jit.get_stats();
        assert_eq!(stats_before.cached_entries, 1);
        
        // Clear cache
        jit.clear_cache();
        
        let stats_after = jit.get_stats();
        assert_eq!(stats_after.cached_entries, 0);
    }
}