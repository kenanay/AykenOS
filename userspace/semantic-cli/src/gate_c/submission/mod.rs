//! # Submission Bridge (Submit-Only)
//!
//! Submit execution plans to orchestrator without execution or waiting.
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use crate::gate_c::{
    error::{SubmissionError, GateCResult},
    types::{ExecutionPlan, SubmissionId},
};

/// Submission bridge trait for submitting plans to orchestrator
pub trait SubmissionBridge {
    /// Submit execution plan to orchestrator
    /// 
    /// Returns SubmissionId on success, SubmissionError on failure.
    /// This operation is idempotent - same plan produces same behavior.
    /// 
    /// Note: No wait_result, poll_status, or async completion methods.
    fn submit_plan(&self, plan: ExecutionPlan) -> GateCResult<SubmissionId>;
}

/// Implementation of submission bridge
pub struct SubmissionBridgeImpl {
    orchestrator_endpoint: OrchestratorEndpoint,
    capability_validator: CapabilityValidator,
}

impl SubmissionBridgeImpl {
    /// Create new submission bridge
    pub fn new(
        orchestrator_endpoint: OrchestratorEndpoint,
        capability_validator: CapabilityValidator,
    ) -> Self {
        Self {
            orchestrator_endpoint,
            capability_validator,
        }
    }
}

impl SubmissionBridge for SubmissionBridgeImpl {
    fn submit_plan(&self, plan: ExecutionPlan) -> GateCResult<SubmissionId> {
        // Validate capabilities first
        self.capability_validator.validate_plan(&plan)
            .map_err(|e| SubmissionError::CapabilityDenied(e.to_string()))?;
        
        // Submit to orchestrator
        self.orchestrator_endpoint.submit(plan)
            .map_err(|e| {
                let submission_error = match e {
                    OrchestratorError::Unavailable => SubmissionError::OrchestratorUnavailable,
                    OrchestratorError::InvalidPlan(msg) => SubmissionError::InvalidPlan(msg),
                    OrchestratorError::Network(msg) => SubmissionError::NetworkError(msg),
                    OrchestratorError::Serialization(msg) => SubmissionError::SerializationError(msg),
                };
                crate::gate_c::error::GateCError::Submission(submission_error)
            })
    }
}

/// Orchestrator endpoint for plan submission
pub struct OrchestratorEndpoint {
    endpoint_url: String,
    timeout_ms: u64,
}

impl OrchestratorEndpoint {
    /// Create new orchestrator endpoint
    pub fn new(endpoint_url: String, timeout_ms: u64) -> Self {
        Self {
            endpoint_url,
            timeout_ms,
        }
    }
    
    /// Submit plan to orchestrator
    pub fn submit(&self, plan: ExecutionPlan) -> Result<SubmissionId, OrchestratorError> {
        // TODO: Implement actual HTTP submission to orchestrator
        // This is a placeholder implementation
        
        // Validate plan is not empty
        if plan.steps.is_empty() {
            return Err(OrchestratorError::InvalidPlan("Plan has no steps".to_string()));
        }
        
        // Generate deterministic submission ID from plan content
        // DETERMINISM FIX: Use simple deterministic approach without Hash trait
        use crate::gate_c::deterministic::deterministic_id_from_plan;
        
        let submission_id = SubmissionId {
            id: deterministic_id_from_plan("plan", &plan.id),
            timestamp: 0, // DETERMINISTIC: No timestamp in submission ID
            fingerprint: Some(format!("plan-{}", plan.id)), // Simple fingerprint based on plan ID
        };
        
        Ok(submission_id)
    }
}

/// Capability validator for plan submission
pub struct CapabilityValidator {
    // TODO: Add capability checking logic
}

impl CapabilityValidator {
    /// Create new capability validator
    pub fn new() -> Self {
        Self {}
    }
    
    /// Validate plan against capabilities
    pub fn validate_plan(&self, _plan: &ExecutionPlan) -> Result<(), CapabilityError> {
        // TODO: Implement capability validation
        // For now, always allow
        Ok(())
    }
}

impl Default for CapabilityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrator communication errors
#[derive(Debug, Clone, PartialEq)]
pub enum OrchestratorError {
    /// Orchestrator is unavailable
    Unavailable,
    /// Invalid plan
    InvalidPlan(String),
    /// Network error
    Network(String),
    /// Serialization error
    Serialization(String),
}

/// Capability validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityError {
    /// Insufficient permissions
    InsufficientPermissions(String),
    /// Invalid capability
    InvalidCapability(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityError::InsufficientPermissions(msg) => {
                write!(f, "Insufficient permissions: {}", msg)
            }
            CapabilityError::InvalidCapability(msg) => {
                write!(f, "Invalid capability: {}", msg)
            }
        }
    }
}

impl std::error::Error for CapabilityError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_c::types::{PlanStep, PlanMetadata, Operation};
    use std::collections::HashMap;

    fn create_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "test-plan".to_string(),
            steps: vec![PlanStep {
                id: "step-1".to_string(),
                operation: Operation::Query {
                    target: "test".to_string(),
                    parameters: HashMap::new(),
                },
                inputs: vec![],
                outputs: vec![],
            }],
            metadata: PlanMetadata {
                name: "Test Plan".to_string(),
                description: Some("Test plan for unit tests".to_string()),
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        }
    }

    #[test]
    fn test_submission_bridge_creation() {
        let endpoint = OrchestratorEndpoint::new("http://localhost:8080".to_string(), 5000);
        let validator = CapabilityValidator::new();
        let bridge = SubmissionBridgeImpl::new(endpoint, validator);
        
        // Test that bridge was created successfully
        assert_eq!(bridge.orchestrator_endpoint.endpoint_url, "http://localhost:8080");
        assert_eq!(bridge.orchestrator_endpoint.timeout_ms, 5000);
    }

    #[test]
    fn test_plan_submission() {
        let endpoint = OrchestratorEndpoint::new("http://localhost:8080".to_string(), 5000);
        let validator = CapabilityValidator::new();
        let bridge = SubmissionBridgeImpl::new(endpoint, validator);
        
        let plan = create_test_plan();
        let result = bridge.submit_plan(plan);
        
        assert!(result.is_ok());
        let submission_id = result.unwrap();
        // Check that ID contains the plan prefix and hash
        assert!(submission_id.id.starts_with("plan_"));
        assert!(submission_id.id.len() > "plan_".len()); // Should have hash suffix
    }

    #[test]
    fn test_empty_plan_rejection() {
        let endpoint = OrchestratorEndpoint::new("http://localhost:8080".to_string(), 5000);
        
        let empty_plan = ExecutionPlan {
            id: "empty-plan".to_string(),
            steps: vec![], // Empty steps
            metadata: PlanMetadata {
                name: "Empty Plan".to_string(),
                description: None,
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };
        
        let result = endpoint.submit(empty_plan);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            OrchestratorError::InvalidPlan(msg) => {
                assert!(msg.contains("no steps"));
            }
            _ => panic!("Expected InvalidPlan error"),
        }
    }

    #[test]
    fn test_submission_determinism() {
        let endpoint = OrchestratorEndpoint::new("http://localhost:8080".to_string(), 5000);
        let validator = CapabilityValidator::new();
        let bridge = SubmissionBridgeImpl::new(endpoint, validator);
        
        let plan1 = create_test_plan();
        let plan2 = create_test_plan();
        
        let result1 = bridge.submit_plan(plan1);
        let result2 = bridge.submit_plan(plan2);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Same plan should produce same submission ID format (deterministic hash)
        let id1 = result1.unwrap();
        let id2 = result2.unwrap();
        assert_eq!(id1.id, id2.id); // Should be identical due to deterministic hashing
    }
}