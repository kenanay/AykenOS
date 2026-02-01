// Constitutional Module: FixApplicationEngine
// This module MUST NOT mutate code without explicit approval and validated plans.
// Atomicity, rollback, and verification are mandatory.

//! Orchestrator for fix application lifecycle (controlled execution).

use crate::arh::application_validation::{ApplicationValidation, ValidationResult};
use crate::arh::rollback_manager::RollbackManager;
use crate::arh::transformation_system::{TransformOutcome, TransformationSystem, TransformationPlan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    Applied,
    RolledBack,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyResult {
    pub outcome: ApplyOutcome,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixPlan {
    pub violation_ids: Vec<String>,
    pub module_id: String,
    pub plans: Vec<TransformationPlan>,
    pub approval: ApprovalArtifact,
    pub is_kernel: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
    Hold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalMode {
    Safe,
    Preview,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalArtifact {
    pub mode: ApprovalMode,
    pub decision: ApprovalDecision,
    pub approver_id: String,
    pub timestamp: String,
    pub proof: String,
    pub kernel_opt_in: bool,
}

pub struct FixApplicationEngine {
    validator: ApplicationValidation,
    transformer: TransformationSystem,
    rollback: RollbackManager,
}

impl FixApplicationEngine {
    pub fn new(validator: ApplicationValidation, transformer: TransformationSystem, rollback: RollbackManager) -> Self {
        Self { validator, transformer, rollback }
    }

    /// Apply a fix plan atomically with rollback on any failure.
    pub fn apply_plan(&self, plan: FixPlan) -> ApplyResult {
        if plan.plans.is_empty() {
            return ApplyResult {
                outcome: ApplyOutcome::Failed,
                message: "No transformation plans supplied".to_string(),
            };
        }

        if plan.approval.decision != ApprovalDecision::Approved {
            return ApplyResult {
                outcome: ApplyOutcome::Failed,
                message: "Fix plan not approved".to_string(),
            };
        }

        if plan.is_kernel && !plan.approval.kernel_opt_in {
            return ApplyResult {
                outcome: ApplyOutcome::Failed,
                message: "Kernel fix requires explicit opt-in approval".to_string(),
            };
        }

        let validation = self.validator.pre_apply(&plan);
        if validation.result != ValidationResult::Ok {
            return ApplyResult {
                outcome: ApplyOutcome::Failed,
                message: format!("Pre-apply validation failed: {}", validation.message),
            };
        }

        let files: Vec<String> = plan.plans.iter().map(|p| p.file.clone()).collect();
        let scope = match self.rollback.begin_scope(&plan.module_id, &files) {
            Ok(scope) => scope,
            Err(err) => {
                return ApplyResult {
                    outcome: ApplyOutcome::Failed,
                    message: format!("Rollback scope failed: {}", err),
                };
            }
        };
        let transform = self.transformer.apply(&plan.plans, &scope);
        if transform != TransformOutcome::Applied {
            if let Err(err) = self.rollback.rollback(scope) {
                return ApplyResult {
                    outcome: ApplyOutcome::Failed,
                    message: format!("Transformation failed; rollback failed: {}", err),
                };
            }
            return ApplyResult {
                outcome: ApplyOutcome::RolledBack,
                message: "Transformation failed; rollback executed".to_string(),
            };
        }

        let post = self.validator.post_apply(&plan);
        if post.result != ValidationResult::Ok {
            if let Err(err) = self.rollback.rollback(scope) {
                return ApplyResult {
                    outcome: ApplyOutcome::Failed,
                    message: format!("Post-apply verification failed; rollback failed: {}", err),
                };
            }
            return ApplyResult {
                outcome: ApplyOutcome::RolledBack,
                message: format!("Post-apply verification failed: {}", post.message),
            };
        }

        if let Err(err) = self.rollback.commit(&scope.snapshot_id) {
            return ApplyResult {
                outcome: ApplyOutcome::Failed,
                message: format!("Commit failed: {}", err),
            };
        }
        ApplyResult {
            outcome: ApplyOutcome::Applied,
            message: "Plan applied successfully".to_string(),
        }
    }
}
