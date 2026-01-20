//! # Gate C Hard Limits
//!
//! Hard limits and constants for Gate C operations to prevent DoS and ensure bounded behavior.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

/// Maximum number of steps in a plan
pub const MAX_PLAN_STEPS: usize = 1024;

/// Maximum number of steps in a pipeline
pub const MAX_PIPELINE_STEPS: usize = 128;

/// Maximum number of nodes in REPL rendering
pub const MAX_RENDER_NODES: usize = 256;

/// Maximum number of edges in dependency graph
pub const MAX_DEP_GRAPH_EDGES: usize = 4096;

/// Maximum output size for security inspection (64KB)
pub const MAX_INSPECT_OUTPUT_BYTES: usize = 64_000;

/// Maximum depth for dependency analysis
pub const MAX_DEPENDENCY_DEPTH: usize = 32;

/// Maximum number of mutation intents per plan
pub const MAX_MUTATION_INTENTS: usize = 64;

/// Maximum size of plan metadata (16KB)
pub const MAX_PLAN_METADATA_BYTES: usize = 16_384;

/// Maximum number of data references per step
pub const MAX_DATA_REFS_PER_STEP: usize = 32;

/// Maximum length of resource path segments
pub const MAX_RESOURCE_PATH_SEGMENTS: usize = 16;

/// Maximum size of content specification (1MB)
pub const MAX_CONTENT_SPEC_BYTES: usize = 1_048_576;

/// Maximum number of changes in a change set
pub const MAX_CHANGESET_CHANGES: usize = 128;

/// Maximum size of explanation output (32KB)
pub const MAX_EXPLANATION_BYTES: usize = 32_768;

/// Maximum number of parallelizable groups
pub const MAX_PARALLELIZABLE_GROUPS: usize = 64;

/// Maximum number of ordering hints
pub const MAX_ORDERING_HINTS: usize = 512;

/// Maximum number of semantic hints per analysis
pub const MAX_SEMANTIC_HINTS: usize = 1024;

/// Timeout for plan analysis operations (milliseconds)
pub const PLAN_ANALYSIS_TIMEOUT_MS: u64 = 5000;

/// Timeout for normalization operations (milliseconds)
pub const NORMALIZATION_TIMEOUT_MS: u64 = 3000;

/// Timeout for security inspection (milliseconds)
pub const SECURITY_INSPECTION_TIMEOUT_MS: u64 = 2000;

/// Timeout for REPL rendering (milliseconds)
pub const REPL_RENDERING_TIMEOUT_MS: u64 = 1000;

/// Limits configuration for runtime enforcement
#[derive(Debug, Clone, PartialEq)]
pub struct GateCLimits {
    /// Maximum plan steps
    pub max_plan_steps: usize,
    /// Maximum pipeline steps
    pub max_pipeline_steps: usize,
    /// Maximum render nodes
    pub max_render_nodes: usize,
    /// Maximum dependency graph edges
    pub max_dep_graph_edges: usize,
    /// Maximum inspection output bytes
    pub max_inspect_output_bytes: usize,
    /// Maximum dependency depth
    pub max_dependency_depth: usize,
    /// Maximum mutation intents
    pub max_mutation_intents: usize,
    /// Maximum plan metadata bytes
    pub max_plan_metadata_bytes: usize,
}

impl Default for GateCLimits {
    fn default() -> Self {
        Self {
            max_plan_steps: MAX_PLAN_STEPS,
            max_pipeline_steps: MAX_PIPELINE_STEPS,
            max_render_nodes: MAX_RENDER_NODES,
            max_dep_graph_edges: MAX_DEP_GRAPH_EDGES,
            max_inspect_output_bytes: MAX_INSPECT_OUTPUT_BYTES,
            max_dependency_depth: MAX_DEPENDENCY_DEPTH,
            max_mutation_intents: MAX_MUTATION_INTENTS,
            max_plan_metadata_bytes: MAX_PLAN_METADATA_BYTES,
        }
    }
}

impl GateCLimits {
    /// Create new limits configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create limits configuration for testing (reduced limits)
    pub fn for_testing() -> Self {
        Self {
            max_plan_steps: 16,
            max_pipeline_steps: 8,
            max_render_nodes: 32,
            max_dep_graph_edges: 64,
            max_inspect_output_bytes: 1024,
            max_dependency_depth: 4,
            max_mutation_intents: 8,
            max_plan_metadata_bytes: 512,
        }
    }
    
    /// Validate plan against limits
    pub fn validate_plan_size(&self, steps: usize) -> Result<(), String> {
        if steps > self.max_plan_steps {
            return Err(format!(
                "Plan has {} steps, exceeds limit of {}",
                steps, self.max_plan_steps
            ));
        }
        Ok(())
    }
    
    /// Validate pipeline against limits
    pub fn validate_pipeline_size(&self, steps: usize) -> Result<(), String> {
        if steps > self.max_pipeline_steps {
            return Err(format!(
                "Pipeline has {} steps, exceeds limit of {}",
                steps, self.max_pipeline_steps
            ));
        }
        Ok(())
    }
    
    /// Validate dependency graph against limits
    pub fn validate_dependency_graph(&self, edges: usize) -> Result<(), String> {
        if edges > self.max_dep_graph_edges {
            return Err(format!(
                "Dependency graph has {} edges, exceeds limit of {}",
                edges, self.max_dep_graph_edges
            ));
        }
        Ok(())
    }
    
    /// Validate output size against limits
    pub fn validate_output_size(&self, size: usize) -> Result<(), String> {
        if size > self.max_inspect_output_bytes {
            return Err(format!(
                "Output size {} bytes exceeds limit of {} bytes",
                size, self.max_inspect_output_bytes
            ));
        }
        Ok(())
    }
    
    /// Validate render nodes against limits
    pub fn validate_render_nodes(&self, nodes: usize) -> Result<(), String> {
        if nodes > self.max_render_nodes {
            return Err(format!(
                "Render has {} nodes, exceeds limit of {}",
                nodes, self.max_render_nodes
            ));
        }
        Ok(())
    }
}

/// Complexity metrics for plans
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexityMetrics {
    /// Number of steps
    pub steps: usize,
    /// Number of dependencies
    pub dependencies: usize,
    /// Number of data references
    pub data_refs: usize,
    /// Dependency depth
    pub dependency_depth: usize,
    /// Number of mutation intents
    pub mutation_intents: usize,
}

impl ComplexityMetrics {
    /// Calculate complexity score (0-100)
    pub fn complexity_score(&self) -> u8 {
        let step_score = (self.steps * 100 / MAX_PLAN_STEPS).min(100);
        let dep_score = (self.dependencies * 100 / MAX_DEP_GRAPH_EDGES).min(100);
        let depth_score = (self.dependency_depth * 100 / MAX_DEPENDENCY_DEPTH).min(100);
        
        ((step_score + dep_score + depth_score) / 3).min(100) as u8
    }
    
    /// Check if complexity is within acceptable bounds
    pub fn is_acceptable(&self, limits: &GateCLimits) -> bool {
        self.steps <= limits.max_plan_steps
            && self.dependencies <= limits.max_dep_graph_edges
            && self.dependency_depth <= limits.max_dependency_depth
            && self.mutation_intents <= limits.max_mutation_intents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = GateCLimits::default();
        assert_eq!(limits.max_plan_steps, MAX_PLAN_STEPS);
        assert_eq!(limits.max_pipeline_steps, MAX_PIPELINE_STEPS);
        assert_eq!(limits.max_render_nodes, MAX_RENDER_NODES);
    }

    #[test]
    fn test_testing_limits() {
        let limits = GateCLimits::for_testing();
        assert!(limits.max_plan_steps < MAX_PLAN_STEPS);
        assert!(limits.max_pipeline_steps < MAX_PIPELINE_STEPS);
    }

    #[test]
    fn test_plan_validation() {
        let limits = GateCLimits::default();
        
        // Valid plan
        assert!(limits.validate_plan_size(100).is_ok());
        
        // Invalid plan (too large)
        assert!(limits.validate_plan_size(2000).is_err());
    }

    #[test]
    fn test_pipeline_validation() {
        let limits = GateCLimits::default();
        
        // Valid pipeline
        assert!(limits.validate_pipeline_size(50).is_ok());
        
        // Invalid pipeline (too large)
        assert!(limits.validate_pipeline_size(200).is_err());
    }

    #[test]
    fn test_complexity_score() {
        let metrics = ComplexityMetrics {
            steps: 512,  // 50% of MAX_PLAN_STEPS
            dependencies: 2048,  // 50% of MAX_DEP_GRAPH_EDGES
            data_refs: 100,
            dependency_depth: 16,  // 50% of MAX_DEPENDENCY_DEPTH
            mutation_intents: 32,
        };
        
        let score = metrics.complexity_score();
        assert_eq!(score, 50);  // Average of 50%, 50%, 50%
    }

    #[test]
    fn test_complexity_acceptability() {
        let limits = GateCLimits::default();
        
        let acceptable_metrics = ComplexityMetrics {
            steps: 100,
            dependencies: 100,
            data_refs: 50,
            dependency_depth: 10,
            mutation_intents: 10,
        };
        assert!(acceptable_metrics.is_acceptable(&limits));
        
        let unacceptable_metrics = ComplexityMetrics {
            steps: 2000,  // Exceeds limit
            dependencies: 100,
            data_refs: 50,
            dependency_depth: 10,
            mutation_intents: 10,
        };
        assert!(!unacceptable_metrics.is_acceptable(&limits));
    }
}