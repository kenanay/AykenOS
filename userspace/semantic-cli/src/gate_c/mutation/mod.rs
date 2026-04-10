//! # Mutation Intent Planning
//!
//! Model mutations as plans without execution.
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use crate::gate_c::{
    error::{GateCResult, MutationError},
    types::{MutationIntent, ResourcePath},
};
use std::collections::BTreeMap;

/// Mutation conflict detector for deterministic conflict analysis
pub struct MutationConflictDetector {
    active_plans: BTreeMap<ResourcePath, Vec<MutationIntent>>,
}

impl MutationConflictDetector {
    /// Create new conflict detector
    pub fn new() -> Self {
        Self {
            active_plans: BTreeMap::new(),
        }
    }

    /// Check for conflicts between mutation intents
    pub fn check_conflict(&self, intent: &MutationIntent) -> GateCResult<()> {
        let target_path = self.get_target_path(intent);

        if let Some(existing_intents) = self.active_plans.get(&target_path) {
            for existing_intent in existing_intents {
                if self.intents_conflict(intent, existing_intent) {
                    return Err(MutationError::MutationConflict(format!(
                        "Conflict detected between {:?} and {:?} on path {}",
                        intent, existing_intent, target_path
                    ))
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Add mutation intent to active plans
    pub fn add_intent(&mut self, intent: MutationIntent) -> GateCResult<()> {
        // Check for conflicts first
        self.check_conflict(&intent)?;

        let target_path = self.get_target_path(&intent);
        self.active_plans
            .entry(target_path)
            .or_insert_with(Vec::new)
            .push(intent);

        Ok(())
    }

    /// Remove intent from active plans
    pub fn remove_intent(&mut self, intent: &MutationIntent) {
        let target_path = self.get_target_path(intent);

        let should_remove_path = if let Some(intents) = self.active_plans.get_mut(&target_path) {
            // Create a copy of the intent to avoid borrowing issues
            let intent_clone = intent.clone();
            intents.retain(|existing| {
                // Use a simple comparison for now - in real implementation this would be more sophisticated
                !std::ptr::eq(existing, &intent_clone)
            });

            intents.is_empty()
        } else {
            false
        };

        if should_remove_path {
            self.active_plans.remove(&target_path);
        }
    }

    /// Get target path from mutation intent
    fn get_target_path(&self, intent: &MutationIntent) -> ResourcePath {
        match intent {
            MutationIntent::InvalidateIntent { target, .. } => target.clone(),
            MutationIntent::UpdateIntent { target, .. } => target.clone(),
            MutationIntent::CreateIntent { path, .. } => path.clone(),
        }
    }

    /// Check if two intents conflict
    fn intents_conflict(&self, intent1: &MutationIntent, intent2: &MutationIntent) -> bool {
        use MutationIntent::*;

        match (intent1, intent2) {
            // Invalidate conflicts with everything
            (InvalidateIntent { .. }, _) | (_, InvalidateIntent { .. }) => true,

            // Update conflicts with update and create
            (UpdateIntent { .. }, UpdateIntent { .. }) => true,
            (UpdateIntent { .. }, CreateIntent { .. }) => true,
            (CreateIntent { .. }, UpdateIntent { .. }) => true,

            // Create conflicts with create
            (CreateIntent { .. }, CreateIntent { .. }) => true,
        }
    }

    /// Check if two intents are equal
    fn intents_equal(&self, intent1: &MutationIntent, intent2: &MutationIntent) -> bool {
        // This is a simplified equality check
        // In a real implementation, this would be more sophisticated
        std::ptr::eq(intent1, intent2)
    }
}

impl Default for MutationConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutation capability validator
pub struct MutationCapabilityValidator {
    capability_checker: CapabilityChecker,
}

impl MutationCapabilityValidator {
    /// Create new capability validator
    pub fn new(capability_checker: CapabilityChecker) -> Self {
        Self { capability_checker }
    }

    /// Validate mutation intent against capabilities
    pub fn validate_intent(&self, intent: &MutationIntent) -> GateCResult<()> {
        match intent {
            MutationIntent::InvalidateIntent { target, .. } => {
                self.capability_checker
                    .check_invalidate_permission(target)
                    .map_err(|e| MutationError::CapabilityDenied(e.to_string()))?;
            }
            MutationIntent::UpdateIntent { target, .. } => {
                self.capability_checker
                    .check_update_permission(target)
                    .map_err(|e| MutationError::CapabilityDenied(e.to_string()))?;
            }
            MutationIntent::CreateIntent { path, .. } => {
                self.capability_checker
                    .check_create_permission(path)
                    .map_err(|e| MutationError::CapabilityDenied(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Perform dry-run validation (no side effects)
    pub fn dry_run_validate(&self, intent: &MutationIntent) -> DryRunResult {
        match self.validate_intent(intent) {
            Ok(()) => DryRunResult::Success,
            Err(e) => DryRunResult::Failure(format!("Validation failed: {}", e)),
        }
    }
}

/// Capability checker for mutation operations
pub struct CapabilityChecker {
    // TODO: Add actual capability checking logic
}

impl CapabilityChecker {
    /// Create new capability checker
    pub fn new() -> Self {
        Self {}
    }

    /// Check invalidate permission
    pub fn check_invalidate_permission(
        &self,
        _target: &ResourcePath,
    ) -> Result<(), CapabilityError> {
        // TODO: Implement actual capability checking
        Ok(())
    }

    /// Check update permission
    pub fn check_update_permission(&self, _target: &ResourcePath) -> Result<(), CapabilityError> {
        // TODO: Implement actual capability checking
        Ok(())
    }

    /// Check create permission
    pub fn check_create_permission(&self, _path: &ResourcePath) -> Result<(), CapabilityError> {
        // TODO: Implement actual capability checking
        Ok(())
    }
}

impl Default for CapabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability error for permission validation
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

/// Result of dry-run validation
#[derive(Debug, Clone, PartialEq)]
pub enum DryRunResult {
    /// Validation succeeded
    Success,
    /// Validation failed
    Failure(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_c::types::{ChangeSet, ContentSpec, InvalidationReason};
    use std::collections::HashMap;

    fn create_test_resource_path() -> ResourcePath {
        ResourcePath {
            segments: vec!["users".to_string(), "123".to_string()],
        }
    }

    fn create_invalidate_intent() -> MutationIntent {
        MutationIntent::InvalidateIntent {
            target: create_test_resource_path(),
            reason: InvalidationReason::Obsolete,
        }
    }

    fn create_update_intent() -> MutationIntent {
        MutationIntent::UpdateIntent {
            target: create_test_resource_path(),
            changes: ChangeSet {
                updates: HashMap::new(),
                removals: vec![],
            },
        }
    }

    fn create_create_intent() -> MutationIntent {
        MutationIntent::CreateIntent {
            path: create_test_resource_path(),
            content: ContentSpec {
                content_type: "application/json".to_string(),
                data: b"{}".to_vec(),
                metadata: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_conflict_detector_creation() {
        let detector = MutationConflictDetector::new();
        assert!(detector.active_plans.is_empty());
    }

    #[test]
    fn test_no_conflict_with_empty_detector() {
        let detector = MutationConflictDetector::new();
        let intent = create_update_intent();

        let result = detector.check_conflict(&intent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalidate_conflicts_with_everything() {
        let mut detector = MutationConflictDetector::new();

        // Add an update intent
        let update_intent = create_update_intent();
        detector.add_intent(update_intent).unwrap();

        // Invalidate should conflict
        let invalidate_intent = create_invalidate_intent();
        let result = detector.check_conflict(&invalidate_intent);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Mutation(MutationError::MutationConflict(_)) => {
                // Expected
            }
            _ => panic!("Expected MutationConflict error"),
        }
    }

    #[test]
    fn test_update_conflicts_with_update() {
        let mut detector = MutationConflictDetector::new();

        // Add first update intent
        let update_intent1 = create_update_intent();
        detector.add_intent(update_intent1).unwrap();

        // Second update should conflict
        let update_intent2 = create_update_intent();
        let result = detector.check_conflict(&update_intent2);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_conflicts_with_create() {
        let mut detector = MutationConflictDetector::new();

        // Add first create intent
        let create_intent1 = create_create_intent();
        detector.add_intent(create_intent1).unwrap();

        // Second create should conflict
        let create_intent2 = create_create_intent();
        let result = detector.check_conflict(&create_intent2);
        assert!(result.is_err());
    }

    #[test]
    fn test_capability_validator_creation() {
        let checker = CapabilityChecker::new();
        let validator = MutationCapabilityValidator::new(checker);

        // Test dry-run validation
        let intent = create_update_intent();
        let result = validator.dry_run_validate(&intent);
        assert_eq!(result, DryRunResult::Success);
    }

    #[test]
    fn test_intent_validation() {
        let checker = CapabilityChecker::new();
        let validator = MutationCapabilityValidator::new(checker);

        let intent = create_update_intent();
        let result = validator.validate_intent(&intent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_conflict_symmetry() {
        let detector = MutationConflictDetector::new();

        let intent1 = create_invalidate_intent();
        let intent2 = create_update_intent();

        let conflict_12 = detector.intents_conflict(&intent1, &intent2);
        let conflict_21 = detector.intents_conflict(&intent2, &intent1);

        // Conflicts should be symmetric
        assert_eq!(conflict_12, conflict_21);
        assert!(conflict_12); // Both should conflict
    }
}
