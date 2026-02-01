// Constitutional Module: Unified Workflow
// This module MUST NOT mutate code or apply fixes.
// It provides a single entry point for detection → recommendation → implementation.

//! Unified refactor workflow (ARRE + ARH, advisory-only).

use crate::arh::arre_arh_integration::{ArreEngine, IntegratedRefactorSystem};
use crate::arh::arh_engine::ViolationInput;
use crate::arh::refactor_guidance::{EnforcementLevel, UnifiedWorkflowResult};

pub struct UnifiedWorkflow<E: ArreEngine> {
    system: IntegratedRefactorSystem<E>,
}

impl<E: ArreEngine> UnifiedWorkflow<E> {
    pub fn new(arre_engine: E) -> Self {
        Self {
            system: IntegratedRefactorSystem::new(arre_engine),
        }
    }

    pub fn run_unified_workflow(&self, violation: ViolationInput) -> UnifiedWorkflowResult {
        let guidance = self.system.integrate(violation);
        let enforcement_level = match guidance.consistency_status {
            crate::arh::refactor_guidance::ConsistencyStatus::Conflict(_) => EnforcementLevel::Block,
            crate::arh::refactor_guidance::ConsistencyStatus::Warning(_) => EnforcementLevel::Warn,
            crate::arh::refactor_guidance::ConsistencyStatus::Aligned => EnforcementLevel::Advisory,
        };

        let next_actions = vec![
            "Review architectural intent".to_string(),
            "Select a tactical hint".to_string(),
            "Use CLI/VS Code preview".to_string(),
        ];

        UnifiedWorkflowResult {
            guidance,
            next_actions,
            enforcement_level,
        }
    }
}
