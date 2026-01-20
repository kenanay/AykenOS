//! # Gate C Core Types
//!
//! Core type definitions for Gate C submission bridge functionality.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a submitted plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionId {
    /// Internal identifier
    pub id: String,
    /// Submission timestamp (for audit)
    pub timestamp: u64,
    /// Optional fingerprint for determinism validation
    pub fingerprint: Option<String>,
}

impl fmt::Display for SubmissionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "submission-{}", self.id)
    }
}

/// Execution plan that can be submitted to orchestrator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Plan identifier
    pub id: String,
    /// Plan steps
    pub steps: Vec<PlanStep>,
    /// Plan metadata
    pub metadata: PlanMetadata,
    /// Dependencies between steps
    pub dependencies: Vec<Dependency>,
}

/// Individual step in an execution plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step identifier
    pub id: String,
    /// Step operation
    pub operation: Operation,
    /// Input data references
    pub inputs: Vec<DataRef>,
    /// Output data references
    pub outputs: Vec<DataRef>,
}

/// Operation that can be performed in a plan step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    /// Query operation
    Query {
        target: String,
        parameters: HashMap<String, String>,
    },
    /// Mutation operation (becomes MutationIntent)
    Mutation {
        intent: MutationIntent,
    },
    /// Computation operation
    Compute {
        function: String,
        arguments: Vec<String>,
    },
}

/// Mutation intent (no actual deletion, only invalidation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MutationIntent {
    /// Invalidate a resource (replaces delete operations)
    InvalidateIntent {
        target: ResourcePath,
        reason: InvalidationReason,
    },
    /// Update a resource
    UpdateIntent {
        target: ResourcePath,
        changes: ChangeSet,
    },
    /// Create a new resource
    CreateIntent {
        path: ResourcePath,
        content: ContentSpec,
    },
}

/// Resource path for mutations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourcePath {
    /// Path segments
    pub segments: Vec<String>,
}

impl fmt::Display for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.segments.join("/"))
    }
}

/// Reason for invalidation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InvalidationReason {
    /// Resource is obsolete
    Obsolete,
    /// Resource conflicts with new data
    Conflict,
    /// Resource violates constraints
    ConstraintViolation,
    /// Custom reason
    Custom(String),
}

/// Set of changes for update operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeSet {
    /// Fields to update
    pub updates: HashMap<String, String>,
    /// Fields to remove
    pub removals: Vec<String>,
}

/// Content specification for create operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentSpec {
    /// Content type
    pub content_type: String,
    /// Content data
    pub data: Vec<u8>,
    /// Content metadata
    pub metadata: HashMap<String, String>,
}

/// Data reference in plan steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataRef {
    /// Reference identifier
    pub id: String,
    /// Data type
    pub data_type: String,
    /// Optional source step
    pub source_step: Option<String>,
}

/// Dependency between plan steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// Source step ID
    pub from: String,
    /// Target step ID
    pub to: String,
    /// Dependency type
    pub dependency_type: DependencyType,
}

/// Type of dependency between steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Data dependency (output → input)
    Data,
    /// Control dependency (execution order)
    Control,
    /// Resource dependency (shared resource access)
    Resource,
}

/// Plan metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanMetadata {
    /// Plan name
    pub name: String,
    /// Plan description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Plan version
    pub version: String,
    /// Additional metadata
    pub extra: HashMap<String, String>,
}

/// Pipeline plan for complex workflows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelinePlan {
    /// Pipeline steps
    pub steps: Vec<PipelineStep>,
    /// Dependency graph
    pub dependencies: DependencyGraph,
    /// Pipeline metadata
    pub metadata: PipelineMetadata,
}

/// Individual step in a pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Step identifier
    pub id: StepId,
    /// Step operation
    pub operation: Operation,
    /// Input data references
    pub inputs: Vec<DataRef>,
    /// Output data references
    pub outputs: Vec<DataRef>,
}

/// Step identifier
pub type StepId = String;

/// Dependency graph for pipeline analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Graph nodes (step IDs)
    pub nodes: Vec<StepId>,
    /// Graph edges (dependencies)
    pub edges: Vec<(StepId, StepId)>,
}

/// Pipeline metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineMetadata {
    /// Pipeline name
    pub name: String,
    /// Pipeline description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
}

/// Canonical plan representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPlan {
    /// Plan fingerprint for deterministic comparison
    pub fingerprint: PlanFingerprint,
    /// Normalized steps
    pub normalized_steps: Vec<CanonicalStep>,
    /// Canonical metadata
    pub metadata: CanonicalMetadata,
}

impl CanonicalPlan {
    /// Get plan fingerprint
    pub fn fingerprint(&self) -> &PlanFingerprint {
        &self.fingerprint
    }
    
    /// Convert to canonical byte representation for deterministic comparison
    /// 
    /// **CONSTITUTIONAL GUARANTEE:** This method produces byte-identical output
    /// for semantically identical plans across all runs, platforms, and Rust versions.
    /// 
    /// **CRITICAL:** This is the ONLY valid method for snapshot testing and
    /// deterministic comparison. Never use Debug, Display, or format! for
    /// deterministic operations.
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        
        
        let mut canonical_repr = Vec::new();
        
        // 1. Fingerprint (deterministic)
        canonical_repr.extend_from_slice(&self.fingerprint.hash.to_le_bytes());
        canonical_repr.extend_from_slice(&self.fingerprint.version.to_le_bytes());
        
        // 2. Metadata (canonical order)
        canonical_repr.extend_from_slice(self.metadata.name.as_bytes());
        canonical_repr.push(0); // null separator
        canonical_repr.extend_from_slice(&self.metadata.canonicalized_at.to_le_bytes());
        canonical_repr.extend_from_slice(self.metadata.version.as_bytes());
        canonical_repr.push(0); // null separator
        
        // 3. Steps (already in canonical order)
        canonical_repr.extend_from_slice(&(self.normalized_steps.len() as u32).to_le_bytes());
        for step in &self.normalized_steps {
            // Step ID
            canonical_repr.extend_from_slice(step.id.as_bytes());
            canonical_repr.push(0);
            
            // Operation (canonical serialization)
            match &step.operation {
                Operation::Query { target, parameters } => {
                    canonical_repr.push(b'Q'); // Query marker
                    canonical_repr.extend_from_slice(target.as_bytes());
                    canonical_repr.push(0);
                    
                    // Parameters SORTED by key for determinism
                    let mut param_keys: Vec<_> = parameters.keys().collect();
                    param_keys.sort();
                    canonical_repr.extend_from_slice(&(param_keys.len() as u32).to_le_bytes());
                    for key in param_keys {
                        canonical_repr.extend_from_slice(key.as_bytes());
                        canonical_repr.push(b'=');
                        canonical_repr.extend_from_slice(parameters[key].as_bytes());
                        canonical_repr.push(0);
                    }
                }
                Operation::Compute { function, arguments } => {
                    canonical_repr.push(b'C'); // Compute marker
                    canonical_repr.extend_from_slice(function.as_bytes());
                    canonical_repr.push(0);
                    
                    // Arguments in order
                    canonical_repr.extend_from_slice(&(arguments.len() as u32).to_le_bytes());
                    for arg in arguments {
                        canonical_repr.extend_from_slice(arg.as_bytes());
                        canonical_repr.push(0);
                    }
                }
                Operation::Mutation { intent } => {
                    canonical_repr.push(b'M'); // Mutation marker
                    // Serialize mutation intent deterministically using canonical format
                    match intent {
                        MutationIntent::InvalidateIntent { target, reason } => {
                            canonical_repr.push(b'I'); // Invalidate marker
                            canonical_repr.extend_from_slice(target.to_string().as_bytes());
                            canonical_repr.push(b':');
                            match reason {
                                InvalidationReason::Obsolete => canonical_repr.push(b'O'),
                                InvalidationReason::Conflict => canonical_repr.push(b'C'),
                                InvalidationReason::ConstraintViolation => canonical_repr.push(b'V'),
                                InvalidationReason::Custom(s) => {
                                    canonical_repr.push(b'X');
                                    canonical_repr.extend_from_slice(s.as_bytes());
                                }
                            }
                        }
                        MutationIntent::UpdateIntent { target, changes } => {
                            canonical_repr.push(b'U'); // Update marker
                            canonical_repr.extend_from_slice(target.to_string().as_bytes());
                            canonical_repr.push(b':');
                            
                            // Serialize updates in sorted order for determinism
                            let mut update_keys: Vec<_> = changes.updates.keys().collect();
                            update_keys.sort();
                            canonical_repr.extend_from_slice(&(update_keys.len() as u32).to_le_bytes());
                            for key in update_keys {
                                canonical_repr.extend_from_slice(key.as_bytes());
                                canonical_repr.push(b'=');
                                canonical_repr.extend_from_slice(changes.updates[key].as_bytes());
                                canonical_repr.push(0);
                            }
                            
                            // Serialize removals in sorted order
                            let mut removals = changes.removals.clone();
                            removals.sort();
                            canonical_repr.extend_from_slice(&(removals.len() as u32).to_le_bytes());
                            for removal in removals {
                                canonical_repr.extend_from_slice(removal.as_bytes());
                                canonical_repr.push(0);
                            }
                        }
                        MutationIntent::CreateIntent { path, content } => {
                            canonical_repr.push(b'C'); // Create marker
                            canonical_repr.extend_from_slice(path.to_string().as_bytes());
                            canonical_repr.push(b':');
                            canonical_repr.extend_from_slice(content.content_type.as_bytes());
                            canonical_repr.push(b':');
                            canonical_repr.extend_from_slice(&content.data);
                            canonical_repr.push(0);
                        }
                    }
                }
            }
            
            // Inputs and outputs (already in canonical order)
            canonical_repr.extend_from_slice(&(step.inputs.len() as u32).to_le_bytes());
            for input in &step.inputs {
                canonical_repr.extend_from_slice(input.id.as_bytes());
                canonical_repr.push(b':');
                canonical_repr.extend_from_slice(input.data_type.as_bytes());
                canonical_repr.push(0);
            }
            
            canonical_repr.extend_from_slice(&(step.outputs.len() as u32).to_le_bytes());
            for output in &step.outputs {
                canonical_repr.extend_from_slice(output.id.as_bytes());
                canonical_repr.push(b':');
                canonical_repr.extend_from_slice(output.data_type.as_bytes());
                canonical_repr.push(0);
            }
        }
        
        canonical_repr
    }
    
    /// Convert to canonical string representation for human-readable comparison
    /// 
    /// **WARNING:** This is for debugging only. Use `to_canonical_bytes()` for
    /// deterministic comparison and snapshot testing.
    pub fn to_canonical_string(&self) -> String {
        let bytes = self.to_canonical_bytes();
        format!("CanonicalPlan[{}bytes:{}]", bytes.len(), 
                crate::gate_c::deterministic::deterministic_hash_fnv1a(&bytes))
    }
}

/// Plan fingerprint for deterministic comparison
/// 
/// **ARCHITECTURAL LOCK:** This fingerprint is used EXCLUSIVELY for:
/// - Audit trails and determinism validation
/// - Property-based testing consistency
/// - Cross-version behavior verification
/// 
/// **MUST NOT be used for:**
/// - Execution identity or cache keys
/// - Runtime behavior or optimization
/// - Any execution-related logic
/// 
/// The fingerprint is derived from canonical plan representation only
/// and provides deterministic comparison for audit and testing purposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanFingerprint {
    /// Hash value derived from canonical representation
    pub hash: u64,
    /// Version for compatibility tracking
    pub version: u32,
}

impl PlanFingerprint {
    /// Create fingerprint from canonical plan
    /// 
    /// **ARCHITECTURAL GUARANTEE:** This method produces deterministic fingerprints
    /// for audit and testing purposes only. The fingerprint MUST NOT be used for
    /// execution identity, caching, or runtime behavior.
    pub fn from_canonical_plan(_plan: &CanonicalPlan) -> Self {
        // Implementation would compute hash from canonical representation
        // This is a placeholder - actual implementation would use proper hashing
        // The hash is derived from the canonical structure, not execution semantics
        Self {
            hash: 0, // TODO: Implement proper hash computation from canonical representation
            version: 1,
        }
    }
}

/// Canonical step representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalStep {
    /// Step identifier
    pub id: String,
    /// Canonical operation
    pub operation: Operation,
    /// Canonical inputs
    pub inputs: Vec<DataRef>,
    /// Canonical outputs
    pub outputs: Vec<DataRef>,
}

/// Canonical metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalMetadata {
    /// Canonical name
    pub name: String,
    /// Canonical version
    pub version: String,
    /// Canonicalization timestamp
    pub canonicalized_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submission_id_display() {
        let id = SubmissionId {
            id: "test-123".to_string(),
            timestamp: 1234567890,
            fingerprint: None,
        };
        assert_eq!(format!("{}", id), "submission-test-123");
    }

    #[test]
    fn test_resource_path_display() {
        let path = ResourcePath {
            segments: vec!["users".to_string(), "123".to_string(), "profile".to_string()],
        };
        assert_eq!(format!("{}", path), "/users/123/profile");
    }

    #[test]
    fn test_plan_fingerprint_creation() {
        let plan = CanonicalPlan {
            fingerprint: PlanFingerprint { hash: 12345, version: 1 },
            normalized_steps: vec![],
            metadata: CanonicalMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                canonicalized_at: 1234567890,
            },
        };
        
        let fingerprint = PlanFingerprint::from_canonical_plan(&plan);
        assert_eq!(fingerprint.version, 1);
    }
}